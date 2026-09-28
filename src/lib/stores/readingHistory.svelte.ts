export type ReadingLocation =
  | { kind: 'entry'; id: number }
  | { kind: 'content'; local: string; sourcePath: string | null };

type HistoryItem = {
  key: string;
  location: ReadingLocation;
  /** Last reader scroll offset, restored when stepping back or forward to it. */
  scrollTop: number;
};

export function readingLocationKey(location: ReadingLocation): string {
  return location.kind === 'entry'
    ? `entry:${location.id}`
    : `content:${location.sourcePath ?? ''}:${location.local}`;
}

/** Browser-style back/forward list of opened entries and pages. */
export function createReadingHistory(limit = 100) {
  // Items change on every scroll; only the cursor and length need to be reactive.
  let items: HistoryItem[] = [];
  let index = $state(-1);
  let length = $state(0);

  return {
    get canGoBack() {
      return index > 0;
    },
    get canGoForward() {
      return index >= 0 && index < length - 1;
    },
    /** Record a newly shown location, dropping any forward entries. Revisiting the current one is a no-op. */
    visit(location: ReadingLocation) {
      const key = readingLocationKey(location);
      if (items[index]?.key === key) return;
      items = items.slice(0, index + 1);
      items.push({ key, location, scrollTop: 0 });
      if (items.length > limit) items = items.slice(items.length - limit);
      index = items.length - 1;
      length = items.length;
    },
    /** Move the cursor and return the location to show, or null at either end. */
    step(delta: -1 | 1): { location: ReadingLocation; scrollTop: number } | null {
      const next = index + delta;
      if (next < 0 || next >= length) return null;
      index = next;
      const item = items[next];
      return { location: item.location, scrollTop: item.scrollTop };
    },
    /** Ignored unless `key` is the cursor's item, so a page being swapped out can't overwrite another's position. */
    recordScroll(key: string, scrollTop: number) {
      const item = items[index];
      if (item?.key === key) item.scrollTop = scrollTop;
    },
    clear() {
      items = [];
      index = -1;
      length = 0;
    }
  };
}

export type ReadingHistory = ReturnType<typeof createReadingHistory>;
