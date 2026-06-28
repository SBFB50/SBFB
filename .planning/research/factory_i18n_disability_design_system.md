> Statut : design doc — note de conception hors-sprint (2026-06-28). Produit par un
> Workflow ultracode (10 agents Opus 4.8 1M, ~986k tokens, 183 outils) sur directive PO
> « traduction pour toutes les langues + design system travaillé pour tous les handicaps ».
> Décisions PO fermes : (1) doc de conception d'abord ; (2) i18n LARGE + RTL complet
> (FR/EN/ES/AR/ZH…) ; (3) handicaps poussés WCAG 2.2 AAA + COGA cognitif (au-delà de AA).
> Ancré dans le code réel de `tools/factory-operator`. La vérification adversariale (Annexe A)
> FAIT AUTORITÉ en cas de conflit avec les sections de synthèse. Déjà livré et conservé comme
> fondation : U1 (tokens contraste AA + échelle typo tokenisée) + L3 (sémantique HTML
> h1/h2/`<main>`/skip-link). Lots a11y L4-L8 en pause, à re-séquencer derrière l'extraction i18n.

# Design doc — i18n multi-langues + design system handicaps (AAA/COGA) — Factory Operator


## Résumé exécutif

Ce document de conception cadre deux chantiers transverses pour le front **factory-operator** (React 19 + Vite + Tailwind v4 CSS-first, CSP `self`) : **(P1) i18n large + RTL complet** (FR/EN/ES/AR/ZH…, incl. langues RTL) et **(P2) design system handicaps poussé au-delà de WCAG AA** (cibles AAA 2.2 + COGA cognitif). Les deux partagent une même fondation : une couche de **custom properties surchargées par `data-*` sur `<html>`** et un **catalogue de messages maison**.

**Mécanisme i18n retenu : catalogue maison 0-dep bâti sur `Intl.*`** (`PluralRules`/`NumberFormat`/`DateTimeFormat`/`RelativeTimeFormat`/`ListFormat`), `t()` + un composant `<Trans>` rich-text. Rationale durci par la vérif : (1) `.size-limit.json` mesure en **octets RAW** (`gzip:false`), budget hero `app=45 kB` avec **~1-5 kB de marge seulement** → react-intl (58 kB RAW) et i18next (65-77 kB RAW) **éliminés**, Lingui (~10 kB) recevable, maison **3-5 kB** (et non « 2 kB » : le `<Trans>` rich-text est un coût réel) ; (2) CSP `default-src 'self'` sans `unsafe-eval` → `Intl` natif = zéro surface eval à auditer à chaque bump.

**Handicaps visés : AA toujours-actif par défaut + AAA/COGA en modes opt-in** (`data-contrast=high` 7:1, `data-pointer=large` 44px, `data-text-spacing=loose`, `data-font=legible`, `data-motion=reduced`, `data-scale`, `data-theme=light`).

**Déjà livré (ne pas refaire) : U1** (tokens contraste WCAG AA + échelle typo `text-scene/card/body/sec/meta` + `.eyebrow`) et **L3** (sémantique HTML h1/h2/`main`/skip-link). reduced-motion OS-media + triple-filet JS/CSS/View-Transition déjà en place. **En cours : L4-L8 + glossaire L6** — à re-séquencer derrière l'extraction i18n.

**5 tensions majeures (la verif adversariale fait autorité) :**
1. **1.4.10 Reflow (critère AA, pas AAA) est EN ÉCHEC aujourd'hui** (rail fixe `w-[158px]` + `App.tsx:48` `overflow-hidden` → clipping 320px/400 %). Aucune revendication AAA n'est crédible tant que ce **AA** échoue → **rail collapsible = bloqueur #1, avant tout mode opt-in**.
2. **Budget RAW + provider eager** : maison-Intl (3-5 kB) ou Lingui (~10 kB) seulement ; même le maison exige un **bump de `app`** ou un chunk `vendor-i18n` dédié (catalogues lazy par locale dans tous les cas).
3. **Anti-FOUC = `/preboot.js` externe same-origin OBLIGATOIRE**, pas `useLayoutEffect` (inline interdit par CSP ; useLayoutEffect peint trop tard → flash visible sur cold-start).
4. **Polices : CJK = système obligatoire** (vendoring 1-3 MB exclu par CSP-self), fallbacks **nommés par script** (pas `system-ui` seul) ; arabe/hébreu système suffit (vendoring ~50-130 kB lazy = cosmétique). Les **woff2 échappent à size-limit** → gate payload-font si vendoring.
5. **L'anti-PASS et le FR-only fuient par la traduction** : le gate doit muter en « zéro littéral hors `t()` » (couvrant `aria-label`/`title`/`placeholder`/`sr-only`) **+** scan anti-verdict sur **les valeurs de TOUTES les locales**. Et **les ratios de contraste cités (U1 + HC) sont calculés à la main → provisoires** jusqu'au gate oklch→WCAG automatisé.

---

## 1. Pilier A — Internationalisation (i18n)

### 1.1 Surface, inventaire des chaînes et taxonomie

**Cadre vérifié (greenfield total).** Aucune lib i18n dans `tools/factory-operator/package.json` (deps runtime = Base UI 1.6.0, `@fontsource-variable/geist`+`geist-mono`, `@xterm/xterm` 6.0.0, `clsx`, `lucide-react`, `motion`, React 19 — vérifié lignes 21-34). `index.html:2` = `<html lang="fr" data-theme="dark">` : **`lang` figé en dur, attribut `dir` absent**, aucune bascule runtime `documentElement.lang/dir`. Un seul formatage locale-aware dans tout `src/` : `lib/sessionDate.ts:12` `d.toLocaleString('fr-FR')` — **locale codée en dur**. Toute la pluralisation est manuelle (suffixe `s` ternaire). Zéro usage `Intl.*` ailleurs.

**Volume (estimation d'inventaire, non exhaustif-machine).** ~**290-320 occurrences** user-facing sur **22 composants + 4 fichiers catalog/lib** ; après dédup des familles répétées, ~**240-260 messages distincts**. Densité très inégale : `ProcedeSurface.tsx` ≈ 60-70 chaînes (~40 % du volume), puis `Atelier.tsx`/`OrientationBar.tsx`/`VerifyScene.tsx`/`DiffViewer.tsx`/`SessionsSurface.tsx` ≈ 16-22 chacun. (Ces compteurs sont des ordres de grandeur d'inventaire à figer par un scan d'extraction une fois le gate `t()` posé, cf. §1.8.)

**Taxonomie (7 classes) :**

| Classe | Volume | Exemples ancrés | Difficulté |
|---|---|---|---|
| **T1 — Libellés statiques** (JSX text, boutons, titres) | ~40 % | `Mur.tsx:74`, `Terminal.tsx:46`, `DiffViewer.tsx:293,302` | Facile |
| **T2 — Placeholders** | ~2 | `Composer.tsx:83` (interpolation + `« »`), `ProcedeSurface.tsx:608` | Moyen (interpolé) |
| **T3 — `aria-label`/`sr-only`/`title`/`role=alert`** | ~45 | `App.tsx:56` skip-link, `Rail.tsx:48`, `OrientationBar.tsx:119`, `DiffViewer.tsx:348`, `Composer.tsx:46` | **Critique AAA/COGA** : souvent OUBLIÉ par les pipelines JSX-text-only → double la surface `t()` |
| **T4 — Chaînes interpolées (variables runtime)** | ~50 | `TechDetails.tsx:53`, `VerifyScene.tsx:145`, `ProcedeSurface.tsx:181`, `SurfaceHost.tsx:34` | Moyen (messages paramétrés `{n}`,`{status}`) |
| **T5 — PLURIELS manuels** | ~8 sites | `ProcedeSurface.tsx:313-315` `ouverte{>1?'s':''}`, `OrientationBar.tsx:93,99`, `SessionsSurface.tsx:224`, `VerifyScene.tsx:145` | **DUR** : règle FR `n>1` fausse pour AR (6 formes), ZH (1), EN (0/1) → `Intl.PluralRules` obligatoire |
| **T6 — Glyphes / ponctuation culturelle** | ~70 | `·` middot (~80×), `« »` (`Mur.tsx:54`,`Composer.tsx:83`), `—`, flèches `←→▸◂▾`, `−` U+2212, glyphes-statut `✓✕•◇⊢≣◦` | **DUR RTL** : flèches à miroiter ; `« »`→`" "` par locale |
| **T7 — Type-literal qui SERT de label ET de clé `===`** | ~2 | `TerminalXterm.tsx:30` `type TerminalStatus = 'connexion…'\|'session active'\|…` rendu direct (`Terminal.tsx:59`) ET comparé (`Terminal.tsx:16-21`) | **TRÈS DUR** : traduire le label casse le `switch` → DÉCOUPLER (clé enum stable + lookup label) AVANT toute traduction |

**Seams déjà en place (~30 % déjà keyés par enum stable — points-source idéaux).** `catalog/intentions.ts:25-44` `INTENTIONS[]` (3×{label,hint,kind}) + `:63-67` `EXEC_PROVIDERS[]` ; `catalog/surfaces.ts:22-41` `SECONDARY_SURFACES[]` (glyph locale-neutre) ; `lib/gateStatus.ts` `gateStatusLabel()`/`gateStatusGlyph()` (switch sur enum wire `gates.rs`) ; `lib/verdict.ts` `VERIFY_ETAT` (map clé→message) ; `components/steer/Atelier.tsx:13-31` `statusLabel()` (à hisser dans un catalog). Le label est la seule chose à traduire, la clé reste invariante. **Le catalogue de messages doit s'organiser sur CES mêmes clés enum** + un namespace par composant pour les littéraux T1/T3. Les ~70 % restants sont des littéraux JSX épars sur les 22 composants — c'est là le travail d'extraction.

### 1.2 Mécanisme retenu — runtime maison sur `Intl.*` (TRANCHÉ)

**Contrainte dure n°1 (CSP).** La CSP de production de l'Operator = `default-src 'self'; connect-src 'self'` (`crates/sbfb-factory/src/operator_server.rs:354`), **sans `'unsafe-inline'` ni `'unsafe-eval'`** ; le commentaire `:345-346` impose explicitement « the greenfield front must ship without inline scripts ». À NE PAS confondre avec `BLOB_SERVE_CSP` (`crates/nexus-core-rs/src/csp.rs:33`) qui contient `'unsafe-eval'` mais ne gouverne que les apps sandboxées en iframe. Conséquence : toute lib qui **compile l'ICU en JS via `new Function`/`eval` au runtime casse**. Le seul coupable réel = le package historique `@messageformat/core` (compile-to-`Function`) — **aucun candidat sérieux ne l'emprunte par défaut**. Vérification adversariale (faisant autorité) : react-intl (interpréteur d'AST), i18next (`String.replace` + `Intl.PluralRules`), Lingui v6 (token-arrays compilés au build) sont **tous eval-free et CSP-`self`-safe**. → **le différenciateur n'est PAS la CSP mais la taille RAW + la doctrine 0-dep.**

**Contrainte dure n°2 (budget RAW, vérifié).** `.size-limit.json` porte `"gzip": false, "brotli": false` sur ses 7 entrées → budgets en **octets BRUTS minifiés**, pas gzip : `app=45 KB`, `css=27 KB`, `vendor-react=210 KB`, `vendor-xterm=360 KB` (lignes 5,33,12,40). Le cadrage « gzip vs 40-44 kB » est **faux**. Le provider i18n enveloppe `<App>` à la racine → **EAGER, chemin critique** : le moteur atterrit dans `app` (déjà ~40-44 kB, **marge ~1-5 kB seulement**) ou dans un chunk dédié.

