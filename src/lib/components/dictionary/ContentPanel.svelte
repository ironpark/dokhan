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
</script>

<section class="nav-panel">
  <div class="panel-top" aria-hidden="true"></div>
  <ul class="entry-list">
    {#each items as item}
      <li class:selected={selectedLocal === item.local}>
        <button type="button" onclick={() => onOpen(item.local)}>{item.title}</button>
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
