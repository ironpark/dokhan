<script lang="ts">
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import type { BookmarkFolder, FavoriteItem } from "$lib/types/dictionary";

  let {
    item,
    folders,
    onOpen,
    onMove,
    onRemove,
  }: {
    item: FavoriteItem;
    folders: BookmarkFolder[];
    onOpen: (item: FavoriteItem) => void;
    onMove: (key: string, folderId: string) => void;
    onRemove: (key: string) => void;
  } = $props();

  function folderNameById(folderId: string): string {
    return folders.find((folder) => folder.id === folderId)?.name ?? "기본";
  }

  function moveFavoriteFromMenu(event: Event, key: string, folderId: string) {
    onMove(key, folderId);
    const button = event.currentTarget as HTMLElement | null;
    const details = button?.closest("details") as HTMLDetailsElement | null;
    if (details) details.open = false;
  }
</script>

<li>
  <button type="button" class="item-btn" onclick={() => onOpen(item)}>
    <span>{item.label}</span>
  </button>
  <div class="row-actions">
    <details class="move-dropdown">
      <summary aria-label="폴더 이동 메뉴">
        {folderNameById(item.folderId)}
      </summary>
      <div class="move-menu">
        {#each folders as targetFolder (targetFolder.id)}
          <button
            type="button"
            class="move-option"
            class:active={targetFolder.id === item.folderId}
            onclick={(event) => moveFavoriteFromMenu(event, item.key, targetFolder.id)}
          >
            {targetFolder.name}
          </button>
        {/each}
      </div>
    </details>
    <button
      type="button"
      class="remove-btn"
      aria-label="북마크 삭제"
      onclick={() => onRemove(item.key)}
      title="삭제"
    >
      <Trash2 size={13} />
    </button>
  </div>
</li>

<style>
  li {
    display: grid;
    grid-template-columns: 1fr auto;
    border-top: 1px solid color-mix(in oklab, var(--color-border), white 10%);
    min-height: 34px;
  }

  .item-btn {
    border: none;
    background: transparent;
    text-align: left;
    padding: 4px 8px;
    display: flex;
    align-items: center;
    cursor: pointer;
    min-height: 100%;
  }

  .item-btn:hover {
    background: var(--color-surface-hover);
  }

  .item-btn span {
    color: var(--color-text);
    font-size: 11px;
    line-height: 1.2;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .row-actions {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 0 6px;
  }

  .move-dropdown {
    position: relative;
  }

  .move-dropdown summary {
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text-muted);
    border-radius: 6px;
    font-size: 9px;
    line-height: 1.1;
    height: 20px;
    min-width: 44px;
    padding: 0 5px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    list-style: none;
    cursor: pointer;
    white-space: nowrap;
  }

  .move-dropdown summary::-webkit-details-marker {
    display: none;
  }

  .move-menu {
    position: absolute;
    right: 0;
    top: calc(100% + 4px);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    background: var(--color-surface);
    box-shadow: 0 8px 18px rgba(0, 0, 0, 0.14);
    min-width: 104px;
    padding: 4px;
    display: grid;
    gap: 2px;
    z-index: 20;
  }

  .move-option {
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    border-radius: 6px;
    font-size: 10px;
    text-align: left;
    padding: 5px 6px;
    cursor: pointer;
  }

  .move-option:hover {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  .move-option.active {
    color: var(--color-accent);
    background: color-mix(in oklab, var(--color-accent), white 92%);
  }

  .remove-btn {
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    width: 20px;
    height: 20px;
    padding: 0;
    border-radius: 5px;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .remove-btn:hover {
    color: var(--color-danger);
    background: color-mix(in oklab, var(--color-danger), white 93%);
  }
</style>
