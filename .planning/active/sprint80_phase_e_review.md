# Sprint 80 — Phase E — Review (avant commit)

**Phase** : E — design-system oklch + 5 signatures de motion sens-porteuses dans `tools/factory-operator/` (front-only, **backend AUCUN**).
**Date** : 2026-06-28.
**Orchestration** : Workflow ultracode (agents Opus 4.8 1M — 6 dimensions + 4 passes adversariales + synthèse).
**Préflight** : `sprint80_phase_e_preflight.md` — verdict **PLAN-ADAPT** (3 corrections load-bearing : budget motion confiné hors hero, anti-déco ancrage transform + garde `useReducedMotion()`, altitude shift VT native).
**Nature du diff** : front React 19 (`tools/factory-operator`) — working tree NON commité sur HEAD=`ed00b4a` (Phase G). 10 fichiers trackés + ~12 fichiers neufs (lib/motion, lib/altitudeShift, lib/usePrefersReducedMotion, components/motion/*, e2e/motion.spec, tests). **0 fichier `crates/`, 0 `.rs`, 0 route, 0 wire.**

> **Verdict bref (re-review 2026-06-28 après correctif)** : **PASS**. Le P1 et le P2 de la 1re review sont **CORRIGÉS et vérifiés au bundle réel**. La cause racine (le `manualChunk vendor-motion` qui faisait fuiter React core dans `vendor-motion` ⇒ façade de ré-export hero→vendor-motion) a été supprimée. Désormais **tout le motion (core + domAnimation + composants) est dans le chunk ASYNC `VerifyScene-DSqtvLQa.js` (86 072 o)**, seul importeur = la surface async VERIFY (import dynamique du hero) ; React core est revenu dans `vendor-react`. **Le graphe eager (`index.html` → hero + `vendor-react` + `rolldown-runtime`) ne contient AUCUN marqueur motion.** Le chunk motion-bearing est désormais MESURÉ par size-limit (`verify-surface` ≤ 92 KB ⇒ 86,07). Commentaires faux corrigés. Invariant Day-0 « motion hors hero » TENU. Détail §10.
>
> **Verdict bref (1re review — historique)** : **FAIL**. Les invariants de DOCTRINE (anti-déco reduced-motion, 0-verdict-calculé-UI, CSP, oklch, rail exclu) sont tous tenus et corroborés. **MAIS la déviation centrale que cette phase devait justifier — « le provider Motion confiné à l'async VERIFY ⇒ `vendor-motion` reste ASYNC, hero = 0 poids motion-lib » — était FAUSSE dans le bundle réellement produit** : `vendor-motion` (86,65 KB, lib Motion) était `modulepreload` dans `index.html` ET importé en statique au top-level du hero `index.js`, donc fetché + parsé + exécuté au premier paint. C'était un P1 réel (invariant Day-0 « motion hors hero » violé + doc malhonnête) → FAIL per règle de sévérité. **CORRIGÉ — cf. §10.**

---

## 1. Périmètre revu

Diff non commité Phase E sur HEAD=`ed00b4a`.

- **Design-system** : `src/index.css` (keyframes settle/gravity + pseudos `::view-transition-*(focal)` + exclusion rail/orientation `view-transition-name:none` + reset `@media (prefers-reduced-motion)`), oklch déjà posé Phase B (non touché).
- **Motion lib (neuf)** : `src/lib/motion.ts` (constantes nommées + énum 5 signatures + `REDUCED_MOTION_QUERY`), `src/lib/altitudeShift.ts` (VT native + `flushSync`), `src/lib/usePrefersReducedMotion.ts` (hook matchMedia synchrone), `src/components/motion/{MotionProvider,GateFlip,Reveal,TokenCount}.tsx`.
- **Câblage** : `src/App.tsx` (`.motion-focal`, VerifyScene lazy), `src/components/OrientationBar.tsx` (TokenCount), `src/components/steer/Mur.tsx` (gravity), `src/components/verify/VerifyScene.tsx` (MotionProvider + GateFlip + Reveal), `src/state/useOperator.ts` (setMode → altitudeShift).
- **Config / gates** : `.size-limit.json` (entrée `vendor-motion` 92 KB + css 20→21), `vite.config.ts` (manualChunk `vendor-motion`), `eslint.config.js` (commentaire seul), `src/test/setup.ts` (matchMedia default).
- **Tests** : `e2e/motion.spec.ts` (T1 anti-déco signature 4) + `src/lib/{motion,altitudeShift,usePrefersReducedMotion}.test.ts` + `src/components/motion/{GateFlip,Reveal,TokenCount}.test.tsx`.

Frozen « Factory hors daemon » / « front-only » tenu : 0 backend, 0 route, 0 wire/canonical/`*_VERSION`, 0 dépendance neuve (Motion déjà au lock).

---

## 2. Résumé des 6 dimensions (après filtrage adversarial)

| # | Dimension | Verdict | Findings retenus |
|---|---|---|---|
| 1 | Correctness (hooks/listeners/edge cases) | **PASS** | 0 P0/P1/P2 ; 1 P3 (commentaire eslint sur-promet) |
| 2 | Scope cuts | **PASS** | 7 cuts respectés réellement ; déviation jugée saine PAR LE SOURCE — réfutée par le bundle (cf. §5) |
| 3 | Préflight-conformance | **PASS sauf 1** | 5 mustFix conformes EN INTENTION ; mustFix §3.1.4 « entrée size-limit pour les chunks de surface m.* » NON honoré (cf. P2) ; 2 P3 |
| 4 | Sécurité + doctrine | **PASS** | CSP intacte, anti-déco OK, 0-verdict OK, 0 surface réseau ; 1 P3 couverture |
| 5 | Tests | **PASS** | suites non-vacantes, T1 anti-déco solide ; 4 P3 couverture |
| 6 | Budget / build | **FAIL (P1)** | `vendor-motion` 86,65 KB **eager au hero** (pas async) → invariant déviation faux ; 1 P3 chunks non budgétés |

**Cœur de doctrine CORRECT et corroboré ligne à ligne.** Les 5 dimensions « source-level » (1-5) ont lu l'INTENTION du code (`VerifyScene = lazy()`, commentaires de confinement, `import type` au hero) et conclu « déviation saine ». La dimension budget/build (6), SEULE à avoir inspecté le **bundle réellement produit**, démontre que l'intention n'est pas réalisée par rolldown. C'est l'écart décisif.

---

## 3. Invariants de DOCTRINE — tenus et vérifiés

- **Anti-déco « motion = sens, jamais déco » (reduced-motion ⇒ état final instantané)** — TENU, vérifié sur chaque chemin : (1) CSS `@media (prefers-reduced-motion)` réduit settle/gravity/VT à 0.01ms (`index.css`) ; (2) `altitudeShift.ts:40-50` SAUTE `startViewTransition` (apply synchrone) sous garde `prefersReducedMotion()`/feature-detect ; (3) `GateFlip`/`Reveal`/`RevealItem` rendent du markup **plain** (zéro composant `m.*` monté ⇒ zéro WAAPI) via `usePrefersReducedMotion()` appelé inconditionnellement en tête (pas de hook conditionnel) ; (4) `usePrefersReducedMotion` lit `matchMedia` SYNCHRONEMENT dans l'initializer `useState` ⇒ 1er rendu déjà correct, pas de flash. **Adversarial claim 2 = CONFIRMED** : aucun chemin de motion n'échappe au reduced-motion ; T1 prouve `__vtCalls===0` ET `running===0` après la bascule, avec preuve positive (verify-scene visible, steer-scene count 0). Seule nuance non-réfutante : les keyframes CSS créent encore des objets `Animation` de 0.01ms (filet belt-and-braces canonique).
- **« 0 verdict calculé UI »** — TENU : `GateFlip` anime la TRANSITION d'une prop `value` RESTITUÉE (`VERIFY_ETAT.bootstrap`, un état énuméré), n'en fabrique aucun ; 0 score/jauge/%/`PASS` fabriqué ; gate `scan-front-discipline` inchangé et vert.
- **CSP** — TENUE : altitude = `document.startViewTransition` NATIF + pseudos `::view-transition-*(focal)` en CSS bundlé ; `animateView()` de Motion (injecte `<style>` sans nonce, bloqué) volontairement évité (référencé en commentaire seulement) ; signatures lib via WAAPI (`element.animate`) + inline styles React, CSP-safe ; 0 `url()`/`@import` distant ; Geist vendored same-origin.
- **Rail exclu (Day-0 D8)** — TENU : `view-transition-name:none` sur `[data-testid="operator-rail"]` ET `[data-testid="operator-orientation"]` (testids réels confirmés `Rail.tsx:57`, `OrientationBar.tsx:33`) ⇒ seul `.motion-focal` morphe.
- **oklch / constantes nommées (README §6.9)** — TENU : tokens oklch posés Phase B non redéfinis ; `lib/motion.ts` centralise durées/easings/transitions/énum 5 signatures, réutilisées par `GateFlip`/`Reveal`.
- **0 surface réseau / wire / backend** — TENU : aucun `fetch`/`XHR`/`WS`/`EventSource` dans les fichiers motion ; working tree strictement sous `tools/factory-operator/` + `.planning/`.

---

## 4. Évidence des suites

| Contrôle | Statut (rapporté main-thread) | Note de cohérence |
|---|---|---|
| `cargo fmt --check` / `clippy` / `nextest --workspace` | clean / clean / **2013/2013** | Cohérent — phase front-only, 0 `.rs` touché (Rust inchangé sauf compte global). |
| `npm run lint` (ESLint) | **0 erreur** | Cohérent — allowlist eslint byte-identique (seul commentaire modifié). |
| `tsc --noEmit` + `npm run build` | OK | — |
| `vitest` unit | **92 passed** | Cohérent avec les ~12 fichiers de test neufs. |
| `size-limit` | **0 (tous chunks sous budget)** | **TROMPEUR** : budgets par-chunk indépendants, aucun budget « graphe eager total ». `index.js`=37,24/40 passe parce que le code Motion est physiquement dans `vendor-motion` — mais ce dernier (86,65 KB) charge AU PREMIER PAINT (cf. P1). Le vert size-limit donne une fausse assurance sur le coût de boot. |
| Gates discipline (no-radix, no-tw-config, scan-front) | clean | Cohérent. |
| `playwright` (T1 E2E) | **7/7** (dont motion.spec altitude anti-déco) | Cohérent et load-bearing (preuve anti-déco réelle, pas faux-vert). |

Spot-checks main-thread du bundle (non re-run des suites) : `bundle/index.html` contient `<link rel="modulepreload" crossorigin href="/assets/vendor-motion-isPJV6d8.js">` ; `bundle/assets/index-BVAIrW04.js` (37243 o) débute par `import{c as t,s as n}from"./vendor-motion-isPJV6d8.js"` ; `vendor-motion-isPJV6d8.js` = 86648 o, contient la lib Motion (`reducedMotion`×8, `VisualElement`×1) ; `VerifyScene-Bur46bQR.js` = 7479 o (non matché par aucun glob size-limit). **Tous les faits du P1 et du P2 vérifiés indépendamment.**

---

## 5. Décision sur la DÉVIATION (provider-in-async) — ÉCART À SIGNALER, pas PLAN-ADAPT sain

Le préflight gelait (§3.1) un `MotionConfig` global au root + `LazyMotion` avec loader paresseux via ré-export local `motion-features.ts`. Le 1er build empirique ayant montré ~30 KB de core moteur tiré au hero (mesure préflight « 2 KB » fausse), l'implémentation a **confiné tout le provider à la surface async VERIFY** (`MotionProvider` importé seulement par `VerifyScene = lazy()`), supprimé `motion-features.ts`, et ajouté un `manualChunk vendor-motion` mesuré.

**Côté intention, c'est cohérent** : les 5 signatures sont livrées, rien de la liste HORS-scope n'est touché, et les invariants de doctrine tiennent (cf. §3). **Côté réalité de build, l'objectif central N'EST PAS ATTEINT** :

- Rolldown a routé la résolution de React du hero **À TRAVERS une ré-export façade hébergée dans `vendor-motion`** (le hero fait `import{c as t,s as n}from"./vendor-motion-isPJV6d8.js"` puis `var o=e(t(),1)` et consomme `o.useState`×14 / `o.useCallback`×12 / `o.lazy` / `o.Suspense`). React reste physiquement dans `vendor-react` (pas de duplication, l'app fonctionne), mais cette façade crée une **arête statique dure hero→vendor-motion**.
- Conséquence : `vendor-motion` (86,65 KB de lib Motion) est `modulepreload` au boot + importé en statique par l'entrée ⇒ **fetché + parsé au premier paint, AVANT toute navigation VERIFY**. L'invariant Day-0 « motion mini-entrypoint / motion hors hero » est **violé**.
- Les commentaires committés sont **factuellement faux** : `vite.config.ts:85-91` (« reached ONLY from the async VERIFY surface … stays ASYNC … never bloats the hero ») et `App.tsx:17-27` (« the whole engine + features load only when VERIFY is entered »).

**Intégration adversariale** : claim 1 (« Motion entièrement dans un chunk async, hero sous 40 KB ») = **REFUTED** sur la composante load-bearing « async » ; claim 4 (« chaque chunk motion mesuré + eslint intact + bump CSS 0-token-factorable ») = **REFUTED** (eslint intact VRAI, mais angle mort de glob sur `VerifyScene-*.js` qui porte GateFlip/Reveal + « 0 token factorable » faux). claim 2 (anti-déco) = **CONFIRMED**. claim 3 = placeholder de test (`"test"`/`"test reasoning"`) — ignoré, non load-bearing.

**Verdict sur la déviation** : ÉCART RÉEL à corriger avant commit. Deux voies acceptables : (a) corriger le chunking pour que React se résolve via `vendor-react` (éviter la façade de ré-export dans `vendor-motion` — forcer l'ordre react avant motion, ou ne pas séparer motion si ça hoiste la façade) ⇒ `vendor-motion` redevient vraiment async ; OU (b) assumer honnêtement que Motion est eager (commentaires rendus vrais + re-justification + garde anti-régression `index.html` ne `modulepreload` PAS `vendor-motion` + budget de graphe eager), ce qui rouvre la question « est-ce acceptable au boot ». Dans les deux cas, les commentaires faux et le budget aveugle doivent partir.

---

## 6. Scope cuts respectés (préflight §6)

- **VT du RAIL exclue** — `view-transition-name:none` sur rail + orientation (testids réels). ✓
- **Onglets « Aperçu scellé » / « Preuve »** — non introduits (→ S81). ✓
- **V5/V6 ligne-fine** — non tentés (bande gates reste « non câblées — Phase G/H »). ✓
- **`domMax` / prop `layout` / shared-layout** — 0 usage code (seulement commentaires d'exclusion) ; Reveal = `variants` + `staggerChildren` (couvert `domAnimation`). ✓
- **`animateView()` / `<ViewTransition>` React** — 0 usage code (VT native impérative). ✓
- **Backend AUCUN** — 0 `.rs`, 0 route, 0 wire/canonical/`*_VERSION`. ✓
- **Câblage front de `GET /api/gates`** — non fait ; `GateFlip` = primitive nourrie par un état restitué, 0 client `getGates`/`fetch` (→ Phase H). ✓

Aucun scope cut violé. Note : les bumps `.size-limit.json` (entrée `vendor-motion` + css 20→21) sont hors-liste HORS-scope (pas une violation de cut), mais le bump CSS contredit la préférence préflight §3.5 « factoriser les tokens avant tout bump argumenté » (cf. P3 easing répété).

---

## 7. Findings — disposition

### P0 — aucun.

### P1 — **1, NON corrigé → FAIL**

- **`vendor-motion` (86,65 KB, lib Motion) chargé EAGER au hero — la déviation ne tient pas + commentaires committés faux.** `bundle/index.html` `modulepreload` `vendor-motion` ; `bundle/assets/index-BVAIrW04.js` l'importe en statique top-level (façade de ré-export React `var o=e(t(),1)`, `o.useState`×14). Les 86,65 KB sont fetchés + parsés au premier paint, pas à l'entrée VERIFY. Invariant Day-0 « motion hors hero » violé ; `vite.config.ts:85-91` + `App.tsx:17-27` affirment l'inverse (doc malhonnête, culture doc-honnêteté du projet). `size-limit` reste vert (budgets par-chunk, `index.js`=37,24 ne contient pas le code Motion) ⇒ fausse assurance, aucun budget de graphe eager. **Vérifié indépendamment au bundle.** Remédiation : §5 voie (a) ou (b) + garde anti-régression + commentaires rendus vrais.

### P2 — **1, à fermer**

- **Angle mort de glob size-limit sur `VerifyScene-*.js` (porte 2 des 5 signatures).** Les globs `.size-limit.json` (index-*/vendor-react-*/vendor-motion-*/index-*.css/vendor-xterm-*) ne matchent PAS `VerifyScene-Bur46bQR.js` (7479 o), qui contient `GateFlip` (sig. 2, `rotateX`) et `Reveal` (sig. 3, `staggerChildren`). Une régression alourdissant ces signatures (import de features motion en plus) atterrirait là **hors mesure**. Le préflight §3.1.4 gelait explicitement « + entrée(s) `.size-limit.json` … pour les chunks de surface qui embarquent `m.*` » — mustFix NON honoré. Ajouter une entrée `VerifyScene-*.js` (ou `ProcedeSurface-*.js`) ferme le gap.

