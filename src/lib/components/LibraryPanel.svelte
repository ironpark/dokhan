<script lang="ts">
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import FolderPlus from "@lucide/svelte/icons/folder-plus";
  import Pencil from "@lucide/svelte/icons/pencil";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import type { BookmarkFolder, FavoriteItem } from "$lib/types/dictionary";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";

  let {
    favorites,
    allFavorites,
    folders,
    activeFolderId,
    onOpenFavorite,
    onRemoveFavorite,
    onSelectFolder,
    onCreateFolder,
    onRenameFolder,
    onDeleteFolder,
    onMoveFavorite,
  }: {
    favorites: FavoriteItem[];
    allFavorites: FavoriteItem[];
    folders: BookmarkFolder[];
    activeFolderId: string;
    onOpenFavorite: (item: FavoriteItem) => void;
    onRemoveFavorite: (key: string) => void;
    onSelectFolder: (folderId: string) => void;
    onCreateFolder: (name: string) => string | null;
    onRenameFolder: (folderId: string, name: string) => void;
    onDeleteFolder: (folderId: string) => void;
    onMoveFavorite: (key: string, folderId: string) => void;
  } = $props();

  const folderCountMap = $derived.by(() => {
    const map = new Map<string, number>();
    for (const item of allFavorites) {
      map.set(item.folderId, (map.get(item.folderId) ?? 0) + 1);
    }
    return map;
  });

  const activeFolderName = $derived(
    folders.find((folder) => folder.id === activeFolderId)?.name ?? "기본",
  );

  let creatingFolder = $state(false);
  let newFolderName = $state("");
  let renamingFolderId = $state<string | null>(null);
  let renamingFolderName = $state("");
  let openFolderIds = $state<string[]>([]);
  let createDialogEl = $state<HTMLDialogElement | null>(null);
  let renameDialogEl = $state<HTMLDialogElement | null>(null);

  $effect(() => {
    const dialog = createDialogEl;
    if (!dialog) return;
    if (creatingFolder) {
      if (!dialog.open) dialog.showModal();
      return;
    }
    if (dialog.open) dialog.close();
  });

  $effect(() => {
    const dialog = renameDialogEl;
    if (!dialog) return;
    if (renamingFolderId) {
      if (!dialog.open) dialog.showModal();
      return;
    }
    if (dialog.open) dialog.close();
  });

  function toggleFolder(folderId: string) {
    if (openFolderIds.includes(folderId)) {
      openFolderIds = openFolderIds.filter((id) => id !== folderId);
    } else {
      openFolderIds = [...openFolderIds, folderId];
      onSelectFolder(folderId);
    }
  }

  function beginCreateFolder() {
    creatingFolder = true;
    newFolderName = "";
  }

  function closeCreateFolderDialog() {
    creatingFolder = false;
    newFolderName = "";
  }

  function submitCreateFolder() {
    const created = onCreateFolder(newFolderName);
    if (!created) {
      window.alert("폴더를 만들 수 없습니다. 이름 또는 최대 개수를 확인해 주세요.");
      return;
    }
    openFolderIds = [...openFolderIds, created];
    closeCreateFolderDialog();
  }

  function onCreateDialogClose() {
    closeCreateFolderDialog();
  }

  function onCreateDialogClick(event: MouseEvent) {
    const dialog = event.currentTarget as HTMLDialogElement | null;
    if (!dialog) return;
    if (event.target === dialog) {
      dialog.close();
    }
  }

  function beginRenameFolder(folderId: string, currentName: string) {
    renamingFolderId = folderId;
    renamingFolderName = currentName;
  }

  function closeRenameFolderDialog() {
    renamingFolderId = null;
    renamingFolderName = "";
  }

  function submitRenameFolder() {
    if (!renamingFolderId) return;
    const trimmed = renamingFolderName.trim();
    if (!trimmed) return;
    const currentName = folders.find((folder) => folder.id === renamingFolderId)?.name?.trim();
    if (currentName === trimmed) {
      closeRenameFolderDialog();
      return;
    }
    onRenameFolder(renamingFolderId, trimmed);
    closeRenameFolderDialog();
  }

  function onRenameDialogClose() {
    closeRenameFolderDialog();
  }

  function onRenameDialogClick(event: MouseEvent) {
    const dialog = event.currentTarget as HTMLDialogElement | null;
    if (!dialog) return;
    if (event.target === dialog) {
      dialog.close();
    }
  }

  function requestDeleteFolder(folderId: string, folderName: string) {
    const ok = window.confirm(`'${folderName}' 폴더를 삭제할까요?\n항목은 기본 폴더로 이동됩니다.`);
    if (!ok) return;
    onDeleteFolder(folderId);
  }

  function folderNameById(folderId: string): string {
    return folders.find((folder) => folder.id === folderId)?.name ?? "기본";
  }

  function moveFavoriteFromMenu(event: Event, key: string, folderId: string) {
    onMoveFavorite(key, folderId);
    const button = event.currentTarget as HTMLElement | null;
    const details = button?.closest("details") as HTMLDetailsElement | null;
    if (details) details.open = false;
  }
