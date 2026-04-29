# Sprint 39 — Design Review Board (G1)

**Date** : 2026-04-29
**Scope** : D1..D5 (PiiRedactor Tier 1 part 2 + CanaryRegistry Tier 2)

## Scoring

| Decision | Verdict | Finding |
|---|---|---|
| D1 | ⚠️ | Regex phone_intl/US patterns divergent Python baseline |
| D2 | ⚠️ | Date parsing TZ ambigue Python↔Rust |
| D3 | ⚠️ | Routes canary + frost adjacentes `/api/canary/*` |
| D4 | ✅ | — |
| D5 | ✅ | — |

G4 rigor signal : 3 ⚠️ (>=1 requis pour PASS).

## Details

### D1 ⚠️ — Regex phone patterns

Les patterns regex Rust proposes divergent du Python baseline.
Python utilise un pattern flexible combinant E.164 + US
(`(?:\+?\d{1,3}[\s-]?)?(?:\(?\d{3}\)?[\s-]?)\d{3}[\s-]?\d{4}`)
vs Rust qui separe `phone_intl` et `phone_us` avec des patterns
differents. Risque faux-negatifs asymetriques entre les deux
implementations, mais acceptable pre-v1.0 (90%+ coverage).

### D2 ⚠️ — Date parsing TZ

Python codifie `date.fromisoformat()` (YYYY-MM-DD) +
`datetime.now(timezone.utc).isoformat(timespec="seconds")` (RFC
3339 abrege). Rust `time::OffsetDateTime` produit un format
potentiellement different. Desynchronisation possible de 1j si
parsing TZ ambigue → asymetrie classification fresh/aging/stale.
Pre-v1.0 (single node) acceptable, mais documenter le format.

### D3 ⚠️ — Routes canary adjacence

Les routes `/api/canary/observed`, `/api/canary/network-health`,
`/api/canary/freshness/:pubkey` coexistent avec les routes FROST
`/api/canary/frost/*` (round1/round2/aggregate/trusted-dealer) dans
le meme namespace. Pas de collision nominale, mais adjacence dans
le routing tree. Design acceptable (canary = observabilite, frost =
signing ceremony, deux aspects du meme domaine).

## Pre-launch protocol

FORMAT_VERSION = 1 (pas de bump). Day 0 : aucune violation.

## Verdict

Proceder Phase A-C. Aucun blocker.
