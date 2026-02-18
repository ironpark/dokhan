<script lang="ts">
  import type { FavoriteItem, RecentViewItem } from "$lib/types/dictionary";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";

  let {
    favorites,
    recents,
    onOpenFavorite,
    onOpenRecent,
    onRemoveFavorite,
  }: {
    favorites: FavoriteItem[];
    recents: RecentViewItem[];
    onOpenFavorite: (item: FavoriteItem) => void;
    onOpenRecent: (item: RecentViewItem) => void;
    onRemoveFavorite: (key: string) => void;
  } = $props();

  const favoriteItems = $derived(favorites.slice(0, 100));
  const recentItems = $derived(recents.slice(0, 20));
</script>

<section class="panel">
  <div class="section favorites-section">
    <h3>즐겨찾기</h3>
    {#if favoriteItems.length}
      <div class="section-body favorites-scroll">
        <ul>
          {#each favoriteItems as item (item.key)}
            <li>
              <button type="button" class="item-btn" onclick={() => onOpenFavorite(item)}>
                <small>{item.kind === "entry" ? "표제어" : "목차"}</small>
                <span>{item.label}</span>
              </button>
              <button
                type="button"
                class="remove-btn"
                aria-label="즐겨찾기 삭제"
                onclick={() => onRemoveFavorite(item.key)}
              >
                제거
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {:else}
      <EmptyState
        title="아직 저장된 즐겨찾기가 없습니다."
        description="본문에서 '저장' 버튼을 눌러 빠르게 다시 열 수 있습니다."
        compact={true}
      />
    {/if}
  </div>

  <div class="section recent-section">
    <h3>최근 열람</h3>
    <div class="section-body recent-scroll">
      {#if recentItems.length}
        <ul>
          {#each recentItems as item (item.key)}
            <li>
              <button type="button" class="item-btn" onclick={() => onOpenRecent(item)}>
                <small>{item.kind === "entry" ? "표제어" : "목차"}</small>
                <span>{item.label}</span>
              </button>
            </li>
          {/each}
        </ul>
      {:else}
        <EmptyState title="최근 열람 기록이 없습니다." compact={true} />
      {/if}
    </div>
  </div>
</section>

<style>
  .panel {
    height: 100%;
    min-height: 0;
    overflow: hidden;
    box-sizing: border-box;
    padding: 10px;
    padding-bottom: 10px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .section {
    flex: 0 0 auto;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
    box-shadow: var(--shadow-sm);
  }

  .section-body {
    min-height: 0;
  }

  .favorites-section {
    flex: 0 0 auto;
    min-height: 130px;
    max-height: 46%;
    display: grid;
    grid-template-rows: auto 1fr;
  }

  .favorites-scroll {
    min-height: 0;
    overflow-y: auto;
    -webkit-overflow-scrolling: touch;
    scrollbar-gutter: stable;
    scrollbar-width: thin;
    scrollbar-color: #b8bfcc transparent;
  }

  .favorites-scroll::-webkit-scrollbar {
    width: 8px;
  }

  .favorites-scroll::-webkit-scrollbar-thumb {
    background: #b8bfcc;
    border-radius: 999px;
  }

  .favorites-scroll::-webkit-scrollbar-track {
    background: transparent;
  }

  .section h3 {
    margin: 0;
    padding: 10px 12px 9px;
    font-size: 11px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--color-text-muted);
    border-bottom: 1px solid var(--color-border);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  li {
    display: grid;
    grid-template-columns: 1fr auto;
    border-bottom: 1px solid var(--color-border);
    min-height: 48px;
    transition: background-color var(--motion-fast);
  }

  li:last-child {
    border-bottom: none;
  }

  li:hover {
    background: var(--color-surface-hover);
  }

  .item-btn {
    border: none;
    background: transparent;
    text-align: left;
    padding: 9px 12px;
    display: grid;
    gap: 2px;
    cursor: pointer;
  }

  .item-btn small {
    color: var(--color-text-subtle);
    font-size: 10px;
  }

  .item-btn span {
    color: var(--color-text);
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .remove-btn {
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 12px;
    padding: 0 12px;
    cursor: pointer;
    transition: color var(--motion-fast);
  }

  .remove-btn:hover {
    color: var(--color-danger);
  }

  .recent-section {
    flex: 1 1 auto;
    min-height: 0;
    display: grid;
    grid-template-rows: auto 1fr;
  }

  .recent-scroll {
    min-height: 0;
    overflow-y: auto;
    -webkit-overflow-scrolling: touch;
    scrollbar-gutter: stable;
    scrollbar-width: thin;
    scrollbar-color: #b8bfcc transparent;
  }

  .recent-scroll::-webkit-scrollbar {
    width: 8px;
  }

  .recent-scroll::-webkit-scrollbar-thumb {
    background: #b8bfcc;
    border-radius: 999px;
  }

  .recent-scroll::-webkit-scrollbar-track {
    background: transparent;
  }
</style>
