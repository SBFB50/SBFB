# Sprint 77 Phase F1 — Preflight G8 (scope FORK re-cadré)

## Verdict: PLAN-ADAPT

> Preflight Workflow ultracode (6 agents, 5 scans factuels + synthèse) sur le scope FORK
> re-cadré, post-arbitrage PO option (a). Le **DESIGN-CONFLICT initial est levé** (cf.
> `sprint77_phase_f_preflight.md` §Résolution + `sprint77_phase_f_pivot_proposal.md`) — il ne
> se re-déclenche QUE sur une nouvelle infaisabilité du scope fork. **Aucun scan n'en signale.**
> Signaux : F1 PLAN-ADAPT, F2 PLAN-ADAPT, F3 PLAN-ADAPT, F4 EXECUTE, F5 PLAN-ADAPT →
> **verdict global PLAN-ADAPT**. Le code suit l'approche corrigée ci-dessous (pas le §9 brut).
>
> **Note honnêteté process** : l'agent de synthèse du Workflow a renvoyé des placeholders
> (« test ») — un flake de l'étape finale. Les **5 scans sont complets et autoritatifs**
> (evidence fichier:ligne vérifiée) ; cette synthèse est reconstruite à la main depuis eux,
> pas depuis le placeholder.

---

## Décision structurante : split Phase F → **F1 + F2** (0 renumérotation G–K)

Scan F2 (PLAN-ADAPT). La Phase F empile 8 livrables hétérogènes (fork/vendor/build lourd
jamais-CI **vs** claim/wiring réseau hermétique). Frontière d'isolation naturelle = **build**
(environnement-dépendant, LLVM/CUDA/Metal) **vs** **réseau** (pur Rust hermétique). Le suffixe
chiffré `F1`/`F2` est **sanctionné par README §4** (re-coupe de sous-phase, regex `Phase [A-Z]+[0-9]?`
byte-valide) → **les phases G/H/I/J/K et leurs §10–14 restent intacts** (0 renumérotation,
supérieur à « insérer + renommer » qui casserait 5 §-titres + 5 commit titles pour 0 gain). R4
du plan anticipe déjà cette sous-découpe. Bilan : **12 commits feat A–K** (F = F1 + F2).

Un split **n'est PAS un defer** : F1 ET F2 restent dans S77 (directive `feedback_ultra_complete_sprints`
respectée). 0 bump wire, 0 dep, 0 re-DESIGN-CONFLICT.

### Phase F1 — « Forked layer-block backend (partial load buildable) »
`feat(worker): Sprint 77 Phase F1 — forked layer-block backend (partial load + build CUDA/Metal)`

Livrables :
- **(a) Vendoring** des **DEUX** crates (`llama-cpp-sys-2` + `llama-cpp-2`) + `[patch.crates-io]`
  path-override (cf. §Stratégie vendoring) + prérequis **LLVM/libclang** (`LIBCLANG_PATH`).
- **(b) API partial-decode stable** (patch C++ prouvé au spike, porté en vraie API) : 4 champs
  `shard_layer_start/end/is_first/is_last` via `llama_context_params` **AVANT** le reserve des
  buffers ; boucle bornée `[start,end)` ; gather `inp_out_ids` au **dernier layer EXÉCUTÉ**
  (`il==shard_end-1`, PAS `n_layer-1`) ; sortie `is_last ? (norm+lm_head) : (t_embd résiduel brut)` ;
  injection 0-patch (`llama_batch.embd` déjà câblé). + setters `with_shard_*()` côté `-2`
  (`context/params.rs`, miroir `with_n_ctx`).
- **(c) P-D chargement partiel** (`TENSOR_SKIP` dans `create_tensor`, `src/llama-model.cpp`) :
  chaque nœud n'alloue/ne lit QUE `[start,end)` du GGUF. **OBLIGATOIRE** (20 Go ne tient pas sur
  16 ou 8 Go seuls). **Non prouvé au spike** — nouveau code à valider (risque R2).
- **(d) Backend Rust** `llm/llama_cpp.rs` étendu : load layer-subset, forward partiel, extract
  hidden state + top-k extractible (prérequis N0, slot `RunProof.activation_fingerprint` reste
  `[0u8;32]` jusqu'à Phase G).
- **(e) Build vert CUDA ET Metal** matérialisé tôt (R2) + `cargo build --features llm_llama_cpp`.
- Gardes preflight portées : #1 (lire champs `ShardAssignment`), #4 (doc convention shape/dtype,
  S4-F7), #5 (hidden extractible sans perte), #6 (0-bump-wire), #7 (no-float).
