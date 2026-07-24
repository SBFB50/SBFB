# Sprint 83 — Plan : chantier « Workflow Agents » (Hub + Runtime + Packs + Protocole)

> **ACTIVABLE** (Cas C ; Phase 0 = audit gate S82 JOUÉE — verdict CONDITIONAL PASS
> `74531d4`, 3 conditions acquittées au kickoff §3/§6). **Source unique d'autorité :
> `sprint83_kickoff.md`** (D1..D5 gelés §4, outline 41 phases §5, carries routés §6,
> scope cuts 14 §7, risk register §9). Ce plan MATÉRIALISE le kickoff — il ne
> ré-ouvre aucune décision gelée. Hiérarchie documentaire : `docs/claude/README.md`
> + `docs/agent/AGENT_SYSTEM.md` + `docs/agent/PROCESS.md` > Research Pack global
> (`workflow_agents_app_conception_ultradeep_2026-07-15.md`, ch.24) > compagnon Hub
> (UX seulement). Le dossier `sprint82_workflow_engine/` est **SUPERSEDED** ; ses
> faits ne sont réutilisés que via la table §9 de `revalidation_code.md`.

> **1 commit atomique par phase** `feat|fix|docs|chore(scope): Sprint 83 Phase X — titre`.
> **Rigueur per-phase uniforme (memory feedback_process_uniform_rigor)** : deep preflight
> G8 (5 scans) → review Workflow indépendante → Codex `gpt-5.6-sol` effort=max →
> réconciliation → commit. **Body 9 sections** (`.claude/templates/commit_body_phase.txt` :
> Contexte / Fichiers / Delta tests / Verification §7.4 / Scope cuts respectés (kickoff §7) /
> G8 traceability / Pre-launch protocol / Codex verification / Carry closure) + trailer
> `Co-Authored-By` (**dérive I-1/I-2/I-3 reprise dès le 1er commit S83 — discipline
> immédiate, kickoff §6.2**). **2 blocs pré-commit** (Rust, Frontend) + **3 blocs Docker
> pré-push** (Rust, Frontend, SPDX) — canon §7.4, AM-6 (jamais « Python », projet
> Rust+Frontend pur depuis S50-51). **Étiquette docs-contrat par frontière neuve**
> (schema drift-gate + `// FRONTIER:` DANS le commit de la primitive — AM-3, §6.12).
> **Gates a11y/i18n fermés AVANT le commit de toute phase de surface** (AM-8 : la
> preuve de conformité appartient à la surface, pas à B).