### P3 — carries / optionnels

- Commentaire eslint gate-4 sur-promet l'interdiction `import * as` de `motion/react-m` (aucune règle `ImportNamespaceSpecifier` ; seul `motion` plein + `<motion.*>` + dynamic `motion/react` sont bannis). Code OK (imports nommés partout) ; soit ajouter la règle, soit adoucir le commentaire en « interdit par review ».
- `SETTLE_MS`/`ALTITUDE_MS`/`GRAVITY_MS` exportés dans `motion.ts` mais consommés nulle part (durées en dur dans `index.css`) — fausse source-de-vérité « kept in sync » non enforced ; câbler ou retirer.
- `cubic-bezier(0.2,0,0,1)` ×3 et `220ms` ×3 en CSS, factorables en custom properties (préflight §3.5 « factoriser avant bump » non tenté) ; réfute l'affirmation « 0 token factorable ». Marginal (~60 B), n'invalide pas le bump.
- `usePrefersReducedMotion` : le sous-abonnement live (`useEffect` `change`) non testé (mock `addEventListener` no-op, aucun dispatch). Chemin 1er rendu couvert.
- Signature 4 : branche motion-ON sans preuve e2e (seul reduced testé ; positif couvert en unitaire `altitudeShift.test.ts`).
- Signature 5 (gravity / MUR) sans test dédié (CSS-only ; repose sur le pin de constante). 4/5 signatures ont une preuve dédiée.
- `GateFlip` « never renders forbidden verdict word » borderline tautologique (rend `{value}` verbatim) ; valeur faible, le vrai garde = gate `scan-front-discipline`.
- Aucun e2e n'exerce le WAAPI lib sur VERIFY motion-ON sous la CSP réelle (defense-in-depth ; risque réel faible, WAAPI CSP-safe par spec).
- Chunks de surface async non budgétés (ProcedeSurface 16,7 KB, etc.) — gap général pré-existant hors-Motion.

