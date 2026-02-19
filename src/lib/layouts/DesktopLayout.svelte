<script lang="ts">
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
            subtitle="Dokhan Dictionary"
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

        <div class="sidebar-content">
            {#if dictionaryStore.activeTab === "content"}
                <ContentPanel
                    items={dictionaryStore.contents}
                    selectedLocal={dictionaryStore.selectedContentLocal}
                    onOpen={(local) => dictionaryStore.openContent(local)}
                />
            {:else if dictionaryStore.activeTab === "index"}
                <IndexPanel
                    query={dictionaryStore.indexPrefix}
                    rows={dictionaryStore.indexRows}
                    loading={dictionaryStore.indexLoading}
                    selectedId={dictionaryStore.selectedEntryId}
                    onQueryChange={(value) => dictionaryStore.setIndexPrefix(value)}
                    onOpen={(id) => dictionaryStore.openEntry(id)}
                />
            {:else}
                {#if dictionaryStore.activeTab === "search"}
                    <SearchPanel
                        query={dictionaryStore.searchQuery}
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
                        recents={dictionaryStore.recentViews}
                        onOpenFavorite={(item) => dictionaryStore.openFavorite(item)}
                        onOpenRecent={(item) => dictionaryStore.openRecentView(item)}
                        onRemoveFavorite={(key) => dictionaryStore.removeFavorite(key)}
                    />
                {/if}
            {/if}
        </div>
    </aside>

    <main class="main-content">
        {#if !dictionaryStore.selectedContent && !dictionaryStore.selectedEntry}
            <div class="empty-state">
                <EmptyState
                    title="본문을 표시할 항목을 선택하세요."
                    description="목차, 색인, 검색 또는 즐겨찾기에서 항목을 선택하면 여기에 표시됩니다."
                />
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
        grid-template-columns: 280px 1fr;
        height: 100vh;
        overflow: hidden;
        background: var(--color-bg);
        font-family: var(--font-sans);
        color: var(--color-text);
    }

    .sidebar {
        display: grid;
        grid-template-rows: auto auto 1fr;
        border-right: 1px solid var(--color-border);
        background: var(--color-bg);
        overflow: hidden;
    }

    .sidebar-content {
        overflow: hidden;
        background: var(--color-surface-soft);
        min-height: 0;
    }

    .main-content {
        background: var(--color-surface);
        overflow: hidden;
        display: flex;
        flex-direction: column;
        position: relative;
    }

    .empty-state {
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        color: var(--color-text-muted);
        user-select: none;
    }

    .tabs-container {
        padding: 0 10px 10px;
    }
</style>
