<script lang="ts" generics="T extends string">
  import { nextRovingIndex } from "$lib/utils/rovingFocus";

  let {
    options,
    value,
    onChange,
    ariaLabel,
    class: className = "",
  }: {
    options: Array<{ value: T; label: string }>;
    value: T;
    onChange: (value: T) => void;
    ariaLabel: string;
    class?: string;
  } = $props();

  const activeIndex = $derived(Math.max(0, options.findIndex((option) => option.value === value)));

  function handleKeydown(event: KeyboardEvent, index: number) {
    const next = nextRovingIndex(event.key, index, options.length, "horizontal");
    if (next === null) return;
    event.preventDefault();
    const group = (event.currentTarget as HTMLElement).parentElement;
    group?.querySelectorAll<HTMLButtonElement>("[role='radio']")[next]?.focus();
    onChange(options[next].value);
  }
</script>

<!-- Radio-group semantics with the same raised-pill look as the sidebar tabs. -->
<div
  class={`segmented ${className}`}
  role="radiogroup"
  aria-label={ariaLabel}
  style={`--seg-count: ${Math.max(1, options.length)}; --seg-index: ${activeIndex};`}
>
  <span class="indicator" aria-hidden="true"></span>
  {#each options as option, index (option.value)}
    <button
      type="button"
      role="radio"
      aria-checked={option.value === value}
      tabindex={option.value === value ? 0 : -1}
      class:active={option.value === value}
      onclick={() => onChange(option.value)}
      onkeydown={(event) => handleKeydown(event, index)}
    >
      {option.label}
    </button>
  {/each}
</div>

<style>
  .segmented {
    --seg-pad: 3px;
    position: relative;
    display: grid;
    grid-template-columns: repeat(var(--seg-count), minmax(0, 1fr));
    padding: var(--seg-pad);
    border-radius: 10px;
    background: color-mix(in oklab, var(--color-surface-active), transparent 35%);
  }

  .indicator {
    position: absolute;
    top: var(--seg-pad);
    bottom: var(--seg-pad);
    left: var(--seg-pad);
    width: calc((100% - 2 * var(--seg-pad)) / var(--seg-count));
    border-radius: 8px;
    background: var(--color-surface);
    box-shadow:
      0 1px 2px rgba(20, 24, 28, 0.08),
      0 0 0 0.5px rgba(20, 24, 28, 0.06);
    transform: translateX(calc(100% * var(--seg-index)));
    transition: transform var(--motion-emphasized);
    pointer-events: none;
  }

  button {
    position: relative;
    z-index: 1;
    height: 30px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 12.5px;
    font-weight: 500;
    cursor: pointer;
    transition: color var(--motion-fast);
  }

  button:hover {
    color: var(--color-text);
  }

  button.active {
    color: var(--color-text);
    font-weight: 650;
  }

  button:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px var(--color-focus-ring);
  }

  @media (prefers-color-scheme: dark) {
    .indicator {
      background: var(--color-surface-active);
      box-shadow: 0 1px 2px rgba(0, 0, 0, 0.4);
    }
  }
</style>
