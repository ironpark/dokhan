<script lang="ts">
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import DropdownMenu from "$lib/components/ui/DropdownMenu.svelte";
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

  const menuOptions = $derived(
    folders.map((folder) => ({ id: folder.id, label: folder.name, active: folder.id === item.folderId })),
  );

  const currentFolderName = $derived(
    folders.find((folder) => folder.id === item.folderId)?.name ?? "기본",
  );
</script>

<li>
  <button type="button" class="item-btn" onclick={() => onOpen(item)}>
    <span>{item.label}</span>
  </button>
  <div class="row-actions">
    <DropdownMenu
      label={currentFolderName}
      ariaLabel={`책갈피 폴더 선택, 현재 ${currentFolderName}`}
      options={menuOptions}
      onSelect={(folderId) => onMove(item.key, folderId)}
    />
    <button
      type="button"
      class="remove-btn"
      aria-label="책갈피 삭제"
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
    border-top: 1px solid color-mix(in oklab, var(--color-border), var(--color-surface) 10%);
    min-height: 42px;
  }

  .item-btn {
    border: none;
    background: transparent;
    text-align: left;
    padding: 7px 10px;
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
    font-size: 13px;
    line-height: 1.35;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .row-actions {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 0 8px;
  }

  .remove-btn {
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    width: 32px;
    height: 32px;
    padding: 0;
    border-radius: 5px;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .remove-btn:hover {
    color: var(--color-danger);
    background: color-mix(in oklab, var(--color-danger), var(--color-surface) 93%);
  }

  .item-btn:focus-visible,
  .remove-btn:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: -2px;
  }

  @media (max-width: 980px) {
    li { min-height: 48px; }
    .remove-btn { width: 40px; height: 40px; }
  }
</style>
