<script lang="ts">
  import TabItem from "$lib/components/ui/tabs/TabItem.svelte";
  import { nextRovingIndex } from "$lib/utils/rovingFocus";

  let {
    items,
    activeId,
    onChange,
    size = "md",
    fullWidth = true,
    scrollable = false,
    animatedIndicator = true,
    panelId,
    ariaLabel,
    class: className = "",
  }: {
    items: Array<{ id: string; label: string }>;
    activeId: string;
    onChange: (id: string) => void;
    size?: "sm" | "md";
    fullWidth?: boolean;
    scrollable?: boolean;
    animatedIndicator?: boolean;
    panelId: string;
    ariaLabel: string;
    class?: string;
  } = $props();

  const activeIndex = $derived(Math.max(0, items.findIndex((item) => item.id === activeId)));
  const showIndicator = $derived(fullWidth && !scrollable);

  function handleNavigate(event: KeyboardEvent, id: string) {
    const index = items.findIndex((item) => item.id === id);
    if (index < 0) return;

    const nextIndex = nextRovingIndex(event.key, index, items.length, "horizontal");
    if (nextIndex === null) return;

    event.preventDefault();
    const currentTab = event.currentTarget as HTMLButtonElement;
    const tabs = currentTab.parentElement?.querySelectorAll<HTMLButtonElement>("[role='tab']");
    tabs?.[nextIndex]?.focus();
    onChange(items[nextIndex].id);
  }
</script>

<div
  class={`tabbar ${className} ${
    scrollable
      ? "flex overflow-x-auto [scrollbar-width:thin]"
      : fullWidth
        ? "grid [grid-template-columns:repeat(var(--tab-count),minmax(0,1fr))]"
        : "inline-flex"
  }`}
  class:is-sm={size === "sm"}
  class:full-width={fullWidth}
  class:scrollable={scrollable}
  class:animated={animatedIndicator}
  role="tablist"
  aria-label={ariaLabel}
  style={`--tab-count: ${Math.max(1, items.length)}; --tab-active-index: ${activeIndex};`}
>
  {#if showIndicator}
    <div class="tab-indicator" aria-hidden="true"></div>
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
  /* Segmented control: a recessed track with a raised pill under the active tab. */
  .tabbar {
    --tab-track-pad: 3px;
    position: relative;
    margin: 0;
    padding: var(--tab-track-pad);
    border-radius: 10px;
    background: color-mix(in oklab, var(--color-surface-active), transparent 35%);
  }

  .tab-indicator {
    position: absolute;
    top: var(--tab-track-pad);
    bottom: var(--tab-track-pad);
    left: var(--tab-track-pad);
    width: calc((100% - 2 * var(--tab-track-pad)) / var(--tab-count));
    border-radius: 8px;
    background: var(--color-surface);
    box-shadow:
      0 1px 2px rgba(20, 24, 28, 0.08),
      0 0 0 0.5px rgba(20, 24, 28, 0.06);
    transform: translateX(calc(100% * var(--tab-active-index)));
    pointer-events: none;
  }

  .tabbar.animated .tab-indicator {
    transition: transform var(--motion-emphasized);
  }

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

  @media (prefers-color-scheme: dark) {
    .tab-indicator {
      background: var(--color-surface-active);
      box-shadow: 0 1px 2px rgba(0, 0, 0, 0.4);
    }
  }
</style>
