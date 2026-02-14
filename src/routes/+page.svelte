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
    MasterFeatureSummary,
    SearchHit,
    Tab
  } from '$lib/types/dictionary';

  let loading = $state(false);
  let error = $state('');
  let debugRoot = $state('');
  let zipPath = $state('asset/dictionary_v77.zip');
  let activeTab = $state<Tab>('content');

  let masterSummary = $state<MasterFeatureSummary | null>(null);
  let contents = $state<ContentItem[]>([]);
  let indexRows = $state<DictionaryIndexEntry[]>([]);
  let searchRows = $state<SearchHit[]>([]);

  let indexPrefix = $state('');
  let searchQuery = $state('');

  let selectedContent = $state<ContentPage | null>(null);
  let selectedEntry = $state<EntryDetail | null>(null);
  let detailMode = $state<DetailMode>('none');

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
      await invoke<string>('start_master_build', { debugRoot });
      while (true) {
        const status = await invoke<BuildStatus>('get_master_build_status', { debugRoot });
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
        invoke<ContentItem[]>('get_master_contents', { debugRoot }),
        invoke<DictionaryIndexEntry[]>('get_index_entries', { prefix: '', limit: 120, debugRoot })
      ]);

      contents = nextContents;
      indexRows = nextIndex;
      searchRows = [];
      selectedEntry = null;
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
    debugRoot = path;
    const ok = await withLoading(() => invoke<MasterFeatureSummary>('analyze_zip_dataset', { zipPath: path }));
    if (!ok) return;
    await bootMasterFeatures();
  }

  async function openContent(local: string) {
    const page = await withLoading(() => invoke<ContentPage>('get_content_page', { local, debugRoot }));
    if (!page) return;
    selectedContent = page;
    selectedEntry = null;
    detailMode = 'content';
  }

  async function openEntry(id: number) {
    const entry = await withLoading(() => invoke<EntryDetail>('get_entry_detail', { id, debugRoot }));
    if (!entry) return;
    selectedEntry = entry;
    selectedContent = null;
    detailMode = 'entry';
  }

  async function loadIndexByPrefix(event: Event) {
    event.preventDefault();
    const rows = await withLoading(() =>
      invoke<DictionaryIndexEntry[]>('get_index_entries', {
        prefix: indexPrefix,
        limit: 200,
        debugRoot
      })
    );
    if (rows) indexRows = rows;
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
        debugRoot
      })
    );
    if (rows) searchRows = rows;
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
          <ContentPanel items={contents} onOpen={openContent} />
        {:else if activeTab === 'index'}
          <IndexPanel
            query={indexPrefix}
            rows={indexRows}
            {loading}
            onQueryChange={(value) => (indexPrefix = value)}
            onSubmit={loadIndexByPrefix}
            onOpen={openEntry}
          />
        {:else}
          <SearchPanel
            query={searchQuery}
            rows={searchRows}
            {loading}
            onQueryChange={(value) => (searchQuery = value)}
            onSubmit={doSearch}
            onOpen={openEntry}
          />
        {/if}
      </aside>

      <ReaderPane mode={detailMode} {selectedContent} {selectedEntry} />
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
    --r-sm: 7px;
    --r-md: 10px;
    --r-lg: 12px;
    font-family: 'Alegreya Sans', 'IBM Plex Sans', 'Pretendard', 'Noto Sans KR', sans-serif;
  }

  .app-shell {
    height: 100vh;
    margin: 0;
    padding: 18px;
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
    border-radius: var(--r-md);
    background: #fff2f1;
    color: var(--danger);
    padding: 8px 10px;
    white-space: pre-wrap;
  }

  .drop-shell {
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
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
    border-radius: var(--r-lg);
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
