<script lang="ts">
  import type { SearchHit } from "$lib/types/dictionary";
  import Input from "$lib/components/ui/Input.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import ListItem from "$lib/components/ui/ListItem.svelte";

  let {
    query,
    rows,
    loading = false,
    selectedId = null,
    onQueryChange,
    onSubmit,
    onOpen,
  }: {
    query: string;
    rows: SearchHit[];
    loading?: boolean;
    selectedId?: number | null;
    onQueryChange: (value: string) => void;
    onSubmit: (event: Event) => void;
    onOpen: (id: number) => void;
  } = $props();

  const rowHeight = 38;
  const overscan = 6;

  let listEl = $state<HTMLElement | null>(null);
  let scrollTop = $state(0);
  let viewportHeight = $state(0);

  const totalCount = $derived(rows.length);
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
  <form class="search-line" onsubmit={onSubmit}>
    <Input
      value={query}
      oninput={(e) => onQueryChange((e.target as HTMLInputElement).value)}
      placeholder="독일어/한국어 검색"
    />
    <Button type="submit" disabled={loading}>검색</Button>
  </form>
  <ul class="entry-list" bind:this={listEl} onscroll={handleScroll}>
    {#if topSpacer > 0}
      <li
        class="spacer"
        style={`height:${topSpacer}px`}
        aria-hidden="true"
      ></li>
    {/if}
    {#each visibleRows as row}
      <ListItem selected={selectedId === row.id} onclick={() => onOpen(row.id)}>
        {row.headword}
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
</section>

<style>
  .panel {
    min-height: 0;
    height: 100%;
    display: grid;
    grid-template-rows: auto 1fr;
  }

  .search-line {
    margin: 0;
    padding: 10px;
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px;
    align-items: center;
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

  .spacer {
    border: 0;
    padding: 0;
    margin: 0;
    pointer-events: none;
  }
</style>
