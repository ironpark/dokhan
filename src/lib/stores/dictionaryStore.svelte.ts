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

function toErrorMessage(errorValue: unknown): string {
  return typeof errorValue === 'string' ? errorValue : String(errorValue);
}

export interface DictionaryStore {
  readonly loading: boolean;
  readonly isBooting: boolean;
  readonly isSearching: boolean;
  readonly isOpeningDetail: boolean;
  readonly autoOpenFirstContent: boolean;
  readonly error: string;
  readonly zipPath: string | null;
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
  readonly searchQuery: string;
  readonly committedSearchQuery: string;
  readonly searchRows: SearchHit[];
  readonly selectedContent: ContentPage | null;
  readonly selectedEntry: EntryDetail | null;
  readonly detailMode: DetailMode;
  readonly selectedContentLocal: string;
  readonly selectedEntryId: number | null;

  dispose(): void;
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
  let detailRequestSeq = 0;
  let searchRequestSeq = 0;
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

  async function withBusy<T>(kind: 'search' | 'detail', task: () => Promise<T>): Promise<T | undefined> {
    beginBusy(kind);
    error = '';
    try {
      return await task();
    } catch (e) {
      error = toErrorMessage(e);
      return undefined;
    } finally {
      endBusy(kind);
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

  async function loadIndexByPrefix(prefix: string) {
    if (!masterSummary) return;
    const trimmed = prefix.trim();
    setRetryAction(async () => {
      await loadIndexByPrefix(trimmed);
    });
    const requestId = ++indexRequestSeq;
    searchIndexState.setIndexLoading(true);
    try {
      const rows = await getIndexEntries(zipPath, trimmed, trimmed ? 500 : null);
      if (requestId === indexRequestSeq && searchIndexState.indexPrefix.trim() === trimmed) {
        searchIndexState.setIndexRows(rows);
      }
    } catch (e) {
      error = toErrorMessage(e);
    } finally {
      if (requestId === indexRequestSeq) searchIndexState.setIndexLoading(false);
    }
  }

  async function runSearch(rawQuery: string, recordRecent: boolean) {
    const searchTerm = rawQuery.trim();
    if (!searchTerm) {
      searchIndexState.clearSearch();
      return;
    }
    searchIndexState.setCommittedSearchQuery(searchTerm);
    const requestId = ++searchRequestSeq;
    setRetryAction(async () => {
      searchIndexState.setSearchQuery(searchTerm);
      searchIndexState.setCommittedSearchQuery(searchTerm);
      const retryRows = await withBusy('search', () => searchEntries(zipPath, searchTerm, 200));
      if (retryRows && requestId === searchRequestSeq) {
        searchIndexState.setSearchRows(retryRows);
      }
    });
    const rows = await withBusy('search', () => searchEntries(zipPath, searchTerm, 200));
    if (rows && requestId === searchRequestSeq) {
      searchIndexState.setSearchRows(rows);
      if (recordRecent && rows.length > 0) {
        pushRecentSearch(searchTerm);
      }
    }
  }

  async function bootMasterFeaturesWithPath(nextZipPath: string | null, silentNoCache = false) {
    setRetryAction(async () => {
      await bootMasterFeaturesWithPath(nextZipPath, false);
    });
    beginBusy('boot');
    error = '';
    showProgress = true;
    progress = { phase: 'start', current: 0, total: 1, message: '초기화 중' };
    try {
      await startMasterBuild(nextZipPath);
      while (true) {
        const status = await getMasterBuildStatus(nextZipPath);
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
          masterSummary = status.summary;
          zipPath = status.summary?.zipPath ?? nextZipPath;
          break;
        }
        await new Promise((resolve) => setTimeout(resolve, BUILD_POLL_MS));
      }

      const [nextContents, nextIndex] = await Promise.all([
        getMasterContents(zipPath),
        getIndexEntries(zipPath, '', null)
      ]);

      contents = nextContents;
      searchIndexState.setIndexRows(nextIndex);
      searchIndexState.setSearchRows([]);
      clearSelection();

      if (autoOpenFirstContent && contents.length) {
        await openContent(contents[0].local);
      }
    } catch (e) {
      const message = toErrorMessage(e);
      if (silentNoCache && message.includes('no managed zip cache found')) {
        error = '';
      } else {
        error = message;
      }
    } finally {
      showProgress = false;
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

  function closeDetail() {
    clearSelection();
  }

  function handleMobileBackNavigation(): boolean {
    if (detailState.selectedEntryId || detailState.selectedContentLocal) {
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
    zipPath = nextPath;
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
    const requestId = ++detailRequestSeq;
    const page = await withBusy('detail', () => getContentPage(zipPath, local, sourcePath));
    if (!page || requestId !== detailRequestSeq) return;
    detailState.setContent(page, local);
    pushRecentView({
      key: `content:${page.sourcePath}:${local}`,
      kind: 'content',
      label: page.title,
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
    const requestId = ++detailRequestSeq;
    detailState.beginEntrySelection(id);
    const entry = await withBusy('detail', () => getEntryDetail(zipPath, id));
    if (!entry || requestId !== detailRequestSeq) return;
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
    searchIndexState.setSearchQuery(value);
    if (!value.trim()) {
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
    const target = await withBusy('detail', () =>
      resolveLinkTarget(zipPath, href, currentSourcePath, currentLocal)
    );
    if (!target) return;
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
    get searchQuery() { return searchIndexState.searchQuery; },
    get committedSearchQuery() { return searchIndexState.committedSearchQuery; },
    get searchRows() { return searchIndexState.searchRows; },
    get selectedContent() { return detailState.selectedContent; },
    get selectedEntry() { return detailState.selectedEntry; },
    get detailMode() { return detailState.detailMode; },
    get selectedContentLocal() { return detailState.selectedContentLocal; },
    get selectedEntryId() { return detailState.selectedEntryId; },
    dispose,
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
