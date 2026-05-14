# Sprint 61 — Design Review Report (G1)

**Date** : 2026-05-13
**Scope** : D1-D5 Day-0 decisions — Public Feed protocol foundation
**Reviewer** : agent Explore independant (session fraiche)

---

## Scoring

| Decision | Verdict | Signal |
|---|---|---|
| D1 — Feed operation type system | ✅ | Pattern SecurityEvent/GossipCmd verifie (enum serde exhaustive) |
| D2 — Feed storage backend | ✅ | Pattern gossip_outbox M6 + kudos_ledger verifie. ⚠️ BLOB vs TEXT encoding a clarifier Phase B |
| D3 — Hash-chain + signature | ✅ | BLAKE3 1.5 + Ed25519 + 14 domaines canonical.rs verifies. Crypto/spec DETER ok |
| D4 — Spec document | ✅ | Premier doc protocol post-v1.0. ⚠️ p2panda research (65-75%) a absorber dans spec |
| D5 — Integration BrowseAggregator | ✅ | Separation feed historique / browse live correcte. BrowseAggregator non modifie |

**Scoring global : D1 ✅, D2 ✅, D3 ✅, D4 ✅, D5 ✅.**

---

## Findings detail

### D2 ⚠️ — BLOB vs TEXT hash encoding

M9 schema (kickoff §4) declare `entry_hash BLOB NOT NULL` et
`prev_hash BLOB NOT NULL`. Mais kudos_ledger.rs stocke les hashes
comme `TEXT` hex. Phase B devra clarifier : soit BLOB (plus
compact, 32 bytes), soit TEXT hex (coherent kudos_ledger). Le choix
est d'implementation, pas d'architecture — la decision D2 (SQLite
append-only) tient.

### D4 ⚠️ — Absorption recherche p2panda

Le roadmap note "Recherche p2panda deja faite (65-75% narrative
spec dans `p2panda_public_protocol_briques.md`)". La spec Phase A
doit absorber les conclusions (operation lifecycle, replay rules,
is_open_source validation rule) ou documenter explicitement ce qui
est differe. Pas de blocker — signal de completude.

---

## Checklists DETER

### Crypto/spec [DETER]

- [x] D3 crypto cite >=1 alternative concurrente <6 mois (SHA-256
  rejete, HMAC rejete)
- [x] Source datee <2 ans (BLAKE3 en dep, Ed25519-dalek en dep,
  canonical.rs 14 domaines depuis S2+)
- [x] Reviewer ⚠️ si alternative absente : non applicable (2
  alternatives evaluees)

### Rust-first [DETER]

- [x] Toutes decisions citent precedents Rust-native in-codebase
  (SecurityEvent, gossip_outbox, kudos_ledger, canonical domains)
- [x] Gap factuel documente si alternative Rust rejetee : non
  applicable (aucune alternative Rust rejetee)
- [x] Reviewer ⚠️ si gap non documente : non applicable

---

## Recommandation

**Toutes D1-D5 : PROCEED.** 0 ❌, 2 ⚠️ (implementation detail +
completude spec). Phase A demarre directement.