---

## 8. Delta tests

- **Vitest** : 77 → **92** (**+15**) — `motion`/`altitudeShift`/`usePrefersReducedMotion`/`GateFlip`/`Reveal`/`TokenCount`.
- **E2E** : 4 → 7 (dont `motion.spec` altitude anti-déco load-bearing).
- **Rust** : **2013 inchangé** (phase front-only, 0 `.rs` touché).

---

## 9. Verdict + justification

**FAIL.** Les invariants de doctrine de la Phase E (anti-déco reduced-motion sur les 5 chemins, 0-verdict-calculé-UI, CSP non assouplie, oklch, rail exclu, 0 surface réseau/backend) sont tenus et corroborés ligne à ligne, suites toutes vertes (Rust 2013/2013, front 92/92, T1 7/7). MAIS la **déviation centrale que cette phase devait justifier ne tient pas dans le bundle réel** : `vendor-motion` (86,65 KB) est `modulepreload` + importé en statique au hero ⇒ il charge au premier paint, ce qui **viole l'invariant Day-0 « motion hors hero » que la déviation prétendait renforcer**, et **deux commentaires committés affirment faussement le confinement async**. C'est un P1 réel non corrigé (corroboré par 2 adversariaux refutés load-bearing + vérification main-thread du bundle) ⇒ par la règle de sévérité, FAIL. S'ajoute un P2 (angle mort de glob `VerifyScene-*.js` = mustFix préflight §3.1.4 non honoré).

