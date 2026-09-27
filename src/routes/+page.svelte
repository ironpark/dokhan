<script lang="ts">
  import { onMount } from "svelte";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { exit, onBackButtonPress } from "@tauri-apps/api/app";
  import { isTauri } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import BookOpen from "@lucide/svelte/icons/book-open";
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import FileUp from "@lucide/svelte/icons/file-up";
  import X from "@lucide/svelte/icons/x";
  import LoadProgress from "$lib/components/LoadProgress.svelte";
  import MobileLayout from "$lib/layouts/MobileLayout.svelte";
  import DesktopLayout from "$lib/layouts/DesktopLayout.svelte";
  import { createDictionaryStore } from "$lib/stores/dictionaryStore.svelte";
  import { platformStore } from "$lib/stores/platform.svelte";

  const dictionaryStore = createDictionaryStore();
  let copyMessage = $state("");
  let copyMessageTimer: ReturnType<typeof setTimeout> | undefined;

  function showCopyMessage() {
    copyMessage = "복사됨";
    if (copyMessageTimer) clearTimeout(copyMessageTimer);
    copyMessageTimer = setTimeout(() => {
      copyMessage = "";
      copyMessageTimer = undefined;
    }, 1200);
  }

  onMount(() => {
    let unlistenDragDrop: (() => void) | undefined;
    let unlistenCloseRequest: (() => void) | undefined;
    let backButtonListener: Awaited<ReturnType<typeof onBackButtonPress>> | undefined;
    let disposed = false;

    (async () => {
      const tauri = isTauri();
      if (tauri) await platformStore.init();
      if (disposed) return;
      dictionaryStore.setAutoOpenFirstContent(!platformStore.isMobile);

      if (tauri && platformStore.platformName === "android") {
        try {
          const listener = await onBackButtonPress(() => {
            if (!dictionaryStore.handleMobileBackNavigation()) void exit(0);
          });
          if (disposed) await listener.unregister();
          else backButtonListener = listener;
        } catch (error) {
          console.error("Android 뒤로가기 리스너를 등록하지 못했습니다.", error);
        }
      }

      await dictionaryStore.bootFromManagedCache();
      if (disposed || !tauri) return;

      if (platformStore.isMobile) {
        const unlisten = await getCurrentWindow().onCloseRequested(
          (event) => {
            if (dictionaryStore.handleMobileBackNavigation()) {
              event.preventDefault();
            }
          },
        );
        if (disposed) unlisten();
        else unlistenCloseRequest = unlisten;
      } else {
        const unlisten = await getCurrentWebview().onDragDropEvent((event) => {
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
        if (disposed) unlisten();
        else unlistenDragDrop = unlisten;
      }

    })();

    return () => {
      disposed = true;
      dictionaryStore.dispose();
      if (copyMessageTimer) clearTimeout(copyMessageTimer);
      if (unlistenDragDrop) unlistenDragDrop();
      if (unlistenCloseRequest) unlistenCloseRequest();
      if (backButtonListener) void backButtonListener.unregister();
    };
  });

  async function copyErrorText() {
    if (!dictionaryStore.error) return;
    const text = dictionaryStore.error;
    try {
      await writeText(text);
      showCopyMessage();
      return;
    } catch {
      // Fallback
    }

    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(text);
        showCopyMessage();
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
      <span class="error-icon" aria-hidden="true"><CircleAlert size={19} /></span>
      <div class="error-copy">
        <strong>사전을 불러오지 못했습니다</strong>
        <p>{dictionaryStore.masterSummary
          ? '기존 사전은 계속 사용할 수 있습니다. 다시 시도하거나 다른 ZIP을 선택해 주세요.'
          : '다시 시도하거나 사전 ZIP 파일을 선택해 주세요.'}</p>
      </div>
      <button type="button" class="error-dismiss" aria-label="오류 알림 닫기" onclick={() => dictionaryStore.clearError()}><X size={17} /></button>
      <div class="error-actions">
        <button type="button" class="error-btn primary" onclick={onRetryClick}>
          다시 시도
        </button>
        <button type="button" class="error-btn" onclick={onPickZipClick}>
          ZIP 선택
        </button>
      </div>
      <details class="error-details">
        <summary>기술 오류 보기</summary>
        <pre>{dictionaryStore.error}</pre>
        <button type="button" class="error-btn" onclick={copyErrorText}>오류 복사</button>
        {#if copyMessage}<small class="copy-message" role="status">{copyMessage}</small>{/if}
      </details>
    </div>
  {/if}

  <LoadProgress visible={dictionaryStore.showProgress} progress={dictionaryStore.progress} />

  <div class="app-content" inert={dictionaryStore.showProgress}>
  {#if !dictionaryStore.masterSummary}
    <section class="entry-shell" class:mobile={platformStore.isMobile} aria-labelledby="welcome-title">
      <div class="entry-card" class:drag-over={dictionaryStore.dragOver}>
        <span class="entry-icon" aria-hidden="true"><BookOpen size={27} strokeWidth={1.8} /></span>
        <p class="eyebrow">DOKHAN · 독일어 한국어 사전</p>
        <h1 id="welcome-title">독한 사전</h1>
        <p class="entry-tagline">찾고, 읽고, 기억하는 사전</p>
        <p class="description">german.kr에서 받은 사전 ZIP 파일을 선택하면 목차와 검색 기능을 준비합니다.</p>
        <button type="button" class="pick-btn" onclick={onPickZipClick}>
          <FileUp size={18} strokeWidth={1.9} />
          <span>ZIP 파일 선택</span>
        </button>
        {#if !platformStore.isMobile}
          <p class="drop-hint">또는 ZIP 파일을 이 창에 끌어 놓으세요</p>
        {/if}
        <p class="entry-footnote">ZIP 압축을 해제하지 않고 원본 파일을 선택하세요.</p>
      </div>
    </section>
  {:else if platformStore.isMobile}
    <MobileLayout {dictionaryStore} />
  {:else}
    <DesktopLayout {dictionaryStore} />
  {/if}
  </div>
</main>

<style>
  :global(html, body) {
    margin: 0;
    padding: 0;
    height: 100%;
    overflow: hidden;
    background: var(--color-bg);
    color: var(--color-text);
  }

  .app-shell {
    height: 100dvh;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .app-content {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .error-box {
    display: grid;
    grid-template-columns: 24px minmax(0, 1fr) auto;
    grid-template-areas:
      "icon copy dismiss"
      ". actions actions"
      ". details details";
    column-gap: 12px;
    row-gap: 10px;
    padding: 14px clamp(16px, 3vw, 28px);
    max-height: min(45dvh, 360px);
    overflow-y: auto;
    border-bottom: 1px solid var(--color-danger-soft-border);
    background: var(--color-danger-soft-bg);
    color: var(--color-text);
    position: relative;
    z-index: 20;
    box-sizing: border-box;
  }

  .error-icon {
    grid-area: icon;
    display: flex;
    align-items: flex-start;
    color: var(--color-danger);
    padding-top: 2px;
  }

  .error-copy { grid-area: copy; }

  .error-copy strong {
    font-size: 14px;
    line-height: 1.4;
  }

  .error-copy p {
    margin: 0;
    margin-top: 3px;
    font-size: 13px;
    line-height: 1.5;
    color: var(--color-text-muted);
  }

  .error-dismiss {
    grid-area: dismiss;
    width: 32px;
    height: 32px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--color-text-muted);
    display: grid;
    place-items: center;
    cursor: pointer;
  }

  .error-actions {
    grid-area: actions;
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .error-btn {
    border: 1px solid var(--color-border-strong);
    background: var(--color-surface);
    color: var(--color-text);
    border-radius: 8px;
    min-height: 34px;
    padding: 6px 12px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }

  .error-btn.primary {
    background: var(--color-text);
    border-color: var(--color-text);
    color: var(--color-surface);
  }

  .error-details {
    grid-area: details;
    color: var(--color-text-muted);
    font-size: 12px;
  }

  .error-details summary {
    width: fit-content;
    cursor: pointer;
  }

  .error-details pre {
    margin: 8px 0;
    padding: 10px;
    max-height: 120px;
    overflow: auto;
    border-radius: 8px;
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 11px;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .copy-message {
    margin-left: 8px;
    color: var(--color-accent);
  }

  .entry-shell {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    min-height: 0;
    background:
      radial-gradient(55rem 40rem at 10% -15%, color-mix(in oklab, var(--color-accent), transparent 91%), transparent 65%),
      radial-gradient(50rem 38rem at 90% 115%, color-mix(in oklab, var(--color-accent), transparent 95%), transparent 70%),
      var(--color-bg);
    padding: clamp(20px, 4vw, 48px);
    box-sizing: border-box;
  }

  .entry-card {
    box-sizing: border-box;
    width: min(560px, 100%);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 22px;
    box-shadow: 0 18px 54px rgba(24, 43, 55, 0.08);
    padding: clamp(28px, 5vw, 44px);
    text-align: center;
    transition:
      border-color var(--motion-base),
      box-shadow var(--motion-base);
  }

  .entry-icon {
    display: inline-grid;
    place-items: center;
    width: 58px;
    height: 58px;
    margin-bottom: 20px;
    border-radius: 17px;
    background: var(--color-accent-soft);
    color: var(--color-accent);
  }

  .eyebrow {
    margin: 0 0 10px;
    font-size: 11px;
    letter-spacing: 0.08em;
    font-weight: 700;
    color: var(--color-accent);
  }

  .entry-card h1 {
    margin: 0;
    font-size: clamp(30px, 4.5vw, 38px);
    line-height: 1.2;
    font-weight: 750;
    letter-spacing: -0.045em;
    color: var(--color-text);
  }

  .entry-tagline {
    margin: 10px 0 0;
    font-size: 16px;
    line-height: 1.45;
    font-weight: 600;
    color: var(--color-text);
  }

  .description {
    margin: 20px auto 0;
    max-width: 390px;
    color: var(--color-text-muted);
    line-height: 1.65;
    font-size: 14px;
  }

  .drop-hint {
    margin: 14px 0 0;
    padding: 12px;
    border: 1px dashed var(--color-border-strong);
    border-radius: 10px;
    background: var(--color-surface-soft);
    color: var(--color-text-muted);
    font-size: 13px;
    line-height: 1.5;
  }

  .drag-over {
    border-color: var(--color-accent);
    box-shadow: 0 0 0 5px var(--color-accent-soft), 0 18px 54px rgba(24, 43, 55, 0.1);
  }

  .entry-footnote {
    margin: 20px 0 0;
    color: var(--color-text-subtle);
    font-size: 12px;
    line-height: 1.5;
  }

  .entry-shell.mobile {
    padding-top: calc(20px + env(safe-area-inset-top));
    padding-bottom: calc(20px + env(safe-area-inset-bottom));
  }

  .pick-btn {
    margin-top: 28px;
    min-height: 48px;
    width: min(280px, 100%);
    padding: 12px 20px;
    background: var(--color-accent);
    color: #fff;
    border: none;
    border-radius: 10px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    font-size: 14px;
    font-weight: 700;
    cursor: pointer;
    box-shadow: 0 7px 18px color-mix(in oklab, var(--color-accent), transparent 77%);
    transition:
      background-color var(--motion-fast),
      transform var(--motion-fast);
  }

  .pick-btn:active {
    transform: translateY(1px);
  }

  .pick-btn:hover {
    background: var(--color-accent-hover);
  }

  .pick-btn:focus-visible,
  .error-btn:focus-visible,
  .error-dismiss:focus-visible,
  .error-details summary:focus-visible {
    outline: 3px solid var(--color-focus-ring);
    outline-offset: 2px;
  }

  @media (max-width: 540px), (max-height: 600px) {
    .entry-shell { overflow-y: auto; justify-content: flex-start; }
    .entry-card { margin: auto 0; padding: 28px 22px; border-radius: 18px; }
    .entry-icon { margin-bottom: 14px; }
    .description { margin-top: 16px; }
    .pick-btn { margin-top: 22px; }
    .error-box { grid-template-columns: 20px minmax(0, 1fr) auto; padding: 12px 14px; }
  }
</style>
