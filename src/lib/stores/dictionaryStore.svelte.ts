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
import {
  loadDictionaryPrefs,
  saveDictionaryPrefs
} from '$lib/stores/dictionaryPrefsStore';
import { createLibraryState, type LibraryState } from '$lib/stores/libraryState.svelte';
import { createReaderPrefsState, type ReaderPrefsState } from '$lib/stores/readerPrefsState.svelte';
import { createSearchIndexState, type SearchIndexState } from '$lib/stores/searchIndexState.svelte';
import { createDetailState, type DetailState } from '$lib/stores/detailState.svelte';
import type {
  BookmarkFolder,
  BuildProgress,
  ContentItem,
  ContentPage,
  DictionaryIndexEntry,
  DetailMode,
  EntryDetail,
  FavoriteItem,
  MasterFeatureSummary,
  ReaderFontSize,
  ReaderLineHeight,
  ReaderWidth,
  RecentViewItem,
  SearchHit,
  Tab
} from '$lib/types/dictionary';

const BUILD_POLL_MS = 80;
const INDEX_DEBOUNCE_MS = 120;
export const INDEX_PAGE_LIMIT = 500;
export const SEARCH_RESULT_LIMIT = 200;

function toErrorMessage(errorValue: unknown): string {
  return typeof errorValue === 'string' ? errorValue : String(errorValue);
}

function zipDisplayName(path: string | null): string {
  if (!path) return '';
  const filename = path.split(/[\\/]/).pop() ?? path;
  return filename.replace(/(-[0-9a-f]{16})?\.zip$/i, '');
}

export interface DictionaryStore {
  readonly loading: boolean;
  readonly isBooting: boolean;
  readonly isSearching: boolean;
  readonly isOpeningDetail: boolean;
  readonly autoOpenFirstContent: boolean;
  readonly error: string;
  readonly zipPath: string | null;
  readonly activeZipName: string;
  readonly activeTab: Tab;
  readonly mobileTab: 'home' | 'search' | 'index' | 'favorites';
  readonly masterSummary: MasterFeatureSummary | null;
  readonly contents: ContentItem[];
  readonly progress: BuildProgress | null;
  readonly showProgress: boolean;
  readonly dragOver: boolean;
  readonly recentSearches: string[];
  readonly recentViews: RecentViewItem[];
  readonly favorites: FavoriteItem[];
  readonly allFavorites: FavoriteItem[];
  readonly legacyFavoriteCount: number;
  readonly bookmarkFolders: BookmarkFolder[];
  readonly activeBookmarkFolderId: string;
  readonly preprocessEnabled: boolean;
  readonly markerPreprocessEnabled: boolean;
  readonly readerFontSize: ReaderFontSize;
  readonly readerLineHeight: ReaderLineHeight;
  readonly readerWidth: ReaderWidth;
  readonly indexPrefix: string;
  readonly indexRows: DictionaryIndexEntry[];
  readonly indexLoading: boolean;
  readonly indexHasMore: boolean;
  readonly indexLoadingMore: boolean;
  readonly searchQuery: string;
  readonly committedSearchQuery: string;
  readonly searchRows: SearchHit[];
  readonly selectedContent: ContentPage | null;
  readonly selectedEntry: EntryDetail | null;
  readonly detailMode: DetailMode;
  readonly selectedContentLocal: string;
  readonly selectedEntryId: number | null;

