# Sprint 79 — Revue de conception : capacité Factory anime.js + daisyUI

> Revue de conception du sprint S79 « app-authoring ». Synthèse des deux études
> Workflow ultracode déjà produites (`factory-integration-design.md` +
> `factory-integration-hardened.md`) : verdict, 8 questions tranchées avec evidence
> fichier:ligne, contrat CSP détaillé, risques, alignement avec les décisions
> OpenCoDesign antérieures. **Rien n'est re-conçu** : ce document fige les preuves et
> les décisions pour qu'une session fraîche EXÉCUTE sans rouvrir le design.

## Verdict

Le design est **durci et prêt à exécuter**. Les deux propositions (« minimale » et
« deepest ») sont la **même architecture** — module de connaissance versionné haché par
provenance + prompt-kind d'authoring + lint CSP déterministe ré-incarné de `check-csp.mjs`
en Rust — divergeant seulement par l'ampleur du scope livré d'un coup
(`factory-integration-design.md:7`). L'**hybride-phase** est recommandé puis surclassé par
la directive PO « sprints ultra-complets » : **A→G d'un bloc, 0 defer du cœur** (question #3).

**3 questions résolues-preuve** (lecture code réel, vérifiées de visu) : #1 emplacement
`docs/factory/knowledge/`, #4 snapshot versions, #6 contrat CSP == `BLOB_SERVE_CSP`.
**3 recommandations de policy fondées code** : #5 nom `app-authoring`, #7 pas de wrapper
skills au 1er jet, #8 gate bloquant dès l'introduction. **2 questions de roadmap pur reste-PO** :
#2 séquençage S79+, #3 découpage en phases.

**Fait neuf depuis le design** : le pack daisyUI **existe maintenant** (`knowledge/daisyui/`,
68 composants, `MANIFEST.json` hashes blake3, 68 CSP-usable / 0 à risque). Le risque
« pack daisyUI inexistant » du design est **éteint** ; Phase E devient une **promotion +
audit des cas à risque**, pas une extraction. C'est le seul écart factuel entre le design
écrit et l'état réel du repo, et il **réduit** le risque du sprint.

**Honnêteté** : deux points non mesurés ici, à confirmer en phase — (a) le poids exact
d'`app.css` minifié avant/après retrait des 8 thèmes built-in (template lean) ; (b) que le
script npm `check:csp` soit effectivement câblable comme base du gate de publish (il a été
lu, il EST la base des règles, reste à vérifier son branchement pipeline).

## Les 8 questions tranchées

