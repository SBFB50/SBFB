# Sprint 77 Phase F — Spike de faisabilité du fork llama.cpp

> Spike JETABLE, hors-sprint (dossier `C:/Users/FlowUP/spike_fork/`, hors repo).
> Décidé après l'arbitrage PO option **(a) fork llama.cpp** (cf.
> `sprint77_phase_f_pivot_proposal.md`) + recherche confirmant qu'aucun chemin
> tiers n'est plus simple (`sprint77_phase_f_backend_research.md`). Objectif :
> GO/NO-GO **avant** d'engager la Phase F, sur le vrai matériel (RTX 5080 + Mac M2).

## Verdict : **GO** (fork faisable, prouvé CPU + CUDA Blackwell + Metal M2 + cross-backend)

Le fork de llama.cpp pour le pipeline layer-split (exécuter un sous-ensemble de
couches `[start,end)`, injecter un hidden state amont, extraire le hidden de
frontière) est **faisable avec un patch minimal (~30 LoC)** et la coupe est
**mathématiquement propre**.

## Le patch (sur la source vendorée llama-cpp-2 0.1.146)

7 éditions, backend-agnostique (compile identique CUDA / Metal / CPU) :
- `src/llama-cparams.h` + `include/llama.h` + `src/llama-context.cpp` : 4 champs
  `shard_layer_start/end/is_first/is_last` passés via `llama_context_params`
  (AVANT le reserve des buffers — leçon : un setter post-création échoue car le
  reserve fige les buffers sur la config par défaut).
- `src/models/llama.cpp` (builder arch LLAMA) :
  - boucle bornée `for (il = shard_start; il < shard_end; ++il)` ;
  - gather `inp_out_ids` au **dernier layer exécuté de chaque shard** (`il ==
    shard_end-1`, PAS seulement `n_layer-1`) — sinon `inp_out_ids` reste
    inutilisé/non-alloué et `set_inputs` assert sur un buffer null (le `build_inp_out_ids`
    retourne TOUJOURS un tenseur, topologie constante pour le pipeline-parallel) ;
  - sortie `is_last ? (output_norm + t_embd + lm_head) : (t_embd = résiduel BRUT)`.
- Injection du hidden amont = **0 patch** : `llama_batch.embd` (champ public) est
  déjà câblé dans `build_inp_embd` (token path vs embd path sélectionné au runtime).

Primitive de validation : `embeddings=true` + pooling NONE → `llama_get_embeddings`
expose le `t_embd` par token (routé sur le résiduel brut pour un shard intermédiaire).

## Preuves (harness `spike.cpp` + `shard_node.cpp`, modèle jouet `stories260K` Llama-arch)

`decode([0,L))` vs `decode([0,k)) → injection frontière → decode([k,L))` :

| Plateforme | Build | max_abs_err | min_cosine | Verdict |
|---|---|---|---|---|
| Windows CPU (AVX512) | VS17 | **0** | 1.00000000 | GO bit-exact (cuts k=1,2,3,4) |
| Windows CUDA sm_120 (RTX 5080 Blackwell) | VS17 + `CMAKE_CUDA_ARCHITECTURES=120a-real` | **0** | 1.00000000 | GO bit-exact (cuts k=1,2,3,4) |
| Mac Metal (M2, macOS 26.3) | Ninja + `GGML_METAL_EMBED_LIBRARY=ON` | **0** | 1.00000000 | GO bit-exact |
| **Cross-backend CUDA→Metal** (5080 émet la frontière, Mac/Metal exécute la queue) | — | 0.0114 | **0.99999888** | **GO** (différence kernels CUDA/Metal négligeable) |

### Validation sur un VRAI modèle quantifié (Mistral-7B-Instruct-v0.3 Q4_K_M, arch llama, L=32, n_embd=4096)

| Test | max_abs_err | mean_abs | min_cosine | Verdict |
|---|---|---|---|---|
| CUDA→CUDA (5080, 32 couches Q4) | **0** | 0 | 1.00000000 | GO bit-exact |
| Metal→Metal (M2, 32 couches Q4) | **0** | 0 | 1.00000000 | GO bit-exact |
| **Cross-backend CUDA→Metal** (5080 émet `[0,16)`, M2 exécute `[16,32)`) | 1.96 | 0.108 | **0.99922882** | **GO** |

