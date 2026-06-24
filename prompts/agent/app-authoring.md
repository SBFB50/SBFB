# app-authoring — sealed-iframe UI mastery (anime.js v4.5)

You are about to author or modify a SBFB app that uses **anime.js** for motion. This
fiche surfaces the CSP-safe authoring doctrine distilled from the versioned knowledge
pack at `docs/factory/knowledge/animejs/` (anime.js `4.5.0`, snapshot `2026-06-23`,
93/93 primitives CSP-usable). It is a **gateway**: the heavy layers (full API,
docs, generative analysis) are referenced by path + hash for `depth=deep` loading.

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
string — including `data: blob:`, `frame-ancestors *` and `sandbox allow-scripts` — lives
in `BLOB_SERVE_CSP` (`crates/nexus-shell-daemon-core/src/blob_serve.rs`) and is posted on
every response (even 404). **Zero network. Zero remote import. Zero worker.**

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
   `var(--color-*)`. Tailwind `fill-*`/`stroke-*` utilities do **not** compile inside the
   sealed iframe (no in-iframe Tailwind build). This is the *recommended* path: animating a
   single `var(--color-*)` (type `CSS_VAR`) repaints N SVG elements from one tween. Source:
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

## Closing reminder (non-authoritative)

This fiche is a repo-visible, git-tracked, hashed authoring guideline. It contains no
out-of-band executable command and no live network URL. It displays the sandbox constraint;
it never lifts it. The code (`primitives.json` / `anime-types.d.ts`) and the deterministic
Factory gates remain the final authority.
