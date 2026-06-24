# Sprint 77 Phase F2 — Preflight G8 (shard claim + sbfb/shard/1 wiring + threat §16)

## Verdict: PLAN-ADAPT

> Preflight Workflow ultracode (10 agents : 5 scans factuels S1a/S1b/S2/S3/S4 + 1 vérification
> adversariale par scan + synthèse). Signaux : **S1a PLAN-ADAPT, S1b EXECUTE, S2 EXECUTE,
> S3 PLAN-ADAPT, S4 EXECUTE → verdict global PLAN-ADAPT**. **0 DESIGN-CONFLICT** : le conflit
> Phase F (forward-partiel infaisable via wrapper safe) est résolu par le fork F1 ; F2 CONSOMME
> le backend désormais faisable et n'introduit aucune nouvelle infaisabilité.
> Artefact Workflow brut : `tasks/w9se4pnwb.output` (run `wf_04d01996-a9f`).
>
> **Note honnêteté process** : la synthèse globale ci-dessous est reconstruite à la main depuis
> les 5 scans + la vérification adversariale (evidence fichier:ligne vérifiée), pas depuis un
> agent de synthèse (suit le précédent F1). L'adaptation matérielle porte sur le **pilier 2
> (sizing VRAM)** : la vérification adversariale a **réfuté** ma méthode initiale (parseur GGUF
> pur-Rust + table GGML maison) au profit de la **voie FFI** déjà vendorée.

---

## Design initial (main-thread) vs design adapté

