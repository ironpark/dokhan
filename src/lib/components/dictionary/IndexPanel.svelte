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
</script>

<section class="nav-panel">
  <div class="panel-top">
    <div class="search-line">
      <input value={query} oninput={(e) => onQueryChange((e.currentTarget as HTMLInputElement).value)} placeholder="접두어 (예: ab, angst)" />
      {#if loading}
        <small class="loading-hint">검색 중…</small>
      {/if}
    </div>
  </div>
  <ul class="entry-list">
    {#each rows as row}
      <li class:selected={selectedId === row.id}>
        <button type="button" onclick={() => onOpen(row.id)}>{row.headword}</button>
      </li>
    {/each}
  </ul>
</section>

<style>
  .nav-panel {
    min-height: 0;
    overflow: hidden;
    padding: 10px;
    display: grid;
    grid-template-rows: 35px 1fr;
    gap: 8px;
  }

  .panel-top {
    height: 35px;
    display: grid;
    align-items: center;
  }

  .search-line {
    margin: 0;
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
    padding: 0;
    list-style: none;
    min-height: 0;
    overflow-y: auto;
    scrollbar-gutter: stable;
  }

  .entry-list li {
    border-bottom: 1px solid var(--line);
  }

  .entry-list li:hover {
    background: #f6f2e8;
  }

  .entry-list li.selected {
    background: #ece5d6;
  }

  .entry-list li button {
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

  .entry-list li:hover button {
    color: #15120d;
  }

  .entry-list li.selected button {
    color: #0d4f40;
  }

</style>