| Mécanisme | RAW min | eval/`new Function` | CSP self | ICU plural/genre | Build oxc/Babel | Doctrine 0-dep | Verdict |
|---|---|---|---|---|---|---|---|
| **0-dep maison (`Intl.*`)** | **~3-5 kB** (à confirmer) | Non (Intl natif) | ✓ par construction | sous-ensemble (Intl) | ✓ nu | **alignée Day-0 D2** | **RETENU** |
| Lingui v6 | ~8-12 kB | Non (tokens) | ✓ | complet | macro Babel | tolérable | **fallback** |
| react-intl | **58 kB** (no-parser ~42) | Non (AST) | ✓ | complet | ✓ | conflit | écarté |
| i18next + react-i18next | **~65-77 kB** | Non (Intl) | ✓ (ICU⇒plugin) | natif non-ICU | ✓ | conflit | écarté |

**Décision : catalogue de messages MAISON, 0 dépendance, bâti sur `Intl.*`.** Rationale, par force décroissante : (1) **CSP-safe par construction**, zéro surface lib à re-auditer à chaque bump ; (2) **~3-5 kB RAW** vs 58/65-77 kB pour react-intl/i18next qui défoncent le budget eager total (le découpage en `vendor-i18n` masque le gate mais **pas le payload cold-start**) ; (3) aligné Day-0 D2 (« Base UI seule primitive runtime ») et cohérent avec le pattern maison du repo (word-diff/diff-viewer 0-dep) ; (4) typage TS total sans codegen ni transform Babel ; (5) l'ICU réellement nécessaire ici = interpolation + ~8 pluriels simples + nombre/date/liste/relatif, **tous couverts par `Intl.*`** ; la directive COGA/AAA pousse aux messages plats → l'ICU imbriqué profond est inutile. **Fallback explicite = Lingui v6** (seule « vraie lib » qui tient le budget, ICU complet, CSP-safe ; Babel est DÉJÀ dans le pipeline — `@rolldown/plugin-babel` + `babel-plugin-react-compiler`, `package.json:40,50` — donc la friction macro est « un plugin Babel de plus », pas « Babel from scratch »). **Écarter react-intl et i18next** sauf explosion réelle du besoin ICU imbriqué (non observée).

