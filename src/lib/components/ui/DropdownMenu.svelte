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
    class="inline-flex min-h-[20px] min-w-[44px] cursor-pointer items-center justify-center whitespace-nowrap rounded-[6px] border border-[var(--color-dokhan-border)] bg-[var(--color-dokhan-surface)] px-[5px] text-[var(--font-size-control-xs)] leading-[1.1] text-[var(--color-text-muted)] transition-[background-color,border-color,color] duration-150 hover:border-[var(--color-border-strong)] hover:bg-[var(--color-interactive-hover)] hover:text-[var(--color-dokhan-text)] focus-visible:outline-none focus-visible:shadow-[0_0_0_2px_var(--color-focus-ring)]"
    aria-haspopup="menu"
    aria-expanded={open}
    onclick={toggle}
  >
    {label}
  </button>

  {#if open}
    <div
      class="absolute right-0 top-[calc(100%+4px)] z-24 grid min-w-[108px] gap-[2px] rounded-[8px] border border-[var(--color-dokhan-border)] bg-[var(--color-surface-elevated)] p-1 shadow-[0_8px_18px_rgba(0,0,0,0.14)] animate-[menuIn_var(--motion-enter)]"
      role="menu"
    >
      {#each options as option (option.id)}
        <button
          type="button"
          role="menuitem"
          class={`cursor-pointer rounded-[6px] border-none bg-transparent px-[6px] py-[5px] text-left text-[var(--font-size-control-xs)] focus-visible:outline-none focus-visible:shadow-[inset_0_0_0_1px_var(--color-focus-ring)] ${
            option.active
              ? "bg-[color-mix(in_oklab,var(--color-dokhan-accent),white_92%)] text-[var(--color-dokhan-accent)]"
              : "text-[var(--color-text-muted)] hover:bg-[var(--color-interactive-hover)] hover:text-[var(--color-dokhan-text)]"
          }`}
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
