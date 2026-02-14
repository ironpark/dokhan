<script lang="ts">
  import type { ContentItem } from '$lib/types/dictionary';

  let {
    items,
    selectedLocal = '',
    onOpen
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
  const visibleCount = $derived(Math.ceil(viewportHeight / rowHeight) + overscan * 2);
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
    <li class="row" class:selected={selectedLocal === item.local}>
      <button type="button" onclick={() => onOpen(item.local)}>{item.title}</button>
    </li>
  {/each}
  {#if bottomSpacer > 0}
    <li class="spacer" style={`height:${bottomSpacer}px`} aria-hidden="true"></li>
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

  .entry-list li.row {
    border-bottom: 1px solid var(--line);
    height: 38px;
    box-sizing: border-box;
  }

  .entry-list li.row:hover {
    background: #f6f2e8;
  }

  .entry-list li.row.selected {
    background: #ece5d6;
  }

  .entry-list li.row button {
    border: 0;
    background: transparent;
    color: var(--text);
    width: 100%;
    display: block;
    padding: 9px 2px;
    text-align: left;
    font-weight: 600;
    cursor: pointer;
    transition: color 100ms ease;
  }

  .entry-list li.row:hover button {
    color: #15120d;
  }

  .entry-list li.row.selected button {
    color: #0d4f40;
  }

  .entry-list li.spacer {
    border: 0;
    padding: 0;
    margin: 0;
    pointer-events: none;
  }
</style>
