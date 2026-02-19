<script lang="ts">
    import type { Snippet } from "svelte";
    import type { HTMLInputAttributes } from "svelte/elements";

    let {
        value = $bindable(""),
        placeholder = "",
        readonly = false,
        clearable = false,
        uiSize = "md",
        icon,
        onclear,
        class: className = "",
        oninput,
        ...rest
    }: HTMLInputAttributes & {
        value?: string;
        placeholder?: string;
        readonly?: boolean;
        clearable?: boolean;
        uiSize?: "sm" | "md";
        icon?: Snippet;
        onclear?: () => void;
        class?: string;
    } = $props();

    function handleClear() {
        if (!clearable || readonly || !value) return;
        value = "";
        onclear?.();
    }
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
        class:sm={uiSize === "sm"}
        class:has-icon={!!icon}
        class:has-clear={clearable && !readonly && !!value}
        {oninput}
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
        spellcheck="false"
        {...rest}
    />
    {#if clearable && !readonly && value}
        <button
            type="button"
            class="clear-btn"
            aria-label="입력 내용 지우기"
            onclick={handleClear}
        >
            ×
        </button>
    {/if}
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

    input.sm {
        height: 34px;
        font-size: 12px;
        border-radius: 10px;
    }

    input.has-clear {
        padding-right: 34px;
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

    .clear-btn {
        position: absolute;
        right: var(--space-2);
        top: 50%;
        transform: translateY(-50%);
        width: 20px;
        height: 20px;
        border: none;
        border-radius: 999px;
        background: var(--color-surface-active);
        color: var(--color-text-muted);
        font-size: 14px;
        line-height: 1;
        padding: 0;
        cursor: pointer;
        display: inline-flex;
        align-items: center;
        justify-content: center;
    }
</style>