**À corriger avant de re-soumettre** : (1) P1 — rendre `vendor-motion` réellement async (corriger la façade de ré-export React) OU assumer/documenter honnêtement le chargement eager + garde anti-régression + budget de graphe ; commentaires `vite.config.ts`/`App.tsx` rendus vrais ; (2) P2 — entrée size-limit sur le chunk de surface portant `GateFlip`/`Reveal`. Les P3 sont des carries. La doctrine étant saine, le périmètre de correction est ciblé et fixable.

Codex non encore exécuté — verdict committable suspendu (1re review = FAIL).

---

## Re-review après correctif (FAIL→fix)

## 10. Re-review (2026-06-28) — correctif P1/P2 vérifié au bundle réel

**Correctif appliqué** (working tree, re-build) :
1. `manualChunk vendor-motion` SUPPRIMÉ de `vite.config.ts` (cause racine : il hoistait React core dans `vendor-motion`, créant la façade de ré-export hero→vendor-motion). Le split naturel route désormais tout le motion dans le chunk async `VerifyScene-*.js`.
2. `.size-limit.json` : entrée `vendor-motion` → `verify-surface` (`bundle/assets/VerifyScene-*.js`, 92 KB) ⇒ le chunk motion-bearing est mesuré (ferme le P2).
3. Commentaires corrigés (`MotionProvider.tsx`, `eslint.config.js`, `App.tsx`, `vite.config.ts`) ; **0 mention résiduelle `vendor-motion`** dans la source (grep src/config = NONE).
4. `SETTLE_MS`/`ALTITUDE_MS`/`GRAVITY_MS` morts retirés de `lib/motion.ts` (P3).
5. Règle eslint `no-restricted-syntax ImportNamespaceSpecifier` ajoutée (bannit `import * as` de `motion/react-m`) (P3).
6. Tests ajoutés : `usePrefersReducedMotion` live-change ; `Mur` classe `motion-gravity` (P3).

