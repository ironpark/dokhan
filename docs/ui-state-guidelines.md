# UI State Guidelines

## Purpose
Define consistent interaction states across shared components in Dokhan.

## Required States
- `default`: semantic text/background/border tokens only.
- `hover`: use `--color-interactive-hover` or equivalent semantic token.
- `focus-visible`: always provide a visible ring (`--color-focus-ring`) without shifting layout.
- `active/selected`: use semantic accent tokens and avoid hardcoded colors when possible.
- `disabled`: reduce opacity and disable pointer interactions.

## Motion Rules
- Entering overlays/menus: `--motion-enter`.
- Standard interaction transitions: `--motion-fast`.
- Emphasized movement (e.g., tab indicator): `--motion-emphasized` or `--motion-base`.

## Typography Rules
- Labels: `--font-size-label-sm`.
- Controls: `--font-size-control-sm` / `--font-size-control-md`.
- Preserve readable line-height tokens.

## Accessibility
- Interactive elements must remain keyboard reachable.
- Focus style must not rely only on color changes.
- Keep `aria-*` attributes in sync with actual open/active state.
