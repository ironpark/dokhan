<script lang="ts">
  import { onMount } from 'svelte';
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import LoadProgress from '$lib/components/LoadProgress.svelte';
  import TabBar from '$lib/components/TabBar.svelte';
  import ContentPanel from '$lib/components/ContentPanel.svelte';
  import IndexPanel from '$lib/components/IndexPanel.svelte';
  import SearchPanel from '$lib/components/SearchPanel.svelte';
  import ReaderPane from '$lib/components/ReaderPane.svelte';
  import { DictionaryStore } from '$lib/stores/dictionary.svelte';

  const vm = new DictionaryStore();
  let copyMessage = $state('');
  let zipInput = $state<HTMLInputElement | null>(null);

  onMount(() => {
    let unlisten: (() => void) | undefined;

    (async () => {
      unlisten = await getCurrentWebview().onDragDropEvent((event) => {
        const payload = event.payload;
        if (payload.type === 'over') {
          vm.dragOver = true;
          return;
        }
        if (payload.type === 'drop') {
          vm.dragOver = false;
          const first = payload.paths?.[0];
          if (first) void vm.useZipPath(first);
          return;
        }
        vm.dragOver = false;
      });

      await vm.tryAutoBootDefaultZip();
    })();

    return () => {
      vm.dispose();
      if (unlisten) unlisten();
    };
  });

  async function copyErrorText() {
    if (!vm.error) return;
    const text = vm.error;
    try {
      await writeText(text);
      copyMessage = '복사됨';
      setTimeout(() => {
        copyMessage = '';
      }, 1200);
      return;
    } catch {
      // Fallback to Web Clipboard API.
    }

    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(text);
        copyMessage = '복사됨';
        setTimeout(() => {
          copyMessage = '';
        }, 1200);
        return;
      }
    } catch {
      // Fallback to execCommand.
    }

    try {
      const textarea = document.createElement('textarea');
      textarea.value = text;
      textarea.setAttribute('readonly', '');
      textarea.style.position = 'fixed';
      textarea.style.opacity = '0';
      textarea.style.pointerEvents = 'none';
      document.body.appendChild(textarea);
      textarea.select();
      textarea.setSelectionRange(0, text.length);
      const ok = document.execCommand('copy');
      document.body.removeChild(textarea);
      copyMessage = ok ? '복사됨' : '복사 실패';
    } catch {
      copyMessage = '복사 실패';
    }
  }

  async function onPickZipClick() {
    const isAndroid = /\bAndroid\b/i.test(navigator.userAgent);
    if (isAndroid && zipInput) {
      try {
        if (typeof zipInput.showPicker === 'function') {
          await zipInput.showPicker();
        } else {
          zipInput.click();
        }
      } catch {
        zipInput.click();
      }
      return;
    }
    await vm.pickZipFile();
  }

  async function onZipInputChange(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    await vm.useZipFile(file);
    input.value = '';
  }
</script>

<main class="app-shell">
  {#if vm.error}
    <div class="error-box">
      <pre>{vm.error}</pre>
      <button type="button" class="copy-btn" onclick={copyErrorText}>복사</button>
      {#if copyMessage}
        <small>{copyMessage}</small>
      {/if}
    </div>
  {/if}

  <LoadProgress visible={vm.showProgress} progress={vm.progress} />

  {#if !vm.masterSummary}
    <div
      class:drag-over={vm.dragOver}
      class="drop-shell"
    >
      <h1>사전 ZIP 파일을 드롭하세요</h1>
      <p>앱 창 위로 `dictionary_v77.zip` 파일을 끌어 놓으면 로딩을 시작합니다.</p>
      <button type="button" class="pick-btn" onclick={onPickZipClick}>ZIP 파일 선택</button>
      <input
        bind:this={zipInput}
        class="zip-input"
        type="file"
        accept=".zip,application/zip"
        onchange={onZipInputChange}
      />
    </div>
  {:else}
    <section class="workspace">
      <aside class="navigator">
        <TabBar activeTab={vm.activeTab} onChange={(tab) => (vm.activeTab = tab)} />

        {#if vm.activeTab === 'content'}
          <ContentPanel items={vm.contents} selectedLocal={vm.selectedContentLocal} onOpen={vm.openContent} />
        {:else if vm.activeTab === 'index'}
          <IndexPanel
            query={vm.indexPrefix}
            rows={vm.indexRows}
            loading={vm.indexLoading}
            selectedId={vm.selectedEntryId}
            onQueryChange={vm.handleIndexQueryChange}
            onOpen={vm.openEntry}
          />
        {:else}
          <SearchPanel
            query={vm.searchQuery}
            rows={vm.searchRows}
            loading={vm.loading}
            selectedId={vm.selectedEntryId}
            onQueryChange={(value) => (vm.searchQuery = value)}
            onSubmit={vm.doSearch}
            onOpen={vm.openEntry}
          />
        {/if}
      </aside>

      <ReaderPane
        mode={vm.detailMode}
        selectedContent={vm.selectedContent}
        selectedEntry={vm.selectedEntry}
        highlightQuery={vm.committedSearchQuery}
        onOpenHref={vm.openInlineHref}
        onResolveImageHref={vm.resolveInlineImageHref}
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
    position: relative;
    padding-right: 68px;
    max-height: 140px;
    overflow: auto;
  }

  .error-box pre {
    margin: 0;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    font: inherit;
  }

  .copy-btn {
    position: absolute;
    top: 8px;
    right: 10px;
    border: 1px solid #d9b5b5;
    background: #fff;
    color: #6d2222;
    font: inherit;
    font-size: 12px;
    padding: 4px 8px;
    cursor: pointer;
  }

  .error-box small {
    display: block;
    margin-top: 6px;
    color: #7a3a3a;
    font-size: 12px;
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

  .pick-btn {
    justify-self: center;
    margin-top: 8px;
    border: 1px solid var(--line);
    background: #f4efe2;
    color: var(--text);
    font: inherit;
    font-weight: 700;
    padding: 9px 14px;
    cursor: pointer;
  }

  .pick-btn:hover {
    background: #ece5d6;
  }

  .zip-input {
    display: none;
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