Décomposition : chaque backend SEUL est bit-exact même sur 32 couches Q4 → le
`max_abs=1.96` du cross-backend est **purement** la différence de kernels CUDA-vs-Metal
(+ dequant Q4) accumulée sur 16 couches, PAS un bug de coupe (sinon le cosine
s'effondrerait). Le cosine 0.99923 sur le hidden final = direction quasi-identique →
token prédit identique après `lm_head` dans la quasi-totalité des cas. **Donnée de
calibration pour TOPLOC N0** (« même modèle sous seuil », conçu exactement pour cette
variation FP hétérogène) : ~0.1 d'erreur abs moyenne cross-backend sur 16 couches.

Modèle 4.1 Go transféré 5080→Mac en 45s (scp LAN). Le cross-backend est le résultat
phare : un GPU NVIDIA et un GPU Apple Silicon collaborent sur les couches d'un même
modèle via un hand-off (fichier ici), résultat numériquement correct. L'inconnue du
pipeline hétérogène est levée.

## Toolchain validée (sur le vrai matériel)

- **Windows** : RTX 5080 (sm_120/`120a`), nvcc 12.8, cmake 4.3.1, MSVC. Piège levé :
  cmake auto-sélectionne VS **v18** (sans intégration CUDA) → forcer `-G "Visual
  Studio 17 2022"` (intégration CUDA 12.8 en `v170`). `libclang` ABSENT → bloque le
  **build du crate Rust** `llama-cpp-sys-2` (bindgen), PAS le build standalone C++.
  **Prérequis F1 : installer LLVM (`LIBCLANG_PATH`).**
- **Mac** : MacBook Air M2, 8 Go, arm64, clang 17 + CommandLineTools (pas Xcode
  complet → pas de compilateur `metal` CLI → `GGML_METAL_EMBED_LIBRARY=ON` compile
  les shaders au runtime). cmake/ninja via `/opt/homebrew/bin`. `ggml-blas` absent
  de la source vendorée → `-DGGML_BLAS=OFF` sur macOS (Accelerate l'auto-active sinon).

## Contraintes matériel (impact PO sur l'acceptation Phase K)

- Rig de test réel = RTX 5080 (16 Go CUDA) + Mac M2 (8 Go Metal). **Le 70B est
  hors de portée** (~40 Go) → l'acceptation D3 « 70B sur 3-5 machines » ne tourne
  pas sur ce rig. À re-scoper PO : démo réaliste = modèle plus petit.
- Modèle de démo réaliste sur disque : `gemma-4-26b-a4b` (15 Go) — mais **arch
  gemma**, le patch ne couvre que **arch LLAMA** → patcher le builder gemma (~10 LoC)
  OU prendre un modèle Llama-arch (ex. Llama-3.1-8B Q4 ~5 Go, tient sur le Mac 8 Go).
- « Trop gros pour une machine » (le vrai intérêt) nécessite **P-D : chargement
  partiel des couches** (`TENSOR_SKIP`, ~100 LoC) — différé du spike, requis pour
  que le Mac 8 Go ne charge QUE ses couches.

## NON couvert par le spike (→ travail Phase F réel)

- **Transport réseau temps-réel** (le hand-off est par fichier ici) → data plane
  `sbfb/shard/1` (Phase B existe) + mesure tok/s WAN/LAN. C'était le « Niveau 2 »
  différé par le PO.
- **P-D chargement partiel** (économie VRAM, le vrai sharding mémoire).
- **Patch par-architecture** (seul le builder LLAMA est patché ; gemma/qwen/... à part).
- **Build du crate Rust** (libclang) + vraie surface API `llama-cpp-2` (vs params internes).
- KV cache multi-step / génération réelle (spike = prefill-only, un forward).

## Re-cadrage Phase F proposé (pour reprise du sprint)

- **F1** — fork llama.cpp + API partial-decode/inject/extract (le patch ci-dessus,
  stabilisé en vraie API) + build CUDA **et** Metal vert + `cargo build --features
  llm_llama_cpp` (prérequis : LLVM/libclang) + tests primitive hermétiques + `#[ignore]` GGUF.
- **F2** — claim `ShardAssignment` (filtre ComputeGroup + cap VRAM fail-closed) +
  câblage du forward partiel sur le data plane `sbfb/shard/1` (recv hidden amont →
  forward → send aval) + gardes P1 (vérif signature manifest, cap VRAM) + section
  surface shard `THREAT_MODEL`.
- **P-D** (chargement partiel) + **patch multi-arch** : à séquencer selon la cible
  de démo (gemma vs Llama) et l'acceptation re-scopée.
