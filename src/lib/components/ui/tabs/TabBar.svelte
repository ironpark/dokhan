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
    panelId = "dictionary-tab-panel",
    class: className = "",
  }: {
    items: Array<{ id: string; label: string }>;
    activeId: string;
    onChange: (id: string) => void;
    size?: "sm" | "md";
    fullWidth?: boolean;
    scrollable?: boolean;
    animatedIndicator?: boolean;
    panelId?: string;
    class?: string;
  } = $props();

  const activeIndex = $derived(Math.max(0, items.findIndex((item) => item.id === activeId)));
  const showIndicator = $derived(fullWidth && !scrollable);

  function handleNavigate(event: KeyboardEvent, id: string) {
    const index = items.findIndex((item) => item.id === id);
    if (index < 0 || !items.length) return;

    let nextIndex: number;
    switch (event.key) {
      case "ArrowRight":
        nextIndex = (index + 1) % items.length;
        break;
      case "ArrowLeft":
        nextIndex = (index - 1 + items.length) % items.length;
        break;
      case "Home":
        nextIndex = 0;
        break;
      case "End":
        nextIndex = items.length - 1;
        break;
      default:
        return;
    }

    event.preventDefault();
    const currentTab = event.currentTarget as HTMLButtonElement;
    const tabs = currentTab.parentElement?.querySelectorAll<HTMLButtonElement>("[role='tab']");
    tabs?.[nextIndex]?.focus();
    onChange(items[nextIndex].id);
  }
</script>

<div
  class={`tabbar relative m-0 border-b border-[var(--color-dokhan-border)] bg-[var(--color-dokhan-surface-soft)] ${className} ${
    scrollable
      ? "flex overflow-x-auto [scrollbar-width:thin]"
      : fullWidth
        ? "grid [grid-template-columns:repeat(var(--tab-count),minmax(0,1fr))]"
        : "inline-flex"
  }`}
  class:is-sm={size === "sm"}
  class:full-width={fullWidth}
  class:scrollable={scrollable}
  role="tablist"
  aria-label="사전 탭"
  style={`--tab-count: ${Math.max(1, items.length)}; --tab-active-index: ${activeIndex};`}
>
  {#if showIndicator}
    <div
      class={`tab-indicator absolute bottom-0 left-0 z-10 h-[2px] w-[calc(100%/var(--tab-count))] bg-[var(--color-dokhan-accent)] pointer-events-none ${
        animatedIndicator ? "transition-transform duration-[220ms] ease-[cubic-bezier(0.2,0.9,0.2,1)]" : ""
      }`}
      style={`transform: translateX(calc(100% * var(--tab-active-index)));`}
      aria-hidden="true"
    ></div>
  {/if}
  {#each items as item (item.id)}
    <TabItem
      id={item.id}
      label={item.label}
      active={activeId === item.id}
      size={size}
      {panelId}
      onSelect={onChange}
      onNavigate={handleNavigate}
    />
  {/each}
</div>

<style>
  .tabbar.scrollable {
    gap: 0;
  }

  .tabbar.scrollable :global(.tab-item) {
    flex: 0 0 auto;
    min-width: 78px;
    padding-inline: 12px;
  }

  .tabbar:not(.full-width):not(.scrollable) :global(.tab-item) {
    min-width: 84px;
  }
</style>
