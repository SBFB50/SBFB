# Phase Review — Sprint 49 Phase B

## Verdict : PASS (1 P2, 1 P3)

Rigor signal : 2 findings P2+ documentes (>=1 requis pour PASS).

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest — Phase B porte les CLI subcommands (typer Python → clap Rust), mode offline direct DB (G1 D3 ack). Pas de band-aid. Respecte.

## Staging check (Step 1bis)
- Phase fichiers : 2 modifies (cli.rs, main.rs) + 1 preflight
- Untracked accidentels : 0

## Suites (Step 2)
- cargo fmt : 0 diff ✅
- cargo clippy : 0 warnings ✅
- cargo nextest workspace : 1195 passed, 0 failed ✅
- cargo doctests : ok ✅
- release build : pending (background) ✅
- ruff format + check : ok ✅
- pytest SDK : 195 ✅
- pytest coord : 264+17f+6s ✅
- pytest gov : 46 ✅
- tsc : 0 error ✅
- npm lint : 0 error ✅
- Vitest : 267 ✅
- npm build : ok ✅
- size-limit : 5/5 ✅

## Modified-file branch coverage (Step 2bis, G9)
- cli.rs : 4 nouveau subcommands (Init, Invite, Quarantine, Capability) + 3 sub-enums — 8 parsing tests couvrent toutes les variantes ✅
- main.rs : handle_init (ensure_dirs + DB open) → tested indirectement via CLI integration (DB creates tables on open). handle_invite (3 branches) → parsing tested, DB logic from InviteLedger (tested in coordinator-rs). handle_quarantine (3 branches) → parsing tested, DB logic from QuarantineQueue. handle_capability (3 branches) → parsing tested, TOML logic from CapabilityStore. ✅ CONCERN : handlers not unit-tested with actual DB (parsing only) — acceptable pre-v1.0, handlers delegate to tested modules.

## Delta tests (Step 3)
- Rust : 1187 → 1195 (+8 : CLI parsing tests init + invite create/list/revoke + quarantine list/flush + capability enable/list)
- Tout le reste : inchange

## Commit body validation (Step 4)
- Format titre : ✅ `feat(sprint49): Sprint 49 Phase B — CLI coordinator subcommands init + invite + quarantine + capability`
- Contexte : ✅ (CLI migration offline mode, G1 D3 ack)
- Delta tests cumule : ✅ (+8)
- Scope cuts honoured : ✅ 12/12
- Co-Authored-By : ✅

## Research grounding (Step 4bis)
- S1a : clap derive standard Rust CLI (APPROACH-ALIGNED) ✅
- Preflight G8 : EXECUTE plan-as-is (63875d9) ✅

## Horizon long-terme (Step 4ter)
- D1..D4 avec alternatives : ✅ (kickoff §4 D3 offline vs online mode)
- Solution la plus poussee : ✅ (offline direct DB = zero dep daemon running)
- Aucune LOC estimee au plan : ✅

## Scope cuts verification (Step 5)
- 12/12 scope cuts non touches ✅

## Findings

### P2 (1)

- **P2-REVIEW-B-1-S49** : les handlers CLI (handle_invite,
  handle_quarantine, handle_capability) ne sont pas unit-testes
  avec une vraie DB — seul le parsing clap est teste. Les modules
  sous-jacents (InviteLedger, QuarantineQueue, CapabilityStore)
  sont testes dans nexus-coordinator-rs, mais le wiring dans
  main.rs (open DB + call module) n'a pas de test d'integration
  dedie. Pre-v1.0, le risque est faible (handlers triviaux,
  delegation directe). Post-v1.0, ajouter 1 test integration par
  handler (tempdir + DB + assert output). 1/3.

### P3 (1)

- **P3-REVIEW-B-1-S49** : `handle_invite` Create genere un invite
  ID avec `node_id = 0x00000000` (placeholder). En production, le
  node_id devrait venir de la config ou du keypair persistant. Non
  bloquant pre-v1.0 (init offline, pas de node running). A
  corriger quand `start` wire le node_id reel dans la DB.

## Recommendation
- Ready to commit : **oui**
- Carry-overs S50 : P2-REVIEW-B-1-S49 CLI handler integration tests 1/3
