<script lang="ts">
  import type { ContentItem, RecentViewItem } from "$lib/types/dictionary";
  import ListItem from "$lib/components/ui/ListItem.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import SectionHeader from "$lib/components/ui/SectionHeader.svelte";

  let {
    items,
    recents = [],
    selectedLocal = "",
    showTocHeader = true,
    onOpen,
    onOpenRecent,
  }: {
    items: ContentItem[];
    recents?: RecentViewItem[];
    selectedLocal?: string;
    showTocHeader?: boolean;
    onOpen: (local: string) => void;
    onOpenRecent: (item: RecentViewItem) => void;
  } = $props();

  const recentItems = $derived(recents.slice(0, 20));

  const recentDateTimeShortFormatter = new Intl.DateTimeFormat("ko-KR", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
  const recentDateTimeFullFormatter = new Intl.DateTimeFormat("ko-KR", {
    year: "2-digit",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });

  function formatViewedAt(value: number): string {
    if (!Number.isFinite(value) || value <= 0) return "";
    const date = new Date(value);
    const now = new Date();
    const sameYear = date.getFullYear() === now.getFullYear();
    return (sameYear ? recentDateTimeShortFormatter : recentDateTimeFullFormatter).format(date);
  }
</script>

<section class="panel">
  {#if showTocHeader}
    <SectionHeader title="목차" class="toc-title" />
  {/if}
  <div class="entry-list">
    {#if items.length}
      <ul>
        {#each items as item (item.local)}
          <ListItem selected={selectedLocal === item.local} onclick={() => onOpen(item.local)}>
            {item.title}
          </ListItem>
        {/each}
      </ul>
    {:else}
      <EmptyState title="목차 항목이 없습니다." compact={true} />
    {/if}
  </div>

  <SectionHeader title="최근 열람" class="recent-head" />
  {#if recentItems.length}
    <div class="recent-scroll">
      <ul>
        {#each recentItems as item (item.key)}
          <li>
            <button type="button" class="recent-btn" onclick={() => onOpenRecent(item)}>
              <small class="recent-meta">
                <span>{item.kind === "entry" ? "표제어" : "목차"}</span>
                <span>{formatViewedAt(item.viewedAt)}</span>
              </small>
              <span>{item.label}</span>
            </button>
          </li>
        {/each}
      </ul>
    </div>
  {:else}
    <div class="recent-empty">
      <EmptyState title="최근 열람 기록이 없습니다." compact={true} />
    </div>
  {/if}
</section>

<style>
  .panel {
    min-height: 0;
    height: 100%;
    display: grid;
    grid-template-rows: auto auto auto 1fr;
    gap: 10px;
    padding: 10px;
    box-sizing: border-box;
  }

  :global(.toc-title) {
    padding: 0 2px;
  }

  :global(.recent-head) {
    padding: 2px 2px 0;
  }

  .recent-scroll {
    min-height: 0;
    overflow-y: auto;
    scrollbar-gutter: stable;
  }

  .recent-scroll {
    border-top: 1px solid var(--color-border);
  }

  .entry-list {
    min-height: 0;
    overflow: visible;
    list-style: none;
    margin: 0;
    padding: 0;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  li {
    border-bottom: 1px solid var(--color-border);
  }

  li:last-child {
    border-bottom: none;
  }

  .recent-btn {
    width: 100%;
    border: none;
    background: transparent;
    text-align: left;
    padding: 7px 11px;
    display: grid;
    gap: 1px;
    cursor: pointer;
  }

  .recent-btn:hover {
    background: var(--color-surface-hover);
  }

  .recent-btn small {
    color: color-mix(in oklab, var(--color-text-subtle), var(--color-text-muted) 45%);
    font-size: 9px;
    letter-spacing: 0.01em;
  }

  .recent-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .recent-meta span:last-child {
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;
  }

  .recent-btn span {
    color: var(--color-text);
    font-size: 11px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .recent-empty {
    padding: 8px;
  }
</style>
