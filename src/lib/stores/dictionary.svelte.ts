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
  FavoriteItem,
  MasterFeatureSummary,
  RecentViewItem,
  SearchHit,
  Tab
} from '$lib/types/dictionary';

const BUILD_POLL_MS = 80;
const INDEX_DEBOUNCE_MS = 120;
const MAX_RECENT_SEARCHES = 10;
const MAX_RECENT_VIEWS = 20;
const MAX_FAVORITES = 100;
const PREFS_KEY = 'dokhan:user-prefs';

function dedupeRecentViews(rows: RecentViewItem[]): RecentViewItem[] {
  const seen = new Set<string>();
  const out: RecentViewItem[] = [];
  for (const row of rows) {
    if (seen.has(row.key)) continue;
    seen.add(row.key);
    out.push(row);
    if (out.length >= MAX_RECENT_VIEWS) break;
  }
  return out;
}

function toErrorMessage(errorValue: unknown): string {
  return typeof errorValue === 'string' ? errorValue : String(errorValue);
}

export class DictionaryStore {
  loading = $state(false);
  isBooting = $state(false);
  isSearching = $state(false);
  isOpeningDetail = $state(false);
  autoOpenFirstContent = $state(true);
  error = $state('');
  zipPath = $state<string | null>(null);
  activeTab = $state<Tab>('content');
  mobileTab = $state<'home' | 'search' | 'index' | 'favorites'>('home');

  masterSummary = $state<MasterFeatureSummary | null>(null);
  contents = $state<ContentItem[]>([]);
  indexRows = $state<DictionaryIndexEntry[]>([]);
  searchRows = $state<SearchHit[]>([]);
  recentSearches = $state<string[]>([]);
  recentViews = $state<RecentViewItem[]>([]);
  favorites = $state<FavoriteItem[]>([]);

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
  #searchRequestSeq = 0;
  #bootBusyCount = 0;
  #searchBusyCount = 0;
  #detailBusyCount = 0;
  #lastRetryAction: (() => Promise<void>) | null = null;

  constructor() {
    this.#restorePrefs();
  }

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

  setAutoOpenFirstContent(enabled: boolean) {
    this.autoOpenFirstContent = enabled;
  }

