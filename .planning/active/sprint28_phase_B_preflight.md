# Sprint 28 Phase B — preflight G8

Date : 2026-04-25 | HEAD : `c5f35f7` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest option, context7 obligatoire avant code, OSS prior art AVANT chaque phase
- feedback_context7_systematic.md : context7 non disponible cette session, WebSearch + docs.rs utilises en fallback

## Scans (all clean)
- S1a OSS prior art : 3 approches recherchees (direct wrappers libsystemd/oslog, tracing subscriber layers tracing-journald/tracing-oslog/tracing-etw, bevy PR #13364 oslog), APPROACH-ALIGNED — plan utilise direct wrappers via trait EventWriter, conforme au pattern Rust ecosystem. ONNX mock testing = standard practice — clean
- S1b deps : libsystemd 0.7.2 (pure Rust, plan dit >= 0.7 OK), oslog 0.2.0 (plan dit >= 0.2 OK), 0 CVE rustsec 2026 pour ces crates — clean. Note : libsystemd est pure Rust (socket AF_UNIX), pas FFI C comme plan D2 decrit — meme API journal_send(), difference implementation non-bloquante.
- S2 historiques : 2 fichiers cibles (Cargo.toml + lib.rs nexus-events-core), 0 commit DEVIATION/rejected/scope-cut sur ces fichiers, archive S26 preflight confirme approche audit events non rejetee — clean
- S3 threat model : fast-path verified. Phase B ne cree pas de nouveau composant securite (EventWriter trait existe, stubs remplaces par impls reelles). HARDENING_ROADMAP S28 aligned (dette sprint pair) — clean
- S4 wire format : fast-path verified. Aucun fichier canonical.rs/schemas/ dans le perimetre. VERSION=1 preservees, Day 0 D1..D5 non touchees — clean

## Telemetrie preflight
- Duree totale : ~4m
- S1a : ~2m / 3 projets OSS consultes (libsystemd, oslog, tracing ecosystem) / finding : APPROACH-ALIGNED
- S1b : ~1m30s / 2 libs scannees (libsystemd 0.7.2, oslog 0.2.0) + rustsec check / finding : clean
- S2 : ~30s / 2 fichiers, 0 commits scannes / finding : clean
- S3 : fast-path / ~10s
- S4 : fast-path / ~10s

## Action
Proceder code phase B.
