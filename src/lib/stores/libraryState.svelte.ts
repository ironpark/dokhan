import {
  DEFAULT_BOOKMARK_FOLDER_ID,
  MAX_BOOKMARK_FOLDERS,
  MAX_FAVORITES,
  MAX_RECENT_SEARCHES,
  MAX_RECENT_VIEWS,
  MAX_STORED_FAVORITES,
  MAX_STORED_RECENT_VIEWS,
  dedupeRecentViews
} from '$lib/stores/dictionaryPrefsStore';
import type { BookmarkFolder, ContentPage, EntryDetail, FavoriteItem, RecentViewItem } from '$lib/types/dictionary';

export type LibrarySnapshot = {
  recentSearches: string[];
  recentViews: RecentViewItem[];
  favorites: FavoriteItem[];
  bookmarkFolders: BookmarkFolder[];
  activeBookmarkFolderId: string;
};

const SCOPE_KEY_PREFIX = 'scope:';

export function createLibraryState(onChange: () => void) {
  let recentSearches = $state<string[]>([]);
  let recentViews = $state<RecentViewItem[]>([]);
  let favorites = $state<FavoriteItem[]>([]);
  let sourceScope = $state<string | null>(null);
  let bookmarkFolders = $state<BookmarkFolder[]>([
    { id: DEFAULT_BOOKMARK_FOLDER_ID, name: '기본', createdAt: 0 }
  ]);
  let activeBookmarkFolderId = $state(DEFAULT_BOOKMARK_FOLDER_ID);

  function ensureDefaultFolder(): BookmarkFolder[] {
    if (bookmarkFolders.some((folder) => folder.id === DEFAULT_BOOKMARK_FOLDER_ID)) {
      return bookmarkFolders;
    }
    return [
      { id: DEFAULT_BOOKMARK_FOLDER_ID, name: '기본', createdAt: 0 },
      ...bookmarkFolders
    ];
  }

  // A source path identifies a CHM inside a ZIP, so the managed ZIP path is
  // required to distinguish entries whose numeric IDs overlap across ZIPs.
  const scopePrefix = $derived(
    sourceScope ? `${SCOPE_KEY_PREFIX}${encodeURIComponent(sourceScope)}::` : null
  );

  function activeKey(key: string): string | null {
    return scopePrefix ? `${scopePrefix}${key}` : null;
  }

  function belongsToActiveSource(key: string): boolean {
    return scopePrefix !== null && key.startsWith(scopePrefix);
  }

  const activeRecentViews = $derived(recentViews.filter((item) => belongsToActiveSource(item.key)));
  const activeFavorites = $derived(favorites.filter((item) => belongsToActiveSource(item.key)));
  const visibleFavorites = $derived(
    activeFavorites.filter((item) => item.folderId === activeBookmarkFolderId)
  );
  const legacyFavoriteCount = $derived(
    favorites.filter((item) => !item.key.startsWith(SCOPE_KEY_PREFIX)).length
  );

  function trimActiveFavorites(rows: FavoriteItem[]): FavoriteItem[] {
    let activeCount = 0;
    return rows
      .filter((item) => !belongsToActiveSource(item.key) || ++activeCount <= MAX_FAVORITES)
      .slice(0, MAX_STORED_FAVORITES);
  }

  function trimActiveRecentViews(rows: RecentViewItem[]): RecentViewItem[] {
    let activeCount = 0;
    return dedupeRecentViews(rows, MAX_STORED_RECENT_VIEWS).filter(
      (item) => !belongsToActiveSource(item.key) || ++activeCount <= MAX_RECENT_VIEWS
    );
  }

  return {
    setSourceScope(zipPath: string | null) {
      sourceScope = zipPath?.trim() || null;
    },
    get recentSearches() {
      return recentSearches;
    },
    get recentViews() {
      return activeRecentViews;
    },
    get favorites() {
      return activeFavorites;
    },
    get legacyFavoriteCount() {
      return legacyFavoriteCount;
    },
    get bookmarkFolders() {
      return bookmarkFolders;
    },
    get activeBookmarkFolderId() {
      return activeBookmarkFolderId;
    },
    get visibleFavorites() {
      return visibleFavorites;
    },
    applySnapshot(snapshot: LibrarySnapshot) {
      recentSearches = snapshot.recentSearches;
      recentViews = snapshot.recentViews;
      favorites = snapshot.favorites;
      bookmarkFolders = snapshot.bookmarkFolders.length
        ? snapshot.bookmarkFolders
        : [{ id: DEFAULT_BOOKMARK_FOLDER_ID, name: '기본', createdAt: 0 }];
      bookmarkFolders = ensureDefaultFolder();
      if (bookmarkFolders.some((folder) => folder.id === snapshot.activeBookmarkFolderId)) {
        activeBookmarkFolderId = snapshot.activeBookmarkFolderId;
      } else {
        activeBookmarkFolderId = DEFAULT_BOOKMARK_FOLDER_ID;
      }
      favorites = favorites.map((item) => ({
        ...item,
        folderId: item.folderId || DEFAULT_BOOKMARK_FOLDER_ID
      }));
    },
    toSnapshot(): LibrarySnapshot {
      return {
        recentSearches,
        recentViews,
        favorites,
        bookmarkFolders,
        activeBookmarkFolderId
      };
    },
    pushRecentSearch(query: string) {
      const next = [query, ...recentSearches.filter((item) => item !== query)];
      recentSearches = next.slice(0, MAX_RECENT_SEARCHES);
      onChange();
    },
    pushRecentView(item: RecentViewItem) {
      const key = activeKey(item.key);
      if (!key) return;
      recentViews = trimActiveRecentViews([
        { ...item, key },
        ...recentViews.filter((row) => row.key !== key)
      ]);
      onChange();
    },
    isFavoriteEntry(id: number): boolean {
      const key = activeKey(`entry:${id}`);
      return key !== null && favorites.some((item) => item.key === key);
    },
    isFavoriteContent(local: string, sourcePath: string | null): boolean {
      const key = activeKey(`content:${sourcePath ?? ''}:${local}`);
      return key !== null && favorites.some((item) => item.key === key);
    },
    toggleFavoriteEntry(entry: Pick<EntryDetail, 'id' | 'headword' | 'sourcePath'>) {
      const key = activeKey(`entry:${entry.id}`);
      if (!key) return;
      if (favorites.some((item) => item.key === key)) {
        favorites = favorites.filter((item) => item.key !== key);
        onChange();
        return;
      }
      const nextItem: FavoriteItem = {
        key,
        kind: 'entry',
        label: entry.headword,
        id: entry.id,
        local: null,
        sourcePath: entry.sourcePath,
        folderId: activeBookmarkFolderId
      };
      favorites = trimActiveFavorites([nextItem, ...favorites]);
      onChange();
    },
    addFavoriteEntry(entry: Pick<EntryDetail, 'id' | 'headword' | 'sourcePath'>, folderId: string) {
      const key = activeKey(`entry:${entry.id}`);
      if (!key) return;
      if (favorites.some((item) => item.key === key)) {
        favorites = favorites.map((item) => (
          item.key === key ? { ...item, folderId } : item
        ));
        onChange();
        return;
      }
      const nextItem: FavoriteItem = {
        key,
        kind: 'entry',
        label: entry.headword,
        id: entry.id,
        local: null,
        sourcePath: entry.sourcePath,
        folderId
      };
      favorites = trimActiveFavorites([nextItem, ...favorites]);
      onChange();
    },
    toggleFavoriteContent(content: Pick<ContentPage, 'local' | 'title' | 'sourcePath'>) {
      const key = activeKey(`content:${content.sourcePath ?? ''}:${content.local}`);
      if (!key) return;
      if (favorites.some((item) => item.key === key)) {
        favorites = favorites.filter((item) => item.key !== key);
        onChange();
        return;
      }
      const nextItem: FavoriteItem = {
        key,
        kind: 'content',
        label: content.title,
        id: null,
        local: content.local,
        sourcePath: content.sourcePath,
        folderId: activeBookmarkFolderId
      };
      favorites = trimActiveFavorites([nextItem, ...favorites]);
      onChange();
    },
    addFavoriteContent(content: Pick<ContentPage, 'local' | 'title' | 'sourcePath'>, folderId: string) {
      const key = activeKey(`content:${content.sourcePath ?? ''}:${content.local}`);
      if (!key) return;
      if (favorites.some((item) => item.key === key)) {
        favorites = favorites.map((item) => (
          item.key === key ? { ...item, folderId } : item
        ));
        onChange();
        return;
      }
      const nextItem: FavoriteItem = {
        key,
        kind: 'content',
        label: content.title,
        id: null,
        local: content.local,
        sourcePath: content.sourcePath,
        folderId
      };
      favorites = trimActiveFavorites([nextItem, ...favorites]);
      onChange();
    },
    setActiveBookmarkFolder(folderId: string) {
      if (!bookmarkFolders.some((folder) => folder.id === folderId)) return;
      activeBookmarkFolderId = folderId;
      onChange();
    },
    createBookmarkFolder(name: string): string | null {
      const trimmed = name.trim();
      if (!trimmed) return null;
      if (bookmarkFolders.length >= MAX_BOOKMARK_FOLDERS) return null;
      const id = `folder:${Date.now().toString(36)}:${Math.random().toString(36).slice(2, 8)}`;
      bookmarkFolders = [
        ...bookmarkFolders,
        { id, name: trimmed, createdAt: Date.now() }
      ];
      activeBookmarkFolderId = id;
      onChange();
      return id;
    },
    renameBookmarkFolder(folderId: string, name: string) {
      const trimmed = name.trim();
      if (!trimmed) return;
      bookmarkFolders = bookmarkFolders.map((folder) => (
        folder.id === folderId ? { ...folder, name: trimmed } : folder
      ));
      onChange();
    },
    deleteBookmarkFolder(folderId: string) {
      if (folderId === DEFAULT_BOOKMARK_FOLDER_ID) return;
      if (!bookmarkFolders.some((folder) => folder.id === folderId)) return;
      bookmarkFolders = bookmarkFolders.filter((folder) => folder.id !== folderId);
      favorites = favorites.map((item) => (
        item.folderId === folderId
          ? { ...item, folderId: DEFAULT_BOOKMARK_FOLDER_ID }
          : item
      ));
      if (activeBookmarkFolderId === folderId) {
        activeBookmarkFolderId = DEFAULT_BOOKMARK_FOLDER_ID;
      }
      bookmarkFolders = ensureDefaultFolder();
      onChange();
    },
    moveFavoriteToFolder(key: string, folderId: string) {
      if (!bookmarkFolders.some((folder) => folder.id === folderId)) return;
      favorites = favorites.map((item) => (
        item.key === key ? { ...item, folderId } : item
      ));
      onChange();
    },
    removeFavorite(key: string) {
      favorites = favorites.filter((item) => item.key !== key);
      onChange();
    }
  };
}

export type LibraryState = ReturnType<typeof createLibraryState>;
