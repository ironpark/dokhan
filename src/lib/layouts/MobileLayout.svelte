<script lang="ts">
    import { onMount } from "svelte";
    import { DictionaryStore } from "$lib/stores/dictionary.svelte";
    import ReaderPane from "$lib/components/ReaderPane.svelte";
    import ContentPanel from "$lib/components/ContentPanel.svelte";
    import SearchPanel from "$lib/components/SearchPanel.svelte";
    import IndexPanel from "$lib/components/IndexPanel.svelte";
    import LibraryPanel from "$lib/components/LibraryPanel.svelte";
    import TitleToolbar from "$lib/components/TitleToolbar.svelte";

    // Props
    let { vm }: { vm: DictionaryStore } = $props();

    let showReader = $derived(
        !!(vm.selectedEntryId || vm.selectedContentLocal),
    );
    let readerHistoryArmed = false;

    function handleBack() {
        if (showReader) {
            history.back();
        }
    }

    onMount(() => {
        const onPopState = () => {
            if (vm.selectedEntryId || vm.selectedContentLocal) {
                vm.closeDetail();
                return;
            }
            if (vm.mobileTab !== "home") {
                vm.mobileTab = "home";
            }
        };
        window.addEventListener("popstate", onPopState);
        return () => {
            window.removeEventListener("popstate", onPopState);
        };
    });

    $effect(() => {
        if (showReader && !readerHistoryArmed) {
            history.pushState({ dokhanReader: true }, "");
            readerHistoryArmed = true;
            return;
        }
        if (!showReader) {
            readerHistoryArmed = false;
        }
    });
</script>

