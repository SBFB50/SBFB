VERDICT: PLAN-ADAPT

# Sprint 80 — Phase B — Preflight G8 (scaffold greenfield front Factory Operator)

- **Type** : preflight G8 (5 scans S1a/S1b/S2/S3/S4 + 5 vérifications adversariales)
- **Date** : 2026-06-27
- **Phase** : B — Scaffold greenfield + 5 gates de discipline + jettison `factory-operator`/`factory-ui`
- **Surfaces** : FRONT scaffold uniquement, **0 route backend** (consomme `/api/*` posé Phase A `a5ace8d`)
- **VERDICT : PLAN-ADAPT**

> Le plan Phase B est exécutable **sans toucher aucune décision Day-0 (D1..D11)** et sans relâcher aucune doctrine gelée. Verdict PLAN-ADAPT (et non EXECUTE) parce que **deux claims load-bearing de la recette/BASE sont réfutés avec correction OSS concrète** — un implémenteur suivant le blueprint ou sa mémoire de training verbatim produirait du code mort/non-fonctionnel :
> 1. **Câblage React Compiler sous Vite 8 / `@vitejs/plugin-react` v6** : le pattern legacy `react({ babel:{ plugins:[...] } })` est **mort** (plugin-react v6 a abandonné Babel pour oxc). Correction OSS = `react()` + `@rolldown/plugin-babel` chargé avec `reactCompilerPreset()`.
> 2. **Portabilité des tokens hi-fi dans `@theme`** : les tokens TERSES de la maquette (`--s0..--s3`, `--tx..--tx4`, `--bd`, `--ok`, `--mur`, `--sans`) **ne génèrent AUCUN utilitaire** s'ils sont collés verbatim — Tailwind v4 `@theme` ne produit des classes que sous namespaces réservés (`--color-*`, `--font-*`, `--radius-*`). Correction = re-namespacer (valeurs hi-fi = spec, nommage à mapper).
>
> Aucune des deux n'affecte les **livrables** ou les **décisions Day-0** : ce sont des adaptations de recette à appliquer au code. Les 5 scans signalent EXECUTE en les traitant comme des « précisions à plier au code » ; je les promeus au rang d'**adaptations BLOQUANTES** parce qu'elles cassent le build ou le rendu si elles sont ignorées (§4.5.7).

---

## §S1a — Recette scaffold (versions EXACTES, vérifiées npm/context7 juin 2026)

Toutes les versions ci-dessous sont `latest` GA au 2026-06-27. Aucune brique non-GA bloquante. Solid 2.0 reste Beta (D1 re-confirmé, cf. §S2). React Compiler v1.0 = GA (7 oct. 2025).

### Versions cibles

