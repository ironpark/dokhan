<script lang="ts">
  import type { SearchHit } from "$lib/types/dictionary";
  import { createVirtualizer } from "@tanstack/svelte-virtual";
  import Input from "$lib/components/ui/Input.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";

  let {
    query,
    rows,
    loading = false,
    inputAtBottom = false,
    recentUnderInput = false,
    recentSearches = [],
    selectedId = null,
    onQueryChange,
    onSubmit,
    onPickRecentSearch,
    onOpen,
  }: {
    query: string;
    rows: SearchHit[];
    loading?: boolean;
    inputAtBottom?: boolean;
    recentUnderInput?: boolean;
    recentSearches?: string[];
    selectedId?: number | null;
    onQueryChange: (value: string) => void;
    onSubmit: (event: Event) => void;
    onPickRecentSearch: (query: string) => void;
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
  const showRecentInline = $derived(
    recentUnderInput && recentSearches.length > 0 && !query.trim(),
  );
  const showRecentInList = $derived(
    !recentUnderInput && recentSearches.length > 0 && !query.trim(),
  );
</script>

<section
  class="panel"
  class:input-bottom={inputAtBottom}
>
  <div class="search-group">
    <form class="search-line" onsubmit={onSubmit}>
    <Input
      value={query}
      oninput={(e) => onQueryChange((e.target as HTMLInputElement).value)}
      onclear={() => onQueryChange("")}
      clearable={true}
      placeholder="독일어/한국어 검색"
    />
      <Button type="submit" disabled={loading || !query.trim()}>검색</Button>
    </form>
    {#if showRecentInline}
      <div class="search-recent">
        <p>최근 검색어</p>
        <div class="recent-list">
          {#each recentSearches as term (term)}
            <button type="button" onclick={() => onPickRecentSearch(term)}>
              {term}
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>
  <div class="entry-list" bind:this={listEl}>
    {#if loading}
      <EmptyState title="검색 결과를 불러오는 중입니다." compact={true} />
    {:else if !rows.length && query.trim()}
      <EmptyState
        title="검색 결과가 없습니다."
        description="다른 검색어 또는 더 짧은 키워드로 시도해 보세요."
        compact={true}
      />
    {:else if !rows.length}
      {#if showRecentInList}
        <div class="recent-block">
          <p class="recent-title">최근 검색어</p>
          <div class="recent-list">
            {#each recentSearches as term (term)}
              <button type="button" onclick={() => onPickRecentSearch(term)}>
                {term}
              </button>
            {/each}
          </div>
        </div>
      {:else}
        <EmptyState
          title="검색어를 입력해 주세요."
          description="검색 버튼을 누르거나 엔터로 결과를 확인할 수 있습니다."
          compact={true}
        />
      {/if}
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

  .search-group {
    background: transparent;
    border-bottom: 1px solid var(--color-border);
  }

  .search-line {
    margin: 0;
    padding: 10px 12px;
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px;
    align-items: center;
  }

  .panel.input-bottom .search-line {
    border-top: none;
    background: transparent;
    padding-bottom: 10px;
  }

  .panel.input-bottom .search-group {
    order: 2;
    border-bottom: none;
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

  .search-recent {
    padding: 6px 12px 10px;
    border-top: none;
    background: transparent;
  }

  .search-recent p {
    margin: 0 0 6px;
    font-size: 11px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .panel.input-bottom .search-recent {
    border-top: none;
  }

  .recent-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .recent-block {
    padding: 10px 12px;
  }

  .recent-title {
    margin: 0 0 6px;
    font-size: 11px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .recent-list button {
    border: 1px solid var(--color-border);
    border-radius: 999px;
    background: color-mix(in oklab, var(--color-surface), #f7faff 20%);
    color: var(--color-text);
    padding: 5px 10px;
    font-size: 12px;
    cursor: pointer;
    transition:
      background-color var(--motion-fast),
      border-color var(--motion-fast);
  }

  .recent-list button:hover {
    border-color: var(--color-border-strong);
    background: var(--color-surface-hover);
  }

  .result-row {
    width: 100%;
    height: 100%;
    border: none;
    border-bottom: 1px solid var(--color-border);
    background: transparent;
    text-align: left;
    padding: 9px 12px 8px;
    display: grid;
    gap: 3px;
    box-sizing: border-box;
    cursor: pointer;
    transition:
      background-color var(--motion-fast),
      box-shadow var(--motion-fast);
  }

  .result-row:hover {
    background: var(--color-surface-hover);
  }

  .result-row.selected {
    background: var(--color-accent-soft);
    box-shadow: inset 3px 0 0 var(--color-accent);
  }

  .result-row strong {
    font-size: 14px;
    color: var(--color-text);
    font-weight: 600;
    line-height: 1.3;
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
    color: var(--color-text-subtle);
    line-height: 1.3;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
