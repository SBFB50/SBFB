# Phase Review — Sprint 30 Phase C

## Verdict : PASS (1 P2 + 1 P3)

Rigor signal : 2 findings documentes (>=1 requis pour PASS rigoureux).

## Memory consultation (Step 1.5)

- `feedback_approach.md` : pick deepest → FROST threshold (deepest
  option vs single-key). Respecte.
- `sprint14_keyoxide_decision.md` : canary key distincte de node
  identity. N/A (canary key = separate crypto surface).
- `feedback_context7_systematic.md` : context7 tente sur frost-
  ed25519 (non indexe), fallback WebSearch. Respecte.

## Staging check (Step 1bis)

- Phase fichiers : 11 (8 modifies + 3 untracked)
  - Modified: Cargo.lock, frost.rs, mod.rs, daemon Cargo.toml,
    cli.rs, http.rs, main.rs, WARRANT_CANARY_HARDENING.md
  - New: canary.toml.sample, ceremony.rs, dkg.rs
- Planning/docs split : chore(planning) preflight fait (commit
  `aaa25cb`). WARRANT_CANARY_HARDENING.md est un livrable Phase C
  (ops runbook §4), pas un doc planning → reste dans le commit phase.
- Untracked accidentels : 0

## Suites (Step 2 — 3 blocs complets)

| Suite | Before | After | Delta | Status |
|---|---|---|---|---|
| Rust (nextest) | 856 | 864 | +8 | ✅ |
| Rust doctests | 0 fail | 0 fail | 0 | ✅ |
| Clippy | 0 | 0 | 0 | ✅ |
| Fmt | clean | clean | 0 | ✅ |
| Release build | OK | OK | — | ✅ |
| SDK (pytest) | 195 | 195 | 0 | ✅ |
| Coord (pytest) | 394+36f | 394+36f | 0 | ✅ (36f PyO3 stale baseline) |
| Gov (pytest) | 46 | 46 | 0 | ✅ |
| Frontend lint | 0 | 0 | 0 | ✅ |
| Frontend tsc | OK | OK | — | ✅ |
| Vitest | 269 | 269 | 0 | ✅ |
| Frontend build | OK | OK | — | ✅ |
| size-limit | 4/4 | 4/4 | 0 | ✅ |

## Modified-file branch coverage (Step 2bis, G9)

| File | New method/branch | LOC | Test exercising it | Status |
|---|---|---|---|---|
| `frost.rs` | `from_package()` | 3 | `dkg::tests::*` via `load_pubkey` | ✅ |
| `frost.rs` | `from_parts()` | 7 | `dkg::tests::dkg_roundtrip_canary_verifies` | ✅ |
| `http.rs` | `frost_trusted_dealer()` | ~12 | core: `dkg::tests::*` (HTTP wiring untested) | ⚠️ P2 |
| `http.rs` | `frost_round1()` | ~15 | core: `ceremony::tests::*` (HTTP wiring untested) | ⚠️ P2 |
| `http.rs` | `frost_round2()` | ~15 | core: `ceremony::tests::*` (HTTP wiring untested) | ⚠️ P2 |
| `http.rs` | `frost_aggregate()` | ~15 | core: `ceremony::tests::*` (HTTP wiring untested) | ⚠️ P2 |
| `main.rs` | `handle_frost()` | ~120 | core tested; file I/O + CLI dispatch untested | ⚠️ P3 |
| `cli.rs` | `FrostCommand` enum | type def | compilation check | ✅ |

## Delta tests reel (Step 3)

+8 Rust tests Phase C :
- `dkg::tests::dkg_generate_serialize_roundtrip`
- `dkg::tests::dkg_roundtrip_produces_valid_signature`
- `dkg::tests::dkg_roundtrip_canary_verifies`
- `dkg::tests::dkg_rejects_invalid_params`
- `ceremony::tests::ceremony_full_roundtrip_3_participants`
- `ceremony::tests::ceremony_insufficient_signers_rejected`
- `ceremony::tests::ceremony_tampered_message_detected`
- `ceremony::tests::ceremony_produces_canary_compatible_signature`

Plan annoncait +6 tests (overlap existant note en preflight). Reel
= +8 (couverture plus large : DKG serialization roundtrip + canary
compat + params validation ajoutés au-delà du plan).

## Commit body validation (Step 4)

- Format titre : ✅ `feat(sprint30): Sprint 30 Phase C — ...`
- Delta tests coherent : ✅ (+8 reel documente)
- Scope cuts honoured : ✅ (voir Step 5)
- Co-Authored-By : ✅ (a inclure)

## Research grounding (Step 4bis)

### 4bis-A — OSS prior art (G10)
- Preflight S1a : ZcashFoundation/frost + RFC 9591 + Blockstream
  ChillDKG + BIP-445. Verdict APPROACH-ALIGNED. **PASS**.

### 4bis-B — Deps/API context7
- Plan §3 : context7 arti-client + WebSearch frost-ed25519 2.1 +
  nym-sdk + iroh 0.98. frost-ed25519 non indexe context7 (fallback
  WebSearch OK). **PASS**.

## Horizon long-terme + documentation amont (Step 4ter)

- Design doc present : ✅ WARRANT_CANARY_HARDENING.md §4 updated
  with real CLI commands
- D1 avec alternatives + rationale : ✅ (4 alternatives rejetees)
- Solution la plus poussee : ✅ (FROST RFC 9591 = state-of-art
  threshold signatures)
- Aucune LOC estimee au plan : ✅ (3 matches = retrospective ou
  dans alternatives rejetees)

## Scope cuts verification (Step 5)

| Scope cut | Fichiers diff | Status |
|---|---|---|
| #4 DKG distribue → post-v1.0 | 0 (trusted dealer only) | ✅ |
| #5 Recrutement → ops post-v1.0 | 0 (code wiring only) | ✅ |
| #1 Tor transport → S31 | 0 | ✅ |
| #2 Nym mixnet → S32+ | 0 | ✅ |
| #3 TEE H100 → scope-cut | 0 | ✅ |
| #6 iroh 0.98 → sprint dedie | 0 | ✅ |
| #8 task_runner → S31 | 0 | ✅ |
| #9 §9.5 output filter → S31 | 0 | ✅ |
| #10 Full process isolation → LT | 0 | ✅ |

## Findings (rigor signal)

- **P2** : HTTP endpoint handlers FROST (`frost_trusted_dealer`,
  `frost_round1`, `frost_round2`, `frost_aggregate` dans http.rs)
  sont des thin wrappers (~12-15 LOC chacun) delegant aux fonctions
  core testees, mais le wiring HTTP (JSON request parsing, error
  response formatting) n'est pas exerce par des tests d'integration.
  Risque faible (handlers triviaux, core bien teste avec 8 tests
  couvrant DKG roundtrip + ceremony + edge cases). Carry S31 si
  juge necessaire.

- **P3** : CLI handler `handle_frost()` (~120 LOC) exerce file I/O +
  delegation core. Non couvert par tests e2e binaires (necessiterait
  temp dir + fixtures). Risque faible — le file I/O est
  straightforward et les core functions sont testees.

## Recommendation

- Ready to commit : **oui**
- Carry-overs S31 : HTTP integration tests FROST endpoints (P2)
- Corrections needed : aucune
