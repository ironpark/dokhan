<script lang="ts">
  import type { DictionaryIndexEntry } from "$lib/types/dictionary";
  import { createVirtualizer } from "@tanstack/svelte-virtual";
  import Input from "$lib/components/ui/Input.svelte";
  import ListItem from "$lib/components/ui/ListItem.svelte";

  let {
    query,
    rows,
    loading = false,
    selectedId = null,
    onQueryChange,
    onOpen,
  }: {
    query: string;
    rows: DictionaryIndexEntry[];
    loading?: boolean;
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

  let lastRowCount = $state(0);
  $effect(() => {
    if (rows.length !== lastRowCount) {
      lastRowCount = rows.length;
      $virtualizer.scrollToIndex(0);
    }
    $virtualizer.setOptions({
      count: rows.length,
      getScrollElement: () => listEl,
    });
    requestAnimationFrame(() => {
      $virtualizer.measure();
    });
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
</script>

<section class="panel">
  <div class="search-line">
    <Input
      value={query}
      oninput={(e) => onQueryChange((e.target as HTMLInputElement).value)}
      placeholder="색인 fuzzy 검색 (예: hnd, ab)"
    />
  </div>
  <div class="entry-list" bind:this={listEl}>
    {#if loading}
      <p class="status-message">색인을 불러오는 중입니다.</p>
    {:else if !rows.length && query.trim()}
      <p class="status-message">일치하는 색인 항목이 없습니다.</p>
    {:else if !rows.length}
      <p class="status-message">색인 데이터가 없습니다.</p>
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
                {#each splitByHighlights(rows[row.index].headword, rows[row.index].headwordHighlights) as seg}
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

  .search-line {
    margin: 0;
    padding: 10px;
    display: grid;
    grid-template-columns: 1fr;
    gap: 8px;
    align-items: center;
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

  .status-message {
    margin: 0;
    padding: 18px 12px;
    font-size: 13px;
    color: var(--color-text-muted);
  }
</style>
