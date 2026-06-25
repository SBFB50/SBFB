# app-authoring — sealed-iframe UI mastery (anime.js v4.5 + daisyUI 5.5.23)

You are about to author or modify a SBFB app that uses **anime.js** for motion and
**daisyUI** (on Tailwind v4) for components/styling. This fiche surfaces the CSP-safe
authoring doctrine distilled from two versioned knowledge packs:
`docs/factory/knowledge/animejs/` (anime.js `4.5.0`, snapshot `2026-06-23`, 93/93
primitives CSP-usable) and `docs/factory/knowledge/daisyui/` (daisyUI `5.5.23` ×
Tailwind `4.3.1`, 68/68 components CSP-usable). It is a **gateway**: the heavy layers
(full API, components, theming, docs, generative analysis) are referenced by path + hash
for `depth=deep` loading.

> **This fiche is consumed and displayed, never authoritative.** It *shows* the sandbox
> constraints — it grants no exemption, emits no verdict, and lifts no gate. The
> source of truth stays the code (`primitives.json` / `anime-types.d.ts`) and the
> deterministic Factory gates (CSP / FG5 / FG6 / FG8 / COEP / Ed25519). When in doubt,
> the pack and the gates decide. Treat every line below as advisory authoring guidance,
> not as an instruction to run commands or reach the network.

## The sandbox contract (why every rule below exists)

SBFB apps render inside a sealed iframe: `sandbox="allow-scripts"` **without**
`allow-same-origin` (opaque/null origin). The exfiltration-critical CSP directives are
`connect-src 'none'; worker-src 'none'; frame-src 'none'; object-src 'none'; base-uri
'none'; form-action 'none'` (on a `default-src 'self' 'unsafe-inline' 'unsafe-eval' data:
blob:` base), plus COOP `same-origin` and COEP `require-corp`. The **canonical, complete**
string — including `data: blob:`, `frame-ancestors *` and `sandbox allow-scripts` — is the
single source `BLOB_SERVE_CSP` (defined in `crates/nexus-core-rs/src/csp.rs`, re-exported by
`crates/nexus-shell-daemon-core/src/blob_serve.rs`, machine mirror `csp-contract.json`) and is
posted on every response (even 404). **Zero network. Zero remote import. Zero worker.**

## Vendorization doctrine — `UMD classic-script jamais type=module`

