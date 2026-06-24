//! Sprint 77 Phase N — compile + run the canonical agent-facing example.
//!
//! The agent-consumable example lives, by design, at
//! `docs/sharding/examples/sign_verify.rs` (next to `WIRING_SPEC.md` so an
//! agent reads the contract and a runnable proof side by side). `cargo nextest`
//! — the workspace's primary runner — does not execute `[[example]]` targets,
//! so we `include!` the example file into this auto-discovered integration test
//! instead. The example carries its own `#[test]` functions; pulling it in here
//! makes nextest compile AND run them.
//!
//! Effect: if the signing API drifts (a renamed field, a changed signature),
//! this build fails — the documented example can never silently rot. This is
//! the executable half of the Phase N source-ref contract; the textual half is
//! `scripts/check-sharding-docs.sh` (source-ref-check).

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/sharding/examples/sign_verify.rs"
));
