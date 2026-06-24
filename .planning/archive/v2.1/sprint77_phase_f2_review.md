# Sprint 77 Phase F2 — Review

## Verdict: PASS

> Review en fan-out avant-plan (3 agents adversariaux parallèles, arbre sale mi-phase → foreground §4.5)
> sur le diff complet F2 (452 insertions, 8 fichiers modifiés + 2 untracked). Dimensions : (A) correctness
> du seam + ordre crypto-avant-I/O du claim ; (B) estimation VRAM fail-closed + accessors GGUF vendorés ;
> (C) scope/wire/Day-0/threat/tests/patterns. **Les 3 agents rendent PASS.** 0 P0, 0 P1. 1 LOW corrigé
> en-phase + INFO/P3 non-bloquants documentés. PASS-PENDING = review OK, Codex pas encore exécuté.

## Dimensions (verdicts)

| Dimension | Verdict | Findings |
|---|---|---|
| A — seam + claim ordering | PASS | 2 P3 (seam-only = scope K intentionnel ; head_dim ignore key_length/value_length — fail-closed-safe) |
| B — GGUF sizing fail-closed + accessors vendorés | PASS | 1 LOW **CORRIGÉ** + 2 INFO (tensor_type non consommé ; tied-embed under-count physiquement correct) |
| C — scope/wire/threat/tests/patterns | PASS | 2 INFO (déclarer delta tests body ; nuance ordre window-check) |

## Finding LOW corrigé en-phase

**B-F1 (LOW) — accessors tenseurs vendorés abort sur `tensor_id` hors-borne.**
`tensor_name`/`tensor_size`/`tensor_type` (vendor gguf/mod.rs) forwardent `tensor_id` au FFI sans borne ;
ggml `GGML_ASSERT(0 <= id < n_tensors)` et **abort le process** sur id hors-borne. **NON exploitable en
F2** : l'unique appelant `read_gguf_model_facts` itère `0..n_tensors()` (jamais hors-borne). **Corrigé** :
note `# Aborts` ajoutée aux 3 accessors (convention F1 `ShardBackend::load # Aborts` — safe-by-doc, le
contrat « toujours itérer 0..n_tensors() » est explicite). Doc-only, n'affecte pas la compilation feature.

## Findings INFO / P3 (non-bloquants, documentés)

- **A-P3 seam-only** : `claim_shard_assignment` / `ShardBackendForwarder` / `shard_protocol_factory(group,
  forwarder)` n'ont pas d'appelant live hors tests. **Intentionnel** : le drive multi-hop live + l'acceptance
  sont Phase K (preflight §scope, plan §14). F2 livre le seam + le claim + la vérif sig dialer, pas le drive.
- **A-P3 head_dim** : `n_embd/n_head` ignore `attention.key_length`/`value_length` explicites. N'alimente que
  l'estimation KV (heuristique fail-closed dominée par le headroom 768 MiB) — pas load-bearing.
- **B-INFO tensor_type non consommé** : ajouté pour compléter la triade name/size/type du `GgufContext` (API
  cohérente, utile Phase G/H vérif) ; pub sur type lib = 0 warning. Conservé.
- **B-INFO tied-embedding** : last shard avec `output.weight` tié à `tok_embd` → under-count = le poids
  partagé n'est alloué qu'une fois (physiquement correct) ; headroom couvre. Pas fail-open.
- **C-INFO delta tests** : §15 budgétait Rust +1 ; réel = **+10 hermétiques CI** (8 shard_claim + 2 seam
  core-rs) **+2 `#[ignore]` GGUF** (non comptés CI). Sur-livraison justifiée (preflight a étendu : crypto-
  avant-I/O, window-out-of-range, residents-par-is_last, KV, seam, fail-closed). **Déclaré dans le commit body.**
- **C-INFO ordre window-check** : la fenêtre est validée DANS `assess_capacity` (après snapshot GPU mais
  AVANT tout load natif = son objet réel) ; crypto précède strictement toute I/O. Nuance doc cosmétique.

## Confirmations clés (evidence vérifiée par les 3 agents)

- **Seam correct, 0 cycle de crate** : `nexus-core-rs/Cargo.toml` n'a aucun dep worker-core ; trait dans
  core-rs, impl `ShardBackendForwarder` dans worker-core (edge worker→core uniquement). Accept loop
  `read_frame → forwarder.forward().map_err(AcceptError)? → write_frame` ; erreur forwarder = abort propre
  (pas de panic), miroir `seed_protocol.rs`. `EchoForwarder` préserve les 5 tests Phase B (fixture migrée) ;
  `DoublingForwarder`/`FailingForwarder` prouvent forward-injecté ≠ echo (faux-positif écarté).
- **Crypto-avant-I/O** : `authorize_claim` = verify_signature() PUIS is_member PUIS in-plan (pur, 0 I/O) ;
  `runtime.rs::claim_shard_assignment` = authorize → read_gguf → gpu_snapshot → assess. Manifeste forgé/
  non-membre n'atteint jamais le disque/GPU (DoS pré-auth fermé). `authorize_rejects_unsigned_before_anything`
  le prouve.
