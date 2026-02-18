<script lang="ts">
  import type { SearchHit } from "$lib/types/dictionary";
  import { createVirtualizer } from "@tanstack/svelte-virtual";
  import Input from "$lib/components/ui/Input.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import ListItem from "$lib/components/ui/ListItem.svelte";

  let {
    query,
    rows,
    loading = false,
    selectedId = null,
    onQueryChange,
    onSubmit,
    onOpen,
  }: {
    query: string;
    rows: SearchHit[];
    loading?: boolean;
    selectedId?: number | null;
    onQueryChange: (value: string) => void;
    onSubmit: (event: Event) => void;
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
</script>

<section class="panel">
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
              <ListItem
                selected={selectedId === rows[row.index].id}
                onclick={() => onOpen(rows[row.index].id)}
              >
                {rows[row.index].headword}
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
    grid-template-columns: 1fr auto;
    gap: 8px;
    align-items: center;
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

  .status-message {
    margin: 0;
    padding: 18px 12px;
    font-size: 13px;
    color: var(--color-text-muted);
  }
</style>
