# Tailwind CSS v4 Usage (SvelteKit)

## 1) Baseline setup
- Vite plugin: `@tailwindcss/vite`
- Global CSS entrypoint (`src/app.css`): `@import "tailwindcss";`
- Do not add legacy `@tailwind base/components/utilities` directives.
- Do not add `tailwind.config.js`/`postcss.config.*` unless project-specific need appears.

## 2) Design token integration
- Keep existing CSS variables as source of truth.
- Expose semantic tokens to utilities with `@theme inline` in `src/app.css`.
- Prefer utilities referencing semantic vars (e.g. `text-[var(--color-dokhan-text)]`).

## 3) Pilot rollout policy
- Start with shared low-risk components first.
- Current pilot targets:
  - `src/lib/components/ui/tabs/TabItem.svelte`
  - `src/lib/components/ui/SectionHeader.svelte`
- Expand only after visual/interaction parity is confirmed.

## 4) Svelte-specific rules
- For `@apply`, `@variant`, `@utility` in component `<style>`, add:

```css
@reference "../../app.css";
```

- Avoid runtime-generated class names such as `bg-${color}`.
- Use static class maps instead:

```ts
const toneClass = {
  info: "text-sky-700 bg-sky-50",
  warn: "text-amber-700 bg-amber-50"
} as const;
```

## 5) Source detection
- Tailwind v4 auto-detects sources.
- If a class is not generated, use `@source` explicitly in CSS for that path.
