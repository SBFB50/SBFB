# Sprint 58 — Audit plan

**Ecrit** : 2026-05-10 (Phase E Sprint 57)
**Scope attendu S58** : P2P storage iroh-docs + MANDATORY carries + stabilisation pre-v1.0

---

## Track A — Phase integrity

1. Verifier que chaque phase A-D a son preflight G8 + review dans `.planning/active/`
2. Verifier coherence delta tests cumule (verification.md §2 vs git log)
3. Verifier que le fix `a3943ed` (sandbox forms) est documente dans verification.md

## Track B — MANDATORY resolution

1. P2-S54-windows-test-cfg-unix : CLOSED `f1f26d5` — verifier §P46 dans PATTERNS.md
2. P2-S54-test-E2E-multi-noeuds : CLOSED `f1f26d5` — verifier test dans multi_daemon.rs
3. P2-STORAGE-SQLITE : CLOSED `636c87c` — verifier migration M7 dans db.rs

## Track C — Security hotfixes post-Phase C

1. 3 commits fix(security) CSP blob-serve (`4780c5a`, `8712890`, `0ee8cf4`)
2. Verifier que le CSP final est correct : sandbox restore + CORP header
3. Verifier que blob-serve dans iframe sandbox fonctionne apres les 3 fixes

## Track D — Apps SBFB quality

1. Protocol Explorer (`examples/sbfb-explorer/`) : verifier F2 liens source + F3 live status
2. Ideas Hub (`examples/sbfb-ideas/`) : verifier propose/vote/delete + degradation gracieuse
3. sbfb-bridge.js copies : verifier SHA256 identique entre `web/public/` et les 2 copies dans `examples/`
4. Verifier taille zip < 500KB (Explorer) et < 300KB (Ideas Hub)

## Track E — Carries entrants S58

| Item | Compteur | Priorite |
|---|---|---|
| P2-JITTER-SCOPE | 3/3 | **MANDATORY** |
| P2-INVITE-U16-WIRE | 3/3 | **MANDATORY** |
| P2-RETAIN-RECENT | 2/3 | carry |
| P2-A-1 rand blocker | exemption | externe |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| AppStorage replication P2P | NEW pre-v1.0 | **decision utilisateur 2026-05-10** |
| sbfb-bridge.js sync script | NEW P2 | carry S57 Phase D review |

## Track F — Research docs quality

1. Verifier que les 4 docs dans `.planning/research/` sont factuellement grounded (URLs, projet names, performance numbers)
2. Cross-reference GPU pooling research avec architecture existante (coordinator dispatch)
3. Cross-reference P2P storage research avec iroh-docs usage dans `docs.rs`
4. Cross-reference vote dispatch research avec gossip infrastructure dans `gossip.rs`
5. Cross-reference code validation research avec curator lists + FROST existants

## Track G — Roadmap coherence

1. CLAUDE.md carries S58 a jour ?
2. HARDENING_ROADMAP last_validated = S57 ?
3. SPRINT_LOG row S57 presente ?
4. Memory nexus_grid_pivot.md tip a jour ?
5. Scope cut §9 AppStorage replication : post-v1.0 dans kickoff mais pre-v1.0 dans CLAUDE.md — documenter la decision utilisateur 2026-05-10

## Track H — S58 pair obligations

1. Sprint 58 est pair → phase dette obligatoire (§6.2.1 Regle 1)
2. 2 items 3/3 MANDATORY (JITTER-SCOPE + INVITE-U16-WIRE) → Phase A
3. AppStorage replication P2P → phase(s) dediee(s) avec research grounding
