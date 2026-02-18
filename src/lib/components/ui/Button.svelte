<script lang="ts">
    import type { Snippet } from "svelte";
    import type { HTMLButtonAttributes } from "svelte/elements";

    type Variant = "default" | "ghost" | "outline" | "icon";
    type Size = "sm" | "md" | "lg" | "icon";

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
        box-shadow: 0 0 0 2px var(--color-accent-soft);
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
        background-color: var(--color-surface-hover);
    }
    .btn.ghost.active {
        background-color: var(--color-surface-active);
        color: var(--color-accent);
    }

    .btn.outline {
        border-color: var(--color-border);
        background-color: var(--color-surface);
    }
    .btn.outline:hover {
        background-color: var(--color-surface-hover);
        border-color: var(--color-border-strong);
    }

    .btn.icon {
        border-radius: var(--radius-full);
        color: var(--color-text-muted);
    }
    .btn.icon:hover {
        background-color: var(--color-surface-hover);
        color: var(--color-text);
    }

    /* Sizes */
    .btn.sm {
        height: 28px;
        padding: 0 var(--space-2);
        font-size: 13px;
    }
    .btn.md {
        height: 36px;
        padding: 0 var(--space-4);
        font-size: 14px;
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
