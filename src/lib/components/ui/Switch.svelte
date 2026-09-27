<script lang="ts">
  let {
    checked,
    label,
    description = "",
    disabled = false,
    onChange,
  }: {
    checked: boolean;
    label: string;
    description?: string;
    disabled?: boolean;
    onChange: () => void;
  } = $props();

  const id = `switch-${Math.random().toString(36).slice(2, 9)}`;
</script>

<!-- The whole row is the switch, so the label is part of the click target. -->
<button
  type="button"
  role="switch"
  class="switch-row"
  aria-checked={checked}
  aria-labelledby={`${id}-label`}
  aria-describedby={description ? `${id}-desc` : undefined}
  {disabled}
  onclick={onChange}
>
  <span class="copy">
    <span id={`${id}-label`} class="label">{label}</span>
    {#if description}<span id={`${id}-desc`} class="description">{description}</span>{/if}
  </span>
  <span class="track" class:on={checked} aria-hidden="true"><span class="thumb"></span></span>
</button>

<style>
  .switch-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 10px;
    margin: 0 -10px;
    width: calc(100% + 20px);
    border: 0;
    border-radius: 10px;
    background: transparent;
    text-align: left;
    cursor: pointer;
    transition: background-color var(--motion-fast), opacity var(--motion-fast);
  }

  .switch-row:hover:not(:disabled) {
    background: var(--color-surface-hover);
  }

  .switch-row:focus-visible {
    outline: 2px solid var(--color-focus-ring);
    outline-offset: -2px;
  }

  .switch-row:disabled {
    cursor: default;
    opacity: 0.45;
  }

  .copy {
    display: grid;
    gap: 2px;
    min-width: 0;
  }

  .label {
    color: var(--color-text);
    font-size: 13px;
    font-weight: 600;
  }

  .description {
    color: var(--color-text-subtle);
    font-size: 11.5px;
    line-height: 1.4;
  }

  .track {
    position: relative;
    flex: none;
    width: 34px;
    height: 20px;
    border-radius: 999px;
    background: var(--color-surface-active);
    box-shadow: inset 0 0 0 1px var(--color-border-strong);
    transition: background-color var(--motion-base), box-shadow var(--motion-base);
  }

  .track.on {
    background: var(--color-accent);
    box-shadow: none;
  }

  .thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.25);
    transition: transform var(--motion-emphasized);
  }

  .track.on .thumb {
    transform: translateX(14px);
  }
</style>
