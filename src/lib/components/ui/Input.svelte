<script lang="ts">
    import type { Snippet } from "svelte";
    import type { HTMLInputAttributes } from "svelte/elements";

    let {
        value = $bindable(""),
        placeholder = "",
        readonly = false,
        icon,
        class: className = "",
        oninput,
        ...rest
    }: HTMLInputAttributes & {
        value?: string;
        placeholder?: string;
        readonly?: boolean;
        icon?: Snippet;
        class?: string;
    } = $props();
</script>

<div class="input-wrapper {className}">
    {#if icon}
        <div class="icon-slot">
            {@render icon()}
        </div>
    {/if}
    <input
        bind:value
        {placeholder}
        {readonly}
        class:has-icon={!!icon}
        {oninput}
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
        spellcheck="false"
        {...rest}
    />
</div>

<style>
    .input-wrapper {
        position: relative;
        width: 100%;
        color: var(--color-text);
    }

    .icon-slot {
        position: absolute;
        left: var(--space-3);
        top: 50%;
        transform: translateY(-50%);
        pointer-events: none;
        color: var(--color-text-muted);
        display: flex;
        align-items: center;
    }

    input {
        width: 100%;
        height: 36px;
        padding: 0 var(--space-3);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-md);
        background-color: var(--color-surface);
        color: inherit;
        font-family: inherit;
        font-size: 14px;
        transition:
            border-color 0.2s ease,
            box-shadow 0.2s ease;
        box-sizing: border-box;
    }

    input.has-icon {
        padding-left: 36px;
    }

    input:focus {
        outline: none;
        border-color: var(--color-accent);
        box-shadow: 0 0 0 2px rgba(0, 122, 255, 0.2);
    }

    input:read-only {
        background-color: var(--color-surface-hover);
        cursor: default;
    }
</style>
