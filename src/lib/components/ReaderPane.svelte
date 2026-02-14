<script lang="ts">
  import type { ContentPage, DetailMode, EntryDetail } from '$lib/types/dictionary';

  let {
    mode,
    selectedContent,
    selectedEntry,
    highlightQuery = '',
    onOpenHref,
    onResolveImageHref
  }: {
    mode: DetailMode;
    selectedContent: ContentPage | null;
    selectedEntry: EntryDetail | null;
    highlightQuery?: string;
    onOpenHref: (href: string, currentSourcePath: string | null, currentLocal: string | null) => void;
    onResolveImageHref: (
      href: string,
      currentSourcePath: string | null,
      currentLocal: string | null
    ) => Promise<string | null>;
  } = $props();

  type RenderContext = {
    sourcePath: string | null;
    local: string | null;
    html: string;
    highlightQuery: string;
  };

  function escapeRegex(text: string): string {
    return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  }

  function clearHighlights(node: HTMLElement) {
    const marks = node.querySelectorAll('mark.search-hit');
    for (const mark of marks) {
      const parent = mark.parentNode;
      if (!parent) continue;
      parent.replaceChild(document.createTextNode(mark.textContent ?? ''), mark);
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
          .filter((term) => term.length > 0)
      )
    );
    if (!terms.length) return;
    terms.sort((a, b) => b.length - a.length);
    const pattern = new RegExp(`(${terms.map(escapeRegex).join('|')})`, 'gi');

    const walker = document.createTreeWalker(node, NodeFilter.SHOW_TEXT);
    const textNodes: Text[] = [];
    let current = walker.nextNode();
    while (current) {
      const textNode = current as Text;
      const parent = textNode.parentElement;
      if (
        parent &&
        !['SCRIPT', 'STYLE', 'MARK'].includes(parent.tagName) &&
        textNode.nodeValue &&
        pattern.test(textNode.nodeValue)
      ) {
        textNodes.push(textNode);
      }
      pattern.lastIndex = 0;
      current = walker.nextNode();
    }

    for (const textNode of textNodes) {
      const text = textNode.nodeValue ?? '';
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
        const mark = document.createElement('mark');
        mark.className = 'search-hit';
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

  function interceptLinksAndResolveImages(node: HTMLElement, initial: RenderContext) {
    let context = initial;
    let revision = 0;

    const onClick = (event: MouseEvent) => {
      const target = event.target as HTMLElement | null;
      const anchor = target?.closest('a') as HTMLAnchorElement | null;
      if (!anchor) return;
      const href = anchor.getAttribute('href')?.trim();
      if (!href) return;
      event.preventDefault();
      onOpenHref(href, context.sourcePath, context.local);
    };

    async function hydrateImages(currentRevision: number, snapshot: RenderContext) {
      const images = Array.from(node.querySelectorAll('img[src]')) as HTMLImageElement[];
      for (const image of images) {
        if (currentRevision !== revision || !node.isConnected) {
          return;
        }
        const src = image.getAttribute('src')?.trim();
        if (!src || src.startsWith('data:') || src.startsWith('http://') || src.startsWith('https://')) {
          continue;
        }
        const resolved = await onResolveImageHref(src, snapshot.sourcePath, snapshot.local);
        if (currentRevision !== revision || !node.isConnected) {
          return;
        }
        if (resolved) {
          image.setAttribute('src', resolved);
        }
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
        try {
          applyHighlights(node, snapshot.highlightQuery);
        } catch {
          clearHighlights(node);
        }
      });
    }

    node.addEventListener('click', onClick);
    scheduleDecorations();
    return {
      update(next: RenderContext) {
        context = next;
        scheduleDecorations();
      },
      destroy() {
        revision += 1;
        node.removeEventListener('click', onClick);
        clearHighlights(node);
      }
    };
  }
</script>

<section class="reader">
  {#if mode === 'content' && selectedContent}
    <article class="body-content">
      {#if selectedContent.bodyHtml}
        {#key `${selectedContent.sourcePath}::${selectedContent.local}::${highlightQuery}`}
          <div
            class="html-rendered"
            use:interceptLinksAndResolveImages={{
              sourcePath: selectedContent.sourcePath,
              local: selectedContent.local,
              html: selectedContent.bodyHtml,
              highlightQuery
            }}
          >
            {@html selectedContent.bodyHtml}
          </div>
        {/key}
      {:else}
        <p>{selectedContent.bodyText}</p>
      {/if}
    </article>
  {:else if mode === 'entry' && selectedEntry}
    <article class="body-content">
      <h2>{selectedEntry.headword}</h2>
      <p class="alias-line">{selectedEntry.aliases.join(' · ')}</p>
      {#if selectedEntry.definitionHtml}
        {#key `${selectedEntry.id}::${highlightQuery}`}
          <div
            class="html-rendered"
            use:interceptLinksAndResolveImages={{
              sourcePath: selectedEntry.sourcePath,
              local: null,
              html: selectedEntry.definitionHtml,
              highlightQuery
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
  min-height: 0;
  overflow: auto;
  padding: 10px 12px;
  background:
    linear-gradient(90deg, rgba(231, 224, 208, 0.35) 1px, transparent 1px) 0 0/22px 22px,
    var(--surface);
}

.body-content {
  max-width: 920px;
}

  .html-rendered {
    font-size: 15px;
    line-height: 1.58;
    color: var(--text);
  }

  .html-rendered :global(p) {
    margin: 0 0 0.72em;
  }

  .html-rendered :global(a) {
    color: var(--accent);
    text-decoration: none;
  }

  .html-rendered :global(a:hover) {
    text-decoration: underline;
  }

  .html-rendered :global(ul),
  .html-rendered :global(ol) {
    margin: 0.3em 0 0.8em;
    padding-left: 1.2em;
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
    margin: 8px 0;
    font-size: 24px;
    letter-spacing: -0.01em;
  }

  .alias-line {
    margin: 6px 0 10px;
    color: #4f483c;
    font-family: 'Alegreya Sans SC', 'IBM Plex Sans', sans-serif;
  }

  .placeholder {
    color: var(--muted);
  }
</style>
