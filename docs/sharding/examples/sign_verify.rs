// Runnable example — sign & verify the two signed shard-protocol envelopes.
//
// This is the canonical, source-anchored example for the agent-consumable
// wiring layer (Sprint 77 Phase N). It is lifted verbatim (signing/verifying
// bodies copied unchanged) from the unit tests `shard_plan_signature_roundtrip`
// and `run_proof_signature_roundtrip` in
// `crates/nexus-core-rs/src/shard_plan.rs`. The private `#[cfg(test)]` fixtures
// `sample_assignment` / `sample_manifest` / `sample_proof` are inlined here
// because an external consumer cannot see test-only items.
//
// Regular `//` comments (not `//!`) on purpose: this file is `include!`d into
// `crates/nexus-core-rs/tests/shard_sign_verify.rs`, so an inner doc comment
// would land mid-file and fail to compile.
//
// It is COMPILED AND EXECUTED by that integration test, which `include!`s this
// file so the workspace test runner (`cargo nextest`) runs it. Any drift in the
// signing API therefore breaks the build — the example can never rot.
//
// What it demonstrates, for an external caller (`use nexus_core_rs::…`):
//   1. Build a `ShardedSessionManifest` (the run the initiator AUTHORISES) and
//      sign it into a `ShardedSessionManifestEntry` under `DOMAIN_SHARD_PLAN_V1`;
//      `verify_signature` re-derives the canonical bytes and checks the Ed25519
//      signature.
//   2. Build a `RunProof` (what a worker EXECUTED) and sign it into a
//      `RunProofEntry` under `DOMAIN_RUN_PROOF_V1`.
//
// Authority: this file is rank-1 (a repo file). It proves the SIGNING PRIMITIVE
// round-trips. In-vivo driver-signed proof emission and the generation-piloting
// orchestrator shipped in S81 I/J (`shard_session.rs`); per-worker proof emission
// stays routed S82 (see `docs/sharding/WIRING_SPEC.md` and THREAT_MODEL §16).

use nexus_core_rs::{
    KeyPair, KvCachePolicy, RunMetrics, RunProof, RunProofEntry, ShardAssignment, ShardPlan,
    ShardRole, ShardedSessionManifest, ShardedSessionManifestEntry, SHARD_PLAN_FORMAT_VERSION,
};

// ── Inlined fixtures (verbatim from shard_plan.rs `#[cfg(test)]` module) ──

fn sample_assignment(worker: &KeyPair, start: u32, end: u32) -> ShardAssignment {
    ShardAssignment {
        worker_pubkey: worker.public_bytes(),
        layer_start: start,
        layer_end: end,
        role: ShardRole::LayerWorker,
        shard_hashes: vec![[7u8; 32]],
        kv_cache_policy: KvCachePolicy::LocalEphemeral,
        fallback_node: None,
        launch_profile_hash: [9u8; 32],
    }
}

fn sample_manifest(initiator: &KeyPair, workers: &[&KeyPair]) -> ShardedSessionManifest {
    let mut assignments = Vec::new();
    for (i, w) in workers.iter().enumerate() {
        let start = (i as u32) * 16;
        assignments.push(sample_assignment(w, start, start + 16));
    }
    ShardedSessionManifest::new(
        initiator.public_bytes(),
        "session-70b-1",
        "pilot-70b",
        1,
        ShardPlan::new(assignments),
        [1u8; 32],
        [2u8; 32],
        [3u8; 32],
    )
}

fn sample_proof(worker: &KeyPair, others: &[&KeyPair]) -> RunProof {
    RunProof::new(
        worker.public_bytes(),
        "session-70b-1",
        [1u8; 32],
        [4u8; 32],
        RunMetrics {
            ttft_ms: 1200,
            decode_milli_tokens_per_sec: 2300,
            p95_token_latency_ms: 450,
            network_rx_bytes: 1_048_576,
            network_tx_bytes: 524_288,
            worker_drop_count: 0,
        },
        others.iter().map(|k| k.public_bytes()).collect(),
    )
}

// ── The two signed-envelope round-trips (verbatim bodies) ──

#[test]
fn shard_plan_signature_roundtrip() {
    let initiator = KeyPair::generate();
    let w1 = KeyPair::generate();
    let w2 = KeyPair::generate();
    let manifest = sample_manifest(&initiator, &[&w1, &w2]);
    let entry = ShardedSessionManifestEntry::sign(manifest, &initiator).unwrap();
    entry
        .verify_signature()
        .expect("freshly signed manifest must verify");
    assert_eq!(entry.initiator, initiator.public_bytes());
    assert_eq!(entry.manifest.version, SHARD_PLAN_FORMAT_VERSION);
    assert_eq!(entry.manifest.plan.assignments.len(), 2);
}

#[test]
fn run_proof_signature_roundtrip() {
    let worker = KeyPair::generate();
    let peer = KeyPair::generate();
    let proof = sample_proof(&worker, &[&peer]);
    let entry = RunProofEntry::sign(proof, &worker).unwrap();
    entry
        .verify_signature()
        .expect("freshly signed run proof must verify");
    assert_eq!(entry.worker_pubkey, worker.public_bytes());
    assert_eq!(
        entry.proof.activation_fingerprint, [0u8; 32],
        "RunProof::new defaults the N0 fingerprint slot to zero (not provided)"
    );
}
