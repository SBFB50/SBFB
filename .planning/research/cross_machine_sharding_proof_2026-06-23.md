# Cross-machine sharding — preuve live (2026-06-23)

Rapport de toutes les stats mesurées lors de la session de preuve du sharding
multi-ordinateur (Sprint 77, feature shard). Toutes les valeurs sont **mesurées
en exécution réelle**, pas estimées.

---

## 0. Résultat principal — split cross-machine == modèle entier

Un modèle (Mistral-7B-Instruct-v0.3 Q4) éclaté sur **deux machines physiques
hétérogènes**, chacune ne chargeant que la moitié des couches, l'état-frontière
traversant le **réseau LAN**, produit le **même résultat** que le modèle entier.

| Métrique | Valeur |
|---|---|
| **cosine(full, split)** | **0.999727** (seuil de validité > 0.999) |
| Verdict | **PROUVÉ : Windows-head + Mac-tail == modèle entier** |
| HEAD (couches [0,16)) | **Windows / RTX 5080 / x86 / CPU** |
| Transport état-frontière | **LAN** (pipe binaire SSH), 98 304 octets |
| TAIL (couches [16,32)) | **Mac M2 / arm64 / Metal** (`ggml_metal_device_init` confirmé) |
| Modèle | Mistral-7B-Instruct-v0.3 Q4 — 32 couches, embedding 4096 |
| Prompt | `"The quick brown fox"` → 6 positions × 4096 = 24 576 f32 |
| `full.bin` / `split.bin` | 98 304 octets chacun (identique en forme) |

C'est cross-machine **ET** cross-backend (x86/CPU ↔ ARM/Metal) **ET** cross-OS
(Windows ↔ macOS).

---

## 1. Matériel

| Hôte | Détail | Rôle |
|---|---|---|
| **Windows** | RTX 5080, 16 Go VRAM (15 751 / 16 303 MiB au pic), x86_64 | head shard + référence + multi-node |
| **Mac** | M2 (Darwin 25.3.0, T8112), arm64, 8 Go, Metal — `192.168.1.53` (LAN) | tail shard |
| **VPS** | Linux Ubuntu (Hetzner `sbfb-eu`) — `135.181.42.188` (WAN) | joignable, non utilisé dans la preuve finale |

---

## 2. Baseline tok/s — modèle ENTIER, mono-machine (5080, Ollama)

| Modèle | Taille | gen tok/s | prompt tok/s | TTFT | VRAM |
|---|---|---|---|---|---|
| `llama3.1:8b` | ~5 Go | **121.4** | 225.1 | 0.12 s | large |
| `gemma-4-26B-A4B-it Q4` (MoE 4B actifs) | ~15 Go | **64.2** | — | (load 26.6 s) | **15 751 / 16 303 MiB** (tient au ras) |

→ La 5080 seule couvre déjà jusqu'à du 26B-class à 64 tok/s en maxant sa VRAM.

---

## 3. Tests de correction du split — in-process (Windows, CPU, Mistral-7B)

`cargo test -p nexus-worker-core --features llm_llama_cpp --release -- --ignored shard_backend`
sur `spike_fork/mistral-7b-q4.gguf` (Mistral-7B-Instruct-v0.3, 32 couches, embd 4096).

| Test | Prouve | Résultat |
|---|---|---|
| `shard_backend_loads_layer_subset` | un nœud charge seulement `[0,1)` des 32 couches | **ok** |
| `shard_backend_partial_equals_full` | split 2-voies `head[0,16) → tail[16,32)` == entier | **ok** (cosine > 0.999) |
| `shard_backend_three_way_equals_full` | pipeline 3-voies `head→milieu→tail` == entier | **ok** (cosine > 0.999) |
| `shard_backend_hidden_state_extractable` | état-frontière extractible + top-k lossless (N0) | **ok** |

**4 passed / 0 failed — 26.70 s.**

---

## 4. Coordination multi-nœud — live (Windows, 2 daemons, iroh loopback)

`cargo nextest run -p nexus-test-harness` — vrais binaires daemon, découverte iroh.

| Test | Prouve | Résultat |
|---|---|---|
| `test_two_daemons_boot_and_respond` | 2 nœuds bootent + répondent | **PASS** |
| `test_cross_daemon_discovery` | découverte nœud↔nœud | **PASS** |
| `test_cross_daemon_gossip_exchange` | gossip propage | **PASS** |
| `test_cross_daemon_feed_sync` | feed (log protocole) se synchronise | **PASS** |
| `test_cross_daemon_storage_sync` | storage se synchronise | **PASS** |
| `test_cross_daemon_blob_transfer` | blob réplique nœud→nœud | **PASS** |
| `cross_daemon_publish_and_serve_blob` | publié sur A, servi depuis B | **PASS** |
| `test_cross_daemon_task_stub` | tâche dispatchée cross-nœud | **PASS** |
| `test_feed_offline_catchup` | nœud offline rattrape | **PASS** |
| `test_feed_replay_idempotent` | replay idempotent | **PASS** |
| `blob_serve_coep_headers_on_real_zip` | entêtes blob-serve | **PASS** |
| `daemon_binary_path_is_constructed` | — | **PASS** |

