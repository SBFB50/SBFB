# Sprint 79 — Kickoff : Capacité Factory « app-authoring » (maîtrise anime.js + daisyUI dans le process de fabrication d'apps SBFB)

> **Sprint Factory dédié, orthogonal au compute.** Il EXÉCUTE un design déjà
> conçu et durci par deux études Workflow ultracode (4 lecteurs ancrés code réel
> + 2 architectes + synthèse adversariale, puis durcissement par preuve) :
> `examples/daisyui-animejs-showcase/knowledge/factory-integration-design.md`
> + `…/factory-integration-hardened.md`. **Rien n'est à re-concevoir.** Le sprint
> transforme la maîtrise UI (animation anime.js + composants daisyUI, tous deux
> annotés CSP-par-primitive/classe) en une **CAPACITÉ DU PROCESS Factory** :
> un module de connaissance versionné + un prompt-kind d'authoring + un gate CSP
> déterministe Rust qui importe `BLOB_SERVE_CSP` comme source de vérité.

**Écrit** : 2026-06-23.
**Type** : **sprint Factory dédié** (orthogonal au compute/sharding). Pas de feature
wire/réseau ; 0 bump wire, 0 dépendance nouvelle (faits canoniques du design durci).
Le travail vit dans `crates/sbfb-factory/` + `docs/factory/knowledge/` + `prompts/agent/`,
plus une factorisation cross-crate du ruleset CSP (`nexus-shell-daemon-core`).
**Budget de phases** : A→G d'un bloc (8 questions tranchées #3 : directive « sprints
ultra-complets », 0 defer du cœur). Le découpage exact en lettres reste un détail de
cadrage repris ci-dessous ; le nombre de phases n'est jamais plafonné (README §4).
**Numéro/insertion roadmap** : **S79, mais ARBITRAGE PO au boot** (cf. §« Ordre vs S78 »).
**Version archive** : v2.1 (OPEN) — capacité Factory, pas de nouvelle version.

---

## Objectif produit

Donner à **tout agent qui fabrique une app SBFB** (Claude / GPT / Codex / modèle
local / humain) la **maîtrise UI CSP-safe par construction** : composer des
composants daisyUI (structure) avec des animations anime.js (mouvement) en
respectant **mécaniquement** la CSP du bac à sable (`connect-src 'none'`, origine
opaque, COEP `require-corp`), sans CDN, sans fetch runtime, vendoré same-origin.

La capacité est une **CAPACITÉ DE PROCESS, pas un runtime ni une vitrine**
(`factory-integration-design.md:13`). Elle a trois étages, tous additifs, tous via
l'infra existante, 0 nouvelle autorité :

1. **Module de connaissance versionné** repo-visible sous `docs/factory/knowledge/{animejs,daisyui}/`,
   haché+signé **gratuitement** par `provenance::compute_output_hash` (tree-walk
   blake3) + FG8 dès qu'il est dans le source. Asset de process, **jamais dans
   l'archive d'une app** (donc 0 impact FG6 lock==prov).
2. **Prompt-kind `app-authoring`** (registre `PROMPT_KINDS`, `process.rs:7-16`),
   fiche-passerelle condensée (synthèse distillée + 9 pièges CSP durs + doctrine de
   vendorisation UMD + pointeurs vers les couches lourdes), servie par
   `sbfb-factory process prompt --kind app-authoring --provider {claude|gpt|local}`,
   vendor-neutre via `depth` + `strip_cloud_references`.
3. **Gate CSP déterministe Rust** branché au pipeline publish à côté de FG5/FG6,
   qui **importe `BLOB_SERVE_CSP`** comme source de vérité (jamais re-hardcodée),
   complète les 2 directives manquantes confirmées (`form-action`/`base-uri`), et
   **bloque** la publication d'une app non conforme dès son introduction.

La connaissance est **CONSOMMÉE/AFFICHÉE, jamais autoritaire** : autorité descendante
process > RRV > Factory tenue, le module n'émet **aucun verdict PASS**
(artifact-draft anti-PASS préservé, `chat_history_authoritative=false`).

## Pourquoi maintenant

- Les deux packs de connaissance sont **prêts** : anime.js v4.5 complet (93 primitives
  toutes annotées `sbfb_csp.usable`, 52 démos, 419 pages doc, synthesis actionnable —
  `knowledge/README.md`) **ET** daisyUI 5.5.23 × Tailwind 4.3.1 (68 composants,
  68 CSP-usable / 0 à risque, `MANIFEST.json` + hashes blake3-court — `knowledge/daisyui/`).
  Le « seul vrai travail d'extraction » annoncé en design est **déjà fait** : Phase E
  bascule d'« extraction » à **promotion + relocalisation** (cf. §Scope).
