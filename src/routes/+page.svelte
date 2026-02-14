<script lang="ts">
  import { onMount } from "svelte";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import LoadProgress from "$lib/components/LoadProgress.svelte";
  import MobileLayout from "$lib/layouts/MobileLayout.svelte";
  import DesktopLayout from "$lib/layouts/DesktopLayout.svelte";
  import { DictionaryStore } from "$lib/stores/dictionary.svelte";
  import { platformStore } from "$lib/stores/platform.svelte";

  const vm = new DictionaryStore();
  let copyMessage = $state("");

  onMount(() => {
    let unlistenDragDrop: (() => void) | undefined;
    let unlistenCloseRequest: (() => void) | undefined;

    (async () => {
      await vm.bootFromManagedCache();

      if (platformStore.isMobile) {
        unlistenCloseRequest = await getCurrentWindow().onCloseRequested(
          (event) => {
            if (vm.handleMobileBackNavigation()) {
              event.preventDefault();
            }
          },
        );
      } else {
        unlistenDragDrop = await getCurrentWebview().onDragDropEvent((event) => {
          const payload = event.payload;
          if (payload.type === "over") {
            vm.dragOver = true;
            return;
          }
          if (payload.type === "drop") {
            vm.dragOver = false;
            const first = payload.paths?.[0];
            if (first) void vm.useZipPath(first);
            return;
          }
          vm.dragOver = false;
        });
      }

    })();

    return () => {
      vm.dispose();
      if (unlistenDragDrop) unlistenDragDrop();
      if (unlistenCloseRequest) unlistenCloseRequest();
    };
  });

  async function copyErrorText() {
    if (!vm.error) return;
    const text = vm.error;
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
    await vm.pickZipFile();
  }
</script>

<main class="app-shell">
  {#if vm.error}
    <div class="error-box">
      <pre>{vm.error}</pre>
      <button type="button" class="copy-btn" onclick={copyErrorText}
        >복사</button
      >
      {#if copyMessage}
        <small>{copyMessage}</small>
      {/if}
    </div>
  {/if}

  <LoadProgress visible={vm.showProgress} progress={vm.progress} />

  {#if !vm.masterSummary}
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
        <div class:drag-over={vm.dragOver} class="entry-card drop-card">
          <p class="eyebrow">Dokhan</p>
          <h1>독한 사전</h1>
          <p class="description">
            german.kr에서 제공하는 사전 압축파일(ZIP)을 이 영역에 드롭해 주세요.
          </p>
          <p class="drop-hint">Drag and Drop ZIP</p>
        </div>
      </section>
    {/if}
  {:else if platformStore.isMobile}
    <MobileLayout {vm} />
  {:else}
    <DesktopLayout {vm} />
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
    background: #ffdbdd;
    color: #c00;
    padding: 10px;
    border-bottom: 1px solid #faa;
    position: relative;
    z-index: 9999;
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
