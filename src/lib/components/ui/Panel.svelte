<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    class: className = "",
    children,
  }: {
    class?: string;
    children?: Snippet;
  } = $props();
</script>

<section class={`ui-panel ${className}`}>
  {@render children?.()}
</section>

<style>
  .ui-panel {
    min-height: 0;
    height: 100%;
    box-sizing: border-box;
  }
</style>
