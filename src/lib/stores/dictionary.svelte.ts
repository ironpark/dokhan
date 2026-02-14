import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { open as openFile, writeFile } from '@tauri-apps/plugin-fs';
import { appCacheDir, join } from '@tauri-apps/api/path';
import {
  analyzeZipDataset,
  getContentPage,
  getEntryDetail,
  getIndexEntries,
  getMasterBuildStatus,
  getMasterContents,
  resolveLinkTarget,
  resolveMediaDataUrl,
  searchEntries,
  startMasterBuild
} from '$lib/api/dictionary';
import type {
  BuildProgress,
  ContentItem,
  ContentPage,
  DetailMode,
  DictionaryIndexEntry,
  DictionaryLinkTarget,
  EntryDetail,
  MasterFeatureSummary,
  SearchHit,
  Tab
} from '$lib/types/dictionary';

const BUILD_POLL_MS = 80;
const INDEX_DEBOUNCE_MS = 120;

function toErrorMessage(errorValue: unknown): string {
  return typeof errorValue === 'string' ? errorValue : String(errorValue);
}

export class DictionaryStore {
  loading = $state(false);
  error = $state('');
  zipPath = $state('asset/dictionary_v77.zip');
  activeTab = $state<Tab>('content');

  masterSummary = $state<MasterFeatureSummary | null>(null);
  contents = $state<ContentItem[]>([]);
  indexRows = $state<DictionaryIndexEntry[]>([]);
  searchRows = $state<SearchHit[]>([]);

  indexPrefix = $state('');
  searchQuery = $state('');
  committedSearchQuery = $state('');
  indexLoading = $state(false);

  selectedContent = $state<ContentPage | null>(null);
  selectedEntry = $state<EntryDetail | null>(null);
  detailMode = $state<DetailMode>('none');
  selectedContentLocal = $state('');
  selectedEntryId = $state<number | null>(null);

  progress = $state<BuildProgress | null>(null);
  showProgress = $state(false);
  dragOver = $state(false);

  #indexDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  #indexRequestSeq = 0;
  #detailRequestSeq = 0;

  dispose = () => {
    if (this.#indexDebounceTimer) {
      clearTimeout(this.#indexDebounceTimer);
      this.#indexDebounceTimer = null;
    }
  };

  tryAutoBootDefaultZip = async () => {
    if (this.masterSummary || this.loading) return;
    try {
      await analyzeZipDataset(this.zipPath);
      await this.bootMasterFeatures();
    } catch {
      // Keep drop-zone UI when default zip is not available.
    }
  };

  #withLoading = async <T>(task: () => Promise<T>): Promise<T | undefined> => {
    this.loading = true;
    this.error = '';
    try {
      return await task();
    } catch (e) {
      this.error = toErrorMessage(e);
      return undefined;
    } finally {
      this.loading = false;
    }
  };

