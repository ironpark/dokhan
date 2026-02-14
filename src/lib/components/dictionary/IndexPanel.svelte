<script lang="ts">
  import type { DictionaryIndexEntry } from '$lib/types/dictionary';

  let {
    query,
    rows,
    loading = false,
    onQueryChange,
    onSubmit,
    onOpen
  }: {
    query: string;
    rows: DictionaryIndexEntry[];
    loading?: boolean;
    onQueryChange: (value: string) => void;
    onSubmit: (event: Event) => void;
    onOpen: (id: number) => void;
  } = $props();
</script>

<section class="nav-panel">
  <div class="panel-top">
    <form class="search-line" onsubmit={onSubmit}>
      <input value={query} oninput={(e) => onQueryChange((e.currentTarget as HTMLInputElement).value)} placeholder="접두어 (예: ab, angst)" />
      <button type="submit" disabled={loading}>조회</button>
    </form>
  </div>
  <ul class="entry-list">
    {#each rows as row}
      <li>
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
    grid-template-columns: 1fr auto;
    gap: 8px;
    align-items: center;
  }

  input,
  button {
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
    padding: 7px 9px;
    font-size: 13px;
    font-family: inherit;
    height: 32px;
    box-sizing: border-box;
  }

  button {
    cursor: pointer;
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }

  button:disabled {
    opacity: 0.6;
    cursor: default;
  }

  .entry-list {
    margin: 0;
    padding: 0;
    list-style: none;
    display: grid;
    gap: 6px;
    min-height: 0;
    overflow-y: auto;
    scrollbar-gutter: stable;
    align-content: start;
  }

  .entry-list li {
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
    background: #fff;
    padding: 7px 8px;
  }

  .entry-list li button {
    border: 0;
    background: transparent;
    color: var(--text);
    padding: 0;
    text-align: left;
    font-weight: 700;
    cursor: pointer;
  }

  @media (max-width: 640px) {
    .search-line {
      grid-template-columns: 1fr;
    }
  }
</style>
