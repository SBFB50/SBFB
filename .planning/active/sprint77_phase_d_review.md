# Sprint 77 — Review Phase D

Scheduler de placement Parallax phase 1 (calcul interne cote initiateur, 0 wire) :
nouveau module `crates/nexus-coordinator-rs/src/placement.rs` (884 lignes) +
`pub mod placement;` dans `lib.rs` (+1 ligne, place alphabetiquement entre
`pii_redactor` et `pow_counter`). Review en lecture seule AVANT commit.

## Verdict: PASS

0 BLOCKER. Review propre cote correctness/scope/wire/securite/tests.
Une seule CONCERN de dimension (D5-1, hygiene API `pub fn` non-bornee) etait
retenue au niveau INFO apres verification adversariale (pas de chemin
exploitable courant, threat model pre-launch). Le finding D2-6 (ecart doc/code
float) est REFUTE (sur-lecture du docstring).

Le verdict initial PASS-PENDING a ete promu **PASS** apres : (1) resolution des
2 concerns INFO en-phase (D5-1 `cluster_order_by_rtt`/`covers_full_model` ->
`pub(crate)` ; D4-6 test frontiere `placement_federation_at_exact_fit` ajoute) ;
(2) gate Codex GPT 5.5 CLEAN (cf. `## Codex reconciliation`).

## Table des dimensions

| Dimension | Verdict | Findings retenus |
|---|---|---|
| D1-algo-correctness | PASS | 5 INFO (invariants prouves sains) |
| D2-scope-wire | PASS | 1 CONCERN→INFO refute (D2-6) + 6 INFO |
| D3-sybil-determinism | PASS | 3 INFO |
| D4-tests-quality | PASS | 6 INFO |
| D5-security-api | CONCERN→INFO | 1 CONCERN upheld INFO (D5-1) + 3 INFO |

## Findings retenus

### D5-1 [INFO — hygiene API, non-bloquant] `cluster_order_by_rtt` est `pub` sans borne sur `n`
- placement.rs:459-485 : `pub fn cluster_order_by_rtt` alloue `vec![vec![0u64; n]; n]`
  (matrice N×N, placement.rs:473) et lance `pam_swap` en O(KMEDOIDS_MAX_ITER · k · n ·
  total_cost) ≈ O(64·k²·n²) (placement.rs:574-601). `covers_full_model:298` est
  egalement `pub` (read-only, benin). `lib.rs:27 pub mod placement;` => atteignable
  cross-crate.
- Pourquoi INFO et non CONCERN (verif adversariale) :
  1. Aucun input controle par un attaquant n'atteint cette fonction. C'est un calcul
     pur en memoire fait par l'initiateur de session sur un candidate-set qu'il
     construit LUI-MEME a partir des membres deja admis dans son `ComputeGroup` prive
     Ed25519-gate (module doc placement.rs:6-8, :85-86). Zero deserialisation wire,
     zero chemin reseau (S77 §17 : placement = compute interne 0-wire).
  2. Le SEUL appelant non-test actuel est `plan_placement:242`, dont le slice passe
     est borne : `select_candidates` s'arrete des couverture (placement.rs:356) puis
     `plan_placement` rejette `selected.len() > SHARD_PLAN_MAX_ASSIGNMENTS=256`
     (placement.rs:234, shard_plan.rs:87). Phase E n'existe pas encore (c'est ce diff).
  3. Le « DoS » decrit est auto-inflige : obtenir 10000 membres Ed25519-admis dans une
     session privee n'est pas un input adversarial realiste.
- Fix suggere (durcissement avant reuse Phase E, OPTIONNEL) : `pub(crate)` sur
  `cluster_order_by_rtt`, ou doc-note + `debug_assert!(n <= SHARD_PLAN_MAX_ASSIGNMENTS)`
  forçant le pre-cap par l'appelant. Aucun invariant casse aujourd'hui ; ne bloque pas
  le commit.

