# Sprint 53 Phase C — preflight G8

Date : 2026-05-06 | HEAD : `f5a7e5f` | Verdict : **SCOPE-CUT-CONSISTENT**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, no band-aid — edition 2024
  upgrade IS the correct deep fix (not wrapping in edition 2021)
- feedback_context7_systematic.md : N/A (no new dep/lib/API)

## Scans

### S1a — OSS prior art
Plan D4 proposed wrapping `set_var` in `unsafe {}` blocks.
Implementation revealed: workspace edition = 2021, where
`std::env::set_var` is a safe function. `unsafe {}` blocks trigger
clippy `unused_unsafe` lint → compilation failure with `-D warnings`.
The functions become actually `unsafe fn` only in **edition 2024**
(Rust 1.85+). Finding: **APPROACH-NAIVE** — wrapping in edition
2021 is incorrect; the fix is edition 2024 upgrade.

### S1b — deps
0 new deps, 0 bumps — clean.

### S2 — decisions historiques
S51 Phase B partially addressed (2/3). No conflicting decision.
The carry "unsafe set_var" was scoped as wrapping, but the real
fix is edition upgrade. Re-scoped.

### S3 — threat model
Fast-path verified — no new security component, no wire format.

### S4 — wire format
Fast-path — Phase C does not touch canonical.rs/schemas, VERSION=1
preserved, Day 0 preserved — clean.

## Finding (SCOPE-CUT-CONSISTENT)

- **P2-REVIEW-B-1-S51 re-scoped** : unsafe set_var wrapping
  impossible in edition 2021. Carry becomes "edition 2024 upgrade"
  for S54 (3/3 MANDATORY). ~70+ call sites across 17 files
  documented in this scan.

## Scope Phase C (adjusted)
- ~~unsafe set_var wrapping~~ → carry S54 as edition 2024 upgrade
- CLAUDE.md update (S53 CLOSED, carries S54, test counters)
- HARDENING_ROADMAP last_validated update
- verification.md 20+ fail-fast rows
- sprint54_audit_plan.md

## Telemetrie preflight
- Duree totale : ~15m (including failed implementation attempt)
- S1a : APPROACH-NAIVE discovered during implementation
- S1b : 10s / clean
- S2 : 30s / clean (re-scoped)
- S3 : fast-path / 20s
- S4 : fast-path / 20s

## Action
Proceder phase C sans le wrapping unsafe. Carry edition 2024
upgrade vers S54 (compteur 3/3 MANDATORY).
