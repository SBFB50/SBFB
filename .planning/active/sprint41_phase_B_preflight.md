# Sprint 41 Phase B — preflight G8

Date : 2026-04-29 | HEAD : `38e1295` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest — contributor_registry/invite = patterns PKI standard
- sprint14_keyoxide_decision.md : deploy verifie Ed25519 — invite.rs signe avec meme crypto. Aligne.
- feedback_context7_systematic.md : deps deja dans workspace (toml, sha2, nexus-core-rs). N/A.

## Scans (all clean)
- S1a OSS prior art : contributor attestation = PKI/X.509 registry pattern. Invite = token-based (JWT/PASETO family). Capability = feature flags (OpenFeature/LaunchDarkly). APPROACH-ALIGNED — clean
- S1b deps : 0 nouvelle dep, nexus-core-rs/toml/sha2 stables — clean
- S2 historiques : hits S3-S4 (v1.0 era, Python originals, non-related decisions) — clean
- S3 threat model : fast-path (port 1:1 existing Python, pas nouveau composant securite) — clean
- S4 wire format : fast-path (0 _VERSION, 0 canonical.rs) — clean

## Action
Proceder code Phase B.
