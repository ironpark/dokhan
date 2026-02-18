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
    <button type="button" {onclick}>
        {@render children()}
    </button>
</li>

<style>
    .list-item {
        position: relative;
        height: 42px;
        border-bottom: 1px solid var(--color-border);
        box-sizing: border-box;
        transition: background-color var(--motion-fast);
    }

    .list-item::before {
        content: "";
        position: absolute;
        left: 0;
        top: 6px;
        bottom: 6px;
        width: 3px;
        border-radius: var(--radius-full);
        background: var(--color-accent);
        opacity: 0;
        transform: scaleY(0.7);
        transition:
            opacity var(--motion-fast),
            transform var(--motion-fast);
    }

    .list-item:hover {
        background-color: var(--color-surface-hover);
    }

    .list-item.selected {
        background-color: var(--color-accent-soft);
    }

    .list-item.selected::before {
        opacity: 1;
        transform: scaleY(1);
    }

    button {
        width: 100%;
        height: 100%;
        border: none;
        background: transparent;
        text-align: left;
        padding: 0 var(--space-3);
        font-size: 14px;
        color: var(--color-text);
        cursor: pointer;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .list-item.selected button {
        color: var(--color-accent);
        font-weight: 600;
    }
</style>