**12 passed / 0 skipped — 1.712 s.**

---

## 5. Pipeline du handoff cross-machine (§0)

```
Windows                                              Mac (192.168.1.53)
─────────                                            ──────────────────
shard_node head mistral.gguf 0 16 "The quick..."
  → forward couches [0,16) sur x86/CPU
  → boundary.bin (98 304 o, 6×4096 f32)
        │
        └── SSH (pipe binaire, LAN) ───────────────► shard_node tail mistral.gguf 16 32
                                                       → forward_hidden(boundary)
                                                       → couches [16,32) sur Metal
                                                       → split.bin (98 304 o)
        ◄───────────────────────────────────────────┘
shard_node whole mistral.gguf "The quick..."  → full.bin (référence, même machine)
cosine(full.bin, split.bin) = 0.999727  ✅
```

Driver : `crates/nexus-worker-core/examples/shard_node.rs` (modes `whole|head|tail`,
état-frontière en f32 little-endian sur stdin/stdout). Bâti `--features
llm_llama_cpp` (Windows) / `llm_llama_cpp_metal` (Mac).

---

## 6. Setup installé sur le Mac (parcours de déblocage)

| Étape | Détail | Résultat |
|---|---|---|
| Transfert GGUF | `mistral-7b-q4.gguf` 4.6 Go, Windows → Mac (LAN) | OK (4.1 Go arrivé) |
| cmake | binaire officiel `3.30.5-macos-universal` (sans brew, non-interactif) | OK |
| Repo sync | tar Windows → Mac (fork vendoré `vendor/llama-cpp-sys-2` + `[patch]` Cargo) | OK (VENDOR_FORK_PRESENT) |
| ccache | shim passthrough `~/binshim/ccache` (`#!/bin/sh\nexec "$@"`) — l'image llama.cpp force `ccache` | OK |
| Build Metal | `cargo build --release --example shard_node --features llm_llama_cpp_metal` | OK (exit 0) |

Blocages rencontrés puis résolus : (1) `cmake` absent → installé ; (2) repo Mac
pré-fork (utilisait `llama-cpp-sys-2 v0.1.145` crates.io) → synchronisé sur le
fork ; (3) `ccache` absent (Error 127) → shim.

---

## 7. Périmètre exact — ce qui est prouvé vs ce qui reste (honnêteté)

**Prouvé sur le rig réel :**
- Le **découpage en blocs de couches est correct** : split 2-voies et 3-voies ==
  modèle entier (in-process, Mistral-7B).
- Le **split tourne cross-machine** : head Windows/x86/CPU → tail Mac/M2/Metal,
  état-frontière sur le LAN, cosine 0.999727 == entier.
- La **coordination multi-nœud** : 12/12 (découverte, gossip, sync, transfert, dispatch).
- Le **code = le fork du sprint** (`ShardBackend` partial-load + `forward_hidden`).

**Reste (intégration SBFB = carry S78, pas de la science) :**
- Handoff via **SSH**, pas encore le data-plane iroh `sbfb/shard/1` (lui, prouvé
  séparément par `multi_daemon`).
- **Un forward**, pas encore la boucle de génération autorégressive token-par-token.
- Pas encore d'orchestrateur de session live ni de `RunProof` in-vivo.

→ La **mécanique multi-ordinateur plus-gros-modèle est prouvée de bout en bout** ;
il reste à câbler le transport iroh + la boucle live (S78).

---

## 8. Suite dual-platform (baseline Sprint 77 Phase K, pour référence)

Win nextest **1949/1949** 0-skip · Docker rust:1.94 fmt/clippy/doctest 0 + nextest
**1947/1953** (6 iroh-networked env-bloqués Docker-on-Windows, verts Win+CI Linux)
· Vitest **411** · E2E **41+1skip** · coverage 87.27/79.01/86.02/88.59.

---

## 9. Modèle PLUS GROS que la VRAM du 5080 — CodeLlama-34B éclaté 5080+Mac (2026-06-23, 2e session)

Objectif de la 2e session : prouver qu'un modèle **trop gros pour les 16 Go de
VRAM** tourne éclaté sur les deux machines via le **GPU réel des deux côtés**
(5080 CUDA + Mac M2 Metal), pas seulement CPU.

### 9.1 Pré-requis débloqués cette session

| Bloqueur | Cause racine | Fix |
|---|---|---|
| Build CUDA Windows échoue (`No CUDA toolset found`) | `cmake-rs` force le générateur **VS 18 2026** (cl 19.50) ; CUDA 12.8 ne supporte que `_MSC_VER ∈ [1910,1950)` et l'intégration MSBuild CUDA n'existe pas pour VS 18 | **Builder sous `vcvars64.bat` de VS 2022** (cl 19.44, supporté) → `cc` lit `VisualStudioVersion=17.0` → générateur « Visual Studio 17 2022 » (intégration CUDA 12.8 présente) ; `CMAKE_CUDA_ARCHITECTURES=120` (sm_120 Blackwell). Binaire CUDA OK (lie cudart/cublas), sm_120 confirmé. |
| gemma-4-26B (le « 20 Go ») refusé | `ShardBackend::load` rejette `arch != "llama"` ; le modèle est **`gemma4`** (MoE) — le fork n'a patché le layer-split que pour le graphe LLAMA | Modèle **archi `llama`** : **CodeLlama-34B-Instruct Q3_K_M** (16,28 Go, 48 couches, n_embd 8192, GQA 64/8), téléchargé en parallèle Windows+Mac |
| Driver CPU-only | `shard_node.rs` câblait `n_gpu_layers=0` | Ajout lecture env **`N_GPU_LAYERS`** (0=CPU défaut, 999=offload total), câblée Windows + Mac |

