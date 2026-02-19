<script lang="ts">
  let {
    label,
    options,
    onSelect,
    class: className = "",
  }: {
    label: string;
    options: Array<{ id: string; label: string; active?: boolean }>;
    onSelect: (id: string) => void;
    class?: string;
  } = $props();

  let open = $state(false);
  let rootEl = $state<HTMLDivElement | null>(null);

  function toggle() {
    open = !open;
  }

  function close() {
    open = false;
  }

  function handleDocumentClick(event: MouseEvent) {
    if (!open) return;
    const target = event.target as Node | null;
    if (rootEl && target && !rootEl.contains(target)) {
      close();
    }
  }

  $effect(() => {
    document.addEventListener("click", handleDocumentClick);
    return () => document.removeEventListener("click", handleDocumentClick);
  });
</script>

<div class={`dropdown ${className}`} bind:this={rootEl}>
  <button
    type="button"
    class="dropdown-trigger"
    aria-haspopup="menu"
    aria-expanded={open}
    onclick={toggle}
  >
    {label}
  </button>

  {#if open}
    <div class="dropdown-menu" role="menu">
      {#each options as option (option.id)}
        <button
          type="button"
          role="menuitem"
          class="dropdown-option"
          class:active={!!option.active}
          onclick={() => {
            onSelect(option.id);
            close();
          }}
        >
          {option.label}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .dropdown {
    position: relative;
  }

  .dropdown-trigger {
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text-muted);
    border-radius: 6px;
    font-size: var(--font-size-control-xs);
    line-height: 1.1;
    min-height: 20px;
    min-width: 44px;
    padding: 0 5px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    white-space: nowrap;
    transition:
      background-color var(--motion-fast),
      border-color var(--motion-fast),
      color var(--motion-fast);
  }

  .dropdown-trigger:hover {
    border-color: var(--color-border-strong);
    color: var(--color-text);
    background: var(--color-interactive-hover);
  }

  .dropdown-trigger:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px var(--color-focus-ring);
  }

  .dropdown-menu {
    position: absolute;
    right: 0;
    top: calc(100% + 4px);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    background: var(--color-surface-elevated);
    box-shadow: 0 8px 18px rgba(0, 0, 0, 0.14);
    min-width: 108px;
    padding: 4px;
    display: grid;
    gap: 2px;
    z-index: 24;
    animation: menuIn var(--motion-enter);
  }

  .dropdown-option {
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    border-radius: 6px;
    font-size: 10px;
    text-align: left;
    padding: 5px 6px;
    cursor: pointer;
  }

  .dropdown-option:hover {
    background: var(--color-interactive-hover);
    color: var(--color-text);
  }

  .dropdown-option:focus-visible {
    outline: none;
    box-shadow: inset 0 0 0 1px var(--color-focus-ring);
  }

  .dropdown-option.active {
    color: var(--color-accent);
    background: color-mix(in oklab, var(--color-accent), white 92%);
  }

  @keyframes menuIn {
    from {
      opacity: 0;
      transform: translateY(-3px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
</style>
