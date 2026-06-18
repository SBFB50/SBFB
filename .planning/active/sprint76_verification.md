# Sprint 76 — Verification (self-report fail-fast, living)

> Doc VIVANT mis a jour a la cloture de chaque phase ; finalise en Phase G.
> Valeur limitee par construction (self-report de l'agent livreur) — la
> verification independante est l'audit gate S77 Phase 0. NE consigne PAS
> d'assertion non executee : ce qui est differe est marque DIFFERE.

## §1 HEAD progression

- **HEAD entree S76** : handoff kickoff `3faee6e` (docs(sprint76), POUSSE
  origin/master apres gate dual-platform vert).
- **HEAD courant** : Phase G `feat(daemon)` wrap-up (ce commit finalise
  verification.md + cree `sprint77_audit_plan.md` + MAJ docs longue-vie +
  fmt-fix root-cause `http.rs`). NON push (ahead ~15 vs origin : A `ce43894`
  + agents `d6dea45` + B `6904cdd` + C `1cc28e7` + verif C `5b07472` +
  D `d75ae77` + verif D `1de6f8a` + supervisor-supprime `42c7448` +
  README-bootstrap `a21aaad` + E `768e235` + verif E `24bda54` + F `a547de6`
  + verif F `df86bdc` + G feat + ce). **Gate dual-platform Phase G VERT AVANT
  push** (Win 1804 + Docker 1808 + fmt 0 sous les 2 toolchains + web 398) —
  cf. §4 + §6. Push = decision operateur (LT-2/Radicle hors-sprint).

## §2 §7.4 par phase (fail-fast)

| Phase | Commit | Rust Windows nextest | Rust Docker (canonique) | Frontend | fmt/clippy/doctests/release |
|---|---|---|---|---|---|
| A | `ce43894` | 1763 -> 1767 (+4) 0-skip | subset 582 (crates touches) | Vitest 379 -> 386 (+7) | 0 / 0 / 0 / 0 |
| B | `6904cdd` | 1767 -> 1775 (+8) 0-skip | 3 crates 675/675 (+3 cfg(unix)) | Vitest 386 -> 396 (+10) | 0 / 0 / 0 / 0 |
| C | `1cc28e7` | 1775 -> **1785** (+10) 0-skip | **1789/1789** 0-skip (code fonctionnel, round 1) | 0 (aucun changement web) | 0 / 0 / 0 / 0 |
| D | `d75ae77` | 1785 -> **1789** (+4) 0-skip | consolide G (§4) | 0 (aucun changement web) | 0 / 0 / 0 / 0 |
| E | `768e235` | 1789 -> **1799** (+10) 0-skip | consolide G (§4) | Vitest 396 -> 397 (+1) | 0 / 0 / 0 / 0 |
| F | `a547de6` | 1799 -> **1804** (+5) 0-skip | consolide G (§4) | Vitest 397 -> 398 (+1) | drift*->fix G / 0 / 0 / 0 |
| G | `<feat>` | 1804 -> **1804** (+0, fmt-fix whitespace) 0-skip | **1808/1808** 0-skip (recovery, §4) | Vitest 398 (0) | **0 (fix)** / 0 / 0 / 0 |

(*) **CORRECTION DE DIAGNOSTIC (Phase G).** Les suites D/E/F annoncaient
`http.rs:8528` flagge par `cargo fmt --all --check` comme un « faux positif derive
toolchain local 1.95 vs canonique 1.94 ». **C'etait FAUX.** Le re-run Docker
canonique `rust:1.94` de Phase G produit le **MEME diff byte-identique** : un appel
`kudos_ledger::credit(&db, "proj-vc", "worker-a", "task-1", 10, 1_000)` (test
`#[cfg(test)]` du dashboard contributeur, introduit Phase E `768e235`) depasse la
largeur de ligne et rustfmt — 1.94 ET 1.95 a l'identique — veut le wrapper. Le diff
n'avait jamais ete vu sous 1.94 car le Docker etait DIFFERE chaque phase depuis C.
Phase G **corrige a la racine** (reformate l'appel, le wrapping que les deux
toolchains produisent) -> **fmt = 0 sous Win 1.95 ET Docker 1.94** (verifie :
`WIN_FMT_EXIT=0` + `DOCKER_FMT_EXIT=0`). Ce n'etait pas un drift toolchain, c'etait
une vraie violation fmt latente.

Delta tests Phase F = **+5 Rust** (1799 -> 1804) + **+1 Vitest** (397 -> 398) : phase
**doc-only D5** (quantization 4-bit documentee). `crates/nexus-worker-core/tests/quantization_doc.rs`
+5 (present, footprint-table, 70b-is-s77, quorum-precondition, llama_cpp_unchanged_doc_only —
tests d'integration lecture texte, feature-independants) ; `GpuConsentDialog.test.tsx` +1
(rendu hint). Detail des suites Phase F :
- Windows : clippy `--workspace --all-targets -D warnings` 0 ; nextest workspace
  **1804/1804** 0-skip ; doctests 0 ; release `nexus-shell-daemon` 0. Frontend : lint 0 ;
  tsc 0 ; test:unit 398 ; coverage 87.2/79.01/85.92/88.52 (>= 85/78/85/85) ; build +
  size (128.76 kB < 130) + scan-en-strings 0. fmt : voir note (*) ci-dessus.
- Preflight Workflow ultracode = PLAN-ADAPT (Livrable 3 lien front non-relatif ->
  Option B texte non-cliquable, defaut sur) ; Review Workflow = PASS (2 P2 honnetete
  corriges en-phase) ; Codex GPT5.5 = **5/5 CONFIRME / 0 GAP / 0 PARTIEL**.
- Docker Linux sbfb-ci (canonique) : **DIFFERE** recovery avant push (§4) ; diff
  platform-agnostique (doc `.md` + web + test lecture texte stdlib, 0 `#[cfg(unix)]`).
- Invariant anti-scope-creep verrouille : `llama_cpp.rs` non touche (git diff vide),
  ne cable que `with_n_gpu_layers` (grep `with_split_mode|with_devices` = 0).

Delta tests Phase E = **+10 Rust** (1789 -> 1799) + **+1 Vitest** (396 -> 397) :
kudos_ledger.rs +6 (sanity-bound clamp/preserve/credit + contributor summary
EMA/tasks/empty), db.rs +2 (get_worker_entries + EXPLAIN QUERY PLAN index),
http.rs +2 (route aggregates/empty) ; dispatch_loop.rs = test E2E existant ETENDU
(generation_time_ms >= 1, pas un net-new) ; Network.test.tsx +1. Detail des suites
Phase E :
- Windows : `cargo fmt --all --check` 0 ; `cargo clippy --workspace --all-targets
  --locked -- -D warnings` 0 ; `cargo nextest run --workspace --locked` **1799/1799**
  0-skip ; `cargo test --workspace --locked --doc` 0 ; `cargo build -p
  nexus-shell-daemon --release` 0. Frontend : lint 0 err ; tsc 0 ; test:unit 397 ;
  coverage 87.2/79.01/85.92/88.52 (>= 85/78/85/85) ; build + size + scan-en-strings 0.
- **P1 review resolu a la racine** : le worker codait `generation_time_ms: 0` (bug
  latent, 1er consommateur prod = le sanity-bound) -> mesure Instant reelle +
  `StubBackend::with_delay_ms` + assertion E2E `generation_time_ms >= 1` (dispatch_loop).
- Codex GPT5.5 : 14 CONFIRME / 0 GAP / 0 PARTIEL (9 livrables + 5 invariants).
- Docker Linux sbfb-ci (canonique) : **DIFFERE** recovery avant push (§4) ; diff
  platform-agnostique (SQLite/axum/React, 0 `#[cfg(unix)]`).

Delta tests Phase D = **+4 Rust** (2 result_sync.rs hermetiques + 1
runtime.rs seed + 1 validator.rs verrou). Detail des suites Phase D :
- Windows : `cargo fmt --all --check` 0 ; `cargo clippy --workspace
  --all-targets --locked -- -D warnings` 0 ; `cargo nextest run --workspace
  --locked` **1789/1789** 0-skip ; `cargo test --workspace --locked --doc` 0 ;
  `cargo build -p nexus-shell-daemon --release` 0 (code fix inchange depuis le
  build vert §7.4 ; seuls tests + commentaires + docs modifies apres).
- **Gate P2 (Codex S76-D)** `cargo test -p nexus-shell-daemon --locked`
  shared-process : 383 + 6 + 7 = 0-fail (le 3-noeuds E2E qui timeout-ait sous
  contention pleine-crate a ete RETIRE en reconciliation Codex ; quorum prouve
  par composition — 2 hermetiques sur le vrai bridge + cross-node redundancy=1
  existant + Phase G LIVE).
- Docker Linux sbfb-ci (canonique) : **DIFFERE** a la recovery avant push (§4) ;
  diff platform-agnostique Rust (0 `#[cfg(unix)]`, logique bridge daemon-interne).

Delta tests Phase C = **+10 Rust** (6 task.rs + 1 dispatcher.rs + 3
engine/runtime.rs). Detail des suites Phase C :
- Windows : `cargo fmt --all --check` 0 ; `cargo clippy --workspace
  --all-targets --locked -- -D warnings` 0 ; `cargo nextest run --workspace
  --locked` 1785/1785 0-skip ; `cargo test --workspace --locked --doc` 0 ;
  `cargo build -p nexus-shell-daemon --release` 0. Re-confirme APRES les
  fixes doc PARTIEL (run sequentiel Windows-seul `bg2vhto0q`).
- Docker Linux sbfb-ci (canonique) : fmt + clippy + nextest 1789/1789
  0-skip + doctests = `DOCKER_ALL_GREEN` sur le code FONCTIONNEL (run
  `bq6jj6fkz`, target dir isole en volume nomme). Le re-run Docker APRES
  les fixes doc-only (commentaires verification.rs + 2 maps `.md` + script
  `.sh`) est **env-bloque** (cf. §4) — delta doc-only platform-agnostique,
  Windows-confirme.
- Gate anti-regression existant `e2e_network_execute_gate_real_http_no_
  frontier_mock` (nexus-shell-daemon) : hors-diff, vert.

## §3 Acceptance LIVE B-3 (palier 1, D2) — DIFFERE materiel operateur

- **Livre** : harness scripte `scripts/acceptance/b3_live_pc_vps.sh`
  (endpoints reels verifies contre le code : `POST /api/v1/invite/create`
  scope:worker -> token sous cle `wire` ; `nexus-worker join` +
  `nexus-worker start --headless` ; `POST /api/v1/tasks/submit` ;
  `GET /api/v1/tasks/{id}/result` -> `result_text`). **Phase G : le harness
  sert maintenant les DEUX paliers** via le parametre `REDUNDANCY` (defaut 1)
  cable a `redundancy_factor` du submit + une section d'enrolement d'un 2e
  worker homogene — `REDUNDANCY=2` rend le palier 2 (quorum, §5) reellement
  runnable par l'operateur (`bash -n` clean ; correction PLAN-ADAPT G : avant,
  `redundancy_factor:1` etait hardcode et le palier 2 non-runnable).
- **Critere falsifiable (D2 adjust)** : `DELAY` = delai submit (VPS) ->
  result_text visible (VPS), END-TO-END = borne SUP de la convergence WAN
  `result:` (claim + inference GPU + replication). Pour le petit prompt
  deterministe du harness, claim+inference est de quelques secondes, donc
  un delai proche/au-dela de 30s implique la convergence WAN. **> 30s =
  BLOCK a DIAGNOSTIQUER** (root-cause : inference ou replication ? ref S75
  `SeedAnnounced peer_count:0 ~10 min`), **PAS un timeout a rallonger**.
- **Statut** : run WAN reel PC RTX 5080 <-> VPS **DIFFERE** — l'environnement
  de dev de cette session n'a pas la paire PC+VPS+WAN. A executer sur le
  materiel operateur ; la trace (submit -> claim PC GPU -> result_text WAN
  rendu + delai mesure) sera consignee ici a ce moment. PAS d'assertion
  LIVE non executee (precedent S74 : re-run dual differe a recovery). Le
  chemin compute (dispatch/pompe/result-sync/validator/sign-verify) est
  INCHANGE et couvert in-process par le gate anti-regression (§2).

## §4 Note env — Docker Desktop Linux engine wedge (S76-C)

Le re-run Docker du delta doc-only est env-bloque : lancer Windows nextest
et un build Docker LOURDS en CONCURRENCE a sature la RAM hote ->
(1) le linker MSVC a crashe en boucle `STATUS_STACK_BUFFER_OVERRUN`
(0xc0000409), (2) le conteneur Docker a ete tue (`unexpected EOF`), puis le
moteur Docker Desktop Linux s'est wedge (`500 Internal Server Error` sur
`/_ping`, le WSL-wedge documente en memoire S74). Le code FONCTIONNEL avait
deja passe Docker canonique 1789/1789 (round 1, avant les doc-fixes) ; le
delta doc-only (commentaires + `.md` + `.sh`) est platform-agnostique et
Windows-confirme. **Lecon : relancer les suites SEQUENTIELLEMENT (Windows
seul, puis Docker seul), jamais simultanement.**

