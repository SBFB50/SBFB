# Étude d'intégration — Open CoDesign → SBFB Factory
## Ce qu'il y a à prendre, et comment l'implémenter au mieux (ultra-deep)

> **Statut** : recherche / conception. **Read-only** — aucun code écrit, aucune décision PO actée. Aucune dépendance ajoutée au lockfile.
> **Date** : 2026-06-22
> **Méthode** : synthèse de 6 workflows ultracode (sous-agents **Opus 4.8 1M**, vérification adversariale), lecture du **code réel** des deux côtés — Open CoDesign (OCD) `packages/{runtime,exporters,core}` + `apps/desktop`, et Factory `crates/sbfb-factory`, `crates/nexus-shell-daemon-core`, `tools/factory-{operator,ui}`. **Sprint 77 non touché.**
> **Cible** : OCD = `@open-codesign/*` v0.2.0, MIT, `Copyright (c) 2026 OpenCoworkAI Contributors`, monorepo Electron, mono-auteur (~84 %), `github.com/OpenCoworkAI/open-codesign`.
> **Portée** : que **prendre** d'OCD pour l'étage « design / maquette » de Factory, et comment l'**implanter** dans l'architecture souveraine SBFB (archétype **C** natif + vendorisation **B** patchée), sans dépendance runtime tierce.

---

## 0. Résumé exécutif (thèse)

**On ne « reprend » pas Open CoDesign — on en prend 3 unités portables MIT, on ré-incarne ses patterns nativement, et on scelle 100 % côté Factory.**

1. **Le fait structurant** : OCD *retire* la CSP de ses artefacts (`removeCspMetaTags`, host Electron permissif qui tire React/Babel/Fonts depuis le réseau) ; SBFB *impose* une CSP au niveau **header HTTP** (`connect-src 'none'`, origine opaque, COEP `require-corp`). « Prendre » n'est donc pas un portage mécanique mais une **re-souveraineté** : on garde la logique de classification/inlining, on jette **tout chemin réseau**.

2. **Ce qu'il y a à prendre** (et rien d'autre) :
   - le **classifieur de rendu** `classifyRenderableSource` (HTML-verbatim vs JSX) — TypeScript pur, 0 Electron/pi-ai ;
   - l'**inliner d'assets locaux** `inlineLocalAssetsInHtml` + table MIME (images **+ polices** woff/woff2/ttf/otf) — le joyau du scellage ;
   - la **couche-1 lint statique** de `done.ts` (synchrone, host-free).
   - **À LAISSER absolument** : `pi-ai`/`pi-agent-core`, le host Electron + Puppeteer (`done-verify.ts`, `rendered-html.ts`), `generate_image_asset` (cloud), le tuple exporters fermé, et **tous** les chemins réseau (Google Fonts hardcodées, `TAILWIND_CDN`, `<base href>`).

3. **L'aubaine maîtresse** : « *the workspace filesystem IS the deliverable* » est **déjà vrai gratuitement** chez SBFB — `provenance::compute_output_hash()` fait un tree-walk blake3 de **tout** le workspace sauf `factory.template.lock`/`factory.provenance.json`. Un design posé dans le source d'une app est **haché et signé Ed25519 (FG8) sans une ligne de code**.

4. **Comment l'implémenter** (archétype C natif) : convention `design/` dans le source ; `DESIGN.md` = DS per-app (instructions de goût, 0 exec) ; pipeline **tokens 2 couches** (source DTCG → CSS vars du shell *[référence]* + `sbfb-tokens.css` embarqué par app *[copie, car `connect-src 'none'` interdit la référence]*) ; self-check **ré-incarné sans Electron** (lint Rust + runtime rejoué dans l'iframe `blob_serve` via `postMessage`, sous la CSP **réelle** de prod) ; copilote via `ExecutionTarget::Ollama` (loopback keyless, jamais pi-ai) ; maquette-first = **gate processus**, pas un FGx bloquant de plus.

5. **Posture build-vs-partner** : **C/B maintenant**, alliance **différée/complémentaire avec kill-switch**. OCD n'a **aucune surface de plugin** (exporters = tuple fermé + chaîne `if` ; plugin-loading & MCP **différés post-1.0** ; API stable seulement à v1.0 ; mono-auteur = latence de merge non maîtrisable). Premier geste coopératif le moins risqué = publier un **`SKILL.md` « goût SBFB »** (0 merge cœur, 0 lockfile, rollback gratuit).

6. **Le scellage n'est jamais délégable** : CSP, COEP/COOP, FG5/FG6/FG8, signature Ed25519 = invariants souverains. Aucune dispense n'est accordée à un artefact « parce qu'il vient d'OCD ».

---

## 1. Cadre de décision

**Impédance centrale.** OCD est une app desktop **Electron MIT, mono-utilisateur, productrice d'artefacts de design** (proto/slides/UI kits, *pas* d'apps full-stack), bâtie sur **Node/pnpm/Turborepo** et l'écosystème `pi` d'un seul auteur. SBFB Factory est un **Operator backend Rust** (Axum loopback `127.0.0.1:3001`) + une **UI Vite/React** + un **socle readonly** lecture-pure, qui publie des **apps zip self-contained rendues en iframe sandbox souverain** : CSP `connect-src 'none'`, COEP `require-corp`, `sandbox allow-scripts` sans `allow-same-origin` (origine opaque), **0 CDN runtime, fonts embarquées**.

L'écart n'est pas une dette à corriger, c'est une **frontière de doctrine** : souveraineté (NO third-party RUNTIME, authoring build-time toléré), solo-maintainer (modèle OpenBSD), pivot Node arbitré (Option **A** zéro-Node / **B** éphémère opt-in / **C** persistant **REJETÉ**).

**Principes déjà actés** (à ne pas re-débattre) :
1. **Le workspace filesystem EST le livrable** — et chez SBFB le design **doit vivre dans le repo source** pour être couvert ; c'est *mécaniquement vrai* via le tree-walk blake3.
2. **Unifier par les TOKENS, format natif par surface** ; 2 couches DS (shell global vs per-app embarqué).
3. **Maquette-first** (S70) : prompt UX → handoff repo-visible **AVANT** code — gate *processus*, pas gate *pipeline*.

L'intégration est donc **en amont d'authoring**, jamais un remplacement du viewer ni du modèle d'app SBFB.

---

## 2. Ce qu'il y a à prendre (inventaire au niveau code)

Cette section descend au niveau fonction sur chaque unité candidate d'Open CoDesign (OCD, `@open-codesign/*` v0.2.0, MIT, `Copyright (c) 2026 OpenCoworkAI Contributors`). Pour chacune : verdict (PRENDRE / S'INSPIRER / LAISSER), portabilité réelle (dépendances Electron / `pi-ai` / `node:*`), implication licence, et **le patch précis** requis pour que l'unité survive sous la souveraineté SBFB (CSP `default-src 'self'` + `connect-src 'none'`, COEP `require-corp`, zéro CDN, fonts embarquées — `crates/nexus-shell-daemon-core/src/blob_serve.rs:286-290`).

### 2.0 Fait structurant : OCD *retire* la CSP, SBFB l'*impose*

Avant tout inventaire, une asymétrie de doctrine doit cadrer chaque décision. La toute première opération de `buildPreviewDocument` / `buildStandaloneDocument` est `removeCspMetaTags(userSource)` (`packages/runtime/src/index.ts:599` et `:646`), implémentée dans `packages/shared/src/html-utils.ts:272-292` : elle supprime activement tout `<meta http-equiv="content-security-policy">` du source agent. OCD assume un host *permissif* (Electron + Chrome système) où le réseau sortant est ouvert : ses documents générés tirent React/Babel/Fonts depuis le réseau. SBFB fait l'exact inverse — il **injecte** une CSP au niveau *header HTTP* sur chaque réponse blob, avec `connect-src 'none'`, `frame-src 'none'`, `object-src 'none'`, `base-uri 'none'`, `form-action 'none'`, `sandbox allow-scripts` (sans `allow-same-origin`, donc origine opaque), `COEP: require-corp`, `COOP: same-origin` (`blob_serve.rs:286-290`). **Conséquence transversale** : tout artefact OCD non patché est *cassé par construction* sous la CSP SBFB (fonts externes bloquées par `default-src 'self'`, `<base href>` neutralisé par `base-uri 'none'`). Le travail de "prise" n'est donc pas un portage mécanique mais une *re-souveraineté* : on garde la logique de classification/inlining, on jette tout chemin réseau.

### 2.1 `classifyRenderableSource` + `buildPreviewDocument` (`packages/runtime/src/index.ts`)

**Verdict : PRENDRE le classifieur, S'INSPIRER du pipeline preview, PATCHER lourdement le wrapper JSX.**

Le cœur réutilisable est le triage déterministe `classifyRenderableSource(source, path)` (`index.ts:76-87`) : ordre de priorité (1) `looksLikeFullHtmlDocument` — `head.startsWith('<!doctype') || '<html'` sur les 2048 premiers caractères (`index.ts:61-64`) → `'html'` **rendu verbatim** ; (2) extension `.tsx`/`.jsx` ; (3) heuristique `looksLikeJsxSource` (`index.ts:66-74` : marqueurs `AGENT_BODY_BEGIN`, `EDITMODE-BEGIN`, `ReactDOM.createRoot`, déclaration `App`/`_App` via `containsNamedDeclaration`, ou balise majuscule via `containsUppercaseJsxTag`) ; (4) `.html` ; (5) `'unknown'`. C'est exactement la frontière "HTML-verbatim vs wrap-JSX" qui est l'archétype B à vendoriser.

Le mode mixte est géré par `needsJsxRuntimeInHtml` (`index.ts:412-423`) : un document HTML qui contient `<script type="text/babel">` (détecté par `hasTextBabelScript`, `index.ts:425-432`), `ReactDOM.createRoot`, `React.createElement`, ou les noms de composants hardcodés `IOSDevice` / `DesignCanvas` / `AppleWatchUltra` / `AndroidPhone` / `MacOSSafari` reçoit la stack runtime ; sinon il passe verbatim (coût inline nul — le commentaire `index.ts:403-411` documente précisément ce raisonnement). **À retenir mais nettoyer** : la liste de noms de composants est un couplage dur au design-system OCD, à ne pas importer tel quel.

**Portabilité** : le package `@open-codesign/runtime` ne dépend QUE de `@open-codesign/shared` (`packages/runtime/package.json` : `dependencies: { "@open-codesign/shared": "workspace:*" }`) — **aucun `node:*`, aucun `electron`, aucun `pi-ai`**. Les deux helpers shared utilisés (`removeCspMetaTags` dans `html-utils.ts`, `ensureEditmodeMarkers` dans `editmode.ts:207`) sont des fonctions de string pures sans import `node:` (vérifié). C'est l'unité **la plus portable** d'OCD : le classifieur peut être ré-incarné en Rust (`template_engine.rs` voisin) ou en TS-navigateur sans toucher au host.

**Le piège dur — Google Fonts hardcodées.** Le chemin JSX (et *seulement* le chemin JSX, pas le chemin HTML-verbatim) injecte trois `<link>` réseau **identiques** dans `wrapJsxAsSrcdoc` (`index.ts:350-352`) ET `wrapJsxAsStandaloneDocument` (`index.ts:383-385`) :
- `<link rel="preconnect" href="https://fonts.googleapis.com" />` (`:350` / `:383`)
- `<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />` (`:351` / `:384`)
- `<link href="https://fonts.googleapis.com/css2?family=Fraunces:...&family=DM+Serif+Display:...&family=DM+Sans:...&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet" />` (`:352` / `:385`)

