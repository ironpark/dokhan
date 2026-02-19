<script lang="ts">
  import UITabBar from "$lib/components/ui/tabs/TabBar.svelte";
  import type { Tab } from "$lib/types/dictionary";

  let {
    activeTab,
    onChange,
    size = "md",
    fullWidth = true,
    scrollable = false,
    animatedIndicator = true,
  }: {
    activeTab: Tab;
    onChange: (tab: Tab) => void;
    size?: "sm" | "md";
    fullWidth?: boolean;
    scrollable?: boolean;
    animatedIndicator?: boolean;
  } = $props();

  const tabs: Array<{ id: Tab; label: string }> = [
    { id: "content", label: "목차" },
    { id: "index", label: "색인" },
    { id: "search", label: "검색" },
    { id: "favorites", label: "책갈피" },
  ];
</script>

<UITabBar
  items={tabs.map((tab) => ({ id: tab.id, label: tab.label }))}
  activeId={activeTab}
  {size}
  {fullWidth}
  {scrollable}
  {animatedIndicator}
  onChange={(id) => onChange(id as Tab)}
/>
