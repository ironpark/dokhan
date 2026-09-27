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
  <div
    class={`fixed left-1/2 z-[1450] min-w-[220px] max-w-[min(90vw,420px)] -translate-x-1/2 rounded-[10px] border px-3 py-[9px] text-[length:var(--font-size-control-sm)] shadow-[var(--shadow-popover)] animate-[toastIn_var(--motion-enter)] ${
      tone === "error"
        ? "border-[var(--color-danger-soft-border)] bg-[var(--color-danger-soft-bg)] text-[var(--color-danger)]"
        : "border-transparent bg-[var(--color-inverse-bg)] text-[var(--color-inverse-text)]"
    }`}
    style="bottom: max(20px, calc(12px + env(safe-area-inset-bottom)));"
    role="status"
    aria-live="polite"
  >
    {message}
  </div>
{/if}

<style>
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
