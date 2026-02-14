<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import LoadProgress from '$lib/components/dictionary/LoadProgress.svelte';
  import TabBar from '$lib/components/dictionary/TabBar.svelte';
  import ContentPanel from '$lib/components/dictionary/ContentPanel.svelte';
  import IndexPanel from '$lib/components/dictionary/IndexPanel.svelte';
  import SearchPanel from '$lib/components/dictionary/SearchPanel.svelte';
  import ReaderPane from '$lib/components/dictionary/ReaderPane.svelte';
  import type {
    BuildProgress,
    BuildStatus,
    ContentItem,
    ContentPage,
    DetailMode,
    DictionaryIndexEntry,
    EntryDetail,
    DictionaryLinkTarget,
    MasterFeatureSummary,
    SearchHit,
    Tab
  } from '$lib/types/dictionary';

  let loading = $state(false);
  let error = $state('');
  let zipPath = $state('asset/dictionary_v77.zip');
  let activeTab = $state<Tab>('content');

  let masterSummary = $state<MasterFeatureSummary | null>(null);
  let contents = $state<ContentItem[]>([]);
  let indexRows = $state<DictionaryIndexEntry[]>([]);
  let searchRows = $state<SearchHit[]>([]);

  let indexPrefix = $state('');
  let searchQuery = $state('');
  let indexLoading = $state(false);

  let indexDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  let indexRequestSeq = 0;

  let selectedContent = $state<ContentPage | null>(null);
  let selectedEntry = $state<EntryDetail | null>(null);
  let detailMode = $state<DetailMode>('none');
  let selectedContentLocal = $state('');
  let selectedEntryId = $state<number | null>(null);

  let progress = $state<BuildProgress | null>(null);
  let showProgress = $state(false);
  let dragOver = $state(false);

  onMount(() => {
    let unlisten: (() => void) | undefined;

    (async () => {
      unlisten = await getCurrentWebview().onDragDropEvent((event) => {
        const payload = event.payload;
        if (payload.type === 'over') {
          dragOver = true;
          return;
        }
        if (payload.type === 'drop') {
          dragOver = false;
          const first = payload.paths?.[0];
          if (first) void useZipPath(first);
          return;
        }
        dragOver = false;
      });
    })();

    return () => {
      if (indexDebounceTimer) {
        clearTimeout(indexDebounceTimer);
        indexDebounceTimer = null;
      }
      if (unlisten) unlisten();
    };
  });

  async function withLoading<T>(task: () => Promise<T>): Promise<T | undefined> {
    loading = true;
    error = '';
    try {
      return await task();
    } catch (e) {
      error = String(e);
      return undefined;
    } finally {
      loading = false;
    }
  }

  async function bootMasterFeatures() {
    loading = true;
    error = '';
    showProgress = true;
    progress = { phase: 'start', current: 0, total: 1, message: '초기화 중' };
    try {
      await invoke<string>('start_master_build', { zipPath });
      while (true) {
        const status = await invoke<BuildStatus>('get_master_build_status', { zipPath });
        progress = {
          phase: status.phase,
          current: status.current,
          total: status.total,
          message: status.message
        };

        if (status.done) {
          if (!status.success) {
            throw new Error(status.error ?? '빌드 실패');
          }
          masterSummary = status.summary;
          break;
        }
        await new Promise((resolve) => setTimeout(resolve, 80));
      }

      const [nextContents, nextIndex] = await Promise.all([
        invoke<ContentItem[]>('get_master_contents', { zipPath }),
        invoke<DictionaryIndexEntry[]>('get_index_entries', { prefix: '', limit: null, zipPath })
      ]);

      contents = nextContents;
      indexRows = nextIndex;
      searchRows = [];
      selectedEntry = null;
      selectedEntryId = null;
      detailMode = 'none';

      if (contents.length) {
        await openContent(contents[0].local);
      }
    } catch (e) {
      error = String(e);
    } finally {
      showProgress = false;
      loading = false;
    }
  }

  async function useZipPath(path: string) {
    if (!path.toLowerCase().endsWith('.zip')) {
      error = 'ZIP 파일만 입력할 수 있습니다.';
      return;
    }
    zipPath = path;
    const ok = await withLoading(() => invoke<MasterFeatureSummary>('analyze_zip_dataset', { zipPath: path }));
    if (!ok) return;
    await bootMasterFeatures();
  }

  async function openContent(local: string, sourcePath: string | null = null) {
    const page = await withLoading(() =>
      invoke<ContentPage>('get_content_page', { local, sourcePath, zipPath })
    );
    if (!page) return;
    selectedContent = page;
    selectedContentLocal = local;
    selectedEntry = null;
    selectedEntryId = null;
    detailMode = 'content';
  }

  async function openEntry(id: number) {
    selectedEntryId = id;
    selectedContentLocal = '';
    const entry = await withLoading(() => invoke<EntryDetail>('get_entry_detail', { id, zipPath }));
    if (!entry) return;
    selectedEntry = entry;
    selectedContent = null;
    detailMode = 'entry';
  }

  async function loadIndexByPrefix(prefix: string) {
    if (!masterSummary) return;
    const trimmed = prefix.trim();
    const requestSeq = ++indexRequestSeq;
    indexLoading = true;
    try {
      const rows = await invoke<DictionaryIndexEntry[]>('get_index_entries', {
        prefix: trimmed,
        limit: trimmed ? 500 : null,
        zipPath
      });
      if (requestSeq === indexRequestSeq && indexPrefix.trim() === trimmed) {
        indexRows = rows;
      }
    } catch (e) {
      error = String(e);
    } finally {
      if (requestSeq === indexRequestSeq) indexLoading = false;
    }
  }

  function handleIndexQueryChange(value: string) {
    indexPrefix = value;
    if (indexDebounceTimer) clearTimeout(indexDebounceTimer);
    indexDebounceTimer = setTimeout(() => {
      void loadIndexByPrefix(value);
    }, 120);
  }

  async function doSearch(event: Event) {
    event.preventDefault();
    if (!searchQuery.trim()) {
      searchRows = [];
      return;
    }
    const rows = await withLoading(() =>
      invoke<SearchHit[]>('search_entries', {
        query: searchQuery,
        limit: 200,
        zipPath
      })
    );
    if (rows) searchRows = rows;
  }

  async function openInlineHref(href: string, currentSourcePath: string | null, currentLocal: string | null) {
    const target = await withLoading(() =>
      invoke<DictionaryLinkTarget>('resolve_link_target', {
        href,
        currentSourcePath,
        currentLocal,
        zipPath
      })
    );
    if (!target) return;
    if (target.kind === 'content') {
      await openContent(target.local, target.sourcePath);
      return;
    }
    await openEntry(target.id);
  }

  async function resolveInlineImageHref(
    href: string,
    currentSourcePath: string | null,
    currentLocal: string | null
  ): Promise<string | null> {
    try {
      return await invoke<string>('resolve_media_data_url', {
        href,
        currentSourcePath,
        currentLocal,
        zipPath
      });
    } catch {
      return null;
    }
  }
