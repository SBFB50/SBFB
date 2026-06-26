# Sprint 80 — Kickoff : Refonte GREENFIELD du front Factory Operator (établi bi-focal STEER/VERIFY)

> **Sprint Factory dédié — front.** On JETTE intégralement le front actuel
> (`tools/factory-operator`) ET le socle partagé `tools/factory-ui` (page blanche
> totale, re-skin in-place interdit). On reconstruit le **control-center local
> privilégié** qui parle à l'API loopback axum `127.0.0.1:3001` (token + Host +
> Origin) plus le terminal xterm PTY. Le design est conçu et durci par une R&D
> Workflow ultracode (blueprint greenfield + paradigme + brief + wireframes
> importés) puis ce kickoff (8 agents recherche D1..D8 + G1 board + 2 lentilles
> adversariales + synthèse, ancres code re-vérifiées). **Décision-grade, pas
> rubber-stamp** : les corrections PO 2026-06-26 surchargent le blueprint
> (shadcn ré-ouvert, lib motion voulue, variantes B/B), et 2 arbitrages PO
> load-bearing ont été tranchés au kickoff (cf. §Arbitrages PO).

**Écrit** : 2026-06-26.
**Type** : **sprint Factory dédié — front** (orthogonal au compute/sharding).
Le travail vit dans un nouveau front `tools/factory-operator/` (greenfield) + de
petits ajouts backend Rust **dans la crate `sbfb-factory` uniquement** (auth-transport
cookie, `ServeDir`, 2 routes VERIFY) — **0 route ajoutée au daemon** (Factory hors
daemon tenu). React 19 + Tailwind v4 + Base UI + Motion.
**Budget de phases** : Phase 0 (audit gate S79, DÉJÀ JOUÉ) + **A→J** (le nombre de
phases n'est jamais plafonné, README §4 ; dimensionné par le travail, JAMAIS par LOC).
**Numéro/version archive** : **S80**, v2.1 (OPEN) — Factory-first tenu (sharding S78
différé + tracké).

---

## Objectif produit

S80 livre la **refonte greenfield du front Factory Operator** : un **établi
bi-focal agent-natif** (pas un IDE, pas un tri-zone) où l'opérateur **exprime une
intention → observe/steer l'agent (STEER)** et **vérifie un diff → lit les gates →
preuve (VERIFY)**, sur une **scène mono-focale** pilotée par l'état, plus un **rail
d'orientation ambiant permanent** (sprint · phase · branche · dirty/staged · pouls
des gates). Le **MUR** de gouvernance (`SENSITIVE_ACTIONS`) reste un mur (jamais un
bouton « faire ») ; la connaissance est **consommée, jamais autoritaire** (0 verdict
PASS, brouillon anti-PASS, `chat_history_authoritative=false`) ; les CTA sont des
**intentions, pas du jargon**.

Stack tranchée (vérifiée juin 2026, ancres code re-confirmées) : **React 19**
(double incumbence Operator+shell `web/`), **Tailwind v4 CSS-first + tokens oklch
maison** (héritiers des hues Reflect, achromatique par défaut), **Base UI 1.x comme
SEULE dépendance runtime** de primitives (shadcn ré-admis **uniquement** comme outil
build-time, jamais dependency runtime), **Motion (`motion/react`)** comme unique lib
de motion **au-dessus** du socle natif CSS/View-Transitions/WAAPI.

**Le 1er geste est backend et BLOQUANT** : l'auth-transport **cookie HttpOnly
same-origin**. Vérifié au code — `auth.rs` lit le token **uniquement** dans le header
`x-sbfb-token` (`:42`, `auth_required:229` → 401 `:258`), or `EventSource` (SSE
`operator_server.rs:136`) et `WebSocket` (`:145`) **ne posent pas d'en-tête custom** →
sous `ServeDir` same-origin sans le proxy Vite, **SSE = 401 / WS = 403**. Sans le
cookie, le steering (J3) et le terminal (J12) sont morts en prod.

