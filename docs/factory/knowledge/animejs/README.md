# Anime Knowledge Pack — anime.js v4.5

Base de connaissances **dev-only** (jamais publiée dans l'archive SBFB, comme `scripts/` et `src/`).
Fondation de l'étape 2 : un moteur de génération d'animations *vraiment* innovantes pour la vitrine
`daisyui-animejs-showcase`.

> **Source de vérité = le code, pas la prose.** Tout est extrait de `juliangarnier/anime` v4.5.0
> (clone du 2026-06-23) : signatures depuis `src/`, sémantique exacte depuis `tests/suites/`, patterns
> depuis `examples/` + `tests/playground/`, types depuis `dist/modules/*.d.ts`, et la doc officielle
> complète scrapée depuis `animejs.com/documentation`. Chaque primitive est annotée de son verdict
> **CSP SBFB** (utilisable dans l'iframe scellée : 0 réseau, scripts classiques, box-shadow non animée,
> SVG peint en `var(--color-*)`).

## Les 5 couches de données

| Artefact | Contenu | Provenance |
|---|---|---|
| `primitives.json` / `PRIMITIVES.md` | **93 primitives** : signature, params (type+défaut), sémantique testée, pièges, verdict CSP, `composes_with`, `novelty_levers` | `src/` (70 fichiers) ✕ `tests/suites/` (33 suites) |
| `examples-bank.json` / `EXAMPLES.md` | **52 démos** distillées : « le truc » + primitives + tags + `novelty_fingerprint` + snippet verbatim | `examples/` (36) + `tests/playground/` (45) |
| `docs.json` / `DOCS.md` | **419 pages** de doc officielle, verbatim : prose + signatures + **836 exemples de code** | `animejs.com/documentation` (scrape complet, 0 échec) |
| `anime-types.d.ts` | **70 définitions TypeScript** canoniques (surface d'API exacte) | `dist/modules/**/*.d.ts` |
| `synthesis.json` | la couche d'analyse : cross-produits, leviers, déjà-vu, heuristique de jugement | synthèse Opus 4.8 sur les 4 couches ci-dessus |

Couverture brute : **93 primitives, 52 démos, 419 pages doc, 836 exemples de code, 70 types**. Les 93
primitives sont **toutes** marquées CSP-usable (l'adaptateur Three.js mis à part, qui est noté hors-périmètre).

## `synthesis.json` — la couche actionnable

C'est ici que vit la valeur pour la génération. Contenu :

- **`matrix_synthesis.cross_products`** — 26 combinaisons de 2-4 primitives, classées
  **11 `unexplored` + 15 `fresh` + 0 `generic`**, chacune avec une *idée d'animation concrète
  rejouable dans l'iframe SBFB* (ex. « 8 anneaux SVG dont l'étirement dérive de l'accélération
  calculée par cascade de `damp` », « jauge dont la progression dépasse puis recule via keyframes %
  + `playbackEase:irregular()` consommée par un `conic-gradient` »).
- **`matrix_synthesis.novelty_levers`** — 11 leviers d'unicité **non-générique**, chacun avec un
  `how_to_push` (comment dépasser le preset documenté). C'est la boussole créative :
  1. `modifier` procédural par-frame (quantize/sin/bruit seedé) sur *n'importe quelle* prop
  2. points custom de `linear()` / `irregular()` comme courbe de timing dessinée main
  3. stagger grid 2D + jitter rampant + `from` fractionnaire / `use:` data-driven
  4. `composition:'add'` = porteuse + modulation (respiration/backlash sur un élément UI unique)
  5. scrub d'une timeline par une source **non-temporelle** (drag / faux-scroll local / dérivée)
  6. **une seule CSS var animée** orchestrant N consommateurs (`conic`/`clip-path`/`calc`)
  7. valeurs/positions relatives (`+=`, `*=`, `<`, `<<`) pour des cadences exponentielles / spins infinis
  8. `loop` ∞ + `onLoop:refresh()` + RNG **seedé** = génératif reproductible (preview iframe stable)
  9. dilatation temporelle globale (`engine.speed` animé) et locale (`stretch` / sous-`progress` en spring)
  10. physique *fake* par `damp`/`lerp` dérivatif frame-indépendant (auto-piloté, sans pointeur)
  11. `morphTo` + `createDrawable` + `createMotionPath` **imbriqués sur un même tracé SVG**
- **`matrix_synthesis.generic_vs_novel`** — 9 clichés à éviter ✕ 10 directions neuves.
- **`matrix_synthesis.coverage_gaps`** — **16 primitives/combos que la vitrine actuelle n'exploite
  PAS** (cibles à plus haut ROI). En tête : `createLayout` FLIP sur données simulées, `engine.speed`
  animé, `timeline.stretch` continu, `composition:'add'` sur éléments UI, `convertEase`→`linear()` CSS,
  `irregular()` par-cible, CSS var comme bus d'orchestration, `damp` en cascade dérivative.
- **`novelty_space.fingerprint_clusters`** (22) — cartographie de ce qui existe déjà (par signature partagée).
- **`novelty_space.dejavu_corpus`** (14) — les patterns qui rendent un candidat **dérivatif** (avec où on les a vus).
- **`novelty_space.novelty_heuristic`** — le **juge** opérationnel de l'étape 3 : 5 dimensions de scoring
  (`surprise_mecanique`, `originalite_de_combinaison`, `profondeur_procedurale`, `vivacite_et_finition`,
  `distance_au_dejavu`) + 14 red-flags.

## Comment l'étape 2 (Idea Engine) consomme ce Pack

Contrat de génération proposé :

1. **Génère** en échantillonnant `primitives.json` (briques) biaisé par `cross_products` (`unexplored`
   d'abord) et `novelty_levers` (`how_to_push`). SEED obligatoire (cf. levier 8 — reproductibilité iframe).
2. **Filtre dur CSP** : ne retenir que des candidats `sbfb_csp.usable`, respectant les pièges
   (motion-path `cx=0`, `morphTo` mono-tracé, glow `::after` opacity-only, SVG en `var()`,
   `prefers-reduced-motion` → branche état-final).
3. **Pression de nouveauté** : rejeter / muter tout candidat dont la `novelty_fingerprint` matche
   `examples-bank.json` ou `dejavu_corpus`. Scorer via `novelty_heuristic` (les 5 dimensions).
4. **Render-in-the-loop** (étape 3) : build → flipbook (`scripts/motion-check.mjs`) → critique vision
   selon les mêmes dimensions → mutation. Tournoi génétique borné par SEED.
5. **Curation** (étape 4) : présenter les flipbooks des survivants — décision humaine finale.

Les harnesses dev existants (`scripts/render-check.mjs`, `motion-check.mjs`, `motion-audit.mjs`,
`path-align.mjs`) sont les vérificateurs mécaniques de la boucle (0 erreur console, tokens ON path,
0 box-shadow animée, Lighthouse).

## Fraîcheur

Snapshot anime.js **v4.5.0** au **2026-06-23**. Le runtime de la vitrine vendore `anime.umd.js` v4.5
(`vendor/`). Avant de t'appuyer sur une primitive, le code reste l'autorité : `primitives.json` cite
le `module_path` exact, `anime-types.d.ts` donne le type résolu, `DOCS.md` la prose officielle.
