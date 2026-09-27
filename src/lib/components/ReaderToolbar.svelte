<script lang="ts">
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import Bookmark from "@lucide/svelte/icons/bookmark";
  import BookmarkCheck from "@lucide/svelte/icons/bookmark-check";
  import SlidersHorizontal from "@lucide/svelte/icons/sliders-horizontal";
  import Button from "$lib/components/ui/Button.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import type {
    ReaderFontSize,
    ReaderLineHeight,
    ReaderWidth,
  } from "$lib/types/dictionary";

  let {
    title,
    kind,
    readingProgress = 0,
    isScrolled = false,
    preprocessEnabled = true,
    markerPreprocessEnabled = true,
    isFavorite = false,
    showReaderTools = false,
    readerFontSize = 100,
    readerLineHeight = "normal",
    readerWidth = "normal",
    onTogglePreprocess = () => {},
    onToggleMarkerPreprocess = () => {},
    onToggleFavorite = () => {},
    onToggleReaderTools = () => {},
    onReturnToTop = () => {},
    onReaderFontSizeChange = () => {},
    onReaderLineHeightChange = () => {},
    onReaderWidthChange = () => {},
  }: {
    title: string;
    kind: "목차" | "표제어";
    readingProgress?: number;
    isScrolled?: boolean;
    preprocessEnabled?: boolean;
    markerPreprocessEnabled?: boolean;
    isFavorite?: boolean;
    showReaderTools?: boolean;
    readerFontSize?: ReaderFontSize;
    readerLineHeight?: ReaderLineHeight;
    readerWidth?: ReaderWidth;
    onTogglePreprocess?: () => void;
    onToggleMarkerPreprocess?: () => void;
    onToggleFavorite?: () => void;
    onToggleReaderTools?: () => void;
    onReturnToTop?: () => void;
    onReaderFontSizeChange?: (value: ReaderFontSize) => void;
    onReaderLineHeightChange?: (value: ReaderLineHeight) => void;
    onReaderWidthChange?: (value: ReaderWidth) => void;
  } = $props();

  function normalizeFontSize(value: number): number {
    const rounded = Math.round(value);
    return Math.min(130, Math.max(80, rounded));
  }

  const fontSliderProgress = $derived(
    `${((normalizeFontSize(readerFontSize) - 80) / 50) * 100}%`,
  );

  function handleFontSizeChange(event: Event) {
    const target = event.target as HTMLInputElement | null;
    if (!target) return;
    const next = normalizeFontSize(Number.parseInt(target.value, 10));
    onReaderFontSizeChange(next);
  }

  function handleLineHeightChange(event: Event) {
    const target = event.target as HTMLSelectElement | null;
    if (!target) return;
    onReaderLineHeightChange(target.value as ReaderLineHeight);
  }

  function handleWidthChange(event: Event) {
    const target = event.target as HTMLSelectElement | null;
    if (!target) return;
    onReaderWidthChange(target.value as ReaderWidth);
  }

  function resetReadingSettings() {
    onReaderFontSizeChange(100);
    onReaderLineHeightChange("normal");
    onReaderWidthChange("normal");
  }

  const markerActive = $derived(markerPreprocessEnabled && preprocessEnabled);
</script>

