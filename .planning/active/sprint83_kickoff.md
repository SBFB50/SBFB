# Sprint 83 — Kickoff : chantier « Workflow Agents » (Cas C)

Date : 2026-07-17. **Sprint 83 IMPAIR** (pas de phase dette obligatoire,
Regle 1 §6.2.1 — la Phase A1 est un lot VOLONTAIRE impose par la condition 2
de l'audit). Tip master d'entree : **`74531d4`** (`chore(planning): Sprint 82
audit findings — verdict CONDITIONAL PASS`). **Phase 0 = audit gate S82 :
DEJA JOUE** — CONDITIONAL PASS `74531d4` (0 P0, 0 P1 frais, 5 P2, 20 P3),
3 conditions acquittees DANS ce kickoff (§3, §6). Version d'archive :
**v2.2 recommandee** (theme structurellement nouveau §7.3 ; question PO Q2 —
les artefacts S82 s'archivent en v2.1 quoi qu'il arrive).

Theme : chantier **« Workflow Agents »** — app visible **« Workflow Hub »**
(app SBFB sandboxee de premier rang) + **Workflow Runtime Rust hote** (seule
autorite d'execution) + **Prompt/Agent Packs portables signes** + integration
protocole. 1er slot trace roadmap v5 bloc S82. La ratification formelle du
slot reste une question du Checkpoint §11 (Q1).

Autorite documentaire : `docs/claude/README.md` + `docs/agent/AGENT_SYSTEM.md`
+ `docs/agent/PROCESS.md` (process) >
`.planning/research/workflow_agents_app_conception_ultradeep_2026-07-15.md`
(architecture/securite/runtime/protocole/sequencement ; plan maj 2026-07-17) >
`.planning/research/workflow_hub_product_conception_2026-07-15.md` (UX/produit
Hub seulement). `.planning/research/sprint82_workflow_engine/` = **SUPERSEDED**
comme base d'architecture — ses faits sont reutilises UNIQUEMENT via la table
§9 de `revalidation_code.md` (9.1 tels quels, 9.2 apres re-mesure, 9.3
interdits). Decision PO 2026-07-17 (noms) : « Workflow Hub » = app visible,
« Workflow Agents » = chantier ; le kickoff ratifie les noms techniques de
crates/schemas mais ne recree PAS de plan Hub autonome — le plan candidat est
le chapitre 24 du pack global (36 phases A..AJ), verifie independamment et
adopte AVEC AMENDEMENTS (D5).

---

## Sources consultees (pre-gel)

**Artefacts fondations (scratchpad `kickoff_s83/`, tous 2026-07-17)** :
`etat_projet.md` (inventaire Step 1, findings S82 764 l lus en entier),
`research_pack_extract.md` (pack global 3119 l lu en entier),
`hub_pack_extract.md` (compagnon Hub 905 l), `revalidation_code.md` (7 claims
§5 + 11 faits blueprint revalides au HEAD `74531d4`, Grep/Read directs),
`verif_plan36.md` (verification independante du plan ch.24 — verdict ADOPTER
AVEC AMENDEMENTS, AM-1..AM-12), `oss_runtime.md`, `oss_workflow_ux.md`,
`g2_g7.md` (triggers G2 + registre LT G7 + deps theme).

**Repo (HEAD `74531d4`, verifie ligne a ligne)** : `csp.rs:33` (nexus-core-rs),
`protocol.ts:20-49` + `useBridge.ts:111-176`, `sbfb-manifest/src/lib.rs:34-67+`,
`provider_router.rs:62-113` + `operator_server.rs:37` (sbfb-factory),
`canonical.rs:77-332` (23 DOMAIN_*_V1) + `task.rs:152-202/:416/:463/:541`
(nexus-core-rs), `provenance.rs:17-29,102-124` (nexus-coordinator-rs),
`deploy.rs:355-376` + `storage_api.rs:27-33,293-340` + `http.rs` (1513 l, 21
modules `*_api.rs`, mesure `wc -l` au tip) (nexus-shell-daemon),
`storage_limiter.rs:12` (nexus-shell-daemon-core), `seed_registry.rs:331`,
`agentctl.py:28/:32/:566`, hooks lightcheck `:88` / auditor-gate `:54`,
`THREAT_MODEL.md:186`, `sprint82_audit_findings.md:674-678` + Carry-Over
l.693-749.

**crates.io API (constate 2026-07-17)** : iroh 1.0.2 (pin =1.0.1 conserve) ;
iroh-docs 0.101.0 ; ed25519-dalek **3.0.0 stable (2026-07-06)** — iroh
1.0.1/1.0.2 pinnent toujours =3.0.0-rc.0 ; frost-ed25519 3.0.0 ;
hickory-resolver 0.26.1 ; rusqlite 0.40.1 (lock 0.36.0) ; schemars 1.2.1 ;
axum 0.8.9 ; tokio 1.53.0 ; arti-client 0.44.0 (2026-06-30, lock 0.41.0) ;
pkarr-relay 1.0.0 + pkarr 7.0.0 ; sigstore 0.14.0 « An experimental crate »
(2026-05-22) ; Obelisk 0.40.0 « Deterministic workflow engine » (2026-07-13) ;
**temporalio-sdk 0.5.0 (2026-06-29), 5 releases en 4 mois depuis 0.1.0-alpha.1
(2026-02-19)** — citation corrigee G1 ; cqrs-es/sqlite-es 0.5.0 (2025-12-30).
`cargo deny --locked check advisories` local (cargo-deny 0.19.2, DB
2026-07-17) = **ok, 0 RUSTSEC actif**.

**npm registry (2026-07-17)** : react 19.2.7 ; @xyflow/react 12.11.2 (ABSENT
du repo — dep APP-SIDE candidate, a re-mesurer a l'activation, QPO-5).

**Specs / etat de l'art (dates)** : MCP revision 2025-11-25 + spec finale
2026-07-28 (blog.modelcontextprotocol.io — mid-S83, veille) ; A2A v1.0.0
(2026-03-12) ; Microsoft Agent Framework 1.0 (2026-04-03) ; OpenAI Agents SDK
handoff/agent-as-tool (2025-03-11) ; LangGraph 1.0 GA (2025-10-22) ; CaMeL
arXiv:2503.18813 (v2 2025-06-24) + « Design Patterns for Securing LLM
Agents » arXiv:2506.08837 (2025-06-10) ; tool poisoning MCP (5,5 % de 1899
serveurs ; MCPTox >60 % succes) ; PoC Morling durable-execution 2025-11-20 ;
DBOS MIT / Restate BSL 1.1 / Inngest SSPL (non-OSI) ; Dify DSL versionne ;
n8n (contre-exemple schema non versionne) ; Flowise 3.1.3 ; promptfoo
post-acquisition OpenAI (2026-03-09).

**CVE / RUSTSEC (dates)** : CVE-2026-54786 + CVE-2026-34945/RUSTSEC-2026-0086
(wasmtime 2026 — decision anti-wasmtime RENFORCEE, 0 dep au lock) ;
CVE-2025-6965 (SQLite bundle 3.49.2, exploitabilite contextuelle basse) ;
CVE-2025-67644 SQLi + CVE-2026-28277 msgpack→RCE (chaine checkpointer
LangGraph, Check Point Research) ; CVE-2026-40933 (reactflow — constat
2026-07-02 SUPERSEDED, a re-mesurer).

---

## 1. Constat d'entree

### 1.1 Git / arbre

- Tip master : `74531d4` (commit Cas A local — audit gate S82). Le tip POUSSE
  (origin == codeberg) au gate etait `aa9db47` ; memory `nexus_grid_pivot.md`
  stale d'1 commit — rattrapage au G6 (§10).
- Arbre de travail : exactement **3 entrees hors-phase PO** (lisibles, jamais
  editables par agents ; staging = Q12) : `M verification_blueprint.md`
  (banniere SUPERSEDED) + 2 untracked `workflow_*.md` (packs 3119 l + 905 l).

### 1.2 Compteurs tests (verifies audit Track A, tous EXACTS)

| Suite | Count |
|---|---|
| Rust nextest Windows natif | **2108** (0 skip) |
| Rust nextest Docker `sbfb-ci` | **2112** (+4 `#[cfg(unix)]`) |
| Vitest web | **412** |
| E2E web Playwright hermetique | **44 passed / 2 skipped** (env-gated @shard/@compute) |
| Vitest operator | **201** (35 fichiers) |
| E2E operator | **10/10** |
| size-limit | 6/6 (css 129.02/130 kB) ; coverage web 87.27/79.01/86.02/88.59 |
| fmt / clippy / doctests / release / gates docs ×3 | verts Win + Docker, exit 0 |
| `http.rs` | **1513 l / 89 routes / 21 modules `*_api.rs`** (mesure au tip) |

### 1.3 CI distante (post push-gate PO-4=C, 2026-07-17)

Verts : rust-ci fmt/clippy/windows/ubuntu (1er ubuntu vert depuis mai, fix GTK
prouve) ; ci.yml « Full verification » SUCCESS ; integration-nightly SUCCESS
junit 10/0/0 (S81-J-2 CLOSED) ; Woodpecker #38 steps lourds. Rouges connus :
macos-14 watcher (10 tests, arbitrage PO non-bloquant, classe routee S83) ;
supply-chain (licences Unlicense/MIT-0 + 3 legs pip-audit zombies — CI-1) ;
leg operator Node 20 vs 22 (CI-2) ; shellcheck chronique (CI-3) ; spdx
Woodpecker glitch (G-3/CI-4) ; deploy.yml startup-failure 0s ; canary-monthly ;
Build worker binaries ; Mirror Codeberg auth CASSEE (action manuelle PO).

### 1.4 Ancrage HARDENING_ROADMAP + G2/G7 + zones rouges

- `docs/security/HARDENING_ROADMAP.md` : re-arme S81 G, MAJ S82 K ; triggers
  evalues bloc par bloc au 2026-07-17 (`g2_g7.md` §1). ARMES : arti-client
  0.44 (bump couple rusqlite = slot maintenance HORS chantier, QPO-3) ;
  pkarr-relay 1.0.0 (revalidation RUNBOOK avant provisioning C8) ; ARMES-SOON
  calendaires : gate C8 01/08 + spec MCP finale 2026-07-28 (mid-S83, Q11/Q14).
  Fait nouveau : dalek 3.0.0 stable — convergence bloquee upstream (iroh pinne
  rc). Triggers INTERNES qui seront armes PAR le theme : nouvelle route
  loopback (inventaire D7 obligatoire) + nouveau checker guardrail (pattern
  chaine S73 A / S75 CARRY-2).
- Registre LT (G7) : aucun trigger nouvellement franchi ; LT-2 seul ARME
  (decision PO hors-sprint, Q15) ; entrees STALE LT-1/LT-5/LT-7 repliees au
  chore (QPO-2, §6.3).
- Zones rouges INCHANGEES : R-iroh-audit P0 (pilote ferme — l'upgrade iroh
  1.0.1 ne la leve pas), R-wasmtime-cve P0 (RENFORCEE 2026), R-libcrux-hax P2,
  R-pyodide-escape. Le Runtime hote + companion IPC creent des surfaces TCB
  nouvelles → THREAT_MODEL vNN confronte au §19 du pack (G8/G9 par phase).

---

## 2. Goal du sprint (critere SMART)

Livrer le chantier Workflow Agents COMPLET en pilote ferme : Hub (app SBFB
sandboxee plein ecran, plan de controle et interface auteur), Runtime Rust
hote (seule autorite d'execution durable), packs portables signes
(`SBFBPromptAgentPack.v1`), integration protocole (publication, decouverte,
feuilles bornees), 4 verticaux (Factory fondateur + Ideas + Conception +
Babel), accessibilite maximale au MVP (WCAG 2.2 AA bloquant + AAA evalue +
ATAG + COGA + EN 301 549 figee — Q6). MVP = premiere version produit COMPLETE
utilisable (decision PO 2026-07-15 §22.1, gelee) ; 11 piliers §22.2 ; scope
floor §22.3 ; acceptances WA-T1..T6 + HUB-T1..T10.

**Critere SMART** : N rows fail-fast VERTES au `sprint83_verification.md`
§Fail-fast checklist (checklist par groupe de phases — 44+ gates depassent le
patron 24-32 rows), mesure binaire au wrap-up AJ1. Les rows incluent
OBLIGATOIREMENT : « T1 GREEN (`npm run test:e2e`, specs Hub nommees §5) » et
« `sprint83_t2_acceptance.json` existe + parse + status dans le vocabulaire
ferme PASS/BLOCK{diagnosis}/RIG-ABSENT » (exigence portee au plan.md §5 —
AM-1 amende).

Statut : pilote ferme tant que les preuves de production publique manquent.

---

## 3. Phase 0 — audit gate S82 (DEJA JOUE)

Verdict **CONDITIONAL PASS** `74531d4` : 0 P0, 0 P1 frais (1 P1 STANDING
herite : app-authoring S79), 5 P2, 20 P3 ; 24/24 findings confirmes, 0 refute.
Aucun fix commit requis. Les 3 conditions s'acquittent dans CE kickoff :
1. **Decision fermante app-authoring S79** (NON-NEGOCIABLE) → §6.1, Q8.
2. **Routage des carries** de la section Carry-Over du findings → §6.2
   (table de routage — chaque item du Carry-Over l.693-749 a une route ou un
   re-route date ; le lot « P3 divers S83+ » l.742-749 inclus).
3. **Chore d'archivage** : reconciliation CLAUDE.md post-push-gate + delta
   E-2 + registre LT + SPRINT_LOG → §6.3.

---

## 4. Decisions Day-0 gelees — D1..D5

### 4.1 D1 — Architecture d'autorite : 4 couches ; Hub sandboxe / Runtime hote / pack signe / protocole ; placement = hote daemon + companion Factory privilegie

**Statut** : ratifiee PO sur le fond (pack §1-2, correction de cadrage
2026-07-15 ; noms 2026-07-17) — le kickoff la GELE avec preuve de revalidation
code et tranche le placement (laisse ouvert par §21.3).

**Sources (datees)** : Research Pack global §1/§2.2-2.4 (2026-07-15, plan maj
2026-07-17) — 4 couches, « app iframe qui execute tout » REFUSE, « route
privilegiee Operator » REFUSE comme produit principal, invariant « un LLM ne
modifie jamais le graphe compile, n'accorde une capacite, ne change une
destination, ne produit un verdict process autoritaire ». `revalidation_code.md`
(2026-07-17, HEAD `74531d4`) : §5.1-5.7 TOUS CONFIRMES — 0 runtime workflow
(13 crates, 0 `sbfb-workflow-*`), bridge 16 verbes SANS intersection grants
(`web/src/bridge/useBridge.ts:111-176`, `crates/sbfb-manifest/src/lib.rs:52-66`
« declarative, NOT the runtime dispatch boundary »), CSP sandbox compatible
(`crates/nexus-core-rs/src/csp.rs:33`), Factory = adaptateurs pas moteur
(`provider_router.rs:62-113`, 0 DB). WA-0/WA-2/WA-3/WA-19 (pack §26.2).
`g2_g7.md` §1.2 : trigger LOOPBACK interne — toute route Runtime entre dans
l'inventaire D7 `inventory_policy` (89 routes S82 T) + tiers §2.1/§8.1 + auth
complete. Decision gelee v4 D2 « Factory hors daemon » (CLAUDE.md).

**Retenu** :
1. **4 couches** : (i) Workflow Hub = app SBFB sandboxee plein ecran —
   pre-valide/edite/affiche, ne possede JAMAIS credential provider, cle
   privee, base autoritaire des runs, execution privilegiee, scheduler
   distribue, appel iframe-vers-iframe ; ne fabrique jamais un succes local.
   (ii) Shell hote de confiance : grants par app/release/pack, HITL dans le
   CHROME hote lie au digest exact, secrets, quotas. (iii) Workflow Runtime
   Rust hote = SEULE autorite : compilateur, reducer, scheduler local, event
   store, inbox/outbox, checkpoints, reprise, annulation honnete, provenance,
   evaluation. (iv) Protocole SBFB : distribution content-addressed + feuilles
   Task/Result signees bornees. Autorite du run = noeud initiateur (WA-3).
2. **Placement (tranche ici, ratification Q3)** : runtime generique
   (`sbfb-workflow-engine`) heberge par l'HOTE daemon — routes loopback
   versees a l'inventaire D7 + tiers + auth complete (bearer + Host + Origin +
   peer creds UDS/NP) ; bridge commande/evenement type vers l'iframe via le
   shell. Le `FactoryAdapter` privilegie (repo/git/shell/PTY/Claude
   Code/Codex) vit dans un **companion local separe** (extension
   sbfb-factory), IPC authentifie capability-scoped, hors TCB reseau (WA-19)
   — le daemon ne depend pas de la Factory (v4 D2 honoree).
3. **Invariant de controle gele** : le graphe deterministe controle le
   processus ; les agents raisonnent dans des ilots bornes (repris comme
   invariant testable en D4).

**Alternatives REJETEES** :
- *App iframe qui execute tout* — REJETE : CSP sandbox revalidee
  (`BLOB_SERVE_CSP` `crates/nexus-core-rs/src/csp.rs:33` : `connect-src 'none'`,
  `sandbox allow-scripts` sans `allow-same-origin`) interdit acces
  reseau/secret depuis l'iframe ; un runtime iframe meurt avec l'onglet
  (contraire §12.6). Refus explicite pack §2.3.
- *Workflow Studio privilegie dans l'Operator* — REJETE : correction de cadrage
  §1 du pack ; l'Operator est un outil local privilegie du noeud, pas une app
  publiable/forkable du reseau — le Hub doit etre de premier rang (WA-0).
- *Runtime generique ENTIER dans un companion hors-daemon* — REJETE : le daemon
  est deja l'autorite d'hebergement (blob-serve, bridge → routes loopback,
  storage_api, singleton strict) ; un second service autoritaire dupliquerait
  auth/singleton/inventaire D7 et creerait 2 TCB reseau. Le companion reste
  reserve au perimetre PRIVILEGIE Factory (WA-19).

**Implications code (fichiers reels revalides)** :
- `crates/nexus-shell-daemon/src/http.rs` (**1513 l au tip `74531d4`**,
  21 modules `*_api.rs` — mesure `wc -l`, corrige G1) : nouveau module
  `workflow_api.rs` (0 collision au HEAD), route par route versee a
  l'inventaire D7 (`inventory_policy`).
- `web/src/bridge/protocol.ts:20-49` + `useBridge.ts:111-176` : extension
  bridge APRES enforcement intersection (Phase C, cf. D4).
- `crates/sbfb-manifest/src/lib.rs:34-66` : `bridge.methods` compose avec les
  grants runtime (intersection cote host).
- `crates/sbfb-factory/` : companion IPC (`operator_server.rs` = precedent
  d'auth locale) ; `provider_router.rs:62-113` reutilise par le ModelAdapter.
- Nouveaux crates : `crates/sbfb-workflow-core` + `crates/sbfb-workflow-engine`
  (0 collision, revalide §5.1).

### 4.2 D2 — Pack portable + signature : `SBFBPromptAgentPack.v1` maison (JCS + Ed25519 + DOMAIN dedies) avec `subject[]` blake3 a la in-toto ; DSSE differe

**Sources (datees)** : Research Pack §9 (2026-07-15) — racine unique
`SBFBPromptAgentPack.v1`, layout §9.2, 6 digests §9.3, PACK.lock §9.4, 8 roles
de signature §17.1. `oss_runtime.md` §0/§3 (2026-07-17) : l'enveloppe existe
DEJA — `serde_jcs 0.2` (Cargo.toml:65), 23 constantes `DOMAIN_*_V1`
(`crates/nexus-core-rs/src/canonical.rs:77-332`), ed25519-dalek 2.1, provenance
SLSA L1 strip/inject (`crates/nexus-shell-daemon/src/deploy.rs:355-376`,
`finalize_deploy` — chemin de crate precise, corrige G1), `schemars 1.2` draft
2020-12 (pattern DOMAIN 25 frozen S82 G), blake3. DSSE = PAE, refuse la
canonicalisation ; SLSA v1.2 courante (« L1 auto-atteste » exact) ; Sigstore =
services centraux, crate 0.14.0 « experimental » (2026-05-22) ; OCI Referrers :
concept `subject` a emprunter, registres hors-sujet. `oss_workflow_ux.md` §1
(2026-07-17) : Dify DSL versionne ; n8n = contre-exemple (schema non versionne,
noms de credentials qui fuient). Pre-launch protocol policy (CLAUDE.md).

**Retenu** :
1. **Format maison etendu** : racine `SBFBPromptAgentPack.v1` (WA-1) —
   `workflow/workflow.json` (`SBFBWorkflowGraph.v1`), `agents/*.json`
   (`SBFBAgentDefinition.v1`), prompts, schemas pinnes, capabilities DEMANDEES
   (jamais accordees), evals, provenance, `layout/` HORS digest semantique.
   Versionne des la v1 (lecon n8n), librement editable pre-launch.
2. **Signature** : JCS RFC 8785 (`serde_jcs`, byte-identique cross-langage,
   fixtures golden Rust/TS) + Ed25519 + constantes dediees
   (`DOMAIN_WORKFLOW_PACK_V1`, `DOMAIN_COMPILED_PLAN_V1`,
   `DOMAIN_RUN_EVENT_V1`, `DOMAIN_PACK_INSTALL_RECEIPT_V1`, …) — un domaine
   par role §17.1, append au pattern 23 DOMAIN.
3. **Emprunt in-toto minimal** : provenance du pack porte
   `subject[] = [{name, digest: {blake3}}]` — cout quasi nul, DSSE atteignable
   plus tard SANS casse si l'interop cosign devient un objectif (Q4).
4. **6 digests separes** (§9.3) : content / graph_semantic (fondement de la
   parite canvas/outline/guide) / presentation / compiled_plan / evaluation /
   publication. `PACK.lock` pinne tout par digest (anti rug-pull, cf. D4).
5. **Interdits durs** : secret/token, endpoint prive credentialise, code
   executable, grant accorde, etat live, memoire personnelle — refus
   compilateur (§10.5), etendu aux REFERENCES nominatives de credentials
   (lecon n8n) : seul `secret_ref` logique opaque.
6. **Distribution** : zip universel + iroh-blobs + feed ; nouvelle op
   `prompt_agent_pack` via raw-op — **0 bump `FEED_FORMAT_VERSION`** ;
   `artifact_kind` introduit cote index (0 occurrence au HEAD).

**Alternatives REJETEES** :
- *DSSE/PAE + Statement in-toto complet des le MVP* — REJETE (pour S83) :
  interop cosign non ratifiee comme objectif produit ; DSSE = DEUXIEME
  mecanisme de domaine coexistant avec les 23 constantes, dette de coherence
  sans consommateur. Le `subject[]` preserve la voie (`oss_runtime.md` §3.5 B).
- *Pile Sigstore (Fulcio/Rekor/cosign)* — REJETE : identites ephemeres + log de
  transparence = SERVICES CENTRAUX, contraire a « no central server » ; crate
  0.14.0 auto-decrit « experimental » (crates.io 2026-05-22).
- *OCI artifacts + registres ORAS* — REJETE : le reseau SBFB EST deja un
  registre content-addressed (blake3 + iroh-blobs).
- *Format tiers autoritaire (Flowise/n8n/Dify)* — REJETE : aucun signe ni borne
  statiquement ; n8n non versionne + fuite de credentials ; Flowise
  `$flow.state` global mutable incompatible §11 ; Flowise = cible d'IMPORT
  data-only (WA-12), jamais le format.

**Implications code (chemins de crate complets — corrige G1)** :
- `crates/nexus-core-rs/src/canonical.rs:77-332` : nouvelles constantes
  `DOMAIN_*_V1` (append).
- `crates/sbfb-workflow-core` (neuf) : schemas pack/graphe/agent + digests +
  fixtures golden Rust/TS ; `schemars` (Cargo.toml:360).
- `crates/nexus-coordinator-rs/src/provenance.rs:17-29,102-124` : pattern
  canonical_bytes reutilise ; **`crates/nexus-shell-daemon/src/deploy.rs:355-376`**
  (`finalize_deploy`, strip `provenance.json` avant hash + re-injection) :
  pattern etendu au publish de packs.
- Feed : raw-op (0 bump), index packs (FTS5, pattern S73).
- `docs/factory/` + `llms.txt` : cloture docs-contrat des schemas (AM-3).

### 4.3 D3 — Moteur durable Rust : memoisation-de-steps + journal SQLite append-only maison (2 crates), PAS de replay-total ni WASM ; echo-target deterministe A LIVRER

**Sources (datees)** : `oss_runtime.md` §1 (2026-07-17) — deux familles :
replay-de-commands (Temporal) vs memoisation-de-steps + journal (DBOS MIT ;
Restate BSL 1.1 non-OSI ; Inngest SSPL non-OSI ; PoC Morling 2025-11-20 : PK
`(flowId, step_seq)`, coeur <1000 lignes). SDK Rust **`temporalio-sdk` 0.5.0
(2026-06-29), 0.x jeune — 5 releases en 4 mois depuis 0.1.0-alpha.1
(2026-02-19)** [re-verifie live crates.io, corrige G1 — l'ancienne mention
« 0.1.0-alpha prerelease constate 2026-07-17 » etait perimee ; seuls les faits
constatables sont cites]. Obelisk 0.40.0 (2026-07-13, AGPL, Rust+SQLite) =
preuve d'existence du design cible mais 0.x rapide (0.39.5→0.40.0 en 8 jours)
+ wasmtime sous le capot. Decision gelee « OS sandbox, pas wasmtime »
RENFORCEE 2026 : CVE-2026-54786 + CVE-2026-34945/RUSTSEC-2026-0086 (`g2_g7.md`
§1.1), 0 dep au lock. `oss_runtime.md` §2 : upcasters chaines, snapshots =
cache jetable ; rusqlite 0.36 + rusqlite_migration 2.2 pinnes
(Cargo.toml:135-136) ; CVE-2025-6965 fermable par bump 0.40.1 SOUS RESERVE
arti-client (Cargo.toml:461-465). §4 : chaine CVE checkpointer LangGraph 2026
(CVE-2025-67644 SQLi + CVE-2026-28277 msgpack→RCE) ; convergence 4 frameworks ;
AUCUN ne fournit idempotence d'effets ni replay contractuel. Precedents
internes : dedup `(worker_pubkey, task_id)` S76 D ; replay outbox S75 A ;
binding fail-closed S81 K ; migration stores par classe S82 T ; feed raw-op.
`revalidation_code.md` B8 (2026-07-17) : echo-target ExecutionTarget
deterministe Rust JAMAIS livre (0 occurrence `crates/sbfb-factory`) ; S80-I a
livre un fixture-daemon JS front a la place.

**Retenu** :
1. **Famille** : memoisation-de-steps + journal append-only — seuls comptent
   l'identite stable des steps et l'idempotence des effets ; seule famille
   compatible « packs tiers non-fiables + noeud solo + 0 dep runtime ».
   Renoncement EXPLICITE au replay-total deterministe (Q5).
2. **Crates** : `sbfb-workflow-core` (PUR : sans Tokio/reseau/DB — schemas,
   canonicalisation+digests, validation statique, compilation
   `CompiledPlan.v1` immuable, fixtures golden) + `sbfb-workflow-engine`
   (hote-neutre : reducer + machine d'etat, scheduler local, retries/timers/
   joins/boucles bornees, inbox/outbox + idempotence, checkpoints + replay,
   traits `store/clock/activity/artifact/signal/secret/policy`, AUCUN couplage
   UI). Adaptateurs §21.4 : Model / NetworkCompute / Factory (companion) /
   McpTool / Knowledge / AppOperation (+A2a futur).
3. **Run store** : SQLite `rusqlite 0.36` + `rusqlite_migration 2.2` (bump
   0.40 = slot maintenance HORS chantier, QPO-3). Journal `RunEvent.v1`
   append-only PK `(run_id, sequence)`, hash de la requete de step journalise,
   cle d'idempotence `(run_id, node_instance_id, effect_id)` ; **outbox
   d'effets dans la MEME transaction** que l'avancement ; inbox dedup
   (at-least-once + dedup — precedent S76 D). `effect_id` stable entre
   retries, `attempt_id` change, `node_instance_id` encode le chemin complet.
4. **Reprise honnete** : kill/restart/replay sans double effet (spike Phase E
   GO/NO-GO AVANT tout gel de format) ; fail-closed sur divergence de replay
   (hash compare) ; effet non idempotent sans recu → `outcome_unknown`,
   JAMAIS de retry auto ni de succes invente ; fencing single-writer (lease) ;
   cancel honnete (le wire Task n'a pas de cancel fort — affiche).
5. **Evolution de schema** : enveloppe versionnee UNE fois
   (`RUN_EVENT_FORMAT_VERSION=1`) + payload `Value` extensible (politique feed
   raw-op) + upcasters a l'interpretation + snapshots jetables + fixtures
   golden events (culture S82 M). Run store = une classe de plus dans
   l'inventaire migrations (pattern S82 T).
6. **Doctrine securite checkpoints/packs (canonisee, Q5)** : Ed25519 AVANT
   tout parse riche ; requetes parametrees partout ; etat serialise
   donnees-pures JSON — miroir du binding fail-closed S81 K, reponse directe a
   la chaine CVE LangGraph 2026.
7. **Echo-target deterministe Rust A LIVRER** (bras d'activite echo/fixture,
   tests moteur hermetiques) — B8 prouve qu'il n'a jamais existe. Des Phase
   E/J.

**Alternatives REJETEES** :
- *Replay-de-commands a la Temporal* — REJETE sur 2 jambes independantes SANS
  argument de version (re-fonde, corrige G1) : (1) le replay-de-commands exige
  un determinisme du code workflow inapplicable a des packs tiers non-fiables
  — argument structurel ; (2) le serveur Temporal est un systeme distribue
  lourd (Cassandra/Postgres), antithese du noeud solo. S'y ajoute : SDK Rust
  0.x a cadence rapide (0.5.0, 5 releases en 4 mois) = meme classe de risque
  qu'Obelisk — reference de design, jamais une dependance.
- *Voie WASM a la Obelisk* — REJETE : impose wasmtime, ecarte par decision
  gelee RENFORCEE (CVE 2026) ; Obelisk 0.x en mouvement rapide — reference de
  design a re-implementer petit.
- *Dependre de cqrs-es/sqlite-es* — REJETE : cadence 1 release/15 mois avant
  0.5.0 (2025-12-30) ; semantique agregat CQRS ≠ journal d'execution ; on
  reecrirait la semantique SBFB (JCS + signature par entree) de toute facon.
- *redb 4 comme run store* — REJETE : KV pur sans requetage (timeline,
  frontier, outbox pending) ni migrations versionnees ; redb reste le store
  iroh-docs (transitif).

**Implications code** :
- Nouveaux : `crates/sbfb-workflow-core`, `crates/sbfb-workflow-engine`.
- `Cargo.toml:135-136` inchange (0 bump pendant le chantier) ; `:461-465`
  contrainte arti-client documentee pour le slot maintenance.
- `crates/nexus-shell-daemon/src/workflow_api.rs` (neuf) : routes run store
  (inventaire D7) ; `storage_api.rs:27-33,293-340` revalide INADAPTE au
  journal (KV last-value, GCRA 10 writes/min
  `crates/nexus-shell-daemon-core/src/storage_limiter.rs:12`) — le runtime
  possede SON store, le KV app n'est pas touche.
- `crates/sbfb-factory/src/provider_router.rs:62-113` : `ExecutionTarget`
  3 bras enveloppe par le ModelAdapter + bras echo deterministe NEUF.
- Golden : fixtures events committees (pattern §P75, 9 goldens S82 M).

### 4.4 D4 — Langage workflow/agents + contrats bridge/grants/HITL : `SBFBWorkflowGraph.v1` data-flow borne statiquement, invariant Plan-Then-Execute, intersection 7 couches AVANT tout verbe Workflow, HITL chrome hote signe

**Sources (datees)** : Research Pack §10-§15 (2026-07-15) — profil 18 nodes
candidat §10.3 (NON gele avant les 4 goldens), handoff ≠ agent-as-tool §10.4,
validation statique §10.5, 9 espaces d'etat §11, `CompiledPlan.v1`/
`RunEvent.v1` §12, placement §13.2, intersection 7 couches + 17 classes
§14.1-14.2, `secret_ref` §14.3, HITL hote signe §14.4, 14 operations `intent`
§15.1. `revalidation_code.md` §5.2 (2026-07-17) : consequence P0 FONDEE — 16
verbes (`protocol.ts:20-49`), dispatch sans consultation release/grant/
politique/consentement (`useBridge.ts:111-176`), manifeste declaratif
(`lib.rs:52-66`) ; B7 : `SENSITIVE_ACTIONS` = substring
`["shell","commit","push","PASS"]` (`operator_server.rs:37`).
`oss_workflow_ux.md` (2026-07-17) : data-flow > control-flow (MS Agent
Framework 1.0, 2026-04-03) ; handoff vs agent-as-tool (OpenAI Agents SDK,
2025-03-11) ; boucle bornee STRUCTURELLE > `recursion_limit` global (LangGraph
1.0 GA, 2025-10-22) ; CaMeL arXiv:2503.18813 + arXiv:2506.08837 ; tool
poisoning MCP (5,5 % de 1899 serveurs ; MCPTox >60 %) → pinning par digest ;
MCP 2025-11-25 pinnee, finale 2026-07-28 mid-S83 ; A2A v1.0.0 (2026-03-12) =
reference data model, PAS une dependance MVP (WA-10). AM-12 (`verif_plan36.md`)
: tension G-WIRE-COMPAT vs pre-launch policy.

**Retenu** :
1. **Noms ratifies (WA-1, Q6)** : `SBFBWorkflowGraph.v1` +
   `SBFBAgentDefinition.v1` dans `SBFBPromptAgentPack.v1` ; profil 18 nodes =
   CANDIDAT, gele seulement apres les 4 goldens (Phase G).
2. **Data-flow a ports types** : sequence/switch/classifier/parallel/map/
   join/`repeat` BORNE structurellement, subworkflow, `human.review`,
   `wait.signal`, sorties typees par schema. `handoff` (transfert explicite,
   payload schematise filtre) ≠ `agent.as_tool` (parent garde synthese +
   guardrails).
3. **Invariant securite ratifie (testable)** : « le graphe signe EST le plan ;
   aucun LLM n'etend le graphe, n'accorde une capacite, ne change une
   destination ni ne produit un verdict process autoritaire au runtime »
   (Plan-Then-Execute/CaMeL). Le compilateur §10.5 refuse tout ce qui la
   casserait. Prompt injection : peut influencer une SORTIE de modele, jamais
   topologie/outils/destinations/grants/classification/HITL/budgets (§19.3).
4. **Ordonnancement P0 ratifie** : AUCUN verbe bridge Workflow privilegie tant
   que l'intersection `caller ∩ methode-declaree-par-release ∩ grant ∩
   politique locale ∩ consentement de l'effet` n'est pas autoritaire dans le
   HOST (Phase C avant tout pouvoir — ordre C→D→E→F). Surface cible : 14
   operations `intent` (§15.1) — `intent` ne vaut JAMAIS approbation. Le gate
   `SENSITIVE_ACTIONS` substring n'est PAS etendu : capacites TYPEES par
   classe (17 classes §14.2), testees par proprietes.
5. **Capacites/secrets/HITL** : `effective_capabilities` = intersection de 7
   couches, chaque couche ne peut qu'ATTENUER ; secrets = `secret_ref` opaque
   + binding keystore hote juste avant l'appel, jamais dans
   pack/lock/journal/prompt/trace/export ; HITL rendu et signe par le CHROME
   hote, lie a `run_id + plan_digest + node_instance_id + effect_id +
   operation + destination + digest des donnees sortantes` — toute mutation
   invalide l'approbation. Approbations accessibles, file utilisable iframe
   fermee.
6. **MCP borne (WA-10)** : adapter CLIENT sortant `capability.call`
   permissionne, outils/schemas pinnes par digest blake3, descriptions =
   contenu non fiable, SSRF/DNS/redirect, pas de token passthrough. Revision
   pinnee 2025-11-25 ; migration stateless 2026-07-28 = contrainte de
   conception tracee (veille, `mcp_server_expose` gate-off). A2A = futur.
7. **Wire pre-launch (AM-12)** : evolutions Task/Result/ArtifactRef ADDITIVES
   par defaut (raw-op, `#[serde(default)]`, versions a 1, 0 bump) ;
   G-WIRE-COMPAT re-scope « tolerance additive prouvee + refus propre d'un
   champ inconnu critique » ; machinerie range-decode = derogation PO ecrite.

**Alternatives REJETEES** :
- *Graphe-as-code a la LangGraph* — REJETE : pas de format d'echange
  declaratif portable (code Python/TS) — non signable, non inspectable avant
  installation, non forkable par diff semantique (contraire §22.2 F/G).
  LangGraph valide en revanche le checkpoint-par-transition JSON (repris D3).
- *Gate substring etendu (SENSITIVE_ACTIONS)* — REJETE : faux positifs
  certains sur « pass »/« push »/« shell » (revalide `operator_server.rs:37`
  + scans :1437/:1539/:1644) ; un gate lexical ne peut porter ni 17 classes ni
  l'attenuation par couches.
- *Boucles non bornees + limite globale (`recursion_limit`)* — REJETE : borne
  globale non analysable statiquement §10.5 ni budgetable par branche §19.6 ;
  `repeat` borne structurel strictement plus fort.
- *A2A/MCP comme wire interne* — REJETE : A2A v1.0.0 a 4 mois ; le modele
  SBFB (noeud initiateur unique) n'a pas besoin d'interop agent-opaque au
  MVP ; la spec MCP change mid-sprint. Le wire SBFB reste autoritaire.

**Implications code** :
- `web/src/bridge/useBridge.ts:111-176` : dispatch consulte l'intersection
  autoritaire host (Phase C), revalidation per-call y compris offline ;
  `protocol.ts:20-49` : verbes Workflow APRES C uniquement.
- `crates/sbfb-manifest/src/lib.rs:34-66` + `BRIDGE_METHOD_ALLOWLIST:67+` :
  parite Rust↔TS etendue (drift-gate `check-frontier-contracts.sh`).
- `crates/sbfb-workflow-core` : validateur + compilateur (invariant
  Plan-Then-Execute testable) ; `schemars` pour les 2 formats.
- `crates/nexus-core-rs/src/task.rs:152-202` : extensions ADDITIVES seulement
  (metadata string-only preserve) ; `:416-453/:541-571` inchangees.
- `docs/security/THREAT_MODEL.md:186` : ligne STALE « 3 methodes whitelist »
  (16 reelles) fermee en Phase C.
- **9 fichiers `sbfb-bridge.js` trackes AU TOTAL** (corrige G1) :
  `web/public/` + 8 copies (templates sbfb-factory react/pyodide/static/
  static-reader + examples compute-tester/sbfb-explorer/sbfb-factory-viewer/
  sbfb-ideas) — le gate de parite anti-drift de la phase bridge enumere CET
  ensemble.

### 4.5 D5 — Structure d'execution : ADOPTION du plan ch.24 (36 phases A..AJ) AMENDE — AM-1..AM-12 + outline 41 phases

**Sources (datees)** : `verif_plan36.md` (2026-07-17) : verdict ADOPTER AVEC
AMENDEMENTS — 0 inversion dure (chaines E<F, L<N, W<X, Q<R, H<T, J<V,
X<Y<AD-AF, P<AF, {D,O,Q,H}<Z..AC, F/H<AG, tout<AH<AI<AJ) ; grammaire
`Phase [A-Z]+[0-9]?` prouvee (`agentctl.py:28/:32/:566`, 2 hooks `:88`/`:54`).
Research Pack ch.24 (maj 2026-07-17) + checklist §31. README §4 + §6.12 +
§7.4 ; conditions audit S82 (`sprint82_audit_findings.md:674-678`) ; decision
PO 2026-07-17 (pas de plan Hub autonome).

**Retenu** : verdict **PLAN-ADAPT** — colonne vertebrale ch.24 + 12
amendements (versions AMENDEES post-review integrees) :

| AM | Amendement (version finale) |
|---|---|
| AM-1 (BLOQUANT) | Mapping T1/T2 : **specs T1 Playwright hermetiques NOMMEES des l'ouverture** — Hub : `web/e2e/workflow-hub-author.spec.ts`, `workflow-hub-run.spec.ts`, `workflow-hub-lifecycle.spec.ts` (BLOQUANTS-CI chaque push, job nomme des R) ; surface shell : `web/e2e/bridge-grant-intersection.spec.ts` (Phase C). Artefact `sprint83_t2_acceptance.json` : top-level PASS/BLOCK{diagnosis}/RIG-ABSENT, paliers WA-T1..T6 en ACTED{evidence}/MIXED/NOT-RUN, chemins RIG-ABSENT pour les paliers 2-machines. **Regle de finalisation bi-temporelle** : AI2 ecrit l'agregat initial avec T6=MIXED{pre-cutover ACTED, cutover NOT-RUN} ; AJ/AJ1 mettent a jour le palier T6 et finalisent AVANT verification.md ; le commit AJ1 porte la version finale. plan.md §5 porte les rows fail-fast « T1 GREEN (`npm run test:e2e`) » + « t2_acceptance.json existe + parse + status vocabulaire ferme ». Motif (reformule) : **sans ce mapping le sprint est infermable par la DoD §4 (c)/(d)** — backstop : audit Track J = P1 ; lightcheck Check 10 = nudge WARN non-bloquant |
| AM-2 | Matrice de gates COMPLETE au plan : proprietaire de fermeture pour les 16 orphelins (G-AUTHORITY→E/J, G-EFFECT/G-UNKNOWN→K, G-EVAL→W, **G-VERTICALS→Z..AC (Factory = 1er vertical, premier golden) + AD-AF + fermeture agregee AI2**, G-I18N→B+phases UI, G-PERF→AI, 9 G-A11Y-*→B/T/W1/AI1) ; fermetures multi-phases explicites ; mecanisme d'enforcement par gate (script/hook/test/artefact vs review humaine) ; statut EXISTANT/ETENDU/NEUF (table §5.1 verif_plan36 : G-HOOK-FINALITY = les 2 hooks canon ex-D17, G-WORKFLOW-CSP etend la source CSP UNIQUE `csp.rs:33`, G-I18N = 3 scripts Lingui, G-PERF etend size-limit ; ~28 gates neufs legitimes). **G-SCOPE-FLOOR : fermeture deplacee en AJ1** (AI2 = pre-fermeture WA-T1..T5 + pre-cutover T6) ; preuve re-ecrite : « WA-T1..T6 complets AU SENS AM-5 ratifie — T6.8 (depreciation legacy sprints reels) = decision humaine post-S83 tracee au scope cut 14 ». **Extension HUB-T1..T10** : chaque HUB-T* recoit une phase proprietaire au plan.md (attendu R..V/X/Y — HUB-DRAFT→R, HUB-T1→S, HUB-T2→U, HUB-T3→V) ou une subsomption documentee par un gate/WA-T* — meme regle que les 16 orphelins G-* |
| AM-3 | Livrables §6.12/§3.3 nommes : etiquette generee (schema drift-gate + `// FRONTIER:`) DANS le commit de chaque primitive de frontiere neuve (bridge etendu, PackV1, GraphV1, ProfileV1, ProjectionV1, routes runtime, IPC companion, wire, MCP) ; cloture GUIDE + `llms.txt` en AJ1 ; THREAT_MODEL vNN + PATTERNS §P76+ au wrap-up |
| AM-4 | Re-coupes pre-planifiees : AI→3 (AI fault/scale/perf ; AI1 a11y/AT/panel round 2 ; AI2 acceptance agregee) ; AJ→2 (AJ shadow+cutover ; AJ1 cloture+wrap-up) ; **P et L PRE-COUPEES au plan.md** (chacune = multi-chantiers d'un ordre de grandeur au-dessus d'une phase S82 mediane — integre plan-challenge) ; X/B = candidates a re-coupe suffixee en cours de sprint ; **cout operateur d'AJ (baseline legacy REELLE rejouee sur l'intake dogfood) budgete explicitement au plan** |
| AM-5 | AJ borne : « sprint complet » shadow = le sprint interne `dogfood_factory` (Z..AC) rejoue ; depreciation legacy pour les VRAIS sprints (WA-T6.8, `authority=new`) = decision humaine POST-S83 via audit gate S84+ ; S83 livre la capacite de bascule PROUVEE (kill switch, rollback, oracle differentiel vert) |
| AM-6 | Reference perimee « trois blocs Rust/Python/frontend » (pack l.2496-2497) → canon §7.4 : 2 blocs pre-commit + 3 blocs Docker pre-push |
| AM-7 | 36 = PLANCHER de labels ; re-coupes suffixees = comportement nominal §4 (« le nombre de phases est une sortie ») — un compte final >41 n'est pas un derapage |
| AM-8 | DoD Phase B = « fondation + logistique » (harness, contrats prefs, i18n/RTL, matrice AT reservee, premier contact panel), PAS « conformite prouvee » (appartient aux surfaces puis AI1) |
| AM-9 | Panel remunere : round 1 = nouvelle phase W1 (apres V/W, surfaces auteur+runs disponibles), round 2 = AI1 |
| AM-10 | Semantique du prealable : « reservation engagee (owner+fallback+exit) » = bloquant ACTIVATION ; « ressource consommee » = checkpoints **B, pre-W1 (panel round 1 recrute et calendarise — reconcilie avec AM-9, integre plan-challenge), pre-R, pre-AI1 (round 2 + matrice AT)** ; defaillance externe = retour PO (jamais de carry masque) |
| AM-11 | Lot de routage des carries S82 au kickoff (§6 + Phase A1) — condition (2) du CONDITIONAL PASS ; la table §6.2 couvre TOUT le Carry-Over du findings (l.693-749), lot « P3 divers » inclus |
| AM-12 | Wire pre-launch : defaut additif-0-bump, G-WIRE-COMPAT re-scope, derogation PO ecrite pour toute machinerie range-decode |

Ajouts structurels kickoff : **Phase A1** (lot carries hors-theme bornes) ;
**exit de Phase G machine-verifiable** (integre plan-challenge) : les 4
goldens (Factory premier) serialises en JSON conformes aux JSON Schema draft
de `sbfb-workflow-core` (schemars), committes comme fixtures avec test de
validation VERT ; gel du profil 18 nodes consigne dans un artefact de decision
date ; H reste la preuve compilateur complete. Dependance implicite a
EXPLICITER au plan.md (constat plan-challenge) : Phase C exige un modele de
grant v0 host-side AVANT L/X — assume par le harness synthetique du pack.

**Alternatives REJETEES** :
- *Adoption du ch.24 TEL QUEL* — REJETE : gap BLOQUANT verifie — 0 mapping
  T1/T2 README §4 (sprint infermable par la DoD §4) ; 16 gates orphelins ;
  reference process perimee (l.2496-2497) ; carries audit S82 (posterieur au
  plan) sans slot.
- *Recreer un plan Hub autonome / re-decouper* — REJETE : INTERDIT par
  decision PO 2026-07-17 ; l'ancienne liste Hub 32 phases est superseded
  (compagnon §20) ; le crosswalk Hub→A..AJ existe (hub_pack §8).
- *Decoupage multi-sprint (coupe apres Viewer)* — REJETE : blueprint
  SUPERSEDED ; decision PO §22.1 « un seul sprint porte la totalite de
  l'objectif » + memory sprints ultra-complets ; clause anti-carry-masque
  (l.2808-2811) = retour PO explicite, jamais coupe silencieuse.

**Implications code/process** :
- `agentctl.py:28/:32/:566` + hooks `:88`/`:54` : labels bi-lettres jamais
  exerces → preuve d'exercice Phase A (G-PHASE-GRAMMAR).
- `.planning/active/` : kickoff/plan/design_review + verification (checklist
  par groupe de phases) + `sprint83_t2_acceptance.json` +
  `sprint84_audit_plan.md`.
- `ci.yml` : job T1 Hub hermetique nomme des la Phase R.

### 4.6 Acknowledged review findings (G1)

Review adversariale pre-gel jouee par 3 panels (g1-board sur D1..D5 ;
plan-challenge sur le sequencement/gates ; repo-fit sur la conformite canon).
Chaque objection est integree INLINE (aucune ignoree) :

| # | Finding | Integration |
|---|---|---|
| g1-D3 (majeur) | Citation crates.io temporalio-sdk perimee (0.1.0-alpha « constate 2026-07-17 » faux — reel : 0.5.0, 2026-06-29) | D3 corrige : citation re-fondee sur faits constatables + rejet Temporal re-fonde sur 2 jambes structurelles sans argument de version (§4.3) |
| g1-D1 (mineur) | http.rs « 1434 l » faux | 1513 l au tip `74531d4` (re-mesure `wc -l` par le writer) — §1.2, §4.1 |
| g1-D2 (mineur) | deploy.rs implicitement place dans nexus-coordinator-rs | Chemins de crate complets partout ou l'ancre est inter-crate (deploy.rs → nexus-shell-daemon ; csp.rs → nexus-core-rs ; storage_limiter.rs → nexus-shell-daemon-core) — §4.2, §4.3 |
| g1-D4 (mineur) | « + 9 copies » sbfb-bridge.js — reel : 9 fichiers AU TOTAL (8 copies) | Reformule avec l'ensemble exact pour le gate de parite — §4.4 |
| plan (bloquant) | Condition 2 non integralement acquittee : lot « P3 divers S83+ » + C-1 (P2) sans route | 2 rows ajoutees a la table §6.2 (C-1→AJ1 ; lot P3 ventile A1 / discipline immediate / re-route trace S84 avec rationale par item) ; le mot « complete » n'est employe qu'apres couverture integrale |
| plan (majeur) | G-SCOPE-FLOOR infermable en AI2 (WA-T6 ferme en AJ ; wording non re-scope AM-5) | Fermeture deplacee AJ1 + preuve re-ecrite dans AM-2 — §4.5, §5 |
| plan (majeur) | Exit Phase G « compiles CONCEPTUELLEMENT » non binaire | Exit machine-verifiable (goldens serialises + validation schemars + artefact de decision date) — §4.5, §5 |
| plan (majeur) | Artefact T2 bi-temporellement faux si ecrit fige en AI2 | Regle de finalisation AM-1 : T6=MIXED en AI2, finalise AJ/AJ1 avant verification.md — §4.5 |
| plan (mineur) | AM-10 non reconcilie avec AM-9 (checkpoint panel) | Checkpoint pre-W1 ajoute — §4.5 |
| plan (mineur) | HUB-T4..T9 orphelins | Extension AM-2 a HUB-T1..T10 (proprietaire ou subsomption documentee au plan.md) — §4.5 |
| plan (mineur) | Evidence app-authoring gamable (run jetable) | Exit renforcee : digest blake3 de l'archive generee + lineage vers le commit initial du Hub — §6.1 |
| plan (mineur) | G-VERTICALS omet Z..AC | Assignation etendue (Factory = 1er vertical) + labels outline — §4.5, §5 |
| repo-fit (majeur) | Specs T1 non nommees (2 surfaces) + rows fail-fast non promises | 4 specs nommees dans AM-1 + rows exigees au plan.md §5 — §2, §4.5, §5 |
| repo-fit (majeur) | Sections canoniques manquantes (Sources pre-gel, Constat §1.1-1.4 + HARDENING, SMART, slot G1, tracabilite) | Toutes ajoutees : Sources consultees, §1 (ancrage HARDENING §1.4), §2 (SMART), §4.6 (ce slot), §8 (table) |
| repo-fit (mineur) | AM-1 sur-vend Check 10 (WARN non-bloquant) | Motif reformule : autorite = DoD §4 (c)/(d) + audit Track J P1 ; Check 10 = nudge — §4.5 |
| repo-fit (mineur) | Risk register sans colonnes Likelihood/Impact | Tabule — §9 |
| repo-fit (mineur) | Chore sans section SPRINT_LOG v2.2 ; AJ1 sans Track J | Ajoutes — §5 (chore), §5 (row AJ1), §10 |
| repo-fit (mineur) | Hook lightcheck Check 5 (G1) = code mort ($PHASE minuscule vs "A") | Discipline de plan : `sprint83_design_review.md` produit AVANT tout commit Phase A (ne pas compter sur le backstop) ; fix du hook route en Phase A1 — §5 |

**Slot G1 formel** : `sprint83_design_review.md` (agent designer-critic) est
DU avant le gel final des D-choices et AVANT tout commit Phase A — le present
tableau consigne la review adversariale pre-gel ; le design review formel s'y
ajoute et ses findings seront acknowledges ICI par amendement du kickoff si
necessaire (le hook Check 5 etant mort, l'enforcement est discipline de plan,
pas mecanique).

---

## 5. Outline FINAL des phases (41 phases + Phase 0 hors compte)

Grammaire : `Phase [A-Z]+[0-9]?` (prouvee). 36 labels ch.24 + 5 pre-coupes
kickoff (A1, W1, AI1, AI2, AJ1) ; P et L pre-coupees AU PLAN (AM-4). AM-7 :
compte = PLANCHER. Chaque phase : preflight G8 + review independante + Codex +
reconciliation + commit atomique (body 9 sections, trailer Co-Authored-By —
derive I-1 reprise des le 1er commit) + 2 blocs pre-commit / 3 blocs Docker
pre-push (AM-6) + etiquette docs-contrat par frontiere neuve (AM-3) + gates
a11y/i18n sur toute phase de surface (AM-8).

| Phase | Titre | Objectif (1 ligne) | Gates principaux |
|---|---|---|---|
| 0 | Audit gate S82 | JOUE — CONDITIONAL PASS `74531d4` ; conditions acquittees §3/§6 | — |
| A | Cadrage, contrats de preuve, ressources | Materialiser D1..D5 en contrats versionnes ; matrice COMPLETE objectif→acceptance→gate→phase→commande→artefact (AM-2, HUB-T1..T10 inclus) ; spec T1/T2 (AM-1) ; preuve labels bi-lettres ; reservations verifiees | G-SCOPE-FLOOR (ouvre), G-PHASE-GRAMMAR, G1/G9 |
| A1 | Lot carries hors-theme bornes | Fermer les carries CI/hygiene S82 : supply-chain (CI-1), Node 20→22 (CI-2), SEEDER-DIAL-TAIL (ou WONTFIX PO), macos-14 watcher, spdx Woodpecker, shellcheck, TEST-ISOLATION state.json, 0-golden familles, dispatch_loop test-group, **fix hook lightcheck Check 5 (comparaison casse — G1 code mort)**, nits doc M-7/C-2 | exit conditions par item (§6.2) |
| B | Fondation a11y, i18n, preferences privees | Preboot, tokens, mode guide, harness WCAG/ATAG/COGA, i18n/RTL, matrice AT + premier contact panel — DoD = fondation+logistique (AM-8) ; checkpoint ressources #1 | G-A11Y-PREFS (ouvre), G-I18N (ouvre) |
| C | Autorite du bridge existant | Intersection caller∩release∩grant∩politique∩consentement AUTORITAIRE per-call (offline inclus) AVANT tout verbe Workflow ; grant v0 host-side (explicite au plan) ; ferme THREAT_MODEL:186 ; spec `web/e2e/bridge-grant-intersection.spec.ts` | G-BRIDGE-GRANT |
| D | Extraction du vrai contrat Factory | `FactoryProcessProfile.v1` machine-lisible lie aux digests sources ; documente vs enforce ; PROFILE_STALE bloquant | G-FACTORY-CONTRACT |
| E | Spike durable BLOQUANT | Kill/restart/replay, journal/outbox atomiques, 0 double effet a chaque frontiere de crash ; echo-target Rust deterministe v0 ; GO/NO-GO — bloque F | G-AUTHORITY (ouvre), G-STORE (ouvre) |
| F | Format portable et signe | `SBFBPromptAgentPack.v1` + layout + PACK.lock + JCS/domaines + corpus hostile + digests Rust/TS concordants | G-IMPORT-HOSTILE (ouvre) |
| G | Langage workflow, agents et roles | Graphes/agents/18 nodes candidats/handoffs/budgets + independance driver/reviewer/Codex ; **exit MACHINE-VERIFIABLE : 4 goldens serialises JSON valides schemars (test VERT, fixtures committees) + artefact de decision date gelant le profil** | gel du profil de nodes |
| H | Compilateur Rust autoritaire | Normalisation, validation statique §10.5, fermeture capacites, digests semantiques, plan immuable, anti-drift Rust/TS — preuve compilateur complete des goldens | invariant Plan-Then-Execute testable |
| I | Run store durable | SQLite event log versionne + upcasters + snapshots + inbox/outbox + retention + backup/restore + GC + fencing ; re-coupe suffixee candidate | G-STORE, G-UPGRADE-REPLAY, G-SINGLE-WRITER |
| J | Orchestrateur local | Reducer/scheduler deterministe, branches, map/join, boucles bornees, timers, wait/signal, sous-workflows, pause/reprise, backpressure, cancel honnete | G-AUTHORITY (ferme avec E) |
| K | Effets, retries et idempotence | effect_id/attempt_id/receipts/timeouts/reconciliation/outcome_unknown ; effets FS/process lies HEAD/index/worktree ; nettoyage descendants (Windows inclus) | G-EFFECT, G-UNKNOWN, G-FS-PROCESS-EFFECT (ouvre) |
| L | Permissions et attenuation (PRE-COUPEE au plan avec L1 coffre/HITL — AM-4) | Attenuation/audience/profondeur/expiration/revalidation ; coffre generique (rotation/zeroize) ; confirmations accessibles chrome hote | G-CAPABILITY, G-SECRET-VAULT, G-SECRET, G-HITL |
| M | Artefacts, ACL et provenance | ArtifactRef type, facade blob, classes public/prive/secret, audience, chiffrement, retention, lineage — digest ≠ droit de lecture | G-ARTIFACT-ACL, G-PROVENANCE |
| N | Modeles locaux et cloud | Adaptateur provider-neutral, Ollama reel, ≥1 provider cloud reel par secret_ref, structured outputs, doubles hermetiques CI sans credential | G-PLACEMENT (ouvre) |
| O | Adaptateurs Claude Code et Codex | AgentSessionAdapter + ClaudeCodeAdapter + CodexExecAdapter ; receipts complets ; roles independants | G-CLAUDE-CODEX-RECEIPT |
| P | Wire et compute SBFB (PRE-COUPEE au plan avec P1 MCP borne — AM-4) | Task/Result/ArtifactRef additifs 0-bump (AM-12) ; plancher public-only + preuve ensemble des lecteurs ; MCP pinne par digest, SSRF/DNS/redirect | G-WIRE-COMPAT (re-scope), G-NETWORK, G-PLACEMENT, G-MCP-SSRF |
| Q | Hote de confiance et companion Factory | Bridge type commandes/events/replay/quotas ; companion IPC capability-scoped hors TCB reseau ; profil CSP Hub strict (source unique) | G-HOST-ISOLATION, G-WORKFLOW-CSP |
| R | Accueil et bibliotheque Workflow Hub | App sandboxee : boot/offline/reconnexion/recherche/templates/versions/conflits/etats vides ; **scaffold cree VIA le parcours Factory app-authoring TRACE avec lineage digest (§6.1)** ; job T1 CI nomme + 3 specs Hub (AM-1) ; checkpoint ressources pre-R | HUB-DRAFT ; evidence app-authoring |
| S | Registre d'agents et brouillons | CRUD/version/diff agents, prompts, policies, budgets, dependances ; autosave/undo/recovery | HUB-T1 |
| T | Canvas, outline et mode guide | 3 projections de premiere classe, modele de commandes unique, MEME digest semantique ; edition sans drag/voix/clavier/lecteur ; re-mesure xyflow a l'activation (QPO-5) | G-A11Y-AUTHORING, G-A11Y-PARITY (ouvre) |
| U | Inspecteurs, capacites et preflight | Formulaires types, grants demandes/effectifs/refuses/expires, preflight 10 points ; mutation invalide le consentement | HUB-T2 |
| V | Centre des Runs | Liste/statuts/timeline bornee/child-runs/HITL/pause/cancel/retry/replay/outcome_unknown ; pas de faux succes | HUB-T3 (avec E/I/J/K) |
| W | Evaluations et comparaison | Datasets/scenarios versionnes, graders deterministes AVANT LLM, export de preuve ; aucun grader LLM n'accorde securite/PASS | G-EVAL |
| W1 | Panel utilisateurs round 1 (AM-9) | Round 1 panel remunere sur surfaces auteur (R..U) + runs (V) ; barrieres P0/P1 routees en rework AVANT X ; checkpoint pre-W1 (AM-10) | G-A11Y-USERS (ouvre) |
| X | Publication, decouverte, installation et RRV | Cycle complet publish/install-desactive/activate/fork/update-reconsent/rollback/depreciation/revocation sans auto-update ; projection publique + Proof Card ; re-coupe suffixee candidate | G-PUBLISH-RECOVERY, G-RRV-PROJECTION, G-LIFECYCLE |
| Y | Operations inter-apps | Operations headless, child-runs idempotents dedupliques, depth/cycles/budgets, handoffs par ArtifactRef sous ACL | G-APPCALL |
| Z | Factory dogfood : kickoff, plan et preflight | Rejouer dans workspace+namespace `dogfood_factory` le cycle d'entree Factory reel | G-VERTICALS (Factory, 1er vertical — ouvre) ; bloquants DESIGN-CONFLICT/PROFILE_STALE |
| AA | Factory dogfood : implementation et tests | Worktree isole, driver, effets lies HEAD/index/worktree, 3 blocs complets ; injection/repo hostile sans mutation de plan/capacites | G-FS-PROCESS-EFFECT (ferme), G-VERTICALS (Factory, suite) |
| AB | Factory dogfood : review, Codex et reconciliation | PASS-PENDING → Codex brut → reconciliation → boucle bornee ; le driver ne s'auto-attribue jamais PASS | G-REVIEW-INDEPENDENCE (ferme), G-VERTICALS (Factory, suite) |
| AC | Factory dogfood : verdict, commit, carries et closure | Promotion Verdict PASS ; commit.intent/push.intent confirmes humains ; hooks refusent ; statuts du sprint INTERNE | G-HOOK-FINALITY ; G-VERTICALS (Factory LIVRE — premier golden) |
| AD | Pack Ideas | Intake→…→ResearchPack+DecisionPack — executable, forkable, evaluable | G-VERTICALS (2e vertical) |
| AE | Pack Conception | 4 analyses paralleles, cross-review, ADRs, ConceptionPack consommable par Factory | G-VERTICALS (3e vertical) |
| AF | Pack Babel | Rights gate, corpus legal borne, ≥100 chunks hermetiques CI, dogfood 2 workers, crash/reprise, reassembly, provenance | G-VERTICALS (4e vertical) |
| AG | Assistance IA + import Flowise | Demande en francais → diff declaratif source-only accepte/refuse ; import AgentFlow V2 data-only (jamais `$flow.state`) | G-AUTHORING-AI-DIFF, G-IMPORT-HOSTILE (ferme) |
| AH | Securite adversariale et supply chain | Fuzz pack/compiler/bridge/wire ; injection/SSRF/revocation/canari/CSP/packs malveillants ; locks/licences/SBOM/build offline | G-SUPPLY-CHAIN |
| AI | Consolidation fault, scale et perf | Fault injection, mixed-version (AM-12), scale, perf/budgets, bundle/storage, 2 state dirs/2 daemons ; dogfood 2 machines (fenetre volet Mac stores S82, §6.2) | G-PERF |
| AI1 | A11y multi-AT et panel round 2 | Matrice 5 OS/AT, WCAG/AAA/EN 301 549, ATAG/COGA, panel round 2 — 0 barriere P0/P1 ; checkpoint pre-AI1 | G-A11Y-* (ferment), G-A11Y-USERS (ferme), G-A11Y-PARITY (ferme) |
| AI2 | Acceptance agregee (PRE-fermeture) | WA-T1..T5 + pre-cutover WA-T6 joues ; 4 packs, 3 placements ; **`sprint83_t2_acceptance.json` ecrit avec T6=MIXED{pre-cutover ACTED, cutover NOT-RUN}** (AM-1) ; aucun P0/P1 en carry de confort | G-VERTICALS (fermeture agregee 4/4) ; G-SCOPE-FLOOR = PRE-fermeture seulement |
| AJ | Shadow Factory, cutover et rollback | Baseline legacy vs nouveau sur meme intake, `authority=legacy`, 1 phase puis sprint dogfood Z..AC rejoue en shadow (AM-5) ; oracle differentiel ; kill switch/snapshot/rollback ; palier T6 mis a jour | G-FACTORY-CUTOVER |
| AJ1 | Cloture docs-contrat, verification et wrap-up | GUIDE + llms.txt + THREAT_MODEL vNN + PATTERNS §P76+ (AM-3) + **note d'en-tete espace §P COMMUN + 7 doublons documentes (C-1) + oracle ±0 canonise §P75 pt 6 (C-3)** ; **finalisation `sprint83_t2_acceptance.json` (palier T6) AVANT verification.md** ; verification.md ; **`sprint84_audit_plan.md` incluant Track J testabilite + Track K docs-contrat** ; go/no-go pilote ferme ; G6/G7 | **G-SCOPE-FLOOR (FERME, au sens AM-5)** ; Track K fermable |

Chore d'archivage (au commit kickoff, hors compte) : archive S82 → v2.1
(findings N-1 et `sprint83_audit_plan.md` restent en active/) ; creation
`.planning/archive/v2.2/` + **section `## v2.2 — <theme>` dans
`docs/claude/SPRINT_LOG.md`** si v2.2 ratifiee (Q2) ; reconciliation CLAUDE.md
post-push-gate + itemisation delta E-2 (condition 3, §6.3) ; registre LT
(QPO-2) ; memory tip rattrape (`aa9db47` → tip kickoff).

---

## 6. Items carry / dette (conditions audit S82 — acquittement)

### 6.1 EN PREMIER — decision fermante app-authoring S79 (condition 1, NON-NEGOCIABLE)

Compteur : carry ne au wrap-up S79, **3 reconductions de kickoff
(S80/S81/S82), 4e sprint sans evidence** — Regle 2 §6.2.1 : 5e reconduction
sans decision = P1 frais. **Ce kickoff tranche.**

**Resolution proposee (a ratifier PO, Q8)** : parcours in-vivo TRACE AU PLAN
comme livrable de la **Phase R**. Le scaffold initial de l'app Workflow Hub
(archive SBFB React sandboxee) est cree VIA le parcours Factory app-authoring
reel : prompt-kind `app-authoring` + knowledge packs blake3 + gate CSP
deterministe `run_gate_csp_authoring` (source unique `BLOB_SERVE_CSP`
`crates/nexus-core-rs/src/csp.rs:33`) + self-check runtime. **Exit condition
RENFORCEE (anti-gaming, integre plan-challenge)** : l'evidence machine-lisible
committee (artefact JSON + receipts) inclut le **digest blake3 de l'archive
generee par le parcours**, ET le commit initial du Hub reference ce digest
(lineage scaffold → app livree) ; si le scaffold est ensuite reecrit, le lien
de filiation reste trace dans l'evidence — un run jetable sans filiation ne
ferme PAS le carry. Arguments : (a) le Hub EST une app SBFB — cas d'usage
exact du carry ; (b) cout marginal quasi nul (la Phase R cree l'app de toute
facon) ; (c) opportunite identifiee independamment par verif_plan36 §6.2 et
etat_projet §3. Alternative si le PO requalifie : requalification ECRITE au
kickoff (non recommandee).

### 6.2 Condition 2 — table de routage des carries (couvre TOUT le Carry-Over du findings l.693-749)

| Carry | Reports | Route S83 | Exit condition |
|---|---|---|---|
| app-authoring S79 in-vivo (P1 STANDING) | 4e sprint | **Phase R** (§6.1) | evidence JSON + digest blake3 + lineage commit Hub |
| SEEDER-DIAL-TAIL `seed_registry.rs:331` (P2) | **≥3 reports (S75→S76→S77/S82-H)** — MANDATORY | **Phase A1** (sampling `blake3(context‖pubkey)` sur `seeders_recent`, pattern §P68 `placement.rs`) OU WONTFIX PO ecrit (Q10) | test unitaire distribution + doc ; sinon WONTFIX date |
| supply-chain GHA rouge (CI-1, P2) | 1 | **Phase A1** | allowlist Unlicense/MIT-0 (ou exceptions par-crate) + 3 legs pip-audit decables + CLAUDE.md:439 rectifie ; job VERT |
| leg operator Node 20 vs 22 (CI-2, P2) | 1 | **Phase A1** (bump ci.yml:70/:176 seul ; review+Codex arc front restent hors-sprint, Q13) | leg VERT ou echec residuel documente arc-front |
| classe watcher macos-14 (10 tests, P2) | 1 (route S83) | **Phase A1** | fix (override nextest / notify backend) OU perennisation PO consignee |
| TEST-ISOLATION state.json classe ≥2 sites (P2) | 1 (route S83) | **Phase A1** | tout Engine de test confine `NEXUS_GRID_ROOT` ; state.json jamais preuve live |
| 0-golden familles feed/search/provenance/preview/proof-card/browse/nodes + `browse_pull` | 1 (route S83) | **Phase A1** | net golden §P75 etendu (oracle ±0 cite) |
| flake dispatch_loop + sigint/running_json + occurrence Win A-1 | 1 | **Phase A1** | test-group nextest borne + description de classe a jour |
| hook lightcheck Check 5 mort (G1 — constat review) | neuf | **Phase A1** | comparaison insensible a la casse (ou `$PHASE_UPPER`) + test |
| spdx Woodpecker (G-3/CI-4, P3) | 1 | **Phase A1** | steps docs-gates non skippes OU glitch env documente avec temoin alternatif |
| shellcheck chronique (CI-3, P3) | 1 | **Phase A1** | 3 warnings fixes ou disable justifie ligne a ligne |
| nits doc M-7 (drift http.rs→test_support.rs:392) + C-2 (module-doc hub) | 1 | **Phase A1** | doc corrigee |
| **C-1 (P2) : note d'en-tete espace §P COMMUN + 7 doublons P33..P39 dans les 2 PATTERNS** | 1 | **Phase AJ1** (au moment d'ecrire §P76+) | note presente dans `docs/rust/PATTERNS.md` ET `docs/shell/PATTERNS.md` + 7 doublons documentes |
| C-3 (P3) : oracle ±0 refacto pur a canoniser §P75 point 6 | 1 | **Phase AJ1** (meme passe doc) | §P75 point 6 canonise |
| I-1/I-2/I-3 (P3) : headers/trailers commits (derive Co-Authored-By) | 1 | **discipline immediate** — preambule plan §3, applique des le 1er commit S83 | verifie par l'audit S84 sur la fenetre S83 |
| **Lot P3 purs re-routes traces → `sprint84_audit_plan.md`** (rationale par item) : deploy.yml startup-failure 0s (trigger workflow_dispatch seul — cosmetique CI, 0 impact verite) ; actionlint local jamais joue (outillage, non bloquant) ; eval-curl `b3_shard_pipeline.sh` 5 sites (B-1 — script sharding, slot rig-chaud) ; casse `phase_A` review A (F-3 — artefact historique) ; K-1 assertion unicite HOW_TO_WIRE (doc-gate, non urgent) ; D-1 perimetre §1 audit_plan (meta-doc) ; E-1 projection Docker body B (meta-doc) | 1 chacun | **re-route date 2026-07-17** | mention par item dans `sprint84_audit_plan.md` (ecrit en AJ1) |
| volet Mac stores + redescente consent L4 Mac | **2 (S82 differe → S83)** | **fenetre Phase AI** (dogfood 2 machines = session Mac chaude naturelle) | observation on-disk par classe + hygiene rule-4 + consents re-decides (Q9) |
| worker recreate-guard (residual bas) | 1 | `sprint84_audit_plan.md` (trace) | — |
| P2-AUDIT-2-RESIDUEL deny.toml (dalek rc) | standing | re-etiquete external-blocked (chore) ; veille yank renforcee (dalek 3.0.0 stable existe, iroh pinne rc) | debloque par release iroh |
| codex sandbox `elevated` casse | 1 | decision PO (Q15) — workaround `--sandbox read-only` | — |
| Mirror Codeberg auth | 1 | action manuelle PO settings GitHub (Q15) | — |
| Topologie A-vs-B (PO-5, due 25/08) + gate C8 01/08 | calendaire | **hors-sprint PO pendant S83** (recommande, Q11) ; revalidation PKARR_RELAY_OPS (pkarr-relay 1.0.0) AVANT provisioning | decision consignee avant 25/08 |
| THREAT_MODEL.md:186 stale (3 vs 16 methodes) | 1 | **Phase C** (docs-contrat de la phase) | ligne corrigee + verifiee |
| bump arti-client 0.44 + rusqlite 0.40 (CVE-2025-6965) | 1 | slot maintenance S84+ (QPO-3 — 0 RUSTSEC ouvert) | trace roadmap |
| Arc front parque `wip/factory-front-arc-post-s82` (87 fichiers) | echeance « post-S82 » ATTEINTE | decision PO Q13 (avant/pendant/apres S83) | review + Codex groupes a la reprise |

### 6.3 Condition 3 — chore d'archivage

Reconciliation CLAUDE.md post-push-gate (blocs « Etat actuel » + « CI »
re-rediges a l'etat REEL : rust-ci 4/5 verts, nightly SUCCESS, rouges
restants listes) + itemisation delta S82 corrigee « A+2 B+1 C0 D+1 H+1 K-1
M+9 » (E-2) + registre LT replie (LT-1 resolved, LT-5 reecrite — pointe du
Python supprime, LT-7 quorum E2E CLOSED S81 K ; QPO-2) + **section
`## v2.2 — <theme>` dans SPRINT_LOG si Q2 ratifie** + memory
`nexus_grid_pivot.md` tip rattrape. Execute au chore du commit kickoff.

### 6.4 Attention 3/3 a l'horizon S84

Items qui atteindraient 3 reports si reconduits sans fermeture en S83 (grille
Track G — a surveiller au wrap-up) : **volet Mac stores** (2 reports : differe
S82, fenetre AI S83 — si la fenetre est ratee, 3e report = escalade
MANDATORY S84) ; **flakes dispatch_loop/sigint** (2e traitement — la classe
doit etre bornee en A1 sinon 3/3) ; **P2-AUDIT-2-RESIDUEL** (external-blocked
— exempt tant que l'etiquette et la veille yank sont tenues a jour).

---

## 7. Scope cuts (14 items — chacun re-evalue contre le code au HEAD `74531d4`)

1. **Scheduler DAG distribue / failover multi-leader** — HORS MVP (§13.4).
   Re-evalue : aucun scheduler n'existe (task.rs = feuilles LWW, grep
   frontier/checkpoint/cancel = 0) ; autorite run = noeud initiateur (WA-3).
2. **Replication live P2P du run state prive** — HORS MVP. Re-evalue :
   `REPLICATED_APPS = ["sbfb-ideas"]` (`storage_api.rs:29`) — seule
   replication = KV public d'app ; le journal reste local.
3. **Co-edition temps reel du canvas** — HORS MVP. Re-evalue : iroh-docs LWW
   sans causalite (task.rs:193-195), aucune infra CRDT ; brouillons locaux +
   fork/versions couvrent le pilote ferme.
4. **Node code-execute / plugins tiers / code importe** — HORS MVP (§10.5
   refus compilateur). Re-evalue : wasmtime ecarte RENFORCE (CVE 2026, 0 dep).
5. **Marketplace centrale / ranking global / token governance** — JAMAIS
   (anti-capture §16.2, kudos non-monnaie, vision_model).
6. **Auto-update des packs** — HORS MVP, contraire au canon (curated_updates) :
   activation = re-consentement, update elargissant les droits BLOQUE.
7. **A2A adapter** — FUTUR (WA-10). Re-evalue : v1.0.0 du 2026-03-12, aucun
   besoin d'interop agent-opaque au MVP ; data model = reference additive.
8. **`DelegationGrant.v1` cross-noeud** — FUTUR (§14.5). Re-evalue :
   `DelegationCert` (`attestations/delegation.rs`) = identite↔forge, PAS un
   grant d'activite.
9. **Cancel fort d'une Task reseau** — HORS MVP. Re-evalue : grep cancel = 0
   dans task.rs ; le produit AFFICHE la limite (§12.7), cancel honnete local.
10. **Envelope chiffree recipient-scoped** — plancher Phase P = public-only
    avec preuve de l'ensemble reel des lecteurs ; envelope declaree FUTURE
    sauf decision PO contraire (Q6/§27-16).
11. **Import executable total Flowise/n8n/LangGraph** — HORS MVP (WA-12) :
    import data-only AgentFlow V2, `$flow.state` jamais mappe.
12. **Memoire personnelle P2P + optimisation auto de prompts** — HORS MVP
    (§22.4).
13. **SBFB comme serveur MCP elargi** — HORS chantier. Re-evalue :
    `mcp_server_expose` gate-off (S25/S28) ; spec 2026-07-28 = veille (Q14),
    0 slot.
14. **Depreciation effective du chemin Factory legacy (`authority=new`)** —
    POST-S83 (AM-5) : S83 livre la capacite de bascule prouvee (shadow vert,
    kill switch, rollback) ; bascule sur les VRAIS sprints = decision humaine
    via audit gate S84+. G-SCOPE-FLOOR se ferme en AJ1 AU SENS de ce cut.

---

## 8. Tracabilite scope S82 → S83 (chaque What's-NOT S82 mappe)

| Item non fait / decale en S82 (roadmap v5 bloc S82 + verification §7) | Devenir S83 |
|---|---|
| workflow-engine (decale PO-9, « futur slot dedie ») | **= CE sprint** (theme S83 ; ratification Q1) |
| Viewer fondation + Apercu scelle / Proof Card | NON-S83 — futur slot trace (reconfirme sans nouveau gap) |
| Reprise arc front (`wip/factory-front-arc-post-s82`) | decision PO Q13 (echeance « post-S82 » atteinte) ; bump Node CI-2 joue en A1 independamment |
| Sharding feature/hardening (R-J-6, F2 KV-cache, SI-12, N3-reveal, tuning) | NON-S83 — slot rig-chaud (D6 S82) |
| Benchmarks re-jeu (BLOCK{rig: cold}) | NON-S83 — va avec le slot rig-chaud |
| Splits fichiers >2000 l (shard_session.rs, engine/runtime.rs, iroh_runtime.rs, db.rs, public_feed.rs) | NON-S83 — prochain refacto (oracle ±0 EXACT a citer) |
| Bump arti-client/rusqlite | NON-S83 — slot maintenance S84+ (QPO-3) |
| Topologie A-vs-B + gate C8 | hors-sprint PO pendant S83 (Q11) |
| Volet Mac stores + consents L4 | fenetre Phase AI (§6.2) |
| macos-14 watcher, TEST-ISOLATION, 0-golden, dispatch_loop (routes S83 par l'audit_plan) | Phase A1 (§6.2) |

---

## 9. Risk register R1..R7

| R | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Dependances externes du prealable (panel 2 rounds, licences/machines AT 5 OS, provider cloud, 2 workers physiques) — plus long lead time | Haute | Haut (gel de phases terminales) | AM-10 (reservation engagee = bloquant activation ; consommation = checkpoints B / pre-W1 / pre-R / pre-AI1) ; AM-9 (round 1 en W1) ; owner+fallback+exit par ressource ; retour PO obligatoire, jamais de carry masque |
| R2 | Densite de phase sans precedent (41 phases, chacune un mini-projet ; record S82 = 24 majoritairement refacto) | Haute | Moyen (allongement, jamais coupe silencieuse) | AM-4 pre-coupes (AI×3, AJ×2, P, L) + AM-7 (re-coupes = nominal) ; process uniforme tenu ; honnetete de compte au wrap-up |
| R3 | UI back-loadee (R..V en positions 20+) + dep canvas a re-mesurer (@xyflow/react 12.11.2 ABSENT du repo, faits blueprint SUPERSEDED) | Moyenne | Moyen (rework schema G/H) | Contract-first (inspecteurs pilotes par schemas) ; compagnon Hub normatif deja ecrit ; re-mesure xyflow a l'activation de T (budget = mesure constatee +10 %) ; risque ACCEPTE consigne |
| R4 | Evenements calendaires mid-sprint : gate C8 01/08 (~2 sem.) + topologie A-vs-B due 25/08 + spec MCP finale 2026-07-28 | Certaine (dates connues) | Moyen (context-switch operateur) | Traites HORS-sprint PO (Q11) ; revalidation PKARR_RELAY_OPS avant provisioning ; MCP = veille gate-off ; context-switch budgete |
| R5 | Surface TCB nouvelle (runtime hote + routes loopback + companion IPC + coffre) sous zones rouges INCHANGEES (R-iroh-audit P0) | Moyenne | Haut (securite) | Inventaire D7 route par route + tiers + auth complete ; THREAT_MODEL vNN confronte au §19 du pack en G8/G9 par phase ; aucune promesse de production publique (§28) |
| R6 | Fenetre de crash effet/journal (LE cas dur durable-execution) | Moyenne | Haut (integrite des runs) | Spike Phase E GO/NO-GO BLOQUANT avant tout gel de format ; cles d'idempotence sur TOUS les effets ; fail-closed sur divergence de replay ; `outcome_unknown` jamais convertible en succes |
| R7 | Enforcement des 44+ gates + labels bi-lettres jamais exerces (« Phase AA » n'a jamais tourne) | Moyenne | Moyen (gates = intentions) | Phase A prouve la grammaire (commit test + agentctl) + matrice AM-2 complete (mecanisme par gate) ; T1 cable CI des R ; G-SCOPE-FLOOR/G-VERTICALS re-assignes (§4.5) |

---

## 10. Audit gate pattern (rappel) + G6 memory-fusion prep

- **Wrap-up AJ1** : `sprint83_verification.md` avec §Acceptance au vocabulaire
  FERME (top-level PASS/BLOCK{diagnosis}/RIG-ABSENT ; paliers
  ACTED{evidence}/MIXED/NOT-RUN — ratifie S82 J) + §Fail-fast checklist
  (critere SMART §2) + **`sprint84_audit_plan.md` incluant explicitement la
  Track J — Testabilite ET la Track K — docs-contrat** (README §4
  Enforcement (2)) + les re-routes traces du §6.2.
- **Phase 0 de S84** = audit gate S83 sur `sprint84_audit_plan.md` ; les
  statuts du dogfood interne (IMPLEMENTED_AUDIT_PENDING/AUDITED_CLOSED)
  restent dans le namespace `dogfood_factory` et ne ferment pas le sprint.
- **Actions thread principal — G6 memory-fusion prep** (verification S82 §8,
  a jouer au moment memoire) : (1) PIEGE espace §P COMMUN aux 2 PATTERNS
  (grep les 2 avant tout mint — lie a C-1/AJ1) ; (2) gate machine
  non-discriminant sans anchors ; (3) gate push bi-temporel + classe watcher
  macos-14 ; (4) TEST-ISOLATION state.json = classe ≥2 sites ; (5) exit par
  classe de store. + memory tip rattrape (`aa9db47` → tip kickoff).

---

## 11. Checkpoint de validation (questions PO — chaque question admet un non constructif)

> **Ratifications PO 2026-07-18** (session ultracode) : **Q1 = Workflow Agents**
> (slot ratifié) ; **Q2 = v2.2 nouvelle** (SPRINT_LOG section ouverte, v2.1
> close à `aa9db47`) ; **Q8 = A** — parcours in-vivo tracé Phase R (scaffold
> Hub via Factory app-authoring, exit renforcée digest blake3 + lineage ;
> ferme le carry P1 STANDING par la preuve) ; **Q9 = ressources engageables**
> (plan d'activation fourni : provider cloud clé+plafond+secret_ref ; panel =
> lead-time long, recrutement à démarrer ; AT VoiceOver/TalkBack gratuits +
> JAWS/Dragon licences/démo + switch/braille achat/emprunt, cellules non
> couvertes = exemption documentée ; 2 workers PC+Mac déjà en main = ré-élever
> consents L4). **Condition 3 audit S82 exécutée** (CLAUDE.md réconcilié
> post-gate + itemisation delta E-2 + SPRINT_LOG v2.2). Restent à trancher au
> fil : Q3-Q7 (détails D1..D5), Q10-Q15. **Phase A peut démarrer** ; les
> ressources à lead-time long (panel Q9-b) sont à engager en parallèle
> (checkpoints AM-10 : B / pré-W1 / pré-R / pré-AI1).

1. **Ratification formelle du slot S83** = chantier Workflow Agents (1er slot
   trace roadmap v5 bloc S82 ; signaux frais : 2 Research Packs + decision
   noms 2026-07-17). Un NON = retour aux 2 autres slots candidats (Viewer
   fondation / reprise arc front).
2. **Version d'archive** : v2.2 NOUVELLE (recommandee — theme structurellement
   nouveau §7.3, v2.1 longue de 18 sprints) vs v2.1 continue. S82 s'archive en
   v2.1 quoi qu'il arrive ; si v2.2 : section SPRINT_LOG au chore.
3. **D1 — placement** : confirmer runtime generique dans l'HOTE daemon
   (routes → inventaire D7) + FactoryAdapter en companion IPC
   capability-scoped (WA-2/WA-19) ; ratifier les noms `sbfb-workflow-core` /
   `sbfb-workflow-engine`. Un NON sur le placement = re-instruction §21.3
   avant Phase A.
4. **D2 — format pack** : ratifier JCS+Ed25519+DOMAIN dedies + `subject[]`
   blake3 ; trancher : l'interop supply-chain tierce (DSSE/cosign) est-elle un
   objectif produit du chantier (sinon differee sans casse) ?
5. **D3 — moteur** : ratifier memoisation-de-steps + journal SQLite maison
   (= renoncement EXPLICITE au replay-total deterministe et a la voie
   WASM/Obelisk) ; canoniser la doctrine securite checkpoints ; confirmer la
   livraison du bras echo-target deterministe Rust (jamais livre, B8).
6. **D4 — langage/contrats** : ratifier WA-0..WA-24 en bloc (deviation
   listee : AM-12 wire additif) + noms de schemas (WA-1) + invariant
   Plan-Then-Execute + ordonnancement P0 (intersection AVANT verbes) ;
   envelope recipient-scoped Phase P : future explicite ou livree ? ;
   HUB-2..HUB-10 + budgets perf chiffres (§15.3) + version EN 301 549 FIGEE
   (WA-16).
7. **D5 — plan** : ratifier l'adoption du ch.24 avec AM-1..AM-12 (versions
   amendees §4.5) + l'outline 41 phases (plancher, re-coupes nominales) + la
   position du push dogfood (« push.intent apres
   verification/audit_plan/wrap-up », Phase AC) + la semantique de
   depreciation legacy post-S83 (AM-5) + la fermeture G-SCOPE-FLOOR en AJ1.
8. **Condition 1 audit S82 — app-authoring S79** : ratifier le parcours
   in-vivo trace en Phase R avec exit RENFORCEE (digest blake3 + lineage,
   §6.1) OU produire la requalification PO ecrite. **NON REPORTABLE** (5e
   reconduction sans decision = P1 frais).
9. **Ressources du prealable d'activation** (chacune : owner, disponibilite,
   fallback, exit condition ; cadre vision_model = depense operationnelle
   personnelle ponctuelle, JAMAIS pattern funding/institutionnel) :
   (a) provider cloud reel — lequel, compte, budget, `secret_ref`, quota
   d'acceptance ; (b) panel handicape remunere ≥12 participants 2 rounds —
   canal, budget, consentement, calendrier (round 1 = W1) ; (c)
   machines/licences AT (JAWS, Dragon/Voice Access, iOS/Android, switch,
   braille) — acquisitions vs exemptions explicites ; (d) 2 workers dogfood
   PC+Mac — re-elevation des consents L4 (PC redescendu, Mac differe S82).
   Un NON sur une ressource = re-scope AM-10 (fallback/exit) avant activation.
10. **SEEDER-DIAL-TAIL** (≥3 reports) : fix sampling blake3 en Phase A1 OU
    WONTFIX PO documente.
11. **Calendaire** : gate C8 01/08 + topologie A-vs-B (due 25/08) — traites
    HORS-sprint pendant S83 (recommande) ou inscrits au plan ? Revalidation
    PKARR_RELAY_OPS (pkarr-relay 0.11→1.0.0) AVANT provisioning dans les deux
    cas.
12. **Staging** au commit kickoff des 2 Research Packs untracked + du
    `verification_blueprint.md` modifie (banniere SUPERSEDED) : oui/non.
13. **Arc front parque** (`wip/factory-front-arc-post-s82`, 87 fichiers,
    review+Codex groupes DUS, echeance « post-S82 » ATTEINTE) : reprise
    avant / pendant / apres S83 ? (Le bump Node CI-2 est joue en A1
    independamment.)
14. **MCP** : pinner la revision 2025-11-25 et tracer la migration stateless
    2026-07-28 comme contrainte de conception (recommande) — confirmer 0
    slot, veille seule.
15. **Divers PO standing** : codex sandbox `elevated` casse (reparation a
    statuer) ; secret Mirror Codeberg (action manuelle settings GitHub —
    quand) ; LT-2 flip Radicle (reconfirmer « parque, hors-sprint ») ;
    registre LT replie au chore (QPO-2) ; @xyflow/react 12.11.2 = dep
    APP-SIDE bundlee dans l'archive Hub (jamais dep runtime daemon/shell),
    re-mesuree a l'activation de T (QPO-5) ; promptfoo post-acquisition
    OpenAI (2026-03-09) : outil red-team pinnable ou ecarte ; triggers
    zombies doc-dette (openai-agents-python ×2, seuil frost WARRANT_CANARY)
    au prochain passage doc (QPO-7).

---

## Invariants tenus (rappel de cadrage)

Sprint 83 IMPAIR (A1 = lot VOLONTAIRE impose par la condition 2) ; pas
d'estimation LOC (§6.7 — deltas par familles de tests uniquement :
`sbfb-workflow-core` +120..200, `sbfb-workflow-engine` +150..250,
daemon/companion/adaptateurs +80..150, Vitest shell +40..80, Vitest Hub
+150..250, E2E Hub +15..30 specs, E2E web +0..5, operator 0..+10 — ordre de
grandeur +550..950, SORTIES de phases, jamais des engagements) ; pre-launch
protocol (versions a 1, raw-op 0-bump, derogation ecrite pour toute machinerie
mixed-version — AM-12) ; JAMAIS funding/fondation/startup (depenses
ponctuelles PO uniquement, Q9) ; kudos non-monnaie (aucun
stake/payment/deposit dans les packs) ; heberger ≠ publier, seeder ≠ auteur ;
MUR jamais bouton, 0 verdict calcule UI ; Factory hors daemon (companion,
v4 D2) ; hooks lightcheck/Codex = seul backstop mecanique (ex-D17 — sauf G1
Check 5 mort : discipline de plan, fix route A1) ; francais docs planning,
anglais code/identifiants ; pilote ferme jusqu'aux preuves publiques
(R-iroh-audit P0).
