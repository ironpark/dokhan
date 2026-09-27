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

  const PHASES: Record<string, { label: string; message?: string }> = {
    'source-prepare': { label: 'ZIP 파일 준비', message: '선택한 ZIP 파일을 준비하고 있습니다.' },
    scan: { label: '데이터 스캔', message: '사전 파일을 확인하고 있습니다.' },
    parse: { label: '사전 파싱', message: '표제어와 본문을 분석하고 있습니다.' },
    'search-index': { label: '검색 인덱스', message: '검색 기능을 준비하고 있습니다.' },
    cache: { label: '저장된 데이터 불러오기', message: '저장된 사전 데이터를 불러오고 있습니다.' },
    start: { label: '사전 준비', message: '사전 데이터를 준비하고 있습니다.' },
    done: { label: '완료' },
    error: { label: '오류' }
  };

  function phaseLabel(phase: string): string {
    return PHASES[phase]?.label ?? phase;
  }

  function progressMessage(p: BuildProgress): string {
    return PHASES[p.phase]?.message ?? p.message;
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
    align-items: center;
    justify-content: center;
    padding: 20px;
    box-sizing: border-box;
    background: var(--color-overlay);
    backdrop-filter: blur(3px);
  }

  .progress-panel {
    width: min(460px, 100%);
    border: 1px solid var(--color-border);
    border-radius: 17px;
    background: var(--color-surface);
    box-shadow: 0 24px 70px rgba(20, 32, 41, 0.18);
    padding: 22px 24px;
    display: grid;
    gap: 16px;
  }

  .progress-top {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    align-items: center;
  }

  .title-group {
    display: grid;
    gap: 5px;
  }

  .title-group strong {
    font-size: 16px;
    line-height: 1.3;
    color: var(--color-text);
  }

  .title-group small {
    color: var(--color-text-muted);
    font-size: 12px;
  }

  .percent {
    font-size: 12px;
    font-weight: 700;
    color: var(--color-accent);
  }

  .meter {
    height: 8px;
    background: var(--color-accent-soft);
    overflow: hidden;
    border-radius: 999px;
  }

  .meter-fill {
    height: 100%;
    background: var(--color-accent);
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
    font-size: 13px;
    line-height: 1.5;
    color: var(--color-text-muted);
  }

  .progress-bottom span {
    font-size: 12px;
    color: var(--color-text-muted);
    white-space: nowrap;
  }

  @media (max-width: 520px) {
    .progress-panel { padding: 19px 20px; }
  }
</style>
