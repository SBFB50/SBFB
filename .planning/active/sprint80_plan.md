# Sprint 80 — Plan : Refonte greenfield du front Factory Operator (bi-focal STEER/VERIFY)

> Phases dimensionnées par le **travail**, JAMAIS par LOC. Phase 0 = audit gate S79
> (déjà joué). Cœur bi-focal COMPLET (Arbitrage PO #1 : VERIFY-plein dans S80). Socle
> `tools/factory-ui` supersedé (Arbitrage PO #2). 1 commit atomique par phase
> `feat(scope): Sprint 80 Phase X — titre` ; preflight G8 → review Workflow → Codex
> avant chaque commit ; T1 hermétique grandit incrémentalement (BLOQUANT au wrap-up +
> CI chaque push), T2 artefact JSON committé.

---

## Phase 0 — Audit gate S79 (DÉJÀ JOUÉ)

- **Verdict** : CONDITIONAL PASS → PASS effectif. 0 P0 · 1 P1 nouveau (P1-1 gate CSP
  non-câblé sur `redeploy`) **RÉSOLU** Option A (`c0a2ffe`) · 8 P2 · 11 P3.
- **Commits** : `3d5d9dc` (research S80) + `96ed018` (findings + routing P1-1) +
  `c0a2ffe` (fix redeploy) + `7f51438` (note résolution).
- **Carries figés** : 2 P1 in-vivo ouverts (sharding S77 RIG-ABSENT, app-authoring
  `Not evidenced`) ; 8 P2/11 P3 → carry sprint dette Rust/docs nommé (P2-8 + P3-6 routés S80).

## Phase A — Préalable backend BLOQUANT : auth cookie HttpOnly + ServeDir + CSP Operator

- **But** : rendre SSE/WS exploitables en prod same-origin. Sans lui, `EventSource`/`WebSocket`
  = 401/403 sous `ServeDir` (ils ne posent pas d'en-tête custom). 1er geste, BLOQUANT.
- **Jobs/surfaces** : sécurité auth-transport. `auth.rs` (fallback cookie) + `operator_server.rs`
  (handler `GET /`, `ServeDir`, interaction CORS-credentials avec la layer CORS outer `:166`).
- **Livrables** : fallback Cookie `sbfb_operator` dans `auth_required` (header `x-sbfb-token`
  d'abord, `:229-262`) ; handler `GET /?token=<hex>` validant le token de query, posant
  `Set-Cookie` HttpOnly + SameSite=Strict + Path=/ + host-only puis redirect 303 vers `/` ;
  `tower_http::services::ServeDir` du bundle (pattern prouvé daemon `http.rs:512`) ; pas de
  `Secure` sur http loopback ; **CSP self-origin minimale Operator** (`default-src 'self'`).
  Garder Host + Origin-si-présent. Revue lens-sécurité (bearer = racine de confiance ; le
  cookie ne distribue pas de session librement sur loopback ; SameSite=Strict = stoppeur CSRF
  primaire ; Host = anti-rebinding).
- **Backend deps** : AJOUTÉ (Rust, crate `sbfb-factory` uniquement, **0 route au daemon**).
  `tower-http` `["cors","fs"]` déjà au workspace (`Cargo.toml:162`).
- **T1** : sous-test (1) ouverture loopback cookie-authentifiée (bootstrap → `Set-Cookie` →
  `/` sans 401) — exécutable dès cette phase contre un bundle minimal.

## Phase B — Scaffold greenfield + gates de discipline + jettison (factory-operator + factory-ui)

- **But** : poser le socle front vérifié et les filets mécaniques AVANT toute UI ; jeter le
  front actuel ET le socle `tools/factory-ui` (supersede S70).
- **Jobs/surfaces** : front scaffold. Backend : aucun (consomme `/api/*`).
- **Livrables** : React 19 (+ React Compiler, défaut) + Tailwind v4 CSS-first + tokens oklch
  `@theme` + Base UI install + Motion (`LazyMotion`+`m`) + Geist sans/mono vendoré. **Jeter**
  `tools/factory-operator` + `tools/factory-ui` (supersede `CLAUDE.md:495-496` tracé). **Lints/gates
  BLOQUANTS** : (1) « 0 import `@radix-ui` ne survit au runtime » ; (2) anti-`tailwind.config.js`-v3 ;
  (3) `size-limit` chiffré (budget hero Motion) — inexistant aujourd'hui ; (4) lint anti-`motion.*`-nu
  (impose `m`+`LazyMotion`) ; (5) scan front anti-PASS (« PASS »/« Vérifié »/« Approuvé » hors slot
  ÉTAT). Purge de `shadcn ^4.8.0` + 6 `@radix-ui/*` des dependencies runtime.
- **Backend deps** : déjà câblé — 0 route Rust.
- **T1** : le harness Playwright hermétique est posé (squelette) ; les 5 gates de discipline
  tournent en CI dès cette phase.

## Phase C — STEER complet (atelier + composeur dock) + rail d'orientation

- **But** : livrer la focale STEER variante B entièrement câblée + le rail ambiant altitude-0,
  intentions-pas-jargon.
- **Jobs/surfaces** : J1 (rail) + J2→J3 (intention → steering). Backend : toutes routes existantes.
- **Livrables** : composeur en dock (exception état-vide = composeur en grand) ; atelier dominant ;
  transcript SSE (J3, `EventSource` via cookie) ; provider = attribut ; MUR `requires_gate` inline ;
  rail = bandeau sprint·phase·branche·dirty/staged·pouls gates + sélecteur de MODE + secondaires
  (sessions, historique, knowledge) ; CTA en intentions, jargon `kind/provider/preflight` replié.
- **Backend deps** : déjà câblé : `/api/status:123`, `/context:127`, `/context-pack:128`,
  `/providers:129`, `/chat/session:133`, `/chat/{id}/send:135`, `/chat/{id}/stream` SSE `:136`,
  `/actions/run:130`, `/actions/log:131`, `/prompt:126`, `/sprint-history*:138-144`.
- **T1** : sous-tests (2) composeur → session créée + (3) **SSE token→Done déterministe** (un seul
  `Done`, PO-14) via cible mockée (préparée Phase I, stub possible dès ici).

## Phase D — Terminal-PTY-as-VERIFY (bootstrap) + diff de commits passés + knowledge advisory + brouillon

- **But** : livrer la surface VERIFY de **bootstrap** (terminal où l'opérateur tape `git diff`/`status`)
  + le diff de commits passés (route existante) + l'inspecteur knowledge advisory + le brouillon
  PASS-bloqué ; honorer le MUR. (Le VERIFY-plein bespoke arrive phases F→H.)
- **Jobs/surfaces** : J12 (terminal), J11 (historique/diff passés), J15 (knowledge), J13 (brouillon).
  Backend : toutes routes existantes.
- **Livrables** : terminal xterm PTY élevé en surface VERIFY ; visualiseur de diff de **commits
  passés** (route existante, 0 ajout) ; inspecteur knowledge advisory (typo mono + bordure pointillée
  + chip hash, lecture seule) ; brouillon non-autoritaire refusant PASS ; **MUR** plein-largeur,
  action unique « Préparer le pack », zéro Forcer/Override.
- **Backend deps** : déjà câblé : `/api/terminal/ws:145`, `/terminal/sessions:146`,
  `/sprint-history/diff/{sha}:144`, `authoring_knowledge:430`, refus PASS `:574/:596`,
  `requires_gate:766-779`.
- **T1** : sous-test (4) MUR `requires_gate` asserté SANS exécution (shell/commit/push/PASS →
  `requires_gate:true`, 0 spawn).

## Phase E — Design-system oklch + 5 signatures de motion sens-porteuses

- **But** : finaliser l'identité visuelle maison + les signatures de motion (motion = sens, jamais déco).
- **Jobs/surfaces** : front design-system. Backend : aucun.
- **Livrables** : tokens oklch canoniques (hues Reflect, achromatique par défaut, couleur seulement
  si un état est VRAI) ; dualité sans/mono comme langage preuve-vs-intention (`tabular-nums` sur les
  compteurs) ; **allowlist figée des 5 signatures** (token settle, gate flip, verification reveal,
  altitude shift via View-Transitions rail-exclu, confirmation gravity) ; `MotionConfig
  reducedMotion='user'` global (état final instantané sous `prefers-reduced-motion`) ; tout composant
  Base UI/shadcn re-thémé sur oklch (jamais le preset par défaut).
- **Backend deps** : aucun.
- **T1** : assertion « état final instantané sous `prefers-reduced-motion` » (anti-déco).

## Phase F — Backend : `GET /api/git/diff` (working-tree, calculé en Rust)

- **But** : exposer le **diff du working-tree** comme vérité repo unique, calculée en Rust (jamais un
  diff JS divergent). Prérequis du diff-viewer bespoke (J4, le bottleneck 2026).
- **Jobs/surfaces** : J4 (diff). `operator_server.rs` (nouvelle route GET) + logique git Rust.
- **Livrables** : `GET /api/git/diff` retournant les hunks en JSON (par-fichier, par-hunk) calculés
  en Rust ; gestion dirty/staged (au-delà des **listes** actuelles `:419-420`) ; 0 route au daemon.
- **Backend deps** : AJOUTÉ (Rust, crate `sbfb-factory`).
- **T1** : la route répond un JSON de hunks déterministe sur un workspace git fixture.

## Phase G — Backend : `GET /api/gates` (sémantique gate-live)

- **But** : **concevoir puis exposer** un état gate « vivant » consommable 1:1 par le panneau front.
  Aujourd'hui `run_gate_csp_authoring` (`gates.rs:386`) n'opère que sur un workspace de **publish**
  (CLI, `pipeline.rs:55`) — la sémantique gate-live (sur quel workspace ? quand ?) est à **figer au
  preflight de cette phase** (point le moins câblé du sprint).
- **Jobs/surfaces** : J5 (gates). `operator_server.rs` (nouvelle route GET) + réutilisation des
  gates Rust existants.
- **Livrables** : `GET /api/gates` exposant chaque `GateResult{passed,name,issues}` + état de
  fraîcheur (`run@<rev>`) ; états `non exécuté`/`informatif`/`BLOQUANT`/`N issues` **distincts**
  (jamais aplatis vert/rouge) ; l'Operator **ne clôt aucun verdict** (diagnostic 1:1, pas d'agrégat
  « PASS »).
- **Backend deps** : AJOUTÉ (Rust, crate `sbfb-factory`).
- **T1** : la route répond un état gate déterministe (au moins 1 gate `non exécuté` + 1 `passed`).

## Phase H — VERIFY-plein front : diff-viewer bespoke + panneau gates + bascule bi-focal

- **But** : livrer la focale VERIFY variante B (le vrai investissement — « décomposition, jamais
  verdict ») et câbler la bascule bi-focal STEER↔VERIFY.
- **Jobs/surfaces** : J4 + J5. Backend : routes F+G (existent désormais) + existantes.
- **Livrables** : diff-viewer React maison sur les hunks JSON Rust (variante VERIFY B : artefacts à
  gauche repliables, ÉTAT toujours visible, colonne repliée = diff plein) ; panneau gates 1:1
  (`GET /api/gates`) ; **slot ÉTAT = machine d'états énumérée nommée** (constante miroir, jamais
  « PASS ») ; actions de hunk = **intentions routées à la session** (« Transmettre la correction à
  #N »), jamais `Approve`/`Merge`/`Commit` ; provenance de fraîcheur (`◦ obsolète, relancer`) ;
  bascule STEER↔VERIFY pilotée par l'état [fin de tour ET diff/gate frais], **jamais arrachée au
  stream** (View-Transitions, rail exclu).
- **Backend deps** : déjà câblé (F+G).
- **T1** : sous-test (5) diff-viewer rend les hunks du `GET /api/git/diff` Rust + panneau gates 1:1,
  **sans jamais afficher « PASS »** dans le slot ÉTAT (scan anti-PASS BLOQUANT).

## Phase I — Testabilité T1/T2 + re-couverture SSE single-Done + comptabilité du delta

- **But** : consolider la harness hermétique et acter la comptabilité honnête du delta de couverture.
- **Jobs/surfaces** : test infra. Backend : `provider_router` (cible de test echo).
- **Livrables** : T1 consolidé (5 sous-tests ci-dessus, BLOQUANT, CI chaque push) ; **re-couverture
  du parsing SSE single-Done** (intention PO-14 portée d'`executionChat.test.ts`) ; **acter le delta**
  (perte des Vitest factory-operator + factory-ui) — interdire la chute silencieuse du total ; T2
  artefact JSON **committé** (corrige P3-6), `PASS` déterministe. Env : `SBFB_HOME` isolé + workspace
  git fixture (ferme `TEST-ISOLATION-SBFB-HOME`).
- **Backend deps** : AJOUTÉ (petit, Rust) : cible `ExecutionTarget` echo/fixture côté
  `provider_router.rs` pour un SSE déterministe (ou stub HTTP/SSE — à figer au preflight).
- **T1/T2** : T1 BLOQUANT-vert complet ; T2 JSON `PASS` committé.

## Phase J — Wrap-up : verification + mémoire + carries S81

- **But** : clore S80 — suites complètes vertes, delta tests honnête, docs/mémoire à jour, carries figés.
- **Jobs/surfaces** : verification + docs. Backend : aucun.
- **Livrables** : pipeline **fail-fast 3 blocs** (Rust dual-platform Win + Docker `sbfb-ci` + frontend
  lint/tsc/vitest/coverage/build/`size`/`scan-en-strings`) ; SPRINT_LOG row 80 + CLAUDE.md S80 DONE +
  `nexus_grid_pivot.md` + MEMORY.md ; `sprint81_audit_plan` (carries P1 in-vivo + 8 P2/11 P3 backend/docs
  + **fondation Viewer S81 re-planifiée** suite au supersede S70 + Aperçu scellé/Proof Card S81) ;
  `sprint80_verification.md` (T1 GREEN + T2 PASS machine-lisibles).
- **Backend deps** : aucun.

---

## Récap dépendances backend (toutes dans la crate `sbfb-factory`, 0 route daemon)

| Phase | Ajout backend Rust |
|---|---|
| A | fallback cookie `auth_required` + `GET /?token` bootstrap + `ServeDir` + CSP Operator |
| F | `GET /api/git/diff` (working-tree, hunks JSON calculés Rust) |
| G | `GET /api/gates` (sémantique gate-live — design au preflight) |
| I | cible `ExecutionTarget` echo/fixture (`provider_router`) pour SSE déterministe |

Tout le reste consomme les routes existantes (`operator_server.rs:122-146`).

## Scope cuts (rappel — cf. kickoff §Out)

Aperçu scellé/Proof Card (Viewer S81) · publish via Operator (reste CLI) · éditeur CM6 riche ·
palette ⌘K (accélérateur, pas cadre) · multi-session board · timeline-canvas · i18next/router complexe ·
auto-bascule arrachée au stream (interdite).
