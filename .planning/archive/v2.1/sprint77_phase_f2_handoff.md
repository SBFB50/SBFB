# Sprint 77 Phase F2 — Handoff / context-pack (reprise nouveau contexte)

> Context-pack repo-visible pour reprendre la Phase F2 (worker shard claim + cablage
> sbfb/shard/1 + threat §16) dans une session fraiche. A coller dans un nouveau contexte
> Claude Code (le bootstrap SBFB + `CLAUDE.md` se chargent tout seuls ; ce pack donne
> l'etat precis + la tache pour ne rien re-deriver). Genere apres F1 (tip `14fa313`).
> Voir aussi `sprint77_phase_f1_preflight.md`, `sprint77_phase_f1_review.md`,
> `sprint77_phase_f1_codex_review.md`, plan §9 (F2) + §5 (Phase B data plane).

---

```
Tu reprends le projet SBFB / nexus-grid (P2P compute + sharding LLM) sur le Sprint 77,
Phase F2. Execute d'abord le pre-flight bootstrap canonique (docs/claude/README.md §0 + §7.1),
puis lis ce contexte AVANT d'agir. Mode ULTRACODE (Workflow multi-agents pour decouverte/
verification ; ecriture de phase main-thread sequentielle, 1 commit atomique).

═══════════════════════════════════════════════════════════════════════════════
ETAT FACTUEL (verifie : git rev-parse --short HEAD doit == 14fa313)
═══════════════════════════════════════════════════════════════════════════════
- HEAD = 14fa313  feat(worker): Sprint 77 Phase F1 — forked layer-block backend (partial load
  + build CUDA/Metal). S77 phases A-E + F1 committees. Phase F2 = PROCHAINE. Cas B.
- La Phase F a ete re-coupee F1/F2 (preflight PLAN-ADAPT, suffixe chiffre, 0 renumerotation G-K).
- Memory : lis MEMORY.md + nexus_grid_pivot.md (tip 14fa313) + sprint77_phase_f_fork_spike.md.

═══════════════════════════════════════════════════════════════════════════════
CE QUE F1 A LIVRE (acquis — ne pas refaire)
═══════════════════════════════════════════════════════════════════════════════
Fork llama.cpp VENDORE (vendor/llama-cpp-sys-2 + vendor/llama-cpp-2, [patch.crates-io] path
0.1.146) : API partial-decode (4 champs shard_* sur cparams/context_params/model_params, builder
LLAMA borne, gather inp_out_ids dernier-layer-execute, sortie is_last?norm+lm_head:residuel-brut)
+ P-D TENSOR_SKIP (tok_embd RESIDENT chaque shard, output skip hors-last). Backend Rust
nexus-worker-core/src/llm/shard.rs : ShardWindow + top_k_by_magnitude + hidden_token_count (purs,
testes CI) + ShardBackend gated (load layer-subset / forward_tokens / forward_hidden(inject embd) /
collect_boundary ; fail-close general.architecture=="llama"). LlamaBatch new_embeddings/
add_embedding sound (embd_n+n_seq_max, guards). Builds CPU+CUDA-sm120-RTX5080+Metal-M2 VERTS.
Tests : 6 hermetiques CI (nextest 1889) + 4 #[ignore] GGUF Mistral-7B-Q4 4/4 (partial==full 2-shard
ET three_way_equals_full 3-shard MIDDLE cosine>0.999). 0-bump-wire. Review Workflow + Codex
GPT5.5 R2 CONFIRME 0 P0/P1.

═══════════════════════════════════════════════════════════════════════════════
TACHE F2 (plan §9 livrables F2 + gardes du preflight F1 + P2 trackes)
═══════════════════════════════════════════════════════════════════════════════
Commit cible : feat(worker): Sprint 77 Phase F2 — shard claim + sbfb/shard/1 forward wiring + threat §16

1. WORKER SHARD CLAIM (crates/nexus-worker-core/src/engine/runtime.rs, nouveau chemin) :
   - filtre ComputeGroup::is_member (l'admission server-side au handshake est DEJA cablee,
     crates/nexus-core-rs/src/shard.rs:222-235 ; F2 = le claim/dialer cote worker).
   - CAP VRAM FAIL-CLOSED (garde P1 S3-F-2) : comparer la VRAM requise par [layer_start,layer_end)
     a GpuStats.vram_free_bytes MESURE (snapshot PONCTUEL, modele gpu_snapshot runtime.rs:641 —
     PAS de nouvelle pompe live continue, scope cut #7). Estimer la VRAM du range via metadata GGUF
     (somme tailles tenseurs blk.{i} du range, lecture header-only, + marge KV-cache). Sur depassement :
     defer/refuse, NE PAS crasher.
   - VERIF SIGNATURE ShardedSessionManifest cote dialer (garde P1 S3-F-1) : appeler
     verify_signature() (crates/nexus-core-rs/src/shard_plan.rs:355, DOMAIN_SHARD_PLAN_V1, initiateur)
     AVANT d'entrer dans la chaine. La primitive existe mais a 0 appelant prod aujourd'hui.

2. CABLAGE sbfb/shard/1 : remplacer le corps ECHO de ShardProtocol::accept
   (crates/nexus-core-rs/src/shard.rs:233, commentaire "Phase F replaces the echo body") par
   recv hidden amont -> forward partiel via ShardBackend (F1) -> send aval. Data plane Phase B existant
   (write_frame/read_frame length-prefixed, cap MAX_SHARD_FRAME_BYTES=64MiB).

3. CAP 64 MiB (S3-F-4) a TRANCHER : pour la cible ~20Go arch-llama, hidden frontiere =
   n_embd × n_tokens × 4 (FP32, pas fp16). n_embd=8192 -> 2048 tok = 64MiB pile, 4096 tok = 128MiB
   DEPASSE. Options : (1) relever MAX_SHARD_FRAME_BYTES a 128/256 MiB (justif DoS, borne par n_ctx max
   du placement) ; (2) chunker le frame ; (3) borner n_ctx au placement. Recommande : (1) + borne n_ctx
   documentee. Ne pas laisser inerte.

4. THREAT_MODEL §16 : creer la section surface shard (SI-1 reconstruction activations High, SI-4
   collusion inter-workers High, caveat "aucun secret app dans les prompts" — activations en clair,
   pas de TEE GPU consumer 2026, scope cut #4). Source docs/security/SPLIT_INFERENCE_DESIGN.md:194-202.
   NOTE : §16 actuel = "Revue et evolution", §15 = seed cross-noeud. Le plan §14 met la section
   longue-vie COMPLETE en Phase K ; F2 amorce au minimum le caveat co-localise avec le code
   activations-en-clair qu'il introduit.

5. P2 TRACKES F1 a absorber ou router (cf. sprint77_phase_f1_review.md) :
   - validation-ordering : ShardBackend::load valide la fenetre en Rust APRES le load natif ; un
     window hors-bornes trip un GGML_ASSERT qui ABORT le process AVANT (documente # Aborts). Ideal F2 :
     probe metadata (vocab_only/no_alloc) pour n_layer + valider AVANT le vrai load, OU s'appuyer sur
     le claim F2 qui pre-valide la fenetre contre le modele.
   - D3-1 : gardes-role backend (forward_tokens sur non-first, etc.) seulement testees en #[ignore] —
     extraire en fns pures testables CI ou documenter.
   - D3-2 : preuve VRAM observable (loads_layer_subset ne prouve que le load reussit).
   - Carry T-NN+3 (JCS dup) reconduit P2 si canonical.rs non touche.

TESTS F2 : shard_assignment_claim_respects_group (hermetique CI : rejet hors ComputeGroup + rejet
VRAM-requise>vram_free snapshot + acceptation sinon). Plus E2E cross-machine si le rig est dispo.
Gate testabilite par-sprint : T1 E2E hermetique + T2 acceptance JSON restent au wrap-up Phase K.

═══════════════════════════════════════════════════════════════════════════════
WIRE / SCOPE
═══════════════════════════════════════════════════════════════════════════════
F2 est 0-BUMP-WIRE (consomme ShardAssignment, produit hidden state opaque ; READ d'une signature
existante). ATTENTION : contrairement a F1, F2 TOUCHE legitimement runtime.rs (claim) + shard.rs
nexus-core-rs (remplace echo) — ce ne sont plus des fichiers "intacts". Le claim VRAM reutilise les
patterns consent.rs/runtime.rs SANS armer de pompe (scope cut #7).

═══════════════════════════════════════════════════════════════════════════════
TOOLCHAIN / BUILD (deja en place depuis F1)
═══════════════════════════════════════════════════════════════════════════════
- LLVM/libclang installe : export LIBCLANG_PATH="C:/Program Files/LLVM/bin" (requis bindgen).
- CUDA : export CMAKE_GENERATOR="Visual Studio 17 2022" ; cargo build -p nexus-worker-core
  --features llm_llama_cpp_cuda (incremental, native cachee si C++ inchange).
- Metal : ssh mac (Mac M2, repo a ~/nexus-sbfb). Re-sync changements via
  tar czf - <fichiers> | ssh mac 'tar xzf - -C ~/nexus-sbfb && cd ~/nexus-sbfb &&
  export PATH="/opt/homebrew/bin:$PATH" && ~/.cargo/bin/cargo build -p nexus-worker-core
  --features llm_llama_cpp_metal'. NE PAS rm -rf ~/nexus-sbfb (preserve le target pour l'incremental).
- Tests GGUF : export SBFB_SHARD_TEST_GGUF="C:/Users/FlowUP/spike_fork/mistral-7b-q4.gguf" ;
  cargo nextest run -p nexus-worker-core --features llm_llama_cpp --run-ignored ignored-only.
- LIGHTCHECK VENDORED-FIXES DEJA COMMITTES (F1) : .gitattributes (vendor/** + *.patch +
  *codex_review.md -whitespace) + scripts/agent/agentctl.py (pub_mod_errors skip vendor/). Le VRAI
  hook pre-commit = scripts/agent/agentctl.py via core.hooksPath=.githooks (PAS .claude/hooks).
- Docker canonique dual-platform : docker run --rm -v "C:/Users/FlowUP/Documents/Code/nexus:/work"
  -w /work -e CARGO_TARGET_DIR=/tmp/td sbfb-ci cargo test --workspace --locked (image sans nextest ;
  le feature n'est jamais builde en CI).

ARTEFACTS A LIRE (.planning/active/) : sprint77_phase_f1_preflight.md (gardes S3-F-1/F-2/F-5 +
F3 cap 64MiB + estimation VRAM GGUF-header), sprint77_phase_f1_review.md (P2 trackes D1-2/D3-1/D3-2),
sprint77_plan.md §9 (livrables F2) + §5 Phase B (data plane sbfb/shard/1 + ComputeGroup existant).

REGLES : ultracode Workflow (preflight + review) ; Codex GPT5.5 gate bloquant ; 1 commit/phase ;
francais docs/commit-body, anglais code ; agents en claude-opus-4-8[1m] (jamais passer model=).
Le spike reste hors repo (jetable). ultracode
```
