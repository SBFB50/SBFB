# Sprint 40 Phase B — preflight G8

Date : 2026-04-29 | HEAD : `2b6e3dd` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : "pick deepest, no band-aid, research before code" — respected (port direct module existant, pas de raccourci)
- feedback_context7_systematic.md : N/A (0 nouvelle dep, strsim/serde/time deja workspace)

## Scans (all clean)
- S1a OSS prior art : BOINC quorum validator + custom equivalence tolerance (ValidationIntro wiki). Known-answer probe + Levenshtein similarity = APPROACH-ALIGNED. BOINC uses quorum replication (= our redundancy.py) + custom validators with tolerance for equivalence. Canary input injection is the complementary known-answer spot-check technique. No OSS project found that does it better for LLM outputs.
- S1b deps : 3 libs checked (strsim 0.11, sha2 0.10, hmac 0.12) — all workspace deps, 0 new. Clean.
- S2 historiques : `canary_input` introduced S22 Phase E (`690fab3`). No DEVIATION/rejected/scope-cut on canary_input in archives. Module has been carried as S40 scope-cut since S39 kickoff. 0 conflicting decisions.
- S3 threat model : fast-path verified — port of existing Python module, no new security component. HARDENING_ROADMAP has no S40-specific prerequisite.
- S4 wire format : fast-path verified — canonical.rs not touched, all `*_VERSION` at 1, Day 0 preserved.

## Action
Proceder code phase B.
