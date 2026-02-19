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
    class={`fixed left-1/2 z-[1450] min-w-[220px] max-w-[min(90vw,420px)] -translate-x-1/2 rounded-[10px] border px-3 py-[9px] text-[var(--font-size-control-sm)] shadow-[0_10px_24px_rgba(0,0,0,0.28)] animate-[toastIn_var(--motion-enter)] ${
      tone === "error"
        ? "border-[var(--color-danger-soft-border)] bg-[color-mix(in_oklab,var(--color-danger),#15181f_78%)] text-[#f4f7fb]"
        : "border-[var(--color-dokhan-border)] bg-[rgba(17,20,25,0.95)] text-[#f4f7fb]"
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
