VERDICT: PASS

# Sprint 80 — Phase B — Review (scaffold greenfield front Factory Operator)

Date : 2026-06-27
Mode : ULTRACODE (5 dimensions + 4 vérifications adversariales, faits ré-exécutés localement)
Verdict : **PASS** (0 P0 / 0 P1 ; Codex reconcilié — 7 tours, tous les P1 fermés, résidus P2/P3 documentés)

---

## §Résumé exécutif

Phase B livre exactement son périmètre : refonte **greenfield** du front Factory Operator.
Suppression intégrale de `tools/factory-ui/` (socle orphelin S70, supersedé) ET de l'ancien
`tools/factory-operator` (pages/ui/i18n/hooks shadcn+Radix), remplacés par un scaffold
React 19 + React Compiler + Tailwind v4 CSS-first oklch + Base UI + Motion + Geist vendoré.
5 gates de discipline BLOQUANTS câblés en CI dès cette phase + squelette T1 Playwright
hermétique sur le bundle BÂTI. **0 fichier `.rs` édité, 0 wire/canonical, 0 route daemon,
0 route backend.**

Diff stat vérifié : `73 files changed, 1758 insertions(+), 11622 deletions(-)` → vraie
suppression massive + scaffold modeste, pas un re-skin. `git diff --cached --name-only | grep
'\.rs$'` = **vide**. Seuls fichiers non-front stagés : `.github/workflows/ci.yml` +
`.woodpecker/ci-linux.yml` (jobs CI, purs appends).

État des vérifications locales (toutes reproduites indépendamment, pas crues) :
- **build** vert, 0 warning, déterministe (rebuild fraîche → hash CSS identique `index-DUShbudM`)
- **tsc** `-p tsconfig.app.json` = 0
- **eslint / gate-4** = 0
- **gate-1 anti-radix** clean (3 couches) ; **gate-2 anti-tw-config** clean ; **gate-5 scan-front** clean
- **size/gate-3** : app 28.57/40 · vendor-react 189.78/210 · css 9.95/20 KB
- **T1 hermétique** PASS (Operator Rust réel :3111, boot cookie → 303, 0 violation CSP sur bundle BUILDÉ)
- **gates avec dents** : testés adversarialement (radix synthétique → catch ; `export const='PASS'`
  → catch ; `import {motion} from 'motion/react'` → eslint error ; `<motion.div>` → 2 errors)
- **0 vuln npm** ; lockfile en sync ; aucun artefact bâti stagé (`bundle`/`test-results` gitignorés)

Aucun P0/P1 sur aucune dimension. 4 P2 de discipline/robustesse + un faisceau de P3.

---

## §Findings par dimension

### D1 — Scope / Day-0 / wire (verdict : conforme)
- **0 Rust / 0 wire / 0 backend** : confirmé `git diff --cached`. Contrat Phase A intact :
  `operator_server.rs:47` `OPERATOR_BUNDLE_SUBDIR = "tools/factory-operator/bundle"` =
  `vite.config.ts` `outDir:'bundle'`.
- **Day-0 D1..D11 honorées** : React 19.2 + `babel-plugin-react-compiler@1.0.0` ;
  `@base-ui/react@1.6.0` (bon nom, PAS `@base-ui-components/react` rc) seule dep runtime, 6
  `@radix-ui/*` + shadcn + cva retirés ; `motion ^12.42.0` seule lib (0 code en B) ; Tailwind v4
  oklch 0 config ; Geist sans+mono vendoré fontsource (OFL, 0 CDN) ; CSP self-origin = Phase A.
- **Jettison propre** : `tools/factory-ui/` entièrement déréférencé du tracking, 0 référence
  résiduelle dans le code ; `src/` greenfield = 6 fichiers ; 0 import radix/shadcn/cva/i18next/
  react-router résiduel.
- **Scope cuts respectés** : `grep -rnE "motion|EventSource|ReadableStream|/api/" src` = vide
  (motion=E, SSE fetch+ReadableStream=C, VERIFY-plein=F-H).

### D2 — Robustesse des 5 gates (verdict : robustes sur le chemin porteur)
- Gate-1 anti-radix (a pkg / b src imports + eslint `patterns:['@radix-ui/*']` / c arbre prod
  `npm ls --omit=dev`) ; gate-2 `tailwind.config.{js,ts,cjs,mjs}` + `@config` ; gate-3 budgets
  chiffrés mesurés+marge ; gate-4 `no-restricted-imports motion` + `no-restricted-syntax
  JSXMemberExpression[object.name='motion']` ; gate-5 `\b(PASS|Vérifié|Approuvé)\b` case-sensitive
  + allowlist `=== 'PASS'`. Tous testés avec dents.