## Findings refutes (INVALID)

### D2-6 [REFUTE] « ecart doc/code : le docstring decrit un design f64 jamais implemente »
- Le coeur de l'invariant (no-float-leak) est CORRECTEMENT confirme : aucun `f64`/`f32`
  n'atteint le `ShardPlan` ni un `Eq` wire. placement.rs:407 =
  `(total * w.vram_free_bytes as u128 / sum_w) as u32` (division entiere u128 pure),
  placement.rs:417-418 = `total * ... as u128 % sum_w` (largest-remainder par modulo
  entier), sorties `water_fill` = `Vec<u32>`, `ShardAssignment.layer_start/end` = u32.
  Grep `f64|f32` = 4 hits TOUS dans le docstring (placement.rs:30,30,30 et la doc), 0
  dans le code reel. Implementation 100% entiere (u128/u64/u32). Invariant respecte.
- MAIS la sous-claim « ecart doc/code » est une SUR-LECTURE non reproductible. Le titre
  « Why floats are allowed *here* » (placement.rs:27) est un cadrage de justification
  (pourquoi les floats SERAIENT permis dans un module in-memory a sortie entiere), pas
  une description de ce que fait le code. Le corps dit explicitement l'inverse
  (placement.rs:34-37) : « We therefore keep the arithmetic integer-exact anyway
  (largest-remainder apportionment, integer-microsecond RTT) … no float leaks into the
  plan. » La doc DECLARE integer-exact, le code EST integer-exact : aucune contradiction.
- Au mieux le heading est une legere maladresse stylistique pour un module sans float —
  ni defaut de code, ni risque wire/runtime. Pas de severite CONCERN justifiee ; tout au
  plus un nit doc-style INFO non-bloquant. La contrainte DURE (no float leak vers
  ShardPlan/Eq wire) est pleinement tenue.

## Invariants verifies sains

- **Couverture exacte [0..total_layers)** : `covers_full_model` (placement.rs:298-306)
  exige `is_pipeline_contiguous()` ET `first.layer_start==0 && last.layer_end==total`.
  Appele AVANT de retourner `Sharded` (placement.rs:279). Le check stateful delegue par
  shard_plan.rs:209 est bien implemente cote scheduler. `select_candidates` garantit
  `cap_sum >= total_layers` (placement.rs:361) ; `water_fill` garantit `assigned ==
  total_layers` a la sortie de la boucle (placement.rs:427-446) et la somme par
  largest-remainder est exacte. Test `placement_handles_5_workers_70b` asserte
  `covers_full_model([0..80))` + somme==80.
- **Pas de gap/overlap/starvation** : la borne « federation atteinte SSI le modele
  tient dans le plus gros worker » (placement.rs:213) implique que tout worker
  selectionne a `layer_capacity < total_layers` (vram_free < quantized). Donc le worker
  dominant ne peut jamais absorber tous les layers : water_fill clamp son floor a
  `caps[i] < total_layers` (placement.rs:408), forçant >= 1 layer sur le 2e worker. Le
  garde collapse-to-1 (placement.rs:284-288) est donc une defensive prouvee inatteignable,
  pas un bug latent. Verifie a la main sur le pire cas (W1 dominant + W2 capacite 1).
- **Sommes / overflow** : `cap_sum += layer_capacity(...)` (placement.rs:355) borne post-
  federation (chaque cap < total_layers <= u32::MAX, slice <= 256) ; `assigned: u32 =
  alloc.iter().sum()` (placement.rs:411) <= total_layers (floors proportionnels somment
  <= total) ; arithmetique proportionnelle en u128 (placement.rs:394,407,417) ne deborde
  pas (vram <= sum_w => quotient <= total_layers). `as_micros().min(u64::MAX as u128)`
  (placement.rs:129) et `.min(u32::MAX as u64) as u32` (placement.rs:385) clampent les
  narrowings. Aucun overflow u64/u128/u32 sur 70B / 256 workers.
