<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import { tick, type Snippet } from "svelte";
  import { nextRovingIndex } from "$lib/utils/rovingFocus";

  let {
    label,
    ariaLabel = label,
    options,
    onSelect,
    trigger,
    variant = "select",
    heading = "",
    class: className = "",
  }: {
    label: string;
    ariaLabel?: string;
    options: Array<{ id: string; label: string; active?: boolean; danger?: boolean }>;
    /** `select`: pick one of several (radio items, check mark). `action`: plain commands. */
    variant?: "select" | "action";
    /** Optional small caption above the items (e.g. "이동할 폴더"). */
    heading?: string;
    onSelect: (id: string) => void;
    /** Custom trigger content (e.g. an icon); `label` is then only the accessible fallback. */
    trigger?: Snippet;
    class?: string;
  } = $props();

  let open = $state(false);
  let rootEl = $state<HTMLDivElement | null>(null);
  let triggerEl = $state<HTMLButtonElement | null>(null);
  let menuEl = $state<HTMLDivElement | null>(null);
  // Fixed coordinates so the menu escapes `overflow: hidden` cards and scroll containers.
  let menuStyle = $state("");

  const MENU_GAP = 4;
  const VIEWPORT_MARGIN = 8;

  function positionMenu() {
    if (!triggerEl || !menuEl) return;
    const rect = triggerEl.getBoundingClientRect();
    const menuHeight = menuEl.offsetHeight;
    const spaceBelow = window.innerHeight - rect.bottom - VIEWPORT_MARGIN;
    const openUp = spaceBelow < menuHeight + MENU_GAP && rect.top > spaceBelow;
    const top = openUp
      ? Math.max(VIEWPORT_MARGIN, rect.top - MENU_GAP - menuHeight)
      : rect.bottom + MENU_GAP;
    // Right-align with the trigger, but keep the whole menu inside the viewport.
    const maxLeft = window.innerWidth - VIEWPORT_MARGIN - menuEl.offsetWidth;
    const left = Math.max(VIEWPORT_MARGIN, Math.min(rect.right - menuEl.offsetWidth, maxLeft));
    menuStyle = `top: ${top}px; left: ${left}px; transform-origin: ${openUp ? "bottom" : "top"} right;`;
  }

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
    positionMenu();
    const items = menuItems();
    const target = focusLast ? items?.[items.length - 1] : items?.[0];
    target?.focus();
  }

  function menuItems() {
    return menuEl?.querySelectorAll<HTMLButtonElement>("[role='menuitemradio'], [role='menuitem']");
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

  // A fixed menu would drift from its trigger on scroll/resize; closing is simpler and expected.
  $effect(() => {
    if (!open) return;
    const dismiss = () => close();
    window.addEventListener("resize", dismiss);
    document.addEventListener("scroll", dismiss, true);
    return () => {
      window.removeEventListener("resize", dismiss);
      document.removeEventListener("scroll", dismiss, true);
    };
  });

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
      class="fixed z-50 grid grid-cols-[minmax(0,1fr)] min-w-[148px] max-w-[min(260px,calc(100vw-16px))] max-h-[min(320px,60vh)] overflow-y-auto gap-[2px] rounded-[10px] border border-[var(--color-border)] bg-[var(--color-surface-elevated)] p-1 shadow-[var(--shadow-popover)] animate-[menuIn_var(--motion-enter)]"
      style={menuStyle}
      role="menu"
      tabindex="-1"
      onkeydown={handleMenuKeydown}
    >
      {#if heading}
        <p class="m-0 px-[10px] pb-[2px] pt-[6px] text-[11px] font-semibold tracking-[0.04em] text-[var(--color-text-subtle)]" aria-hidden="true">{heading}</p>
      {/if}
      {#each options as option (option.id)}
        <button
          type="button"
          role={variant === "select" ? "menuitemradio" : "menuitem"}
          aria-checked={variant === "select" ? !!option.active : undefined}
          class={`menu-option flex min-h-[34px] cursor-pointer items-center gap-2 rounded-[7px] border-none px-[10px] py-[6px] text-left text-[length:var(--font-size-control-sm)] focus-visible:outline-none focus-visible:shadow-[inset_0_0_0_2px_var(--color-focus-ring)] ${
            option.active
              ? "bg-[var(--color-accent-soft)] font-semibold text-[var(--color-accent)]"
              : option.danger
                ? "bg-transparent text-[var(--color-danger)] hover:bg-[var(--color-danger-soft-bg)]"
                : "bg-transparent text-[var(--color-text)] hover:bg-[var(--color-interactive-hover)]"
          }`}
          onclick={() => {
            onSelect(option.id);
            close(true);
          }}
        >
          <span class="min-w-0 flex-1 truncate" title={option.label}>{option.label}</span>
          {#if variant === "select"}
            <span class="inline-flex w-4 shrink-0" aria-hidden="true">
              {#if option.active}<Check size={14} />{/if}
            </span>
          {/if}
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