| # | Question | Décision | Type | Evidence |
|---|---|---|:--:|---|
| 1 | Emplacement canonique du module | `docs/factory/knowledge/{animejs,daisyui}/`, PAS `prompts/agent/` (répertoire plat). 0 modif `process.rs` pour héberger ; ajout context-pack = 1 ligne `file_hash` additive (modèle `process_docs`). | **resolu-preuve** | `docs/factory/` existe (FACTORY_GATES.md), `knowledge/` absent ; `prompts/agent/` plat avec invariant testé `prompt_kinds_resolve_to_existing_files` (`process.rs:888-905`) ; `handle_context_pack` set fixe code-en-dur (`operator_server.rs:404-409`) |
| 2 | Séquençage roadmap (S79+ vs S78) | Recommandation : sprint Factory **dédié S79+, JAMAIS dans S78** (sharding orchestrateur in-vivo + benchmark + 4 carries, déjà saturé). Aucune dépendance technique anime/daisyUI → sharding. Numéro/insertion exacts = PO. | **reste-PO** | CLAUDE.md état actuel (S78 chargé) ; 0 dépendance technique |
| 3 | Scope 1er sprint (A→D puis E→G vs A→G d'un bloc) | Recommandation : **A→G d'un bloc** (directive sprints ultra-complets, 0 defer du cœur ; dette/gates = phases du sprint). Découpage exact en lettres = cadrage PO au kickoff. | **reste-PO** | memory `feedback_ultra_complete_sprints` ; README §4 budget de phases non plafonné |
| 4 | Snapshot daisyUI/Tailwind + thème défaut | Figer le **trio résolu du lock** : daisyUI **5.5.23**, Tailwind **4.3.1** (tailwindcss+@tailwindcss/cli+node+oxide), anime **4.5.0**. Thème défaut **sbfb-reflect** (oklch dark custom, default:true). Figer les versions résolues, PAS les carets. Retirer les 8 thèmes built-in pour un template lean. | **resolu-preuve** | `package-lock.json:2409` (daisyui 5.5.23), `:3668` (tailwind 4.3.1), `:1827` (@tailwindcss/cli 4.3.1) ; `src/input.css:20-24` (sbfb-reflect default:true), `:13-15` (8 thèmes opt-in) ; `index.html:7` (data-theme) |
| 5 | Nom du kind (app-authoring / ui-knowledge / design-knowledge) | **app-authoring**. Les kinds existants sont des RÔLES/étapes orient-action (base, universal, handoff, preflight, phase-review, commit-body, audit-gate, phase-auditor), pas des domaines de savoir. Le corpus va dans `docs/factory/knowledge/` ; le kind ORCHESTRE son usage ⇒ nom d'activité. Style tirets cohérent (phase-review, audit-gate). | **recommandé** | `process.rs:7-16` (PROMPT_KINDS = rôles), `:888-905` (test fichier obligatoire) |
| 6 | CSP daemon == contrat du gate | **resolu-preuve**. La CSP réelle servie au contenu untrusted EST `BLOB_SERVE_CSP`. Le gate Rust DOIT **importer** cette constante (PAS re-hardcoder, PAS lire le commentaire `check-csp.mjs:3-4` périmé/incomplet). Drift esm/umd confirmé. | **resolu-preuve** | `blob_serve.rs:286` (chaîne complète, lue de visu) ; `check-csp.mjs:3-4` (commentaire sous-ensemble), `:12` 'anime.esm.js' vs `:75` read('vendor/anime.umd.js') |
| 7 | Wrapper `.claude/skills/` en plus du prompt-kind ? | **NON au 1er jet**. Le prompt-kind portable suffit (consommable cross-provider via `prompt_data` + `strip_cloud_references`). Les skills existants sont des GATES de process (verdict EXECUTE/PASS), pas des capacités d'authoring. Additif/différable ; devra référencer `prompts/agent/app-authoring.md`. | **recommandé** | `process.rs:796-835` (prompt_data portable), `:34/:89-101` (providers + strip local), `:907-948` (test wrapper→prompt) ; `.claude/skills/nexus-phase-*` = gates |
| 8 | Gate CSP bloquant dès D, ou advisory puis bascule ? | **BLOQUANT dès l'introduction**, PAS advisory. Le gate de testabilité par-sprint (README §4) impose un vérifiant déterministe ; un gate advisory laisserait fuiter une app violant `connect-src 'none'` (exfiltration). Coût faible (check statique déjà prouvé via `check-csp.mjs`). La lettre de phase exacte = cadrage PO. | **recommandé** | README §4 gate testabilité ; `check:csp` runnable (`package.json:11`) ; `blob_serve.rs:283-285` rationale anti-exfiltration form-action |

## Contrat CSP (détaillé)

- **CSP servie** (`BLOB_SERVE_CSP`, `blob_serve.rs:286`, vérifiée de visu) :
  `default-src 'self' 'unsafe-inline' 'unsafe-eval' data: blob:; connect-src 'none';
  worker-src 'none'; frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none';
  frame-ancestors *; sandbox allow-scripts`. + COOP `same-origin` (`:288`), COEP `require-corp`
  (`:290`), CORP `cross-origin` (`http.rs:572-575`), `X-Content-Type-Options: nosniff` (`http.rs:558`).
  Posée par `blob_serve_csp_middleware` sur CHAQUE réponse y compris 404 (`http.rs:551-577`).
  iframe shell : `sandbox="allow-scripts"` sans `allow-same-origin` ⇒ origine opaque/null
  (`BrowsedProject.tsx:604`).
- **`check-csp.mjs` fidèle ?** — **fidélité PARTIELLE** (confirmé de visu). Fidèle sur le COEUR :
  les 13 regex NETWORK (`:23-37`) couvrent `connect-src 'none'` (fetch/XHR/WebSocket/EventSource/
  sendBeacon), `worker-src 'none'` (Worker/SharedWorker/importScripts/serviceWorker), `default-src`
  (@import url()/remote url()/remote link/script). **ÉCARTS** : (1) commentaire CSP `:3-4` =
  SOUS-ENSEMBLE (omet base-uri, form-action, frame-ancestors, sandbox) ; (2) NETWORK ne contient
  **aucune** règle `form-action` ni `base-uri` (les deux dans la CSP réelle ; form-action
  explicitement justifiée anti-exfiltration `blob_serve.rs:283-285`) ; (3) DRIFT doc : commentaire
  `:12` dit 'anime.esm.js' alors que code `:75` lit 'vendor/anime.umd.js' (code correct, commentaire
  périmé) ; (4) check-csp ne vérifie pas COOP/COEP/CORP/nosniff (hors-scope : headers runtime daemon).
- **Règles du gate Rust** :
  1. Importer `BLOB_SERVE_CSP` de `nexus-shell-daemon-core` (source de vérité).
  2. Reprendre les 13 regex NETWORK du **CODE** `check-csp.mjs:23-37` (pas du commentaire).
  3. 3 tiers : authored `{index.html,app.js}` = 0 http(s) absolu + 0 NETWORK ; compiled `{app.css}`
     = 0 NETWORK + chaque URL absolue ∈ CSS_URL_ALLOW `{http://www.w3.org/2000/svg,
     http://www.w3.org/1999/xlink, https://tailwindcss.com}` ; vendored `{vendor/anime.umd.js}` = 0 NETWORK live.
  4. **AJOUTER les 2 règles manquantes (GAP confirmé)** : `form-action 'none'` → `<form action=>`/
     `action=` dynamiques ; `base-uri 'none'` → `<base href>`.
  5. AJOUTER `object-src 'none'` → `<object>`/`<embed>` ; `frame-src 'none'` → iframes imbriquées.
  6. Valider `app.css` en `<link rel=stylesheet href=relatif>` + `vendor/*.js` en classic `<script src>`
     same-origin, JAMAIS `type=module` (CORS impossible en origine opaque sous COEP require-corp).
  7. **Test cross-crate** : NETWORK couvre toutes les directives `'none'` de `BLOB_SERVE_CSP`.
- **Action factorisation (source unique)** : vérité aujourd'hui dupliquée/désync à 3 endroits
  (`blob_serve.rs:286` canonique, `check-csp.mjs:3-12` commentaire périmé+incomplet, docstring
  `http.rs:234`). (i) Export Rust `BLOB_SERVE_CSP` + manifeste de règles machine-lisible dérivé ;
  (ii) `check-csp.mjs` ET le gate Rust consomment ce manifeste plutôt que de re-dériver les regex ;
  (iii) test cross-crate de couverture. Corrections immédiates : commentaire `check-csp.mjs:12`
  esm→umd + complétion NETWORK avec form-action + base-uri.

## Pièges CSP durs (à coder verbatim dans `app-authoring.md`)

- **box-shadow JAMAIS animée** (non-composite, parse complexe) : glow = box-shadow **statique** sur
  `::after`, seule l'`opacity` transitionne (GPU-composite) — `primitives.json:273,1289`.
- **SVG peint en `var(--color-*)`/`color-mix(in oklch,…)`** : les utilitaires Tailwind `fill-*`/
  `stroke-*` **ne compilent PAS** de façon fiable dans l'iframe (piège le plus récurrent) —
  `primitives.json:274,374,1785` + `knowledge/daisyui/README.md:35`.
- **`createMotionPath`** : l'élément déplacé doit avoir `cx=0 cy=0` (translate s'AJOUTE à la
  géométrie) — `primitives.json:224,1842`.
