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

  function escapeRegex(text: string): string {
    return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  }

  function highlightSegments(text: string, rawQuery: string): Segment[] {
    const terms = Array.from(
      new Set(
        rawQuery
          .split(/\s+/)
          .map((term) => term.trim())
          .filter((term) => term.length > 0),
      ),
    );
    if (!terms.length) {
      return [{ text, hit: false }];
    }
    terms.sort((a, b) => b.length - a.length);
    const pattern = new RegExp(`(${terms.map(escapeRegex).join("|")})`, "gi");
    const parts = text.split(pattern);
    return parts
      .filter((part) => part.length > 0)
      .map((part) => ({
        text: part,
        hit: terms.some((term) => part.localeCompare(term, undefined, { sensitivity: "accent" }) === 0)
      }));
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
                {#each highlightSegments(rows[row.index].headword, query) as seg}
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
