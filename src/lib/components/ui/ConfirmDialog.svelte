<script lang="ts">
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import Button from "$lib/components/ui/Button.svelte";

  let {
    open,
    title,
    description,
    confirmLabel = "확인",
    cancelLabel = "취소",
    danger = false,
    onCancel,
    onConfirm,
  }: {
    open: boolean;
    title: string;
    description: string;
    confirmLabel?: string;
    cancelLabel?: string;
    danger?: boolean;
    onCancel: () => void;
    onConfirm: () => void;
  } = $props();
</script>

<Dialog open={open} ariaLabel={title} title={title} description={description} onOpenChange={(next) => {
  if (!next) onCancel();
}}>
  {#snippet actions()}
    <Button type="button" size="xs" variant="soft" onclick={onCancel}>{cancelLabel}</Button>
    <Button type="button" size="xs" variant={danger ? "danger-soft" : "pill-active"} onclick={onConfirm}
      >{confirmLabel}</Button
    >
  {/snippet}
</Dialog>