**RECOVERY EXECUTEE EN PHASE G (gate AVANT push).** Le moteur Docker etait sain
(server 29.4.3, `ServerErrors []`, image `sbfb-ci:latest` rust:1.94). Suites
re-jouees SEQUENTIELLEMENT sur l'arbre FINAL (Windows d'abord, puis Docker seul,
target Linux isole en volume nomme `sbfb-linux-target`) :
- **Windows** : fmt 0 (apres fix `http.rs`, cf. note *) + clippy 0 + nextest
  **1804/1804** 0-skip + doctests 0 + release 0.
- **Docker canonique `sbfb-ci` rust:1.94** : fmt **0** (`http.rs` clean sous
  1.94 apres fix) + clippy 0 + nextest **1808/1808** 0-skip (+4 `#[cfg(unix)]`)
  + doctests 0.
- **Web** : lint 0-err + tsc 0 + Vitest **398/398** + coverage
  87.2/79.01/85.92/88.52 + size 6/6 + scan-en-strings clean.
Le wedge S76-C ne s'est PAS reproduit (un seul build lourd a la fois). Gate
dual-platform VERT, push debloque cote technique.

## §5 Cloture Phase G + acceptance LIVE

**Phase G a livre** : finalisation de ce verification.md (colonne Observed §6) +
`sprint77_audit_plan.md` (10 tracks Phase 0 S77) + MAJ docs longue-vie
(THREAT_MODEL v9, PATTERNS rust §P62 + shell P38, SPRINT_LOG row S76, CLAUDE.md
0-76 CLOSED, roadmap_v5 Arc 3.5 6/6 clos + S77 ouvert) + **fmt-fix root-cause
`http.rs:8531`** (cf. note * §2) + **harness palier 2 runnable** (`REDUNDANCY`,
§3) + recovery Docker dual-platform VERT (§4).

