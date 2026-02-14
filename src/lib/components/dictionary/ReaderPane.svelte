<script lang="ts">
  import type { ContentPage, DetailMode, EntryDetail } from '$lib/types/dictionary';

  let {
    mode,
    selectedContent,
    selectedEntry
  }: {
    mode: DetailMode;
    selectedContent: ContentPage | null;
    selectedEntry: EntryDetail | null;
  } = $props();
</script>

<section class="reader">
  {#if mode === 'content' && selectedContent}
    <article class="body-content">
      <h2>{selectedContent.title}</h2>
      <p>{selectedContent.bodyText}</p>
    </article>
  {:else if mode === 'entry' && selectedEntry}
    <article class="body-content">
      <h2>{selectedEntry.headword}</h2>
      <p class="alias-line">{selectedEntry.aliases.join(' · ')}</p>
      <p>{selectedEntry.definitionText}</p>
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
