<script lang="ts">
    import type { Snippet } from "svelte";

    let {
        selected = false,
        onclick,
        children,
        class: className = "",
    }: {
        selected?: boolean;
        onclick?: () => void;
        children: Snippet;
        class?: string;
    } = $props();
</script>

<li class="list-item {className}" class:selected>
    <button type="button" {onclick} aria-current={selected ? "true" : undefined}>
        <span class="label">{@render children()}</span>
    </button>
</li>

<style>
    /* Flat rows; the selection is an inset pill rather than a full-bleed band.
       Total height stays 38px to match virtualized list estimates. */
    .list-item {
        min-height: 38px;
        padding: 1px 8px;
        box-sizing: border-box;
        display: flex;
        align-items: stretch;
    }

    button {
        width: 100%;
        min-height: 36px;
        border: none;
        border-radius: var(--radius-sm);
        background: transparent;
        text-align: left;
        padding: 0 10px;
        display: flex;
        align-items: center;
        font-size: 14px;
        color: var(--color-text);
        cursor: pointer;
        min-width: 0;
        transition:
            background-color var(--motion-fast),
            color var(--motion-fast);
    }

    button:hover {
        background: var(--color-surface-hover);
    }

    .list-item.selected button {
        background: var(--color-accent-soft);
        color: var(--color-accent);
        font-weight: 600;
    }

    /* Ellipsis needs a block-level child; it has no effect on the flex button itself. */
    .label {
        min-width: 0;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    button:focus-visible {
        outline: 2px solid var(--color-focus-ring);
        outline-offset: -2px;
    }
</style>
