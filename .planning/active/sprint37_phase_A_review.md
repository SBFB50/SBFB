# Phase Review — Sprint 37 Phase A

## Verdict : PASS (2 P2 + 1 P3)

Rigor signal : 3 findings documentes (>=1 P2+ requis pour PASS rigoureux).

## Memory consultation
- feedback_approach.md : pick deepest, no band-aid — respecte (tracing-appender = pattern daemon, pas band-aid File::create)
- feedback_context7_systematic.md : context7 avant code — respecte (preflight S1b queries tracing-appender + icns)
- Violations : aucune

## Staging check (Step 1bis)
- Phase fichiers : 12 (+ tools/png-to-icns/ 2 new)
- Planning/docs split : chore(planning) preflight committe `c174360` avant feat ✅
- Untracked accidentels : 0

## Suites
- Rust nextest : 936 → 940 (+4) ✅
- Rust doctests : 0 pass, 1 ignored (inchange) ✅
- Python SDK : 195 (inchange) ✅
- Python coord : 409+36f+6s (36 fail PyO3 stale, pre-existing) ✅
- Python gov : 46 (inchange) ✅
- Vitest : 267 (inchange) ✅
- Frontend build + size : OK ✅
- `cargo clippy --workspace` : 0 warnings ✅
- `cargo fmt --all --check` : OK ✅
- `cargo build -p nexus-shell-daemon --release` : OK ✅

## Delta tests
- Plan §A.4 attendu : +3 mutex poisoned + +1 log path = +4
- Reel : 936 → 940 = +4 ✅ (match exact)

## Commit body validation
- Format titre : ✅ `feat(sprint37): Sprint 37 Phase A — MANDATORY log convergence + .icns + P2 batch audit/review S36`
- Contexte present : ✅ (§A.1, §A.2, §A.3 detailles)
- Fichiers touches avec rationale : ✅
- Delta tests cumule coherent : ✅ (+4 match plan + reel)
- Scope cuts honoured : ✅ (SC-1, SC-5, SC-12 listes)
- Co-Authored-By present : ✅

## Research grounding (Step 4bis)
- §4bis-A OSS prior art : ✅ — preflight S1a documente 3 domaines (log rotation, icns generation, error handling), APPROACH-ALIGNED
- §4bis-B deps context7 : ✅ — plan §5 Research consulte liste tracing-appender 0.2, blake3 1.5, serde_jcs 0.2, icns 0.3 avec source + status

## Modified-file branch coverage (G9)
- paths.rs : `log_dir()` change → teste par `log_dir_is_under_grid_root_not_daemon_dir` + `shell_daemon_paths_are_nested_under_root` ✅
- config.rs : `ShellDaemonPaths::resolve` default case change → teste par `paths_with_no_custom_config_resolves_under_env_root` (assertion log_dir ajoutee) ✅
- validator.rs : `validate_result()` retourne tuple → 5 tests existants adaptes ✅
- http.rs : `match serde_json::to_value` Err branches (2x) — defensif, serialization de structs derives ✅ CONCERN (Err impossible en pratique)
- http.rs : 3 mutex poisoned tests — SONT des tests ✅
- launcher main.rs : `launcher_log_dir()` + `setup_tracing()` — init code, `paths::log_dir()` teste, subscriber global non unit-testable ✅ CONCERN

## Scope cuts verification
- SC-1 migration complete coordinator → 0 touche ✅
- SC-5 validator loop LiveEvents → 0 touche ✅
- SC-12 verify_chain endpoint HTTP → 0 touche ✅
- Faux positifs grep : HARDENING_ROADMAP mentionne des items roadmap dans le texte existant (compteurs update seulement), config.rs matche un commentaire VPS existant — pas d'implementation ✅

## Horizon long-terme + documentation amont
- Design doc present : ✅ kickoff D1-D5 complets pour chaque sous-item
- D1..D5 avec alternatives + rationale : ✅ (chaque Di cite Retenu + Rejete)
- Solution la plus poussee : ✅ (tracing-appender = pattern daemon, pas File::create; icns crate = reference ecosystem)
- Aucune LOC estimee au plan : ✅

## Findings

### P2-REVIEW-A-1 — launcher setup_tracing() non testable unitairement
Le refactor du launcher logging (OnceLock+lprint → tracing-appender) n'a pas de test unitaire sur `setup_tracing()` ni `launcher_log_dir()`. La fonction `paths::log_dir()` sous-jacente EST testee (+1 test paths.rs), mais le wiring launcher est couvert uniquement par le boot du binaire. Mitigation : le daemon a le meme pattern (logging.rs sans test unitaire sur `init_logging()`). Risk : faible — setup_tracing est une init one-shot, toute regression se manifeste par absence de log file.

### P2-REVIEW-A-2 — serde_json::to_value Err branch sans test
Les 2 replacements `unwrap_or_default()` → `match + 500` dans http.rs ajoutent une branche Err qui ne peut pas etre declenchee en pratique (structs 100% Serialize-derives, types primitifs). Pas de test car impossible a trigger sans mock de serde. Risk : zero fonctionnel, amelioration defensive pure.

### P3-REVIEW-A-1 — png-to-icns max taille 512px
Le crate icns 0.3 ne supporte pas 1024x1024 (IconType::RGBA32_1024x1024 absent). macOS 11+ utilise le Retina @2x rendering qui down-scale depuis 512x512. Risk register R1 du kickoff couvre ce fallback. Impact visuel negligeable.

## Recommendation
- Ready to commit : **oui**
- Carry-overs S38 : P2-REVIEW-A-1 (launcher logging test coverage, low priority)
