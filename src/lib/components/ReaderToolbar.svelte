<script lang="ts">
  import Bookmark from "@lucide/svelte/icons/bookmark";
  import BookmarkCheck from "@lucide/svelte/icons/bookmark-check";
  import Button from "$lib/components/ui/Button.svelte";
  import Select from "$lib/components/ui/Select.svelte";
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
    readerFontSize = 100,
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
</script>

<div class="doc-sticky-shell">
  <header class="doc-header">
    <h2 class="doc-title">{title}</h2>
    <div class="doc-actions">
      <Button
        type="button"
        size="xs"
        class="toolbar-action"
        aria-pressed={preprocessEnabled}
        variant={preprocessEnabled ? "pill-active" : "pill"}
        onclick={onTogglePreprocess}
      >
        {preprocessEnabled ? "전처리 ON" : "전처리 OFF"}
      </Button>
      <Button
        type="button"
        size="xs"
        class="toolbar-action"
        aria-pressed={markerPreprocessEnabled}
        variant={markerPreprocessEnabled ? "pill-active" : "pill"}
        onclick={onToggleMarkerPreprocess}
        disabled={!preprocessEnabled}
      >
        {markerPreprocessEnabled ? "표기 태그 ON" : "표기 태그 OFF"}
      </Button>
      <Button
        type="button"
        size="xs"
        class={`toolbar-action favorite-btn ${isFavorite ? "favorite-active" : ""}`}
        aria-pressed={isFavorite}
        variant={isFavorite ? "pill-active" : "pill"}
        onclick={onToggleFavorite}
      >
        {#if isFavorite}
          <BookmarkCheck size={14} />
          <span>북마크됨</span>
        {:else}
          <Bookmark size={14} />
          <span>북마크</span>
        {/if}
      </Button>
      <Button
        type="button"
        size="xs"
        class="toolbar-action"
        aria-pressed={showReaderTools}
        variant={showReaderTools ? "pill-active" : "pill"}
        onclick={onToggleReaderTools}
      >
        보기 옵션
      </Button>
    </div>
  </header>

  {#if showReaderTools}
    <div class="doc-tools-row">
      <div class="view-controls">
        <label class="option-field">
          <span class="option-label">글자 크기</span>
          <div class="font-slider-wrap" style={`--font-slider-progress: ${fontSliderProgress};`}>
            <span class="font-slider-percent" aria-hidden="true">
              {normalizeFontSize(readerFontSize)}%
            </span>
            <input
              type="range"
              class="font-slider"
              min="80"
              max="130"
              step="2"
              value={normalizeFontSize(readerFontSize)}
              oninput={handleFontSizeChange}
              aria-label="글자 크기"
            />
            <div class="font-slider-labels" aria-hidden="true">
              <span>80%</span>
              <span>100%</span>
              <span>130%</span>
            </div>
          </div>
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
    </div>
  {/if}
</div>

<style>
  .doc-title {
    margin: 6px 0 4px;
    font-size: clamp(24px, 3vw, 30px);
    line-height: 1.15;
    letter-spacing: -0.015em;
  }

  .doc-sticky-shell {
    position: sticky;
    top: 0;
    z-index: 12;
    margin-bottom: 8px;
    padding-bottom: 8px;
    background:
      linear-gradient(
        180deg,
        color-mix(in oklab, var(--color-surface), white 2%) 0%,
        color-mix(in oklab, var(--color-surface), white 0%) 82%,
        rgba(255, 255, 255, 0) 100%
      );
    backdrop-filter: blur(2px);
  }

  .doc-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 0;
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

  :global(.toolbar-action) {
    margin-top: 6px;
    white-space: nowrap;
    font-size: 12px;
    min-height: 28px;
    padding-inline: 10px;
    color: var(--color-text-muted);
  }

  :global(.toolbar-action:hover) {
    color: var(--color-text);
  }

  :global(.toolbar-action[aria-pressed="true"]) {
    color: var(--color-accent);
  }

  :global(.toolbar-action[disabled]) {
    color: var(--color-text-muted);
  }

  :global(.toolbar-action.favorite-active) {
    color: #ad7a00;
  }

  :global(.favorite-active) {
    color: #ad7a00;
    border-color: #e8ca77;
    background: #fff8dc;
  }

  :global(.favorite-btn) {
    display: inline-flex;
    align-items: center;
    gap: 6px;
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

  .font-slider-wrap {
    position: relative;
    display: grid;
    gap: 6px;
    padding-top: 2px;
  }

  .font-slider-percent {
    position: absolute;
    top: -26px;
    left: clamp(10px, var(--font-slider-progress), calc(100% - 10px));
    transform: translateX(-50%);
    pointer-events: none;
    opacity: 0;
    visibility: hidden;
    transition:
      opacity var(--motion-fast),
      transform var(--motion-fast),
      visibility var(--motion-fast);
    color: var(--color-accent);
    font-size: 12px;
    padding: 2px 8px;
    line-height: 1.2;
    border-radius: 999px;
    background: color-mix(in oklab, var(--color-accent), white 88%);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
    z-index: 1;
  }

  .font-slider-wrap:hover .font-slider-percent,
  .font-slider-wrap:focus-within .font-slider-percent {
    opacity: 1;
    visibility: visible;
    transform: translateX(-50%) translateY(-2px);
  }

  .font-slider {
    width: 100%;
    margin: 0;
    cursor: pointer;
    appearance: none;
    height: 6px;
    border-radius: 999px;
    background: linear-gradient(
      90deg,
      var(--color-accent) 0%,
      var(--color-accent) var(--font-slider-progress),
      color-mix(in oklab, var(--color-border), white 18%) var(--font-slider-progress),
      color-mix(in oklab, var(--color-border), white 18%) 100%
    );
  }

  .font-slider::-webkit-slider-thumb {
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    border: 2px solid var(--color-accent);
    background: var(--color-surface);
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.12);
  }

  .font-slider::-moz-range-thumb {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    border: 2px solid var(--color-accent);
    background: var(--color-surface);
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.12);
  }

  .font-slider-labels {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: var(--color-text-muted);
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
