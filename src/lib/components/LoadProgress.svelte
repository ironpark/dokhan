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

  function phaseLabel(phase: string): string {
    if (phase === 'scan') return '데이터 스캔';
    if (phase === 'parse') return '사전 파싱';
    if (phase === 'search-index') return '검색 인덱스';
    if (phase === 'done') return '완료';
    if (phase === 'error') return '오류';
    return phase;
  }
</script>

{#if visible && progress}
  <section class="progress-wrap" aria-live="polite">
    <div class="progress-panel">
      <div class="progress-top">
        <div class="title-group">
          <strong>데이터 로딩 중</strong>
          <small>{phaseLabel(progress.phase)}</small>
        </div>
        <span class="percent">{progressPercent(progress)}%</span>
      </div>

      <div class="meter" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={progressPercent(progress)}>
        <div class="meter-fill" style={`width:${progressPercent(progress)}%`}></div>
      </div>

      <div class="progress-bottom">
        <p>{progress.message}</p>
        <span>{progress.current}/{progress.total}</span>
      </div>
    </div>
  </section>
{/if}

<style>
  .progress-wrap {
    position: fixed;
    top: 14px;
    left: 50%;
    transform: translateX(-50%);
    width: min(560px, calc(100vw - 24px));
    z-index: 1200;
    pointer-events: none;
  }

  .progress-panel {
    border: 1px solid color-mix(in oklab, var(--line), #8ea2c9 20%);
    background: color-mix(in oklab, var(--surface), #f8fbff 35%);
    box-shadow: 0 14px 28px rgba(24, 36, 64, 0.14);
    backdrop-filter: blur(8px);
    padding: 10px 12px;
    display: grid;
    gap: 8px;
  }

  .progress-top {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    align-items: center;
  }

  .title-group {
    display: grid;
    gap: 2px;
  }

  .title-group strong {
    font-size: 13px;
    line-height: 1.2;
  }

  .title-group small {
    color: var(--muted);
    font-size: 11px;
  }

  .percent {
    font-size: 12px;
    font-weight: 700;
    color: #345ea8;
  }

  .meter {
    height: 6px;
    background: #dde4f3;
    overflow: hidden;
    border-radius: 999px;
  }

  .meter-fill {
    height: 100%;
    background: linear-gradient(90deg, #3b82f6, #2563eb);
    transition: width 120ms linear;
    border-radius: 999px;
  }

  .progress-bottom {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .progress-bottom p {
    margin: 0;
    font-size: 12px;
    color: var(--muted);
  }

  .progress-bottom span {
    font-size: 11px;
    color: #657598;
    white-space: nowrap;
  }
</style>