- **P2-2 (robustesse)** : gate-1 couche (c) `check-no-radix-runtime.sh:36` masquage pipefail —
  voir vérification adversariale #1.

### D3 — build / CSP / ADAPT (verdict : build-validé sur le bundle réel)
- **ADAPT-1** React Compiler via `react()` + `babel({ presets: [reactCompilerPreset()] })`
  (`vite.config.ts:50-54`) — pattern canonique v6 confirmé par README du plugin installé, PAS de
  `react({babel})` mort. `optimizeDeps.include: ["react/compiler-runtime"]` (React 19, pas le shim).
- **ADAPT-2** `@theme` SIMPLE oklch (valeurs directes), PAS `@theme inline`+`:root`. Vérifié sur le
  CSS BÂTI : `.bg-s0/.bg-s1/.border-bd/.text-tx/.text-tx2/.text-tx3/.font-mono/.font-sans/.rounded-md`
  émis, `oklch` présent, **0 fuite `@theme`/`@utility`/`@custom-variant`**. 20/20 classes consommées
  par `App.tsx` résolues dans le CSS. Raffinement build-validé, pas une dérive.
- **CSP-clean** : `modulePreload.polyfill:false` + `assetsInlineLimit:0` → bundle 0 script inline,
  0 URI `data:`, woff2 same-origin. `App.tsx` 100% classes Tailwind, 0 `style={{}}`.

### D4 — T1 hermétique / CI (verdict : squelette hermétique réel)
- `serve-operator.mjs` build le front puis spawn le **vrai** Operator Rust (pas `vite dev`) ;
  `playwright.config.ts` injecte `SBFB_HOME` tmpdir (0 `~/.sbfb` touché) + token 64-hex fixe + port
  dédié 3111 (≠ défaut CLI 3001). `boot.spec.ts` : `/?token` → cookie HttpOnly → 303 → rail visible
  → reload cookie-seul → collecteur 0 violation CSP.
- CI GHA `factory-operator` : tsc → lint → `npm run gates` → build → size → **T1 BLOQUANT** (build
  binaire Rust + chromium, sans `continue-on-error`) ; Woodpecker = gates+build+size (T1 délégué GHA,
  documenté). Diffs CI = purs appends.
