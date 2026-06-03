# Phase Review — Sprint 72 Phase B (dette pair : P2-F-3 3/3 + P2-A-2 + P3)

## Verdict: PASS

Promu de PASS-PENDING a PASS apres : (1) review Claude 5 dimensions
adversariales = 0 finding ; (2) Codex GPT 5.5 reconcilie 5/5 CONFIRME /
0 GAP / 0 PARTIEL ; (3) code Phase B prouve propre sur Linux NATIF
(fmt/clippy/doctest verts, 3 livrables verts, operator_server vert en
natif). Les 2 seuls tests rouges du gate (operator_server bind-mount,
process_cli SHA-hardcode) sont pre-existants, reproduits a l'identique
sur master pur `105c054`, et exclus du binaire par construction (diff
process.rs 100% `#[cfg(test)]`). Cf. §Suites + §Codex reconciliation.

(Rigor signal : 0 finding code P2+ apres review adversariale 5 axes +
verification refutatoire — phase dette mecanique, surface tres reduite.
Observations ENVIRONNEMENT documentees §Suites : 2 binaires de test
pre-existants rouges sous bind-mount/SHA-hardcode, CI-Linux-only/zombie,
hors-code et hors-diff Phase B.)

## Staging check (Step 1bis)
- Phase fichiers (6 code/docs) : `crates/nexus-core-rs/src/blobs.rs`,
  `crates/nexus-core-rs/src/lib.rs`,
  `crates/nexus-worker-core/src/engine/runtime.rs`,
  `crates/nexus-shell-daemon/src/dispatch_loop.rs`,
  `crates/sbfb-factory/src/process.rs`, `docs/agent/AGENT_SYSTEM.md`.
- Artefacts phase untracked attendus : `sprint72_phase_b_preflight.md`,
  ce `sprint72_phase_b_review.md`, + `sprint72_phase_b_codex_review.md`
  (a venir). Entrent dans le commit de phase (pas de chore intermediaire).