<div class="doc-sticky-shell" class:compact={isScrolled}>
  <header class="doc-header">
    <div class="title-block">
      <span class="doc-kind">{kind}</span>
      <h2 class="doc-title" title={title}>{title}</h2>
    </div>
    <div class="doc-actions">
      <Button
        type="button"
        size="sm"
        class="toolbar-action favorite-btn"
        aria-label={isFavorite ? "책갈피 삭제" : "책갈피 추가"}
        aria-pressed={isFavorite}
        variant={isFavorite ? "toolbar-pill-warn-active" : "toolbar-pill"}
        onclick={onToggleFavorite}
      >
        {#if isFavorite}
          <BookmarkCheck size={16} />
          <span>책갈피됨</span>
        {:else}
          <Bookmark size={16} />
          <span>책갈피</span>
        {/if}
      </Button>
      <Button
        type="button"
        size="sm"
        class="toolbar-action settings-button"
        aria-label="읽기 설정"
        aria-expanded={showReaderTools}
        aria-controls="reader-view-options"
        variant={showReaderTools ? "toolbar-pill-active" : "toolbar-pill"}
        onclick={onToggleReaderTools}
      >
        <SlidersHorizontal size={16} />
        <span>읽기 설정</span>
      </Button>
      {#if isScrolled}
        <Button
          type="button"
          size="icon-sm"
          class="toolbar-action back-to-top"
          variant="toolbar-pill"
          aria-label="본문 맨 위로"
          title="본문 맨 위로"
          onclick={onReturnToTop}
        >
          <ArrowUp size={16} />
        </Button>
      {/if}
    </div>
  </header>

  {#if showReaderTools}
    <div id="reader-view-options" class="reader-options">
      <div class="options-heading">
        <div>
          <strong>읽기 설정</strong>
          <p>글자와 본문 표시를 편하게 조정하세요.</p>
        </div>
        <Button type="button" size="xs" variant="soft" onclick={resetReadingSettings}>
          글자 설정 초기화
        </Button>
      </div>
      <div class="view-controls">
        <label class="option-field font-field">
          <span class="option-header">
            <span class="option-label">글자 크기</span>
            <output class="font-value">{normalizeFontSize(readerFontSize)}%</output>
          </span>
          <input
            type="range"
            class="font-slider"
            style={`--font-slider-progress: ${fontSliderProgress};`}
            min="80"
            max="130"
            step="2"
            value={normalizeFontSize(readerFontSize)}
            oninput={handleFontSizeChange}
            aria-label="글자 크기"
            aria-valuetext={`${normalizeFontSize(readerFontSize)}퍼센트`}
          />
          <span class="font-slider-labels" aria-hidden="true">
            <span>작게</span><span>기본</span><span>크게</span>
          </span>
        </label>
        <label class="option-field">
          <span class="option-label">줄 간격</span>
          <Select bind:value={readerLineHeight} uiSize="sm" oninput={handleLineHeightChange}>
            <option value="tight">좁게</option>
            <option value="normal">보통</option>
            <option value="loose">넓게</option>
          </Select>
        </label>
        <label class="option-field">
          <span class="option-label">본문 폭</span>
          <Select bind:value={readerWidth} uiSize="sm" oninput={handleWidthChange}>
            <option value="narrow">좁게</option>
            <option value="normal">보통</option>
            <option value="wide">넓게</option>
          </Select>
        </label>
      </div>
      <div class="text-options">
        <div class="text-options-description">
          <span class="option-label">본문 표시</span>
          <p>번호와 표기 설명을 읽기 쉽게 보여줍니다.</p>
        </div>
        <div class="text-option-actions">
          <Button
            type="button"
            size="sm"
            class="toolbar-action"
            aria-pressed={preprocessEnabled}
            variant={preprocessEnabled ? "toolbar-pill-active" : "toolbar-pill"}
            onclick={onTogglePreprocess}
          >
            본문 정리 {preprocessEnabled ? "켬" : "끔"}
          </Button>
          <Button
            type="button"
            size="sm"
            class="toolbar-action"
            aria-pressed={markerActive}
            variant={markerActive ? "toolbar-pill-active" : "toolbar-pill"}
            onclick={onToggleMarkerPreprocess}
            disabled={!preprocessEnabled}
          >
            표기 설명 {markerActive ? "켬" : "끔"}
          </Button>
        </div>
      </div>
    </div>
  {/if}
  <div class="reading-progress" aria-hidden="true">
    <span style={`width: ${readingProgress}%`}></span>
  </div>
</div>

<style>
  .doc-sticky-shell {
    position: sticky;
    top: 0;
    z-index: 12;
    margin-bottom: 16px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    box-shadow: var(--shadow-sm);
  }

  .doc-header {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: 16px;
    min-height: 78px;
    padding: 12px 0;
  }

  .title-block { min-width: 0; }

  .doc-kind {
    display: block;
    margin-bottom: 2px;
    color: var(--color-text-muted);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.04em;
  }

  .doc-title {
    margin: 0;
    font-size: clamp(23px, 2.5vw, 30px);
    font-weight: 730;
    line-height: 1.25;
    letter-spacing: -0.025em;
    overflow-wrap: anywhere;
  }

  .doc-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
  }

  .doc-sticky-shell :global(.toolbar-action) {
    min-height: 34px;
    font-size: 12px;
    white-space: nowrap;
  }

  .doc-sticky-shell :global(.favorite-btn) { color: var(--color-text); }
  .doc-sticky-shell :global(.back-to-top) { width: 34px; }

  .compact .doc-header {
    min-height: 55px;
    padding: 8px 0;
  }

  .compact .doc-kind { display: none; }

  .compact .doc-title {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    font-size: 17px;
  }

  .reader-options { padding: 4px 0 16px; }

  .options-heading,
  .text-options {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .options-heading strong { font-size: 13px; }

  .options-heading p,
  .text-options-description p {
    margin: 2px 0 0;
    color: var(--color-text-muted);
    font-size: 12px;
    line-height: 1.4;
  }

  .view-controls {
    display: grid;
    grid-template-columns: minmax(160px, 1.4fr) repeat(2, minmax(110px, 1fr));
    gap: 14px;
    margin-top: 12px;
    padding: 14px;
    border: 1px solid var(--color-border);
    border-radius: 12px;
    background: var(--color-surface-soft);
  }

  .option-field {
    display: grid;
    align-content: start;
    gap: 8px;
    min-width: 0;
  }

  .option-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .option-label {
    color: var(--color-text-muted);
    font-size: 12px;
    font-weight: 650;
  }

  .font-value {
    color: var(--color-accent-hover);
    font-size: 12px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .font-slider {
    width: 100%;
    height: 34px;
    margin: 0;
    cursor: pointer;
    appearance: none;
    background: transparent;
  }

  .font-slider::-webkit-slider-runnable-track {
    height: 7px;
    border-radius: 99px;
    background: linear-gradient(90deg, var(--color-accent) var(--font-slider-progress), var(--color-divider) var(--font-slider-progress));
  }

  .font-slider::-moz-range-track {
    height: 7px;
    border-radius: 99px;
    background: var(--color-divider);
  }

  .font-slider::-moz-range-progress {
    height: 7px;
    border-radius: 99px;
    background: var(--color-accent);
  }

  .font-slider::-webkit-slider-thumb {
    appearance: none;
    width: 22px;
    height: 22px;
    margin-top: -7px;
    border: 2px solid var(--color-accent);
    border-radius: 50%;
    background: var(--color-surface);
    box-shadow: var(--shadow-sm);
  }

  .font-slider::-moz-range-thumb {
    width: 19px;
    height: 19px;
    border: 2px solid var(--color-accent);
    border-radius: 50%;
    background: var(--color-surface);
    box-shadow: var(--shadow-sm);
  }

  .font-slider:focus-visible {
    outline: 3px solid var(--color-focus-ring);
    outline-offset: 2px;
    border-radius: 8px;
  }

  .font-slider-labels {
    display: flex;
    justify-content: space-between;
    margin-top: -7px;
    color: var(--color-text-muted);
    font-size: 11px;
  }

  .text-options {
    margin-top: 12px;
    padding: 0 2px;
  }

  .text-option-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 6px;
  }

  .reading-progress {
    height: 3px;
    margin: 0 calc(-1 * var(--space-8));
  }

  .reading-progress span {
    display: block;
    height: 100%;
    background: var(--color-accent);
    transition: width 90ms linear;
  }

  @media (max-width: 768px) {
    .doc-header {
      grid-template-columns: minmax(0, 1fr);
      gap: 10px;
      min-height: 0;
      padding: 14px 0 10px;
    }

    .doc-title { font-size: 23px; }
    .doc-actions { justify-content: flex-start; }

    .compact .doc-header {
      grid-template-columns: minmax(0, 1fr) auto;
      gap: 8px;
    }

    .compact .doc-actions :global(.favorite-btn span),
    .compact .doc-actions :global(.settings-button span) {
      display: none;
    }

    .view-controls {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 12px;
    }

    .font-field { grid-column: 1 / -1; }

    .text-options {
      align-items: flex-start;
      flex-direction: column;
    }

    .text-option-actions { justify-content: flex-start; }
    .reading-progress { margin: 0 calc(-1 * var(--space-4)); }
  }

  @media (max-width: 420px) {
    .options-heading {
      align-items: flex-start;
      flex-direction: column;
    }

    .view-controls { grid-template-columns: 1fr; }
    .font-field { grid-column: auto; }
  }

  @media (prefers-reduced-motion: reduce) {
    .reading-progress span { transition: none; }
  }
</style>
