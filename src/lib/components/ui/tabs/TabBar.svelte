<script lang="ts">
  import TabItem from "$lib/components/ui/tabs/TabItem.svelte";

  let {
    items,
    activeId,
    onChange,
  }: {
    items: Array<{ id: string; label: string }>;
    activeId: string;
    onChange: (id: string) => void;
  } = $props();

  const activeIndex = $derived(Math.max(0, items.findIndex((item) => item.id === activeId)));
</script>

<div
  class="tabbar"
  role="tablist"
  aria-label="사전 탭"
  style={`--tab-count: ${Math.max(1, items.length)}; --tab-active-index: ${activeIndex};`}
>
  <div class="tab-indicator" aria-hidden="true"></div>
  {#each items as item (item.id)}
    <TabItem id={item.id} label={item.label} active={activeId === item.id} onSelect={onChange} />
  {/each}
</div>

<style>
  .tabbar {
    position: relative;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(var(--tab-count), minmax(0, 1fr));
    gap: 0;
    background: transparent;
    border-bottom: 1px solid var(--color-border);
  }

  .tab-indicator {
    position: absolute;
    bottom: 0;
    left: 0;
    width: calc(100% / var(--tab-count));
    height: 2px;
    background: var(--color-accent);
    transform: translateX(calc(100% * var(--tab-active-index)));
    transition: transform var(--motion-base);
    pointer-events: none;
  }
</style>
