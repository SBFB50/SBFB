# Sprint 43 Phase C — preflight G8

Date : 2026-04-30 | HEAD : `a766496` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest — conversion proxy→direct = eliminiation dep Python, conforme migration

## Scans (all clean)
- S1a OSS prior art : routes API thin wrappers sur modules Rust deja portes (CanaryInputManager S40, ContributorRegistry S41). Pas de domaine nouveau.
- S1b deps : 0 nouvelle dep. CanaryInputManager et ContributorRegistry deja dans nexus-coordinator-rs.
- S2 historiques : 0 commit match sur canary.py et contributor.py. 0 conflit.
- S3 threat model : fast-path verified. Routes canary existantes (observed, network-health) deja portees. Contributor verify = conversion proxy→direct (meme DB, meme logique).
- S4 wire format : fast-path verified. 0 canonical.rs/schemas touche.

## Note
3 routes canary deja portees dans http.rs (network-health, observed, freshness). Phase C ajoute inject-rate + observed-divergence. Contributor verify = conversion proxy→direct (proxy_contributor_verify → handler natif). 2 nouvelles routes contributor (list, envelope).

## Action
Proceder code phase C.
