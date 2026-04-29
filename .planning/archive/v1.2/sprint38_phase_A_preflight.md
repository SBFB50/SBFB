# Sprint 38 Phase A — preflight G8

Date : 2026-04-29 | HEAD : `2cf4c8f` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, research before code — validator_loop event-driven = deepest approach vs HTTP polling. Clean.
- feedback_context7_systematic.md : context7 consulte au kickoff (iroh Watcher pattern, strsim). Clean.
- feedback_kudos_non_monetary.md : verify_chain endpoint = read-only integrity check, pas monetaire. Clean.

## Scans (all clean)
- S1a OSS prior art : event-driven validation standard (BOINC, iroh Watcher pattern, tokio broadcast channel = idiomatic Rust async). APPROACH-ALIGNED — clean
- S1b deps : 0 nouvelle dep Phase A (tokio::sync::broadcast deja dans tokio workspace dep). 0 delta — clean
- S2 historiques : 6 fichiers scannes, 0 commits DEVIATION/rejected sur les fichiers cibles. runtime.rs touche S29 (TraceProvider) et S20 (encryption) — sans conflit avec validator_loop. http.rs touche S36 (result submission) — predecesseur compatible — clean
- S3 threat model : fast-path verified. Phase A = event plumbing interne, pas de nouveau composant securite ni wire format. HARDENING_ROADMAP pas de ligne S38 specifique (sprint migration) — clean
- S4 wire format : fast-path verified. 0 fichier canonical.rs/schemas touche. verify_chain endpoint = read-only, pas de *_VERSION bump. Pre-launch policy preserved — clean

## Telemetrie preflight
- Duree totale : ~2m
- S1a : ~30s / 2 domaines (event-driven validation, tokio patterns) / finding : clean (APPROACH-ALIGNED)
- S1b : ~15s / 0 lib nouvelle / finding : clean
- S2 : ~30s / 6 fichiers scannes / finding : clean
- S3 : fast-path / ~15s
- S4 : fast-path / ~15s

## Action
Proceder code phase A.