- Le contrat CSP est **prouvé de visu** dans le code réel (`blob_serve.rs:286`) et le
  GAP de `check-csp.mjs` est confirmé (`form-action`/`base-uri` manquants, drift esm→umd).
  Le contrat de gate est donc figeable sans incertitude technique.
- C'est le **maillon manquant du pipeline Factory** : aujourd'hui la guidance d'authoring
  + la contrainte CSP vivent uniquement **inline dans les templates** (commentaires HTML
  + README), il n'existe **aucune** connaissance d'authoring d'app dans le process ni dans
  le crate (`factory-integration-design.md:177` factory-cli-process). La maîtrise UI est
  donc invisible à tout agent qui ne lit pas un template par hasard.

## Scope

### In (A→G, 0 defer du cœur)
- **A — Promotion pack anime.js** : déplacer `examples/daisyui-animejs-showcase/knowledge/`
  (5 couches : primitives/PRIMITIVES, examples-bank/EXAMPLES, docs/DOCS, anime-types.d.ts,
  synthesis) → `docs/factory/knowledge/animejs/` + `MANIFEST.json` (version v4.5.0, date
  snapshot 2026-06-23, hash des couches, table verdict CSP déjà présente). `provenance.rs`
  INCHANGÉ (tree-walk récursif). Asset hors workspace d'app (0 impact FG6).
- **B — Prompt-kind `app-authoring`** (1er geste, le plus direct) : créer
  `prompts/agent/app-authoring.md` (synthèse distillée + 9 pièges CSP durs verbatim +
  doctrine vendorisation UMD + pointeurs hash vers couches lourdes) ; ajouter `app-authoring`
  à `PROMPT_KINDS` (`process.rs:7-16`). Le test `prompt_kinds_resolve_to_existing_files`
  (`process.rs:888-905`) garantit **mécaniquement** le couplage (build cassé si kind sans
  fichier). 0 exécution, rollback = supprimer 1 `.md` + 1 entrée.
- **C — Injection context-pack + routing zone UI** : champ `authoring_knowledge {path, hash}`
  (modèle `process_docs`, 1 ligne `file_hash` additive) dans `handle_context_pack`
  (`operator_server.rs:355-427`) + `handle_chat_session` (`:648-700`). Zone
  « UI/animation/design app SBFB » ajoutée aux Routing-tables des 2 `SKILL.md`
  (preflight + review), comme la zone « lib externe » pointe déjà vers context7.
- **D — Gate CSP déterministe Rust** : `run_gate_authoring_csp(workspace)` dans
  `gates.rs`, **importe `BLOB_SERVE_CSP`** de `nexus-shell-daemon-core` ; reprend les 13
  regex NETWORK du **CODE** `check-csp.mjs:23-37` (pas du commentaire périmé) + CSS_URL_ALLOW
  + 3 tiers (authored/compiled/vendored) ; **AJOUTE les 2 règles manquantes** confirmées
  (`form-action 'none'` → interdire `<form action=>`/`action=` dynamiques ; `base-uri 'none'`
  → interdire `<base href>`) + `object-src`/`frame-src` ; tree-walk WalkDir comme FG5 ;
  branché à `publish.rs:14` à côté FG5/FG6 ; **BLOQUANT dès l'introduction** ; tests fixtures
  clean/dirty ; doc `docs/factory/FACTORY_GATES.md`.
- **Phase factorisation (peut être D ou phase dédiée)** : **source unique du ruleset CSP**.
  La vérité CSP est aujourd'hui dupliquée/désync à 3 endroits (`blob_serve.rs:286` canonique,
  `check-csp.mjs:3-12` commentaire périmé+incomplet, docstring `http.rs:234`). Export Rust
  `BLOB_SERVE_CSP` + manifeste de règles machine-lisible dérivé d'elle ; `check-csp.mjs` ET
  le gate Rust consomment ce manifeste ; **test cross-crate** asserttant que NETWORK couvre
  toutes les directives `'none'` de `BLOB_SERVE_CSP` (anti-drift). Corrections immédiates :
  commentaire `check-csp.mjs:12` esm→umd + complétion `form-action`/`base-uri`.
