<script lang="ts">
  import type {
    ContentPage,
    DetailMode,
    EntryDetail,
  } from "$lib/types/dictionary";

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
  } = $props();

  type RenderContext = {
    sourcePath: string | null;
    local: string | null;
    html: string;
    highlightQuery: string;
    preprocessEnabled: boolean;
  };

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
      for (const image of images) {
        if (currentRevision !== revision || !node.isConnected) {
          return;
        }
        const src = image.getAttribute("src")?.trim();
        if (
          !src ||
          src.startsWith("data:") ||
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
          image.setAttribute("src", resolved);
        }
      }
    }

    function extractSenseNo(node: Node): number | null {
      if (!(node instanceof HTMLSpanElement)) return null;
      const text = node.textContent?.trim() ?? "";
      const match = text.match(/^(\d+)\.$/);
      if (!match) return null;
      const no = Number.parseInt(match[1], 10);
      if (!Number.isFinite(no) || no <= 0) return null;
      return no;
    }

    function extractAlphaSenseNo(node: Node): number | null {
      if (!(node instanceof HTMLSpanElement)) return null;
      const text = node.textContent?.trim() ?? "";
      const match = text.match(/^([a-z])\)$/i);
      if (!match) return null;
      const code = match[1].toLowerCase().charCodeAt(0) - 96;
      if (!Number.isFinite(code) || code <= 0) return null;
      return code;
    }

    function collectMarkers(
      nodes: Node[],
      extract: (node: Node) => number | null,
      minimum = 2,
    ): Array<{ idx: number; no: number }> {
      const markers = nodes
        .map((node, idx) => ({ idx, no: extract(node) }))
        .filter((row): row is { idx: number; no: number } => row.no !== null);
      if (markers.length < minimum) return [];
      for (let i = 1; i < markers.length; i += 1) {
        if (markers[i].no <= markers[i - 1].no) return [];
      }
      return markers;
    }

    function buildOrderedList(
      nodes: Node[],
      markers: Array<{ idx: number; no: number }>,
      options: {
        listClassName: string;
        itemClassName: string;
        type?: "a";
      },
    ): { preface: DocumentFragment; list: HTMLOListElement } {
      const preface = document.createDocumentFragment();
      const first = markers[0];
      for (let i = 0; i < first.idx; i += 1) {
        preface.appendChild(nodes[i]);
      }

      const list = document.createElement("ol");
      list.className = options.listClassName;
      if (options.type) {
        list.setAttribute("type", options.type);
      }
      if (first.no > 1) {
        list.setAttribute("start", String(first.no));
      }

      for (let i = 0; i < markers.length; i += 1) {
        const marker = markers[i];
        const end = i + 1 < markers.length ? markers[i + 1].idx : nodes.length;
        const li = document.createElement("li");
        li.className = options.itemClassName;
        for (let j = marker.idx + 1; j < end; j += 1) {
          li.appendChild(nodes[j]);
        }
        list.appendChild(li);
      }

      return { preface, list };
    }

    function applyAlphaSubSenseList(listItem: HTMLElement) {
      if (listItem.dataset.alphaSenseListApplied === "1") return;
      const nodes = Array.from(listItem.childNodes);
      const markers = collectMarkers(nodes, extractAlphaSenseNo);
      if (!markers.length) return;
      const { preface, list } = buildOrderedList(nodes, markers, {
        listClassName: "dict-subsense-list",
        itemClassName: "dict-subsense-item",
        type: "a",
      });
      listItem.replaceChildren(preface, list);
      listItem.dataset.alphaSenseListApplied = "1";
    }

    function applySenseList(root: HTMLElement) {
      if (root.dataset.senseListApplied === "1") return;
      const nodes = Array.from(root.childNodes);
      const markers = collectMarkers(nodes, extractSenseNo);
      if (!markers.length) return;

      const { preface, list } = buildOrderedList(nodes, markers, {
        listClassName: "dict-sense-list",
        itemClassName: "dict-sense-item",
      });

      root.replaceChildren(preface, list);

      const topLevelItems = Array.from(
        list.querySelectorAll(":scope > li.dict-sense-item"),
      ) as HTMLElement[];
      for (const item of topLevelItems) {
        applyAlphaSubSenseList(item);
      }

      root.dataset.senseListApplied = "1";
    }

    function applyBrSpacing(root: HTMLElement) {
      const breaks = Array.from(root.querySelectorAll("br"));
      for (const br of breaks) {
        if ((br as HTMLElement).dataset.spaced === "1") continue;
        const spacer = document.createElement("span");
        spacer.className = "dict-br-spacer";
        spacer.setAttribute("aria-hidden", "true");
        br.insertAdjacentElement("afterend", spacer);
        (br as HTMLElement).dataset.spaced = "1";
      }
    }

    function scheduleDecorations() {
      const currentRevision = ++revision;
      const snapshot = { ...context };
      queueMicrotask(async () => {
        if (currentRevision !== revision || !node.isConnected) return;
        try {
          await hydrateImages(currentRevision, snapshot);
        } catch {
          // Keep rendering stable even if media resolution fails.
        }
        if (currentRevision !== revision || !node.isConnected) return;
        if (snapshot.preprocessEnabled) {
          try {
            applySenseList(node);
          } catch {
            // Keep rendering stable even if sense list transformation fails.
          }
          if (currentRevision !== revision || !node.isConnected) return;
          try {
            applyBrSpacing(node);
          } catch {
            // Keep rendering stable even if spacing injection fails.
          }
        }
        if (currentRevision !== revision || !node.isConnected) return;
        try {
          applyHighlights(node, snapshot.highlightQuery);
        } catch {
          clearHighlights(node);
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
      },
    };
  }
</script>

<section class="reader">
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
            class="favorite-btn mini-btn"
            class:active={isFavorite}
            onclick={onToggleFavorite}
          >
            {isFavorite ? "★ 저장됨" : "☆ 저장"}
          </button>
        </div>
      </header>
      {#if selectedContent.bodyHtml}
        {#key `${selectedContent.sourcePath}::${selectedContent.local}::${highlightQuery}::${preprocessEnabled}`}
          <div
            class="html-rendered"
            use:interceptLinksAndResolveImages={{
              sourcePath: selectedContent.sourcePath,
              local: selectedContent.local,
              html: selectedContent.bodyHtml,
              highlightQuery,
              preprocessEnabled,
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
            class="favorite-btn mini-btn"
            class:active={isFavorite}
            onclick={onToggleFavorite}
          >
            {isFavorite ? "★ 저장됨" : "☆ 저장"}
          </button>
        </div>
      </header>
      <p class="alias-line">{selectedEntry.aliases.join(" · ")}</p>
      {#if selectedEntry.definitionHtml}
        {#key `${selectedEntry.id}::${highlightQuery}::${preprocessEnabled}`}
          <div
            class="html-rendered"
            use:interceptLinksAndResolveImages={{
              sourcePath: selectedEntry.sourcePath,
              local: null,
              html: selectedEntry.definitionHtml,
              highlightQuery,
              preprocessEnabled,
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
    max-width: 860px;
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
    line-height: 1.66;
    font-size: 15px;
  }

  .html-rendered :global(ol.dict-subsense-list) {
    margin: 0.38em 0 0.2em;
    padding-left: 1.48em;
  }

  .html-rendered :global(li.dict-subsense-item) {
    margin: 0 0 0.32em;
    line-height: 1.62;
    font-size: 15px;
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
    line-height: 1.62;
    font-size: 15px;
  }

  .html-rendered :global(h3) {
    margin: 1.1em 0 0.55em;
    font-size: 18px;
    line-height: 1.35;
  }

  .html-rendered :global(h4) {
    margin: 0.9em 0 0.45em;
    font-size: 16px;
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
    display: inline-flex;
    gap: 6px;
    flex-shrink: 0;
  }

  .mini-btn {
    border: 1px solid var(--color-border);
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
      color var(--motion-fast);
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

  .mini-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px var(--color-accent-soft);
  }

  @media (max-width: 768px) {
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
