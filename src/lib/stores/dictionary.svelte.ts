import { open as openDialog } from '@tauri-apps/plugin-dialog';
import {
  getContentPage,
  getEntryDetail,
  getIndexEntries,
  getMasterBuildStatus,
  getMasterContents,
  prepareZipSource,
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
  isBooting = $state(false);
  isSearching = $state(false);
  isOpeningDetail = $state(false);
  error = $state('');
  zipPath = $state<string | null>(null);
  activeTab = $state<Tab>('content');
  mobileTab = $state<'home' | 'search' | 'index'>('home');

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
  #bootBusyCount = 0;
  #searchBusyCount = 0;
  #detailBusyCount = 0;
  #lastRetryAction: (() => Promise<void>) | null = null;

  dispose() {
    if (this.#indexDebounceTimer) {
      clearTimeout(this.#indexDebounceTimer);
      this.#indexDebounceTimer = null;
    }
  }

  #syncLoadingState() {
    this.isBooting = this.#bootBusyCount > 0;
    this.isSearching = this.#searchBusyCount > 0;
    this.isOpeningDetail = this.#detailBusyCount > 0;
    this.loading = this.isBooting || this.isSearching || this.isOpeningDetail;
  }

  #beginBusy(kind: 'boot' | 'search' | 'detail') {
    if (kind === 'boot') this.#bootBusyCount += 1;
    if (kind === 'search') this.#searchBusyCount += 1;
    if (kind === 'detail') this.#detailBusyCount += 1;
    this.#syncLoadingState();
  }

  #endBusy(kind: 'boot' | 'search' | 'detail') {
    if (kind === 'boot') this.#bootBusyCount = Math.max(0, this.#bootBusyCount - 1);
    if (kind === 'search') this.#searchBusyCount = Math.max(0, this.#searchBusyCount - 1);
    if (kind === 'detail') this.#detailBusyCount = Math.max(0, this.#detailBusyCount - 1);
    this.#syncLoadingState();
  }

  #setRetryAction(action: (() => Promise<void>) | null) {
    this.#lastRetryAction = action;
  }

  async retryLastOperation() {
    if (this.#lastRetryAction) {
      await this.#lastRetryAction();
      return;
    }
    if (this.zipPath) {
      await this.useZipPath(this.zipPath);
      return;
    }
    await this.bootFromManagedCache();
  }

  async #withBusy<T>(kind: 'search' | 'detail', task: () => Promise<T>): Promise<T | undefined> {
    this.#beginBusy(kind);
    this.error = '';
    try {
      return await task();
    } catch (e) {
      this.error = toErrorMessage(e);
      return undefined;
    } finally {
      this.#endBusy(kind);
    }
  }

  #clearSelection() {
    this.selectedEntry = null;
    this.selectedEntryId = null;
    this.selectedContent = null;
    this.selectedContentLocal = '';
    this.detailMode = 'none';
  }

  closeDetail() {
    this.#clearSelection();
  }

  handleMobileBackNavigation(): boolean {
    if (this.selectedEntryId || this.selectedContentLocal) {
      this.closeDetail();
      return true;
    }
    if (this.mobileTab !== 'home') {
      this.mobileTab = 'home';
      return true;
    }
    return false;
  }

  async bootMasterFeatures() {
    await this.#bootMasterFeaturesWithPath(this.zipPath);
  }

  async bootFromManagedCache() {
    await this.#bootMasterFeaturesWithPath(null, true);
  }

  async #bootMasterFeaturesWithPath(zipPath: string | null, silentNoCache = false) {
    this.#setRetryAction(async () => {
      await this.#bootMasterFeaturesWithPath(zipPath, false);
    });
    this.#beginBusy('boot');
    this.error = '';
    this.showProgress = true;
    this.progress = { phase: 'start', current: 0, total: 1, message: '초기화 중' };
    try {
      await startMasterBuild(zipPath);
      while (true) {
        const status = await getMasterBuildStatus(zipPath);
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
          this.zipPath = status.summary?.zipPath ?? zipPath;
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
      const message = toErrorMessage(e);
      if (silentNoCache && message.includes('no managed zip cache found')) {
        this.error = '';
      } else {
        this.error = message;
      }
    } finally {
      this.showProgress = false;
      this.#endBusy('boot');
    }
  }

  async useZipPath(path: string) {
    const nextPath = path.trim();
    if (!nextPath) {
      this.error = 'ZIP 경로가 비어 있습니다.';
      return;
    }
    this.zipPath = nextPath;
    this.#setRetryAction(async () => {
      await this.useZipPath(nextPath);
    });
    await this.#bootMasterFeaturesWithPath(nextPath);
  }

  async pickZipFile() {
    try {
      const selected = await openDialog({
        multiple: false,
        directory: false,
        pickerMode: 'document',
        fileAccessMode: 'copy',
        filters: [{ name: 'ZIP', extensions: ['zip'] }]
      });
      if (!selected || Array.isArray(selected)) return;
      this.#beginSourcePrepare();
      const resolvedPath = await this.#resolvePickedZipPath(selected);
      await this.useZipPath(resolvedPath);
    } catch (e) {
      this.error = `파일 선택 실패: ${toErrorMessage(e)}`;
    } finally {
      this.#endSourcePrepare();
    }
  }

  async #resolvePickedZipPath(selected: string): Promise<string> {
    return prepareZipSource(selected);
  }

  #beginSourcePrepare() {
    this.error = '';
    this.showProgress = true;
    this.progress = {
      phase: 'source-prepare',
      current: 0,
      total: 1,
      message: 'ZIP 파일 준비 중'
    };
  }

  #endSourcePrepare() {
    if (this.progress?.phase === 'source-prepare') {
      this.showProgress = false;
      this.progress = null;
    }
  }

  async openContent(local: string, sourcePath: string | null = null) {
    this.#setRetryAction(async () => {
      await this.openContent(local, sourcePath);
    });
    const reqSeq = ++this.#detailRequestSeq;
    const page = await this.#withBusy('detail', () => getContentPage(this.zipPath, local, sourcePath));
    if (!page || reqSeq !== this.#detailRequestSeq) return;
    this.selectedContent = page;
    this.selectedContentLocal = local;
    this.selectedEntry = null;
    this.selectedEntryId = null;
    this.detailMode = 'content';
  }

  async openEntry(id: number) {
    this.#setRetryAction(async () => {
      await this.openEntry(id);
    });
    const reqSeq = ++this.#detailRequestSeq;
    this.selectedEntryId = id;
    this.selectedContentLocal = '';
    const entry = await this.#withBusy('detail', () => getEntryDetail(this.zipPath, id));
    if (!entry || reqSeq !== this.#detailRequestSeq) return;
    this.selectedEntry = entry;
    this.selectedContent = null;
    this.detailMode = 'entry';
  }

  async #loadIndexByPrefix(prefix: string) {
    if (!this.masterSummary) return;
    const trimmed = prefix.trim();
    this.#setRetryAction(async () => {
      await this.#loadIndexByPrefix(trimmed);
    });
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
  }

  handleIndexQueryChange(value: string) {
    this.indexPrefix = value;
    if (this.#indexDebounceTimer) clearTimeout(this.#indexDebounceTimer);
    this.#indexDebounceTimer = setTimeout(() => {
      void this.#loadIndexByPrefix(value);
    }, INDEX_DEBOUNCE_MS);
  }

  async doSearch(event: Event) {
    event.preventDefault();
    if (!this.searchQuery.trim()) {
      this.searchRows = [];
      this.committedSearchQuery = '';
      return;
    }
    this.committedSearchQuery = this.searchQuery.trim();
    const searchTerm = this.searchQuery;
    this.#setRetryAction(async () => {
      this.searchQuery = searchTerm;
      this.committedSearchQuery = searchTerm.trim();
      const retryRows = await this.#withBusy('search', () =>
        searchEntries(this.zipPath, searchTerm, 200)
      );
      if (retryRows) this.searchRows = retryRows;
    });
    const rows = await this.#withBusy('search', () => searchEntries(this.zipPath, searchTerm, 200));
    if (rows) this.searchRows = rows;
  }

  async openInlineHref(
    href: string,
    currentSourcePath: string | null,
    currentLocal: string | null
  ) {
    this.#setRetryAction(async () => {
      await this.openInlineHref(href, currentSourcePath, currentLocal);
    });
    const target = await this.#withBusy('detail', () =>
      resolveLinkTarget(this.zipPath, href, currentSourcePath, currentLocal)
    );
    if (!target) return;
    if (target.kind === 'content') {
      await this.openContent(target.local, target.sourcePath);
      return;
    }
    await this.openEntry(target.id);
  }

  async resolveInlineImageHref(
    href: string,
    currentSourcePath: string | null,
    currentLocal: string | null
  ): Promise<string | null> {
    try {
      return await resolveMediaDataUrl(this.zipPath, href, currentSourcePath, currentLocal);
    } catch {
      return null;
    }
  }
}