- **E — Pack daisyUI : PROMOTION (pas extraction)** : le pack **existe déjà**
  (`knowledge/daisyui/`, 68 composants, `MANIFEST.json`, hashes). E = relocaliser →
  `docs/factory/knowledge/daisyui/` + figer le trio résolu (daisyUI 5.5.23 / Tailwind 4.3.1 /
  anime 4.5.0) + valider/auditer les cas à risque (`url()`/`@apply`/`backdrop-filter`/`mask`/
  SVG `fill-*`) + étendre `app-authoring.md` daisyUI (compositions composant×anime CSP-safe).
- **F — Copilote Ollama + starter template daisyui** : bloc capacité UI prepend dans
  `assemble_prompt` (`llm_bridge.rs:61-93`) AVANT dispatch `ExecutionTarget::Ollama` keyless
  (après le gate SENSITIVE_ACTIONS) ; 5e `TemplateConfig` `daisyui` (`template_engine.rs:170-203,
  90-126`) vendorant `vendor/anime.umd.js` (UMD classic-script) + `app.css` daisyUI compilé
  same-origin (recette `tailwindcss -i src/input.css -o app.css --minify`, `@import "tailwindcss"
  source(none)` + `@source` explicites, **template lean = retirer les 8 thèmes built-in**),
  thème défaut `sbfb-reflect` (oklch dark custom), README CSP, décline le pattern react UMD no-build.
- **G — Self-check runtime + wrap-up** : rejeu de l'app dans le viewer iframe `blob_serve`
  sous la CSP **réelle** de prod (origine opaque, `connect-src 'none'`, COEP require-corp) via
  postMessage — filet RUNTIME pour les `url()`/`@font-face` construits à l'exécution qui
  échappent au lint statique ; confirmation `BLOB_SERVE_CSP == contrat de gate` ; PATTERNS +
  FACTORY_GATES doc ; **T1 E2E Playwright hermétique BLOQUANT** + **T2 acceptance artefact JSON**.

### Out (différé / explicitement hors-scope)
- **Wrapper `.claude/skills/app-authoring`** : NON au 1er jet (question #7 tranchée
  recommandée). Le prompt-kind portable suffit (déjà cross-provider via `prompt_data` +
  `strip_cloud_references`). Additif/différable plus tard (devra référencer
  `prompts/agent/app-authoring.md`).
- **Alliance PR-plugin Open CoDesign** : DIFFÉRÉE (OCD sans surface plugin avant sa v1.0 ;
  mono-auteur ; étude factory_opencodesign).
- **Auto-fetch de fraîcheur** : interdit (`connect-src 'none'`). Re-extraction MANUELLE au
  bump, gouvernée par date+version dans `MANIFEST.json`.
- **Nouvelle route daemon / autorité de verdict** : aucune. Daemon neutre, scellage 100% Factory.

## Les 2 assets + leur état

| Pack | État | Emplacement source | Versions figées |
|---|---|---|---|
| **anime.js** | **Complet** (5 couches, 93 primitives toutes CSP-usable, 52 démos, 419 pages doc, 70 types, synthesis actionnable) | `examples/daisyui-animejs-showcase/knowledge/` | anime.js **4.5.0**, snapshot **2026-06-23** |
| **daisyUI** | **Complet** (68 composants / 68 CSP-usable / 0 à risque, theming oklch 35 thèmes, synthesis compositions ×anime, `docs-llms.txt` verbatim, `MANIFEST.json` + hashes) | `examples/daisyui-animejs-showcase/knowledge/daisyui/` | daisyUI **5.5.23**, Tailwind **4.3.1** (cli+node+oxide), anime **4.5.0** ; thème **sbfb-reflect** |

> **Implication kickoff** : Phase E n'est PAS une extraction risquée (le design durci la
> notait « seul vrai travail d'extraction » avant que le pack daisyUI existe). C'est une
> **promotion + relocalisation + audit des cas à risque**, au même statut que A. Le risque
> « pack daisyUI inexistant » du design est **éteint** ; reste l'audit des classes
> `url()`/`@apply`/`backdrop-filter`/`mask`/SVG-fill (le pack les liste : `knowledge/daisyui/README.md:32-39`).

## Le contrat CSP (gelé, source de vérité = `BLOB_SERVE_CSP`)

