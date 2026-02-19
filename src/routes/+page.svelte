<script lang="ts">
  import { onMount } from "svelte";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import LoadProgress from "$lib/components/LoadProgress.svelte";
  import MobileLayout from "$lib/layouts/MobileLayout.svelte";
  import DesktopLayout from "$lib/layouts/DesktopLayout.svelte";
  import { createDictionaryStore } from "$lib/stores/dictionaryStore.svelte";
  import { platformStore } from "$lib/stores/platform.svelte";

  const dictionaryStore = createDictionaryStore();
  let copyMessage = $state("");

  onMount(() => {
    let unlistenDragDrop: (() => void) | undefined;
    let unlistenCloseRequest: (() => void) | undefined;

    (async () => {
      dictionaryStore.setAutoOpenFirstContent(!platformStore.isMobile);
      await dictionaryStore.bootFromManagedCache();

      if (platformStore.isMobile) {
        unlistenCloseRequest = await getCurrentWindow().onCloseRequested(
          (event) => {
            if (dictionaryStore.handleMobileBackNavigation()) {
              event.preventDefault();
            }
          },
        );
      } else {
        unlistenDragDrop = await getCurrentWebview().onDragDropEvent((event) => {
          const payload = event.payload;
          if (payload.type === "over") {
            dictionaryStore.setDragOver(true);
            return;
          }
          if (payload.type === "drop") {
            dictionaryStore.setDragOver(false);
            const first = payload.paths?.[0];
            if (first) void dictionaryStore.useZipPath(first);
            return;
          }
          dictionaryStore.setDragOver(false);
        });
      }

    })();

    return () => {
      dictionaryStore.dispose();
      if (unlistenDragDrop) unlistenDragDrop();
      if (unlistenCloseRequest) unlistenCloseRequest();
    };
  });

  async function copyErrorText() {
    if (!dictionaryStore.error) return;
    const text = dictionaryStore.error;
    try {
      await writeText(text);
      copyMessage = "복사됨";
      setTimeout(() => {
        copyMessage = "";
      }, 1200);
      return;
    } catch {
      // Fallback
    }

    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(text);
        copyMessage = "복사됨";
        setTimeout(() => {
          copyMessage = "";
        }, 1200);
        return;
      }
    } catch {}
  }

  async function onPickZipClick() {
    await dictionaryStore.pickZipFile();
  }

  async function onRetryClick() {
    await dictionaryStore.retryLastOperation();
  }
</script>

