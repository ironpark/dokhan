<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import type { Tab } from "$lib/types/dictionary";

  let {
    activeTab,
    onChange,
  }: {
    activeTab: Tab;
    onChange: (tab: Tab) => void;
  } = $props();
</script>

<div class="tabbar" role="tablist" aria-label="사전 탭">
  <Button
    type="button"
    variant="ghost"
    size="sm"
    class={`tab-btn ${activeTab === "content" ? "active" : ""}`}
    onclick={() => onChange("content")}>목차</Button
  >
  <Button
    type="button"
    variant="ghost"
    size="sm"
    class={`tab-btn ${activeTab === "index" ? "active" : ""}`}
    onclick={() => onChange("index")}>색인</Button
  >
  <Button
    type="button"
    variant="ghost"
    size="sm"
    class={`tab-btn ${activeTab === "search" ? "active" : ""}`}
    onclick={() => onChange("search")}>검색</Button
  >
  <Button
    type="button"
    variant="ghost"
    size="sm"
    class={`tab-btn ${activeTab === "favorites" ? "active" : ""}`}
    onclick={() => onChange("favorites")}>북마크</Button
  >
</div>

<style>
  .tabbar {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    background: transparent;
    border-bottom: 1px solid var(--color-border);
  }

  :global(.tab-btn) {
    border-radius: 0;
    height: 36px;
    padding: var(--space-2);
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-muted);
    border-bottom: 2px solid transparent;
  }

  :global(.tab-btn:hover) {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  :global(.tab-btn.active) {
    color: var(--color-accent);
    border-bottom-color: var(--color-accent);
  }
</style>