  dispose(): void;
  clearError(): void;
  retryLastOperation(): Promise<void>;
  closeDetail(): void;
  handleMobileBackNavigation(): boolean;
  bootMasterFeatures(): Promise<void>;
  setAutoOpenFirstContent(enabled: boolean): void;
  setActiveTab(tab: Tab): void;
  setMobileTab(tab: 'home' | 'search' | 'index' | 'favorites'): void;
  setDragOver(value: boolean): void;
  setPreprocessEnabled(enabled: boolean): void;
  setMarkerPreprocessEnabled(enabled: boolean): void;
  setReaderFontSize(value: ReaderFontSize): void;
  setReaderLineHeight(value: ReaderLineHeight): void;
  setReaderWidth(value: ReaderWidth): void;
  bootFromManagedCache(): Promise<void>;
  useZipPath(path: string): Promise<void>;
  pickZipFile(): Promise<void>;
  openContent(local: string, sourcePath?: string | null): Promise<void>;
  openEntry(id: number): Promise<void>;
  setIndexPrefix(value: string): void;
  loadMoreIndex(): Promise<void>;
  setSearchQuery(value: string): void;
  submitSearch(): Promise<void>;
  useRecentSearch(query: string): void;
  openRecentView(item: RecentViewItem): void;
  isFavoriteEntry(id: number): boolean;
  isFavoriteContent(local: string, sourcePath: string | null): boolean;
  toggleFavoriteEntry(entry: Pick<EntryDetail, 'id' | 'headword' | 'sourcePath'>): void;
  toggleFavoriteContent(content: Pick<ContentPage, 'local' | 'title' | 'sourcePath'>): void;
  toggleCurrentFavorite(): void;
  isCurrentFavorite(): boolean;
  /** Folder holding the open entry/page's bookmark, or null when it isn't bookmarked. */
  currentFavoriteFolderId(): string | null;
  removeFavorite(key: string): void;
  setActiveBookmarkFolder(folderId: string): void;
  createBookmarkFolder(name: string): string | null;
  renameBookmarkFolder(folderId: string, name: string): void;
  deleteBookmarkFolder(folderId: string): void;
  moveFavoriteToFolder(key: string, folderId: string): void;
  addCurrentFavoriteToFolder(folderId: string): void;
  openFavorite(item: FavoriteItem): void;
  openInlineHref(href: string, currentSourcePath: string | null, currentLocal: string | null): Promise<void>;
  resolveInlineImageHref(href: string, currentSourcePath: string | null, currentLocal: string | null): Promise<string | null>;
}

