<script lang="ts">
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import FolderPlus from "@lucide/svelte/icons/folder-plus";
  import Pencil from "@lucide/svelte/icons/pencil";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import Button from "$lib/components/ui/Button.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
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
  let createFolderError = $state("");
  let renamingFolderId = $state<string | null>(null);
  let renamingFolderName = $state("");
  let deletingFolder = $state<{ id: string; name: string } | null>(null);
  let openFolderIds = $state<string[]>([]);

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
      createFolderError = "폴더를 만들 수 없습니다. 이름 또는 최대 개수를 확인해 주세요.";
      return;
    }
    openFolderIds = [...openFolderIds, created];
    closeCreateFolderDialog();
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

  function requestDeleteFolder(folderId: string, folderName: string) {
    deletingFolder = { id: folderId, name: folderName };
  }

  function closeDeleteFolderDialog() {
    deletingFolder = null;
  }

  function confirmDeleteFolder() {
    if (!deletingFolder) return;
    onDeleteFolder(deletingFolder.id);
    deletingFolder = null;
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
      <Button type="button" size="xs" variant="soft" class="add-folder-btn" onclick={beginCreateFolder}>
        <FolderPlus size={14} />
        <span>폴더</span>
      </Button>
    {/if}
  </div>

  <Dialog
    open={creatingFolder}
    ariaLabel="새 폴더 추가"
    onOpenChange={(next) => {
      creatingFolder = next;
      if (!next) newFolderName = "";
    }}
  >
    {#snippet header()}
      <h4>
        <FolderPlus size={16} />
        <span>새 폴더 추가</span>
      </h4>
    {/snippet}
    {#snippet children()}
      <input
        class="folder-input"
        type="text"
        bind:value={newFolderName}
        placeholder="폴더 이름"
        maxlength="24"
        onkeydown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            submitCreateFolder();
          }
        }}
      />
    {/snippet}
    {#snippet actions()}
      <Button type="button" size="xs" variant="soft" onclick={closeCreateFolderDialog}>취소</Button>
      <Button type="button" size="xs" variant="pill-active" onclick={submitCreateFolder} disabled={!newFolderName.trim()}
        >추가</Button
      >
    {/snippet}
  </Dialog>

  <Dialog
    open={!!createFolderError}
    ariaLabel="폴더 생성 오류"
    title="폴더 생성 실패"
    description={createFolderError}
    onOpenChange={(next) => {
      if (!next) createFolderError = "";
    }}
  >
    {#snippet actions()}
      <Button type="button" size="xs" variant="pill-active" onclick={() => (createFolderError = "")}
        >확인</Button
      >
    {/snippet}
  </Dialog>

  <Dialog
    open={!!renamingFolderId}
    ariaLabel="폴더 이름 변경"
    onOpenChange={(next) => {
      if (!next) closeRenameFolderDialog();
    }}
  >
    {#snippet header()}
      <h4>
        <Pencil size={16} />
        <span>폴더 이름 변경</span>
      </h4>
    {/snippet}
    {#snippet children()}
      <input
        class="folder-input"
        type="text"
        bind:value={renamingFolderName}
        placeholder="폴더 이름"
        maxlength="24"
        onkeydown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            submitRenameFolder();
          }
        }}
      />
    {/snippet}
    {#snippet actions()}
      <Button type="button" size="xs" variant="soft" onclick={closeRenameFolderDialog}>취소</Button>
      <Button
        type="button"
        size="xs"
        variant="pill-active"
        onclick={submitRenameFolder}
        disabled={!renamingFolderName.trim()}>저장</Button
      >
    {/snippet}
  </Dialog>

  <Dialog
    open={!!deletingFolder}
    ariaLabel="폴더 삭제 확인"
    title="폴더 삭제"
    description={
      deletingFolder
        ? `'${deletingFolder.name}' 폴더를 삭제할까요? 항목은 기본 폴더로 이동됩니다.`
        : ""
    }
    onOpenChange={(next) => {
      if (!next) closeDeleteFolderDialog();
    }}
  >
    {#snippet actions()}
      <Button type="button" size="xs" variant="soft" onclick={closeDeleteFolderDialog}>취소</Button>
      <Button type="button" size="xs" variant="danger-soft" onclick={confirmDeleteFolder}>삭제</Button>
    {/snippet}
  </Dialog>

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

  :global(.add-folder-btn) {
    font-size: 12px;
    padding: 5px 9px;
    gap: 6px;
  }

  .panel :global(.ui-dialog-header h4) {
    margin: 0;
    font-size: 15px;
    color: var(--color-text);
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  .folder-input {
    border: 1px solid var(--color-border);
    border-radius: 10px;
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 12px;
    padding: 8px 10px;
    outline: none;
  }

  .folder-input:focus-visible {
    border-color: var(--color-accent);
    box-shadow: 0 0 0 3px color-mix(in oklab, var(--color-accent), white 84%);
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
