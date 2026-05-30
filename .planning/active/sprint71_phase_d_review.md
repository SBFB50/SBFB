# Phase Review — Sprint 71 Phase D (reconciliation off-sprint G5/G6 + fix securite P1)

## Verdict: PASS

Promu de PASS-PENDING a PASS apres reconciliation du Codex de phase (3 passes
GPT-5.5, voir §Codex reconciliation). Verdict Codex final : "no remaining real
P0/P1 code defect, committable as fix(factory)".

(Rigor signal : 3 findings P2 + 1 P3 documentes, PLUS 2 findings securite
trouves par Codex et FERMES in-phase (P1 git option injection, P2 terminal
drive-prefix) / >=1 P2+ requis pour PASS.)

Date : 2026-05-30. HEAD : `a0337c6` (commit Phase D non pose). Fallback skill
`nexus-phase-review` (l'agent `nexus-phase-review-deep` n'est pas enregistre
dans cette session — memes verdicts, profondeur reduite). Preflight verdict
PLAN-ADAPT (crate binary-only -> tests inline + harness HTTP).

## Staging check (Step 1bis)
- Phase fichiers source (5) : `crates/sbfb-factory/src/operator_server.rs`,
  `process.rs`, `sprint_history.rs`, `terminal.rs`,
  `tests/operator_server.rs`. Diff 339+/4-.
- Artefacts planning de CETTE phase (3 untracked) :
  `sprint71_phase_d_preflight.md` + `sprint71_offsprint_retro_review.md` (G5)
  + `sprint71_offsprint_codex_review.md` (G5 brut) — vont dans le commit
  phase (G8 artefact). Ce `review.md` + le `phase_d_codex_review.md` a venir
  s'y ajoutent.
- Planning/docs split : N/A — aucun fichier d'un sprint anterieur, aucun doc
  hors-scope. Les 2 artefacts G5 sont des livrables explicites du plan §8 D.2.
- Untracked accidentels : 0. Aucun scope-cut leak (grep ProviderRouter/
  SearchManifest/tree-sitter/sharding/gpu-share sur le diff = 0 hit).
- **Working tree coherent pour un commit phase atomique.**

## Memory consultation (Step 1.5)
| Memory | Contrainte | Statut |
|--------|-----------|--------|
| `feedback_approach.md` | pick deepest, no band-aid, research before code | RESPECTE — fix P1 root-cause double couche (guard app + `--end-of-options` natif git), pas un patch ad-hoc ; preflight G8 fait avant code |
| `feedback_context7_systematic.md` | context7 avant code lib/API | N/A — zero nouvelle dep/API (tests + guard sur API git existante) ; preflight S1b trace `portable-pty 0.9.0` deja scanne |
| `feedback_model_46.md` | claude-opus-4-8[1m] | RESPECTE — aucun modele touche (Phase D = tests/securite) |
| `feedback_codex_raw_output.md` | codex artefact = sortie brute | RESPECTE — retro-Codex G5 brut non reecrit ; Codex de phase a venir idem |

Aucune violation memory (pas de P1 memory).

## Suites (§7.4) — etat FINAL avec le fix P1
- cargo fmt --all --check : **clean**.
- cargo clippy --workspace --all-targets --locked -- -D warnings : **0 warning**
  (`is_safe_git_rev` clippy-clean).
- cargo nextest run --workspace --locked : **1528 passed, 0 skipped**
  (baseline Phase C 1512 -> **+16**).
- cargo test --workspace --locked --doc : 0 fail.
- cargo build -p nexus-shell-daemon --release : OK.
- sbfb-factory ciblé : 112 -> **128** (+16).
- Frontend non-regression (front NON touche par le diff, lance par discipline
  §7.4 2-blocs) : factory-operator `eslint` 0 + `npm run build` OK ;
  web/ shell `tsc` 0 + Vitest **279/279**. Zero regression cross-stack.

| Suite | Avant | Apres | Delta |
|-------|-------|-------|-------|
| Rust workspace (nextest) | 1512 | 1528 | +16 |
| Rust doctests | 0 | 0 | +0 |
| Vitest unit (web/) | 279 | 279 | +0 (non touche) |
| size-limit | 6/6 | 6/6 | inchange (front non touche) |

