<script lang="ts">
    import BookOpen from "@lucide/svelte/icons/book-open";
    import type { DictionaryStore } from "$lib/stores/dictionaryStore.svelte";
    import SearchPanel from "$lib/components/SearchPanel.svelte";
    import IndexPanel from "$lib/components/IndexPanel.svelte";
    import ReaderPane from "$lib/components/ReaderPane.svelte";
    import ContentPanel from "$lib/components/ContentPanel.svelte";
    import LibraryPanel from "$lib/components/LibraryPanel.svelte";
    import TabBar from "$lib/components/TabBar.svelte";
    import TitleToolbar from "$lib/components/TitleToolbar.svelte";
    import EmptyState from "$lib/components/ui/EmptyState.svelte";

    let { dictionaryStore }: { dictionaryStore: DictionaryStore } = $props();
</script>

<div class="desktop-layout">
    <aside class="sidebar">
        <TitleToolbar
            title="독한 사전"
            subtitle={dictionaryStore.activeZipName}
            showZipAction={true}
            onPickZip={() => dictionaryStore.pickZipFile()}
        />

        <div class="tabs-container">
            <TabBar
                activeTab={dictionaryStore.activeTab}
                onChange={(tab) => {
                    dictionaryStore.setActiveTab(tab);
                }}
            />
        </div>

        <div
            class="sidebar-content"
            id="dictionary-tab-panel"
            role="tabpanel"
            aria-labelledby={`tab-${dictionaryStore.activeTab}`}
            tabindex="0"
        >
            {#if dictionaryStore.activeTab === "content"}
                <ContentPanel
                    items={dictionaryStore.contents}
                    recents={dictionaryStore.recentViews}
                    selectedLocal={dictionaryStore.selectedContentLocal}
                    onOpen={(local) => dictionaryStore.openContent(local)}
                    onOpenRecent={(item) => dictionaryStore.openRecentView(item)}
                />
            {:else if dictionaryStore.activeTab === "index"}
                <IndexPanel
                    query={dictionaryStore.indexPrefix}
                    rows={dictionaryStore.indexRows}
                    loading={dictionaryStore.indexLoading}
                    hasMore={dictionaryStore.indexHasMore}
                    loadingMore={dictionaryStore.indexLoadingMore}
                    onLoadMore={() => dictionaryStore.loadMoreIndex()}
                    selectedId={dictionaryStore.selectedEntryId}
                    onQueryChange={(value) => dictionaryStore.setIndexPrefix(value)}
                    onOpen={(id) => dictionaryStore.openEntry(id)}
                />
            {:else}
                {#if dictionaryStore.activeTab === "search"}
                    <SearchPanel
                        query={dictionaryStore.searchQuery}
                        committedQuery={dictionaryStore.committedSearchQuery}
                        rows={dictionaryStore.searchRows}
                        loading={dictionaryStore.isSearching}
                        recentSearches={dictionaryStore.recentSearches}
                        selectedId={dictionaryStore.selectedEntryId}
                        onQueryChange={(value) => dictionaryStore.setSearchQuery(value)}
                        onSubmit={() => dictionaryStore.submitSearch()}
                        onPickRecentSearch={(query) => dictionaryStore.useRecentSearch(query)}
                        onOpen={(id) => dictionaryStore.openEntry(id)}
                    />
                {:else}
                    <LibraryPanel
                        favorites={dictionaryStore.favorites}
                        allFavorites={dictionaryStore.allFavorites}
                        legacyFavoriteCount={dictionaryStore.legacyFavoriteCount}
                        folders={dictionaryStore.bookmarkFolders}
                        activeFolderId={dictionaryStore.activeBookmarkFolderId}
                        onOpenFavorite={(item) => dictionaryStore.openFavorite(item)}
                        onSelectFolder={(folderId) => dictionaryStore.setActiveBookmarkFolder(folderId)}
                        onCreateFolder={(name) => dictionaryStore.createBookmarkFolder(name)}
                        onRenameFolder={(folderId, name) =>
                            dictionaryStore.renameBookmarkFolder(folderId, name)}
                        onDeleteFolder={(folderId) => dictionaryStore.deleteBookmarkFolder(folderId)}
                        onMoveFavorite={(key, folderId) =>
                            dictionaryStore.moveFavoriteToFolder(key, folderId)}
                        onRemoveFavorite={(key) => dictionaryStore.removeFavorite(key)}
                    />
                {/if}
            {/if}
        </div>
    </aside>

    <main class="main-content">
        {#if !dictionaryStore.selectedContent && !dictionaryStore.selectedEntry}
            <div class="empty-state">
                <div class="empty-state-card">
                    <span class="empty-state-icon" aria-hidden="true"><BookOpen size={25} strokeWidth={1.7} /></span>
                    <EmptyState
                        title="읽을 항목을 선택하세요"
                        description="왼쪽의 목차, 색인, 검색 또는 책갈피에서 항목을 선택하면 본문이 이곳에 표시됩니다."
                    />
                </div>
            </div>
        {:else}
            <ReaderPane
                mode={dictionaryStore.detailMode}
                selectedContent={dictionaryStore.selectedContent}
                selectedEntry={dictionaryStore.selectedEntry}
                highlightQuery={dictionaryStore.committedSearchQuery}
                onOpenHref={(href, path, local) =>
                    dictionaryStore.openInlineHref(href, path, local)}
                onResolveImageHref={(href, path, local) =>
                    dictionaryStore.resolveInlineImageHref(href, path, local)}
                isFavorite={dictionaryStore.isCurrentFavorite()}
                onToggleFavorite={() => dictionaryStore.toggleCurrentFavorite()}
                bookmarkFolders={dictionaryStore.bookmarkFolders}
                activeBookmarkFolderId={dictionaryStore.activeBookmarkFolderId}
                currentBookmarkFolderId={dictionaryStore.currentFavoriteFolderId()}
                onAddBookmarkToFolder={(folderId) => dictionaryStore.addCurrentFavoriteToFolder(folderId)}
                onCreateBookmarkFolder={(name) => dictionaryStore.createBookmarkFolder(name)}
                preprocessEnabled={dictionaryStore.preprocessEnabled}
                onTogglePreprocess={() =>
                    dictionaryStore.setPreprocessEnabled(!dictionaryStore.preprocessEnabled)}
                markerPreprocessEnabled={dictionaryStore.markerPreprocessEnabled}
                onToggleMarkerPreprocess={() =>
                    dictionaryStore.setMarkerPreprocessEnabled(!dictionaryStore.markerPreprocessEnabled)}
                readerFontSize={dictionaryStore.readerFontSize}
                readerLineHeight={dictionaryStore.readerLineHeight}
                readerWidth={dictionaryStore.readerWidth}
                onReaderFontSizeChange={(value) => dictionaryStore.setReaderFontSize(value)}
                onReaderLineHeightChange={(value) =>
                    dictionaryStore.setReaderLineHeight(value)}
                onReaderWidthChange={(value) => dictionaryStore.setReaderWidth(value)}
            />
        {/if}
    </main>
</div>

<style>
    .desktop-layout {
        display: grid;
        grid-template-columns: clamp(276px, 24vw, 336px) minmax(0, 1fr);
        flex: 1;
        height: 100%;
        min-height: 0;
        width: 100%;
        overflow: hidden;
        background: var(--color-surface);
        font-family: var(--font-sans);
        color: var(--color-text);
    }

    .sidebar {
        display: grid;
        grid-template-rows: auto auto minmax(0, 1fr);
        min-width: 0;
        min-height: 0;
        border-right: 1px solid var(--color-border);
        background: var(--color-surface-soft);
        overflow: hidden;
    }

    .sidebar-content {
        overflow: hidden;
        background: var(--color-surface-soft);
        min-height: 0;
        outline-offset: -3px;
    }

    .main-content {
        background: var(--color-surface);
        overflow: hidden;
        display: flex;
        flex-direction: column;
        position: relative;
        min-width: 0;
        min-height: 0;
    }

    .empty-state {
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        color: var(--color-text-muted);
        user-select: none;
        padding: 28px;
        box-sizing: border-box;
        background: radial-gradient(circle at 50% 42%, var(--color-surface-soft) 0, var(--color-surface) 65%);
    }

    .empty-state-card {
        width: min(100%, 420px);
        padding: 36px 30px;
        border: 1px solid var(--color-border);
        border-radius: var(--radius-xl);
        background: var(--color-surface);
        box-shadow: var(--shadow-sm);
        text-align: center;
    }

    .empty-state-icon {
        width: 52px;
        height: 52px;
        display: grid;
        place-items: center;
        margin: 0 auto 12px;
        border-radius: 16px;
        background: var(--color-accent-soft);
        color: var(--color-accent);
    }

    .empty-state-card :global(.empty-state) {
        padding: 0;
        gap: 8px;
    }

    .empty-state-card :global(.empty-state .title) {
        color: var(--color-text);
        font-size: 16px;
    }

    .empty-state-card :global(.empty-state .description) {
        color: var(--color-text-muted);
        font-size: 13px;
        line-height: 1.55;
    }

    .tabs-container {
        padding: 2px 12px 10px;
        background: var(--color-surface-soft);
    }

    /* In the sidebar the header flows straight into the tabs; no divider needed. */
    .sidebar :global(.title-toolbar) {
        border-bottom: 0;
        min-height: 64px;
        padding: 12px 12px 10px 16px;
    }

    @media (max-width: 820px) {
        .desktop-layout {
            grid-template-columns: 248px minmax(0, 1fr);
        }
    }

    @media (max-width: 620px) {
        .desktop-layout {
            grid-template-columns: minmax(0, 1fr);
            grid-template-rows: minmax(220px, 42%) minmax(0, 1fr);
        }

        .sidebar {
            border-right: 0;
            border-bottom: 1px solid var(--color-border-strong);
        }

        .empty-state-card {
            padding: 24px 20px;
        }
    }
</style>
