# Sprint 77 Phase F1 — Review

## Verdict: PASS

> Review Workflow ultracode (7 agents : 4 dimensions + vérification adversariale des P0/P1 +
> synthèse). Verdict initial **CONCERN** (2 P1 réels, 0 P0 après réfutation adversariale, 0
> scope-creep F1/F2, 0 bump wire). **Les 2 P1 + les P2 actionnables ont été corrigés et
> re-validés** ci-dessous → PASS-PENDING (review OK ; Codex pas encore exécuté). Artefact
> Workflow complet : `tasks/w702zkp5g.output`.

## Dimensions (verdicts initiaux)

| Dimension | Verdict initial | Après fixes |
|---|---|---|
| cpp-patch-pd (patch C++ + P-D) | CONCERN (1 P1 D1-1, 2 P2) | résolu |
| rust-backend (backend + embd injection) | CONCERN (1 P1 D2-1) | résolu |
| tests-helpers (sémantique tests) | CONCERN (P2 couverture) | renforcé |
| process-scope-build | PASS | inchangé |

Confirmations positives (INFO) : comptabilité P-D `n_created==n_tensors` correcte first/middle/last
(tied et non-tied) ; split F1/F2 propre (grep ComputeGroup/VRAM-fail-closed/verify_signature/
sbfb·shard·1/§16 = 0 hit) ; 0-bump-wire (0 `DOMAIN_*`, 0 `*_FORMAT_VERSION`, `shard_plan.rs`/
`canonical.rs`/`consent.rs` intacts) ; 2 fixes pré-existants = vrais fixes R2 (pas band-aids) ;
setter `set_shard_range` retiré = design correct ; licence MIT restaurée + `[patch.crates-io]` path
correct + impact CI nul.

## Findings P1 corrigés (must-fix)

### D1-1 (P1) — Middle-shard NULL-deref — CORRIGÉ + PROUVÉ
**Problème** : le P-D mettait `tok_embd` en `TENSOR_SKIP` sur un shard intermédiaire
(`is_first==false && is_last==false`) → `model.tok_embd == NULL`. Mais le builder LLAMA appelle
`build_inp_embd(model.tok_embd)` **inconditionnellement** et matérialise `ggml_get_rows(tok_embd,…)`
au build du graphe (le choix token vs embd est un select runtime sur un graphe déjà construit) →
NULL-deref/abort. Le test `partial_equals_full` n'utilisait que **2 shards** (head `is_first` + tail
`is_last`), donc `shard_tok_flags` n'était jamais `TENSOR_SKIP` → mode middle non-exercé.
**Fix** : `tok_embd` reste **résident sur chaque shard** (`create_tensor(TOKEN_EMBD, …, 0)`
inconditionnel pour le cas LLAMA) — c'est un tenseur unique (`n_vocab × n_embd`), négligeable vs les
poids de couches que P-D skip déjà ; `output_norm + lm_head` restent skippés hors last-shard
(prouvé sûr : le head 2-shard le faisait déjà). Local `shard_is_first` retiré du loader (inutilisé).
**Preuve** : nouveau test `#[ignore]` `shard_backend_three_way_equals_full` (head `[0,k)` + **MIDDLE**
`[k,m)` + tail `[m,L)`) — **PASS sur Mistral-7B-Q4, cosine > 0.999 vs forward complet** (le boundary
est relayé head→middle→tail par injection embd). Le mode intermédiaire est désormais exercé+vert.

