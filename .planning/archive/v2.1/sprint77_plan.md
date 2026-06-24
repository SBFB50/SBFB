# Sprint 77 — Plan : Sharding pipeline (modèle 70B éclaté cross-machine)

**Écrit** : 2026-06-20. **Révisé 2026-06-20** (arbitrage PO scope maximal D3/D4).
**Tip master** : `4da9800` (poussé origin/master).
**Roadmap** : Sprint 7/7 arc compute (S71-S77), v2.1 Arc 3.5 Factory Complete Vision.
**Design figé** : `sharding_design_addendum_sota_2026-05-30.md` + `remote_user_sharded_llm_rnd.md` §10 + `SPLIT_INFERENCE_DESIGN.md`.

> **Scope MAXIMAL (arbitrage PO)** : 70B complet (scheduler Parallax 2-phases + benchmark
> 3-5 machines) + vérif N0+N1+N2+N3 + incentive curator-reputation. N4 zkML hors-scope.
> Phases ILLIMITÉES (regex `Phase [A-Z]+[0-9]?`, README §4) : **11 phases A-K** pilotées
> par le travail réel (sprint très large assumé PO). `Phase 0` = audit gate S76 DÉJÀ JOUÉ
> et CLOS (CONDITIONAL PASS, `52e70d1`) — le plan démarre à Phase A.

---

## §1 État vérifié à l'entrée

| Suite | Count | Commande | Observed |
|---|---|---|---|
| Rust nextest (Win) | 1805 | `cargo nextest run --workspace --locked` | |
| Rust nextest (Docker canonique) | 1809 | `docker run sbfb-ci ... cargo nextest run --workspace --locked` | |
| Rust doctests | ok | `cargo test --workspace --locked --doc` | |
| cargo fmt | 0 diff | `cargo fmt --all --check` | |
| cargo clippy | 0 warnings | `cargo clippy --workspace --all-targets --locked -- -D warnings` | |
| release build | ok | `cargo build -p nexus-shell-daemon --release` | |
| Vitest | 402 | `(cd web && npm run test:unit)` | |
| Vitest coverage | ≥ seuils | `(cd web && npm run test:coverage)` | |
| size-limit | 6/6 | `(cd web && npm run size)` | |
| E2E Playwright hermétique | 39 PASS (13 specs) | `(cd web && npm run test:e2e)` | |
| scan-en-strings | clean | `bash web/scripts/scan-en-strings.sh` | |
| **Total** | **~2260** | | |

---

## §2 Decisions Day 0 (gelées)

| D# | Decision | Implication code |
|---|---|---|
| D1 | Convergence delivery WAN d'abord (diagnostic-puis-fix) | `dispatch_loop.rs`, `result_sync.rs`, `nexus-core-rs/src/docs.rs` (subscribe/sync/download policy), `node.rs` (gossip join doc) |
| D2 | Data plane ALPN `sbfb/shard/1` sur iroh-QUIC `open_bi` long-vécu + `conn.stats()` | `node.rs:385/395` (`extra_protocols` + `ShardProtocol`), nouveau `nexus-core-rs/src/shard.rs` (framing), worker-core shard |
| D3 | Pipeline-parallel + scheduler Parallax 2-phases COMPLET + benchmark 70B 3-5 machines | scheduler `nexus-coordinator-rs`/`nexus-shell-daemon-core` (water-filling + k-medoids RTT + routing DAG + churn actif + perf-map iroh-docs), `gpu/mod.rs` (`vram_free_bytes`), `node.rs` (`conn.stats()`), `ShardPlan`/`ShardAssignment`/`ShardedSessionManifest` `nexus-core-rs` |
| D4 | Vérif COMPLÈTE N0+N1+N2+N3 + incentive curator-reputation (N4 post-S77) | `verification.rs` (N0 remplace L3), `llm/llama_cpp.rs` (extraction top-k), `task.rs:383` `logprobs_hash` réel, `rerun.rs` (N1 VRF/DiFR), `redundancy.rs` (N2 tolérant), nouveau module N3 (commit-reveal + SENTINEL), `validator.rs` (N2 additif), curator-reputation (incentive) |
| D5 | Mode groupe privé `ComputeGroup` allowlist Ed25519 (livrable net-new) | nouveau `nexus-core-rs/src/compute_group.rs`, `canonical.rs` (`DOMAIN_COMPUTE_GROUP_V1`), `node.rs` (rejet handshake), réutilise `invite.rs` |

---

## §3 Graphe de dépendances inter-phases

```
A (convergence WAN) ──┬──> B (data plane ALPN + ComputeGroup)
   [prereq DUR]       │         │
                      │         ├──> C (primitives wire shard + RunProof)
                      │         │         │
                      │         │         ├──> D (scheduler placement: water-fill + k-medoids)
                      │         │         │         │
                      │         │         │         ├──> E (scheduler routing DAG + churn actif)
                      │         │         │         │
                      │         │         └─────────┼──> F (backend shard llama_cpp: bloc couches 70B)
                      │         │                   │         │
                      │         │                   │         ├──> G (N0 TOPLOC fingerprint)
                      │         │                   │         │         │
                      │         │                   │         │         ├──> H (N1 VRF spot-check + incentive)
                      │         │                   │         │         │
                      │         │                   │         │         ├──> I (N2 tolérant + N3 opML/SENTINEL)
                      │         │                   │         │         │
                      └─────────┴───────────────────┴─────────┴─────────┴──> J (front session shard E2E)
                                                                                   │
                                                                                   └──> K (benchmark 70B 3→5 + acceptance + wrap-up)
```

**Dépendances explicites** :
- **A = prérequis DUR** : sans convergence WAN, les sous-tâches de shard n'atteignent pas
  N workers distants → B-I restent hermétiques (2-nœuds local), seule la preuve cross-machine
  (K) dépend de A résolu. Mitigation R1/R4 : B-I avancent en parallèle hermétique.
- **B dépend de A** : le data plane transporte les activations, mais le montage de session
  passe par la propagation des sous-tâches.
- **C dépend de B** : les primitives wire voyagent sur l'ALPN (B) ou le doc (A).
- **D dépend de C** : le scheduler produit un `ShardPlan` (primitive C) ; le placement lit
  `vram_free_bytes` (gpu) + RTT (`conn.stats()` exposé en B).
- **E dépend de D** : le routing DAG + churn opèrent sur le placement (D) ; la perf-map
  republiée nourrit le re-routing.
- **F dépend de C/D** : le backend exécute une `ShardAssignment` (C) selon le placement (D).
- **G dépend de F** : N0 TOPLOC extrait le hidden state produit par le bloc de couches (F).
- **H dépend de G** : N1 VRF spot-check ré-exécute un prefill et compare au fingerprint (G).
- **I dépend de G/F** : N2 compare des fingerprints (G) ; N3 bissecte des commitments
  d'activations (F).
- **J dépend de A-I** : le panneau front expose une session shard montée + son niveau de vérif.
- **K dépend de tout** : le benchmark 70B exerce le pipeline complet bout-en-bout.

> Note (mitigation R1/R4) : Phases B-I sont **testables hermétiquement** (2-nœuds local,
> primitives, scheduler sur fixtures, vérif sur activations fixtures) même si A reste un
> BLOCK PO. Seule la **preuve cross-machine 70B** (K) dépend de A résolu.

---

## §4 Phase A — Convergence delivery WAN (prérequis dur)

### §4.1 Scope
Diagnostiquer puis fixer le bug live S76 : une entrée `task:` créée APRÈS la souscription
d'un worker distant ne se propage pas à sa réplique (`recv:0`, gossip neighborhood non
formé), reproduit LAN + WAN. Prérequis DUR de tout le sharding. Approche rouge-d'abord (D1) :
reproduire le bug en test hermétique 2-nœuds AVANT toute correction, puis fork (fix câblage
SBFB OU BLOCK PO si iroh-intrinsèque).

### §4.2 Livrables
- **Test convergence Rust 2-nœuds** (`nexus-shell-daemon`) : 2 nœuds iroh vrai discovery ;
  nœud A écrit `task:{id}` **incrémentale APRÈS** subscribe de B ; assert B reçoit
  l'entrée (`InsertRemote`) < budget. Rouge d'abord, vert après fix. Germe
  `process_evolution_commit2_handoff.md` l.58-60.