Ces trois familles (Fraunces, DM Serif Display, DM Sans, JetBrains Mono) sont *baked* dans le `<style>` suivant (`body{font-family:'DM Sans',...}`, `:353` / `:386`). Sous CSP SBFB, `default-src 'self'` bloque les trois `<link>` ; les fonts ne se chargent jamais et le fallback `system-ui` s'applique silencieusement. **Patch souverain** : (a) supprimer les trois `<link>` réseau ; (b) vendoriser les `.woff2` des familles retenues en assets locaux du template, ou les inliner en data-URI via le mécanisme §2.2 ; (c) réécrire le `font-family` du `<style>` sur les fonts embarquées. C'est précisément ce que SBFB fait déjà côté React-template (`crates/sbfb-factory/src/templates/react/index.html:8-14` : "React 18 UMD, vendored same-origin (NO CDN, NO build, NO runtime fetch)").

**Le deuxième piège — `<base href>`.** `buildPreviewDocument` injecte un `<base href>` via `baseTag()` (`index.ts:243-245`, appelé `:350`/`:383`/`:603`/`:618`) pour résoudre les assets relatifs. Sous CSP SBFB, `base-uri 'none'` (`blob_serve.rs:286`) **neutralise** tout `<base>`. **Patch** : ne pas porter l'injection `baseHref` ; la résolution d'assets se fait à l'écriture-archive (inlining data-URI ou chemins relatifs d'archive), pas au runtime via `<base>`.

**Les blobs UMD.** Le wrapper assemble `<script>${REACT_UMD}</script>`, `<script>${REACT_DOM_UMD}</script>`, `<script>${BABEL_STANDALONE}</script>` (`jsxRuntimeBaseScripts`, `index.ts:328-335`) où `REACT_UMD`/`REACT_DOM_UMD`/`BABEL_STANDALONE` sont des imports `?raw` de copies **vendorisées** (`index.ts:24-28` : `../vendor/babel.standalone.js?raw`, etc.) — donc déjà same-origin, pas de CDN. Bonne nouvelle pour la souveraineté : ces blobs sont inline, conformes à `default-src 'self' 'unsafe-inline' 'unsafe-eval'`. **Nuance licence** : ces vendor blobs sont React (MIT) / Babel (MIT) sous *leur* copyright, distincts du copyright OCD — toute reprise doit conserver leurs en-têtes de licence respectifs (le test SBFB `template_engine.rs:574` vérifie déjà "React UMD license header preserved").

**Implication licence MIT** : la reprise de `classifyRenderableSource` + helpers exige de conserver une notice MIT `OpenCoworkAI Contributors` dans `THIRD-PARTY-NOTICES.md` (modèle existant : entrées "vendored + forked" type llama.cpp). SBFB étant AGPL-3.0-or-later, MIT est compatible en aval ; le patch SBFB est releasé AGPL, l'origine MIT préservée.

### 2.2 `inlineLocalAssetsInHtml` / `isLocalReference` + table MIME (`packages/exporters/src/assets.ts`)

