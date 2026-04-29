# Sprint 41 — Design Review Board (G1)

**Reviewer** : agent Explore independant (session fraiche).
**Date** : 2026-04-29.
**Verdict** : **3 ✅ + 2 ⚠️** (0 ❌). G1 rigor satisfait.

## Scoring

### D1 — Port direct 7 modules → ✅

Pattern port direct etabli et valide S38-S40 (10+ modules portes).
LOC counts verifies (1730 total). Alternatives rejetees avec
rationale explicite. Audit S40 PASS (0 P0/P1) confirme la
solidite de la methode.

### D2 — Schema extension CoordinatorDb → ✅

Pattern P39 singleton confirme. rusqlite 0.36 workspace dep
verifiee. CREATE TABLE IF NOT EXISTS idempotent et pre-launch
compatible. Alternatives (DB separee, diesel/refinery) rejetees
avec rationale documente.

### D3 — Background loops differees → ⚠️

Pattern coherent avec S40 (canary_input/rerun/etc. portes sans
wire-up). Alternative "loops vides au boot" correctement rejetee.

**Angle mort** : le kickoff assume que le wire-up lifecycle
(tokio::spawn, shutdown signal, graceful drain) sera
"naturellement fait" en S42-44 sans documenter la dependance
explicitement. Risque : orchestration des loops ajoutee sous
pression de temps sans spec. **Mitigation** : S42 preflight
devrait explicitement gater l'activation des loops sur le
Tier 5 completion.

**Non-bloquant** — precedent S40 confirme l'absence de risque
dead-code.

### D4 — PyO3 → direct nexus-core-rs → ✅

PyO3 functions verifiees dans nexus-core-py/src/lib.rs
(mint_invite L1019, decode_invite L1058,
build_contributor_attestation L1190). Underlying Rust confirme
dans nexus-core-rs::crypto (sign L96, verify L164). Appels
directs eliminant l'intermediaire FFI.

### D5 — Scope cuts → ✅

12 scope cuts enumeres et tracables au roadmap migration
§S41-S48. Aucun conflit avec engagements ROADMAP_COMMITMENTS.
Jalon "Python supprimable" preserve.
