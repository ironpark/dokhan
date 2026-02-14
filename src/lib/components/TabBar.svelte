<script lang="ts">
  import type { Tab } from '$lib/types/dictionary';

  let {
    activeTab,
    onChange
  }: {
    activeTab: Tab;
    onChange: (tab: Tab) => void;
  } = $props();
</script>

<div class="tabbar" role="tablist" aria-label="사전 탭">
  <button type="button" class:active={activeTab === 'content'} onclick={() => onChange('content')}>내용</button>
  <button type="button" class:active={activeTab === 'index'} onclick={() => onChange('index')}>색인</button>
  <button type="button" class:active={activeTab === 'search'} onclick={() => onChange('search')}>검색</button>
</div>

<style>
  .tabbar {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    background: #f3efe5;
    border-bottom: 1px solid var(--line);
  }

  button {
    border: 0;
    border-right: 1px solid var(--line);
    border-radius: 0;
    background: transparent;
    color: var(--text);
    padding: 10px 8px;
    font-weight: 700;
    cursor: pointer;
  }

  button:last-child {
    border-right: 0;
  }

  button.active {
    background: var(--accent-2);
  }
</style>
