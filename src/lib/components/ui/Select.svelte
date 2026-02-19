<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLSelectAttributes } from "svelte/elements";

  let {
    value = $bindable(),
    uiSize = "sm",
    class: className = "",
    children,
    oninput,
    ...rest
  }: HTMLSelectAttributes & {
    value?: string;
    uiSize?: "sm" | "md";
    class?: string;
    children?: Snippet;
  } = $props();
</script>

<select bind:value class={`ui-select ${uiSize} ${className}`} {oninput} {...rest}>
  {@render children?.()}
</select>

<style>
  .ui-select {
    appearance: none;
    border: 1px solid var(--color-border);
    border-radius: 10px;
    background: var(--color-surface);
    color: var(--color-text);
    width: 100%;
    outline: none;
    background-image:
      linear-gradient(45deg, transparent 50%, currentColor 50%),
      linear-gradient(135deg, currentColor 50%, transparent 50%);
    background-position:
      calc(100% - 15px) calc(50% - 2px),
      calc(100% - 10px) calc(50% - 2px);
    background-size:
      5px 5px,
      5px 5px;
    background-repeat: no-repeat;
    transition:
      border-color var(--motion-fast),
      box-shadow var(--motion-fast),
      background-color var(--motion-fast);
  }

  .ui-select.sm {
    font-size: 12px;
    font-weight: 600;
    line-height: 1;
    padding: 8px 30px 8px 10px;
    min-height: 34px;
  }

  .ui-select.md {
    font-size: 14px;
    padding: 9px 30px 9px 10px;
    min-height: 36px;
  }

  .ui-select:hover {
    border-color: var(--color-border-strong);
  }

  .ui-select:focus-visible {
    border-color: var(--color-accent);
    box-shadow: 0 0 0 3px color-mix(in oklab, var(--color-accent), white 80%);
  }
</style>