| Pilier | Design initial | Verdict | Design adapté (à coder) |
|---|---|---|---|
| 1. Seam crate-boundary | trait `ShardForwarder` dans core-rs, `ShardProtocol` détient `Arc<dyn>`, accept forward | **EXECUTE** (idiomatique Petals/llama.cpp-RPC/exo) | inchangé + migrer les 7 tests Phase B + `EchoForwarder` pub |
| 2. Cap VRAM fail-closed | parseur GGUF **pur-Rust** + table GGML type-size maison | **PLAN-ADAPT** (réfuté) | **étendre `GgufContext` FFI** (`gguf_get_tensor_size` natif exact) ; arithmétique d'estimation pure CI-testable |
| 3. THREAT §16 | SI-1 + SI-4 minimal, reste en K | **EXECUTE** + adapt | **SI-1..SI-5 + incentive complet** (résout contradiction check #41) |
| 4. Cap 64 MiB | à trancher | **P1** | relever à **256 MiB** + doc `fp16`→`fp32` + named const `MAX_SHARD_N_CTX` |

---

## Pilier 1 — Seam `ShardForwarder` (EXECUTE)

Frontière de crate **confirmée factuellement** : `nexus-core-rs/Cargo.toml` ne dépend PAS de
`nexus-worker-core` ; `worker-core/Cargo.toml:47` dépend de core-rs (sens unique). Donc
`ShardBackend` (worker-core, feature `llm_llama_cpp`) **ne peut pas** être appelé depuis
`ShardProtocol::accept` (core-rs) — l'inversion de dépendance par trait est la seule voie sans
cycle, et c'est le pattern dyn-dispatch injecté de Petals/llama.cpp-RPC/exo.

À coder :
- `pub trait ShardForwarder: Send + Sync + std::fmt::Debug` dans `core-rs/src/shard.rs` :
  `fn forward(&self, upstream_frame: &[u8]) -> Result<Vec<u8>>`.
- `ShardProtocol` gagne `forwarder: Arc<dyn ShardForwarder>` (en plus de `admission`). Le corps
  echo (`shard.rs:233-243`) devient `recv → forwarder.forward(frame) → send` ; sur erreur du
  forwarder, **fermer proprement** (pas de panic).
- `pub struct EchoForwarder` (core-rs) renvoyant le frame tel quel — **préserve les tests Phase B**.
- **MIGRATION OBLIGATOIRE** (adversarial S1a/S1b missed) : les 3 constructeurs changent de
  signature — `ShardProtocol::new(group, forwarder)`, `from_verified(group, forwarder)`,
  `shard_protocol_factory(group, forwarder)` — et la fixture `two_node_shard_fixture` + les 5
  tokio-tests réseau (`shard_alpn_registered_in_router`, `shard_frame_roundtrip_two_nodes`,
  `shard_handshake_admits_member`, `shard_handshake_rejects_non_member`, `shard_conn_stats_exposes_rtt`)
  doivent injecter `EchoForwarder`. Run vert du module `shard.rs` post-seam = exigence review.
- **Rôle accept = aval uniquement** : le serveur `accept()` reçoit un hidden → `forward_hidden`.
  Le 1er shard (tokens → `forward_tokens`) est piloté côté DIALER/orchestrateur, **hors accept()**.
  Le drive multi-hop live complet est **Phase K** (S2-B confirmé : `ShardProtocol`/`ShardBackend`
  ont 0 appelant prod aujourd'hui).
- `ShardBackendForwarder` (worker-core, **feature-gated** `llm_llama_cpp`) implémente
  `ShardForwarder` en enveloppant `ShardBackend::forward_hidden` (+ validation forme via
  `hidden_token_count` déjà présente). Compilé dans les builds CUDA/Metal, exercé sur le rig.

## Pilier 2 — Cap VRAM fail-closed (PLAN-ADAPT, méthode réfutée)

**Réfutation adversariale (S3-F-2 refuted + S1a/S2/S4 missed convergents)** : écrire un parseur
GGUF pur-Rust + une table GGML type-size maison = band-aid avec 3 risques de correctness :
(a) skip incorrect des metadata-KV (variable-length, String/Array récursifs) → mauvais offset →
somme fausse ; (b) table type-size fausse (Q4_K=256/144, Q6_K=256/210…) → sous-estimation
silencieuse ; (c) une **sous-estimation accepte un claim qui OOM** au load = **fail-OPEN**
(l'inverse de l'intention). Le repo contient DÉJÀ un lecteur GGUF header-only natif
(`vendor/llama-cpp-2/src/gguf/mod.rs`, `GgufContext`, `gguf_init_from_file no_alloc=true`).

**Méthode adaptée (no-band-aid, root-cause)** :
- **Étendre `GgufContext`** (vendor) avec 3 accessors FFI déjà liés (vérifiés `gguf.h:124-127`) :
  `tensor_name(i64) -> Option<&str>` (`gguf_get_tensor_name`), `tensor_size(i64) -> u64`
  (`gguf_get_tensor_size`, **taille exacte calculée par llama.cpp**, gère block-quant + nouveaux
  types), `tensor_type(i64) -> ggml_type` (`gguf_get_tensor_type`). Patch regénéré (`patches/`).
- **Frontière feature / CI** (mirror F1 double-tier) :
  - `gguf_meta.rs` PUR (hors feature, CI-testable) : `estimate_shard_resident_bytes(tensor_sizes,
    window, kv_cfg) -> u64` (filtre `blk.{i}` pour `i ∈ [start,end)` + résidents `tok_embd`
    toujours + `output_norm`/`output` si `is_last` via `ShardWindow::is_last()` + KV-cache + headroom)
    et `evaluate_shard_claim(...)` PUR.
  - feature-gated : `read_gguf_tensor_table(path)` via `GgufContext` étendu → `(name, size)` list +
    `block_count`/`architecture` métadonnées. Testé `#[ignore]` GGUF (Mistral-7B-Q4).
- **Fail-closed verrouillé** (adversarial : `gguf_get_tensor_size` est une **borne inférieure** —
  pas de padding/buffers compute/contexte CUDA ~300-600 MiB/fragmentation) :
  `required = Σ resident_tensor_bytes + kv_cache_bytes + VRAM_BACKEND_OVERHEAD_BYTES` (named const
  généreux, over-estimate). Header illisible (magic/version/EOF) → **REJET du claim**, jamais
  « estimer 0 » (0 ≤ free = fail-open total).
- `evaluate_shard_claim` **crypto-AVANT-IO** (adversarial lock, miroir admission `accept:227`) :
  1) `manifest_entry.verify_signature()` (`shard_plan.rs:355`, DOMAIN_SHARD_PLAN_V1) →
  2) `is_member(self)` (`compute_group.rs:149`) →
  3) assignment-de-soi présent dans le plan →
  4) fenêtre `[layer_start,layer_end) ⊆ [0,n_layer)` (**pré-valide AVANT load** = ferme le P2
     validation-ordering F1 `llm/shard.rs:237-242` GGML_ASSERT abort) →
  5) `required_vram ≤ vram_free_bytes` (snapshot ponctuel `gpu_snapshot` runtime.rs:641).
  Un manifest non-signé/non-membre ne doit JAMAIS atteindre le read GGUF ou le GPU snapshot
  (DoS pré-auth). `gpu_snapshot` = **snapshot ponctuel one-shot**, PAS une pompe live → scope
  cut #7 tenu (S2-A confirmé, kickoff §7:756 carve-out PLACEMENT).

## Pilier 3 — THREAT_MODEL §16 (EXECUTE + adapt)

- Insérer `## 16. Surface sharding inference (Sprint 77)` entre §15.3 (`THREAT_MODEL.md:976`) et
  l'actuel `## 16. Revue et evolution` (`:980`), renommé **§17**. Ligne `v10 (Sprint 77 Phase F2)`
  au changelog.
- **Résolution contradiction scoping (adversarial S2 missed)** : le check fail-fast #41 du plan
  grep `§16 SI-1..SI-5 + incentive` (vise le wrap-up Phase K), alors que le handoff parle d'amorce
  SI-1/SI-4. Résolution : **écrire SI-1..SI-5 + incentive COMPLET dès F2** (source
  `SPLIT_INFERENCE_DESIGN.md §3.1` porte les 5 lignes ; incentive = R8 réputationnel non-monétaire).
  Satisfait check #41 ET le handoff « au minimum », et honore `feedback_ultra_complete_sprints`.
  Format `## 16.` (numéro+point) cohérent avec §15/§17 ; le check #41 grep « §16 » reste vert (le
  titre contient « 16 »).
