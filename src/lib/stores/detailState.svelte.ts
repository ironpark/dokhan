import type { ContentPage, DetailMode, EntryDetail } from '$lib/types/dictionary';

export function createDetailState() {
  let selectedContent = $state<ContentPage | null>(null);
  let selectedEntry = $state<EntryDetail | null>(null);
  let detailMode = $state<DetailMode>('none');
  let selectedContentLocal = $state('');
  let selectedEntryId = $state<number | null>(null);

  return {
    get selectedContent() {
      return selectedContent;
    },
    get selectedEntry() {
      return selectedEntry;
    },
    get detailMode() {
      return detailMode;
    },
    get selectedContentLocal() {
      return selectedContentLocal;
    },
    get selectedEntryId() {
      return selectedEntryId;
    },
    clearSelection() {
      selectedEntry = null;
      selectedEntryId = null;
      selectedContent = null;
      selectedContentLocal = '';
      detailMode = 'none';
    },
    setContent(page: ContentPage, local: string) {
      selectedContent = page;
      selectedContentLocal = local;
      selectedEntry = null;
      selectedEntryId = null;
      detailMode = 'content';
    },
    setEntry(entry: EntryDetail, id: number) {
      selectedEntry = entry;
      selectedContent = null;
      selectedEntryId = id;
      selectedContentLocal = '';
      detailMode = 'entry';
    },
    beginEntrySelection(id: number) {
      selectedEntryId = id;
      selectedContentLocal = '';
    }
  };
}

export type DetailState = ReturnType<typeof createDetailState>;
