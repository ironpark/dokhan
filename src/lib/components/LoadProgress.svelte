<script lang="ts">
  import type { BuildProgress } from '$lib/types/dictionary';

  let {
    progress = null,
    visible = false
  }: {
    progress: BuildProgress | null;
    visible: boolean;
  } = $props();

  function progressPercent(p: BuildProgress | null): number {
    if (!p || p.total <= 0) return 0;
    return Math.max(2, Math.min(100, Math.round((p.current / p.total) * 100)));
  }
</script>

{#if visible && progress}
  <section class="progress-panel">
    <div class="progress-header">
      <strong>데이터 로딩 중</strong>
      <span>{progress.message}</span>
    </div>
    <div class="meter"><div class="meter-fill" style={`width:${progressPercent(progress)}%`}></div></div>
    <p class="progress-meta">{progress.phase} · {progress.current}/{progress.total}</p>
  </section>
{/if}

<style>
  .progress-panel {
    border: 1px solid var(--line);
    background: var(--surface);
    padding: 8px 10px;
    display: grid;
    gap: 6px;
  }

  .progress-header {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    align-items: center;
    font-size: 14px;
  }

  .meter {
    height: 10px;
    background: #e6dfd1;
    overflow: hidden;
  }

  .meter-fill {
    height: 100%;
    background: linear-gradient(90deg, #0f6c58, #2f9678);
    transition: width 120ms linear;
  }

  .progress-meta {
    margin: 0;
    font-size: 12px;
    color: var(--muted);
  }
</style>
