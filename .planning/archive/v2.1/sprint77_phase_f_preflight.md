# Sprint 77 Phase F — Preflight G8

## Verdict: DESIGN-CONFLICT — RÉSOLU (arbitrage PO option (a) fork + spike GO)

> **Résolution 2026-06-21.** Le PO a tranché l'option **(a) fork llama.cpp**. La recherche
> multi-source (`sprint77_phase_f_backend_research.md`) a confirmé qu'aucun runtime tiers
> (Ollama, exo, Petals, Parallax, candle…) n'est plus simple sous les 4 contraintes SBFB.
> Un **spike de faisabilité jetable** (`sprint77_phase_f_spike.md`) a prouvé le **GO** :
> patch ~30 LoC backend-agnostique, coupe **bit-exact** CPU / CUDA sm_120 (RTX 5080) /
> Metal (Mac M2), et **cross-backend CUDA↔Metal cosine 0.999** sur Mistral-7B Q4 réel.
> L'approche corrigée (fork) remplace le livrable §9.2 « forward partiel via le wrapper
> safe » devenu PLAN-ADAPT exécutable.
>
> **Amendement PO 2026-06-21** : le **70B est ABANDONNÉ**. Cible = un modèle **~20 Go**
> arch-llama éclaté sur le rig réel **RTX 5080 (16 Go CUDA) + Mac M2 (8 Go Metal)**. Un
> 20 Go ne tient sur aucune machine seule → le **chargement partiel des couches (P-D)
> devient OBLIGATOIRE** dans la Phase F. Phase F + Phase K re-cadrées (plan §9 + §14).
>
> Le verdict factuel initial ci-dessous reste l'analyse qui a mené à cet arbitrage.

Deux scans independants (S1a OSS prior-art + verification adversariale) signalent le MEME conflit non-resolu et la refutation a ECHOUE : le livrable coeur §9.2 #1 (forward PARTIEL d'un bloc `layer_start..layer_end` avec injection d'un hidden state amont en entree et extraction du hidden state intermediaire de frontiere) est **infaisable via le wrapper safe `llama-cpp-2`** pinne au workspace ; le fallback R2 du plan (§9.4 « degrader N2 ») ne couvre que l'extraction du hidden state FINAL et n'adresse donc PAS la dimension infaisable. Conformement a la regle G8 (DESIGN-CONFLICT si un scan le signale et n'est pas resolu), le verdict global est DESIGN-CONFLICT et l'arbitrage PO est requis AVANT le 1er Edit Phase F.

---

## S1a — OSS prior-art (faisabilite extraction hidden state + forward partiel layer-subset)

**Signal : DESIGN-CONFLICT.** Le plan Phase F conflate deux capacites distinctes. La (1) — extraction du hidden state final + top-k — est faisable sans fork ; la (2) — forward partiel d'un sous-ensemble de couches avec injection d'entree — ne l'est pas via l'API choisie.