**A-VERIFIER (ne pas traiter comme acquis) :** (a) le budget maison **3-5 kB RAW est une estimation honnête** (la cible « 2 kB » de la recherche sous-estimait le besoin d'un composant `<Trans>` rich-text, cf. §1.3) → à confirmer par un vrai `npm run size` post-spike ; (b) les deltas RAW du tableau sont des estimations esbuild/bundlephobia → confirmer sur le build réel après `manualChunks`+tree-shaking rolldown ; (c) si Lingui est retenu : `grep 'new Function('/'eval('` sur le `dist` (formalité, pas risque réel) + spike build macro/oxc.

**Conséquence budget (load-bearing) :** même le maison étant eager, avec une marge `app` de ~1-5 kB il faudra **soit bumper le budget `app` de quelques kB, soit créer un chunk eager `vendor-i18n`** (pattern déjà établi : `vendor-react`, `vendor-xterm`) avec sa propre entrée `.size-limit.json`. Les **catalogues par locale restent lazy** (`import(./locales/${l}.js)`, chunks same-origin autorisés par `default-src 'self'`) dans tous les cas — seul le moteur est eager. **C'est une décision PO ouverte (§1.9).**

### 1.3 Architecture du catalogue + API

**Catalogue plat par locale, `fr` = clé de référence typée**, id sémantique réutilisant les seams enum existants :

```ts
// src/i18n/locales/fr.ts — source de vérité des clés
export default {
  'rail.openProcede': 'Ouvrir le procédé',
  'verify.title': 'Vérifier avant validation',
  'sessions.count': { one: '{n} session', other: '{n} sessions' },
  'banner.offline': "Nœud injoignable — l'Operator ne répond pas…",
} as const

// src/i18n/core.ts
type Catalog = typeof import('./locales/fr').default
type Key = keyof Catalog
export function t(key: Key, vars?: Record<string, string|number>): string
//  1) lookup msg dans la locale active
//  2) interpolation {x} via String.replace ; pluriel via pr(locale).select(n) → variante one/few/many/other
//  3) jamais d'eval — tout via Intl.*

export async function loadLocale(l: string) {        // lazy, code-split par langue
  const cat = (await import(`./locales/${l}.ts`)).default
  // setActive(l, cat) ; document.documentElement.lang = l ;
  // document.documentElement.dir = directionOf(l)
}
```

**Pièces requises :** `t()` (interpolation + pluriel `Intl.PluralRules`) ; **un composant `<Trans>`-like obligatoire** pour le rich-text — les cas `<TokenCount value/> modifiés` (`OrientationBar.tsx:93,99`), guillemets et segments en N nœuds JSX **ne peuvent pas** passer par un `t()` qui rend une string (un nombre = élément React séparé du mot). Sans `<Trans>`, il faut restructurer ~5-10 sites ; avec, +~1-2 kB. C'est la part de budget sous-pondérée dans l'estimation « 2 kB ». `directionOf(l)` = `new Intl.Locale(l).getTextInfo?.().direction` (natif, CSP-safe) + fallback liste statique `RTL_LOCALES = {ar,he,fa,ur,ps,sd,ug,yi}`. Provider React minimal (Context + `useT()`) monté à la racine `src/main.tsx` ; instances `Intl.*` mémoïsées par locale (coûteuses à construire). Avant traduction, **découpler T7** (`TerminalXterm.tsx:30`) : clé enum stable + lookup label.

### 1.4 RTL complet

**Tailwind v4.3.1 fournit DÉJÀ** les utilitaires logiques (`ps-/pe-/ms-/me-/start-/end-/text-start/text-end/border-s/border-e/rounded-s/rounded-e`) **et** les variants `rtl:`/`ltr:` out-of-the-box (ciblent `[dir=rtl]`/`[dir=ltr]`). Preuve interne : ~41 occurrences logiques compilent déjà dans 16 fichiers → la chaîne fonctionne, **aucun plugin à ajouter**.

**Travail de bascule :** poser `dir` sur `<html>` piloté par la locale (aujourd'hui ABSENT, `index.html:2`), à côté de `lang`. Convertir **~35 occurrences physiques sur 12 fichiers (plancher, `px-/py-/gap-` symétriques exclus)** : `ml-auto`→`ms-auto` (~12 sites, motif dominant), `border-l/border-r`→`border-s/border-e` (séparateurs + indicateur actif rail `Rail.tsx:36-37`), `pl-/pr-/ml-/mr-`→`ps-/pe-/ms-/me-`, `text-left/text-right`→`text-start/text-end`, `left-3`→`start-3` (`App.tsx:54`). Flexbox/`gap`/`flex-row` s'inversent automatiquement sous `dir=rtl` (0 travail).

**Zones verrouillées `dir="ltr"` (~8 sites).** Le code source est intrinsèquement LTR : envelopper `DiffViewer.tsx` (gouttières numéros de ligne `text-right`, split `border-r`) + `Terminal`/`TerminalXterm` (xterm LTR par nature) dans `dir="ltr"` → les `text-right` deviennent corrects-par-construction et immunisés au flip global.

**Glyphes directionnels en CONTENU (non auto-miroités).** Flèches `←→` (`SurfaceHost.tsx:37`, `DiffViewer.tsx:386`, `ProcedeSurface.tsx:323,653`), chevrons/disclosures `▸◂▾`, séparateurs `▸` (`OrientationBar.tsx:82,86,102`) : soit swap glyphe par `dir`, soit `rtl:-scale-x-100` / `[dir=rtl] .x{transform:scaleX(-1)}`. Reco : remplacer les `▸` séparateurs `aria-hidden` par un séparateur bidi-neutre (`/`,`·`) OU un chevron CSS rotable. Les `« »` (`Mur.tsx:54`, `Composer.tsx:83`) sont des caractères Unicode bidi-mirrored (auto-flip OK) mais relèvent du **style de citation par locale** → délégués au catalogue. Le mouvement est déjà RTL-safe (`translateY`/`y`/`rotateX`, aucun `translateX`) ; inscrire « axe Y / propriétés logiques uniquement » comme contrainte DS pour toute future signature.

**Piège critique :** `letter-spacing`/`tracking-*` cassent le *joining* arabe → guard obligatoire `[dir="rtl"] .eyebrow, [dir="rtl"] [class*="tracking-"] { letter-spacing: normal; }` (`.eyebrow` a déjà abandonné `uppercase`, bien). **Ampleur RTL ≈ 1-1,5 j** (27 conversions chrome mécaniques + ~8 wraps `dir="ltr"` + ~4 séparateurs à neutraliser). Ajouter un gate anti-régression interdisant les nouveaux `pl-/pr-/ml-/mr-/text-left/text-right/border-l/border-r` hors zones `dir="ltr"` whitelistées (§1.8).

### 1.5 Expansion de texte — règles de layout

DE/FI +30-35 %, AR +20-30 % vs EN. Cibles à risque : labels d'inspecteurs (`SECONDARY_SURFACES.label`), phrases tutélaires du rail (`Rail.tsx:91`), `OrientationBar` (rangée unique `h-12` + `truncate` sur phase/branch). **Règles fermes :** (1) `min-w-0` systématique sur les flex-children de texte tronqué (sinon `truncate` ne déclenche pas dans un flex ; déjà ponctuel via l'idiome `w-0`, à généraliser) ; (2) jamais de largeur figée sur du texte — le rail `w-[158px]` reste car c'est une colonne de layout, mais ses enfants texte doivent autoriser le wrap ; envisager `min-w-[158px]`/`w-40 lg:w-44` ; (3) `line-clamp-2` plutôt que `truncate` mono-ligne sur intentions/labels longs ; (4) `min-h-*` plutôt que `h-*` fixe sur barres et boutons (`OrientationBar` `h-12`, banner offline `h-7` `App.tsx:67`) — **prérequis partagé avec le mode espacement-texte WCAG 1.4.12 du Pilier D** ; (5) `text-wrap: pretty/balance` sur la prose.

### 1.6 Polices & scripts (tension CJK/arabe — reco chiffrée)

**Couverture Geist vérifiée (vendorée).** Subsets présents : latin, latin-ext, cyrillic, cyrillic-ext, vietnamese (+`symbols2` mono). **AUCUN subset grec** — le grec retombe déjà sur la cascade `--font-sans` (`index.css:60`). Pas de CJK, arabe, hébreu, devanagari. Payload vendoré ≈ 154 KB total ; téléchargement réel FR/EN par `unicode-range` ≈ 59 KB (latin sans+mono) ; `font-display: swap` déjà posé par fontsource. **`size-limit` ne compte PAS les woff2** (entrées `*.js`/`*.css` seulement, vérifié) → les polices échappent au gate.

**Contrainte CSP self :** `font-src` retombe sur `default-src 'self'` → **toute police doit être same-origin** (CDN Google Fonts bloqué) ; `vite.config.ts` `assetsInlineLimit:0` → woff2 en fichiers séparés, jamais `data:`. Donc tout vendoring = same-origin.

**Ordres de grandeur (confirmés adversarialement) :** CJK Noto SC/TC/JP/KR = **4-9 MB/famille** non-subsetté, subset agressif **1-3 MB** ; arabe Noto Sans Arabic **~80-130 KB** ; hébreu **~30-60 KB**.

**Reco tranchée (exploite « Operator = outil local sur le poste de l'opérateur ») :**
1. **CJK + grec → polices SYSTÈME, 0 KB vendoré, NON négociable** (vendoring 1-3 MB exclu par CSP-self + payload). Correction de mécanisme : `system-ui` ne contient PAS les glyphes CJK — c'est le **last-resort du navigateur** (HarfBuzz) qui sélectionne la police CJK OS. → **ajouter des fallbacks NOMMÉS par script** dans `--font-sans` (`index.css:60`), pas se reposer sur `system-ui` seul : `'Geist Variable','PingFang SC','Microsoft YaHei','Hiragino Sans','Malgun Gothic','Noto Sans CJK SC', ui-sans-serif, system-ui, sans-serif`. La tension MB **s'évapore**.
2. **Arabe / hébreu → système suffit FONCTIONNELLEMENT** (le shaping/joining est géré par le moteur + police OS, 0 KB par défaut). Vendoring Noto ~50-130 KB **lazy, optionnel, marque seule**, chargé uniquement quand la locale RTL est active (jamais dans le hero).
3. **Latin reste Geist** (identité), inchangé.
4. **Caveat** `font-synthesis: none` (`index.css:94`) : un script rendu par police système sans la graisse demandée (`font-semibold`) ne sera pas synthétisé (rendu plus fin) — acceptable, à noter ; `font-synthesis: weight` si gêne visuelle.
5. **Si vendoring** (arabe/hébreu marque, ou Atkinson Hyperlegible du mode dyslexie Pilier D — latin-only, à composer avec fallback locale `'Atkinson Hyperlegible', <fallback locale>, system-ui`) : **ajouter un gate de payload font** car les woff2 échappent à `size-limit` → sinon un subset de 1 MB passe inaperçu.

### 1.7 Formatage `Intl.*` natif (0 dep, CSP-safe)

Tout `Intl.*` (`NumberFormat`, `DateTimeFormat`, `PluralRules`, `RelativeTimeFormat`, `ListFormat`) est natif sur navigateur evergreen → 0 dépendance, 0 eval. `Intl.PluralRules` rend correctement les **6 catégories arabes** (zero/one/two/few/many/other) **sans polyfill sur cible evergreen** (doute initial rétrogradé par la vérification adversariale). **Dettes à corriger :** `sessionDate.ts:12` `toLocaleString('fr-FR')` → consommer la locale active (changement trivial, déjà Intl-backed) ; pluriels codés en dur (`OrientationBar` « modifiés/indexés », `VerifyScene.tsx:145`) → router via `Intl.PluralRules(locale).select(n)` + variantes catalogue, jamais `(s)` FR ; compteurs (`SessionsSurface.tsx:254` « ko ») → `Intl.NumberFormat` (séparateur de milliers, shaping arabe-indien des digits, unité traduite). **Reco :** mince couche `lib/format.ts` exposant `fmtNumber/fmtDate/fmtPlural/fmtList` paramétrées par la locale active, mémoïsant les instances `Intl.*`. Garder `tabular-nums` (12 sites) pour l'alignement latin. Aucun impact size-limit/CSP.

### 1.8 Refonte du gate de strings

**État réel :** `scan-en-strings.sh` (blocklist d'anglais hardcodé) vit dans `web/scripts/`, **PAS dans le tool** → l'i18n ne casse aucun gate FR-only dans l'Operator. Le tool a `scripts/scan-front-discipline.sh`, qui **n'enforce PAS la langue** : il interdit les mots-verdict `\b(PASS|Vérifié|Verifie|Approuvé|Approuve)\b` comme texte UI visible dans `src/` (ligne 23), strip de la comparaison légitime `=== 'PASS'` (ligne 49), exclusion `*.test.*` (ligne 32). La directive CLAUDE.md « repenser scan-en-strings pour i18n » vise le **concept**, à porter ici. **Inversion conceptuelle :** sous i18n, l'anglais devient une locale cible légitime ; le problème n'est plus « pas d'anglais » mais « pas de littéral non-traduit ». Trois gates :

- **Gate A — no-literal (le vrai remplaçant).** Interdire tout littéral user-facing hors `t()`/`<Trans>` dans les composants : JSX text nu, **ET `placeholder=`/`title=`/`aria-label=`/`sr-only` en chaîne nue** (sinon ~45 chaînes a11y T3 non traduites = régression AAA/COGA). Scan bash maison sur le modèle `scan-front-discipline.sh`, ou `eslint-plugin-formatjs`/`i18next` règle `no-literal-string`.
- **Gate B — anti-verdict cross-locale.** L'invariant anti-PASS **fuit par la traduction** : un traducteur peut réintroduire « PASS »/« Approved »/« Aprobado »/« 通过 »/« معتمد » dans un `xx.json` et contourner le gate qui ne scanne que `src/`. → étendre `scan-front-discipline.sh` pour scanner **les VALEURS de TOUTES les locales** `src/i18n/locales/*` avec une **forbidden-list par langue** (PASS universel + mots-verdict par locale). Tension réelle : la discipline anti-verdict se multiplie par N locales.
- **Gate C — parité de clés.** Toute clé de `fr` (référence) présente dans chaque locale ; manquante = build-fail (ou WARN+fallback `fr`, cf. §1.9). Natif, 0 dep, scriptable.

Étendre la T1 CSP existante (e2e) pour couvrir le changement de locale (import dynamique) + l'application des prefs → toujours 0 violation CSP, 0 inline-style.

### 1.9 Décisions PO ouvertes (propres à l'i18n)

1. **Périmètre du lot 1 :** liste exacte des locales livrées d'emblée et lesquelles RTL (la directive PO #2 dit « FR/EN/ES/AR/ZH… incl. RTL » — confirmer le set initial et l'ordre de curation).
2. **Politique parité de clé pré-launch :** build-fail strict (Gate C dur) **vs** WARN + fallback `fr` tant qu'une locale est incomplète. Impacte le rythme de traduction.
3. **Budget eager :** **bumper `app` de quelques kB** (marge actuelle ~1-5 kB) **vs** créer un chunk dédié `vendor-i18n` avec sa propre entrée `.size-limit.json` (le payload cold-start total est identique dans les deux cas ; seul le gate change).
4. **Vendoring marque RTL :** vendorer Noto Sans Arabic (~80-130 KB) / Hebrew (~30-60 KB) lazy pour la cohérence de marque **vs** s'appuyer sur le système (0 KB, shaping OK). Si oui → Gate de payload font obligatoire (§1.6).
5. **Niveau de lecture 3.1.5 (AAA cognitif) = revue éditoriale humaine PAR LANGUE**, non gatable mécaniquement : « Travailler »/« Vérifier » sont des choix plain-FR, **pas** une traduction machine ; chaque locale exige sa propre curation plain-language (bar variable par script : densité ZH, longueur DE, morphologie AR). Décider **qui cure** chaque locale. Le seul gate mécanisable = complétude du catalogue (Gate C) + façade ⊆ glossaire.
6. **Guillemets / ponctuation par locale** (`« »` FR vs `" "` EN vs autres) : confirmer qu'ils relèvent du catalogue, pas de littéraux composant.

**A-VERIFIER restant avant code :** budget maison réel (`npm run size` post-spike, estimation 3-5 kB RAW) ; deltas RAW par moteur sur build réel ; si Lingui retenu, grep `new Function`/`eval` sur `dist` + spike macro/oxc. Le RTL (bascule `dir` + migration physique→logique + miroir flèches) est un **chantier indépendant et parallèle** du choix de mécanisme, à mener de toute façon.

## 2. Pilier B — Design system handicaps (AAA + COGA)

Décision PO #3 : viser WCAG 2.2 **AAA** + **COGA** au-delà de AA. Ce pilier pose la doctrine par catégorie de handicap puis les **modes préférence** opt-in. Contrainte transverse qui conditionne tout : la CSP Operator (`crates/sbfb-factory/src/operator_server.rs:354` = `default-src 'self'; connect-src 'self'`, sans `'unsafe-inline'`/`'unsafe-eval'`) interdit `<script>` inline et `style="…"` parsé du markup — toute la mécanique de préférence passe par `setAttribute`/CSSOM (non gouvernés par `style-src`).

**Bloqueur antérieur à toute revendication AAA (escaladé) :** le **critère AA 1.4.10 Reflow est en échec aujourd'hui**. Le shell est `flex h-screen … overflow-hidden` (`App.tsx:48`) avec rail `w-[158px] flex-shrink-0` (`Rail.tsx:73`) + `main flex-1` côte-à-côte. À 320 px / zoom 400 %, rail et contenu forcent un scroll 2D et clippent. **On ne peut pas afficher une AAA cognitive/visuelle en échouant un AA fondamental** → le **rail collapsible** (drawer/top-bar sous breakpoint) est le chantier prioritaire, en tête de doc, avant tout mode opt-in.

**Caveat de fiabilité :** toutes les valeurs de ratio de contraste citées plus bas (défaut L1 + candidates haut-contraste) sont des conversions oklch→sRGB→WCAG **calculées à la main** par les agents → **provisoires** jusqu'à l'existence du gate de contraste automatisé (oklch→linéaire→WCAG, BLOQUANT par mode) défini au pilier gates. Ne pas les traiter comme acquises.

---

### 2.1 Basse vision (AAA 1.4.6 / 1.4.8 / 1.4.12 + AA 1.4.10)

**État actuel (ancré).** Les tokens d'encre sont à 4 niveaux (`index.css:44-47` : `tx 0.930` / `tx2 0.720` / `tx3 0.700` / `tx4 0.630`) sur 4 surfaces (`index.css:28-31` : `s0 0.165`→`s3 0.290`). L'audit L1 a porté tx3 et tx4 au-dessus de AA texte normal sur s0–s2, mais **par construction tx4 reste un ghost décoratif** (échoue AA sur s3) et `field` (`index.css:39`, bordure de contrôle) ne tient le 3:1 non-texte que sur s0–s2. L'échelle typo est tokenisée **en px** (`index.css:70-74`) avec body `font-size:16px; line-height:1.5` **hardcodés** (`index.css:92-93`).

**Cibles.** 1.4.6 (7:1 texte AAA) ; 1.4.8 (prose AAA : ≤80 ch, espacement) ; 1.4.12 (espacement texte modifiable) ; 1.4.4 (zoom 200 % sans perte) ; 1.4.10 (reflow 320 px).

**Actions.**
- **7:1 = mode opt-in, JAMAIS défaut.** Pousser tx2/tx3/tx4 à 7:1 sur s3 les agglutine tous vers le blanc et détruit la hiérarchie « 3 encres lisibles + 1 ghost » qui est le cœur du design (confirmé adversarialement). Défaut = AA (acquis L1) ; AAA via `[data-contrast="high"]` (§2.8).
- **1.4.8 prose ≤80 ch :** poser `max-w-[80ch]` sur la prose seulement (atelier `whitespace-pre-wrap` `Atelier.tsx:75` en panneau large dépasse), **jamais** sur code/diff/terminal (LTR verrouillé, largeur = sémantique).
- **1.4.4 / échelle :** migrer base + tokens typo de **px → rem** (`html { font-size }` piloté par `data-text-size`, tokens `--text-* : 1rem`…) — honore à la fois le zoom OS/navigateur ET le contrôle in-app. **Tension :** retouche un livrable L2 récent (à signaler PO). Préférer `html{font-size:calc(…)}` au `calc()` dans le namespace spécial `--text-*` (line-height implicite TW v4, à vérifier au build).
- **1.4.12 :** migrer les **hauteurs fixes** (`h-12` OrientationBar `OrientationBar.tsx:66`, `h-7` bannière offline `App.tsx:67`) en `min-h-*` — sinon une feuille d'espacement utilisateur clippe. Override d'espacement = §2.8.
- **1.4.10 reflow :** rail collapsible (bloqueur AA ci-dessus).

---

### 2.2 Daltonisme (règle système « couleur jamais seule »)

**État actuel.** Le pattern correct existe déjà : `gateStatus.ts` couple glyphe (`✓/✕/—`, `:42-54`) + libellé FR (`gateStatusLabel` `:73-86`) + ton ; Atelier couple dot + texte nommé (`Atelier.tsx:56-60`) ; bannières offline/loopback couplent dot + texte (`App.tsx:69-72`, `OrientationBar.tsx:124-130`).

**FINDING bloquant (1.4.1).** `gateStatusGlyph` renvoie **le même `•` pour `informational` ET `not_run`** (`gateStatus.ts:51-52`) alors que leurs tons diffèrent (warn ambre vs neu gris, `:63-66`) → ces deux statuts ne sont distingués **que par la couleur**. **Action :** glyphes distincts (ex. `informational → ◆`, `not_run → ○`).

**Cible / règle DS canonisée.** *Toute information encodée par couleur porte AUSSI un glyphe + un libellé (visible ou `sr-only`).* **Action mécanisable :** étendre `scan-front-discipline.sh` d'un check « toute classe `text-(ok|warn|bad|info|mur)` co-localisée avec un glyphe ou un `sr-only` ». Corollaire : pour AAA, **écrire en `tx` sur fond coloré** (`tx`/`ok-bg` ≈ 10.9, `tx`/`bad-bg` ≈ 12.5) plutôt qu'en encre colorée sur fond coloré (`bad`/`bad-bg` ≈ 4.91, échoue AAA).

---

### 2.3 Aveugle / lecteur d'écran (sémantique, live regions, noms-rôles-états)

**Acquis (L3).** Skip-link 2.4.1 (`App.tsx:52-57`), `main tabIndex=-1` (`App.tsx:88`), `nav aria-label` (`Rail.tsx:72`), `role=group`/`aria-pressed` (Rail, Composer), `role=status` offline (`App.tsx:65`), `aria-busy` refresh (`OrientationBar.tsx:116`), `sr-only` sur bouton-icône (`OrientationBar.tsx:118-119`).

**FINDING L5 majeur — l'atelier streaming n'a AUCUNE live region.** Le transcript SSE (`Atelier.tsx:69-88`) et le `turn-status` (`Atelier.tsx:58`, 8 états `statusLabel:13-31` : « l'agent travaille » → « terminé · prêt à examiner » → « interrompu — erreur de flux ») changent **sans `aria-live`**. Un utilisateur SR n'entend ni progression, ni fin, ni abort, ni erreur du tour.

**Cible / action.**
- `aria-live="polite"` sur le conteneur de statut (annonce les transitions d'état nommé).
- Pour le flux de tokens : **ne pas live-régionner chaque delta** (verbeux) → annoncer uniquement les jalons (début/fin/erreur/gate) dans une live region dédiée ; exposer le corps final en `polite`.
- **Curseur de frappe `▌`** (`Atelier.tsx:87`) : caractère statique injecté dans le `<pre>` → le masquer du SR (`aria-hidden`) car il pollue la lecture (problème 4.1.3, pas vestibulaire).
- **Piège focus xterm (2.1.2) :** `TerminalXterm` capture le clavier → garantir et documenter une sortie clavier (Esc).

---

### 2.4 Moteur (cibles tactiles, focus, clavier, timing)

**État actuel (cibles mesurées).** AA 2.5.8 (24×24) presque tenu ; **2 contrôles échouent** : le refresh OrientationBar `px-1.5 py-0.5` ≈ 20–23 px (`OrientationBar.tsx:109-120`) et le toggle « détails techniques » `text-meta` sans padding ≈ 19 px (`Composer.tsx:105-112`). Le bouton « Lancer » `px-4 py-2.5` ≈ 44 px (`Composer.tsx:118-126`) est le seul conforme AAA.

**Focus (2.4.13 AAA) — FINDING.** Le défaut global `:focus-visible { outline:2px info; offset:2px }` (`index.css:100-103`) est conforme AAA. **MAIS** le textarea et le select du Composer font `focus:outline-none` + bascule de **bordure 1px** `focus:border-info` (`Composer.tsx:85,96`) → indicateur 1px sous l'aire 2.4.13, trop proche du « couleur seule ».

**Cibles / actions.**
- **2.5.8 (AA 24×24) = plancher toujours-actif :** 2 fixes ciblés (`min-h-6 min-w-6`/padding sur refresh + toggle).
- **2.5.5 (AAA 44×44) = mode opt-in `[data-pointer="large"]`** (§2.8). 44 px partout détruit la densité bi-focale (rail compact, barre d'orientation dense = valeur de design **assumée**) → jamais défaut.
- **Focus :** bannir `focus:outline-none` sans ring 2px de remplacement ; fournir une util `.focus-ring` partagée + **gate lint anti-`outline-none`**.
- **Clavier (2.1.1) :** OK — boutons/select natifs, focal `s`/`v` (`useFocalKeys`), Ctrl/Cmd+Enter submit (`Composer.tsx:78`). Roving-tabindex sur le `role=group` du rail = nice-to-have, non requis.
- **Timing (2.2.1/2.2.3 AAA) :** aucun timer auto-avançant ; SSE contrôlé par l'utilisateur (Interrompre `Atelier.tsx:99`) → conforme. Vérifier qu'aucun toast n'auto-disparaît.

---

### 2.5 Cognitif / COGA (langage clair, jargon en disclosure, identification, prévention d'erreurs, aide mémoire)

Charge cognitive #1 : un métalangage de procédé dense (2 modes anglais STEER/VERIFY, 3 inspecteurs `surfaces.ts:22-41`, vocabulaire `gate/préflight/review/verdict/diff/hunk`). ~25 atomes à l'arrivée.

**3.1.5 niveau de lecture / 3.1.3 mots inhabituels — lien glossaire L6.** L'audit fournit déjà le mapping plain-language (`factory_universal_ux_a11y_audit.md:56-99` : `STEER`→**Travailler**, `VERIFY`→**Vérifier**, `gates`→**contrôles**, `verdict`→**conclusion** « jamais PASS », `préflight`→**préparation**, glyphes seuls `⊢ ≣ ◇` → icône **+ libellé visible**, `◦` poly-sémique → 3 libellés distincts). **Action structurante :** matérialiser ce glossaire comme **donnée centrale** `catalog/glossary.ts` (`term → {label, definition}` par locale), source unique consommée par (a) le renommage façade, (b) un primitif `<Glossaire term=…>` (`<button aria-expanded>` + popover Base UI, **jamais `title=`**), (c) une surface « Connaissances ». Le glossaire devient **l'unité conjointe d'a11y cognitive ET de localisation** (cf. Pilier i18n) — pièce à poser AVANT le câblage i18n.

**3.1.3 / disclosure de jargon — pattern de référence déjà présent.** `TechDetails` replie `kind/provider/prompt` hors du CTA (`Composer.tsx:105-129`, `aria-expanded`) ; les CTA parlent en **intentions** (`catalog/intentions.ts`). **Action :** généraliser en primitif `<DetailsTechniques>` ; Home 4 cartes-intentions + « Mode avancé » repliant la télémétrie (charge ~25 → ~6).

**3.2.4 identification cohérente.** Tenu (`OrientationBar.tsx:118-119`), cassé pour les glyphes seuls `⊢ ≣ ◇` du rail (`surfaces.ts:23,29,35` rendus `aria-hidden` `Rail.tsx:113-115`). **Action :** primitif `IconLabel` forçant le nom accessible ; 1 fonction = 1 paire (icône, libellé) stable, jamais un glyphe seul ni surchargé.

**3.3.x prévention d'erreurs — `Mur.tsx` est l'exemplaire COGA, à canoniser en patron DS.** `role="alert"` + `aria-label="Barrière de gouvernance"` + `<h2>` + prose plain-language du **pourquoi** + exactement 2 actions (« Préparer le pack » / « Retour ») + **zéro affordance Forcer/Override/Bypass** (3.3.4 actions irréversibles : commit/push/shell). `Atelier.tsx:13-31` (`statusLabel`, 8 états nommés en FR clair) = patron de **restitution plain-language, jamais un verdict**. **Action :** patron DS « barrière explicative » réutilisable pour toute action irréversible ; aide contextuelle 3.3.5 par scène (pas qu'au Mur).

**3.2.5 changement sur demande (acquis, invariant D6).** La bascule STEER→VERIFY n'est jamais auto (`Rail.tsx:42-49`, point `verify-ready` = indice manuel). **Gap COGA résiduel (L6) :** remplacer le point 6 px par un **bandeau de guidage explicite** « Travail terminé — prêt à vérifier ▸ » + bouton manuel (COGA-positif ET D6-safe).

**Tension COGA × invariant 0-verdict-UI (à surveiller en revue).** L'aide COGA explique le **vocabulaire** (« ce qu'est un préflight/gate ») = aide définitionnelle ; elle **n'asserte ni ne calcule jamais la valeur** d'un verdict. Le Mur prouve la compatibilité. Risque : un tooltip qui glisse de « ce qu'est X » vers « X est réussi ».

**Risque non gatable.** Le niveau de lecture 3.1.5 est une **revue éditoriale humaine par locale**, pas un gate code. Ne pas sur-promettre une AAA cognitive automatisée ; le seul gate mécanisable = complétude du catalogue + « jargon de façade ⊆ glossaire ».

**`title=` omniprésent = échec 3.1.3 (FINDING).** Info-clé portée en `title` à `OrientationBar.tsx:114`, `Atelier.tsx:96,107,117`, `Rail.tsx:46`, `GatePulse OrientationBar.tsx:41-43` — inaccessible au tactile, partiel au clavier, ignoré par nombre de SR. **Action :** disclosure réelle ou `aria-describedby` vers élément visible.

---

### 2.6 Auditif (N/A prouvé, règle d'inscription préventive)

**État (prouvé).** Aucun audio dans l'app : xterm `^6.0.0` a retiré le bell sonore du moteur, `TerminalXterm`/`CastXterm` construisent `new Terminal({…})` **sans `bellStyle`** et **sans `onBell`→`Audio`** ; aucun `<video>`/`new Audio()`. → **1.2.1–1.2.9, 1.4.2, 1.4.7 = N/A.**

**Règle DS à inscrire (si un son est un jour ajouté — notif fin de tour/bell) :** (a) optionnel/désactivable (1.4.2) ; (b) doublé d'un signal visuel `aria-live` ; (c) jamais le seul porteur d'info (1.1.1 / 1.4.1 étendu au son).

---

### 2.7 Vestibulaire / photosensible (audit animation par animation)

**Inventaire complet du mouvement** (allowlist `lib/motion.ts`, 5 signatures, « mouvement = sens »), audité contre 2.3.1/2.3.2 (3 flashs, A+AAA) et 2.3.3 (animation sur interaction, AAA) :

| # | Signature | Mécanisme | Durée | Itér. | Ancrage | Flash |
|---|---|---|---|---|---|---|
| 1 | token-settle | CSS `.motion-settle` (`index.css:126,128`) | 220 ms | 1 (re-key) | `translateY(5px)`+opacity | OK |
| 2 | gate-flip | JS Motion `rotateX` | 180 ms | 1 | transform | OK |
| 3 | verification-reveal | JS stagger `y:8→0` | 220 ms +40/enf. | 1 | transform | OK |
| 4 | altitude-shift | View Transition native (`index.css:134-139`) | 220 ms | 1, **interaction seule** | `translateY` | OK |
| 5 | confirmation-gravity | CSS `.motion-gravity` (`index.css:127,129`) | 320 ms | 1 | `translateY(-10px) scale .985` | OK |

**2.3.1/2.3.2 = PASS** : zéro animation en boucle (grep `animate-(pulse|spin|ping|bounce)|infinite` négatif), 4 keyframes single-shot transform-ancrés (`index.css:128,129,138,139`), aucune variation de luminance >3 Hz, aucune grande surface clignotante (seul changement = opacité 0→1 = fondu, pas flash).

**2.3.3 = EXEMPLAIRE (triple filet, à conserver tel quel).** (1) reset CSS `@media (prefers-reduced-motion: reduce)` collapse universel (`index.css:144-150`) ; (2) garde JS au call-site (`GateFlip`/`Reveal` rendent un nœud plain sous `usePrefersReducedMotion`) — nécessaire car **le reset CSS ne touche pas WAAPI** (`element.animate`) ; (3) court-circuit de la View Transition dans `altitudeShift.ts` (non auto-gatée par MotionConfig). **Inscrire comme contrainte DS :** tout le mouvement est sur l'**axe Y** (translateY / rotateX, zéro translateX) → RTL-safe par construction ; toute future signature reste axe-Y ou propriétés logiques.

**SEUL résidu (FINDING 2.2.2).** `TerminalXterm` : `cursorBlink: true` → curseur clignotant ~1–1,2 Hz, auto-démarré, géré en interne par xterm (hors `@media` CSS et hors WAAPI gaté). Pas un déclencheur photosensible (1 cellule), mais 2.2.2 Pause/Stop/Hide s'applique au clignotement auto. **Action triviale :** lier `cursorBlink` à `usePrefersReducedMotion` (ou option « curseur fixe ») — ferme 2.2.2 + confort COGA.

---

### 2.8 Modes préférence (le cœur « design system pour tous »)

**Architecture (CSP-self compatible, 0 dep).** Tailwind v4 `@theme` (non-inline, `index.css:26`) émet des utilitaires qui **référencent** `var(--color-*)` → **un mode = redéfinir les custom properties sous un sélecteur `[data-*]` sur `<html>`** (unlayered, après `@theme`, comme `.eyebrow`/`.motion-*` déjà), zéro duplication, zéro variant `hc:`. 7 axes orthogonaux mappés sur des attributs (writes non-style, 100 % CSP-safe) :

```
data-theme=dark|light · data-contrast=normal|high · data-scale=100|125|150|175|200
data-spacing=normal|loose · data-font=default|hyperlegible · data-motion=full|reduced
data-pointer=normal|large   (+ lang/dir pour i18n)
```

**Anti-FOUC = `/preboot.js` externe same-origin OBLIGATOIRE (PAS `useLayoutEffect`).** L'inline est interdit par la CSP ; `useLayoutEffect` mount après le parse du bundle → flash visible 100–300 ms au cold-start si pref ≠ défaut HTML. `default-src 'self'` **autorise** un `<script src="/preboot.js">` bloquant dans `<head>` : il lit localStorage et pose `documentElement.dataset.*`/`lang`/`dir` avant le 1er paint. Le serveur Operator doit le servir en statique ; y déplacer aussi `data-theme="dark"` aujourd'hui figé `index.html:2`. Persistance = `localStorage` (clé `factory-operator.prefs`, JSON schema-versionné). `system` résolu via `matchMedia` (`prefers-color-scheme`, `prefers-contrast`, `prefers-reduced-motion`). Panneau « Accessibilité & langue » = `Dialog` Base UI (seule primitive runtime, Day-0 D2) déclenché depuis l'OrientationBar, aperçu live.

| Mode | Statut | Mécanisme | Valeur | Coût / tension |
|---|---|---|---|---|
| **Contraste élevé** `[data-contrast="high"]` | **opt-in** | override ~18 custom props oklch + `--focus-w:3px` (2.4.13) | les 4 encres tiennent ≥7:1 (1.4.6), contours `field`/`bd2` ≥3:1 | ~0,5 kB CSS. **Jamais défaut** (7:1 défaut casse la hiérarchie tonale). Axe distinct de Windows HC → ajouter `@media (forced-colors:active)` + system-color keywords (sinon bordures couleur-seule disparaissent). Ratios **provisoires** → gate contraste auto. |
| **Échelle police** `data-scale=100→200` | **opt-in** (défaut 100) | `html{font-size:calc(16px*--ui-scale)}` + tokens typo en **rem** | 1.4.4 (200 %) + honore zoom OS | **Retouche L2** (px→rem, flag PO). Tension échelle↔layout : `h-7/h-12` fixes clippent → `min-h-*` (cf. 2.1). |
| **Espacement texte** `[data-spacing="loose"]` | **opt-in** | vars `--ui-leading 1.6 / --ui-tracking .12em / --ui-word .16em / --ui-para 2em` | 1.4.12 modifiable par l'utilisateur | **R1 bloquant :** les `leading-*`/`tracking-*` per-composant + `line-height:1.5` body (`index.css:93`) **écrasent** l'override → 1.4.12 cosmétique tant qu'on ne tokenise pas (`--leading-body`…) + gate anti-`leading-[…]`/`tracking-[…]` brut. **RTL :** guard `[dir="rtl"] [class*="tracking-"]{letter-spacing:normal}` (casse le joining arabe). |
| **Police lisible** `data-font=hyperlegible` | **opt-in** (défaut Geist) | `--font-sans:'Atkinson Hyperlegible',…` (garde `--font-mono`) | dyslexie + basse vision (désambiguïsation glyphes) | **Atkinson Hyperlegible** (SIL OFL) > OpenDyslexic (preuve faible). Vendoré `@fontsource` same-origin, **latin-only** → fallback par locale ; ~2×35 kB woff2, lazy de fait (fetch seulement si actif), **hors size-limit** (woff2 non gatés) → ajouter un gate de payload font. OpenDyslexic différé/carry. |
| **Reduced-motion in-app** `data-motion=reduced` | **opt-in** (défaut = OS via `matchMedia`) | dupliquer le reset `index.css:144-150` sous l'attribut + `usePrefersReducedMotion` en OR la pref + `MotionConfig reducedMotion={pref==='reduced'?'always':'user'}` | contrôle indépendant de l'OS | câblage à 3 endroits (sinon les `m.*` VERIFY échappent). |
| **Cibles larges** `data-pointer=large` | **opt-in** (défaut AA 24×24) | règle globale `min-h-11 min-w-11` sur tous les interactifs | 2.5.5 AAA (44 px) | accepte le reflow (rail s'élargit). **Jamais défaut** (détruit la densité bi-focale). |

**Composition.** Tous les axes sont orthogonaux ; ordre de cascade = `@theme` (plancher) → `[data-theme=light]` → `[data-contrast=high]` puis `[data-contrast=high][data-theme=light]` → scale → spacing → font → motion. Le thème **light** (à créer ; aujourd'hui le `@custom-variant dark` `index.css:14` est inerte, les valeurs `@theme` SONT le jeu sombre) recolore surfaces/encres + `color-scheme:light` — auditer les composants qui supposent dark en dur.

**Gate d'anti-régression dédié (pilier gates) :** contrast-checker oklch→WCAG BLOQUANT **par mode** (automatise les ratios manuels, provisoires), + snapshot Playwright par combinaison de mode (incl. `dir=rtl`) sur une route `/demo` rendant la matrice tokens × modes.

---

**TENSIONS escaladées (synthèse).** (1) **1.4.10 Reflow AA cassé** = bloqueur #1, rail collapsible avant toute AAA. (2) 7:1 et 44 px = **opt-in**, jamais défaut (densité load-bearing). (3) **preboot.js externe** seul chemin anti-FOUC CSP-self. (4) échelle px→rem = retouche L2 à arbitrer PO. (5) override 1.4.12 cosmétique tant que `leading-*`/`tracking-*` non tokenisés (R1). (6) `gateStatusGlyph` `•` partagé informational/not_run = 1.4.1 (fix glyphes). (7) Atelier sans `aria-live` = invisible SR (L5). (8) `title=` porteurs d'info-clé = 3.1.3 (disclosure réelle). (9) ratios contraste **provisoires** jusqu'au gate automatisé. (10) police lisible latin-only + woff2 hors size-limit = gate payload font requis.

## 3. Architecture transverse

### 3.1 Couche tokens étendue (zéro duplication de `@theme`)

Principe directeur : `@theme` (`src/index.css:26-79`) reste la **source de vérité « dark normal »** ; chaque mode = un **bloc de surcharge des `--color-*` / `--ui-*`**, unlayered (pattern déjà établi par `.eyebrow`/`.motion-*`/reset reduced-motion, qui battent `@layer theme`). Comme `.bg-s0`/`.text-tx3` compilent en `var(--color-*)` (vérifié par le commentaire `index.css:16-25` : `@theme inline` n'émettait pas les utilities), **surcharger la variable recolore les ~189 usages sans toucher un composant**.

Blocs à ajouter (oklch, deltas uniquement), dans cet **ordre de cascade** (spécificité `(0,1,0)` départagée par l'ordre source ; le combiné `(0,2,0)` gagne) :

```
@theme                                  → base dark normal (plancher, @layer theme)
[data-theme="light"]                    → recolore surfaces/encres + color-scheme:light
[data-contrast="high"]                  → AAA 7:1 (encres montées, bd→field, focus 3px)
[data-contrast="high"][data-theme="light"]
[data-scale="125|150|175|200"]          → --ui-scale
[data-spacing="loose"]                  → --ui-leading/tracking/word/para (seuils 1.4.12)
[data-font="legible"]                   → --font-sans = 'Atkinson Hyperlegible', …
[data-motion="reduced"]                 → reset motion (miroir du @media reduced-motion)
@media (forced-colors: active)          → system-color keywords (axe DISTINCT de high-contrast)
```

Points durs vérifiés :
- **Échelle typo** : préférer **`html { font-size: calc(16px * var(--ui-scale)) }` + tokens typo en `rem`** plutôt que `calc(20px*var(--ui-scale))` dans le namespace spécial `--text-*` (line-height implicite TW v4 = risque, à confirmer dans le CSS bâti). Avantage rem : honore **aussi** le zoom OS/navigateur (1.4.4 200 %). Coût honnête PO : retouche les 5 tokens px de U1.
- **Focus ring tokenisé** : `outline-width: var(--focus-w, 2px)` + `[data-contrast="high"]{ --focus-w:3px; --focus-offset:3px }` (vise 2.4.13).
- **Espacement 1.4.12 réellement tenu** : les `leading-*`/`tracking-*` bruts par composant **écrasent** `--ui-leading` → introduire des rôles `--leading-body`/`--leading-tight` tokenisés + **gate interdisant `leading-[…]`/`tracking-[…]` bruts**. Sinon l'override est cosmétique.
- **Guard RTL obligatoire** : `[dir="rtl"] .eyebrow, [dir="rtl"] [class*="tracking-"] { letter-spacing: normal }` (le letter-spacing casse le joining arabe).

### 3.2 PreferencesProvider (7 axes, 0-dep, CSP-safe par construction)

```ts
interface Preferences {
  locale: string                                    // 'fr'|'en'|'es'|'ar'|'zh'…
  theme:    'system'|'dark'|'light'
  contrast: 'system'|'normal'|'high'
  scale:    100|125|150|175|200
  spacing:  'normal'|'loose'
  font:     'default'|'hyperlegible'
  motion:   'system'|'full'|'reduced'
}
```

Application : **uniquement** via `documentElement.setAttribute('data-*', …)` + `.lang` + `.dir` (writes d'attribut, jamais `style=`). Aucun `setProperty` requis par défaut (les buckets `data-scale`/`data-spacing` portent `--ui-scale`) ; un slider continu optionnel passerait par `documentElement.style.setProperty('--ui-scale', v)` (CSSOM, **exempté de `style-src`**, donc CSP-safe). `system` résolu via `matchMedia` (`prefers-color-scheme`, `prefers-contrast`, `prefers-reduced-motion`, `prefers-reduced-transparency`) + listeners. Persistance `localStorage` clé unique JSON schema-versionnée (drop→défaut si invalide).

**Anti-FOUC (CSP-dur) — `/preboot.js` externe same-origin** : `<script src="/preboot.js">` bloquant dans `<head>` AVANT le module `main.tsx`. `default-src 'self'` (sans `script-src`) **autorise les scripts same-origin externes** ; l'inline est interdit (`operator_server.rs:345` « no inline scripts »). preboot lit localStorage et pose `dataset.*`/`lang`/`dir` **avant le 1er paint**. **Le serveur Operator doit servir `/preboot.js` en statique.** Déplacer le `data-theme="dark"` figé (`index.html:2`) dans ce preboot. → **Écarter `useLayoutEffect`** (peint après le bundle = flash visible 100-300 ms sur pref persistée ≠ défaut HTML).

### 3.3 i18n-comme-préférence (le locale est un axe de préférence)

- **`dir`/`lang`** câblés par `applyPrefs`. `directionOf(locale)` via `new Intl.Locale(locale).getTextInfo?.().direction` (natif, 0 dep) + fallback set statique RTL (`ar he fa ur ps sd ug yi`).
- **Catalogue maison** : `messages/fr.ts` (clé de référence pour le typage `Catalog = typeof import('./messages/fr').default`), entrées plates `id → string | {plural variants}`. Pluriels via `Intl.PluralRules(locale).select(n)` (les 6 catégories arabes `zero/one/two/few/many/other` supportées sans polyfill sur evergreen). `<Trans>` rich-text pour les cas « nombre = élément React séparé du mot » (`TokenCount … modifiés`, `OrientationBar.tsx:93,99`), guillemets/segments multi-nœuds.
- **Chargement** : `fr` eager (le moteur ~3-5 kB), autres locales en **`import(./messages/${l}.js)` lazy** (chunks same-origin autorisés par `default-src 'self'`). Le moteur i18n enveloppe `<App>` à la racine → **EAGER** : soit bumper le budget `app`, soit créer une entrée `vendor-i18n` (pattern `vendor-react=210`/`vendor-xterm=360`).
- **Formatage** : couche `lib/format.ts` (`fmtNumber/fmtDate/fmtPlural/fmtList`) mémoïsant les instances `Intl.*`. Remplace `sessionDate.ts:12` (`toLocaleString('fr-FR')` figé) par la locale active.
- **a11y partage le catalogue** : `aria-label`/`sr-only`/`role=status`/skip-link passent par le **même `t()`** (~45 chaînes a11y, sinon régression AAA/COGA).
- **Seams déjà keyés (réutiliser)** : `catalog/intentions.ts`, `catalog/surfaces.ts`, `lib/gateStatus.ts` (`gateStatusLabel`), `lib/verdict.ts` (`VERIFY_ETAT`), `Atelier.tsx:13-31` (`statusLabel`, à hisser en catalog). ~30 % des chaînes y sont déjà séparées du rendu. **Découpler `TerminalXterm.tsx:30`** (type-literal qui sert de label ET de clé `===` → clé enum stable + lookup label, avant toute traduction).

### 3.4 Contrat `<html>` + gates anti-régression

```
<html lang dir
  data-theme="dark|light" data-contrast="normal|high"
  data-scale="100|125|150|175|200" data-spacing="normal|loose"
  data-font="default|hyperlegible" data-motion="full|reduced">
```

Gates à ajouter dans `scripts/` (famille `scan-front-discipline.sh`/`check-no-tailwind-config.sh`) :
1. **Contrast gate** (Node 0-dep, ~50 lignes oklch→linéaire→WCAG) : parse les blocs token par mode, assert chaque paire (encre, surface) : texte AA 4.5 / **AAA 7** par mode, non-texte 3:1. **BLOQUANT**. Automatise les ratios manuels de U1 (provisoires sinon).
2. **i18n gate** (`scan-i18n-discipline.sh`, remplace le concept FR-only) : (a) interdit tout littéral user-facing hors `t()`, **y compris `aria-label`/`title`/`placeholder`/`sr-only`** ; (b) parité de clés cross-locale (manquante = fail).
3. **Anti-PASS étendu** : `scan-front-discipline.sh` scanne désormais **les VALEURS de `messages/*.ts` sur TOUTES les locales** (`PASS`/`Approved`/`Aprobado`/`通过`…), pas seulement la source TS.
4. **RTL/logical gate** : interdit `\b(pl|pr|ml|mr|left|right)-`, `text-left|text-right`, `border-l|border-r` dans les `className` (hors îlots `dir="ltr"` whitelistés) + vérifie le guard letter-spacing.
5. **Payload-font gate** : si vendoring non-latin/hyperlegible, entrée de poids dédiée (les woff2 échappent à size-limit).
6. **Playwright (T1 hermétique)** : screenshot par combinaison de mode × `dir=rtl` ; **T1 CSP étendue** : changement de locale (import dynamique) + application prefs = toujours 0 violation CSP, 0 inline-style.

---

## 4. Feuille de route (lots séquencés, fusion i18n + AAA/COGA + modes + L4-L8)

**Principe d'ordonnancement non négociable : l'extraction i18n vient AVANT d'ajouter de nouveaux libellés a11y.** L4 (icône+label), L5 (live regions), L6 (glossaire) **ajoutent des chaînes** ; les câbler en dur puis extraire double le travail. Donc le seam i18n se pose tôt, et les lots a11y en cours émettent via `t()` dès qu'il existe. **Décision sur les chaînes déjà committées (~240-260 messages distincts) : extraction one-shot vers `messages/fr.ts`** au Lot A (pas de réécriture, juste déplacement clé→valeur, en réutilisant les seams `catalog/*` + `lib/{verdict,gateStatus}` déjà keyés).

| Lot | Contenu | Fusion lots a11y | Effort |
|---|---|---|---|
| **A — Seam i18n + extraction** | Catalogue maison + `t()` + `<Trans>` + `lib/format.ts` (Intl) ; extraction des ~240-260 chaînes vers `fr.ts` ; découplage `TerminalXterm.tsx:30` ; nouveau `scan-i18n-discipline.sh` + anti-PASS multi-locale + parité clés ; `directionOf` + `dir`/`lang` dynamiques. **Locale unique FR à ce stade.** | Pré-requis de L4-L8 | **L** |
| **B — Fondation modes** | `PreferencesProvider` (7 axes) + `/preboot.js` externe (serveur Operator) + contrat `data-*` ; blocs de surcharge tokens (light/HC/scale/spacing/font/motion) ; focus ring tokenisé ; migration tokens typo **rem** (retouche U1) ; **contrast gate oklch→WCAG BLOQUANT** (lève le « provisoire » des ratios). | — | **M** |
| **C — Reflow AA (BLOQUEUR #1)** | Rail collapsible (drawer/top-bar sous breakpoint), retrait `App.tsx:48 overflow-hidden`, `h-7/h-12 → min-h-*` (`App.tsx:67`, `OrientationBar.tsx:66`), `max-w-[80ch]` sur la prose (`Atelier.tsx:75`). **Débloque toute revendication AAA.** | L7 (clavier) partiel | **M** |
| **D — RTL complet** | Sweep physique→logique (35 sites/12 fichiers : `ml-auto→ms-auto`, `border-l/r→border-s/e`, `text-left/right→text-start/end`, `App.tsx:54 left-3→start-3`) ; îlots `dir="ltr"` pour diff/terminal/cast (`DiffViewer`, `TerminalXterm`) ; neutralisation glyphes directionnels `▸/←/→` (séparateur bidi-neutre ou `rtl:rotate-180`) ; guard letter-spacing ; RTL gate. **+ locales ES + AR** (1ʳᵉ RTL). | — | **M/L** |
| **E — COGA façade + glossaire (L4+L6)** | Glossaire = **donnée** `catalog/glossary.ts` (`term→{label,definition}` par locale) ; primitive `<Glossaire term>` (button+`aria-expanded`, **PAS `title`**) ; remplacement des `title=` porteurs d'info (`OrientationBar:114`, `Mur:62`, `Atelier:96,107,117`) par disclosure/`aria-describedby` ; primitive `IconLabel` (force nom accessible, bannit glyphe seul/poly-sémique `◦`) ; **glyphes distincts `informational` vs `not_run`** (`gateStatus.ts:42-54`, aujourd'hui `•` partagé = distinction couleur-seule 1.4.1) ; bandeau **manuel** STEER→VERIFY (3.2.5, jamais auto-switch). | **L4 + L6** | **M** |
| **F — Live regions (L5)** | `aria-live="polite"` sur statut Atelier (`Atelier.tsx:58`) + jalons début/fin/erreur du flux SSE (`:69-88`, pas chaque delta) ; `cursorBlink` gaté reduced-motion (`TerminalXterm.tsx:54`, 2.2.2) ; sortie clavier terminal (Esc, 2.1.2) ; masquer `▌` (`Atelier.tsx:87`) du lecteur d'écran. | **L5** | **S/M** |
| **G — Panneau prefs + AAA opt-in** | `Dialog` Base UI « Accessibilité & langue » (depuis `OrientationBar`) câblant les 7 axes (libellés via `t()`) ; `data-pointer=large` (44px AAA, +2 fixes AA plancher 24px : refresh `OrientationBar:109`, toggle `Composer:105`) ; `data-contrast=high` ; `data-spacing=loose` ; `data-font=legible` (**Atkinson Hyperlegible** lazy + payload-font gate) ; `data-motion=reduced` in-app (`MotionProvider` `reducedMotion="always"` + `usePrefersReducedMotion` OR pref) ; bannir `focus:outline-none` 1px (`Composer:85,96`) → ring 2px partagé + gate lint. | L7/L8 | **M** |
| **H — Polices + formatage** | Fallbacks **nommés par script** dans `--font-sans` (`index.css:60` : PingFang/YaHei/Hiragino/Malgun/Noto CJK + `ui-sans-serif`) ; pluriels codés en dur → `Intl.PluralRules` (`OrientationBar`, `VerifyScene:145`) ; compteurs via `Intl.NumberFormat` (shaping AR-indien) en gardant `tabular-nums`. **+ locale ZH.** | — | **S** |
| **I — Vérification / wrap** | Playwright screenshots par mode × `dir` ; T1 CSP étendue (switch locale + prefs = 0 violation) ; `docs/DESIGN_SYSTEM.md` (tokens par mode, contrat `data-*`, API `usePreferences`, conventions catalogue, règles RTL) ; contrast gate vert tous modes. | U4/U5 wrap | **S/M** |

**Jalons :** J1 = A+B+C (seam i18n + fondation modes + Reflow AA réparé → AA-complet revendicable, FR seul) ; J2 = D+E+F (RTL + COGA + live regions → AR/ES/ZH ajoutables, AAA-COGA crédible) ; J3 = G+H+I (modes opt-in UI + polices + verification → AAA opt-in livré, doc + gates verts). **Codex (U5) groupé en fin de J3** (cohérent directive session « review GPT/Codex en un seul passage groupé après les ajouts »).

---

## 5. Décisions PO ouvertes

1. **Mécanisme i18n : maison-Intl (reco ferme) vs Lingui.** Maison = 3-5 kB RAW, 0-dep, aligné Day-0 D2, possession du formateur. Lingui = ~10 kB, ICU complet, pseudo-loc, **friction macro plus faible qu'annoncée** (Babel déjà présent : `@rolldown/plugin-babel` + `babel-plugin-react-compiler`, `package.json:40,50`). Trancher : posséder le code (maison) ou déléguer l'ICU (Lingui). *Reco : maison.*
2. **Set de langues v1.** Reco : FR+EN d'abord (J1), ES+AR+ZH en J2/J3. PO confirme le périmètre « large d'emblée » (toutes en v1) vs incrémental. Impact : chaque locale = revue éditoriale plain-language **par langue** (3.1.5 non automatisable), pas une MT.
3. **Vendoring polices CJK : NON (reco ferme).** Vendoring 1-3 MB exclu par CSP-self + payload ; CJK via fallbacks système nommés. PO confirme que l'Operator = outil local (fontes OS fiables).
4. **Vendoring RTL (Noto Arabic/Hebrew ~50-130 kB lazy) : optionnel/cosmétique.** Le système shape déjà correctement. Marque RTL souhaitée → vendoring lazy + payload-font gate ; sinon 0 kB.
5. **AAA en défaut vs mode (reco : opt-in).** 7:1 (1.4.6) et 44px (2.5.5) au défaut **détruisent** la hiérarchie tonale et la densité bi-focale. Reco : AA + 2.5.8 (24px) plancher toujours-actif, AAA via `data-contrast=high`/`data-pointer=large`. PO valide.
6. **Police dyslexie : Atkinson Hyperlegible (reco) vs OpenDyslexic.** Atkinson = base probante meilleure, aide aussi la basse vision, SIL OFL. **Latin-only** → fallback locale pour AR/ZH. OpenDyslexic = preuve d'efficacité contestée. *Reco : Atkinson, 1 face, lazy ; OpenDyslexic écarté.*
7. **Budget `app` : bumper de quelques kB vs chunk `vendor-i18n` dédié.** Marge actuelle ~1-5 kB ; le moteur i18n eager la dépasse. Décision d'architecture size-limit.
8. **Échelle typo : migration tokens px→`rem` (reco) vs `calc(var(--ui-scale))` dans `--text-*`.** rem honore aussi le zoom OS (1.4.4) mais retouche U1 récent (flag honnête). À confirmer empiriquement dans le CSS bâti.
9. **Couverture `@media (forced-colors: active)` (Windows High Contrast) : oui/non en v1.** Axe distinct de `data-contrast` ; sans lui, les bordures couleur-seule disparaissent en forced-colors. Reco : inclure (peu coûteux, system-color keywords).
10. **Reflow rail collapsible : forme UX** (drawer latéral vs top-bar sous breakpoint). Bloqueur AA, à designer (Claude Design ?).

---

## 6. Invariants préservés

- **0-verdict-calculé-UI / anti-PASS.** L'i18n ne déplace pas le calcul : `t()` ne fait que **restituer** des libellés ; les états restent issus du backend (`statusLabel`, `VERIFY_ETAT`, `gateStatusLabel`), jamais asserts. **Risque de fuite par traduction** → mitigé par le **scan anti-verdict sur les valeurs de TOUTES les locales** (Lot A) : `PASS`/`Approved`/`Aprobado`/`通过` interdits dans chaque `messages/*.ts`. L'aide COGA explique le **vocabulaire** (« qu'est-ce qu'un préflight/gate »), **jamais la VALEUR** d'un verdict (le `Mur` prouve la compatibilité) — à surveiller en revue (un tooltip ne doit pas glisser de « ce qu'est X » vers « X est réussi »).
- **Restitution 1:1.** Le catalogue est une couche de présentation pure (clé→libellé par locale) ; aucune logique métier n'y entre. Les modes (data-attributes) ne touchent que la présentation (couleurs/tailles/espacement/direction), jamais les données restituées.
- **CSP `default-src 'self'`.** Tenue par construction : (a) i18n maison = `Intl` natif, **zéro eval/`new Function`** (le seul coupable `@messageformat/core` n'est pas utilisé) ; (b) catalogues lazy = chunks **same-origin** (`import(./messages/${l})`) ; (c) preboot = `<script src>` **externe** same-origin (inline interdit) ; (d) prefs = `setAttribute`/`dataset`/`.lang`/`.dir` + CSSOM `setProperty` (exempté de `style-src`), **zéro `style=`/`<style>` injecté** ; (e) polices **same-origin vendorées** (`@fontsource`, `assetsInlineLimit:0`), CDN exclu. **T1 CSP étendue** asserte 0 violation sur switch locale + application prefs.
- **0-dep lourde (Day-0 D2 : Base UI seule primitive runtime).** Mécanisme i18n maison = **+0 dépendance** ; le panneau prefs réutilise le `Dialog` Base UI **déjà en dep** ; `Intl.*` est natif ; Atkinson Hyperlegible (si retenu) = **asset** vendoré, pas une dep runtime. react-intl/i18next écartés précisément pour préserver cet invariant + les budgets RAW. Les modes = **CSS pur** (surcharge de custom properties), zéro lib.
- **Gates `scan-front-discipline.sh` + discipline FR.** Conservés et **durcis**, pas contournés : le FR-only mute en « zéro littéral hors `t()` » (couvrant aria/title/placeholder/sr-only, sinon ~45 chaînes a11y non traduites = régression AAA/COGA) ; l'anti-PASS devient multi-locale ; parité de clés = build-fail. Le **niveau de lecture (3.1.5) reste une revue éditoriale humaine par langue** — ne pas sur-promettre un gate automatisé ; le seul mécanisable = complétude catalogue + « façade ⊆ glossaire ».

---

## Annexe A — Vérification adversariale (fait autorité)

Faits porteurs vérifiés dans le code. Je rends le rapport adversarial.

---

# RAPPORT ADVERSARIAL — vérification des claims i18n + a11y

Préambule : 3 faits porteurs confirmés en lecture directe, ils conditionnent tout le reste.
- **`.size-limit.json` : `"gzip": false, "brotli": false` sur les 7 entrées.** Budgets en **octets BRUTS minifiés**, pas gzip. `app=45 KB` / `css=27 KB` / `vendor-react=210 KB` / `vendor-xterm=360 KB`. Le cadrage « gzip vs 40-44 kB » du prompt est **faux** — c'est du RAW. Confirmé.
- **CSP = `default-src 'self'; connect-src 'self'`** (`operator_server.rs:354`), aucun `script-src`/`style-src`/`font-src` → tout retombe sur `'self'`, sans `'unsafe-inline'` ni `'unsafe-eval'`. Le commentaire `:345-346` dit explicitement « the greenfield front must ship without inline scripts ». Confirmé.
- **Babel EST déjà dans le build** : `@rolldown/plugin-babel ^0.2.3` + `babel-plugin-react-compiler ^1.0.0` + `@babel/core` (devDeps `package.json:37,40,50`). xterm = `^6.0.0`. Confirmé. → invalide partiellement la TENSION Lingui (cf. T5).

---

## TARGET 1 — CSP/eval des libs i18n

**Claim attaqué** : « les 4 candidats sont CSP-safe ; seul `messageformat`/`@messageformat/core` compile via `new Function` et casse ».

**[CONFIRME — avec montée de confiance]**. Décomposition par lib :
- **`@formatjs/intl-messageformat` (react-intl)** : v10+ = **interpréteur d'AST récursif-descendant**, zéro `new Function`/`eval` au runtime. CSP-safe par architecture, pas par configuration. Le build `…/no-parser` (pré-parse au build) est une optimisation de **taille (~10-13 kB)**, **PAS** une exigence CSP. Le doute « non vérifié ligne-source » de la dimension B doit être **rétrogradé** : c'est établi par l'architecture de la lib depuis des années, pas une incertitude.
- **i18next core** : interpolation par `String.replace`, pluriels par `Intl.PluralRules` natif (v23). Eval-free. `i18next-icu` tire `intl-messageformat` (AST, CSP-safe). CSP-safe.
- **Lingui v6** : `lingui compile` au **build** → catalogues = **tableaux de tokens** interprétés au runtime, jamais des fonctions. Même la compilation runtime optionnelle (`@lingui/message-utils`) produit un token-array via lexer moo, sans `eval`. CSP-safe.
- **Standalone `messageformat`/`@messageformat/core`** : compile l'ICU en JS via `new Function(...)` → **casse sous CSP self**. C'est le seul vrai coupable, et **aucun des 4 candidats ne l'emprunte par défaut**.

**Formulation à retenir** : « Tous les mécanismes envisagés (react-intl, i18next, Lingui, maison-Intl) sont eval-free et CSP-`self`-compatibles. Le différenciateur N'EST PAS la CSP mais la taille RAW + la doctrine 0-dep. Le seul chemin qui casse la CSP est la compilation-vers-fonction du package historique `@messageformat/core` (compile-to-`new Function`), que personne n'utilise ici. Le doute résiduel = spot-grep `new Function(`/`eval(` sur le `dist` SI une lib est retenue, mais c'est une formalité, pas un risque réel. » Pour le **maison-Intl**, la question disparaît entièrement (Intl natif).

---

## TARGET 2 — Poids réel vs budget 45 kB RAW + lazy locales

**Claim attaqué** : tableau des tailles + « catalogues lazy par locale ».

**[CONFIRME le verdict, CORRIGE deux chiffres + un raisonnement de découpage]**.

- **Le gate est RAW** (vérifié) → comparer à la colonne **min/RAW**, pas gzip. Cela ~triple le coût effectif vs le cadrage du prompt.
- **react-intl** : 58 kB RAW / ~17,6 gzip — **CONFIRME** (bundlephobia ~57-59 min). Même optimisé (precompile AST + no-parser) il reste **~40-45 kB RAW** = quasiment **tout le budget `app`**. → hors-jeu.
- **i18next + react-i18next** : la dimension B dit 76,7 kB RAW. **CORRIGE (légèrement haut)** : réel ≈ **65-77 kB RAW / 22-26 kB gzip** (i18next ~46 min + react-i18next ~21 min). Conclusion inchangée : **1,4-1,7× le budget hero**. → hors-jeu.
- **Lingui** ~8-12 kB RAW / 3-5 gzip — **CONFIRME**. Seule « vraie lib » qui tient.
- **Maison-Intl** « ~2 kB » — **CORRIGE** : sous-estimé. Un moteur COMPLET (sous-set ICU plural + interpolation + wrappers `Intl.NumberFormat/DateTimeFormat/RelativeTimeFormat/ListFormat` mémoïsés + Context provider + **un composant `<Trans>` pour rich-text**, cf. T5) ≈ **3-5 kB RAW**. Reste négligeable vs 45 kB.

**Erreur de raisonnement à corriger sur le découpage** : la dimension B suggère qu'un `vendor-i18n` séparé « n'allège pas le total, seulement le découpage » — **vrai et load-bearing**. Le provider i18n **enveloppe `<App>` à la racine → EAGER, chemin critique**, quel que soit le chunk. Un nouveau budget `vendor-i18n` (pattern déjà établi : `vendor-react=210`, `vendor-xterm=360`) **passe le gate par construction mais ne réduit pas le payload eager téléchargé au cold-start**. Donc le critère réel n'est pas « passer le gate » (gameable en ajoutant une entrée) mais **la croissance du JS eager total** : +58-77 kB pour react-intl/i18next, +8-12 kB Lingui, +3-5 kB maison.

**Piège supplémentaire non signalé** : le budget `app` est à **45 kB pour un chunk déjà ~40-44 kB → ~1-5 kB de marge seulement**. Même le maison (~3-5 kB), étant eager, **peut faire dépasser `app`** → il faudra soit bumper le budget `app` de quelques kB, soit créer un `vendor-i18n` dédié. Les **catalogues par locale sont lazy** (`import(./locales/${l}.js)`, chunks same-origin autorisés par `default-src 'self'`) dans TOUS les cas — seul le **moteur** est eager.

**Formulation à retenir** : « Gate RAW (gzip:false vérifié). react-intl (58 kB RAW) et i18next (~65-77 kB RAW) défoncent le budget eager total, le découpage en `vendor-i18n` masque le gate mais pas le payload. Lingui (~10 kB) et maison (~3-5 kB) sont les seuls viables. La marge `app` étant ~1-5 kB, même le maison exige soit un bump de quelques kB du budget `app`, soit un chunk dédié ; les catalogues restent lazy par locale. »

---

## TARGET 3 — Polices CJK/arabe sous CSP self

**Claim attaqué** : « polices SYSTÈME pour tout non-latin, 0 kB vendoré ; CJK via la cascade `--font-sans` ».

**[CONFIRME le cœur, CORRIGE le mécanisme + chiffre]**.

Chiffrage (woff2) :
- **CJK Noto SC/TC/JP/KR : 4-9 MB/famille** non-subsetté ; subset commun agressif **1-3 MB**. Sous CSP `'self'`, **impossible de lazy-loader depuis un CDN** → tout vendoring serait same-origin et MB-scale. → **vendoring CJK = exclu, non négociable**. La reco système HOLD = **TRUE**, et c'est la seule option réaliste.
- **Arabe (Noto Sans Arabic) ~80-130 kB** (subset ~40-100) ; **Hébreu ~30-60 kB**. → abordables, optionnels, lazy via `unicode-range` (le navigateur ne fetch que si des glyphes du range sont rendus).

**Correction de mécanisme** : la dimension C affirme que la cascade « rend DÉJÀ CJK/arabe via `system-ui` ». **Imprécis** : `system-ui`/`ui-sans-serif` résolvent vers la police UI de l'OS (Segoe UI, SF) qui **ne contient pas** les glyphes CJK — c'est le **fallback last-resort du navigateur** (HarfBuzz) qui sélectionne une police CJK système (Microsoft YaHei/Malgun sur Win, PingFang/Hiragino sur macOS, Noto CJK sur Linux). Résultat identique (le CJK s'affiche), mais **la reco doit ajouter des fallbacks NOMMÉS par script** dans `--font-sans` pour la cohérence inter-OS, pas se reposer sur `system-ui` seul :
`--font-sans: 'Geist Variable', 'PingFang SC','Microsoft YaHei','Hiragino Sans','Malgun Gothic','Noto Sans CJK SC', ui-sans-serif, system-ui, sans-serif`.

**Pour l'arabe/hébreu** : le fallback système **shape correctement** (joining géré par le moteur + police système, indépendamment de la marque) → **0 kB par défaut tient fonctionnellement**. Vendoring ~50-130 kB lazy = **purement cosmétique (marque)**, optionnel.

**Confirmé en lecture** : `.size-limit.json` ne cible que `*.js`/`*.css` → **les woff2 échappent au gate**. Donc (a) la reco système ne consomme aucun budget ✓, MAIS (b) **risque : un vendoring non-latin n'est gouverné par AUCUN gate** → si vendoring, ajouter une entrée size-limit (ou un check de payload font) explicite, sinon un subset CJK de 1 MB passerait inaperçu.

**Caveat** : `font-synthesis: none` (`index.css:94`) + police système sans la graisse demandée → glyphes non-latins `font-semibold` non synthétisés (rendu plus fin). Mineur, acceptable, à noter.

**Formulation à retenir** : « CJK = système obligatoire (vendoring 1-3 MB exclu par CSP-self + payload) ; ajouter des fallbacks NOMMÉS par script (le navigateur fait le last-resort, pas `system-ui`). Arabe/hébreu = système suffit fonctionnellement (shaping OK), vendoring ~50-130 kB lazy optionnel pour la marque. Les woff2 échappent à size-limit → ajouter un gate de payload font si vendoring. »

---

## TARGET 4 — Faisabilité AAA : 7:1 + 44px partout

**Claim attaqué** : « 7:1 (1.4.6) et 44px (2.5.5) partout casse la densité → opt-in modes ».

**[CONFIRME, et ÉLÈVE un point sous-pondéré]**. Par critère :
- **1.4.6 (7:1) → OPT-IN `[data-contrast="high"]`**. CONFIRME : la hiérarchie tonale « 3 encres lisibles + 1 ghost » est load-bearing ; 7:1 sur s3 agglutine tx2/tx3/tx4 vers le blanc et détruit le design. Défaut = AA (déjà livré L1).
- **2.5.5 (44×44) → OPT-IN `[data-pointer="large"]`**. CONFIRME : 44px partout détruit la densité bi-focale (rail compact, OrientationBar dense = valeur de design assumée). **Plancher AA toujours-actif = 2.5.8 (24×24, WCAG 2.2)**, atteignable avec ~2 fixes (refresh OrientationBar, toggle « détails techniques »).
- **Focus (2.4.13 AAA)** : CONFIRME le défaut `focus:outline-none` + bordure 1px (Composer textarea/select) = **défaut réel** (aire < 2px-perimeter, contraste limite) → bannir `outline-none` sans ring 2px de remplacement + gate lint.

**ÉLÉVATION (le point que les findings sous-pondèrent)** : **1.4.10 Reflow est un critère AA, PAS AAA, et il est EN ÉCHEC aujourd'hui** (rail fixe `w-[158px]` + `main` côte-à-côte + racine `overflow-hidden`, clipping à 320px/400 %). **On ne peut pas revendiquer une AAA cognitive/visuelle en échouant un critère AA fondamental.** → **le rail collapsible (drawer sous breakpoint) est le bloqueur #1, prioritaire sur tout mode AAA opt-in.** À mettre en tête du design doc, pas en note.

**A-VERIFIER (surconfiance des tables de contraste)** : **TOUTES les valeurs de ratio** des dimensions D (table L1 par défaut ET table HC candidate « VÉRIFIÉS ≥7:1 ») sont des **conversions oklch→sRGB→WCAG calculées à la main** par l'agent. La conversion (matrice OKLab + gamut-map + luminance relative) est **error-prone**. → **ne pas traiter ces nombres comme acquis** ; le mitigeant correct est exactement le **gate de contraste automatisé** (dimension F item #1, oklch→linaire→WCAG, BLOQUANT par mode). Tant qu'il n'existe pas, marquer tous les ratios « provisoires ».

**Formulation à retenir** : « 1.4.6 et 2.5.5 = modes opt-in (`data-contrast=high`, `data-pointer=large`) ; défaut AA + 2.5.8 (24px). MAIS le vrai bloqueur est 1.4.10 Reflow (AA) actuellement cassé → rail collapsible AVANT toute revendication AAA. Les ratios de contraste cités sont calculés à la main = provisoires jusqu'au gate automatisé. »

---

## TARGET 5 — Claims surconfiants / non étayés

**[CORRIGE] Anti-FOUC par `useLayoutEffect` (dimension tokens-modes)** — **contradiction inter-findings, tranchée**. La dimension tokens-modes propose d'appliquer les prefs dans `useLayoutEffect` à la racine du Provider. **FAUX/inférieur** : l'HTML initial (`data-theme="dark"` figé `index.html:2`) **peint le body AVANT** le téléchargement+parse du bundle et le mount React ; sur cold-start (100-300 ms), un utilisateur ayant persisté light/high-contrast voit un **flash visible**, pas « sub-frame ». La dimension D a **la bonne réponse** : **`/preboot.js` externe same-origin**, `<script src>` bloquant dans `<head>` avant le module. Vérifié faisable : `default-src 'self'` (sans script-src) **autorise les scripts same-origin externes** ; l'inline est interdit (commentaire `operator_server.rs:345` confirme « no inline scripts »). preboot.js lit localStorage et pose `documentElement.dataset.*`/`lang`/`dir` synchronement avant 1er paint. → **Retenir preboot.js, écarter useLayoutEffect** ; le serveur Operator doit servir `/preboot.js` en statique.

**[CORRIGE] TENSION Lingui « build oxc, pas de Babel » (dimension B)** — **partiellement faux**. Babel **est déjà dans le pipeline** (`@rolldown/plugin-babel` + `babel-plugin-react-compiler`, `package.json:40,50`). La macro Lingui s'enchaînerait via `babel-plugin-macros` dans la passe existante — friction = « configurer un plugin Babel de plus à côté de react-compiler », **pas** « introduire Babel from scratch ». La friction Lingui est donc **plus faible qu'annoncée** (mais le maison reste préférable sur taille + 0-dep).

**[CORRIGE] Maison « ~2 kB » + rich-text non traité** — la dimension A relève des cas `<TokenCount value/> modifiés` (nombre = élément React **séparé** du mot), guillemets/segments en N nœuds JSX. Un `t()` retournant une **string ne peut pas interpoler des éléments React**. Le maison doit livrer **AUSSI un `<Trans>`-like** (interpolation rich-text/composants), comme `<FormattedMessage>`/`<Trans>` des libs. C'est un coût réel sous-pondéré (+~1-2 kB, +complexité). Sans lui, il faut **restructurer** les ~5-10 cas rich-text. Budget honnête maison = **3-5 kB**, pas 2.

**[CONFIRME] `calc(20px * var(--ui-scale))` dans le namespace `--text-*`** — correctement flaggé « à vérifier » par tokens-modes. Le namespace font-size de TW v4 a un comportement spécial (line-height implicite). **Fallback plus robuste et standard = `html { font-size: calc(16px * var(--ui-scale)) }` + tokens typo en `rem`** (honore AUSSI le zoom OS/navigateur, 1.4.4). À privilégier sur le `calc` dans `--text-*`.

**[CONFIRME] Intl.PluralRules — arabe 6 catégories** : solide sur tout moteur evergreen (V8/SpiderMonkey/JSC). Aucun polyfill requis sur cible evergreen. Le doute « à vérifier AR 6 formes » peut être **rétrogradé** (supporté depuis des années).

**[CONFIRME] Pas d'audio (dimension E)** : xterm `^6.0.0` vérifié, `bellStyle` retiré du moteur, aucun `onBell`→`Audio` câblé → 1.2.x/1.4.2/1.4.7 = N/A. Résidu réel = `cursorBlink:true` non gaté reduced-motion (2.2.2) → fix trivial.

**[CONFIRME] Le reset CSS reduced-motion ne couvre pas WAAPI** (`element.animate`) → le triple-filet JS de `GateFlip`/`Reveal` + le court-circuit View-Transition de `altitudeShift.ts` sont **nécessaires**, pas redondants. Bien vu par les findings.

---

## 5 RISQUES / TENSIONS À METTRE EN AVANT DANS LE DESIGN DOC

1. **Bloqueur AA antérieur à toute AAA : 1.4.10 Reflow cassé** (rail fixe 158px + `overflow-hidden`). Le rail collapsible est le prérequis #1 ; aucune revendication AAA n'est crédible tant que ce critère **AA** échoue. À traiter avant les modes opt-in contraste/pointeur.

2. **Budget RAW (gzip:false) + provider eager = react-intl/i18next éliminés** (58-77 kB RAW), **maison (3-5 kB) ou Lingui (~10 kB) seulement**. Mais marge `app` ≈ 1-5 kB → même le maison exige un bump de budget `app` OU un chunk `vendor-i18n` dédié ; catalogues lazy par locale dans tous les cas. Décision : **maison-Intl + `<Trans>` rich-text**, budget honnête 3-5 kB, avec accommodation explicite du gate.

3. **Anti-FOUC = `/preboot.js` externe same-origin OBLIGATOIRE, pas `useLayoutEffect`** (inline interdit par CSP ; useLayoutEffect trop tardif → flash visible sur cold-start). Le serveur Operator doit servir `/preboot.js` avant le bundle ; il pose theme/contrast/scale/lang/dir avant 1er paint. C'est le SEUL chemin CSP-self sans flash.

4. **Polices : CJK = système obligatoire (vendoring 1-3 MB exclu par CSP-self), arabe/hébreu = système suffit (vendoring ~50-130 kB lazy optionnel, marque seule)**. Ajouter fallbacks NOMMÉS par script (le navigateur fait le last-resort, pas `system-ui`). **Les woff2 échappent à size-limit** → si vendoring (Atkinson Hyperlegible inclus), ajouter un gate de payload font, sinon dérive non gouvernée.

5. **L'invariant anti-PASS + le gate FR-only fuient par la traduction.** `scan-front-discipline.sh` (mots-verdict) et le concept FR-only doivent muter : (a) gate « zéro littéral user-facing hors `t()` » couvrant AUSSI `aria-label`/`title`/`placeholder`/`sr-only` (sinon ~45 chaînes a11y non traduites = régression AAA/COGA) ; (b) **scan anti-verdict sur les VALEURS de TOUTES les locales** (`PASS`/`Approved`/`Aprobado`/`通过`…), pas seulement la source TS ; (c) parité de clés cross-locale = build-fail. Tension annexe : les ratios de contraste cités (L1 + HC) sont calculés à la main → **provisoires** jusqu'au gate de contraste automatisé oklch→WCAG (BLOQUANT par mode), à construire en parallèle.