**Périmètre VERIFY = COMPLET (arbitrage PO, cf. §Arbitrages PO #1)** : S80 livre le
**bi-focal entier**, y compris le VERIFY réel — donc **2 routes backend neuves**
(`GET /api/git/diff` du working-tree calculé en Rust = vérité repo ; `GET /api/gates`
dont la sémantique gate-live est à concevoir) + le **diff-viewer bespoke** (hunks JSON
du diff Rust) + le **panneau gates** (1:1 `GateResult`). Le terminal-PTY est élevé en
surface VERIFY de **bootstrap** (l'opérateur tape `git diff`/`status`) tant que le
diff-viewer n'est pas livré, puis coexiste.

## Pourquoi maintenant

- Arc **Factory-first** tenu (sharding S78 différé + tracké). S79 a livré la capacité
  app-authoring ; le front Operator est désormais le maillon le plus visible à habiller.
- Le front actuel repose sur un **design-system de test** (preset de tokens shadcn
  GitHub-dark dans `index.css:1-57`, hex `#0d1117`/`#58a6ff`/`--sidebar-*`/`--chart-1..5`)
  et une **auth header-only** qui **casse structurellement** en prod same-origin sous
  `ServeDir` (SSE/WS = 401/403, aujourd'hui masqué par le proxy injecteur de Vite).
- Le greenfield remet le front sur une **stack vérifiée** tout en préservant le socle
  React 19 mutualisé avec le shell `web/`, et pose l'**auth cookie** qui est la
  précondition d'un parcours testable hermétique (T1 inexistant aujourd'hui pour l'Operator).

## Arbitrages PO (tranchés au kickoff — load-bearing)

1. **Cœur S80 = bi-focal COMPLET (incl. VERIFY réel)** — *Option B*. La directive
   « 0 defer du coeur » prime : VERIFY-plein n'est **pas** carry S81. S80 ajoute
   **2 routes backend** (`GET /api/git/diff` working-tree calculé Rust ; `GET /api/gates`
   sémantique gate-live à concevoir) + diff-viewer bespoke + panneau gates front
   (phases F→H). Le diff-viewer est la vraie « décomposition, jamais verdict » (le
   bottleneck 2026). Conséquence honnête : S80 est **plus gros** qu'un re-skin cosmétique
   (backend + front), et la sémantique du gate-live est un **travail de design** (sur
   quel workspace tourne un gate « vivant » ? quand ?), pas un simple câblage.
2. **Socle gelé S70 `@sbfb/factory-ui` : SUPERSEDÉ** — *Option B*. La décision gelée
   `CLAUDE.md:495-496` (« Viewer + Operator réutilisent un socle `tools/factory-ui/src/readonly` »)
   est **explicitement supersédée et tracée ici**. Le greenfield **jette aussi**
   `tools/factory-ui` (l'Operator actuel ne l'importe même pas — socle orphelin vérifié).
   La fondation partagée Viewer/Operator est **re-planifiée from scratch en S81** (quand
   le Viewer scellé sera repris). Conséquence : D2 est **libéré** de toute contrainte
   d'héritage (on écrit les composants readonly frais, sur Base UI + oklch) ; le delta
   de couverture inclut la perte des tests de `factory-ui` **et** de `factory-operator`
   (cf. §Testabilité + §Invariants « total interdit de descendre silencieusement »).

## Scope

### In (Phase 0 + A→J, cœur bi-focal complet, 0 defer du cœur)
- **Phase 0 — Audit gate S79** (DÉJÀ JOUÉ) : CONDITIONAL PASS → PASS effectif ; P1-1
  (gate CSP non-câblé sur `redeploy`) résolu hors corps (`c0a2ffe`, Option A). 2 carries
  P1 in-vivo restent ouverts (hors-portée front).
- **A — Préalable backend BLOQUANT** : fallback `Cookie` (`sbfb_operator`) dans
  `auth_required` (header `x-sbfb-token` d'abord) + handler `GET /?token=<hex>` (valide le
  token de query, pose `Set-Cookie` HttpOnly + SameSite=Strict + Path=/ + host-only, 303
  vers `/`) + `tower_http::services::ServeDir` du bundle ; + **CSP self-origin minimale de
  l'Operator** (`default-src 'self'`, défense en profondeur — « hors doctrine apps-publiées »
  ≠ « zéro en-tête »). 0 route au daemon. Revue lens-sécurité (bearer = racine de confiance).
- **B — Scaffold greenfield + gates de discipline + jettison** : React 19 + Tailwind v4
  CSS-first + tokens oklch `@theme` + Base UI + Motion (`LazyMotion` + `m`) + Geist
  sans/mono vendoré. **Jeter** `tools/factory-operator` ET `tools/factory-ui` (supersede
  S70). **Lints/gates BLOQUANTS** : (1) « 0 import `@radix-ui` ne survit au runtime » ;
  (2) anti-`tailwind.config.js`-v3 ; (3) `size-limit` chiffré (budget incluant le hero
  Motion, `LazyMotion`+`m`) — le front n'en a **aucun** aujourd'hui ; (4) lint anti-`motion.*`-nu ;
  (5) scan front anti-PASS (garde « PASS »/« Vérifié »/« Approuvé » hors du slot ÉTAT).
- **C — STEER complet + rail ambiant** : composeur en dock (variante B ; exception
  état-vide = composeur en grand), atelier dominant, transcript SSE (J3), provider =
  attribut, MUR `requires_gate` inline ; rail altitude-0 (J1) ; intentions-pas-jargon.
- **D — Terminal-PTY-as-VERIFY (bootstrap) + diff de commits passés + knowledge advisory
  + brouillon** : terminal xterm élevé en surface VERIFY de démarrage ; visualiseur de diff
  de **commits passés** (route existante) ; inspecteur knowledge advisory (typo mono +
  bordure pointillée + chip hash, lecture seule) ; brouillon non-autoritaire refusant PASS ;
  MUR plein-largeur, action unique « Préparer le pack ».
- **E — Design-system oklch + 5 signatures de motion** : tokens oklch canoniques
  (achromatique par défaut, couleur seulement si un état est VRAI), dualité sans/mono
  (preuve vs intention), allowlist figée des 5 signatures (token settle, gate flip,
  verification reveal, altitude shift via View-Transitions rail-exclu, confirmation
  gravity), `MotionConfig reducedMotion='user'`.
- **F — Backend : `GET /api/git/diff` (working-tree, calculé Rust)** : la vérité diff
  est calculée **en Rust** (vérité repo unique, jamais un diff JS divergent) ; hunks
  exposés en JSON. Dans `operator_server.rs` (crate sbfb-factory), 0 route daemon.
- **G — Backend : `GET /api/gates` (sémantique gate-live)** : **concevoir** la sémantique
  d'un gate « vivant » (sur quel workspace ? quand ?) — `run_gate_csp_authoring` n'opère
  aujourd'hui que sur un workspace de publish (CLI). Exposer un état gate consommable 1:1
  par le panneau front (chaque badge ↔ `GateResult{passed,name,issues}`, états `non
  exécuté`/`informatif`/`BLOQUANT`/`N issues` distincts, jamais aplatis).
- **H — VERIFY-plein front : diff-viewer bespoke + panneau gates + bascule bi-focal** :
  diff-viewer React maison sur les hunks JSON Rust (variante VERIFY B : artefacts à
  gauche repliables, ÉTAT toujours visible, colonne repliée = diff plein) ; panneau gates
  1:1 ; **slot ÉTAT = machine d'états énumérée nommée** (jamais « PASS ») ; actions de hunk
  = **intentions routées à la session** (« Transmettre la correction à #N »), jamais
  `Approve`/`Merge`/`Commit` ; provenance de fraîcheur (`run@<rev>`, `◦ obsolète`) ; bascule
  STEER↔VERIFY pilotée par l'état [fin de tour ET diff/gate frais], **jamais arrachée au stream**.
- **I — Testabilité T1/T2 + re-couverture SSE single-Done** : harness Playwright hermétique
  (inexistante aujourd'hui) ; re-couvrir le parsing SSE single-Done (intention PO-14 portée
  d'`executionChat.test.ts`) ; acter le delta de couverture (perte factory-operator +
  factory-ui Vitest) ; T2 artefact JSON committé.
- **J — Wrap-up** : suites complètes vertes (fail-fast 3 blocs, Rust dual-platform Win+Docker
  + frontend lint/tsc/vitest/coverage/build/size/scan-strings) ; SPRINT_LOG + CLAUDE.md +
  mémoire ; `sprint81_audit_plan` (carries P1 in-vivo + 8 P2/11 P3 + fondation Viewer S81).

### Out (différé / explicitement hors-scope)
- **Aperçu scellé + Proof Card (Viewer scellé)** — REPORTÉ S81+ (acté PO ; rouvrirait le
  P1 app-authoring in-vivo). La fondation partagée Viewer/Operator est re-planifiée S81.
- **Verbe `publish` via l'Operator** — HORS scope (publish reste CLI ; PASS = `SENSITIVE_ACTION`
  refusé via Operator `:574/:596` ; l'Operator pilote, ne scelle pas).
- **Éditeur CM6 riche** (édition de fichiers) — différé (YAGNI mono-utilisateur ; le terminal
  xterm reste le cœur de l'édition par l'agent).
- **Palette transversale ⌘K** — accélérateur seulement, jamais cadre (heurte intentions-pas-jargon).
- **Multi-session board / Mission-Control concurrent** — coupé (Factory = solo single-PTY
  séquentiel) ; récupéré en simple liste de sessions tiroir.
- **Timeline-canvas de procédé** — différé (moteur de graphe + drift canvas↔repo) ; un ruban
  read-only altitude-0 reste possible au MVP (Q-PO preflight).
- **i18next + router complexe** — différables (mono-locale FR ; routing peu profond = état
  d'altitude en store + deep-link minimal).
- **Auto-bascule STEER→VERIFY arrachée au stream** — INTERDITE (state-driven seulement).

## Day-0 — décisions gelées (NE PAS re-débattre)

1. **Greenfield TOTAL** : jeter `tools/factory-operator` ET `tools/factory-ui` (pas de
   re-skin). La décision gelée S70 `CLAUDE.md:495-496` (socle readonly partagé) est
   **supersédée et tracée** ; fondation Viewer/Operator re-planifiée S81 (Arbitrage PO #2).
2. **Framework = React 19** (SPA/SSG statique servie par `ServeDir` Rust). Solid 2.0 = Beta
   non-GA (vérifié juin 2026) → fenêtre fermée ; charnière à re-vérifier au preflight de la
   1re phase front. Rust-WASM/Svelte/Lit rejetés (fluence-agent + interop CM6/xterm).
3. **Composants = Base UI 1.x, SEULE dépendance runtime** de primitives. Lint BLOQUANT
   « 0 import `@radix-ui` ne survit au runtime ». **shadcn ré-admis (correction PO #1) mais
   UNIQUEMENT comme outil build-time** (devDep épinglée OU `npx` ad-hoc, syntaxe réelle
   `shadcn create --base=base` / `init --base=base`), **jamais dependency runtime** ; tout
   code émis est dépouillé de son preset de tokens et **re-thémé sur oklch avant commit**
   (verrou D2↔D5). daisyUI écarté côté Operator (réservé aux apps scellées produites).
4. **Motion = `motion/react`** (unique lib, correction PO #2), en escape-hatch **au-dessus**
   du socle natif (CSS/View-Transitions/WAAPI = défaut). anime.js v4 écarté côté Operator
   (impératif non-idiomatique React 19 ; sa maîtrise S79 vise les apps scellées, surface
   disjointe). **3 verrous BLOQUANTS** : `size-limit` chiffré (hero `LazyMotion`+`m` ~4,6 kb),
   lint anti-`motion.*`-nu, allowlist figée des 5 signatures sens-porteuses.
5. **Auth-transport = cookie HttpOnly same-origin** (1er geste, BLOQUANT) : fallback Cookie
   dans `auth_required` (header d'abord) + bootstrap `GET /?token` (préserve le bearer comme
   racine de confiance) + `ServeDir`. **SameSite=Strict = stoppeur CSRF primaire** ; Host =
   anti-DNS-rebinding (pas anti-CSRF) ; **Origin-vérifié-seulement-si-présent** (un SSE
   same-origin omet souvent Origin — ne PAS exiger Origin présent). Pas de `Secure` sur http
   loopback. Cookie host-only, ne distribue pas de session librement.
6. **Styling = Tailwind v4 CSS-first + tokens oklch maison** (`@theme`/`@custom-variant`,
   0 `tailwind.config.js`), 0 kit CSS. Le « design-system test » JETÉ = preset shadcn
   GitHub-dark `index.css:1-57` (PAS daisyUI). Dette corpus v4 nommée → mitigation lint.
   Dualité Geist sans/mono vendorée fontsource (0 Google Fonts).
7. **Operator HORS CSP scellée** : `BLOB_SERVE_CSP` (`csp.rs:33`) n'est injecté que par le
   daemon + le gate authoring (vérifié : absent d'`operator_server.rs`) → styling/motion/fonts
   non bridés. MAIS « hors CSP scellée » ≠ « zéro en-tête » : CSP self-origin minimale
   (`default-src 'self'`) sur l'Operator (défense en profondeur).
8. **Établi bi-focal, variantes STEER B / VERIFY B** : rail ambiant altitude-0 (ne transitionne
   jamais) + UNE scène mono-focale pilotée par l'état. STEER B (atelier dominant, composeur en
   dock). VERIFY B (artefacts repliables + slot ÉTAT + panneau gates). Bascule jamais arrachée
   au stream.
9. **Factory hors daemon** : tous les ajouts backend restent dans la crate `sbfb-factory`
   (`operator_server.rs`/`auth.rs`/routes diff+gates) ; **0 route ajoutée à `nexus-shell-daemon`**.
   `tower-http` features `["cors","fs"]` déjà au workspace (`Cargo.toml:162`) → `ServeDir` sans
   toucher au daemon.
10. **Browser = client** : SPA React servie statiquement par `ServeDir` Rust (le proxy Vite
    disparaît en prod) ; pas de Tauri/Electron/Node persistant prod. AGPL-3.0 : toutes deps
    permissives (MIT/OFL), Geist vendoré 0 CDN.
11. **Invariants backend reflétés 1:1** : MUR `SENSITIVE_ACTIONS` (`:35`) = mur jamais bouton ;
    0 verdict PASS via Operator (`:574/:596`) ; `chat_history_authoritative=false` (`:437`) ;
    diff = vérité Rust (jamais un diff JS).

## Gate de testabilité par-sprint (README §4, NON-négociable)

- **T1** : E2E Playwright **hermétique BLOQUANT** au wrap-up (+ CI à chaque push). Sous-tests
  minimaux : (1) ouverture loopback authentifiée via **COOKIE** (bootstrap `GET /?token` →
  `Set-Cookie` HttpOnly → `/` charge sans 401) ; (2) composeur → session créée
  (`POST /api/chat/session`) ; (3) **SSE token→Done DÉTERMINISTE** via cible `ExecutionTarget`
  echo/fixture mockée (provider_router) — **un seul `Done`** (invariant PO-14) ; (4) **MUR
  `requires_gate` asserté SANS exécution** (shell/commit/push/PASS → `requires_gate:true`, 0
  spawn) ; (5) une fois F→H livrés : **diff-viewer rend les hunks du `GET /api/git/diff` Rust**
  + **panneau gates 1:1** sans jamais afficher « PASS » dans le slot ÉTAT. Env : `SBFB_HOME`
  isolé + workspace git fixture.
- **T2** : acceptance **artefact JSON committé** (corrige P3-6 ; pas gitignored), `PASS`
  déterministe / `BLOCK{diagnosis}`. **`RIG-ABSENT` illégitime** (Operator 100 % loopback
  bound `127.0.0.1`). Couvre : boot cookie-authentifié, composeur→session, mur non-exécutant,
  diff-viewer/panneau gates sur données réelles, et les gates de build front verts (size-limit
  budget Motion + anti-`@radix-ui`-runtime + anti-`tailwind.config.js`-v3 + scan anti-PASS).

## Invariants

- **MUR** `SENSITIVE_ACTIONS = ['shell','commit','push','PASS']` (`operator_server.rs:35`) :
  barrière en-flux pleine largeur, action unique « Préparer le pack », **zéro** Forcer/Override,
  JAMAIS un bouton « faire » (`requires_gate:true` sans spawn, `:766-779`).
- **Connaissance consommée JAMAIS autoritaire** : 0 verdict PASS via Operator (refus écriture
  `:574/:596`), brouillon anti-PASS, `chat_history_authoritative=false` (`:437`) ; slot ÉTAT =
  machine d'états énumérée nommée qui ne dit jamais « PASS ».
- **Intentions-pas-jargon** : CTA en intentions, jargon `kind/provider/preflight` replié.
- **Single-source runtime structurel (pas disciplinaire)** : Base UI = SEULE dep de primitives ;
  lint BLOQUANT « 0 `@radix-ui` survit » ; shadcn jamais dependency runtime ; preset de tokens
  toujours dépouillé/re-thémé oklch avant commit.
- **Diff = vérité Rust** : le diff working-tree est calculé en Rust (`GET /api/git/diff`), jamais
  un diff JS divergent ; les actions de hunk sont des intentions routées à la session (ré-applique
  sous gate), jamais des mutations directes.
- **Total de tests interdit de descendre silencieusement** : jeter factory-operator + factory-ui
  retire leurs Vitest (dont `executionChat.test.ts` single-Done PO-14) → delta acté + intention
  re-couverte (Phase I).
- **Frozen tenu** : Factory hors daemon (0 route daemon) ; browser = client ; AGPL-3.0.
- **Discipline commit** : 1 commit par phase `feat(scope): Sprint 80 Phase X — titre`, body riche
  (delta tests cumulé + scope cuts) ; preflight G8 → review → Codex avant chaque commit.

## Questions ouvertes — à trancher au preflight de phase (défauts recommandés)

> Les 2 arbitrages load-bearing (cœur S80, socle factory-ui) sont TRANCHÉS ci-dessus.
> Les points suivants sont des détails de preflight ; défaut recommandé entre parenthèses.

- **[D2]** shadcn générateur (devDep/`npx`) **vs wrappers Base UI écrits-main** pour ~8
  primitives (*recommandé : wrappers main — sobriété/anti-drift lignée-OpenBSD, surtout que le
  supersede factory-ui libère tout héritage ; shadcn reste un devtool de bootstrap optionnel*).
  Base UI pin strict (churn 1.0→1.6 assumé) vs Radix hedge.
- **[D4]** Durée de vie cookie (*recommandé : session-only, sans `Max-Age`*) ; `ServeDir`
  derrière `auth_required` (*recommandé : oui*).
- **[D5]** Ajouter `@fontsource-variable/geist-mono` (*recommandé : oui — sinon dualité
  preuve/intention dégradée*) ; version Tailwind pin datée au preflight ; nom du lint jumeau de
  `scan-en-strings.sh` (*ex. `scripts/scan-front-discipline.sh`*).
- **[D6]** Auto-bascule STEER→VERIFY = **manuel par défaut** (*recommandé, pour le déterminisme
  T1*) ; ruban procédé read-only altitude-0 au MVP (*recommandé : oui, le rail est cœur*).
- **[G]** Sémantique exacte de `GET /api/gates` gate-live (sur quel workspace, quand) — **design
  à figer au preflight Phase G** (le point le moins câblé du sprint).
- **[D8]** Carry (vs absorb) des **8 P2/11 P3** de l'audit S79 vers un **sprint dette Rust/docs
  NOMMÉ** (*recommandé : carry — items backend/docs-contract hors-thème d'un sprint front ;
  acter dans `sprint81_audit_plan §3`*) ; seuls P2-8 (durcir kickoff factuel) + P3-6 (committer
  le JSON T2) routés en S80. `TEST-ISOLATION-SBFB-HOME` fermé en S80 (*recommandé : oui, T1/T2
  l'exigent*).
- **[D1]** React Compiler activé dès le scaffold (*recommandé : oui, défaut blueprint*) ; clause
  de veille v2.1 pour réévaluer un portage Solid si 2.0 passe GA.

## Carries entrants

- **P1-1** (gate CSP non-câblé sur `redeploy`) — **RÉSOLU** Phase 0 S79 (`c0a2ffe`, Option A),
  hors corps S80.
- **2 carries P1 in-vivo OUVERTS** (non adressés par S80 — un rewrite front ne les ferme pas,
  assumé) : sharding S77 RIG-ABSENT (rig 2 machines + orchestrateur de session absents) ;
  app-authoring in-vivo `Not evidenced` (parcours auteur cross-pair jamais exercé).
- **8 P2 / 11 P3** S79 (backend/docs-contract : scanner CSP évadable, Vendored-par-nom,
  source-ref substring, `PROMISE_RE`, footgun 16-hex…) → **carry vers un sprint dette Rust/docs
  NOMMÉ** (pas indéfini) ; P2-8 + P3-6 routés en S80 process.
- **Régression de couverture** (CONFIRMED_ISSUE) : jeter factory-operator + factory-ui retire
  leurs Vitest réels (dont `executionChat.test.ts` single-Done PO-14) → intention re-couverte
  Phase I, delta acté, total interdit de descendre silencieusement.
- **Fondation Viewer S81** : suite au supersede S70, le socle partagé Viewer/Operator est
  re-planifié from scratch en S81 (avec la reprise du Viewer scellé).
- **Externes inchangés** : iroh pin 0.98 (P2-AUDIT-2), rand (P2-A-1 exemption), iframe Rust-wasm
  (§P34), P3-OS-1 ; LT-2 Radicle ARMÉ (flip = décision PO hors-sprint).

## Références

- **Blueprint (BASE)** : `.planning/research/factory_front_greenfield_blueprint.md` (décision-grade).
- **Paradigme** : `.planning/research/factory_interface_paradigm_rnd.md` (control-center agent-natif).
- **Best-approach (Viewer scellé, reporté S81)** : `.planning/research/factory_front_best_approach_research.md`.
- **Brief + wireframes (variantes B/B)** : `.planning/research/factory_front_operator_wireframe_brief.md` +
  `.planning/research/wireframes_factory_operator/`.
- **Directive PO** : memory `po_directive_factory_front_redesign.md` (corrections 2026-06-26).
- **Code Factory touché** : `crates/sbfb-factory/src/auth.rs` (`:229-262` header-only → fallback cookie),
  `operator_server.rs` (`:35` MUR, `:122-146` routes, `:419-420` dirty/staged listes, `:437`/`:574`/`:596`
  invariants, `ServeDir`+`GET /`+routes diff/gates à ajouter), `gates.rs`/`pipeline.rs` (gate publish-CLI),
  `provider_router.rs` (`ExecutionTarget` StreamChunk unique, cible test echo).
- **CSP** : `crates/nexus-core-rs/src/csp.rs:33` (`BLOB_SERVE_CSP` — **ne s'applique PAS à l'Operator**).
- **À jeter** : `tools/factory-operator/` (front actuel) + `tools/factory-ui/` (socle, supersede S70).
- **Audit gate S79** : `.planning/active/sprint79_audit_findings.md` + `sprint80_audit_plan.md §3`.