### 9.2 Preuve « trop gros pour le 5080 » (OOM mesuré)

`N_GPU_LAYERS=999 shard_node whole codellama-34b.gguf "..."` sur le 5080 :

| Mesure | Valeur |
|---|---|
| VRAM libre 5080 avant | **13 320 MiB** |
| Buffer modèle requis sur CUDA0 | **15 421 MiB** (94 % d'une carte 16 Go) |
| Résultat | **`CUDA error: out of memory`** (`ggml_backend_cuda_buffer_set_tensor`), exit 127, 0 octet |

→ Le modèle **ne tient pas** sur le seul RTX 5080. Le split est nécessaire.

### 9.3 Split GPU cross-machine exécuté

```
Windows / RTX 5080 / CUDA sm_120                     Mac M2 / Metal
shard_node head codellama 0 K "The quick brown fox"
  → couches [0,K) sur GPU 5080 (buffer CUDA0 mesuré)
  → boundary 196 608 o (6 tok × 8192 f32)
        └── SSH (pipe binaire) ─────────────────────► shard_node tail codellama K 48
                                                        → couches [K,48) sur Metal
                                                        → split.bin (196 608 o)
shard_node whole codellama "..." (CPU, 16,3 Go en 31 Go RAM) → full.bin (référence, 32 s)
```

- head `[0,36)` : buffer CUDA0 **11 423 MiB** (tient). head `[0,44)` : **13 951 MiB** (tient).
- tail Metal : noyaux `kernel_swiglu_f32`, `kernel_rms_norm_f32`, `kernel_mul_mv_ext_q6_K` exécutés (Metal actif).

### 9.4 Correction — décomposition complète (toutes mesurées, vs whole-CPU)

| Configuration | cosine | Ce que ça isole |
|---|---|---|
| **split tout-CPU vs whole-CPU** | **1.000000** | **La LOGIQUE de split est EXACTE** (mêmes backends → reproduction bit-proche) |
| head CUDA vs head CPU (frontière, 36 couches) | 0.999968 | Le **head 5080-CUDA est fidèle** |
| head CUDA + tail CPU | 0.986191 | Contribution de la dérive **CUDA** (amplifiée par le tail) |
| head CPU + tail Metal | 0.991046 | Contribution de la dérive **Metal** |
| **head CUDA + tail Metal (K=36)** | **0.978497** | Pipeline GPU réel des deux côtés |
| head CUDA + tail Metal (K=44, 4 couches Metal) | 0.978353 | Idem — **indépendant du nb de couches Metal** |

**Interprétation (honnête).** La logique de découpage est **exacte** (1.000000 quand
les backends coïncident). L'écart du pipeline GPU hétérogène (0.978) est de la
**divergence numérique inter-backend** : les dérives CUDA (0.986) et Metal (0.991)
se composent (0.986 × 0.991 ≈ 0.977 ≈ 0.978 mesuré). Elle est **indépendante du
point de split** (0.978 à K=36 comme à K=44) donc ce n'est PAS de l'accumulation
par couche : une différence de frontière minime (0.999968) est **amplifiée** par les
**activations-outliers** du 34B (dimensions de très forte magnitude du résidu, propres
aux LLM), aggravée par le quant bas **Q3_K**. Ce n'est **pas un bug du sharding SBFB**.

### 9.5 Verdict 2e session

- ✅ **Un modèle qui OOM le RTX 5080 (16 Go) tourne éclaté 5080-CUDA + Mac-Metal.**
- ✅ Mécanique de split **prouvée exacte** (1.000000 same-backend) ; head 5080 fidèle (0.999968).
- ⚠️ Cosine bout-en-bout GPU hétérogène = **0.978** = numérique CUDA+Metal+Q3_K, caractérisé et attribué (pas une erreur de découpage).
- Le « 20 Go » visé (gemma 26B) est **incompatible** (archi `gemma4` non patchée) ; supporter gemma4 = patcher le graphe gemma du fork (feature S78, pas ce test).
- Note Q4 : un quant plus haut (Q4_K_S ≈ 19 Go) réduirait la dérive par-backend
  (source quantique) — mais tient tout juste dans le budget combiné ~20 Go (5080 libre
  ~14,8 + Mac ~5,5) ; à retenter si on veut un cosine cross-machine plus serré.

Driver : même `examples/shard_node.rs` (modes `whole|head|tail`) + env `N_GPU_LAYERS`.
**Untracked** — candidat à folder dans S78 (orchestrateur live), pas committé ici.
