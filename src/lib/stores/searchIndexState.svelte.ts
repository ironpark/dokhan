import type { DictionaryIndexEntry, SearchHit } from '$lib/types/dictionary';

export function createSearchIndexState() {
  let indexPrefix = $state('');
  let indexRows = $state<DictionaryIndexEntry[]>([]);
  let indexLoading = $state(false);
  // Pages beyond the first are fetched as the list nears its end.
  let indexHasMore = $state(false);
  let indexLoadingMore = $state(false);

  let searchQuery = $state('');
  let committedSearchQuery = $state('');
  let searchRows = $state<SearchHit[]>([]);

  return {
    get indexPrefix() {
      return indexPrefix;
    },
    get indexRows() {
      return indexRows;
    },
    get indexLoading() {
      return indexLoading;
    },
    get indexHasMore() {
      return indexHasMore;
    },
    get indexLoadingMore() {
      return indexLoadingMore;
    },
    get searchQuery() {
      return searchQuery;
    },
    get committedSearchQuery() {
      return committedSearchQuery;
    },
    get searchRows() {
      return searchRows;
    },
    setIndexPrefix(value: string) {
      indexPrefix = value;
    },
    setIndexLoading(value: boolean) {
      indexLoading = value;
    },
    setIndexRows(rows: DictionaryIndexEntry[], hasMore = false) {
      indexRows = rows;
      indexHasMore = hasMore;
    },
    appendIndexRows(rows: DictionaryIndexEntry[], hasMore: boolean) {
      if (rows.length) indexRows = [...indexRows, ...rows];
      indexHasMore = hasMore;
    },
    setIndexLoadingMore(value: boolean) {
      indexLoadingMore = value;
    },
    setSearchQuery(value: string) {
      searchQuery = value;
    },
    setCommittedSearchQuery(value: string) {
      committedSearchQuery = value;
    },
    setSearchRows(rows: SearchHit[]) {
      searchRows = rows;
    },
    clearSearch() {
      searchRows = [];
      committedSearchQuery = '';
    }
  };
}

export type SearchIndexState = ReturnType<typeof createSearchIndexState>;