<div class="mobile-layout">
    <main class="content-area">
        {#if showReader}
            <div class="reader-overlay">
                <header class="reader-header">
                    <button
                        class="back-btn"
                        onclick={handleBack}
                        aria-label="뒤로가기"
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
                    <span class="header-title">본문</span>
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
                        isFavorite={vm.isCurrentFavorite()}
                        onToggleFavorite={() => vm.toggleCurrentFavorite()}
                    />
                </div>
            </div>
        {:else}
            <TitleToolbar
                title="독한 사전"
                subtitle="Dokhan Dictionary"
                compact={true}
                showZipAction={true}
                onPickZip={() => vm.pickZipFile()}
            />

            {#if vm.mobileTab === "home"}
                <div class="home-view">
                    <div class="hero">
                        <h1>독한 사전</h1>
                        <p>독일어-한국어 전자사전</p>
                    </div>
                    <div class="search-box">
                        <button
                            type="button"
                            class="search-launch"
                            aria-label="검색 탭으로 이동"
                            onclick={() => (vm.mobileTab = "search")}
                        >
                            <svg
                                width="18"
                                height="18"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                            >
                                <circle cx="11" cy="11" r="8"></circle>
                                <line
                                    x1="21"
                                    y1="21"
                                    x2="16.65"
                                    y2="16.65"
                                ></line>
                            </svg>
                            <span>사전 검색 열기</span>
                        </button>
                    </div>

                    <div class="content-list">
                        <h3>목차</h3>
                        <ContentPanel
                            items={vm.contents}
                            selectedLocal={vm.selectedContentLocal}
                            onOpen={(local) => vm.openContent(local)}
                        />
                    </div>
                </div>
            {:else if vm.mobileTab === "search"}
                <div class="panel-container">
                    <SearchPanel
                        query={vm.searchQuery}
                        rows={vm.searchRows}
                        loading={vm.isSearching}
                        inputAtBottom={true}
                        recentSearches={vm.recentSearches}
                        selectedId={vm.selectedEntryId}
                        onQueryChange={(value) => vm.handleSearchQueryChange(value)}
                        onSubmit={(e) => vm.doSearch(e)}
                        onPickRecentSearch={(query) => vm.useRecentSearch(query)}
                        onOpen={(id) => vm.openEntry(id)}
                    />
                </div>
            {:else if vm.mobileTab === "index"}
                <div class="panel-container">
                    <IndexPanel
                        query={vm.indexPrefix}
                        rows={vm.indexRows}
                        loading={vm.indexLoading}
                        inputAtBottom={true}
                        selectedId={vm.selectedEntryId}
                        onQueryChange={(val) => vm.handleIndexQueryChange(val)}
                        onOpen={(id) => vm.openEntry(id)}
                    />
                </div>
            {:else if vm.mobileTab === "favorites"}
                <div class="panel-container">
                    <LibraryPanel
                        favorites={vm.favorites}
                        recents={vm.recentViews}
                        onOpenFavorite={(item) => vm.openFavorite(item)}
                        onOpenRecent={(item) => vm.openRecentView(item)}
                        onRemoveFavorite={(key) => vm.removeFavorite(key)}
                    />
                </div>
            {/if}
        {/if}
    </main>

    {#if !showReader}
        <nav class="bottom-nav">
            <button
                class:active={vm.mobileTab === "home"}
                onclick={() => (vm.mobileTab = "home")}
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
                <span>홈</span>
            </button>
            <button
                class:active={vm.mobileTab === "search"}
                onclick={() => (vm.mobileTab = "search")}
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
                <span>검색</span>
            </button>
            <button
                class:active={vm.mobileTab === "index"}
                onclick={() => (vm.mobileTab = "index")}
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
                <span>색인</span>
            </button>
            <button
                class:active={vm.mobileTab === "favorites"}
                onclick={() => (vm.mobileTab = "favorites")}
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
                            d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"
                        ></path></svg
                    >
                </div>
                <span>저장</span>
            </button>
        </nav>
    {/if}
</div>

<style>
    .mobile-layout {
        display: grid;
        grid-template-rows: 1fr auto;
        height: 100dvh;
        min-height: 100svh;
        padding-top: env(safe-area-inset-top);
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
        flex: 1;
        min-height: 0;
        overflow: hidden;
        background: var(--color-surface);
    }

    .bottom-nav {
        display: grid;
        grid-template-columns: 1fr 1fr 1fr 1fr;
        gap: 6px;
        border-top: 1px solid var(--color-border);
        background: rgba(255, 255, 255, 0.88);
        backdrop-filter: blur(20px);
        -webkit-backdrop-filter: blur(20px);
        padding-top: 6px;
        padding-bottom: calc(8px + env(safe-area-inset-bottom));
        padding-left: 8px;
        padding-right: 8px;
        min-height: 60px;
        align-items: center;
    }

    .bottom-nav button {
        position: relative;
        background: transparent;
        border: none;
        padding: 8px 8px;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 4px;
        font-size: 10px;
        min-height: 52px;
        color: var(--color-text-muted);
        cursor: pointer;
        border-radius: 14px;
        overflow: hidden;
        isolation: isolate;
        -webkit-tap-highlight-color: transparent;
        touch-action: manipulation;
        transition:
            color 160ms ease,
            transform 140ms ease;
    }

    .bottom-nav button::before {
        content: "";
        position: absolute;
        inset: 3px 4px;
        border-radius: 12px;
        background: color-mix(in oklab, var(--color-accent), white 84%);
        opacity: 0;
        transform: scale(0.92);
        transition:
            opacity 180ms ease,
            transform 220ms cubic-bezier(0.16, 1, 0.3, 1);
        z-index: -1;
    }

    .bottom-nav button.active {
        color: var(--color-accent);
    }

    .bottom-nav button.active::before {
        opacity: 1;
        transform: scale(1);
    }

    .bottom-nav button:active {
        transform: scale(0.97);
    }

    .bottom-nav button:focus-visible {
        outline: 2px solid color-mix(in oklab, var(--color-accent), white 35%);
        outline-offset: -2px;
    }

    .bottom-nav .icon {
        display: flex;
        align-items: center;
        justify-content: center;
        transition: transform 200ms cubic-bezier(0.16, 1, 0.3, 1);
    }

    .bottom-nav button span {
        font-weight: 500;
        transition:
            transform 180ms ease,
            letter-spacing 180ms ease;
    }

    .bottom-nav button.active svg {
        stroke-width: 2.5;
    }

    .bottom-nav button.active .icon {
        transform: translateY(-1px);
    }

    .bottom-nav button.active span {
        transform: translateY(-0.5px);
        letter-spacing: 0.01em;
    }

    @media (prefers-reduced-motion: reduce) {
        .bottom-nav button,
        .bottom-nav button::before,
        .bottom-nav .icon,
        .bottom-nav button span {
            transition: none;
        }
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
        flex: 1;
        min-height: 0;
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

    .search-launch {
        width: 100%;
        height: 42px;
        border: 1px solid var(--color-border);
        border-radius: var(--radius-md);
        background: var(--color-surface);
        color: var(--color-text-muted);
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 0 12px;
        font-size: 14px;
        cursor: pointer;
    }

    .search-launch:active {
        background: var(--color-surface-hover);
    }

    .content-list h3 {
        font-size: 20px;
        margin: 0 0 10px 0;
        color: var(--color-text);
    }
</style>
