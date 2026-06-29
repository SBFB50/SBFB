# Factory Operator Accessibility Design System

This is the shipped contract for the Factory Operator shell. It is organized by
functional need, not by medical diagnosis. A person can select several needs at
the same time; the resolver combines them into one deterministic UI state.

## Research Base

Normative and product sources used for the current contract:

- WCAG 2.2: perceivable, operable, understandable, robust criteria.
  <https://www.w3.org/TR/WCAG22/>
- WAI WCAG quick reference: implementation mapping by success criterion.
  <https://www.w3.org/WAI/WCAG22/quickref/>
- W3C COGA usable guidance: cognitive and learning accessibility patterns.
  <https://www.w3.org/TR/coga-usable/>
- WAI media accessibility: captions, transcripts, audio description.
  <https://www.w3.org/WAI/media/av/>
- MDN media features: `prefers-reduced-motion`, `prefers-contrast`,
  `forced-colors`.
  <https://developer.mozilla.org/en-US/docs/Web/CSS/@media>

Scope note: no UI can claim to cover "all disabilities" as diagnoses. The
Factory contract covers the major interaction barriers that matter to an
operator interface: visual perception, color perception, assistive tech,
vestibular/photosensitive safety, motor access, reading, cognition, attention,
auditory media access, speech independence, and sensory load.

## Runtime Contract

`/preboot.js` runs before the React bundle and applies persisted preferences to
`<html>` with no inline script. The lazy React panel then owns the same
attributes for in-session changes:

- `data-theme="dark|light|calm|paper|forced"`
- `data-contrast="standard|high"`
- `data-color-vision="standard|safe|monochrome"`
- `data-pointer="standard|large"`
- `data-text-spacing="standard|loose"`
- `data-font="standard|legible"`
- `data-motion="standard|reduced"`
- `data-transparency="standard|reduced"`
- `data-density="standard|focus"`
- `data-reading="standard|assist"`
- `data-focus="standard|strong"`
- `data-scale="100|112|125|150"`
- `data-shortcuts="off|on"`
- `data-assistive-tech="standard|screen-reader"`
- `data-captions="off|on"`
- `data-needs="<space-separated need ids>"`

The storage key is `factory-operator.accessibility.v2`. The reader still accepts
`factory-operator.accessibility.v1` for migration. Unknown or malformed values
are sanitized before reaching the DOM.

## Page Contract

Every focal page and inspector page must render through `AdaptiveSurface`. This
puts a stable `data-adaptive-surface="<kind>"` marker on the page or sub-page so
the accumulated preferences affect the actual UX of that page, not only global
tokens.

Current surface kinds:

- `steer`: composer, MUR, and observable agent output.
- `verify`: diff/gates/terminal focal verification.
- `surface-host`: common frame for secondary inspectors.
- `procede`: sprint/phase procedure tree and embedded commit diffs.
- `sessions`: action journal, resumable sessions, chat log, terminal replays.
- `knowledge`: consultative knowledge and context-pack area.
- `documents`: live project document map, pinned LLM file usage, read/use/write/scan roles.
- `context-pack`: sealed references, hashes, and drift markers.
- `diff`: working-tree and historical commit diff viewer.
- `terminal`: PTY start and live terminal surface.

The CSS applies accumulated modes at this surface layer:

- reading assist un-truncates text and allows wrapping inside each surface;
- focus density hides only marked secondary UI, never primary controls;
- 150 percent scale makes surface headers/toolbars wrap and scroll safely;
- large pointer expands controls inside each surface;
- monochrome mode adds non-color state decoration to status text;
- reduced transparency removes filters/shadows inside each surface.

The gate checks that each shipped page-level component imports `AdaptiveSurface`.
Adding a new page without the marker is a failing accessibility-system gate.

## Need Selection

The panel exposes these stackable needs:

- `lowVision`: high contrast, forced theme in auto mode, larger targets, strong
  focus, and at least 125 percent text tokens.
- `blindAssistive`: screen-reader-oriented DOM state, focused density, strong
  focus, and single-letter shortcuts off.
- `colorVision`: color-safe state palette and strong focus.
- `noColor`: monochrome state palette, high contrast, and strong focus.
- `vestibular`: reduced motion and reduced transparency.
- `photosensitive`: reduced motion and reduced transparency.
- `motor`: larger targets, strong focus, and single-letter shortcuts off.
- `cognitive`: focused density, assisted reading, loose text spacing, and
  single-letter shortcuts off.
- `dyslexia`: legible font, assisted reading, loose spacing, and at least
  112 percent text tokens.
- `attention`: focused density, reduced motion, and reduced transparency.
- `auditory`: captions/media text track preference on.
- `speech`: no speech-only assumption; single-letter shortcuts off.
- `sensory`: focused density, reduced motion, and reduced transparency.

## Stacking Rules

The resolver applies the strictest safe value per axis:

- Theme auto priority: forced > calm > paper > dark. Manual theme selection can
  override auto.
- Contrast: selected visual/color needs or system high contrast force high.
- Scale: the largest selected minimum wins.
- Motion and transparency: any vestibular, photosensitive, attention, or
  sensory need forces the reduced mode.
- Shortcuts: any screen-reader, motor, cognitive, or speech need forces
  single-letter shortcuts off.
- Color: no-color perception wins over color-safe mode.

This avoids brittle "profiles" where selecting a second handicap erases the
first. A user with low vision plus dyslexia plus photosensitivity resolves to
forced theme, high contrast, 125 percent type, legible font, assisted reading,
reduced motion, and reduced transparency.

## Coverage

- Low vision: readable type floor, user scale up to 150 percent, light/calm/paper
  and forced themes, high contrast mode, visible focus, non-Latin system
  fallbacks.
- Color blindness and no-color perception: color-safe and monochrome state
  palettes, plus text labels for diff and gate states.
- Screen readers: skip link, landmarks, status live regions, expanded states,
  and textual state labels for diff lines and gate statuses.
- Motor accessibility: large target mode, responsive rail reflow, and
  single-letter `s/v` shortcuts disabled until safe.
- Cognitive and attention accessibility: focused density, stable focal layout,
  assisted reading, persistent preferences, and no hidden auto-switching.
- Dyslexia and reading disabilities: legible font option, loose spacing, reading
  mode, and larger type tokens.
- Vestibular/photosensitive/sensory: reduced-motion follows OS by default,
  selected needs force reduced motion, transparency reduction removes visual
  effects.
- Auditory and speech: media/caption preference is expressed in the runtime
  contract; no speech-only command path is introduced.

## Gate

`npm run gate:accessibility-system` checks that the preboot, public CSS modes,
stackable need resolver, page-level `AdaptiveSurface` markers, shortcut guard,
reflow, live status, and non-color diff labels stay wired. `npm run gates`
includes this gate.

The behavioral CSS lives in `tools/factory-operator/public/accessibility.css` so
the anti-flash path and the accessible modes are same-origin assets, independent
from the Tailwind app bundle.
