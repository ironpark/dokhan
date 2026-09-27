<script lang="ts">
  import type { DictionaryIndexEntry } from "$lib/types/dictionary";
  import { createVirtualizer } from "@tanstack/svelte-virtual";
  import Input from "$lib/components/ui/Input.svelte";
  import ListItem from "$lib/components/ui/ListItem.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";

  let {
    query,
    rows,
    loading = false,
    inputAtBottom = false,
    selectedId = null,
    onQueryChange,
    onOpen,
  }: {
    query: string;
    rows: DictionaryIndexEntry[];
    loading?: boolean;
    inputAtBottom?: boolean;
    selectedId?: number | null;
    onQueryChange: (value: string) => void;
    onOpen: (id: number) => void;
  } = $props();

  let listEl = $state<HTMLElement | null>(null);

  const virtualizer = createVirtualizer({
    count: 0,
    getScrollElement: () => listEl,
    estimateSize: () => 38,
    overscan: 5,
  });

  let lastRows: DictionaryIndexEntry[] | null = null;
  let lastVirtualizerCount = $state(-1);
  let highlightCacheToken = $state("");
  const highlightSegmentCache = new Map<string, Segment[]>();

  $effect(() => {
    const nextCount = rows.length;
    if (rows !== lastRows) {
      lastRows = rows;
      if (listEl) listEl.scrollTop = 0;
      $virtualizer.scrollToIndex(0);
    }
    if (nextCount !== lastVirtualizerCount) {
      lastVirtualizerCount = nextCount;
      $virtualizer.setOptions({
        count: nextCount,
        getScrollElement: () => listEl,
      });
      queueMicrotask(() => {
        $virtualizer.measure();
      });
    }
  });

  $effect(() => {
    const nextToken = `${query}\u0001${rows.length}`;
    if (nextToken !== highlightCacheToken) {
      highlightCacheToken = nextToken;
      highlightSegmentCache.clear();
    }
  });

  $effect(() => {
    if (!listEl) return;
    const ro = new ResizeObserver(() => {
      $virtualizer.measure();
    });
    ro.observe(listEl);
    return () => ro.disconnect();
  });

  const virtualRows = $derived($virtualizer.getVirtualItems());
  const totalSize = $derived($virtualizer.getTotalSize());

  type Segment = { text: string; hit: boolean };

  function splitByHighlights(
    text: string,
    highlights: Array<{ start: number; end: number }>,
  ): Segment[] {
    const chars = Array.from(text);
    if (!chars.length || !highlights.length) {
      return [{ text, hit: false }];
    }

    const ranges = highlights
      .map(({ start, end }) => ({
        start: Math.max(0, Math.min(chars.length, start)),
        end: Math.max(0, Math.min(chars.length, end)),
      }))
      .filter((range) => range.end > range.start)
      .sort((a, b) => a.start - b.start);

    if (!ranges.length) {
      return [{ text, hit: false }];
    }

    const merged: Array<{ start: number; end: number }> = [];
    for (const range of ranges) {
      const last = merged[merged.length - 1];
      if (last && range.start <= last.end) {
        last.end = Math.max(last.end, range.end);
      } else {
        merged.push({ ...range });
      }
    }

    const segments: Segment[] = [];
    let cursor = 0;
    for (const range of merged) {
      if (range.start > cursor) {
        segments.push({ text: chars.slice(cursor, range.start).join(""), hit: false });
      }
      segments.push({ text: chars.slice(range.start, range.end).join(""), hit: true });
      cursor = range.end;
    }
    if (cursor < chars.length) {
      segments.push({ text: chars.slice(cursor).join(""), hit: false });
    }
    return segments.length ? segments : [{ text, hit: false }];
  }

  function getSegmentsForRow(row: DictionaryIndexEntry): Segment[] {
    const rangeKey = row.headwordHighlights.length
      ? row.headwordHighlights
          .map(({ start, end }) => `${start}-${end}`)
          .join(",")
      : "";
    const cacheKey = `${row.id}\u0001${row.headword}\u0001${rangeKey}`;
    const cached = highlightSegmentCache.get(cacheKey);
    if (cached) return cached;

    const next = splitByHighlights(row.headword, row.headwordHighlights);
    if (highlightSegmentCache.size >= 1200) {
      const oldest = highlightSegmentCache.keys().next().value;
      if (oldest) highlightSegmentCache.delete(oldest);
    }
    highlightSegmentCache.set(cacheKey, next);
    return next;
  }
</script>

<section class="panel" class:input-bottom={inputAtBottom}>
  <div class="search-line">
    <Input
      class="index-input"
      value={query}
      aria-label="색인 검색어"
      oninput={(e) => onQueryChange((e.target as HTMLInputElement).value)}
      onclear={() => onQueryChange("")}
      clearable={true}
      placeholder="색인 검색 (예: hnd, ab)"
    />
    {#if !loading && !query.trim() && rows.length >= 500}
      <p class="index-limit-notice" role="status">
        최대 500개를 표시합니다. 단어를 입력하면 전체 색인에서 찾습니다.
      </p>
    {/if}
    {#if !loading && query.trim() && rows.length > 0}
      <p class="index-result-summary" role="status">{rows.length >= 500 ? '상위 500개 색인 항목' : `색인 항목 ${rows.length}개`}</p>
    {/if}
  </div>
  <div class="entry-list" bind:this={listEl}>
    {#if loading}
      <EmptyState title="색인을 불러오는 중입니다." compact={true} />
    {:else if !rows.length && query.trim()}
      <EmptyState
        title="일치하는 색인 항목이 없습니다."
        description="철자나 키워드를 바꿔 다시 시도해 보세요."
        compact={true}
      />
    {:else if !rows.length}
      <EmptyState title="색인 데이터가 없습니다." compact={true} />
    {:else}
      <div style="height: {totalSize}px; width: 100%; position: relative;">
        {#each virtualRows as row (row.index)}
          <div
            style="position: absolute; top: 0; left: 0; width: 100%; height: {row.size}px; transform: translateY({row.start}px);"
          >
            {#if rows[row.index]}
              <ListItem
                selected={selectedId === rows[row.index].id}
                onclick={() => onOpen(rows[row.index].id)}
              >
                {#each getSegmentsForRow(rows[row.index]) as seg}
                  {#if seg.hit}
                    <strong>{seg.text}</strong>
                  {:else}
                    {seg.text}
                  {/if}
                {/each}
              </ListItem>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
</section>

<style>
  .panel {
    min-height: 0;
    height: 100%;
    display: grid;
    grid-template-rows: auto 1fr;
  }

  .panel.input-bottom {
    grid-template-rows: 1fr auto;
  }

  .search-line {
    margin: 0;
    padding: 12px 14px;
    display: grid;
    grid-template-columns: 1fr;
    gap: 8px;
    align-items: center;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
  }

  :global(.index-input input) {
    height: 44px;
  }

  .index-limit-notice,
  .index-result-summary {
    margin: 0;
    color: var(--color-text-muted);
    font-size: var(--font-size-control-sm);
    line-height: var(--line-height-normal);
  }

  .panel.input-bottom .search-line {
    order: 2;
    border-top: 1px solid var(--color-border);
    background: var(--color-surface);
    padding-bottom: calc(10px + env(safe-area-inset-bottom));
  }

  .entry-list {
    margin: 0;
    padding: 0; /* Changed from 0 10px 10px */
    list-style: none;
    min-height: 0;
    height: 100%;
    box-sizing: border-box;
    overflow-y: auto;
    scrollbar-gutter: stable;
    position: relative;
  }

  .panel.input-bottom .entry-list {
    order: 1;
  }

</style>
