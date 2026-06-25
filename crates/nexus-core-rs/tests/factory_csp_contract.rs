// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 79 Phase I — compile + run the canonical Factory authoring example.
//!
//! The agent-consumable example lives, by design, at
//! `docs/factory/examples/csp_contract.rs` (next to `docs/factory/WIRING_SPEC.md`
//! so an agent reads the contract and a runnable proof side by side). `cargo
//! nextest` — the workspace's primary runner — does not execute `[[example]]`
//! targets, so we `include!` the example file into this auto-discovered
//! integration test instead. The example carries its own `#[test]` functions;
//! pulling it in here makes nextest compile AND run them.
//!
//! Effect: if the CSP contract drifts, the documented authoring rules can never
//! silently rot — a value change (a `'none'` directive added/removed, a
//! `CSS_URL_ALLOW` edit) fails this test; an API change fails the build. This is
//! the executable half of the Phase I source-ref contract; the textual half is
//! `scripts/check-factory-docs.sh` (source-ref-check).
//!
//! Mirrors the Sprint 77 Phase N sharding pattern
//! (`crates/nexus-core-rs/tests/shard_sign_verify.rs`). The example targets the
//! library API `nexus_core_rs::csp` (re-exported at `lib.rs`), NOT the Factory
//! gate: `sbfb-factory` is a binary-only crate with no lib target, so its gate
//! (`run_gate_csp_authoring`) cannot be lifted into a `use`. The runnable proof
//! is therefore on the CSP source-of-truth the gate imports, which is the right
//! anti-drift anchor.

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/factory/examples/csp_contract.rs"
));