  #persistPrefs() {
    if (typeof window === 'undefined') return;
    try {
      const payload = {
        recentSearches: this.recentSearches,
        recentViews: this.recentViews,
        favorites: this.favorites
      };
      window.localStorage.setItem(PREFS_KEY, JSON.stringify(payload));
    } catch {
      // Ignore persistence failures (private mode, quota, etc.)
    }
  }

  #restorePrefs() {
    if (typeof window === 'undefined') return;
    try {
      const raw = window.localStorage.getItem(PREFS_KEY);
      if (!raw) return;
      const parsed = JSON.parse(raw) as {
        recentSearches?: string[];
        recentViews?: RecentViewItem[];
        favorites?: FavoriteItem[];
      };
      this.recentSearches = Array.isArray(parsed.recentSearches)
        ? parsed.recentSearches.slice(0, MAX_RECENT_SEARCHES)
        : [];
      this.recentViews = Array.isArray(parsed.recentViews)
        ? dedupeRecentViews(parsed.recentViews)
        : [];
      this.favorites = Array.isArray(parsed.favorites)
        ? parsed.favorites.slice(0, MAX_FAVORITES)
        : [];
    } catch {
      // Ignore malformed prefs.
    }
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

      if (this.autoOpenFirstContent && this.contents.length) {
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
    this.#pushRecentView({
      key: `content:${page.sourcePath}:${local}`,
      kind: 'content',
      label: page.title,
      id: null,
      local,
      sourcePath: page.sourcePath,
      viewedAt: Date.now()
    });
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
    this.#pushRecentView({
      key: `entry:${id}`,
      kind: 'entry',
      label: entry.headword,
      id,
      local: null,
      sourcePath: entry.sourcePath,
      viewedAt: Date.now()
    });
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

  handleSearchQueryChange(value: string) {
    this.searchQuery = value;
    if (!value.trim()) {
      this.searchRows = [];
      this.committedSearchQuery = '';
    }
  }

  async doSearch(event: Event) {
    event.preventDefault();
    await this.#runSearch(this.searchQuery, true);
  }

  async #runSearch(rawQuery: string, recordRecent: boolean) {
    const searchTerm = rawQuery.trim();
    if (!searchTerm) {
      this.searchRows = [];
      this.committedSearchQuery = '';
      return;
    }
    this.committedSearchQuery = searchTerm;
    const reqSeq = ++this.#searchRequestSeq;
    this.#setRetryAction(async () => {
      this.searchQuery = searchTerm;
      this.committedSearchQuery = searchTerm;
      const retryRows = await this.#withBusy('search', () =>
        searchEntries(this.zipPath, searchTerm, 200)
      );
      if (retryRows && reqSeq === this.#searchRequestSeq) {
        this.searchRows = retryRows;
      }
    });
    const rows = await this.#withBusy('search', () => searchEntries(this.zipPath, searchTerm, 200));
    if (rows && reqSeq === this.#searchRequestSeq) {
      this.searchRows = rows;
      if (recordRecent) {
        this.#pushRecentSearch(searchTerm);
      }
    }
  }

  useRecentSearch(query: string) {
    this.searchQuery = query;
    void this.#runSearch(query, false);
  }

  #pushRecentSearch(query: string) {
    const next = [query, ...this.recentSearches.filter((item) => item !== query)];
    this.recentSearches = next.slice(0, MAX_RECENT_SEARCHES);
    this.#persistPrefs();
  }

  #pushRecentView(item: RecentViewItem) {
    const next = [item, ...this.recentViews.filter((row) => row.key !== item.key)];
    this.recentViews = dedupeRecentViews(next);
    this.#persistPrefs();
  }

  openRecentView(item: RecentViewItem) {
    if (item.kind === 'entry' && item.id != null) {
      void this.openEntry(item.id);
      return;
    }
    if (item.kind === 'content' && item.local) {
      void this.openContent(item.local, item.sourcePath);
    }
  }

  isFavoriteEntry(id: number): boolean {
    return this.favorites.some((item) => item.kind === 'entry' && item.id === id);
  }

  isFavoriteContent(local: string, sourcePath: string | null): boolean {
    const key = `content:${sourcePath ?? ''}:${local}`;
    return this.favorites.some((item) => item.key === key);
  }

  toggleFavoriteEntry(entry: Pick<EntryDetail, 'id' | 'headword' | 'sourcePath'>) {
    const key = `entry:${entry.id}`;
    if (this.favorites.some((item) => item.key === key)) {
      this.favorites = this.favorites.filter((item) => item.key !== key);
      this.#persistPrefs();
      return;
    }
    const nextItem: FavoriteItem = {
      key,
      kind: 'entry',
      label: entry.headword,
      id: entry.id,
      local: null,
      sourcePath: entry.sourcePath
    };
    this.favorites = [
      nextItem,
      ...this.favorites
    ].slice(0, MAX_FAVORITES);
    this.#persistPrefs();
  }

  toggleFavoriteContent(content: Pick<ContentPage, 'local' | 'title' | 'sourcePath'>) {
    const key = `content:${content.sourcePath ?? ''}:${content.local}`;
    if (this.favorites.some((item) => item.key === key)) {
      this.favorites = this.favorites.filter((item) => item.key !== key);
      this.#persistPrefs();
      return;
    }
    const nextItem: FavoriteItem = {
      key,
      kind: 'content',
      label: content.title,
      id: null,
      local: content.local,
      sourcePath: content.sourcePath
    };
    this.favorites = [
      nextItem,
      ...this.favorites
    ].slice(0, MAX_FAVORITES);
    this.#persistPrefs();
  }

  toggleCurrentFavorite() {
    if (this.detailMode === 'entry' && this.selectedEntry) {
      this.toggleFavoriteEntry(this.selectedEntry);
      return;
    }
    if (this.detailMode === 'content' && this.selectedContent) {
      this.toggleFavoriteContent(this.selectedContent);
    }
  }

  isCurrentFavorite(): boolean {
    if (this.detailMode === 'entry' && this.selectedEntry) {
      return this.isFavoriteEntry(this.selectedEntry.id);
    }
    if (this.detailMode === 'content' && this.selectedContent) {
      return this.isFavoriteContent(this.selectedContent.local, this.selectedContent.sourcePath);
    }
    return false;
  }

  removeFavorite(key: string) {
    this.favorites = this.favorites.filter((item) => item.key !== key);
    this.#persistPrefs();
  }

  openFavorite(item: FavoriteItem) {
    if (item.kind === 'entry' && item.id != null) {
      void this.openEntry(item.id);
      return;
    }
    if (item.kind === 'content' && item.local) {
      void this.openContent(item.local, item.sourcePath);
    }
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
