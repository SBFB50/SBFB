# Sprint 80 — Phase E — Preflight (deep, 5 scans + 3 adversariaux)

**Date** : 2026-06-28
**Phase** : E — design-system oklch + 5 signatures de motion sens-porteuses dans `tools/factory-operator/` (front-only, **backend AUCUN**). Motion 12.42.0 (déjà au lock, 0 import dans `src/` aujourd'hui) ; Base UI re-thémé oklch ; View-Transitions natives pour l'altitude shift.
**Verdict** : **PLAN-ADAPT**

> Le plan §Phase E (L159-170) est intégralement aligné Day-0 (D3/D4/D5/D6/D8) — aucune décision figée n'est contredite — mais **trois corrections load-bearing imposées par le code réel de Motion + la CSP réelle de l'Operator** doivent être figées avant la première ligne, sans toucher un invariant Day-0 : (1) **budget** — `domAnimation` pèse **66 KB RAW** (mesure rolldown) ; placé au hero il pulvérise la marge ; il faut une architecture motion confinée aux chunks async + `manualChunk vendor-motion` + entrées `.size-limit.json` (miroir xterm Phase D), et le loader paresseux DOIT passer par un ré-export LOCAL (eslint bannit `import('motion/react')`) ; (2) **anti-déco** — `reducedMotion='user'` n'instantanéise QUE les clés transform/positionnelles, JAMAIS `opacity`/`color`/`backgroundColor` ; chaque signature doit donc s'**ancrer sur un transform** + garde `useReducedMotion()` pour le non-transform, sinon l'assertion T1 « état final instantané » échoue ; (3) **CSP** — l'altitude shift doit utiliser `document.startViewTransition` NATIF + règles `::view-transition-*` dans le CSS bundlé, JAMAIS le wrapper `animateView()` de Motion (injecte un `<style>` sans nonce, bloqué par `default-src 'self'`).

> Les **trois claims adversariaux sont tous REFUTED** sur des composantes load-bearing (budget hero, mécanisme « amender l'allowlist », réduction de mouvement + CSP wrapper). Par la règle §4.5.7 (un claim refuté qui impacte le verdict force ≥ PLAN-ADAPT), le plan littéral ne peut pas s'exécuter tel quel → **PLAN-ADAPT**. Aucun refus ne porte sur une **contradiction Day-0 irréductible** (toutes les corrections sont des raffinements d'implémentation qui PRÉSERVENT les décisions figées : `motion/react` unique, `reducedMotion='user'` global, View-Transitions pour l'altitude, rail exclu, oklch) → **pas de DESIGN-CONFLICT**.

> Orchestration : 5 scans (S1a prior-art OSS profond / S1b deps-CVE-budget / S2 décisions historiques + gates / S3 threat model / S4 wire) + 3 adversariaux. Faits load-bearing **re-vérifiés en main-thread** : `.size-limit.json` mesure le **RAW** (`gzip:false`, `brotli:false`, app `40 KB`) ; `eslint.config.js:75` bannit `ImportExpression[source.value='motion/react']` (le loader lazy `() => import('motion/react')` proposé par les scans serait BLOQUÉ → ré-export local obligatoire) ; commentaire eslint L9-17 PRÉ-ARME l'entrypoint `m`/LazyMotion/MotionConfig/AnimatePresence/domAnimation (n'interdit que `motion` plein + `<motion.*>` + dynamic `motion/react`) ; CSP `default-src 'self'; connect-src 'self'` sans `'unsafe-inline'` (operator_server.rs:354).

---

## 1. Synthèse des 5 scans

| Scan | Objet | Verdict | Apport load-bearing |
|---|---|---|---|
| **S1a** Prior-art OSS profond | 5 signatures réalisables sur l'API réelle ? | EXECUTE + 2 correctifs | Code réel `motion-dom` : `reducedMotion='user'` ⇒ `{type:false}` SEULEMENT pour `positionalKeys` (width/height/top/left/right/bottom + transform) ; `opacity`/`color`/`backgroundColor` **tweenent quand même** (visual-element-target.mjs:84-87 + keys-position.mjs:3-11). `animateView()` injecte un `<style>` sans nonce (view/utils/css.mjs) → bloqué CSP. **VT native CSP-safe**. Budget `m`+LazyMotion ≈ 4.6 kb GZIP, mais `domAnimation`=66 KB RAW. `domAnimation` n'inclut PAS la prop `layout` (réservée à `domMax`) → AnimatePresence + keyframes transform, jamais `layout`. Base UI = headless, oklch est l'état naturel. |
| **S1b** Deps / CVE / budget | Coût bundle + sûreté deps | EXECUTE + concern budget | **0 dépendance neuve** (motion/tailwind/lucide/base-ui/geist déjà au lock). **0 CVE** (`npm audit --omit=dev` = 0 ; CVE-2025-55182 = RSC, N/A SPA client). `.size-limit.json` mesure le **RAW**. Mesures rolldown : infra LazyMotion+MotionConfig=2154 B, AnimatePresence=4552 B, **domAnimation=66148 B**, `m.div/button/span` nommés=16614 B, `import * as m`=~78 KB. **Angle mort gate** : un chunk auto-généré (ni `index-*`, ni `vendor-react-*`, ni `vendor-xterm-*`) n'est PAS mesuré par les globs. |
| **S2** Décisions / gates | Plan §E ↔ Day-0 + 5 gates | aligné (designConflictRisk=FALSE) | Gate eslint anti-motion **pré-armé** (n'a PAS à être amendé ; l'amender pour `motion` plein = signal d'écart). **Le vrai verrou = size-limit** : motion non chunkée (`vite.config.ts` ne route que react+xterm) → tomberait au hero. **L'allowlist des 5 sigs n'est mécanisée par AUCUN script** (doctrinale, vérifiée review/Codex). Tokens oklch DÉJÀ posés Phase B (ADAPT-2, hue 260) → E re-thème, ne redéfinit pas. hi-fi : 2/5 sigs annotées (altitude shift L256, gravity L469) → 3 à concevoir (token settle, gate flip, reveal). |
| **S3** Threat model | CSP / a11y / surface réseau | EXECUTE | **Aucun assouplissement CSP** : Motion anime via CSSOM `element.style` (hors style-src) + WAAPI `element.animate()` (hors CSP) + VT native (pseudo-éléments UA). Scan `node_modules/motion/dist` = **0 `eval`/`new Function`/`insertRule`/`createElement('style')`** au cœur. **0 surface réseau neuve, 0 wire/route.** **Aucun amendement THREAT_MODEL.** Caveat a11y : la VT native n'est PAS gatée par `MotionConfig` → garde `prefers-reduced-motion` propre à la signature (4). |
| **S4** Wire / API / routes | Format / canonical / routes | EXECUTE (front-only) | **0 wire / 0 API / 0 route / 0 canonical / 0 `*_VERSION`.** Les 2 hits `_VERSION` dans `nexus-core-rs` sont inertes (commentaires). La « gate flip » consomme le `GET /api/gates` **déjà livré Phase G** (`ed00b4a`, read-only/idempotent) — 0 champ neuf. Worktree propre. |

---

## 2. Verdicts adversariaux intégrés

| Claim | Verdict | Composante refutée | mustFix intégré |
|---|---|---|---|
| « 0 dep neuve **ET** importer motion tient dans size-limit (~36/40 KB) » | **REFUTED** | Conjoint 1 VRAI ; conjoint 2 FAUX : LazyMotion+MotionConfig+domAnimation **au hero = ~104 KB RAW** (builds empiriques), bust de ~64-68 KB. Ne tient QUE sous confinement async **non-énoncé** + le plan exige un `MotionConfig` GLOBAL au root. | **BLOQUANT** : architecture motion confinée (cf. §3.1) + `MotionConfig` au root sans les features lourdes. |
| « Les 5 sigs traversent les 3 verrous **en amendant l'allowlist, sans assouplir un gate** » | **REFUTED** | Mécanisme inversé : (a) l'allowlist eslint est pré-armée — l'amender = assouplir ; (b) l'« allowlist 5 sigs » n'est mécanisée par aucun script (rien à amender). + un agent mesure le gate size **déjà rouge** (cf. §5 discordance à lever). | **BLOQUANT** : NE PAS amender eslint pour `motion`/dynamic-import ; encoder l'allowlist en **constantes nommées** (doctrine, pas gate). |
| « Motion+VT CSP-safe **ET** dégradent à un état final instantané sous reduced-motion » | **REFUTED** | Conjoint B FAUX : `opacity`/`color` tweenent sous `reducedMotion='user'` ; VT native non auto-gatée. Conjoint A vrai SOUS CONDITION : `animateView()` injecte `<style>` sans nonce (bloqué) → VT native obligatoire. | **BLOQUANT** : ancrage transform + garde `useReducedMotion()` (cf. §3.2) ; VT **native** + CSS bundlé, jamais `animateView()` (cf. §3.3). |

Les 3 claims refutés impactent le verdict (§4.5.7) → verdict global ≥ PLAN-ADAPT. Aucun ne porte sur une contradiction Day-0 irréductible (corrections = raffinements préservant les décisions figées) → verdict global ≠ DESIGN-CONFLICT.

---

## 3. Décisions load-bearing FIGÉES

### 3.1 Architecture motion sous budget RAW (verrou size-limit)
**Baseline retenue** : hero `index-*.js` ≈ **35.96 KB RAW** / 40 KB (Phase D committé vert `152df25`, cohérent avec `.size-limit.json` gzip:false et la mesure S1b/claim-1) → **marge RAW ≈ 4 KB**. *(La mesure divergente d'un adversaire — 104 KB RAW à zéro motion, gate déjà rouge — est un constat de build-state à **lever empiriquement au 1er build de la phase** ; cf. §5.1 ; l'architecture ci-dessous tient sous les deux lectures.)*

Règles figées :
1. **Root (hero)** : `<MotionConfig reducedMotion="user">` (global, honore plan L167) + `<LazyMotion strict features={lazyDomAnimation}>`. Coût hero ≈ 2.1 KB RAW (infra seule). **Aucune feature lourde au hero.**
2. **`domAnimation` (66 KB RAW)** chargé **paresseux** — MAIS via un **ré-export LOCAL** : `lib/motion-features.ts` fait `export { domAnimation } from 'motion/react'` (import statique), et le loader est `() => import('./motion-features').then((m) => m.domAnimation)`. **Raison dure** : `eslint.config.js:75` bannit `ImportExpression[source.value='motion/react']` ; un `() => import('motion/react')` direct (proposé par les scans) serait **rejeté par le gate**. L'import dynamique d'un **chemin local** n'est pas banni.
3. **`m.*` + `AnimatePresence`** : imports **nommés** (`import { m } …` / `motion/react-m` nommés — JAMAIS `import * as m` qui pull ~78 KB) confinés dans les **surfaces déjà code-splittées** (Procédé/Sessions/Knowledge/Verify, React.lazy Phase D) → atterrissent dans les chunks async de surface, jamais au hero. **Prop `layout` INTERDITE** (réservée `domMax`, non couverte par `domAnimation`).
4. **`vite.config.ts`** : ajouter un `manualChunk` **`vendor-motion`** (miroir exact du pattern `vendor-xterm` Phase D) pour router `motion`/`motion-dom`/`framer-motion`, **+ entrée(s) `.size-limit.json` chiffrée(s)** pour ce chunk ET pour les chunks de surface qui embarquent `m.*` — **ferme l'angle mort des globs** (un chunk non couvert n'est pas mesuré). Aucun budget existant relâché.
5. **eslint** : NE PAS amender l'allowlist (pré-armée). **Corriger le commentaire L11-17** : refléter le loader lazy par ré-export local et clarifier l'unité (le commentaire cite « ~4.6 kb » GZIP alors que le gate mesure le RAW).

### 3.2 Anti-déco : `reducedMotion='user'` ne suffit pas (verrou T1)
**Fait** : `reducedMotion='user'` ⇒ `{type:false}` UNIQUEMENT pour `positionalKeys` (transform + width/height/top/left/right/bottom). `opacity`/`color`/`backgroundColor`/`filter` continuent de tweener (visual-element-target.mjs:84-87). Une « verification reveal » en fade d'opacité pur **échouerait** l'assertion T1 « état final instantané ».

Règles figées (chaque signature) :
1. **Ancrer sur un transform** (`y`/`scale`/`rotate`) pour bénéficier de l'instantané natif sous reduced-motion.
2. Pour toute valeur **non-transform** (opacity/color) : garde `const reduce = useReducedMotion()` → rendre l'état final / `transition={{duration:0}}` quand `reduce`.
3. **Belt-and-braces CSS** dans `index.css` : `@media (prefers-reduced-motion: reduce){ *,*::before,*::after{ animation-duration:.01ms!important; transition-duration:.01ms!important } }`.
4. **T1 Playwright** : `page.emulateMedia({ reducedMotion: 'reduce' })` puis asserter l'état final instantané — **l'assertion DOIT couvrir explicitement la signature (4) altitude shift** (VT native, non gatée par MotionConfig), pas seulement les signatures pilotées par `m`. (Ne pas utiliser `MotionConfig skipAnimations` : skip global inconditionnel ≠ préférence utilisateur.)

### 3.3 Altitude shift (signature 4) : VT NATIVE obligatoire (verrou CSP)
**Fait** : `default-src 'self'; connect-src 'self'` sans `'unsafe-inline'` (operator_server.rs:354). Le wrapper `animateView()`/`ViewTransitionBuilder` de Motion injecte `<style id="motion-view">` runtime **sans nonce** (view/utils/css.mjs) → **bloqué**. `document.startViewTransition` natif n'injecte aucun markup ; les pseudo-éléments `::view-transition-old/new/group(<name>)` se stylent par CSS **bundlé** (même-origine, OK sous `default-src 'self'`) et `element.style.viewTransitionName` (CSSOM) n'est pas soumis à style-src.

Règles figées :
1. **`document.startViewTransition` NATIF** + règles `::view-transition-*` dans `index.css` (bundlé). **JAMAIS `animateView()` de Motion.**
2. Intégration React 19 : `document.startViewTransition(() => flushSync(() => setFocus('verify')))` (`flushSync` de `react-dom` force l'application synchrone du DOM dans la capture). API impérative native, **pas** le `<ViewTransition>` expérimental React.
3. **Garde reduced-motion + feature-detect** (la VT n'est pas auto-gatée) : `reduce || !document.startViewTransition ? setFocus(x) : document.startViewTransition(() => flushSync(() => setFocus(x)))`.
4. **Rail EXCLU** : `view-transition-name: none` sur le rail (ne pas le nommer) → il reste fixe pendant la bascule STEER↔VERIFY (Day-0 D8 + hi-fi L256 + plan §Phase H L213).

### 3.4 Patterns ancrés des 5 signatures (sens, jamais déco)
- **(1) token settle** — `m.span` clé=valeur sous AnimatePresence OU keyframe `animate={{ y:[6,0] }}` spring bas-rebond (`{type:'spring',visualDuration:0.22,bounce:0.18}`) ; **`tabular-nums`** (font-variant-numeric) sur le conteneur chiffré (0 reflow de glyphes). `y` positionnel ⇒ instant reduced-motion.
- **(2) gate flip** — `<AnimatePresence mode='wait'>` autour d'un `m.div` clé=`gateStatus` (la **constante miroir** `GateStatus` 5 valeurs snake_case de `GET /api/gates`, Phase G) avec exit/enter `rotateX`/`scale` (transform ⇒ instant). Le verdict est **RESTITUÉ** (clé pilotée par l'API), le flip n'en CALCULE aucun ; couleur = sens (tokens `--color-ok/warn/bad`), jamais un faux `PASS` fabriqué (gate `scan-front-discipline.sh`).
- **(3) verification reveal** — variants parent `staggerChildren` + enfants `{ opacity, y }` (le `y` porte l'instant ; opacity gatée `useReducedMotion()`). AnimatePresence sur la liste, **pas de prop `layout`**.
- **(4) altitude shift** — VT native (§3.3).
- **(5) confirmation gravity** — MUR gouvernance plein-largeur (amber `--color-mur`) : `m.div` entrée `{ y, scale }` spring « lourd » (`{type:'spring',mass:1.2,stiffness:140,damping:26}`) — la physique = gravité = SENS de la conséquence (0-Forcer/Override/Bypass). Transform ⇒ instant reduced-motion.

### 3.5 Design-system oklch + dualité Geist (déjà 80% posé Phase B)
Tokens oklch **déjà complets** (index.css, hue 260, ADAPT-2 Phase B) — E **ne redéfinit pas** (sinon drift). Travail réel : (a) **re-thémer chaque composant Base UI** par data-attributes/parts sur les tokens (état NATUREL de Base UI headless, jamais un preset) ; (b) **`tabular-nums`** sur les compteurs ; (c) **dualité Geist sans/mono** = langage preuve-vs-intention (familles déjà vendored `--font-sans`/`--font-mono`, 0 dep neuve) ; (d) les 5 signatures. **Marge CSS = 706 B seulement** (index-*.css 19.29/20 KB) → surveiller au build ; si dépassement, factoriser les tokens avant tout bump argumenté.

---

## 4. Approche d'implémentation

1. **Provider racine** (`App.tsx`) : `<MotionConfig reducedMotion="user"><LazyMotion strict features={lazyDomAnimation}>…</LazyMotion></MotionConfig>` + `lib/motion-features.ts` (ré-export local de `domAnimation`).
2. **`vite.config.ts`** : `manualChunks` → `vendor-motion` (motion/motion-dom/framer-motion), miroir `vendor-xterm`. **`.size-limit.json`** : entrées chiffrées `vendor-motion` + chunks de surface portant `m.*`.
3. **Constantes nommées motion** (README §6.9 / doctrine constantes-miroir) : un module `lib/motion.ts` centralisant durées/easings/springs ET l'**énumération des 5 signatures** (allowlist lisible & review-able, puisque non mécanisée par script). Réutilisées partout (pas de magic number).
4. **5 signatures** : §3.4, ancrées transform + gardes reduced-motion (§3.2) ; altitude shift VT native (§3.3).
5. **Re-thémage Base UI oklch** + `tabular-nums` + dualité Geist (§3.5).
6. **eslint** : commentaire L11-17 corrigé (loader lazy local + unité RAW). **0 amendement d'allowlist.**
7. **T1** : `page.emulateMedia({reducedMotion:'reduce'})` ; assertion état-final-instantané couvrant **les 5 signatures dont (4)**.
8. **Backend = AUCUN** (front-only). 0 wire/route/canonical (S4). La « gate flip » consomme `GET /api/gates` existant (Phase G), read-only, sans champ neuf.

---

## 5. Risques résiduels / cibles adversariales

1. **Discordance de mesure du baseline bundle** — claim-1 mesure index ≈ **35.97 KB RAW** (gate vert, ~4 KB marge, cohérent Phase D committé) ; claim-2 mesure ≈ **104.16 KB RAW** (gate déjà rouge, `npm run size` exit 1). À **lever empiriquement au 1er build de la phase** (`npm run build` puis `ls -l bundle/assets` + `npx size-limit`) AVANT de coder la motion. Si 104 KB se confirme, il y a une **régression de chunking préexistante** (react non splitté / globs désalignés) à traiter en amont — l'architecture §3.1 reste néanmoins la bonne (motion lourde hors hero + chunks gatés).
2. **Angle mort des globs size-limit** — un chunk motion auto-généré non couvert par `index-*`/`vendor-react-*`/`vendor-xterm-*` ne serait PAS mesuré → motion lourde « invisible » au gate. **Mitigation figée** : `manualChunk vendor-motion` nommé + entrées `.size-limit.json` explicites (§3.1.4).
3. **Loader lazy bloqué par eslint** — `() => import('motion/react')` est banni (L75) ; **ré-export local obligatoire** (§3.1.2). À ne pas oublier sinon le lint casse.
4. **Fade pur sous reduced-motion** — réflexe naturel d'une reveal en opacité = échec T1 ; ancrage transform + garde `useReducedMotion()` (§3.2).
5. **VT via `animateView()` de Motion** — bloquée CSP ; **VT native uniquement** (§3.3). VT non auto-gatée reduced-motion → garde explicite.
6. **Faux verdict via gate flip** — animer la transition d'état mais NE JAMAIS fabriquer un `PASS`/point-vert sans verdict restitué backend ; couleur = sens (gate `scan-front-discipline.sh`).
7. **Marge CSS 706 B** — re-thémage Base UI + keyframes peut la grignoter ; factoriser avant bump.
8. **`import * as m`** (~78 KB) — interdit ; imports nommés confinés async (§3.1.3).

---

## 6. Scope

**DANS le scope S80 Phase E** :
- Provider `MotionConfig reducedMotion="user"` global + `LazyMotion strict` features lazy via ré-export local + `vendor-motion` chunk + entrées `.size-limit.json`.
- Les 5 signatures (token settle, gate flip, verification reveal, altitude shift VT-native, confirmation gravity), ancrées transform + gardes reduced-motion, paramétrées par constantes nommées.
- Re-thémage Base UI sur tokens oklch (data-attributes/parts), `tabular-nums` sur les compteurs, dualité Geist sans/mono.
- Belt-and-braces CSS `@media (prefers-reduced-motion: reduce)`.
- Correction du commentaire eslint (loader lazy local + unité RAW). **0 amendement d'allowlist.**
- T1 Playwright `reducedMotion:'reduce'` couvrant les 5 signatures dont l'altitude shift.

**HORS scope (scope cuts confirmés)** :
- **View-Transitions du RAIL** — exclu (Day-0 D8 + hi-fi L256 + Phase H L213) : `view-transition-name:none`.
- **Onglets « Aperçu scellé » + « Preuve »** — différés **S81** (Phase H L226 : les coder rouvrirait le P1 app-authoring in-vivo).
- **V5 (pouls-gate gouttière `fichier:ligne`) / V6 (filtre par gate) ligne-fine** — **carry P1 S81** (Décision C Phase G : `LintDiagnostic` sans champ ligne ; rattachement gate↔fichier au niveau fichier seulement).
- **`domMax` / prop `layout` / shared-layout** — exclu (`domAnimation` only ; les morphs passent par AnimatePresence + transform ou VT native).
- **Wrapper `animateView()` / `<ViewTransition>` React expérimental** — exclu (CSP + API native impérative).
- **Backend** — AUCUN (front-only) ; 0 wire/route/canonical/`*_VERSION` ; 0 dépendance neuve ; 0 amendement THREAT_MODEL.
- **Câblage front de `GET /api/gates`** (client `getGates` + panneau gates riche) — Phase H (la « gate flip » de E est une primitive de motion réutilisable, pas le panneau).
