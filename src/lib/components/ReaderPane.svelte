<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import ReaderToolbar from "$lib/components/ReaderToolbar.svelte";
  import Toast from "$lib/components/ui/Toast.svelte";
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
    currentBookmarkFolderId = null,
    onAddBookmarkToFolder = () => {},
    onCreateBookmarkFolder = () => null,
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
    /** Folder of the open item's bookmark; null when it isn't bookmarked. */
    currentBookmarkFolderId?: string | null;
    onAddBookmarkToFolder?: (folderId: string) => void;
    onCreateBookmarkFolder?: (name: string) => string | null;
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
    /** Page title already shown in the toolbar; a matching leading heading is hidden. */
    title?: string;
  };

  function markDuplicateTitle(root: HTMLElement, title: string | undefined) {
    const first = root.firstElementChild;
    if (!title || !first || !/^H[1-3]$/.test(first.tagName)) return;
    const normalize = (text: string) => text.replace(/\s+/g, " ").trim();
    if (normalize(first.textContent ?? "") === normalize(title)) {
      first.classList.add("dict-duplicate-title");
    }
  }

  /** CHM pages often pad with `<h2>&nbsp;</h2>`; hide them so their borders/margins don't leave gaps. */
  function markEmptyHeadings(root: HTMLElement) {
    for (const heading of root.querySelectorAll("h1, h2, h3, h4, h5, h6")) {
      if (heading.querySelector("img")) continue;
      if ((heading.textContent ?? "").replace(/[\s\u00a0]+/g, "") === "") {
        heading.classList.add("dict-empty-heading");
      }
    }
  }

  const readerLineHeightMap: Record<ReaderLineHeight, string> = {
    tight: "1.55",
    normal: "1.7",
    loose: "1.85",
  };
  const readerWidthMap: Record<ReaderWidth, string> = {
    narrow: "680px",
    normal: "780px",
    wide: "980px",
  };

  let showReaderTools = $state(false);
  let linkError = $state("");
  let readerEl = $state<HTMLElement | null>(null);
  let readingProgress = $state(0);
  let isScrolled = $state(false);

  $effect(() => {
    const selected = mode === "entry" ? selectedEntry : selectedContent;
    if (selected && readerEl) readerEl.scrollTop = 0;
    readingProgress = 0;
    isScrolled = false;
  });

  function normalizeFontScale(value: ReaderFontSize): number {
    const rounded = Math.round(value);
    return Math.min(130, Math.max(80, rounded));
  }

  const readerStyleVars = $derived(
    `--reader-font-size: ${(16 * normalizeFontScale(readerFontSize)) / 100}px;` +
      ` --reader-line-height: ${readerLineHeightMap[readerLineHeight] ?? readerLineHeightMap.normal};` +
      ` --reader-max-width: ${readerWidthMap[readerWidth] ?? readerWidthMap.normal};`,
  );

  // The CHM title often repeats the headword verbatim; only show genuinely different names.
  const entryAliases = $derived(
    selectedEntry ? selectedEntry.aliases.filter((alias) => alias.trim() !== selectedEntry.headword.trim()) : [],
  );

  function updateReadingPosition(event: Event) {
    const target = event.currentTarget as HTMLElement;
    const scrollableHeight = target.scrollHeight - target.clientHeight;
    readingProgress = scrollableHeight > 0
      ? Math.min(100, Math.max(0, Math.round((target.scrollTop / scrollableHeight) * 100)))
      : 0;
    isScrolled = target.scrollTop > 160;
  }

  function scrollBehavior(): ScrollBehavior {
    return window.matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth";
  }

  function returnToTop() {
    readerEl?.scrollTo({ top: 0, behavior: scrollBehavior() });
  }

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
      const target = event.target;
      const anchor = target instanceof Element ? target.closest("a") : null;
      if (!anchor) return;
      const href = anchor.getAttribute("href")?.trim();
      if (!href) return;
      event.preventDefault();
      if (href.startsWith("#")) {
        let fragment = href.slice(1);
        try {
          fragment = decodeURIComponent(fragment);
        } catch {
          // Treat malformed percent escapes as a literal fragment.
        }
        const destination = fragment
          ? Array.from(node.querySelectorAll<HTMLElement>("[id], a[name]")).find(
              (element) => element.id === fragment || element.getAttribute("name") === fragment,
            )
          : node;
        if (destination) {
          destination.scrollIntoView({
            behavior: scrollBehavior(),
            block: "start",
          });
        } else {
          linkError = "본문에서 이동할 위치를 찾지 못했습니다.";
        }
        return;
      }
      if (/^(https?:|mailto:)/i.test(href)) {
        linkError = "";
        void openUrl(href).catch(() => {
          linkError = "외부 링크를 열지 못했습니다.";
        });
        return;
      }
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
    let lastHighlightQuery: string | null = null;
    let lastHtml = "";
    const activeObjectUrls = new Set<string>();

    async function hydrateImages(
      currentRevision: number,
      snapshot: RenderContext,
    ) {
      const images = Array.from(
        node.querySelectorAll("img[src]"),
      ) as HTMLImageElement[];
      for (const image of images) {
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
        snapshot.preprocessEnabled ? "1" : "0",
        snapshot.markerPreprocessEnabled ? "1" : "0",
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
      const snapshot = { ...context };
      const nextStructureSignature = computeStructureSignature(snapshot);
      const needsStructureWork =
        snapshot.html !== lastHtml || nextStructureSignature !== lastStructureSignature;
      if (!needsStructureWork && snapshot.highlightQuery === lastHighlightQuery) return;
      const currentRevision = ++revision;
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
          markEmptyHeadings(node);
          markDuplicateTitle(node, snapshot.title);
        }
        if (currentRevision !== revision || !node.isConnected) return;
        try {
          applyHighlights(node, snapshot.highlightQuery);
        } catch {
          clearHighlights(node);
        }
        lastStructureSignature = nextStructureSignature;
        lastHighlightQuery = snapshot.highlightQuery;
        lastHtml = snapshot.html;
        void hydrateImages(currentRevision, snapshot).catch(() => {
          // Keep rendering stable even if media resolution fails.
        });
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
    let touchHideTimer: ReturnType<typeof setTimeout> | null = null;

    function clearTouchHideTimer() {
      if (touchHideTimer) clearTimeout(touchHideTimer);
      touchHideTimer = null;
    }

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
      clearTouchHideTimer();
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

    function onPointerDown(event: PointerEvent) {
      if (event.pointerType !== "touch") return;
      const marker = getMarkerFromTarget(event.target);
      if (!marker) return;
      showTooltip(marker);
      clearTouchHideTimer();
      touchHideTimer = setTimeout(hideTooltip, 3500);
    }

    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape" && activeMarker) hideTooltip();
    }

    function onViewportChange() {
      if (!activeMarker || !tooltipEl) return;
      const markerRect = activeMarker.getBoundingClientRect();
      if (markerRect.bottom < 0 || markerRect.top > window.innerHeight) {
        hideTooltip();
        return;
      }
      positionTooltip(activeMarker, tooltipEl);
    }

    node.addEventListener("mouseover", onMouseOver);
    node.addEventListener("mouseout", onMouseOut);
    node.addEventListener("focusin", onFocusIn);
    node.addEventListener("focusout", onFocusOut);
    node.addEventListener("pointerdown", onPointerDown);
    node.addEventListener("keydown", onKeyDown);
    window.addEventListener("scroll", onViewportChange, true);
    window.addEventListener("resize", onViewportChange);

    return {
      destroy() {
        node.removeEventListener("mouseover", onMouseOver);
        node.removeEventListener("mouseout", onMouseOut);
        node.removeEventListener("focusin", onFocusIn);
        node.removeEventListener("focusout", onFocusOut);
        node.removeEventListener("pointerdown", onPointerDown);
        node.removeEventListener("keydown", onKeyDown);
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

<section
  class="reader"
  style={readerStyleVars}
  bind:this={readerEl}
  onscroll={updateReadingPosition}
  aria-label="사전 본문"
>
  {#if mode === "content" && selectedContent}
    <article class="body-content">
      <ReaderToolbar
        title={selectedContent.title}
        kind="목차"
        {readingProgress}
        {isScrolled}
        onReturnToTop={returnToTop}
        {preprocessEnabled}
        {markerPreprocessEnabled}
        {isFavorite}
        {bookmarkFolders}
        {activeBookmarkFolderId}
        {currentBookmarkFolderId}
        {onAddBookmarkToFolder}
        {onCreateBookmarkFolder}
        {showReaderTools}
        {readerFontSize}
        {readerLineHeight}
        {readerWidth}
        {onTogglePreprocess}
        {onToggleMarkerPreprocess}
        {onToggleFavorite}
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
              title: selectedContent.title,
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
        kind="표제어"
        {readingProgress}
        {isScrolled}
        onReturnToTop={returnToTop}
        {preprocessEnabled}
        {markerPreprocessEnabled}
        {isFavorite}
        {bookmarkFolders}
        {activeBookmarkFolderId}
        {currentBookmarkFolderId}
        {onAddBookmarkToFolder}
        {onCreateBookmarkFolder}
        {showReaderTools}
        {readerFontSize}
        {readerLineHeight}
        {readerWidth}
        {onTogglePreprocess}
        {onToggleMarkerPreprocess}
        {onToggleFavorite}
        onToggleReaderTools={() => (showReaderTools = !showReaderTools)}
        onReaderFontSizeChange={onReaderFontSizeChange}
        onReaderLineHeightChange={onReaderLineHeightChange}
        onReaderWidthChange={onReaderWidthChange}
      />
      {#if entryAliases.length}
        <p class="alias-line"><span class="alias-label">다른 이름</span>{entryAliases.join(" · ")}</p>
      {/if}
      {#if selectedEntry.definitionHtml}
        {#key `${selectedEntry.id}::${selectedEntry.definitionHtml.length}`}
          <div
            class="html-rendered entry-body"
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

<Toast
  open={!!linkError}
  message={linkError}
  tone="error"
  onOpenChange={(next) => {
    if (!next) linkError = "";
  }}
/>

<style>
  .reader {
    height: 100%;
    min-height: 0;
    overflow: auto;
    padding: 0 var(--space-8) var(--space-8);
    background: var(--color-surface);
    color: var(--color-text);
  }

  .body-content {
    max-width: var(--reader-max-width);
    margin: 0 auto;
    font-size: var(--reader-font-size);
    line-height: var(--reader-line-height);
    overflow-wrap: anywhere;
  }

  .html-rendered {
    font-size: inherit;
    line-height: inherit;
  }

  .html-rendered :global(a) {
    color: var(--color-accent-hover);
    text-decoration-thickness: 1px;
    text-underline-offset: 0.18em;
  }

  .html-rendered :global(a:hover) {
    color: var(--color-accent);
    text-decoration-thickness: 2px;
  }

  .html-rendered :global(a:focus-visible) {
    outline: 2px solid var(--color-accent);
    outline-offset: 3px;
    border-radius: 2px;
  }

  .html-rendered :global(blockquote) {
    margin: 1em 0;
    padding: 0.2em 0 0.2em 1em;
    border-left: 3px solid var(--color-divider);
    color: var(--color-text-muted);
  }

  .html-rendered :global(table) {
    display: block;
    max-width: 100%;
    overflow-x: auto;
    margin: 0.4em 0 1em;
    border-collapse: collapse;
    font-size: 0.94em;
  }

  .html-rendered :global(th),
  .html-rendered :global(td) {
    padding: 0.4em 0.75em;
    border: 1px solid var(--color-border);
    text-align: left;
    vertical-align: top;
  }

  .html-rendered :global(th) {
    background: var(--color-surface-soft);
    color: var(--color-text-muted);
    font-weight: 650;
  }

  .html-rendered :global(hr) {
    margin: 1.4em 0;
    border: 0;
    border-top: 1px solid var(--color-border);
  }

  .html-rendered :global(ul),
  .html-rendered :global(ol) {
    margin: 0 0 0.9em;
    padding-left: 1.6em;
  }

  .html-rendered :global(li) {
    margin-bottom: 0.42em;
  }

  /* Tailwind preflight strips list markers; dictionary senses rely on them for 1. / a) numbering. */
  .html-rendered :global(ul) {
    list-style-type: disc;
  }

  .html-rendered :global(ol) {
    list-style-type: decimal;
  }

  .html-rendered :global(ol[type="a"]) {
    list-style-type: lower-alpha;
  }

  .html-rendered :global(ol.dict-subsense-list) {
    list-style-type: dict-paren-alpha;
  }

  .html-rendered :global(li.dict-sense-item::marker),
  .html-rendered :global(li.dict-subsense-item::marker) {
    color: var(--color-accent);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
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

  /* Tailwind preflight flattens headings; restore a clear hierarchy for front-matter pages. */
  .html-rendered :global(h1),
  .html-rendered :global(h2),
  .html-rendered :global(h3),
  .html-rendered :global(h4),
  .html-rendered :global(h5),
  .html-rendered :global(h6) {
    color: var(--color-text);
    font-weight: 700;
    letter-spacing: -0.01em;
  }

  .html-rendered :global(:is(h1, h2, h3, h4, h5, h6):first-child) {
    margin-top: 0;
  }

  .html-rendered :global(.dict-duplicate-title),
  .html-rendered :global(.dict-empty-heading) {
    display: none;
  }

  .html-rendered :global(h1) {
    margin: 1.2em 0 0.6em;
    font-size: calc(var(--reader-font-size) * 1.45);
    line-height: 1.3;
  }

  .html-rendered :global(h2) {
    margin: 1.5em 0 0.6em;
    padding-bottom: 0.3em;
    border-bottom: 1px solid var(--color-border);
    font-size: calc(var(--reader-font-size) * 1.28);
    line-height: 1.32;
  }

  .html-rendered :global(h3) {
    margin: 1.4em 0 0.55em;
    font-size: calc(var(--reader-font-size) * 1.2);
    line-height: 1.35;
  }

  .html-rendered :global(h4) {
    margin: 0.9em 0 0.45em;
    font-size: calc(var(--reader-font-size) * 1.08);
    line-height: 1.35;
  }

  .html-rendered :global(h5),
  .html-rendered :global(h6) {
    margin: 0.8em 0 0.4em;
    font-size: var(--reader-font-size);
    line-height: 1.35;
  }

  .html-rendered :global(img) {
    max-width: 100%;
    height: auto;
  }

  .html-rendered :global(mark.search-hit) {
    background: var(--color-highlight-bg);
    color: var(--color-highlight-text);
    /* Bleed via shadow instead of padding so mid-word hits (Sc|haus|pielen) don't split the word. */
    padding: 0;
    border-radius: 2px;
    box-shadow: -1px 0 0 var(--color-highlight-bg), 1px 0 0 var(--color-highlight-bg);
    -webkit-box-decoration-break: clone;
    box-decoration-break: clone;
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

  .html-rendered :global(span.dict-marker:focus-visible) {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
    border-radius: 2px;
  }

  :global(.marker-tooltip) {
    position: fixed;
    z-index: 1200;
    max-width: min(340px, calc(100vw - 16px));
    width: max-content;
    padding: 7px 9px;
    border-radius: 8px;
    background: var(--color-inverse-bg);
    color: var(--color-inverse-text);
    font-size: 11px;
    line-height: 1.35;
    font-weight: 500;
    letter-spacing: 0;
    white-space: normal;
    box-shadow: var(--shadow-popover);
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
    color: var(--color-marker-register);
  }

  .html-rendered :global(span.dict-marker-round.dict-marker-region) {
    color: var(--color-marker-region);
  }

  .html-rendered :global(span.dict-marker-round.dict-marker-time) {
    color: var(--color-marker-time);
  }

  .html-rendered :global(span.dict-marker-round.dict-marker-usage),
  .html-rendered :global(span.dict-marker-square.dict-marker-usage),
  .html-rendered :global(span.dict-marker-square.dict-marker-meaning) {
    color: var(--color-marker-usage);
  }

  .html-rendered :global(span.dict-marker-square.dict-marker-domain) {
    color: var(--color-marker-domain);
  }

  .html-rendered :global(span.dict-marker-square.dict-marker-orthography) {
    color: var(--color-marker-orthography);
  }

  .html-rendered :global(span.dict-marker-angle.dict-marker-grammar) {
    color: var(--color-marker-grammar);
    font-weight: 600;
  }

  /* Entries open with "<b>Headword</b>, das; -es, …": give the headword a
     dictionary-style serif lead-in. */
  .entry-body > :global(b:first-child),
  .entry-body > :global(p:first-child > b:first-child) {
    font-family: var(--font-serif);
    font-size: 1.28em;
    font-weight: 700;
    letter-spacing: -0.005em;
  }

  .alias-line {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 4px 10px;
    margin: -8px 0 18px;
    color: var(--color-text-muted);
    font-family: var(--font-serif);
    font-size: calc(var(--reader-font-size) * 0.92);
    line-height: inherit;
  }

  .alias-label {
    color: var(--color-text-subtle);
    font-family: var(--font-sans);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.04em;
    white-space: nowrap;
  }

  .placeholder {
    color: var(--color-text-muted);
  }

  @media (max-width: 768px) {
    .reader {
      padding: 0 var(--space-4) var(--space-5);
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
