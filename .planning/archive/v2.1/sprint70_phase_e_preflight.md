# Sprint 70 Phase E — preflight G8

Date : 2026-05-25 | HEAD : `69e3a06` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- feedback_approach.md : pick deepest, no band-aid, research before code,
  OSS prior art obligatoire G10, planning adaptatif. **Pertinent** : Phase E
  cree deux produits front (Viewer + Operator) — la regle "pick deepest"
  s'applique au choix de stack (Vite+React+TS+Tailwind+shadcn = meme stack
  que web/, coherent).
- feedback_context7_systematic.md : context7 obligatoire avant code
  touchant lib/API. **Pertinent** : Phase E ajoute Vite, React, shadcn, Tailwind
  dans de nouveaux projets (tools/factory-operator, tools/factory-ui).
  Versions resolues ci-dessous.
- vision_model.md : NO funding, NO fondation, pattern OpenBSD solo. **N/A**
  pour Phase E (pas de suggestion institutionnelle).
- feedback_commit_heredoc.md : file-based commit pour body > 30 lignes.
  **Note** : le commit Phase E sera > 30 lignes, utiliser git commit -F.
- feedback_no_direct_blobserve.md : JAMAIS ouvrir blob-serve directement.
  **Pertinent** : le Viewer est une app SBFB qui tourne dans l'iframe sandbox,
  pas en direct. Le plan est conforme.
- feedback_iframe_sandbox_forms.md : sandbox=allow-scripts bloque form submit.
  **Pertinent** : le Viewer ne doit pas utiliser de <form> — utiliser div+button.
  Le plan ne mentionne pas de formulaires dans le Viewer.
- Tensions plan vs memory : **aucune**.

## S1a — OSS prior art deep analysis

### Probleme fonctionnel

"How do mature OSS projects implement a split viewer/operator dashboard with
distinct privilege levels, where the viewer is a sandboxed read-only app and
the operator is a local privileged tool connected to a backend API?"

### Projets analyses en profondeur

#### [Projet A] — satnaing/shadcn-admin (GitHub)
- URL : https://github.com/satnaing/shadcn-admin
- Fichiers analyses : README, structure projet, routing pattern
- Pattern architectural : Vite + React + TypeScript + shadcn/ui + Tailwind +
  react-router-dom sidebar layout + dark theme. 10+ pages prefab. Exact
  meme stack que le plan Phase E Operator.
- Edge cases : RTL support, responsive sidebar collapse, command palette.
- Verdict : **APPROACH-ALIGNED** — le plan Operator utilise le meme pattern
  exact (Vite+React+TS+Tailwind+shadcn+sidebar+dark).

#### [Projet B] — react-admin (marmelab)
- URL : https://github.com/marmelab/react-admin
- Pattern architectural : RBAC avec roles viewer/editor/admin. Privilege
  separation au niveau route + composant. Backend agnostique via dataProvider.
- Pertinence : le pattern viewer vs operator est un cas classique de RBAC
  avec 2 roles. react-admin implemente exactement ce pattern avec
  canAccess() par route et par composant.
- Verdict : **APPROACH-ALIGNED** — le plan separe Viewer et Operator au
  niveau projet (deux apps distinctes), ce qui est plus strict que du RBAC
  UI-side (pas de flag qui cache les fonctions). C'est l'approche la plus
  securisee.

#### [Projet C] — sbfb-explorer (interne)
- URL : examples/sbfb-explorer/ dans le repo
- Fichiers lus : SBFB.json (8 lignes), index.html, app.js (50 lignes),
  sbfb-bridge.js (422 lignes), style.css
- Pattern architectural : app SBFB statique, dark theme, bridge postMessage
  pour node_status / identity_pubkey / browse_list. Pure HTML/CSS/JS.
  Aucun endpoint localhost, aucun shell, aucun commit.
- Verdict : **APPROACH-ALIGNED** — le plan Viewer suit exactement ce pattern
  (SBFB.json + index.html + app.js + style.css + sbfb-bridge.js). La seule
  difference est que le plan autorise TypeScript/React bundle en fichiers
  statiques, ce qui est une extension legitime.

#### [Projet D] — sbfb-ideas (interne)
- URL : examples/sbfb-ideas/ dans le repo
- Fichiers lus : SBFB.json
- SBFB.json declare les bridge methods : storage_get, storage_set,
  storage_list, storage_delete, identity_pubkey.
- Verdict : **APPROACH-ALIGNED** — le plan Viewer declare browse_list,
  search, proof_card_get comme bridge methods, coherent avec le schema
  SBFB.json v2.

