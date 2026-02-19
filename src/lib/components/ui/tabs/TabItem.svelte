<script lang="ts">
  let {
    id,
    label,
    active = false,
    onSelect,
  }: {
    id: string;
    label: string;
    active?: boolean;
    onSelect: (id: string) => void;
  } = $props();
</script>

<button
  type="button"
  role="tab"
  class="tab-item"
  class:is-active={active}
  aria-selected={active}
  aria-controls={`panel-${id}`}
  onclick={() => onSelect(id)}
>
  <span>{label}</span>
</button>

<style>
  .tab-item {
    position: relative;
    border: 0;
    border-radius: 0;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    height: 36px;
    padding: 0 var(--space-2);
    font-size: 13px;
    font-weight: 500;
    line-height: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition:
      color var(--motion-fast),
      background-color var(--motion-fast);
  }

  .tab-item:hover {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  .tab-item:focus-visible {
    outline: none;
    box-shadow: inset 0 -2px 0 var(--color-accent);
  }

  .tab-item.is-active {
    color: var(--color-accent);
    background: transparent;
  }

  @media (max-width: 768px) {
    .tab-item {
      height: 30px;
      font-size: 12px;
      padding: 0 var(--space-1);
    }
  }
</style>
