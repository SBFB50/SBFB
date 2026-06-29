# Factory Operator Accessibility Design System

This is the shipped contract for the Factory Operator shell.

## Runtime Contract

`/preboot.js` runs before the React bundle and applies persisted preferences to
`<html>` with no inline script. The lazy React panel then owns the same
attributes for in-session changes:

- `data-theme="dark|light"`
- `data-contrast="standard|high"`
- `data-pointer="standard|large"`
- `data-text-spacing="standard|loose"`
- `data-font="standard|legible"`
- `data-motion="standard|reduced"`
- `data-scale="100|112|125"`
- `data-shortcuts="off|on"`

The storage key is `factory-operator.accessibility.v1`. Unknown or malformed
values are sanitized before reaching the DOM.

## Coverage

- Low vision: readable type floor, user scale, light theme, high contrast mode,
  visible focus, non-Latin system fallbacks.
- Color blindness: diff and gate states expose text labels in addition to color
  and glyphs.
- Screen readers: skip link, landmarks, status live regions, expanded states,
  and textual state labels for diff lines and gate statuses.
- Motor accessibility: large target mode, responsive rail reflow, and
  single-letter `s/v` shortcuts disabled until the operator opts in.
- Cognitive accessibility: stable focal layout, explicit mode state, persistent
  preferences, and no hidden auto-switching.
- Vestibular/photosensitive: reduced-motion follows the OS by default and can
  be forced in the panel.

## Gate

`npm run gate:accessibility-system` checks that the preboot, public CSS modes,
shortcut guard, reflow, live status, and non-color diff labels stay wired.
`npm run gates` includes this gate.

The behavioral CSS lives in `tools/factory-operator/public/accessibility.css` so
the anti-flash path and the accessible modes are same-origin assets, independent
from the Tailwind app bundle.
