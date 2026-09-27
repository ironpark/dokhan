<script lang="ts">
  import FileText from "@lucide/svelte/icons/file-text";
  import FolderInput from "@lucide/svelte/icons/folder-input";
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

  // Only the other folders are useful move targets.
  const moveTargets = $derived(
    folders
      .filter((folder) => folder.id !== item.folderId)
      .map((folder) => ({ id: folder.id, label: folder.name })),
  );
</script>

<li class="row">
  <button type="button" class="item-btn" onclick={() => onOpen(item)}>
    {#if item.kind === "content"}
      <span class="kind-icon" aria-hidden="true"><FileText size={13} /></span>
    {/if}
    <span class="label" class:headword={item.kind === "entry"}>{item.label}</span>
    <span class="sr-only">{item.kind === "entry" ? "표제어" : "목차 항목"}</span>
  </button>
  <!-- Actions stay out of the way until the row is hovered or focused (always shown on touch). -->
  <div class="row-actions">
    {#if moveTargets.length}
      <DropdownMenu
        label="다른 폴더로 이동"
        ariaLabel={`${item.label} 다른 폴더로 이동`}
        heading="이동할 폴더"
        options={moveTargets}
        onSelect={(folderId) => onMove(item.key, folderId)}
      >
        {#snippet trigger()}
          <FolderInput size={14} aria-hidden="true" />
        {/snippet}
      </DropdownMenu>
    {/if}
    <button
      type="button"
      class="remove-btn"
      aria-label={`${item.label} 책갈피 삭제`}
      title="삭제"
      onclick={() => onRemove(item.key)}
    >
      <Trash2 size={14} />
    </button>
  </div>
</li>

<style>
  .row {
    position: relative;
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    min-height: 36px;
    border-radius: var(--radius-sm);
    transition: background-color var(--motion-fast);
  }

  .row:hover,
  .row:focus-within {
    background: var(--color-surface-hover);
  }

  .item-btn {
    display: flex;
    align-items: center;
    gap: 7px;
    min-width: 0;
    min-height: 36px;
    padding: 0 8px 0 10px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    text-align: left;
    cursor: pointer;
  }

  .kind-icon {
    display: inline-flex;
    flex: none;
    color: var(--color-text-subtle);
  }

  .label {
    min-width: 0;
    color: var(--color-text);
    font-size: 13.5px;
    line-height: 1.35;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .label.headword {
    font-family: var(--font-serif);
    font-size: 15px;
  }

  .row-actions {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    padding-right: 4px;
    opacity: 0;
    transition: opacity var(--motion-fast);
  }

  .row:hover .row-actions,
  .row:focus-within .row-actions {
    opacity: 1;
  }

  .row-actions :global(button[aria-haspopup]) {
    width: 28px;
    min-height: 28px;
  }

  .remove-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border: none;
    border-radius: 7px;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    transition: color var(--motion-fast), background-color var(--motion-fast);
  }

  .remove-btn:hover {
    color: var(--color-danger);
    background: var(--color-danger-soft-bg);
  }

  .item-btn:focus-visible,
  .remove-btn:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: -2px;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }

  /* No hover on touch: keep actions visible and targets larger. */
  @media (hover: none) {
    .row-actions { opacity: 1; }
    .row { min-height: 44px; }
    .item-btn { min-height: 44px; }
    .remove-btn,
    .row-actions :global(button[aria-haspopup]) { width: 38px; height: 38px; }
  }
</style>
