<script lang="ts">
  import type { ContentItem } from '$lib/types/dictionary';

  let {
    items,
    onOpen
  }: {
    items: ContentItem[];
    onOpen: (local: string) => void;
  } = $props();
</script>

<section class="nav-panel">
  <div class="panel-top" aria-hidden="true"></div>
  <ul class="entry-list">
    {#each items as item}
      <li>
        <button type="button" onclick={() => onOpen(item.local)}>{item.title}</button>
        <small>{item.local}</small>
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
    display: grid;
    gap: 3px;
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

  .entry-list li small {
    color: var(--muted);
    font-size: 12px;
    word-break: break-word;
  }
</style>