| Finding | Sev | Claim | Evidence |
|---|---|---|---|
| S1a-1 | P1 | Hidden state FINAL + logits **extractibles sans fork** : `LlamaContext::embeddings_ith(i) -> Result<&[f32], EmbeddingsError>` (slice `n_embd`, dernier hidden layer apres norm avant lm_head) + `get_logits`/`get_logits_ith`/`candidates_ith`. Couvre TOPLOC N0 sur le DERNIER shard. Le plan a raison sur ce point precis. | docs.rs/llama-cpp-2 LlamaContext ; Context7 `/utilityai/llama-cpp-rs` ; ggml-org/llama.cpp/discussions/3643 |
| S1a-2 | **P0** | FORWARD PARTIEL d'un range `layer_start..layer_end` (injecter un hidden state amont, executer un bloc, extraire le hidden state intermediaire) = **INFAISABLE via le wrapper safe**. `LlamaModel` n'expose aucune methode layer-range : `n_layer()` read-only, pas d'eval-callback, pas d'injection d'embedding d'entree, pas d'arret a une couche. Exige un fork/patch C++. | docs.rs/llama-cpp-2 LlamaModel ; `sharding_design_addendum_sota_2026-05-30.md:98-104` (muet sur forward partiel) |
| S1a-3 | P1 | `gguf-split` ne produit PAS de shards executables par-couche standalone : partitions tensor d'UN modele monolithique co-charge par un seul process. Pas de voie « GGUF par shard » native. | ggml-org/llama.cpp/discussions/6404 + PR #6187 ; `tools/gguf-split/README.md` |
| S1a-4 | P1 | Aucun SOTA pipeline-parallel cross-machine n'utilise GGUF/llama.cpp pour le forward partiel. Parallax tourne sur sglang/vLLM/MLX. HyperCluster fait du selective layer loading via acces tensor bas-niveau (pas un wrapper safe). RPC backend llama.cpp = graphe central, worker passif. | github.com/GradientHQ/parallax ; HyperCluster Springer 10.1007/978-3-032-27358-1_2 ; llama.cpp/tools/rpc/README.md |
| S1a-5 | P1 | R2 (kickoff l.781) et §9.4 conflatent les 2 capacites : ils n'adressent que l'extraction finale (cas 1). Le fallback « degrader N2 » ne resout PAS l'infaisabilite du forward partiel (cas 2), coeur du 70B-eclate du scope MAXIMAL PO. Pas de fallback pour cette dimension. | `sprint77_plan.md:295-319` ; `sprint77_kickoff.md:781` + `:13-19` |
| S1a-6 | INFO | Options d'arbitrage PO : (a) fork/patch llama.cpp (forward partiel + injection via eval-callback ggml, effort C++) ; (b) Phase F reduite au forward COMPLET in-process + extraction finale + primitives + spike (SCOPE-CUT a assumer contre 0-defer) ; (c) backend ggml custom. Porter au PO avant le 1er Edit. | `sharding_design_addendum_sota_2026-05-30.md:51-53` (SBFB n'ecrit aucun kernel ggml/CUDA) ; `feedback_ultra_complete_sprints.md` |

**Conclusion S1a :** conflit design reel. L'archi pipeline-parallel est figee, mais le BACKEND d'execution du bloc n'est pas resolu. NE PAS coder §9.2 #1 tel qu'ecrit.

---

## S1b — Deps / CVE

**Signal : EXECUTE.** Phase F n'ajoute AUCUNE dependance.

| Finding | Sev | Claim | Evidence |
|---|---|---|---|
| S1B-F-1 | INFO | 0 nouvelle dep : `llama-cpp-2` + `llguidance` declares optional/feature-gated au workspace depuis S20 ; `llm_llama_cpp = ["dep:llama-cpp-2","dep:llguidance"]`. Phase F etend l'usage, n'introduit pas de crate. | `crates/nexus-worker-core/Cargo.toml:27,142,148` ; `Cargo.toml:362,373` |
| S1B-F-2 | INFO | Discrepance version (non bloquante) : plan/D4 cite `0.1.143` mais le lock resout **0.1.146** (caret `^0.1.143`). Resolution PRE-existante (lock committe S74) — Phase F ne provoque PAS de bump. Ne PAS regenerer/bump le lock. Corriger un eventuel doc-note D4 en 0.1.146 (cosmetique). | `Cargo.lock:4623-4624` (llama-cpp-2 0.1.146) ; `Cargo.toml:362` contrainte `^0.1.143` ; lock = `b76a084` (S74) |
| S1B-F-3 | INFO | 0 advisory RUSTSEC pour `llama-cpp-2`/`llama-cpp-sys-2`. Le CVE llama.cpp actif (CVE-2026-21869/27940, OOB write) vise le SERVEUR HTTP C/C++ upstream (`tools/server`), hors surface du binding in-process (`LlamaBackend::init`/`load_from_file`). | rustsec.org/packages/ ; nvd.nist.gov/.../cve-2026-27940 ; `llm/llama_cpp.rs:69-73,152` |
| S1B-F-4 | INFO | Zones rouges deps intactes : iroh pinne **0.98.2** (R-iroh-audit), **0 wasmtime** dans le lock (R-wasmtime-cve, coherent « OS sandbox pas wasmtime »). Phase F ne touche ni iroh ni la surface sandbox. | `Cargo.lock` iroh 0.98.2 ; `grep "name = \"wasmtime\"" Cargo.lock` = 0 match (verifie) |
| S1B-F-5 | INFO | Pas de `cargo audit` local (non installe), verif manuelle Cargo.lock + RUSTSEC/NVD. Reco non bloquante : ajouter `cargo audit` au pipeline pour la chaine build-time `llama-cpp-sys-2` (bindgen/cc/cmake/find_cuda_helper) plutot qu'un gate manuel. | `Cargo.lock:4636-4648` ; docs.rs/llama-cpp-2 (EmbeddingsError) ; `sprint77_plan.md:319` |

