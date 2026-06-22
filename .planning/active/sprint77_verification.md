# Sprint 77 — Verification (Phase K wrap-up + gate produit)

**Sprint** : 77 — sharding pipeline LLM (modèle ~20 Go éclaté cross-machine
RTX 5080 CUDA + Mac M2 Metal, pipeline-parallel `sbfb/shard/1`, scheduler
Parallax, vérif graduée N0-N3, fork llama.cpp layer-block, front `/compute`).
**Tip code phases A-J** : `66259c6`. **Phase K** : wrap-up, 0 code fonctionnel
net (harness shell + docs + verification + audit_plan).
**Gate testabilité (README §4)** : T1 E2E hermétique BLOQUANT + T2 artefact JSON
machine-lisible (jamais `DIFFERE-materiel` prose).

---

## §1 Fail-fast checklist (Observed rempli)

Baseline = HEAD `66259c6` (Phase K n'ajoute aucun code Rust/web fonctionnel).

| # | Check | Critère | Observed |
|---|---|---|---|
| 1 | fmt (Win 1.95 + Docker 1.94) | 0 diff | **0** sous les 2 toolchains |
| 2 | clippy `--all-targets -D warnings` | 0 | **0** (Win + Docker) |
| 3 | nextest Win `--workspace --locked` | 0-skip | **1949 run / 1949 passed / 0 skipped** |
| 4 | doctests `--doc` | 0 | **0** (Win + Docker) |
| 5 | release `nexus-shell-daemon` | OK | **OK** |
| 6 | nextest Docker canonique (rust:1.94 `sbfb-ci`, `target-linux`) | 0-skip | **1947 / 1953 passed** ; les **6 fails = tests iroh-networked cross-daemon** (`test_two_daemons_boot_and_respond`, `test_cross_daemon_discovery`, `test_cross_daemon_blob_transfer`, `test_cross_daemon_task_stub`, `cross_daemon_publish_and_serve_blob`, `blob_serve_coep_headers_on_real_zip`) **env-bloqués en Docker-on-Windows** (réseau hôte dégradé `create_node`, doc mémoire S74) — **verts sur Win natif (1949/1949) + le CI Linux**. Env canonique : `SBFB_AUTH_TOKEN` set (cf. carry TEST-ISOLATION §Carries) |
| 7 | web tsc | 0 | **0** |
| 8 | web lint | 0 err | **0** |
| 9 | web Vitest | ~405 | **411 passed (38 files)** |
| 10 | web coverage | ≥ seuils | **87.27 / 79.01 / 86.02 / 88.59** (stmt/branch/func/line) |
| 11 | web build + size | 6/6 | **OK / 6/6** |
| 12 | scan-en-strings | clean | **clean** |
| 13 | scan-trust-wording | clean / N-A | N-A (script absent) |
| 14-37 | tests par-phase A-I (noms §6) | PASS | **tous RÉSOLUS + PASS** (cf. §6 invariant clôture) |
| 38 | 0 bump wire | wire S77 inchangé ; nouveaux `DOMAIN_*` additifs | **PROUVÉ** : 5 `DOMAIN_*_V1` additifs S77 (COMPUTE_GROUP[B]/SHARD_PLAN+RUN_PROOF[C]/VRF_DRAW[H]/ACTIVATION_COMMIT[I]), 0 `_V2+`, tous les `*_FORMAT_VERSION`/`*_ANNOUNCEMENT_VERSION` du wire S77 =1 (`INVITE_FORMAT_VERSION=2` pré-existant S74, hors-S77, non touché), `FeedEntry` byte-stable |
| 39 | **T1 E2E shard hermétique** | GREEN BLOQUANT | **GREEN** — `compute-shard.spec.ts` : 41 passed + 1 skipped (`@shard`) |
| 40 | **T2 acceptance ~20Go JSON** | `status` ∈ {PASS,BLOCK{diag},RIG-ABSENT} | **RIG-ABSENT** (artefact ci-dessous, cf. ## Acceptance) |
| 41 | THREAT §16 sharding écrit | présent SI-1..5 + incentive | **présent** + v14 + §5.9 STRIDE + §2 A8 + §4 DFD + §6 LINDDUN + route |
| 42 | verification.md écrit | présent + ## Acceptance | **ce fichier** |
| 43 | sprint78_audit_plan.md écrit | présent + Track Testabilité | **présent** (4 carries 3/3 + SHARD-PROVISIONAL P1 + Track standing) |
| 44 | invariant clôture noms tests | tous grep-résolus | **tous résolus** (24 rows §6 / 27 fns, incl. les 4 hermétiques de la row F) |

---

## §6 Invariant clôture — noms de tests cités → fn `#[test]` réelle

Tout nom cité résout à une vraie fn `#[test]` (anti-faux-vert) :

| Test (rôle) | fn réelle | fichier |
|---|---|---|
| A convergence incrémentale | `convergence_incremental_task_reaches_remote_replica` | `nexus-shell-daemon/src/dispatch_loop.rs` |
| A boot catch-up | `convergence_boot_catchup_still_works` | `dispatch_loop.rs` |
| B ALPN shard | `shard_alpn_registered_in_router` | `nexus-core-rs/src/shard.rs` |
| B frame roundtrip | `shard_frame_roundtrip_two_nodes` | `shard.rs` |
| B compute_group sig | `compute_group_signature_roundtrip` | `compute_group.rs` |
| B handshake rejette | `shard_handshake_rejects_non_member` | `shard.rs` |
| C shard_plan sig | `shard_plan_signature_roundtrip` | `shard_plan.rs` |
| C run_proof sig | `run_proof_signature_roundtrip` | `shard_plan.rs` |
| D water-fill VRAM | `placement_water_fills_vram_free` | `nexus-coordinator-rs/src/placement.rs` |
| D refuse single | `placement_refuses_when_model_fits_single_worker` | `placement.rs` |
| D k-medoids RTT | `kmedoids_groups_low_rtt_consecutive_layers` | `placement.rs` |
| D 5-workers-70b | `placement_handles_5_workers_70b` | `placement.rs` |
| D sybil tail | `sybil_seeder_tail_sampling_is_deterministic_non_lexicographic` | `placement.rs` |
| E routing DAG | `routing_dag_sweep_selects_min_latency_chain` | `routing.rs` |
| E churn replace | `churn_replaces_failed_server_oturn` | `routing.rs` |
| E perf-map | `perf_map_republished_to_doc` | `routing.rs` |
| F backend subset (`#[ignore]`+feature) | `shard_backend_loads_layer_subset` | `nexus-worker-core/src/llm/shard.rs` |
| **F primitive shard hermétique CI** (ex-row 30 corrigée) | `shard_window_validates_contiguous_range`, `top_k_extracts_largest_by_magnitude_deterministically`, `hidden_token_count_validates_shape`, `toploc_commitment_is_deterministic_and_swap_sensitive` | `nexus-worker-core/src/llm/shard.rs:564-658` (`mod tests`, NON-`#[ignore]`, NON-feature) |
| G TOPLOC swap | `toploc_detects_model_swap` | `nexus-core-rs/src/toploc.rs` |
| H N1 VRF déterministe | `n1_vrf_selects_deterministic_verifier` | `verifiable_draw.rs` |
| H N1 randomise | `n1_spot_check_randomizes_temp_and_seed` | `verifiable_draw.rs` |
| H incentive réput. | `incentive_credits_reputation_on_honest_spotcheck` | `nexus-coordinator-rs/src/rerun.rs` |
| I N2 tolérant | `n2_tolerant_quorum_accepts_close_fingerprints` | `redundancy.rs` |
| I N3 SENTINEL | `n3_sentinel_localizes_corrupted_stage` | `sentinel.rs` |

> **Correction PLAN-ADAPT (preflight Phase K)** : le plan §16 row 30 citait
> `shard_backend_primitive` — **= 0 fn (placeholder jamais matérialisé)** → faux-vert
> silencieux. Remplacé par les 4 noms hermétiques CI réels ci-dessus. Les
> `shard_backend_*` réels sont `#[ignore]`+feature `llm_llama_cpp` (`shard.rs:682`)
> = jamais-CI, prouvés runnable localement sur Mistral-7B-Q4 (spike).

---

## Acceptance

### T1 — E2E hermétique (BLOQUANT) : **GREEN**
`(cd web && npm run test:e2e)` → `web/e2e/compute-shard.spec.ts` :
**41 passed + 1 skipped** (`@shard`, hors-CI par design). BLOQUANT au wrap-up
+ chaque push CI.

### T2 — Benchmark ~20 Go cross-machine (artefact JSON) : **RIG-ABSENT**
`bash scripts/acceptance/b3_shard_pipeline.sh` →

```json
{"status":"RIG-ABSENT","stage":"preflight","model":"","n_shards":2,
 "ttft_s":null,"toks_per_s":null,"rtt_frontier_ms":null,"run_proof":"",
 "diagnosis":"no shard session to drive: SHARD_SESSION_ID is unset and no
 production orchestrator creates one. The sbfb/shard/1 data-plane serves a
 long-lived bi-stream, forwarding each boundary frame through one layer block
 with admission control, but no production caller drives a token-by-token
 cross-shard generation, measures TTFT/tok-s, or emits an in-vivo RunProof.
 The HTTP route GET /api/daemon/shard-session/{id} is a Phase J read-only STUB
 (live_shard_session -> None). The sharding CORE is delivered and hermetically
 tested; only the live SESSION ORCHESTRATOR remains — an S78 carry.",
 "last_response":""}
```

> Diagnostic abrégé ci-dessus pour la lisibilité ; l'artefact réel
> `scripts/acceptance/.b3_shard_last_result.json` (gitignoré) porte la chaîne
> complète — liste du cœur livré + phrase finale « Set SHARD_SESSION_ID once it
> lands and a session can be mounted. »

**Pourquoi RIG-ABSENT (honnête, structurel)** — deux faits cumulés :
1. **Aucun orchestrateur de session in-vivo** : le data-plane `sbfb/shard/1`
   sert un bi-stream long-lived, forwardant chaque frame layer-block avec
   admission, mais aucun chemin prod ne monte une session, pilote une génération
   token-par-token cross-shard, mesure
   TTFT/tok-s, ou émet un `RunProof` signé (aucun caller prod ; les seuls appels
   `RunProof::new`/`RunProofEntry::sign` vivent sous `#[cfg(test)]`,
   route `/shard-session` = stub `None`). `pass()` exige `run_proof` non-vide ET
   `toks_per_s ≥ 1` → **structurellement inatteignable aujourd'hui**, avec OU sans
   le rig.
2. **Rig 2-machines absent** : pas de Mac M2 + modèle ~20 Go en environnement dev.

**C'est PRÉVU par le plan §14.4** : `status` = champ JSON machine-lisible (jamais
`DIFFERE-materiel` prose) → la **feature shard reste PROVISIONAL + carry P1 S78**.

### Statut feature
**Cœur sharding LIVRÉ + testé hermétiquement** : primitives wire signées (C),
placement Parallax (D), routing + churn (E), fork llama.cpp layer-block (F1),
claim + data-plane (F2), vérification N0-N3 (G/H/I), front + route read-only (J).
**Feature shard = PROVISIONAL** : le benchmark live cross-machine (T2) sort de
RIG-ABSENT seulement quand l'**orchestrateur de session in-vivo** atterrit (S78)
et que le rig 5080+Mac M2 est présent → alors `b3_shard_pipeline.sh` produit
PASS (`run_proof` non-vide + `toks_per_s ≥ 1`) ou BLOCK diagnostiqué.

---

## Carries → S78 (détail dans `sprint78_audit_plan.md`)

- **SHARD-PROVISIONAL (P1)** : orchestrateur de session in-vivo + benchmark live
  cross-machine (le gros — sort la feature de PROVISIONAL).
- **4 escalades 3/3** : seeder `catalog_len:0`, REVISION-HOME-DURABILITY,
  KNOWN-ENTRY-OVERCOUNT, RE-DRIVE-ON-INGEST (T2 RIG-ABSENT n'a pas prouvé la
  convergence WAN).
- **TEST-ISOLATION-SBFB-HOME (P2, découvert Phase K)** : les e2e daemon-spawn
  (`nexus-shell-daemon/tests/e2e.rs`) ne fixent pas `SBFB_HOME` par TempDir →
  course sur `$HOME/.sbfb/auth_token` en conteneur frais parallèle. Pré-existant
  (0 delta Rust S77), masqué sur Win/CI (`~/.sbfb` présent). Root-cause =
  `.env("SBFB_HOME", tmp)` par test ; contournement Docker canonique =
  `SBFB_AUTH_TOKEN` env.
- **Env-block iroh-networked Docker-on-Windows** : 6 tests cross-daemon (réseau
  hôte dégradé) — verts Win + CI Linux, non-régression.
- SI-9 withholding, SI-11/recalibration bf16/TOPLOC, SI-5 padding, T-NN+3 JCS,
  MEDIAN-DE-GROUPE/SANITY-BOUND, B10-PARITE-FIXTURE, OWN-DOC-FLOOR,
  DIRECTORY-EAGER-HAPPY-PATH, P3-D-3 (à confirmer).

---

## Verdict

**Sprint 77 cœur sharding LIVRÉ** (12 phases A-K, F en F1+F2) ; tout testé
hermétiquement (Win 1949/1949 0-skip, Docker fmt/clippy/doctest 0 + 1947/1953
[6 iroh-networked env-bloqués Docker-on-Windows], Vitest 411, E2E 41+1skip).
**Feature shard = PROVISIONAL** (T2 RIG-ABSENT honnête → carry P1 S78).
**0 bump wire, 0 dep ajoutée.** Invariant **héberger != publier, seeder !=
auteur** tenu. Scope cuts respectés. Audit gate S77 = `sprint78_audit_plan.md`.
