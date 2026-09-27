<script lang="ts">
  let {
    id,
    label,
    active = false,
    size = "md",
    panelId,
    onSelect,
    onNavigate,
  }: {
    id: string;
    label: string;
    active?: boolean;
    size?: "sm" | "md";
    panelId: string;
    onSelect: (id: string) => void;
    onNavigate: (event: KeyboardEvent, id: string) => void;
  } = $props();

  const sizeClassMap = {
    md: "h-8 px-2 text-[length:var(--font-size-control-sm)]",
    sm: "h-7 px-1 text-[length:var(--font-size-control-sm)]",
  } as const;

  const baseClass =
    "tab-item relative z-[1] inline-flex items-center justify-center border-0 rounded-[8px] bg-transparent " +
    "cursor-pointer leading-none transition-[color] duration-150";

  const hoverClass =
    "hover:text-[var(--color-text)] " +
    "focus-visible:outline-none focus-visible:shadow-[0_0_0_2px_var(--color-focus-ring)]";

  // Active and inactive colors are exclusive so utility order can't decide which wins.
  const activeClass = "font-semibold text-[var(--color-text)]";
  const inactiveClass = "font-medium text-[var(--color-text-muted)]";
</script>

<button
  id={`tab-${id}`}
  type="button"
  role="tab"
  class={`${baseClass} ${hoverClass} ${sizeClassMap[size]} ${active ? activeClass : inactiveClass}`}
  aria-selected={active}
  aria-controls={panelId}
  tabindex={active ? 0 : -1}
  onclick={() => onSelect(id)}
  onkeydown={(event) => onNavigate(event, id)}
>
  <span>{label}</span>
</button>
