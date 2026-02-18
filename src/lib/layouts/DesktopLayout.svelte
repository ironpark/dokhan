<script lang="ts">
    import { DictionaryStore } from "$lib/stores/dictionary.svelte";
    import SearchPanel from "$lib/components/SearchPanel.svelte";
    import IndexPanel from "$lib/components/IndexPanel.svelte";
    import ReaderPane from "$lib/components/ReaderPane.svelte";
    import ContentPanel from "$lib/components/ContentPanel.svelte";
    import LibraryPanel from "$lib/components/LibraryPanel.svelte";
    import TabBar from "$lib/components/TabBar.svelte";
    import TitleToolbar from "$lib/components/TitleToolbar.svelte";
    import EmptyState from "$lib/components/ui/EmptyState.svelte";

    let { vm }: { vm: DictionaryStore } = $props();
</script>

<div class="desktop-layout">
    <aside class="sidebar">
        <TitleToolbar
            title="독한 사전"
            subtitle="Dokhan Dictionary"
            showZipAction={true}
            onPickZip={() => vm.pickZipFile()}
        />

        <div class="tabs-container">
            <TabBar
                activeTab={vm.activeTab}
                onChange={(tab) => {
                    vm.activeTab = tab;
                }}
            />
        </div>

        <div class="sidebar-content">
            {#if vm.activeTab === "content"}
                <ContentPanel
                    items={vm.contents}
                    selectedLocal={vm.selectedContentLocal}
                    onOpen={(local) => vm.openContent(local)}
                />
            {:else if vm.activeTab === "index"}
                <IndexPanel
                    query={vm.indexPrefix}
                    rows={vm.indexRows}
                    loading={vm.indexLoading}
                    selectedId={vm.selectedEntryId}
                    onQueryChange={(val) => vm.handleIndexQueryChange(val)}
                    onOpen={(id) => vm.openEntry(id)}
                />
            {:else}
                {#if vm.activeTab === "search"}
                    <SearchPanel
                        query={vm.searchQuery}
                        rows={vm.searchRows}
                        loading={vm.isSearching}
                        recentSearches={vm.recentSearches}
                        selectedId={vm.selectedEntryId}
                        onQueryChange={(value) => vm.handleSearchQueryChange(value)}
                        onSubmit={(e) => vm.doSearch(e)}
                        onPickRecentSearch={(query) => vm.useRecentSearch(query)}
                        onOpen={(id) => vm.openEntry(id)}
                    />
                {:else}
                    <LibraryPanel
                        favorites={vm.favorites}
                        recents={vm.recentViews}
                        onOpenFavorite={(item) => vm.openFavorite(item)}
                        onOpenRecent={(item) => vm.openRecentView(item)}
                        onRemoveFavorite={(key) => vm.removeFavorite(key)}
                    />
                {/if}
            {/if}
        </div>
    </aside>

    <main class="main-content">
        {#if !vm.selectedContent && !vm.selectedEntry}
            <div class="empty-state">
                <EmptyState
                    title="본문을 표시할 항목을 선택하세요."
                    description="목차, 색인, 검색 또는 즐겨찾기에서 항목을 선택하면 여기에 표시됩니다."
                />
            </div>
        {:else}
            <ReaderPane
                mode={vm.detailMode}
                selectedContent={vm.selectedContent}
                selectedEntry={vm.selectedEntry}
                highlightQuery={vm.committedSearchQuery}
                onOpenHref={(href, path, local) =>
                    vm.openInlineHref(href, path, local)}
                onResolveImageHref={(href, path, local) =>
                    vm.resolveInlineImageHref(href, path, local)}
                isFavorite={vm.isCurrentFavorite()}
                onToggleFavorite={() => vm.toggleCurrentFavorite()}
                preprocessEnabled={vm.preprocessEnabled}
                onTogglePreprocess={() =>
                    vm.setPreprocessEnabled(!vm.preprocessEnabled)}
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