**Verdict : PRENDRE (c'est le joyau pour le scellage souverain) — patch minimal.**

C'est l'unité la plus directement alignée avec la doctrine "fonts embarquées, zéro CDN". `inlineLocalAssetsInHtml` (`assets.ts:46-58`) inline en data-URI les assets **locaux** référencés en `src`/`href`/`poster` (`RESOURCE_ATTR_RE`, `assets.ts:30`), `srcset` (`assets.ts:31`), et `url()` CSS (`URL_FUNC_RE`, `assets.ts:29`), avec descente **récursive** dans les CSS imbriqués (`readReferenceAsDataUri` → `inlineCssUrls` → `collectCssUrlReplacements`, `assets.ts:284-321`, garde anti-cycle via `seen: Set<string>`).

**La frontière locaux/distants — exactement ce que demande SBFB.** `isLocalReference(raw)` (`assets.ts:394-399`) est la garde clé :
```
if (!value || value.startsWith('#') || value.startsWith('//')) return false;   // ancres + protocol-relative
if (/^[a-zA-Z][a-zA-Z0-9+.-]*:/u.test(value)) return false;                    // tout schéma URL (http:, https:, data:, file:)
return true;
```
Donc : `http(s)://`, `//cdn...` (protocol-relative) et les ancres `#` sont **exclus → laissés EXTERNES**, tels quels dans la sortie. Seuls les chemins relatifs/absolus-workspace sont inlinés. Containment renforcé par `resolveAssetReference` (`assets.ts:372-392`) qui résout sous `root.rootPath` et rejette tout `archivePath.startsWith('../')` (`assets.ts:390`) + `isInsideRoot` (`assets.ts:421-424`). C'est un anti-traversal robuste, cohérent avec FG5 SBFB.

**Table MIME** (`mimeForPath`, `assets.ts:446-486`) : images `.avif`→image/avif, `.bmp`→image/bmp, `.gif`, `.jpg`/`.jpeg`→image/jpeg, `.png`, `.svg`→image/svg+xml, `.webp` ; **fonts** `.otf`→font/otf, `.ttf`→font/ttf, `.woff`→font/woff, `.woff2`→font/woff2 ; texte `.css`/`.js`/`.mjs`/`.json`/`.html`/`.htm`/`.txt` ; fallback `application/octet-stream`. Le set `TEXT_ENCODINGS` (`assets.ts:33-44`) décide encodage URL-text vs base64 (`toDataUri`, `assets.ts:437-440`). C'est exactement la table couvrant le besoin "fonts embarquées" (woff/woff2/ttf/otf) qui résout §2.1.

**Portabilité** : `assets.ts` importe `node:path` (`:1`) en statique et `node:fs/promises` en dynamique (`:64`, `:290`) — **dépendant Node** mais aucun Electron, aucun `pi-ai`. La logique de regex/replacement/MIME est pure ; seuls les `fs.readFile` touchent le disque. Le package `@open-codesign/exporters` traîne par ailleurs des deps lourdes (`puppeteer-core ^24`, `pptxgenjs ^4`, `zip-lib`) **mais elles sont lazy-loadées** (`import()` dynamiques, cf. `index.ts:41-68`) ; `assets.ts` n'en charge aucune. On peut donc extraire `assets.ts` quasi seul.

**Patch souverain** : (1) `isLocalReference` est *trop permissive* pour SBFB — elle "laisse externe" `http(s)://` au lieu de les **rejeter**. Sous CSP `connect-src 'none'` + `default-src 'self'`, un asset externe laissé tel quel ne charge jamais : il faut transformer le comportement "laisser externe" en *erreur de gate* (refus du draft, cohérent avec `operator_server.rs` `/api/artifacts/draft` qui rejette déjà des contenus). (2) Forcer `inlineLocalAssets` à **on** systématique (équivalent du défaut `?? true` côté `exportHtml`, `html.ts:43`) et faire de l'inlining une étape **obligatoire** du scellage, pas une option. (3) La table MIME se reprend telle quelle. **Licence** : MIT, notice à conserver.

### 2.3 `html.ts` → `buildStandaloneDocument` + le piège `injectTailwind` / CDN

**Verdict : S'INSPIRER de `buildHtmlDocument` ; PRENDRE `buildStandaloneDocument` (runtime) ; LAISSER absolument `injectTailwind`.**

`buildStandaloneDocument` (`runtime/index.ts:642-659`) est la variante "fichier autonome" de `buildPreviewDocument` (mêmes verdicts §2.1, sans l'overlay d'édition). `buildHtmlDocument` (`html.ts:51-70`) l'enveloppe : shell doctype + bannière + `<meta name="generator">`, puis prettify maison (`prettifyHtml`, `html.ts:128-198`, volontairement sans `prettier`/`js-beautify` pour le budget deps — `html.ts:123-127`).

**Le piège CDN.** `html.ts:12-13` définit `TAILWIND_CDN = 'https://cdn.tailwindcss.com'` et `TAILWIND_TAG = <script src="...">`. `buildHtmlDocument` injecte ce script si `opts.injectTailwind` est vrai et qu'aucun script Tailwind n'existe (`html.ts:63-65`). Le défaut est **`false`** (`html.ts:52` : `opts.injectTailwind ?? false`, "Defaults to false for offline exports", `html.ts:16`) — donc l'export HTML "à plat" est offline-safe par défaut. **MAIS** le chemin de rendu navigateur force le défaut à **`true`** : `rendered-html.ts:22-30` (`buildExportHtmlDocument`) passe `injectTailwind: opts.injectTailwind ?? true`. Sous CSP SBFB, ce `<script src="https://cdn.tailwindcss.com">` est bloqué par `default-src 'self'` → page sans styles Tailwind, silencieusement. **Patch souverain** : ne jamais importer `rendered-html.ts` (cf. §2.6) ; si Tailwind est requis, vendoriser un build Tailwind statique same-origin (équivalent doctrine du React-template SBFB). Traiter `injectTailwind` comme interdit dans tout chemin de scellage. C'est le piège le plus insidieux car le défaut diffère entre deux call-sites.

**Portabilité** : `html.ts` importe `node:fs/promises` dynamiquement (`html.ts:41`) ; le reste est pur. Dépendant Node, pas Electron/pi-ai.

### 2.4 `done.ts` : couche-1 lint statique (PURE, à PRENDRE) vs couche-2 `DoneRuntimeVerifier` (Electron/Chrome, NON portable, à RÉ-INCARNER)

**Verdict : PRENDRE intégralement la couche-1 (pure) ; RÉ-INCARNER la couche-2 dans le viewer sandbox SBFB (iframe + postMessage), surtout pas Electron/Puppeteer.**

`makeDoneTool` (`done.ts:450-543`) compose les erreurs en deux strates. **Couche-1 — lint statique pur** (`done.ts:498-506`) : `findJsxStructuralIssues` (`done.ts:307-448` : balance parenthèses/accolades/crochets avec machine à états chaînes/commentaires `done.ts:330-382`, détection fence markdown résiduel `done.ts:321-328`, exigence d'ancres `ReactDOM.createRoot` + `function App()` `done.ts:403-418`, détection contenu parasite après le dernier `.render()` `done.ts:423-445`), `findUnclosedTags` (`done.ts:143-175`, stack de balises + `VOID_ELEMENTS`), `findDuplicateIds` (`done.ts:177-202`), `findMissingAlt` (`done.ts:204-216`), `findBrokenHashLinks` (`done.ts:218-253`, vérifie que chaque `href="#x"` cible un `id` existant), plus la validation `DESIGN.md` (§2.5). **Tout cela est synchrone, déterministe, host-free** — le commentaire d'en-tête le dit explicitement (`done.ts:5-7` : "Cheap and host-free; runs in every environment"). **Aucun `node:*`, aucun Electron.** Seule dépendance : `@mariozechner/pi-agent-core` pour les *types* `AgentTool`/`AgentToolResult` (`done.ts:17`) — un import de type effaçable, et `validateDesignMd` depuis `@open-codesign/shared` (`done.ts:18`). C'est portable vers Rust ou TS pur sans friction. **À PRENDRE** comme socle d'un gate "design" SBFB (qui n'existe pas aujourd'hui : `gates.rs`/`pipeline.rs` couvrent FG4-FG8 mais aucun gate "design").

**Couche-2 — le runtime verifier, NON portable.** Le type `DoneRuntimeVerifier = (artifactSource) => Promise<DoneError[]>` (`done.ts:124`) est *injecté par le host* et n'est exécuté que si fourni et si le path est rendable (`done.ts:507-517`). Son incarnation réelle est `apps/desktop/src/main/done-verify.ts` : `makeRuntimeVerifier()` (`done-verify.ts:217-231`) wrappe le source via `buildSrcdoc` (`done-verify.ts:219`), écrit un fichier temp, puis `verifyWithSystemChrome` (`done-verify.ts:157-215`) lance **Puppeteer + Chrome système** (`puppeteer.launch({ headless, args:['--headless=new',...] })`, `done-verify.ts:167-178`), intercepte les requêtes (`page.on('request')` + `isDoneVerifierRequestAllowed` qui autorise `http:`/`https:`/`data:`/`blob:`/`file:`==fichier-verify, `done-verify.ts:56-79`), capture `console` (`done-verify.ts:185-191`) + `pageerror` (`done-verify.ts:192-194`) sur une fenêtre de settle (`SETTLE_AFTER_LOAD_MS = 1200`, `done-verify.ts:27` ; `VERIFY_LOAD_TIMEOUT_MS = 15000`, `done-verify.ts:26`). **Dépendances rédhibitoires** : `puppeteer-core` + Chrome système (`findSystemChrome` depuis `@open-codesign/exporters`, `done-verify.ts:22`) ; le commentaire `done.ts:8-9` mentionne aussi une variante "hidden Electron BrowserWindow". Couplage dur au host desktop. Le cap de réparation `MAX_DONE_ERROR_ROUNDS = 3` vit côté agent (`packages/core/src/agent.ts:341`, boucle `agent.ts:676-700`), avec auto-repair via `str_replace_based_edit_tool`.

**Patch / ré-incarnation souveraine** : SBFB ne doit **pas** importer `done-verify.ts` ni Puppeteer/Electron. La couche-2 se rejoue dans le **viewer sandbox SBFB** déjà existant — l'iframe servie sous CSP `blob_serve.rs:286` (`sandbox allow-scripts`, origine opaque) — en captant `window.onerror`/`console.error` via `postMessage` vers l'opérateur, exactement le pattern `sbfb-bridge.js` du React-template. On obtient la même boucle "exécute → collecte erreurs → auto-repair ≤3 rounds" *sans aucun host natif*, et sous une CSP **plus stricte** que celle d'OCD (qui, elle, autorise même `http(s)://` dans son intercepteur — `done-verify.ts:66-68`). Note de sécurité : l'intercepteur OCD autorise large par design (host permissif) ; la version SBFB hérite gratuitement de `connect-src 'none'`.

### 2.5 Le PATTERN `DESIGN.md` + skills (à RÉ-INCARNER comme processus, pas comme code)

**Verdict : S'INSPIRER — c'est un pattern de gouvernance, zéro code à porter.**

`DESIGN.md` est un *handoff system* exigé par `done` : `requiredDesignMdErrors` (`done.ts:109-119`) émet une erreur bloquante si un design source "user" (filtrage `isUserDesignSourcePath`, `done.ts:63-76`, qui exclut `frames/`, `skills/`, `_starters/`, `assets/`) est finalisé sans `DESIGN.md` ; `designMdWorkspaceErrors` (`done.ts:87-107`) force un `DESIGN.md` dès qu'il y a >1 source rendable (multi-screen). La validation de contenu passe par `validateDesignMd` (depuis `@open-codesign/shared`, `done.ts:18`,`:78-85`) qui exige version/name/colors/typography/rounded/spacing + Overview (message `done.ts:113-116`). C'est l'instrument "tokens-first / DTCG-compatible" évoqué dans la doctrine SBFB (unifier par tokens). **Le ZIP exporter** l'embarque automatiquement (`zip.ts:96-103` : lit `DESIGN.md` à la racine workspace et l'ajoute à l'archive + au manifest).

Les **skills** (`skill.ts`, `tools/skill.ts:223-295`) sont des fiches markdown de *goût* (`form-layout`, `empty-states`, `surface-elevation`, `brand:<slug>`…) chargées à la demande avec dé-dup par session (`done.ts`/`skill.ts:179-181`) ; frontmatter typé (`SkillManifestEntry`, `skill.ts:20-29` : name/category/aliases/dependencies/validationHints). **Zéro exécution** : `invokeSkill` (`skill.ts:173-197`) ne fait que `readFile` la fiche après garde anti-symlink (`resolveSafeManifestPath`, `skill.ts:45-71`). C'est de la donnée + un protocole, pas de la logique. **Ré-incarnation SBFB** : reproduire le *contrat* (un `DESIGN.md` DTCG obligatoire au scellage, des skills-fiches read-only) dans le `maquette-first S70` (gate processus, pas pipeline) et le 2-couches DS (shell global vs per-app embarqué). Rien à vendoriser ; juste à adopter la convention. **Bonus souverain gratuit** : un `DESIGN.md` dans le source EST déjà hashé par `provenance.rs::compute_output_hash()` (tree-walk blake3 de tous fichiers sauf lock/provenance) — le handoff design devient *provenance-signé Ed25519* (FG8, DOMAIN_PROVENANCE_V1) sans effort.

### 2.6 `zip.ts` / `rendered-html.ts`

**`zip.ts` — Verdict : S'INSPIRER de la logique de réécriture+containment ; LAISSER la dépendance `zip-lib` (SBFB a son propre `zip`).**

`exportZip` (`zip.ts:60-181`) produit un bundle déterministe : `index.html` racine, `assets/` (collectés via `collectLocalAssetsFromHtml` §2.2), `source/<path>`, `manifest.json` (schemaVersion+files triés, `zip.ts:124-136`), `DESIGN.md` si présent (`zip.ts:96-103`), README. La réécriture des refs vers chemins d'archive est `rewriteHtmlLocalAssetReferences` (`assets.ts:105-162`) — alternative *non-inline* au data-URI (utile si on préfère des fichiers séparés dans l'archive plutôt que tout inliner). Le containment anti-traversal est exemplaire et **explicitement Windows-aware** : normalisation `\`→`/` *avant* `path.resolve` pour bloquer `..\..\etc\passwd` (`zip.ts:148-160`, lève `EXPORTER_ZIP_UNSAFE_PATH`) ; `sourceArchivePath` (`zip.ts:183-197`) re-valide segments. **À s'inspirer** pour le packaging d'archive SBFB, mais l'unité dépend de `zip-lib` (lazy, `zip.ts:68`) que SBFB remplace par son crate `zip` natif (`blob_serve.rs:305` utilise `zip::ZipWriter`). On reprend la *discipline de réécriture/containment*, pas le code zip.

**`rendered-html.ts` — Verdict : LAISSER intégralement.** `renderArtifactBodyHtml` (`rendered-html.ts:54-92`) lance **Puppeteer + Chrome** pour pré-rendre le DOM statique (`puppeteer.launch`, `:65-76` ; `page.setContent`, `:79`). Pire, `buildExportHtmlDocument` (`rendered-html.ts:22-35`) force `injectTailwind ?? true` (§2.3) → CDN. Double rédhibitoire (host Chrome + CDN). Aucun intérêt sous SBFB : le viewer sandbox rend déjà le DOM dans l'iframe servie.

### 2.7 Ce qu'il NE faut PAS prendre

| Unité OCD | Pourquoi LAISSER |
|---|---|
| `@mariozechner/pi-ai` / `pi-agent-core` | Règle dure OCD = "tout LLM via pi-ai". SBFB a sa propre couche `provider_router.rs` (ExecutionTarget Claude/Ollama-loopback-keyless/Network, `StreamChunk` unifié) et la souveraineté NO third-party *runtime*. Ne garder que les *types* `AgentTool` effaçables. |
| Host Electron / `apps/desktop/*` (`done-verify.ts`, `electron-runtime.ts`, IPC) | Couplage natif Electron+Puppeteer+Chrome système. Viole "zéro host natif" ; la couche-2 done se rejoue en iframe sandbox (§2.4). |
| `generate_image_asset` (`tools/generate-image-asset.ts`) | Cloud-only opt-in (`GenerateImageAssetFn` retourne un asset via provider cloud). Génération d'image réseau incompatible avec la posture offline/`connect-src 'none'`. À exclure ou ré-incarner local-only plus tard. |
| Tuple exporters fermé + chaîne `if` | `EXPORTER_FORMATS = ['html','pdf','pptx','zip','markdown']` (`exporters/index.ts:13`) + chaîne `if` dans `exportArtifact` (`exporters/index.ts:45-74`) = ajout de format = PR cœur, pas plugin. C'est un *choix* de gatekeeping OCD (assumé : plugins/MCP différés post-1.0), pas un asset à porter ; SBFB a déjà `template_engine.rs` (4 `TemplateConfig`) côté scaffold. Reprendre la *philosophie* (formats fermés, audités), pas la structure. |
| Chemins réseau : `TAILWIND_CDN` (`html.ts:12`), Google Fonts `<link>` (`runtime/index.ts:350-352`,`383-385`), `<base href>` (`baseTag`, `runtime/index.ts:243`) | Tous bloqués/neutralisés par CSP SBFB. À supprimer, pas à porter. |

### 2.8 Tableau de synthèse

| Unité (file:line) | Prendre / Laisser | Portable ? (Electron / pi-ai / node:*) | Deps exactes | Patch souverain requis |
|---|---|---|---|---|
| `classifyRenderableSource` + helpers (`runtime/index.ts:76-87`, `61-74`, `412-423`) | **PRENDRE** | ✅ Aucun host ; aucun `node:*` ; QUE `@open-codesign/shared` (string-pur) | `@open-codesign/shared` (`html-utils`,`editmode`) | Retirer la liste de noms de composants hardcodés (`IOSDevice`…) si non pertinents ; sinon tel quel |
| `buildPreviewDocument` / `wrapJsxAsSrcdoc` (`runtime/index.ts:595-624`, `337-370`) | **S'INSPIRER** (pipeline) + **PATCHER** (wrapper) | ✅ pas de host ; mais injecte réseau | shared + blobs UMD `?raw` | **Supprimer** 3 `<link>` Google Fonts (`:350-352`) → vendoriser woff2 + réécrire `font-family` ; **supprimer** injection `<base href>` (`base-uri 'none'`) ; garder UMD inline (conforme `unsafe-inline`/`unsafe-eval`) |
| `buildStandaloneDocument` (`runtime/index.ts:642-659`) | **PRENDRE** (patché idem) | ✅ | shared + UMD `?raw` | Idem fonts/base que §2.1 ; pas d'overlay |
| `inlineLocalAssetsInHtml` + `isLocalReference` + table MIME (`exporters/assets.ts:46-58`,`394-399`,`446-486`) | **PRENDRE** (joyau) | ⚠️ `node:path`+`node:fs/promises` ; pas Electron/pi-ai | `node:*` seulement | Rendre l'inlining **obligatoire** au scellage ; transformer "laisser externe http(s)://" en **refus de gate** ; table MIME telle quelle |
| `html.ts buildHtmlDocument` + `injectTailwind`/`TAILWIND_CDN` (`exporters/html.ts:51-70`,`12-13`,`63-65`) | **S'INSPIRER** shell ; **LAISSER** Tailwind CDN | ⚠️ `node:fs/promises` (dyn) | `node:*` | Interdire `injectTailwind` dans tout chemin scellage (défaut `true` côté `rendered-html`!) ; Tailwind = build statique same-origin si requis |
| done.ts **couche-1** lint statique (`done.ts:143-448`, appel `498-506`) | **PRENDRE** | ✅ synchrone, host-free ; QUE types `pi-agent-core` (effaçables) + `validateDesignMd` shared | types only + shared | Réécrire en Rust/TS-pur ; alimente un nouveau **gate "design"** (absent de FG4-FG8) |
| done.ts **couche-2** `DoneRuntimeVerifier` (`done.ts:124`,`507-517`) + `done-verify.ts:157-231` | **RÉ-INCARNER** (pas porter) | ❌ Puppeteer + Chrome système + (Electron BrowserWindow) | `puppeteer-core`, Chrome | Rejouer dans **viewer sandbox SBFB** (iframe CSP `blob_serve.rs:286` + `postMessage` à la `sbfb-bridge.js`) ; boucle auto-repair ≤3 (cap `agent.ts:341`) |
| Pattern `DESIGN.md` + skills (`done.ts:109-119`,`87-107`; `skill.ts:173-295`; `zip.ts:96-103`) | **S'INSPIRER** (processus) | ✅ données + protocole, pas de logique runtime | `readFile` only | Adopter contrat DESIGN.md DTCG obligatoire au gate maquette-first S70 ; gratuit : hashé par `provenance.rs` + signé FG8 |
| `zip.ts` containment/réécriture (`zip.ts:60-197`, `assets.ts:105-162`) | **S'INSPIRER** ; **LAISSER** `zip-lib` | ⚠️ `node:*` + `zip-lib` (lazy) | `zip-lib` | Reprendre discipline anti-traversal (norm `\`→`/` avant resolve, `zip.ts:148-160`) ; utiliser crate `zip` SBFB natif |
| `rendered-html.ts` (`rendered-html.ts:22-92`) | **LAISSER** | ❌ Puppeteer + Chrome ; force CDN Tailwind | `puppeteer-core` | Aucun — viewer sandbox rend déjà |
| `pi-ai` / Electron host / `generate_image_asset` / tuple exporters fermé | **LAISSER** | ❌ | — | Remplacés par `provider_router.rs` / viewer sandbox / `template_engine.rs` |

**Notice licence transversale** : toute unité prise (classifieur, inliner, table MIME, lint couche-1) requiert une entrée `THIRD-PARTY-NOTICES.md` créditant `@open-codesign/*` v0.2.0 MIT (`Copyright (c) 2026 OpenCoworkAI Contributors`), sur le modèle des entrées "vendored + forked" existantes ; les blobs React/Babel UMD repris gardent *leur propre* en-tête MIT (déjà testé `template_engine.rs:574`). MIT → AGPL-3.0-or-later est compatible en aval.

---

## 3. Où ça s'accroche dans Factory (surfaces d'intégration, file:line)

Cette section cartographie, mécanisme par mécanisme, le **point de greffe exact** dans le code Factory actuel pour absorber les deux patches retenus d'Open CoDesign (classifieur HTML-verbatim + inline-assets locaux) et le verrou de goût « maquette-first ». La règle de lecture : on **réutilise un socle natif déjà là** (archétype C) et on n'ajoute que des deltas bornés. Chaque sous-section donne (a) l'ancre `file:line`, (b) ce qui existe déjà, (c) ce qu'il faut ajouter — **sans écrire de code**, et en respectant la doctrine de souveraineté (zéro runtime tiers, scellage CSP 100% Factory, solo-maintainer).

Repère structurant : **le filesystem du workspace EST le livrable**. Tout ce qui suit consiste à faire entrer « le design » dans cette surface filesystem déjà gouvernée (lock → provenance → gates → blob-serve), pas à créer un pipeline parallèle.

### 3.0 Tableau de synthèse — mécanisme × surface × delta

| # | Mécanisme | Surface Factory (file:line) | Existe déjà | À ajouter (sans code) |
|---|-----------|------------------------------|-------------|------------------------|
| 1 | **Écriture du design dans le source** | `operator_server.rs:26-33` (`ARTIFACT_DRAFT_ALLOWLIST`) + `handle_artifact_draft` `:521-634` | Endpoint `POST /api/artifacts/draft` : normalise `\`→`/`, rejette `..` `:527`, rejette verdict PASS `:543`/`:559`, allowlist préfixes `:581-587`, `create_dir_all`+`write` `:606-610` | Un **second allowlist borné « atelier »** (constante sœur, ex. `ATELIER_DRAFT_ALLOWLIST` racine `apps/<name>/design/`) **OU** un nouvel endpoint `POST /api/atelier/draft` ; PAS d'élargissement de l'allowlist `.planning/` existant (frontière de confiance distincte : artefact process vs source d'app) |
| 2 | **5e TemplateConfig « design »** (per-app DS) | `template_engine.rs:170-203` (`TEMPLATES`) + `:41-153` (`*_TEMPLATE` consts) + `:205-210` (`find_template`) | Tuple **ouvert en pratique** de 4 `TemplateConfig` (static, static-reader, react, pyodide) ; chaque template = `&'static [TemplateFile]` via `include_str!`, `create()` `:212` instancie lock+manifest+provenance | Une **5e entrée `TemplateConfig { id:"design", … }`** + son `DESIGN_TEMPLATE` const (tokens DTCG embarqués same-origin, 0 CDN). `create()`/`expected_files()`/`validate()` la consomment **sans modification** (find_template est générique). C'est le porteur du patch « inline-assets » d'Open CoDesign |
| 3 | **Provenance couvrant le design** | `provenance.rs:49-80` (`compute_output_hash`) + `:7` (`EXCLUDED_FILES`) | Tree-walk blake3 de **TOUS** les fichiers du workspace sauf `factory.template.lock` + `factory.provenance.json` `:64` ; tri déterministe `:72` → **un design posé dans le source est hashé gratuitement, sans une ligne ajoutée** | **Rien pour couvrir** le design (déjà couvert). Seul delta éventuel : **étendre `EXCLUDED_FILES` `:7`** pour exclure un répertoire de session éphémère (ex. `design/.session/`) afin que les rejouts de self-check ne polluent pas `output_hash`. À arbitrer : exclusion = surface non signée |
| 4 | **Gate « design présent/cohérent »** | `gates.rs:47-247` (FG4-FG8) + `pipeline.rs:22-63` (orchestration) | 5 gates, **aucun gate "design"** : FG4 diff (informatif `:58`), FG5 sandbox/symlink (bloquant `:30-34`), FG6 secrets+cohérence `lock==prov` `:144-158`, FG7 preview index.html+daemon `:167-182`, FG8 Ed25519 `:208-247`. `validate()` (template_engine.rs:291) = check structurel séparé | **Choix d'archi** : soit un **FG additionnel non-bloquant** (`run_gate_fg9_design`, signature `GateResult` `:11-16`, poussé dans `pipeline.rs` après FG4) ; soit **enrichir `validate()`** (template_engine.rs:302 `issues`). Recommandé : verrou **processus** (maquette-first S70), donc check léger dans `validate()` + FG informatif, **pas** un gate bloquant de plus |
| 5 | **Rendu maquette via preview/blob-serve** | `preview_cmd.rs:13-46` (`run`) + `blob_serve.rs:286-290` (CSP/COEP) | `sbfb-factory preview` zippe `:19`, POST `/api/v1/preview/load` `:21-30`, imprime `…/blob-serve/{hash}/index.html` `:44`. CSP scellée 100% Factory `:286`, COEP `require-corp` `:290` | **Rien de nouveau côté serveur** : une maquette HTML-verbatim/React-UMD passe déjà par ce chemin et hérite du scellage. Le seul delta = côté **authoring** : produire un `index.html` autoportant (assets inlinés, cf. #2). **NE PAS** router le rendu par un doc privilégié — blob-serve reste l'unique surface de rendu |
| 6 | **Self-check rejoué en viewer sandbox** | `blob_serve.rs:8` (`/blob-serve/{hash}/{path}` → iframe) + `:286` (`sandbox allow-scripts` **sans** `allow-same-origin` → origine opaque) | Le doc-comment décrit le service vers des **iframes sandboxées** `:8` ; CSP donne une origine opaque `:286`. OCD fait son self-check dans un `BrowserWindow` Electron caché — **Factory n'a pas Electron** | Un **harnais viewer iframe + `postMessage`** côté UI (factory-operator) qui charge la maquette blob-serve, capture erreurs console/`did-fail-load`-équivalents, renvoie un verdict. **Aucun privilège natif** : incarnation du self-check OCD dans la frontière sandbox de Factory |
| 7 | **Copilote design via `ExecutionTarget::Ollama`** | `provider_router.rs:80-94` (`from_provider`) + `:99-112` (`run`) + `:147-240` (`ollama_stream`) ; câblé en `operator_server.rs:971-972` | Dispatch fermé Claude/Ollama/Network, **arm Ollama keyless loopback** (`Ollama::default()` → `127.0.0.1:11434` `:163-173`), `StreamChunk` unifié, gate `SENSITIVE_ACTIONS` **avant** dispatch `:932-945`. UI picker `["claude","ollama","network"]` (`ExecutionChat.tsx:35`) | **Rien dans le routeur** : le copilote design réutilise l'arm Ollama tel quel (`cost_usd:0.0` `:218`). Delta = un **prompt/skill design** (couche prompt, hors routeur). Verrou « pas de SDK direct » respecté : tout passe par `ExecutionTarget` |
| 8 | **Restitution via `factory-ui/readonly`** | `factory-ui/src/readonly/index.ts:3-27` (barrel) + `PreviewList.tsx:5-46` + `types.ts:40-46` (`AppEntry`) | Composants read-only purs : `PreviewList` (grille, `onSelect` `:24`), `ProofCard`, `SprintTimeline`, `StatusBadge`, `VerdictChip`. `AppEntry` = name/version/category/description/published | Un composant **`DesignCard`/`DesignGallery`** sœur (même contrat read-only, 0 exécution) + extension de `AppEntry` (ou `DesignEntry`) portant le hash de la maquette → ouvre l'URL blob-serve (#5). `PreviewList.onSelect` câble déjà la sélection |
| 9 | **Design-pack hashé via context-pack** | `operator_server.rs:340-353` (`file_hash`) + `:355-427` (`handle_context_pack`) | `file_hash()` `:340` : blake3, **8 hex** `:347`, `{path,hash,exists}`. `handle_context_pack` agrège prompts + `active_artifacts` `:368-374` + `runtime_context` ; `chat_history_authoritative:false` `:416` | Un **champ `design_pack`** dans le JSON `:388-418` agrégeant les `file_hash` des fichiers de la maquette (`apps/<name>/design/*`), réutilisant `file_hash` **verbatim**. Aucune nouvelle primitive crypto |

### 3.1 Écriture du design dans le source — `artifacts/draft` vs allowlist « atelier »

Le seul chemin réseau qui **écrit dans l'arbre source** aujourd'hui est `handle_artifact_draft` (`operator_server.rs:521-634`). Il est intentionnellement étroit : normalisation `\`→`/` (`:525`) puis rejet `..` (`:527-541`) ; **double rejet du verdict PASS** par chemin (`:543`) ET par contenu (`:559-564`) ; **allowlist fermée de préfixes** (`ARTIFACT_DRAFT_ALLOWLIST`, `:26-33` : `.planning/active/`, `docs/agent/`, `docs/claude/`, `prompts/agent/`, plus `AGENTS.md`/`CLAUDE.md` exacts, `:581-587`) ; écriture seulement après `create_dir_all(parent)` (`:606-610`), chaque issue journalisée via `log_action` (`:86-95`).

**Décision d'accroche.** Le design d'app n'appartient **pas** à la frontière de confiance « artefacts de process ». **NE PAS** ajouter `apps/` à `ARTIFACT_DRAFT_ALLOWLIST`. Deux options bornées : (1) une **constante sœur** `ATELIER_DRAFT_ALLOWLIST` (préfixe `apps/<name>/design/`) ; ou (2) un **endpoint dédié** `POST /api/atelier/draft` réutilisant la même défense (normalisation + anti-traversal + journalisation), **sans** la garde verdict-PASS, **avec** une garde « le préfixe doit nommer une app existante ». Le câblage du routeur est `build_router` (`:122-167`), sous le middleware `auth::auth_required` (`:162-165`) qui impose Host+Origin+token — le nouvel endpoint hérite gratuitement de l'authentification loopback.

> **Trou connu à ne pas reproduire** : `SENSITIVE_ACTIONS` (`:35`) est keyword-based (`shell/commit/push/PASS`) et **ne couvre pas** l'écriture de fichiers arbitraires. Un endpoint atelier doit porter sa **propre** allowlist structurelle.

### 3.2 Cinquième `TemplateConfig` « design » — porteur du patch inline-assets

`TEMPLATES` (`template_engine.rs:170-203`) est un `&'static [TemplateConfig]` consommé via `find_template(id)` (`:205-210`, lookup générique). Les 4 templates actuels (static, static-reader, react, pyodide) prouvent que **le tuple est extensible sans toucher au moteur** : `create()` (`:212-269`) itère `config.files`, substitue `{{name}}`/`{{version}}`, génère lock + provenance **de manière template-agnostique**.

**Accroche.** Ajouter une 5e entrée `TemplateConfig { id:"design", files: DESIGN_TEMPLATE, … }` + une const `DESIGN_TEMPLATE: &[TemplateFile]` (`include_str!`) embarquant : un `index.html` autoportant (assets inlinés en data-URIs — c'est ici qu'on **vendorise `inlineLocalAssetsInHtml()`** d'OCD) + les **tokens DTCG/Terrazzo embarqués same-origin** (couche « per-app embarqué »), zéro CDN. Le React no-build (template `react`, `:90-126`) est le **précédent exact** : runtime vendorisé same-origin sous `default-src 'self'` (test « no CDN » `:580-596`). Tous les tests de `validate()` passent **sans modification** (IDs non énumérés).

### 3.3 Provenance — le design est déjà couvert, n'exclure que l'éphémère

`compute_output_hash` (`provenance.rs:49-80`) fait un **tree-walk blake3 récursif de TOUT le workspace**, n'excluant que les 2 fichiers d'auto-référence (`EXCLUDED_FILES`, `:7`), avec tri déterministe (`:72`). **Conséquence forte** : un design posé dans le source est intégré à `output_hash` gratuitement. La provenance Ed25519 (FG8, `gates.rs:208-247`, `DOMAIN_PROVENANCE_V1`) le scelle déjà.

Le **seul delta éventuel** est inverse : si le self-check rejoué (#6) écrit des fichiers de session **éphémères**, ils entreraient dans `output_hash` et casseraient la cohérence `lock==prov` vérifiée par FG6 (`gates.rs:144-158`). Il faut alors **étendre `EXCLUDED_FILES` `:7`** (ou filtre `design/.session/`). Arbitrage doctrinal : **tout fichier exclu sort de la surface signée** — l'éphémère doit être réellement jetable.

### 3.4 Gate « design présent/cohérent » — verrou processus, pas pipeline

`pipeline.rs:22-63` orchestre FG4→FG8 : FG4 diff **informatif** (`:58`), FG5/FG6 **bloquants**, FG8 post-publish. **Aucun gate ne raisonne sur le design.** `validate()` (`template_engine.rs:291-353`) est un check structurel séparé (manifest, symlinks, secrets) agrégeant dans `issues`. Deux options : **(a) FG bloquant additionnel** `run_gate_fg9_design` ; **(b) check léger dans `validate()`** + FG informatif. La conclusion S70 (« maquette-first = gate **processus** ») tranche pour **(b)** : la présence/cohérence du design est un **verrou de goût humain en amont**, pas un blocage cryptographique. Cohérent avec OCD où `DESIGN.md` est exigé par l'outil `done` (couche skill/goût), pas par un vérificateur runtime.

### 3.5 Rendu maquette — `preview_cmd`/blob-serve, jamais un doc privilégié

`preview_cmd::run` (`preview_cmd.rs:13-46`) : zip (`:19`), POST `/api/v1/preview/load` (`:21-30`), impression de l'URL `…/blob-serve/{hash}/index.html` (`:44`). Côté daemon, **chaque** réponse blob-serve porte le scellage **100% Factory** :
```
BLOB_SERVE_CSP = "default-src 'self' 'unsafe-inline' 'unsafe-eval' data: blob:;
                  connect-src 'none'; worker-src 'none'; frame-src 'none';
                  object-src 'none'; base-uri 'none'; form-action 'none';
                  frame-ancestors *; sandbox allow-scripts"
```
(`blob_serve.rs:286`), `sandbox allow-scripts` **sans** `allow-same-origin` → **origine opaque**, COOP `same-origin` (`:288`) + COEP `require-corp` (`:290`). **Il n'y a rien à ajouter côté serveur** : une maquette (HTML-verbatim ou React-UMD vendorisé) passe déjà par ce chemin. Travail = côté **authoring** (#2). **Anti-pattern à proscrire** : router le rendu de maquette par un « doc privilégié » qui contournerait la CSP.

### 3.6 Self-check rejoué — viewer sandbox iframe + postMessage, pas Electron

OCD exécute son `DoneRuntimeVerifier` dans un `BrowserWindow` **Electron caché** (~3 s, console/`did-fail-load`, auto-repair, cap ≈3). **Factory n'a pas Electron** et ne doit pas en introduire. La surface équivalente existe : blob-serve sert vers des **iframes sandboxées à origine opaque** (`blob_serve.rs:8`, CSP `:286`). **Accroche** : un **harnais viewer côté UI** (factory-operator) qui (1) charge la maquette via l'URL blob-serve, (2) écoute les erreurs via `postMessage`/`onerror` de l'iframe, (3) émet un verdict. C'est **l'incarnation fidèle** du self-check OCD **dans la frontière sandbox de Factory** — même intention, confinée à l'origine opaque au lieu d'un process natif.

### 3.7 Copilote design — `ExecutionTarget::Ollama`, keyless loopback

Le routeur de providers est fermé et déjà branché : `from_provider` (`provider_router.rs:80-94`) mappe `"ollama"|"local"` → `Ollama{model}`, `run()` (`:99-112`) dispatche vers `ollama_stream` (`:147-240`) qui cible `Ollama::default()` → **`127.0.0.1:11434` keyless** (`:159-174`), `cost_usd:0.0` (`:218`), `StreamChunk` unifié. Câblage SSE `operator_server.rs:971-972`, gate `SENSITIVE_ACTIONS` **avant** dispatch (`:932-945`). UI : picker `["claude","ollama","network"]` (`ExecutionChat.tsx:35`). **Rien à modifier dans le routeur** : un copilote design réutilise l'arm Ollama tel quel (local, gratuit, keyless). Delta = un **prompt/skill « design »** hors routeur (comme les `SKILL.md` 3-tiers d'OCD : instructions de goût, 0 exécution). Verrou « tout LLM via le routeur, aucun SDK direct » respecté par construction.

### 3.8 Restitution — `factory-ui/readonly`, contrat 0-exécution

Le barrel read-only (`factory-ui/src/readonly/index.ts:3-27`) expose des composants **purs** : `PreviewList` (`onSelect` `:24`), `ProofCard`, `SprintTimeline`, `StatusBadge`, `VerdictChip`. Type `AppEntry` (`types.ts:40-46`). **Accroche** : un composant sœur **`DesignCard`/`DesignGallery`** suivant le même contrat read-only + un type `DesignEntry` portant le hash de la maquette pour ouvrir son URL blob-serve (#5). `PreviewList.onSelect` montre déjà le pattern de sélection.

### 3.9 Design-pack hashé — réutilisation verbatim de `file_hash`

`handle_context_pack` (`operator_server.rs:355-427`) assemble un pack auditable : prompts, `active_artifacts` (chaque `.planning/active/*` via `file_hash`), `runtime_context`, avec `chat_history_authoritative:false` (`:416`). Primitive : `file_hash` (`:340-353`), blake3 **tronqué à 8 hex**. **Accroche** : ajouter un champ **`design_pack`** (`:388-418`) agrégeant les `file_hash` des fichiers de la maquette, en **réutilisant `file_hash` verbatim**. Aucune nouvelle primitive : un design-pack est **vérifiable par recalcul** côté agent, au même titre que `active_artifacts`. La garde anti-traversal de `specialized_kind` (`:378`) est le modèle à imiter si le nom d'app vient du client.

### 3.10 Récapitulatif des invariants préservés

- **Souveraineté runtime** : aucun mécanisme n'introduit de runtime tiers ni de CDN. Tokens/maquettes vendorisés same-origin (#2), self-check confiné à l'iframe opaque (#6), copilote Ollama loopback keyless (#7).
- **Scellage 100% Factory** : la CSP `blob_serve.rs:286` reste l'unique frontière de rendu, identique app/maquette (#5).
- **Frontières de confiance disjointes** : le design **n'élargit pas** l'allowlist process `.planning/` (#1).
- **Provenance gratuite** : le tree-walk blake3 couvre déjà le design (#3) ; seul soin = exclure l'éphémère sans le rendre livrable.
- **Verrou de goût en amont** : maquette-first reste un gate **processus** (#4).
- **Routeur LLM unique** : le copilote passe par `ExecutionTarget`, jamais par un SDK direct (#7).

---

## 4. Comment l'implémenter au mieux (conception native)

Cette section décrit la **mécanique** de l'étage design natif de Factory : un archétype C (patterns natifs réimplantés dans l'architecture souveraine), avec deux patches vendorisés issus d'Open CoDesign (classifieur HTML-verbatim + inline-assets) traités au §5. Aucune surface plugin, aucun runtime tiers : tout transite par les primitives Factory existantes.

Le principe directeur repris d'Open CoDesign — *« the workspace filesystem IS the deliverable »* — est, dans Factory, **déjà vrai gratuitement** : `provenance::compute_output_hash()` (`provenance.rs:49-80`) fait un tree-walk blake3 de **tous** les fichiers du workspace sauf `factory.template.lock` et `factory.provenance.json` (`EXCLUDED_FILES`, `:7`). Conséquence fondatrice : **tout fichier de design ajouté au source d'une app est hashé, scellé et signé sans une ligne de code**. C'est le pivot de toute la conception.

### 4.1 « workspace = deliverable » → la convention `design/` couverte par la provenance

On définit une **convention de dossier** dans le source de chaque app :
```
<app>/
├── SBFB.json                  # manifest (inchangé)
├── index.html                 # entrypoint (inchangé)
├── design/
│   ├── DESIGN.md              # design system per-app (instructions de goût)
│   ├── tokens.json            # source DTCG (Design Tokens Community Group)
│   └── maquette/              # snapshots HTML/PNG de référence (la « maquette »)
├── sbfb-tokens.css            # SORTIE compilée, EMBARQUÉE same-origin
└── factory.{template.lock,provenance.json}
```
Aucune modification de `provenance.rs` : le tree-walk descend récursivement (`WalkDir::new(dir).follow_links(false)`) et inclut `design/*` + `sbfb-tokens.css` dans le hash ; chemin normalisé en `/` (`:62`) puis trié (`:72`) — déterminisme cross-OS garanti (`test_provenance_hash_deterministic`, `:100`).

**Conséquence de scellage.** FG6 (`gates.rs:127-165`) vérifie déjà `lock_hash == prov_hash` (`:148-157`) ; FG8 (`:208-247`) signe en Ed25519 (`DOMAIN_PROVENANCE_V1`, `:6`/`:240`). **Un design altéré après scellage casse FG6/FG8 mécaniquement** — la provenance design est une propriété émergente, pas un gate dédié.

**Garde-fou template.** Pour matérialiser la convention, on ajoute `design/DESIGN.md` + `design/tokens.json` + `sbfb-tokens.css` comme `TemplateFile` (`template_engine.rs:35-39`), même mécanisme `include_str!` que les 4 templates. `expected_files()`/`validate()` les couvrent sans cas particulier.

### 4.2 DESIGN.md → DS per-app + modèle 2 couches + pipeline TOKENS

C'est le cœur. On reprend la **convention DESIGN.md** (exigée par `done` côté OCD), articulée sur le modèle **2 couches** propre à Factory.

| Couche | Périmètre | Support physique | Référence vs copie |
|---|---|---|---|
| **Shell global** | `factory-operator` (Vite/React + shadcn/Tailwind) + `factory-ui/readonly` | CSS vars du shell (`var(--primary)` déjà utilisé : `ContextPackBuilder.tsx:203`) | **référence** OK (même origine, build-time) |
| **Per-app embarqué** | Chaque app publiée, servie par `blob_serve` | `sbfb-tokens.css` **copié** dans le source de l'app | **copie obligatoire** |

La frontière n'est pas un choix de style : elle est **imposée par la CSP**. `BLOB_SERVE_CSP` (`blob_serve.rs:286`) impose `connect-src 'none'` + `default-src 'self'`. Sous `connect-src 'none'`, **une app publiée ne peut JAMAIS référencer un token sheet distant** (CDN, Google Fonts, `@import url(https://…)`).

> **Règle mécanique : la couche per-app COPIE ses tokens (`sbfb-tokens.css` embarqué same-origin), elle ne les RÉFÉRENCE jamais.** Le shell global vit dans une origine normale et peut référencer ses CSS vars build-time.

**Le pipeline TOKENS (source unique → 2 sorties)** :
```
design/tokens.json  (DTCG, source de vérité, hashée §4.1)
        │  compilation BUILD-TIME (style-dictionary / terrazzo-like, authoring toléré)
        ├──────────► CSS vars du SHELL  (Tailwind v4 @theme / shadcn)        [couche 1, référence]
        └──────────► sbfb-tokens.css EMBARQUÉ par app  (COPIE same-origin)   [couche 2, copie]
                       écrit dans le source → hashé par provenance → servi sous default-src 'self'
```
- **Source DTCG.** `tokens.json` est le seul endroit où une couleur/typo/spacing existe. Compilation *authoring build-time* (tolérée, comme `htm`/React UMD vendorisés `template_engine.rs:90-126`), jamais un runtime.
- **Sortie shell.** Tailwind v4 expose ses tokens via `@theme { --color-… }` ; le shell shadcn les consomme (couche **référence**).
- **Sortie per-app.** Le générateur émet `sbfb-tokens.css` (un `:root { --… }` pur, zéro `@import` distant) **dans le source**. Hashé gratuitement, servi sous `default-src 'self'`. Les fonts suivent : **embarquées** (le patch inline-assets §5 transforme les `@font-face` locaux en data-URIs ; aucun Google Fonts hardcodé).

**DESIGN.md = instructions de goût, 0 exécution.** Comme les `SKILL.md` 3-tiers d'OCD : lu par le copilote (§4.4) comme contexte de style ; existence/contenu minimal vérifié par la couche lint statique (§4.3) ; dans le pack de contexte au même titre que les prompts agent.

### 4.3 Self-check « maquette » ré-incarné SANS Electron

OCD fait son self-check `done` en 2 couches (`done.ts`). **Factory n'a pas Electron et ne doit pas l'introduire.** On réincarne les 2 couches :

**Couche 1 — lint statique natif (Rust pur).** Un module natif (p.ex. `design_check.rs`) réimplémente les vérifs statiques en Rust, dans l'esprit de `secret_scanner::scan_directory` (déjà appelé par `validate()` `template_engine.rs:330` et FG6 `gates.rs:130`) : balises non fermées, IDs dupliqués, `alt` manquant, liens d'ancrage cassés, **+ présence/forme minimale de `design/DESIGN.md`** + cohérence `sbfb-tokens.css` ↔ `tokens.json`. Synchrone, déterministe, zéro réseau.

**Couche 2 — runtime rejoué dans le viewer `blob_serve` (iframe + postMessage).** Au lieu d'une BrowserWindow Electron, on rejoue l'artefact dans **le même sandbox iframe que la preview de publication** (`blob_serve.rs:8`, `frame-ancestors *` `:286` autorise l'embedding). Mécanique : (1) l'Operator charge l'app dans un iframe blob-serve (origine opaque) ; (2) un shim de capture **injecté dans l'archive** (same-origin, autorisé sous `default-src 'self'`) installe `window.onerror` + wrap `console.error` et **`postMessage(err, '*')` vers le parent** (`postMessage` n'est PAS du réseau → non bloqué par `connect-src 'none'`) ; (3) le parent écoute `message` + `did-fail-load`-équivalent pendant ~3 s ; (4) les erreurs alimentent une boucle d'auto-repair bornée (le copilote Ollama applique des `str_replace`, plafond ≈3 rounds).

**Bénéfice de souveraineté décisif** : le verifier tourne sous la CSP **RÉELLE de production**. OCD vérifie dans Electron (permissif), donc un design qui marche dans le verifier peut casser une fois scellé. Le verifier Factory s'exécute sous `connect-src 'none'` + origine opaque + COEP `require-corp` : si un design dépend d'un CDN/font distante, **il échoue au self-check, pas en production**.

**Branchement : gate processus, pas étage pipeline.** Le self-check se branche **avant FG4** comme **gate de processus** (maquette-first S70), pas comme un FGx bloquant. Raison : le pipeline `gates.rs`/`pipeline.rs` est la chaîne de scellage *cryptographique* ; y injecter un verifier runtime navigateur mélangerait deux natures. Le résultat est *journalisable* via `artifacts/draft`, jamais comme verdict PASS (bloqué, `operator_server.rs:543-579`).

### 4.4 Copilote design = boucle agentique offline via `ExecutionTarget::Ollama`

OCD route tout LLM via `pi-ai`. Factory a sa règle symétrique : **NO third-party runtime** + tout via `provider_router::ExecutionTarget`. Le copilote design **n'utilise jamais pi-ai** ; il utilise l'arm **Ollama** déjà câblée :
```
copilote design  ──►  ExecutionTarget::Ollama { model }   (provider_router.rs:66, :82)
                        ├─ Ollama::default() → 127.0.0.1:11434 (loopback, keyless)   :159-174
                        ├─ generate_stream → StreamChunk::{Delta,Done,Error}          :209-224
                        └─ cost_usd = 0.0  (inférence locale gratuite)                :219, :235
```
- **Souveraineté** : loopback keyless durci, aucune clé/cloud/pi-ai. Exact pendant du Ollama keyless d'OCD, branché sur l'enum souveraine Factory.
- **Contrat unifié** : `StreamChunk` → réutilise la couche SSE de l'Operator (`handle_chat_stream`, `:886-992`).
- **Génération d'images** : équivalent de `generate_image_asset` reste **opt-in, hors chemin par défaut** ; la boucle copilote par défaut est 100 % locale.
- **Gate de sensibilité préservé** : `SENSITIVE_ACTIONS` (`:35`) vérifié *avant* dispatch (`:931-945`), provider-indépendant.

### 4.5 Maquette-first comme gate (mécanisé vs processus)

Maquette-first S70 = **gate de processus** : concevoir `design/maquette/` **avant** d'écrire l'app. Mécanisation minimale : (a) `design/maquette/index.html` est un fichier du workspace → hashé par provenance (§4.1) ; (b) on reprend la logique *déterministe* de `verify_ui_kit_parity` d'OCD (parité élément-count / visible-text-coverage / token-coverage, pondérée 40/30/30, seuil 0.85) — **string-based, zéro LLM, zéro rendu** — réimplémentée en Rust pur, branchée au **même point processus** (avant FG4). **Pourquoi processus et pas pipeline** : un seuil de parité 0.85 est une décision *de goût*, pas une invariante de sécurité ; on ne dilue pas la sémantique « FAIL = blocage cryptographique » des FGx.

### 4.6 `decompose_to_ui_kit` → UI kit = DS per-app (retombe sur §4.2)

`decompose_to_ui_kit` produit `ui_kits/<slug>/` = `index.html` + `components/*.tsx` + **`tokens.css`** + `manifest.json` + `README.md`. Dans Factory, **l'UI kit n'est pas une nouvelle abstraction : c'est la matérialisation de la couche per-app du §4.2.**

| Sortie `decompose_to_ui_kit` | Mapping Factory |
|---|---|
| `ui_kits/<slug>/tokens.css` | → la **sortie per-app** (= `sbfb-tokens.css`, copie same-origin) |
| `ui_kits/<slug>/components/*.tsx` | → source app ; sous template `react` no-build (`:90-126`), consommable via htm UMD vendorisé, **sans build/CDN** |
| `manifest.json` | → cohabite avec `SBFB.json` (`:243-255`) ; hashé par provenance |
| `README.md` handoff | → instructions de goût, 0 exécution |

Point critique : `decompose_to_ui_kit` génère un `tokens.css` **copié** — précisément la règle « copie jamais référence » imposée par `connect-src 'none'`. Donc l'UI kit Factory **est** le DS per-app, et `verify_ui_kit_parity` devient le self-check de cohérence kit↔maquette.

### 4.7 comment-mode / `tweaks` → UX d'intentions côté `factory-operator`

`tweaks` d'OCD est **advisory et non-mutant** : scanne le workspace pour des blocs `EDITMODE` et renvoie une liste clé/valeur que le *tweaks panel* utilise pour ajuster **sans re-prompter l'agent**. À reprendre, branché sur la surface d'écriture **déjà durcie** : `POST /api/artifacts/draft` (`:521-634`, allowlist + anti-traversal + anti-PASS). **Conséquence** : pour que les intentions design soient écrivables, soit étendre l'allowlist avec un préfixe dédié, soit les router dans `.planning/active/`. Dans tous les cas, le **même chemin durci** ; aucune nouvelle route d'écriture. Le `tweaks`-scan (lecture des blocs `EDITMODE`) est read-only, exposable comme GET. Les mots-clés `SENSITIVE_ACTIONS` (`:35`) interceptent toute intention gateée → un commentaire de design passe ; un « commit ce design » est intercepté.

### 4.8 Schéma d'articulation (qui-appelle-quoi)

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│ COUCHE 1 — SHELL GLOBAL (origine normale, build-time, RÉFÉRENCE OK)                   │
│  factory-operator (Vite/React, shadcn/Tailwind v4)   factory-ui/readonly             │
│        │  CSS vars  var(--primary)  [ContextPackBuilder.tsx:203]                       │
│        │  (7) intentions design                     (4) copilote design               │
│        │  POST /api/artifacts/draft ───────┐         (chat SSE)                        │
│        │  tweaks scan (read-only GET)       │            │                             │
└────────┼────────────────────────────────────┼────────────┼─────────────────────────────┘
         ▼                                     ▼            ▼
   handle_artifact_draft               SENSITIVE_ACTIONS gate     handle_chat_stream
   operator_server.rs:521              operator_server.rs:35      operator_server.rs:886
     • allowlist  :26                    (avant dispatch :931)         │
     • anti-traversal :527                                            ▼
     • anti-PASS  :543                              provider_router::ExecutionTarget
         │                                          provider_router.rs:62  (from_provider :80)
         │ écrit .planning/active/… , design/…             │
         ▼                                          ┌───────┴────────────────────┐
   (1) WORKSPACE = DELIVERABLE                      ▼ Ollama                      ▼ Claude
   ┌──────────────────────────────────┐    127.0.0.1:11434 keyless        (pilote, opt-in)
   │ <app>/                           │    provider_router.rs:159-174     jamais pi-ai
   │  ├ design/DESIGN.md   ◄── (2) DS per-app, instructions de goût, 0 exec
   │  ├ design/tokens.json ◄── (2) SOURCE DTCG (unique)
   │  ├ design/maquette/   ◄── (5) maquette-first (référence scellée)
   │  ├ sbfb-tokens.css    ◄── (2) SORTIE per-app, COPIE same-origin (connect-src 'none')
   │  └ ui_kits/<slug>/    ◄── (6) decompose_to_ui_kit  →  retombe sur couche 2
   └───────────────┬──────────────────┘
                   │ (1) hashé GRATUITEMENT (tree-walk blake3, sauf lock+prov)
                   ▼
   provenance::compute_output_hash()   provenance.rs:49-80
                   │
   ┌───────────────┴──────────────────────────────────────────────────────────┐
   │ (3) SELF-CHECK « maquette » — GATE PROCESSUS (avant FG4, pas un FGx)       │
   │  couche statique (Rust pur)          couche runtime (viewer blob_serve)    │
   │  design_check.rs ~ secret_scanner    iframe sandbox  blob_serve.rs:8       │
   │   • tags/ids/alt/links               CSP RÉELLE  BLOB_SERVE_CSP :286       │
   │   • DESIGN.md minimal                 connect-src 'none' + opaque origin   │
   │   • tokens.json ↔ sbfb-tokens.css     shim same-origin → postMessage err   │
   │  (5) verify_ui_kit_parity (0.85)      window.onerror / did-fail-load ~3s   │
   │                                       auto-repair str_replace ≤3 rounds    │
   └───────────────┬────────────────────────────────────────────────────────────┘
                   │ (verdict NON-PASS → journalisable via artifacts/draft)
                   ▼
   PIPELINE DE SCELLAGE (inchangé, cryptographique)   pipeline.rs:15-70
     FG4 diff (info) → FG5 sandbox/symlink (bloquant) → FG6 secrets + lock==prov →
     publish → FG7 preview daemon → FG8 provenance Ed25519 (DOMAIN_PROVENANCE_V1)
                   ▼
   blob_serve  →  iframe sandbox prod  (BLOB_SERVE_CSP, COEP require-corp, 0 CDN, fonts embarquées)
```
**Aucune flèche ne sort vers un réseau tiers ; aucune flèche n'entre depuis pi-ai ou un CDN.**

### 4.9 Synthèse mécanique

| Brique Open CoDesign | Mécanique Factory native (archétype C) | Point d'ancrage code |
|---|---|---|
| « workspace IS deliverable » | tree-walk blake3 hashe `design/` gratuitement | `provenance.rs:49-80` |
| `DESIGN.md` (0 exec) | `design/DESIGN.md` per-app, lint statique minimal | `template_engine.rs:35-39`, `done.ts` |
| tokens (DTCG/Terrazzo) | source `tokens.json` → 2 sorties (shell réf / app copie) | imposé par `connect-src 'none'` `blob_serve.rs:286` |
| `done` 2 couches (Electron) | lint statique Rust + runtime iframe `blob_serve` + postMessage | `blob_serve.rs:8,286` ; `secret_scanner` |
| copilote LLM (pi-ai) | `ExecutionTarget::Ollama` loopback keyless, jamais pi-ai | `provider_router.rs:66,82,159-174` |
| maquette-first | gate **processus** (avant FG4), parité déterministe 0.85 | `pipeline.rs:23` ; `verify_ui_kit_parity` |
| `decompose_to_ui_kit` | UI kit = couche 2 DS per-app (tokens copiés) | §4.2 + template `react` `template_engine.rs:90-126` |
| `tweaks` / comment-mode | UX d'intentions via `artifacts/draft` durci | `operator_server.rs:521-634` |
| scellage / CSP | **100 % Factory, rien à reprendre** | `BLOB_SERVE_CSP` `blob_serve.rs:286` ; FG8 `gates.rs:208` |

---

## 5. Risques, posture build-vs-partner, séquencement & décisions

### 5.1 Les tensions à fermer (et leur mitigation précise)

**T1 — CSP / COEP / origine opaque vs assets et fonts externes.** Header réel (`blob_serve.rs:286`) avec COOP `same-origin` (`:288`) + COEP `require-corp` (`:290`), `sandbox allow-scripts` **sans** `allow-same-origin` (origine opaque). Trois collisions : (1) **Google Fonts hardcodées** sur le chemin JSX (`runtime/index.ts` `wrapJsxAsSrcdoc`/`wrapJsxAsStandaloneDocument`) ; (2) **assets distants laissés externes** par conception (`assets.ts` `isLocalReference()` renvoie `false` pour `http(s)://`/`//`) ; (3) **`removeCspMetaTags()`** strippe la CSP de l'artefact (cohérent avec « scellage 100 % Factory »). **Mitigation** : le scellage ne se négocie pas, reste 100 % Factory. La vendorisation B doit **échouer-fort sur référence distante** (transformer le « laisser externe » en erreur d'export bloquante / inlining forcé). Les Google Fonts hardcodées doivent être **retirées du patch**. `frame-ancestors *` mérite une décision explicite (D-CSP).

**T2 — Double runtime React : UMD+htm (no-build) vs Babel-in-iframe.** Factory a tranché : `REACT_TEMPLATE` (`template_engine.rs:87-122`) vendorise same-origin React/ReactDOM/htm, sans build, sans transpile runtime. OCD vendorise `@babel/standalone` + React UMD et transpile JSX *in-iframe*. Sous la CSP Factory `'unsafe-eval'` est présent, donc Babel *tournerait* — mais c'est ce que « no third-party runtime » veut éviter (transpileur ~3 Mo embarqué). **Mitigation** : ne pas importer le chemin Babel-in-iframe. Le **classifieur** est la pièce à vendoriser (B), pas le wrapper Babel. Chemin « HTML document complet » → rendu **verbatim** (aligné) ; chemin JSX → rail htm/no-build de Factory ; le classifieur sert seulement à *router*.

**T3 — pi-ai vs ProviderRouter.** La tension n'est **pas** sur Ollama (convergence loopback keyless des deux côtés) mais sur la *brique d'agent* (`pi-agent-core`, `generateViaAgent`) : la réimporter forcerait pi-ai. **Mitigation** : ne pas importer la couche provider d'OCD. `ExecutionTarget` reste l'unique frontière. On emprunte le *comportemental* (design-tools, DESIGN.md, SKILL.md 3-tiers), pas l'*infrastructurel*. Le copilote s'appuie sur le bras `Ollama` **déjà présent** (`:147-240`).

**T4 — Pivot Node A/B/C.** **A (zéro-Node)** et **B (Node éphémère opt-in)** vivants, **C (persistant) REJETÉ**. OCD *est* un monorepo Electron, donc tout import verbatim tend à tirer du Node/Electron. Or les deux briques B (classifieur + `inlineLocalAssetsInHtml`) sont du **TypeScript pur, pas Electron** : ré-implémentables en Rust (C, à privilégier) ou exécutées en Node **éphémère** (B) build-time. **Mitigation** : si la vendorisation B reste *build-time* (transformation d'export, jamais dans l'artefact servi), elle est compatible A/B. Le seul élément exigeant du Node *runtime* serait le `DoneRuntimeVerifier` Electron — d'où le rejeu en viewer sandbox (T6). Pivot C reste rejeté.

**T5 — Trou RCE : le terminal n'est pas couvert par `SENSITIVE_ACTIONS`.** `SENSITIVE_ACTIONS` (`operator_server.rs:35`) = `["shell","commit","push","PASS"]`, appliqué **par correspondance de mot-clé sur le texte du message** (`:724-727`, `:826-829`, `:931-945`). **Mais** l'endpoint dangereux est `/api/terminal/ws` (`:145` → `handle_terminal_ws` `:1019` → `terminal.rs`), qui **spawn un PTY interactif** (`terminal.rs:69-89` lance `claude.cmd`/`claude` via `CommandBuilder`+`spawn_command`). Ce chemin **ne traverse jamais `SENSITIVE_ACTIONS`** : gardé uniquement par `auth_required` (Host+Origin loopback+token) + CORS loopback. Le mot-clé `"shell"` protège la *conversation*, pas la *porte d'exécution*. **Mitigation** : (a) documenter que la liste est un filtre de *contenu de chat*, jamais un contrôle d'exécution ; (b) un vrai gate sur l'ouverture de PTY doit être posé *au niveau de l'upgrade WebSocket*. Important : importer OCD **n'aggrave pas** ce trou (OCD n'ouvre pas de PTY web), mais le copilote/chat ne doivent jamais devenir un *proxy* vers ce terminal — la frontière `ExecutionTarget` (aucun bras « shell ») l'en empêche structurellement.

**T6 — Self-check : `DoneRuntimeVerifier` Electron vs viewer sandbox.** Vérif 2 couches : (1) lint **statique pur** (portable), (2) `DoneRuntimeVerifier` = callback injecté, implémenté en chargeant l'artefact dans une `BrowserWindow` Electron cachée (~3 s console/`did-fail-load`). **Mitigation** : la couche statique est portable telle quelle (B, build-time) ; la couche runtime ne doit PAS importer Electron : Factory la rejoue dans **son** viewer sandbox (iframe origine opaque + `postMessage`), sous la CSP `blob_serve.rs:286`. La borne `MAX_DONE_ERROR_ROUNDS = 3` **est de la doctrine** (guidance textuelle), pas une constante déclarée (cf. incertitudes).

**T7 — Supply-chain : v0.2 non signé, mono-auteur ~84 %.** Factory **signe** (provenance blake3 `:49-80` + FG8 Ed25519 `gates.rs:208-247`, `DOMAIN_PROVENANCE_V1`). **Mitigation** : le code vendorisé d'OCD entre par **patch lu ligne-à-ligne**, jamais par dépendance npm transitive : on copie le *comportement* sous revue, on n'ajoute pas `open-codesign` au lockfile. La provenance Factory couvre le risque *en aval* : tout ce qui finit dans l'artefact est haché et signé, quelle que soit son origine.

### 5.2 Posture build-vs-partner : pourquoi C/B maintenant, alliance différée

**Construire maintenant (C socle + B patchée), partenariat plus tard.** Trois raisons code-level :
1. **OCD n'a aucune surface de plugin.** `EXPORTER_FORMATS` = tuple fermé + chaîne `if` (pas un registre) ; `done.ts` n'expose qu'un *callback hôte*, pas une API d'extension. **Plugin-loading et MCP explicitement différés post-1.0.** Aucun moyen d'« étendre » OCD sans patcher son cœur — un partenariat technique exigerait un merge upstream, dont la **latence n'est pas maîtrisable** (mono-auteur, issue-first, ~400 LOC/PR).
2. **L'API ne se stabilise qu'à v1.0.** Bâtir sur une API v0.2 = recoller à chaque release. C/B nous rend **immunisés** : on vendorise un comportement figé, sous notre revue.
3. **Le scellage et la provenance ne sont jamais délégables.** CSP, COEP/COOP, FG5/FG6/FG8, Ed25519 = invariants souverains. Le périmètre « partenable » est structurellement réduit au *goût*, pas au *scellage*.

**Kill-switch.** La posture reste réversible : tant qu'on ne dépend d'aucune surface upstream et qu'on n'ajoute pas `open-codesign` au lockfile, on peut **retirer** la vendorisation B sans casser le socle C. Toute brique empruntée est un patch isolé, désactivable, qui ne franchit jamais la frontière `ExecutionTarget` ni le scellage.

**Premier geste coopératif le moins risqué.** Publier un **`SKILL.md` « goût SBFB »** (instructions de goût, 3-tiers, **0 exécution**) : aucun merge dans le cœur d'OCD, ne touche ni `ExecutionTarget`, ni le scellage, ni le lockfile. Artefact de connaissance, rollback gratuit, qui teste la complémentarité culturelle avant tout engagement technique.

### 5.3 Séquencement en briques (descriptif, non daté)

1. **Per-app DS + provenance (gratuit).** Prérequis : rien (déjà acquis structurellement). Première brique car coût nul ; ancre la couche DS per-app.
2. **Copilote Ollama.** Dépend de (1) ; s'appuie sur le bras **déjà présent** `ExecutionTarget::Ollama`. Aucun pi-ai.
3. **Template design.** Dépend de (1)+(2). Variante « design » sur le rail htm/no-build (T2), jamais Babel-in-iframe.
4. **Vendorisation B patchée.** Dépend de (3). Importe `classifyRenderableSource` + `inlineLocalAssetsInHtml` **patché** (échouer-fort sur ref distante, **sans** Google Fonts). Build-time uniquement (compatible A/B, T4).
5. **Rendu viewer sandbox.** Dépend de (4). Sert l'artefact sous `blob_serve.rs:286` ; c'est ici que les patchs de (4) se prouvent.
6. **Self-check.** Dépend de (5). Couche statique portée (B) + couche runtime **rejouée dans le viewer sandbox**, PAS Electron (T6).
7. **UI + preuve.** Dépend de tout. Surface l'artefact + provenance + preuves de gate dans `factory-ui/readonly`.

Dépendances critiques : (4) **avant** (5) (sinon un asset distant casse l'affichage au lieu d'échouer-fort à l'export) ; (1) **avant** tout ; (6) **après** (5).

### 5.4 Décisions à trancher (PO) — recommandations penchées

| # | Décision | Options | Recommandation penchée | Ancrage code |
|---|----------|---------|------------------------|--------------|
| D-VENDOR | Vendorisation B en Rust pur (C) ou Node éphémère (B) ? | (a) Ré-impl Rust ; (b) Node build-time éphémère | **(a) Rust pur** si l'effort de port reste borné ; sinon (b) build-time strict | `assets.ts`, `index.ts` ; pivot A/B → `template_engine.rs` |
| D-ASSET | Référence distante à l'export | (a) laisser externe ; (b) inliner si local ; (c) **erreur d'export** | **(c) erreur d'export** (le `connect-src 'none'` la casserait — échouer tôt, lisiblement) | `isLocalReference()` ; `blob_serve.rs:286` |
| D-FONTS | Google Fonts du chemin JSX | (a) garder ; (b) **retirer** | **(b) retirer** — « 0 CDN, fonts embarquées » | `wrapJsxAsSrcdoc` (`runtime/index.ts`) |
| D-REACT | Runtime React de la variante design | (a) Babel-in-iframe ; (b) **UMD+htm no-build** | **(b)** — déjà vendorisé same-origin | `template_engine.rs:87-122` |
| D-SELFCHECK | Couche runtime du self-check | (a) Electron `BrowserWindow` ; (b) **viewer sandbox iframe+postMessage** | **(b)** — pas de runtime tiers | `done.ts` ; `blob_serve.rs:286` |
| D-TERMINAL | Gate de l'ouverture de PTY `/api/terminal/ws` | (a) statu quo (auth loopback seul) ; (b) gate explicite à l'upgrade WS | **(b) à évaluer** — `SENSITIVE_ACTIONS` ne couvre PAS ce chemin | `operator_server.rs:35`, `:1019` ; `terminal.rs:69-89` |
| D-CSP | `frame-ancestors *` sur blob-serve | (a) garder `*` ; (b) restreindre à l'origine viewer | **(a) garder** pour le viewer embarquable, **mais documenter** (origine opaque + `frame-src 'none'` limitent déjà l'exfiltration) | `blob_serve.rs:283-286` |
| D-PARTNER | Premier geste vers OCD | (a) PR cœur ; (b) **SKILL.md « goût SBFB »** ; (c) rien | **(b)** — 0 merge cœur, 0 lockfile, rollback gratuit | contrat SKILL.md 3-tiers (OCD) |
| D-PIAI | Découpler pi-ai pour reprendre l'agent de goût ? | (a) oui ; (b) **non, garder `ExecutionTarget`** | **(b)** — coût de découplage non mesuré ; emprunter le *comportement*, pas l'infra | `provider_router.rs:63-113` |

### 5.5 Encadré — incertitudes honnêtes

> **Ce qui reste non vérifié et doit être tenu pour incertain :**
> - **Coût de découplage de pi-ai NON mesuré.** Reprendre l'agent de goût (`pi-agent-core`, `generateViaAgent`) sans tirer `pi-ai` n'a pas été chiffré. La reco D-PIAI repose sur ce coût *présumé* élevé.
> - **`DoneRuntimeVerifier` exact non lu intégralement.** Signature + `BrowserWindow` + console/`did-fail-load` + ~3 s connus — mais la *traduction fidèle* vers iframe+postMessage (quels événements DOM/console équivalents, quelle fenêtre temporelle) reste à valider empiriquement.
> - **`MAX_DONE_ERROR_ROUNDS = 3` n'est PAS une constante déclarée** dans le `done.ts` lu : guidance textuelle (« after 3 error rounds ») du prompt d'outil, pas un `const` vérifiable. La borne réelle est doctrinale, à fixer côté Factory.
> - **Google Fonts sur le chemin verbatim ?** Confirmé hardcodées sur le chemin **JSX**. Il n'est **pas** établi qu'un document HTML rendu **verbatim** reste exempt de fonts/CDN distants : un document utilisateur *peut* contenir ses propres `<link>` Google Fonts que le rendu verbatim ne strippe pas. À auditer sur la branche verbatim.
> - **Appels runtime cachés dans l'artefact exporté.** `inlineLocalAssetsInHtml` couvre images + polices + CSS imbriqués récursifs, mais un `fetch()` JS, un `<script src>` distant, un `@font-face` dynamique ou une `url()` construite à l'exécution ne sont **pas** capturés par l'inlining statique — ils ne se révéleraient qu'au rendu sous `connect-src 'none'`. Le viewer sandbox est le filet, mais l'exhaustivité de l'inlining n'est pas prouvée.
> - **Latence de merge upstream non maîtrisable.** Mono-auteur, issue-first, ~400 LOC/PR, plugin-loading/MCP différés post-1.0 : tout geste *technique* vers OCD (au-delà du SKILL.md) dépend d'un calendrier qui n'est pas le nôtre. La posture C/B est précisément ce qui nous en affranchit.

---

## 6. Carte des références (file:line)

**Open CoDesign** (`github.com/OpenCoworkAI/open-codesign`, v0.2.0, MIT) :
- `packages/runtime/src/index.ts` — `classifyRenderableSource` (`:76-87`), `looksLikeFullHtmlDocument` (`:61-64`), `looksLikeJsxSource` (`:66-74`), `needsJsxRuntimeInHtml` (`:412-423`), Google Fonts hardcodées (`:350-352`, `:383-385`), `baseTag` (`:243-245`), UMD `?raw` (`:24-28`, `:328-335`), `buildStandaloneDocument` (`:642-659`), `removeCspMetaTags` call (`:599`, `:646`)
- `packages/shared/src/html-utils.ts:272-292` — `removeCspMetaTags`
- `packages/exporters/src/assets.ts` — `inlineLocalAssetsInHtml` (`:46-58`), `isLocalReference` (`:394-399`), `mimeForPath` (`:446-486`), `resolveAssetReference`/`isInsideRoot` (`:372-392`, `:421-424`), `rewriteHtmlLocalAssetReferences` (`:105-162`)
- `packages/exporters/src/html.ts` — `TAILWIND_CDN` (`:12-13`), `buildHtmlDocument` (`:51-70`), défaut `injectTailwind` (`:52`)
- `packages/exporters/src/rendered-html.ts:22-92` — Puppeteer + `injectTailwind ?? true`
- `packages/exporters/src/zip.ts:60-197` — `exportZip`, containment (`:148-160`)
- `packages/core/src/tools/done.ts` — couche-1 lint (`:143-448`, `:498-506`), `DoneRuntimeVerifier` type (`:124`, `:507-517`), `DESIGN.md` (`:109-119`, `:87-107`)
- `apps/desktop/src/main/done-verify.ts:157-231` — Puppeteer/Chrome (NON portable)
- `packages/core/src/agent.ts:341` — cap auto-repair ; `packages/core/src/tools/skill.ts:173-295` — skills

**SBFB Factory** :
- `crates/sbfb-factory/src/operator_server.rs` — `ARTIFACT_DRAFT_ALLOWLIST` (`:26-33`), `SENSITIVE_ACTIONS` (`:35`), `handle_artifact_draft` (`:521-634`), `file_hash` (`:340-353`), `handle_context_pack` (`:355-427`), `build_router` (`:122-167`), `handle_chat_stream` (`:886-992`, gate `:931-945`), `handle_terminal_ws` (`:1019`)
- `crates/sbfb-factory/src/template_engine.rs` — `TEMPLATES` (`:170-203`), `*_TEMPLATE` (`:41-153`), `REACT_TEMPLATE` (`:87-126`), `find_template` (`:205-210`), `create` (`:212-269`), `validate` (`:291-353`), test « no CDN » (`:580-596`), test React license header (`:574`)
- `crates/sbfb-factory/src/provenance.rs` — `compute_output_hash` (`:49-80`), `EXCLUDED_FILES` (`:7`), test déterministe (`:100`)
- `crates/sbfb-factory/src/gates.rs` — FG4-FG8 (`:47-247`), FG6 `lock==prov` (`:144-158`), FG8 Ed25519 (`:208-247`, `DOMAIN_PROVENANCE_V1` `:6`)
- `crates/sbfb-factory/src/pipeline.rs:15-70` — orchestration
- `crates/sbfb-factory/src/provider_router.rs` — `ExecutionTarget` (`:62-72`), `from_provider` (`:80-94`), `ollama_stream` (`:147-240`), `Ollama::default()` loopback (`:159-174`)
- `crates/sbfb-factory/src/preview_cmd.rs:13-46` ; `crates/sbfb-factory/src/terminal.rs:69-89`
- `crates/nexus-shell-daemon-core/src/blob_serve.rs` — `BLOB_SERVE_CSP` (`:286`), COOP (`:288`), COEP (`:290`), service iframe (`:8`)
- `tools/factory-operator/src/components/ContextPackBuilder.tsx:203`, `ExecutionChat.tsx:35`
- `tools/factory-ui/src/readonly/index.ts:3-27`, `PreviewList.tsx`, `types.ts:40-46`

**Études connexes** : `.planning/research/factory_embedded_ide_study.md` (pivot Node A/B/C), `.planning/research/factory_as_client_gap_analysis.md`.

---

*Document de recherche — synthèse de 6 workflows ultracode (agents Opus 4.8 1M), lecture du code réel des deux côtés. Read-only ; aucune décision actée ; sprint 77 non touché. 2026-06-22.*