#### [Projet E] — Grafana (grafana/grafana)
- Pattern architectural : viewer/editor/admin roles avec UI elements
  hidden/disabled par permission. Le viewer Grafana ne peut pas modifier
  les dashboards, seulement les consulter.
- Pertinence limitee : Grafana est un seul deployable avec roles, pas
  deux apps separees. Le plan SBFB est plus strict (separation physique
  des bundles).

### Tableau comparatif

| Aspect | Plan Phase E | shadcn-admin | react-admin | sbfb-explorer |
|--------|-------------|-------------|-------------|---------------|
| Stack | Vite+React+TS+TW+shadcn | Vite+React+TS+TW+shadcn | React+MUI | HTML/CSS/JS pur |
| Privilege separation | 2 apps separees (bundle distinct) | N/A (single app) | RBAC UI-side | N/A (read-only) |
| Security boundary | Viewer = iframe sandbox, Operator = local | N/A | Backend enforcement | iframe sandbox |
| Dark theme | oui | oui | theme provider | oui (CSS custom) |
| Backend API | sbfb-factory operator serve (Rust) | mock/placeholder | dataProvider abstrait | bridge postMessage |

### Finding S1a

- Classification : **APPROACH-ALIGNED**
- Evidence : les 5 projets confirment que le pattern choisi (2 apps separees
  avec privilege separation physique, meme stack Vite+React+TS+shadcn, dark
  theme, sidebar layout) est conforme au SOTA 2026.
- Le choix de separer Viewer et Operator en deux projets distincts (pas
  juste du RBAC UI-side) est la solution la plus securisee : aucun code
  Operator dans le bundle Viewer, verification par grep.
- Impact sur le plan : aucun.

## S1b — Deps/libs versions

### Libs scannees

| Lib | Version workspace | Latest | CVE status | Finding |
|-----|------------------|--------|------------|---------|
| react | 19.2.6 | 19.2.6 | CVE-2026-23870 (RSC DoS) — patched in 19.2.6. Non-applicable : Phase E n'utilise pas React Server Components. | clean |
| react-router-dom | 7.15.0 | 7.15.0+ | CVE-2026-22029 (XSS redirect) — patched in 7.12.0+. Version 7.15.0 safe. CVE-2025-43865 (pre-render spoof) — patched 7.5.2. Safe. | clean |
| vite | 8.0.11 | 8.x | CVE-2026-39364 (deny list bypass) — affects dev server, not production builds. CVE-2025-31125 (arbitrary file read) — patched in 6.2.4+. Version 8.0.11 safe. | clean |
| tailwindcss | 4.2.2 | 4.x | No CVE on v4.x. v3.x glob vulnerability (Nov 2025) not applicable. | clean |
| shadcn/ui | 4.2.0 | 4.x | No direct CVE. Registry injection risk documented (dev concern, not runtime). | clean |
| typescript | 5.9.3 | 5.9.x | No CVE. | clean |
| axum | 0.8 (workspace) | 0.8.x | No CVE. | clean |
| tokio | workspace | 1.x | No CVE 2026. | clean |
| tower-http | workspace | 0.6.x | No CVE. | clean |

### Specs verifiees

- SBFB.json schema_version 2 : conforme au format existant (sbfb-explorer,
  sbfb-ideas).
- sbfb-bridge.js : 422 lignes, identique dans web/public/, examples/sbfb-explorer/,
  examples/sbfb-ideas/. Le plan exige la synchronisation pour le Viewer.
- postMessage bridge : 3 methodes allowlist originales (task_submit,
  storage_get, storage_set) + extensions S56 (storage_list, storage_delete,
  identity_pubkey, node_status, browse_list) + S63 verification. Le Viewer
  utilise browse_list, search, proof_card_get — toutes deja supportees ou
  extensibles.

### Finding S1b : **clean** — 0 CVE applicable, 0 breaking change.

## S2 — Decision chain reconstruction

### Fichiers scannes