Decomposition +16 : `terminal::tests` 2 (session_log_roundtrip,
list_sessions_filters_correct_extension) ; `process::tests` 3
(resolve_kind_aliases, providers_list_is_canonical, repo_root_resolves) ;
`sprint_history::tests` 3 (parse_unified_diff_classifies_line_kinds,
extract_section_stops_at_next_header, extract_verdict_reads_plan_adapt) ;
integration `operator_server` 8 (sprint_history, sprint_history_all,
commit_diff_returns_inline_code, commit_diff_rejects_invalid_sha,
commit_diff_rejects_option_injection, audit_rejects_option_injection,
terminal_sessions, terminal_session_content_rejects_traversal).

## Modified-file branch coverage (Step 2bis, G9)
Fichiers EXISTANTS modifies + nouvelles branches :
- `operator_server.rs` `is_safe_git_rev` (NEW) + guard `handle_audit` →
  `operator_audit_rejects_option_injection` (400 + assert zero fichier) ✅
- `operator_server.rs` guard etendu `handle_commit_diff` →
  `operator_commit_diff_rejects_option_injection` + `..._rejects_invalid_sha` ✅
- `operator_server.rs` guard `handle_terminal_session_content` →
  `operator_terminal_session_content_rejects_traversal` ✅
- `sprint_history.rs` `--end-of-options` dans `commit_diff_data` → exercice
  par `operator_commit_diff_endpoint_returns_inline_code` (HEAD, 200) +
  l'injection test (option neutralisee) ✅
- `process.rs` `--end-of-options` dans `audit_commit_data` → exercice par
  `operator_audit_endpoint` (HEAD existant) + injection test ✅
Toutes les nouvelles branches de logique securite ont >= 1 test. ✅

## Commit body validation (Step 4 / 4bis)
Titre cible : `fix(factory): Sprint 71 Phase D — reconcile off-sprint block +
harden git rev injection (retro-review + coverage)`. Type promu `test`->`fix`
(le diff inclut un fix securite P1, pas seulement des tests). Matche
`PHASE_TITLE_RE` (Phase D). Draft body 9 headers (genere au G-COMMIT) :
`## Contexte`, `## Fichiers`, `## Delta tests`, `## Verification §7.4`,
`## Scope cuts`, `## G8 traceability`, `## Pre-launch protocol`,
`## Codex verification`, `## Carry closure`. Delta +16 coherent.
Co-Authored-By Opus 4.8 (1M) requis.

## Research grounding (Step 4ter)
- **4ter-A preflight G8** : `sprint71_phase_d_preflight.md` existe, 5 scans
  S1a-S4 presents, verdict PLAN-ADAPT documente avec evidence (binary-only
  crate, asciicast v2 spec, RustSec portable-pty). S1a nomme docs.asciinema.org
  + rustsec.org. PASS.
- **4ter-B deps** : zero nouvelle crate au lock (Phase D = tests + guard +
  flag git). PASS.

## Horizon long-terme + documentation amont (Step 4quater)
- Pas de nouveau module structurant (tests inline + guard dans fichier
  existant). ✅
