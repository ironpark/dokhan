<script lang="ts">
  import Download from "@lucide/svelte/icons/download";
  import X from "@lucide/svelte/icons/x";
  import Button from "$lib/components/ui/Button.svelte";
  import type { UpdateState } from "$lib/stores/updateState.svelte";

  let { updateState }: { updateState: UpdateState } = $props();

  const status = $derived(updateState.status);
  const busy = $derived(status.phase === "downloading" || status.phase === "installing");
</script>

{#if status.phase !== "idle"}
  <div class="update-banner" class:error={status.phase === "error"} role="status" aria-live="polite">
    <span class="update-icon" aria-hidden="true"><Download size={18} /></span>
    <div class="update-copy">
      {#if status.phase === "available"}
        <strong>새 버전 {status.version}을 사용할 수 있습니다</strong>
        <p>{updateState.isDownloadOnly ? "APK를 내려받아 설치하면 업데이트됩니다." : "지금 업데이트하면 앱이 다시 시작됩니다."}</p>
      {:else if status.phase === "downloading"}
        <strong>{status.version} 내려받는 중{status.progress !== null ? ` · ${status.progress}%` : "…"}</strong>
        <div
          class="update-progress"
          role="progressbar"
          aria-label="업데이트 내려받기"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={status.progress ?? undefined}
        >
          <span class:indeterminate={status.progress === null} style={`width: ${status.progress ?? 30}%`}></span>
        </div>
      {:else if status.phase === "installing"}
        <strong>{status.version} 설치 중</strong>
        <p>설치가 끝나면 앱이 다시 시작됩니다.</p>
      {:else if status.phase === "error"}
        <strong>업데이트하지 못했습니다</strong>
        <p class="error-message">{status.message}</p>
      {/if}
    </div>
    {#if !busy}
      <div class="update-actions">
        <Button size="sm" onclick={() => updateState.install()}>
          {#if status.phase === "error"}
            다시 시도
          {:else}
            {updateState.isDownloadOnly ? "다운로드" : "지금 업데이트"}
          {/if}
        </Button>
        <button type="button" class="update-dismiss" aria-label="업데이트 알림 닫기" onclick={() => updateState.dismiss()}>
          <X size={17} />
        </button>
      </div>
    {/if}
  </div>
{/if}

<style>
  .update-banner {
    display: grid;
    grid-template-columns: 24px minmax(0, 1fr) auto;
    align-items: center;
    column-gap: 12px;
    padding: 10px clamp(16px, 3vw, 28px);
    border-bottom: 1px solid var(--color-accent-border);
    background: var(--color-accent-soft);
    color: var(--color-text);
    position: relative;
    z-index: 19;
    box-sizing: border-box;
  }

  .update-banner.error {
    border-bottom-color: var(--color-danger-soft-border);
    background: var(--color-danger-soft-bg);
  }

  .update-icon {
    display: flex;
    color: var(--color-accent);
  }

  .error .update-icon { color: var(--color-danger); }

  .update-copy {
    min-width: 0;
    display: grid;
    gap: 3px;
  }

  .update-copy strong {
    font-size: 13px;
    line-height: 1.4;
  }

  .update-copy p {
    margin: 0;
    font-size: 12px;
    line-height: 1.45;
    color: var(--color-text-muted);
  }

  .error-message {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .update-progress {
    height: 4px;
    margin-top: 3px;
    border-radius: var(--radius-full);
    background: color-mix(in oklab, var(--color-accent), transparent 80%);
    overflow: hidden;
  }

  .update-progress span {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--color-accent);
    transition: width var(--motion-fast);
  }

  .update-progress span.indeterminate {
    animation: update-indeterminate 1.1s ease-in-out infinite alternate;
  }

  @keyframes update-indeterminate {
    from { transform: translateX(-100%); }
    to { transform: translateX(330%); }
  }

  .update-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .update-dismiss {
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

  .update-dismiss:hover { background: var(--color-interactive-hover); }

  .update-dismiss:focus-visible {
    outline: 3px solid var(--color-focus-ring);
    outline-offset: 2px;
  }

  /* Phones: actions drop under the copy so the version text isn't squeezed. */
  @media (max-width: 540px) {
    .update-banner {
      grid-template-columns: 20px minmax(0, 1fr);
      row-gap: 8px;
      padding: 10px 14px;
    }

    .update-actions {
      grid-column: 2;
      justify-content: space-between;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .update-progress span.indeterminate { animation: none; }
  }
</style>
