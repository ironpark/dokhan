<script lang="ts">
    import { onMount, tick } from "svelte";
    import { isTauri } from "@tauri-apps/api/core";
    import ChevronRight from "@lucide/svelte/icons/chevron-right";
    import Search from "@lucide/svelte/icons/search";
    import type { DictionaryStore } from "$lib/stores/dictionaryStore.svelte";
    import ReaderPane from "$lib/components/ReaderPane.svelte";
    import SearchPanel from "$lib/components/SearchPanel.svelte";
    import IndexPanel from "$lib/components/IndexPanel.svelte";
    import LibraryPanel from "$lib/components/LibraryPanel.svelte";
    import TitleToolbar from "$lib/components/TitleToolbar.svelte";

    let { dictionaryStore }: { dictionaryStore: DictionaryStore } = $props();

    let searchPanelHost = $state<HTMLElement | null>(null);
    const inTauri = isTauri();
    // Expansion is tied to the ZIP it was opened for, so switching sources collapses it.
    let expandedContentsZip = $state<string | null | undefined>(undefined);
    let showAllContents = $derived(expandedContentsZip === dictionaryStore.zipPath);
    let showReader = $derived(
        dictionaryStore.selectedEntryId !== null || !!dictionaryStore.selectedContentLocal,
    );
    let recentItems = $derived(dictionaryStore.recentViews.slice(0, 4));
    let visibleContents = $derived(
        showAllContents ? dictionaryStore.contents : dictionaryStore.contents.slice(0, 12),
    );
    let readerTitle = $derived(
        dictionaryStore.selectedEntry?.headword ??
        dictionaryStore.selectedContent?.title ??
        "본문을 불러오는 중",
    );
    let readerHistoryArmed = false;

    function handleBack() {
        if (!showReader) return;
        if (inTauri) dictionaryStore.closeDetail();
        else history.back();
    }

    async function openSearch() {
        dictionaryStore.setMobileTab("search");
        await tick();
        searchPanelHost?.querySelector<HTMLInputElement>("input")?.focus();
    }

    onMount(() => {
        if (inTauri) return;
        const onPopState = () => {
            if (dictionaryStore.selectedEntryId !== null || dictionaryStore.selectedContentLocal) {
                dictionaryStore.closeDetail();
                return;
            }
            if (dictionaryStore.mobileTab !== "home") {
                dictionaryStore.setMobileTab("home");
            }
        };
        window.addEventListener("popstate", onPopState);
        return () => {
            window.removeEventListener("popstate", onPopState);
        };
    });

    $effect(() => {
        if (!inTauri && showReader && !readerHistoryArmed) {
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
    <div class="content-area">
        {#if showReader}
            <div class="reader-overlay">
                <header class="reader-header">
                    <button
                        type="button"
                        class="back-btn"
                        onclick={handleBack}
                        aria-label="목록으로 돌아가기"
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
                        <span>목록</span>
                    </button>
                    <span class="header-title" title={readerTitle}>{readerTitle}</span>
                </header>
                <div class="reader-content">
                    {#if dictionaryStore.isOpeningDetail && !dictionaryStore.selectedEntry && !dictionaryStore.selectedContent}
                        <div class="detail-loading" role="status">본문을 불러오는 중입니다.</div>
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
                        onAddBookmarkToFolder={(folderId) =>
                            dictionaryStore.addCurrentFavoriteToFolder(folderId)}
                        preprocessEnabled={dictionaryStore.preprocessEnabled}
                        onTogglePreprocess={() =>
                            dictionaryStore.setPreprocessEnabled(!dictionaryStore.preprocessEnabled)}
                        markerPreprocessEnabled={dictionaryStore.markerPreprocessEnabled}
                        onToggleMarkerPreprocess={() =>
                            dictionaryStore.setMarkerPreprocessEnabled(!dictionaryStore.markerPreprocessEnabled)}
                        readerFontSize={dictionaryStore.readerFontSize}
                        readerLineHeight={dictionaryStore.readerLineHeight}
                        readerWidth={dictionaryStore.readerWidth}
                        onReaderFontSizeChange={(value) =>
                            dictionaryStore.setReaderFontSize(value)}
                        onReaderLineHeightChange={(value) =>
                            dictionaryStore.setReaderLineHeight(value)}
                        onReaderWidthChange={(value) =>
                            dictionaryStore.setReaderWidth(value)}
                    />
                    {/if}
                </div>
            </div>
        {:else}
            <TitleToolbar
                title="독한 사전"
                subtitle={dictionaryStore.activeZipName}
                compact={true}
                showZipAction={true}
                onPickZip={() => dictionaryStore.pickZipFile()}
            />

            {#if dictionaryStore.mobileTab === "home"}
                <div class="home-view">
                    <div class="hero">
                        <p class="hero-kicker">독일어 · 한국어</p>
                        <h2>무엇을 찾으세요?</h2>
                        <p>단어를 검색하거나 목차에서 내용을 살펴보세요.</p>
                    </div>
                    <button type="button" class="search-launch" onclick={openSearch}>
                        <Search aria-hidden="true" size={21} />
                        <span>독일어·한국어 검색</span>
                        <ChevronRight class="launch-arrow" aria-hidden="true" size={18} />
                    </button>

                    {#if recentItems.length}
                        <section class="home-section" aria-labelledby="recent-heading">
                            <div class="section-heading">
                                <h3 id="recent-heading">최근 열람</h3>
                                <span>이어 읽기</span>
                            </div>
                            <ul class="home-list">
                                {#each recentItems as item (item.key)}
                                    <li>
                                        <button type="button" class="home-list-button" onclick={() => dictionaryStore.openRecentView(item)}>
                                            <span class="list-copy"><strong>{item.label}</strong><small>{item.kind === "entry" ? "표제어" : "목차"}</small></span>
                                            <ChevronRight aria-hidden="true" size={18} />
                                        </button>
                                    </li>
                                {/each}
                            </ul>
                        </section>
                    {/if}

                    <section class="home-section" aria-labelledby="contents-heading">
                        <div class="section-heading">
                            <h3 id="contents-heading">목차</h3>
                            <span>{dictionaryStore.contents.length}개 항목</span>
                        </div>
                        {#if visibleContents.length}
                            <ul class="home-list">
                                {#each visibleContents as item (item.local)}
                                    <li>
                                        <button type="button" class="home-list-button" onclick={() => dictionaryStore.openContent(item.local)}>
                                            <span class="list-copy"><strong>{item.title}</strong></span>
                                            <ChevronRight aria-hidden="true" size={18} />
                                        </button>
                                    </li>
                                {/each}
                            </ul>
                            {#if dictionaryStore.contents.length > 12}
                                <button type="button" class="more-contents" aria-expanded={showAllContents} onclick={() => (expandedContentsZip = showAllContents ? undefined : dictionaryStore.zipPath)}>
                                    {showAllContents ? "목차 접기" : `목차 전체 보기 (${dictionaryStore.contents.length}개)`}
                                </button>
                            {/if}
                        {:else}
                            <p class="home-empty">표시할 목차가 없습니다. 검색에서 단어를 찾아보세요.</p>
                        {/if}
                    </section>
                </div>
            {:else if dictionaryStore.mobileTab === "search"}
                <div class="panel-container" bind:this={searchPanelHost}>
                    <SearchPanel
                        query={dictionaryStore.searchQuery}
                        committedQuery={dictionaryStore.committedSearchQuery}
                        rows={dictionaryStore.searchRows}
                        loading={dictionaryStore.isSearching}
                        inputAtBottom={true}
                        recentUnderInput={true}
                        recentSearches={dictionaryStore.recentSearches}
                        selectedId={dictionaryStore.selectedEntryId}
                        onQueryChange={(value) => dictionaryStore.setSearchQuery(value)}
                        onSubmit={() => dictionaryStore.submitSearch()}
                        onPickRecentSearch={(query) => dictionaryStore.useRecentSearch(query)}
                        onOpen={(id) => dictionaryStore.openEntry(id)}
                    />
                </div>
            {:else if dictionaryStore.mobileTab === "index"}
                <div class="panel-container">
                    <IndexPanel
                        query={dictionaryStore.indexPrefix}
                        rows={dictionaryStore.indexRows}
                        loading={dictionaryStore.indexLoading}
                        inputAtBottom={true}
                        selectedId={dictionaryStore.selectedEntryId}
                        onQueryChange={(value) => dictionaryStore.setIndexPrefix(value)}
                        onOpen={(id) => dictionaryStore.openEntry(id)}
                    />
                </div>
            {:else if dictionaryStore.mobileTab === "favorites"}
                <div class="panel-container">
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
                </div>
            {/if}
        {/if}
    </div>

    {#if !showReader}
        <nav class="bottom-nav" aria-label="주요 메뉴">
            <button
                class:active={dictionaryStore.mobileTab === "home"}
                aria-current={dictionaryStore.mobileTab === "home" ? "page" : undefined}
                onclick={() => dictionaryStore.setMobileTab("home")}
            >
                <div class="icon" aria-hidden="true">
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
                class:active={dictionaryStore.mobileTab === "search"}
                aria-current={dictionaryStore.mobileTab === "search" ? "page" : undefined}
                onclick={() => dictionaryStore.setMobileTab("search")}
            >
                <div class="icon" aria-hidden="true">
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
                class:active={dictionaryStore.mobileTab === "index"}
                aria-current={dictionaryStore.mobileTab === "index" ? "page" : undefined}
                onclick={() => dictionaryStore.setMobileTab("index")}
            >
                <div class="icon" aria-hidden="true">
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
                class:active={dictionaryStore.mobileTab === "favorites"}
                aria-current={dictionaryStore.mobileTab === "favorites" ? "page" : undefined}
                onclick={() => dictionaryStore.setMobileTab("favorites")}
            >
                <div class="icon" aria-hidden="true">
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
                <span>책갈피</span>
            </button>
        </nav>
    {/if}
</div>

<style>
    .mobile-layout {
        display: grid;
        grid-template-rows: minmax(0, 1fr) auto;
        flex: 1;
        height: 100%;
        min-height: 0;
        min-width: 0;
        overflow: hidden;
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
        min-height: 0;
        min-width: 0;
    }

    .panel-container {
        flex: 1;
        min-height: 0;
        overflow: hidden;
        background: var(--color-surface);
    }

    .bottom-nav {
        display: grid;
        grid-template-columns: repeat(4, minmax(0, 1fr));
        gap: 2px;
        border-top: 1px solid var(--color-border);
        background: color-mix(in oklab, var(--color-surface), transparent 6%);
        backdrop-filter: blur(20px);
        -webkit-backdrop-filter: blur(20px);
        padding: 5px 8px calc(5px + env(safe-area-inset-bottom));
        min-height: 62px;
        align-items: center;
    }

    .bottom-nav button {
        position: relative;
        background: transparent;
        border: none;
        padding: 7px 4px;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 3px;
        font-size: 11px;
        min-height: 52px;
        color: var(--color-text-muted);
        cursor: pointer;
        border-radius: 12px;
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
        inset: 1px 2px;
        border-radius: 10px;
        background: var(--color-accent-soft);
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
        outline: 2px solid color-mix(in oklab, var(--color-accent), var(--color-surface) 35%);
        outline-offset: -2px;
    }

    .bottom-nav .icon {
        display: flex;
        align-items: center;
        justify-content: center;
        transition: transform 200ms cubic-bezier(0.16, 1, 0.3, 1);
    }

    .bottom-nav button span {
        font-weight: 600;
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
        inset: 0;
        background: var(--color-surface);
        z-index: 100;
        display: grid;
        grid-template-rows: auto minmax(0, 1fr);
        animation: slideUp 220ms cubic-bezier(0.16, 1, 0.3, 1);
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
        min-height: 54px;
        border-bottom: 1px solid var(--color-border);
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
        padding: 4px 12px;
        background: color-mix(in oklab, var(--color-surface), transparent 4%);
        backdrop-filter: blur(10px);
        z-index: 10;
    }

    .back-btn {
        background: transparent;
        border: none;
        border-radius: var(--radius-md);
        min-height: 44px;
        padding: 0 8px 0 2px;
        display: inline-flex;
        align-items: center;
        gap: 3px;
        color: var(--color-accent);
        font-size: 14px;
        font-weight: 650;
        cursor: pointer;
        touch-action: manipulation;
    }

    .back-btn:active {
        background: var(--color-accent-soft);
    }

    .back-btn:focus-visible,
    .search-launch:focus-visible,
    .home-list-button:focus-visible,
    .more-contents:focus-visible {
        outline: 2px solid var(--color-focus-ring);
        outline-offset: -2px;
    }

    .header-title {
        font-weight: 600;
        font-size: 14px;
        min-width: 0;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        color: var(--color-text-muted);
    }

    .reader-content {
        overflow: hidden;
        position: relative;
        min-height: 0;
    }

    .detail-loading {
        height: 100%;
        display: grid;
        place-items: center;
        color: var(--color-text-muted);
        font-size: var(--font-size-control-md);
    }

    .home-view {
        flex: 1;
        min-height: 0;
        padding: 22px 16px 28px;
        display: flex;
        flex-direction: column;
        gap: 20px;
        overflow-y: auto;
        overscroll-behavior-y: contain;
        -webkit-overflow-scrolling: touch;
    }

    .hero {
        padding: 4px 2px 0;
    }

    .hero .hero-kicker {
        color: var(--color-accent);
        font-size: 11px;
        font-weight: 700;
        letter-spacing: 0.06em;
        margin-bottom: 7px;
    }

    .hero h2 {
        font-size: clamp(22px, 6vw, 27px);
        line-height: 1.2;
        margin: 0 0 6px;
        letter-spacing: -0.03em;
        color: var(--color-text);
    }

    .hero p {
        margin: 0;
        color: var(--color-text-muted);
        font-size: 13px;
        line-height: 1.45;
    }

    .search-launch {
        width: 100%;
        min-height: 56px;
        border: 1px solid var(--color-border-strong);
        border-radius: var(--radius-lg);
        background: var(--color-surface);
        color: var(--color-text);
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 0 16px;
        font-size: 15px;
        font-weight: 600;
        cursor: pointer;
        box-shadow: var(--shadow-sm);
        text-align: left;
        touch-action: manipulation;
    }

    .search-launch > :global(svg:first-child) {
        color: var(--color-accent);
        flex: none;
    }

    .search-launch > :global(.launch-arrow) {
        margin-left: auto;
        color: var(--color-text-muted);
        flex: none;
    }

    .search-launch:active {
        background: var(--color-surface-hover);
    }

    .home-section {
        display: grid;
        gap: 8px;
        min-width: 0;
    }

    .section-heading {
        display: flex;
        align-items: baseline;
        justify-content: space-between;
        gap: 12px;
        padding: 0 2px;
    }

    .section-heading h3 {
        margin: 0;
        color: var(--color-text);
        font-size: 15px;
        font-weight: 700;
    }

    .section-heading span {
        color: var(--color-text-muted);
        font-size: 11px;
        white-space: nowrap;
    }

    .home-list {
        list-style: none;
        margin: 0;
        padding: 0;
        background: var(--color-surface);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-lg);
        overflow: hidden;
    }

    .home-list li + li {
        border-top: 1px solid var(--color-border);
    }

    .home-list-button {
        width: 100%;
        min-height: 54px;
        padding: 10px 14px;
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
        border: 0;
        background: transparent;
        color: var(--color-text);
        text-align: left;
        cursor: pointer;
        touch-action: manipulation;
    }

    .home-list-button:active {
        background: var(--color-surface-hover);
    }

    .home-list-button > :global(svg) {
        flex: none;
        color: var(--color-text-muted);
    }

    .list-copy {
        min-width: 0;
        display: grid;
        gap: 2px;
    }

    .list-copy strong {
        font-size: 14px;
        line-height: 1.35;
        font-weight: 600;
        overflow-wrap: anywhere;
    }

    .list-copy small {
        color: var(--color-text-muted);
        font-size: 11px;
    }

    .more-contents {
        min-height: 44px;
        border: 1px solid var(--color-border);
        border-radius: var(--radius-md);
        background: var(--color-surface);
        color: var(--color-accent);
        font-size: 13px;
        font-weight: 600;
        cursor: pointer;
        touch-action: manipulation;
    }

    .more-contents:active {
        background: var(--color-surface-hover);
    }

    .home-empty {
        margin: 0;
        padding: 16px;
        border: 1px solid var(--color-border);
        border-radius: var(--radius-lg);
        background: var(--color-surface);
        color: var(--color-text-muted);
        font-size: 13px;
    }

    @media (max-height: 540px) {
        .home-view { gap: 14px; padding-top: 14px; }
        .hero { padding-top: 0; }
        .hero h2 { font-size: 22px; }
    }

    @media (prefers-reduced-motion: reduce) {
        .reader-overlay { animation: none; }
    }

</style>