- **CSP servie au contenu untrusted** (`blob_serve.rs:286`, vérifiée de visu) :
  `default-src 'self' 'unsafe-inline' 'unsafe-eval' data: blob:; connect-src 'none';
  worker-src 'none'; frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none';
  frame-ancestors *; sandbox allow-scripts`.
  + COOP `same-origin` (`:288`), COEP `require-corp` (`:290`), CORP `cross-origin`, nosniff.
  Posée par `blob_serve_csp_middleware` sur CHAQUE réponse y compris 404 (`http.rs:551-577`).
  iframe shell : `sandbox="allow-scripts"` sans `allow-same-origin` ⇒ origine opaque/null
  (`BrowsedProject.tsx:604`).
- **Gate Rust (règles)** :
  - **Importer** `BLOB_SERVE_CSP` (PAS re-hardcoder, PAS lire le commentaire `check-csp.mjs:3-4`).
  - Reprendre les **13 regex NETWORK** du CODE `check-csp.mjs:23-37` : `fetch`/`XMLHttpRequest`/
    `WebSocket`/`EventSource`/`navigator.sendBeacon` (=`connect-src 'none'`) ; `new Worker`/
    `new SharedWorker`/`importScripts`/`navigator.serviceWorker` (=`worker-src 'none'`) ;
    remote `<link href https:>`/`<script src https:>`/`@import url()`/remote `url()` asset (=`default-src`).
  - **3 tiers** : authored `{index.html,app.js}` = 0 http(s) absolu + 0 NETWORK ; compiled
    `{app.css}` = 0 NETWORK + chaque URL absolue ∈ CSS_URL_ALLOW `{http://www.w3.org/2000/svg,
    http://www.w3.org/1999/xlink, https://tailwindcss.com}` ; vendored `{vendor/anime.umd.js}` = 0 NETWORK live.
  - **AJOUTER (GAP confirmé)** : `form-action 'none'` → `<form action=>`/`action=` dynamiques ;
    `base-uri 'none'` → `<base href>` ; + `object-src 'none'` → `<object>`/`<embed>` ;
    `frame-src 'none'` → iframes imbriquées.
  - Valider : `app.css` en `<link rel=stylesheet href=relatif>` + `vendor/*.js` en classic
    `<script src>` same-origin, **JAMAIS `type=module`** (CORS impossible en origine opaque
    sous COEP require-corp), jamais CDN/fetch.
  - **Test cross-crate** : asserter que NETWORK couvre toutes les directives `'none'` de
    `BLOB_SERVE_CSP` (anti-drift futur).

## Day-0 — décisions gelées (NE PAS re-débattre)

1. **Architecture** = module de connaissance versionné `docs/factory/knowledge/{animejs,daisyui}/`
   + prompt-kind `app-authoring` + gate CSP déterministe Rust important `BLOB_SERVE_CSP`.
2. **Emplacement** = `docs/factory/knowledge/` (existe déjà via FACTORY_GATES.md), PAS
   `prompts/agent/` (répertoire plat de kinds, invariant testé) — question #1 resolu-preuve.
3. **Nom du kind** = `app-authoring` (rôle/action, cohérent avec base/universal/handoff/
   preflight/phase-review/commit-body/audit-gate/phase-auditor) — question #5 recommandé.
4. **Source CSP unique** : importer `BLOB_SERVE_CSP`, jamais re-hardcoder ni lire le
   commentaire périmé ; factoriser en manifeste partagé + test cross-crate — question #6 resolu-preuve.
5. **Gate BLOQUANT dès son introduction** (pas advisory-puis-bascule) — question #8 recommandé.
6. **Pas de wrapper `.claude/skills/` au 1er jet** — question #7 recommandé.
7. **Copilote via `ExecutionTarget::Ollama` keyless**, jamais pi-ai/SDK direct ; `provider_router`
   inchangé ; provider (qui lit) ≠ backend exécution worker (process.rs:24-34/D8).
8. **Scellage 100% Factory non-délégable** : le lint authoring est ADDITIF, aucune dispense
   CSP/COEP/COOP/FG5/FG6/FG8/Ed25519 ; FG6 lock==prov reste vrai (tout asset vendoré hashé).
9. **Vendorisation same-origin UMD** (jamais ESM/CDN) ; daisyUI/Tailwind compilé build-time
   en `app.css` ; Tailwind-CDN + Google Fonts interdits (memory).
10. **Versions figées** : daisyUI 5.5.23 / Tailwind 4.3.1 (cli+node+oxide) / anime 4.5.0 ;
    thème défaut `sbfb-reflect` ; figer les versions résolues du lock, PAS les carets.
11. **Sprint A→G d'un bloc, 0 defer du cœur** (directive sprints ultra-complets) — questions #2/#3.