**Vérification indépendante au bundle bâti** (`tools/factory-operator/bundle/`, spot-checks main-thread, suites non re-run) :

| Sous-check | Évidence bundle | Verdict |
|---|---|---|
| (a) `index.html` ne modulepreload AUCUN chunk motion | `index.html` : `modulepreload` UNIQUEMENT `rolldown-runtime-CNC7AqOf.js` + `vendor-react-D0i3EZzp.js` ; stylesheet `index-bEA16Wfe.css`. **0 `vendor-motion`, 0 `VerifyScene` préchargé.** | ✓ |
| (b) le hero n'importe AUCUN chunk motion en statique | hero `index-DbUOCScn.js` imports statiques = `{rolldown-runtime, vendor-react}` SEULS ; imports **dynamiques** = `import(VerifyScene-…)` + `import(SurfaceHost-…)`. | ✓ |
| (c) le code motion est dans `VerifyScene-*.js` (async) | `VerifyScene-DSqtvLQa.js` (86 072 o) = SEUL chunk portant la lib (`reducedMotion`×2, `VisualElement`×1, `animate`×2, `cubic-bezier`×1) ; scan tous-chunks = lib motion uniquement dans VerifyScene. Importeurs statiques de VerifyScene = **NONE** ; importeur dynamique = hero seul ⇒ async. | ✓ |
| (d) React core dans `vendor-react` ; hero importe React de là | `vendor-react-D0i3EZzp.js` (189 811 o) contient React core (`react.dev`×1, `createElement`/`useState`/`useReducer`/`Fragment`) ; hero importe React via `from"./vendor-react"` ; VerifyScene importe React via `from"./vendor-react"` (`react.dev`×0 ⇒ pas de duplication). | ✓ |
| Graphe eager total | clôture = `{index hero, vendor-react, rolldown-runtime}` (vendor-react n'importe que rolldown-runtime ; rolldown-runtime n'importe rien). **0 marqueur motion** dans les 3. | ✓ |
| P2 blind spot | `verify-surface` (path `VerifyScene-*.js`, 92 KB) mesure le chunk motion-bearing ; scan confirme la lib motion N'EST QUE dans VerifyScene ⇒ aucun autre chunk de surface ne porte de motion non-mesuré. | ✓ fermé |