- **Determinisme** : 0 `thread_rng`/`rand`/`SeedableRng` (grep clean — 0 hit code). PAM
  BUILD greedy pur avec tie-break pubkey (placement.rs:539-568), PAM SWAP n'accepte que
  des swaps strictement decroissants (`c < current`, placement.rs:586) => suite d'entiers
  positifs strictement decroissante, ne peut osciller, bornee par KMEDOIDS_MAX_ITER=64.
  `BTreeMap` pour `RttMatrix.entries` (placement.rs:111) ET `clusters` (placement.rs:491)
  — iteration ordonnee, 0 `HashMap`. Pas d'exposition RUSTSEC-2026-0097. Test
  `missing_rtt_is_treated_as_far_not_panic` asserte la reproductibilite cross-call
  (placement.rs:839).
- **0-bump wire** : aucun `DOMAIN_*`, aucun `*_FORMAT_VERSION`, aucun type
  `Serialize/Deserialize`, aucune edition de `canonical.rs`/`shard_plan.rs`. Les 4 types
  internes (`WorkerPlacementProfile`, `RttMatrix`, `ModelSpec`, `PlacementOutcome`)
  derivent seulement Debug/Clone/PartialEq/Eq (+Copy/Default), jamais Serialize. Seul
  type Serialize manipule = `ShardPlan`/`ShardAssignment` venant de shard_plan.rs
  INCHANGE, consomme en construction read-only. Grep `DOMAIN_/FORMAT_VERSION/Serialize/
  canonical_bytes/sign/verify_signature` = hits uniquement dans le docstring.
- **Scope cut #7** : `consent.rs`/`estimated_vram_mb`/`GpuStats` n'apparaissent que dans
  les doc-comments (placement.rs:22,25,89) qui DOCUMENTENT leur non-modification. Aucun
  import de consent ni d'API runtime VRAM-live. La VRAM mesuree (`vram_free_bytes`) est
  un champ d'entree fourni par le caller, lu au PLACEMENT seulement.
- **Anti-recentralisation D3** : k-medoids opere EXCLUSIVEMENT sur `RttMatrix` (micro-
  secondes entieres mesurees, conn_rtt Phase B). Aucune table geo/ASN/region (termes
  uniquement dans le docstring placement.rs:42 affirmant leur absence).
  `pam_build`/`pam_swap`/`cluster_order_by_rtt` ne lisent que `d[i][j]` (RTT) et pubkeys
  (tie-break).
- **Carry SYBIL-SEEDER-TAIL absorbe** : selection des candidats triee par
  `(capacity desc, sampling_key asc)` ou `sampling_key = blake3(session_id||pubkey)`
  (placement.rs:312-317,344-349). Test `sybil_seeder_tail_sampling_is_deterministic_
  non_lexicographic` (placement.rs:751-787) prouve independamment via `assert_ne!(tails,
  lex)` que l'ordre n'est PAS lexicographique + reproductible. Un attaquant ne peut pas
  biaiser sa position en mintant un pubkey a faible prefixe.
- **Named constants (README §6.9)** : MIN_SHARD_WORKERS (placement.rs:59),
  KMEDOIDS_DEFAULT_K (:65), KMEDOIDS_MAX_ITER (:70), MISSING_RTT_PENALTY_MICROS (:77),
  reuse de SHARD_PLAN_MAX_ASSIGNMENTS importe de shard_plan.rs. 0 magic number nu pour
  seuils/k/bornes.
- **Pas de panic sur input degenere** : early-returns sur candidates vides /
  total_layers==0 / quantized==0 (placement.rs:189-203) AVANT toute division ;
  `per_layer_bytes = div_ceil(...).max(1)` (placement.rs:219-222) avec total_layers deja
  prouve non-nul => pas de div-by-zero, per_layer >= 1. Les `expect` non-test sont tous
  prouves surs (placement.rs:212 garde par :189 ; :549/:500 garde par n>=1 ; le retour
  anticipe :465 protege le cas n<=1).
