import {
  DEFAULT_BOOKMARK_FOLDER_ID,
  MAX_BOOKMARK_FOLDERS,
  MAX_FAVORITES,
  MAX_RECENT_SEARCHES,
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

export function createLibraryState(onChange: () => void) {
  let recentSearches = $state<string[]>([]);
  let recentViews = $state<RecentViewItem[]>([]);
  let favorites = $state<FavoriteItem[]>([]);
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

  return {
    get recentSearches() {
      return recentSearches;
    },
    get recentViews() {
      return recentViews;
    },
    get favorites() {
      return favorites;
    },
    get bookmarkFolders() {
      return bookmarkFolders;
    },
    get activeBookmarkFolderId() {
      return activeBookmarkFolderId;
    },
    get visibleFavorites() {
      return favorites.filter((item) => item.folderId === activeBookmarkFolderId);
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
      const next = [item, ...recentViews.filter((row) => row.key !== item.key)];
      recentViews = dedupeRecentViews(next);
      onChange();
    },
    isFavoriteEntry(id: number): boolean {
      return favorites.some((item) => item.kind === 'entry' && item.id === id);
    },
    isFavoriteContent(local: string, sourcePath: string | null): boolean {
      const key = `content:${sourcePath ?? ''}:${local}`;
      return favorites.some((item) => item.key === key);
    },
    toggleFavoriteEntry(entry: Pick<EntryDetail, 'id' | 'headword' | 'sourcePath'>) {
      const key = `entry:${entry.id}`;
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
      favorites = [nextItem, ...favorites].slice(0, MAX_FAVORITES);
      onChange();
    },
    addFavoriteEntry(entry: Pick<EntryDetail, 'id' | 'headword' | 'sourcePath'>, folderId: string) {
      const key = `entry:${entry.id}`;
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
      favorites = [nextItem, ...favorites].slice(0, MAX_FAVORITES);
      onChange();
    },
    toggleFavoriteContent(content: Pick<ContentPage, 'local' | 'title' | 'sourcePath'>) {
      const key = `content:${content.sourcePath ?? ''}:${content.local}`;
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
      favorites = [nextItem, ...favorites].slice(0, MAX_FAVORITES);
      onChange();
    },
    addFavoriteContent(content: Pick<ContentPage, 'local' | 'title' | 'sourcePath'>, folderId: string) {
      const key = `content:${content.sourcePath ?? ''}:${content.local}`;
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
      favorites = [nextItem, ...favorites].slice(0, MAX_FAVORITES);
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