**Évidence suites (rapportée, cohérente)** : build 0 ; size-limit 0 (app 37,16/40, vendor-react 189,81/210, verify-surface 86,07/92, css 20,64/21, vendor-xterm 341,54/360) ; lint 0 ; tsc 0 ; vitest 94/94 ; gates clean ; T1 E2E 7/7 (motion.spec anti-déco `vtCalls==0` + `getAnimations running==0`) ; Rust 2013/2013 inchangé (0 `.rs`).

**Synthèse re-review (agent de synthèse, vérification indépendante)** : les faits du correctif sont re-confirmés par spot-checks bundle main-thread, NON sur foi du diff :
- `bundle/index.html` `modulepreload` = `rolldown-runtime-CNC7AqOf.js` + `vendor-react-D0i3EZzp.js` SEULS (0 motion).
- Hero `index-DbUOCScn.js` : imports **statiques** = `{rolldown-runtime, vendor-react}` ; imports **dynamiques** = `import(\`./VerifyScene-DSqtvLQa.js\`)` + `import(\`./SurfaceHost-…\`)`.
- Marqueurs motion (`domAnimation|reducedMotion|VisualElement|createVisualElement|makeUseVisualState`) = **10 dans `VerifyScene-DSqtvLQa.js` SEUL**, **0 dans les 11 autres chunks** (hero, vendor-react, vendor-xterm, rolldown-runtime, toutes les surfaces lazy).
- `VerifyScene-DSqtvLQa.js` = 86 072 o, mesuré par `verify-surface` (≤ 92 KB).
- `grep vendor-motion` sur `src/` + `vite.config.ts` + `eslint.config.js` + `.size-limit.json` = **NONE**.

