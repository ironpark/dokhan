<script lang="ts">
  import type { ContentItem } from "$lib/types/dictionary";
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

  const rowHeight = 38;
  const overscan = 8;

  let listEl = $state<HTMLElement | null>(null);
  let scrollTop = $state(0);
  let viewportHeight = $state(0);

  const totalCount = $derived(items.length);
  const visibleCount = $derived(
    Math.ceil(viewportHeight / rowHeight) + overscan * 2,
  );
  const startIndex = $derived.by(() => {
    const raw = Math.max(0, Math.floor(scrollTop / rowHeight) - overscan);
    const maxStart = Math.max(0, totalCount - visibleCount);
    return Math.min(raw, maxStart);
  });
  const endIndex = $derived(Math.min(totalCount, startIndex + visibleCount));
  const topSpacer = $derived(startIndex * rowHeight);
  const bottomSpacer = $derived((totalCount - endIndex) * rowHeight);
  const visibleItems = $derived(items.slice(startIndex, endIndex));

  function handleScroll() {
    if (!listEl) return;
    scrollTop = listEl.scrollTop;
    viewportHeight = listEl.clientHeight;
  }

  $effect(() => {
    viewportHeight = listEl?.clientHeight ?? 0;
  });
</script>

<ul class="entry-list" bind:this={listEl} onscroll={handleScroll}>
  {#if topSpacer > 0}
    <li class="spacer" style={`height:${topSpacer}px`} aria-hidden="true"></li>
  {/if}
  {#each visibleItems as item}
    <ListItem
      selected={selectedLocal === item.local}
      onclick={() => onOpen(item.local)}
    >
      {item.title}
    </ListItem>
  {/each}
  {#if bottomSpacer > 0}
    <li
      class="spacer"
      style={`height:${bottomSpacer}px`}
      aria-hidden="true"
    ></li>
  {/if}
</ul>

<style>
  .entry-list {
    margin: 0;
    padding: 10px;
    list-style: none;
    min-height: 0;
    height: 100%;
    box-sizing: border-box;
    overflow-y: auto;
    scrollbar-gutter: stable;
  }

  .spacer {
    border: 0;
    padding: 0;
    margin: 0;
    pointer-events: none;
  }
</style>