| Item | Version cible | Licence | Note |
|---|---|---|---|
| `react` / `react-dom` | `^19.2.7` | MIT | aligner sur shell `web/` (`^19.2.4` → web rattrapera) |
| `vite` | `^8.1.0` (≥ 8.0.16 mini) | MIT | Rolldown ; ≥8.0.16 clôt 2 avis high build-time |
| `@vitejs/plugin-react` | `^6.0.1` | MIT | **v6 = oxc, plus Babel** → wiring Compiler change (cf. ADAPT-1) |
| `babel-plugin-react-compiler` | `^1.0.0` | MIT | GA ; **pin exact** (couverture T1 pas dense, plan D1 #3) |
| `@rolldown/plugin-babel` | (latest) | MIT | **REQUIS** pour faire tourner le Compiler sous Vite 8 |
| `@babel/core` | (peer) | MIT | peer de `@rolldown/plugin-babel` |
| `tailwindcss` / `@tailwindcss/vite` | `4.3.1` | MIT | CSS-first, 0 `tailwind.config.js` |
| `@base-ui/react` | **pin exact** 1.6.0 | MIT | nom GA canonique (cf. ci-dessous) |
| `motion` | `^12.42.0` | MIT | `motion/react` + `motion/react-m` ; tire `framer-motion`+`tslib` (note A) |
| `@fontsource-variable/geist` | `^5.2.9` | OFL-1.1 | déjà au repo |
| `@fontsource-variable/geist-mono` | `^5.2.8` | OFL-1.1 | **NET-ADD** (D5) — note : 5.2.8, PAS 5.2.9 |
| `size-limit` + `@size-limit/file` | `12.1.0` | MIT | miroir EXACT du shell `web/` (`@size-limit/file`, pas preset-app) |
| `@playwright/test` | (devDep) | Apache-2.0 | squelette T1 hermétique |

> **Nom de package Base UI TRANCHÉ (D2)** : `@base-ui/react` (latest 1.6.0, auteur MUI Team, repo `github.com/mui/base-ui`, MIT). L'ancien `@base-ui-components/react` est gelé à `1.0.0-rc.0` (mort) — **ne PAS l'utiliser**. Les deux package.json du repo (`web/` `^1.3.0`, `factory-operator/` `^1.5.0`) emploient déjà `@base-ui/react` (build vert). Dependencies de `@base-ui/react` = `@floating-ui` + `@babel/runtime` + `@base-ui/utils`, **0 `@radix-ui` transitif** → gate (1) « 0 radix runtime » atteignable.

**Note A — `motion` n'est pas une feuille** : `motion@12.42` dépend de `framer-motion@^12.42` (rebrand ; `motion/react` re-exporte `framer-motion`). Deux MIT → 0 souci licence ; le « 1 lib motion » Day-0 D4 se matérialise en 2 paquets installés. Tree-shaking préserve le hero `LazyMotion`+`m` → à confirmer par le gate (3).

### `@theme` Tailwind v4 — tokens oklch (source de vérité = hi-fi, PAS blueprint §5.4)

**Réconciliation importante** : le blueprint §5.4 utilisait `oklch(15% 0 0)` (chroma 0 PUR) ; la maquette hi-fi `Factory Operator - hi-fi.dc.html:17-37` utilise **hue 260 + micro-chroma** (gris-bleu froid). Le plan (l.21-22) fait foi du **hi-fi**. **ADAPT-2** : porter les valeurs hi-fi MAIS re-namespacer (les tokens terses ne génèrent aucun utilitaire en `@theme` — cf. adversarial #3).

Mapping recommandé (valeur hi-fi → token `@theme` namespacé) :

```css
@import "tailwindcss";
@custom-variant dark (&:where([data-theme="dark"], [data-theme="dark"] *));

:root {
  /* Surfaces (séparation par la ligne, pas l'ombre) */
  --s0: oklch(0.165 0.004 260);  --s1: oklch(0.205 0.005 260);
  --s2: oklch(0.245 0.006 260);  --s3: oklch(0.290 0.007 260);
  --bd: oklch(0.330 0.006 260);  --bd2: oklch(0.430 0.008 260);
  /* Encre (4 niveaux) */
  --tx: oklch(0.930 0.004 260);  --tx2: oklch(0.720 0.005 260);
  --tx3: oklch(0.540 0.006 260); --tx4: oklch(0.450 0.006 260); /* consultatif/fantôme */
  /* État (couleur = sens, jamais déco) */
  --ok: oklch(0.74 0.14 150);    --warn: oklch(0.80 0.12 78);
  --bad: oklch(0.68 0.18 25);    --info: oklch(0.72 0.11 240);  --neu: oklch(0.54 0.006 260);
  --ok-bg: oklch(0.30 0.045 150);--bad-bg: oklch(0.31 0.055 27);
  /* MUR gouvernance (amber = gravité) */
  --mur: oklch(0.66 0.105 78);   --mur-bg: oklch(0.235 0.030 78);
}

@theme inline {
  /* DOIT être sous namespaces réservés sinon 0 classe générée */
  --color-s0: var(--s0);  --color-s1: var(--s1);  --color-s2: var(--s2);  --color-s3: var(--s3);
  --color-bd: var(--bd);  --color-bd2: var(--bd2);
  --color-tx: var(--tx);  --color-tx2: var(--tx2); --color-tx3: var(--tx3); --color-tx4: var(--tx4);
  --color-ok: var(--ok);  --color-warn: var(--warn); --color-bad: var(--bad);
  --color-info: var(--info); --color-neu: var(--neu);
  --color-ok-bg: var(--ok-bg); --color-bad-bg: var(--bad-bg);
  --color-mur: var(--mur); --color-mur-bg: var(--mur-bg);
  --font-sans: 'Geist Variable', ui-sans-serif, system-ui, sans-serif;
  --font-mono: 'Geist Mono Variable', ui-monospace, monospace;
  --radius-sm: 3px; --radius-md: 4px; --radius-lg: 5px; /* rayons serrés hi-fi, PAS 8/16px blueprint */
}
```

**Décisions de design portées par le hi-fi (à ne pas perdre)** :
- **CTA primaire = `--tx` (quasi-blanc) sur fond sombre**, PAS un accent coloré (interaction states hi-fi l.839-843).
- **focus-visible = `outline:2px solid var(--info)` offset 2px** (hi-fi l.831,841). La hi-fi n'a PAS de token « accent interactif » unique → teinte focus/lien = `--info` (240).
- **rayons serrés 3-5px** (le blueprint disait 8/16px = trop arrondi → suivre hi-fi).
- `tabular-nums` sur tous les compteurs/données mono. Dark par défaut (`--s0` = backdrop).
- La hi-fi charge Geist via Google Fonts CDN (l.12-14) = **maquette uniquement** → vendorer (cf. §S3).

### Base UI — primitives (import composable)

~8 primitives à comportement coûteux mappées aux surfaces : **Dialog** (focus-trap MUR), **Tabs** (Diff/Aperçu/Preuve + altitudes), **Menu/DropdownMenu** (provider/agent), **Select/Combobox** (provider), **Tooltip**, **Popover**, **Collapsible** (replier « ▸ détails techniques »), **ScrollArea**. Import réel = parts composables : `import { Dialog } from '@base-ui/react/dialog'` → `<Dialog.Root><Dialog.Trigger/><Dialog.Portal><Dialog.Popup/>…`. Tout le vocabulaire métier (cartes diff/tool-call, lignes de gate, rail, MUR) reste **HTML + Tailwind maison**.

### Motion — `LazyMotion` + `m` (hero ~4,6 kb CONFIRMÉ)

Budget hero confirmé context7 `/websites/motion_dev` : composant `motion` plein = **34 kb** ; `m` + `LazyMotion` = **« just under 4.6kb for the initial render »**. `domAnimation` ≈ 15 kb chargé à la demande ; `domMax` ≈ 27 kb **non nécessaire** (0 layout/drag).

**ADAPT-3 (import path moderne)** : `m` s'importe de `motion/react-m`, `LazyMotion`/`domAnimation` de `motion/react` :
```jsx
import { LazyMotion, domAnimation, MotionConfig } from 'motion/react'
import * as m from 'motion/react-m'
<MotionConfig reducedMotion="user">          {/* Phase E : état final instantané */}
  <LazyMotion features={domAnimation} strict>{/* strict = interdit <motion.*>, force <m.*> */}
    <m.div animate={{ opacity: 1 }} />
  </LazyMotion>
</MotionConfig>
```
`features={domAnimation}` couvre les 5 signatures (token settle, gate flip, verification reveal, confirmation gravity) ; l'altitude-shift = **View Transitions natif** (pas Motion). `strict` = filet runtime complémentaire au lint (4). Le code motion est **Phase E** ; en Phase B, le gate (4) est **préventif** (légitime) et le gate (3) mesure le chunk réel.

### Geist vendoré (fontsource, 0 CDN)

`import '@fontsource-variable/geist'` + `import '@fontsource-variable/geist-mono'` dans l'entrée. `--font-sans: 'Geist Variable'` / `--font-mono: 'Geist Mono Variable'` (noms fontsource variable, PAS `'Geist'`/`'Geist Mono'` des liens CDN de la maquette).

### size-limit (gate chiffré)

Calque `web/.size-limit.json` (`@size-limit/file`, `brotli:false gzip:false`). `.size-limit.json` listant les chunks (entrée hero Motion isolée + vendor-react + css). **Ne PAS hard-coder 4,6 kb par décret** : poser `LazyMotion features={domAnimation}`, **mesurer** le hero réel, figer le budget = mesuré + marge (~5 kb). Le gate vérifie la non-régression, pas une promesse. `size-limit` est **inexistant** côté factory-operator aujourd'hui (gate genuinement neuf).

---

## §S1b — Deps / licences / CVE + cible de purge + impl des 5 gates

### Licences / CVE (vérifiées npm live + arbre installé)

Toutes les nouvelles deps runtime = **MIT/OFL, AGPL-compatibles, 0 CVE bloquante runtime**. Nuances à tracer dans la note licences du scaffold (honnêteté, pas blocage) :
- **`lightningcss` (transitif Tailwind v4) = MPL-2.0** — seule non-MIT/OFL. Build-time only (minifieur CSS natif, jamais dans le bundle navigateur) ; MPL-2.0 = GPL/AGPL-compatible (Secondary Licenses §3.3) ; **déjà présent** via le `tailwindcss@4` actuel → pas une nouvelle exposition. Non-bloquant.
- **`npm audit` arbre actuel = 4 vulns (2 high / 1 mod / 1 low), TOUTES build-time** :
  - `vite 8.0.0–8.0.15` (high) → `npm audit fix` → ≥8.0.16 au scaffold.
  - `js-yaml` (mod) + **`hono ≤4.12.24` (5× high)** → provenance dure : `shadcn@4.8.0 → @modelcontextprotocol/sdk → @hono/node-server → hono`. **Preuve que `shadcn` ne doit PAS être une dependency runtime** : le sortir en devDep/`npx` (D3) **élimine ces 5 high du périmètre runtime**.

### Cible de PURGE (depuis `tools/factory-operator/package.json` racine — lignes exactes)

**`dependencies` à RETIRER (runtime)** :
- 6× `@radix-ui/*` (`:18-23`) : `react-dialog`, `react-dropdown-menu`, `react-scroll-area`, `react-select`, `react-tabs`, `react-tooltip` → consolidés dans `@base-ui/react`.
- `shadcn ^4.8.0` (`:36`) → **déplacer en devDep épinglée ou `npx`** (build-time only ; tire hono/MCP).
- `tw-animate-css ^1.4.0` (`:39`) → redondant avec Motion (blueprint §6.4). RETIRER.
- `i18next ^26.2.0` (`:30`) + `react-i18next ^17.0.8` (`:34`) → hors MVP mono-locale FR. RETIRER.
- `react-router-dom ^7.14.0` (`:35`) → hors MVP (altitude en store Zustand). RETIRER.

**À TRANCHER en D2 (pas un retrait automatique)** : `class-variance-authority ^0.7.1` (`:28`, Apache-2.0) + `clsx ^2.1.1` (`:29`) + `tailwind-merge ^3.6.0` (`:37`). Le blueprint §6.2/§6.5 les conserve. Si **D2 = wrappers Base UI écrits-main** (défaut recommandé), `cva` devient probablement superflu → purger `cva`, **garder `clsx` + `tailwind-merge`**.

**À CONSERVER** : `@base-ui/react` (pin exact 1.6.0), `@fontsource-variable/geist`, `tailwindcss`+`@tailwindcss/vite` (4.3.1), `@xterm/*` (terminal J12), `lucide-react ^1.16.0` (ISC ; vérifier compat React 19 au code), `react`/`react-dom`.

**À AJOUTER** : `motion ^12.42.0`, `@fontsource-variable/geist-mono ^5.2.8` (runtime) ; devDeps `@playwright/test`, `size-limit`+`@size-limit/file`, `babel-plugin-react-compiler`+`@rolldown/plugin-babel` (`eslint-plugin-react-hooks ^7` déjà présent porte les règles compiler).

**Anti-typosquat (adversarial) PASSÉ** : `@base-ui/react` installé = authentique MUI Base UI. Pas de squat.

### Impl recommandée des 5 gates BLOQUANTS

1. **« 0 `@radix-ui` ne survit au runtime »** — 2 couches déterministes : (a) ESLint flat `no-restricted-imports` patterns `@radix-ui/*` sur `src/**` ; (b) assertion shell `npm ls --omit=dev --all` ne contient **aucun** `@radix-ui` (échoue sinon) + assert absence des clés `@radix-ui/*` dans `dependencies`. **Couvrir package.json + arbre prod**, pas seulement un grep `src/` (le front actuel n'importe DÉJÀ aucun radix → la valeur du gate est **anti-drift** : attraper une réintroduction par shadcn-`add` qui émet du Radix par registry tiers). Script `scripts/check-no-radix-runtime.sh`.
2. **anti-`tailwind.config.{js,ts,cjs,mjs}`-v3** — TRIVIAL : `for f in tailwind.config.*; do [ -e "$f" ] && exit 1; done` (v4 = CSS-first ; interdit aussi `@config`).
3. **`size-limit` chiffré** — `.size-limit.json` + `npm run size`. Isoler l'import Motion (`{LazyMotion, domAnimation}` + `motion/react-m`) dans **sa propre entrée** sinon la croissance app masque une régression Motion. **Mesurer puis figer**, ne pas asserter 4,6 kb à l'aveugle.
4. **anti-`motion.*`-nu** — `eslint.config.js`, 2 règles : `no-restricted-imports` (bannir l'export nommé `motion` de `motion/react`) + `no-restricted-syntax` (`JSXMemberExpression[object.name='motion']`). **Allowlister** `m` (de `motion/react-m`), `LazyMotion`, `MotionConfig`, `AnimatePresence`, `domAnimation`. Doublé par `<LazyMotion strict>` runtime.
5. **scan front anti-PASS** — jumeau bash de `web/scripts/scan-en-strings.sh` → `scripts/scan-front-discipline.sh` (D5). Grep `\b(PASS|Vérifié|Approuvé)\b` dans `src/**/*.{ts,tsx}`, `exit 1` sur match. **Piège load-bearing** : « PASS » légitime dans la restitution (`audit.verdict === 'PASS'`) + le fichier de la constante-miroir ÉTAT → (a) exclure le seul fichier ÉTAT ; (b) cibler le texte JSX visible, autoriser `=== 'PASS'` ; (c) filtrer commentaires ; (d) documenter l'allowlist. Étendu Phase I au scan anti-score/jauge.

---

## §S2 — Décisions historiques traversées (DESIGN-CONFLICT detector) — RAS

**Day-0 D1..D11 vs livrables Phase B — chaque item honoré** : D1 React 19 + Compiler (Solid 2.0 = Beta, 3 conditions de réouverture non remplies) ; D2/D3 Base UI seule dep runtime + shadcn build-time + gate (1) + purge ; D4 Motion + gates (3)/(4) ; D5 geist-mono NET-ADD ; D6 Tailwind v4 oklch + gate (2) ; D7 CSP self-origin déjà posée Phase A ; D9 Factory hors daemon ; D10 Geist vendoré 0 CDN ; D1 greenfield jeter `factory-operator`+`factory-ui`. **0 contradiction.**

**Jettison `tools/factory-ui` VRAIMENT orphelin — VÉRIFIÉ (3 scans)** : référencé QUE dans des `.md` + son propre `package.json` — **0 import source hors de son répertoire**. Supersede S70 (`CLAUDE.md:495-496`) tracé kickoff Arbitrage PO #2 + Day-0 #1. Fondation re-planifiée S81. **Preset à jeter = bon artefact** : `tools/factory-operator/src/index.css:1-57` = preset hex GitHub-dark NON-oklch → tout JETÉ.

**Delta couverture (S5) — acté avec nuance d'honnêteté Phase I** : perte Vitest **à 100 % de `factory-operator`** (2 fichiers, ~7-8 tests, dont `executionChat.test.ts` PO-14 single-Done) ; **`factory-ui` = 0 test perdu** (source-coverage uniquement). Re-couverture via `useTokenStream` Phase I conforme. Tracer au body commit B, solder I/J.

**Aucun DESIGN-CONFLICT.**

---

## §S3 — Threat / supply-chain

Phase B = scaffold front + gates, **0 route backend, 0 wire, 0 crypto**. La sécurité = les 5 gates + consommation correcte de la CSP Phase A.

### CSP self-origin Phase A — valeur EXACTE (`operator_server.rs:348-360`)

```
Content-Security-Policy: default-src 'self'; connect-src 'self'
X-Content-Type-Options: nosniff
```
Pas de `style-src`/`script-src`/`'unsafe-inline'`/`'unsafe-eval'`/nonce/`data:`. Header statique sur 200/303/401/403/404. Cookie `sbfb_operator` (secret distinct du bearer, `auth.rs:55,76-78`), header `x-sbfb-token` d'abord puis cookie (`:299-301`). `ServeDir` enraciné sur bundle dédié (`OPERATOR_BUNDLE_SUBDIR`, `:40-47`). **Phase A landée → préalable BLOQUANT levé.** Commentaire code : « the greenfield front must ship without inline scripts ».

### Compatibilité bundle Vite ↔ CSP — inchangée, à 2 conditions de config

| Surface | Verdict | Mitigation |
|---|---|---|
| Entry JS / Tailwind CSS / Geist woff2 / React `style={{}}` / SSE / WS | ✅ | externes même-origine ou CSSOM ; WS à asserter T1 |
| **modulepreload polyfill** | ⚠️→OK | script inline → **`build.modulePreload.polyfill = false`** |
| **assets < 4 kb** | ⚠️→OK | `data:` URI → **`build.assetsInlineLimit = 0`** |
| **Motion `m`+`LazyMotion`** | ⚠️ RÉSIDUEL Phase E | WAAPI/CSSOM OK ; si `<style>` injecté → escape `MotionConfig nonce`, **PAS** blanket `unsafe-inline` |

**Risque concret unique** : une dep qui écrit un `<style>` runtime casse le rendu → **valider EMPIRIQUEMENT dès Phase B** via T1 sur le **bundle BUILDÉ** (pas `vite dev`).

> **CSP ENFORCE le vendoring Geist (WIN)** : `default-src 'self'` bloque `fonts.googleapis.com`. La maquette hi-fi charge Geist en CDN (l.12-14) → interdit ET cassé par la CSP → Phase B **doit** vendorer (D5+D10). Contrainte *enforced*, pas seulement disciplinaire.

### Supply chain / pin
Lockfile commité = vrai pin (`@base-ui/react` exact, churn 1.0→1.6 assumé D2) ; shadcn devtool build-time only (génère du Radix → c'est le rôle du gate 1) ; footgun Vite 8/Rolldown #22620 (ne pas introduire de `sideEffects` restrictif) ; pin exact `babel-plugin-react-compiler`.

---

## §S4 — Invariants wire format

**N/A — 0 wire touché (CONFIRMÉ).** Front-only (greenfield `tools/factory-operator/` + suppression dossiers). 0 ligne Rust éditée → 0 canonical, 0 bump. Aucun `FeedEntry`/`ProjectAnnouncement`/`CuratorList`/`Task`/`*_ANNOUNCEMENT_VERSION`/`DOMAIN_*`. Les 2 occurrences `_VERSION` de `nexus-core-rs` (`key_rotation.rs:15`, `schemas/mod.rs:35`) sont des commentaires. 0 route daemon ET 0 route backend en B (ajouts backend = A `a5ace8d`, F `bb35d39`, G/I + 2 triviaux D — pas B). SSE `fetch()`+`ReadableStream` = Phase C.

---

## §Verdict global + justification (§4.5.7)

**PLAN-ADAPT.** Le plan est exécutable sans toucher Day-0 (0 DESIGN-CONFLICT confirmé par S2/S4 + adversariaux #1/#2/#4/#5 CONFIRMED), et les 5 scans signalent EXECUTE. Je rends **PLAN-ADAPT** parce que deux claims load-bearing de la **recette/BASE** sont réfutés avec **correction OSS concrète** — code mort/non-rendu si ignorés :
1. **ADAPT-1 — React Compiler sous Vite 8 / plugin-react v6** : `react({ babel })` mort → `react()` + `@rolldown/plugin-babel` + `reactCompilerPreset()`.
2. **ADAPT-2 — tokens hi-fi dans `@theme`** : tokens terses → 0 utilitaire → re-namespacer `--color-*`/`--font-*`/`--radius-*` (valeurs hi-fi = spec). (Adversarial #3 : sous-claim « portent proprement verbatim » REFUTED.)

Plus ADAPT-3 (import `m` de `motion/react-m`, à câbler dans le gate 4). Aucune n'affecte les livrables ni une décision Day-0 → PLAN-ADAPT (pas DESIGN-CONFLICT).

---

## §Risques top

1. **CSP découverte tard = page blanche prod** (HAUT, mitigé) : invisible en `vite dev` → T1 cible le **bundle BUILDÉ** servi par l'Operator Rust, assert 0 violation CSP (boot+SSE+WS) ; `modulePreload.polyfill:false` + `assetsInlineLimit:0` + `outDir`=`bundle`.
2. **Gate 5 faux-positifs** (MOYEN) : allowlister fichier ÉTAT + `=== 'PASS'`.
3. **Gate 3 mal ciblé** (MOYEN) : entrée size-limit Motion dédiée + mesurer-puis-figer.
4. **shadcn-`add` réintroduit Radix** (MOYEN, rôle du gate 1) : couvrir package.json + `npm ls --omit=dev`.
5. **Régression couverture B→I** (MOYEN) : −7/−8 Vitest (100 % factory-operator) → tracer body B, solder I/J.
6. **Churn Base UI 1.x** (BAS) : pin exact + lockfile gelé.

---

## §Recette d'exécution actionnable (todo-list main-thread)

1. **Jettison** : supprimer `tools/factory-ui/` + `tools/factory-operator/` (tracer supersede `CLAUDE.md:495-496` au body). Recréer `tools/factory-operator/` greenfield.
2. **Scaffold deps** : `package.json` runtime (react 19.2.7, `@base-ui/react` pin 1.6.0, motion 12.42, geist + geist-mono, tailwindcss 4.3.1, clsx, tailwind-merge, lucide-react, @xterm) — SANS 6 radix/`tw-animate-css`/i18next/react-router/cva(si D2 main)/shadcn ; devDeps (vite ≥8.0.16, plugin-react ^6, babel-plugin-react-compiler ^1, @rolldown/plugin-babel, @babel/core, @playwright/test, size-limit+@size-limit/file, eslint+react-hooks ^7, shadcn build-time). `npm install` + committer lockfile + `npm audit fix`.
3. **`vite.config.ts`** : `react()` + `babel({ presets:[reactCompilerPreset()] })` + `tailwindcss()` ; `build:{ outDir:'bundle', modulePreload:{polyfill:false}, assetsInlineLimit:0 }`. (PAS `react({babel})`, PAS de `sideEffects` restrictif.)
4. **`src/index.css`** : `@import "tailwindcss"` + `@custom-variant dark` + `:root` tokens hi-fi oklch + `@theme inline` re-namespacé + import fontsource geist + geist-mono. Dark défaut, focus `--info`, CTA `--tx`, rayons 3-5px.
5. **Squelette app** : entrée React 19 minimale, store/provider stubs, AUCUN `EventSource`, motion non utilisé (Phase E).
6. **Gates (5) + CI BLOQUANT** : `scripts/check-no-radix-runtime.sh`, `scripts/check-no-tailwind-config.sh`, `.size-limit.json`+`npm run size`, `eslint.config.js` (anti-`motion.*`), `scripts/scan-front-discipline.sh`. Câbler les 5 en CI dès cette phase.
7. **Squelette T1 Playwright hermétique** : `vite build` → servir bundle via Operator Rust → assert 0 violation CSP en console. En CI chaque push.
8. **Note licences** : MIT/OFL + lightningcss MPL build-time + shadcn→devDep.
9. **Vérif finale** : lint + `tsc --noEmit` + 5 gates verts + `vite build` + `npm run size` + T1 vert. Body : delta tests (−7/−8 Vitest 100 % factory-operator, re-couvert I), supersede S70 tracé, scope cuts (motion=E, SSE=C).

---

## §Questions ouvertes tranchées (défauts recommandés du kickoff)

- **D2 — wrappers vs shadcn-gen** : **défaut = wrappers main** (supersede S70 libère l'héritage ; shadcn build-time only en filet) → purger `cva`, garder `clsx`+`tailwind-merge`. Package = **`@base-ui/react`** pin exact 1.6.0.
- **D5 — geist-mono** : **OUI**, `^5.2.8` (≠ 5.2.9 du sans). Script gate 5 = **`scripts/scan-front-discipline.sh`**.
- **D1 — React Compiler** : **ON**, GA v1.0, pin exact ; `reactCompilerPreset()` + `@rolldown/plugin-babel` (PAS `react({babel})`). Vitest sans transform Babel → `"use no memo"` seulement sur fixtures qui mutent.
- **D4 — cookie** : **session-only** (HttpOnly, secret distinct, posé Phase A `auth.rs:55`) ; consommation cookie + header `x-sbfb-token` d'abord ; bootstrap `GET /?token` + ServeDir.

Fichier écrit : `C:\Users\FlowUP\Documents\Code\nexus\.planning\active\sprint80_phase_b_preflight.md`

SIGNAL: PLAN-ADAPT