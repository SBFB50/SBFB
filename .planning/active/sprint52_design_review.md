# Sprint 52 — Design Review Board (G1)

**Date** : 2026-05-02.
**Reviewer** : agent Explore independant (session fraiche).

## Scoring

| Decision | Score | Detail |
|---|---|---|
| D1 Dispatch oneshot | ✅ | Pattern idiomatique tokio confirme. http_shutdown (runtime.rs:620) utilise le meme oneshot::channel::<()>(). Alternatives bien comparees. |
| D2 Docs legacy DELETE | ✅ | Grep crates/ + web/ = 0 consommateur. Content audit : API-REFERENCE.md decrit FastAPI, DATABASE_SCHEMA.md reference nexus/db/sqlite_db.py (supprime S50). Zero risque. |
| D3 Release workflow dry-run | ⚠️ | **Cosign version gap** : release.yml pin v2.4.1, current v3.0.6 (2026-04-06). Breaking change v3 = --bundle flag. scripts/release-attest.sh a verifier Phase B. **upload-artifact** : v4 pincee, v8 courante. v4 supportee mais 4 releases en retard. Le dry-run Phase B exposera les issues. |
| D4 CLAUDE.md stale carry | ✅ | Factuel, zero ambiguite. |

## Rigor signal

Scoring : D1 ✅, D2 ✅, D3 ⚠️, D4 ✅.
G4 satisfait (1 ⚠️ sur 4, actionnable Phase B).

## Angles morts

1. **D3 cosign v2→v3** : scripts/release-attest.sh peut echouer
   si cosign v3 est installe (--bundle obligatoire). Recommande :
   soit garder pin v2.4.1 (stable, fonctionnel), soit upgrader
   en Phase B avec verification release-attest.sh.

2. **D3 upload-artifact v4→v8** : v4 still supported. v8 apporte
   non-zipped artifacts. Upgrade non-critique mais a considerer
   Phase B.