- **Diagnostic cause-racine** : (a) download policy iroh-docs, (b) re-subscribe absent,
  (c) gossip topic du doc non joint, (d) auteur/permission, ou (e) iroh-intrinsèque. Tracé
  dans le commit `## Contexte`.
- **Fix câblage SBFB** (`docs.rs`/`node.rs`/`result_sync.rs`/`dispatch_loop.rs`) selon la
  cause. SI iroh-intrinsèque → BLOCK PO documenté.

### §4.3 Tests plan
1. `convergence_incremental_task_reaches_remote_replica` — entrée `task:` post-subscribe
   propagée < budget (gate cross-machine, rouge→vert prouvé par revert).
2. `convergence_boot_catchup_still_works` — non-régression : le sync initial bulk continue.
3. `convergence_remote_write_visible_to_local_subscriber` — symétrie `result:` inverse.

T1 E2E : `N-A-no-frontend-change` (Phase A daemon-interne).

### §4.4 Critère d'acceptation
`cargo nextest run -p nexus-shell-daemon --locked` vert (3 tests) ; #1 rouge→vert prouvé.
SI BLOCK PO : #1 `#[ignore]` + doc-comment + artefact `b3` documente le stage atteint.

T2 acceptance : la convergence est le prérequis de l'acceptance cross-machine — le harness
`b3_shard_pipeline.sh` (Phase K) doit atteindre au minimum le stage `claim`. Verdict T2
inscrit Phase K.

### §4.5 Commit cible
`feat(daemon): Sprint 77 Phase A — WAN task delivery convergence`
Body 9 sections : Contexte (diagnostic + fork), Fichiers, Delta tests, Vérification §7.4,
Scope cuts respectés, G8 traceability, Pre-launch protocol (0 bump wire), Codex verification,
Carry closure (RE-DRIVE-ON-INGEST si fermé en cascade).

---

## §5 Phase B — Data plane ALPN `sbfb/shard/1` + ComputeGroup admission

### §5.1 Scope
Canal point-à-point des activations (ALPN custom iroh-QUIC) + admission groupe privé
(`ComputeGroup` allowlist Ed25519). Un worker non-allowlisté est rejeté au handshake ALPN
AVANT tout calcul. `conn.stats()` exposé pour la perf-map (D).

### §5.2 Livrables
- **ALPN `sbfb/shard/1`** : `pub const SHARD_ALPN: &[u8] = b"sbfb/shard/1"` (`node.rs`,
  miroir `SEED_ALPN:68`) ; `ShardProtocol` handler via `extra_protocols` (`node.rs:395`).
  `open_bi` long-vécu réutilisé, framing longueur-préfixe.
