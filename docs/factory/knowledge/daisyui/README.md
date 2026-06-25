# daisyUI Knowledge Pack — daisyUI 5.5.23 × Tailwind 4.3.1

Base de connaissances **dev-only**, miroir du pack anime.js (`../animejs/`). Donne à un LLM/agent la
maîtrise de daisyUI pour fabriquer des apps SBFB **stylées et conformes à la CSP du bac à sable**.
Capacité du process Factory (prompt-kind `app-authoring`), promue ici sous
`docs/factory/knowledge/daisyui/` (Sprint 79 Phase F). Étude de conception :
`examples/daisyui-animejs-showcase/knowledge/factory-integration-design.md`.

> **Source de vérité = le code.** Composants/classes extraits du **CSS source local**
> (`node_modules/daisyui/components/*.css`, vérité terrain), theming depuis `theme/` + la
> vitrine `src/input.css`, prose/exemples depuis le **`llms.txt` officiel** (la doc « skill »
> que daisyUI publie pour les LLM, sauvegardée verbatim). Versions **figées** (résolues du lock,
> pas les carets).

## Les couches

| Artefact | Contenu | Source |
|---|---|---|
| `components.json` / `COMPONENTS.md` | **68 entrées** : classe, sous-parties, modifiers, mécanismes CSS, **verdict CSP + risques**, exemple HTML verbatim | `components/*.css` + `llms.txt` |
| `theming.json` | système **oklch** : 29 vars, `data-theme`, syntaxe `@plugin "daisyui/theme"`, color-mix, **35 thèmes** built-in, recette thème custom | `theme/` + `themes.css` + `src/input.css` |
| `synthesis.json` | ruleset CSP daisyUI + classes à risque + pièges Tailwind + **compositions daisyUI × anime.js** | synthèse Opus |
| `docs-llms.txt` | la doc officielle daisyUI « skill » (1967 lignes), verbatim | `daisyui.com/llms.txt` |
| `MANIFEST.json` | versions figées + date + hashes BLAKE-court par couche | — |

**Versions figées** : daisyUI **5.5.23**, Tailwind **4.3.1** (cli + node + oxide), anime.js **4.5.0**. Thème défaut **sbfb-reflect** (oklch dark custom).

## Constat CSP (le cœur de la valeur)

daisyUI est **CSP-safe par défaut** : tout est du CSS pur une fois compilé build-time en un
`app.css` vendoré same-origin. Les 68 composants sont utilisables dans l'iframe scellée — **les
risques ne sont pas des blocages de daisyUI mais des usages à éviter** :

- **Images** (`<img src>`, `background-image:url()`, `mask`, `hero`, `avatar`, `chat-image`…) : une URL http distante est **bloquée** (`connect-src 'none'` + COEP + origine opaque ; les exemples daisyUI pointent vers `img.daisyui.com`). → servir en `data:` / `blob:` / fichier relatif same-origin.
- **`<form>` submit** (`input`, `select`, `validator`, `<form method=dialog>`…) : bloqué (`form-action 'none'` + sandbox). → idiome `div`/`button type=button` + handler JS local, lire `input.value`/`.checked`.
- **Composants pilotés par JS** (`calendar` Cally/Pikaday, `carousel` autoplay, `countdown`, `radial-progress`, `toast`, `modal.showModal`) : habillage CSS safe, mais **comportement à coder en JS local** (lib **vendorée** dans l'archive, jamais CDN).
- **Icônes SVG** (`dock`, `fab`, `stat`, `timeline`, `steps`, `mask`) : les utilitaires Tailwind `fill-*`/`stroke-*` **se compilent** pourtant en CSS statique (`fill:var(--color-*)`, CSP-safe) — le vrai risque n'est PAS la compilation mais (a) l'absence de build Tailwind **au runtime** dans l'iframe scellée et (b) la **purge** si la classe n'est pas vue par `@source` (ex. composée en JS). → chemin **fiable** : peindre le `<svg>` directement via `fill="currentColor"` / `var(--color-*)` / `color-mix(in oklch …)` (robuste à la purge + theme-aware). Tout `fill="url(http…)"` / `<use href="http…">` reste **bloqué** (réseau). **C'est le piège le plus récurrent.**
- **`backdrop-filter`** (`glass`, backdrops de `modal`/`drawer`) : autorisé (composite GPU), juste un coût perf.
- **`--fx-noise`/`--btn-noise`** : par défaut un SVG `feTurbulence` en `data:` URI inline (`base/svg.css`) = OK ; ne le repointe jamais vers un http distant.

**Pièges Tailwind v4** : purge `source(none)` + `@source` explicites (sinon classes dynamiques `tab-active`/`sm:`/`md:` absentes du build) ; jamais `@apply` vers une classe purgée ; plugin `typography` chargé au build ; thème par défaut explicite dans le même `app.css` (sinon toutes les `var(--color-*)` indéfinies) ; aucun `@import`/police distant.

## Compositions daisyUI × anime.js (CSP-safe)

11 croisements concrets dans `synthesis.json`, ex. : `btn` + spring micro-pop · `card` + `createAnimatable` tilt 3D · `stat` + compteur `animate` · `steps` + `stagger` reveal · `modal` (`@starting-style`) + timeline overshoot · `tabs` + View Transition cross-fade · `toast` + timeline enter/dismiss · `radial-progress` + `animate('--value')` · `swap`/`fab` + spring d'icône.

## Couplage

Ce pack + le pack anime.js (`../animejs/`) forment la matière de la capacité **`app-authoring`** du
process Factory : un agent qui fabrique une app SBFB reçoit la maîtrise composants (daisyUI) +
mouvement (anime.js) + le contrat CSP, et produit du CSP-safe par construction. Le contrat CSP est
la source unique `BLOB_SERVE_CSP` (`crates/nexus-core-rs/src/csp.rs`, miroir `csp-contract.json`).
Études de conception : `examples/daisyui-animejs-showcase/knowledge/factory-integration-design.md` +
`factory-integration-hardened.md`.