**Conclusion S1b :** aucun blocage deps/CVE. Note : version effective = **0.1.146**, lock fige, ne pas bumper.

---

## S2 — Decisions historiques

**Signal : PLAN-ADAPT.** Aucun conflit avec une decision figee ; les 3 contraintes critiques sont deja honorees. Le motif PLAN-ADAPT est une lacune d'API amont (la meme que S1a-2/S1b), pas un conflit de decision.

| Finding | Sev | Claim | Evidence |
|---|---|---|---|
| S2-F-01 | INFO | (a) Slot `activation_fingerprint=[0u8;32]` RESERVE Phase G, force a zero par `RunProof::new`. Phase F prepare l'extraction top-k mais ne remplit PAS le slot wire. | `shard_plan.rs:431-439` + `:466` ; `sprint77_plan.md:298` |
| S2-F-02 | INFO | (b) No-float wire HONORE : `RunMetrics` tout-entiers, fige Phase C. L'extraction TOPLOC calcule sur float en interne (autorise) mais ne touche aucune struct signee ; slot fingerprint = `[u8;32]`. | `shard_plan.rs:379-392` + `:46-53` ; `sprint77_phase_c_preflight.md:155-165` |
| S2-F-03 | INFO | (c) Backend jamais-CI -> double test : decision D4 + risque R2, deja au plan §9.3/9.4. Code confirme : `llama_cpp.rs` documente « CI runs without the feature and never touches this module ». | `llm/llama_cpp.rs:41-52` ; `sprint77_plan.md:304-318` ; `sprint77_kickoff.md:781` |
| S2-F-04 | P1 | MOTIF PLAN-ADAPT (pas DESIGN-CONFLICT cote S2) : lacune d'API amont. `embeddings_ith`/logits couvrent le hidden state FINAL, mais aucune primitive pour (i) charger un range contigu `layer_start..layer_end` ni (ii) emettre un hidden state INTERMEDIAIRE de frontiere. Seul controle = `n_gpu_layers` (offload count). A materialiser tot via `cargo build --features llm_llama_cpp` + spike layer-subset AVANT le claim. | Context7 `/utilityai/llama-cpp-rs` ; `llm/llama_cpp.rs:135,150` ; `addendum:98-104` ; `sprint77_plan.md:296-298` |
| S2-F-05 | INFO | Pre-requis Phase B PRESENT : `ShardProtocol::accept` rejette tout pubkey hors allowlist au handshake (`is_member` AVANT `accept_bi`) ; code annote « Phase F replaces the echo body ». D5 deja cable data-plane, Phase F branche le backend sous ce gate. | `shard.rs:192,227` (verifie) ; `sprint77_plan.md:299-301` |
| S2-F-06 | P2 | Couplage VRAM claim-gate : §9.2 demande un claim « sous caps VRAM », mais scope cut #7 gele la VRAM-live admission runtime a post-S77 (« la pompe runtime garde le check sur estimated_* declare »). Phase F doit filtrer sur les caps DECLARES (`runtime.rs:1340 max_vram_mb` / `:980 estimated_vram_mb`), PAS introduire une pompe `gpu.snapshot()` live (sinon scope creep #7). | `sprint77_kickoff.md:745` ; `runtime.rs:980,1340` ; `sprint77_plan.md:299-300` |
| S2-F-07 | INFO | Carry T-NN+3 (canonical_bytes dup JCS) NON pertinent Phase F : pas de `canonical.rs`/`DOMAIN_*`/primitive signee touchee. Carry reconduit, pas d'absorption opportuniste ici. | `sprint77_kickoff.md:679` ; `sprint77_plan.md:294-301` |

