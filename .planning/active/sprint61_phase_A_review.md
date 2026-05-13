# Phase Review — Sprint 61 Phase A

## Verdict : PASS

Rigor signal : 1 finding P2 + 1 finding P3 documentes (>=1 requis
pour PASS rigoureux G4).

## Memory consultation (Step 1.5)

- feedback_approach.md : pick deepest, research before code —
  respecte (spec ecrite, OSS prior art 3 projets dans preflight)
- feedback_context7_systematic.md : N/A (pas de nouvelle dep)
- Tensions plan vs memory : aucune

## Staging check (Step 1bis)

- Phase fichiers : 5 (canonical.rs, lib.rs x2, public_feed.rs NEW,
  PUBLIC_FEED_SPEC.md NEW)
- Planning/docs split : N/A (pas de planning file dans le staging)
- Untracked accidentels : 0

## Suites

| Suite | Avant | Apres | Delta | Status |
|---|---|---|---|---|
| Rust nextest | 1259 | 1262 | +3 | pass |
| Rust doctests | 6 (1 ignored) | 6 (1 ignored) | +0 | pass |
| Vitest | 258 | 258 | +0 | pass |
| size-limit | 6/6 | 6/6 | = | pass |
| cargo fmt | 0 diff | 0 diff | = | pass |
| cargo clippy | 0 warnings | 0 warnings | = | pass |

## Commit body validation

- Format titre : `feat(feed): Sprint 61 Phase A — spec executable
  + types PublicFeedOperation` — conforme
- Delta tests coherent : +3 Rust match (plan predit +3, reel +3)
- Scope cuts honoured : 12/12 (pas de code sync P2P, pas d'endpoint
  HTTP, pas de bridge method, pas de UI)
- Co-Authored-By : a inclure

## Modified-file branch coverage (Step 2bis, G9)

- `canonical.rs` : +1 constante `DOMAIN_FEED_V1` — exercee par
  `test_canonical_bytes_feed_deterministic` via
  `compute_feed_canonical_bytes()` qui appelle
  `canonical_bytes(_, DOMAIN_FEED_V1)`. PASS.
- `lib.rs` (core-rs) : re-export only, no logic. PASS.
- `lib.rs` (coordinator-rs) : `pub mod` only, no logic. PASS.
- `public_feed.rs` : NEW file (pas existing modifie). 3 fonctions
  publiques + 3 tests. Toutes couvertes. PASS.
- `PUBLIC_FEED_SPEC.md` : doc file, no coverage needed. PASS.

## Research grounding (Step 4bis)

### 4bis-A — OSS prior art (G10)

- Preflight S1a present : oui
  (`sprint61_phase_A_preflight.md §S1a`)
- Projets consultes : SSB, Certificate Transparency, p2panda (3)
- Verdict preflight : APPROACH-ALIGNED
- PASS

### 4bis-B — Deps/API context7

- Plan §3 Research consulte : 7 sources listees (kudos_ledger,
  canonical.rs, db.rs, browse.rs, p2panda research, roadmap,
  gossip_outbox)
- 0 dep nouvelle ajoutee
- PASS

## Horizon long-terme + documentation amont (Step 4ter)

- Design doc present : `docs/protocol/PUBLIC_FEED_SPEC.md` (8
  sections, > 1 sprint lifetime). PASS.
- D1..D5 avec alternatives + rationale : oui (kickoff §4, chaque
  D cite 2-3 alternatives rejetees). PASS.
- Solution la plus poussee : BLAKE3 + Ed25519 + JCS = patterns
  existants audites dans le codebase. PASS.
- Aucune LOC estimee au plan : grep clean. PASS.

## Scope cuts verification

| Scope cut | Fichiers diff | Status |
|---|---|---|
| Sync P2P durable | 0 | pass |
| Anti-spam feed | 0 | pass |
| CuratorVouched | 0 | pass |
| BuildQuorumReached | 0 | pass |
| Endpoint HTTP verify | 0 | pass |
| Bridge methods | 0 | pass |
| UI proof-chain | 0 | pass |
| Tests adversariaux | 0 | pass |
| Go-live public | 0 | pass |
| AppImage Linux | 0 | pass |
| Interop externe | 0 | pass |
| Audit tiers | 0 | pass |

12/12 scope cuts respectes.

## Findings

### P2 — Kickoff D3 genesis convention inconsistente

Kickoff §4 D3 ecrit `prev_hash = [0u8; 32]` (zeros bytes) pour
genesis. Le code utilise `GENESIS_PREV_HASH = "genesis"` (string
literal, aligne avec kudos_ledger.rs pattern). La spec
PUBLIC_FEED_SPEC.md §4 documente correctement la convention
string. L'inconsistance est dans le kickoff (non-editable,
snapshot fige), pas dans le code ni la spec. Carry-over :
l'audit S62 devra noter que le kickoff D3 mentionne zeros mais
le pattern reel est string "genesis" — la spec fait foi.

### P3 — `provenance_hash` optionnel sans doc runtime tolerance

`ReleasePublishedPayload::provenance_hash` est `Option<String>`
avec `#[serde(default)]`. Legitime (projets sans SLSA provenance
deserializent a None au lieu de parse error). Le champ porte la
doc comment `is_open_source` validation rule dans la spec §2.1
mais pas un rationale inline `#[serde(default)]` dans le code.
Nit cosmetic — le rationale est dans la spec.

## Recommendation

- Ready to commit : oui
- Carry-overs S62 : P2 kickoff D3 genesis convention (informatif
  pour auditeur)
- Corrections needed : aucune
