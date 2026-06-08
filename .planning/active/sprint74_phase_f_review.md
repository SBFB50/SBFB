# Sprint 74 Phase F — Adversarial Review

Date: 2026-06-08
HEAD base: `0854953` (Phase F preflight) on `b76a084` (Phase E)
Scope: `feat(daemon): Sprint 74 Phase F — remote seed boot re-announce +
SeedAnnounced registry (raw-op) + multi-seed counter`

Method: Workflow 5-dimension adversarial review (10 agents, ~533k tokens) —
4 dimension reviewers (correctness / security / tests / scope-arch) producing
schema'd findings, each P0/P1/P2 finding adversarially verified by an independent
agent instructed to REFUTE. `run_id wf_2bdb1f2e-0aa`.

## Verdict: PASS

0 P0, 0 P1, 0 P2 confirmed. 2 P3 confirmed (both test-quality, both ADDRESSED
in-phase). Security 0 confirmed (2 raw, both refuted). Scope-arch 0 raw.

## Raw → confirmed (after adversarial verify)

| Dimension | Raw | Confirmed | Notes |
|---|---|---|---|
| correctness | 2 | 1 (P3) | E2E identity-assertion gap (test quality) |
| security | 2 | 0 | both refuted (pilot-bounded documented residuals) |
| tests | 3 | 1 (P3) | E2E asserts count not seeder id (test quality) |
| scope-arch | 0 | 0 | spec/invariants honored, no scope creep |

## Confirmed findings (P3) — both ADDRESSED

### P3-1 (correctness) — E2E does not tie the seeder identity to node_a
The E2E generates a standalone `a_keypair` rather than deriving from node_a's
iroh identity, so it does not independently re-prove the `pow_keypair == node_id`
equivalence. **Verifier verdict: code is CORRECT** — runtime.rs:319-362 mints
`pow_keypair` from the SAME `secret_bytes` handed to `NodeConfig::with_secret_key`,
so the equivalence is structurally guaranteed and cannot be silently broken; this
is a test-clarity issue (downgrade P2→P3).
**Addressed**: the E2E already asserts `a_pub != node_id_b`; strengthened further
by P3-2's identity assertion.

### P3-2 (tests) — E2E asserted the count, not WHICH seeder
A mutation corrupting the stored `seeder_node_id` could pass on the count alone.
**Verifier verdict: PARTIAL** — the three `record_announced` gates are covered by
the unit test, so a corrupted-id mutation would fail unit tests first; this is a
defense-in-depth gap (P3).
**Addressed**: added `SeedRegistry::seeders_recent` (`#[cfg(test)]` introspection)
and asserted the recorded seeder is EXACTLY `hex(a_keypair.public_bytes())` in both
the E2E (`feed_sync.rs`) and the unit test (`seed_registry.rs`).

## Refuted findings (not defects — recorded for transparency)
- **Security: forged SeedAnnounced over-count / Sybil** — refuted as a frozen,
  documented best-effort residual: content-addressing (BLAKE3) is the truth of
  reachability, a forged announcement cannot let a node serve bytes it lacks, the
  feed PoW (16 bits) + closed pilot bound the surface. Route Phase G THREAT_MODEL
  §10/§16 note (planned carry).
- **Security: feed growth from boot re-emit** — refuted as the intended IPFS-style
  reprovide cost, pilot-bounded; documented in the commit body + Phase G note.

## Verification snapshot (pre-Codex)
- `cargo fmt --all --check` clean.
- `cargo clippy --workspace --all-targets -- -D warnings` 0 warning.
- `cargo build -p nexus-shell-daemon --release` 0 warning (dead-code check; the
  test-only `seeders_recent` is `#[cfg(test)]`, absent from the release bin).
- Targeted nextest (coordinator + daemon): 5 required tests + db getter PASS,
  incl. the 2-node E2E `remote_seeder_reannounces_after_reboot_e2e`.
- `web` Vitest targeted: AvailabilitySheet 8/8, daemon.ts 32/32.

Full dual-platform fail-fast + Codex gate run AFTER this report (process order).