### D2-1 (P1) — `add_embedding` non-borné en largeur — CORRIGÉ
**Problème** : `add_embedding` prenait `embd.len()` du caller comme stride ET longueur de copie
unsafe, sans le vérifier contre la largeur d'allocation de `new_embeddings` → primitive d'écriture
heap OOB pour tout caller dont la slice ≠ `n_embd` alloué (le caller in-tree `forward_hidden` était
sûr, mais l'API pub était unsound, et F2 ingérera des activations cross-machine non-fiables).
**Fix** : champ `embd_n: usize` stocké par `new_embeddings` (0 pour les batches token, posé aux 3
sites de construction), et `add_embedding` retourne `BatchAddError::WrongEmbeddingWidth { expected,
got }` si `embd.len() != self.embd_n` ; le stride utilise `self.embd_n`. Clause `# Errors` à jour.

## Findings P2 traités (should-fix)

- **D1-2 (P2)** — inversion d'ordre de validation : le `GGML_ASSERT` natif (fenêtre hors-bornes)
  abort dans `load_from_file` AVANT la validation Rust `ShardWindow::new`. **Documenté** dans
  `ShardBackend::load` (section `# Aborts` : le scheduler/F2 DOIT pré-valider ; le check post-load est
  un backstop défensif). Déplacement de la validation avant le load natif = **P2 tracké pour F2**.
- **D3-1 (P2 partiel)** — garde de forme du backend non testée en CI (vivait dans le module gated) :
  extraite en fn pure `hidden_token_count(hidden_len, n_embd) -> Option<usize>` (testée en CI sans
  GGUF, `hidden_token_count_validates_shape`) et utilisée par `forward_hidden`. Les gardes de rôle
  (`forward_tokens` sur non-first) restent couvertes par les `#[ignore]` — documenté honnêtement.
- **D3-5 (P2)** — setters vendorés sans test : **doctests round-trip** ajoutés à
  `with_shard_range` (model + context params), capturent une inversion start/end/is_first/is_last.
- **D3-2 (P2)** — `shard_backend_loads_layer_subset` ne prouve pas la réduction VRAM (seulement que
  le load réussit + géométrie correcte). Le doc du test reste **honnête** sur ce point ; la réduction
  est prouvée structurellement (`size_data -= nbytes` sur `TENSOR_SKIP` + logs unused-tensor).
- **D4-7 (P3 nit)** — `THIRD-PARTY-NOTICES` : prose « marqueur à chaque hunk » nuancée (le patch
  tracé `patches/llama-cpp-shard.patch` est l'enregistrement autoritatif).

## Re-validation post-fix

- Hermétiques (CI, sans feature) : **6/6 PASS** (ShardWindow ×3, top_k ×2, hidden_token_count ×1).
- GGUF `#[ignore]` (Mistral-7B-Q4) : **4/4 PASS** — `loads_layer_subset`, `hidden_state_extractable`,
  `partial_equals_full` (2-shard, cosine > 0.999), **`three_way_equals_full` (3-shard middle, cosine
  > 0.999)**.
- `cargo fmt --check` clean ; `clippy --workspace --all-targets -D warnings` clean ; nextest
  `--workspace` **1889/1889** (1883 baseline + 6 hermétiques) ; doctests OK.
- Builds fork : CPU vert ; **CUDA sm_120 (RTX 5080)** re-build incrémental vert (47 s) ; **Metal
  (Mac M2 arm64)** re-build vert.
- Patch régénéré (`patches/llama-cpp-shard.patch`, `shard_tok_flags` retiré ; 5 fichiers).

## Codex reconciliation

Codex GPT 5.5 (`codex exec`), 2 rounds, artefact brut `sprint77_phase_f1_codex_review.md`.
- **R1 = PARTIEL** : 1 P1 réel — `LlamaBatch::add_embedding` unsound hors chemin ShardBackend
  (`embd_n == 0` + slice vide → write dans `embd` null ; `seq_ids` non borné par `n_seq_max`) +
  P1/P2 absence de fail-close architecture dans `ShardBackend::load`.
- **Fixes** : guards `NotAnEmbeddingBatch` (rejet batch token) + `WrongEmbeddingWidth` +
  `TooManySequences` (champs `embd_n` / `n_seq_max` stockés aux 3 sites de construction) ;
  `ShardBackend::load` fail-close `general.architecture == "llama"` (Llama/Mistral/Mixtral).
- **R2 = CONFIRMÉ, GAP P0/P1 : aucun** (réserve non-bloquante : Codex n'a pas relancé les GGUF
  `#[ignore]` faute de `SBFB_SHARD_TEST_GGUF` — relancés localement par nous, 4/4 PASS).
- Re-run post-fix : 4/4 GGUF (incl. 3-shard middle), 6/6 hermétiques, nextest 1889/1889,
  clippy/fmt/doctests clean, builds CPU+CUDA+Metal verts.
