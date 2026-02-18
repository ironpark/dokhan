<script lang="ts">
  let {
    title,
    description = "",
    compact = false,
  }: {
    title: string;
    description?: string;
    compact?: boolean;
  } = $props();
</script>

<div class="empty-state" class:compact>
  <p class="title">{title}</p>
  {#if description}
    <p class="description">{description}</p>
  {/if}
</div>

<style>
  .empty-state {
    padding: 18px 14px;
    display: grid;
    gap: 4px;
    color: var(--color-text-muted);
  }

  .empty-state.compact {
    padding: 12px;
    gap: 2px;
  }

  .title {
    margin: 0;
    font-size: 13px;
    color: var(--color-text-muted);
    font-weight: 600;
  }

  .description {
    margin: 0;
    font-size: 12px;
    line-height: 1.4;
    color: var(--color-text-subtle);
  }
</style>
