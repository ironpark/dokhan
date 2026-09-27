<script lang="ts">
  import { onMount } from "svelte";
  import type { ContentItem, RecentViewItem } from "$lib/types/dictionary";
  import ListItem from "$lib/components/ui/ListItem.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import SectionHeader from "$lib/components/ui/SectionHeader.svelte";

  let {
    items,
    recents = [],
    selectedLocal = "",
    onOpen,
    onOpenRecent,
  }: {
    items: ContentItem[];
    recents?: RecentViewItem[];
    selectedLocal?: string;
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

  // --- Resizable split between the TOC (top) and recent views (bottom) ---
  // `split` is the TOC's share of the space both lists compete for. It is a
  // per-viewer layout preference, so localStorage is enough (failures ignored).
  const SPLIT_STORAGE_KEY = "dokhan:toc-recent-split";
  const MIN_TOC_PX = 76;
  const MIN_RECENT_PX = 110;
  const KEY_STEP = 0.05;

  let tocEl = $state<HTMLElement | null>(null);
  let recentEl = $state<HTMLElement | null>(null);
  let split = $state(0.55);
  let dragging = $state(false);

  function readStoredSplit(): number | null {
    try {
      const raw = window.localStorage.getItem(SPLIT_STORAGE_KEY);
      const value = raw === null ? NaN : Number(raw);
      return Number.isFinite(value) ? value : null;
    } catch {
      return null;
    }
  }

  function storeSplit(value: number | null) {
    try {
      if (value === null) window.localStorage.removeItem(SPLIT_STORAGE_KEY);
      else window.localStorage.setItem(SPLIT_STORAGE_KEY, value.toFixed(3));
    } catch {
      // Storage unavailable (private mode etc.): the split just isn't remembered.
    }
  }

  /** Pixel height the TOC and the recent section share (everything but the handle). */
  function sharedHeight(): number {
    if (!tocEl || !recentEl) return 0;
    return tocEl.offsetHeight + recentEl.offsetHeight;
  }

  function clampSplit(value: number): number {
    const total = sharedHeight();
    if (total <= 0) return Math.min(0.85, Math.max(0.15, value));
    const min = Math.min(0.5, MIN_TOC_PX / total);
    const max = Math.max(0.5, 1 - MIN_RECENT_PX / total);
    return Math.min(max, Math.max(min, value));
  }

  /** Default: give the TOC just enough room for its items (within limits). */
  function fitSplit(): number {
    const total = sharedHeight();
    const listHeight = tocEl?.querySelector("ul")?.scrollHeight ?? 0;
    if (total <= 0 || listHeight <= 0) return 0.55;
    return clampSplit(Math.min(0.65, (listHeight + 4) / total));
  }

  onMount(() => {
    const stored = readStoredSplit();
    split = stored === null ? fitSplit() : clampSplit(stored);
  });

  function splitFromPointer(clientY: number): number {
    if (!tocEl) return split;
    const top = tocEl.getBoundingClientRect().top;
    return clampSplit((clientY - top) / sharedHeight());
  }

  function onHandlePointerDown(event: PointerEvent) {
    if (event.button !== 0) return;
    event.preventDefault();
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    dragging = true;
  }

  function onHandlePointerMove(event: PointerEvent) {
    if (!dragging) return;
    split = splitFromPointer(event.clientY);
  }

  function onHandlePointerUp(event: PointerEvent) {
    if (!dragging) return;
    dragging = false;
    (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
    storeSplit(split);
  }

  function onHandleKeydown(event: KeyboardEvent) {
    let next: number | null = null;
    if (event.key === "ArrowUp") next = split - KEY_STEP;
    else if (event.key === "ArrowDown") next = split + KEY_STEP;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = 1;
    else if (event.key === "Enter") next = fitSplit();
    if (next === null) return;
    event.preventDefault();
    split = clampSplit(next);
    storeSplit(split);
  }

  function resetSplit() {
    storeSplit(null);
    split = fitSplit();
  }
</script>

<section class="panel" class:dragging style={`--toc-share: ${split}; --recent-share: ${1 - split};`}>
  <!-- The active tab already names this list; no separate "목차" heading. -->
  <div class="entry-list" aria-label="목차" bind:this={tocEl}>
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

  <!-- A focusable separator is the WAI-ARIA "window splitter" pattern, so it is interactive. -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <div
    class="split-handle"
    role="separator"
    aria-orientation="horizontal"
    aria-label="목차와 최근 열람 영역 크기 조절"
    aria-valuemin={0}
    aria-valuemax={100}
    aria-valuenow={Math.round(split * 100)}
    tabindex="0"
    title="드래그해서 크기 조절 · 더블클릭으로 초기화"
    onpointerdown={onHandlePointerDown}
    onpointermove={onHandlePointerMove}
    onpointerup={onHandlePointerUp}
    onpointercancel={onHandlePointerUp}
    ondblclick={resetSplit}
    onkeydown={onHandleKeydown}
  >
    <span class="grip" aria-hidden="true"></span>
  </div>

  <div class="recent-section" bind:this={recentEl}>
  <div class="section-heading recent-heading">
    <SectionHeader title="최근 열람" />
    {#if recentItems.length}
      <span class="item-count" aria-label={`최근 열람 ${recentItems.length}개`}>{recentItems.length}</span>
    {/if}
  </div>
  <div class="recent-scroll">
    {#if recentItems.length}
      <ul>
        {#each recentItems as item (item.key)}
          <li>
            <button type="button" class="recent-btn" onclick={() => onOpenRecent(item)}>
              <span class="recent-label" class:headword={item.kind === "entry"}>{item.label}</span>
              <small class="recent-meta">
                <span>{item.kind === "entry" ? "표제어" : "목차"}</span>
                <span aria-hidden="true">·</span>
                <span>{formatViewedAt(item.viewedAt)}</span>
              </small>
            </button>
          </li>
        {/each}
      </ul>
    {:else}
      <p class="recent-empty">열어 본 항목이 여기에 표시됩니다.</p>
    {/if}
  </div>
  </div>
</section>

<style>
  .panel {
    min-height: 0;
    height: 100%;
    display: flex;
    flex-direction: column;
    padding: 2px 0 10px;
    box-sizing: border-box;
  }

  .panel.dragging {
    cursor: row-resize;
    user-select: none;
  }

  /* Both lists share the leftover height by the user-controlled ratio. */
  .entry-list {
    flex: var(--toc-share) 1 0px;
  }

  /* Recent views sit on a slightly recessed sheet so the two areas read as
     separate sections, not one continuous list. */
  .recent-section {
    flex: var(--recent-share) 1 0px;
    min-height: 0;
    display: flex;
    flex-direction: column;
    margin: 0 8px;
    border-radius: var(--radius-md);
    background: color-mix(in oklab, var(--color-surface-active), transparent 55%);
  }

  .recent-scroll {
    flex: 1 1 0px;
    padding-bottom: 4px;
  }

  .entry-list,
  .recent-scroll {
    min-height: 0;
    overflow-y: auto;
  }

  /* The handle doubles as the section divider: a hairline with a centered grip. */
  .split-handle {
    position: relative;
    flex: none;
    height: 16px;
    margin: 2px 0;
    cursor: row-resize;
    touch-action: none;
    outline: none;
  }

  .split-handle::before {
    content: "";
    position: absolute;
    left: 16px;
    right: 16px;
    top: 50%;
    border-top: 1px solid var(--color-divider);
    transition: border-color var(--motion-fast);
  }

  .grip {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 32px;
    height: 5px;
    transform: translate(-50%, -50%);
    border-radius: var(--radius-full);
    background: var(--color-surface-soft);
    box-shadow: inset 0 0 0 1px var(--color-border-strong);
    transition:
      background-color var(--motion-fast),
      box-shadow var(--motion-fast),
      width var(--motion-fast);
  }

  .split-handle:hover::before,
  .dragging .split-handle::before {
    border-color: var(--color-accent-border);
  }

  .split-handle:hover .grip,
  .dragging .grip,
  .split-handle:focus-visible .grip {
    width: 40px;
    background: var(--color-accent);
    box-shadow: none;
  }

  .split-handle:focus-visible::before {
    border-color: var(--color-focus-ring);
  }

  .section-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 10px 14px 4px;
    min-height: 20px;
  }

  .item-count {
    color: var(--color-text-subtle);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    font-weight: 600;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .recent-scroll li {
    padding: 1px 4px;
  }

  .recent-btn {
    width: 100%;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    text-align: left;
    padding: 7px 10px;
    display: grid;
    gap: 2px;
    cursor: pointer;
    transition: background-color var(--motion-fast);
  }

  .recent-btn:hover {
    background: var(--color-surface);
  }

  .recent-btn:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: -2px;
  }

  .recent-label {
    color: var(--color-text);
    font-size: 13.5px;
    font-weight: 550;
    line-height: 1.35;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .recent-label.headword {
    font-family: var(--font-serif);
    font-size: 14.5px;
  }

  .recent-meta {
    display: flex;
    gap: 5px;
    color: var(--color-text-subtle);
    font-size: 11px;
    line-height: 1.3;
    font-variant-numeric: tabular-nums;
  }

  .recent-empty {
    margin: 0;
    padding: 2px 14px 10px;
    color: var(--color-text-subtle);
    font-size: 12px;
  }

  @media (max-height: 620px) {
    .recent-btn {
      padding-block: 5px;
    }
  }
</style>
