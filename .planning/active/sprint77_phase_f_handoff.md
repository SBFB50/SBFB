# Sprint 77 Phase F — Handoff / context-pack (reprise nouveau contexte)

> Context-pack repo-visible pour reprendre la Phase F (fork llama.cpp) dans une session
> fraîche. À coller dans un nouveau contexte Claude Code (le bootstrap SBFB + `CLAUDE.md`
> se chargent tout seuls ; ce pack donne l'état précis + la tâche pour ne rien re-dériver).
> Généré 2026-06-21 (tip `e3ccd10`). Voir aussi `sprint77_phase_f_preflight.md`,
> `sprint77_phase_f_spike.md`, `sprint77_phase_f_backend_research.md`, plan §9/§14, kickoff D3.

---

```
Tu reprends le projet SBFB / nexus-grid (P2P compute + sharding LLM) sur le Sprint 77,
Phase F. Exécute d'abord le pre-flight bootstrap canonique (docs/claude/README.md §0 + §7.1),
puis lis ce contexte AVANT d'agir. Mode ULTRACODE (Workflow multi-agents pour découverte/
vérification ; écriture de phase main-thread séquentielle, 1 commit atomique).

═══════════════════════════════════════════════════════════════════════════════
ÉTAT FACTUEL (vérifie d'abord : git rev-parse --short HEAD doit == e3ccd10)
═══════════════════════════════════════════════════════════════════════════════
- HEAD = e3ccd10  chore(planning): Sprint 77 Phase F re-cadrage — fork (a) + spike GO + amendement D3.
- S77 phases A-E committées. Phase F = PROCHAINE, RE-CADRÉE fork-based (cf. ci-dessous). Cas B.
- Memory : lis MEMORY.md + sprint77_phase_f_fork_spike.md + nexus_grid_pivot.md (tip e3ccd10).

═══════════════════════════════════════════════════════════════════════════════
DÉCISIONS PO GRAVÉES (NE PAS re-débattre — déjà tranché et prouvé)
═══════════════════════════════════════════════════════════════════════════════
1. Préflight Phase F = DESIGN-CONFLICT : le forward partiel d'un range de couches avec
   injection du hidden state amont est INFAISABLE via le wrapper safe `llama-cpp-2` 0.1.146
   (aucun eval-callback / layer-range / injection). → Arbitrage PO : option (a) FORK llama.cpp.
2. Recherche multi-source : aucun runtime tiers plus simple (Ollama, exo, Petals, candle…).
3. SPIKE de faisabilité JETABLE = GO. Vit HORS repo dans C:/Users/FlowUP/spike_fork/
   (source llama.cpp vendorée patchée + harnesses spike.cpp/shard_node.cpp + builds CUDA/Metal).
   Patch minimal backend-agnostique prouvé BIT-EXACT sur CPU, CUDA sm_120 (RTX 5080),
   Metal (Mac M2) ; cross-backend CUDA→Metal cosine 0.99923 sur Mistral-7B-Q4 réel (32 couches).
4. AMENDEMENT PO 2026-06-21 (D3) : le 70B est ABANDONNÉ. Cible = un modèle ~20 Go arch-llama
   (ex. Mixtral-8x7B Q3 MoE ou 34B Q4 dense) éclaté sur le rig RÉEL : RTX 5080 (16 Go CUDA)
   + Mac M2 (8 Go Metal). Chaque machine ne charge QUE ses couches → le CHARGEMENT PARTIEL
   des couches (P-D, TENSOR_SKIP) est désormais OBLIGATOIRE en Phase F (un 20 Go ne tient
   sur aucune machine seule). Mécanisme inchangé (pipeline/layer-split, scheduler Phase D/E
   déjà codé, water-filling sur VRAM libre).

═══════════════════════════════════════════════════════════════════════════════
LE PATCH PROUVÉ (à porter du spike vers une vraie API stable + vendoré)
═══════════════════════════════════════════════════════════════════════════════
Sur la source llama.cpp vendorée par llama-cpp-sys-2 0.1.146 (structure moderne src/models/) :
- 4 champs `shard_layer_start/end/is_first/is_last` ajoutés à `llama_cparams`
  (src/llama-cparams.h) + `llama_context_params` (include/llama.h) + copie params→cparams
  (src/llama-context.cpp). CRUCIAL : passer la config via context_params AVANT
  llama_init_from_model (le reserve des buffers fige la config ; un setter post-création échoue).
- src/models/llama.cpp (builder arch LLAMA, couvre Llama/Mistral/Mixtral) :
  • boucle bornée `for (il = shard_start; il < shard_end; ++il)` ;
  • gather `inp_out_ids` au dernier layer EXÉCUTÉ : `if (il == shard_end-1 && inp_out_ids)`
    (PAS `il == n_layer-1` — sinon inp_out_ids reste inutilisé/non-alloué et set_inputs
    assert sur buffer null ; build_inp_out_ids retourne TOUJOURS un tenseur) ;
  • sortie : `if (shard_last) { output_norm; t_embd; if(!embed) lm_head/t_logits }
    else { t_embd = résiduel BRUT }`.
- Injection = ZÉRO patch : champ public `llama_batch.embd` déjà câblé dans build_inp_embd.
- Validation : embeddings=true + pooling NONE → llama_get_embeddings expose t_embd par token.
- À AJOUTER pour la Phase F (pas dans le spike) : P-D chargement partiel (TENSOR_SKIP dans
  src/llama-model.cpp create_tensor : ne charger QUE les couches [start,end)) + patch
  multi-arch si la cible n'est pas llama-arch.
Le code exact est récupérable depuis C:/Users/FlowUP/spike_fork/llama/ (diff vs registry).

═══════════════════════════════════════════════════════════════════════════════
TOOLCHAIN (validée live au spike)
═══════════════════════════════════════════════════════════════════════════════
- Windows : nvcc 12.8, RTX 5080 compute_cap 12.0 → CMAKE_CUDA_ARCHITECTURES=120a-real,
  GÉNÉRATEUR "Visual Studio 17 2022" (PAS VS18 : sans intégration CUDA → "No CUDA toolset"),
  cmake 4.3.1. BLOQUEUR F1 : libclang ABSENT → installer LLVM (LIBCLANG_PATH) pour bindgen
  (requis par le build du crate Rust llama-cpp-sys-2 ; le spike standalone C++ l'évitait).
- Mac : `ssh mac` (192.168.1.53, user theophilevasseur), M2 8 Go arm64, clang 17 +
  CommandLineTools (PAS de compilateur `metal` CLI → GGML_METAL_EMBED_LIBRARY=ON),
  cmake/ninja via /opt/homebrew/bin, GGML_BLAS=OFF (ggml-blas absent de la source vendorée).
- Mistral-7B-Q4 (~4.1 Go, arch llama, L=32) déjà sur les 2 machines (spike).

═══════════════════════════════════════════════════════════════════════════════
ARTEFACTS À LIRE (.planning/active/)
═══════════════════════════════════════════════════════════════════════════════
- sprint77_phase_f_preflight.md  (§Résolution + verdict factuel + points de code)
- sprint77_phase_f_spike.md      (patch détaillé + commandes build + résultats + re-cadrage F1/F2)
- sprint77_phase_f_backend_research.md  (pourquoi fork vs Ollama/candle)
- sprint77_plan.md §9 (Phase F re-cadrée) + §14 (Phase K ~20 Go) + sprint77_kickoff.md D3 (amendé)

═══════════════════════════════════════════════════════════════════════════════
TÂCHE : implémenter la Phase F (fork)
═══════════════════════════════════════════════════════════════════════════════
1. Préflight Phase F sur le scope FORK (Workflow ultracode : OSS vendoring/fork strategy +
   threat model claim/VRAM + wire 0-bump + décisions historiques). La Phase F est LARGE
   (fork + P-D + claim + câblage sbfb/shard/1) → le préflight décide d'un split éventuel en
   sous-phases atomiques (renumérotation G→K traitée alors). Verdict EXECUTE/PLAN-ADAPT attendu
   (le DESIGN-CONFLICT est déjà résolu).
2. Installer LLVM/libclang (LIBCLANG_PATH) — prérequis dur du build crate Rust.
3. Vendorer un fork de llama-cpp-sys-2 (submodule llama.cpp patché OU source override Cargo)
   portant le patch ci-dessus + P-D (TENSOR_SKIP). Build vert `cargo build -p nexus-worker
   --features llm_llama_cpp` sur CUDA ET Metal.
4. Backend Rust : nexus-worker-core/src/llm/llama_cpp.rs (charge layer_start..end via le fork,
   forward partiel, hidden state émis aval) + worker shard claim engine/runtime.rs
   (filtre ComputeGroup + cap VRAM fail-closed sur GpuStats.vram_free_bytes mesuré — PAS de
   nouvelle pompe live, scope cut #7 ; + vérif signature ShardedSessionManifest côté dialer)
   + câblage sbfb/shard/1 (data plane Phase B existant).
5. Tests : hermétiques CI (primitive layer-range/format) + #[ignore] GGUF (P-D charge-subset,
   partial==full bit-exact, hidden extract). Cross-backend cosine > 0.99 (spike-validé).
6. Review Workflow → Codex GPT5.5 → commit feat(worker): Sprint 77 Phase F — …

RÈGLES : ultracode Workflow (preflight/review) ; Codex gate bloquant ; 1 commit/phase ;
hook lightcheck (PAS d'estimation LoC dans les plans !) ; français docs/commit-body, anglais
code ; agents en claude-opus-4-8[1m] (jamais passer model=). Le spike reste hors repo (jetable ;
~9 Go de modèles/builds nettoyables sur les 2 machines).
```