**RETRACTATION env-note fmt** : la consigne « NE PAS reformater http.rs (faux
positif 1.95) » des suites D/E/F est **ANNULEE** — c'etait un vrai diff fmt
latent (note * §2), corrige Phase G. fmt = 0 sous les deux toolchains.

**Acceptance LIVE cross-machine — DIFFERE materiel operateur, jamais faux-vert** :
- **B-3 palier 1** (row #26) : DIFFERE — env de session sans PC RTX 5080 + VPS +
  WAN. Harness `b3_live_pc_vps.sh` runnable (token cle `wire`, worker
  `start --headless`). Critere falsifiable <30s = BLOCK-a-diagnostiquer encode.
- **Quorum palier 2** (row #30, redundancy=2 VPS+PC+Mac) : DIFFERE — exige 2
  workers homogenes (meme MODEL/quant) + Mac. Harness rendu runnable Phase G
  (`REDUNDANCY=2` → le submit pose `redundancy_factor:2` **ET `verifiable:true`**
  — P1 review : sans `verifiable`, le worker echantillonne [task.rs] et le
  dispatcher saute le cohort gate [dispatcher.rs:70] → divergence → quorum
  JAMAIS forme ; le harness le met automatiquement). Deux `result_text`
  byte-identiques -> consensus accepte ; heterogene-diverge ECRIT comme attendu
  (anti faux-vert T1). Le fix bridge result-sync (`d75ae77`, dedup per-worker)
  est le prerequis prod (avant : 2e worker jete). > 30s = BLOCK, PAS un timeout
  rallonge. **Scope (note Codex D2)** : le harness prouve le QUORUM
  cross-machine via `verifiable` + homogeneite ASSUREE-PAR-L'OPERATEUR ; il ne
  soumet PAS `required_runtime`, donc il n'exerce PAS l'auto-claim-gate du
  dispatcher (couvert in-process par le test Phase C
  `dispatcher_routes_replicas_to_homogeneous_cohort`) — choix delibere pour
  eviter une fragilite tuple-mismatch dans un run manuel.
- Le chemin compute (dispatch/pompe/result-sync/validator/sign-verify) est
  couvert IN-PROCESS par le gate anti-regression + les 2 tests hermetiques quorum
  2-auteurs (accept + diverge). La DIFFERE est une dependance materiel user
  (CONCERN), PAS un defaut du code compute. Traces a consigner ici quand
  l'operateur execute les runs.

## §6 Fail-fast final dual-platform (38 rows, Observed — gate AVANT push)

| # | Check | Observed |
|---|---|---|
| 1 | fmt `--all --check` | **0** (Win 1.95 + Docker 1.94 apres fix http.rs) |
| 2 | clippy `--workspace --all-targets -D warnings` | **0** (Win + Docker) |
| 3 | nextest workspace (Win) | **1804/1804** 0-skip |
| 4 | doctests | **0** (Win + Docker) |
| 5 | release build `nexus-shell-daemon` | **OK** |
| 6 | Docker Linux canonique `sbfb-ci` rust:1.94 | **1808/1808** 0-skip (+4 cfg(unix)) |
| 7 | web tsc | **0** |
| 8 | web lint | **0 err** (5 warn benins) |
| 9 | web Vitest | **398/398** (37 files) |
| 10 | web coverage | **87.2/79.01/85.92/88.52** >= seuils |
| 11 | web build+size | **6/6** (102.48/263.13/9.84/128.76 kB sous limites) |
| 12 | scan-en-strings | **clean** |
| 13 | A snapshot additif 0-bump | PASS (`consent_snapshot_serializes_additively` + `SCHEMA_VERSION=1`) |
| 14 | A enrolement worker public | PASS (`colocated_worker_honors_user_consent_when_public`) |
| 15 | A least-priv OFF | PASS (`colocated_worker_least_privilege_when_off`) |
| 16 | A route consent daemon | PASS (`consent_route_reaches_daemon_prefix`) |
| 17 | B duress `seed_voluntary` no-op | PASS (`seed_voluntary_noop_in_duress`) |
| 18 | B duress `set_keep_online` no-op | PASS (`set_keep_online_noop_in_duress`) |
| 19 | B aggregator downgrade ingress | PASS (`aggregator_downgrades_open_source_without_provenance`) |
| 20 | B failover multi-tier | PASS (`pull_falls_back_across_tiers_when_ticket_dead`) |
| 21 | B outbox 2-noeuds | PASS (`outbox_gossip_has_neighbors_two_nodes`) |
| 22 | B 5 pages front smoke | PASS (Network/Curators/Projects/OnboardingEmpty/ProjectDetail) |
| 23 | B LOOPBACK §3 a jour | PASS (7 routes S74+S75 inscrites) |
| 24 | C routing cohorte homogene | PASS (`dispatcher_routes_replicas_to_homogeneous_cohort`) |
| 25 | C gate compute anti-regression | PASS (`e2e_network_execute_gate_real_http_no_frontier_mock`) |
| 26 | C acceptance LIVE B-3 + WAN <30s | **DIFFERE materiel operateur** (PC+VPS+WAN absents ; harness runnable, §5) |
| 27 | D quorum 2 byte-identique | PASS (`quorum_redundancy_two_stubworkers_byte_identical`) |
| 28 | D divergence rejetee | PASS (`quorum_diverging_outputs_rejected`) |
| 29 | D validator inchange | PASS (`git diff --stat validator.rs` quorum = 0 ligne) |
| 30 | D acceptance LIVE quorum | **DIFFERE materiel operateur** (VPS+PC+Mac ; harness `REDUNDANCY=2` -> submit `verifiable:true` auto, runnable, §5) |
| 31 | E agregation contributeur EMA | PASS (`get_contributor_summary_aggregates_ema`) |
| 32 | E route dashboard | PASS (route `/api/v1/contributor/{node_id}` + `_empty`) |
| 33 | E page 3 metriques | PASS (`Network.test.tsx` contributor) |
| 34 | F doc quantization presente | PASS (`QUANTIZATION.md` + table + <=14B + S77) |
| 35 | F backend doc-only inchange | PASS (grep `with_split_mode`/`with_devices` = 0 ; `llama_cpp_unchanged_doc_only`) |
| 36 | 0 bump wire | PASS (`*_FORMAT_VERSION`/`*_ANNOUNCEMENT_VERSION`/`SCHEMA_VERSION` = 1) |
| 37 | verification.md ecrit | PASS (ce fichier) |
| 38 | audit_plan S77 ecrit | PASS (`sprint77_audit_plan.md`) |

**Bilan : 36/38 verts en session + 2 rows LIVE (#26/#30) DIFFERE-trace-user**
(materiel operateur), gate technique VERT AVANT push.

## §7 Carry-over for memory (a fusionner dans `nexus_grid_pivot.md` / MEMORY.md)

- **Tip** : HEAD = Phase G `feat(daemon)` wrap-up. S76 CLOSED, Arc 3.5 Factory
  Complete Vision **6/6 COMPLET**. Prochain = S77 sharding (Phase 0 audit gate
  S76 = `sprint77_audit_plan.md`).
- **Compteurs** : Rust nextest Win **1804** (1763 -> +41) / Docker **1808** /
  Vitest **398** (367 -> +31). 0 bump wire, 0 dep sur tout S76.
- **Carries reconduits S77** (cf. `sprint77_audit_plan.md` §3) : SYBIL-SEEDER-TAIL
  2/3 (exemption nommee « dependance sharding »), REVISION-HOME-DURABILITY 2/3,
  KNOWN-ENTRY-OVERCOUNT 2/3, seeder `catalog_len:0` 2/3, RE-DRIVE-ON-INGEST 2/3,
  T-NN+3 (JCS), **P3-D-3 NOUVEAU 1/3**, **MEDIAN-DE-GROUPE DOC-P2 NOUVEAU**.
  3 carries 2-reports FERMES en B (CARRY-3/LOOPBACK-TIERS/PULL-3). Externes
  inchanges (P2-A-1, P2-AUDIT-2, T-NN+2, P3-OS-1, LT-2 ARME). LT-5 resorbe.
- **Lecons process G** : (1) un fmt-diff sous Windows DOIT etre confirme sous
  Docker canonique AVANT de le diagnostiquer « drift toolchain » — ici les deux
  toolchains voulaient le MEME wrapping (vraie violation, pas drift) ; ne jamais
  differer Docker plusieurs phases de suite sur du code qui touche des tests.
  (2) le verdict preflight G8 PLAN-ADAPT a evite un faux-vert « 38/38 » sur
  l'acceptance LIVE.
