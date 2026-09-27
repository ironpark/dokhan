<script lang="ts">
  import { slide } from "svelte/transition";
  import Bookmark from "@lucide/svelte/icons/bookmark";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Ellipsis from "@lucide/svelte/icons/ellipsis";
  import Folder from "@lucide/svelte/icons/folder";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import FolderPlus from "@lucide/svelte/icons/folder-plus";
  import Pencil from "@lucide/svelte/icons/pencil";
  import DropdownMenu from "$lib/components/ui/DropdownMenu.svelte";
  import BookmarkItemRow from "$lib/components/bookmarks/BookmarkItemRow.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Toast from "$lib/components/ui/Toast.svelte";
  import type { BookmarkFolder, FavoriteItem } from "$lib/types/dictionary";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";

  let {
    favorites,
    allFavorites,
    legacyFavoriteCount = 0,
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
    legacyFavoriteCount?: number;
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

  let creatingFolder = $state(false);
  let newFolderName = $state("");
  let createFolderError = $state("");
  let toastMessage = $state("");
  let renamingFolderId = $state<string | null>(null);
  let renamingFolderName = $state("");
  let deletingFolder = $state<{ id: string; name: string } | null>(null);
  let openFolderIds = $state<string[]>([]);
  const totalCount = $derived(allFavorites.length);
  const reduceMotion =
    typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  const slideDuration = reduceMotion ? 0 : 180;

  function handleFolderMenu(folder: BookmarkFolder, action: string) {
    if (action === "rename") beginRenameFolder(folder.id, folder.name);
    else if (action === "delete") requestDeleteFolder(folder.id, folder.name);
  }
  let initializedFolder = false;

  $effect(() => {
    if (!initializedFolder && folders.length) {
      openFolderIds = [activeFolderId];
      initializedFolder = true;
    }
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
      createFolderError = "폴더를 만들 수 없습니다. 이름 또는 최대 개수를 확인해 주세요.";
      return;
    }
    toastMessage = "폴더를 추가했습니다.";
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
    toastMessage = "폴더 이름을 변경했습니다.";
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
    toastMessage = "폴더를 삭제했습니다.";
    deletingFolder = null;
  }

</script>

<section class="panel">
  <header class="library-head">
    <h3>책갈피{#if totalCount}<span class="total">{totalCount}</span>{/if}</h3>
    {#if !creatingFolder}
      <Button
        type="button"
        size="xs"
        variant="ghost"
        class="new-folder-btn"
        onclick={beginCreateFolder}
      >
        <FolderPlus size={14} />
        <span>새 폴더</span>
      </Button>
    {/if}
  </header>
  {#if legacyFavoriteCount > 0}
    <p class="legacy-notice" role="status">
      이전 버전 책갈피 {legacyFavoriteCount}개는 사전을 식별할 수 없어 보관 중입니다.
    </p>
  {/if}

  <Dialog
    open={creatingFolder}
    ariaLabel="새 폴더 추가"
    onOpenChange={(next) => {
      creatingFolder = next;
      if (!next) newFolderName = "";
    }}
  >
    {#snippet header()}
      <h4 class="m-0 inline-flex items-center gap-2 text-[16px] font-semibold text-[var(--color-text)]">
        <FolderPlus size={16} />
        <span>새 폴더 추가</span>
      </h4>
    {/snippet}
    {#snippet children()}
      <Input
        class="folder-input"
        bind:value={newFolderName}
        placeholder="폴더 이름"
        maxlength={24}
        uiSize="sm"
        onkeydown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            submitCreateFolder();
          }
        }}
      />
    {/snippet}
    {#snippet actions()}
      <Button type="button" size="sm" variant="outline" onclick={closeCreateFolderDialog}>취소</Button>
      <Button type="button" size="sm" onclick={submitCreateFolder} disabled={!newFolderName.trim()}
        >추가</Button
      >
    {/snippet}
  </Dialog>

  <ConfirmDialog
    open={!!createFolderError}
    title="폴더 생성 실패"
    description={createFolderError}
    confirmLabel="확인"
    cancelLabel="닫기"
    onCancel={() => (createFolderError = "")}
    onConfirm={() => (createFolderError = "")}
  />

  <Dialog
    open={!!renamingFolderId}
    ariaLabel="폴더 이름 변경"
    onOpenChange={(next) => {
      if (!next) closeRenameFolderDialog();
    }}
  >
    {#snippet header()}
      <h4 class="m-0 inline-flex items-center gap-2 text-[16px] font-semibold text-[var(--color-text)]">
        <Pencil size={16} />
        <span>폴더 이름 변경</span>
      </h4>
    {/snippet}
    {#snippet children()}
      <Input
        class="folder-input"
        bind:value={renamingFolderName}
        placeholder="폴더 이름"
        maxlength={24}
        uiSize="sm"
        onkeydown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            submitRenameFolder();
          }
        }}
      />
    {/snippet}
    {#snippet actions()}
      <Button type="button" size="sm" variant="outline" onclick={closeRenameFolderDialog}>취소</Button>
      <Button
        type="button"
        size="sm"
        onclick={submitRenameFolder}
        disabled={!renamingFolderName.trim()}>저장</Button
      >
    {/snippet}
  </Dialog>

  <ConfirmDialog
    open={!!deletingFolder}
    title="폴더 삭제"
    description={
      deletingFolder
        ? `'${deletingFolder.name}' 폴더를 삭제할까요? 항목은 기본 폴더로 이동됩니다.`
        : ""
    }
    confirmLabel="삭제"
    cancelLabel="취소"
    danger={true}
    onCancel={closeDeleteFolderDialog}
    onConfirm={confirmDeleteFolder}
  />

  <div class="folder-list">
    {#if totalCount === 0 && folders.length <= 1}
      <div class="library-empty">
        <span class="empty-icon" aria-hidden="true"><Bookmark size={20} /></span>
        <strong>저장한 책갈피가 없습니다</strong>
        <p>본문 위의 <b>책갈피</b> 버튼을 누르면<br />단어와 문서가 여기에 모입니다.</p>
      </div>
    {:else}
      {#each folders as folder (folder.id)}
        {@const isOpen = openFolderIds.includes(folder.id)}
        {@const folderItems = allFavorites.filter((item) => item.folderId === folder.id)}
        <section class="folder-group" class:open={isOpen} class:has-menu={folder.id !== "default"}>
          <header class="folder-head">
            <button
              type="button"
              class="folder-toggle"
              onclick={() => toggleFolder(folder.id)}
              aria-expanded={isOpen}
              aria-label={`${folder.name} 폴더, ${folderCountMap.get(folder.id) ?? 0}개`}
            >
              <span class="chevron" aria-hidden="true"><ChevronRight size={14} /></span>
              <span class="folder-icon" aria-hidden="true">
                {#if isOpen}<FolderOpen size={15} />{:else}<Folder size={15} />{/if}
              </span>
              <strong>{folder.name}</strong>
              <small>{folderCountMap.get(folder.id) ?? 0}</small>
            </button>

            {#if folder.id !== "default"}
              <div class="folder-menu">
                <DropdownMenu
                  label="폴더 메뉴"
                  ariaLabel={`${folder.name} 폴더 메뉴`}
                  variant="action"
                  options={[
                    { id: "rename", label: "이름 변경" },
                    { id: "delete", label: "폴더 삭제", danger: true },
                  ]}
                  onSelect={(action) => handleFolderMenu(folder, action)}
                >
                  {#snippet trigger()}
                    <Ellipsis size={15} aria-hidden="true" />
                  {/snippet}
                </DropdownMenu>
              </div>
            {/if}
          </header>

          {#if isOpen}
            <div class="folder-body" transition:slide={{ duration: slideDuration }}>
              {#if folderItems.length}
                <ul>
                  {#each folderItems as item (item.key)}
                    <BookmarkItemRow
                      {item}
                      {folders}
                      onOpen={onOpenFavorite}
                      onMove={onMoveFavorite}
                      onRemove={onRemoveFavorite}
                    />
                  {/each}
                </ul>
              {:else}
                <p class="folder-empty">비어 있습니다. 본문에서 저장하거나 다른 폴더의 항목을 옮겨 오세요.</p>
              {/if}
            </div>
          {/if}
        </section>
      {/each}
    {/if}
  </div>
  <Toast
    open={!!toastMessage}
    message={toastMessage}
    onOpenChange={(next) => {
      if (!next) toastMessage = "";
    }}
  />
</section>

<style>
  .panel {
    min-height: 0;
    height: 100%;
    overflow: hidden;
    box-sizing: border-box;
    padding: 2px 0 12px;
    display: flex;
    flex-direction: column;
  }

  .library-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 4px 10px 6px 18px;
  }

  .library-head h3 {
    display: inline-flex;
    align-items: baseline;
    gap: 6px;
    margin: 0;
    color: var(--color-text-subtle);
    font-size: 11.5px;
    font-weight: 600;
    letter-spacing: 0.06em;
  }

  .total {
    font-variant-numeric: tabular-nums;
  }

  .library-head :global(.new-folder-btn) {
    height: 28px;
    gap: 5px;
    padding: 0 9px;
    border-radius: var(--radius-full);
    color: var(--color-accent);
    font-size: 12px;
    font-weight: 600;
  }

  .legacy-notice {
    margin: 0 12px 8px;
    padding: 9px 10px;
    border: 1px solid var(--color-accent-border);
    border-radius: 8px;
    background: var(--color-accent-soft);
    color: var(--color-text-muted);
    font-size: 12px;
    line-height: 1.45;
  }

  :global(.folder-input input) {
    border: 1px solid var(--color-border);
    border-radius: 10px;
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 12px;
    padding: 8px 10px;
    outline: none;
  }

  :global(.folder-input input:focus-visible) {
    border-color: var(--color-accent);
    box-shadow: 0 0 0 3px color-mix(in oklab, var(--color-accent), var(--color-surface) 84%);
  }

  .folder-list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 0 8px;
  }

  /* Folders are plain disclosure groups: no card chrome, items indent under the name. */
  .folder-group {
    flex-shrink: 0;
  }

  .folder-head {
    position: relative;
    display: flex;
    align-items: center;
    border-radius: var(--radius-sm);
    transition: background-color var(--motion-fast);
  }

  .folder-head:hover,
  .folder-head:focus-within {
    background: var(--color-surface-hover);
  }

  .folder-toggle {
    flex: 1;
    min-width: 0;
    min-height: 36px;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 8px 0 4px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
  }

  .folder-toggle:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: -2px;
  }

  .chevron {
    display: inline-flex;
    color: var(--color-text-subtle);
    transition: transform var(--motion-base);
  }

  .folder-group.open .chevron {
    transform: rotate(90deg);
  }

  .folder-icon {
    display: inline-flex;
    color: var(--color-accent);
  }

  .folder-toggle strong {
    min-width: 0;
    font-size: 13.5px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .folder-toggle small {
    margin-left: auto;
    color: var(--color-text-subtle);
    font-size: 11.5px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  /* The ⋯ menu overlays the count slot, so counts line up across all folders
     and swap for the menu on hover/focus. */
  /* Centered with flex, not transform: a transformed ancestor would become the
     containing block of the dropdown's position:fixed menu and throw it off. */
  .folder-menu {
    position: absolute;
    top: 0;
    right: 4px;
    bottom: 0;
    display: flex;
    align-items: center;
    opacity: 0;
    transition: opacity var(--motion-fast);
  }

  .folder-head:hover .folder-menu,
  .folder-head:focus-within .folder-menu {
    opacity: 1;
  }

  .folder-toggle small {
    transition: opacity var(--motion-fast);
  }

  .has-menu .folder-head:hover .folder-toggle small,
  .has-menu .folder-head:focus-within .folder-toggle small {
    opacity: 0;
  }

  .folder-menu :global(button[aria-haspopup]) {
    width: 28px;
    min-height: 28px;
  }

  .folder-body {
    padding: 2px 0 8px 22px;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0 0 0 8px;
    border-left: 1px solid var(--color-border);
    display: grid;
    gap: 1px;
  }

  .folder-empty {
    margin: 0;
    padding: 6px 10px 6px 16px;
    border-left: 1px solid var(--color-border);
    color: var(--color-text-subtle);
    font-size: 12px;
    line-height: 1.5;
  }

  .library-empty {
    display: grid;
    justify-items: center;
    gap: 6px;
    margin: 24px 8px 0;
    padding: 24px 16px;
    border: 1px dashed var(--color-border-strong);
    border-radius: var(--radius-md);
    text-align: center;
  }

  .empty-icon {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    margin-bottom: 4px;
    border-radius: 12px;
    background: var(--color-warn-soft-bg);
    color: var(--color-warn-text);
  }

  .library-empty strong {
    color: var(--color-text);
    font-size: 13.5px;
  }

  .library-empty p {
    margin: 0;
    color: var(--color-text-subtle);
    font-size: 12px;
    line-height: 1.6;
  }

  .library-empty b {
    color: var(--color-text-muted);
  }

  @media (hover: none) {
    .folder-menu { opacity: 1; }
    .has-menu .folder-toggle small { margin-right: 36px; }
    .folder-toggle { min-height: 44px; }
    .folder-menu :global(button[aria-haspopup]) { width: 38px; min-height: 38px; }
  }
</style>
