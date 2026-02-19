import { MAX_FAVORITES, MAX_RECENT_SEARCHES, dedupeRecentViews } from '$lib/stores/dictionaryPrefsStore';
import type { ContentPage, EntryDetail, FavoriteItem, RecentViewItem } from '$lib/types/dictionary';

export type LibrarySnapshot = {
  recentSearches: string[];
  recentViews: RecentViewItem[];
  favorites: FavoriteItem[];
};

export function createLibraryState(onChange: () => void) {
  let recentSearches = $state<string[]>([]);
  let recentViews = $state<RecentViewItem[]>([]);
  let favorites = $state<FavoriteItem[]>([]);

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
    applySnapshot(snapshot: LibrarySnapshot) {
      recentSearches = snapshot.recentSearches;
      recentViews = snapshot.recentViews;
      favorites = snapshot.favorites;
    },
    toSnapshot(): LibrarySnapshot {
      return {
        recentSearches,
        recentViews,
        favorites
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
        sourcePath: entry.sourcePath
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
        sourcePath: content.sourcePath
      };
      favorites = [nextItem, ...favorites].slice(0, MAX_FAVORITES);
      onChange();
    },
    removeFavorite(key: string) {
      favorites = favorites.filter((item) => item.key !== key);
      onChange();
    }
  };
}

export type LibraryState = ReturnType<typeof createLibraryState>;