Les 4 vérifications (p1-motion-async, p2-measured, comments-honest, no-regression) sont toutes `resolved=true` ; l'unique adversarial (« P1 résolu / tout motion async ») = **confirmed**, aucun adversarial réfuté load-bearing. P3 résiduels non bloquants : petites surfaces lazy (<17 KB, 0 motion) hors glob size-limit (gap pré-existant, hors P2) ; chiffres RAW ~30 KB/~37 KB non re-dérivables séparément (estimations honnêtes hedgées `~`, le gate agrégé `verify-surface` est exact).

**Conclusion re-review** : P1 (motion eager) RÉSOLU + vérifié au bundle ; P2 (blind spot size-limit) FERMÉ ; P3 traités ; 0 nouveau P0/P1 ; doctrine inchangée (§3). Motion réellement HORS du graphe eager — invariant Day-0 « motion hors hero » tenu. **resolved=true.**

**Nouveau verdict** : **PASS-PENDING** — le P1+P2 sont vérifiés résolus au bundle, 0 nouveau P0/P1, adversarial confirmé. Le verdict committable `## Verdict: PASS` exact sera promu **APRÈS** passage Codex (gate BLOQUANTE review→commit non encore exécutée).

## Codex reconciliation

Gate Codex (GPT-5.5, `codex exec`, reasoning effort xhigh) exécutée APRÈS la
re-review PASS-PENDING — sortie brute dans `sprint80_phase_e_codex_review.md`
(non réécrite). **Verdict Codex : 18/18 livrables CONFIRMÉ, 0 GAP, 0 PARTIEL.**

Codex a vérifié INDÉPENDAMMENT les deux points qui avaient fait échouer la 1re
review :
- **Motion hors hero (au bundle)** — CONFIRMÉ : `vite.config.ts` ne force que
  `vendor-react` + `vendor-xterm` (aucun `vendor-motion`) ; `size-limit` mesure
  `verify-surface` sur `VerifyScene-*.js` (86,07/92) ; hero `app` 37,16/40 sans
  motion. Codex a relancé `npm run size`.
- **Anti-déco + 0-verdict + CSP + constantes nommées** — CONFIRMÉ ligne à ligne ;
  Codex a relancé `npm run lint`, `npm run gate:scan-front`, les tests unitaires
  ciblés (7 + 20 passés) et `npx playwright test e2e/motion.spec.ts` (1 passed).

Aucun GAP P0/P1/P2/P3 soulevé par Codex. Les P3 résiduels documentés §7 (petites
surfaces lazy <17 KB sans motion hors glob size-limit = gap pré-existant ; chiffres
RAW ~30/~37 KB non re-dérivables séparément après fusion dans l'unique chunk
`verify-surface`, estimations honnêtes hedgées) restent des carries non bloquants.
Suites non re-jouées par Codex (Rust 2013/2013) inchangées — phase front-only.

Séquence respectée : preflight PLAN-ADAPT → review FAIL → correctif racine →
re-review PASS-PENDING → Codex CLEAN → promotion PASS. La gate adversariale a
prouvé sa valeur (P1 « motion eager » attrapé puis corrigé à la racine).

## Verdict: PASS
