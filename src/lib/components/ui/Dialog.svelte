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
    class={`ui-dialog ${className}`}
    onclose={onDialogClose}
    onclick={onDialogClick}
    aria-label={ariaLabel}
  >
    <div class="ui-dialog-surface">
      {#if header}
        <div class="ui-dialog-header custom">{@render header()}</div>
      {:else if title}
        <div class="ui-dialog-header">
          <h4>{title}</h4>
          {#if description}<p>{description}</p>{/if}
        </div>
      {/if}
      {#if children}
        <div class="ui-dialog-body">{@render children()}</div>
      {/if}
      {#if actions}
        <div class="ui-dialog-actions">{@render actions()}</div>
      {/if}
    </div>
  </dialog>
{/if}

<style>
  .ui-dialog {
    width: min(420px, calc(100vw - 24px));
    max-width: none;
    border: 1px solid color-mix(in oklab, var(--color-border), white 12%);
    border-radius: 14px;
    background: var(--color-surface);
    color: var(--color-text);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.2);
    padding: 0;
    z-index: 1400;
  }

  .ui-dialog::backdrop {
    background: rgba(8, 10, 14, 0.28);
    backdrop-filter: blur(2px);
  }

  .ui-dialog-surface {
    display: grid;
    gap: 10px;
    padding: 14px;
  }

  .ui-dialog-header {
    display: grid;
    gap: 6px;
  }

  .ui-dialog-header h4 {
    margin: 0;
    font-size: 15px;
    color: var(--color-text);
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  .ui-dialog-header p {
    margin: 0;
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .ui-dialog-body {
    display: grid;
    gap: 10px;
  }

  .ui-dialog-actions {
    display: inline-flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
