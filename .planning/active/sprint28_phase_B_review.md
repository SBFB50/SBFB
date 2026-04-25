# Phase Review — Sprint 28 Phase B

## Verdict : PASS

Rigor signal : 2 findings P2 documentes (>= 1 requis pour PASS rigoureux)

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, context7 avant code — respecte (WebSearch sur libsystemd/oslog/rustsec, docs.rs API verification)
- feedback_context7_systematic.md : context7 non disponible cette session, fallback WebSearch + docs.rs — respecte (meilleur effort)

## Staging check (Step 1bis)
- Phase fichiers : 4 (Cargo.lock, nexus-events-core/Cargo.toml, nexus-events-core/src/lib.rs, wrapper.test.ts)
- Planning split : `sprint28_phase_B_preflight.md` untracked → chore(planning) AVANT phase commit
- Untracked accidentels : 0

## Suites (Step 2)
- Rust nextest : 823 → 828 (+5) all pass ✅
- Rust doctests : pass ✅
- Rust fmt + clippy : clean ✅
- Release build : nexus-shell-daemon OK ✅
- Python SDK : 195 pass ✅
- Python coord : 391 pass + 36 fail (stale PyO3 wheel, identique baseline) + 6 skip ✅
- Python gov : 46 pass ✅
- Python ruff : clean ✅
- Vitest : 264 → 268 (+4) all pass ✅
- Frontend lint : 0 errors (7 pre-existing warnings) ✅
- Frontend tsc : clean ✅
- Frontend build + size-limit : 7/7 pass ✅

## Modified-file branch coverage (Step 2bis, G9)
- `lib.rs` : `event_type_name()` (15 LOC) → tested by `event_type_name_matches_serde_tag` ✅
- `lib.rs` : `format_journal_fields()` (5 LOC) → tested by `format_journal_fields_structured` + `format_journal_fields_all_variants` ✅
- `lib.rs` : `format_oslog_message()` (4 LOC) → tested by `format_oslog_message_structured` + `format_oslog_message_all_variants` ✅
- `lib.rs` : `JournaldWriter::write_event` stub (3 LOC, cfg not-linux) → tested by `stub_writers_noop` ✅
- `lib.rs` : `OsLogWriter::write_event` stub (3 LOC, cfg not-macos) → tested by `stub_writers_noop` ✅
- `lib.rs` : `init_platform_emitter()` (7 LOC, 3 cfg-gated branches) → CONCERN : pas de test direct, mais trivial (chaque branche appelle `init_emitter(Box::new(X))` avec un writer deja teste)
- `wrapper.test.ts` : fichier test, pas de coverage check requis

## Commit body validation (Step 4)
- Format titre : `feat(sprint28): Sprint 28 Phase B — platform writers journald/oslog + ONNX CI fixture` ✅
- Delta tests coherent : +5 Rust, +4 Vitest = +9 total ✅
- Scope cuts honoured : 12 items kickoff §7 non touches ✅
- Co-Authored-By : a ajouter ✅

## Research grounding (Step 4bis)
### 4bis-A — OSS prior art
- Preflight `.planning/active/sprint28_phase_B_preflight.md` §S1a : 3 projets OSS consultes (libsystemd, oslog, tracing ecosystem bevy PR), verdict APPROACH-ALIGNED ✅
### 4bis-B — Deps/API
- Kickoff §4 D2 §Research consulte + Plan §2.1 reference G9 platform writers — `libsystemd` >= 0.7 et `oslog` >= 0.2 documentes avec API verification docs.rs ✅

## Horizon long-terme + documentation amont (Step 4ter)
- Design doc : N/A (remplacement stubs existants, pas nouveau module structurant) ✅
- D1..D5 alternatives + rationale : D2 cite 3 alternatives rejetees (direct FFI, tracing-subscriber-only, Windows Event Log direct) ✅
- Solution la plus poussee : crate wrappers natifs (libsystemd pure Rust, oslog FFI Apple) = option la plus profonde ✅
- LOC estimees : kickoff §4 D3/D4 mentionnent "~400 LOC doc" / "~200 LOC doc" — contexte prospectif Phases C/D, pas Phase B. Plan §2.1 R-S28-3 "~100 LOC" = seuil de scope-cut, pas estimation. Pas de LOC estimee pour Phase B ✅

## Scope cuts verification (Step 5)
- Nym mixnet : 0 fichier touche ✅
- MIG partitioning : 0 ✅
- D2 broker/executor : 0 ✅
- C4 task-scoped sandbox : 0 ✅
- Tor/arti/domain fronting : 0 (match `arti` dans `PartialEq` = faux positif) ✅
- GPU lockup / SQLiteSession / streaming bridge / Gate 3 : 0 ✅

## Findings

### P2-B-1 : JournaldWriter / OsLogWriter impls reelles non testees fonctionnellement

Les implementations `#[cfg(target_os = "linux")]` et `#[cfg(target_os = "macos")]` utilisent respectivement `libsystemd::logging::journal_send` et `oslog::OsLog::with_level`. Sur Windows dev, seuls les stubs sont compiles et testes. Les format helpers (`format_journal_fields`, `format_oslog_message`) sont testes cross-platform, mais le wiring final vers les APIs natives n'a pas de couverture CI.

**Mitigation** : les crates `libsystemd` (pure Rust, socket AF_UNIX) et `oslog` (FFI Apple stable) sont matures. Le code est trivial (< 10 LOC par impl). Les format helpers sont testes exhaustivement (all_variants).

**Carry S29** : CI runners Linux/macOS pour test fonctionnel.

### P2-B-2 : init_platform_emitter() sans test direct

Fonction publique 7 LOC (3 branches cfg-gated). Chaque branche appelle `init_emitter(Box::new(WriterType))` — trivial et chaque writer est teste individuellement. Pas de test qui verifie la selection automatique.

**Mitigation** : trivialite (3 lignes, chaque branche = 1 appel). OnceLock singleton empeche le multi-test dans le meme process.

## Recommendation
- Ready to commit : **oui**
- chore(planning) d'abord pour `sprint28_phase_B_preflight.md` + `sprint28_phase_B_review.md`
- Carry-overs S29 : P2-B-1 (CI Linux/macOS runners) + P2-B-2 (init_platform_emitter test, mineur)
