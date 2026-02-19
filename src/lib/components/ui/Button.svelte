<script lang="ts">
    import type { Snippet } from "svelte";
    import type { HTMLButtonAttributes } from "svelte/elements";

    type Variant =
        | "default"
        | "ghost"
        | "outline"
        | "icon"
        | "pill"
        | "pill-active"
        | "soft"
        | "danger-soft";
    type Size = "xs" | "sm" | "md" | "lg" | "icon";

    let {
        variant = "default",
        size = "md",
        class: className = "",
        children,
        onclick,
        disabled = false,
        ...rest
    }: HTMLButtonAttributes & {
        variant?: Variant;
        size?: Size;
        class?: string;
        children?: Snippet;
        disabled?: boolean;
    } = $props();
</script>

<button class="btn {variant} {size} {className}" {onclick} {disabled} {...rest}>
    {@render children?.()}
</button>

<style>
    .btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        border: 1px solid transparent;
        border-radius: var(--radius-sm);
        font-weight: 500;
        cursor: pointer;
        transition:
            background-color var(--motion-fast),
            color var(--motion-fast),
            border-color var(--motion-fast),
            box-shadow var(--motion-fast),
            transform var(--motion-fast);
        background: transparent;
        color: var(--color-text);
        padding: 0;
        line-height: 1;
    }

    .btn:disabled {
        opacity: 0.45;
        pointer-events: none;
    }

    .btn:focus-visible {
        outline: none;
        box-shadow: 0 0 0 2px var(--color-focus-ring);
    }

    /* Variants */
    .btn.default {
        background-color: var(--color-accent);
        color: white;
    }
    .btn.default:hover {
        background-color: var(--color-accent-hover);
    }
    .btn.default:active {
        transform: translateY(0.5px);
    }

    .btn.ghost {
        background-color: transparent;
    }
    .btn.ghost:hover {
        background-color: var(--color-interactive-hover);
    }
    .btn.ghost.active {
        background-color: var(--color-interactive-active);
        color: var(--color-accent);
    }

    .btn.outline {
        border-color: var(--color-border);
        background-color: var(--color-surface);
    }
    .btn.outline:hover {
        background-color: var(--color-interactive-hover);
        border-color: var(--color-border-strong);
    }

    .btn.icon {
        border-radius: var(--radius-full);
        color: var(--color-text-muted);
    }
    .btn.icon:hover {
        background-color: var(--color-interactive-hover);
        color: var(--color-text);
    }

    .btn.pill {
        border-radius: var(--radius-full);
        border-color: transparent;
        background: var(--color-surface);
        color: var(--color-text-muted);
    }
    .btn.pill:hover {
        border-color: var(--color-border-strong);
    }

    .btn.pill-active {
        border-radius: var(--radius-full);
        color: var(--color-accent);
        border-color: color-mix(in oklab, var(--color-accent), white 62%);
        background: var(--color-accent-soft);
    }
    .btn.pill-active:hover {
        border-color: color-mix(in oklab, var(--color-accent), white 52%);
    }

    .btn.soft {
        border-color: var(--color-border);
        background: var(--color-surface);
        color: var(--color-text-muted);
    }
    .btn.soft:hover {
        border-color: var(--color-border-strong);
        background: var(--color-interactive-hover);
    }

    .btn.danger-soft {
        border-color: var(--color-danger-soft-border);
        background: var(--color-danger-soft-bg);
        color: var(--color-danger);
    }
    .btn.danger-soft:hover {
        border-color: var(--color-danger-soft-border-hover);
    }

    /* Sizes */
    .btn.xs {
        height: 24px;
        padding: 0 var(--space-2);
        font-size: var(--font-size-control-sm);
    }
    .btn.sm {
        height: 28px;
        padding: 0 var(--space-2);
        font-size: var(--font-size-control-sm);
    }
    .btn.md {
        height: 36px;
        padding: 0 var(--space-4);
        font-size: var(--font-size-control-md);
    }
    .btn.lg {
        height: 44px;
        padding: 0 var(--space-6);
        font-size: 16px;
    }
    .btn.icon {
        width: 36px;
        height: 36px;
        padding: 0;
    }
</style>