- **Fail-closed VRAM** : residents corrects (blk.{i} ∈ window, disambig blk.1≠blk.10 via trailing dot ;
  tok_embd toujours ; output_norm/output si is_last ; hors-fenêtre exclu) ; toutes ops `saturating_*` (pas de
  wrap) ; header illisible / cle manquante → `ModelUnreadable` (REJET, jamais « estimer 0 ») ; headroom 768
  MiB + KV ajoutés (over-estimate sur la borne inférieure `gguf_get_tensor_size`). `meta_u32` type-check
  AVANT `val_*` (pas d'abort sur type/array erroné) ; casts `try_from` overflow-safe.
- **Pré-validation fenêtre** ferme le P2 F1 (ShardWindow::new avant tout load natif → pas de GGML_ASSERT
  abort). `assess_rejects_window_past_model` le prouve.
- **0-bump-wire** (check #38) : 0 nouveau DOMAIN_*, 0 bump FORMAT_VERSION, trait/EchoForwarder/accessors/
  evaluate internes (0 serde wire), frame opaque, F2 READ une signature existante.
- **Scope cut #7** : snapshot ponctuel `gpu_snapshot`, pas de pompe live.
- **Named constants** : MAX_SHARD_FRAME_BYTES, MAX_SHARD_N_CTX, VRAM_BACKEND_OVERHEAD_BYTES,
  KV_CACHE_DTYPE_BYTES, GGUF_TENSOR_*/GGUF_BLOCK_PREFIX — tous nommés/doc/réutilisés.
- **Cap frame** : 64→256 MiB, doc fp16→fp32, 8192×8192×4=256 MiB exact (cohérent MAX_SHARD_N_CTX=8192),
  rejet avant alloc (header_to_frame_len).
- **THREAT §16** (check #41) : SI-1..SI-5 sévérités exactes (source §3.1), caveat cardinal, incentive R8 non-
  monétaire, §16→§17, changelog v10, note doc-honnêteté §4.2-superseded juste, **0 overstate** (allowlist =
  admission, PAS confidentialité, explicite).
- **Patch tracé** : note F2 (accessors + meta_str/meta_u32) en en-tête.
- **Re-coupe F1/F2** : suffixe chiffré, 0 renumérotation G-K, 0 creep Phase J/K (web/http/spec non touchés).

## Gates (rappel verification)

Windows non-feature : fmt ✓ · clippy ✓ (`-D warnings`, all-targets) · nextest 1899/1899 (+10 vs F1 1889) ·
doctests ✓ · release ✓. Docker canonique Linux : core-rs 326 + worker-core 385 + suites ✓ (seul
`sbfb-factory operator_server` KO = limitation env S72 Docker-on-Windows bind-mount, orthogonale F2).
CUDA build sm_120 ✓ · Metal M2 ✓ · feature tests + GGUF rig **233/233** (F1 GGUF 4/4 → 0 régression + 2
GGUF F2) · claim ciblé 10/10.

## Codex reconciliation

Codex GPT-5.5 (`codex exec`), 2 rounds, artefact brut `sprint77_phase_f2_codex_review.md`
(R2 = output final). Lu, GAPs/PARTIELs triés, fixes appliqués + suites relancées.

- **R1 = PARTIEL** : 8 livrables, 6 CONFIRME, 0 GAP, **2 PARTIEL** — deux chemins **fail-OPEN**
  VRAM (la dimension B de la review Claude les avait sous-estimés comme « fail-closed-safe » ;
  Codex a eu raison de durcir : un KV-cache de plusieurs Go n'est PAS couvert par le headroom
  fixe 768 MiB) :
  - **R1-PARTIEL-A** (livrable 2) : `assess_capacity` avec `n_head == 0` (GGUF malformé) →
    `head_dim = 0` → terme KV = 0 → sous-estimation acceptée.
  - **R1-PARTIEL-B** (livrable 3) : `read_gguf_model_facts` *sautait* un tenseur au nom illisible
    (`if let Some(name)`) → sous-compte des poids résidents → sous-estimation.
- **Fixes (fail-closed, root cause)** :
  - Helper pur `is_degenerate_geometry(n_layer, n_embd, n_head)` (CI-testable) rejetant
    `n_layer==0 || n_embd==0 || n_head==0 || n_embd<n_head` (ce dernier ferme aussi le cas
    `head_dim = n_embd/n_head` collapsé à 0 par division entière) → `ModelUnreadable`. `n_head_kv`
    présent-mais-0 retombe sur `n_head`. Test `degenerate_geometry_is_rejected_fail_closed`.
  - Boucle tenseurs : `ctx.tensor_name(i).ok_or_else(|| ModelUnreadable(...))?` — un nom illisible
    REJETTE tout le header (jamais de skip silencieux).
  - Bonus : note `# Aborts` sur les accessors tenseurs vendorés (finding LOW review B).
- **Re-run boucle complète** : fmt ✓ · clippy `-D warnings` ✓ · nextest workspace **1900/1900** (+11
  vs F1 1889, dont `degenerate_geometry_is_rejected_fail_closed`) · CUDA build ✓ · feature claim
  tests **11/11** · Docker Linux touched (core-rs + worker-core) ✓.
- **R2 = CLEAN** : 8 livrables, **8 CONFIRME, 0 GAP, 0 PARTIEL**. R1-PARTIEL-A et R1-PARTIEL-B
  explicitement « RESOLU ». « Aucun chemin fail-open VRAM, aucun manifeste non-vérifié n'atteint
  le claim autorisé, aucun bump wire caché, aucun panic ajouté sur entrée réseau. »
