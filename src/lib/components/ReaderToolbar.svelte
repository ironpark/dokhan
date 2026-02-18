<script lang="ts">
  import type {
    ReaderFontSize,
    ReaderLineHeight,
    ReaderWidth,
  } from "$lib/types/dictionary";

  let {
    title,
    preprocessEnabled = true,
    markerPreprocessEnabled = true,
    isFavorite = false,
    showReaderTools = false,
    readerFontSize = "md",
    readerLineHeight = "normal",
    readerWidth = "normal",
    onTogglePreprocess = () => {},
    onToggleMarkerPreprocess = () => {},
    onToggleFavorite = () => {},
    onToggleReaderTools = () => {},
    onReaderFontSizeChange = () => {},
    onReaderLineHeightChange = () => {},
    onReaderWidthChange = () => {},
  }: {
    title: string;
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
    onReaderFontSizeChange?: (value: ReaderFontSize) => void;
    onReaderLineHeightChange?: (value: ReaderLineHeight) => void;
    onReaderWidthChange?: (value: ReaderWidth) => void;
  } = $props();

  function handleFontSizeChange(event: Event) {
    onReaderFontSizeChange((event.currentTarget as HTMLSelectElement).value as ReaderFontSize);
  }

  function handleLineHeightChange(event: Event) {
    onReaderLineHeightChange((event.currentTarget as HTMLSelectElement).value as ReaderLineHeight);
  }

  function handleWidthChange(event: Event) {
    onReaderWidthChange((event.currentTarget as HTMLSelectElement).value as ReaderWidth);
  }
</script>

<header class="doc-header">
  <h2 class="doc-title">{title}</h2>
  <div class="doc-actions">
    <button
      type="button"
      class="mini-btn"
      class:active={preprocessEnabled}
      onclick={onTogglePreprocess}
    >
      {preprocessEnabled ? "전처리 ON" : "전처리 OFF"}
    </button>
    <button
      type="button"
      class="mini-btn"
      class:active={markerPreprocessEnabled}
      onclick={onToggleMarkerPreprocess}
      disabled={!preprocessEnabled}
    >
      {markerPreprocessEnabled ? "표기 태그 ON" : "표기 태그 OFF"}
    </button>
    <button
      type="button"
      class="favorite-btn mini-btn"
      class:active={isFavorite}
      onclick={onToggleFavorite}
    >
      {isFavorite ? "★ 저장됨" : "☆ 저장"}
    </button>
    <button
      type="button"
      class="mini-btn"
      class:active={showReaderTools}
      onclick={onToggleReaderTools}
    >
      보기 옵션
    </button>
  </div>
</header>

{#if showReaderTools}
  <div class="doc-tools-row">
    <div class="view-controls">
      <label class="option-field">
        <span class="option-label">글자 크기</span>
        <select value={readerFontSize} onchange={handleFontSizeChange}>
          <option value="sm">작게</option>
          <option value="md">보통</option>
          <option value="lg">크게</option>
        </select>
      </label>
      <label class="option-field">
        <span class="option-label">줄 간격</span>
        <select value={readerLineHeight} onchange={handleLineHeightChange}>
          <option value="tight">좁게</option>
          <option value="normal">보통</option>
          <option value="loose">넓게</option>
        </select>
      </label>
      <label class="option-field">
        <span class="option-label">본문 폭</span>
        <select value={readerWidth} onchange={handleWidthChange}>
          <option value="narrow">좁게</option>
          <option value="normal">보통</option>
          <option value="wide">넓게</option>
        </select>
      </label>
    </div>
  </div>
{/if}

<style>
  .doc-title {
    margin: 6px 0 4px;
    font-size: clamp(24px, 3vw, 30px);
    line-height: 1.15;
    letter-spacing: -0.015em;
  }

  .doc-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .doc-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 6px;
    flex-shrink: 0;
  }

  .doc-tools-row {
    display: flex;
    width: 100%;
    margin: 8px 0 10px;
  }

  .mini-btn {
    border: 1px solid transparent;
    background: var(--color-surface);
    color: var(--color-text-muted);
    border-radius: 999px;
    font-size: 12px;
    line-height: 1;
    padding: 7px 10px;
    cursor: pointer;
    white-space: nowrap;
    margin-top: 6px;
    transition:
      background-color var(--motion-fast),
      border-color var(--motion-fast),
      color var(--motion-fast),
      opacity var(--motion-fast);
  }

  .mini-btn.active {
    color: var(--color-accent);
    border-color: color-mix(in oklab, var(--color-accent), white 62%);
    background: var(--color-accent-soft);
  }

  .favorite-btn.active {
    color: #ad7a00;
    border-color: #e8ca77;
    background: #fff8dc;
  }

  .mini-btn:hover {
    border-color: var(--color-border-strong);
  }

  .mini-btn:disabled {
    cursor: default;
    opacity: 0.5;
  }

  .mini-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px var(--color-accent-soft);
  }

  .view-controls {
    width: 100%;
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 10px;
    border: 1px solid color-mix(in oklab, var(--color-border), white 25%);
    border-radius: 14px;
    background:
      linear-gradient(
        180deg,
        color-mix(in oklab, var(--color-surface-soft), white 45%) 0%,
        color-mix(in oklab, var(--color-surface-soft), white 20%) 100%
      );
    padding: 10px;
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.55),
      0 1px 2px rgba(0, 0, 0, 0.04);
  }

  .option-field {
    display: grid;
    gap: 6px;
  }

  .option-label {
    font-size: 10px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--color-text-muted);
    font-weight: 600;
  }

  .view-controls select {
    appearance: none;
    border: 1px solid var(--color-border);
    border-radius: 10px;
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 12px;
    font-weight: 600;
    line-height: 1;
    padding: 9px 30px 9px 10px;
    outline: none;
    background-image:
      linear-gradient(45deg, transparent 50%, currentColor 50%),
      linear-gradient(135deg, currentColor 50%, transparent 50%);
    background-position:
      calc(100% - 15px) calc(50% - 2px),
      calc(100% - 10px) calc(50% - 2px);
    background-size:
      5px 5px,
      5px 5px;
    background-repeat: no-repeat;
    transition:
      border-color var(--motion-fast),
      box-shadow var(--motion-fast),
      background-color var(--motion-fast);
  }

  .view-controls select:hover {
    border-color: var(--color-border-strong);
  }

  .view-controls select:focus-visible {
    border-color: var(--color-accent);
    box-shadow: 0 0 0 3px color-mix(in oklab, var(--color-accent), white 80%);
  }

  @media (max-width: 768px) {
    .doc-header {
      flex-direction: column;
    }

    .doc-actions {
      width: 100%;
      justify-content: flex-start;
    }

    .doc-tools-row {
      margin-top: 6px;
      margin-bottom: 8px;
    }

    .view-controls {
      grid-template-columns: 1fr;
      gap: 8px;
      padding: 9px;
      border-radius: 12px;
    }
  }
</style>
