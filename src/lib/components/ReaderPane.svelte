<script lang="ts">
  import ReaderToolbar from "$lib/components/ReaderToolbar.svelte";
  import type {
    BookmarkFolder,
    ContentPage,
    DetailMode,
    EntryDetail,
    ReaderFontSize,
    ReaderLineHeight,
    ReaderWidth,
  } from "$lib/types/dictionary";
  import { applyDictionaryPreprocess } from "$lib/utils/readerPreprocess";

  let {
    mode,
    selectedContent,
    selectedEntry,
    highlightQuery = "",
    onOpenHref,
    onResolveImageHref,
    isFavorite = false,
    onToggleFavorite = () => {},
    bookmarkFolders = [],
    activeBookmarkFolderId = "default",
    onAddBookmarkToFolder = () => {},
    preprocessEnabled = true,
    onTogglePreprocess = () => {},
    markerPreprocessEnabled = true,
    onToggleMarkerPreprocess = () => {},
    readerFontSize = 100,
    readerLineHeight = "normal",
    readerWidth = "normal",
    onReaderFontSizeChange = () => {},
    onReaderLineHeightChange = () => {},
    onReaderWidthChange = () => {},
  }: {
    mode: DetailMode;
    selectedContent: ContentPage | null;
    selectedEntry: EntryDetail | null;
    highlightQuery?: string;
    onOpenHref: (
      href: string,
      currentSourcePath: string | null,
      currentLocal: string | null,
    ) => void;
    onResolveImageHref: (
      href: string,
      currentSourcePath: string | null,
      currentLocal: string | null,
    ) => Promise<string | null>;
    isFavorite?: boolean;
    onToggleFavorite?: () => void;
    bookmarkFolders?: BookmarkFolder[];
    activeBookmarkFolderId?: string;
    onAddBookmarkToFolder?: (folderId: string) => void;
    preprocessEnabled?: boolean;
    onTogglePreprocess?: () => void;
    markerPreprocessEnabled?: boolean;
    onToggleMarkerPreprocess?: () => void;
    readerFontSize?: ReaderFontSize;
    readerLineHeight?: ReaderLineHeight;
    readerWidth?: ReaderWidth;
    onReaderFontSizeChange?: (value: ReaderFontSize) => void;
    onReaderLineHeightChange?: (value: ReaderLineHeight) => void;
    onReaderWidthChange?: (value: ReaderWidth) => void;
  } = $props();

  type RenderContext = {
    sourcePath: string | null;
    local: string | null;
    html: string;
    highlightQuery: string;
    preprocessEnabled: boolean;
    markerPreprocessEnabled: boolean;
  };

  const readerLineHeightMap: Record<ReaderLineHeight, string> = {
    tight: "1.5",
    normal: "1.62",
    loose: "1.76",
  };
  const readerWidthMap: Record<ReaderWidth, string> = {
    narrow: "760px",
    normal: "860px",
    wide: "980px",
  };

  let showReaderTools = $state(false);
  let showBookmarkFolderDialog = $state(false);
  let bookmarkTargetFolderId = $state("default");

  $effect(() => {
    if (!showBookmarkFolderDialog) {
      bookmarkTargetFolderId = activeBookmarkFolderId;
    }
  });

  function handleFavoriteClick() {
    if (isFavorite) {
      onToggleFavorite();
      return;
    }
    if (bookmarkFolders.length <= 1) {
      onToggleFavorite();
      return;
    }
    bookmarkTargetFolderId = activeBookmarkFolderId;
    showBookmarkFolderDialog = true;
  }

  function cancelBookmarkFolderDialog() {
    showBookmarkFolderDialog = false;
  }

  function confirmBookmarkFolderDialog() {
    onAddBookmarkToFolder(bookmarkTargetFolderId);
    showBookmarkFolderDialog = false;
  }

  function normalizeFontScale(value: ReaderFontSize): number {
    const rounded = Math.round(value);
    return Math.min(130, Math.max(80, rounded));
  }

  const readerStyleVars = $derived(
    `--reader-font-size: ${(15 * normalizeFontScale(readerFontSize)) / 100}px;` +
      ` --reader-line-height: ${readerLineHeightMap[readerLineHeight] ?? readerLineHeightMap.normal};` +
      ` --reader-max-width: ${readerWidthMap[readerWidth] ?? readerWidthMap.normal};`,
  );

  function escapeRegex(text: string): string {
    return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  }

  function clearHighlights(node: HTMLElement) {
    const marks = node.querySelectorAll("mark.search-hit");
    for (const mark of marks) {
      const parent = mark.parentNode;
      if (!parent) continue;
      parent.replaceChild(
        document.createTextNode(mark.textContent ?? ""),
        mark,
      );
      parent.normalize();
    }
  }

  function applyHighlights(node: HTMLElement, query: string) {
    clearHighlights(node);
    const terms = Array.from(
      new Set(
        query
          .split(/\s+/)
          .map((term) => term.trim())
          .filter((term) => term.length > 0),
      ),
    );
    if (!terms.length) return;
    terms.sort((a, b) => b.length - a.length);
    const pattern = new RegExp(`(${terms.map(escapeRegex).join("|")})`, "gi");

    const walker = document.createTreeWalker(node, NodeFilter.SHOW_TEXT);
    const textNodes: Text[] = [];
    let current = walker.nextNode();
    while (current) {
      const textNode = current as Text;
      const parent = textNode.parentElement;
      if (
        parent &&
        !["SCRIPT", "STYLE", "MARK"].includes(parent.tagName) &&
        textNode.nodeValue &&
        pattern.test(textNode.nodeValue)
      ) {
        textNodes.push(textNode);
      }
      pattern.lastIndex = 0;
      current = walker.nextNode();
    }

    for (const textNode of textNodes) {
      const text = textNode.nodeValue ?? "";
      pattern.lastIndex = 0;
      if (!pattern.test(text)) continue;
      pattern.lastIndex = 0;

      const frag = document.createDocumentFragment();
      let last = 0;
      let match: RegExpExecArray | null;
      while ((match = pattern.exec(text)) !== null) {
        const idx = match.index;
        if (idx > last) {
          frag.appendChild(document.createTextNode(text.slice(last, idx)));
        }
        const mark = document.createElement("mark");
        mark.className = "search-hit";
        mark.textContent = match[0];
        frag.appendChild(mark);
        last = idx + match[0].length;
        if (pattern.lastIndex === idx) {
          pattern.lastIndex += 1;
        }
      }
      if (last < text.length) {
        frag.appendChild(document.createTextNode(text.slice(last)));
      }
      textNode.parentNode?.replaceChild(frag, textNode);
    }
  }

  function interceptLinks(
    node: HTMLElement,
    initial: Pick<RenderContext, "sourcePath" | "local">,
  ) {
    let context = initial;
    const onClick = (event: MouseEvent) => {
      const target = event.target as HTMLElement | null;
      const anchor = target?.closest("a") as HTMLAnchorElement | null;
      if (!anchor) return;
      const href = anchor.getAttribute("href")?.trim();
      if (!href) return;
      event.preventDefault();
      onOpenHref(href, context.sourcePath, context.local);
    };

    node.addEventListener("click", onClick);
    return {
      update(next: Pick<RenderContext, "sourcePath" | "local">) {
        context = next;
      },
      destroy() {
        node.removeEventListener("click", onClick);
      },
    };
  }

  function decorateRenderedHtml(
    node: HTMLElement,
    initial: RenderContext,
  ) {
    let context = initial;
    let revision = 0;
    let lastStructureSignature = "";
    let lastHighlightSignature = "";
    const activeObjectUrls = new Set<string>();

    async function hydrateImages(
      currentRevision: number,
      snapshot: RenderContext,
    ) {
      const images = Array.from(
        node.querySelectorAll("img[src]"),
      ) as HTMLImageElement[];
      let processed = 0;
      for (const image of images) {
        if (processed >= 24) {
          return;
        }
        if (currentRevision !== revision || !node.isConnected) {
          return;
        }
        const src = image.getAttribute("src")?.trim();
        if (
          !src ||
          src.startsWith("data:") ||
          src.startsWith("blob:") ||
          src.startsWith("http://") ||
          src.startsWith("https://")
        ) {
          continue;
        }
        processed += 1;
        const resolved = await onResolveImageHref(
          src,
          snapshot.sourcePath,
          snapshot.local,
        );
        if (currentRevision !== revision || !node.isConnected) {
          return;
        }
        if (resolved) {
          if (resolved.startsWith("data:")) {
            try {
              const blob = await (await fetch(resolved)).blob();
              if (currentRevision !== revision || !node.isConnected) {
                return;
              }
              const blobUrl = URL.createObjectURL(blob);
              activeObjectUrls.add(blobUrl);
              image.setAttribute("src", blobUrl);
              continue;
            } catch {
              // Fallback to raw URL assignment below.
            }
          }
          image.setAttribute("src", resolved);
        }
      }
    }

    function revokeObjectUrls() {
      for (const url of activeObjectUrls) {
        URL.revokeObjectURL(url);
      }
      activeObjectUrls.clear();
    }

    function computeStructureSignature(snapshot: RenderContext): string {
      return [
        snapshot.sourcePath ?? "",
        snapshot.local ?? "",
        String(snapshot.html.length),
        snapshot.preprocessEnabled ? "1" : "0",
        snapshot.markerPreprocessEnabled ? "1" : "0",
      ].join("\u0001");
    }

    function computeHighlightSignature(snapshot: RenderContext): string {
      return [
        snapshot.sourcePath ?? "",
        snapshot.local ?? "",
        String(snapshot.html.length),
        snapshot.highlightQuery,
      ].join("\u0001");
    }

    function resetPreprocessFlags() {
      delete node.dataset.combinedSenseSplit;
      delete node.dataset.senseListApplied;
      delete node.dataset.alphaSenseListApplied;
      delete node.dataset.inlineMarkersApplied;
      delete node.dataset.preprocessVersion;
    }

    function scheduleDecorations() {
      const currentRevision = ++revision;
      const snapshot = { ...context };
      const nextStructureSignature = computeStructureSignature(snapshot);
      const nextHighlightSignature = computeHighlightSignature(snapshot);
      const needsStructureWork = nextStructureSignature !== lastStructureSignature;
      const needsHighlightWork = nextHighlightSignature !== lastHighlightSignature;
      if (!needsStructureWork && !needsHighlightWork) return;
      queueMicrotask(async () => {
        if (currentRevision !== revision || !node.isConnected) return;
        if (needsStructureWork) {
          revokeObjectUrls();
          // Always restore the original HTML before optional preprocess.
          // Without this, toggling preprocess off leaves previously transformed DOM intact.
          node.innerHTML = snapshot.html;
          resetPreprocessFlags();
          if (snapshot.preprocessEnabled) {
            try {
              applyDictionaryPreprocess(node, {
                markerTagging: snapshot.markerPreprocessEnabled,
              });
            } catch {
              // Keep rendering stable even if preprocess transformation fails.
            }
          }
        }
        if (needsHighlightWork) {
          if (currentRevision !== revision || !node.isConnected) return;
          try {
            applyHighlights(node, snapshot.highlightQuery);
          } catch {
            clearHighlights(node);
          }
        }
        lastStructureSignature = nextStructureSignature;
        lastHighlightSignature = nextHighlightSignature;
        if (needsStructureWork) {
          void hydrateImages(currentRevision, snapshot).catch(() => {
            // Keep rendering stable even if media resolution fails.
          });
        }
      });
    }

    scheduleDecorations();
    return {
      update(next: RenderContext) {
        context = next;
        scheduleDecorations();
      },
      destroy() {
        revision += 1;
        clearHighlights(node);
        revokeObjectUrls();
      },
    };
  }

  function smartMarkerTooltip(node: HTMLElement) {
    let activeMarker: HTMLElement | null = null;
    let tooltipEl: HTMLDivElement | null = null;

    function ensureTooltip(): HTMLDivElement {
      if (tooltipEl && document.body.contains(tooltipEl)) return tooltipEl;
      tooltipEl = document.createElement("div");
      tooltipEl.className = "marker-tooltip";
      tooltipEl.setAttribute("role", "tooltip");
      tooltipEl.setAttribute("aria-hidden", "true");
      document.body.appendChild(tooltipEl);
      return tooltipEl;
    }

    function getMarkerFromTarget(target: EventTarget | null): HTMLElement | null {
      if (!(target instanceof HTMLElement)) return null;
      const marker = target.closest("span.dict-marker[data-tooltip]");
      return marker instanceof HTMLElement ? marker : null;
    }

    function hideTooltip() {
      activeMarker = null;
      if (!tooltipEl) return;
      tooltipEl.classList.remove("visible");
      tooltipEl.setAttribute("aria-hidden", "true");
    }

    function positionTooltip(marker: HTMLElement, tip: HTMLDivElement) {
      const gap = 10;
      const viewportPadding = 8;
      const markerRect = marker.getBoundingClientRect();
      const tipRect = tip.getBoundingClientRect();

      let left = markerRect.left + markerRect.width / 2 - tipRect.width / 2;
      left = Math.max(
        viewportPadding,
        Math.min(left, window.innerWidth - tipRect.width - viewportPadding),
      );

      let top = markerRect.top - tipRect.height - gap;
      let place = "top";
      if (top < viewportPadding) {
        top = markerRect.bottom + gap;
        place = "bottom";
      }
      if (top + tipRect.height > window.innerHeight - viewportPadding) {
        top = Math.max(
          viewportPadding,
          window.innerHeight - tipRect.height - viewportPadding,
        );
      }

      tip.style.left = `${Math.round(left)}px`;
      tip.style.top = `${Math.round(top)}px`;
      tip.dataset.place = place;
    }

    function applyTooltipTypography(marker: HTMLElement, tip: HTMLDivElement) {
      const readerHost = marker.closest(".reader") as HTMLElement | null;
      const source = readerHost ?? node;
      const readerFontSizeRaw = getComputedStyle(source)
        .getPropertyValue("--reader-font-size")
        .trim();
      const readerFontSize = Number.parseFloat(readerFontSizeRaw);
      if (Number.isFinite(readerFontSize)) {
        const tooltipFontSize = Math.min(
          14,
          Math.max(11, Math.round(readerFontSize * 0.82)),
        );
        tip.style.fontSize = `${tooltipFontSize}px`;
      } else {
        tip.style.fontSize = "";
      }
    }

    function showTooltip(marker: HTMLElement) {
      const text = marker.dataset.tooltip?.trim();
      if (!text) {
        hideTooltip();
        return;
      }

      const tip = ensureTooltip();
      activeMarker = marker;
      tip.textContent = text;
      tip.setAttribute("aria-hidden", "false");
      tip.dataset.place = "top";
      applyTooltipTypography(marker, tip);
      tip.classList.add("visible");
      positionTooltip(marker, tip);
    }

    function onMouseOver(event: MouseEvent) {
      const marker = getMarkerFromTarget(event.target);
      if (!marker || marker === activeMarker) return;
      showTooltip(marker);
    }

    function onMouseOut(event: MouseEvent) {
      if (!activeMarker) return;
      const related = event.relatedTarget as Node | null;
      if (related && activeMarker.contains(related)) return;
      const nextMarker = getMarkerFromTarget(related);
      if (nextMarker) {
        showTooltip(nextMarker);
        return;
      }
      hideTooltip();
    }

    function onFocusIn(event: FocusEvent) {
      const marker = getMarkerFromTarget(event.target);
      if (!marker) return;
      showTooltip(marker);
    }

    function onFocusOut(event: FocusEvent) {
      if (!activeMarker) return;
      const nextMarker = getMarkerFromTarget(event.relatedTarget);
      if (nextMarker) {
        showTooltip(nextMarker);
        return;
      }
      hideTooltip();
    }

    function onViewportChange() {
      if (!activeMarker || !tooltipEl) return;
      positionTooltip(activeMarker, tooltipEl);
    }

    node.addEventListener("mouseover", onMouseOver);
    node.addEventListener("mouseout", onMouseOut);
    node.addEventListener("focusin", onFocusIn);
    node.addEventListener("focusout", onFocusOut);
    window.addEventListener("scroll", onViewportChange, true);
    window.addEventListener("resize", onViewportChange);

    return {
      destroy() {
        node.removeEventListener("mouseover", onMouseOver);
        node.removeEventListener("mouseout", onMouseOut);
        node.removeEventListener("focusin", onFocusIn);
        node.removeEventListener("focusout", onFocusOut);
        window.removeEventListener("scroll", onViewportChange, true);
        window.removeEventListener("resize", onViewportChange);
        hideTooltip();
        if (tooltipEl?.parentNode) {
          tooltipEl.parentNode.removeChild(tooltipEl);
        }
        tooltipEl = null;
      },
    };
  }
