<script lang="ts">
  import TabItem from "$lib/components/ui/tabs/TabItem.svelte";

  let {
    items,
    activeId,
    onChange,
    size = "md",
    fullWidth = true,
    scrollable = false,
    animatedIndicator = true,
    class: className = "",
  }: {
    items: Array<{ id: string; label: string }>;
    activeId: string;
    onChange: (id: string) => void;
    size?: "sm" | "md";
    fullWidth?: boolean;
    scrollable?: boolean;
    animatedIndicator?: boolean;
    class?: string;
  } = $props();

  const activeIndex = $derived(Math.max(0, items.findIndex((item) => item.id === activeId)));
  const showIndicator = $derived(fullWidth && !scrollable);
</script>

<div
  class={`tabbar ${className}`}
  class:is-sm={size === "sm"}
  class:full-width={fullWidth}
  class:scrollable={scrollable}
  role="tablist"
  aria-label="사전 탭"
  style={`--tab-count: ${Math.max(1, items.length)}; --tab-active-index: ${activeIndex};`}
>
  {#if showIndicator}
    <div class="tab-indicator" class:animated={animatedIndicator} aria-hidden="true"></div>
  {/if}
  {#each items as item (item.id)}
    <TabItem
      id={item.id}
      label={item.label}
      active={activeId === item.id}
      size={size}
      onSelect={onChange}
    />
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

  .tabbar.scrollable {
    display: flex;
    overflow-x: auto;
    scrollbar-width: thin;
  }

  .tabbar.scrollable :global(.tab-item) {
    flex: 0 0 auto;
    min-width: 78px;
    padding-inline: 12px;
  }

  .tabbar:not(.full-width):not(.scrollable) {
    display: inline-flex;
  }

  .tabbar:not(.full-width):not(.scrollable) :global(.tab-item) {
    min-width: 84px;
  }

  .tabbar.is-sm {
    border-bottom-width: 1px;
  }

  .tab-indicator {
    position: absolute;
    bottom: 0;
    left: 0;
    width: calc(100% / var(--tab-count));
    height: 2px;
    background: var(--color-accent);
    transform: translateX(calc(100% * var(--tab-active-index)));
    pointer-events: none;
  }

  .tab-indicator.animated {
    transition: transform var(--motion-base);
  }
</style>
