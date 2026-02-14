<script lang="ts">
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
  <button
    type="button"
    class:active={activeTab === "content"}
    onclick={() => onChange("content")}>내용</button
  >
  <button
    type="button"
    class:active={activeTab === "index"}
    onclick={() => onChange("index")}>색인</button
  >
  <button
    type="button"
    class:active={activeTab === "search"}
    onclick={() => onChange("search")}>검색</button
  >
</div>

<style>
  .tabbar {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    background: transparent;
    border-bottom: 1px solid var(--color-border);
  }

  button {
    border: none;
    background: none;
    padding: var(--space-2);
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-muted);
    transition: all 0.2s;
    border-bottom: 2px solid transparent;
  }

  button:hover {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  button.active {
    color: var(--color-accent);
    border-bottom-color: var(--color-accent);
  }
</style>
