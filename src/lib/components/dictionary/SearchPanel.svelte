<script lang="ts">
  import type { SearchHit } from '$lib/types/dictionary';

  let {
    query,
    rows,
    loading = false,
    selectedId = null,
    onQueryChange,
    onSubmit,
    onOpen
  }: {
    query: string;
    rows: SearchHit[];
    loading?: boolean;
    selectedId?: number | null;
    onQueryChange: (value: string) => void;
    onSubmit: (event: Event) => void;
    onOpen: (id: number) => void;
  } = $props();
</script>

<section class="nav-panel">
  <div class="panel-top">
    <form class="search-line" onsubmit={onSubmit}>
      <input value={query} oninput={(e) => onQueryChange((e.currentTarget as HTMLInputElement).value)} placeholder="독일어/한국어 검색" />
      <button type="submit" disabled={loading}>검색</button>
    </form>
  </div>
  <ul class="entry-list">
    {#each rows as row}
      <li class:selected={selectedId === row.id}>
        <button type="button" onclick={() => onOpen(row.id)}>{row.headword}</button>
        <small>score {row.score} · {row.snippet}</small>
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
    border-radius: 0;
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
    min-height: 0;
    overflow-y: auto;
    scrollbar-gutter: stable;
  }

  .entry-list li {
    border-bottom: 1px solid var(--line);
    padding: 8px 2px;
    display: grid;
    gap: 3px;
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
    padding: 0;
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

  .entry-list li small {
    color: var(--muted);
    font-size: 12px;
    word-break: break-word;
  }

  @media (max-width: 640px) {
    .search-line {
      grid-template-columns: 1fr;
    }
  }
</style>