  #clearSelection = () => {
    this.selectedEntry = null;
    this.selectedEntryId = null;
    this.selectedContent = null;
    this.selectedContentLocal = '';
    this.detailMode = 'none';
  };

  bootMasterFeatures = async () => {
    this.loading = true;
    this.error = '';
    this.showProgress = true;
    this.progress = { phase: 'start', current: 0, total: 1, message: '초기화 중' };
    try {
      await startMasterBuild(this.zipPath);
      while (true) {
        const status = await getMasterBuildStatus(this.zipPath);
        this.progress = {
          phase: status.phase,
          current: status.current,
          total: status.total,
          message: status.message
        };

        if (status.done) {
          if (!status.success) {
            throw new Error(status.error ?? '빌드 실패');
          }
          this.masterSummary = status.summary;
          break;
        }
        await new Promise((resolve) => setTimeout(resolve, BUILD_POLL_MS));
      }

      const [nextContents, nextIndex] = await Promise.all([
        getMasterContents(this.zipPath),
        getIndexEntries(this.zipPath, '', null)
      ]);

      this.contents = nextContents;
      this.indexRows = nextIndex;
      this.searchRows = [];
      this.#clearSelection();

      if (this.contents.length) {
        await this.openContent(this.contents[0].local);
      }
    } catch (e) {
      this.error = toErrorMessage(e);
    } finally {
      this.showProgress = false;
      this.loading = false;
    }
  };

  useZipPath = async (path: string) => {
    const nextPath = path.trim();
    if (!nextPath) {
      this.error = 'ZIP 경로가 비어 있습니다.';
      return;
    }
    this.zipPath = nextPath;
    const ok = await this.#withLoading(() => analyzeZipDataset(nextPath));
    if (!ok) return;
    await this.bootMasterFeatures();
  };

  pickZipFile = async () => {
    try {
      const selected = await openDialog({
        multiple: false,
        directory: false,
        pickerMode: 'document',
        fileAccessMode: 'copy',
        filters: [{ name: 'ZIP', extensions: ['zip'] }]
      });
      if (!selected || Array.isArray(selected)) return;
      const resolvedPath = await this.#resolvePickedZipPath(selected);
      await this.useZipPath(resolvedPath);
    } catch (e) {
      this.error = `파일 선택 실패: ${toErrorMessage(e)}`;
    }
  };

  #resolvePickedZipPath = async (selected: string): Promise<string> => {
    const raw = selected.trim();
    if (raw.startsWith('content://')) {
      const persisted = await this.#createTempZipPath();
      await this.#copyContentUriToPath(raw, persisted);
      return persisted;
    }
    if (raw.startsWith('file://')) {
      try {
        return decodeURIComponent(new URL(raw).pathname);
      } catch {
        return raw;
      }
    }
    return raw;
  };

  #createTempZipPath = async (): Promise<string> => {
    const cache = await appCacheDir();
    const suffix = Math.random().toString(36).slice(2, 8);
    const filename = `picked-${Date.now()}-${suffix}.zip`;
    return join(cache, filename);
  };

  #copyContentUriToPath = async (sourceUri: string, destinationPath: string): Promise<void> => {
    const src = await openFile(sourceUri, { read: true });
    const dst = await openFile(destinationPath, { write: true, create: true, truncate: true });
    const buffer = new Uint8Array(256 * 1024);
    try {
      while (true) {
        const read = await src.read(buffer);
        if (read === null || read === 0) break;
        await dst.write(buffer.subarray(0, read));
      }
    } finally {
      await Promise.allSettled([src.close(), dst.close()]);
    }
  };

  openContent = async (local: string, sourcePath: string | null = null) => {
    const reqSeq = ++this.#detailRequestSeq;
    const page = await this.#withLoading(() => getContentPage(this.zipPath, local, sourcePath));
    if (!page || reqSeq !== this.#detailRequestSeq) return;
    this.selectedContent = page;
    this.selectedContentLocal = local;
    this.selectedEntry = null;
    this.selectedEntryId = null;
    this.detailMode = 'content';
  };

  openEntry = async (id: number) => {
    const reqSeq = ++this.#detailRequestSeq;
    this.selectedEntryId = id;
    this.selectedContentLocal = '';
    const entry = await this.#withLoading(() => getEntryDetail(this.zipPath, id));
    if (!entry || reqSeq !== this.#detailRequestSeq) return;
    this.selectedEntry = entry;
    this.selectedContent = null;
    this.detailMode = 'entry';
  };

  #loadIndexByPrefix = async (prefix: string) => {
    if (!this.masterSummary) return;
    const trimmed = prefix.trim();
    const requestSeq = ++this.#indexRequestSeq;
    this.indexLoading = true;
    try {
      const rows = await getIndexEntries(this.zipPath, trimmed, trimmed ? 500 : null);
      if (requestSeq === this.#indexRequestSeq && this.indexPrefix.trim() === trimmed) {
        this.indexRows = rows;
      }
    } catch (e) {
      this.error = toErrorMessage(e);
    } finally {
      if (requestSeq === this.#indexRequestSeq) this.indexLoading = false;
    }
  };

  handleIndexQueryChange = (value: string) => {
    this.indexPrefix = value;
    if (this.#indexDebounceTimer) clearTimeout(this.#indexDebounceTimer);
    this.#indexDebounceTimer = setTimeout(() => {
      void this.#loadIndexByPrefix(value);
    }, INDEX_DEBOUNCE_MS);
  };

  doSearch = async (event: Event) => {
    event.preventDefault();
    if (!this.searchQuery.trim()) {
      this.searchRows = [];
      this.committedSearchQuery = '';
      return;
    }
    this.committedSearchQuery = this.searchQuery.trim();
    const rows = await this.#withLoading(() => searchEntries(this.zipPath, this.searchQuery, 200));
    if (rows) this.searchRows = rows;
  };

  openInlineHref = async (
    href: string,
    currentSourcePath: string | null,
    currentLocal: string | null
  ) => {
    const target = await this.#withLoading(() =>
      resolveLinkTarget(this.zipPath, href, currentSourcePath, currentLocal)
    );
    if (!target) return;
    if (target.kind === 'content') {
      await this.openContent(target.local, target.sourcePath);
      return;
    }
    await this.openEntry(target.id);
  };

  resolveInlineImageHref = async (
    href: string,
    currentSourcePath: string | null,
    currentLocal: string | null
  ): Promise<string | null> => {
    try {
      return await resolveMediaDataUrl(this.zipPath, href, currentSourcePath, currentLocal);
    } catch {
      return null;
    }
  };
}