- **`morphTo`** : mono-tracé, même type d'élément (path d↔d ou polygon/polyline points↔points),
  prend seulement le 1er élément résolu — `primitives.json:1904,1950`.
- **`prefers-reduced-motion`** non géré par anime → brancher l'état-final (revert/seek(duration)/
  utils.set/modifier(1)/innerHTML) + garde-fou CSS global `durations 0.001ms !important` —
  `app.js:20,43`, `input.css:779-788`, `primitives.json:966,1161`.
- **daisyUI cas à risque** (le pack les liste, 0 blocage mais usages à éviter) : `<img src>`/
  `background-image:url()`/`mask`/`hero`/`avatar` http distant → bloqué (servir data:/relatif) ;
  `<form>` submit → bloqué (idiome `div`/`button type=button` + handler local) ; composants pilotés
  JS (calendar/carousel/countdown/radial-progress/toast/modal.showModal) → habillage CSS safe mais
  comportement à coder en JS local vendoré ; `backdrop-filter` autorisé (coût perf) — `knowledge/daisyui/README.md:32-39`.
- **Doctrine vendorisation** : anime.js en UMD/global `window.anime` (script classique), JAMAIS
  `type=module`/ESM (origine opaque + COEP interdisent le fetch CORS) ; daisyUI/Tailwind compilé
  build-time en `app.css` ; purge `@import "tailwindcss" source(none)` + `@source` explicites.