- Fix P1 = solution la plus poussee : `--end-of-options` (barriere git native
  couvrant toute la classe d'injection) au-dela du `git rev-parse --verify`
  suggere par Codex, + guard d'entree defense-en-profondeur. ✅
- Aucune estimation LOC au plan. ✅

## Scope cuts verification (Step 5)
Aucune ligne du diff ne touche un scope cut (kickoff §8 / plan §12) :
- #1 ProviderRouter, #2 routage chat, #3-6 recherche reseau/FTS5/SearchManifest,
  #7-9 fork/projet/templates, #10-11 GPU/cross-machine, #12 sharding,
  #13 logprobs, #15 tree-sitter, #16 packaging → 0 hit dans le diff. ✅
- Diff limite a `sbfb-factory` (src + tests) + artefacts `.planning/`. ✅
- Wire format / `_VERSION` / canonical : 0 touche (S4 preflight clean). ✅

## Findings (rigor signal — 3 P2 + 1 P3)
- **P2** : **G12 PARTIEL** (carry Phase C). Le retro-Codex note que le timeout
  spawn est *idle-only* (`llm_bridge.rs`), pas un deadline total, et qu'il n'y
  a pas de resolver pre-spawn (seulement un diagnostic post-erreur). Ferme
  fonctionnellement Phase C (timeout + diagnostic presents), residual =
  deadline total + pre-spawn resolver. **Carry S72**.
- **P2** : **Champ `validator.sha256` mal nomme** (retro-Codex). Il stocke le
  `result_text` brut, pas un hash (commentaire `validator.rs:86`). Dette de
  nommage heritee Phase B, sans impact fonctionnel (quorum exact correct).
  **Carry S72**.
- **P2** : **Tests endpoint happy-path structurels** (status + shape), pas
  d'assertion de contenu profond (note "shallow" du retro-Codex). DELIBERE :
  robustesse aux clones CI shallow (le diff endpoint utilise `HEAD`, assert
  `files` array sans exiger non-vide) ; la profondeur de contenu vit dans le
  test hermetique `parse_unified_diff_classifies_line_kinds` + les 3 tests
  securite (400 + zero fichier ecrit). Acceptable, documente.
- **P3** : `is_safe_git_rev` est un denylist (rejet leading-`-`/whitespace/
  control), pas un allowlist hex strict (suggestion Codex). Justifie :
  `--end-of-options` ferme deja toute la classe d'injection cote git ; un
  allowlist hex strict rejetterait des revs legitimes (`HEAD~3`, tags) que
  l'endpoint audit doit accepter. Defense-en-profondeur suffisante.

Aucun P0/P1 ouvert. **Le P1 git option injection trouve par le retro-Codex G5
est FERME dans cette meme phase** (guard `is_safe_git_rev` + `--end-of-options`
sur /api/audit ET /api/sprint-history/diff + 3 tests reproduisant les
live-probes Codex). Le P2 traversal terminal idem ferme.

## Codex gate (§4.5) — zero exemption
- Status : **FAIT** — Codex de PHASE GPT-5.5 (`sprint71_phase_d_codex_review.md`,
  DISTINCT du retro-Codex G5). 3 passes brutes `codex exec -o`, jamais
  reecrites. Le retro-Codex G5 (off-sprint) ET le Codex de phase ont chacun
  *live-probe* les endpoints (injection `--output=`, traversal `C%3A...`).
- Verdict final (3e passe) : 7/7 livrables CONFIRME, **0 P0/P1, 0 GAP**.

## Codex reconciliation
- Status : **FAIT** (3 passes — convergence CLEAN).
- **Passe 1 (retro-Codex G5, off-sprint)** : a debusque le **P1 git option
  injection** (`--output=` ecrit un fichier sur /api/audit + /diff) + G12/G6
  PARTIEL. → P1 FIXE in-phase (is_safe_git_rev + --end-of-options) + 2 tests.
- **Passe 2 (Codex de phase, diff complet)** : git injection CONFIRME CLOSED ;
  a trouve (a) **P1 mapping G16** errone dans le retro-review → CORRIGE
  (G16=P1 DEFER S72+) ; (b) **P2 drive-prefix** terminal (`C:` echappait
  `.planning/terminal/` sur Windows, live-probe) → FIXE (rejet `:` + check
  `path.parent()==term_dir`) + test reproduisant le probe ; (c) hygiene tests
  injection (cleanup avant assert) → CORRIGE. Findings structurels (index
  vide / PASS-PENDING / type test->fix) = ordre de gate normal, resolus par
  staging + type fix(factory) + cette promotion PASS.
- **Passe 3 (confirmation)** : git injection CLOSED + terminal traversal
  (drive-prefix inclus) CLOSED + G16 CONFIRME + regression 128/128 +
  scope/Day-0 CONFIRME. **New real P0/P1 : NONE FOUND.**
- Suites relancees apres chaque correction : fmt clean, clippy 0,
  nextest sbfb-factory 128/128, workspace 1528. Aucun GAP P0/P1 ouvert.
- Review final : **PASS** (3 rapports Codex lus, tous GAPs P0/P1 corriges,
  P2/P3 documentes, securite live-probee CLOSED).

## Recommendation
- Ready to commit : **OUI** (PASS final, Codex reconcilie sur 3 passes, 0 P0/P1).
- Carry-overs S72 (entree obligatoire `sprint72_audit_findings.md`) :
  G12 deadline total + pre-spawn resolver ; `validator.sha256` renommage ;
  endpoint happy-path content-depth (optionnel) ; G16/G4/G10 P1 hors socle
  (DEFER S72-S74 per audit-absorb).
- Corrections appliquees in-phase : P1 git option injection (guard +
  --end-of-options) ; P2 terminal drive-prefix (guard `:` + parent check) ;
  retro-review G16 reclassifie P1 ; hygiene tests injection.

## Post-commit obligatoire
- [ ] Update `nexus_grid_pivot.md` (tip SHA + compteurs 1528 + carries P2).
- [ ] Update `MEMORY.md` (ligne index pivot).
- [ ] `review.md` + `preflight.md` + retro-review + retro-Codex + Codex de
      phase stages dans le commit phase.
