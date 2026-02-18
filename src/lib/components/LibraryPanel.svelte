<script lang="ts">
  import type { FavoriteItem, RecentViewItem } from "$lib/types/dictionary";

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
</script>

<section class="panel">
  <div class="section">
    <h3>즐겨찾기</h3>
    {#if favorites.length}
      <ul>
        {#each favorites as item (item.key)}
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
    {:else}
      <p class="empty">아직 저장된 즐겨찾기가 없습니다.</p>
    {/if}
  </div>

  <div class="section">
    <h3>최근 열람</h3>
    {#if recents.length}
      <ul>
        {#each recents as item (item.key)}
          <li>
            <button type="button" class="item-btn" onclick={() => onOpenRecent(item)}>
              <small>{item.kind === "entry" ? "표제어" : "목차"}</small>
              <span>{item.label}</span>
            </button>
          </li>
        {/each}
      </ul>
    {:else}
      <p class="empty">최근 열람 기록이 없습니다.</p>
    {/if}
  </div>
</section>

<style>
  .panel {
    height: 100%;
    overflow-y: auto;
    box-sizing: border-box;
    padding: 10px;
    display: grid;
    gap: 14px;
    align-content: start;
  }

  .section {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 10px;
    overflow: hidden;
  }

  .section h3 {
    margin: 0;
    padding: 10px 12px;
    font-size: 13px;
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
    min-height: 46px;
  }

  li:last-child {
    border-bottom: none;
  }

  .item-btn {
    border: none;
    background: transparent;
    text-align: left;
    padding: 8px 12px;
    display: grid;
    gap: 2px;
    cursor: pointer;
  }

  .item-btn small {
    color: var(--color-text-muted);
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
    padding: 0 10px;
    cursor: pointer;
  }

  .empty {
    margin: 0;
    padding: 10px 12px 12px;
    font-size: 12px;
    color: var(--color-text-muted);
  }
</style>
