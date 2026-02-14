<script lang="ts">
  import type { DictionaryIndexEntry } from '$lib/types/dictionary';

  let {
    query,
    rows,
    loading = false,
    selectedId = null,
    onQueryChange,
    onOpen
  }: {
    query: string;
    rows: DictionaryIndexEntry[];
    loading?: boolean;
    selectedId?: number | null;
    onQueryChange: (value: string) => void;
    onOpen: (id: number) => void;
  } = $props();

  const rowHeight = 38;
  const overscan = 8;

  let listEl = $state<HTMLElement | null>(null);
  let scrollTop = $state(0);
  let viewportHeight = $state(0);

  const totalCount = $derived(rows.length);
  const visibleCount = $derived(Math.ceil(viewportHeight / rowHeight) + overscan * 2);
  const startIndex = $derived.by(() => {
    const raw = Math.max(0, Math.floor(scrollTop / rowHeight) - overscan);
    const maxStart = Math.max(0, totalCount - visibleCount);
    return Math.min(raw, maxStart);
  });
  const endIndex = $derived(Math.min(totalCount, startIndex + visibleCount));
  const topSpacer = $derived(startIndex * rowHeight);
  const bottomSpacer = $derived((totalCount - endIndex) * rowHeight);
  const visibleRows = $derived(rows.slice(startIndex, endIndex));

  function handleScroll() {
    if (!listEl) return;
    scrollTop = listEl.scrollTop;
    viewportHeight = listEl.clientHeight;
  }

  $effect(() => {
    viewportHeight = listEl?.clientHeight ?? 0;
  });
</script>

<section class="panel">
  <div class="panel-controls">
    <input value={query} oninput={(e) => onQueryChange((e.currentTarget as HTMLInputElement).value)} placeholder="접두어 (예: ab, angst)" />
    {#if loading}
      <small class="loading-hint">검색 중…</small>
    {/if}
  </div>
  <ul class="entry-list" bind:this={listEl} onscroll={handleScroll}>
    {#if topSpacer > 0}
      <li class="spacer" style={`height:${topSpacer}px`} aria-hidden="true"></li>
    {/if}
    {#each visibleRows as row}
      <li class="row" class:selected={selectedId === row.id}>
        <button type="button" onclick={() => onOpen(row.id)}>{row.headword}</button>
      </li>
    {/each}
    {#if bottomSpacer > 0}
      <li class="spacer" style={`height:${bottomSpacer}px`} aria-hidden="true"></li>
    {/if}
  </ul>
</section>

<style>
  .panel {
    min-height: 0;
    height: 100%;
    display: grid;
    grid-template-rows: auto 1fr;
  }

  .panel-controls {
    margin: 0;
    padding: 10px 10px 8px;
    display: grid;
    gap: 4px;
    align-items: center;
  }

  input {
    border: 1px solid var(--line);
    border-radius: 0;
    padding: 7px 9px;
    font-size: 13px;
    font-family: inherit;
    height: 32px;
    box-sizing: border-box;
  }

  .loading-hint {
    color: var(--muted);
    font-size: 12px;
    line-height: 1;
  }

  .entry-list {
    margin: 0;
    padding: 0 10px 10px;
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