<main class="app-shell">
  {#if dictionaryStore.error}
    <div class="error-box" role="alert" aria-live="assertive">
      <strong>작업 중 오류가 발생했습니다.</strong>
      <p>다시 시도하거나 ZIP 파일을 다시 선택해 복구해 주세요.</p>
      <div class="error-actions">
        <button type="button" class="error-btn primary" onclick={onRetryClick}>
          다시 시도
        </button>
        <button type="button" class="error-btn" onclick={onPickZipClick}>
          ZIP 다시 선택
        </button>
        <button type="button" class="error-btn" onclick={copyErrorText}>오류 복사</button>
      </div>
      <details>
        <summary>기술 오류 보기</summary>
        <pre>{dictionaryStore.error}</pre>
      </details>
      {#if copyMessage}
        <small>{copyMessage}</small>
      {/if}
    </div>
  {/if}

  <LoadProgress visible={dictionaryStore.showProgress} progress={dictionaryStore.progress} />

  {#if !dictionaryStore.masterSummary}
    {#if platformStore.isMobile}
      <section class="entry-shell mobile" aria-label="ZIP 선택">
        <div class="entry-card">
          <p class="eyebrow">Dokhan</p>
          <h1>독한 사전</h1>
          <p class="description">
            german.kr에서 제공하는 사전 압축파일(ZIP)을 선택해 주세요.
          </p>
          <button type="button" class="pick-btn" onclick={onPickZipClick}
            >ZIP 파일 선택</button
          >
        </div>
      </section>
    {:else}
      <section class="entry-shell desktop">
        <div class:drag-over={dictionaryStore.dragOver} class="entry-card drop-card">
          <p class="eyebrow">Dokhan</p>
          <h1>독한 사전</h1>
          <p class="description">
            german.kr에서 제공하는 사전 압축파일(ZIP)을 이 영역에 드롭해 주세요.
          </p>
          <button type="button" class="pick-btn" onclick={onPickZipClick}
            >ZIP 파일 선택</button
          >
          <p class="drop-hint">Drag and Drop ZIP</p>
        </div>
      </section>
    {/if}
  {:else if platformStore.isMobile}
    <MobileLayout {dictionaryStore} />
  {:else}
    <DesktopLayout {dictionaryStore} />
  {/if}
</main>

<style>
  :root {
    --bg: #f3f4f8;
    --surface: #ffffff;
    --line: #d3d8e2;
    --text: #1c1c1e;
    --muted: #6c7280;
    --accent: #2563eb;
    --danger: #ff3b30;
    --shadow: 0 10px 24px rgba(26, 34, 56, 0.1);
    --ring: rgba(37, 99, 235, 0.16);

    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
      Helvetica, Arial, sans-serif;
  }

  :global(html, body) {
    margin: 0;
    padding: 0;
    height: 100%;
    overflow: hidden;
    background: var(--bg);
    color: var(--text);
  }

  .app-shell {
    height: 100dvh;
    min-height: 100svh;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  /* Utility / Shared Styles */
  .error-box {
    background: #fff2f2;
    color: #7c1d1d;
    padding: 10px 12px;
    border-bottom: 1px solid #faa;
    position: relative;
    z-index: 9999;
    display: grid;
    gap: 8px;
  }

  .error-box strong {
    font-size: 13px;
  }

  .error-box p {
    margin: 0;
    font-size: 12px;
    color: #8f3434;
  }

  .error-box pre {
    margin: 4px 0 0;
    font-size: 11px;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .error-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .error-btn {
    border: 1px solid #eab3b3;
    background: #fff;
    color: #8c2727;
    border-radius: 8px;
    padding: 6px 10px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }

  .error-btn.primary {
    background: #c24141;
    border-color: #c24141;
    color: #fff;
  }

  .entry-shell {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    min-height: 0;
    background:
      radial-gradient(72rem 42rem at 12% -8%, #dbeafe 0%, transparent 48%),
      radial-gradient(60rem 36rem at 88% 108%, #e7f6ef 0%, transparent 42%),
      var(--bg);
    padding: 24px;
  }

  .entry-card {
    width: min(620px, 100%);
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: 10px;
    box-shadow: var(--shadow);
    padding: 28px 28px 24px;
    text-align: center;
    transition:
      border-color 160ms ease,
      transform 160ms ease,
      box-shadow 160ms ease,
      background-color 160ms ease;
  }

  .eyebrow {
    margin: 0 0 8px;
    font-size: 11px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    font-weight: 700;
    color: var(--accent);
  }

  .entry-card h1 {
    margin: 0;
    font-size: 26px;
    line-height: 1.15;
    font-weight: 700;
    color: var(--text);
  }

  .description {
    margin: 12px auto 0;
    max-width: 460px;
    color: var(--muted);
    line-height: 1.55;
    font-size: 14px;
  }

  .drop-card {
    border: 1px dashed #9db5f2;
    background: linear-gradient(180deg, #ffffff, #f8fbff);
    border-radius: 12px;
  }

  .drop-hint {
    margin: 14px 0 0;
    color: #5f6f93;
    font-size: 12px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .drag-over {
    border-color: var(--accent);
    box-shadow: 0 0 0 6px var(--ring), var(--shadow);
    transform: translateY(-2px);
    background: linear-gradient(180deg, #f8fbff, #eef5ff);
  }

  .mobile .entry-card {
    max-width: 520px;
    width: calc(100% - 20px);
    padding: 26px 22px 24px;
  }

  .entry-shell.mobile {
    padding-top: calc(24px + env(safe-area-inset-top));
    padding-bottom: calc(24px + env(safe-area-inset-bottom));
  }

  .mobile .entry-card h1 {
    margin: 0;
    font-size: 24px;
  }

  .pick-btn {
    margin-top: 16px;
    padding: 11px 20px;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: 10px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    box-shadow: 0 8px 18px rgba(37, 99, 235, 0.32);
    transition:
      transform 140ms ease,
      box-shadow 140ms ease,
      opacity 140ms ease;
  }

  .pick-btn:active {
    transform: translateY(1px);
    box-shadow: 0 4px 10px rgba(37, 99, 235, 0.22);
  }

  .pick-btn:hover {
    opacity: 0.95;
  }
</style>