- Untracked accidentels : 0 (confirme par dimension scope-process —
  exactement 6 fichiers attendus + l'artefact preflight).

## Suites (§7.4)
- **Gate Docker Linux (AUTORITAIRE)** : image
  `rust:1.94@sha256:b644cc33…` (l'image CI Woodpecker/GitHub exacte, pas
  l'image dev `sbfb-ci`), `apt libgtk-3-dev libxdo-dev pkg-config`, puis
  `cargo fmt --all --check` + `cargo clippy --workspace --all-targets
  --locked -- -D warnings` + `cargo test --workspace --locked` + `cargo
  test --workspace --locked --doc`. Workspace COMPLET, `nexus-launcher`
  INCLUS (le CI ne l'exclut pas ; le launcher compile sur Linux grace a
  GTK). Plusieurs runs ont ete necessaires (3 reboots PC, Docker
  intermittent, bind-mount lent) ; le run AUTORITAIRE final est
  `b70ndqwc1` (Phase B en filesystem natif, detaille plus bas). Lecon
  retenue : ne jamais se fier a un marqueur agrege — un `set -e` a gauche
  d'un `&&` avait produit un faux `GATE_LINUX_ALL_GREEN` ; tous les RC
  sont desormais captures et lus individuellement, persistes dans
  `.planning/active/s72b_gate_evidence/`.
  - **2 binaires de test en echec sur bind-mount, TOUS DEUX PROUVES
    pre-existants et hors-diff Phase B** (investigation complete ;
    artefacts auditables persistes dans
    `.planning/active/s72b_gate_evidence/`) :

    **(i) `tests/operator_server.rs` — 13 echecs `reqwest TimedOut`
    (environnement bind-mount).** Les 13 handlers en echec
    (`/api/chat/*`, `/api/context`, `/api/context-pack`,
    `/api/sprint-history` single) appellent in-process
    `process::context_data` (`run_git` rev-parse/branch) ou
    `sprint_history_data` (`git log`) ; ces `git` s'executent sur le
    `.git` BIND-MONTE Windows->Docker (latence 9p/virtiofs) et depassent
    le timeout reqwest 5s. Discriminant : `/api/sprint-history/all`
    (`all_sprints_data` -> lit `.planning/` sans git) PASSE, vs
    `/api/sprint-history` single (git log) ECHOUE. Re-run en filesystem
    NATIF du conteneur : **operator_server PASSE** (les 13 disparaissent ;
    `git_native` rapide). Verifie aussi : `git show HEAD:operator_server.rs`
    n'a AUCUN `spawn_factory`/`Command::new`/`current_exe` (une hypothese
    intermediaire "re-spawn binaire 520 MB" etait fausse — c'est `git`
    in-process).

    **(ii) `tests/process_cli.rs` — 1 echec
    `audit_commit_valid_phase_commit` (test zombie SHA-hardcode).** Le
    test exige que `audit-commit --rev 6fb95df` (commit S70 Phase F, SHA
    HARDCODE dans le test ligne ~473) passe. `audit-commit` retourne
    `ok:false, issues:["missing review file","missing codex_review file"]`
    car les artefacts S70 ont ete ARCHIVES (`git mv active/ ->
    archive/v2.1/`) apres ce commit. Pre-existant, fragile, sans rapport
    avec Phase B.

  - **PREUVE D'INDEPENDANCE Phase B (double, decisive)** :
    1. **Empirique** : les DEUX echecs (operator_server 13x + process_cli
       1x) se reproduisent **a l'identique sur master PUR `105c054`**
       (clone `--no-hardlinks`, `DIRTY=0`, overlay Phase B retire) —
       `audit_probe.log`. Ils ne dependent donc pas du diff Phase B.
    2. **Par construction** : le diff Phase B sur `process.rs` est
       entierement sous `#[cfg(test)] mod tests` (hunk `@@ -883,4 +883,92
       @@`, `mod tests {` ouvre ligne 842) -> EXCLU du binaire
       `sbfb-factory` compile -> le binaire que `operator_server` et
       `process_cli` SPAWNENT est byte-identique avec/sans Phase B. Les
       resultats de ces 2 binaires sont mathematiquement insensibles a
       Phase B. Les 5 autres fichiers du diff (re-export `Store`,
       `blob_store`, test `dispatch_loop`, doc `AGENT_SYSTEM`) ne sont pas
       dans leur call-graph.

  - **Gate Phase B en filesystem NATIF (iso-environnement, run
    `b70ndqwc1`, persiste `native_phaseb.log` + `n_rc_*.txt` +
    `n_test_workspace.log`)** : overlay des 6 fichiers Phase B sur un
    clone natif `105c054` (`PB_OVERLAY_DIFF=6`), meme environnement que
    le run master qui donne operator_server 37/37. Resultat lu
    explicitement (pas de marqueur agrege ; le `GATE_LINUX_ALL_GREEN`
    d'un run anterieur etait un faux positif `set -e`-a-gauche-d-`&&`) :
    - `RC_SUMMARY: FMT=0 CLIPPY=0 DOCTEST=0` ; `TEST` non-zero **du seul
      fait du test zombie process_cli (i) ci-dessus**, hors-diff.
    - `OK_BINARIES=24`, `FAILED_BINARIES=1` (= process_cli pre-existant).
    - operator_server : **PASSE en natif** (13 echecs bind-mount evapores).
    - **3 livrables Phase B verts** : `process::tests::
      prompt_kinds_resolve_to_existing_files ... ok`,
      `agent_wrappers_reference_existing_prompts ... ok`,
      `dispatch_loop::tests::dispatched_task_is_claimed_and_executed_by_worker_engine
      ... ok`.
    Conclusion gate : le **code Phase B est propre** (fmt/clippy/doctest
    verts, 3 livrables verts, operator_server vert en natif). Les seuls
    rouges sont 2 tests pre-existants fragiles, reproduits a l'identique
    sur master pur et exclus du binaire par construction.
- **Windows deterministe (informationnel)** : `cargo fmt --all --check`
  → 0 diff ; `cargo clippy --workspace --all-targets --locked -- -D
  warnings` → vert ; `sbfb-factory` unit + tests passent (dont les 2
  tests P2-F-3). Concordant avec Linux.
- **Windows full workspace (informationnel, NON-bloquant)** : l'E2E
  P2-A-2 et ~15 tests reseau iroh (`blobs`/`docs`/`discovery`/`gossip` +
  http daemon) + `second_start_refuses_when_first_still_running`
  (e2e.rs:247, singleton) **hang/timeout sur Windows natif** (flakiness
  iroh-reseau sous charge, portmapper UPnP — cf. memory `P2-A-1(S71)`
  worker-pump iroh-docs hang Windows natif = CI-Linux-only ;
  `feedback_wsl_before_push`). Aucun ne touche le diff Phase B et l'E2E
  P2-A-2 passe sur le gate Linux. Gate autoritaire = Docker Linux.
- **Delta tests** : **+2 Rust** (2 tests P2-F-3) ; P2-A-2 RENFORCE un E2E
  existant (assertions ajoutees, pas de nouveau test → +0). Front (`web/`)
  intouche → Vitest 279 / size 6 N/A. Cible canonique Linux : 1528 → 1530
  (re-mesure exacte au vert Docker + Phase F).

## Critere d'acceptation (plan §5.4)
| Check | Attendu | Observe | Signal |
|---|---|---|---|
| `cargo nextest -p sbfb-factory` | P2-F-3 verts | 130/130 (2 nouveaux PASS) | ✅ |
| `cargo nextest -p nexus-shell-daemon` (E2E signature) | assert signature vert | PASS [2.082s] | ✅ |
| P2-F-3 ferme (check mecanique en place) | binaire | 2 tests bidirectionnels | ✅ |
| P2-A-2 assertion signature presente | binaire | `verify_signature().expect()` | ✅ |
| P3 tranches (fix ou re-doc) | binaire | re-doc §P53 (0 code, intentionnel) | ✅ |

## Modified-file branch coverage (Step 2bis, G9)
- `process.rs` : 2 nouvelles fonctions test + helper `prompt_refs_in`
  (couvert par ses 2 appelants test ; branches `.md`/non-`.md`,
  terminateurs, exercees sur les wrappers reels). ✅
- `runtime.rs` `Engine::blob_store()` : couvert par l'E2E P2-A-2 (le
  clone est consomme par `BlobsClient::get_bytes`). ✅
- `dispatch_loop.rs` : la branche d'assertion signature est sur le chemin
  nominal du test (echoue si signature invalide). ✅
- `blobs.rs`/`lib.rs` : re-export pur (pas de branche executable). N/A.

## Research grounding (Step 4ter)
- 4ter-A — Preflight G8 : `sprint72_phase_b_preflight.md` existe, verdict
  **EXECUTE** (5 scans ; S2 reverse-commit confirme P2-F-3 NON resolu
  depuis S70 ; S3 P2-A-2 renforce la couverture ; S4 0 wire touche). → PASS.
- 4ter-B — Deps/API : 0 dep ajoutee (le bump ollama-rs est Phase C, pas B).
  API `iroh_blobs::api::Store` : `#[derive(Clone)] #[repr(transparent)]`
  sur `irpc::Client` partage — verifie source vendored 0.100.0 (le clone
  partage le backend, condition de correction de P2-A-2). ✅

## Horizon long-terme + documentation amont (Step 4quater)
- Solution la plus poussee retenue (preflight option a renforcee) : test
  Rust **bidirectionnel** co-localise avec la source de verite (`process.rs`
  detient `PROMPT_KINDS`/`prompt_filename`/`repo_root`), au lieu d'un hook
  bash hors-compte (option b) ou d'une note doc non-mecanique (option c).
  La note doc (`AGENT_SYSTEM.md`) vient EN PLUS du garde-fou, pas a la
  place. P2-F-3 ferme 3/3, plus jamais carry. ✅
- P2-A-2 : `verify_signature()` self-contained (cle embarquee) > simple
  comparaison de pubkey (assertion plus forte, prouve l'authenticite). ✅
- Aucune LOC estimee au plan (§6.7) : ✅.

## Scope cuts verification (Step 5)
- Phase DETTE pure (dimension scope-process CONFIRMEE) : 0 feature,
  3 garde-fous mecaniques + 1 accesseur test-support + 1 re-export + doc.
- Scope cuts plan §11 (recherche S73, fork S74, packaging S74, GPU S75,
  sharding S76, multi-cloud hors roadmap, streaming WAN jamais PO-14) :
  aucun touche par le diff. ✅
- P3 intentionnellement sans code (cast `as i32` preserve pour re-verif
  migration Phase C ; §P53 documente sha256 misnomer + seed cast +
  task_id). ✅

## Adversarial review (5 dimensions, 0 finding confirme)
Review multi-agent (workflow `wf_02f41153-488`, 5 dimensions x verif
refutatoire) — **0 P0/P1/P2/P3 souleve, 0 a refuter** :
- **correctness P2-A-2** : SOUND — clone partage le backend (usage
  `node.rs:337`), pas de race GC (MemStore `data_dir:None`, blobs pin par
  iroh-docs), format serde coherent (`to_vec`↔`from_slice<ResultEntry>`),
  hash BLAKE3 coherent, `verify_signature()` non-vacant (Ed25519 sur
  `canonical_bytes(payload, DOMAIN_RESULT_V1)`).
- **robustesse P2-F-3** : CLEAN — 8 kinds + 8 refs verifies, garde
  `checked>0`, parser sans infinite-loop (marker sans terminateur,
  `max(1)`), `PathBuf::join` cross-OS, echouerait sur rename/typo/ajout.
- **securite/invariants** : CLEAN — `canonical.rs`/`DOMAIN_RESULT_V1`/
  `TASK_FORMAT_VERSION`/`*_ANNOUNCEMENT_VERSION` zero-diff confirme ;
  re-export `Store` = ergonomie (deja atteignable via `BlobsClient::new`/
  `Node::blobs_store`) ; tout read-only.
- **scope/process** : CLEAN — dette pure, P3 intouche, 6 fichiers attendus.
- **style** : CLEAN — `blob_store`(owned)/`blobs_store`(borrowed)
  desambiguise + doc ; helper idiomatique (slices UTF-8, pas d'unwrap nu).

## Findings
- **0 finding code** apres review adversariale + verification refutatoire.
- **1 observation ENVIRONNEMENT (non-code, non-bloquante)** : flakiness
  iroh-reseau Windows-natif (15 timeouts + 1 singleton) — pre-existante,
  documentee CI-Linux-only (carry `P2-A-1(S71)`, 2/3 → 3/3 MANDATORY S73).
  N'affecte pas le diff Phase B. Le gate canonique Docker Linux tranche.

## Codex gate (§4.5) — zero exemption
- Status : **FAIT** — `codex exec` GPT 5.5 lance avec
  `.git/CODEX_SPRINT72_PHASE_B.txt` (5 livrables), output brut dans
  `sprint72_phase_b_codex_review.md` (102 lignes, 0 CR / 0 ANSI,
  **NON reecrit/condense par Claude**).
- Docs/test n'exempte PAS (§4.5.6) : la phase a du code, Codex execute.

## Codex reconciliation
- **Verdict Codex : 5/5 CONFIRME, 0 GAP, 0 PARTIEL.** Output brut lu et
  trie ; aucun GAP P0/P1, aucun P2/P3 — rien a corriger, pas de boucle
  re-run necessaire.
- Detail par livrable (cf. `sprint72_phase_b_codex_review.md`) :
  1. P2-F-3 tests prompt/wrapper — CONFIRME (`process.rs:887,907,955` ;
     8 kinds, 8 fichiers, 8 refs wrappers, `checked > 0` non-vacant).
  2. P2-F-3 doc AGENT_SYSTEM — CONFIRME (`AGENT_SYSTEM.md:220`, cite les
     2 tests, ferme 3/3).
  3. P2-A-2 assertion signature E2E — CONFIRME (`dispatch_loop.rs:223,
     248-256` ; assertion peut reellement echouer : result lu depuis
     blob, pubkey depuis le `ResultEntry` stocke, pas fabrique cote test).
  4. P2-A-2 support `blob_store`/`Store` re-export — CONFIRME ; Codex a
     verifie INDEPENDAMMENT la source vendored `iroh-blobs 0.100.0`
     (`api.rs:212-215`, `#[derive(Clone)] #[repr(transparent)]`) et
     `irpc 0.14.0` (`lib.rs:1287-1290`) : le clone partage le backend —
     condition de correction de P2-A-2 etablie cross-source.
  5. P3 doc PATTERNS sans code — CONFIRME (`PATTERNS.md:2736` §P53 ;
     aucun code P3 modifie ; le seul fichier P3-adjacent touche,
     `runtime.rs`, ne contient que `Engine::blob_store()` pour P2-A-2).
- Invariants transverses confirmes par Codex : 0 bump wire
  (`TASK_FORMAT_VERSION`/`*_ANNOUNCEMENT_VERSION`), 0 changement
  `canonical_bytes`/`DOMAIN_RESULT_V1`, dette pure (0 feature).
- Suites NON relancees apres Codex pour le CODE (0 GAP Codex). Le gate
  Linux est en revanche re-lance pour cause d'artefact harness (binaire
  `sbfb-factory` absent du volume → 13 echecs `operator_server` hors
  diff), pas pour une correction de code Phase B.
- Promotion : `## Verdict: PASS-PENDING` -> `## Verdict: PASS`. Conditions
  remplies : (1) Codex 5/5 CONFIRME 0 GAP reconcilie ; (2) code Phase B
  prouve propre sur Linux NATIF (fmt/clippy/doctest verts, 3 livrables
  verts, operator_server vert en natif) ; les 2 seuls rouges (operator_server
  bind-mount, process_cli SHA-hardcode) sont pre-existants, reproduits a
  l'identique sur master pur `105c054`, et exclus du binaire par
  construction (diff 100% `#[cfg(test)]`).

## Recommendation
- Ready to commit : **OUI** — Codex reconcilie 5/5 (0 GAP) ; gate code
  Phase B vert en filesystem natif (operator_server inclus) ; les 2 tests
  rouges sont des artefacts pre-existants hors-diff (preuve empirique
  master-pur + preuve par construction `#[cfg(test)]`), persistes et
  auditables dans `.planning/active/s72b_gate_evidence/`.
- Corrections needed : aucune sur le CODE Phase B (review 0 finding,
  Codex 0 GAP, 3 livrables verts).
- Carry-overs S73 (a ecrire dans `sprint73_audit_plan.md`) :
  1. `audit_commit_valid_phase_commit` (`tests/process_cli.rs`) : SHA
     hardcode `6fb95df` dont les artefacts review/codex ont migre en
     archive -> test zombie, a dé-hardcoder (creer un fixture commit ou
     pointer un SHA dont les artefacts vivent encore dans active/). Echoue
     deja sur master pur — pre-existant, a tracer comme dette test.
  2. `operator_server.rs` (13 tests) : timeout `git` sur `.git`
     bind-monte Windows->Docker. CI-Linux-only (passe en natif). Meme
     famille que P2-A-1(S71) iroh-docs pump Windows natif — root-cause OU
     exemption formelle CI-Linux-only a ecrire S73.
  3. P2-A-1(S71) iroh-docs pump Windows passe 3/3 MANDATORY (deja note
     plan §6 / kickoff).

## Post-commit obligatoire
- [ ] Update `nexus_grid_pivot.md` (tip SHA + P2-F-3 CLOSED 3/3 + P2-A-2
      CLOSED + Phase B done)
- [ ] Update `MEMORY.md` (ligne index pivot)
- [ ] Verifier preflight.md + review.md + codex_review.md stage dans le
      commit de phase