</script>

<section class="reader" style={readerStyleVars}>
  {#if mode === "content" && selectedContent}
    <article class="body-content">
      <ReaderToolbar
        title={selectedContent.title}
        {preprocessEnabled}
        {markerPreprocessEnabled}
        {isFavorite}
        {showReaderTools}
        {readerFontSize}
        {readerLineHeight}
        {readerWidth}
        {onTogglePreprocess}
        {onToggleMarkerPreprocess}
        onToggleFavorite={handleFavoriteClick}
        onToggleReaderTools={() => (showReaderTools = !showReaderTools)}
        onReaderFontSizeChange={onReaderFontSizeChange}
        onReaderLineHeightChange={onReaderLineHeightChange}
        onReaderWidthChange={onReaderWidthChange}
      />
      {#if selectedContent.bodyHtml}
        {#key `${selectedContent.sourcePath}::${selectedContent.local}::${selectedContent.bodyHtml.length}`}
          <div
            class="html-rendered"
            use:interceptLinks={{
              sourcePath: selectedContent.sourcePath,
              local: selectedContent.local,
            }}
            use:decorateRenderedHtml={{
              sourcePath: selectedContent.sourcePath,
              local: selectedContent.local,
              html: selectedContent.bodyHtml,
              highlightQuery,
              preprocessEnabled,
              markerPreprocessEnabled,
            }}
            use:smartMarkerTooltip
          >
            {@html selectedContent.bodyHtml}
          </div>
        {/key}
      {:else}
        <p>{selectedContent.bodyText}</p>
      {/if}
    </article>
  {:else if mode === "entry" && selectedEntry}
    <article class="body-content">
      <ReaderToolbar
        title={selectedEntry.headword}
        {preprocessEnabled}
        {markerPreprocessEnabled}
        {isFavorite}
        {showReaderTools}
        {readerFontSize}
        {readerLineHeight}
        {readerWidth}
        {onTogglePreprocess}
        {onToggleMarkerPreprocess}
        onToggleFavorite={handleFavoriteClick}
        onToggleReaderTools={() => (showReaderTools = !showReaderTools)}
        onReaderFontSizeChange={onReaderFontSizeChange}
        onReaderLineHeightChange={onReaderLineHeightChange}
        onReaderWidthChange={onReaderWidthChange}
      />
      <p class="alias-line">{selectedEntry.aliases.join(" · ")}</p>
      {#if selectedEntry.definitionHtml}
        {#key `${selectedEntry.id}::${selectedEntry.definitionHtml.length}`}
          <div
            class="html-rendered"
            use:interceptLinks={{
              sourcePath: selectedEntry.sourcePath,
              local: null,
            }}
            use:decorateRenderedHtml={{
              sourcePath: selectedEntry.sourcePath,
              local: null,
              html: selectedEntry.definitionHtml,
              highlightQuery,
              preprocessEnabled,
              markerPreprocessEnabled,
            }}
            use:smartMarkerTooltip
          >
            {@html selectedEntry.definitionHtml}
          </div>
        {/key}
      {:else}
        <p>{selectedEntry.definitionText}</p>
      {/if}
    </article>
  {:else}
    <article class="body-content placeholder">
      <h2>항목을 선택하세요</h2>
      <p>왼쪽에서 내용/색인/검색 항목을 선택하면 본문이 여기에 표시됩니다.</p>
    </article>
  {/if}
</section>

{#if showBookmarkFolderDialog}
  <dialog class="bookmark-folder-dialog" open aria-label="북마크 폴더 선택">
    <form
      class="bookmark-folder-dialog-form"
      onsubmit={(event) => {
        event.preventDefault();
        confirmBookmarkFolderDialog();
      }}
    >
      <h4>북마크 폴더 선택</h4>
      <p>이 항목을 저장할 폴더를 선택하세요.</p>
      <select bind:value={bookmarkTargetFolderId}>
        {#each bookmarkFolders as folder (folder.id)}
          <option value={folder.id}>{folder.name}</option>
        {/each}
      </select>
      <div class="bookmark-folder-dialog-actions">
        <button type="button" class="ghost" onclick={cancelBookmarkFolderDialog}>취소</button>
        <button type="submit">추가</button>
      </div>
    </form>
  </dialog>
{/if}

<style>
  .reader {
    height: 100%;
    min-height: 0;
    overflow: auto;
    padding: var(--space-5) var(--space-8);
    background: var(--color-surface);
    color: var(--color-text);
  }

  .bookmark-folder-dialog {
    width: min(380px, calc(100vw - 24px));
    border: 1px solid color-mix(in oklab, var(--color-border), white 12%);
    border-radius: 14px;
    background: var(--color-surface);
    color: var(--color-text);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.2);
    padding: 0;
    z-index: 1400;
  }

  .bookmark-folder-dialog::backdrop {
    background: rgba(8, 10, 14, 0.28);
    backdrop-filter: blur(2px);
  }

  .bookmark-folder-dialog-form {
    display: grid;
    gap: 10px;
    padding: 14px;
  }

  .bookmark-folder-dialog-form h4 {
    margin: 0;
    font-size: 15px;
  }

  .bookmark-folder-dialog-form p {
    margin: 0;
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .bookmark-folder-dialog-form select {
    border: 1px solid var(--color-border);
    border-radius: 10px;
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 12px;
    padding: 8px 10px;
  }

  .bookmark-folder-dialog-actions {
    display: inline-flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .bookmark-folder-dialog-actions button {
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text-muted);
    border-radius: 8px;
    padding: 7px 10px;
    font-size: 12px;
    cursor: pointer;
  }

  .bookmark-folder-dialog-actions button:not(.ghost) {
    color: var(--color-accent);
    border-color: color-mix(in oklab, var(--color-accent), white 65%);
    background: color-mix(in oklab, var(--color-accent), white 92%);
  }

  .body-content {
    max-width: var(--reader-max-width);
    margin: 0 auto;
    font-size: var(--reader-font-size);
    line-height: var(--reader-line-height);
  }

  .html-rendered {
    font-size: inherit;
    line-height: inherit;
  }

  .html-rendered :global(ul),
  .html-rendered :global(ol) {
    margin: 0 0 0.9em;
    padding-left: 1.6em;
  }

  .html-rendered :global(li) {
    margin-bottom: 0.42em;
  }

  .html-rendered :global(span.dict-br-spacer) {
    display: block;
    height: 0.52em;
  }

  .html-rendered :global(ol.dict-sense-list) {
    margin: 0.72em 0 0.68em;
    padding-left: 1.68em;
  }

  .html-rendered :global(li.dict-sense-item) {
    margin: 0 0 0.64em;
    line-height: var(--reader-line-height);
    font-size: var(--reader-font-size);
  }

  .html-rendered :global(ol.dict-subsense-list) {
    margin: 0.38em 0 0.2em;
    padding-left: 1.48em;
  }

  .html-rendered :global(li.dict-subsense-item) {
    margin: 0 0 0.32em;
    line-height: var(--reader-line-height);
    font-size: var(--reader-font-size);
  }

  .html-rendered :global(li.dict-sense-item > :first-child),
  .html-rendered :global(li.dict-subsense-item > :first-child) {
    margin-top: 0;
  }

  .html-rendered :global(li.dict-sense-item > :last-child),
  .html-rendered :global(li.dict-subsense-item > :last-child) {
    margin-bottom: 0;
  }

  .html-rendered :global(p) {
    margin: 0 0 0.82em;
    line-height: inherit;
    font-size: inherit;
  }

  .html-rendered :global(h3) {
    margin: 1.1em 0 0.55em;
    font-size: calc(var(--reader-font-size) * 1.2);
    line-height: 1.35;
  }

  .html-rendered :global(h4) {
    margin: 0.9em 0 0.45em;
    font-size: calc(var(--reader-font-size) * 1.08);
    line-height: 1.35;
  }

  .html-rendered :global(img) {
    max-width: 100%;
    height: auto;
  }

  .html-rendered :global(mark.search-hit) {
    background: #ffe38f;
    color: #2b2300;
    padding: 0 1px;
  }

  .html-rendered :global(span.dict-marker) {
    position: relative;
    display: inline;
    margin: 0;
    padding: 0;
    border: 0;
    background: transparent;
    font-size: 0.95em;
    line-height: inherit;
    font-weight: 700;
    letter-spacing: 0;
    cursor: help;
  }

  :global(.marker-tooltip) {
    position: fixed;
    z-index: 1200;
    max-width: min(340px, calc(100vw - 16px));
    width: max-content;
    padding: 7px 9px;
    border-radius: 8px;
    background: rgba(18, 21, 28, 0.96);
    color: #f6f8fb;
    font-size: 11px;
    line-height: 1.35;
    font-weight: 500;
    letter-spacing: 0;
    white-space: normal;
    box-shadow: 0 8px 20px rgba(0, 0, 0, 0.25);
    pointer-events: none;
    opacity: 0;
    visibility: hidden;
    transform: translateY(2px);
    transition:
      opacity 80ms ease,
      transform 80ms ease,
      visibility 80ms ease;
  }

  :global(.marker-tooltip.visible) {
    opacity: 1;
    visibility: visible;
    transform: translateY(0);
  }

  .html-rendered :global(span.dict-marker-round.dict-marker-register) {
    color: #0f5a72;
  }

  .html-rendered :global(span.dict-marker-round.dict-marker-region) {
    color: #2f6a2f;
  }

  .html-rendered :global(span.dict-marker-round.dict-marker-time) {
    color: #8a5a10;
  }

  .html-rendered :global(span.dict-marker-round.dict-marker-usage),
  .html-rendered :global(span.dict-marker-square.dict-marker-usage),
  .html-rendered :global(span.dict-marker-square.dict-marker-meaning) {
    color: #60428a;
  }

  .html-rendered :global(span.dict-marker-square.dict-marker-domain) {
    color: #4a4a4a;
  }

  .html-rendered :global(span.dict-marker-square.dict-marker-orthography) {
    color: #0a5f52;
  }

  .html-rendered :global(span.dict-marker-angle.dict-marker-grammar) {
    color: #9a4f00;
    font-weight: 600;
  }

  .alias-line {
    margin: 5px 0 14px;
    color: var(--color-text-muted);
    font-size: calc(var(--reader-font-size) * 0.92);
    line-height: inherit;
    font-family: "Alegreya Sans SC", "IBM Plex Sans", sans-serif;
  }

  .placeholder {
    color: var(--color-text-muted);
  }

  @media (max-width: 768px) {
    .reader {
      padding: var(--space-4) var(--space-4) var(--space-5);
    }

    .html-rendered :global(ol.dict-sense-list) {
      margin: 0.58em 0 0.52em;
      padding-left: 1.5em;
    }

    .html-rendered :global(li.dict-sense-item) {
      margin: 0 0 0.52em;
      line-height: inherit;
    }

    .html-rendered :global(ol.dict-subsense-list) {
      margin: 0.28em 0 0.14em;
      padding-left: 1.34em;
    }

    .html-rendered :global(li.dict-subsense-item) {
      margin: 0 0 0.26em;
      line-height: inherit;
    }
  }
</style>