## Risques

- **Poids tokens** : `docs.json` 781KB + `primitives.json` 314KB dépassent tout contexte d'un
  provider local. **Mitigation** : `app-authoring.md` = synthèse distillée (~64KB) + pièges seuls par
  défaut ; couches lourdes RÉFÉRENCÉES par chemin+hash, chargées en `depth=deep` (mécanisme existant).
- **Dérive lint vitrine vs gate Factory** : `check-csp.mjs` codé en dur pour l'arbre vitrine (chemins
  relatifs) ET commentaire périmé. **Mitigation** : re-incarnation Rust **paramétrable par workspace**
  + CODE comme source unique des règles + source partagée des regex CSP + test cross-crate anti-drift.
- **Gate statique insuffisante seule** : un `url()` construit à l'exécution / `@font-face` dynamique
  échappe au lint regex. **Mitigation** : le self-check runtime viewer (Phase G) est le filet
  **OBLIGATOIRE**, pas optionnel, pour les animations qui s'exécutent dans le temps.
- **Fraîcheur** : anime 4.5.0 / daisyUI 5.5.23 / Tailwind 4.3.1 snapshot 2026-06-23 périment au bump.
  **Mitigation** : date+version dans `MANIFEST.json` + re-extraction MANUELLE (pas d'auto-fetch,
  conforme `connect-src 'none'`) ; gate G2 long-life freshness peut surveiller la date.
- **Audit des classes daisyUI à risque** (risque résiduel après extinction du « pack inexistant ») :
  `url()`/`@apply`/`backdrop-filter`/`mask`/SVG-fill subtilement non-CSP-safe une fois purgées.
  **Mitigation** : auditer ces cas explicitement (Phase E) + le lint mécanique attrape ce que le
  verdict advisory rate (double filet).
- **Sur-ingénierie si mal borné** : kind+gate+daisyUI+template+copilote+self-check = sprint Factory
  complet. **Mitigation** = ordonner A→G en phases additives d'UN sprint dédié, jamais des defers
  ni glissé dans S78.
- **CSP réelle blob-serve == contrat de gate** : confirmé de visu (`BLOB_SERVE_CSP`), reste à confirmer
  en Phase G que le viewer self-check rejoue bien CETTE chaîne (et non une copie divergente).

## Alignement avec les décisions OpenCoDesign / études antérieures

- **1er geste = prompt-kind/SKILL.md « goût SBFB » 0-exécution rollback-gratuit** — honore
  `factory_opencodesign:299/444`.
