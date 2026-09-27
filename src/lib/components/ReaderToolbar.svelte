<script lang="ts">
  import { untrack } from "svelte";
  import { cubicOut } from "svelte/easing";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import Bookmark from "@lucide/svelte/icons/bookmark";
  import BookmarkCheck from "@lucide/svelte/icons/bookmark-check";
  import Check from "@lucide/svelte/icons/check";
  import Folder from "@lucide/svelte/icons/folder";
  import FolderPlus from "@lucide/svelte/icons/folder-plus";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import SlidersHorizontal from "@lucide/svelte/icons/sliders-horizontal";
  import X from "@lucide/svelte/icons/x";
  import Button from "$lib/components/ui/Button.svelte";
  import SegmentedControl from "$lib/components/ui/SegmentedControl.svelte";
  import Switch from "$lib/components/ui/Switch.svelte";
  import type {
    BookmarkFolder,
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
    bookmarkFolders = [],
    activeBookmarkFolderId = "default",
    currentBookmarkFolderId = null,
    onAddBookmarkToFolder = () => {},
    onCreateBookmarkFolder = () => null,
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
    bookmarkFolders?: BookmarkFolder[];
    activeBookmarkFolderId?: string;
    currentBookmarkFolderId?: string | null;
    /** Saves the open item into the folder, or moves it there if already saved. */
    onAddBookmarkToFolder?: (folderId: string) => void;
    onCreateBookmarkFolder?: (name: string) => string | null;
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

  const FONT_MIN = 80;
  const FONT_MAX = 130;
  const FONT_STEP = 10;

  const lineHeightOptions: Array<{ value: ReaderLineHeight; label: string }> = [
    { value: "tight", label: "좁게" },
    { value: "normal", label: "보통" },
    { value: "loose", label: "넓게" },
  ];
  const widthOptions: Array<{ value: ReaderWidth; label: string }> = [
    { value: "narrow", label: "좁게" },
    { value: "normal", label: "보통" },
    { value: "wide", label: "넓게" },
  ];

  let settingsButtonEl = $state<HTMLElement | null>(null);
  let popoverEl = $state<HTMLDivElement | null>(null);
  let bookmarkButtonEl = $state<HTMLElement | null>(null);
  let bookmarkPopoverEl = $state<HTMLDivElement | null>(null);

  // --- Bookmark popover: one click saves to the last-used folder, then offers
  // folder choice / new folder / removal without a blocking dialog. ---
  let showBookmarkPopover = $state(false);
  let justSaved = $state(false);
  let creatingFolder = $state(false);
  let newFolderName = $state("");
  let folderError = $state("");

  const currentFolderName = $derived(
    bookmarkFolders.find((folder) => folder.id === currentBookmarkFolderId)?.name ?? "",
  );

  function closeBookmarkPopover(restoreFocus = false) {
    if (!showBookmarkPopover) return;
    showBookmarkPopover = false;
    creatingFolder = false;
    newFolderName = "";
    folderError = "";
    if (restoreFocus) queueMicrotask(() => bookmarkButtonEl?.querySelector("button")?.focus());
  }

  function handleBookmarkClick() {
    if (showBookmarkPopover) {
      closeBookmarkPopover();
      return;
    }
    if (showReaderTools) onToggleReaderTools();
    justSaved = !isFavorite;
    if (!isFavorite) {
      const target = bookmarkFolders.some((folder) => folder.id === activeBookmarkFolderId)
        ? activeBookmarkFolderId
        : (bookmarkFolders[0]?.id ?? "default");
      onAddBookmarkToFolder(target);
    }
    showBookmarkPopover = true;
  }

  function handleSettingsClick() {
    closeBookmarkPopover();
    onToggleReaderTools();
  }

  function removeBookmark() {
    if (isFavorite) onToggleFavorite();
    closeBookmarkPopover(true);
  }

  function submitNewFolder() {
    const name = newFolderName.trim();
    if (!name) return;
    const created = onCreateBookmarkFolder(name);
    if (!created) {
      folderError = "폴더를 만들 수 없습니다. 이름이 겹치거나 최대 개수에 도달했습니다.";
      return;
    }
    onAddBookmarkToFolder(created);
    creatingFolder = false;
    newFolderName = "";
    folderError = "";
  }

  function focusOnMount(node: HTMLElement) {
    node.focus();
  }

  // The item changed underneath (navigated away): drop the stale popover.
  $effect(() => {
    void title;
    untrack(() => closeBookmarkPopover());
  });

  function normalizeFontSize(value: number): number {
    const rounded = Math.round(value);
    return Math.min(FONT_MAX, Math.max(FONT_MIN, rounded));
  }

  const fontSize = $derived(normalizeFontSize(readerFontSize));
  const fontSliderProgress = $derived(`${((fontSize - FONT_MIN) / (FONT_MAX - FONT_MIN)) * 100}%`);
  const isDefaultTypography = $derived(
    fontSize === 100 && readerLineHeight === "normal" && readerWidth === "normal",
  );
  const markerActive = $derived(markerPreprocessEnabled && preprocessEnabled);

  function handleFontSizeInput(event: Event) {
    const target = event.target as HTMLInputElement | null;
    if (!target) return;
    onReaderFontSizeChange(normalizeFontSize(Number.parseInt(target.value, 10)));
  }

  function stepFontSize(direction: -1 | 1) {
    onReaderFontSizeChange(normalizeFontSize(fontSize + direction * FONT_STEP));
  }

  function resetReadingSettings() {
    onReaderFontSizeChange(100);
    onReaderLineHeightChange("normal");
    onReaderWidthChange("normal");
  }

  function closeSettings(restoreFocus = false) {
    if (!showReaderTools) return;
    onToggleReaderTools();
    if (restoreFocus) queueMicrotask(() => settingsButtonEl?.querySelector("button")?.focus());
  }

  // Light-dismiss for the bookmark popover.
  $effect(() => {
    if (!showBookmarkPopover) return;
    const onPointerDown = (event: PointerEvent) => {
      const target = event.target as Node | null;
      if (!target) return;
      if (bookmarkPopoverEl?.contains(target) || bookmarkButtonEl?.contains(target)) return;
      closeBookmarkPopover();
    };
    const onKeydown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        closeBookmarkPopover(true);
      }
    };
    document.addEventListener("pointerdown", onPointerDown, true);
    document.addEventListener("keydown", onKeydown);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown, true);
      document.removeEventListener("keydown", onKeydown);
    };
  });

  // Light-dismiss: outside pointer or Escape closes the popover.
  $effect(() => {
    if (!showReaderTools) return;
    const onPointerDown = (event: PointerEvent) => {
      const target = event.target as Node | null;
      if (!target) return;
      if (popoverEl?.contains(target) || settingsButtonEl?.contains(target)) return;
      closeSettings();
    };
    const onKeydown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        closeSettings(true);
      }
    };
    document.addEventListener("pointerdown", onPointerDown, true);
    document.addEventListener("keydown", onKeydown);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown, true);
      document.removeEventListener("keydown", onKeydown);
    };
  });

  /** Drop-in from the trigger; collapses to an instant swap under reduced motion. */
  function popover(_node: Element, { enter = true }: { enter?: boolean } = {}) {
    const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    return {
      duration: reduce ? 0 : enter ? 190 : 130,
      easing: cubicOut,
      css: (t: number) =>
        `opacity: ${t}; transform: translateY(${(1 - t) * -6}px) scale(${0.97 + 0.03 * t});`,
    };
  }
