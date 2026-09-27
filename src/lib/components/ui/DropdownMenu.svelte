<script lang="ts">
  import { tick, type Snippet } from "svelte";
  import { nextRovingIndex } from "$lib/utils/rovingFocus";

  let {
    label,
    ariaLabel = label,
    options,
    onSelect,
    trigger,
    class: className = "",
  }: {
    label: string;
    ariaLabel?: string;
    options: Array<{ id: string; label: string; active?: boolean }>;
    onSelect: (id: string) => void;
    /** Custom trigger content (e.g. an icon); `label` is then only the accessible fallback. */
    trigger?: Snippet;
    class?: string;
  } = $props();

  let open = $state(false);
  let rootEl = $state<HTMLDivElement | null>(null);
  let triggerEl = $state<HTMLButtonElement | null>(null);
  let menuEl = $state<HTMLDivElement | null>(null);

  function toggle() {
    if (open) {
      close(true);
    } else {
      void openMenu();
    }
  }

  async function openMenu(focusLast = false) {
    open = true;
    await tick();
    if (!open) return;
    const items = menuItems();
    const target = focusLast ? items?.[items.length - 1] : items?.[0];
    target?.focus();
  }

  function menuItems() {
    return menuEl?.querySelectorAll<HTMLButtonElement>("[role='menuitemradio']");
  }

  function close(restoreFocus = false) {
    open = false;
    if (restoreFocus) {
      queueMicrotask(() => {
        if (triggerEl?.isConnected) triggerEl.focus();
      });
    }
  }

  function handleDocumentClick(event: MouseEvent) {
    if (!open) return;
    const target = event.target as Node | null;
    if (rootEl && target && !rootEl.contains(target)) {
      close();
    }
  }

  function handleDocumentFocus(event: FocusEvent) {
    if (!open) return;
    const target = event.target as Node | null;
    if (rootEl && target && !rootEl.contains(target)) close();
  }

  function handleTriggerKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && open) {
      event.preventDefault();
      close(true);
    } else if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      void openMenu(event.key === "ArrowUp");
    }
  }

  function handleMenuKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      close(true);
      return;
    }

    const items = menuItems();
    if (!items?.length) return;
    const index = Array.from(items).indexOf(document.activeElement as HTMLButtonElement);
    const nextIndex = nextRovingIndex(event.key, index, items.length, "vertical");
    if (nextIndex === null) return;
    event.preventDefault();
    items[nextIndex].focus();
  }

  $effect(() => {
    document.addEventListener("click", handleDocumentClick);
    document.addEventListener("focusin", handleDocumentFocus);
    return () => {
      document.removeEventListener("click", handleDocumentClick);
      document.removeEventListener("focusin", handleDocumentFocus);
    };
  });
</script>

<div class={`dropdown ${className}`} bind:this={rootEl}>
  <button
    bind:this={triggerEl}
    type="button"
    class={`inline-flex min-h-[32px] ${trigger ? "w-8 px-0 !border-transparent !bg-transparent hover:!bg-[var(--color-interactive-hover)]" : "min-w-[52px] px-2"} cursor-pointer items-center justify-center whitespace-nowrap rounded-[7px] border border-[var(--color-border)] bg-[var(--color-surface)] text-[length:var(--font-size-control-sm)] leading-[1.1] text-[var(--color-text-muted)] transition-[background-color,border-color,color] duration-150 hover:border-[var(--color-border-strong)] hover:bg-[var(--color-interactive-hover)] hover:text-[var(--color-text)] focus-visible:outline-none focus-visible:shadow-[0_0_0_2px_var(--color-focus-ring)]`}
    aria-haspopup="menu"
    aria-expanded={open}
    aria-label={ariaLabel}
    onclick={toggle}
    onkeydown={handleTriggerKeydown}
  >
    {#if trigger}{@render trigger()}{:else}{label}{/if}
  </button>

  {#if open}
    <div
      bind:this={menuEl}
      class="absolute right-0 top-[calc(100%+4px)] z-24 grid min-w-[108px] gap-[2px] rounded-[8px] border border-[var(--color-border)] bg-[var(--color-surface-elevated)] p-1 shadow-[var(--shadow-popover)] animate-[menuIn_var(--motion-enter)]"
      role="menu"
      tabindex="-1"
      onkeydown={handleMenuKeydown}
    >
      {#each options as option (option.id)}
        <button
          type="button"
          role="menuitemradio"
          aria-checked={!!option.active}
          class={`cursor-pointer rounded-[6px] border-none bg-transparent px-[6px] py-[5px] text-left text-[length:var(--font-size-control-xs)] focus-visible:outline-none focus-visible:shadow-[inset_0_0_0_1px_var(--color-focus-ring)] ${
            option.active
              ? "bg-[var(--color-accent-soft)] text-[var(--color-accent)]"
              : "text-[var(--color-text-muted)] hover:bg-[var(--color-interactive-hover)] hover:text-[var(--color-text)]"
          }`}
          onclick={() => {
            onSelect(option.id);
            close(true);
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