export function createDictionaryStore(): DictionaryStore {
  let loading = $state(false);
  let isBooting = $state(false);
  let isSearching = $state(false);
  let isOpeningDetail = $state(false);
  let autoOpenFirstContent = $state(true);
  let error = $state('');
  let zipPath = $state<string | null>(null);
  let activeTab = $state<Tab>('content');
  let mobileTab = $state<'home' | 'search' | 'index' | 'favorites'>('home');

  let masterSummary = $state<MasterFeatureSummary | null>(null);
  let contents = $state<ContentItem[]>([]);

  let progress = $state<BuildProgress | null>(null);
  let showProgress = $state(false);
  let dragOver = $state(false);

  let indexDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  let indexRequestSeq = 0;
  let indexRowsPrefix = '';
  let detailRequestSeq = 0;
  let searchRequestSeq = 0;
  let bootRequestSeq = 0;
  let bootBusyCount = 0;
  let searchBusyCount = 0;
  let detailBusyCount = 0;
  let lastRetryAction: (() => Promise<void>) | null = null;

  const libraryState: LibraryState = createLibraryState(() => persistPrefs());
  const readerPrefsState: ReaderPrefsState = createReaderPrefsState(() => persistPrefs());
  const searchIndexState: SearchIndexState = createSearchIndexState();
  const detailState: DetailState = createDetailState();

  const prefs = loadDictionaryPrefs();
  libraryState.applySnapshot({
    recentSearches: prefs.recentSearches,
    recentViews: prefs.recentViews,
    favorites: prefs.favorites,
    bookmarkFolders: prefs.bookmarkFolders,
    activeBookmarkFolderId: prefs.activeBookmarkFolderId
  });
  readerPrefsState.applySnapshot({
    preprocessEnabled: prefs.preprocessEnabled,
    markerPreprocessEnabled: prefs.markerPreprocessEnabled,
    readerFontSize: prefs.readerFontSize,
    readerLineHeight: prefs.readerLineHeight,
    readerWidth: prefs.readerWidth
  });

  function syncLoadingState() {
    isBooting = bootBusyCount > 0;
    isSearching = searchBusyCount > 0;
    isOpeningDetail = detailBusyCount > 0;
    loading = isBooting || isSearching || isOpeningDetail;
  }

  function beginBusy(kind: 'boot' | 'search' | 'detail') {
    if (kind === 'boot') bootBusyCount += 1;
    if (kind === 'search') searchBusyCount += 1;
    if (kind === 'detail') detailBusyCount += 1;
    syncLoadingState();
  }

  function endBusy(kind: 'boot' | 'search' | 'detail') {
    if (kind === 'boot') bootBusyCount = Math.max(0, bootBusyCount - 1);
    if (kind === 'search') searchBusyCount = Math.max(0, searchBusyCount - 1);
    if (kind === 'detail') detailBusyCount = Math.max(0, detailBusyCount - 1);
    syncLoadingState();
  }

  function setRetryAction(action: (() => Promise<void>) | null) {
    lastRetryAction = action;
  }

  function persistPrefs() {
    saveDictionaryPrefs({
      ...libraryState.toSnapshot(),
      ...readerPrefsState.toSnapshot()
    });
  }

  function clearSelection() {
    detailState.clearSelection();
  }

  function startRequest(kind: 'search' | 'detail'): number {
    if (kind === 'search') {
      invalidateSearchRequests();
      return searchRequestSeq;
    }
    invalidateDetailRequests();
    return detailRequestSeq;
  }

  function isCurrentRequest(kind: 'search' | 'detail', requestId: number): boolean {
    return requestId === (kind === 'search' ? searchRequestSeq : detailRequestSeq);
  }

  async function withBusy<T>(
    kind: 'search' | 'detail',
    requestId: number,
    task: () => Promise<T>
  ): Promise<T | undefined> {
    beginBusy(kind);
    error = '';
    try {
      return await task();
    } catch (e) {
      if (isCurrentRequest(kind, requestId)) error = toErrorMessage(e);
      return undefined;
    } finally {
      // Invalidation already reset the busy count for superseded requests.
      if (isCurrentRequest(kind, requestId)) endBusy(kind);
    }
  }

  function beginSourcePrepare() {
    error = '';
    showProgress = true;
    progress = {
      phase: 'source-prepare',
      current: 0,
      total: 1,
      message: 'ZIP 파일 준비 중'
    };
  }

  function endSourcePrepare() {
    if (progress?.phase === 'source-prepare') {
      showProgress = false;
      progress = null;
    }
  }

  async function resolvePickedZipPath(selected: string): Promise<string> {
    return prepareZipSource(selected);
  }

  function pushRecentSearch(query: string) {
    libraryState.pushRecentSearch(query);
  }

  function pushRecentView(item: RecentViewItem) {
    libraryState.pushRecentView(item);
  }

  function invalidateSourceRequests() {
    indexRequestSeq += 1;
    invalidateSearchRequests();
    invalidateDetailRequests();
    if (indexDebounceTimer) {
      clearTimeout(indexDebounceTimer);
      indexDebounceTimer = null;
    }
    searchIndexState.setIndexLoading(false);
    searchIndexState.setIndexLoadingMore(false);
  }

  function invalidateSearchRequests() {
    searchRequestSeq += 1;
    searchBusyCount = 0;
    syncLoadingState();
  }

  function invalidateDetailRequests() {
    detailRequestSeq += 1;
    detailBusyCount = 0;
    syncLoadingState();
  }

  async function loadIndexByPrefix(prefix: string) {
    if (!masterSummary) return;
    const trimmed = prefix.trim();
    setRetryAction(async () => {
      await loadIndexByPrefix(trimmed);
    });
    const requestId = ++indexRequestSeq;
    const activeZipPath = zipPath;
    searchIndexState.setIndexLoading(true);
    searchIndexState.setIndexLoadingMore(false);
    try {
      const rows = await getIndexEntries(activeZipPath, trimmed, INDEX_PAGE_LIMIT);
      if (requestId === indexRequestSeq && searchIndexState.indexPrefix.trim() === trimmed) {
        indexRowsPrefix = trimmed;
        searchIndexState.setIndexRows(rows, rows.length === INDEX_PAGE_LIMIT);
      }
    } catch (e) {
      if (requestId === indexRequestSeq) error = toErrorMessage(e);
    } finally {
      if (requestId === indexRequestSeq) searchIndexState.setIndexLoading(false);
    }
  }

  async function loadMoreIndex() {
    if (
      !masterSummary ||
      !searchIndexState.indexHasMore ||
      searchIndexState.indexLoading ||
      searchIndexState.indexLoadingMore
    ) {
      return;
    }
    // A new prefix or source bumps the sequence, which discards this page.
    const requestId = indexRequestSeq;
    const activeZipPath = zipPath;
    const prefix = indexRowsPrefix;
    // The typed prefix is ahead of the rows while its debounce is pending.
    if (searchIndexState.indexPrefix.trim() !== prefix) return;
    const offset = searchIndexState.indexRows.length;
    searchIndexState.setIndexLoadingMore(true);
    try {
      const rows = await getIndexEntries(activeZipPath, prefix, INDEX_PAGE_LIMIT, offset);
      if (
        requestId === indexRequestSeq &&
        searchIndexState.indexPrefix.trim() === prefix &&
        searchIndexState.indexRows.length === offset
      ) {
        searchIndexState.appendIndexRows(rows, rows.length === INDEX_PAGE_LIMIT);
      }
    } catch (e) {
      if (requestId === indexRequestSeq) error = toErrorMessage(e);
    } finally {
      if (requestId === indexRequestSeq) searchIndexState.setIndexLoadingMore(false);
    }
  }

  async function runSearch(rawQuery: string, recordRecent: boolean) {
    const searchTerm = rawQuery.trim();
    const requestId = startRequest('search');
    if (!searchTerm) {
      searchIndexState.clearSearch();
      return;
    }
    searchIndexState.setCommittedSearchQuery(searchTerm);
    searchIndexState.setSearchRows([]);
    const activeZipPath = zipPath;
    setRetryAction(async () => {
      searchIndexState.setSearchQuery(searchTerm);
      await runSearch(searchTerm, false);
    });
    const rows = await withBusy('search', requestId, () =>
      searchEntries(activeZipPath, searchTerm, SEARCH_RESULT_LIMIT)
    );
    if (rows && isCurrentRequest('search', requestId)) {
      searchIndexState.setSearchRows(rows);
      if (recordRecent && rows.length > 0) {
        pushRecentSearch(searchTerm);
      }
    }
  }

  async function bootMasterFeaturesWithPath(nextZipPath: string | null, silentNoCache = false) {
    const requestId = ++bootRequestSeq;
    invalidateSourceRequests();
    setRetryAction(async () => {
      await bootMasterFeaturesWithPath(nextZipPath, false);
    });
    beginBusy('boot');
    error = '';
    showProgress = true;
    progress = { phase: 'start', current: 0, total: 1, message: '초기화 중' };
    try {
      const buildKey = await startMasterBuild(nextZipPath);
      while (requestId === bootRequestSeq) {
        const status = await getMasterBuildStatus(nextZipPath, buildKey);
        if (requestId !== bootRequestSeq) return;
        progress = {
          phase: status.phase,
          current: status.current,
          total: status.total,
          message: status.message
        };

        if (status.done) {
          if (!status.success) {
            throw new Error(status.error ?? '빌드 실패');
          }
          if (!status.summary) throw new Error('사전 구축 결과가 비어 있습니다.');
          const nextSummary = status.summary;
          const resolvedPath = nextSummary.zipPath;
          const [nextContents, nextIndex] = await Promise.all([
            getMasterContents(resolvedPath),
            getIndexEntries(resolvedPath, '', INDEX_PAGE_LIMIT)
          ]);
          if (requestId !== bootRequestSeq) return;

          invalidateSourceRequests();
          zipPath = resolvedPath;
          masterSummary = nextSummary;
          contents = nextContents;
          libraryState.setSourceScope(resolvedPath);
          searchIndexState.setIndexPrefix('');
          indexRowsPrefix = '';
          searchIndexState.setIndexRows(nextIndex, nextIndex.length === INDEX_PAGE_LIMIT);
          searchIndexState.setSearchQuery('');
          searchIndexState.clearSearch();
          clearSelection();

          if (autoOpenFirstContent && nextContents.length) {
            await openContent(nextContents[0].local);
          }
          break;
        }
        await new Promise((resolve) => setTimeout(resolve, BUILD_POLL_MS));
      }
    } catch (e) {
      if (requestId !== bootRequestSeq) return;
      const message = toErrorMessage(e);
      if (silentNoCache && message.includes('no managed zip cache found')) {
        error = '';
      } else {
        error = message;
      }
    } finally {
      if (requestId === bootRequestSeq) showProgress = false;
      endBusy('boot');
    }
  }

  async function retryLastOperation() {
    if (lastRetryAction) {
      await lastRetryAction();
      return;
    }
    if (zipPath) {
      await useZipPath(zipPath);
      return;
    }
    await bootFromManagedCache();
  }

  function dispose() {
    if (indexDebounceTimer) {
      clearTimeout(indexDebounceTimer);
      indexDebounceTimer = null;
    }
  }

  function clearError() {
    error = '';
  }

  function closeDetail() {
    invalidateDetailRequests();
    clearSelection();
  }

  function handleMobileBackNavigation(): boolean {
    if (detailState.selectedEntryId !== null || detailState.selectedContentLocal) {
      closeDetail();
      return true;
    }
    if (mobileTab !== 'home') {
      mobileTab = 'home';
      return true;
    }
    return false;
  }

  async function bootMasterFeatures() {
    await bootMasterFeaturesWithPath(zipPath);
  }

  function setAutoOpenFirstContent(enabled: boolean) {
    autoOpenFirstContent = enabled;
  }

  function setActiveTab(tab: Tab) {
    activeTab = tab;
  }

  function setMobileTab(tab: 'home' | 'search' | 'index' | 'favorites') {
    mobileTab = tab;
  }

  function setDragOver(value: boolean) {
    dragOver = value;
  }

  function setPreprocessEnabled(enabled: boolean) {
    readerPrefsState.setPreprocessEnabled(enabled);
  }

  function setMarkerPreprocessEnabled(enabled: boolean) {
    readerPrefsState.setMarkerPreprocessEnabled(enabled);
  }

  function setReaderFontSize(value: ReaderFontSize) {
    readerPrefsState.setReaderFontSize(value);
  }

  function setReaderLineHeight(value: ReaderLineHeight) {
    readerPrefsState.setReaderLineHeight(value);
  }

  function setReaderWidth(value: ReaderWidth) {
    readerPrefsState.setReaderWidth(value);
  }

  async function bootFromManagedCache() {
    await bootMasterFeaturesWithPath(null, true);
  }

  async function useZipPath(path: string) {
    const nextPath = path.trim();
    if (!nextPath) {
      error = 'ZIP 경로가 비어 있습니다.';
      return;
    }
    if (!/\.zip$/i.test(nextPath)) {
      error = 'ZIP 파일만 선택할 수 있습니다.';
      return;
    }
    setRetryAction(async () => {
      await useZipPath(nextPath);
    });
    await bootMasterFeaturesWithPath(nextPath);
  }

  async function pickZipFile() {
    try {
      const selected = await openDialog({
        multiple: false,
        directory: false,
        pickerMode: 'document',
        fileAccessMode: 'copy',
        filters: [{ name: 'ZIP', extensions: ['zip'] }]
      });
      if (!selected || Array.isArray(selected)) return;
      beginSourcePrepare();
      const resolvedPath = await resolvePickedZipPath(selected);
      await useZipPath(resolvedPath);
    } catch (e) {
      error = `파일 선택 실패: ${toErrorMessage(e)}`;
    } finally {
      endSourcePrepare();
    }
  }

  async function openContent(local: string, sourcePath: string | null = null) {
    setRetryAction(async () => {
      await openContent(local, sourcePath);
    });
    const requestId = startRequest('detail');
    const activeZipPath = zipPath;
    detailState.beginContentSelection(local);
    const page = await withBusy('detail', requestId, () =>
      getContentPage(activeZipPath, local, sourcePath)
    );
    if (!isCurrentRequest('detail', requestId)) return;
    if (!page) {
      clearSelection();
      return;
    }
    // Pages without a <title> fall back to their file name; prefer the TOC label instead.
    const tocTitle = contents.find((item) => item.local === local)?.title;
    const resolvedPage = page.title === page.local && tocTitle ? { ...page, title: tocTitle } : page;
    detailState.setContent(resolvedPage, local);
    pushRecentView({
      key: `content:${page.sourcePath}:${local}`,
      kind: 'content',
      label: resolvedPage.title,
      id: null,
      local,
      sourcePath: page.sourcePath,
      viewedAt: Date.now()
    });
  }

  async function openEntry(id: number) {
    setRetryAction(async () => {
      await openEntry(id);
    });
    const requestId = startRequest('detail');
    const activeZipPath = zipPath;
    detailState.beginEntrySelection(id);
    const entry = await withBusy('detail', requestId, () => getEntryDetail(activeZipPath, id));
    if (!isCurrentRequest('detail', requestId)) return;
    if (!entry) {
      clearSelection();
      return;
    }
    detailState.setEntry(entry, id);
    pushRecentView({
      key: `entry:${id}`,
      kind: 'entry',
      label: entry.headword,
      id,
      local: null,
      sourcePath: entry.sourcePath,
      viewedAt: Date.now()
    });
  }

  function setIndexPrefix(value: string) {
    searchIndexState.setIndexPrefix(value);
    if (indexDebounceTimer) clearTimeout(indexDebounceTimer);
    indexDebounceTimer = setTimeout(() => {
      void loadIndexByPrefix(value);
    }, INDEX_DEBOUNCE_MS);
  }

  function setSearchQuery(value: string) {
    const changed = value !== searchIndexState.searchQuery;
    searchIndexState.setSearchQuery(value);
    if (changed) {
      invalidateSearchRequests();
      searchIndexState.clearSearch();
    }
  }

  async function submitSearch() {
    await runSearch(searchIndexState.searchQuery, true);
  }

  function useRecentSearch(query: string) {
    searchIndexState.setSearchQuery(query);
    void runSearch(query, false);
  }

  function openRecentView(item: RecentViewItem) {
    if (item.kind === 'entry' && item.id != null) {
      void openEntry(item.id);
      return;
    }
    if (item.kind === 'content' && item.local) {
      void openContent(item.local, item.sourcePath);
    }
  }

  function isFavoriteEntry(id: number): boolean {
    return libraryState.isFavoriteEntry(id);
  }

  function isFavoriteContent(local: string, sourcePath: string | null): boolean {
    return libraryState.isFavoriteContent(local, sourcePath);
  }

  function toggleFavoriteEntry(entry: Pick<EntryDetail, 'id' | 'headword' | 'sourcePath'>) {
    libraryState.toggleFavoriteEntry(entry);
  }

  function toggleFavoriteContent(content: Pick<ContentPage, 'local' | 'title' | 'sourcePath'>) {
    libraryState.toggleFavoriteContent(content);
  }

  function toggleCurrentFavorite() {
    if (detailState.detailMode === 'entry' && detailState.selectedEntry) {
      toggleFavoriteEntry(detailState.selectedEntry);
      return;
    }
    if (detailState.detailMode === 'content' && detailState.selectedContent) {
      toggleFavoriteContent(detailState.selectedContent);
    }
  }

  function isCurrentFavorite(): boolean {
    if (detailState.detailMode === 'entry' && detailState.selectedEntry) {
      return isFavoriteEntry(detailState.selectedEntry.id);
    }
    if (detailState.detailMode === 'content' && detailState.selectedContent) {
      return isFavoriteContent(detailState.selectedContent.local, detailState.selectedContent.sourcePath);
    }
    return false;
  }

  function currentFavoriteFolderId(): string | null {
    if (detailState.detailMode === 'entry' && detailState.selectedEntry) {
      return libraryState.favoriteFolderId(`entry:${detailState.selectedEntry.id}`);
    }
    if (detailState.detailMode === 'content' && detailState.selectedContent) {
      const { sourcePath, local } = detailState.selectedContent;
      return libraryState.favoriteFolderId(`content:${sourcePath ?? ''}:${local}`);
    }
    return null;
  }

  function removeFavorite(key: string) {
    libraryState.removeFavorite(key);
  }

  function setActiveBookmarkFolder(folderId: string) {
    libraryState.setActiveBookmarkFolder(folderId);
  }

  function createBookmarkFolder(name: string): string | null {
    return libraryState.createBookmarkFolder(name);
  }

  function renameBookmarkFolder(folderId: string, name: string) {
    libraryState.renameBookmarkFolder(folderId, name);
  }

  function deleteBookmarkFolder(folderId: string) {
    libraryState.deleteBookmarkFolder(folderId);
  }

  function moveFavoriteToFolder(key: string, folderId: string) {
    libraryState.moveFavoriteToFolder(key, folderId);
  }

  function addCurrentFavoriteToFolder(folderId: string) {
    if (!libraryState.bookmarkFolders.some((folder) => folder.id === folderId)) return;
    // The last folder used becomes the default target for the next save.
    libraryState.setActiveBookmarkFolder(folderId);
    if (detailState.detailMode === 'entry' && detailState.selectedEntry) {
      libraryState.addFavoriteEntry(detailState.selectedEntry, folderId);
      return;
    }
    if (detailState.detailMode === 'content' && detailState.selectedContent) {
      libraryState.addFavoriteContent(detailState.selectedContent, folderId);
    }
  }

  function openFavorite(item: FavoriteItem) {
    if (item.kind === 'entry' && item.id != null) {
      void openEntry(item.id);
      return;
    }
    if (item.kind === 'content' && item.local) {
      void openContent(item.local, item.sourcePath);
    }
  }

  async function openInlineHref(
    href: string,
    currentSourcePath: string | null,
    currentLocal: string | null
  ) {
    setRetryAction(async () => {
      await openInlineHref(href, currentSourcePath, currentLocal);
    });
    const requestId = startRequest('detail');
    const activeZipPath = zipPath;
    const target = await withBusy('detail', requestId, () =>
      resolveLinkTarget(activeZipPath, href, currentSourcePath, currentLocal)
    );
    if (!target || !isCurrentRequest('detail', requestId)) return;
    if (target.kind === 'content') {
      await openContent(target.local, target.sourcePath);
      return;
    }
    await openEntry(target.id);
  }

  async function resolveInlineImageHref(
    href: string,
    currentSourcePath: string | null,
    currentLocal: string | null
  ): Promise<string | null> {
    try {
      return await resolveMediaDataUrl(zipPath, href, currentSourcePath, currentLocal);
    } catch {
      return null;
    }
  }

  return {
    // NOTE: do not destructure reactive getters from this object;
    // always access as `dictionaryStore.someValue` to preserve rune reactivity.
    get loading() { return loading; },
    get isBooting() { return isBooting; },
    get isSearching() { return isSearching; },
    get isOpeningDetail() { return isOpeningDetail; },
    get autoOpenFirstContent() { return autoOpenFirstContent; },
    get error() { return error; },
    get zipPath() { return zipPath; },
    get activeZipName() { return zipDisplayName(zipPath); },
    get activeTab() { return activeTab; },
    get mobileTab() { return mobileTab; },
    get masterSummary() { return masterSummary; },
    get contents() { return contents; },
    get progress() { return progress; },
    get showProgress() { return showProgress; },
    get dragOver() { return dragOver; },
    get recentSearches() { return libraryState.recentSearches; },
    get recentViews() { return libraryState.recentViews; },
    get favorites() { return libraryState.visibleFavorites; },
    get allFavorites() { return libraryState.favorites; },
    get legacyFavoriteCount() { return libraryState.legacyFavoriteCount; },
    get bookmarkFolders() { return libraryState.bookmarkFolders; },
    get activeBookmarkFolderId() { return libraryState.activeBookmarkFolderId; },
    get preprocessEnabled() { return readerPrefsState.preprocessEnabled; },
    get markerPreprocessEnabled() { return readerPrefsState.markerPreprocessEnabled; },
    get readerFontSize() { return readerPrefsState.readerFontSize; },
    get readerLineHeight() { return readerPrefsState.readerLineHeight; },
    get readerWidth() { return readerPrefsState.readerWidth; },
    get indexPrefix() { return searchIndexState.indexPrefix; },
    get indexRows() { return searchIndexState.indexRows; },
    get indexLoading() { return searchIndexState.indexLoading; },
    get indexHasMore() { return searchIndexState.indexHasMore; },
    get indexLoadingMore() { return searchIndexState.indexLoadingMore; },
    get searchQuery() { return searchIndexState.searchQuery; },
    get committedSearchQuery() { return searchIndexState.committedSearchQuery; },
    get searchRows() { return searchIndexState.searchRows; },
    get selectedContent() { return detailState.selectedContent; },
    get selectedEntry() { return detailState.selectedEntry; },
    get detailMode() { return detailState.detailMode; },
    get selectedContentLocal() { return detailState.selectedContentLocal; },
    get selectedEntryId() { return detailState.selectedEntryId; },
    dispose,
    clearError,
    retryLastOperation,
    closeDetail,
    handleMobileBackNavigation,
    bootMasterFeatures,
    setAutoOpenFirstContent,
    setActiveTab,
    setMobileTab,
    setDragOver,
    setPreprocessEnabled,
    setMarkerPreprocessEnabled,
    setReaderFontSize,
    setReaderLineHeight,
    setReaderWidth,
    bootFromManagedCache,
    useZipPath,
    pickZipFile,
    openContent,
    openEntry,
    setIndexPrefix,
    loadMoreIndex,
    setSearchQuery,
    submitSearch,
    useRecentSearch,
    openRecentView,
    isFavoriteEntry,
    isFavoriteContent,
    toggleFavoriteEntry,
    toggleFavoriteContent,
    toggleCurrentFavorite,
    isCurrentFavorite,
    currentFavoriteFolderId,
    removeFavorite,
    setActiveBookmarkFolder,
    createBookmarkFolder,
    renameBookmarkFolder,
    deleteBookmarkFolder,
    moveFavoriteToFolder,
    addCurrentFavoriteToFolder,
    openFavorite,
    openInlineHref,
    resolveInlineImageHref
  };
}
