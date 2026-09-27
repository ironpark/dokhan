<script lang="ts">
  import type { BuildProgress } from '$lib/types/dictionary';

  let {
    progress = null,
    visible = false
  }: {
    progress: BuildProgress | null;
    visible: boolean;
  } = $props();

  function progressPercent(p: BuildProgress | null): number | null {
    if (!p || p.total <= 1) return null;
    return Math.max(0, Math.min(100, Math.round((p.current / p.total) * 100)));
  }

  const percent = $derived(progressPercent(progress));

  function phaseLabel(phase: string): string {
    if (phase === 'scan') return '데이터 스캔';
    if (phase === 'parse') return '사전 파싱';
    if (phase === 'search-index') return '검색 인덱스';
    if (phase === 'source-prepare') return 'ZIP 파일 준비';
    if (phase === 'start') return '사전 준비';
    if (phase === 'cache') return '저장된 데이터 불러오기';
    if (phase === 'done') return '완료';
    if (phase === 'error') return '오류';
    return phase;
  }

  function progressMessage(p: BuildProgress): string {
    if (p.phase === 'source-prepare') return '선택한 ZIP 파일을 준비하고 있습니다.';
    if (p.phase === 'scan') return '사전 파일을 확인하고 있습니다.';
    if (p.phase === 'parse') return '표제어와 본문을 분석하고 있습니다.';
    if (p.phase === 'search-index') return '검색 기능을 준비하고 있습니다.';
    if (p.phase === 'cache') return '저장된 사전 데이터를 불러오고 있습니다.';
    if (p.phase === 'start') return '사전 데이터를 준비하고 있습니다.';
    return p.message;
  }
</script>

{#if visible && progress}
  <section class="progress-wrap" aria-label="사전 데이터 준비 중">
    <div class="progress-panel">
      <div class="progress-top">
        <div class="title-group">
          <strong>데이터 로딩 중</strong>
          <small>{phaseLabel(progress.phase)}</small>
        </div>
        <span class="percent">{percent === null ? '준비 중' : `${percent}%`}</span>
      </div>

      <div class="meter" class:indeterminate={percent === null} role="progressbar" aria-label={phaseLabel(progress.phase)} aria-valuemin="0" aria-valuemax="100" aria-valuenow={percent ?? undefined}>
        <div class="meter-fill" style={percent === null ? '' : `width:${percent}%`}></div>
      </div>

      <div class="progress-bottom">
        <p role="status">{progressMessage(progress)}</p>
        {#if percent !== null}<span>{progress.current}/{progress.total}</span>{/if}
      </div>
    </div>
  </section>
{/if}

<style>
  .progress-wrap {
    position: fixed;
    inset: 0;
    z-index: 1200;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: calc(14px + env(safe-area-inset-top)) 12px;
    box-sizing: border-box;
    background: rgba(20, 28, 44, 0.24);
    pointer-events: auto;
  }

  .progress-panel {
    width: min(560px, 100%);
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

  .meter.indeterminate .meter-fill {
    width: 35%;
    animation: loading-slide 1.4s ease-in-out infinite alternate;
  }

  @keyframes loading-slide {
    from { transform: translateX(0); }
    to { transform: translateX(185%); }
  }

  @media (prefers-reduced-motion: reduce) {
    .meter.indeterminate .meter-fill { animation: none; }
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
