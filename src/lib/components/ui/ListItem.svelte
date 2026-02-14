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
        height: 38px;
        border-bottom: 1px solid var(--color-border);
        box-sizing: border-box;
    }

    .list-item:hover {
        background-color: var(--color-surface-hover);
    }

    .list-item.selected {
        background-color: var(--color-surface-active);
    }

    button {
        width: 100%;
        height: 100%;
        border: none;
        background: transparent;
        text-align: left;
        padding: 0 var(--space-2);
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