**Note de coherence S2-F-06 vs S3-F-2 :** S2 (decisions) impose de NE PAS introduire de pompe VRAM live (scope cut #7) ; S3 (threat) demande un cap VRAM fail-closed au claim. Resolution : le cap au claim doit utiliser la VRAM **deja mesuree** au boot/snapshot existant (`GpuStats.vram_free_bytes`, deja peuple), comparee a la VRAM requise par le range — SANS armer une nouvelle pompe d'admission runtime continue. C'est un check ponctuel au moment du claim, pas la pompe runtime gelee par #7.

**Conclusion S2 :** 0 conflit de decision figee. La lacune layer-subset (S2-F-04) renforce S1a-2.

---

## S3 — Threat model

**Signal : PLAN-ADAPT.** 2 menaces deja couvertes (Phase B), 2 gardes manquent au plan §9.

| Finding | Sev | Claim | Evidence |
|---|---|---|---|
| S3-F-1 | INFO | Menace #1 (claim hors ComputeGroup) DEJA mitigee server-side au handshake (rejet sur `conn.remote_id()` Ed25519 non-spoofable, AVANT `accept_bi`). Complement Phase F : le DIALER amont doit verifier la signature du `ShardedSessionManifest` (`DOMAIN_SHARD_PLAN_V1`, initiateur) au claim — garde a expliciter §9.2 (sinon un manifest forge place un worker honnete dans une chaine non-autorisee). | `shard.rs:222-231` (verifie) + `:203-208` ; `shard_plan.rs:144-147` |
| S3-F-2 | **P1** | Menace #2 (bypass cap VRAM au claim = DoS/OOM mid-pipeline). La VRAM existe a 2 endroits DISJOINTS qui ne couvrent PAS le claim de shard : `placement.rs` (initiateur-side, ne touche pas le runtime) et `consent.rs:422-425` (chemin TASK, pas shard). Phase F DOIT comparer la VRAM requise par `[layer_start,layer_end)` a `GpuStats.vram_free_bytes` MESURE local, **fail-closed** (defer/refuse, ne pas crasher), AVANT de charger. A nommer comme livrable §9.2. | `placement.rs:21-26,322-323` ; `consent.rs:422-425` ; `runtime.rs:641` (gpu_snapshot) ; `sprint77_plan.md:299-301` |
| S3-F-3 | INFO | Menace #3 (hidden-state poisoning) correctement DEFEREE a N0-N3 (Phases G-I). Phase F ne VERIFIE pas mais DOIT PRODUIRE le top-k (k=128) du dernier hidden state **sans perte d'info** (slot `RunProof.activation_fingerprint` reserve+zeroe Phase C). SI extraction logits impossible sans fork -> degrader (§9.4 prevu). | `shard_plan.rs:40-44` ; `sprint77_plan.md:297-298` + §9.3 #2 + `:319` ; `sprint77_kickoff.md:457-462` |
| S3-F-4 | INFO | Menace #4 (DoS buffers recus) DEJA mitigee Phase B : `read_frame` enforce `MAX_SHARD_FRAME_BYTES=64 MiB` AVANT allocation. Garde residuel : re-valider que pour le 70B (hidden dim reel + max prefill) un hidden state legitime ne depasse pas 64 MiB (faux-positif), et borner nb frames/duree d'une connexion inactive amont/aval (slowloris-like, non couvert par le cap de taille). | `shard.rs:78,100-149` (verifie) ; pas de timeout/idle-cap observe |
| S3-F-5 | **P1** | THREAT_MODEL.md n'a PAS la section surface shard promise par kickoff D4/D5 (« §16 »). §16 = « Revue et evolution » (verifie), §15 = surface seed cross-noeud ; SI-1..SI-5 confines a `SPLIT_INFERENCE_DESIGN.md:198-202`. Phase F = 1ere phase qui calcule sur des activations EN CLAIR entre pairs -> armer §16.3 (« nouveau composant -> §5.x STRIDE + §6 LINDDUN + §2 Assets + §4 DFD »). Porter SI-1 (reconstruction activations, High) + SI-4 (collusion inter-workers, High) + caveat « aucun secret app dans les prompts ». | `THREAT_MODEL.md:980` (§16) + `:825` (§15 seed) — verifie ; `SPLIT_INFERENCE_DESIGN.md:198-202` ; `sprint77_kickoff.md:455,510` ; `shard.rs:42-45` |
| S3-F-6 | P2 | Surface additionnelle : `ShardAssignment.shard_hashes` (BLAKE3 pin) + `launch_profile_hash`. Pour un sens DEFENSIF, le worker honnete devrait verifier au chargement que les bytes des couches chargees correspondent aux `shard_hashes` AVANT de produire des activations (sinon mauvais GGUF/quant pollue silencieusement, N0 ne l'attrape qu'aval). A minima emettre les `shard_hashes` mesures dans le RunProof (pas de perte d'info, cf. S3-F-3). Doc dit « detectable », pas « rejected » — coherent avec deferer la garantie a N0. | `shard_plan.rs:161-164,176-178` ; `sprint77_plan.md:295-298` |

**Conclusion S3 :** PLAN-ADAPT. 2 gardes P1 a cabler/nommer (cap VRAM fail-closed au claim S3-F-2 ; section surface shard THREAT_MODEL S3-F-5) + verif signature manifest au claim (S3-F-1) + P2 borne idle-connexion (S3-F-4) + emission shard_hashes (S3-F-6).

---

## S4 — Wire format

**Signal : EXECUTE.** Phase F est 0-BUMP-WIRE.

| Finding | Sev | Claim | Evidence |
|---|---|---|---|
| S4-F1 | INFO | 0-BUMP-WIRE : Phase F ne touche aucune struct wire signee. Modifie `llm/llama_cpp.rs` (backend) + `engine/runtime.rs` (claim). CONSOMME `ShardAssignment` (lecture `layer_start/layer_end`), PRODUIT un hidden state binaire sur `sbfb/shard/1`. Top-k dans RunProof = Phase G. | `sprint77_plan.md:294-302` ; `shard.rs:190-194` |
| S4-F2 | INFO | `sbfb/shard/1` = canal d'octets opaques length-prefixed (4-byte BE len + payload), borne `MAX_SHARD_FRAME_BYTES=64 MiB` (anti-DoS, write ET read). Pas besoin de nouvelle struct wire signee pour le hidden state. | `shard.rs:70-150` (verifie) |
| S4-F3 | INFO | `ShardAssignment` expose les champs lus par le backend : `layer_start: u32` (inclusif), `layer_end: u32` (exclusif), bloc `[start,end)`. Claim lit en plus `worker_pubkey`, `shard_hashes`, `role`, `kv_cache_policy`, `launch_profile_hash` (lecture seule). | `shard_plan.rs:142-179` (verifie) |
| S4-F4 | INFO | Filtre groupe prive = allowlist `ComputeGroup::is_member` deja signee (`conn.remote_id` QUIC-authentifie). Cap VRAM lue depuis `GpuStats.vram_free_bytes/vram_total_bytes` = type LOCAL non-wire (jamais signe/propage). 0 format wire introduit. | `compute_group.rs:238` ; `shard.rs:226-231` ; `gpu/nvml.rs:117-121` |
| S4-F5 | INFO | 0 DOMAIN_* nouveau. Budget §19 clos a QUATRE : compute_group / shard_plan / run_proof / activation_commit (ce dernier reserve N3). 3 existent en code. Phase F = 0 DOMAIN_*, 0 `*_FORMAT_VERSION` bump (pre-launch additive). | `routing.rs:30-37` ; `canonical.rs:255,273,286` |
| S4-F6 | INFO | `RunProof.activation_fingerprint` reste `[0u8;32]` (slot N0 reserve). `RunProof::new` l'init a zero, test `run_proof_signature_roundtrip` asserte « reserved-zero until Phase G ». Phase F conserve l'invariant. | `shard_plan.rs:431-439,466,686-689` |
| S4-F7 | P3 | A clarifier au PLAN (pas un conflit) : le format du hidden state (shape, dtype fp16/bf16, ordre couches) = convention INTERNE du data plane NON-signe. La regle no-float ne s'applique qu'aux payloads SIGNES. Documenter la convention shape/dtype (accord 2 shards via `launch_profile_hash`) SANS struct serde signee. Une struct serde signee pour l'activation -> hors-budget §19 -> DESIGN-CONFLICT (PAS le plan actuel). | `addendum:141-144` ; `shard_plan.rs:46-53,176-178` |

**Conclusion S4 :** EXECUTE, 0-bump-wire confirme. Documenter la convention shape/dtype du tenseur sans creer de struct serde signee.

---

## Verification adversariale (crux S1a)

**Signal : DESIGN-CONFLICT (refutation ECHOUEE).** Tentative active de trouver une voie no-fork pour le forward partiel — chaque piste close. La conclusion S1a TIENT.

| Finding | Sev | Claim | Evidence |
|---|---|---|---|
| S1b-1 | **P0** | S1a-2 CONFIRME. `LlamaContextParams` ne contient ni `cb_eval`, ni eval-callback, ni `with_cb_eval`, ni abort-callback, ni limiteur de couches, ni injection d'embeddings : uniquement n_ctx/n_batch/pooling_type/embeddings/etc. Seul `embeddings_ith` = decode FULL-MODEL. Impossible de charger un range + injecter une entree + extraire un intermediaire via cette API. | `Cargo.toml:362` (verifie) ; docs.rs/llama-cpp-2 `LlamaContextParams` ; Context7 `/utilityai/llama-cpp-rs` ; `llm/llama_cpp.rs:152` |
| S1b-2 | P1 | La piste eval-callback (seule voie no-fork plausible) NE rend PAS faisable : `ggml_backend_eval_callback` n'existe que dans `llama-cpp-sys-2` (FFI unsafe), OBSERVE les tenseurs intermediaires pendant un forward COMPLET + early-stop, mais (a) n'injecte pas un hidden state amont, (b) ne demarre pas a une couche arbitraire (N premieres couches calculees quand meme), (c) non remonte dans le wrapper safe. | docs.rs/llama-cpp-sys-2 `ggml_backend_eval_callback` ; absence dans docs.rs/llama-cpp-2 |
| S1b-3 | P1 | Voie graph-split runtime inexistante : hidden state d'une couche arbitraire + early-exit exige un decoupage AHEAD-OF-TIME (compile-time), exit-layer FIXE, modif KV cache, contournement des « layer-number-specific checks ». PAS dans l'API runtime publique. | ggml-org/llama.cpp/discussions/10787 (LayerSkip) |
| S1b-4 | P1 | RPC backend ne donne PAS de bloc autonome : distribue les couches (transmet seulement le hidden state, few kB) mais via rpc-server PASSIFS pilotes par UN graphe orchestrateur central. Pas un shard node-centrique decentralise. Corrobore S1a-4. | WebSearch llama.cpp RPC ; ggml-org/llama.cpp/discussions/20252 ; `tools/rpc/README.md` |
| S1b-5 | P1 | Preuve par l'exemple inverse : **prima.cpp** — seul SOTA 70B pipeline-parallel par blocs sur GGUF (piped-ring, scheduler Halda) — est un **FORK** de llama.cpp (« our distributed implementation of llama.cpp »), pas un consommateur du wrapper safe. Demonstration directe : la cible Phase F se realise par fork/patch, jamais via l'API safe. Renforce S1a-4. | ggml-org/llama.cpp/discussions/12852 (prima.cpp) ; arxiv.org/pdf/2504.08791 |
| S1b-6 | P1 | SCOPE-CUT-CONSISTENT INVALIDE comme chemin Phase F. Le fallback R2 ne traite que la capacite (1), qui est faisable (R2 ne se declenche meme pas) ; (2) le forward partiel — vrai coeur du 70B-eclate — n'a AUCUN fallback. Un scope-cut consistant preserverait l'objectif ; ici il ne touche pas la dimension infaisable. Signal correct = DESIGN-CONFLICT. | `sprint77_plan.md:294-319` ; `sprint77_kickoff.md:457-459` |
| S1b-7 | INFO | Arbitrage PO REQUIS avant le 1er Edit. Options gradees par fidelite 70B vs effort/0-defer : (a) FORK/patch llama.cpp (modele prima.cpp) ; (b) Phase F = forward COMPLET in-process + extraction finale + primitives [Phase C en place] + claim + spike toy, DIFFERER le forward partiel (SCOPE-CUT explicite contre 0-defer) ; (c) backend ggml custom. §9.4 (build feature + hermetiques + #[ignore] GGUF) reste atteignable pour les sous-livrables (b). | `addendum` (SBFB n'ecrit aucun kernel ggml) ; `feedback_ultra_complete_sprints.md` ; `sprint77_kickoff.md:13-19` ; `sprint77_plan.md:309-312` |

**Conclusion adversariale :** la refutation a echoue sur tous les angles (eval-callback, graph-split runtime, RPC, exemple prima.cpp). Capacite (1) extraction finale = faisable ; capacite (2) forward partiel = infaisable sans fork. DESIGN-CONFLICT confirme.

---

## Approche retenue pour le code Phase F

**Le verdict est DESIGN-CONFLICT : NE PAS coder le livrable §9.2 #1 (forward partiel) tel qu'ecrit. L'arbitrage PO est requis AVANT le 1er Edit.** Tant que le PO n'a pas tranche entre (a)/(b)/(c), aucun code du forward partiel ne doit etre ecrit.

### Decision a remonter au PO (bloquante)

Le design pipeline-parallel est figE (Day-0 D3, addendum), MAIS le **backend d'execution du bloc** n'a pas de chemin via le wrapper safe `llama-cpp-2`. Trois options :

- **(a) Fork/patch llama.cpp** — exposer un `llama_decode` partiel + injection d'embeddings d'entree via graph-split C++ (modele prima.cpp). Maximise la fidelite au scope MAXIMAL 70B, mais brise « sans fork », effort C++, en tension directe avec « SBFB n'ecrit aucun kernel ggml/CUDA » (addendum:51-53).
- **(b) Phase F = forward COMPLET in-process + sous-livrables realisables** — le worker tient le modele entier (ou un endpoint) + extraction du hidden state FINAL [faisable, S1a-1] + primitives shard wire [deja Phase C] + claim ComputeGroup/VRAM + spike toy multi-process. DIFFERE le forward partiel reel a une phase/sprint dedie. **SCOPE-CUT explicite a assumer contre la directive 0-defer** — preserve N0/wire/transport/claim mais perd le 70B-eclate reel ce sprint.
- **(c) Backend ggml custom from-scratch** — meme tension que (a), effort maximal.

**Recommandation preflight :** porter le DESIGN-CONFLICT au PO. Si le PO choisit (b), le code Phase F ci-dessous est executable pour les sous-livrables non-conflictuels.

### Points de code (executables quel que soit l'arbitrage, pour les sous-livrables hors forward-partiel)

Ces points sont valides pour (b) integralement, et restent les pre-requis pour (a)/(c) :

1. **Champs `ShardAssignment` a lire** (lecture seule, 0-bump-wire — S4-F3) : `layer_start: u32` (inclusif), `layer_end: u32` (exclusif), bloc `[layer_start, layer_end)` ; `worker_pubkey` (doit etre `self`), `shard_hashes` (pin BLAKE3), `role` (`LayerWorker`), `kv_cache_policy`, `launch_profile_hash`. Fichiers : `shard_plan.rs:142-179`.

2. **Filtre groupe prive au claim** (S2-F-05, S3-F-1) : reutiliser l'admission Phase B (`ComputeGroup::is_member` au handshake `sbfb/shard/1`, deja cable server-side, `shard.rs:227`). **Garde a AJOUTER cote dialer** : verifier la signature du `ShardedSessionManifest` (`DOMAIN_SHARD_PLAN_V1`, initiateur) au claim avant d'entrer dans une chaine — sinon un manifest forge place un worker honnete dans un pipeline non-autorise.

3. **Cap VRAM au claim, fail-closed** (S3-F-2 P1, contrainte de coherence S2-F-06) : comparer la VRAM requise par `[layer_start, layer_end)` a `GpuStats.vram_free_bytes` **deja mesure** (`runtime.rs:641 gpu_snapshot`), AVANT de charger les couches. Sur depassement : **defer/refuse le claim, ne pas crasher**. Important — utiliser la VRAM deja snapshotee + les caps DECLARES existants (`runtime.rs:980 estimated_vram_mb`, `:1340 max_vram_mb`) ; **NE PAS** armer une nouvelle pompe d'admission VRAM-live continue (scope cut #7, kickoff:745).

4. **Format hidden state** (S4-F7 P3) : tenseur binaire opaque sur `sbfb/shard/1` via `write_frame`/`read_frame` existants (cap `MAX_SHARD_FRAME_BYTES=64 MiB`, `shard.rs:78`). **Documenter** la convention shape/dtype (fp16/bf16, ordre couches ; accord 2 shards consecutifs via `launch_profile_hash`) en commentaire — **SANS** creer de struct serde signee (sinon hors-budget §19 -> DESIGN-CONFLICT).

5. **Defere a Phase G** (S2-F-01, S3-F-3, S4-F6) : le slot `RunProof.activation_fingerprint` reste `[0u8;32]` (encodage LSH top-k k=128 reel = Phase G). Phase F doit rendre le top-k du dernier hidden state EXTRACTIBLE sans perte d'info (prerequis N0, test §9.3 #2) mais ne remplit PAS le slot.

6. **0-bump-wire confirme** (S4-F5) : 0 DOMAIN_* nouveau, 0 `*_FORMAT_VERSION` bump. §19 reste clos a 4.

7. **No-float respecte** (S2-F-02) : le calcul TOPLOC sur float interne (`embeddings_ith -> &[f32]`) est autorise ; aucun f32/f64 ne doit entrer dans un payload signe (`RunMetrics` reste tout-entier).

8. **THREAT_MODEL** (S3-F-5 P1) : creer la section surface shard promise par D4/D5 (actuellement inexistante — §16 = « Revue et evolution »). Porter SI-1 (reconstruction activations, High) + SI-4 (collusion inter-workers, High) + caveat « aucun secret app dans les prompts » depuis `SPLIT_INFERENCE_DESIGN.md:198-202`. A nommer dans le plan Phase F (ou deferer explicitement a un wrap-up nomme).

### Si le PO choisit (b) — SCOPE-CUT a documenter

Doc-note exact a inscrire (commit body + `nexus_grid_pivot.md`) : « **SCOPE-CUT Phase F (arbitrage PO) :** le forward PARTIEL d'un range `layer_start..layer_end` avec injection d'un hidden state amont est infaisable via le wrapper safe `llama-cpp-2` 0.1.146 (aucun eval-callback/layer-range/injection ; SOTA 70B-GGUF = fork prima.cpp). Phase F livre le forward COMPLET in-process + extraction hidden state final + primitives shard + claim ComputeGroup/VRAM + spike. Le forward partiel reel (70B-eclate cross-machine) est DIFFERE a [phase/sprint dedie], routE comme carry P1 avec rig de convergence. Contre la directive 0-defer du coeur — assume explicitement par arbitrage PO. »

---

## Risques residuels & notes

- **R2 (backend `llm_llama_cpp` jamais en CI) -> double test** : maintenu. Tests hermetiques (3-4, sans GGUF, dans CI) + `#[ignore]`-gated GGUF (runnable localement). Materialiser `cargo build -p nexus-worker --features llm_llama_cpp` **TOT** (1er livrable), avant le claim runtime, pour faire echouer vite si la chaine build-time native (`bindgen/cc/cmake/find_cuda_helper`) casse. **Ajout preflight :** ce build precoce sert aussi de spike de faisabilite layer-subset — il confirme empiriquement l'infaisabilite S1a-2/S1b-1 (aucune API de range) avant tout code de forward partiel.
- **Version effective `llama-cpp-2` = 0.1.146** (lock fige S74, caret `^0.1.143`). Ne PAS regenerer/bump le lock. Corriger un eventuel doc-note D4 citant 0.1.143 en 0.1.146 (cosmetique, non bloquant).
- **Cap VRAM au claim (S3-F-2)** : fail-closed obligatoire ; un worker qui accepte un range trop gros fait tomber tout le pipeline (single-slowest-worker fragility). Utiliser la VRAM deja mesuree, pas une nouvelle pompe (scope cut #7).
- **Dimensionnement 64 MiB (S3-F-4)** : re-valider qu'un hidden state legitime 70B (hidden dim reel + max prefill tokens) ne depasse pas `MAX_SHARD_FRAME_BYTES` (faux-positif DoS auto-inflige). Borner aussi nb frames + duree d'une connexion ALPN inactive (slowloris-like, P2).
- **shard_hashes (S3-F-6 P2)** : a minima emettre les `shard_hashes` mesures dans le RunProof pour permettre la detection aval du swap-de-poids (pas de perte d'info) ; verif fail-closed au load = a trancher (coherent avec deferer la garantie a N0).
- **Carry T-NN+3 (JCS dup)** : reconduit, non absorbable Phase F (zone non-JCS).
- **Zones rouges deps intactes** : iroh 0.98.2 pinne, 0 wasmtime, 0 advisory `llama-cpp-2`/`-sys-2`. CVE llama.cpp = serveur HTTP upstream, hors surface in-process.
