<script lang="ts">
    import { DictionaryStore } from "$lib/stores/dictionary.svelte";
    import ReaderPane from "$lib/components/ReaderPane.svelte";
    import ContentPanel from "$lib/components/ContentPanel.svelte";
    import SearchPanel from "$lib/components/SearchPanel.svelte";
    import IndexPanel from "$lib/components/IndexPanel.svelte";
    import Input from "$lib/components/ui/Input.svelte";

    // Props
    let { vm }: { vm: DictionaryStore } = $props();

    let activeTab = $state<"home" | "search" | "index">("home");
    let showReader = $derived(
        !!(vm.selectedEntryId || vm.selectedContentLocal),
    );

    function handleBack() {
        if (showReader) {
            // "Close" reader
            vm.selectedEntryId = null;
            vm.selectedContentLocal = "";
        }
    }
</script>

<div class="mobile-layout">
    <main class="content-area">
        {#if showReader}
            <div class="reader-overlay">
                <header class="reader-header">
                    <button
                        class="back-btn"
                        onclick={handleBack}
                        aria-label="Go back"
                    >
                        <svg
                            width="24"
                            height="24"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                        >
                            <path d="M19 12H5M12 19l-7-7 7-7" />
                        </svg>
                    </button>
                    <span class="header-title">Detail</span>
                </header>
                <div class="reader-content">
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
                </div>
            </div>
        {:else if activeTab === "home"}
            <div class="home-view">
                <div class="hero">
                    <h1>German-Korean<br />Dictionary</h1>
                    <p>Simple & Fast.</p>
                </div>
                <div class="search-box">
                    <Input
                        placeholder="Search dictionary..."
                        onclick={() => (activeTab = "search")}
                        readonly
                    >
                        {#snippet icon()}
                            <svg
                                width="18"
                                height="18"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                            >
                                <circle cx="11" cy="11" r="8"></circle>
                                <line x1="21" y1="21" x2="16.65" y2="16.65"
                                ></line>
                            </svg>
                        {/snippet}
                    </Input>
                </div>

                <div class="content-list">
                    <h3>Contents</h3>
                    <ContentPanel
                        items={vm.contents}
                        selectedLocal={vm.selectedContentLocal}
                        onOpen={(local) => vm.openContent(local)}
                    />
                </div>
            </div>
        {:else if activeTab === "search"}
            <div class="panel-container">
                <SearchPanel
                    query={vm.searchQuery}
                    rows={vm.searchRows}
                    loading={vm.loading}
                    selectedId={vm.selectedEntryId}
                    onQueryChange={(value) => (vm.searchQuery = value)}
                    onSubmit={vm.doSearch}
                    onOpen={(id) => vm.openEntry(id)}
                />
            </div>
        {:else if activeTab === "index"}
            <div class="panel-container">
                <IndexPanel
                    query={vm.indexPrefix}
                    rows={vm.indexRows}
                    loading={vm.indexLoading}
                    selectedId={vm.selectedEntryId}
                    onQueryChange={vm.handleIndexQueryChange}
                    onOpen={(id) => vm.openEntry(id)}
                />
            </div>
        {/if}
    </main>

    {#if !showReader}
        <nav class="bottom-nav">
            <button
                class:active={activeTab === "home"}
                onclick={() => (activeTab = "home")}
            >
                <div class="icon">
                    <svg
                        width="24"
                        height="24"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        ><path
                            d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"
                        ></path><polyline points="9 22 9 12 15 12 15 22"
                        ></polyline></svg
                    >
                </div>
                <span>Home</span>
            </button>
            <button
                class:active={activeTab === "search"}
                onclick={() => (activeTab = "search")}
            >
                <div class="icon">
                    <svg
                        width="24"
                        height="24"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        ><circle cx="11" cy="11" r="8"></circle><line
                            x1="21"
                            y1="21"
                            x2="16.65"
                            y2="16.65"
                        ></line></svg
                    >
                </div>
                <span>Search</span>
            </button>
            <button
                class:active={activeTab === "index"}
                onclick={() => (activeTab = "index")}
            >
                <div class="icon">
                    <svg
                        width="24"
                        height="24"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        ><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path><path
                            d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"
                        ></path></svg
                    >
                </div>
                <span>Index</span>
            </button>
        </nav>
    {/if}
</div>

<style>
    .mobile-layout {
        display: grid;
        grid-template-rows: 1fr auto;
        height: 100vh;
        background: var(--color-bg);
        color: var(--color-text);
    }

    .content-area {
        overflow: hidden;
        position: relative;
        background: var(--color-bg);
        display: flex;
        flex-direction: column;
    }

    .panel-container {
        height: 100%;
        overflow: hidden;
        background: var(--color-surface);
    }

    .bottom-nav {
        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        border-top: 1px solid var(--color-border);
        background: rgba(255, 255, 255, 0.8);
        backdrop-filter: blur(20px);
        -webkit-backdrop-filter: blur(20px);
        padding-bottom: env(safe-area-inset-bottom);
        height: 60px;
        align-items: center;
    }

    .bottom-nav button {
        background: transparent;
        border: none;
        padding: 0;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 4px;
        font-size: 10px;
        color: var(--color-text-muted);
        cursor: pointer;
    }

    .bottom-nav button.active {
        color: var(--color-accent);
    }

    .bottom-nav button.active svg {
        stroke-width: 2.5;
    }

    .reader-overlay {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: var(--color-surface);
        z-index: 100;
        display: grid;
        grid-template-rows: auto 1fr;
        animation: slideUp 0.3s cubic-bezier(0.16, 1, 0.3, 1);
    }

    @keyframes slideUp {
        from {
            transform: translateY(100%);
        }
        to {
            transform: translateY(0);
        }
    }

    .reader-header {
        height: 50px;
        border-bottom: 1px solid var(--color-border);
        display: flex;
        align-items: center;
        padding: 0 8px;
        background: rgba(255, 255, 255, 0.95);
        backdrop-filter: blur(10px);
        position: sticky;
        top: 0;
        z-index: 10;
    }

    .back-btn {
        background: none;
        border: none;
        padding: 8px;
        color: var(--color-accent);
        cursor: pointer;
    }

    .header-title {
        font-weight: 600;
        font-size: 17px;
        margin-left: 8px;
    }

    .reader-content {
        overflow: hidden;
        position: relative;
    }

    .home-view {
        padding: 20px;
        display: flex;
        flex-direction: column;
        gap: 24px;
        overflow-y: auto;
    }

    .hero {
        margin-top: 20px;
    }

    .hero h1 {
        font-size: 32px;
        line-height: 1.1;
        margin: 0 0 8px 0;
        letter-spacing: -0.02em;
        color: var(--color-text);
    }

    .hero p {
        margin: 0;
        color: var(--color-text-muted);
        font-weight: 500;
    }

    .search-box {
        position: relative;
    }

    .content-list h3 {
        font-size: 20px;
        margin: 0 0 10px 0;
        color: var(--color-text);
    }
</style>