</script>

<section class="panel">
  <div class="panel-head">
    <h3>북마크 폴더</h3>
    {#if !creatingFolder}
      <button type="button" class="add-folder-btn" onclick={beginCreateFolder}>
        <FolderPlus size={14} />
        <span>폴더</span>
      </button>
    {/if}
  </div>

  {#if creatingFolder}
    <dialog
      bind:this={createDialogEl}
      class="create-dialog"
      onclose={onCreateDialogClose}
      onclick={onCreateDialogClick}
      aria-label="새 폴더 추가"
      >
        <form
          class="create-dialog-form"
        onsubmit={(event) => {
          event.preventDefault();
          submitCreateFolder();
        }}
        >
        <h4>
          <FolderPlus size={16} />
          <span>새 폴더 추가</span>
        </h4>
        <input
          type="text"
          bind:value={newFolderName}
          placeholder="폴더 이름"
          maxlength="24"
        />
        <div class="create-dialog-actions">
          <button type="button" class="ghost" onclick={closeCreateFolderDialog}>취소</button>
          <button type="submit" disabled={!newFolderName.trim()}>추가</button>
        </div>
      </form>
    </dialog>
  {/if}

  {#if renamingFolderId}
    <dialog
      bind:this={renameDialogEl}
      class="create-dialog"
      onclose={onRenameDialogClose}
      onclick={onRenameDialogClick}
      aria-label="폴더 이름 변경"
      >
      <form
        class="create-dialog-form"
        onsubmit={(event) => {
          event.preventDefault();
          submitRenameFolder();
        }}
      >
        <h4>
          <Pencil size={16} />
          <span>폴더 이름 변경</span>
        </h4>
        <input
          type="text"
          bind:value={renamingFolderName}
          placeholder="폴더 이름"
          maxlength="24"
        />
        <div class="create-dialog-actions">
          <button type="button" class="ghost" onclick={closeRenameFolderDialog}>취소</button>
          <button type="submit" disabled={!renamingFolderName.trim()}>저장</button>
        </div>
      </form>
    </dialog>
  {/if}

  <div class="folder-list">
    {#each folders as folder (folder.id)}
      {@const isActive = activeFolderId === folder.id}
      {@const isOpen = openFolderIds.includes(folder.id)}
      {@const folderItems = allFavorites.filter((item) => item.folderId === folder.id)}
      <section class="folder-card" class:active={isActive} class:open={isOpen}>
        <header class="folder-card-head">
          <button
            type="button"
            class="folder-toggle"
            onclick={() => toggleFolder(folder.id)}
            aria-expanded={isOpen}
            aria-label={`${folder.name} 폴더 열기`}
          >
            <span class="chevron" aria-hidden="true">
              <ChevronDown size={14} />
            </span>
            <strong>{folder.name}</strong>
            <small>{folderCountMap.get(folder.id) ?? 0}</small>
          </button>

          {#if folder.id !== "default"}
            <div class="folder-actions">
              <button
                type="button"
                class="icon-btn"
                onclick={(event) => {
                  event.stopPropagation();
                  beginRenameFolder(folder.id, folder.name);
                }}
                aria-label="폴더 이름 변경"
                title="이름 변경"
              >
                <Pencil size={13} />
              </button>
              <button
                type="button"
                class="icon-btn danger"
                onclick={() => requestDeleteFolder(folder.id, folder.name)}
                aria-label="폴더 삭제"
                title="삭제"
              >
                <Trash2 size={13} />
              </button>
            </div>
          {/if}
        </header>

        <div class="folder-card-body" class:open={isOpen} hidden={!isOpen}>
          <div class="folder-card-inner">
            {#if isOpen}
              {#if folderItems.length}
                <ul>
                  {#each folderItems as item (item.key)}
                    <li>
                      <button type="button" class="item-btn" onclick={() => onOpenFavorite(item)}>
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
                                onclick={(event) =>
                                  moveFavoriteFromMenu(event, item.key, targetFolder.id)}
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
                          onclick={() => onRemoveFavorite(item.key)}
                          title="삭제"
                        >
                          <Trash2 size={13} />
                        </button>
                      </div>
                    </li>
                  {/each}
                </ul>
              {:else}
                <div class="folder-empty">
                  <EmptyState
                    title={`'${folder.name}' 폴더가 비어 있습니다.`}
                    description="본문에서 '북마크' 버튼으로 저장하거나 다른 폴더를 선택하세요."
                    compact={true}
                  />
                </div>
              {/if}
            {/if}
          </div>
        </div>
      </section>
    {/each}
  </div>
</section>

<style>
  .panel {
    min-height: 0;
    height: 100%;
    overflow: hidden;
    box-sizing: border-box;
    padding: 10px;
    display: grid;
    grid-template-rows: auto 1fr;
    gap: 10px;
  }

  .panel-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 2px;
  }

  .panel-head h3 {
    margin: 0;
    font-size: 11px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .add-folder-btn {
    border: 1px solid var(--color-border);
    background: color-mix(in oklab, var(--color-surface), white 10%);
    color: var(--color-text);
    font-size: 12px;
    border-radius: 8px;
    padding: 5px 9px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    transition: border-color var(--motion-fast), background-color var(--motion-fast);
  }

  .add-folder-btn:hover {
    border-color: var(--color-border-strong);
    background: var(--color-surface-hover);
  }

  .create-dialog {
    width: min(420px, calc(100vw - 24px));
    max-width: none;
    border: 1px solid color-mix(in oklab, var(--color-border), white 12%);
    border-radius: 14px;
    background: var(--color-surface);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.2);
    padding: 0;
    z-index: 1400;
  }

  .create-dialog::backdrop {
    background: rgba(8, 10, 14, 0.28);
    backdrop-filter: blur(2px);
  }

  .create-dialog-form {
    padding: 14px;
    display: grid;
    gap: 10px;
  }

  .create-dialog-form h4 {
    margin: 0;
    font-size: 15px;
    color: var(--color-text);
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  .create-dialog-form input {
    border: 1px solid var(--color-border);
    border-radius: 10px;
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 12px;
    padding: 8px 10px;
    outline: none;
  }

  .create-dialog-form input:focus-visible {
    border-color: var(--color-accent);
    box-shadow: 0 0 0 3px color-mix(in oklab, var(--color-accent), white 84%);
  }

  .create-dialog-actions {
    display: inline-flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .create-dialog-form button {
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text-muted);
    border-radius: 8px;
    padding: 7px 10px;
    font-size: 12px;
    cursor: pointer;
  }

  .create-dialog-form button:not(.ghost) {
    color: var(--color-accent);
    border-color: color-mix(in oklab, var(--color-accent), white 65%);
    background: color-mix(in oklab, var(--color-accent), white 92%);
  }

  .folder-list {
    min-height: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-right: 2px;
  }

  .folder-card {
    border: 1px solid color-mix(in oklab, var(--color-border), white 18%);
    border-radius: 8px;
    background: var(--color-surface);
    overflow: hidden;
    transition: border-color var(--motion-fast), box-shadow var(--motion-fast);
  }

  .folder-card.active {
    border-color: color-mix(in oklab, var(--color-accent), white 62%);
    box-shadow: 0 0 0 1px color-mix(in oklab, var(--color-accent), white 78%);
  }

  .folder-card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 6px 8px;
    background: color-mix(in oklab, var(--color-surface-soft), white 24%);
    border-bottom: 1px solid color-mix(in oklab, var(--color-border), white 18%);
  }

  .folder-toggle {
    border: none;
    background: transparent;
    color: var(--color-text);
    width: 100%;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    text-align: left;
    cursor: pointer;
    min-width: 0;
  }

  .folder-toggle strong {
    font-size: 13px;
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .folder-toggle small {
    margin-left: auto;
    font-size: 11px;
    color: var(--color-text-muted);
    border: 1px solid var(--color-border);
    border-radius: 999px;
    padding: 2px 7px;
  }

  .chevron {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted);
    transform: rotate(-90deg);
    transition: transform var(--motion-fast);
  }

  .folder-card.open .chevron {
    transform: rotate(0deg);
  }

  .folder-actions {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    flex: 0 0 auto;
  }

  .icon-btn {
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text-muted);
    line-height: 0;
    border-radius: 8px;
    width: 26px;
    height: 24px;
    padding: 0;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .icon-btn.danger:hover {
    color: var(--color-danger);
    border-color: color-mix(in oklab, var(--color-danger), white 62%);
  }

  .folder-card-body {
    display: block;
  }

  .folder-card-body.open {
    display: block;
  }

  .folder-card-inner {
    min-height: 0;
    overflow: hidden;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: min(46vh, 360px);
    overflow-y: auto;
  }

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

  .folder-empty {
    padding: 8px;
  }
</style>
