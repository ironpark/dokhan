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

  const headerClass =
    "m-0 px-[2px] flex items-center justify-between gap-2";
  const titleClass =
    "m-0 text-[var(--font-size-label-sm)] tracking-[0.05em] uppercase text-[var(--color-text-muted)]";
  const actionsClass = "inline-flex items-center gap-1.5";
</script>

<header class={`${headerClass} ${className}`}>
  <h3 class={titleClass}>{title}</h3>
  {#if actions}
    <div class={actionsClass}>{@render actions()}</div>
  {/if}
</header>
