<script lang="ts">
  import type { ContentPage, DetailMode, EntryDetail } from '$lib/types/dictionary';

  let {
    mode,
    selectedContent,
    selectedEntry,
    onOpenHref,
    onResolveImageHref
  }: {
    mode: DetailMode;
    selectedContent: ContentPage | null;
    selectedEntry: EntryDetail | null;
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
  };

  function interceptLinksAndResolveImages(node: HTMLElement, initial: RenderContext) {
    let context = initial;

    const onClick = (event: MouseEvent) => {
      const target = event.target as HTMLElement | null;
      const anchor = target?.closest('a') as HTMLAnchorElement | null;
      if (!anchor) return;
      const href = anchor.getAttribute('href')?.trim();
      if (!href) return;
      event.preventDefault();
      onOpenHref(href, context.sourcePath, context.local);
    };

    async function hydrateImages() {
      const images = Array.from(node.querySelectorAll('img[src]')) as HTMLImageElement[];
      for (const image of images) {
        const src = image.getAttribute('src')?.trim();
        if (!src || src.startsWith('data:') || src.startsWith('http://') || src.startsWith('https://')) {
          continue;
        }
        const resolved = await onResolveImageHref(src, context.sourcePath, context.local);
        if (resolved) {
          image.setAttribute('src', resolved);
        }
      }
    }

    node.addEventListener('click', onClick);
    queueMicrotask(() => {
      void hydrateImages();
    });
    return {
      update(next: RenderContext) {
        context = next;
        void hydrateImages();
      },
      destroy() {
        node.removeEventListener('click', onClick);
      }
    };
  }
</script>

<section class="reader">
  {#if mode === 'content' && selectedContent}
    <article class="body-content">
      {#if selectedContent.bodyHtml}
        <div
          class="html-rendered"
          use:interceptLinksAndResolveImages={{
            sourcePath: selectedContent.sourcePath,
            local: selectedContent.local,
            html: selectedContent.bodyHtml
          }}
        >
          {@html selectedContent.bodyHtml}
        </div>
      {:else}
        <p>{selectedContent.bodyText}</p>
      {/if}
    </article>
  {:else if mode === 'entry' && selectedEntry}
    <article class="body-content">
      <h2>{selectedEntry.headword}</h2>
      <p class="alias-line">{selectedEntry.aliases.join(' · ')}</p>
      {#if selectedEntry.definitionHtml}
        <div
          class="html-rendered"
          use:interceptLinksAndResolveImages={{
            sourcePath: selectedEntry.sourcePath,
            local: null,
            html: selectedEntry.definitionHtml
          }}
        >
          {@html selectedEntry.definitionHtml}
        </div>
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