> **Invariants tenus tout le sprint** : pre-launch protocol (`*_VERSION` restent à 1 ;
> op feed `prompt_agent_pack` via raw-op **0 bump `FEED_FORMAT_VERSION`** ;
> `RUN_EVENT_FORMAT_VERSION=1` + payload `Value` extensible ; `#[serde(default)]`
> runtime-tolerance ; **AM-12** : évolutions Task/Result/ArtifactRef ADDITIVES par défaut,
> toute machinerie range-decode = dérogation PO écrite) ; **0 dep runtime ajoutée** au
> daemon/shell (`@xyflow/react` = dep APP-SIDE bundlée dans l'archive Hub, jamais
> runtime, re-mesurée à l'activation de T — QPO-5) ; **iroh reste =1.0.1** ;
> `sbfb-workflow-core` PUR (0 Tokio/réseau/DB) ; **le graphe signé EST le plan — aucun
> LLM n'étend le graphe, n'accorde une capacité, ne change une destination ni ne produit
> un verdict process autoritaire** (Plan-Then-Execute/CaMeL) ; `heberger != publier,
> seeder != auteur` ; **MUR jamais bouton, 0 verdict calculé UI** ; Factory hors daemon
> (companion, v4 D2) ; JAMAIS funding/fondation/startup (dépenses ponctuelles PO
> uniquement, Q9) ; kudos non-monnaie (aucun stake/payment/deposit dans les packs) ;
> pilote fermé jusqu'aux preuves publiques (R-iroh-audit P0 INCHANGÉE).

> **Cadrage phases (AM-4 / AM-7)** : 41 labels = **PLANCHER** ; les re-coupes suffixées
> en cours de sprint sont le comportement nominal §4 (« le nombre de phases est une
> sortie »). Le plan PRÉ-COUPE **P → P + P1** (MCP borné) et **L → L + L1** (coffre/HITL)
> conformément à AM-4 ; X et B restent candidates à re-coupe suffixée. **Verrou dur** :
> aucun format d'exécution n'est gelé avant le spike durable Phase E (GO/NO-GO
> BLOQUANT) ; le profil de 18 nodes n'est gelé qu'en Phase G après les 4 goldens.

---

## Phase 0 — Audit gate S82 : JOUÉE le 2026-07-17

- **Verdict** : **CONDITIONAL PASS** `74531d4` — 0 P0, **0 P1 frais** (1 P1 STANDING
  hérité : app-authoring S79), 5 P2, 20 P3 ; 24/24 findings confirmés, 0 réfuté.
  Aucun fix commit requis.
- **3 conditions acquittées au kickoff** : (1) décision fermante app-authoring S79
  (Phase R, exit renforcée §6.1 — Q8) ; (2) routage de TOUT le Carry-Over du findings
  (kickoff §6.2, table de routage — lot « P3 divers » inclus) ; (3) chore d'archivage
  (reconciliation CLAUDE.md post-push-gate + itemisation delta E-2 + registre LT +
  SPRINT_LOG v2.2 — kickoff §6.3, exécuté au commit kickoff).
- **Baseline tests FIGÉE** (référence d'entrée §1) : Rust nextest **2108** Win / **2112**
  Docker ; Vitest web **412** ; Vitest operator **201** ; E2E operator **10** ; E2E web
  **44 / 2 skip**. Tip master `74531d4`.

---

## 1. État vérifié à l'entrée (baseline FIGÉE — colonne *Observed* remplie à l'exécution)

Tip master : **`74531d4`**. Mesures vérifiées par l'audit Track A (kickoff §1.2), toutes
EXACTES. La colonne *Observed* est remplie au fil du sprint (delta cumulé par familles,
§6.7 — jamais de LOC).

| Suite | Baseline `74531d4` | Observed (rempli à l'exécution) |
|---|---|---|
| Rust nextest Windows natif | **2108** (0 skip) | |
| Rust nextest Docker `sbfb-ci` | **2112** (+4 `#[cfg(unix)]`) | |
| Vitest web (`web/`) | **412** | |
| E2E web Playwright hermétique | **44 passed / 2 skipped** (env-gated @shard/@compute) | |
| Vitest operator (35 fichiers) | **201** | |
| E2E operator Playwright | **10 / 10** | |
| size-limit web | 6/6 (css 129.02/130 kB) | |
| Coverage web | 87.27 / 79.01 / 86.02 / 88.59 (seuils ≥85/85/78/85) | |
| fmt / clippy / doctests / release | verts Win + Docker | |
| Gates docs ×3 (`check-frontier-contracts` / `check-factory-docs` / `check-sharding`) | exit 0 | |
| `http.rs` | **1513 l / 89 routes / 21 modules `*_api.rs`** (mesure `wc -l` au tip) | |

**Cibles de sortie (ordres de grandeur, SORTIES de phases — jamais des engagements,
§6.7)** : `sbfb-workflow-core` +120..200 ; `sbfb-workflow-engine` +150..250 ;
daemon/companion/adaptateurs +80..150 ; Vitest shell +40..80 ; Vitest Hub +150..250 ;
E2E Hub +15..30 specs ; E2E web +0..5 ; operator 0..+10. **Total +550..950**.

**Nouveaux artefacts d'infrastructure du sprint** (0 collision au HEAD, revalidé
`revalidation_code.md` §5.1) : crates `crates/sbfb-workflow-core` (PUR) +
`crates/sbfb-workflow-engine` (hôte-neutre) ; module daemon
`crates/nexus-shell-daemon/src/workflow_api.rs` ; companion IPC dans
`crates/sbfb-factory/` ; app SBFB `examples/workflow-hub/` (archive React sandboxée).

---

## 2. Décisions Day-0 D1..D5 (résumé + implications code — autorité = kickoff §4)

| D | Décision gelée | Implications code (chemins revalidés au tip `74531d4`) |
|---|---|---|
| **D1** | **4 couches d'autorité** : Hub sandboxé (jamais credential/exécution privilégiée) → shell hôte de confiance (grants/HITL/secrets) → **Workflow Runtime Rust hôte = SEULE autorité** → protocole SBFB (distribution + feuilles bornées). **Placement (Q3)** : runtime générique `sbfb-workflow-engine` hébergé par le **daemon hôte** (routes loopback → inventaire D7) ; `FactoryAdapter` privilégié dans un **companion sbfb-factory** (IPC capability-scoped, hors TCB réseau, WA-19). | `http.rs` (1513 l, 21 modules) → nouveau `workflow_api.rs` (route par route versée à `inventory_policy`) ; `csp.rs:33` (`BLOB_SERVE_CSP` : `connect-src 'none'`, `sandbox allow-scripts` sans `allow-same-origin`) = base sandbox confirmée ; `crates/sbfb-factory/operator_server.rs` = précédent d'auth locale companion ; nouveaux crates `sbfb-workflow-core`/`-engine`. |
| **D2** | **Pack `SBFBPromptAgentPack.v1` maison** : JCS RFC 8785 + Ed25519 + constantes `DOMAIN_*_V1` dédiées (1 domaine/rôle §17.1) ; `subject[] = [{name, digest:{blake3}}]` à la in-toto (DSSE atteignable sans casse, différé Q4) ; 6 digests séparés ; `PACK.lock` pinne tout ; interdits durs (secret/code/grant/état live) refusés par le compilateur, étendus aux références nominatives de credentials (leçon n8n → seul `secret_ref` opaque). | `crates/nexus-core-rs/src/canonical.rs:77-332` (23 `DOMAIN_*_V1` — append) ; `serde_jcs 0.2` (Cargo.toml:65) ; `schemars 1.2` (Cargo.toml:360, draft 2020-12) ; `crates/nexus-shell-daemon/src/deploy.rs:355-376` (`finalize_deploy`, strip `provenance.json` avant hash + ré-injection — pattern étendu aux packs) ; `crates/nexus-coordinator-rs/src/provenance.rs:17-29,102-124` (`canonical_bytes` réutilisé). |
| **D3** | **Moteur durable = mémoïsation-de-steps + journal SQLite append-only maison** (2 crates), PAS de replay-total ni WASM. Renoncement EXPLICITE au replay-total déterministe (Q5). Journal `RunEvent.v1` PK `(run_id, sequence)` ; idempotence `(run_id, node_instance_id, effect_id)` ; **outbox dans la MÊME transaction** que l'avancement ; fail-closed sur divergence de replay ; `outcome_unknown` jamais convertible en succès ; **echo-target Rust déterministe À LIVRER** (B8 : jamais existé). Doctrine sécurité checkpoints (Ed25519 AVANT parse riche, requêtes paramétrées, état données-pures — réponse à la chaîne CVE LangGraph 2026). | Nouveaux `sbfb-workflow-core`/`-engine` ; `rusqlite 0.36` + `rusqlite_migration 2.2` (Cargo.toml:135-136, **0 bump pendant le chantier** ; bump 0.40 = slot maintenance S84+, QPO-3) ; `storage_api.rs:27-33,293-340` revalidé INADAPTE (KV last-value, GCRA 10/min `storage_limiter.rs:12`) — le runtime possède SON store ; `provider_router.rs:62-113` (`ExecutionTarget` 3 bras) + bras echo NEUF. |
| **D4** | **Langage `SBFBWorkflowGraph.v1` + `SBFBAgentDefinition.v1`** data-flow à ports typés, borné statiquement (profil 18 nodes CANDIDAT, gelé Phase G). Invariant Plan-Then-Execute testable. **Ordonnancement P0** : AUCUN verbe bridge Workflow privilégié tant que l'intersection `caller ∩ méthode-déclarée-par-release ∩ grant ∩ politique locale ∩ consentement de l'effet` n'est pas autoritaire dans le HOST (Phase C AVANT tout pouvoir — ordre C→D→E→F). Capacités TYPÉES par classe (17 classes), pas le gate substring. `secret_ref` opaque + HITL chrome hôte signé lié au digest exact. MCP borné (WA-10), A2A futur. | `web/src/bridge/useBridge.ts:111-176` (dispatch consulte l'intersection autoritaire host, Phase C) + `protocol.ts:20-49` (16 verbes ; verbes Workflow APRÈS C) ; `crates/sbfb-manifest/src/lib.rs:34-66` (`bridge.methods` déclaratif « NOT the runtime dispatch boundary ») + `BRIDGE_METHOD_ALLOWLIST:67+` (parité Rust↔TS) ; `operator_server.rs:37` (`SENSITIVE_ACTIONS` substring — NON étendu) ; `task.rs:152-202` (extensions ADDITIVES, metadata string-only préservé) ; `THREAT_MODEL.md:186` (ligne STALE « 3 méthodes » fermée Phase C) ; **9 fichiers `sbfb-bridge.js` trackés** (web/public + 8 copies : templates react/pyodide/static/static-reader + examples compute-tester/sbfb-explorer/sbfb-factory-viewer/sbfb-ideas). |
| **D5** | **ADOPTION du ch.24 (36 phases A..AJ) AMENDÉE — AM-1..AM-12 + outline 41 phases** (kickoff §4.5). Verdict PLAN-ADAPT. Mapping T1/T2 (specs Playwright NOMMÉES, artefact `sprint83_t2_acceptance.json` vocabulaire fermé) ; matrice de gates COMPLÈTE (47 gates, propriétaires + mécanismes + statut EXISTANT/ÉTENDU/NEUF) ; G-SCOPE-FLOOR ferme en AJ1 ; wire pre-launch AM-12 ; pré-coupes P/L au plan. | `agentctl.py:28/:32/:566` + hooks `phase-precommit-lightcheck.sh:88` / `phase-auditor-gate.sh:54` (grammaire `Phase [A-Z]+[0-9]?` prouvée — labels bi-lettres A1..AJ1 parsent ; preuve d'exercice Phase A) ; `ci.yml` (job T1 Hub nommé dès Phase R). |

---

## 3. Graphe de dépendances inter-phases

Le sprint impose l'ordre canon **C → D → E → F** (bridge autoritaire, contrat Factory,
spike durable, format signé) puis la chaîne backend **F < G < H < I < J < K**, la couche
sécurité **L/L1 < N** (secret_ref/coffre avant provider cloud), la couche protocole/hôte
**Q < R** (companion + bridge typé avant la 1re surface Hub), et les surfaces UI **B <
{R..V}** (fondation a11y avant toute surface). Aucune inversion dure (verif_plan36 §1).

```
Phase 0 (audit gate S82 — JOUÉ)
   │
   ▼
   A  cadrage + contrats + matrice gates + preuve grammaire  ─── ouvre G-SCOPE-FLOOR
   │        A1  lot carries hors-thème (CI, watcher, TEST-ISOLATION, goldens, hook G1)
   │            (indépendant — jouable tôt, borné)
   ▼
   B  fondation a11y / i18n / préférences privées  ──────────────┐ (précède R..V)
   │                                                             │
   ▼                                                             │
   C  AUTORITÉ du bridge (intersection 7 couches per-call)  ─────┤  grant v0 host-side
   │   ferme THREAT_MODEL:186 ; ouvre G-BRIDGE-GRANT             │  (AVANT L/L1, X)
   ▼                                                             │
   D  contrat Factory (FactoryProcessProfile.v1) ── ferme G-FACTORY-CONTRACT
   │                                                             │
   ▼                                                             │
   E  SPIKE DURABLE (GO/NO-GO) ── BLOQUE F ; ouvre G-AUTHORITY,G-STORE ; echo-target v0
   │                                                             │
   ▼                                                             │
   F  format pack signé (SBFBPromptAgentPack.v1) ── ouvre G-IMPORT-HOSTILE
   │                                                             │
   ▼                                                             │
   G  langage + agents + 18 nodes candidats + 4 goldens ── GEL du profil (exit machine)
   │                                                             │
   ▼                                                             │
   H  compilateur Rust autoritaire (CompiledPlan.v1) ── invariant Plan-Then-Execute
   │        │                                                    │
   │        └──────────────► T (canvas, 2e moitié) ; AG (import) │
   ▼                                                             │
   I  run store durable SQLite ── G-STORE,G-UPGRADE-REPLAY,G-SINGLE-WRITER
   │                                                             │
   ▼                                                             │
   J  orchestrateur local déterministe ── ferme G-AUTHORITY (avec E)
   │        └──────────────► V (Centre des Runs, 2e moitié)      │
   ▼                                                             │
   K  effets / retries / idempotence ── G-EFFECT,G-UNKNOWN ; ouvre G-FS-PROCESS-EFFECT
   │        └──────────────► O (adaptateurs), Q, AA              │
   ▼                                                             │
   L  permissions & atténuation ── G-CAPABILITY                  │
   │                                                             │
   ▼                                                             │
   L1 coffre secrets + HITL hôte (pré-coupe AM-4) ── G-SECRET-VAULT,G-SECRET,G-HITL
   │        └──────────────► N (secret_ref) ; X (2e moitié)      │
   ▼                                                             │
   M  artefacts / ACL / provenance ── G-ARTIFACT-ACL,G-PROVENANCE│
   │                                                             │
   ▼                                                             │
   N  modèles locaux + cloud (par secret_ref) ── ouvre G-PLACEMENT
   │                                                             │
   ▼                                                             │
   O  adaptateurs Claude Code + Codex ── G-CLAUDE-CODEX-RECEIPT (fermé AB/AI, 2e moitié)
   │                                                             │
   ▼                                                             │
   P  wire + compute SBFB ── G-WIRE-COMPAT(re-scope),G-NETWORK,G-PLACEMENT
   │        └──────────────► AF (Babel, 2e moitié)               │
   ▼                                                             │
   P1 MCP borné (pré-coupe AM-4) ── G-MCP-SSRF                   │
   │                                                             │
   ▼                                                             │
   Q  hôte de confiance + companion Factory ── G-HOST-ISOLATION,G-WORKFLOW-CSP
   │                                                             │
   ▼                                                             │
   R  ◄──────────────────────────────────────────────────────────┘
   R  accueil + bibliothèque Workflow Hub (1re surface) ── HUB-DRAFT ; T1 CI nommé
   │   scaffold créé VIA parcours Factory app-authoring (évidence lineage §6.1)
   ▼
   S  registre d'agents et brouillons ── HUB-T1
   │
   ▼ … (2e moitié : T canvas, U inspecteurs, V runs, W evals, W1 panel r1, X publication,
        Y inter-apps, Z..AC dogfood Factory, AD..AF packs, AG import, AH sécurité,
        AI/AI1/AI2 consolidation+acceptance, AJ/AJ1 cutover+clôture)
```

**Dépendances clés à expliciter (constats verif_plan36 / plan-challenge)** :
- **C fournit un modèle de grant v0 host-side** consommé par L/L1 et X (2e moitié) —
  assumé par le harness synthétique de C (pas d'attente d'une surface UI).
- **E bloque F** : aucun format d'exécution gelé avant le GO du spike durable. Le profil
  de nodes (G) et `CompiledPlan.v1` (H) ne se figent qu'après. `FactoryProcessProfile.v1`
  (D) précède E mais c'est un golden d'entrée versionné éditable pre-launch, pas un
  format d'exécution.
- **L1 < N** : le provider cloud (N) n'est autorisé que « par `secret_ref` », donc le
  coffre hôte (L1) doit exister avant.
- **K < O/Q/AA** : les receipts d'effets (K) sont la brique des adaptateurs Claude/Codex
  (O), du companion (Q) et du dogfood (AA, 2e moitié).
- **Q < R** : le bridge typé + companion + profil CSP Hub strict (Q) précèdent la 1re
  surface Hub (R). B (a11y) précède R..V.
- **Forward-closures explicites** (gate ouvert amont / fermé aval) : G-BRIDGE-GRANT (C
  ouvre → L/V/AI ferment la révocation pendant un vrai run) ; G-AUTHORITY (E ouvre → J
  ferme) ; G-FS-PROCESS-EFFECT (K ouvre → K/Q/AA vérifient) ; G-REVIEW-INDEPENDENCE (D
  contrat candidat → AB ferme → AI reteste). La matrice AM-2 (§4 du plan, 2e moitié)
  capture chaque fermeture multi-phases.

---

## Phase A — Cadrage, contrats de preuve et grammaire de phase

### A.1 Scope
Matérialiser D1..D5 en artefacts vérifiables AVANT tout code fonctionnel. **Ne ré-ouvre
aucune décision gelée.** Trois livrables mécaniques : (1) la **matrice COMPLÈTE**
`objectif → acceptance → gate → phase propriétaire → commande → artefact` pour les 47
gates §23 + les acceptances **WA-T1..T6 ET HUB-T1..T10** (AM-2, chaque gate a un mécanisme
d'enforcement — script/hook/test/artefact vs review humaine — et un statut
EXISTANT/ÉTENDU/NEUF) ; (2) la **spec T1/T2** (AM-1 : specs Playwright nommées + schéma
de l'artefact agrégé `sprint83_t2_acceptance.json`) ; (3) la **preuve d'exercice des
labels bi-lettres** (G-PHASE-GRAMMAR : le repo n'a jamais exercé un label > lettre+chiffre).
Ouvre **G-SCOPE-FLOOR**. Vérifie mécaniquement les réservations du préalable d'activation
(owner/dispo/fallback/exit par ressource — Q9) sans les consommer (AM-10 : checkpoint #1
= Phase B). Gates : **G-SCOPE-FLOOR (ouvre), G-PHASE-GRAMMAR, G1, G9**.

### A.2 Livrables
- `.planning/active/sprint83_gate_matrix.md` — matrice 47 gates + WA-T1..T6 + HUB-T1..T10
  (colonnes : gate, acceptance, phase propriétaire de fermeture, fermetures multi-phases,
  mécanisme d'enforcement, statut EXISTANT/ÉTENDU/NEUF, commande, artefact).
- `.planning/active/sprint83_t2_spec.md` — schéma versionné de `sprint83_t2_acceptance.json`
  (top-level `PASS|BLOCK{diagnosis}|RIG-ABSENT` ; paliers WA-T1..T6 en
  `ACTED{evidence}|MIXED|NOT-RUN` ; chemins RIG-ABSENT pour les paliers 2-machines) +
  liste des specs T1 nommées et de leur job CI.
- **Preuve grammaire G-PHASE-GRAMMAR** : cas de test dans `scripts/agent/test_agentctl.py`
  couvrant `Phase A1`, `Phase W1`, `Phase AA`, `Phase AI2`, `Phase AJ1` (parse via
  `PHASE_TITLE_RE`/`FALLBACK` `agentctl.py:28/:32` + `re.search` `:566`) ; assert que les
  deux hooks (`phase-precommit-lightcheck.sh:88`, `phase-auditor-gate.sh:54`) matchent la
  même grammaire (fixture de commit-message bi-lettre).
- Stubs de contrats versionnés (schémas vides mais nommés) dans `crates/sbfb-workflow-core`
  amorcé : `SBFBPromptAgentPack.v1`, `SBFBWorkflowGraph.v1`, `SBFBAgentDefinition.v1`,
  `FactoryProcessProfile.v1`, `RunEvent.v1`, `CompiledPlan.v1`, `WorkflowPublicProjection.v1`
  déclarés comme types `schemars` (frontières nommées, remplis dans les phases dédiées).
- `docs/factory/WORKFLOW_AGENTS_OVERVIEW.md` (couche GUIDE amorcée — cloture docs-contrat
  finalisée AJ1).

### A.3 Tests plan
- `test_agentctl.py::test_bi_letter_phase_labels_parse` — 5 labels bi-lettres parsent
  (NEUF, `scripts/agent/`).
- `test_agentctl.py::test_numeric_phase_label_refused` — `Phase 3` refusé (garde
  anti-régression).
- `sbfb-workflow-core` : `schema_snapshot_stub` — chaque type stub sérialise un JSON
  Schema draft 2020-12 non vide (`schemars`), fixture golden committée.
- **Pas de surface front** → pas de spec Playwright ; T1 = `N-A-no-frontend-change`.

### A.4 Critère d'acceptation
- `python scripts/agent/test_agentctl.py` (ou `pytest scripts/agent/`) exit 0, 5 nouveaux
  cas verts.
- `cargo nextest run -p sbfb-workflow-core --locked` exit 0 (stubs + schema snapshot).
- `cargo fmt --all --check` + `cargo clippy --workspace --all-targets --locked -- -D warnings`
  exit 0.
- **Condition binaire** : matrice = 47 gates + 6 WA-T + 10 HUB-T, chacun avec phase
  propriétaire + mécanisme + statut (0 orphelin) ; `sprint83_t2_spec.md` fige le
  vocabulaire fermé. `bash scripts/check-frontier-contracts.sh` exit 0.
- T2 : `N-A` (pas de comportement cross-machine ; l'agrégat est écrit AI2, §5 2e moitié).

### A.5 Commit cible
`chore(planning): Sprint 83 Phase A — cadrage, matrice de gates et grammaire de phase`
(le stub-crate + le test grammaire sortent du périmètre `.planning` → si le hook
« deletions/chore(planning) » l'exige, scinder en `feat(workflow-core): … stubs de
contrats` + `chore(planning): … matrice+spec` — décision au preflight). Body 9 sections :
Contexte (D1..D5 → contrats ; ordonnancement P0) · Fichiers (matrice, t2_spec, stubs,
test_agentctl) · Delta tests (`+2 agentctl, +1..3 workflow-core`) · Verification §7.4 · Scope
cuts respectés (kickoff §7, 14 items) · G8 traceability (preflight EXECUTE) · Pre-launch
protocol (`*_VERSION` inchangés ; nouveaux `DOMAIN_*` déclarés v1) · Codex verification ·
Carry closure (ouvre G-SCOPE-FLOOR).

---

## Phase A1 — Lot carries hors-thème bornés (condition 2 audit S82)

### A1.1 Scope
Lot VOLONTAIRE (sprint IMPAIR : pas de phase dette obligatoire, Règle 1 — mais la
condition 2 du CONDITIONAL PASS impose le routage, kickoff §6.2). Fermer les carries
CI/hygiène S82 bornés, **hors-thème Workflow Agents**, chacun avec son exit condition.
Ne touche pas au chantier. Regroupé pour ne pas polluer les phases de fond.

### A1.2 Livrables
- **CI-1 supply-chain** : allowlist licences `Unlicense`/`MIT-0` (ou exceptions par-crate)
  dans `deny.toml` + décâblage des 3 legs pip-audit zombies (packages Python purgés
  S50-51) + rectification `CLAUDE.md:439`.
- **CI-2 Node 20→22** : bump `node-version` `ci.yml:70/:176` (leg « Factory Operator
  front » — `@lingui/cli` importe `globSync` absent du Node 20 ; review+Codex arc front
  restent hors-sprint, Q13).
- **SEEDER-DIAL-TAIL** (≥3 reports — MANDATORY, Q10) : sampling `blake3(context‖pubkey)`
  sur `seeders_recent` (`seed_registry.rs:331`, pattern §P68 `placement.rs`) OU WONTFIX PO
  écrit daté.
- **macos-14 watcher** : override nextest / backend `notify` pour les 10 tests
  fs-watcher, OU pérennisation de l'arbitrage PO non-bloquant consignée.
- **TEST-ISOLATION state.json (classe ≥2 sites)** : tout Engine de test confiné
  `NEXUS_GRID_ROOT` (`result_sync.rs:494`, `engine/runtime.rs:1956/:2085`) ; state.json
  jamais une preuve live.
- **0-golden familles** feed/search/provenance/preview/proof-card/browse/nodes +
  `browse_pull` : net golden §P75 étendu (oracle ±0 EXACT cité).
- **flake dispatch_loop + sigint/running_json + occurrence Win A-1** : test-group nextest
  borné + description de classe à jour.
- **Hook lightcheck Check 5 mort (G1)** : comparaison de casse corrigée (`$PHASE_UPPER`
  ou insensible) `phase-precommit-lightcheck.sh` + test.
- **spdx Woodpecker (P3)** / **shellcheck chronique (P3)** / **nits doc M-7/C-2** : fixés
  ou désactivés justifiés ligne à ligne.

### A1.3 Tests plan
- `seed_registry::tests::dial_set_sampling_distribution` — distribution
  `blake3(context‖pubkey)` (NEUF, si fix retenu).
- `engine::runtime::tests::test_engine_confined_root` — Engine de test refuse d'écrire
  hors `NEXUS_GRID_ROOT` (classe).
- Golden : `+7..8` snapshots families (feed/search/provenance/preview/proof-card/browse/
  nodes/browse_pull), test `golden_families_stable`.
- `test_lightcheck_g1_case_insensitive.sh` — Check 5 matche `Phase A` quel que soit la
  casse.
- T1 = `N-A-no-frontend-change`.

### A1.4 Critère d'acceptation
- `bash .claude/hooks/phase-precommit-lightcheck.sh` (fixture) : Check 5 se déclenche
  (plus code mort).
- `cargo nextest run --workspace --locked` : baseline `2108 → 2108 + (goldens + sampling
  + isolation)`, 0 régression.
- **CI (post-push, vérifié au wrap-up)** : leg supply-chain VERT ou exceptions
  documentées ; leg operator Node 22 VERT ou échec résiduel documenté arc-front ;
  shellcheck exit 0 ou disable justifié.
- **Condition binaire** : chaque item du §6.2 kickoff routé A1 a son exit prouvé (fix +
  test) OU son WONTFIX/perennisation PO daté. `deny.toml` re-étiquette P2-AUDIT-2-RESIDUEL
  `external-blocked/S83`.
- T2 : `N-A`.

### A1.5 Commit cible
`fix(ci): Sprint 83 Phase A1 — lot carries hors-thème (supply-chain, Node 22, watcher, TEST-ISOLATION, goldens, hook G1, SEEDER-DIAL-TAIL)`.
Body 9 sections : Contexte (condition 2 audit S82 ; sprint impair = lot volontaire) ·
Fichiers (deny.toml, ci.yml, seed_registry.rs, engine/runtime.rs, result_sync.rs, hooks,
goldens) · Delta tests (goldens + sampling + isolation) · Verification §7.4 · Scope cuts ·
G8 traceability · Pre-launch protocol (0 wire) · Codex verification · Carry closure (ferme
CI-1/CI-2/SEEDER-DIAL-TAIL/macos-14/TEST-ISOLATION/0-golden/dispatch_loop/hook-G1 ; P3
divers re-routés `sprint84_audit_plan.md`).

---

## Phase B — Fondation accessibilité, i18n et préférences privées

### B.1 Scope
AVANT toute surface (R-ORD-1) : installer la **fondation + logistique** a11y/i18n, PAS la
preuve de conformité (**AM-8** — la preuve appartient aux surfaces R..V puis AI1). Preboot,
design tokens, mode guidé (contrat), profil de préférences résolues **sans diagnostic**
(`ResolvedAccessibilityProfile.v1`, 16 axes host→app — `needs[]` LOCAL, jamais
DOM/logs/prompts/trace/bridge/provider), harness exécutables WCAG 2.2 AA / ATAG 2.0 / COGA
(sur surfaces existantes du shell), architecture i18n/RTL (Lingui OVERRIDE PO), matrice
handicap/AT/OS **réservée** + **premier contact panel**. Réutilise le contrat a11y Operator
(`accessibility.ts`, `check-accessibility-system.sh`) **via module commun — JAMAIS importer
l'Operator** (§20.7). **EN 301 549 version FIGÉE au kickoff (WA-16, Q6)**. Ouvre
**G-A11Y-PREFS, G-I18N, 9 G-A11Y-\*** (ferment aux surfaces + AI1). **Checkpoint ressources
#1** (AM-10 : matrice AT réservée, owner/fallback/exit ; panel premier contact).

### B.2 Livrables
- `crates/sbfb-workflow-core` (ou module commun `tools/factory-ui/readonly` réutilisé) :
  `ResolvedAccessibilityProfile.v1` (schéma `schemars`, `needs[]` opaque) + politique de
  résolution host→app (empilable, appliquée avant affichage, propagée sans reload).
- `examples/workflow-hub/` amorcé : preboot a11y, design tokens oklch, contrat mode guidé
  (3 projections = même modèle de commandes — implémentées T, 2e moitié).
- Harness : `web/scripts/` + operator `check-accessibility-system.sh` étendus au Hub
  (axe-core, Playwright clavier-only, reduced-motion/forced-colors/zoom/RTL, scan
  anti-flash, gate profil sans `needs[]`) — exécutables sur surfaces existantes.
- i18n : socle Lingui + 3 gates discipline (no-literal scopé Hub / anti-verdict
  cross-locale / parité clés) + français produit + RTL.
- `.planning/active/sprint83_at_matrix.md` — matrice 5 OS/AT réservée (owner/dispo/
  fallback/exit) + trace premier contact panel (round 1 = W1, AM-9).

### B.3 Tests plan
- `web/e2e/workflow-hub-a11y.spec.ts` (NEUF, hermétique) — preboot + reduced-motion +
  forced-colors + RTL sur les surfaces existantes (shell) ; **cadre le futur T1 Hub**.
- `profile_resolution.test.ts` — profil empilable résout sans exposer `needs[]` (assert :
  0 occurrence de `needs` dans DOM/trace/réseau).
- Rust : `accessibility_profile::tests::needs_never_serialized_outward` — `needs[]`
  serialize-only local, absent des exports.
- Gates i18n : `scan-en-strings.sh` (Hub scopé) + `check-i18n-key-parity.sh` +
  `check-i18n-verdict-cross-locale.sh` exit 0.

### B.4 Critère d'acceptation
- `npm run test:e2e -- workflow-hub-a11y` exit 0 ; `npm run test:unit` (nouveaux tests
  profil verts) ; `bash web/scripts/scan-en-strings.sh` exit 0.
- `cargo nextest run -p sbfb-workflow-core --locked` exit 0.
- **Condition binaire** : harness a11y exécutable + vert sur ≥1 surface existante ; profil
  résolu 0 fuite `needs[]` ; matrice AT réservée avec owner/fallback/exit par ligne ; EN
  301 549 figée consignée. **DoD = fondation, PAS conformité** (AM-8).
- T2 : `N-A` (les paliers WA-T4 se jouent aux surfaces + AI1).

### B.5 Commit cible
`feat(a11y): Sprint 83 Phase B — fondation accessibilité, i18n et préférences privées`.
Body 9 sections : Contexte (WCAG 2.2 AA bloquant + AAA évalué + ATAG + COGA + EN 301 549
figée ; §20 pack ; AM-8 fondation) · Fichiers (profile schema, harness, i18n socle,
at_matrix) · Delta tests (a11y spec + profil + i18n gates) · Verification §7.4 · Scope
cuts · G8 traceability · Pre-launch protocol (profil v1) · Codex verification · Carry
closure (ouvre G-A11Y-PREFS/G-I18N/9 G-A11Y-\* ; checkpoint ressources #1).

---

## Phase C — Autorité du bridge existant (intersection 7 couches per-call)

### C.1 Scope
**Conséquence P0 §5.2 (revalidée `useBridge.ts:111-176` : dispatch SANS consultation
release/grant/politique/consentement)** : rendre AUTORITAIRE dans le HOST, à CHAQUE appel/
effet (offline/local inclus), l'intersection `caller ∩ méthode-déclarée-par-release ∩ grant
d'installation/activation ∩ politique locale ∩ consentement de l'effet` — **AVANT tout
verbe Workflow privilégié** (ordre C→D→E→F). Fournit le **modèle de grant v0 host-side**
(explicite au plan, consommé par L/L1/X). Ferme la ligne STALE **`THREAT_MODEL.md:186`**
(« 3 méthodes » → 16 réelles, docs-contrat de la phase). Le gate `SENSITIVE_ACTIONS`
substring n'est PAS étendu. Ouvre **G-BRIDGE-GRANT** (L/V/AI ferment la révocation pendant
un vrai run). Surface shell touchée → spec T1 nommée.

### C.2 Livrables
- Enforcement d'intersection dans le host : `web/src/bridge/useBridge.ts:111-176` consulte
  l'intersection autoritaire (release installée + grant + politique locale + consentement
  per-effet), revalidation per-call y compris offline ; méthode non-déclarée/non-accordée
  refusée malgré message iframe valide.
- `crates/sbfb-manifest/src/lib.rs:34-66` + `BRIDGE_METHOD_ALLOWLIST:67+` : `bridge.methods`
  composé avec les grants runtime (intersection **côté host**) ; parité Rust↔TS étendue
  (drift-gate `check-frontier-contracts.sh`).
- Modèle **grant v0** host-side : structure `GrantSet` (par app/release/pack) +
  résolution ; harness synthétique de test d'usurpation/expiration/révocation per-call
  (pas d'attente d'une surface UI).
- `docs/security/THREAT_MODEL.md` : §5.2 row I (ligne 186, PAS 179 = ligne vide) corrigée + vérifiée (docs-contrat de C).
- Étiquette docs-contrat frontière `bridge` étendue (`// FRONTIER:` + schema drift-gate).

### C.3 Tests plan
- `web/e2e/bridge-grant-intersection.spec.ts` (**T1 NOMMÉ, AM-1**, hermétique) — message
  iframe valide + méthode non accordée → REFUS ; grant expiré → REFUS ; révocation
  per-call → REFUS ; offline → même verdict.
- `useBridge` unit (`web/src/bridge/__tests__/`) : `intersection_authoritative.test.ts` —
  dispatch consulte les 5 couches ; mirror parité `protocol.test.ts`.
- Rust : `sbfb_manifest::tests::bridge_methods_intersect_grants` ;
  `grant_v0::tests::{usurpation_refused, expiry_refused, revocation_per_call}`.
- Parité : `check-frontier-contracts.sh` (BRIDGE_METHOD_ALLOWLIST Rust==TS).

### C.4 Critère d'acceptation
- `npm run test:e2e -- bridge-grant-intersection` exit 0 (**row fail-fast T1 GREEN**).
- `npm run test:unit` (intersection + parité verts) ; `cargo nextest run -p sbfb-manifest
  --locked` exit 0 ; `bash scripts/check-frontier-contracts.sh` exit 0.
- **Condition binaire** : aucun verbe bridge n'est dispatché sans intersection autoritaire
  (test d'usurpation ROUGE-avant/VERT-après) ; `THREAT_MODEL.md:186` = 16 méthodes.
- T2 : contribue au palier WA-T5 (refus usurpation/grant expiré/capability non déclarée) —
  `NOT-RUN` jusqu'à AI2 ; le harness synthétique fournit l'`evidence` de C.

### C.5 Commit cible
`feat(bridge): Sprint 83 Phase C — autorité du bridge (intersection 7 couches per-call)`.
Body 9 sections : Contexte (P0 §5.2 fondée au HEAD ; ordre C→D→E→F ; grant v0) · Fichiers
(useBridge.ts, protocol.ts, sbfb-manifest lib.rs, grant_v0, THREAT_MODEL.md) · Delta tests
(bridge-grant-intersection.spec + intersection unit + grant_v0 Rust) · Verification §7.4 ·
Scope cuts · G8 traceability · Pre-launch protocol (0 bump wire ; bridge additif) · Codex
verification · Carry closure (ferme THREAT_MODEL:186 ; ouvre G-BRIDGE-GRANT ; débloque
verbes Workflow).

---

## Phase D — Extraction du vrai contrat Factory (`FactoryProcessProfile.v1`)

### D.1 Scope
Extraire le **vrai** système sprint en un contrat machine-lisible `FactoryProcessProfile.v1`
lié aux **digests de ses sources** (`docs/agent/PROCESS.md`, `AGENT_SYSTEM.md`,
`RRV_FACTORY_CONTRACT.md`, `FACTORY_GATES.md`, hooks, code courant) : 9 rôles, 10 modes,
G1-G10, FG0-FG10, preflight S1a-S4, artefacts, statuts, PASS-PENDING, Codex brut,
réconciliation, 3 blocs, body 9 sections, carries, audit, wrap-up. **Sépare** gates process
/ gates FG publication / hooks de finalité ; **marque documenté vs mécaniquement enforcé**
(généralisé à la table §23 elle-même — AM-2). `gate-check` = diagnostic optionnel, JAMAIS
une autorité. Mismatch digests → **`PROFILE_STALE` bloquant**. Ferme **G-FACTORY-CONTRACT** ;
définit le contrat candidat de **G-REVIEW-INDEPENDENCE** (fermé AB, reteste AI, 2e moitié).
Précède E (golden d'entrée versionné, pas un format d'exécution).

### D.2 Livrables
- `crates/sbfb-workflow-core` : `FactoryProcessProfile.v1` (schéma `schemars` complet) +
  extraction lieuse aux digests sources (blake3 de chaque doc/hook source).
- Détecteur `PROFILE_STALE` : compare les digests embarqués aux sources courantes → bloque
  si mismatch (miroir du binding fail-closed S81 K « loaded-stage ↔ manifeste signé »).
- Marquage `documented|enforced` par règle Factory (colonne mécanisme d'enforcement).
- Fixture golden `factory_profile.golden.json` (le profil extrait, committé, revalidé en
  test).
- Étiquette docs-contrat frontière `FactoryProcessProfile.v1`.

### D.3 Tests plan
- `factory_profile::tests::extract_matches_golden` — profil extrait == golden.
- `factory_profile::tests::profile_stale_on_source_digest_mismatch` — mutation d'un digest
  source → `PROFILE_STALE` (ROUGE-avant / bloc-après).
- `factory_profile::tests::documented_vs_enforced_marked` — chaque règle porte son
  mécanisme.
- T1 = `N-A-no-frontend-change` (contrat backend).

### D.4 Critère d'acceptation
- `cargo nextest run -p sbfb-workflow-core --locked` exit 0 (extraction + PROFILE_STALE +
  marquage).
- `bash scripts/check-factory-docs.sh` exit 0 (frontière docs-contrat).
- **Condition binaire** : les 9 rôles / 10 modes / G1-G10 / FG0-FG10 présents dans le
  profil, chacun `documented|enforced` ; mismatch de digest source → BLOQUANT (test).
- T2 : contribue au palier WA-T6.1 (« extraire contrat machine-lisible ») — `NOT-RUN`
  jusqu'à AI2 ; `factory_profile.golden.json` = evidence de D.

### D.5 Commit cible
`feat(workflow-core): Sprint 83 Phase D — extraction du contrat Factory (FactoryProcessProfile.v1)`.
Body 9 sections : Contexte (WA-17 ; contrat golden avant langage ; PROFILE_STALE
fail-closed) · Fichiers (factory_profile schema + extractor + golden) · Delta tests · Verification
§7.4 · Scope cuts · G8 traceability · Pre-launch protocol (profil v1) · Codex verification ·
Carry closure (ferme G-FACTORY-CONTRACT ; ouvre contrat candidat G-REVIEW-INDEPENDENCE).

---

## Phase E — Spike durable BLOQUANT (GO/NO-GO)

### E.1 Scope
**LE cas dur du durable-execution** (R6). Prouver, sur `sbfb-workflow-engine`,
kill/restart/replay à CHAQUE frontière de crash **sans double sortie externe contre
une cible qui déduplique dans le backend journalisé, en single-writer sérialisé**
(formulation resserrée au round 2/3 de la vérification croisée : un « 0 double effet »
inconditionnel est faux — cf. `sprint83_e_durable_spike.json §guarantee`), journal/outbox
atomiques (append event + update snapshot/frontier + enqueue outbox dans la MÊME
transaction), frontier identique après restart. **AUCUN format d'exécution gelé avant ce
GO** (bloque F). Livrer le **bras echo-target Rust déterministe v0** (B8 : jamais existé —
bras d'activité echo/fixture, tests moteur hermétiques). Verdict explicite **GO/NO-GO**.
Ouvre **G-AUTHORITY** (fermé J), **G-STORE** (fermé I). Aucune dépendance UI/réseau.

### E.2 Livrables
- `crates/sbfb-workflow-engine` : noyau spike — reducer minimal + journal append-only
  SQLite (`rusqlite 0.36`) PK `(run_id, sequence)`, outbox transactionnel, clé
  d'idempotence `(run_id, node_instance_id, effect_id)`, hash de requête de step
  journalisé + comparé au replay (fail-closed sur divergence).
- **Echo-target déterministe Rust** : bras d'activité `echo` (input → output déterministe)
  branché comme premier `Activity` du trait — support des tests moteur hermétiques.
- Harness de crash : injection de crash aux 3 frontières (avant effet / entre effet et
  journal du résultat / après) ; assert 0 double effet, frontier identique.
- `.planning/active/sprint83_e_durable_spike.json` — artefact GO/NO-GO (résultat des
  scénarios de crash, décision).
- Fixtures golden events (`RunEvent` v0, culture S82 M).

### E.3 Tests plan
- `store_kill_restart_replay_no_duplicate_side_output` — kill à chaque frontière, une
  seule sortie externe contre une cible dédupliquante (contrôle rouge : cible menteuse
  ⇒ 2 sorties). Nom adapté : aucun test de E ne doit contenir `effect`, qui appartient
  au gate de la phase K.
- `engine::spike::tests::journal_outbox_atomic` — crash entre append et outbox → aucun
  effet orphelin.
- `engine::spike::tests::replay_diverges_fails_closed` — step rejoué qui ne redemande pas
  le même hash → `fail-closed`.
- `engine::echo::tests::echo_target_deterministic` — même input → même output, 0 effet
  externe.
- T1 = `N-A-no-frontend-change`.

### E.4 Critère d'acceptation
- `cargo nextest run -p sbfb-workflow-engine --locked` exit 0 (tous scénarios de crash
  verts, contrôle rouge prouvé).
- **Condition binaire (GATE GO/NO-GO)** : aux 3 frontières, une seule sortie externe
  contre une cible dédupliquante dans le backend journalisé (single-writer sérialisé)
  + fail-closed sur divergence + echo-target déterministe, **chaque garantie portant un
  bras qui la retire** → **GO** ⇒ débloque F. Sinon **NO-GO** ⇒ retour PO (jamais un
  format gelé sans preuve).
- **T2 artefact** : `sprint83_e_durable_spike.json` committé, `status=GO|NO-GO` +
  scénarios ; alimente le palier WA-T1 (reprise durable) et WA-T5 (crashs injectés) —
  paliers `NOT-RUN` jusqu'à AI2, evidence = ce fichier.

### E.5 Commit cible
`feat(workflow-engine): Sprint 83 Phase E — spike durable bloquant (journal/outbox atomiques, echo-target)`.
Body 9 sections : Contexte (règle d'or du journal Morling/DBOS ; fail-closed = miroir S81 K ;
verrou « aucun format avant le spike ») · Fichiers (engine spike, echo, crash harness,
spike.json) · Delta tests · Verification §7.4 · Scope cuts · G8 traceability · Pre-launch
protocol (`RUN_EVENT_FORMAT_VERSION=1` amorcé) · Codex verification · Carry closure (GO
débloque F ; ouvre G-AUTHORITY/G-STORE ; ferme B8 echo-target).

---

## Phase F — Format portable et signé (`SBFBPromptAgentPack.v1`)

### F.1 Scope
Geler le **format pack** après le GO de E : racine `SBFBPromptAgentPack.v1` (WA-1) —
layout §9.2, `PACK.lock`, licences, limites, canonicalisation **JCS RFC 8785** + domaines
de signature **`DOMAIN_*_V1` dédiés** (append à `canonical.rs:77-332`), **6 digests séparés**
(content / graph_semantic / presentation / compiled_plan / evaluation / publication),
`subject[]` blake3 à la in-toto (Q4), **corpus hostile** (§19.2 : tailles/profondeur/
cardinalité, `deny_unknown_fields`, pas d'interpolation de chemin, pas d'eval/script,
digests+signatures vérifiés AVANT import actif, scan secrets/endpoints). **Digests Rust/TS
concordants** (fixtures golden). Aucun code importé/secret/grant/état live. Ouvre
**G-IMPORT-HOSTILE** (fermé AG, 2e moitié).

### F.2 Livrables
- `crates/sbfb-workflow-core` : structs `SBFBPromptAgentPack.v1` + layout + `PACK.lock` +
  les 6 fonctions de digest + `subject[]` ; constantes `DOMAIN_WORKFLOW_PACK_V1`,
  `DOMAIN_COMPILED_PLAN_V1`, `DOMAIN_RUN_EVENT_V1`, `DOMAIN_PACK_INSTALL_RECEIPT_V1`, …
  (`canonical.rs`).
- Parseur borné + `deny_unknown_fields` sur le profil signé ; scanner secrets/endpoints ;
  rapport `mapped/approximated/refused`.
- Vérif signature Ed25519 AVANT parse riche (doctrine sécurité D3).
- Fixtures golden Rust **et** TS (`web/src/`) : digests byte-identiques cross-langage
  (`serde_jcs` / `jcs`).
- Corpus hostile committé (zip/path/casse/Unicode/duplicate JSON/SVG/Markdown).

### F.3 Tests plan
- `pack::tests::six_digests_stable` — chaque digest golden stable ; déplacer un node ne
  change pas `graph_semantic_digest`, changer un prompt le change.
- `pack::tests::jcs_cross_lang_parity` (Rust) + `pack.digest.test.ts` (TS) — même octets.
- `pack::tests::hostile_corpus_refused` — chaque entrée hostile rejetée AVANT rendu
  (borne/deny_unknown/no-eval).
- `pack::tests::signature_before_parse` — pack non signé → refus avant parse riche.
- T1 = `N-A-no-frontend-change` (format ; l'auteur visuel arrive R/S/T).

### F.4 Critère d'acceptation
- `cargo nextest run -p sbfb-workflow-core --locked` exit 0 ; `npm run test:unit`
  (digest parity TS verte) ; `bash scripts/check-frontier-contracts.sh` exit 0.
- **Condition binaire** : 6 digests séparés stables + parité JCS Rust/TS byte-identique +
  100 % du corpus hostile refusé avant rendu + signature vérifiée avant parse.
- T2 : palier WA-T5.1 (pack malveillant sans exécution) — evidence = corpus hostile ;
  `NOT-RUN` jusqu'à AI2.

### F.5 Commit cible
`feat(workflow-core): Sprint 83 Phase F — format portable et signé (SBFBPromptAgentPack.v1)`.
Body 9 sections : Contexte (D2 gelé après GO E ; JCS+Ed25519+DOMAIN ; subject[] in-toto ;
corpus hostile §19.2) · Fichiers (pack schema, canonical.rs DOMAIN, parseur, goldens
Rust/TS, corpus) · Delta tests · Verification §7.4 · Scope cuts · G8 traceability ·
Pre-launch protocol (pack v1 ; 0 bump feed ; `DOMAIN_*` append) · Codex verification ·
Carry closure (ouvre G-IMPORT-HOSTILE).

---

## Phase G — Langage workflow, agents et rôles (4 goldens)

### G.1 Scope
Définir `SBFBWorkflowGraph.v1` (§10.1) + `SBFBAgentDefinition.v1` (§10.2) + le **profil de
18 nodes CANDIDAT** (§10.3) + handoffs (`handoff` ≠ `agent.as_tool`, §10.4) + budgets +
contraintes d'indépendance driver/reviewer/Codex. **Exit MACHINE-VÉRIFIABLE (intègre
plan-challenge)** : les **4 goldens** (Factory PREMIER, puis Ideas, Conception, Babel)
sérialisés en JSON conformes aux JSON Schema draft de `sbfb-workflow-core` (`schemars`),
committés comme fixtures avec **test de validation VERT** ; **gel du profil de 18 nodes
consigné dans un artefact de décision daté**. H reste la preuve compilateur complète (les
goldens compilent sans pouvoir acquis par un LLM). Aucun format d'exécution figé avant les
4 goldens.

### G.2 Livrables
- `crates/sbfb-workflow-core` : `SBFBWorkflowGraph.v1`, `SBFBAgentDefinition.v1`, l'enum des
  18 nodes (ports typés), `handoff`/`agent.as_tool` distincts, `error_policy`/
  `timeout_policy`/`budget_policy`/`cancellation_policy`.
- **4 goldens** : `factory.workflow.golden.json`, `ideas.workflow.golden.json`,
  `conception.workflow.golden.json`, `babel.workflow.golden.json` — sérialisés, valides
  schemars, committés.
- `.planning/active/sprint83_node_profile_freeze.md` — artefact de décision daté gelant le
  profil de 18 nodes (après validation des 4 goldens).
- JSON Schema générés (`schema_for!`) exportés + drift-gate.

### G.3 Tests plan
- `graph::tests::four_goldens_validate` — chaque golden valide contre son JSON Schema
  (VERT = exit machine).
- `graph::tests::handoff_not_agent_as_tool` — sémantiques distinctes (payload filtré vs
  synthèse+guardrails parent).
- `graph::tests::node_profile_18_frozen` — l'enum matche l'artefact de gel.
- `schema_drift::tests::graph_schema_matches_snapshot`.
- T1 = `N-A-no-frontend-change` (langage ; le canvas est Phase T).

### G.4 Critère d'acceptation
- `cargo nextest run -p sbfb-workflow-core --locked` exit 0 : **4 goldens valident** (test
  VERT) ; `bash scripts/check-frontier-contracts.sh` exit 0.
- **Condition binaire (exit machine-vérifiable)** : 4 goldens JSON committés + valides
  schemars + artefact de gel du profil daté présent. (H prouvera la compilation complète.)
- T2 : contribue à WA-T1.1/WA-T3 (les 4 packs) — evidence = 4 goldens ; `NOT-RUN` jusqu'à
  AI2.

### G.5 Commit cible
`feat(workflow-core): Sprint 83 Phase G — langage workflow, agents et rôles (4 goldens)`.
Body 9 sections : Contexte (data-flow > control-flow MS Agent Framework ; handoff vs
agent-as-tool OpenAI SDK ; repeat borné > recursion_limit ; exit machine-vérifiable) ·
Fichiers (graph/agent schemas, 4 goldens, node_profile_freeze) · Delta tests · Verification
§7.4 · Scope cuts · G8 traceability · Pre-launch protocol (graph/agent v1) · Codex
verification · Carry closure (gel profil 18 nodes ; débloque H).

---

## Phase H — Compilateur Rust autoritaire (`CompiledPlan.v1`)

### H.1 Scope
**UNE seule autorité sémantique** : le compilateur Rust hôte (§12.1 — l'iframe peut linter,
jamais décider). Normalisation, **validation statique §10.5** (refuse node/sortie
inaccessible, cycle hors boucle bornée, `map` sans limite, join incohérent, référence non
pinnée, output non typé, capacité absente du manifeste, provider incompatible, handoff non
autorisé, secret literal, endpoint/code exécutable, budget enfant > parent, publication sans
provenance/HITL/éval requis), **fermeture des capacités**, digests sémantiques, `CompiledPlan.v1`
**immuable** (IDs d'occurrence uniques, `node_instance_id` encode le chemin complet, budgets
hérités, politique placement/retry/timeout/cancel, version+profil compilateur), anti-drift
Rust/TS. **Invariant Plan-Then-Execute TESTABLE** : « aucun LLM n'étend le graphe compilé,
n'accorde une capacité, ne change une destination ». Les 4 goldens compilent complètement,
Factory inclus.

### H.2 Livrables
- `crates/sbfb-workflow-core` : compilateur `compile(pack) -> CompiledPlan.v1` + validateur
  statique (chaque règle §10.5) + fermeture des capacités + génération des identifiants.
- `CompiledPlan.v1` (schéma) : digest pack source, digests exacts workflows/agents/prompts/
  schemas/outils, capacités fermées, budgets hérités, profil compilateur.
- Fixtures : `*.compiled.golden.json` pour les 4 packs (Factory premier).
- Tests de propriété de l'invariant Plan-Then-Execute (aucune mutation LLM ne modifie le
  plan).

### H.3 Tests plan
- `compiler::tests::four_goldens_compile` — 4 packs → `CompiledPlan` valide (Factory
  premier).
- `compiler::tests::static_validation_rejects_{unreachable_node, unbounded_map,
  unpinned_ref, secret_literal, budget_overflow, unauthorized_handoff}` — chaque règle
  §10.5 (un test par classe de refus).
- `compiler::tests::plan_then_execute_invariant` — un « LLM » simulé ne peut étendre le
  graphe / accorder capacité / changer destination.
- `schema_drift::tests::compiled_plan_matches_snapshot` (Rust/TS).
- T1 = `N-A-no-frontend-change`.

### H.4 Critère d'acceptation
- `cargo nextest run -p sbfb-workflow-core --locked` exit 0 : 4 goldens compilent + chaque
  refus §10.5 testé ; `bash scripts/check-frontier-contracts.sh` exit 0.
- **Condition binaire** : `CompiledPlan.v1` immuable produit pour les 4 packs + invariant
  Plan-Then-Execute prouvé par propriété + 0 drift Rust/TS.
- T2 : palier WA-T1.2 (canvas puis outline = même digest — fondé sur `graph_semantic_digest`)
  — evidence = compiled goldens ; `NOT-RUN` jusqu'à AI2.

### H.5 Commit cible
`feat(workflow-core): Sprint 83 Phase H — compilateur Rust autoritaire (CompiledPlan.v1)`.
Body 9 sections : Contexte (une autorité sémantique ; validation §10.5 ; invariant
Plan-Then-Execute/CaMeL) · Fichiers (compiler, validator, CompiledPlan schema, compiled
goldens) · Delta tests · Verification §7.4 · Scope cuts · G8 traceability · Pre-launch
protocol (compiled_plan v1) · Codex verification · Carry closure (débloque I ; alimente T/V/AG).

---

## Phase I — Run store durable SQLite

### I.1 Scope
Le **run store** complet du runtime (§5.4 revalidé : le KV app est INADAPTE). SQLite
(`rusqlite 0.36` + `rusqlite_migration 2.2`, **0 bump**), **event log versionné**
(`RunEvent.v1`, enveloppe versionnée UNE fois + payload `Value` extensible — politique feed
raw-op), **upcasters chaînés** à l'interprétation, **snapshots jetables** (cache invalidable,
jamais source de vérité), **inbox/outbox** transactionnels + idempotence, rétention,
backup/restore, GC, **fencing single-writer** (lease). Rejouer un ancien run après upgrade,
refuser un downgrade dangereux. Le run store = **une classe de plus dans l'inventaire de
migrations** (pattern S82 T, par classe de store). Doctrine sécurité checkpoints (requêtes
paramétrées, état données-pures, Ed25519 avant parse). Candidate à re-coupe suffixée (I1)
si mid-sprint. Gates : **G-STORE, G-UPGRADE-REPLAY, G-SINGLE-WRITER**.

### I.2 Livrables
- `crates/sbfb-workflow-engine` : run store SQLite (schéma `runs`/`run_events`/`outbox`/
  `inbox`/`snapshots`) + migrations `rusqlite_migration` + traits `store`.
- Upcasters `RunEvent` v→v (chaîne à l'interprétation) + snapshots (invalidés par
  `snapshot_version`).
- Fencing single-writer (lease) empêchant deux runtimes d'émettre la même outbox.
- backup/restore + GC + rétention (documentés).
- Fixtures golden events committées (rejouées en test — culture S82 M).
- Route(s) daemon run store dans `workflow_api.rs` versées à l'inventaire D7
  (`inventory_policy`).

### I.3 Tests plan
- `store::tests::journal_fault_injection` — crash journal/outbox → 0 perte / 0 double
  émission (G-STORE).
- `store::tests::old_run_replays_after_upgrade` — upcasters ramènent un ancien event à la
  version courante ; run rejoué (G-UPGRADE-REPLAY).
- `store::tests::single_writer_lease_fencing` — 2 runtimes → 1 seul émetteur outbox
  (G-SINGLE-WRITER).
- `store::tests::snapshot_is_disposable_cache` — snapshot invalidé, reconstruit du log.
- `store::tests::golden_events_replay_stable`.
- T1 = `N-A-no-frontend-change`.

### I.4 Critère d'acceptation
- `cargo nextest run -p sbfb-workflow-engine --locked` exit 0 (fault injection + replay
  après upgrade + fencing verts).
- Docker canonique `sbfb-ci` (`SBFB_TEST_HTTP_TIMEOUT_SECS=120`) : mêmes tests verts.
- **Condition binaire** : journal fault-injection 0 perte/0 double + replay post-upgrade +
  single-writer prouvés ; run store inscrit dans l'inventaire migrations par classe.
- T2 : palier WA-T5.3 (crashs injectés journal/outbox sans perte) — evidence = fault
  injection ; `NOT-RUN` jusqu'à AI2.

### I.5 Commit cible
`feat(workflow-engine): Sprint 83 Phase I — run store durable SQLite`.
Body 9 sections : Contexte (§5.4 KV inadapté ; enveloppe versionnée 1× + Value + upcasters ;
CVE LangGraph → paramétré+données-pures ; inventaire migrations par classe) · Fichiers
(store schema, migrations, upcasters, fencing, workflow_api routes) · Delta tests · Verification
§7.4 · Scope cuts · G8 traceability · Pre-launch protocol (`RUN_EVENT_FORMAT_VERSION=1` ;
route → inventaire D7) · Codex verification · Carry closure (ferme G-STORE partiel ; ouvre
G-UPGRADE-REPLAY/G-SINGLE-WRITER).

---

## Phase I1 — Routes daemon run store (re-coupe B6, ajoutée mid-sprint)

*Section ajoutée 2026-07-22 en exécution de la re-coupe B6 (autorisée par I.1
« Candidate à re-coupe suffixée (I1) si mid-sprint », plan:743 ; DÉCIDÉE au
preflight Phase I adaptation 22, committée au body I `e48d8280` §Scope cuts).*

### I1.1 Scope
Surface loopback du run store Phase I : module NEUF `workflow_api.rs` daemon +
inventaire D7 + famille golden_http + compte routes 89→89+N + clôture
docs-contrat par-famille (llms.txt/REFERENCE). Autorité d'exécution =
`sprint83_phase_i1_preflight.md` (delta-scan sur socle I §11.2, verdict
PLAN-ADAPT A1..A13) : coupe **READS-ONLY** — la famille mutante
restore/GC/lease-force est tier-classée T1 dans D7 avec implémentation DATÉE
J/K (T1 design-only, restore exige store CLOSED, GC sans clock avant J,
lease-force sans appelant légitime).

### I1.2 Livrables
- `crates/nexus-shell-daemon/src/workflow_api.rs` : 2 GET
  (`/api/daemon/workflow/run/{run_id}` agrégat scalaire allowlist +
  `/api/daemon/workflow/lease` row brute), `run_id` serveur-validé,
  dans `authed_routes` (auth complète), 0 RunEvent brut/snapshot/receipt.
- Câblage : dep intra-workspace engine (sans `spike-faults`), champ
  `DaemonHttpState.run_store`, boot eager miroir coordinator.db, AUCUN
  lease daemon (lecture seule).
- D7 : +1 row T0 représentative + 3 rows T1 FUTURES datées J/K ;
  THREAT_MODEL v24 (concrétisation §19) ; `docs/workflow/` layer neuf.
- Famille `golden_http_workflow_domain` + sondes `golden_families_stable`.

### I1.3 Tests plan
Projections pures 3 statuts + allowlist key-set + garde run_id + contrat
HTTP + duress-identique avec seed + golden famille. Mutations actives sur
chaque garde. T1 = `N-A-no-frontend-change` ; integration-nightly N-A.

### I1.4 Critère d'acceptation
`cargo nextest run -p nexus-shell-daemon --locked` exit 0 Win + Docker ;
routes == 91 ; engine/core BYTE-INTACTS ; census DOMAIN 29 ; frontier +
SPDX exit 0.

### I1.5 Commit cible
`feat(daemon): Sprint 83 Phase I1 — run store loopback read routes (D7 inventory, golden_http)`.

---

## Phase J — Orchestrateur local déterministe

### J.1 Scope
Le **reducer/scheduler déterministe** hôte-neutre : branches (`switch`/`classifier`),
parallélisme (`parallel`), `map`/`join`, boucles **bornées** (`repeat`), timers,
`wait.signal`, sous-workflows, **pause/reprise**, backpressure, **cancel honnête**
(§12.7 : stop d'émission + demande d'annulation aux adaptateurs qui la supportent ;
résultat tardif conservé sans propagation ; le wire Task n'a pas de cancel fort — affiché).
**Autorité du run = nœud initiateur** (WA-3). L'état rendu = réduction du journal (§12.3 :
11 états de run, 10 états de node). **Ferme G-AUTHORITY** (avec E : replay déterministe ;
tuer l'iframe ne tue pas le run). Alimente V (Centre des Runs, 2e moitié).

### J.2 Livrables
- `crates/sbfb-workflow-engine` : reducer (journal → état), scheduler local (frontier,
  ready-set), exécuteurs de nodes (branch/parallel/map/join/repeat borné/wait.signal/
  subworkflow), timers, backpressure, cancel honnête, `outcome_unknown` (préparé, complété
  K).
- Machine d'états run/node (§12.3) pilotée par le journal.
- Traits `clock`/`signal` hôte-neutres.
- Fixtures : exécution déterministe des 4 goldens compilés (echo-target de E comme
  activité).

### J.3 Tests plan
- `orchestrator::tests::deterministic_replay` — même journal → même frontier (G-AUTHORITY).
- `orchestrator::tests::iframe_death_does_not_kill_run` — le run survit à la fermeture de
  l'iframe (harness).
- `orchestrator::tests::{bounded_repeat, map_join_cardinality, parallel_branches,
  wait_signal_resume, subworkflow}` — chaque construction.
- `orchestrator::tests::honest_cancel` — cancel = stop émission + résultat tardif conservé,
  pas de faux succès.
- T1 = `N-A-no-frontend-change` (la timeline UI est Phase V).

### J.4 Critère d'acceptation
- `cargo nextest run -p sbfb-workflow-engine --locked` exit 0 (replay déterministe + toutes
  constructions + cancel honnête).
- **Condition binaire** : replay déterministe + iframe-death-safe prouvés ⇒ **G-AUTHORITY
  fermé** (avec E) ; boucles bornées structurellement (0 `map` non borné).
- T2 : palier WA-T1.3/WA-T1.5 (waiting_human, fin sans réexécution) — evidence =
  orchestrateur ; `NOT-RUN` jusqu'à AI2.

### J.5 Commit cible
`feat(workflow-engine): Sprint 83 Phase J — orchestrateur local déterministe`.
Body 9 sections : Contexte (autorité run = nœud initiateur WA-3 ; état = réduction du
journal ; cancel honnête §12.7) · Fichiers (reducer, scheduler, node executors, state
machine) · Delta tests · Verification §7.4 · Scope cuts · G8 traceability · Pre-launch
protocol (0 wire) · Codex verification · Carry closure (ferme G-AUTHORITY avec E ; alimente V).

---

## Phase K — Effets, retries et idempotence

### K.1 Scope
La couche **effets** : `effect_id` (stable entre retries), `attempt_id` (change à chaque
retry), **receipts**, timeouts, réconciliation, **`outcome_unknown`** (§12.5 : effet non
idempotent sans reçu → `outcome_unknown` → réconciliation ou décision humaine, **JAMAIS de
retry auto ni de succès inventé**). Effets FS/processus **liés à HEAD/index/worktree** +
préconditions ; **nettoyage des descendants** après cancel/crash (**Windows inclus**).
Ouvre **G-FS-PROCESS-EFFECT** (vérifié K/Q/AA). Gates : **G-EFFECT, G-UNKNOWN,
G-FS-PROCESS-EFFECT (ouvre)**. Alimente O (adaptateurs), Q (companion), AA (dogfood).

### K.2 Livrables
- `crates/sbfb-workflow-engine` : gestion d'effets (idempotence `(run_id, node_instance_id,
  effect_id)`), receipts signés (rôle §17.1), retries/backoff, timeouts, réconciliation,
  transition `outcome_unknown`.
- Effets FS/process liés HEAD/index/worktree + préconditions vérifiées ; kill de l'arbre
  de processus descendant (cross-platform, Windows inclus).
- Traits `activity` complétés (au-delà de l'echo v0 de E).

### K.3 Tests plan
- `effects::tests::double_command_single_transition` — doubles commandes/résultats sans
  double transition ni effet (G-EFFECT).
- `effects::tests::non_idempotent_without_receipt_becomes_unknown` — `outcome_unknown`, 0
  retry auto (G-UNKNOWN).
- `effects::tests::fs_effect_bound_to_head_index_worktree` — précondition mismatch → refus.
- `effects::tests::descendant_process_cleanup_windows` (`#[cfg(windows)]`) + `_unix`.
- T1 = `N-A-no-frontend-change`.

### K.4 Critère d'acceptation
- `cargo nextest run -p sbfb-workflow-engine --locked` exit 0 (Win) ; Docker `sbfb-ci`
  (`+cfg(unix)` descendant cleanup) exit 0.
- **Condition binaire** : double-effet impossible + `outcome_unknown` jamais convertible en
  succès + descendants nettoyés (Windows inclus) + effet FS lié aux préconditions.
- T2 : palier WA-T5.3/WA-T1.4 (timeout/dup/tardif/outcome_unknown ; outil hôte idempotent)
  — evidence = tests effets ; `NOT-RUN` jusqu'à AI2.

### K.5 Commit cible
`feat(workflow-engine): Sprint 83 Phase K — effets, retries et idempotence`.
Body 9 sections : Contexte (§12.5 ; dedup `(worker,task)` S76 D précédent ; outcome_unknown
jamais succès) · Fichiers (effects, receipts, process cleanup) · Delta tests · Verification
§7.4 · Scope cuts · G8 traceability · Pre-launch protocol (receipts v1) · Codex verification ·
Carry closure (ferme G-EFFECT/G-UNKNOWN ; ouvre G-FS-PROCESS-EFFECT ; alimente O/Q/AA).

---

## Phase L — Permissions et atténuation

### L.1 Scope
`effective_capabilities` = **intersection de 7 couches** (§14.1 : OS toggle ∩ politique
locale nœud ∩ grants release app ∩ grants version pack ∩ fermeture plan compilé ∩ capacité
de l'effet courant ∩ décision HITL valide) — **chaque couche ne peut qu'ATTÉNUER**. **17
classes de capacités** typées (§14.2), testées **par propriétés** (atténuation, expiration,
audience, profondeur). Consomme le grant v0 host-side de C. **Pré-coupée AM-4** : L =
permissions/atténuation ; **L1 = coffre secrets + HITL** (phase suivante). Gate :
**G-CAPABILITY**.

### L.2 Livrables
- `crates/sbfb-workflow-engine` (+ host) : moteur `effective_capabilities` (intersection 7
  couches, monotone atténuante) ; 17 classes typées ; audience/profondeur/expiration.
- Trait `policy` hôte-neutre ; câblage des grants release/pack/plan/effet distincts.
- Property tests (proptest-style) : toute couche n'atténue jamais n'élargit.

### L.3 Tests plan
- `capability::tests::intersection_only_attenuates` (property) — ajouter une couche ne peut
  qu'atténuer.
- `capability::tests::{expiry, audience, depth}_enforced` — chaque axe.
- `capability::tests::seventeen_classes_typed` — chaque classe portée typée (pas de
  substring `SENSITIVE_ACTIONS`).
- T1 = `N-A-no-frontend-change` (l'UI capacités est Phase U).

### L.4 Critère d'acceptation
- `cargo nextest run -p sbfb-workflow-engine --locked` exit 0 (property tests
  d'atténuation + expiration/audience/profondeur).
- **Condition binaire** : intersection prouvée monotone atténuante sur les 7 couches ; 17
  classes typées (0 dépendance au gate substring).
- T2 : palier WA-T5.2 (refus grant expiré/capability non déclarée) — evidence = property
  tests ; `NOT-RUN` jusqu'à AI2.

### L.5 Commit cible
`feat(workflow-engine): Sprint 83 Phase L — permissions et atténuation (7 couches, 17 classes)`.
Body 9 sections : Contexte (§14.1-14.2 ; capacités typées > gate substring ; grant v0 de C) ·
Fichiers (capability engine, 17 classes, policy trait) · Delta tests · Verification §7.4 ·
Scope cuts · G8 traceability · Pre-launch protocol (0 wire) · Codex verification · Carry
closure (ouvre G-CAPABILITY ; précède L1).

---

## Phase L1 — Coffre de secrets et HITL hôte (pré-coupe AM-4)

### L1.1 Scope
Le **coffre de secrets générique hôte** + le **HITL autoritaire chrome hôte**. Secrets =
`secret_ref` **opaque** (§14.3 : jamais dans iframe/pack/lock/layout/event-log/snapshot/
prompt/contexte/Task/trace/métrique/erreur/ArtifactRef/export/dataset) + binding par le
keystore hôte **juste avant l'appel** (rotation/suppression/zeroize/audit). **HITL** rendu
et signé par le **CHROME hôte** (§14.4), lié à `run_id + plan_digest + node_instance_id +
effect_id + opération + destination + digest des données sortantes + classe d'effet + coût
max + principal/nonce/séquence/expiration` — **toute mutation invalide l'approbation**.
Approbations **accessibles** (focus géré, pas de keyboard trap/type-to-confirm/délai
court), file utilisable **iframe fermée** (§20.10). Précède N (secret_ref). Gates :
**G-SECRET-VAULT, G-SECRET, G-HITL**.

### L1.2 Livrables
- Coffre hôte (`crates/nexus-shell-daemon` / `nexus-core-rs keystore` étendu) : backend
  Windows/macOS/Linux + CI hermétique, binding `secret_ref` → credential juste-à-temps,
  rotation/suppression/zeroize/audit ; `DOMAIN_KEYSTORE_V1` existant réutilisé.
- HITL chrome hôte : rendu d'approbation lié au digest exact, signature hôte, invalidation
  sur mutation, file d'approbation persistante (survit iframe fermée), accessible.
- Canari `secret_ref` de test (absent partout — WA-T1 étape finale).

### L1.3 Tests plan
- `secret::tests::canary_absent_everywhere` — le canari n'apparaît dans app KV / DB / logs
  / blobs / prompts / traces / exports (G-SECRET).
- `secret::tests::vault_bind_rotate_zeroize` — binding JIT, rotation, zeroize (G-SECRET-VAULT).
- `hitl::tests::mutation_invalidates_approval` — muter un paramètre invalide la décision
  (G-HITL).
- `web/e2e/workflow-hub-a11y.spec.ts` (étendu) — approbation hôte accessible (focus, pas de
  keyboard trap), file utilisable iframe fermée.
- T1 = surface chrome hôte (spec a11y étendue).

### L1.4 Critère d'acceptation
- `cargo nextest run --workspace --locked` exit 0 (canari absent + coffre + HITL) ; Docker
  `sbfb-ci` (backend keystore CI hermétique) exit 0 ; `npm run test:e2e --
  workflow-hub-a11y` exit 0.
- **Condition binaire** : canari `secret_ref` prouvé absent de TOUTES les surfaces
  d'export ; approbation HITL invalidée par toute mutation ; file HITL survit iframe morte
  et est accessible.
- T2 : palier WA-T1.6/WA-T4 (HITL chrome hôte accessible ; secret canari absent) —
  evidence = tests canari + a11y ; `NOT-RUN` jusqu'à AI2.

### L1.5 Commit cible
`feat(workflow-engine): Sprint 83 Phase L1 — coffre de secrets et HITL hôte`.
Body 9 sections : Contexte (§14.3-14.4 ; secret_ref opaque ; HITL signé lié au digest ;
approbations accessibles §20.10 ; précède N) · Fichiers (keystore vault, HITL chrome, canary)
· Delta tests · Verification §7.4 · Scope cuts · G8 traceability · Pre-launch protocol
(`DOMAIN_KEYSTORE_V1` réutilisé) · Codex verification · Carry closure (ferme G-SECRET-VAULT/
G-SECRET/G-HITL ; débloque N).

---

## Phase M — Artefacts, ACL et provenance

### M.1 Scope
`ArtifactRef` typé (content-addressed), façade blob hôte, classes **public/privé/secret/
réseau**, audience, chiffrement, rétention, suppression, **lineage**, receipts — **digest ≠
droit de lecture** (connaître le digest ne permet PAS de lire un artefact privé). Graphe de
provenance §17.2 (Source pack → PublishedPack → InstalledPack+grants → CompiledPlan → Run
inputs → Node/Activity receipts → Human decisions → Output artifacts → Eval attestations),
**8 rôles de signature non interchangeables** (§17.1 : une clé qui publie n'atteste pas un
résultat). `nexus-trace-core` **À CORRIGER avant réutilisation** (§17.3 : JCS non garanti,
racines OTel). Gates : **G-ARTIFACT-ACL, G-PROVENANCE**.

### M.2 Livrables
- `crates/sbfb-workflow-engine` : `ArtifactRef` typé + façade blob (réutilise iroh-blobs via
  le host) + classes/audience/chiffrement/rétention ; ACL (digest ≠ lecture).
- Graphe de provenance signé (8 rôles domaine-séparés) ; mutation d'un digest rejetée.
- Correction `nexus-trace-core` (JCS garanti, propagation de parent) OU wrapper conforme.
- Trait `artifact` hôte-neutre.

### M.3 Tests plan
- `artifact::tests::digest_is_not_read_right` — connaître le digest d'un artefact privé ≠
  droit de lecture (G-ARTIFACT-ACL).
- `artifact::tests::{audience, encryption, retention}_enforced`.
- `provenance::tests::mutation_of_each_digest_rejected` — chaque classe de digest muté →
  rejet (G-PROVENANCE).
- `provenance::tests::roles_not_interchangeable` — clé publisher n'atteste pas un run.
- `trace_core::tests::jcs_guaranteed` (correction §17.3).
- T1 = `N-A-no-frontend-change`.

### M.4 Critère d'acceptation
- `cargo nextest run -p sbfb-workflow-engine --locked` + `-p nexus-trace-core --locked` exit
  0 ; `bash scripts/check-frontier-contracts.sh` exit 0.
- **Condition binaire** : ACL prouvée (digest ≠ lecture) + mutation de chaque digest rejetée
  + 8 rôles non interchangeables + trace-core JCS-conforme.
- T2 : palier WA-T2.6/WA-T5.6 (ArtifactRef typé consommé par une autre app ; mutation de
  chaque classe de digest rejetée) — `NOT-RUN` jusqu'à AI2.

### M.5 Commit cible
`feat(workflow-engine): Sprint 83 Phase M — artefacts, ACL et provenance`.
Body 9 sections : Contexte (§17 ; digest ≠ droit ; 8 rôles ; trace-core à corriger §17.3) ·
Fichiers (ArtifactRef, blob façade, provenance graph, trace-core fix) · Delta tests · Verification
§7.4 · Scope cuts · G8 traceability · Pre-launch protocol (ArtifactRef v1) · Codex
verification · Carry closure (ferme G-ARTIFACT-ACL/G-PROVENANCE).

---

## Phase N — Adaptateurs modèles locaux et cloud

### N.1 Scope
`ModelAdapter` **provider-neutral** : **Ollama local réel** autonome, **≥1 provider cloud
RÉEL par `secret_ref`** pour l'acceptance (double hermétique CI **sans credential**),
structured outputs, streaming borné (coalescence des deltas LLM), coûts/quotas, rédaction
avant provider distant. Réutilise `provider_router.rs:62-113` (`ExecutionTarget` 3 bras)
enveloppé par le ModelAdapter. Le placement (`local_only` ne sort jamais) est câblé ici,
`G-PLACEMENT` ouvert (fermé avec P). Consomme le coffre L1 (secret_ref). Ouvre
**G-PLACEMENT**. **Ressource PO (Q9a)** : provider cloud réel (compte/budget/quota) —
checkpoint pré-R déjà passé ; en CI, double hermétique.

### N.2 Livrables
- `crates/sbfb-workflow-engine` : `ModelAdapter` (trait) + impl Ollama réelle + impl
  provider cloud réel (bindé par `secret_ref` via L1) ; structured outputs ; streaming
  borné ; rédaction.
- Double hermétique CI (fixture déterministe sans credential) — miroir de la culture
  fixture-daemon S80.
- Câblage placement `local_only`/`local_preferred`/`approved_provider` (§13.2).

### N.3 Tests plan
- `model_adapter::tests::ollama_real_roundtrip` (env-gated `@ollama`, skip sans runtime) +
  `model_adapter::tests::hermetic_double_no_credential` (toujours vert CI).
- `model_adapter::tests::local_only_never_leaves` — placement `local_only` ne sort jamais.
- `model_adapter::tests::structured_output_typed`.
- T1 = `N-A-no-frontend-change` (la sélection UI est Phase U/V).

### N.4 Critère d'acceptation
- `cargo nextest run -p sbfb-workflow-engine --locked` exit 0 (double hermétique vert sans
  credential) ; le test Ollama réel est `@ollama`-gated.
- **Condition binaire** : contrat identique local + double hermétique + `local_only` prouvé
  non-sortant ; provider cloud réel bindé par secret_ref (jamais en clair).
- **T2** : palier **WA-T2.4/WA-T1.4** (« même contrat Ollama réel + cloud réel + Task SBFB »)
  — chemin **RIG-ABSENT** si pas de credential/Ollama en CI ; `ACTED{evidence}` à
  l'acceptance AI2 avec provider réel. Le double hermétique fournit l'evidence CI.

### N.5 Commit cible
`feat(workflow-engine): Sprint 83 Phase N — adaptateurs modèles locaux et cloud`.
Body 9 sections : Contexte (Pilier D ; provider cloud par secret_ref ; double hermétique CI ;
local_only ne sort jamais) · Fichiers (ModelAdapter, Ollama impl, cloud impl, hermetic
double) · Delta tests · Verification §7.4 · Scope cuts · G8 traceability · Pre-launch
protocol (0 bump wire ; secret_ref via L1) · Codex verification · Carry closure (ouvre
G-PLACEMENT ; consomme L1).

---

## Phase O — Adaptateurs Claude Code et Codex

### O.1 Scope
`AgentSessionAdapter` + `ClaudeCodeAdapter` + `CodexExecAdapter` : **receipts complets**
(provider/modèle/CLI-version/prompt-input-output digests/exit code/**sortie brute**),
contextes/rôles **indépendants** (driver/reviewer/Codex par rôles+contexte+famille de
provider), fallback même-provider **visible, dégradé**, JAMAIS assimilé à une vérification
indépendante. Codex = CLI `gpt-5.6-sol` effort=max, PAS un simple appel LLM (risque P0
§25). Consomme K (receipts). Gate : **G-CLAUDE-CODEX-RECEIPT** (fermé AB/AI reteste, 2e
moitié).

### O.2 Livrables
- `crates/sbfb-factory` (companion — WA-19, hors TCB réseau) : `AgentSessionAdapter` +
  `ClaudeCodeAdapter` + `CodexExecAdapter` ; receipts signés (rôle §17.1) avec sortie
  brute.
- Contrats d'indépendance (rôles/contextes séparés) ; fallback dégradé visible et arbitré.
- Câblage `provider` de bout en bout (gate SENSITIVE_ACTIONS avant dispatch tous providers
  — précédent S71/S72 préservé, PAS étendu comme autorité de capacité).

### O.3 Tests plan
- `agent_adapter::tests::receipt_has_provider_model_cli_digests_exit_rawoutput` — receipt
  complet (G-CLAUDE-CODEX-RECEIPT).
- `agent_adapter::tests::driver_reviewer_codex_independent` — rôles/contextes/familles
  distincts.
- `agent_adapter::tests::same_provider_fallback_visible_degraded` — fallback jamais assimilé
  à une vérif indépendante.
- `codex_adapter::tests::raw_output_never_rewritten` — sortie Codex brute (feedback
  codex_raw_output).
- T1 = `N-A-no-frontend-change`.

### O.4 Critère d'acceptation
- `cargo nextest run -p sbfb-factory --locked` exit 0 ; `SBFB_TEST_HTTP_TIMEOUT_SECS=120`
  Docker exit 0.
- **Condition binaire** : receipts portent provider/modèle/CLI-version/digests/exit/sortie
  brute + indépendance driver/reviewer/Codex prouvée + fallback dégradé visible.
- T2 : palier WA-T6.4 (indépendance driver/reviewer/Codex + sortie Codex brute) — evidence
  = receipts ; consolidé AB/AI (2e moitié).

### O.5 Commit cible
`feat(factory): Sprint 83 Phase O — adaptateurs Claude Code et Codex (receipts, indépendance)`.
Body 9 sections : Contexte (Pilier D ; Codex ≠ appel LLM P0 §25 ; sortie brute ; fallback
dégradé visible) · Fichiers (AgentSessionAdapter, Claude/Codex adapters, receipts) · Delta
tests · Verification §7.4 · Scope cuts · G8 traceability · Pre-launch protocol (receipts v1) ·
Codex verification · Carry closure (ouvre G-CLAUDE-CODEX-RECEIPT ; consomme K).

---

## Phase P — Wire et compute SBFB (pré-coupée AM-4)

### P.1 Scope
Le **wire compute** : `Task`/`Result`/`ArtifactRef` **additifs 0-bump (AM-12)** avec
schéma/classification/audience/deadline en compat mixed-version (extensions ADDITIVES,
`#[serde(default)]`, versions à 1 ; machinerie range-decode = dérogation PO écrite) ;
feuilles SBFB **bornées signées seulement**. **Plancher MVP** : sans envelope chiffrée
recipient-scoped, seules les données **`public`** entrent dans une Task partagée + **preuve
de l'ensemble réel des lecteurs** du tasks-doc (§13.3). **Pré-coupée AM-4** : P = wire+compute ;
**P1 = MCP borné**. `G-PLACEMENT` fermé ici (avec N). Précède AF (Babel, 2e moitié). Gates :
**G-WIRE-COMPAT (re-scope), G-NETWORK, G-PLACEMENT**.

### P.2 Livrables
- `crates/nexus-core-rs/src/task.rs:152-202` : extensions ADDITIVES (schéma/classification/
  audience/deadline) — metadata string-only préservé, `:416-453/:541-571` inchangées ;
  `G-WIRE-COMPAT` re-scope « tolérance additive prouvée + refus propre d'un champ inconnu
  critique ».
- `crates/nexus-shell-daemon/src/workflow_api.rs` : contrat typé de feuille réseau (refs
  artefacts/schemas/deadlines/classifications) sans surcharger metadata ; preuve de
  l'ensemble réel des lecteurs.
- `NetworkComputeAdapter` (`sbfb-workflow-engine`) : Task/Result signés, IDs de corrélation
  opaques, AUCUN secret/grant/historique.
- Placement `sbfb_network_allowed`/`sbfb_network_required` (§13.2).

### P.3 Tests plan
- `wire::tests::additive_no_bump` — champ ajouté, ancien décodeur tolère ; champ inconnu
  critique → refus propre (G-WIRE-COMPAT re-scope).
- `wire::tests::public_only_shared_task` — donnée non-`public` refusée dans une Task
  partagée (G-NETWORK).
- `wire::tests::real_reader_set_proven` — l'ensemble réel des lecteurs du tasks-doc est
  prouvé.
- `placement::tests::same_contract_local_network` (G-PLACEMENT, fermé avec N).
- T1 = `N-A-no-frontend-change`.

### P.4 Critère d'acceptation
- `cargo nextest run --workspace --locked` exit 0 ; Docker `sbfb-ci` exit 0 ; count baseline
  préservé sur `task.rs` (0 bump wire — mixed-version additif).
- **Condition binaire** : Task partagée public-only + ensemble réel des lecteurs prouvé +
  tolérance additive + refus propre d'un champ inconnu critique.
- **T2** : palier **WA-T2.5** (archiver Task/claims/Result + limites de preuve) — evidence
  = wire tests ; chemin **RIG-ABSENT** si pas de 2e worker, `ACTED` à AI2/AF.

### P.5 Commit cible
`feat(daemon): Sprint 83 Phase P — wire et compute SBFB (additif 0-bump, plancher public-only)`.
Body 9 sections : Contexte (AM-12 pre-launch additif ; plancher public-only §13.3 ; envelope
chiffrée = futur sauf Q6) · Fichiers (task.rs additif, workflow_api leaf, NetworkComputeAdapter)
· Delta tests · Verification §7.4 · Scope cuts · G8 traceability · Pre-launch protocol
(**0 bump wire** ; additif ; dérogation PO si range-decode) · Codex verification · Carry
closure (ferme G-PLACEMENT avec N ; re-scope G-WIRE-COMPAT ; précède P1/AF).

---

## Phase P1 — MCP borné (pré-coupe AM-4)

### P1.1 Scope
`McpToolAdapter` **client sortant** `capability.call` permissionné : outils/schemas **pinnés
par digest blake3**, descriptions = **contenu non fiable**, protections **SSRF/DNS/redirect**
(§19.5 : destinations fixes pinnées ; loopback/link-local/metadata cloud/réseaux privés
bloqués par défaut ; pas de token passthrough ; scopes minimaux + audience liée ; output
schema+taille vérifiés). Révision MCP **pinnée 2025-11-25** ; migration stateless
**2026-07-28 = contrainte de conception tracée (veille, `mcp_server_expose` gate-off, Q14)**.
Tool poisoning MCP (5,5 % de 1899 serveurs ; MCPTox >60 %) → pinning par digest. A2A =
futur (WA-10). Gate : **G-MCP-SSRF**.

### P1.2 Livrables
- `crates/sbfb-workflow-engine` : `McpToolAdapter` (client) — pinning digest, revalidation
  DNS/IP/redirect côté adaptateur hôte, blocage destinations privées, output borné, pas de
  token passthrough.
- Snapshots d'outils MCP dans `PACK.lock` (jamais credentials) — changement dynamique de
  liste d'outils ne devient JAMAIS silencieusement une capacité du plan compilé.
- Doc de veille : contrainte migration stateless 2026-07-28.

### P1.3 Tests plan
- `mcp::tests::tools_pinned_by_digest` — un outil dont le digest change → refus.
- `mcp::tests::ssrf_private_destinations_blocked` — loopback/link-local/metadata bloqués
  (G-MCP-SSRF).
- `mcp::tests::dns_redirect_revalidated` — redirect vers destination non pinnée → refus.
- `mcp::tests::no_token_passthrough` — aucun token propagé.
- T1 = `N-A-no-frontend-change`.

### P1.4 Critère d'acceptation
- `cargo nextest run -p sbfb-workflow-engine --locked` exit 0 (pinning + SSRF + redirect +
  no-passthrough).
- **Condition binaire** : outils pinnés par digest + destinations privées bloquées + DNS/
  redirect revalidés + 0 token passthrough.
- T2 : palier WA-T5.4 (destinations privées bloquées) — evidence = tests SSRF ; `NOT-RUN`
  jusqu'à AI2.

### P1.5 Commit cible
`feat(workflow-engine): Sprint 83 Phase P1 — MCP borné (pinning digest, SSRF/DNS/redirect)`.
Body 9 sections : Contexte (WA-10 ; tool poisoning MCP ; MCP 2025-11-25 pinnée, 2026-07-28
veille ; A2A futur) · Fichiers (McpToolAdapter, pack.lock snapshots) · Delta tests · Verification
§7.4 · Scope cuts · G8 traceability · Pre-launch protocol (0 wire ; mcp_server_expose
gate-off) · Codex verification · Carry closure (ferme G-MCP-SSRF).

---

## Phase Q — Hôte de confiance et companion Factory

### Q.1 Scope
Le **bridge typé** commandes/événements/replay/quotas (§15.1 : 14 opérations `intent`, le
suffixe `intent` ne vaut JAMAIS approbation) + **push/replay/backpressure** authentifié
borné (§15.2 : curseur/séquence de reprise, catch-up par page, coalescence deltas LLM,
filtrage par app+grants, source de vérité = journal local). Le **companion Factory** local
séparé (repo/git/shell/PTY/Claude Code/Codex) — **IPC authentifié capability-scoped, hors
TCB réseau** (WA-19, le daemon ne dépend pas de la Factory, v4 D2). **Profil CSP Hub strict**
= variante de la **source CSP UNIQUE `csp.rs:33`** (surtout PAS une 2e source). `commit.intent`
/`push.intent` sans exécution autonome. Précède R. Gates : **G-HOST-ISOLATION,
G-WORKFLOW-CSP**.

### Q.2 Livrables
- `crates/nexus-shell-daemon/src/workflow_api.rs` : 14 opérations `intent` typées (bridge
  commande/événement) + SSE/channel de replay borné (curseur/reprise/catch-up) ; routes
  versées à l'inventaire D7 (`inventory_policy`) + tiers + auth complète (bearer + Host +
  Origin + peer creds UDS/NP).
- Companion Factory : service local IPC capability-scoped (`operator_server.rs` = précédent
  d'auth locale) ; crash sans fuite ; le daemon ne dépend pas de la Factory.
- Profil CSP Hub strict dérivé de `BLOB_SERVE_CSP` (`csp.rs:33`) — extension de la source
  unique, non-régression `check-frontier-contracts.sh`.

### Q.3 Tests plan
- `workflow_api::tests::intent_never_approval` — un `*.intent` ne déclenche aucun effet
  privilégié.
- `workflow_api::tests::replay_cursor_bounded` — reprise par curseur, catch-up paginé,
  coalescence.
- `companion::tests::ipc_capability_scoped_crash_no_leak` — IPC authentifié ; crash
  companion → 0 fuite (G-HOST-ISOLATION).
- `csp::tests::hub_strict_profile_single_source` — profil Hub = variante de la source unique
  (G-WORKFLOW-CSP) ; XSS incapable d'obtenir un effet bridge.
- Route inventory : `inventory_policy` couvre chaque nouvelle route (test D7).
- T1 = surface bridge/hôte (couvert par bridge-grant-intersection étendu).

### Q.4 Critère d'acceptation
- `cargo nextest run -p nexus-shell-daemon --locked` exit 0 ; `SBFB_TEST_HTTP_TIMEOUT_SECS=120`
  Docker exit 0 ; `bash scripts/check-frontier-contracts.sh` exit 0 (CSP source unique).
- **Condition binaire** : companion IPC capability-scoped hors TCB réseau + crash sans fuite
  + profil CSP Hub = source unique + chaque route dans l'inventaire D7 + `intent` ≠ effet.
- T2 : palier WA-T6.5/WA-T5.9 (commit.intent/push.intent confirmés humains ; app opérable
  états d'erreur) — `NOT-RUN` jusqu'à AI2.

### Q.5 Commit cible
`feat(daemon): Sprint 83 Phase Q — hôte de confiance et companion Factory (IPC capability-scoped, CSP Hub)`.
Body 9 sections : Contexte (§15 ; WA-19 hors TCB réseau ; CSP source unique csp.rs:33 ;
intent ≠ approbation) · Fichiers (workflow_api intent+replay, companion IPC, CSP Hub profile)
· Delta tests · Verification §7.4 · Scope cuts · G8 traceability · Pre-launch protocol
(routes → inventaire D7 ; 0 bump) · Codex verification · Carry closure (ferme
G-HOST-ISOLATION/G-WORKFLOW-CSP ; débloque R).

---

## Phase R — Accueil et bibliothèque Workflow Hub

### R.1 Scope
**Première surface Hub** (app SBFB sandboxée plein écran) : boot/offline/reconnexion,
navigation, recherche, récents, templates, versions, conflits, états vides/erreurs,
responsive. **Bibliothèque opérationnelle = accueil** (HUB-DRAFT). **Condition 1 audit S82
(NON-NÉGOCIABLE, Q8)** : le **scaffold initial de l'app Workflow Hub est créé VIA le parcours
Factory app-authoring RÉEL** (prompt-kind `app-authoring` + knowledge packs blake3 + gate
CSP déterministe `run_gate_csp_authoring` source unique `BLOB_SERVE_CSP` `csp.rs:33` +
self-check runtime). **Exit RENFORCÉE (anti-gaming, §6.1)** : l'évidence machine-lisible
inclut le **digest blake3 de l'archive générée** ET le **commit initial du Hub référence ce
digest** (lineage scaffold → app livrée) ; un run jetable sans filiation ne ferme PAS le
carry. **Job T1 CI nommé + 3 specs Hub (AM-1)**. Checkpoint ressources **pré-R**. Ferme
les exigences a11y de la surface AVANT commit (AM-8).

### R.2 Livrables
- `examples/workflow-hub/` : app React sandboxée — accueil/bibliothèque, boot/offline/
  reconnexion, recherche/récents/templates/versions/conflits/états vides, responsive ;
  `sbfb-bridge.js` (10e copie trackée — inventaire de parité étendu).
- **Parcours app-authoring** : scaffold généré via Factory `app-authoring` + évidence
  `sprint83_r_appauthoring_evidence.json` (digest blake3 de l'archive + receipts) ; le
  commit R référence ce digest.
- **Job CI T1** dans `ci.yml` : « Workflow Hub E2E » (`npm run test:e2e` filtré
  `workflow-hub-*.spec.ts`, **BLOQUANT chaque push**).
- Gates a11y de la surface fermés (axe-core, clavier-only, forced-colors, RTL).

### R.3 Tests plan
- `web/e2e/workflow-hub-author.spec.ts` (**T1 NOMMÉ, AM-1**, hermétique) — boot/offline/
  reconnexion + bibliothèque + recherche + templates + états vides ; clavier-only.
- `workflow-hub.a11y` (axe-core) sur l'accueil : 0 violation critique/sérieuse.
- Vitest Hub : `library.test.tsx`, `offline-reconnect.test.tsx`, `empty-states.test.tsx`.
- `appauthoring_lineage.test` (Rust/companion) — l'évidence lie le digest de l'archive au
  commit initial.
- **fixture daemon hermétique** (culture S80-I) pour le SSE/bridge.

### R.4 Critère d'acceptation
- `npm run test:e2e -- workflow-hub-author` exit 0 (**row fail-fast T1 GREEN — `npm run
  test:e2e`**) ; job CI « Workflow Hub E2E » vert au push.
- `npm run test:unit` + `npm run test:coverage` (seuils Hub tenus) + `npm run build` +
  `npm run size` + `bash web/scripts/scan-en-strings.sh` exit 0 ; `npx tsc --noEmit -p
  tsconfig.app.json` exit 0.
- **Condition binaire (condition 1 audit S82)** : scaffold Hub généré via parcours
  app-authoring RÉEL + évidence JSON committée avec **digest blake3** + **commit initial du
  Hub référence ce digest** (lineage prouvé). Sans ce lineage, le carry n'est PAS fermé.
- **T2** : `sprint83_r_appauthoring_evidence.json` committé (`status=ACTED{digest, lineage}`)
  — contribue au palier WA-T1 + ferme le P1 STANDING app-authoring S79.

### R.5 Commit cible
`feat(hub): Sprint 83 Phase R — accueil et bibliothèque Workflow Hub (scaffold via app-authoring)`.
Body 9 sections : Contexte (1re surface HUB-DRAFT ; condition 1 audit S82 ; app-authoring
in-vivo exit renforcée §6.1 ; T1 CI nommé) · Fichiers (examples/workflow-hub, ci.yml job,
evidence json, bridge copy) · Delta tests (E2E Hub + Vitest Hub + lineage) · Verification
§7.4 · Scope cuts · G8 traceability · Pre-launch protocol (0 wire ; 10e bridge copy à la
parité) · Codex verification · Carry closure (**ferme app-authoring S79 P1 STANDING** ;
ouvre HUB-DRAFT ; câble T1 CI).

---

## Phase S — Registre d'agents et brouillons

### S.1 Scope
Le **registre d'agents** séparé du canvas (HUB-T1) : CRUD/version/diff des agents, prompts,
policies, outils, contexte, mémoire, budgets, dépendances ; révisions, **autosave**,
undo/redo, **recovery**, conflits. Piloté par les schémas `SBFBAgentDefinition.v1` (G) — le
registre est **contract-first** (restitution, jamais autorité : validation locale NON
autoritaire, l'autorité reste le compilateur H). Surface Hub → a11y fermée avant commit
(AM-8). Ferme **HUB-T1**.

### S.2 Livrables
- `examples/workflow-hub/` : registre d'agents (CRUD/version/diff), éditeur de brouillons,
  autosave/undo/redo/recovery, gestion de conflits ; formulaires typés pilotés par
  `SBFBAgentDefinition.v1`.
- Validation locale NON autoritaire (lint côté iframe ; le verdict reste au host/compilateur).
- Gates a11y de la surface fermés (clavier-only, lecteur d'écran, forced-colors, RTL).

### S.3 Tests plan
- `web/e2e/workflow-hub-author.spec.ts` (étendu) — CRUD agent + version + diff sémantique +
  autosave/undo/recovery ; clavier-only + lecteur d'écran.
- Vitest Hub : `agent-registry.test.tsx`, `draft-autosave-recovery.test.tsx`,
  `agent-diff.test.tsx`, `conflict-resolution.test.tsx`.
- `agents.a11y` (axe-core) : 0 violation critique/sérieuse.
- `agent_definition_schema.test.ts` — formulaires conformes au schéma G (contract-first).

### S.4 Critère d'acceptation
- `npm run test:e2e -- workflow-hub-author` exit 0 (**T1 GREEN**) ; `npm run test:unit` +
  `npm run test:coverage` (seuils tenus) + `npm run build` + `bash web/scripts/scan-en-strings.sh`
  exit 0 ; `npx tsc --noEmit` exit 0.
- **Condition binaire** : CRUD/version/diff/autosave/undo/recovery opérationnels + registre
  piloté par le schéma (0 verdict calculé UI — validation locale non autoritaire) + a11y de
  la surface fermée.
- T2 : palier HUB-T1 (registre) — evidence = E2E Hub ; agrégé AI2.

### S.5 Commit cible
`feat(hub): Sprint 83 Phase S — registre d'agents et brouillons`.
Body 9 sections : Contexte (HUB-T1 ; registre séparé du canvas ; contract-first ; validation
non autoritaire) · Fichiers (workflow-hub agent registry, drafts, diff, conflicts) · Delta
tests (E2E Hub étendu + Vitest Hub + a11y) · Verification §7.4 · Scope cuts · G8 traceability ·
Pre-launch protocol (0 wire ; formulaires pilotés par schéma G) · Codex verification · Carry
closure (ferme HUB-T1 ; précède T canvas).

---

## Phase T — Canvas, outline et mode guidé (3 projections de première classe)

### T.1 Scope
Les **3 projections d'édition de première classe** du même graphe (HUB-T4) : canvas visuel,
**outline structurel** et **mode guidé** pas-à-pas. Modèle de commandes UNIQUE →
**MÊME digest sémantique** quelle que soit la projection (parité prouvée, pas trois éditeurs
divergents). Édition complète **sans drag, sans voix, sans souris, sans lecteur d'écran
requis** (AM-8 + §20.8 pack). `@xyflow/react` 12.11.2 = **dep APP-SIDE bundlée dans l'archive
Hub** (jamais dep runtime daemon/shell) — **re-mesure à l'activation (QPO-5)** : les faits
xyflow du blueprint sont SUPERSEDED. Surface Hub → a11y fermée avant commit. Ouvre
`G-A11Y-PARITY`.

### T.2 Livrables
- `examples/workflow-hub/` : canvas (`@xyflow/react`), outline arborescent, mode guidé ;
  **command model unique** produisant les 3 projections depuis la même source `SBFBWorkflowGraph.v1`.
- Sérialiseur → digest sémantique : test de parité (3 projections d'un graphe → digest identique).
- Gates a11y : `G-A11Y-AUTHORING` (édition non-visuelle complète) + `G-A11Y-PARITY` (les 3
  projections offrent les mêmes opérations).

### T.3 Tests plan
- `web/e2e/workflow-hub-author.spec.ts` (étendu, **T1**) — construire un graphe via canvas
  PUIS via outline PUIS via mode guidé → **même digest** ; parcours clavier-only + lecteur d'écran.
- Vitest Hub : `canvas-command-model.test.tsx`, `outline-projection.test.tsx`,
  `guided-mode.test.tsx`, `projection-parity.test.tsx` (digest identique 3 voies).
- `authoring.a11y` (axe-core + forced-colors + RTL) : 0 violation critique/sérieuse.

### T.4 Critère d'acceptation
- `npm run test:e2e -- workflow-hub-author` exit 0 (**T1 GREEN**) ; suites web (unit/coverage/
  build/size/scan/tsc) exit 0.
- **Condition binaire** : les 3 projections éditent le même graphe et produisent un **digest
  sémantique identique** ; édition complète prouvée sans drag/voix/souris/lecteur.
- T2 : palier HUB-T4 (canvas+outline+guidé, parité) — evidence = E2E Hub ; agrégé AI2.

### T.5 Commit cible
`feat(hub): Sprint 83 Phase T — canvas, outline et mode guidé (parité digest sémantique)`.
Body 9 sections : Contexte (HUB-T4 ; 3 projections première classe ; parité non-visuelle ;
xyflow app-side re-mesuré QPO-5) · Fichiers (workflow-hub canvas/outline/guided, parity test,
xyflow bundlé) · Delta tests (E2E + Vitest Hub + a11y parité) · Verification §7.4 · Scope cuts
(co-édition temps réel HORS MVP, cut 3) · G8 traceability · Pre-launch protocol (0 wire ;
xyflow jamais dep daemon/shell) · Codex verification · Carry closure (ferme HUB-T4 ; ouvre
G-A11Y-PARITY).

---

## Phase U — Inspecteurs, capacités et preflight

### U.1 Scope
Les **inspecteurs typés** (HUB-T5) : formulaires pilotés par schémas, panneau de **capacités**
(grants demandés / effectifs / refusés / expirés — projection RESTITUÉE, jamais calculée UI),
**preflight 10 points** avant exécution (schéma, capacités, budgets, placement, secrets,
HITL, coûts, données, provenance, réversibilité). **Invariant** : toute mutation du graphe
**invalide le consentement** (re-preflight obligatoire). Surface Hub → a11y fermée avant
commit. Ferme HUB-T2 (au sens inspecteurs+capacités).

### U.2 Livrables
- `examples/workflow-hub/` : inspecteurs typés (agent/node/graphe), panneau capacités, écran
  preflight 10 points ; formulaires pilotés par `SBFBWorkflowGraph.v1` + `SBFBAgentDefinition.v1`.
- Logique « mutation → consentement invalidé » (côté iframe, restitution ; l'autorité de grant
  reste host-side, Phase C/L).
- Gates a11y de la surface fermés.

### U.3 Tests plan
- `web/e2e/workflow-hub-author.spec.ts` (étendu, **T1**) — remplir inspecteurs + preflight
  10 points + muter le graphe → consentement invalidé (re-preflight exigé) ; clavier-only.
- Vitest Hub : `inspectors.test.tsx`, `capabilities-panel.test.tsx`, `preflight-10.test.tsx`,
  `mutation-invalidates-consent.test.tsx`.
- `inspectors.a11y` (axe-core) : 0 violation critique/sérieuse.

### U.4 Critère d'acceptation
- `npm run test:e2e -- workflow-hub-author` exit 0 (**T1 GREEN**) ; suites web exit 0.
- **Condition binaire** : preflight 10 points opérationnel + panneau capacités RESTITUÉ
  (0 verdict calculé UI) + mutation du graphe invalide le consentement (re-preflight).
- T2 : palier HUB-T5 (inspecteurs+capacités+preflight) — evidence = E2E Hub ; agrégé AI2.

### U.5 Commit cible
`feat(hub): Sprint 83 Phase U — inspecteurs, capacités et preflight 10 points`.
Body 9 sections : Contexte (HUB-T5 ; capacités restituées ; mutation→consentement invalidé) ·
Fichiers · Delta tests (E2E + Vitest Hub + a11y) · Verification §7.4 · Scope cuts · G8
traceability · Pre-launch protocol (0 wire ; grants host-side, iframe restitue) · Codex
verification · Carry closure (ferme HUB-T2 volet inspecteurs).

---

## Phase V — Centre des Runs

### V.1 Scope
Le **Centre des Runs** (HUB-T3) : liste, statuts, **timeline bornée** (pas de croissance non
bornée), child-runs, **HITL** (intervention humaine requise), pause/cancel/retry/replay,
**`outcome_unknown` affiché honnêtement — jamais un faux succès** (MUR jamais bouton, §12.7).
Consomme le runtime E/I/J/K (autorité host-side ; le Hub RESTITUE l'état du journal, ne le
possède pas). Surface Hub → a11y fermée avant commit. Ferme HUB-T3.
**Reliure DS-1 (héritée de Phase L) — contrat explicite** : c'est ici que la computation
autoritaire `effective_capabilities` (module `capability`, Phase L) est CÂBLÉE dans le dispatch
d'effet (`orchestrator::run_effect`). Exigences NON négociables de la reliure (Codex r2 P2) :
(1) intersection recomputée PAR-ATTEMPT à l'émission contre le `now_ms` watermarké (jamais un
verdict d'autorité mis en cache) ; (2) refus d'autorité JOURNALISÉS comme décisions (le replay
lit le journal, jamais la policy live) ; (3) résolveur autoritaire `capability_ref`
(String pack/plan) → `BoundedCapability` typée **fail-closed : inconnu/mal-mappé ⇒ REFUS**
(ferme la moitié frontière de T-CAP-CLASS-UNKNOWN-FAILOPEN, aujourd'hui « bornée ») ;
(4) provenance autoritaire de `EffectNeed` (dérivée du plan compilé, jamais d'une sortie modèle) ;
(5) validation de la destination/quota/timeout/data-policy de l'effet RÉEL contre la borne
retournée ; (6) unicité — ou erreur typée surfacée à l'opérateur — pour plusieurs grants
d'une même ref/audience (le module droppe fail-closed mais silencieusement, cf. borne T-CAP-WIDEN-BYPASS) ;
(7) distinction d'API entre candidat déclaré (`from_declared`) et résultat calculé (un futur
type/newtype de résultat non forgeable) ; (8) liaison à l'identité EXACTE (Codex r3) — la reliure
ne passe à la couche release QUE les grants de la release installée (digest/identité), à la couche
pack QUE ceux de la version pack courante (un grant release-A ne vaut pas pour release-B, pack-v1
pas pour pack-v2 — `GrantAudience` ne distingue que le TYPE, le sujet exact est de la responsabilité
de l'appelant) + `EffectNeed::One` imposé au dispatch avec concordance classe/opération réelle.
Les décisions HITL rendues ici alimentent la couche 7.
Ferme T-CAP-EXPIRY-TOCTOU-RETRY / T-CAP-REPLAY-REAUTHORIZE (THREAT §19.3). **Dette possédée par V** :
`compile.rs:606-610` (« per-effect enforcement binding is Phases J/K/L », champ schemars laissé
byte-intact pour préserver le golden) devient factuellement exact quand cette reliure est livrée.

### V.2 Livrables
- `examples/workflow-hub/` : centre des runs (liste/statuts/timeline/child-runs/HITL/
  pause/cancel/retry/replay) ; rendu `outcome_unknown` distinct de succès/échec.
- Câblage bridge → runtime (commandes host `run.pause/cancel/retry`, events `run.step/hitl`).
- Gates a11y de la surface fermés (annonces live-region pour les transitions de statut).

### V.3 Tests plan
- `web/e2e/workflow-hub-run.spec.ts` (**T1 NOMMÉ, AM-1**, hermétique — fixture daemon
  déterministe) — lancer/suivre/pause/reprendre/annuler/retry un run ; HITL ; `outcome_unknown`
  rendu sans faux succès ; clavier-only + live-region.
- Vitest Hub : `runs-center.test.tsx`, `run-timeline-bounded.test.tsx`, `hitl-panel.test.tsx`,
  `outcome-unknown-render.test.tsx`.
- `runs.a11y` (axe-core) : 0 violation critique/sérieuse.

### V.4 Critère d'acceptation
- `npm run test:e2e -- workflow-hub-run` exit 0 (**T1 GREEN**) ; job CI « Workflow Hub E2E »
  vert (spec run ajoutée au filtre) ; suites web exit 0.
- **Condition binaire** : run pilotable de bout en bout (pause/reprise/cancel/retry/replay) +
  `outcome_unknown` jamais rendu comme succès + timeline bornée.
- T2 : palier HUB-T3 (runs) — evidence = E2E Hub `workflow-hub-run` ; agrégé AI2 (dépend
  E/I/J/K livrés).

### V.5 Commit cible
`feat(hub): Sprint 83 Phase V — centre des Runs (HITL, outcome_unknown honnête)`.
Body 9 sections : Contexte (HUB-T3 ; restitution du journal host ; MUR jamais bouton) ·
Fichiers (workflow-hub runs, bridge run commands) · Delta tests (E2E run nommé + Vitest Hub +
a11y) · Verification §7.4 · Scope cuts (cancel fort réseau HORS MVP, cut 9) · G8 traceability ·
Pre-launch protocol (0 wire ; commandes run bridge additives) · Codex verification · Carry
closure (ferme HUB-T3 ; 2e spec T1 câblée CI).

---

## Phase W — Évaluations et comparaison

### W.1 Scope
Le **banc d'évaluation** (HUB-T6) : datasets et scénarios **versionnés**, **graders
déterministes exécutés AVANT tout grader LLM**, comparaison de résultats, export de preuve.
**Invariant cardinal** : **aucun grader LLM n'accorde de sécurité ni de verdict PASS** — un
grader LLM produit un signal, jamais une autorité (parité §18.2 pack). Ferme `G-EVAL`.

### W.2 Livrables
- `crates/sbfb-workflow-engine` (ou module dédié) : `EvaluationSpec.v1`, exécution
  déterministe-d'abord, ordre des graders imposé (déterministes → LLM signal).
- `examples/workflow-hub/` : UI datasets/scénarios versionnés, comparaison, export preuve.
- Gate `G-EVAL` : test prouvant qu'un grader LLM ne peut pas fermer un critère de sécurité.

### W.3 Tests plan
- Rust : `eval_spec_versioned.rs`, `graders_deterministic_first.rs`,
  `llm_grader_cannot_grant_pass.rs` (le grader LLM est un signal, jamais une autorité).
- `web/e2e/workflow-hub-run.spec.ts` (étendu) — créer un eval, comparer 2 runs, exporter preuve.
- Vitest Hub : `eval-datasets.test.tsx`, `eval-compare.test.tsx`.

### W.4 Critère d'acceptation
- Rust nextest (nouveaux) verts ; `llm_grader_cannot_grant_pass` prouve l'invariant.
- `npm run test:e2e -- workflow-hub-run` exit 0 (**T1 GREEN**) ; suites web exit 0.
- **Condition binaire** : graders déterministes exécutés avant LLM + export de preuve
  reproductible + aucun grader LLM n'accorde PASS/sécurité.
- T2 : palier HUB-T6 (evals) — evidence = tests Rust + E2E ; agrégé AI2.

### W.5 Commit cible
`feat(engine+hub): Sprint 83 Phase W — évaluations versionnées, graders déterministes d'abord`.
Body 9 sections : Contexte (HUB-T6 ; grader LLM = signal jamais autorité ; ordre §18.2) ·
Fichiers (engine eval, workflow-hub eval UI) · Delta tests (Rust eval + E2E + Vitest) ·
Verification §7.4 · Scope cuts · G8 traceability · Pre-launch protocol (0 wire ;
EvaluationSpec.v1 additif) · Codex verification · Carry closure (ferme G-EVAL).

---

## Phase W1 — Panel utilisateurs round 1 (AM-9)

### W1.1 Scope
**Round 1 du panel handicapé rémunéré** (AM-9 ; ressource du préalable AM-10) sur les surfaces
**auteur (R..U)** + **runs (V)** désormais disponibles. Objectif : détecter les barrières
réelles d'usage AVANT la publication (X). **Toute barrière P0/P1 découverte est routée en
rework AVANT X** (pas un carry). **Checkpoint ressources pré-W1** (AM-10) : panel recruté,
calendarisé, consentement obtenu. Ouvre `G-A11Y-USERS`.

### W1.2 Livrables
- Protocole de panel (tâches, consentement, rémunération — cadre vision_model : dépense
  opérationnelle ponctuelle, JAMAIS pattern institutionnel), matrice participant × handicap × AT.
- `.planning/active/sprint83_w1_panel_round1.json` : findings structurés (barrière, sévérité,
  surface, AT) + décisions de rework.
- Rework des barrières P0/P1 identifiées (commits de correction sur les surfaces R..V).

### W1.3 Tests plan
- Pas de nouveau test unitaire (phase de validation humaine) ; **les rework produits sont
  couverts par les suites a11y existantes** (axe-core, clavier-only, forced-colors) des
  surfaces corrigées, re-vertes.
- Régression E2E : `workflow-hub-author` + `workflow-hub-run` restent GREEN après rework.

### W1.4 Critère d'acceptation
- Panel round 1 joué ; `sprint83_w1_panel_round1.json` committé (findings + décisions).
- **Condition binaire** : **0 barrière P0/P1 non traitée avant X** (chaque P0/P1 → rework
  committé + suite a11y re-verte, OU décision PO tracée). Un P0/P1 non résolu BLOQUE X.
- T2 : palier WA-T-a11y-users round 1 — status `ACTED{panel_json}` OU `RIG-ABSENT{panel
  non recruté}` si la ressource AM-10 défaille (→ retour PO, jamais carry masqué).

### W1.5 Commit cible
`test(a11y): Sprint 83 Phase W1 — panel utilisateurs round 1 + rework barrières P0/P1`.
Body 9 sections : Contexte (AM-9 ; ressource AM-10 ; barrières avant X) · Fichiers (protocole,
panel json, rework surfaces) · Delta tests (a11y re-vertes ; 0 nouveau) · Verification §7.4 ·
Scope cuts · G8 traceability · Pre-launch protocol (0 wire) · Codex verification · Carry
closure (ouvre G-A11Y-USERS ; barrières routées rework, pas carry).

---

## Phase X — Publication, découverte, installation et RRV

### X.1 Scope
Le **cycle de vie complet des packs** (HUB-T7) : publish → install-**désactivé** → activate →
fork → **update-reconsent** (une mise à jour élargissant les droits est BLOQUÉE, curated_updates)
→ rollback → dépréciation → révocation — **sans auto-update**. **Projection publique + Proof
Card** (RRV : source vérifiable, provenance, digests) — jamais un ranking global présenté
comme vérité (§16.2). Re-coupe suffixée candidate (AM-4). Ferme `G-PUBLISH-RECOVERY`,
`G-RRV-PROJECTION`, `G-LIFECYCLE`.

### X.2 Livrables
- Runtime/companion : cycle publish/install/activate/fork/update-reconsent/rollback/deprecate/
  revoke connecté au protocole (feed raw-op additif 0-bump, provenance SLSA existante).
- `examples/workflow-hub/` : bibliothèque publique, écran d'installation (désactivé par défaut),
  réconsentement à l'activation, Proof Card (projection RRV).
- Gates : `G-PUBLISH-RECOVERY` (rollback prouvé), `G-RRV-PROJECTION` (Proof Card = restitution),
  `G-LIFECYCLE` (transitions légales seulement).

### X.3 Tests plan
- `web/e2e/workflow-hub-lifecycle.spec.ts` (**T1 NOMMÉ, AM-1**, hermétique) — publish → install
  désactivé → activate (reconsent) → fork → update élargissant droits BLOQUÉ → rollback →
  revoke ; clavier-only.
- Rust : `pack_lifecycle_transitions.rs`, `update_widening_grants_blocked.rs`,
  `publish_rollback.rs`, `rrv_projection_is_restitution.rs`.
- Vitest Hub : `lifecycle-ui.test.tsx`, `proof-card.test.tsx`, `reconsent-on-activate.test.tsx`.

### X.4 Critère d'acceptation
- `npm run test:e2e -- workflow-hub-lifecycle` exit 0 (**T1 GREEN**) ; job CI « Workflow Hub
  E2E » vert (3e spec au filtre) ; Rust nextest verts ; suites web exit 0.
- **Condition binaire** : cycle complet sans auto-update + update élargissant les droits BLOQUÉ
  + rollback prouvé + Proof Card = restitution (0 ranking-vérité).
- T2 : palier HUB-T7 (lifecycle+RRV) — evidence = E2E lifecycle + Rust ; agrégé AI2.

### X.5 Commit cible
`feat(hub+runtime): Sprint 83 Phase X — cycle de vie packs + RRV Proof Card (sans auto-update)`.
Body 9 sections : Contexte (HUB-T7 ; curated_updates ; heberger≠publier ; §16.2 anti-capture) ·
Fichiers (runtime lifecycle, workflow-hub library/proof-card, feed raw-op) · Delta tests (E2E
lifecycle nommé + Rust + Vitest) · Verification §7.4 · Scope cuts (marketplace/ranking JAMAIS
cut 5 ; auto-update HORS MVP cut 6) · G8 traceability · Pre-launch protocol (feed raw-op
0-bump) · Codex verification · Carry closure (ferme G-PUBLISH-RECOVERY/G-RRV-PROJECTION/
G-LIFECYCLE ; 3e spec T1 câblée CI).

---

## Phase Y — Opérations inter-apps

### Y.1 Scope
Les **opérations inter-apps** (HUB-T8) : une app SBFB autorisée invoque une **opération
headless** du Hub et reçoit un artefact typé. **child-runs idempotents dédupliqués**
(clé `(caller, op, args-digest)`), garde **depth/cycles/budgets**, **handoffs par
`ArtifactRef` sous ACL** (le handoff est une DONNÉE signée, jamais du control-flow —
[[idea_hub_factory_ultracode_redesign]]). Ferme `G-APPCALL`.

### Y.2 Livrables
- Runtime : dispatch d'opération inter-apps (intersection de grants Phase C), déduplication
  idempotente, gardes depth/cycles/budgets, résolution `ArtifactRef` sous ACL (Phase M).
- Contrat host `app.invoke` (bridge type, capability-scoped) ; jamais d'appel iframe→iframe.
- Gate `G-APPCALL` : test de dédup + garde de cycle + ACL de handoff.

### Y.3 Tests plan
- Rust : `appcall_idempotent_dedup.rs`, `appcall_depth_cycle_budget_guard.rs`,
  `handoff_artifactref_acl.rs` (un ArtifactRef hors ACL est refusé), `appcall_grant_intersection.rs`.
- `web/e2e/workflow-hub-run.spec.ts` (étendu) — une app invoque une opération Hub → artefact typé.
- Vitest Hub : `app-operations.test.tsx`.

### Y.4 Critère d'acceptation
- Rust nextest (nouveaux) verts ; dédup + gardes + ACL prouvés.
- `npm run test:e2e -- workflow-hub-run` exit 0 (**T1 GREEN**) ; suites web exit 0.
- **Condition binaire** : child-run rejoué avec mêmes args = 1 seul effet (idempotent) +
  cycle/budget gardés + handoff refusé hors ACL.
- T2 : palier HUB-T8 (inter-apps) — evidence = Rust + E2E ; agrégé AI2.

### Y.5 Commit cible
`feat(runtime+hub): Sprint 83 Phase Y — opérations inter-apps (child-runs idempotents, handoff ACL)`.
Body 9 sections : Contexte (HUB-T8 ; handoff = donnée signée jamais control-flow ; pas d'appel
iframe→iframe) · Fichiers (runtime appcall, bridge app.invoke, workflow-hub ops) · Delta tests
(Rust appcall + E2E + Vitest) · Verification §7.4 · Scope cuts · G8 traceability · Pre-launch
protocol (ArtifactRef additif) · Codex verification · Carry closure (ferme G-APPCALL).

---

## Phase Z — Factory dogfood : kickoff, plan et préflight (vertical fondateur)

### Z.1 Scope
**Premier vertical golden (Factory)** — le chantier se prouve sur SON PROPRE processus.
Rejouer, dans un workspace + namespace **`dogfood_factory`** isolé, le **cycle d'entrée
Factory réel** : kickoff → plan → préflight G8, tel que défini par `FactoryProcessProfile.v1`
(Phase D). Les statuts du sprint interne restent dans le namespace `dogfood_factory` et **ne
ferment pas S83**. Ouvre `G-VERTICALS` (Factory = 1er des 4). Bloquants : `DESIGN-CONFLICT`,
`PROFILE_STALE`.

### Z.2 Livrables
- `crates/sbfb-workflow-engine` (+ packs first-party sous `packs/factory/`) : le workflow
  Factory kickoff/plan/préflight exécutable via le moteur, lié au profil D (digests revalidés).
- Namespace/workspace `dogfood_factory` hermétique (isolé du repo réel, `NEXUS_GRID_ROOT`
  confiné — culture TEST-ISOLATION A1).
- Artefacts internes kickoff/plan/préflight produits PAR le moteur (pas à la main).

### Z.3 Tests plan
- Rust : `dogfood_factory_kickoff_runs.rs`, `profile_stale_blocks.rs` (mismatch digest →
  `PROFILE_STALE` bloquant), `design_conflict_halts.rs`, `namespace_isolation.rs` (le dogfood
  n'écrit jamais hors `dogfood_factory`).
- Doubles hermétiques CI (pas de credential, LLM stubbed).

### Z.4 Critère d'acceptation
- Rust nextest verts ; `PROFILE_STALE` et `DESIGN-CONFLICT` prouvés bloquants ; isolation du
  namespace prouvée (0 écriture hors sandbox).
- **Condition binaire** : le cycle d'entrée Factory (kickoff/plan/préflight) est REJOUÉ par le
  moteur dans `dogfood_factory`, produisant les artefacts attendus, sans toucher le repo réel.
- T2 : palier WA-T-Factory-entrée — evidence = Rust dogfood ; agrégé AI2 (G-VERTICALS 1/4 ouvert).

### Z.5 Commit cible
`feat(engine): Sprint 83 Phase Z — Factory dogfood (kickoff/plan/préflight, namespace isolé)`.
Body 9 sections : Contexte (vertical fondateur ; profil D ; PROFILE_STALE ; isolation namespace) ·
Fichiers (engine, packs/factory, dogfood namespace) · Delta tests (Rust dogfood) · Verification
§7.4 · Scope cuts · G8 traceability · Pre-launch protocol (0 wire ; artefacts internes) · Codex
verification · Carry closure (ouvre G-VERTICALS Factory 1/4).

---

## Phase AA — Factory dogfood : implémentation et tests

### AA.1 Scope
Le cœur d'exécution du vertical Factory : **worktree isolé** (culture `isolation: worktree`),
driver d'implémentation, **effets FS/process liés à HEAD/index/worktree** (préconditions
vérifiées, Phase K), **3 blocs de vérification complets** (2 pré-commit + 3 Docker pré-push,
AM-6) rejoués par le moteur. Robustesse adversariale : **injection / repo hostile sans mutation
du plan ni des capacités** (le plan est immuable post-compilation, Plan-Then-Execute). Ferme
`G-FS-PROCESS-EFFECT` ; `G-VERTICALS` (Factory, suite).

### AA.2 Livrables
- `crates/sbfb-workflow-engine` : driver d'implémentation dogfood, effets liés HEAD/index/
  worktree, nettoyage des processus descendants (Windows inclus, Phase K).
- Harness d'injection : repo/prompt hostile → plan et capacités INCHANGÉS.
- Rejeu des 3 blocs de vérification par le moteur dans le worktree.

### AA.3 Tests plan
- Rust : `dogfood_impl_worktree_isolated.rs`, `effect_bound_to_head_index.rs` (précondition
  divergente → effet refusé), `hostile_repo_no_plan_mutation.rs`,
  `descendant_process_cleanup_windows.rs`.
- `G-FS-PROCESS-EFFECT` : test de fermeture (effet FS/process idempotent + lié + nettoyé).

### AA.4 Critère d'acceptation
- Rust nextest verts ; injection hostile prouvée sans mutation de plan/capacités ; nettoyage
  descendants prouvé (Windows).
- **Condition binaire** : implémentation dogfood dans worktree isolé + effets liés HEAD/index +
  3 blocs rejoués + repo hostile neutralisé.
- T2 : palier WA-T-Factory-impl — evidence = Rust ; agrégé AI2.

### AA.5 Commit cible
`feat(engine): Sprint 83 Phase AA — Factory dogfood implémentation (worktree, effets liés HEAD)`.
Body 9 sections : Contexte (Plan-Then-Execute immuable ; effets liés ; injection neutralisée) ·
Fichiers · Delta tests (Rust dogfood impl) · Verification §7.4 · Scope cuts · G8 traceability ·
Pre-launch protocol (0 wire) · Codex verification · Carry closure (ferme G-FS-PROCESS-EFFECT ;
G-VERTICALS Factory suite).

---

## Phase AB — Factory dogfood : review, Codex et réconciliation

### AB.1 Scope
La **chaîne de vérification indépendante** du vertical Factory : `PASS-PENDING` → **Codex brut**
→ **réconciliation** → boucle bornée jusqu'à CLEAN ou P2/P3 documentés. **Invariant cardinal** :
le **driver ne s'auto-attribue JAMAIS `PASS`** — l'indépendance driver/reviewer/Codex est
structurelle (§10 contraintes, Phase G). Ferme `G-REVIEW-INDEPENDENCE` (dont le contrat
candidat a été défini en D) ; `G-VERTICALS` (Factory, suite).

### AB.2 Livrables
- `crates/sbfb-workflow-engine` : rôles review/Codex/réconciliation avec séparation d'autorité
  (le driver ne peut pas promouvoir son propre verdict) ; Codex output BRUT préservé (jamais
  réécrit — [[feedback_codex_raw_output]]).
- Boucle de réconciliation bornée (P0/P1 → correction → re-run ; P2/P3 → documentés).
- Gate `G-REVIEW-INDEPENDENCE` : test prouvant l'impossibilité d'auto-attribution PASS.

### AB.3 Tests plan
- Rust : `driver_cannot_self_pass.rs` (le rôle driver n'a pas la capacité de promouvoir),
  `codex_output_raw_preserved.rs`, `reconciliation_loop_bounded.rs`,
  `review_independence_roles.rs`.

### AB.4 Critère d'acceptation
- Rust nextest verts ; auto-attribution PASS prouvée IMPOSSIBLE ; Codex brut préservé.
- **Condition binaire** : PASS-PENDING → Codex → réconciliation joués par le moteur ; le driver
  ne ferme jamais son propre verdict ; boucle bornée.
- T2 : palier WA-T-Factory-review — evidence = Rust ; agrégé AI2.

### AB.5 Commit cible
`feat(engine): Sprint 83 Phase AB — Factory dogfood review/Codex/réconciliation (indépendance)`.
Body 9 sections : Contexte (indépendance structurelle ; driver ≠ verdict ; Codex brut) ·
Fichiers · Delta tests (Rust indépendance) · Verification §7.4 · Scope cuts · G8 traceability ·
Pre-launch protocol (0 wire) · Codex verification · Carry closure (ferme G-REVIEW-INDEPENDENCE ;
G-VERTICALS Factory suite).

---

## Phase AC — Factory dogfood : verdict, commit, carries et closure (premier golden LIVRÉ)

### AC.1 Scope
La **fermeture du vertical Factory** : promotion **Verdict PASS**, **`commit.intent` et
`push.intent` = confirmations HUMAINES** (le moteur n'a pas l'autorité de commit/push final —
§17.1, hooks de finalité ex-D17), **les hooks REFUSENT** un commit/push non confirmé, statuts
du sprint INTERNE (`IMPLEMENTED_AUDIT_PENDING` / `AUDITED_CLOSED`) restant dans
`dogfood_factory`. **Position du push dogfood ratifiée (Q7)** : `push.intent` APRÈS
verification/audit_plan/wrap-up interne. Ferme `G-HOOK-FINALITY` ; **G-VERTICALS : Factory LIVRÉ
(1er golden complet)**.

### AC.2 Livrables
- `crates/sbfb-workflow-engine` : promotion de verdict, points d'intention commit/push
  confirmés-humain, intégration hooks de finalité (refus si non confirmé).
- Statuts du sprint interne dans le namespace `dogfood_factory` (ne ferment pas S83).
- Le workflow Factory complet (Z→AC) exécutable de bout en bout = **premier golden prouvé**.

### AC.3 Tests plan
- Rust : `verdict_promotion_requires_pass.rs`, `commit_intent_needs_human.rs`,
  `hooks_refuse_unconfirmed.rs`, `internal_status_stays_in_namespace.rs`,
  `factory_golden_end_to_end.rs` (Z→AC enchaînés dans le namespace).

### AC.4 Critère d'acceptation
- Rust nextest verts ; commit/push sans confirmation humaine REFUSÉ par les hooks ; le golden
  Factory tourne de bout en bout dans `dogfood_factory`.
- **Condition binaire** : Factory (Z→AC) = vertical golden COMPLET exécuté par le moteur, avec
  finalité humaine et statuts internes cloisonnés.
- T2 : palier WA-T-Factory-closure — evidence = `factory_golden_end_to_end` ; G-VERTICALS 1/4
  FERMÉ (Factory) ; agrégé AI2.

### AC.5 Commit cible
`feat(engine): Sprint 83 Phase AC — Factory dogfood closure (verdict, finalité humaine, golden 1/4)`.
Body 9 sections : Contexte (finalité humaine ex-D17 ; statuts internes cloisonnés ; push après
wrap-up interne Q7) · Fichiers · Delta tests (Rust closure + golden e2e) · Verification §7.4 ·
Scope cuts (dépréciation legacy POST-S83 cut 14) · G8 traceability · Pre-launch protocol (0 wire) ·
Codex verification · Carry closure (ferme G-HOOK-FINALITY ; G-VERTICALS Factory 1/4 LIVRÉ).

---

## Phase AD — Pack Ideas (2e vertical)

### AD.1 Scope
Le vertical **Ideas** : Intake → clarification → grounding repo + recherche externe en
parallèle → génération d'options → agents critiques indépendants → synthèse → arbitrage humain →
**`ResearchPack` + `DecisionPack`** — **exécutable, forkable, evaluable** via le moteur.
Réutilise la base commune (pas un moteur ad hoc — §1 correction de cadrage). `G-VERTICALS` (2/4).

### AD.2 Livrables
- `packs/ideas/` : `WorkflowDefinition` Ideas + `AgentDefinition`s (clarificateur, grounding,
  critiques, synthèse) + schémas de sortie `ResearchPack.v1` / `DecisionPack.v1`.
- Câblage handoff optionnel Ideas → Conception/Factory par `ArtifactRef` (Phase Y).

### AD.3 Tests plan
- Rust : `ideas_pack_compiles_and_runs.rs`, `ideas_parallel_grounding_join.rs`
  (fan-out/fan-in), `ideas_human_arbitration_hitl.rs`, `researchpack_decisionpack_typed.rs`.
- `web/e2e/workflow-hub-run.spec.ts` (étendu) — lancer Ideas, produire les 2 packs.

### AD.4 Critère d'acceptation
- Rust nextest verts ; Ideas compile + tourne + produit ResearchPack+DecisionPack typés ;
  fan-out/fan-in + HITL prouvés.
- **Condition binaire** : Ideas exécutable/forkable/evaluable, produit des artefacts typés,
  réutilise la base commune.
- T2 : palier WA-T-Ideas — evidence = Rust + E2E ; G-VERTICALS 2/4 ; agrégé AI2.

### AD.5 Commit cible
`feat(packs): Sprint 83 Phase AD — pack Ideas (ResearchPack + DecisionPack, base commune)`.
Body 9 sections : Contexte (2e vertical ; réutilise la base, pas de moteur ad hoc) · Fichiers
(packs/ideas) · Delta tests (Rust Ideas + E2E) · Verification §7.4 · Scope cuts · G8 traceability ·
Pre-launch protocol (0 wire ; packs déclaratifs) · Codex verification · Carry closure
(G-VERTICALS 2/4).

---

## Phase AE — Pack Conception (3e vertical)

### AE.1 Scope
Le vertical **Conception** : **4 analyses parallèles**, cross-review, **ADRs**, `ConceptionPack`
**consommable par Factory** (chaîne Ideas → Conception → Factory prouvée par composition).
`G-VERTICALS` (3/4).

### AE.2 Livrables
- `packs/conception/` : `WorkflowDefinition` (4 analyses parallèles + cross-review + ADR
  synthesis) + schéma `ConceptionPack.v1` consommable par le pack Factory (Phase D/Z).

### AE.3 Tests plan
- Rust : `conception_four_parallel_analyses.rs`, `conception_cross_review.rs`,
  `conceptionpack_consumable_by_factory.rs` (handoff typé Conception → Factory).
- `web/e2e/workflow-hub-run.spec.ts` (étendu) — Conception produit un ConceptionPack.

### AE.4 Critère d'acceptation
- Rust nextest verts ; 4 analyses parallèles + cross-review + ConceptionPack consommé par Factory.
- **Condition binaire** : Conception exécutable + ConceptionPack typé consommable par le vertical
  Factory (composition prouvée).
- T2 : palier WA-T-Conception — evidence = Rust + E2E ; G-VERTICALS 3/4 ; agrégé AI2.

### AE.5 Commit cible
`feat(packs): Sprint 83 Phase AE — pack Conception (4 analyses, ADRs, ConceptionPack→Factory)`.
Body 9 sections : Contexte (3e vertical ; composition inter-packs) · Fichiers (packs/conception) ·
Delta tests (Rust Conception + E2E) · Verification §7.4 · Scope cuts · G8 traceability ·
Pre-launch protocol (0 wire) · Codex verification · Carry closure (G-VERTICALS 3/4).

---

## Phase AF — Pack Babel (4e vertical)

### AF.1 Scope
Le vertical **Babel** : **rights gate** (droits d'usage du corpus vérifiés), corpus légal borné,
**≥100 chunks hermétiques en CI**, **dogfood 2 workers** (calcul distribué SBFB), crash/reprise,
**reassembly**, provenance. Vertical le plus lourd (calcul réseau réel). `G-VERTICALS` (4/4).

### AF.2 Livrables
- `packs/babel/` : `WorkflowDefinition` Babel (rights gate → chunking → traduction distribuée
  2 workers → reassembly → provenance).
- Fixtures hermétiques CI (≥100 chunks, corpus légal borné, 0 credential).
- Chemin crash/reprise (journal Phase I) + reassembly déterministe.

### AF.3 Tests plan
- Rust : `babel_rights_gate_blocks_unlicensed.rs`, `babel_100_chunks_hermetic.rs`,
  `babel_crash_resume_reassembly.rs`, `babel_provenance_chain.rs`.
- T2 acceptance 2 workers : `sprint83_t2_acceptance.json` palier Babel (chemins RIG-ABSENT si
  2e worker absent — jamais un DIFFERE prose).

### AF.4 Critère d'acceptation
- Rust nextest (hermétiques, ≥100 chunks) verts ; rights gate bloque un corpus non licencié ;
  crash/reprise/reassembly prouvés.
- **Condition binaire** : Babel exécutable + rights gate + ≥100 chunks hermétiques CI +
  crash/reprise/reassembly + provenance ; dogfood 2 workers `ACTED` OU `RIG-ABSENT` honnête.
- T2 : palier WA-T-Babel + WA-T6-partiel (2 workers) — evidence = Rust + acceptance JSON ;
  G-VERTICALS 4/4 (ouvre la fermeture agrégée AI2) ; agrégé AI2.

### AF.5 Commit cible
`feat(packs): Sprint 83 Phase AF — pack Babel (rights gate, ≥100 chunks, dogfood 2 workers)`.
Body 9 sections : Contexte (4e vertical ; calcul distribué réel ; rights gate) · Fichiers
(packs/babel, fixtures) · Delta tests (Rust Babel hermétiques) · Verification §7.4 · Scope cuts ·
G8 traceability · Pre-launch protocol (Task/Result additifs 0-bump, Phase P) · Codex verification ·
Carry closure (G-VERTICALS 4/4 ; ouvre fermeture agrégée AI2).

---

## Phase AG — Assistance IA + import Flowise (source-only)

### AG.1 Scope
L'**assistance IA à l'édition** : une demande en français → **diff déclaratif source-only**
proposé, **accepté ou refusé** par l'utilisateur (jamais appliqué en autorité — l'IA propose,
l'humain dispose, §20.9 contrôle de l'IA). **Import AgentFlow V2 (Flowise) data-only** : la
structure déclarative est importée, **`$flow.state` n'est JAMAIS mappé** (pas de code importé,
§10.5). Ferme `G-AUTHORING-AI-DIFF` et `G-IMPORT-HOSTILE`.

### AG.2 Livrables
- `examples/workflow-hub/` : panneau d'assistance IA (demande FR → diff déclaratif review-able).
- Importeur Flowise AgentFlow V2 data-only (mapping structurel ; refus de tout code/`$flow.state`).
- Gates : `G-AUTHORING-AI-DIFF` (diff proposé jamais auto-appliqué), `G-IMPORT-HOSTILE`
  (fermeture : corpus d'import hostile → refus sans exécution).

### AG.3 Tests plan
- Rust/companion : `import_flowise_data_only.rs` (`$flow.state` rejeté), `import_hostile_corpus_refused.rs`.
- `web/e2e/workflow-hub-author.spec.ts` (étendu) — demande FR → diff proposé → accepté/refusé
  (jamais auto-appliqué).
- Vitest Hub : `ai-assist-diff.test.tsx`.

### AG.4 Critère d'acceptation
- Rust + E2E verts ; import Flowise mappe la structure mais REJETTE `$flow.state`/tout code ;
  diff IA jamais auto-appliqué.
- **Condition binaire** : assistance IA = diff proposé (humain dispose) + import data-only +
  corpus hostile refusé.
- T2 : palier WA-T-AI-assist + WA-T-import — evidence = Rust + E2E ; agrégé AI2.

### AG.5 Commit cible
`feat(hub): Sprint 83 Phase AG — assistance IA (diff source-only) + import Flowise data-only`.
Body 9 sections : Contexte (IA propose/humain dispose §20.9 ; import data-only §10.5 ;
$flow.state jamais mappé) · Fichiers (workflow-hub ai-assist, importeur) · Delta tests (Rust
import + E2E) · Verification §7.4 · Scope cuts (import exécutable total HORS MVP cut 11) · G8
traceability · Pre-launch protocol (0 wire) · Codex verification · Carry closure (ferme
G-AUTHORING-AI-DIFF + G-IMPORT-HOSTILE).

---

## Phase AH — Sécurité adversariale et supply chain

### AH.1 Scope
La **passe de durcissement adversarial** transversale : **fuzz** pack/compiler/bridge/wire ;
scénarios injection / SSRF / révocation per-call / warrant canari / CSP / packs malveillants ;
**supply chain** : locks/licences/SBOM, **build offline reproductible**. Confronte le
`THREAT_MODEL vNN` au §19 du pack. Ferme `G-SUPPLY-CHAIN`. (Intègre le durcissement des rouges
CI fermés en A1 : deny.toml licences, etc.)

### AH.2 Livrables
- Harness de fuzz (pack import, compiler, bridge, wire) ; corpus adversarial.
- Scénarios sécurité : injection prompt, SSRF/DNS/redirect (Phase P), révocation mid-run,
  canari, CSP Hub, pack malveillant refusé.
- Supply chain : `deny.toml` propre (advisories+licences), SBOM, preuve de build offline.

### AH.3 Tests plan
- Rust : `fuzz_pack_import.rs`, `fuzz_compiler.rs`, `bridge_wire_fuzz.rs`,
  `revocation_mid_run.rs`, `ssrf_dns_redirect_blocked.rs`, `malicious_pack_refused.rs`.
- `cargo deny check` (advisories + licenses) VERT ; SBOM généré ; build offline prouvé.
- `web/e2e/*` régression : CSP Hub strict non contournable.

### AH.4 Critère d'acceptation
- Fuzz sans crash/panic exploitable ; `cargo deny check` vert (licences incluses — clôt CI-1) ;
  build offline reproductible ; SSRF/révocation/canari/CSP prouvés.
- **Condition binaire** : surface adversariale (pack/compiler/bridge/wire) fuzzée sans faille +
  supply chain propre + build offline.
- T2 : palier WA-T-security — evidence = Rust fuzz + cargo-deny + SBOM ; agrégé AI2.

### AH.5 Commit cible
`feat(security): Sprint 83 Phase AH — durcissement adversarial + supply chain (fuzz, SBOM, offline)`.
Body 9 sections : Contexte (THREAT vNN vs §19 pack ; zones rouges INCHANGÉES ; clôt CI-1) ·
Fichiers (fuzz harness, deny.toml, SBOM) · Delta tests (Rust fuzz + deny) · Verification §7.4 ·
Scope cuts · G8 traceability · Pre-launch protocol (0 wire) · Codex verification · Carry closure
(ferme G-SUPPLY-CHAIN ; confirme CI-1 clos).

---

## Phase AI — Consolidation fault, scale et perf

### AI.1 Scope
La **consolidation robustesse** (re-coupe AI×3, AM-4) : **fault injection** (crash à chaque
frontière d'effet/journal), **mixed-version** (AM-12 : un run ancien rejoué après upgrade de
schéma), **scale** (nombreux runs/nœuds), **perf/budgets** (TTFT/latence, bundle, storage),
**2 state dirs / 2 daemons** sur une machine + **dogfood 2 machines** (PC+Mac). **La fenêtre
2-machines EST la session Mac chaude** qui solde le **volet Mac stores S82** (§6.2 : observation
on-disk par classe + hygiène rule-4 + redescente consent L4 Mac, Q9). Ferme `G-PERF`.

### AI.2 Livrables
- Harness fault-injection (frontières crash) + mixed-version replay (upcasters Phase I) +
  scale ; budgets perf chiffrés (§15.3 kickoff) mesurés + comparés à un seuil.
- Test 2 state dirs / 2 daemons (culture S80 hermétique) ; acceptance 2 machines PC+Mac.
- **Volet Mac stores S82 soldé** : observation on-disk par classe de store + rule-4 + consent L4.

### AI.3 Tests plan
- Rust : `fault_injection_all_effect_boundaries.rs`, `mixed_version_old_run_replays.rs`,
  `scale_many_runs.rs`, `two_daemons_two_state_dirs.rs`, `perf_budgets_within_threshold.rs`.
- T2 : `sprint83_t2_acceptance.json` paliers fault/scale/perf + volet Mac (`ACTED{on-disk}` OU
  `RIG-ABSENT{Mac froid}` honnête).

### AI.4 Critère d'acceptation
- Rust nextest verts ; 0 double effet à toute frontière de crash ; run ancien rejoué après
  upgrade ; budgets perf sous seuil ; 2 daemons isolés.
- **Condition binaire** : robustesse fault/mixed-version/scale/perf prouvée + volet Mac stores
  soldé (on-disk `ACTED` ou `RIG-ABSENT` tracé).
- T2 : paliers WA-fault/scale/perf + Mac — evidence = Rust + acceptance JSON ; agrégé AI2.

### AI.5 Commit cible
`test(engine): Sprint 83 Phase AI — fault injection, mixed-version, scale, perf + volet Mac stores`.
Body 9 sections : Contexte (durable-execution frontières ; AM-12 mixed-version ; volet Mac S82
§6.2) · Fichiers (fault harness, perf, acceptance json) · Delta tests (Rust robustesse) ·
Verification §7.4 · Scope cuts · G8 traceability · Pre-launch protocol (mixed-version = dérogation
écrite AM-12) · Codex verification · Carry closure (ferme G-PERF ; solde volet Mac S82).

---

## Phase AI1 — A11y multi-AT et panel round 2

### AI1.1 Scope
La **fermeture accessibilité complète** (re-coupe AM-4) : **matrice 5 OS × AT** (JAWS,
Dragon/Voice Access, VoiceOver iOS, TalkBack Android, switch, braille), conformité
**WCAG 2.2 AAA / EN 301 549 / ATAG / COGA**, **panel round 2** (AM-9). **Objectif : 0 barrière
P0/P1**. **Checkpoint ressources pré-AI1** (AM-10 : round 2 recruté + matrice AT disponible).
Ferme les `G-A11Y-*`, `G-A11Y-USERS`, `G-A11Y-PARITY`.

### AI1.2 Livrables
- Matrice de vérification 5 OS/AT jouée (résultats par cellule) ; conformité WCAG AAA/EN 301 549
  documentée par surface.
- `.planning/active/sprint83_ai1_panel_round2.json` : findings round 2 + rework.
- Rework des barrières résiduelles ; suites a11y étendues (multi-AT).

### AI1.3 Tests plan
- `web/e2e/*.a11y` étendus (axe-core AAA + forced-colors + RTL) sur toutes les surfaces Hub.
- Vitest a11y : parité canvas/outline/guidé re-confirmée (G-A11Y-PARITY) ; navigation non-visuelle.
- Régression : les 3 specs T1 Hub restent GREEN sous conditions AT.

### AI1.4 Critère d'acceptation
- Suites a11y AAA vertes ; matrice 5 OS/AT jouée ; **0 barrière P0/P1** (chaque résiduel →
  rework committé + re-vert, OU décision PO tracée).
- **Condition binaire** : conformité multi-AT prouvée + panel round 2 joué + 0 P0/P1 ouvert.
- T2 : palier WA-T-a11y-complet — status `ACTED{matrice+panel2}` OU `RIG-ABSENT{AT/panel absent}`
  (→ retour PO AM-10) ; agrégé AI2.

### AI1.5 Commit cible
`test(a11y): Sprint 83 Phase AI1 — matrice multi-AT, WCAG AAA/EN 301 549, panel round 2`.
Body 9 sections : Contexte (fermeture a11y ; 0 barrière P0/P1 ; AM-9/AM-10) · Fichiers (matrice,
panel2 json, rework) · Delta tests (a11y AAA multi-AT) · Verification §7.4 · Scope cuts · G8
traceability · Pre-launch protocol (0 wire) · Codex verification · Carry closure (ferment
G-A11Y-*, G-A11Y-USERS, G-A11Y-PARITY).

---

## Phase AI2 — Acceptance agrégée (PRÉ-fermeture)

### AI2.1 Scope
L'**acceptance agrégée** du chantier (AM-4) : les paliers **WA-T1..T5 + pré-cutover WA-T6**
joués sur les **4 packs** (Factory/Ideas/Conception/Babel) × **3 placements** (local/cloud/réseau
SBFB). **`sprint83_t2_acceptance.json` écrit avec `T6 = MIXED{pre-cutover ACTED, cutover
NOT-RUN}`** (AM-1 règle bi-temporelle — la finalisation T6 est en AJ1). **Aucun P0/P1 en carry
de confort.** Fermeture agrégée `G-VERTICALS` (4/4) ; **`G-SCOPE-FLOOR` = PRÉ-fermeture
seulement** (fermeture réelle en AJ1, AM-2).

### AI2.2 Livrables
- `.planning/active/sprint83_t2_acceptance.json` : top-level `PASS`/`BLOCK{diagnosis}`/
  `RIG-ABSENT` ; paliers WA-T1..T5 `ACTED{evidence}`/`MIXED`/`NOT-RUN` ; **WA-T6 =
  MIXED{pre-cutover ACTED, cutover NOT-RUN}** ; 4 packs × 3 placements (chemins RIG-ABSENT
  documentés pour les paliers 2-machines/cloud si ressource absente).
- Récapitulatif de fermeture des 4 verticaux (G-VERTICALS 4/4).

### AI2.3 Tests plan
- Agrégation : script d'acceptance qui collationne les evidences des paliers (Z..AH) et écrit le
  JSON ; validation `jq`/`python -c json.load` (parse + vocabulaire fermé).
- Re-run des suites de fermeture des 4 verticaux (pas de nouveau test unitaire ; agrégation).

### AI2.4 Critère d'acceptation
- `sprint83_t2_acceptance.json` existe, parse, vocabulaire fermé ; WA-T1..T5 renseignés ;
  T6=MIXED ; 4 packs × 3 placements présents.
- **Condition binaire** : acceptance agrégée écrite (pré-fermeture) sans P0/P1 en carry de
  confort ; G-VERTICALS 4/4 fermé ; G-SCOPE-FLOOR pré-fermé.
- **T2** : c'est LE row d'acceptance agrégé — `sprint83_t2_acceptance.json` (finalisé AJ1).

### AI2.5 Commit cible
`test(acceptance): Sprint 83 Phase AI2 — acceptance agrégée pré-fermeture (4 packs × 3 placements)`.
Body 9 sections : Contexte (AM-1 bi-temporel T6=MIXED ; AM-2 G-SCOPE-FLOOR pré-fermeture) ·
Fichiers (t2_acceptance.json, script agrégat) · Delta tests (agrégation, 0 unitaire neuf) ·
Verification §7.4 · Scope cuts · G8 traceability · Pre-launch protocol (0 wire) · Codex
verification · Carry closure (G-VERTICALS 4/4 ; G-SCOPE-FLOOR pré-fermé).

---

## Phase AJ — Shadow Factory, cutover et rollback

### AJ.1 Scope
La **preuve de bascule** (AM-4/AM-5) : **shadow** = rejouer le sprint interne `dogfood_factory`
(Z..AC) en `authority=legacy` sur le **même intake** que le chemin Factory legacy réel, **oracle
différentiel** (les deux produisent le même artefact frontière), **kill switch / snapshot /
rollback** prouvés. **Le palier WA-T6 est mis à jour** (cutover joué en shadow). **La
dépréciation du chemin legacy pour les VRAIS sprints (`authority=new`, WA-T6.8) est une décision
humaine POST-S83** (AM-5, scope cut 14) — S83 livre la CAPACITÉ prouvée, pas la bascule. Ferme
`G-FACTORY-CUTOVER`.

### AJ.2 Livrables
- Mode shadow `authority=legacy` + oracle différentiel (comparaison frontière legacy vs moteur).
- Kill switch + snapshot + rollback opérationnels et testés.
- Mise à jour du palier WA-T6 dans une évidence intermédiaire (finalisée AJ1).

### AJ.3 Tests plan
- Rust : `shadow_replays_same_intake.rs`, `differential_oracle_matches.rs`,
  `kill_switch_halts.rs`, `snapshot_rollback_restores.rs`.
- T2 : palier WA-T6 cutover joué en shadow (`ACTED{oracle vert}` OU `BLOCK{diagnosis}`).

### AJ.4 Critère d'acceptation
- Rust nextest verts ; oracle différentiel VERT (legacy ≡ moteur sur même intake) ; kill switch
  + rollback prouvés.
- **Condition binaire** : bascule PROUVÉE en shadow (oracle vert, kill/rollback) — la
  dépréciation legacy réelle reste une décision humaine POST-S83 (cut 14).
- T2 : palier WA-T6 mis à jour (cutover shadow) — finalisé AJ1.

### AJ.5 Commit cible
`feat(engine): Sprint 83 Phase AJ — shadow Factory, oracle différentiel, kill switch/rollback`.
Body 9 sections : Contexte (AM-5 ; capacité de bascule prouvée ≠ dépréciation ; cut 14) ·
Fichiers (shadow mode, oracle, kill/rollback) · Delta tests (Rust cutover) · Verification §7.4 ·
Scope cuts (dépréciation legacy POST-S83 cut 14) · G8 traceability · Pre-launch protocol (0 wire) ·
Codex verification · Carry closure (ferme G-FACTORY-CUTOVER ; met à jour WA-T6).

---

## Phase AJ1 — Clôture docs-contrat, verification et wrap-up

### AJ1.1 Scope
Le **wrap-up de fermeture** (livrable de fermabilité, DoD §4 (c)/(d)) : **clôture docs-contrat**
(GUIDE `docs/factory/` + `llms.txt` + `WIRING_SPEC` pour chaque frontière neuve ; THREAT_MODEL
`vNN` + PATTERNS **§P76+**, AM-3) ; **note d'en-tête « espace §P COMMUN » + 7 doublons P33..P39
documentés dans les DEUX PATTERNS** (C-1) ; **oracle ±0 refacto pur canonisé §P75 point 6**
(C-3) ; **finalisation `sprint83_t2_acceptance.json` (palier T6) AVANT `verification.md`**
(AM-1) ; `sprint83_verification.md` (§Acceptance vocabulaire fermé + §Fail-fast) ;
**`sprint84_audit_plan.md` incluant explicitement Track J — Testabilité ET Track K —
docs-contrat** + les re-routes P3 tracés (§6.2) ; **go/no-go pilote fermé** ; G6/G7. Ferme
**`G-SCOPE-FLOOR` (au sens AM-5)** ; Track K fermable.

### AJ1.2 Livrables
- `docs/factory/` GUIDE + `llms.txt` + `WIRING_SPEC.md` indexant TOUTES les frontières neuves
  (bridge étendu, PackV1, GraphV1, ProfileV1, ProjectionV1, routes runtime, IPC companion, wire,
  MCP) ; `check-frontier-contracts.sh` étendu et vert.
- `docs/security/THREAT_MODEL.md vNN` (confronté §19 pack) ; `docs/rust/PATTERNS.md` §P76+ +
  **note d'en-tête espace §P commun + 7 doublons (C-1) + §P75 pt 6 oracle ±0 (C-3)** ;
  `docs/shell/PATTERNS.md` note d'en-tête symétrique (C-1).
- `sprint83_t2_acceptance.json` finalisé (T6 mis à jour) ; `sprint83_verification.md` ;
  `sprint84_audit_plan.md` (Tracks A..K + Track J testabilité + Track K docs-contrat + CI).

### AJ1.3 Tests plan
- `bash scripts/check-sharding-docs.sh` + `check-frontier-contracts.sh` + `check-factory-docs.sh`
  exit 0 (anchors ROW/ENTRY-level sur les frontières neuves — mutation-test discriminant,
  [[gate-machine-discriminant]]).
- `test -f` sur tous les artefacts documentaires requis (GUIDE, llms.txt, WIRING_SPEC,
  verification.md, sprint84_audit_plan.md, t2_acceptance.json).
- Suites complètes (2 blocs pré-commit + 3 blocs Docker) vertes une dernière fois.

### AJ1.4 Critère d'acceptation
- Les 3 gates docs exit 0 avec anchors discriminants sur les frontières neuves ;
  `sprint83_verification.md` §Acceptance au vocabulaire fermé ; `sprint84_audit_plan.md` porte
  Track J + Track K + CI + les re-routes P3.
- **Condition binaire** : docs-contrat fermée (GUIDE+llms.txt+WIRING pour chaque frontière) +
  C-1/C-3 traités + T2 finalisé AVANT verification.md + audit_plan S84 écrit + go/no-go pilote.
- T2 : finalisation `sprint83_t2_acceptance.json` (palier T6) — top-level `PASS` si tous les
  paliers `ACTED`/`RIG-ABSENT` justifiés (jamais `BLOCK` non diagnostiqué).

### AJ1.5 Commit cible
`docs(sprint83): Sprint 83 Phase AJ1 — clôture docs-contrat + verification + audit_plan S84 + wrap-up`.
Body 9 sections : Contexte (fermabilité DoD §4 ; AM-1/AM-3 ; C-1/C-3 ; Track J+K standing) ·
Fichiers (GUIDE, llms.txt, WIRING, THREAT vNN, PATTERNS §P76+, verification, audit_plan S84,
t2 final) · Delta tests (0 code ; gates docs discriminants) · Verification §7.4 · Scope cuts
(les 14, respectés) · G8 traceability · Pre-launch protocol (0 wire au wrap-up) · Codex
verification · Carry closure (ferme G-SCOPE-FLOOR au sens AM-5 ; Track K fermable ; G6/G7 ;
attention 3/3 S84 §6.4).

---

## Delta tests estimé (par familles — PAS de LOC, §6.7)

Ordres de grandeur (SORTIES attendues, jamais des engagements — un compte réel différent au
wrap-up n'est pas un dérapage, AM-7). Baseline d'entrée : Rust **2108** Win / **2112** Docker ;
Vitest web **412** + E2E **44/2skip** ; operator **201** + **10** E2E.

| Phase(s) | Rust | Vitest (web/Hub) | E2E | Détail |
|---|---|---|---|---|
| A / A1 | +6..12 | 0 | 0 | grammaire phase + lot carries (goldens, isolation, hook Check 5) |
| B | +2..5 | +8..15 | 0 | harness a11y/i18n/RTL, préférences |
| C | +6..10 | +3..6 | +1 spec | intersection 7 couches, grant v0, bridge spec |
| D / E | +10..18 | 0 | 0 | FactoryProcessProfile.v1 + spike durable (journal/outbox atomiques) |
| F / G / H | +18..30 | 0 | 0 | PackV1 + corpus hostile + GraphV1/18 nodes + compilateur + 4 goldens fixtures |
| I / J / K | +18..30 | 0 | 0 | run store SQLite + orchestrateur + effets/idempotence |
| L / L1 / M | +12..20 | 0 | 0 | permissions/atténuation + coffre/HITL + artefacts/ACL/provenance |
| N / O / P / P1 | +14..24 | 0 | 0 | adaptateurs modèles/agents + wire compute + MCP borné |
| Q | +4..8 | +2..4 | 0 | hôte de confiance + companion IPC + CSP Hub |
| R..V | +10..18 | +40..70 | **+3 specs T1** | surfaces Hub (accueil, registre, canvas, inspecteurs, runs) |
| W / W1 | +4..8 | +6..12 | 0 | evals + panel round 1 (rework a11y) |
| X / Y | +12..20 | +8..15 | +0 (specs étendues) | lifecycle/RRV + inter-apps |
| Z..AC | +20..35 | 0 | 0 | Factory dogfood (golden 1/4, e2e interne) |
| AD / AE / AF | +18..30 | +4..8 | 0 | packs Ideas/Conception/Babel (≥100 chunks Babel) |
| AG / AH | +12..22 | +3..6 | 0 | assistance IA/import + fuzz/supply-chain |
| AI / AI1 / AI2 | +12..22 | +6..12 | 0 | fault/scale/perf + a11y multi-AT + acceptance agrégée |
| AJ / AJ1 | +6..12 | 0 | 0 | shadow/cutover + clôture docs (0 code) |
| **Total (ordre de grandeur)** | **+200..320** | **+100..180** | **+3..8 specs** | `sbfb-workflow-core`/`-engine` majoritaires ; Hub Vitest + 3 specs T1 |

Note : le delta réel est une conséquence du travail. Le critère de non-régression est **jamais
en baisse** sur les baselines d'entrée (2108/2112/412/201) — un split behavior-preserving reste
±0 EXACT (oracle §P75 pt 6, C-3).

---

## Fail-fast checklist (critère SMART du goal §2 — colonne *Observed* remplie au verification.md)

| # | Check | Commande | Critère |
|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warning |
| 3 | cargo nextest (Win) | `cargo nextest run --workspace --locked` | ≥ 2108, 0 régression |
| 4 | doctests | `cargo test --workspace --locked --doc` | ok |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok |
| 6 | nextest (Docker sbfb-ci) | `docker run … cargo nextest run --workspace --locked` | ≥ 2112 (flakes solo-PASS) |
| 7 | web lint | `(cd web && npm run lint)` | 0 |
| 8 | web tsc | `(cd web && npx tsc --noEmit -p tsconfig.app.json)` | 0 |
| 9 | web vitest | `(cd web && npm run test:unit)` | ≥ 412, 0 régression |
| 10 | web coverage | `(cd web && npm run test:coverage)` | seuils tenus |
| 11 | web build | `(cd web && npm run build)` | ok |
| 12 | web size-limit | `(cd web && npm run size)` | budgets verts (Hub inclus) |
| 13 | scan-en-strings | `bash web/scripts/scan-en-strings.sh` | 0 hit |
| 14 | operator vitest | `(cd tools/factory-operator && npm run test:unit)` | ≥ 201 |
| 15 | operator tsc+lint | `(cd tools/factory-operator && npx tsc -b && npm run lint)` | 0 |
| 16 | **T1 — E2E Hub author** | `(cd web && npm run test:e2e -- workflow-hub-author)` | **GREEN (BLOQUANT)** |
| 17 | **T1 — E2E Hub run** | `(cd web && npm run test:e2e -- workflow-hub-run)` | **GREEN (BLOQUANT)** |
| 18 | **T1 — E2E Hub lifecycle** | `(cd web && npm run test:e2e -- workflow-hub-lifecycle)` | **GREEN (BLOQUANT)** |
| 19 | T1 — bridge grant intersection | `(cd web && npm run test:e2e -- bridge-grant-intersection)` | GREEN |
| 20 | E2E web (régression) | `(cd web && npm run test:e2e)` | ≥ 44 pass / 2 skip |
| 21 | operator E2E | `(cd tools/factory-operator && npm run test:e2e)` | 10 pass |
| 22 | gate docs sharding | `bash scripts/check-sharding-docs.sh` | exit 0 (anchors discriminants) |
| 23 | gate frontier contracts | `bash scripts/check-frontier-contracts.sh` | exit 0 (frontières neuves indexées) |
| 24 | gate factory docs | `bash scripts/check-factory-docs.sh` | exit 0 |
| 25 | cargo deny | `cargo deny check advisories licenses` | vert (clôt CI-1) |
| 26 | spike durable E (GO/NO-GO) | `cargo nextest run -p sbfb-workflow-engine --locked` | GO (suite COMPLETE : un filtre etroit ne selectionnerait que le validateur de forme et laisserait passer la suppression d'une assertion comportementale) |
| 27 | invariant grader LLM ≠ PASS | `cargo nextest … -E 'test(llm_grader_cannot_grant_pass)'` | vert |
| 28 | invariant driver ≠ self-PASS | `cargo nextest … -E 'test(driver_cannot_self_pass)'` | vert |
| 29 | Factory golden e2e | `cargo nextest … -E 'test(factory_golden_end_to_end)'` | vert |
| 30 | app-authoring lineage | `cargo nextest … -E 'test(appauthoring_lineage)'` + `test -f sprint83_r_appauthoring_evidence.json` | vert + digest+lineage |
| 31 | **T2 acceptance JSON** | `python -c "import json;d=json.load(open('.planning/active/sprint83_t2_acceptance.json'));assert d['status'] in ('PASS','BLOCK','RIG-ABSENT','N-A-no-cross-machine-feature')"` | parse + vocabulaire fermé |
| 32 | artefacts docs-contrat | `test -f docs/factory/llms.txt && test -f docs/sharding/WIRING_SPEC.md` (+ GUIDE frontières neuves) | présents |
| 33 | artefacts wrap-up | `test -f .planning/active/sprint83_verification.md && test -f .planning/active/sprint84_audit_plan.md` | présents |
| 34 | PATTERNS §P76+ + note §P commune (C-1) | `grep -q 'espace.*COMMUN' docs/rust/PATTERNS.md docs/shell/PATTERNS.md` | note présente 2 fichiers |

---

## Scope cuts (reprise exhaustive — kickoff §7, 14 items)

1. Scheduler DAG distribué / failover multi-leader — HORS MVP (§13.4).
2. Réplication live P2P du run state privé — HORS MVP (journal local).
3. Co-édition temps réel du canvas — HORS MVP (brouillons+fork/versions).
4. Node code-execute / plugins tiers / code importé — HORS MVP (wasmtime écarté, 0 dep).
5. Marketplace centrale / ranking global / token governance — JAMAIS (anti-capture, kudos non-monnaie).
6. Auto-update des packs — HORS MVP (activation = re-consentement, update élargissant BLOQUÉ).
7. A2A adapter — FUTUR (WA-10).
8. `DelegationGrant.v1` cross-nœud — FUTUR (§14.5).
9. Cancel fort d'une Task réseau — HORS MVP (cancel honnête local, limite affichée).
10. Envelope chiffrée recipient-scoped — plancher public-only (Q6) ; envelope FUTURE sauf décision PO.
11. Import exécutable total Flowise/n8n/LangGraph — HORS MVP (data-only, `$flow.state` jamais mappé).
12. Mémoire personnelle P2P + optimisation auto de prompts — HORS MVP (§22.4).
13. SBFB comme serveur MCP élargi — HORS chantier (`mcp_server_expose` gate-off ; MCP = veille).
14. Dépréciation effective du chemin Factory legacy (`authority=new`) — POST-S83 (AM-5) ;
    S83 livre la capacité de bascule prouvée ; G-SCOPE-FLOOR ferme en AJ1 au sens de ce cut.

---

## Risks (reprise — kickoff §9)

| R | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Dépendances externes du préalable (panel 2 rounds, licences/machines AT 5 OS, provider cloud, 2 workers) | Haute | Haut | AM-10 (réservation=bloquant activation ; consommation=checkpoints B/pré-W1/pré-R/pré-AI1) ; AM-9 ; owner+fallback+exit ; retour PO jamais carry masqué |
| R2 | Densité de phase sans précédent (41 phases, record S82=24) | Haute | Moyen | AM-4 pré-coupes (AI×3, AJ×2, P, L) + AM-7 (re-coupes nominales) ; process uniforme ; honnêteté de compte |
| R3 | UI back-loadée (R..V pos 20+) + dep canvas à re-mesurer (@xyflow/react 12.11.2 ABSENT, faits blueprint SUPERSEDED) | Moyenne | Moyen | contract-first ; compagnon Hub normatif ; re-mesure xyflow à l'activation de T (QPO-5) ; risque ACCEPTÉ |
| R4 | Événements calendaires mid-sprint (gate C8 01/08 + topologie A-vs-B 25/08 + spec MCP 2026-07-28) | Certaine | Moyen | hors-sprint PO (Q11) ; revalidation PKARR_RELAY_OPS ; MCP veille gate-off |
| R5 | Surface TCB nouvelle (runtime hôte + routes loopback + companion IPC + coffre) sous R-iroh-audit P0 INCHANGÉ | Moyenne | Haut | inventaire D7 route par route + tiers + auth ; THREAT vNN vs §19 en G8/G9 ; aucune promesse production publique |
| R6 | Fenêtre de crash effet/journal (LE cas dur durable-execution) | Moyenne | Haut | Spike Phase E GO/NO-GO BLOQUANT avant tout gel de format ; idempotence sur tous les effets ; fail-closed replay ; `outcome_unknown` jamais succès |
| R7 | Enforcement des 44+ gates + labels bi-lettres jamais exercés | Moyenne | Moyen | Phase A prouve la grammaire + matrice AM-2 complète (mécanisme par gate) ; T1 câblé CI dès R ; G-SCOPE-FLOOR/G-VERTICALS ré-assignés |

---

## Checkpoint de clôture (conditions binaires pour dire « S83 fermé »)

- [ ] Toutes les rows fail-fast (1..34) **vertes** au `sprint83_verification.md` (§Observed rempli).
- [ ] **3 specs T1 Hub** (`workflow-hub-author/run/lifecycle`) GREEN + job CI « Workflow Hub E2E » BLOQUANT vert chaque push (rows 16-18).
- [ ] **T2** : `sprint83_t2_acceptance.json` finalisé (AJ1), parse, vocabulaire fermé, top-level `PASS` (paliers `ACTED`/`RIG-ABSENT` justifiés) ; WA-T6 finalisé avant `verification.md`.
- [ ] **Condition 1 audit S82** : app-authoring S79 P1 STANDING **FERMÉ** en Phase R (evidence JSON + digest blake3 + lineage commit Hub) OU requalification PO écrite (Q8).
- [ ] **Condition 2** : tous les carries §6.2 fermés/routés (A1 exécuté ; C-1/C-3 en AJ1 ; lot P3 tracé `sprint84_audit_plan.md`).
- [ ] **Condition 3** : chore d'archivage joué (CLAUDE.md réconcilié, itemisation delta E-2, registre LT, SPRINT_LOG v2.2 si Q2, memory tip).
- [ ] **G-VERTICALS 4/4** (Factory livré Z..AC + Ideas AD + Conception AE + Babel AF) ; **G-SCOPE-FLOOR fermé AJ1** (au sens AM-5).
- [ ] **Capacité de bascule Factory prouvée** en shadow (AJ : oracle différentiel vert, kill switch/rollback) — dépréciation legacy réelle = décision POST-S83 (cut 14).
- [ ] **Clôture docs-contrat** : GUIDE + `llms.txt` + `WIRING_SPEC` indexent toutes les frontières neuves ; 3 gates docs exit 0 avec anchors discriminants ; THREAT vNN + PATTERNS §P76+ + note §P commune (C-1) + §P75 pt 6 (C-3).
- [ ] **~46 commits** (Phase 0 + A..AJ1 avec pré-coupes + re-coupes suffixées éventuelles) + 1 chore kickoff + 1 docs wrap-up ; chacun body 9 sections + trailer Co-Authored-By (dès le 1er, dérive I-1).
- [ ] `sprint84_audit_plan.md` écrit (Tracks A..K + **Track J testabilité** + **Track K docs-contrat** + Track CI) + re-routes P3.
- [ ] Memory `nexus_grid_pivot.md` + `MEMORY.md` à jour ; SPRINT_LOG row 83 ; PATTERNS mis à jour.
- [ ] **0 barrière a11y P0/P1** ouverte (W1 + AI1) ; **0 P0/P1 en carry de confort** (AI2).