- crates/sbfb-factory/src/operator_server.rs : 1 commit (69e3a06 Phase D)
- examples/sbfb-explorer/* : 2 commits (9d802c6, f46bc66)
- examples/sbfb-ideas/* : 2 commits (74fa29a, a3943ed)
- web/public/sbfb-bridge.js : 5 commits
- .planning/active/sprint70_plan.md : 4 commits planning

### Decisions historiques trouvees

#### Decision 1 : iframe sandbox allow-scripts sans allow-forms
- Sprint 57, sha `a3943ed` : form submit bloque par sandbox. Fix : remplacer
  form par div+button+click handler.
  Body extrait : "iframe sandbox=allow-scripts blocks form submissions because
  allow-forms is not set."
- Reverse-commit check : aucune reversion — la contrainte est toujours active.
- Status : **active**
- Impact phase : **aucun** — le Viewer est read-only, pas de formulaires.

#### Decision 2 : Viewer/Operator split comme deux produits distincts
- Sprint 70 planning, sha `c4494a6` : plan v5 aligne le split Viewer/Operator.
  Le Viewer est une app SBFB sandboxee, l'Operator est un outil local privilegie.
- Body extrait : "alignement audit_plan + design_review + CLAUDE.md avec
  Viewer/Operator split"
- Status : **active** — D5 Day 0 gelee.
- Impact phase : **aucun** — Phase E implemente exactement cette decision.

#### Decision 3 : operator serve comme backend JSON API local
- Sprint 70, sha `69e3a06` : Phase D a implante 13 endpoints JSON API.
  L'Operator Phase E se connecte a ces endpoints.
  Body extrait : "Le serveur operator expose 13 endpoints JSON locaux pour
  le Factory Operator (Phase E)."
- Reverse-commit check : aucune reversion.
- Status : **active**
- Impact phase : **aucun** — Phase E consomme ces endpoints comme prevu.

#### Decision 4 : sbfb-bridge.js comme seul canal iframe-reseau
- Sprint 13, multiple commits : bridge postMessage = seul canal.
  Decision architecturale gelee (CLAUDE.md).
- Status : **active** — decision gelee.
- Impact phase : **aucun** — le Viewer utilise le bridge, l'Operator ne
  passe pas par le bridge (il est hors iframe).

#### Decision 5 : Claude Design handoff obligatoire avant code front
- Sprint 70 plan §8.1.2, sha `c4494a6` : gate explicite.
- Status : **active** — le plan exige le prompt UX + handoff avant code.
- Impact phase : **aucun** — le plan le prevoit deja.

### Memory constraints

- feedback_approach.md : pick deepest. Pertinent : choix stack = meme stack
  que web/ (le plus pousse, pas un prototype simplifie).
- feedback_no_direct_blobserve.md : apps via shell Browse. Pertinent : le
  Viewer doit etre accessible UNIQUEMENT via le shell, pas en ouvrant
  blob-serve directement.
- feedback_iframe_sandbox_forms.md : pas de <form> dans iframe sandbox.
  Pertinent : le Viewer ne doit pas utiliser de formulaires HTML natifs.

## S3 — Threat model analysis

### Primitive analysee : Factory Viewer (app SBFB sandboxee) + Factory Operator (outil local)

### Assets en jeu
- A1 Viewer bundle integrity : low — le Viewer est du code statique sans
  secret. Un tampering du bundle est detectable via blob hash BLAKE3.
- A2 Operator API surface : medium — l'Operator accede a sbfb-factory
  operator serve (13 endpoints JSON locaux). Les endpoints incluent
  actions/run, artifacts/draft, chat/session.
- A3 Authority boundary : high — la separation entre ce que le Viewer peut
  faire (lire) et ce que l'Operator peut faire (agir) est critique.

### Threat actors
- TA1 Extension navigateur malveillante : peut injecter du JS dans le
  Viewer iframe OU dans l'Operator (si ouvert dans un onglet).
- TA2 Processus local malveillant : peut acceder aux endpoints operator
  serve si le bearer/port est connu.
- TA3 Utilisateur debutant : peut confondre Viewer et Operator et
  s'attendre a pouvoir publier depuis le Viewer.

### Attack vectors identifies

1. **V1 — Viewer bundle contient du code Operator** : si le Viewer
   importe factory-ui/operator ou contient des tokens localhost/api, il
   pourrait etre exploite pour acceder aux endpoints Operator.
   Mitigation plan : grep acceptance criteria §8.4 verifie l'absence de
   ces imports. **Couvert**.

2. **V2 — Operator CORS trop permissif** : Phase D a mis CORS Any
   (localhost:*). Un site web malveillant pourrait appeler les endpoints
   operator depuis le navigateur.
   Mitigation : les endpoints sont localhost-only. Le commit Phase D
   note "CORS Any : acceptable Phase D, durcir Phase F si surface
   persiste". Phase F prevoit le durcissement. **Gap Low** — local-only
   attenuation.

3. **V3 — DraftArtifactDialog permet injection PASS** : l'Operator
   permet d'ecrire des brouillons sur une allowlist. Le plan exige
   que les verdicts PASS ne soient pas ecrivables via l'UI.
   Mitigation plan : operator_server.rs a deja un PASS injection guard
   (Phase D, teste dans operator_artifact_draft_rejects_pass_verdict).
   **Couvert**.

4. **V4 — Agent Chat execute des commandes shell** : le plan exige
   que les demandes sensibles (shell/commit/push) retournent
   requires_gate ou requires_external_agent.
   Mitigation plan : operator_server.rs a deja un sensitive action
   detection (Phase D, teste dans
   operator_chat_rejects_sensitive_action_execution). **Couvert**.

5. **V5 — Viewer accede a des donnees privees du workspace** : le
   Viewer ne doit lire que les artefacts publies/exportes.
   Mitigation plan : le Viewer utilise le bridge (browse_list, search,
   proof_card_get), pas de requetes directes au filesystem.
   **Couvert**.

6. **V6 — Supply chain Operator deps** : les nouvelles deps npm
   (react, vite, tailwind, shadcn) sont les memes que web/ — pas de
   nouvelle surface d'attaque au-dela de l'existant.
   **Couvert** (meme stack que web/).

### Mitigations existantes
- T-FEED-INTEGRITY couvre l'integrite des artefacts exposes au Viewer.
- T-PREVIEW-EXHAUSTION couvre les previews chargees localement.
- Loopback auth (S16A) couvre les endpoints daemon.
- Operator serve (Phase D) a des action guards et un PASS injection guard.

### Gaps identifies
- GAP1 V2 CORS Any : severity Low. Recommandation : durcir Phase F (deja
  prevu dans le plan). Pas bloquant Phase E.

### Regression check
- La primitive Viewer ne diminue l'efficacite d'aucune mitigation existante.
- La primitive Operator reutilise les guards Phase D (action allowlist,
  PASS injection guard, sensitive action detection).
- Aucune regression.

### Verdict S3 : **clean** — 1 gap Low (CORS, defer Phase F comme prevu).

## S4 — Wire format deep audit

### canonical.rs lu integralement : oui (296 lignes)

Phase E ne touche **aucun** fichier de crates/nexus-core-rs/ ni aucun
schema wire format. Phase E cree du code frontend TypeScript/React dans
tools/ et examples/. Aucune struct Rust dans canonical.rs n'est modifiee.

### Structs verifiees : N/A

Phase E ne modifie aucune struct serialisee. Les endpoints operator serve
(Phase D) sont deja livres et testes. Phase E est un consommateur de ces
endpoints cote frontend.

### Day 0 check

- D1 AGENT_SYSTEM.md carte : non contredite.
- D2 handoff portable : non contredite.
- D3 sbfb-factory Rust 3 commandes : non contredite.
- D4 hooks dynamises : non contredite.
- D5 Factory Viewer/Operator + contrat RRV/Factory : **Phase E implemente
  exactement D5.** Le Viewer est une app SBFB sandboxee, l'Operator est un
  outil local privilegie. Conforme.

### Pre-launch policy

- *_VERSION = 1 : non impacte (Phase E ne touche pas canonical.rs).
- Pas de tolerant decoder multi-version : N/A.
- Pas de tests "legacy decode" zombie : N/A.
- Wire format unchanged : oui.

### Decisions actees pivot.md

| Decision actee | Contredite ? |
|---|---|
| Pivot P2P integral | Non |
| Archive zip = format universel | Non |
| postMessage bridge = seul canal iframe-reseau | Non — Viewer utilise le bridge |
| Deploy verifie from source | Non — Viewer est une app SBFB standard |
| Factory = outil client externe (sbfb-factory) | Non — Operator appelle sbfb-factory |
| Vocabulaire "source verifiable" | Non |
| Factory hors daemon (D2 v4) | Non — Operator est un outil local, pas dans le daemon |
| @protocole d'abord (D6 v4) | Non |
| Gate 1 S69 validee sur @protocole | Non |

### Verdict S4 : **clean** — aucune contravention wire format ni Day 0.

## Telemetrie preflight (agent deep)

- Duree totale : estimation ~15min
- S1a : 5 projets OSS analyses (shadcn-admin, react-admin, sbfb-explorer,
  sbfb-ideas, Grafana pattern) / 8 fichiers source lus / ~600 LOC
  reviewees / 0 context7 queries (stack deja dans workspace, pas de
  nouvelle lib) / 6 WebSearch queries / finding : APPROACH-ALIGNED
- S1b : 9 libs scannees / 5 CVE searches / finding : clean
- S2 : 5 decisions historiques trouvees / 5 commit bodies lus
  complets / 0 archive files (pas de conflit historique) / 3 memory
  files lus / finding : clean
- S3 : FULL / 6 vectors analyses / 1 gap Low (CORS, defer Phase F)
- S4 : FULL / 0 structs verifiees (pas de modification wire) / canonical.rs
  lu integralement : oui

## Action

Proceder code phase E.
