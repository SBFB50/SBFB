# Sprint 36 — Design Review Board (G1)

**Date** : 2026-04-28
**Reviewer** : agent Explore independant (session fraiche)
**Sprint** : S36 migration Rust native Phase 2 integration

## Scoring

| Decision | Source | Alt. verifiee | Coherence | Score |
|---|---|---|---|---|
| D1 | ✅ S35 recent | ⚠️ r2d2 2026-04-15 non date | ✅ Mutex sound | ⚠️ |
| D2 | ✅ S35 code | ⚠️ LiveEvents API non researche | ✅ HTTP proven | ⚠️ |
| D3 | ✅ Python LOC verifie | ✅ alternatives justifiees | ⚠️ JCS byte-identity | ⚠️ |
| D4 | ✅ P2-REVIEW-B-1 | ❌ P2-REVIEW-A-2 non trouve audit | ⚠️ HARDENING date | ❌ |
| D5 | ⚠️ CuratorRuntimeHandle partiel | ❌ iroh 0.98 LiveEvents | ✅ HTTP scope | ⚠️ |

## Findings

### ❌ D4-1 : P2-REVIEW-A-2 "double-open" source non trouvee dans audit_findings

Le reviewer n'a pas trouve P2-REVIEW-A-2 dans sprint35_audit_findings.md.
Source reelle : sprint35_phase_A_review.md + sprint35_verification.md §5
carry-over L82. Le "double-open" refere a la possibilite que
CoordinatorDb soit instancie plusieurs fois (une par handler call)
au lieu d'un singleton. Resolu par D1 (singleton DaemonHttpState).

### ❌ D4-2 : HARDENING_ROADMAP last_validated circulaire

last_validated = 2026-04-28 parait circulaire (date du kickoff S36).
Realite : S34 Phase D ET S35 se sont deroules le meme jour (2026-04-28).
Le last_validated = 2026-04-28 est la date reelle de validation S34
Phase D. Pas de circularite.

### ⚠️ D3-1 : JCS canonicalization equivalence Python ↔ Rust

Pas de test cross-platform prouvant que jcs (Python) et serde_jcs
(Rust) produisent des bytes identiques pour les memes entrees kudos.

### ⚠️ D2-1 : iroh 0.98 LiveEvents API non researche

Le claim "Doc handle pas expose" n'est pas prouve avec la doc iroh
0.98. L'API LiveEvents pourrait supporter une subscription sans
Doc handle direct.

### ⚠️ D1-1 : async-SQLite non evalue

Pas d'evaluation de rusqlite_async ou tokio-rusqlite. Mutex blocking
acceptable pour loopback single-writer mais non documente.

### ⚠️ D5-1 : Worker submission protocol non defini

Pas de design doc specifiant quand le worker choisit HTTP vs P2P.
Affecte le scope S37.