- SI-1 (reconstruction activations, **High**) + SI-4 (collusion inter-workers, **High**) présentés
  comme **résiduels ASSUMÉS** (PAS mitigés par l'allowlist — `shard.rs:42-45` le dit déjà) ;
  SI-2 N/A (inference-only), SI-3 Medium, SI-5 Low. Caveat « activations en clair / pas de TEE GPU
  consumer 2026 (scope cut #4) / aucun secret app dans les prompts ».
- **Doc-honnêteté (adversarial missed)** : `SPLIT_INFERENCE_DESIGN.md §4.2:276` (« ne pas forker
  llama.cpp, préférer un wrapper ») est **superseded** par le fork S77 (F1). §16 doit acter que ce
  doc (S30) PRÉCÈDE la décision de fork, pour ne pas prendre §4.2 pour une règle vivante.

## Pilier 4 — Cap frame (S3-F-4, P1)

`MAX_SHARD_FRAME_BYTES=64 MiB` (`shard.rs:78`) trop serré : frontière = `n_embd × n_tokens × 4`
(**fp32**, le doc `:73` dit `fp16` = erroné, sous-estime 2×). n_embd=8192 → 2048 tok = 64 MiB pile,
4096 tok = 128 MiB DÉPASSE. **Relever à 256 MiB** (couvre 8192×4096×4=128 MiB avec marge 2× pour
n_embd≤~16K ; un frame >cap reste rejeté AVANT alloc, `header_to_frame_len:100`). Corriger doc
`fp16`→`fp32`. Ajouter `MAX_SHARD_N_CTX` named const (la borne effective vient du placement :
borner n_ctx borne SIMULTANÉMENT le frame ET le KV-cache).

---

## Invariants confirmés

- **0-bump-wire** (S4 EXECUTE, 0 finding) : aucun nouveau `DOMAIN_*` (clos à 4), aucun
  `*_FORMAT_VERSION` bump (SHARD_PLAN/RUN_PROOF restent =1), le trait/EchoForwarder/GgufContext-ext/
  evaluate_shard_claim sont **internes** (0 serde wire), le frame `sbfb/shard/1` = buffer hidden
  opaque. F2 ne fait que **READ** une signature existante. `ShardAssignment` suffit (0 champ nouveau ;
  n_layer vient du GGUF, n_ctx du launch profile).
- **Scope cut #7** (S2-A) : snapshot ponctuel au claim ≠ pompe VRAM live continue runtime.
- **Day-0** : iroh 0.98 intact, 0 wasmtime, GGUF Q4, groupe privé, 0 nouvelle dep crate (voie FFI
  vendorée = 0 dep, S1a-NO-NEW-CRATE-DEP).
- **Re-coupe F1/F2** sanctionnée README §4 (suffixe chiffré, 0 renumérotation G-K).

## Plan de tests (étendu vs §15 « +1 » — justifié `feedback_ultra_complete_sprints`)

Hermétiques CI (worker-core, hors feature) :
- `shard_assignment_claim_respects_group` (budgeté) : rejet hors ComputeGroup + rejet
  `required>free` + acceptation sinon.
- `claim_rejects_unsigned_manifest_before_gguf_read` : ordre crypto-avant-IO.
- `claim_rejects_assignment_not_in_plan`.
- `claim_rejects_window_out_of_range` : `layer_end > n_layer` defer (pas de crash).
- `estimate_*` : sélection résidents par `is_last`, formule KV, headroom, filtre `blk.{i}` (sizes mockées).

Hermétiques CI (core-rs `shard.rs`) :
- `shard_forward_invokes_forwarder` : un `DoublingForwarder` prouve que ce n'est plus l'echo.
- `shard_forwarder_error_closes_cleanly`.
- (les 5 tokio Phase B re-câblés sur `EchoForwarder`, doivent rester verts.)

Feature-gated `#[ignore]` GGUF (Mistral-7B-Q4) :
- `gguf_tensor_table_sizes_subset` : `GgufContext` étendu somme `blk.{i}` du range < modèle entier.
- `ShardBackendForwarder` exercé sur le rig (drive live = Phase K).

## Risques résiduels

- **R-vendor-patch** : l'extension `GgufContext` ajoute un hunk au patch vendoré → regénérer
  `patches/llama-cpp-shard.patch` + valider builds CPU/CUDA/Metal (F1 a établi l'infra).
- **R-headroom** : `VRAM_BACKEND_OVERHEAD_BYTES` est une heuristique ; calibrée généreuse
  (fail-closed). Le bench réel Phase K affinera.
- **R2 (hérité)** : sizing GGUF + forwarder = feature jamais-CI → double-test (pur hermétique +
  `#[ignore]` GGUF), miroir F1.
