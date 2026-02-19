<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLAnchorAttributes, HTMLButtonAttributes } from "svelte/elements";

  type Variant =
    | "default"
    | "destructive"
    | "outline"
    | "secondary"
    | "ghost"
    | "ghost-danger"
    | "link"
    | "icon"
    | "pill"
    | "pill-active"
    | "toolbar-pill"
    | "toolbar-pill-active"
    | "toolbar-pill-warn-active"
    | "soft"
    | "danger-soft";

  type Size =
    | "default"
    | "xs"
    | "sm"
    | "md"
    | "lg"
    | "icon"
    | "icon-xs"
    | "icon-sm"
    | "icon-lg";

  type ButtonProps = (HTMLButtonAttributes & HTMLAnchorAttributes) & {
    variant?: Variant;
    size?: Size;
    class?: string;
    children?: Snippet;
    href?: string | undefined;
    disabled?: boolean;
  };

  let {
    class: className = "",
    variant = "default",
    size = "default",
    href = undefined,
    type = "button",
    disabled = false,
    children,
    onclick,
    ...restProps
  }: ButtonProps = $props();

  const baseClass =
    "inline-flex shrink-0 items-center justify-center gap-2 whitespace-nowrap font-medium outline-none " +
    "transition-all focus-visible:ring-[3px] focus-visible:ring-[var(--color-focus-ring)] " +
    "disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50 " +
    "[&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4";

  const variantClassMap: Record<Variant, string> = {
    default: "bg-[var(--color-dokhan-accent)] text-white hover:bg-[var(--color-accent-hover)] shadow-xs",
    destructive:
      "bg-[var(--color-danger)] text-white hover:bg-[color-mix(in_oklab,var(--color-danger),black_10%)] shadow-xs",
    outline:
      "border border-[var(--color-dokhan-border)] bg-[var(--color-dokhan-surface)] hover:bg-[var(--color-interactive-hover)] shadow-xs",
    secondary:
      "bg-[var(--color-surface-soft)] text-[var(--color-dokhan-text)] hover:bg-[var(--color-interactive-hover)] shadow-xs",
    ghost: "hover:bg-[var(--color-interactive-hover)] hover:text-[var(--color-dokhan-text)]",
    "ghost-danger":
      "text-[var(--color-text-muted)] hover:bg-[color-mix(in_oklab,var(--color-danger),white_94%)] hover:text-[var(--color-danger)]",
    link: "text-[var(--color-dokhan-accent)] underline-offset-4 hover:underline",
    icon:
      "rounded-[var(--radius-full)] border border-transparent text-[var(--color-text-muted)] hover:bg-[var(--color-interactive-hover)] hover:text-[var(--color-dokhan-text)]",
    pill:
      "!rounded-[var(--radius-full)] border border-transparent bg-[var(--color-dokhan-surface)] text-[var(--color-text-muted)] hover:border-[var(--color-border-strong)]",
    "pill-active":
      "!rounded-[var(--radius-full)] border border-[color-mix(in_oklab,var(--color-dokhan-accent),white_62%)] bg-[var(--color-accent-soft)] text-[var(--color-dokhan-accent)] hover:border-[color-mix(in_oklab,var(--color-dokhan-accent),white_52%)]",
    "toolbar-pill":
      "!rounded-[var(--radius-full)] border border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-text-muted)] hover:border-[var(--color-border-strong)] hover:bg-[var(--color-interactive-hover)] hover:text-[var(--color-text)]",
    "toolbar-pill-active":
      "!rounded-[var(--radius-full)] border border-[color-mix(in_oklab,var(--color-accent),white_62%)] bg-[var(--color-accent-soft)] text-[var(--color-accent)] hover:border-[color-mix(in_oklab,var(--color-accent),white_52%)]",
    "toolbar-pill-warn-active":
      "!rounded-[var(--radius-full)] border border-[#e8ca77] bg-[#fff8dc] text-[#ad7a00] hover:border-[#ddb95a]",
    soft:
      "rounded-[var(--radius-sm)] border border-[var(--color-dokhan-border)] bg-[var(--color-dokhan-surface)] text-[var(--color-text-muted)] hover:border-[var(--color-border-strong)] hover:bg-[var(--color-interactive-hover)]",
    "danger-soft":
      "rounded-[var(--radius-sm)] border border-[var(--color-danger-soft-border)] bg-[var(--color-danger-soft-bg)] text-[var(--color-danger)] hover:border-[var(--color-danger-soft-border-hover)]",
  };

  const sizeClassMap: Record<Size, string> = {
    default: "h-9 px-4 py-2 text-[var(--font-size-control-md)] has-[>svg]:px-3 rounded-[var(--radius-sm)]",
    xs: "h-6 px-2 text-[var(--font-size-control-sm)] rounded-[var(--radius-sm)]",
    sm: "h-8 gap-1.5 px-3 text-[var(--font-size-control-sm)] has-[>svg]:px-2.5 rounded-[var(--radius-sm)]",
    md: "h-9 px-4 text-[var(--font-size-control-md)] rounded-[var(--radius-sm)]",
    lg: "h-10 px-6 text-[var(--font-size-control-md)] has-[>svg]:px-4 rounded-[var(--radius-sm)]",
    icon: "size-9 rounded-[var(--radius-sm)]",
    "icon-xs": "size-7 rounded-[var(--radius-sm)]",
    "icon-sm": "size-8 rounded-[var(--radius-sm)]",
    "icon-lg": "size-10 rounded-[var(--radius-sm)]",
  };

  function cn(...parts: Array<string | false | null | undefined>): string {
    return parts.filter(Boolean).join(" ");
  }

  const classes = $derived(cn(baseClass, sizeClassMap[size], variantClassMap[variant], className));
</script>

{#if href}
  <a
    data-slot="button"
    class={classes}
    href={disabled ? undefined : href}
    aria-disabled={disabled}
    role={disabled ? "link" : undefined}
    tabindex={disabled ? -1 : undefined}
    {onclick}
    {...restProps}
  >
    {@render children?.()}
  </a>
{:else}
  <button data-slot="button" class={classes} {type} {disabled} {onclick} {...restProps}>
    {@render children?.()}
  </button>
{/if}