- **Connaissance = couche prompt/skill read-only + tokens dans le source**, consommée par le
  copilote — jamais runtime tiers (OCD prior_decisions). Asset haché+signé **gratuitement** par
  `provenance::compute_output_hash` tree-walk blake3 + FG8 (l'aubaine « workspace=deliverable »).
- **Copilote via `ExecutionTarget::Ollama` loopback keyless**, jamais pi-ai/SDK direct ;
  `provider_router` inchangé.
- **Provider (qui lit le prompt) ≠ backend exécution worker** (Ollama/llama_cpp) NON unifiés — la
  profondeur s'adapte par provider sans toucher cet axe (`process.rs:24-34`/D8).
- **Scellage 100% Factory non-délégable** : le lint authoring est ADDITIF, aucune dispense
  CSP/COEP/FG5/FG6/FG8/Ed25519 (`factory_opencodesign:30`). FG6 lock==prov reste vrai.
- **Gates déterministes, pas de ML/scoring opaque** (`FACTORY_GATES.md:205-207`) : le lint CSP est un
  scan statique, pas un jugement LLM.
- **Tokens 2 couches** : per-app COPIE same-origin (`app.css` compilé), jamais référence CDN —
  Tailwind-CDN + Google Fonts explicitement interdits (memory).
- **Runtime vendoré same-origin UMD jamais ESM/CDN** (précédent template react S74 Phase C,
  `vendor-anime.mjs`) : la voie anime/daisyUI est build-time + vendore.
- **Knowledge dev-only jamais publié dans l'archive** (`README knowledge:3`) = exactement le statut
  d'un asset de process repo-visible (CLAUDE.md context-pack repo-visible vs mémoire de chat) ;
  hors workspace d'app ⇒ 0 impact FG6.
- **Autorité descendante process > RRV > Factory** : le module est consommé/affiché, jamais une
  autorité de verdict (`RRV_FACTORY_CONTRACT:54-85`) ; artifact-draft anti-PASS préservé ;
  `chat_history_authoritative=false`.
- **Maquette-first reste gate PROCESSUS** (avant FG4), pas un FGx cryptographique ; le gate CSP
  authoring vérifie la conformité SANDBOX (invariant souverain déterministe), distinct du jugement
  de goût maquette.
- **Pre-launch protocol** : nouveau prompt-kind + champ context-pack additif + gate additif + template
  additif = 0 bump wire, contrat 8→9 kinds étendu proprement, 0 dépendance nouvelle, SBFB.json
  schema_version=2 inchangé.
- **Alliance PR-plugin OCD DIFFÉRÉE** : aucune surface plugin touchée, posture C (ré-incarnation
  native) + B (vendorisation patchée) maintenue, mono-auteur respecté.

## Ajustements du plan (vs design écrit)

1. Le gate CSP **importe `BLOB_SERVE_CSP`** + complète form-action/base-uri + object-src/frame-src —
   crate Rust qui réutilise le coeur des 13 regex NETWORK, PAS une re-dérivation manuelle.
2. **Phase factorisation** : source unique du ruleset CSP (export Rust + manifeste partagé consommé
   par `check-csp.mjs` ET le gate) + test cross-crate de couverture des directives `'none'`.
3. Corrections immédiates : commentaire `check-csp.mjs:12` (esm→umd) + ajout form-action/base-uri à NETWORK.
4. Context-pack : ajout `file_hash` additif (1 ligne par doc, modèle `process_docs` `operator_server.rs:404-409`).
5. **Template lean** : retirer les 8 thèmes daisyUI built-in pour réduire `app.css` minifié.
6. Wrapper `.claude/skills/` NON inclus au 1er jet (additif, différable).
7. Gate CSP **bloquant dès sa phase d'introduction** (pas advisory), conforme gate de testabilité README §4.
8. Sprint **A→G d'un bloc** (directive sprints ultra-complets), 0 defer du cœur.
9. **Écart factuel à acter** : le pack daisyUI EXISTE (`knowledge/daisyui/`, 68 composants, MANIFEST
   hashes) — Phase E = promotion + audit des cas à risque, PAS extraction. Risque « pack inexistant » éteint.

## Reste au PO (à trancher au boot)

- **#2 Numéro/insertion roadmap** → reco : sprint dédié **S79+** après fermeture du carry P1 sharding
  S78 ; JAMAIS dans S78. **Mais le PO a demandé de démarrer ce sprint à la prochaine session,
  potentiellement AVANT S78** — arbitrage ordre à trancher (Option 1 : S78 d'abord ; Option 2 :
  intercaler maintenant). Aucune dépendance technique anime/daisyUI ↔ sharding.
- **#3 Découpage exact en phases** → reco : A→G d'un bloc, 0 defer du cœur. Lettres exactes = cadrage kickoff.
- **Verrouiller le lock du template Factory** → reco : figer le trio résolu (5.5.23/4.3.1/4.5.0), PAS
  les carets — un `npm install` ultérieur dériverait et casserait la reproductibilité du gate CSP.
- **Phase 0 = audit gate du sprint réellement clos au démarrage** : audit gate **S77** (`sprint78_audit_plan.md`
  écrit, tip `0f597cf`) si S78 encore ouvert ; sinon audit gate **S78**. À déterminer au boot.
- **À confirmer en phase** : poids exact d'`app.css` minifié avant/après retrait des 8 thèmes ; que
  `check:csp` (`package.json:11`) soit câblable comme base du gate de publish (règles lues, branchement à vérifier).
