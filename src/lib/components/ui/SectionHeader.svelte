<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    class: className = "",
    actions,
  }: {
    title: string;
    class?: string;
    actions?: Snippet;
  } = $props();
</script>

<header class={`section-header ${className}`}>
  <h3>{title}</h3>
  {#if actions}
    <div class="section-actions">{@render actions()}</div>
  {/if}
</header>

<style>
  .section-header {
    margin: 0;
    padding: 0 2px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .section-header h3 {
    margin: 0;
    font-size: var(--font-size-label-sm);
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .section-actions {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
</style>
