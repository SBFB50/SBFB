# Phase Review — Sprint 26 Phase A

## Verdict : PASS

Rigor signal : 2 findings P2 documentes (>=1 requis pour PASS rigoureux).

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, no band-aid — Phase A = 5 root-cause
  fixes identifies par audit S25, pas des pansements. Conforme.
- Routing table : aucune zone specifique matchee (admin_check, capability_store,
  key_rotation, guardrails ne sont pas dans les zones kudos/deploy/crypto/lib).
- Tensions plan vs memory : aucune.

## Staging check (Step 1bis)
- Phase fichiers : 9 (2 Rust src + 2 Python src + 2 Python tests + 1 guardrails src + 2 docs)
- Planning/docs split : ROADMAP_COMMITMENTS et HARDENING_ROADMAP sont explicitement
  dans le plan §A.6/A.7 — inclus dans le commit phase, pas chore separe.
- Untracked accidentels : 0

## Suites (Step 2)
- Rust nextest : 790 -> 792 (+2) PASS
- Rust doctests : pass (0 new)
- Rust clippy : 0 warning
- Rust fmt : clean
- Python SDK : 185 -> 185 (+0) PASS
- Python coord : 372 -> 376 (+4 pass), 5 -> 6 skip (+1 chmod Unix), 32 fail (PyO3 stale inchange) PASS
- Python gov : 46 -> 46 (+0) PASS
- Python ruff : clean
- Vitest : 264 -> 264 (+0) PASS (no frontend change)
- Playwright : 43 -> 43 (+0) PASS
- Size-limit : 7/7 PASS
- Release build nexus-shell-daemon : PASS
- scan-en-strings : PASS

## Delta tests cumule (Step 3)

| Suite | Before | After | Delta |
|---|---|---|---|
| Rust nextest | 790 | 792 | +2 |
| Python coord (pass) | 372 | 376 | +4 |
| Python coord (skip) | 5 | 6 | +1 |
| **Total new tests** | | | **+7** |

Tests ajoutes :
- Rust : `cache_rejects_stale_rotation`, `cache_rejects_same_timestamp_rotation`
  (rename `cache_overwrites_on_second_rotation` -> `cache_accepts_newer_rotation`)
- Python : `test_toml_roundtrip_determinism` (P2-HASH-1),
  `test_check_mil_high_has_null_guards` (P2-ADMIN-1),
  `test_sbfb_dir_permissions_0o700` (P2-CAPS-1, skip Windows),
  `test_stage_guards_valid_keys_accepted` (P2-STAGE-1),
  `test_stage_guards_invalid_key_raises` (P2-STAGE-1)

Plan estimait +8, reel +7. Ecart : test P2-ADMIN-1 est structurel
(source inspection) au lieu de comportemental (ctypes Win32 API
non mockable proprement dans pytest). Acceptable.

## Commit body validation (Step 4)
- Format titre : `feat(sprint26): Phase A — P2 batch S25 audit (5 fixes) + reclassification G7 LT-5/LT-6` MATCH
- Contexte present : oui (5 P2 detailles + reclassification)
- Fichiers touches avec rationale : oui (chaque P2 documente)
- Delta tests cumule coherent : oui (+7 reel, +8 plan)
- Scope cuts honoured : N/A (Phase A = batch cleanup)
- Co-Authored-By present : oui

## Research grounding (Step 4bis)
- **4bis-A** : S1a preflight `sprint26_phase_A_preflight.md` presente.
  Verdict APPROACH-ALIGNED — phase = defensive fixes, pas de design novel.
  Pas de projet OSS a consulter (NULL guards, permissions, stale rejection
  sont des patterns standard). PASS.
- **4bis-B** : pas de nouvelle dep ajoutee. Pas d'API externe touchee.
  §Research consulte du kickoff couvre le sprint. PASS.

## Horizon long-terme + documentation amont (Step 4ter)
- Design doc : N/A (Phase A = bug fixes, pas de nouveau module)
- D1..D5 avec alternatives : D4 dans kickoff cite "distribuer P2 dans
  phases B-D" et "defer S27+" comme alternatives rejetees. PASS.
- Solution la plus poussee : les fixes sont root-cause, pas workaround. PASS.
- LOC estimees au plan : **P2-PLAN-LOC** — voir §Findings ci-dessous.

## Modified-file branch coverage (Step 2bis, G9)
- `key_rotation.rs` : `apply_verified()` stale check branch (15 LOC)
  -> teste par `cache_rejects_stale_rotation` + `cache_rejects_same_timestamp_rotation` PASS
- `key_rotation_handler.rs` : `Err(e)` branch dans match apply_verified
  (4 LOC, warn + Err string) -> pas de test direct, mais `apply_verified`
  rejection testee en amont. CONCERN (< 10 LOC, logging only).
- `admin_check.py` : `if not sub_count_ptr` / `if not sub_auth_ptr`
  (2 LOC chacun) -> test structurel uniquement. CONCERN (trivial
  defensive guards, ctypes Win32 non mockable).
- `capability_store.py` : `if os.name != "nt": os.chmod(parent, 0o700)`
  (1 LOC) -> teste par `test_sbfb_dir_permissions_0o700`. PASS.
- `guardrails.py` : `validate_stage_guard_map()` (4 LOC)
  -> teste par 2 tests (valid + invalid). PASS.

## Scope cuts verification (Step 5)
12 scope cuts kickoff §7 — aucun touche par le diff Phase A :
- Tor transport : 0 fichier
- Arti library-embed : 0 fichier
- Domain fronting : 0 fichier
- Reliable-workers curator : 0 fichier
- GPU lockup : 0 fichier
- A4 process role tagging : 0 fichier
- C1 SQLiteSession : 0 fichier
- C5 streaming bridge : 0 fichier
- RAG sanitization : 0 fichier
- Pluggable transports : 0 fichier
- Full 12 events wire A3 : 0 fichier
- Platform writers journald/oslog : 0 fichier
PASS — 0 scope cut viole.

## Findings

- **P2-PLAN-LOC** : plan.md §6 "Budget LOC + tests" contient des
  estimations LOC prospectives par phase (~140, ~600, ~500, ~300, ~100).
  Contraire a feedback_approach.md §6 "Pas d'estimation LOC en amont".
  Kickoff §D4 contient aussi des LOC estimees. Plan et kickoff deja
  commites (`a97f7ca`), correction retroactive non applicable. Carry
  S27 audit track : verifier que les prochains plans/kickoffs n'ont
  pas d'estimation LOC.

- **P2-HANDLER-COVERAGE** : `key_rotation_handler.rs` branche Err
  du match `apply_verified` (stale rejection logging, 4 LOC) pas
  directement testee dans le handler. La logique sous-jacente
  `apply_verified` est couverte par 2 tests Rust. Risque faible
  (le handler test handle_valid_rotation_message exerce le happy path).

## Recommendation
- Ready to commit : **oui**
- Carry-overs S27 : P2-PLAN-LOC (verifier absence LOC estimates
  dans futurs plans)
- Corrections needed : aucune
