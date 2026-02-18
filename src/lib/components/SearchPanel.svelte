<script lang="ts">
  import type { SearchHit } from "$lib/types/dictionary";
  import { createVirtualizer } from "@tanstack/svelte-virtual";
  import Input from "$lib/components/ui/Input.svelte";
  import Button from "$lib/components/ui/Button.svelte";

  let {
    query,
    rows,
    loading = false,
    inputAtBottom = false,
    selectedId = null,
    onQueryChange,
    onSubmit,
    onOpen,
  }: {
    query: string;
    rows: SearchHit[];
    loading?: boolean;
    inputAtBottom?: boolean;
    selectedId?: number | null;
    onQueryChange: (value: string) => void;
    onSubmit: (event: Event) => void;
    onOpen: (id: number) => void;
  } = $props();

  let listEl = $state<HTMLElement | null>(null);

  const virtualizer = createVirtualizer({
    count: 0,
    getScrollElement: () => listEl,
    estimateSize: () => 56,
    overscan: 5,
  });

  function normalizeSnippet(snippet: string): string {
    return snippet
      .replace(/<[^>]*>/g, " ")
      .replace(/\s+/g, " ")
      .trim();
  }

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
</script>

<section class="panel" class:input-bottom={inputAtBottom}>
  <form class="search-line" onsubmit={onSubmit}>
    <Input
      value={query}
      oninput={(e) => onQueryChange((e.target as HTMLInputElement).value)}
      placeholder="독일어/한국어 검색"
    />
    <Button type="submit" disabled={loading || !query.trim()}>검색</Button>
  </form>
  <div class="entry-list" bind:this={listEl}>
    {#if loading}
      <p class="status-message">검색 결과를 불러오는 중입니다.</p>
    {:else if !rows.length && query.trim()}
      <p class="status-message">검색 결과가 없습니다.</p>
    {:else if !rows.length}
      <p class="status-message">검색어를 입력하고 검색 버튼을 눌러주세요.</p>
    {:else}
      <div style="height: {totalSize}px; width: 100%; position: relative;">
        {#each virtualRows as row (row.index)}
          <div
            style="position: absolute; top: 0; left: 0; width: 100%; height: {row.size}px; transform: translateY({row.start}px);"
          >
            {#if rows[row.index]}
              <button
                type="button"
                class="result-row"
                class:selected={selectedId === rows[row.index].id}
                onclick={() => onOpen(rows[row.index].id)}
              >
                <strong>{rows[row.index].headword}</strong>
                {#if rows[row.index].snippet}
                  <small>{normalizeSnippet(rows[row.index].snippet)}</small>
                {/if}
              </button>
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
    padding: 10px;
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px;
    align-items: center;
  }

  .panel.input-bottom .search-line {
    order: 2;
    border-top: 1px solid var(--color-border);
    background: color-mix(in oklab, var(--color-surface), white 12%);
    padding-bottom: calc(10px + env(safe-area-inset-bottom));
  }

  .entry-list {
    margin: 0;
    padding: 0;
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

  .status-message {
    margin: 0;
    padding: 18px 12px;
    font-size: 13px;
    color: var(--color-text-muted);
  }

  .result-row {
    width: 100%;
    height: 100%;
    border: none;
    border-bottom: 1px solid var(--color-border);
    background: transparent;
    text-align: left;
    padding: 8px 10px;
    display: grid;
    gap: 3px;
    box-sizing: border-box;
    cursor: pointer;
  }

  .result-row:hover {
    background: var(--color-surface-hover);
  }

  .result-row.selected {
    background: var(--color-surface-active);
  }

  .result-row strong {
    font-size: 14px;
    color: var(--color-text);
    font-weight: 600;
    line-height: 1.25;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .result-row.selected strong {
    color: var(--color-accent);
  }

  .result-row small {
    margin: 0;
    font-size: 12px;
    color: var(--color-text-muted);
    line-height: 1.2;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
