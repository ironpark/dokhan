<script lang="ts">
  import { tick } from "svelte";

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
    const items = menuEl?.querySelectorAll<HTMLButtonElement>("[role='menuitemradio']");
    const target = focusLast ? items?.[items.length - 1] : items?.[0];
    target?.focus();
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

    const items = menuEl?.querySelectorAll<HTMLButtonElement>("[role='menuitemradio']");
    if (!items?.length) return;
    const index = Array.from(items).indexOf(document.activeElement as HTMLButtonElement);
    let nextIndex: number;
    switch (event.key) {
      case "ArrowDown":
        nextIndex = (index + 1) % items.length;
        break;
      case "ArrowUp":
        nextIndex = (index - 1 + items.length) % items.length;
        break;
      case "Home":
        nextIndex = 0;
        break;
      case "End":
        nextIndex = items.length - 1;
        break;
      default:
        return;
    }
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
    class="inline-flex min-h-[32px] min-w-[52px] cursor-pointer items-center justify-center whitespace-nowrap rounded-[7px] border border-[var(--color-dokhan-border)] bg-[var(--color-dokhan-surface)] px-2 text-[var(--font-size-control-sm)] leading-[1.1] text-[var(--color-text-muted)] transition-[background-color,border-color,color] duration-150 hover:border-[var(--color-border-strong)] hover:bg-[var(--color-interactive-hover)] hover:text-[var(--color-dokhan-text)] focus-visible:outline-none focus-visible:shadow-[0_0_0_2px_var(--color-focus-ring)]"
    aria-haspopup="menu"
    aria-expanded={open}
    aria-label={`책갈피 폴더 선택, 현재 ${label}`}
    onclick={toggle}
    onkeydown={handleTriggerKeydown}
  >
    {label}
  </button>

  {#if open}
    <div
      bind:this={menuEl}
      class="absolute right-0 top-[calc(100%+4px)] z-24 grid min-w-[108px] gap-[2px] rounded-[8px] border border-[var(--color-dokhan-border)] bg-[var(--color-surface-elevated)] p-1 shadow-[0_8px_18px_rgba(0,0,0,0.14)] animate-[menuIn_var(--motion-enter)]"
      role="menu"
      tabindex="-1"
      onkeydown={handleMenuKeydown}
    >
      {#each options as option (option.id)}
        <button
          type="button"
          role="menuitemradio"
          aria-checked={!!option.active}
          class={`cursor-pointer rounded-[6px] border-none bg-transparent px-[6px] py-[5px] text-left text-[var(--font-size-control-xs)] focus-visible:outline-none focus-visible:shadow-[inset_0_0_0_1px_var(--color-focus-ring)] ${
            option.active
              ? "bg-[color-mix(in_oklab,var(--color-dokhan-accent),white_92%)] text-[var(--color-dokhan-accent)]"
              : "text-[var(--color-text-muted)] hover:bg-[var(--color-interactive-hover)] hover:text-[var(--color-dokhan-text)]"
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
