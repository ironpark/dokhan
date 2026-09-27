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
    md: "h-9 px-2 text-[var(--font-size-control-sm)]",
    sm: "h-8 px-1 text-[var(--font-size-control-sm)]",
  } as const;

  const baseClass =
    "relative inline-flex items-center justify-center border-0 rounded-none bg-transparent " +
    "cursor-pointer font-medium leading-none text-[var(--color-text-muted)] " +
    "transition-[color,background-color] duration-150";

  const hoverClass =
    "hover:bg-[var(--color-interactive-hover)] hover:text-[var(--color-dokhan-text)] " +
    "focus-visible:outline-none focus-visible:shadow-[inset_0_-2px_0_var(--color-dokhan-accent)]";

  const activeClass = "text-[var(--color-dokhan-accent)] bg-transparent";
</script>

<button
  id={`tab-${id}`}
  type="button"
  role="tab"
  class={`${baseClass} ${hoverClass} ${sizeClassMap[size]} ${active ? activeClass : ""}`}
  aria-selected={active}
  aria-controls={panelId}
  tabindex={active ? 0 : -1}
  onclick={() => onSelect(id)}
  onkeydown={(event) => onNavigate(event, id)}
>
  <span>{label}</span>
</button>
