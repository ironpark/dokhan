<script lang="ts">
    import { DictionaryStore } from "$lib/stores/dictionary.svelte";
    import SearchPanel from "$lib/components/SearchPanel.svelte";
    import IndexPanel from "$lib/components/IndexPanel.svelte";
    import ReaderPane from "$lib/components/ReaderPane.svelte";
    import ContentPanel from "$lib/components/ContentPanel.svelte";
    import TabBar from "$lib/components/TabBar.svelte";
    import TitleToolbar from "$lib/components/TitleToolbar.svelte";

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
                <SearchPanel
                    query={vm.searchQuery}
                    rows={vm.searchRows}
                    loading={vm.loading}
                    selectedId={vm.selectedEntryId}
                    onQueryChange={(value) => (vm.searchQuery = value)}
                    onSubmit={(e) => vm.doSearch(e)}
                    onOpen={(id) => vm.openEntry(id)}
                />
            {/if}
        </div>
    </aside>

    <main class="main-content">
        {#if !vm.selectedContent && !vm.selectedEntry}
            <div class="empty-state">
                <div class="logo-placeholder">
                    <h1>독한 사전</h1>
                    <p>항목을 선택하면 본문이 표시됩니다.</p>
                </div>
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
        background: var(--color-bg);
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

    .logo-placeholder {
        text-align: center;
    }

    .logo-placeholder h1 {
        font-size: 24px;
        color: var(--color-border);
        margin: 0 0 10px 0;
    }

    .tabs-container {
        padding: 0 10px 10px;
    }
</style>