</script>

<div class="doc-sticky-shell" class:compact={isScrolled}>
  <header class="doc-header">
    <div class="title-block">
      <span class="doc-kind">{kind}</span>
      <h2 class="doc-title" class:headword={kind === "표제어"} title={title}>{title}</h2>
    </div>
    <div class="doc-actions">
      <span class="popover-anchor" bind:this={bookmarkButtonEl}>
        <Button
          type="button"
          size="sm"
          class="toolbar-action favorite-btn"
          aria-label={isFavorite ? `책갈피 관리, ${currentFolderName} 폴더에 저장됨` : "책갈피에 저장"}
          aria-expanded={showBookmarkPopover}
          aria-controls="bookmark-options"
          aria-haspopup="dialog"
          variant={isFavorite ? "toolbar-pill-warn-active" : "toolbar-pill"}
          onclick={handleBookmarkClick}
        >
          {#if isFavorite}
            <BookmarkCheck size={16} />
            <span>저장됨</span>
          {:else}
            <Bookmark size={16} />
            <span>책갈피</span>
          {/if}
        </Button>
      </span>
      <span class="popover-anchor" bind:this={settingsButtonEl}>
        <Button
          type="button"
          size="sm"
          class="toolbar-action settings-button"
          aria-label="읽기 설정"
          aria-expanded={showReaderTools}
          aria-controls="reader-view-options"
          aria-haspopup="dialog"
          variant={showReaderTools ? "toolbar-pill-active" : "toolbar-pill"}
          onclick={handleSettingsClick}
        >
          <SlidersHorizontal size={16} />
          <span>읽기 설정</span>
        </Button>
      </span>
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

      {#if showBookmarkPopover}
        <div
          id="bookmark-options"
          class="reader-popover bookmark-popover"
          role="dialog"
          aria-label="책갈피"
          bind:this={bookmarkPopoverEl}
          in:popover={{ enter: true }}
          out:popover={{ enter: false }}
        >
          <div class="popover-head">
            <strong class="saved-title" class:saved={isFavorite}>
              {#if isFavorite}
                <span class="saved-icon" aria-hidden="true"><BookmarkCheck size={15} /></span>
                {justSaved ? "책갈피에 저장했습니다" : "저장된 책갈피"}
              {:else}
                책갈피
              {/if}
            </strong>
            <div class="head-actions">
              <button type="button" class="icon-btn" aria-label="책갈피 창 닫기" onclick={() => closeBookmarkPopover(true)}>
                <X size={15} aria-hidden="true" />
              </button>
            </div>
          </div>

          <section class="field">
            <span class="field-label">폴더</span>
            <div class="folder-options" role="radiogroup" aria-label="책갈피 폴더">
              {#each bookmarkFolders as folder (folder.id)}
                {@const selected = isFavorite && folder.id === currentBookmarkFolderId}
                <button
                  type="button"
                  role="radio"
                  aria-checked={selected}
                  class="folder-option"
                  class:selected
                  onclick={() => onAddBookmarkToFolder(folder.id)}
                >
                  <Folder size={15} aria-hidden="true" />
                  <span class="folder-name">{folder.name}</span>
                  <span class="folder-check" aria-hidden="true">
                    {#if selected}<Check size={15} />{/if}
                  </span>
                </button>
              {/each}
            </div>

            {#if creatingFolder}
              <form
                class="new-folder"
                onsubmit={(event) => {
                  event.preventDefault();
                  submitNewFolder();
                }}
              >
                <input
                  class="new-folder-input"
                  bind:value={newFolderName}
                  placeholder="새 폴더 이름"
                  maxlength={24}
                  aria-label="새 폴더 이름"
                  use:focusOnMount
                  onkeydown={(event) => {
                    if (event.key === "Escape") {
                      event.stopPropagation();
                      event.preventDefault();
                      creatingFolder = false;
                      folderError = "";
                    }
                  }}
                />
                <button type="submit" class="new-folder-submit" disabled={!newFolderName.trim()}>추가</button>
              </form>
              {#if folderError}<p class="folder-error" role="alert">{folderError}</p>{/if}
            {:else}
              <button type="button" class="folder-option add" onclick={() => (creatingFolder = true)}>
                <FolderPlus size={15} aria-hidden="true" />
                <span class="folder-name">새 폴더에 저장</span>
              </button>
            {/if}
          </section>

          <div class="popover-foot">
            {#if isFavorite}
              <button type="button" class="text-btn danger" onclick={removeBookmark}>
                <Trash2 size={13} aria-hidden="true" />
                <span>책갈피 삭제</span>
              </button>
            {/if}
            <button type="button" class="done-btn" onclick={() => closeBookmarkPopover(true)}>완료</button>
          </div>
        </div>
      {/if}

      {#if showReaderTools}
        <div
          id="reader-view-options"
          class="reader-popover"
          role="dialog"
          aria-label="읽기 설정"
          bind:this={popoverEl}
          in:popover={{ enter: true }}
          out:popover={{ enter: false }}
        >
          <div class="popover-head">
            <strong>읽기 설정</strong>
            <div class="head-actions">
              <button
                type="button"
                class="text-btn"
                onclick={resetReadingSettings}
                disabled={isDefaultTypography}
              >
                <RotateCcw size={13} aria-hidden="true" />
                <span>기본값</span>
              </button>
              <button type="button" class="icon-btn" aria-label="읽기 설정 닫기" onclick={() => closeSettings(true)}>
                <X size={15} aria-hidden="true" />
              </button>
            </div>
          </div>

          <section class="field">
            <div class="field-head">
              <span class="field-label">글자 크기</span>
              {#key fontSize}
                <output class="font-value" in:popover={{ enter: true }}>{fontSize}%</output>
              {/key}
            </div>
            <div class="font-row">
              <button
                type="button"
                class="step-btn small"
                aria-label="글자 작게"
                disabled={fontSize <= FONT_MIN}
                onclick={() => stepFontSize(-1)}>가</button
              >
              <input
                type="range"
                class="font-slider"
                style={`--font-slider-progress: ${fontSliderProgress};`}
                min={FONT_MIN}
                max={FONT_MAX}
                step="2"
                value={fontSize}
                oninput={handleFontSizeInput}
                aria-label="글자 크기"
                aria-valuetext={`${fontSize}퍼센트`}
              />
              <button
                type="button"
                class="step-btn large"
                aria-label="글자 크게"
                disabled={fontSize >= FONT_MAX}
                onclick={() => stepFontSize(1)}>가</button
              >
            </div>
          </section>

          <section class="field">
            <span class="field-label">줄 간격</span>
            <SegmentedControl
              ariaLabel="줄 간격"
              options={lineHeightOptions}
              value={readerLineHeight}
              onChange={onReaderLineHeightChange}
            />
          </section>

          <section class="field">
            <span class="field-label">본문 폭</span>
            <SegmentedControl
              ariaLabel="본문 폭"
              options={widthOptions}
              value={readerWidth}
              onChange={onReaderWidthChange}
            />
          </section>

          <div class="divider" role="presentation"></div>

          <section class="field switches">
            <span class="field-label">본문 표시</span>
            <Switch
              label="본문 정리"
              description="뜻풀이 번호와 줄바꿈을 읽기 쉽게 정리"
              checked={preprocessEnabled}
              onChange={onTogglePreprocess}
            />
            <Switch
              label="표기 설명"
              description="약어·표기 기호에 설명 표시"
              checked={markerActive}
              disabled={!preprocessEnabled}
              onChange={onToggleMarkerPreprocess}
            />
          </section>
        </div>
      {/if}
    </div>
  </header>

  <div class="reading-progress" aria-hidden="true">
    <span style={`width: ${readingProgress}%`}></span>
  </div>
</div>

<style>
  /* The shell lives in the centered text column; a pseudo-element paints its
     background out to the pane edges (via shadow, so it adds no scroll width)
     so it never reads as a floating slab. Only the pseudo is clipped, which
     leaves the settings popover free to overflow below.
     Font metrics are pinned so the reader's text-size setting doesn't scale
     the toolbar or the settings popover. */
  .doc-sticky-shell {
    position: sticky;
    top: 0;
    z-index: 12;
    isolation: isolate;
    margin-bottom: 20px;
    font-size: 14px;
    line-height: 1.4;
  }

  .doc-sticky-shell::before {
    content: "";
    position: absolute;
    inset: 0;
    z-index: -1;
    background: var(--color-surface);
    box-shadow: 0 0 0 100vmax var(--color-surface);
    clip-path: inset(0 -100vmax);
  }

  /* The divider only appears once the body scrolls underneath. */
  .doc-sticky-shell.compact::before {
    box-shadow:
      0 0 0 100vmax var(--color-surface),
      0 1px 0 100vmax var(--color-border);
    clip-path: inset(0 -100vmax -1px);
  }

  .doc-header {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: 16px;
    min-height: 88px;
    padding: 22px 0 14px;
  }

  .title-block { min-width: 0; }

  .doc-kind {
    display: block;
    margin-bottom: 4px;
    color: var(--color-accent);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
  }

  .doc-title {
    margin: 0;
    font-size: clamp(24px, 2.6vw, 31px);
    font-weight: 720;
    line-height: 1.22;
    letter-spacing: -0.022em;
    overflow-wrap: anywhere;
  }

  /* German headwords get the serif; Korean page titles stay in the sans. */
  .doc-title.headword {
    font-family: var(--font-serif);
    font-size: clamp(28px, 3vw, 36px);
    font-weight: 650;
    line-height: 1.15;
    letter-spacing: -0.01em;
  }

  .doc-actions {
    position: relative;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
  }

  .popover-anchor { display: contents; }

  .doc-sticky-shell :global(.toolbar-action) {
    min-height: 34px;
    font-size: 12px;
    white-space: nowrap;
  }

  .doc-sticky-shell :global(.favorite-btn) { color: var(--color-text); }
  .doc-sticky-shell :global(.back-to-top) { width: 34px; }

  .compact .doc-header {
    min-height: 56px;
    padding: 8px 0;
  }

  .compact .doc-kind { display: none; }

  .compact .doc-title {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    font-size: 19px;
  }

  /* --- Settings popover --- */
  .reader-popover {
    position: absolute;
    top: calc(100% + 10px);
    right: 0;
    z-index: 30;
    width: min(340px, calc(100vw - 32px));
    display: grid;
    gap: 16px;
    padding: 14px 16px 12px;
    box-sizing: border-box;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface-elevated);
    box-shadow: var(--shadow-md);
    color: var(--color-text);
    text-align: left;
    transform-origin: top right;
  }

  .popover-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: -4px;
  }

  .popover-head strong {
    font-size: 14px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }

  .head-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    margin-right: -6px;
  }

  .text-btn,
  .icon-btn,
  .step-btn {
    border: 0;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    transition:
      background-color var(--motion-fast),
      color var(--motion-fast),
      opacity var(--motion-fast);
  }

  .text-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 28px;
    padding: 0 8px;
    border-radius: 8px;
    font-size: 12px;
    font-weight: 600;
  }

  .icon-btn {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 8px;
  }

  .text-btn:hover:not(:disabled),
  .icon-btn:hover,
  .step-btn:hover:not(:disabled) {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  .text-btn:disabled,
  .step-btn:disabled {
    opacity: 0.35;
    cursor: default;
  }

  .text-btn:focus-visible,
  .icon-btn:focus-visible,
  .step-btn:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: 1px;
  }

  .field {
    display: grid;
    gap: 8px;
    min-width: 0;
  }

  .field-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }

  .field-label {
    color: var(--color-text-subtle);
    font-size: 11.5px;
    font-weight: 650;
    letter-spacing: 0.04em;
  }

  .font-value {
    color: var(--color-accent);
    font-size: 12.5px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .font-row {
    display: grid;
    grid-template-columns: 30px minmax(0, 1fr) 30px;
    align-items: center;
    gap: 8px;
  }

  .step-btn {
    width: 30px;
    height: 30px;
    border-radius: 8px;
    color: var(--color-text);
    font-weight: 600;
    line-height: 1;
  }

  .step-btn.small { font-size: 12px; }
  .step-btn.large { font-size: 17px; }

  .font-slider {
    width: 100%;
    height: 30px;
    margin: 0;
    cursor: pointer;
    appearance: none;
    background: transparent;
  }

  .font-slider::-webkit-slider-runnable-track {
    height: 4px;
    border-radius: 99px;
    background: linear-gradient(90deg, var(--color-accent) var(--font-slider-progress), var(--color-divider) var(--font-slider-progress));
  }

  .font-slider::-moz-range-track {
    height: 4px;
    border-radius: 99px;
    background: var(--color-divider);
  }

  .font-slider::-moz-range-progress {
    height: 4px;
    border-radius: 99px;
    background: var(--color-accent);
  }

  .font-slider::-webkit-slider-thumb {
    appearance: none;
    width: 18px;
    height: 18px;
    margin-top: -7px;
    border: 0;
    border-radius: 50%;
    background: #fff;
    box-shadow:
      0 0 0 1px color-mix(in oklab, var(--color-accent), transparent 55%),
      0 1px 3px rgba(0, 0, 0, 0.22);
    transition: transform var(--motion-fast);
  }

  .font-slider:active::-webkit-slider-thumb {
    transform: scale(1.15);
  }

  .font-slider::-moz-range-thumb {
    width: 18px;
    height: 18px;
    border: 0;
    border-radius: 50%;
    background: #fff;
    box-shadow:
      0 0 0 1px color-mix(in oklab, var(--color-accent), transparent 55%),
      0 1px 3px rgba(0, 0, 0, 0.22);
  }

  .font-slider:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: 2px;
    border-radius: 8px;
  }

  /* --- Bookmark popover --- */
  .bookmark-popover {
    width: min(300px, calc(100vw - 32px));
    gap: 12px;
  }

  .saved-title {
    display: inline-flex;
    align-items: center;
    gap: 7px;
  }

  .saved-icon {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 7px;
    background: var(--color-warn-soft-bg);
    color: var(--color-warn-text);
    animation: savedPop 320ms cubic-bezier(0.2, 0.9, 0.3, 1.4);
  }

  @keyframes savedPop {
    from { transform: scale(0.5); opacity: 0; }
    to { transform: scale(1); opacity: 1; }
  }

  .folder-options {
    display: grid;
    /* minmax(0) so a long folder name truncates instead of widening the column. */
    grid-template-columns: minmax(0, 1fr);
    gap: 2px;
    max-height: 216px;
    overflow-y: auto;
  }

  .folder-option {
    display: flex;
    align-items: center;
    gap: 9px;
    min-height: 36px;
    padding: 0 10px;
    width: 100%;
    border: 0;
    border-radius: 9px;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 13px;
    text-align: left;
    cursor: pointer;
    transition: background-color var(--motion-fast), color var(--motion-fast);
  }

  .folder-option:hover {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  .folder-option.selected {
    background: var(--color-accent-soft);
    color: var(--color-accent);
    font-weight: 650;
  }

  .folder-option.add {
    color: var(--color-accent);
    font-weight: 600;
  }

  .folder-option:focus-visible,
  .done-btn:focus-visible,
  .new-folder-submit:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: -2px;
  }

  .folder-name {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .folder-check {
    display: inline-flex;
    width: 15px;
  }

  .new-folder {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 6px;
  }

  .new-folder-input {
    height: 34px;
    min-width: 0;
    padding: 0 10px;
    border: 1px solid var(--color-border);
    border-radius: 9px;
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 13px;
    outline: none;
    transition: border-color var(--motion-fast), box-shadow var(--motion-fast);
  }

  .new-folder-input:focus {
    border-color: var(--color-accent);
    box-shadow: 0 0 0 2px var(--color-focus-ring);
  }

  .new-folder-submit,
  .done-btn {
    height: 34px;
    padding: 0 14px;
    border: 0;
    border-radius: 9px;
    background: var(--color-accent);
    color: var(--color-on-accent);
    font-size: 12.5px;
    font-weight: 650;
    cursor: pointer;
    transition: background-color var(--motion-fast), opacity var(--motion-fast);
  }

  .new-folder-submit:hover:not(:disabled),
  .done-btn:hover {
    background: var(--color-accent-hover);
  }

  .new-folder-submit:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .folder-error {
    margin: 0;
    color: var(--color-danger);
    font-size: 11.5px;
    line-height: 1.4;
  }

  .popover-foot {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    padding-top: 10px;
    border-top: 1px solid var(--color-border);
  }

  .popover-foot .text-btn.danger {
    margin-right: auto;
    margin-left: -8px;
    color: var(--color-danger);
  }

  .popover-foot .text-btn.danger:hover {
    background: var(--color-danger-soft-bg);
    color: var(--color-danger);
  }

  .done-btn {
    height: 30px;
    padding: 0 14px;
  }

  .divider {
    height: 1px;
    margin: -2px 0;
    background: var(--color-border);
  }

  .switches {
    gap: 2px;
  }

  .switches .field-label {
    margin-bottom: 4px;
  }

  .reading-progress {
    height: 2px;
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

    .doc-title { font-size: 26px; }
    .doc-actions { justify-content: flex-start; }

    /* Actions sit under the title on narrow screens: anchor the popover left. */
    .reader-popover {
      left: 0;
      right: auto;
      transform-origin: top left;
    }

    .compact .doc-header {
      grid-template-columns: minmax(0, 1fr) auto;
      gap: 8px;
    }

    .compact .doc-actions :global(.favorite-btn span),
    .compact .doc-actions :global(.settings-button span) {
      display: none;
    }

    .compact .reader-popover {
      left: auto;
      right: 0;
      transform-origin: top right;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .reading-progress span { transition: none; }
  }
</style>
