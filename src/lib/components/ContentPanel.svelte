<script lang="ts">
  import type { ContentItem } from "$lib/types/dictionary";
  import { createVirtualizer } from "@tanstack/svelte-virtual";
  import ListItem from "$lib/components/ui/ListItem.svelte";

  let {
    items,
    selectedLocal = "",
    onOpen,
  }: {
    items: ContentItem[];
    selectedLocal?: string;
    onOpen: (local: string) => void;
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
</script>

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

<style>
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
</style>
