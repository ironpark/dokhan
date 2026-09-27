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

<section class="panel" class:without-header={!showTocHeader} class:has-recents={recentItems.length > 0}>
  {#if showTocHeader}
    <div class="section-heading">
      <SectionHeader title="목차" class="toc-title" />
      <span class="item-count" aria-label={`목차 ${items.length}개`}>{items.length}</span>
    </div>
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

  <div class="section-heading recent-heading">
    <SectionHeader title="최근 열람" class="recent-head" />
    {#if recentItems.length}
      <span class="item-count" aria-label={`최근 열람 ${recentItems.length}개`}>{recentItems.length}</span>
    {/if}
  </div>
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
    grid-template-rows: auto minmax(0, 1.35fr) auto minmax(0, 1fr);
    gap: 9px;
    padding: 13px 12px 14px;
    box-sizing: border-box;
  }

  .panel.without-header {
    grid-template-rows: minmax(0, 1.35fr) auto minmax(0, 1fr);
  }

  .panel:not(.has-recents) {
    grid-template-rows: auto minmax(0, 1fr) auto auto;
  }

  .panel.without-header:not(.has-recents) {
    grid-template-rows: minmax(0, 1fr) auto auto;
  }

  .section-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 0 4px;
    min-height: 20px;
  }

  .item-count {
    color: var(--color-text-subtle);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: var(--radius-full);
    background: var(--color-surface-hover);
  }

  :global(.toc-title) {
    padding: 0;
  }

  :global(.recent-head) {
    padding: 0;
  }

  .recent-scroll {
    min-height: 0;
    overflow-y: auto;
    scrollbar-gutter: stable;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface);
  }

  .entry-list {
    min-height: 0;
    overflow-y: auto;
    scrollbar-gutter: stable;
    list-style: none;
    margin: 0;
    padding: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface);
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
    min-height: 58px;
    border: none;
    background: transparent;
    text-align: left;
    padding: 9px 12px;
    display: grid;
    gap: 4px;
    cursor: pointer;
    transition: background-color var(--motion-fast);
  }

  .recent-btn:hover {
    background: var(--color-surface-hover);
  }

  .recent-btn:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -3px;
  }

  .recent-btn small {
    color: var(--color-text-subtle);
    font-size: 11px;
    line-height: 1.25;
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
    font-size: 13px;
    font-weight: 600;
    line-height: 1.35;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .recent-empty {
    border: 1px dashed var(--color-border-strong);
    border-radius: var(--radius-md);
    padding: 4px;
    background: var(--color-surface);
  }

  @media (max-height: 620px) {
    .panel {
      gap: 6px;
      padding: 8px;
    }

    .recent-btn {
      min-height: 48px;
      padding-block: 6px;
    }
  }
</style>