</script>

<main class="app-shell">
  {#if error}
    <p class="error-box">{error}</p>
  {/if}

  <LoadProgress visible={showProgress} {progress} />

  {#if !masterSummary}
    <div
      class:drag-over={dragOver}
      class="drop-shell"
    >
      <h1>사전 ZIP 파일을 드롭하세요</h1>
      <p>앱 창 위로 `dictionary_v77.zip` 파일을 끌어 놓으면 로딩을 시작합니다.</p>
    </div>
  {:else}
    <section class="workspace">
      <aside class="navigator">
        <TabBar {activeTab} onChange={(tab) => (activeTab = tab)} />

        {#if activeTab === 'content'}
          <ContentPanel items={contents} selectedLocal={selectedContentLocal} onOpen={openContent} />
        {:else if activeTab === 'index'}
          <IndexPanel
            query={indexPrefix}
            rows={indexRows}
            loading={indexLoading}
            selectedId={selectedEntryId}
            onQueryChange={handleIndexQueryChange}
            onOpen={openEntry}
          />
        {:else}
          <SearchPanel
            query={searchQuery}
            rows={searchRows}
            {loading}
            selectedId={selectedEntryId}
            onQueryChange={(value) => (searchQuery = value)}
            onSubmit={doSearch}
            onOpen={openEntry}
          />
        {/if}
      </aside>

      <ReaderPane
        mode={detailMode}
        {selectedContent}
        {selectedEntry}
        onOpenHref={openInlineHref}
        onResolveImageHref={resolveInlineImageHref}
      />
    </section>
  {/if}
</main>

<style>
  :root {
    --bg: #efece3;
    --surface: #fffdf7;
    --line: #d4ccbc;
    --text: #1d1a15;
    --muted: #6f6759;
    --accent: #0f6c58;
    --danger: #992f2f;
    font-family: 'Alegreya Sans', 'IBM Plex Sans', 'Pretendard', 'Noto Sans KR', sans-serif;
  }

  :global(html, body) {
    margin: 0;
    padding: 0;
    height: 100%;
    overflow: hidden;
    overscroll-behavior-y: none;
    overscroll-behavior-x: none;
  }

  .app-shell {
    height: 100vh;
    margin: 0;
    padding: 0;
    overflow: hidden;
    overscroll-behavior: none;
    color: var(--text);
    background:
      radial-gradient(1000px 420px at 0% 0%, #dcebe4 0%, transparent 60%),
      radial-gradient(900px 360px at 100% 100%, #efe0c7 0%, transparent 60%),
      var(--bg);
    display: grid;
    gap: 12px;
    overflow: hidden;
  }

  .error-box {
    margin: 0;
    border: 1px solid #e7c4c4;
    background: #fff2f1;
    color: var(--danger);
    padding: 8px 10px;
    white-space: pre-wrap;
  }

  .drop-shell {
    border: 1px solid var(--line);
    background: var(--surface);
    min-height: 70vh;
    display: grid;
    place-content: center;
    gap: 8px;
    text-align: center;
    padding: 20px;
    transition: border-color 120ms ease, background-color 120ms ease;
  }

  .drop-shell h1 {
    margin: 0;
    font-size: 28px;
    letter-spacing: -0.01em;
  }

  .drop-shell p {
    margin: 0;
    color: var(--muted);
  }

  .drag-over {
    border-color: var(--accent);
    background: #f5fbf8;
  }

  .workspace {
    border: 1px solid var(--line);
    background: var(--surface);
    overflow: hidden;
    min-height: 66vh;
    display: grid;
    grid-template-columns: minmax(270px, 360px) 1fr;
  }

  .navigator {
    border-right: 1px solid var(--line);
    min-height: 0;
    display: grid;
    grid-template-rows: auto 1fr;
  }

  @media (max-width: 980px) {
    .workspace {
      grid-template-columns: 1fr;
    }

    .navigator {
      border-right: 0;
      border-bottom: 1px solid var(--line);
      min-height: 44vh;
    }

    .drop-shell h1 {
      font-size: 24px;
    }
  }

  @media (max-width: 640px) {
    .app-shell {
      padding: 10px;
    }
  }
</style>