## Ordre vs S78 (ARBITRAGE PO requis au boot)

> **À TRANCHER PAR LE PO avant de démarrer.** Le design durci recommande **S79+ dédié,
> JAMAIS dans S78** (S78 = orchestrateur de session sharding in-vivo + benchmark live + 4
> carries 3/3, déjà saturé ; aucune dépendance technique anime/daisyUI → sharding ;
> question #2 reste-PO). Le PO a demandé de **démarrer ce sprint à la prochaine session**,
> potentiellement **AVANT** la fermeture du carry P1 sharding S78 (feature shard PROVISIONAL).
> Cet arbitrage n'est PAS tranché ici :
> - **Option 1** : fermer d'abord S78 (carry P1 sharding) puis ce sprint en S79.
> - **Option 2** : intercaler ce sprint maintenant (le numéroter S79 quand même, S78 décalé
>   ou parallèle), car la capacité Factory est orthogonale et sans dépendance.
>
> **Phase 0 = audit gate du sprint précédent réellement clos au démarrage.** Si S78 est encore
> ouvert, Phase 0 = audit gate **S77** (`sprint78_audit_plan.md` déjà écrit, tip `0f597cf`).
> Si S78 a été fermé avant, Phase 0 = audit gate **S78**. À déterminer au boot selon l'état réel.

## Gate de testabilité par-sprint (README §4, NON-négociable)

- **T1** : E2E Playwright **hermétique BLOQUANT** au wrap-up (+ CI à chaque push). Cible :
  rejeu self-check viewer (Phase G) d'une app daisyui+anime fixture sous CSP réelle ; +
  fixtures gate CSP clean (publish OK) / dirty (publish bloqué).
- **T2** : acceptance **artefact JSON machine-lisible** (`PASS` / `BLOCK{diagnosis}` /
  `RIG-ABSENT`) — jamais un `DIFFERE-materiel` en prose. Le gate CSP étant un check statique
  d'assets local (pas de rig 2-machines), T2 doit pouvoir être **PASS** déterministe (build
  `tailwindcss --minify` + scan gate + self-check viewer en local).

## Invariants

- **Connaissance CONSOMMÉE/AFFICHÉE, jamais autoritaire** : 0 verdict PASS, artifact-draft
  anti-PASS préservé, `chat_history_authoritative=false`.
- **0 bump wire, 0 dep nouvelle** : nouveau kind + champ context-pack additif + gate additif
  + template additif ; contrat 8→9 kinds étendu proprement ; SBFB.json schema_version=2 inchangé.
- **Asset de process repo-visible, jamais dans l'archive d'app** (hors workspace ⇒ 0 impact FG6).
- **Gate déterministe** (regex/scan statique), aucun composant ML/scoring opaque (FACTORY_GATES.md:205-207).
- **Discipline commit** : 1 commit par phase `feat(scope): Sprint 79 Phase X — titre`, body riche
  avec delta de tests cumulé + scope cuts respectés ; preflight G8 → review → Codex avant chaque commit.

## Références
- **Design durci (LA référence)** : `examples/daisyui-animejs-showcase/knowledge/factory-integration-design.md`
  + `…/factory-integration-hardened.md`.
- **Packs** : `…/knowledge/README.md` (anime.js) ; `…/knowledge/daisyui/README.md` + `…/daisyui/MANIFEST.json`.
- **Code Factory touché** : `crates/sbfb-factory/src/process.rs` (PROMPT_KINDS l.7-16, test l.888-905),
  `operator_server.rs` (handle_context_pack l.355-427, handle_chat_session l.648-700),
  `gates.rs`, `publish.rs:14`, `template_engine.rs:170-203,90-126`, `llm_bridge.rs:61-93` ;
  `prompts/agent/*.md` ; `docs/factory/FACTORY_GATES.md` (FG0-FG10).
- **CSP** : `crates/nexus-shell-daemon-core/src/blob_serve.rs:286` (`BLOB_SERVE_CSP`) ;
  `examples/daisyui-animejs-showcase/scripts/check-csp.mjs` (à ré-incarner en Rust ;
  GAP confirmé : manque `form-action`/`base-uri`, drift esm→umd l.12 vs code l.75).
- **Études antérieures** : memory `factory_opencodesign_design_integration_study` (PRENDRE
  classifieur HTML-verbatim + inline-assets + lint statique ; copilote Ollama jamais pi-ai ;
  alliance PR-plugin DIFFÉRÉE).