- **Module `nexus-core-rs/src/shard.rs`** : primitive connexion shard (ouvrir/accepter
  `sbfb/shard/1`, encoder/décoder frame d'activation longueur-préfixe) + exposition
  `conn.stats()` (RTT/jitter).
- **`nexus-core-rs/src/compute_group.rs`** : `ComputeGroup { members: Vec<[u8;32]>, ... }`
  signé Ed25519+JCS (`DOMAIN_COMPUTE_GROUP_V1` additif `canonical.rs`), réutilise `invite.rs`.
  Vérification d'appartenance au handshake (`ShardProtocol::accept` rejette si `peer_pubkey`
  absent de l'allowlist).
- **Absorption P3-D-3** : si le handler ajoute un chemin result-sync avec `seen.remove`
  send-failure, test ciblant la branche récepteur-droppé ; sinon doc-note.

### §5.3 Tests plan
1. `shard_alpn_registered_in_router` — ALPN accepté par le Router.
2. `shard_frame_roundtrip_two_nodes` — 2 nœuds échangent un frame sur `open_bi` persistant.
3. `compute_group_signature_roundtrip` — `ComputeGroup` signé/vérifié, canonical stable.
4. `shard_handshake_rejects_non_member` — worker hors allowlist rejeté AVANT tout frame.
5. `shard_handshake_admits_member` — worker allowlisté ouvre la connexion.
6. `shard_conn_stats_exposes_rtt` — `conn.stats()` RTT lisible (alimente la perf-map D).
7. (si touché) `result_sync_send_failure_unmarks_seen` — P3-D-3.

T1 E2E : `N-A-no-frontend-change`.

### §5.4 Critère d'acceptation
`cargo nextest run -p nexus-core-rs -p nexus-shell-daemon --locked` verts (1-6) ;
canonical-bytes round-trip stable (0 bump wire, `DOMAIN_COMPUTE_GROUP_V1` additif) ; grep
`SHARD_ALPN` présent dans le Router builder.

### §5.5 Commit cible
`feat(core): Sprint 77 Phase B — shard data plane ALPN + private compute group`
Pre-launch : `DOMAIN_COMPUTE_GROUP_V1` net-new additif, `schema_version: 1`.

---

## §6 Phase C — Primitives wire shard + RunProof

### §6.1 Scope
Les primitives décrivant un pipeline de shards (`ShardPlan`/`ShardAssignment`/
`ShardedSessionManifest`/`RunProof`, `remote_user_rnd.md` §10). Candidat absorption T-NN+3
(factoriser le JCS dup) si le code canonical est touché.

### §6.2 Livrables
- **Primitives wire** (`nexus-core-rs/src/shard.rs` ou module dédié) : `ShardPlan` (liste
  ordonnée de `ShardAssignment`), `ShardAssignment { worker_pubkey, layer_start, layer_end,
  ... }`, `ShardedSessionManifest` (signé, `DOMAIN_SHARD_PLAN_V1`), `RunProof` (signé,
  `DOMAIN_RUN_PROOF_V1` : porte les fingerprints N0 + métadonnées d'exécution). `#[serde(default)]`
  runtime-tolerance documenté.

### §6.3 Tests plan
1. `shard_plan_signature_roundtrip` — `ShardedSessionManifest` signé/vérifié, canonical stable.
2. `shard_assignment_serde_roundtrip` — `ShardAssignment` serde stable, `#[serde(default)]`.
3. `run_proof_signature_roundtrip` — `RunProof` signé/vérifié, canonical stable.

T1 E2E : `N-A-no-frontend-change`.

### §6.4 Critère d'acceptation
`cargo nextest run -p nexus-core-rs --locked` verts ; canonical-bytes `DOMAIN_SHARD_PLAN_V1`/
`DOMAIN_RUN_PROOF_V1` round-trip stable.

### §6.5 Commit cible
`feat(core): Sprint 77 Phase C — shard wire primitives + run proof`
Carry closure : T-NN+3 (absorption opportuniste si JCS touché).

---

## §7 Phase D — Scheduler placement (water-filling VRAM + k-medoids RTT)

### §7.1 Scope
Phase 1 du scheduler Parallax : placement DP water-filling de la VRAM libre mesurée +
clustering k-medoids sur matrice RTT pairwise (groupe les couches consécutives entre peers
à faible RTT mutuel) + seuil sharding. Absorbe SYBIL-SEEDER-TAIL (sampling du dial-set).

### §7.2 Livrables
- **Placement water-filling** (`nexus-coordinator-rs`/`nexus-shell-daemon-core`) : DP +
  contrainte VRAM/worker sur `GpuInfo.vram_free_bytes` mesuré (pas déclaré) + contrainte de
  lien. Seuil sharding : ne shard que si `VRAM_modèle > VRAM_max_worker`, sinon endpoint
  federation.
- **k-medoids RTT** : clustering empirique sur matrice RTT pairwise mesurée (`conn.stats()`
  exposé en B), pas de géoIP central (anti-recentralisation).
- **Absorption SYBIL-SEEDER-TAIL** : le sampling anti-Sybil du tail seeder traité dans la
  sélection des workers candidats du dial-set ; test couvrant le sampling OU doc-note.

### §7.3 Tests plan
1. `placement_water_fills_vram_free` — répartit les couches selon `vram_free_bytes` (3-5
   workers de VRAM libre distincte → blocs proportionnels).
2. `placement_refuses_when_model_fits_single_worker` — seuil : `VRAM_modèle ≤ VRAM_max` →
   endpoint federation (pas de shard).
3. `kmedoids_groups_low_rtt_consecutive_layers` — k-medoids groupe les couches consécutives
   entre peers à faible RTT pairwise.
4. `placement_handles_5_workers_70b` — placement valide d'un 70B Q4 sur 5 shards.
5. `sybil_seeder_tail_sampling_*` — sampling anti-Sybil dans la sélection candidats OU doc-note.

T1 E2E : `N-A-no-frontend-change`.

### §7.4 Critère d'acceptation
`cargo nextest run -p nexus-core-rs -p nexus-coordinator-rs --locked` verts ; le scheduler
produit un `ShardPlan` valide pour 3-5 shards sur des `vram_free_bytes` + matrice RTT fixtures.

### §7.5 Commit cible
`feat(core): Sprint 77 Phase D — Parallax placement scheduler (water-filling + k-medoids)`
Carry closure : SYBIL-SEEDER-TAIL (absorbé).

---

## §8 Phase E — Scheduler routing DAG + churn re-balancing actif

### §8.1 Scope
Phase 2 du scheduler Parallax + churn Petals. Routing DAG layer-indexé (sweep DP G→D
relaxation Parallax) + re-équilibrage ACTIF sur churn (`replace_failed_server` O(t) + heap
fallback + cache activations client-side) + perf-map (rho, tau) republiée 1-2s iroh-docs.

### §8.2 Livrables
- **Routing DAG** : DAG layer-indexé + 1 sweep DP G→D (`dp2(l+1,g') = min(..., dp2(l,g) +
  rho<g,g'> + tau<g',l+1>)`), O(L·R²) négligeable à 3-5 peers.
- **Churn actif** : heap de fallback ordonné par latence + cache client-side d'activations +
  `replace_failed_server` O(t) (PAS le « clé DHT expire » de Parallax, faille churn).
- **Perf-map** : (rho = RTT one-way/paire, tau = latence profilée couche/GPU) republiée
  toutes 1-2s dans iroh-docs (raw-op, 0 bump feed version).

### §8.3 Tests plan
1. `routing_dag_sweep_selects_min_latency_chain` — le sweep DP choisit la chaîne de latence
   minimale sur perf-map fixture.
2. `churn_replaces_failed_server_oturn` — un worker drop → `replace_failed_server` re-route
   via le heap fallback, l'inférence continue (cache activations).
3. `perf_map_republished_to_doc` — la perf-map (rho, tau) est écrite/relue sur iroh-docs.
4. `routing_recomputed_on_perf_map_update` — un changement de perf-map déclenche un re-routing.

T1 E2E : `N-A-no-frontend-change`.

### §8.4 Critère d'acceptation
`cargo nextest run -p nexus-coordinator-rs --locked` verts ; routing recalculé sur
worker-drop ; perf-map propagée sur le doc.

### §8.5 Commit cible
`feat(core): Sprint 77 Phase E — DAG routing + active churn rebalancing`

---

## §9 Phase F (= Phase F1 + Phase F2) — Backend shard FORKÉ `llm_llama_cpp` (fork llama.cpp, layer-subset + chargement partiel)

> **RE-CADRÉ 2026-06-21** après DESIGN-CONFLICT du préflight Phase F + arbitrage PO
> option (a) fork + **spike de faisabilité GO** (cf. `sprint77_phase_f_preflight.md`
> §Résolution + `sprint77_phase_f_spike.md`). Le livrable original « forward partiel via
> le wrapper safe `llama-cpp-2` » est **INFAISABLE** (aucun eval-callback / layer-range /
> injection dans l'API safe) ; remplacé par le fork **prouvé** bit-exact sur CUDA + Metal.
> Amendement PO : cible ~20 Go sur 5080(16Go)+Mac(8Go) → **P-D chargement partiel OBLIGATOIRE**.
>
> **SPLIT F1/F2 2026-06-21** (préflight `sprint77_phase_f1_preflight.md`, verdict PLAN-ADAPT) :
> la Phase F est re-coupée en **F1** (vendor+patch+P-D+backend Rust+build CUDA/Metal+tests) et
> **F2** (claim ComputeGroup+cap VRAM fail-closed+vérif sig manifest, câblage `sbfb/shard/1`,
> threat §16). Suffixe chiffré sanctionné README §4 → **0 renumérotation** de G–K. Tout reste
> dans S77 (pas un defer). §9.1–9.4 ci-dessous décrivent F1 ; les livrables claim/wiring/threat
> sont portés en F2.

### §9.1 Scope
Forker llama.cpp (vendoré par `llama-cpp-2`) pour exécuter un sous-ensemble contigu de
couches `[layer_start, layer_end)`, **injecter** le hidden state amont (`llama_batch.embd`,
déjà câblé) et **extraire** le hidden de frontière. Spike validé : patch minimal
backend-agnostique, coupe bit-exact CPU/CUDA-sm120/Metal-M2 + cross-backend cosine 0.999
sur Mistral-7B Q4. Le worker shard claim une `ShardAssignment` et ne charge QUE ses couches.

### §9.2 Livrables
- **Fork llama.cpp patché** (vendoré via fork de `llama-cpp-sys-2` — submodule patché OU
  source override ; patch minimal backend-agnostique) :
  - API partial-decode : `shard_layer_start/end/is_first/is_last` via `llama_context_params`
    (AVANT le reserve des buffers) ; boucle bornée `[start,end)` ; gather `inp_out_ids` au
    dernier layer **exécuté** de chaque shard ; sortie `is_last ? (norm+lm_head) : résiduel brut`.
  - **P-D chargement partiel** (`TENSOR_SKIP` : chaque nœud n'alloue/ne lit QUE les couches
    de son shard depuis le GGUF — **OBLIGATOIRE** pour qu'un 20 Go tienne sur 16+8).
  - extraction top-k du dernier hidden state (matériel TOPLOC N0, encodage réel = Phase G).
  - Patch par **architecture** : builder LLAMA d'abord (couvre Llama/Mistral/Mixtral) ;
    autres archs (gemma…) à part selon la cible de démo.
- **Backend Rust** (`nexus-worker-core/src/llm/llama_cpp.rs` étendu, feature-gated
  `llm_llama_cpp`) : charge `layer_start..layer_end` via le fork, forward partiel, hidden
  state transmis aval via `sbfb/shard/1`. **Prérequis build : LLVM/libclang** (`LIBCLANG_PATH`,
  requis par bindgen pour `llama-cpp-sys-2`).
- **Worker shard claim** (`nexus-worker-core/src/engine/runtime.rs`) : claim une
  `ShardAssignment` (filtre `ComputeGroup` + **cap VRAM fail-closed** sur `GpuStats.vram_free_bytes`
  mesuré — VRAM déjà snapshotée, PAS de nouvelle pompe live, scope cut #7 respecté ; +
  vérif signature `ShardedSessionManifest` côté dialer), connexion `sbfb/shard/1` amont/aval.
- **Build vert CUDA ET Metal** (les 2 backends du rig réel) matérialisé tôt (R2).

### §9.3 Tests plan
1. `shard_backend_loads_layer_subset` (`#[ignore]`-gated, GGUF) — P-D : charge SEULEMENT
   layer_start..layer_end (VRAM réduite), hidden state de la bonne forme.
2. `shard_backend_partial_equals_full` (`#[ignore]`-gated, GGUF) — preuve spike portée en
   test : `decode([0,k))+inject+decode([k,L)) == decode([0,L))` (bit-exact même backend).
3. `shard_backend_hidden_state_extractable` (`#[ignore]`-gated) — top-k extractible (prérequis N0).
4. `shard_assignment_claim_respects_group` (hermétique) — claim ssi dans `ComputeGroup` + sous caps VRAM.
5. `shard_backend_primitive_*` (hermétique, sans GGUF) — découpage layer-range + format hidden state (CI).

T1 E2E : `N-A-no-frontend-change`.

### §9.4 Critère d'acceptation
`cargo nextest run -p nexus-worker-core --locked` vert (hermétiques CI) ; `cargo build -p
nexus-worker --features llm_llama_cpp` réussit **sur CUDA ET Metal** ; tests `#[ignore]` GGUF
runnable localement (P-D + partial==full). Cross-backend CUDA↔Metal cosine > 0.99 (spike-validé,
calibre le seuil TOPLOC N0). Le fork est jamais-CI (R2) → double test (primitive hermétique + GGUF).

### §9.5 Commit cible
`feat(worker): Sprint 77 Phase F — forked layer-block execution backend (partial load)`
Phase LARGE (fork + P-D + claim + wiring) : le préflight de F finalisera un éventuel split
en sous-phases atomiques (renumérotation G→… traitée alors). Risk R2 tracé.

---

## §10 Phase G — Vérification N0 TOPLOC fingerprint

### §10.1 Scope
N0 TOPLOC (LSH top-k hidden state) remplace le Layer3 logprob inerte de `verification.rs`.
`task.rs:383` `logprobs_hash` devient le slot réel.

### §10.2 Livrables
- **N0 TOPLOC** (`nexus-core-rs/src/verification.rs`) : encodage fingerprint (LSH top-k
  k=128 du dernier hidden state, encodage polynomial 258 B/32 tok) + comparaison seuil
  global ; remplace `LayerResult logprobs` L3. `logprobs_hash` slot TOPLOC réel (0 bump wire).
- Worker côté production produit le fingerprint TOPLOC après son bloc de couches (porté dans
  `RunProof`).

### §10.3 Tests plan
1. `toploc_fingerprint_encode_decode_roundtrip` (hermétique, CI) — encodage/décodage sur
   activations fixtures.
2. `toploc_detects_model_swap` (hermétique, CI) — fingerprint d'un modèle différent détecté
   (propriété cœur TOPLOC).
3. `toploc_accepts_same_model_within_threshold` (hermétique, CI) — même modèle (variation FP)
   sous le seuil.

T1 E2E : `N-A-no-frontend-change`.

### §10.4 Critère d'acceptation
`cargo nextest run -p nexus-core-rs --locked` verts (1-3 CI) ; la primitive TOPLOC détecte
le swap modèle sur fixtures.

### §10.5 Commit cible
`feat(core): Sprint 77 Phase G — N0 TOPLOC fingerprint`

---

## §11 Phase H — Vérification N1 spot-check VRF + incentive curator-reputation

### §11.1 Scope
N1 spot-check : vérifieur tiré par VRF Ed25519 (prefill-only VeriLLM/DiFR, one-honest-verifier,
randomise temp+seed). Incentive curator-reputation (kudos réputationnel non-monétaire) +
mapping criticité→niveau. Candidat absorption MEDIAN-DE-GROUPE (l'incentive touche le scoring).

### §11.2 Livrables
- **N1 VRF spot-check** (`nexus-core-rs/src/rerun.rs` étendu) : VRF Ed25519 tire un vérifieur
  déterministe ; prefill-only re-exécution (~1%) comparée au fingerprint N0 ; **randomise
  temperature ET seed** (faille DiFR).
- **Incentive curator-reputation** : le vérifieur VRF gagne du kudos réputationnel
  (non-monétaire, non-transférable) pour un spot-check honnête, via le mécanisme curator/
  reputation existant. **Note honnête** : mitigation réputationnelle, PAS garantie économique.
- **Mapping criticité→niveau** (addendum §3) : haute-criticité = N2 obligatoire ;
  faible-criticité = N0 seul ; N1 échantillonnage VRF 1-5% ; N3 sur litige.

### §11.3 Tests plan
1. `n1_vrf_selects_deterministic_verifier` — la VRF tire un vérifieur déterministe et
   vérifiable (one-honest-verifier).
2. `n1_spot_check_randomizes_temp_and_seed` — le spot-check randomise temp+seed (anti-faille
   DiFR).
3. `incentive_credits_reputation_on_honest_spotcheck` — un spot-check honnête crédite du
   kudos réputationnel (non-monétaire).
4. `criticality_maps_to_verification_level` — le mapping criticité→niveau (N0/N1/N2/N3) est
   correct.

T1 E2E : `N-A-no-frontend-change`.

### §11.4 Critère d'acceptation
`cargo nextest run -p nexus-core-rs -p nexus-coordinator-rs --locked` verts (1-4) ; la VRF
est déterministe et vérifiable ; l'incentive crédite réputationnel (jamais monétaire).

### §11.5 Commit cible
`feat(core): Sprint 77 Phase H — N1 VRF spot-check + reputation incentive`
Carry MEDIAN-DE-GROUPE (candidat absorption si l'incentive touche le scoring).

---

## §12 Phase I — Vérification N2 redondance tolérante + N3 bissection opML/SENTINEL

### §12.1 Scope
N2 redondance tolérante (fingerprint, pas hash byte ; quorum exact INCHANGÉ). N3 bissection
opML sur contestation (commit-reveal activations par-frontière + SENTINEL EMA inter-stages).

### §12.2 Livrables
- **N2 redondance tolérante** (`nexus-core-rs/src/redundancy.rs` + `validator.rs` chemin
  ADDITIF) : M-of-N, comparaison fingerprint TOLÉRANT. Le quorum result_text exact existant
  (`validate_quorum_pre_guardrail`) reste INCHANGÉ (N2 est un nouveau chemin pour shard).
- **N3 bissection opML** (nouveau module) : commitments d'activations par-frontière (ancres
  iroh-docs + commit-reveal Ed25519, `DOMAIN_ACTIVATION_COMMIT_V1`, PAS de smart-contract) ;
  **SENTINEL** (EMA inter-stages) localise le stage corrompu. O(1 bloc).

### §12.3 Tests plan
1. `n2_tolerant_quorum_accepts_close_fingerprints` — N2 accepte 2 fingerprints proches.
2. `n2_tolerant_quorum_rejects_divergent` — N2 rejette des fingerprints divergents.
3. `validator_exact_quorum_unchanged` — quorum result_text exact INCHANGÉ (`git diff` = 0
   ligne hors N2 additif).
4. `n3_activation_commit_reveal_roundtrip` — commit-reveal d'activation par-frontière
   (canonical stable, `DOMAIN_ACTIVATION_COMMIT_V1`).
5. `n3_sentinel_localizes_corrupted_stage` — SENTINEL EMA localise un stage corrompu sur
   fixture (bissection O(1 bloc)).

T1 E2E : `N-A-no-frontend-change`.

### §12.4 Critère d'acceptation
`cargo nextest run -p nexus-core-rs -p nexus-coordinator-rs --locked` verts (1-5) ; N2
accept/reject ; N3 localise un stage corrompu ; quorum exact prouvé inchangé.

### §12.5 Commit cible
`feat(core): Sprint 77 Phase I — N2 tolerant redundancy + N3 opML bisection + SENTINEL`
Invariant « validator exact inchangé » prouvé.

---

## §13 Phase J — Front session shard + UX intentions

### §13.1 Scope
Panneau front pour démarrer/rejoindre une session shard (groupe privé), en intentions
utilisateur (pas de jargon `shard`/`ALPN`/`ComputeGroup` en CTA — PO-9/UX). Spec T1 E2E.

### §13.2 Livrables
- **Panneau session shard** (`web/src/`) : groupe de calcul privé (membres, statut pipeline,
  niveau de vérif), CTA « Rejoindre un groupe de calcul » / « Lancer un gros modèle en
  réseau ». Pas de jargon technique en surface.
- **Route daemon read-only** (`http.rs`) : `GET /api/daemon/shard-session/{id}` → statut
  (additif, auth loopback, 0 bump wire).
- **Spec T1** `web/e2e/compute-shard.spec.ts` (hermétique, vrai daemon Playwright) : charge
  le panneau session shard, rendu FR + état vide ; scénario tag `@shard` env-gated pour la
  partie cross-machine (miroir `compute-tester`).
- scan-en-strings clean (FR).

### §13.3 Tests plan
1. `ShardSessionPanel.test.tsx` (Vitest) — rendu, état vide, intentions FR.
2. `shard-session.api.test.ts` (Vitest) — `GET /api/daemon/shard-session` mocké, Zod `.strict()`.
3. **T1** `web/e2e/compute-shard.spec.ts` (Playwright hermétique) : panneau rendu contre le
   vrai daemon, état vide, intentions FR byte-exact. `npm run test:e2e`. BLOQUANT-vert
   wrap-up + CI.

T1 E2E : **`GREEN`** attendu — `compute-shard.spec.ts` couvre la surface front nouvelle (PAS
`N-A` : Phase J ajoute du frontend).

### §13.4 Critère d'acceptation
`(cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json && npm run test:unit &&
npm run test:coverage && npm run build && npm run size && bash scripts/scan-en-strings.sh)`
verts ; `(cd web && npm run test:e2e)` inclut `compute-shard.spec.ts` GREEN.

### §13.5 Commit cible
`feat(web): Sprint 77 Phase J — shard session panel + hermetic E2E`
T1 spec nommée. Route additive 0-bump.

---

## §14 Phase K — Benchmark ~20 Go sur le rig réel (RTX 5080 + Mac M2) + acceptance + wrap-up

> **RE-CADRÉ 2026-06-21 (amendement PO).** Le **70B sur 3-5 machines est ABANDONNÉ** (hors
> de portée du rig réel : 16 Go + 8 Go ≈ 24 Go). Cible = un modèle **~20 Go arch-llama**
> éclaté sur **2 machines hétérogènes : RTX 5080 (CUDA) + Mac M2 (Metal)**, chacune ne
> chargeant QUE ses couches (P-D). Le modèle ne tient sur AUCUNE machine seule → prouve la
> vraie valeur du sharding (et le pipeline hétérogène, déjà validé au spike cross-backend).

### §14.1 Scope
La preuve falsifiable. Le bring-up cross-backend est **déjà fait** au spike
(`sprint77_phase_f_spike.md` : CUDA↔Metal prouvé cosine 0.999 sur Mistral-7B Q4, hand-off
fichier). Benchmark réel : un modèle **~20 Go** (ex. Mixtral-8x7B Q3 ~20 Go MoE, ou un 34B Q4
dense, arch llama) éclaté **5080↔Mac via `sbfb/shard/1`** (transport réseau temps-réel, plus
le fichier du spike), gate réseau GO/NO-GO produit + test worker-drop. Plus le wrap-up
canonique. C'est le **gate produit** du sprint phare. Placement = scheduler Phase D
(water-filling sur VRAM libre mesurée : ~2/3 des couches sur la 5080, ~1/3 sur le Mac).

### §14.2 Livrables
- **Harness acceptance** `scripts/acceptance/b3_shard_pipeline.sh` (étend `b3_live_pc_vps.sh`) :
  monte une session shard, soumet un prompt, mesure TTFT + tok/s + RTT/frontière, vérifie le
  `RunProof` (fingerprints N0-N3) de chaque shard, teste un worker-drop (churn). Artefact JSON
  `{status, stage, model, n_shards, ttft_s, toks_per_s, rtt_frontier_ms, run_proof, diagnosis,
  last_response}` ; exit PASS=0 / BLOCK=1 / RIG-ABSENT=3. Gate réseau : `BLOCK{rtt>80ms}` ou
  `BLOCK{relay-hot-path}` = NO-GO produit, pas un timeout à rallonger. Cible : un modèle
  **~20 Go arch-llama** éclaté **5080↔Mac M2** (2 shards, P-D, placement Phase D water-filling).
- **`sprint77_verification.md`** : fail-fast (Observed) + section `## Acceptance` (T1 GREEN +
  T2 du benchmark : PASS / BLOCK{diag} / RIG-ABSENT). Si DIFFERE-matériel honnête (5 PC + GPU)
  → PROVISIONAL + carry P1 (jamais DIFFERE en prose).
- **`sprint78_audit_plan.md`** : tracks Phase 0 S78 + **Track Testabilité standing** (audit
  gate S78 vérifie spec T1 `compute-shard.spec.ts` + statut CI + artefact JSON T2).
- **Docs longue-vie** : THREAT_MODEL **§16 (nouvelle section sharding** : SI-1/SI-3/SI-4/SI-5
  + mode groupe privé + caveat confidentialité + incentive-vérif réputationnel sev M) ;
  PATTERNS rust (ALPN shard, scheduler Parallax, TOPLOC/N1/N2/N3, perf-map) + shell ;
  SPRINT_LOG row S77 ; CLAUDE.md (0-77 CLOSED, Arc 3.5 compute clos) ; roadmap_v5 livraison.
- **Re-check invariant clôture** : tout nom de test cité dans `verification.md` §6 grep-résout
  à une fn `#[test]`.

### §14.3 Tests plan
Pas de nouveau code fonctionnel (wrap-up) ; la fail-fast ré-établit la baseline + exerce le
benchmark. `b3_shard_pipeline.sh` `bash -n` clean.

### §14.4 Critère d'acceptation
Fail-fast `verification.md` toutes rows vertes (Win nextest + Docker canonique + web + E2E).
**T1** : `compute-shard.spec.ts` GREEN (BLOQUANT). **T2** : `b3_shard_pipeline.sh` produit un
artefact JSON — `PASS` (benchmark ~20 Go sur 5080+Mac < gate réseau, tok/s ≥ 1) OU `BLOCK{diagnosis}` OU
`RIG-ABSENT` (matériel absent). Si BLOCK/RIG-ABSENT → la feature shard reste **PROVISIONAL +
carry P1** vers S78. Verdict T2 = champ JSON `status`, JAMAIS `DIFFERE-materiel` en prose.

`scripts/acceptance/b3_shard_pipeline.sh` → `status` dans {`PASS`, `BLOCK{diagnosis}`,
`RIG-ABSENT`, `N-A-no-cross-machine-feature`} (S77 EST cross-machine, donc pas N-A).

### §14.5 Commit cible
`feat(daemon): Sprint 77 Phase K — 70B shard benchmark acceptance + wrap-up`
Carry closure (RE-DRIVE-ON-INGEST selon D1, invariant clôture). G8 complet. Acceptance JSON
status (pas prose).

---

## §15 Delta tests estimé

| Phase | Rust | Vitest | Détail |
|---|---|---|---|
| A | +3 | +0 | convergence 2-nœuds (incremental/boot-catchup/remote-visible) |
| B | +6 | +0 | ALPN registered, frame roundtrip, compute_group sig, handshake admit/reject, conn.stats, (P3-D-3) |
| C | +3 | +0 | shard_plan sig, shard_assignment serde, run_proof sig |
| D | +5 | +0 | water-fill VRAM, refuse-single, k-medoids RTT, 5-workers-70b, sybil sampling |
| E | +4 | +0 | DAG sweep min-latency, churn replace-failed, perf-map republish, routing-recompute |
| F1 | +5 | +0 | hermétiques CI : ShardWindow valide/end-0/rejette + top_k largest/clamp-NaN (5). Plus 3 `#[ignore]` GGUF (load-subset P-D, partial==full bit-exact spike porté, hidden-extract) — non comptés CI, **3/3 prouvés local sur Mistral-7B-Q4**. Fork llama.cpp build CUDA+Metal vert |
| F2 | +1 | +0 | claim-respects-group+VRAM (hermétique CI) |
| G | +3 | +0 | TOPLOC encode/decode, detect-swap, accept-within-threshold |
| H | +4 | +0 | VRF deterministic verifier, randomize temp+seed, reputation credit, criticality mapping |
| I | +5 | +0 | N2 accept/reject, validator-exact-unchanged, N3 commit-reveal, SENTINEL localize-stage |
| J | +0 | +3 | ShardSessionPanel, shard-session api, + 1 spec E2E `compute-shard.spec.ts` (hors src/, hors count Vitest) |
| K | +0 | +0 | wrap-up (pas de net-new fonctionnel) |
| **Total** | **+37** | **+3** | (+1 spec E2E hors count) |
| **Sortie estimée** | **~1842** (Win) / **~1846** (Docker) | **~405** | **~2300** |

> Estimation indicative (anti-faux-vert au wrap-up : le git-count des `#[test]` ajoutés doit
> égaler chaque delta, audit S76 invariant). Tests `#[ignore]`-gated F comptés au `nextest
> list` mais non exécutés en CI (GGUF absent) — documentés runnable localement.

---

## §16 Fail-fast checklist

| # | Check | Commande | Critère | Observed |
|---|---|---|---|---|
| 1 | fmt | `cargo fmt --all --check` | 0 diff (Win 1.95 + Docker 1.94) | |
| 2 | clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | |
| 3 | nextest (Win) | `cargo nextest run --workspace --locked` | ~1842 0-skip | |
| 4 | doctests | `cargo test --workspace --locked --doc` | 0 | |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | OK | |
| 6 | nextest (Docker canonique) | `docker run sbfb-ci ... cargo nextest run --workspace --locked` | ~1846 0-skip | |
| 7 | web tsc | `(cd web && npx tsc --noEmit -p tsconfig.app.json)` | 0 | |
| 8 | web lint | `(cd web && npm run lint)` | 0 err | |
| 9 | web Vitest | `(cd web && npm run test:unit)` | ~405 | |
| 10 | web coverage | `(cd web && npm run test:coverage)` | ≥ seuils | |
| 11 | web build+size | `(cd web && npm run build && npm run size)` | 6/6 | |
| 12 | scan-en-strings | `bash web/scripts/scan-en-strings.sh` | clean | |
| 13 | scan-trust-wording (si présent) | `bash web/scripts/scan-trust-wording.sh` | clean / N-A | |
| 14 | A convergence incrémentale | `cargo nextest run -p nexus-shell-daemon -E 'test(convergence_incremental)'` | PASS (rouge→vert prouvé) | |
| 15 | A boot catch-up non-régression | `... -E 'test(convergence_boot_catchup)'` | PASS | |
| 16 | B ALPN shard registered | `... -E 'test(shard_alpn_registered)'` | PASS | |
| 17 | B frame roundtrip 2-nœuds | `... -E 'test(shard_frame_roundtrip)'` | PASS | |
| 18 | B compute_group signature | `... -E 'test(compute_group_signature)'` | PASS | |
| 19 | B handshake rejette non-membre | `... -E 'test(shard_handshake_rejects)'` | PASS | |
| 20 | C shard_plan + run_proof signature | `... -E 'test(shard_plan_signature) + test(run_proof_signature)'` | PASS | |
| 21 | D placement water-fill VRAM | `... -E 'test(placement_water_fills_vram)'` | PASS | |
| 22 | D seuil refuse single-worker | `... -E 'test(placement_refuses_when_model_fits)'` | PASS | |
| 23 | D k-medoids RTT grouping | `... -E 'test(kmedoids_groups_low_rtt)'` | PASS | |
| 24 | D placement 70B 5 workers | `... -E 'test(placement_handles_5_workers_70b)'` | PASS | |
| 25 | D SYBIL-SEEDER-TAIL sampling | `... -E 'test(sybil_seeder_tail)'` | PASS / doc-note | |
| 26 | E routing DAG min-latency | `... -E 'test(routing_dag_sweep)'` | PASS | |
| 27 | E churn replace-failed actif | `... -E 'test(churn_replaces_failed_server)'` | PASS | |
| 28 | E perf-map republished | `... -E 'test(perf_map_republished)'` | PASS | |
| 29 | F backend layer-subset (#[ignore]) | `... --features llm_llama_cpp -E 'test(shard_backend_loads_layer_subset)' --run-ignored` | runnable local (GGUF) | |
| 30 | F primitive shard hermétique | `... -E 'test(shard_backend_primitive)'` | PASS (CI) | |
| 31 | G TOPLOC détecte swap (CI) | `... -E 'test(toploc_detects_model_swap)'` | PASS | |
| 32 | H N1 VRF vérifieur déterministe | `... -E 'test(n1_vrf_selects_deterministic_verifier)'` | PASS | |
| 33 | H N1 randomise temp+seed | `... -E 'test(n1_spot_check_randomizes)'` | PASS | |
| 34 | H incentive réputationnel (non-monétaire) | `... -E 'test(incentive_credits_reputation)'` | PASS (jamais monétaire) | |
| 35 | I N2 tolérant accept/reject | `... -E 'test(n2_tolerant_quorum)'` | PASS | |
| 36 | I validator exact INCHANGÉ | `git diff --stat validator.rs` (quorum exact) | 0 ligne hors N2 additif | |
| 37 | I N3 bissection localise stage | `... -E 'test(n3_sentinel_localizes)'` | PASS | |
| 38 | 0 bump wire | grep `*_FORMAT_VERSION`/`*_ANNOUNCEMENT_VERSION`/`SCHEMA_VERSION` | tous = 1 ; nouveaux `DOMAIN_*` additifs | |
| 39 | **T1 E2E shard hermétique** | `(cd web && npm run test:e2e)` (`web/e2e/compute-shard.spec.ts`) | **GREEN** (BLOQUANT wrap-up + CI) | |
| 40 | **T2 acceptance benchmark ~20Go (5080+Mac) JSON** | `bash scripts/acceptance/b3_shard_pipeline.sh` | `status` ∈ {`PASS`,`BLOCK{diag}`,`RIG-ABSENT`} (jamais prose) | |
| 41 | THREAT_MODEL §16 sharding écrit | `test -f` + grep `§16` SI-1..SI-5 + incentive | présent | |
| 42 | verification.md écrit | `test -f .planning/active/sprint77_verification.md` | présent + `## Acceptance` | |
| 43 | sprint78_audit_plan.md écrit | `test -f .planning/active/sprint78_audit_plan.md` | présent + Track Testabilité | |
| 44 | invariant clôture noms tests | grep noms cités §6 → fn `#[test]` | tous grep-résolus | |

> Colonne `Observed` vide au plan, remplie au `verification.md`.

---

## §17 Scope cuts

(Reprise exhaustive depuis kickoff §7. Les ex-#1/#2/#3 sont devenus des LIVRABLES sur
arbitrage PO scope maximal.)

| # | Item | Sprint cible | Rationale |
|---|---|---|---|
| 1 | N4 zkML (DeepProve/NANOZK preuve ZK) | post-S77 (Gate-4+) | PO a demandé N1/N3, PAS N4 ; 50-15000× overhead prohibitif |
| 2 | Tensor-parallel mono-machine 2-GPU | ENTERRÉ | « personne n'a 2 GPU » + all-reduce LAN-only |
| 3 | Streaming token-par-token interactif (chat live) | jamais (NO-GO) | 1-3 tok/s WAN = chat live NON VIABLE |
| 4 | Confidentialité face aux workers | jamais (limite physique) | pas de TEE GPU consumer 2026 ; activations en clair |
| 5 | KV-cache distribué / activation cache O(t) gros contexte | post-S77 | explose sur gros contexte ; KV-cache local + cache churn borné |
| 6 | Quant blockwise dynamique des activations | post-S77 | optimisation BP post-preuve benchmark 70B |
| 7 | VRAM-live admission runtime (pompe `gpu.snapshot()`) | post-S77 | scheduler lit `vram_free_bytes` au PLACEMENT ; pompe runtime garde `estimated_*` |
| 8 | Mode public / découverte ouverte de groupes | jamais (R-iroh-audit P0) | groupe privé explicite, zéro worker anonyme |
| 9 | Upgrade iroh 0.98 → 1.0 | Gate-1/PO | Day-0 gelé ; sauf BLOCK D1 (convergence exige 1.0) |
| 10 | `execute_build` LT-7 câblage | post-S77 | orthogonal au sharding inférence |
| 11 | Garantie économique de l'incentive-à-vérifier (stake/token) | jamais (décision gelée) | kudos non-monétaire PO-12 interdit le stake ; incentive S77 = réputationnel (mitigation, pas garantie) |
| 12 | Reconnaissance contributeur publique des shards | post-launch | orthogonal ; reconnaissance publique = post R-iroh-audit |
| 13 | AWQ/GPTQ/EXL2 | rejeté | GGUF Q4 retenu pour le sizing shard |
| 14 | Push live origin/master (`4da9800` déjà poussé) | décision PO hors-sprint | push = décision opérateur (LT-2/Radicle hors-sprint) |

---

## §18 Risks

(Reprise depuis kickoff §9.)

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Convergence WAN intrinsèque iroh non fixable sans 1.0 → sharding BLOQUÉ | Medium | High | Diagnostic rouge-d'abord + fork PO ; B-I avancent hermétique en parallèle |
| R2 | Backend `llm_llama_cpp` jamais-CI ne build/extrait pas → N0/N1 infaisables | Medium | High | Double test (primitive CI + intégration GGUF) ; build tôt Phase F ; dégrade N2 + doc-note |
| R3 | Perf benchmark 70B < 2 tok/s sur 3-5 machines → NO-GO produit | Medium | Medium | Gate réseau RTT ; verdict 1-3 tok/s écrit ; bring-up séquencé (toy→70B/3→70B/5) ; PROVISIONAL si DIFFERE |
| R4 | **Sprint TRÈS LARGE assumé PO (11 phases)** → débordement / qualité par phase | High | Medium | Scope maximal assumé PO (override G1) ; budget ouvert ; phases découplées ; 1 commit atomique/phase ; sous-découpe (L, M…) plutôt que bâcler |
| R5 | `ComputeGroup` + 4 `DOMAIN_*` net-new → large surface crypto Ed25519+JCS | Medium | High | Réutilise crypto M19 ; rejet handshake ; tests + Codex gate par primitive ; T-NN+3 absorbé |
| R6 | iroh `TransportConfig` 0.98 ne permet pas le tuning streams promis | Low | Medium | `cargo doc` AVANT promesse ; `open_bi` de base confirmé context7 ; `conn.stats()` RTT à valider résidentiel |
| R7 | Acceptance benchmark 70B DIFFERE-matériel (3-5 machines + GPU 16+ GB) | High | Low | Harness JSON runnable ; PROVISIONAL+carry P1 honnête ; CI couvre convergence + primitives vérif + spike toy |
| R8 | **Incentive-à-vérifier non-garanti économiquement** : lazy verifier rationnel ne vérifie pas | Medium | Medium | Mitigation réputationnelle (kudos curator-reputation VRF) + note honnête ; pilote fermé (D5) + anti-Sybil amont bornent ; THREAT_MODEL §16 sev M ; N4 zkML (garantie crypto) post-S77 |

---

## §19 Checkpoint de clôture

Conditions binaires pour dire « S77 fermé » :
- [ ] 44/44 fail-fast verts (Win + Docker + web + E2E) — OU PROVISIONAL documenté pour #40
      si DIFFERE-matériel (carry P1 S78)
- [ ] 12 commits feat (A-K, F=F1+F2) (wrap-up intégré Phase K)
- [ ] `sprint77_verification.md` écrit + section `## Acceptance` (T1 GREEN + T2 status JSON)
- [ ] `sprint78_audit_plan.md` écrit + Track Testabilité standing
- [ ] THREAT_MODEL §16 sharding (SI-1..SI-5 + incentive) + PATTERNS rust/shell mis à jour
- [ ] Invariant clôture : tout nom de test cité §6 grep-résout à une fn `#[test]`
- [ ] 0 bump wire (versions = 1, nouveaux `DOMAIN_*` additifs : compute_group, shard_plan,
      run_proof, activation_commit)
- [ ] Validator quorum exact result_text prouvé INCHANGÉ (N2 additif seul)
- [ ] Memory `nexus_grid_pivot.md` à jour (tip + compteurs + carries S78)
- [ ] SPRINT_LOG.md row S77 ajoutée + CLAUDE.md 0-77 CLOSED
- [ ] D1 fork tranché : fix livré OU BLOCK PO documenté (test convergence vert ou `#[ignore]`
      justifié)
- [ ] Gate produit GO/NO-GO inscrit (benchmark 70B mesuré ou PROVISIONAL honnête)
- [ ] Incentive-à-vérifier conçu (curator-reputation) + note honnête (mitigation, pas garantie)

---

## §20 — AVENANT 2026-06-23 : 3 phases documentation (L, M, N)

> **Directive PO 2026-06-23.** Le coeur sharding (S77 A-K) est livré, mais la feature n'est
> pas « ultra-complète » sans sa **documentation d'usage**. On ajoute **3 phases atomiques**
> L/M/N livrant (1) le contrat machine-lisible, (2) la doc humaine FR, (3) la couche
> agent-consommable — couvrant **« comment ça marche »** + **« comment câbler un projet
> du protocole pour utiliser la feature »**, pour un lecteur **humain** ET pour un **LLM/agent**.
> Cadrage produit par Workflow ultracode (4 lecteurs Opus 4.8 1M + synthèse) ancré dans le
> code réel. Chaque phase a un **gate d'acceptation vérifiable non-prose** (testabilité
> par-sprint respectée — docs NON exemptées). Honnêteté **PROVISIONAL/S78 enforced
> mécaniquement** (grep des marqueurs), pas seulement promise.

**Ordre imposé L → M → N** (single-source-of-truth) : les schémas générés (L) sont la vérité
machine ancrée au code ; la doc humaine (M) et la couche agent (N) y pointent sans inventer.

**5 décisions PO tranchées par défaut (override possible — dis-le, je fais un commit de suivi) :**
1. **Quadrant Diataxis TUTORIAL** → **DIFFÉRÉ S78** : pas d'orchestrateur in-vivo (T2
   RIG-ABSENT) → un walkthrough end-to-end sur-promettrait ; le hub renvoie au harness
   hermétique `b3_shard_pipeline.sh` + `compute-shard.spec.ts` comme preuve-de-vie.
2. **`#[derive(JsonSchema)]`** additif sur les types wire shard → **OUI** : schemars 1.2 déjà
   dep workspace, précédent `TaskResponse` ; c'est le mécanisme anti-drift de L (pas un
   type-miroir fragile).
3. **`llms.txt` racine** → indexe **uniquement le sous-système sharding** : un llms.txt
   repo-entier = livrable transverse hors scope sharding (sa propre phase/sprint).
4. **Méthode bridge shard** → marquée **PROPOSED / GAP-not-shipped**, nom + forme **figés en
   S78** : ne pas pré-engager le design de l'opt-in iframe.
5. **Seuils TOPLOC/SENTINEL/spot-check** dans REFERENCE → **valeurs actuelles documentées +
   marqueur « S78-pending tuning »** : utile à l'implémenteur, honnête sur la calibration.

### §20.1 Phase L — Spec wire machine-lisible + JSON Schemas générés (contrat LLM/agent, drift-gated)

**Audience** : LLM / machine. **Dépend de** : rien (types wire S77 figés).

**Scope.** Le contrat machine-lisible de la feature : une spec wire structurée + des JSON
Schemas **générés par schemars** depuis les types Rust, gardés par un test de drift (miroir
de `test_schema_snapshot_matches_struct` existant). Couche que l'agent/LLM ingère sans
deviner ; ancrage vérifiable que M et N référencent. Couvre « comment ça marche » au niveau
primitives signées + ALPN.

**Livrables.**
- `docs/protocol/SHARD_PROTOCOL_SPEC.md` (anglais, style `PUBLIC_FEED_SPEC.md` ; banner régime
  pre-v1.0 raw-op additif : `*_FORMAT_VERSION=1`, 5 `DOMAIN_*_V1`, 0-bump/0-dep ; section par
  type `ShardAssignment`/`ShardPlan`/`ShardedSessionManifest(+Entry)`/`RunProof(+Entry)`/
  `RunMetrics`/`ComputeGroup(+Entry)` ; ALPN `sbfb/shard/1` : bi-stream QUIC long-lived,
  framing length-prefixed BE, `MAX_SHARD_FRAME_BYTES=256MiB`, `MAX_SHARD_N_CTX=8192`,
  `is_member` crypto-before-`accept_bi`, caps DoS sign ET verify, verdict hors-bande ;
  renvoi THREAT_MODEL §16 + PATTERNS §P64-69).
- `crates/nexus-core-rs/src/schemas/shard.rs` (`schema_for!` pour `ShardPlan`,
  `ShardedSessionManifest`, `RunProof`, `RunMetrics`, `ShardAssignment` + DTO
  `ShardSessionView`/`ShardSessionStatusResponse` ; enums fermés `ShardRole`/`KvCachePolicy`).
- `crates/nexus-core-rs/src/schemas/*.schema.json` snapshots générés (draft 2020-12).
- `#[derive(JsonSchema)]` additif sur les types wire shard + DTO (décision PO #2).

**Tests plan.** +2/+3 Rust `nexus-core-rs` : `shard_schema_snapshot_matches_struct` (drift),
`schema_parses_as_valid_json_object` + required-fields par type, `spec_consts_exist`
(grep-assert : chaque `DOMAIN_*_V1`/cap citée dans la spec existe comme const, anti-drift doc↔code).

**Critère d'acceptation.** `cargo nextest run -p nexus-core-rs` : drift-test FAIL loud si un
`.schema.json` commité ≠ `schema_for!(T)` régénéré ; const-check doc↔code vert.

**Commit cible.** `feat(core): Sprint 77 Phase L — machine wire spec + generated shard schemas`
(porte du code : schemas + derives + tests → phase feat avec gate Codex + 9 sections).

### §20.2 Phase M — Docs humaines Diataxis : comment ça marche + comment câbler (FR, honnêteté PROVISIONAL)

**Audience** : humain. **Dépend de** : L (REFERENCE = jumeau humain des schémas générés).

**Scope.** Docs humaines françaises sous `docs/sharding/` : hub + explication + how-to +
référence. Couvre (a) « comment ça marche » et (b) « comment câbler un projet » côté humain,
en s'appuyant sur la spec machine L sans dupliquer THREAT_MODEL §16 ni PATTERNS.

**Livrables.**
- `docs/sharding/README.md` (FR, hub Diataxis + banner statut PROVISIONAL/T2 RIG-ABSENT +
  caveat cardinal **admission ≠ confidentialité** en gras + table des 4 quadrants).
- `docs/sharding/EXPLANATION.md` (FR : pipeline-parallel pas tensor-parallel [latence-bound
  WAN-friendly], bloc `[layer_start,layer_end)` demi-ouvert, initiator-signe-le-plan /
  worker-signe-le-RunProof, frontière = auto-attestation, échelle N0 TOPLOC → N1 VRF → N2
  quorum tolérant → N3 commit-reveal/SENTINEL en termes simples, invariant no-floats-signés,
  posture honest-but-curious ; **indexe** THREAT_MODEL §16, ne duplique pas).
- `docs/sharding/HOW_TO_WIRE.md` (FR par rôle : START via `/compute` « Lancer un gros modèle
  en réseau » [texte explicatif seul aujourd'hui], JOIN « Rejoindre un groupe de calcul »
  [lookup read-only id hors-bande], OBSERVE `GET /api/daemon/shard-session/{id}` → member_count ;
  contrainte **llama-arch-only + même-GGUF** ; bannière honnête : pas de store live,
  orchestrateur = carry **S78**, `sbfb-bridge.js` n'a **AUCUNE** méthode shard → entrée =
  panel shell, pas appel bridge).
- `docs/sharding/REFERENCE.md` (corps anglais, audience both, style `PUBLIC_FEED_SPEC.md` :
  jumeau humain lisible des schémas L, table par type name/type/units/signed?/cap + exemple
  JSON + DOMAIN tag ; relation single-source-of-truth avec `SHARD_PROTOCOL_SPEC.md` énoncée).

**Tests plan.** 0 net-new fonctionnel Rust/Vitest ; gate = script doc-lint net-new.

**Critère d'acceptation.** `scripts/check-sharding-docs.sh` en CI : (1) **link-check** — chaque
lien repo-relatif + ancre §-citée (THREAT_MODEL §16, PATTERNS §P64-69/§P39, routes, sources)
résout ; (2) **honesty-gate** — grep : README+EXPLANATION+HOW_TO_WIRE contiennent le marqueur
`PROVISIONAL` + la phrase caveat admission≠confidentialité, et HOW_TO_WIRE contient `S78` sur
le bloc orchestrateur ; (3) **scan-en-strings** réutilisé → corps humain FR.

**Commit cible.** `docs(sharding): Sprint 77 Phase M — docs humaines how-it-works + how-to-wire`.

### §20.3 Phase N — Couche agent-consommable : llms.txt + WIRING_SPEC + exemples runnable (source-anchored)

**Audience** : LLM / agent. **Dépend de** : L et M.

**Scope.** Couche agent au-dessus des schémas (L) et des docs humaines (M) : index `llms.txt`,
spec contract-dense ancrée `file:symbol`, exemples **runnable**. Couvre « comment câbler »
pour un agent qui doit câbler/réviser **sans halluciner**, avec garantie que les snippets
compilent et que l'index résout vers la vérité.

**Livrables.**
- `docs/sharding/llms.txt` (index Markdown : Truth Stack `repo files > .planning/active/ >
  commits > prompts > chat`, annotations 1-ligne + liens repo-relatifs vers
  `SHARD_PROTOCOL_SPEC.md`, `schemas/`, `shard_plan.rs:symbol`, route `http.rs`, `daemon.ts`,
  THREAT_MODEL §16, harness ; règle « fait absent du rang-1 = Not evidenced »).
- `llms.txt` racine du repo indexant `docs/sharding/llms.txt` (décision PO #3 : **sharding seul**).
- `docs/sharding/WIRING_SPEC.md` (anglais contract-dense, ordre fixe : (1) authority +
  Truth-Stack header ; (2) actor model initiator/worker/observer + séquence
  start→plan→sign→claim→run-proof→observe ; (3) contrat par étape : source `file:line`,
  signed?, DOMAIN tag, caps, préconditions [`is_pipeline_contiguous` couvre `[0..L)`,
  `is_member` avant `accept_bi`, `authorize_claim` crypto-before-IO] ; (4) contrat HTTP
  control-plane : méthode/path/auth tier loopback bearer+Host+Origin / réponse stub
  `{found:false,session:null}` ; (5) invariants INVIOLABLES : no-floats signés, jamais exposer
  `worker_pubkey`/`initiator`, additive-only pre-v1.0, 0-bump ; chaque claim porte un source_ref).
- `docs/sharding/examples/` : (a) `sign_verify.rs` **lifté VERBATIM** des `#[test]`
  `shard_plan_signature_roundtrip` + `run_proof_signature_roundtrip` [compilable] ;
  (b) `observe.curl.md` `GET /api/daemon/shard-session/{id}` + headers loopback + réponse
  `{found:false,session:null}` ; (c) `bridge_gap.md` : `sbfb-bridge.js` n'expose AUCUNE
  méthode shard ; signature future marquée **PROPOSED / GAP-not-shipped** (décision PO #4).

**Tests plan.** +1 Rust (`sign_verify.rs` compilé+exécuté comme example cargo OU ré-importé en
`#[test]` ; vert par construction car lifté verbatim) ; extension `check-sharding-docs.sh`.

**Critère d'acceptation.** (1) `examples/sign_verify.rs` compilé+exécuté par la suite (drift =
échec compile) ; (2) `check-sharding-docs.sh` **source-ref-check** — chaque `file:symbol` /
`file:line` cité dans WIRING_SPEC + llms.txt existe ET est grep-trouvé (fail si ancrage dans le
vide), + assertion Truth-Stack-header + règle « Not evidenced » présents.

**Commit cible.** `docs(sharding): Sprint 77 Phase N — couche agent llms.txt + WIRING_SPEC + exemples`.

### §20.4 Delta tests + checkpoint (avenant)

- **Delta tests** : L ~+2/+3 Rust (drift + parse + const-check) ; M +0 fonctionnel (gate =
  `check-sharding-docs.sh`) ; N +1 Rust (example) + extension doc-lint. **Avenant ≈ +4 Rust**
  + 1 script CI net-new (`check-sharding-docs.sh`).
- **Commits** : A-N = **15 commits** (12 A-K + 3 doc L/M/N). Le « 12 commits » du §19 devient
  indicatif pré-avenant.
- **Conditions checkpoint additionnelles** :
  - [ ] `docs/protocol/SHARD_PROTOCOL_SPEC.md` + schemas générés + **drift-test vert**
  - [ ] `docs/sharding/{README,EXPLANATION,HOW_TO_WIRE,REFERENCE}.md` + `check-sharding-docs.sh` **vert**
  - [ ] `docs/sharding/{llms.txt,WIRING_SPEC.md,examples/}` + **source-ref-check vert** + example compilé
  - [ ] honnêteté PROVISIONAL/S78 **grep-enforced** dans M + N (pas seulement promise)
- **Note testabilité** : aucune phase doc n'est exemptée de gate — drift-test schemars (L),
  link+honesty+lang (M), compile+source-ref (N). T1/T2 du sprint restent inchangés.
