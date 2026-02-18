<script lang="ts">
  import type {
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
    preprocessEnabled = true,
    onTogglePreprocess = () => {},
    markerPreprocessEnabled = true,
    onToggleMarkerPreprocess = () => {},
    readerFontSize = "md",
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

  const readerFontSizeMap: Record<ReaderFontSize, string> = {
    sm: "14px",
    md: "15px",
    lg: "17px",
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

  const readerStyleVars = $derived(
    `--reader-font-size: ${readerFontSizeMap[readerFontSize] ?? readerFontSizeMap.md};` +
      ` --reader-line-height: ${readerLineHeightMap[readerLineHeight] ?? readerLineHeightMap.normal};` +
      ` --reader-max-width: ${readerWidthMap[readerWidth] ?? readerWidthMap.normal};`,
  );

  function handleFontSizeChange(event: Event) {
    const value = (event.currentTarget as HTMLSelectElement)
      .value as ReaderFontSize;
    onReaderFontSizeChange(value);
  }

  function handleLineHeightChange(event: Event) {
    const value = (event.currentTarget as HTMLSelectElement)
      .value as ReaderLineHeight;
    onReaderLineHeightChange(value);
  }

  function handleWidthChange(event: Event) {
    const value = (event.currentTarget as HTMLSelectElement).value as ReaderWidth;
    onReaderWidthChange(value);
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

  function interceptLinksAndResolveImages(
    node: HTMLElement,
    initial: RenderContext,
  ) {
    let context = initial;
    let revision = 0;
    let lastStructureSignature = "";
    let lastHighlightSignature = "";
    const activeObjectUrls = new Set<string>();

    const onClick = (event: MouseEvent) => {
      const target = event.target as HTMLElement | null;
      const anchor = target?.closest("a") as HTMLAnchorElement | null;
      if (!anchor) return;
      const href = anchor.getAttribute("href")?.trim();
      if (!href) return;
      event.preventDefault();
      onOpenHref(href, context.sourcePath, context.local);
    };

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

    node.addEventListener("click", onClick);
    scheduleDecorations();
    return {
      update(next: RenderContext) {
        context = next;
        scheduleDecorations();
      },
      destroy() {
        revision += 1;
        node.removeEventListener("click", onClick);
        clearHighlights(node);
        revokeObjectUrls();
      },
    };
  }
</script>

<section class="reader" style={readerStyleVars}>
  {#if mode === "content" && selectedContent}
    <article class="body-content">
      <header class="doc-header">
        <h2>{selectedContent.title}</h2>
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
            onclick={() => (showReaderTools = !showReaderTools)}
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
      {#if selectedContent.bodyHtml}
        {#key `${selectedContent.sourcePath}::${selectedContent.local}::${highlightQuery}::${preprocessEnabled}::${markerPreprocessEnabled}`}
          <div
            class="html-rendered"
            use:interceptLinksAndResolveImages={{
              sourcePath: selectedContent.sourcePath,
              local: selectedContent.local,
              html: selectedContent.bodyHtml,
              highlightQuery,
              preprocessEnabled,
              markerPreprocessEnabled,
            }}
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
      <header class="doc-header">
        <h2>{selectedEntry.headword}</h2>
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
            onclick={() => (showReaderTools = !showReaderTools)}
          >
            보기 옵션
          </button>
        </div>
      </header>
      <p class="alias-line">{selectedEntry.aliases.join(" · ")}</p>
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
      {#if selectedEntry.definitionHtml}
        {#key `${selectedEntry.id}::${highlightQuery}::${preprocessEnabled}::${markerPreprocessEnabled}`}
          <div
            class="html-rendered"
            use:interceptLinksAndResolveImages={{
              sourcePath: selectedEntry.sourcePath,
              local: null,
              html: selectedEntry.definitionHtml,
              highlightQuery,
              preprocessEnabled,
              markerPreprocessEnabled,
            }}
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

<style>
  .reader {
    height: 100%;
    min-height: 0;
    overflow: auto;
    padding: var(--space-5) var(--space-8);
    background: var(--color-surface);
    color: var(--color-text);
  }

  .body-content {
    max-width: var(--reader-max-width);
    margin: 0 auto;
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
    line-height: var(--reader-line-height);
    font-size: var(--reader-font-size);
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

  .html-rendered :global(span.dict-marker[data-tooltip]::before) {
    content: attr(data-tooltip);
    position: absolute;
    left: 50%;
    bottom: calc(100% + 10px);
    transform: translateX(-50%) translateY(2px);
    z-index: 30;
    max-width: min(340px, 72vw);
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
    opacity: 0;
    visibility: hidden;
    pointer-events: none;
    transition:
      opacity 80ms ease,
      transform 80ms ease,
      visibility 80ms ease;
  }

  .html-rendered :global(span.dict-marker[data-tooltip]::after) {
    content: "";
    position: absolute;
    left: 50%;
    bottom: calc(100% + 4px);
    transform: translateX(-50%) translateY(2px);
    border: 6px solid transparent;
    border-top-color: rgba(18, 21, 28, 0.96);
    opacity: 0;
    visibility: hidden;
    pointer-events: none;
    transition:
      opacity 80ms ease,
      transform 80ms ease,
      visibility 80ms ease;
  }

  .html-rendered :global(span.dict-marker[data-tooltip]:hover::before),
  .html-rendered :global(span.dict-marker[data-tooltip]:hover::after),
  .html-rendered :global(span.dict-marker[data-tooltip]:focus-visible::before),
  .html-rendered :global(span.dict-marker[data-tooltip]:focus-visible::after) {
    opacity: 1;
    visibility: visible;
    transform: translateX(-50%) translateY(0);
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

  h2 {
    margin: 6px 0 4px;
    font-size: clamp(24px, 3vw, 30px);
    line-height: 1.15;
    letter-spacing: -0.015em;
  }

  .alias-line {
    margin: 5px 0 14px;
    color: var(--color-text-muted);
    font-size: 13px;
    line-height: 1.45;
    font-family: "Alegreya Sans SC", "IBM Plex Sans", sans-serif;
  }

  .placeholder {
    color: var(--color-text-muted);
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

  .mini-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px var(--color-accent-soft);
  }

  @media (max-width: 768px) {
    .reader {
      padding: var(--space-4) var(--space-4) var(--space-5);
    }

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

    .html-rendered :global(ol.dict-sense-list) {
      margin: 0.58em 0 0.52em;
      padding-left: 1.5em;
    }

    .html-rendered :global(li.dict-sense-item) {
      margin: 0 0 0.52em;
      line-height: 1.62;
    }

    .html-rendered :global(ol.dict-subsense-list) {
      margin: 0.28em 0 0.14em;
      padding-left: 1.34em;
    }

    .html-rendered :global(li.dict-subsense-item) {
      margin: 0 0 0.26em;
      line-height: 1.58;
    }
  }
</style>