- **P3** : T1 asserte l'absence de violations mais pas la présence de l'en-tête CSP (cf. adversarial #2).

### D5 — deps / grounding / body (verdict : conforme, body à valider au commit)
- Lockfile en sync (base-ui 1.6.0, @rolldown/plugin-babel 0.2.3, react-compiler 1.0.0, motion
  12.42.0, geist 5.2.9, playwright 1.61.1). 0 vuln npm. `@base-ui/react` pin exact `1.6.0`.
- Aucune promesse future in-code orpheline (anti STALE-PHASE-K) : les « land in Phase X » sont
  descriptifs/planning.
- **P2-1** vitest non-déclaré ; **P2-3** doc factory-ui périmée ; **P2-4** couverture Vitest → 0 unité.

---

## §Vérifications adversariales

**#1 — Les 5 gates sont robustes, pas du théâtre → CONFIRMED (avec P2).**
La claim tient sur le chemin porteur. En CI, `npm ci` (ci.yml) reconstruit un arbre lock-exact
propre AVANT `npm run gates` → `npm ls --omit=dev` sort 0 → la couche (c) attrape réellement le
radix transitif. Tout radix effectivement importé est pris par (a)+(b)+eslint indépendamment de
l'arbre. **Faille réelle isolée** : `check-no-radix-runtime.sh:36` `npm ls (exit≠0 arbre sale) |
grep -q (match)` sous `set -euo pipefail` hérite du non-zéro de `npm ls` → le `if` est faux →
radix transitif manqué EN LOCAL si l'arbre est sale. Backstoppé par (a)+(b)+eslint → **pas P1**.
Correction 1-ligne recommandée : `out=$(npm ls --omit=dev --all 2>/dev/null || true); echo "$out"
| grep -q "@radix-ui"`. **Classé P2-2.**

**#2 — Le T1 valide la CSP sur le bundle BUILDÉ + cross-platform Win+Linux → REFUTED (sur les 2
portions load-bearing).**
Solide : hermétique (SBFB_HOME tmpdir), bundle BÂTI (pas `vite dev`), boot cookie réel.
Faille A (validation CSP) : `boot.spec.ts:46` n'assert que `violations.toEqual([])` via collecteur
console — jamais `resp.headers()['content-security-policy']`. Le scaffold étant 100% same-origin
(0 inline, 0 cross-origin), **0 violation se produirait même SANS aucune CSP** → l'assertion est
vacuously vraie et ne détecterait pas une régression de l'en-tête Phase A. Correction : asserter la
**présence** de l'en-tête `default-src 'self'` sur le doc bootstrap. **Classé P3** (renforcement
C/H), backstoppé par le contrat Phase A séparé déjà committé.
Faille B (cross-platform) : T1 ne tourne qu'en **Linux-CI** (GHA ubuntu-latest ; Woodpecker n'a pas
Playwright) + **Win-local-dev best-effort**. Aucun runner Windows n'exécute T1 en CI. **Classé P3**
(recadrer le wording « cross-platform Win+Linux » → « Linux-CI / Win-local »).
→ Le T1 reste un squelette hermétique valide et non-vacuous **pour le boot cookie** ; il ne valide
simplement pas encore la CSP comme le claim le prétendait. Aucun P0/P1.

**#3 — 0 wire/route backend + Day-0 honorées sans relâcher la doctrine → CONFIRMED.**
0 `.rs` édité, 0 canonical, 0 route daemon/backend ; diff strictement front + suppression + jobs
CI. Seule nuance = l'edge pipefail gate-1 (P2-2), affaiblissement anti-drift backstoppé, pas un
trou exploitable. Doctrine non relâchée.

**#4 — Le passage @theme inline → @theme simple est un raffinement build-validé, non une dérive →
CONFIRMED.**
Rebuild fraîche déterministe : 9 utilitaires token générés avec oklch, 0 fuite ; 20/20 classes
consommées résolues ; 0 `tailwind.config.*`/`@config` (gate-2 clean, pas d'escape-hatch).
Tailwind installé = 4.3.1 (ancre la note du commentaire). Forme retenue = canonique Tailwind v4.
Aucune correction nécessaire.

---

## §Décision

**PASS-PENDING.** 0 P0 / 0 P1. Le scaffold livre exactement son scope, toutes les affirmations
« déjà vert » reproduites indépendamment, invariants tenus. Les 4 P2 sont des points de
discipline/robustesse à régler au commit ou en Phase I, **non bloquants**.

Aucun P0/P1 BLOQUANT.

Étape suivante obligatoire avant commit : **gate Codex** (review→commit, BLOQUANTE) + commit body
9 sections conforme.

---

## §Points pour le commit body

- **Titre** : `feat(factory-operator): Sprint 80 Phase B — scaffold greenfield front (React 19 +
  Compiler + Tailwind v4 oklch + Base UI + Motion + Geist vendoré + 5 gates + T1)`.
- **Delta tests** : tracer la descente **−7/−8 Vitest factory-operator** (`executionChat.test.ts`
  single-Done PO-14 + `ExecutionChat.test.tsx` supprimés) → re-couverte **Phase I** via
  `useTokenStream`. Total interdit de descendre silencieusement (CLAUDE.md). Ajout : +T1 Playwright
  hermétique (1 spec, 5 assertions boot/CSP).
- **Scope cuts** : motion = Phase E ; SSE fetch+ReadableStream = Phase C ; VERIFY-plein (diff-viewer
  + panneau gates) = Phases F-H ; re-couverture unités = Phase I.
- **Supersede S70** : jettison `tools/factory-ui` (`@sbfb/factory-ui`) acté (CLAUDE.md:495-496 tracé
  par le kickoff) ; reconciliation doc `README.md:430` + `RRV_FACTORY_CONTRACT.md:106-141` →
  **différée S81** (fondation Viewer re-planifiée) OU corrigée au wrap-up (P2-3).
- **Ce qu'il reste (Phases C-J)** : C câblage API + SSE fetch+ReadableStream ; D arborescence procédé
  + docs (quick-wins §5.1) ; E motion (LazyMotion+m) ; F-H VERIFY-plein (GET /api/git/diff déjà livré
  Phase F backend + GET /api/gates) ; I re-couverture Vitest (PO-14) ; J intégration finale.

---

## §P2/P3 à documenter (non bloquants)

- **P2-1** `vitest` invoqué (`test:unit`/`test:unit:watch`/`test:coverage`) mais retiré de
  `package.json` devDependencies (seul `@vitest/coverage-v8` reste) ; résout transitivement (peer
  hoisté). Fragile à un futur `npm install`. Sans impact CI/T1 (vitest non lancé en CI). Fix :
  redéclarer `"vitest": "^4.1.9"` en devDep au plus tard Phase I.
- **P2-2** gate-1 couche (c) masquage pipefail `npm ls` non-zéro (cf. adversarial #1) — fix 1-ligne.
- **P2-3** doc périmée vers `tools/factory-ui` supprimé : `README.md:430` (arbre liste encore
  `factory-ui/`), `RRV_FACTORY_CONTRACT.md:106,109,140,141`. À tracer différée-S81 ou corriger.
- **P2-4** couverture Vitest → 0 unité, pas de filet CI front jusqu'à Phase I (acté plan + CLAUDE.md ;
  pas de faux-vert injecté car CI ne lance ni unit ni coverage).
- **P3** : (a) gate-4 ne couvre pas `motion()` factory ni `import {motion} from 'motion'` racine —
  tout `<motion.*>` JSX pris quelle que soit la source ; (b) T1 asserte l'absence mais pas la
  présence de l'en-tête CSP (renforcer C/H : `resp.headers()['content-security-policy']`) ;
  (c) « cross-platform Win+Linux » sur-affirmé → Linux-CI / Win-local-dev ; (d) preuve empirique
  React Compiler reportée (scaffold trivial = 0 artefact `_c(`/`compiler-runtime`) — câblage prouvé,
  transform non démontrée in-situ ; (e) `e2e/*.ts` + `playwright.config.ts` + `vitest.config.ts` hors
  `tsconfig.include` → pas de typecheck `tsc` (esbuild gère, eslint couvre) ; (f) sub-test « transport
  cookie » légèrement sur-affirmé (`GET /` bootstrap public ; cookie réellement exercé au 1er
  chargement post-303) ; (g) woff2 Geist sur-vendorés (sous-ensembles cyrillic/vietnamese inutiles UI
  FR) non budgétés ; (h) drift commentaire `operator_server.rs:43` « dist thrown away » alors que
  sortie = `bundle/` (pre-existant, non stagé) ; (i) `SBFB_FACTORY_BIN` sans normalisation `.exe`
  (dev Windows doit l'inclure ; fallback `cargo run` OK) ; (j) `sprint80_phase_b_preflight.md`
  untracked à committer selon convention .planning.

---

## Verdict: PASS

0 P0 / 0 P1. Codex reconcilié sur 7 tours (tous les P1 fermés, résidus P2/P3
documentés). PASS-PENDING absent du commit final.

## Codex reconciliation

Gate Codex (GPT 5.5, `codex exec`) jouée sur **7 tours** (output brut
`sprint80_phase_b_codex_review.md`, jamais réécrit). Verdict final R7 :
6 livrables, 4 CONFIRME + 2 PARTIEL, **0 P0, 0 GAP**. Chaque P1 détecté a
été corrigé puis re-vérifié au tour suivant :

- **R1** — gate-5 ratait `Verifie`/`Approuve` sans accent ; T1 n'assertait
  pas explicitement 303 + `Set-Cookie HttpOnly` → corrigés.
- **R2** — allowlist gate-5 jetait la ligne entière (`=== 'PASS' ? <span>PASS</span>`
  passait) ; launcher T1 testait un bundle stale → sed-strip + rebuild systématique.
- **R3** — `import('@radix-ui')` / `import('motion/react')` dynamiques échappaient
  aux gates → grep gate-1 + sélecteurs `ImportExpression` ESLint.
- **R4** — docs contractuelles (README, CLAUDE, RRV_FACTORY_CONTRACT) référençaient
  `factory-ui` supprimé → reconciliées (notes supersede S80 Phase B).
- **R5/R6** — couche (b) gate-1 ratait l'import side-effect → grep rendu airtight
  (match de chaîne pure, toutes formes d'import).
- **R7** — `README_EN` référençait encore `factory-ui` → reconcilié.

**Résidus P2/P3 documentés (committables, non bloquants)** :
- Les chaînes `tools/factory-ui` restantes (CLAUDE.md:495, RRV:109/142) sont
  les **notes supersede elles-mêmes** (« factory-ui jeté ») + le texte historique
  S70 conservé — documentation correcte, pas une assertion stale (faux-positif
  du pattern-match Codex sur le chemin).
- `lightningcss MPL-2.0` (transitif Tailwind v4) : build-time only (minifieur CSS
  natif, jamais dans le bundle navigateur), **AGPL-compatible** (MPL §3.3) ; déjà
  présent via le Tailwind actuel. L'invariant réel = AGPL-compatible (tenu), pas
  « MIT-strict ».
- Mentions `factory-ui` dans `.planning/research/*` : point-in-time, non-contractuelles
  (Codex P2/P3) → laissées.

review.md promu **PASS** ; PASS-PENDING absent du commit final.

SIGNAL: PASS