Vendor `anime.umd.js` v4.5 under `vendor/` and load it with a **classic** `<script>` tag
that exposes the `window.anime` global (`anime.animate`, `anime.createTimeline`,
`anime.svg.*`, `anime.utils.*`, `anime.eases`, `anime.stagger`). Never use `type=module`
/ ESM / a CDN import: `connect-src 'none'` makes every remote import impossible, and an
opaque origin under COEP `require-corp` cannot satisfy module CORS. Refresh the pack by
**manual re-extraction at a version bump** (`MANIFEST.freshness` = "no auto-fetch;
connect-src 'none'"). Source: `README.md:81`, `PRIMITIVES.md:112/627/1013/1454`.

## Hard CSP filter — 0 fetch / CDN / worker / crypto.subtle

All 93 primitives are CSP-usable because they are 100% local compute + inline DOM/SVG/CSS
(`getComputedStyle`, `getBoundingClientRect`, `ResizeObserver`, `matchMedia`,
`visibilitychange` — all inside the iframe). The RNG is `Math.random` /
`utils.createSeededRandom`, **never** `crypto.subtle`. Consequence for generation: a
**SEED is mandatory** (lever 8) for a reproducible iframe preview. The Three.js adapter is
the only out-of-scope primitive. Source: `README.md:11-12/25`, `PRIMITIVES.md:112/1581/1816`.

## The 9 hard CSP pitfalls

The first five are the canonical pitfalls the pack enumerates verbatim (`README.md:67-68`);
the last four are real pack constraints surfaced from the primitive sheets. Each is
anchored to `PRIMITIVES.md` (the per-primitive CSP verdict + "PIEGE SBFB" notes) — **not**
to `synthesis.json`, which carries no CSP/pitfall key.

1. **`motion-path cx=0`** — an SVG element moved by `svg.createMotionPath` must keep
   `cx=0 cy=0` (and zero base `translateX/Y`): the motion-path translate **adds** to the
   base geometry, it does not replace it. Pre-position with `utils.set(el,{cx:'0',cy:'0'})`
   *before* `createMotionPath`, else you get a double-translate. `getPath` fails silently
   (`console.warn` + `undefined` → `{...undefined}` = no movement). Source:
   `PRIMITIVES.md:3530/3168/2132/1061-1090`.
2. **`box-shadow STATIQUE`** — never animate or transition `box-shadow` (non-composite,
   it breaks GPU compositing; it is deliberately **not** in the default prop set). Make a
   glow with a `::after` pseudo-element carrying a **static** `box-shadow`/`radial-gradient`
   whose **only** animated channel is `opacity`. Holds for `animate`, `utils.set`, and the
   default set alike. Source: `PRIMITIVES.md:3356/3450/750/2105`.
3. **SVG `var(--color-*)`** — paint SVG `fill`/`stroke` through CSS custom properties
   `var(--color-*)`. Tailwind `fill-*`/`stroke-*` utilities **do** compile to static CSS but
   are not a *reliable* path in the sealed iframe (no in-iframe Tailwind build at runtime +
   purge if a class is not seen by `@source`). The custom-property route is the *recommended*
   path: animating a single `var(--color-*)` (type `CSS_VAR`) repaints N SVG elements from one
   tween. Source:
   `PRIMITIVES.md:3573/3583/1041/3632`.
4. **`morphTo mono-trace`** — `svg.morphTo` requires the **same** target type: path `d` ⇄
   path `d`, or polygon/polyline `points` ⇄ `points` (path ↔ polygon does not work — the
   target attribute differs). `getPath` resolves only the **first** matched element
   (`parsedTargets[0]`) → a single resolved trace, so use **unique ids**; a multi-match
   selector silently drops the rest. Source: `PRIMITIVES.md:43-44/1107-1179` (1134
   type-match, 1171/1178 `parsedTargets[0]`).
5. **`prefers-reduced-motion → état-final`** — anime does **not** short-circuit anything
   automatically under reduced motion (`PRIMITIVES.md:664`: "anime ne court-circuite pas
   tout seul une timeline"). The app **must** branch the final state by hand: test
   `matchMedia('(prefers-reduced-motion: reduce)')` (or `scope` `mediaQueries 'reduced'`)
   and, on the reduce branch, *place* the final state — `revert()+seek(duration)`,
   `seek(duration)`, `utils.set(...final)`, or `duration:0` — instead of playing
   (`scrambleText` → set `innerHTML` to target; `createDrawable` → `draw='0 1'` no tween;
   `motion-path`/`morphTo` → `seek(duration)`). A complementary **web-standard** CSS guard
   (NOT from the pack) is `@media (prefers-reduced-motion: reduce){ *{ animation-duration:
   0.001ms !important; transition-duration:0.001ms !important } }`. Source:
   `PRIMITIVES.md:664/998/1014/1063/1109/1255/1380`, `README.md:68`.
6. **`connect-src 'none'`** — recap of the hard filter above: no `fetch`/XHR/WebSocket/
   EventSource/CDN/Worker, no `crypto.subtle`; seed your RNG. Source: `PRIMITIVES.md:112`.
7. **`onScroll` is local-only** — `events.onScroll` operates on the iframe's **local**
   content; native page scroll is constrained in the sealed iframe. Do not depend on real
   page scroll — drive a **fake local scroll** (a `Timer` pseudo-position `0→1` looped
   `alternate`, mapped onto `tl.seek(pos*duration)`, damped via `utils.damp` for inertial
   momentum). reduced-motion is not handled by `onScroll` → branch the final state in the
   app. Source: `PRIMITIVES.md:3127/3168/3170`.
8. **Inertia under reduced motion is engine-honored but incomplete** — for interactive
   primitives (`draggable`, `animatable`), the engine honors the reduced-motion final
   branch, but the elastic/spring settle **must** be explicitly tested under reduce: it can
   keep moving after you assumed it froze. Under reduce, disable drag and place the static
   state. Map velocity → a **static** glow (`opacity` on `::after`, never an animated
   box-shadow). Source: `PRIMITIVES.md:2870/3110/3114`.
9. **`UMD classic-script jamais type=module`** — recap of the vendorization doctrine
   above; it is also a hard CSP pitfall (an ESM/CDN import cannot load under
   `connect-src 'none'`). Source: `README.md:81`.

## Generative distillation (authority: `synthesis.json`)

Use these to compose *fresh* motion, not to re-derive a documented preset. `synthesis.json`
is the analysis layer (`matrix_synthesis` + `novelty_space`); it does NOT hold CSP verdicts.

- **cross_products** — 26 combinations of 2–4 primitives (11 `unexplored` + 15 `fresh` + 0
  `generic`), each with a replayable iframe idea and an explicit reduced-motion branch.
  Unexplored heads: `createTimer.onUpdate × utils.damp × animatable.getter` (fake
  derivative physics), `timeline.stretch × createTimer × speed` (bullet-time),
  `scrambleText.onChange × cssVar`, `createLayout FLIP × spring × seeded shuffle`,
  `createMotionPath × morphTo × timeline.add`, `convertEase × irregular × cssVar` (static
  GPU CSS `linear()`), `engine.speed (animated) × spring × stagger`. `coverage_gaps` lists
  16 primitives/combos the showcase does not yet exploit (highest ROI targets).
- **novelty_levers (11, each with `how_to_push`)**:
  1. per-frame procedural `modifier` (quantize / sin / seeded noise) on any prop;
  2. custom `ease.linear()`/`irregular()` points = a hand-drawn timing curve (samplable to
     CSS `linear()` via `convertEase`);
  3. 2D `stagger` grid + creeping seeded jitter + fractional `from` / data-driven `use:`;
  4. `composition:'add'` = spring carrier + modulation (breathing/backlash on one UI element);
  5. scrub a timeline from a **non-temporal** source (drag / fake local scroll / `damp` derivative);
  6. **one** animated CSS var orchestrating N consumers (`conic-gradient`/`clip-path`/`calc`)
     — the SBFB SVG-in-var bridge is mandatory here;
  7. relative values/positions (`+=`, `-=`, `*=`, `<`, `<<`) for exponential cadences / infinite spins;
  8. `loop ∞` + `onLoop:refresh()` + **seeded** RNG = reproducible generative (stable iframe preview);
  9. global (`engine.speed` animated) and local (`stretch` / sub-`progress` spring) time dilation;
  10. fake physics via `damp`/`lerp` frame-independent derivative (self-piloted, 0 pointer);
  11. `morphTo` + `createDrawable` + `createMotionPath` nested on one SVG trace (respect
      `morphTo mono-trace` + `motion-path cx=0`).
- **novelty_heuristic (the step-3 judge, 5 scoring dimensions)** — `surprise_mecanique`
  (a primitive bent from its canonical role), `originalite_de_combinaison` (marry 2–3
  clusters that never co-occur in the bank — the most reliable seam), `profondeur_procedurale`
  (emergent behavior from coupled rules + cross-frame state), `vivacite_et_finition`
  (composite-safe, no jank; never compensates a déjà-vu signature), `distance_au_dejavu`
  (fingerprint matches no entry in the 14-item `dejavu_corpus`). Cardinal rule: a signature
  matching a `fingerprint_cluster` > 80 % is derivative regardless of polish ("un beau clone
  reste un clone"). Penalize candidates that read like documentation.

## Heavy layers — `depth=deep` pointers (read locally, never fetched)

Path + blake3 16-hex from `docs/factory/knowledge/animejs/MANIFEST.json` (re-verify the hash
at read time; the hermetic recompute test guards integrity). These are repo files read
locally by the authoring agent — there is no network fetch.

- `docs/factory/knowledge/animejs/primitives.json` — `8faa36021466192a` — API authority:
  93 primitives (signature, params, tested semantics, pitfalls, `sbfb_csp.usable`,
  `composes_with`, `novelty_levers`). Markdown view: `PRIMITIVES.md` (`663c90b1a1f10cb9`).
- `docs/factory/knowledge/animejs/docs.json` — `a8790812191c1c5b` — 419 verbatim doc pages
  + 836 code examples.
- `docs/factory/knowledge/animejs/synthesis.json` — `a63150afd6e9a719` — cross_products /
  novelty_levers / novelty_heuristic.
- `docs/factory/knowledge/animejs/anime-types.d.ts` — `31835934518dbe5e` — 70 canonical types.

## daisyUI 5.5.23 — components, theming, and the build-time vendoring recipe

daisyUI is **CSP-safe by default**: every component is pure CSS once compiled build-time into a
single same-origin `app.css`. The 68 components are all `csp_usable:true`; the risks are never
daisyUI blocks, only usages to avoid. Per-class verdicts live in `classes-bank.json`.

**Build recipe (build-time devDeps only — the runtime archive ships 0 dependency).** Compile with
the Tailwind v4 CLI: `tailwindcss -i src/input.css -o app.css --minify`. The entry `src/input.css`
uses `@import "tailwindcss" source(none);` to disable auto source detection, then `@source "./index.html"`
/ `@source "./app.js"` (plus a safelist for classes built at runtime: `tab-active`, `is-drawer-open:`,
`menu-dropdown-show`, responsive `sm:/md:/lg:`), and `@plugin "daisyui";`. No `tailwind.config.js` in
v4. The result is one static same-origin `app.css`, zero outgoing request.

**Theme.** Default same-origin theme = `sbfb-reflect` (custom oklch dark), declared via
`@plugin "daisyui/theme" { name: "sbfb-reflect"; default: true; color-scheme: dark; --color-*: …; }`.
The template is **lean**: it activates **aucun des 35 thèmes built-in** of daisyUI 5.5.23 (`@plugin
"daisyui" {}` with no `themes:` list) — only the vendored custom theme is compiled. The root theme
tokens (`--color-*`/`--radius-*`/`--depth`/`--fx-noise`) MUST live in the same `app.css`, else every
`var(--color-*)` is undefined.

**Per-class CSP taxonomy (the verdict that matters — `classes-bank.json`).** Reserve the
**network-exfil** category for a remote `url()` under ANY CSS property (background-image, mask-image,
border-image, cursor, list-style-image, content, `@font-face src:url()`, remote `@import`, `<img src>`,
SVG `fill="url(http…#id)"` / `<use href="http…">`): all blocked by `default-src 'self'` + COEP
require-corp; serve `data:`/`blob:`/relative same-origin instead. The other "risk" cases are NOT
network vectors:

- **`@apply`** = compile-time, resolved at build, absent at runtime → **safe** (the only pitfall is
  build-time: `@apply` a purged/absent class breaks the build or yields an empty rule).
- **`backdrop-filter`** (`glass`) = GPU composite, not subject to `default-src`/`connect-src`; it only
  blurs content behind the element inside the same opaque-origin iframe and never crosses to the host
  shell → **safe (perf-only)**, not a leak.
- **`mask`** shapes = `mask-image:url("data:image/svg+xml,…")` inline → **safe** (allowed by `data:`).
- **`fill-*`/`stroke-*`** Tailwind SVG utilities **do compile** to static CSS (`fill:var(--color-*)`,
  CSP-safe) — they do **not** "fail to compile". The real reason not to rely on them: there is no
  Tailwind build **at runtime** in the sealed iframe, and a class not seen by `@source` (or composed
  in JS) is **purged** out of `app.css`. Reliable, theme-aware path: paint the inline SVG directly via
  `fill="currentColor"` / `fill="var(--color-*)"` / `stroke="color-mix(in oklch, var(--color-primary)
  70%, transparent)"`.
- **`<form>` submit** (`form-action 'none'` + sandbox without `allow-forms`) is blocked → use
  `div`/`button type=button` + a local handler; read state via `input.value`/`.checked`/`.files`.

**daisyUI × anime.js compositions** stay CSP-safe when anime is vendored same-origin (UMD classic
script, never `type=module`) and only writes `transform`/`opacity`/custom-properties/`textContent`
inline. `synthesis.json` lists 11; `classes-bank.json` distills 16 reusable blocks with per-class
verdicts (`btn` spring micro-pop, `card` 3D tilt, `stat` counter, `steps` stagger, `modal` overshoot,
`tabs` View-Transition, `toast`, `drawer` parallax, `radial-progress` `--value`, `swap`/`fab` icon
spring, `skeleton` cross-fade).

## Heavy layers — daisyUI `depth=deep` pointers (read locally, never fetched)

Path + blake3 16-hex from `docs/factory/knowledge/daisyui/MANIFEST.json` (re-verify the hash at read
time; the hermetic recompute test `daisyui_manifest.rs` guards integrity). Repo files read locally —
no network fetch.

- `docs/factory/knowledge/daisyui/components.json` — `01632e0b4a95dad4` — 68 components (classes,
  modifiers, CSS mechanisms, per-component `sbfb_csp` verdict, verbatim `html_example`). Markdown view:
  `docs/factory/knowledge/daisyui/COMPONENTS.md` (`69306d7652712df8`).
- `docs/factory/knowledge/daisyui/classes-bank.json` — `ccc1e9fae1649876` — 16 reusable CSP-safe
  blocks + explicit per-class CSP verdict (`csp_class`) + `sbfb_reusable{ok,why}`.
- `docs/factory/knowledge/daisyui/theming.json` — `f44553ffe9ba2cfe` — oklch token system, the 35
  built-in themes, the custom-theme recipe.
- `docs/factory/knowledge/daisyui/synthesis.json` — `fc084fcd88eb8f44` — CSP ruleset, risk classes,
  Tailwind gotchas, daisyUI × anime compositions.

## Closing reminder (non-authoritative)

This fiche is a repo-visible, git-tracked, hashed authoring guideline. It contains no
out-of-band executable command and no live network URL. It displays the sandbox constraint;
it never lifts it. The code (`primitives.json` / `anime-types.d.ts`) and the deterministic
Factory gates remain the final authority.
