import type { DictionaryIndexEntry, SearchHit } from '$lib/types/dictionary';

export function createSearchIndexState() {
  let indexPrefix = $state('');
  let indexRows = $state<DictionaryIndexEntry[]>([]);
  let indexLoading = $state(false);

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
    setIndexRows(rows: DictionaryIndexEntry[]) {
      indexRows = rows;
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
