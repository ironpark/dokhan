<script lang="ts">
  import { onMount } from "svelte";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import LoadProgress from "$lib/components/LoadProgress.svelte";
  import MobileLayout from "$lib/layouts/MobileLayout.svelte";
  import DesktopLayout from "$lib/layouts/DesktopLayout.svelte";
  import { DictionaryStore } from "$lib/stores/dictionary.svelte";
  import { platformStore } from "$lib/stores/platform.svelte";

  const vm = new DictionaryStore();
  let copyMessage = $state("");

  onMount(() => {
    let unlisten: (() => void) | undefined;

    (async () => {
      if (!platformStore.isMobile) {
        unlisten = await getCurrentWebview().onDragDropEvent((event) => {
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
      if (unlisten) unlisten();
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
      <section class="mobile-pick-shell" aria-label="ZIP 선택">
        <h1>독한 사전</h1>
        <p>german.kr에서 제공하는 사전 압축파일(ZIP)을 선택해 주세요.</p>
        <button type="button" class="pick-btn" onclick={onPickZipClick}
          >ZIP 파일 선택</button
        >
      </section>
    {:else}
      <div class:drag-over={vm.dragOver} class="drop-shell">
        <h1>독한 사전</h1>
        <p>german.kr에서 제공하는 사전 압축파일(ZIP)을 이 영역에 드롭해 주세요.</p>
      </div>
    {/if}
  {:else if platformStore.isMobile}
    <MobileLayout {vm} />
  {:else}
    <DesktopLayout {vm} />
  {/if}
</main>

<style>
  :root {
    /* New Design Tokens */
    --bg: #f5f5f7;
    --surface: #ffffff;
    --line: #d1d1d6; /* Apple-like separator */
    --text: #1c1c1e;
    --muted: #8e8e93;
    --accent: #007aff; /* Apple Blue */
    --danger: #ff3b30;

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
    height: 100vh;
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

  .drop-shell {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    min-height: 0;
    text-align: center;
    background: var(--surface);
    transition: background 0.2s;
  }

  .mobile-pick-shell {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    flex: 1;
    min-height: 0;
    text-align: center;
    background: var(--surface);
    padding: 24px;
  }

  .mobile-pick-shell h1 {
    font-size: 24px;
    margin: 0;
  }

  .mobile-pick-shell p {
    margin: 0;
    color: var(--muted);
    font-size: 14px;
  }

  .drag-over {
    background: #eefbff;
  }

  .drop-shell h1 {
    font-size: 24px;
    margin-bottom: 10px;
  }

  .pick-btn {
    margin-top: 20px;
    padding: 10px 20px;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    cursor: pointer;
  }
</style>