- Tests (4) : `shard_backend_loads_layer_subset` (#[ignore] GGUF, P-D VRAM réduite),
  `shard_backend_partial_equals_full` (#[ignore], spike porté bit-exact),
  `shard_backend_hidden_state_extractable` (#[ignore]), `shard_backend_primitive_*` (hermétique CI).

### Phase F2 — « Shard claim + sbfb/shard/1 wiring + gardes + threat »
`feat(worker): Sprint 77 Phase F2 — shard claim + sbfb/shard/1 forward wiring + threat §16`

Livrables :
- **(a) Worker shard claim** (`engine/runtime.rs`, nouveau chemin) : filtre `ComputeGroup::is_member`
  + **cap VRAM fail-closed** (S3-F-2 P1) sur `GpuStats.vram_free_bytes` MESURÉ (snapshot ponctuel,
  modèle `gpu_snapshot` runtime.rs:641 — **PAS de pompe live**, scope cut #7) + **vérif signature
  `ShardedSessionManifest` côté dialer** (S3-F-1 P1, `verify_signature()` shard_plan.rs:355 AVANT
  d'entrer dans la chaîne).
- **(b) Câblage `sbfb/shard/1`** : remplacer le corps echo de `ShardProtocol::accept`
  (shard.rs:233) par recv hidden amont → forward partiel (F1) → send aval.
- **(c) THREAT_MODEL §16 surface shard** (S3-F-5) : SI-1 (reconstruction activations, High) + SI-4
  (collusion inter-workers, High) + caveat « aucun secret app dans les prompts ». (La section
  longue-vie complète reste programmée Phase K ; F2 amorce le caveat minimal co-localisé.)
- **(d) Cap frame** (S3-F-4) : trancher `MAX_SHARD_FRAME_BYTES` (cf. §Cap 64 MiB).
- Gardes preflight portées : #2, #3, #8 (les 2 P1 atterrissent ici où la menace devient concrète).
- Tests (1) : `shard_assignment_claim_respects_group` (hermétique CI : rejet hors ComputeGroup +
  rejet si VRAM-requise > vram_free snapshot + acceptation sinon).

**Renumérotation : AUCUNE.** Plan §15 : ligne F (+5) ventilée F1 (+4) / F2 (+1), total Rust +37
inchangé. §19 : « 11 commits A–K » → « 12 (F = F1+F2) ». G (N0 TOPLOC) démarre dès **F1** fermé
(F1 livre l'extraction du hidden, prérequis G ; F2 n'est pas prérequis de G).

---

## Stratégie vendoring (scan F1, PLAN-ADAPT)

**Découverte clé vérifiée** : `llama-cpp-sys-2 0.1.146` **bundle l'arbre llama.cpp ENTIER** dans
le crate publié (22 Mo / 859 fichiers, snapshot figé, **pas** de submodule à fetch ; provenance
`.cargo_vcs_info.json` = utilityai/llama-cpp-rs sha `4afdaf0`). `build.rs` (1130 l.) lit la source
**en place** (`CARGO_MANIFEST_DIR/llama.cpp`, l.213-214) via CMake — donc un patch appliqué à une
copie vendorée est consommé directement.

**Stratégie recommandée = vendor-local des 2 crates + `[patch.crates-io]` path + patch tracé** :
1. Copier `llama-cpp-sys-2-0.1.146` ET `llama-cpp-2-0.1.146` du registry vers `vendor/` du repo,
   **version 0.1.146 EXACTE conservée** (contrainte `links="llama"` : un override `[patch]` doit
   garder une version ≥ la contrainte, sinon conflit `links` / patch non appliqué — F1-7).
2. Ajouter `[patch.crates-io]` `llama-cpp-sys-2`/`llama-cpp-2` = `{ path = "vendor/..." }` à la
   racine `Cargo.toml`. Le caret `0.1.143` du workspace reste inchangé ; le lock bascule **2 entrées**
   registry→path (diff lock minimal, aucun autre crate touché — F1-9).
3. **Patch sur les 2 crates** (F1-4) : `-sys-2` = 4 champs `shard_*` dans `include/llama.h`
   (bindgen 0.72.1 régénère `bindings.rs` automatiquement) + logique C (`cparams.h`,
   `llama-context.cpp`, `models/llama.cpp`, `llama-model.cpp` TENSOR_SKIP) ; `-2` = setters
   `with_shard_*()` dans `context/params.rs`.
4. **Diff traçable** : versionner `patches/llama-cpp-shard.patch` (git diff vs source vendorée
   vierge) + note d'application, plutôt que des éditions noyées dans 22 Mo (F1-8). Re-appliquable
   au prochain re-vendor upstream.

**Build** : aucune modif `build.rs` requise — sm_120 (RTX 5080) déjà géré par le CMake bundle
(défaut `native` + `120a-real`, F1-3) ; CUDA/Metal via les features existantes `-2/cuda`, `-2/metal`.
**Impact CI = NUL** : aucun job ne build `llm_llama_cpp*` (F1-5). **Impact repo = +~22 Mo** source
vendorée (les 2 crates partagent l'arbre ; `-2` ne contient que le wrapper Rust, pas de duplication).

**Prérequis dur** : **LLVM/libclang** (`LIBCLANG_PATH`) sur Windows pour bindgen (absent, en cours
d'install winget) ; le Mac M2 a déjà libclang via Xcode CommandLineTools (F1-6).

---

## Gardes sécurité (scan F3, PLAN-ADAPT) — répartition F1/F2

| Garde | Sev | État | Sous-phase |
|---|---|---|---|
| Admission server-side ComputeGroup (shard.rs:222-235, AVANT accept_bi) | — | **CÂBLÉE** (inchangée) | — |
| S3-F-1 vérif signature `ShardedSessionManifest` côté dialer (`verify_signature` shard_plan.rs:355) | **P1** | primitive existe, **0 appelant prod** | **F2** |
| S3-F-2 cap VRAM fail-closed au claim sur `vram_free_bytes` mesuré (snapshot ponctuel, pas pompe) | **P1** | **n'existe pas** (consent.rs = `estimated` déclaré, pas mesuré) | **F2** |
| S3-F-4 `MAX_SHARD_FRAME_BYTES=64 MiB` trop serré pour n_embd≥8192 prefill long | P2 | à trancher | **F2** |
| S3-F-5 section surface shard THREAT_MODEL (SI-1/SI-4 + caveat) | P2 | absente (§16 = changelog) | **F2** amorce / **K** complète |

**Estimation VRAM sans charger** (F3-7) : faisable via metadata GGUF (sommer la taille des tenseurs
`blk.{i}.*` dont l'index ∈ `[start,end)`, lecture header-only, + marge KV-cache). Heuristique
documentée acceptable en F2 (somme tailles range × facteur sécurité).

**Cohérence scope cut #7** (F3-4) : le claim fait UN snapshot ponctuel (modèle `gpu_snapshot`
one-shot), **jamais une pompe d'admission continue** (la pompe-live reste post-S77 ; le runtime
garde `estimated_*` via consent).

---

## Wire format (scan F4, EXECUTE) — 0-bump confirmé

- **0 nouveau `DOMAIN_*`, 0 `*_FORMAT_VERSION` bump** (§19 reste clos à 4 ; les 3 shard existent,
  `activation_commit` = N3/Phase H-I). `RunProof.activation_fingerprint` reste `[0u8;32]` (test
  reserved-zero shard_plan.rs:686-689 doit rester vert ; F1 ne touche PAS le slot).
- **Convention hidden state à DOCUMENTER** (commentaire data-plane, **0 struct serde**) : tenseur
  logique `[n_tokens, n_embd]`, dtype **FP32** (`float` C — l'ABI llama.h fixe `llama_batch.embd`
  et `llama_get_embeddings` à `float*` des deux côtés ; ggml dequantize Q4→fp32 pour le calcul,
  le hidden à la frontière API EST fp32 quel que soit le stockage VRAM), **contigu row-major**
  (token-major), **little-endian natif** (x86_64 + aarch64 LE — à documenter explicitement). Les
  octets bruts du frame `sbfb/shard/1` = exactement le buffer `llama_get_embeddings` amont,
  injecté tel quel dans `llama_batch.embd` aval.
- **Ancre d'accord = `launch_profile_hash`** (déjà dans `ShardAssignment`, signé sous
  `DOMAIN_SHARD_PLAN_V1`, lecture seule) : lie n_embd/precision/params entre shards. Une struct
  serde d'en-tête d'activation serait **redondante + hors-budget §19 → DESIGN-CONFLICT artificiel**.
  Ne PAS versionner/serder l'activation.
- **No-float OK** : f32 cantonné au payload opaque non-signé ; `RunMetrics` reste tout-entiers.
- **P-D** = local backend, **0 surface wire** (l'intégrité des poids chargés est couverte par
  `ShardAssignment.shard_hashes` déjà signé).

---

## Cap 64 MiB (S3-F-4 / F3-6) — à trancher en F2

`MAX_SHARD_FRAME_BYTES=64 MiB` = `n_embd × n_tokens × 4` (fp32 — **pas fp16**, cf. F4). Pour la
cible ~20 Go arch-llama : 34B-class n_embd=8192 → 2048 tok = 64 MiB pile, **4096 tok = 128 MiB
DÉPASSE**. (Note : le calcul fp32 est 2× le fp16 supposé par le 1er preflight ; la marge est donc
encore plus serrée.) Trois options : (1) relever à 128/256 MiB avec justification DoS écrite
(borné par n_ctx max du placement) ; (2) chunker le frame ; (3) borner n_ctx au placement Phase D.
**Recommandé : (1) + borne n_ctx documentée** (minimal, traçable, le placement connaît n_ctx).
Ne pas laisser inerte.

---

## Licence + tension Day-0 (scan F5, PLAN-ADAPT) — à acter au commit body

- **F5-1** : la **lettre** « SBFB n'écrit aucun kernel ggml/CUDA » (addendum l.51) **reste tenue** —
  le patch modifie l'orchestration C++ et **réutilise** les kernels ggml verbatim (coupe bit-exact
  prouvée sans toucher un `.cu`/`.metal`).
- **F5-2** : « sans fork » était une **hypothèse de design R&D** (addendum l.100-104 « fork interdit »),
  **PAS une décision Day-0 gelée** de CLAUDE.md (aucune ligne l.436-449 ne dit « no fork »).
  Surclassée par l'arbitrage PO option (a) du 2026-06-21 (`e3ccd10`). À acter, pas re-litiger.
- **F5-3 (P2 actionnable EN-PHASE F1)** : **hygiène licence**. Le crate publié `-sys-2` **strip le
  LICENSE racine de llama.cpp** (via `include=[...]` allowlist) → l'arbre vendoré n'a aucune notice
  MIT. La copie forkée DANS le repo SBFB **DOIT porter** le texte MIT de llama.cpp + copyright
  « ggml authors / Georgi Gerganov et al. » + des **SPDX `MIT`** en tête des fichiers C++ patchés,
  et un `THIRD-PARTY-NOTICES`. Sinon distribution AGPL sans attribution MIT = violation de la clause
  MIT. MIT permissive **compatible** AGPL-3.0-or-later ; obligation source-disclosure AGPL déjà
  satisfaite (SBFB source-verifiable).
- **F5-7** : Day-0 toutes respectées — iroh 0.98 intact, 0 wasmtime introduit, GGUF Q4 retenu
  (scope cut #2/#13 non en tension), groupe privé intact. Carry T-NN+3 (JCS dup) non pertinent →
  reconduit P2.

---

## Décisions structurantes ouvertes (ressort PO)

1. **Vendoring vendor-in-repo (+22 Mo, recommandé) vs git-fork hébergé** : le vendor-in-repo est
   self-contained, CI-safe, aligné AGPL source-verifiable, mais alourdit l'historique git de ~22 Mo
   de façon permanente. L'alternative (fork hébergé GitHub/Radicle + `[patch]` git) évite le bloat
   mais ajoute une dépendance d'hébergement externe (outward-facing) + coordination de rebase. **Le
   PO listait les deux** (handoff : « submodule patché OU source override »). → **question posée**.
2. **Modèle de démo ~20 Go** (Mixtral-8x7B Q3 vs 34B Q4 dense) : décision **Phase K**, pas F1 (F1
   teste sur Mistral-7B-Q4 déjà présent sur les 2 machines depuis le spike). Différée.

---

## Risques résiduels

- **R2 — P-D TENSOR_SKIP non prouvé** : le spike n'incluait PAS le chargement partiel ; `create_tensor`
  est du code C++ nouveau (~100 LoC) sur la logique d'allocation du model loader. Risque d'implémentation
  réel. Mitigation : build tôt + test `#[ignore]` `shard_backend_loads_layer_subset` qui prouve la
  VRAM réduite localement.
- **R2 — build fork jamais-CI** : double test (primitive hermétique CI + `#[ignore]` GGUF runnable
  local). Build CUDA long + Metal via SSH Mac (multi-machine, peut déborder une session).
- **Cap 64 MiB fp32** : franchi pour n_embd≥8192 prefill long (cf. supra).
- **Toolchain F1** : LLVM/libclang install (winget, en cours) ; si échec admin → install user-driven.