- **Delta tests** : 11 `#[test]` dans placement.rs (les 10 d'origine + le test frontiere
  `placement_federation_at_exact_fit` ajoute en reponse a D4-6). Le body annoncera +11
  Rust : exact, pas
  de faux-vert de comptage (audit S76). Les 5 tests mandates par le plan §7.3 sont
  presents (`placement_water_fills_vram_free`,
  `placement_refuses_when_model_fits_single_worker`,
  `kmedoids_groups_low_rtt_consecutive_layers`, `placement_handles_5_workers_70b`,
  `sybil_seeder_tail_sampling_*`), avec assertions par valeur absolue (l2==40, len==5,
  somme==80) — pas de tautologies.

## Renforcements mineurs non-bloquants (INFO)

- D4-6 [RESOLU en-phase] : le test boundary `placement_federation_at_exact_fit`
  (`quantized_vram_bytes == max_free => EndpointFederation`) a ete ajoute, couvrant la
  frontiere `<=` exacte sharding/federation (placement.rs:213).
- D4-5 : 4 branches d'erreur internes defensives sans test negatif dedie (collapse
  MIN_SHARD_WORKERS post-water_fill :284 ; selected>MAX :234 ; garde `!progress`
  :439-444 ; erreur interne covers_full_model :279). Toutes inatteignables sans bug
  prealable ; couvertes indirectement par les asserts positifs `covers_full_model` des
  tests 1 et 4. Acceptables comme sentinelles de regression logique.

## Codex reconciliation

Gate Codex GPT 5.5 (`codex exec`, modele cross-verification, output brut dans
`sprint77_phase_d_codex_review.md` — non reecrit). 2 rounds :

- **Round 1** : 8 livrables → 6 CONFIRME, 0 GAP, **2 PARTIEL**.
  1. PARTIEL (robustesse) : `pam_build` sommait une ligne RTT en `u64` via `.sum()` sans
     saturation → overflow/debug-panic theorique avec des `set_micros` extremes.
     **CORRIGE root-cause** : `.fold(0u64, u64::saturating_add)` (placement.rs:545-548),
     coherent avec `total_cost` qui saturait deja. Comportement inchange pour tout input
     realiste (RTT via `conn_rtt`).
  2. PARTIEL (faux positif) : Codex affirmait que `consent.rs` n'existe pas. Il EXISTE
     (`crates/nexus-worker-core/src/consent.rs:423`) — recherche Codex hors scope. Le
     doc-comment a ete precise en chemin complet `nexus-worker-core/src/consent.rs`
     (placement.rs:22) pour lever l'ambiguite.
- **Round 2** (apres fixes + re-run suites) : 8 livrables → **8 CONFIRME, 0 GAP, 0 PARTIEL.
  CLEAN.**

Suites relancees apres correction Codex : `nexus-coordinator-rs` nextest 11/11 placement
verts, `cargo fmt --all --check` 0 diff, `cargo clippy -p nexus-coordinator-rs
--all-targets -D warnings` 0 warning.

## Note sur les suites (state final)

- nextest `nexus-coordinator-rs` : 307 verts dont 11 tests placement (Windows natif).
- Pre-fix : Windows full workspace nextest **1862** verts (1852 + 10 placement) + Docker
  canonique rust:1.94 (coordinator+core, Linux) verts + web 402 verts.
- Post-fix (saturating + boundary test +1) : re-run full workspace Windows + Docker
  canonique coordinator+core en cours, a reconcilier dans `verification.md` (Phase K).
  Module pur integer-only, plateforme-agnostique (0 reseau/iroh/fs/time).

Aucun BLOCKER. Verdict final : **PASS** (review clean + Codex CLEAN round 2).
