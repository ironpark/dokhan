<script lang="ts">
  let {
    open = false,
    message = "",
    tone = "info",
    duration = 2200,
    onOpenChange = () => {},
  }: {
    open?: boolean;
    message?: string;
    tone?: "info" | "error";
    duration?: number;
    onOpenChange?: (next: boolean) => void;
  } = $props();

  $effect(() => {
    if (!open) return;
    const timeout = window.setTimeout(() => {
      onOpenChange(false);
    }, duration);
    return () => window.clearTimeout(timeout);
  });
</script>

{#if open && message}
  <div class={`toast ${tone}`} role="status" aria-live="polite">
    {message}
  </div>
{/if}

<style>
  .toast {
    position: fixed;
    left: 50%;
    bottom: max(20px, calc(12px + env(safe-area-inset-bottom)));
    transform: translateX(-50%);
    z-index: 1450;
    min-width: 220px;
    max-width: min(90vw, 420px);
    padding: 9px 12px;
    border-radius: 10px;
    border: 1px solid var(--color-border);
    background: rgba(17, 20, 25, 0.95);
    color: #f4f7fb;
    font-size: var(--font-size-control-sm);
    box-shadow: 0 10px 24px rgba(0, 0, 0, 0.28);
    animation: toastIn var(--motion-enter);
  }

  .toast.error {
    border-color: var(--color-danger-soft-border);
    background: color-mix(in oklab, var(--color-danger), #15181f 78%);
  }

  @keyframes toastIn {
    from {
      opacity: 0;
      transform: translate(-50%, 4px);
    }
    to {
      opacity: 1;
      transform: translate(-50%, 0);
    }
  }
</style>
