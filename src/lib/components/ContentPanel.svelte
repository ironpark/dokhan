<script lang="ts">
  import type { ContentItem, RecentViewItem } from "$lib/types/dictionary";
  import { createVirtualizer } from "@tanstack/svelte-virtual";
  import ListItem from "$lib/components/ui/ListItem.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";

  let {
    items,
    recents = [],
    selectedLocal = "",
    onOpen,
    onOpenRecent,
  }: {
    items: ContentItem[];
    recents?: RecentViewItem[];
    selectedLocal?: string;
    onOpen: (local: string) => void;
    onOpenRecent: (item: RecentViewItem) => void;
  } = $props();

  let listEl = $state<HTMLElement | null>(null);

  const virtualizer = createVirtualizer({
    count: 0,
    getScrollElement: () => listEl,
    estimateSize: () => 38,
    overscan: 5,
  });

  let lastItemCount = $state(0);
  $effect(() => {
    if (items.length !== lastItemCount) {
      lastItemCount = items.length;
      $virtualizer.scrollToIndex(0);
    }
    $virtualizer.setOptions({
      count: items.length,
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
  const recentItems = $derived(recents.slice(0, 20));
</script>

<section class="panel">
  <h3 class="toc-title">목차</h3>
  <div class="entry-list" bind:this={listEl}>
    <div style="height: {totalSize}px; width: 100%; position: relative;">
      {#each virtualRows as row (row.index)}
        <div
          style="position: absolute; top: 0; left: 0; width: 100%; height: {row.size}px; transform: translateY({row.start}px);"
        >
          {#if items[row.index]}
            <ListItem
              selected={selectedLocal === items[row.index].local}
              onclick={() => onOpen(items[row.index].local)}
            >
              {items[row.index].title}
            </ListItem>
          {/if}
        </div>
      {/each}
    </div>
  </div>

  <div class="recent-box">
    <h3>최근 열람</h3>
    {#if recentItems.length}
      <div class="recent-scroll">
        <ul>
          {#each recentItems as item (item.key)}
            <li>
              <button type="button" class="recent-btn" onclick={() => onOpenRecent(item)}>
                <small>{item.kind === "entry" ? "표제어" : "목차"}</small>
                <span>{item.label}</span>
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {:else}
      <div class="recent-empty">
        <EmptyState title="최근 열람 기록이 없습니다." compact={true} />
      </div>
    {/if}
  </div>
</section>

<style>
  .panel {
    min-height: 0;
    height: 100%;
    display: grid;
    grid-template-rows: auto 1fr minmax(140px, 35%);
    gap: 10px;
    padding: 10px;
    box-sizing: border-box;
  }

  .recent-box {
    min-height: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface);
    display: grid;
    grid-template-rows: auto 1fr;
    overflow: hidden;
  }

  h3 {
    margin: 0;
    padding: 10px 12px 9px;
    font-size: 11px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--color-text-muted);
    border-bottom: 1px solid var(--color-border);
  }

  .toc-title {
    padding: 0 2px;
    border-bottom: none;
  }

  .recent-scroll,
  .entry-list {
    min-height: 0;
    overflow-y: auto;
    scrollbar-gutter: stable;
  }

  .entry-list {
    list-style: none;
    margin: 0;
    padding: 0;
    position: relative;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  li {
    border-bottom: 1px solid var(--color-border);
  }

  li:last-child {
    border-bottom: none;
  }

  .recent-btn {
    width: 100%;
    border: none;
    background: transparent;
    text-align: left;
    padding: 8px 12px;
    display: grid;
    gap: 2px;
    cursor: pointer;
  }

  .recent-btn:hover {
    background: var(--color-surface-hover);
  }

  .recent-btn small {
    color: var(--color-text-subtle);
    font-size: 10px;
  }

  .recent-btn span {
    color: var(--color-text);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .recent-empty {
    padding: 8px;
  }
</style>
