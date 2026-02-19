<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    open = false,
    ariaLabel,
    title,
    description = "",
    class: className = "",
    onOpenChange = () => {},
    header,
    children,
    actions,
  }: {
    open?: boolean;
    ariaLabel: string;
    title?: string;
    description?: string;
    class?: string;
    onOpenChange?: (next: boolean) => void;
    header?: Snippet;
    children?: Snippet;
    actions?: Snippet;
  } = $props();

  let dialogEl = $state<HTMLDialogElement | null>(null);

  $effect(() => {
    const dialog = dialogEl;
    if (!dialog) return;
    if (open) {
      if (!dialog.open) dialog.showModal();
      return;
    }
    if (dialog.open) dialog.close();
  });

  function closeDialog() {
    onOpenChange(false);
  }

  function onDialogClose() {
    closeDialog();
  }

  function onDialogClick(event: MouseEvent) {
    const dialog = event.currentTarget as HTMLDialogElement | null;
    if (!dialog) return;
    if (event.target === dialog) {
      dialog.close();
    }
  }
</script>

{#if open}
  <dialog
    bind:this={dialogEl}
    class={`ui-dialog w-[min(420px,calc(100vw-24px))] max-w-none rounded-[14px] border border-[color-mix(in_oklab,var(--color-dokhan-border),white_12%)] bg-[var(--color-surface-elevated)] p-0 text-[var(--color-dokhan-text)] shadow-[0_16px_40px_rgba(0,0,0,0.2)] z-[1400] animate-[dialogIn_var(--motion-enter)] ${className}`}
    onclose={onDialogClose}
    onclick={onDialogClick}
    aria-label={ariaLabel}
  >
    <div class="grid gap-2.5 p-3.5">
      {#if header}
        <div class="grid gap-1.5">{@render header()}</div>
      {:else if title}
        <div class="grid gap-1.5">
          <h4 class="m-0 inline-flex items-center gap-2 text-[15px] text-[var(--color-dokhan-text)]">
            {title}
          </h4>
          {#if description}
            <p class="m-0 text-[var(--font-size-control-sm)] text-[var(--color-text-muted)]">{description}</p>
          {/if}
        </div>
      {/if}
      {#if children}
        <div class="grid gap-2.5">{@render children()}</div>
      {/if}
      {#if actions}
        <div class="inline-flex justify-end gap-2">{@render actions()}</div>
      {/if}
    </div>
  </dialog>
{/if}

<style>
  .ui-dialog[open] {
    position: fixed;
    inset: 0;
    margin: auto;
    height: fit-content;
  }

  .ui-dialog::backdrop {
    background: var(--color-overlay);
    backdrop-filter: blur(2px);
  }

  @keyframes dialogIn {
    from {
      opacity: 0;
      transform: translateY(4px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
</style>
