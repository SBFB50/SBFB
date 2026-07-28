// SPDX-License-Identifier: AGPL-3.0-or-later
//! Host-authoritative HITL approval binding (Sprint 83 Phase L1) — the
//! DIGEST binding that makes "any mutation invalidates the approval" (§14.4)
//! enforceable, keyless, and testable.
//!
//! ## Division of authority (who signs vs what binds)
//!
//! [`crate::signal`] already validates a human decision's nonce (mono-usage),
//! expiry and label; it deliberately has NO field for the principal or the
//! signature — "that binding is the Phase L1 host chrome". This module is that
//! binding, split across two layers so the engine stays keyless:
//!
//! - **who signs** = the HOST. Stated in the FUTURE because none of it is wired
//!   yet (see "What this phase does NOT wire" below — the primitives exist, the
//!   path does not): the browser shell WILL RENDER the exact approval tuple
//!   (WYSIWYS — what the human sees is what is signed) and the daemon WILL SIGN
//!   its digest with the node identity key (`nexus_core_rs::hitl_approval`),
//!   under a LOCAL context string — never a new signing family, so the census
//!   stays at 29. The untrusted iframe holds no key and cannot decide: the
//!   bridge allowlist is CLOSED and contains no approval verb (16 verbs, pinned
//!   by a literal list AND by a named assertion in
//!   `web/src/bridge/__tests__/authority.test.ts`), and the frame has no
//!   `allow-same-origin`, so it cannot reach the chrome's DOM either. There is
//!   today no channel through which it could even REQUEST one — no bridge verb,
//!   no queue producer. `BLOB_SERVE_CSP` seals the frame's origin, network and
//!   sub-frames; it is not what withholds approval authority from it.
//! - **what binds** = THIS module. At effect time the engine recomputes
//!   [`hitl_binding_digest`] from the tuple the effect will ACTUALLY run with
//!   and requires it to equal the approved digest: any mutation of any field
//!   changes the digest and the approval no longer matches. This half is pure
//!   and keyless, and is falsified by `mutation_invalidates_approval`.
//!
//! ## The caller CANNOT skip the signature (ex-review P1-5/D1)
//!
//! The first cut of this module took `signed_digest: &str`, which let a caller
//! pass `effect.digest()` computed by itself — a trivial self-validation that
//! type-checked, and nothing in the signature said the host verification was
//! mandatory. Now [`verify_binding`] and [`decide_ref`] accept ONLY a
//! [`HostVerifiedApproval`], whose sole constructor is
//! [`HostVerifiedApproval::attest_verified_signature`] — named after the act
//! it presupposes. The engine still performs no crypto (it holds no key); what
//! changed is that skipping the host half is now a deliberate, visible lie at
//! the call site instead of the shortest path. The honest bound stays: this is
//! a WITNESS type, not a cryptographic proof — a host that calls `attest_*`
//! without verifying has forged its own evidence.
//!
//! ## Layer 7 of the capability intersection
//!
//! A verified, granted approval turns into a "decided ref" in
//! [`crate::capability::HitlValidity`]: layer 7 drops any capability whose
//! effective approval is [`crate::capability::ApprovalLevel::Human`] UNLESS its
//! ref is decided. The approval is bound to ONE `capability_ref`, so an
//! approval obtained for one capability cannot decide another.
//!
//! ## What this phase does NOT wire (0 call-site)
//!
//! L1 ships this primitive and its tests. NOTHING in production calls
//! [`verify_binding`], [`decide_ref`] or the host signer yet: no
//! [`crate::capability::HitlValidity`] is built outside tests, and
//! [`crate::activity::EffectContext`] has no approval field. The RELIURE — the
//! daemon verifying the signature, the effect adapter gating on the intersection,
//! the refusal journaled — is the phase that ships the run surface (same DS-1
//! posture as Phase L's `capability.rs`). Read `docs/security/THREAT_MODEL.md`
//! §19.4 for the bound, not for a promise.
//!
//! ## Pre-launch protocol / frontier
//!
//! [`HITL_BINDING_CONTEXT`] is a LOCAL context string, not a `DOMAIN_*_V1`
//! signing family: the binding digest is computed and checked on ONE node,
//! never gossiped. The census of signing families stays at 29 and no wire
//! `*_FORMAT_VERSION` is touched.
//!
//! ## Frontier — who READS [`HitlBinding`] (§6.12 test-acteur)
//!
//! The "no wire bump" test is NOT what makes something a frontier; the test is
//! "who reads this primitive?". Three distinct readers, none of them this crate:
//!
//! - **A HUMAN**, field by field, through a DIFFERENT runtime: the browser
//!   shell renders every field of this struct as the thing being approved.
//!   Rename a field and the screen silently stops showing it — while the digest
//!   (and therefore the signature) still covers it.
//! - **A TS MIRROR**, `web/src/pages/approvalsQueue.ts` (`HitlBinding`,
//!   camelCase by front-end style). Its labels are a `Record<keyof HitlRequest,
//!   string>`, so the FRONT cannot silently drop a field — but that only
//!   compares the mirror to its own TS definition.
//! - **A SHELL GATE**, the `HITL-PARITY` leg of `scripts/check-frontier-contracts.sh`,
//!   the only authority on "the mirror drifted from its source". Its scope is
//!   NAMES only: a field whose TYPE changes keeps parity green, and is caught
//!   instead by `binding_digest_matches_its_golden_vector`.
//!
//! Syntactic constraint that follows from that gate: it extracts fields by
//! matching `pub <name>:` inside `pub struct HitlBinding {`. Making the fields
//! private (a plausible answer to "the `pub` fields bypass `new`") would empty
//! the extraction and fail the leg with the named diagnostic
//! `HITL-PARITY: could not extract fields (RS=0 TS=15)` — fail-closed AND
//! diagnosable. That was not true until review round 7: the four extraction
//! assignments had no `|| true`, so under `set -euo pipefail` an empty
//! extraction aborted the script BEFORE the `-z` guard, exiting 1 with zero
//! output. Measured both ways (guard removed → 0 lines; guard present → the
//! line above), which is why this note now names the string it prints.

use crate::capability::DecidedRef;
use crate::hash::{
    HashError, canonical_digest, is_forbidden_in_identity, validate_identity_component,
};

/// Context string mixed into the HITL binding digest. Local to this crate,
/// like [`crate::hash::STEP_REQUEST_HASH_CONTEXT`]: NOT a signing domain,
/// never gossiped, so it deliberately does not follow the `DOMAIN_*_V1` naming
/// of the signed-wire families (keeping the census frozen at 29).
pub const HITL_BINDING_CONTEXT: &str = "sbfb.workflow.engine.hitl-binding.v0";

/// What the human actually answered. Part of the binding (hence of the
/// digest), because §14.4 binds the decision to "the choice granted or
/// refused" — an approval and a refusal must never share a digest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecision {
    /// The human authorised the effect.
    Granted,
    /// The human refused it. A refusal NEVER yields a decided ref.
    Refused,
}

/// The exact tuple a human approval is bound to (§14.4). EVERY field is part
/// of the digest, so mutating any one of them invalidates a prior approval —
/// which is the whole point: an approval authorises ONE effect, for ONE
/// capability, with ONE destination, ONE outgoing-data digest, at ONE moment,
/// with ONE verdict.
///
/// The cost ceiling is RECORDED and BOUND, never ENFORCED (ex-review r2 P2-13):
/// [`decide_ref`] does not receive the `BoundedCapability`, so nothing here
/// compares `max_cost` to a quota, and within its freshness window an approval
/// can be presented N times. Enforcing the ceiling and single-use is the binding
/// phase's job — see §19.4's honest bounds.
///
/// Built through [`HitlBinding::new`], the validation gate: every string
/// component is bounded and control-character-free (identity-component domain).
/// The FIELDS ARE `pub`, though, so the literal struct expression bypasses that
/// gate — the tests here use it deliberately. Treat `new` as the gate for
/// untrusted input, not as an invariant of the type.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HitlBinding {
    /// Owning run.
    pub run_id: String,
    /// Digest of the signed plan this effect belongs to — ties the approval to
    /// a specific compiled plan, so an approval cannot be replayed against a
    /// different plan that happens to share ids.
    pub plan_digest: String,
    /// Node instance the effect runs at.
    pub node_instance_id: String,
    /// Engine-derived effect id.
    pub effect_id: String,
    /// The capability ref this approval decides. Bound so an approval obtained
    /// for one capability cannot be presented for another (ex-review D2).
    pub capability_ref: String,
    /// The operation the human is approving (e.g. the verb the effect calls).
    pub operation: String,
    /// The destination the effect reaches (a fixed network destination, a repo
    /// remote, …). Mutating it after approval invalidates the decision.
    pub destination: String,
    /// Digest of the OUTGOING data (the bytes that would leave the node). The
    /// human approves sending THIS payload; changing it changes the digest.
    pub outgoing_digest: String,
    /// Effect class name (a [`crate::capability::CapabilityClass`] rendering).
    pub effect_class: String,
    /// Maximum cost the human authorises for this effect.
    pub max_cost: u64,
    /// The principal whose decision this is (who approved).
    pub principal: String,
    /// The mono-usage wait nonce this approval answers (from
    /// [`crate::signal`]). Ties the approval to ONE interruption.
    pub nonce: String,
    /// Monotone approval sequence — a replayed-but-stale approval carries an
    /// older `seq` and binds to a different digest.
    pub seq: u64,
    /// Absolute expiry (ms). An approval consumed at or after this instant is
    /// refused; the field is part of the digest, so the expiry the human saw
    /// is the expiry enforced.
    pub expires_at_ms: i64,
    /// The verdict the human gave.
    pub decision: ApprovalDecision,
}

impl HitlBinding {
    /// Build a validated HITL binding. Every string component is checked
    /// against the identity-component domain (non-empty, bounded,
    /// control-character-free).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        run_id: impl Into<String>,
        plan_digest: impl Into<String>,
        node_instance_id: impl Into<String>,
        effect_id: impl Into<String>,
        capability_ref: impl Into<String>,
        operation: impl Into<String>,
        destination: impl Into<String>,
        outgoing_digest: impl Into<String>,
        effect_class: impl Into<String>,
        max_cost: u64,
        principal: impl Into<String>,
        nonce: impl Into<String>,
        seq: u64,
        expires_at_ms: i64,
        decision: ApprovalDecision,
    ) -> Result<Self, HashError> {
        let binding = Self {
            run_id: run_id.into(),
            plan_digest: plan_digest.into(),
            node_instance_id: node_instance_id.into(),
            effect_id: effect_id.into(),
            capability_ref: capability_ref.into(),
            operation: operation.into(),
            destination: destination.into(),
            outgoing_digest: outgoing_digest.into(),
            effect_class: effect_class.into(),
            max_cost,
            principal: principal.into(),
            nonce: nonce.into(),
            seq,
            expires_at_ms,
            decision,
        };
        for (kind, value) in [
            ("run_id", &binding.run_id),
            ("plan_digest", &binding.plan_digest),
            ("node_instance_id", &binding.node_instance_id),
            ("effect_id", &binding.effect_id),
            ("capability_ref", &binding.capability_ref),
            ("operation", &binding.operation),
            ("destination", &binding.destination),
            ("outgoing_digest", &binding.outgoing_digest),
            ("effect_class", &binding.effect_class),
            ("principal", &binding.principal),
            ("nonce", &binding.nonce),
        ] {
            validate_identity_component(kind, value)?;
        }
        Ok(binding)
    }

    /// The digest a host approval signs. Injective over the tuple (JCS members
    /// are named), so two different bindings never collide.
    pub fn digest(&self) -> Result<String, HashError> {
        hitl_binding_digest(self)
    }
}

/// BLAKE3 over `HITL_BINDING_CONTEXT ‖ 0x1f ‖ jcs(binding)`, hex — the value a
/// host approval signs, and the value the engine recomputes at effect time.
pub fn hitl_binding_digest(binding: &HitlBinding) -> Result<String, HashError> {
    canonical_digest(HITL_BINDING_CONTEXT, binding)
}

/// Evidence that the HOST verified an Ed25519 signature over a binding digest
/// with the node identity key.
///
/// This type exists so the host half cannot be skipped by accident: the engine
/// is keyless and CANNOT verify a signature itself, so it demands the witness
/// instead of a bare string (ex-review P1-5/D1). Honest bound, stated on the
/// constructor: a host that attests without verifying forges its own evidence
/// — the type closes the accident, not the lie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostVerifiedApproval {
    digest: String,
}

impl HostVerifiedApproval {
    /// Attest that the host has ALREADY verified the identity signature over
    /// `binding_digest` (`nexus_core_rs::hitl_approval::verify_hitl_approval`
    /// returning `Ok`).
    ///
    /// Call this ONLY on that success path. It is the single constructor, and
    /// it is named after its precondition on purpose.
    pub fn attest_verified_signature(binding_digest: impl Into<String>) -> Self {
        Self {
            digest: binding_digest.into(),
        }
    }

    /// The digest whose signature was verified.
    pub fn digest(&self) -> &str {
        &self.digest
    }
}

/// A HITL approval refused at the binding gate.
#[derive(Debug, thiserror::Error)]
pub enum HitlError {
    /// The effect's tuple no longer matches the approved digest: a field was
    /// mutated after the human approved. Fail-closed — the effect does not
    /// run. The message carries only digests (no payload).
    #[error("HITL binding mismatch: effect digest {effect} != approved digest {approved}")]
    BindingMismatch {
        /// Digest recomputed from the effect's actual tuple.
        effect: String,
        /// Digest the host approval was signed over.
        approved: String,
    },
    /// The approval's own expiry (part of the binding) has passed.
    #[error("HITL approval expired at {expires_at_ms}, now {now_ms}")]
    Expired {
        /// Absolute expiry stamped in the binding.
        expires_at_ms: i64,
        /// Clock reading at the consumption attempt.
        now_ms: i64,
    },
    /// The human REFUSED. Never a decided ref — and never confusable with an
    /// approval, since the verdict is inside the digest.
    #[error("HITL decision is a refusal")]
    Refused,
    /// The approval decides a different capability than the one being gated.
    ///
    /// Both refs are RENDERED here, and `approved` comes from the binding, whose
    /// fields are `pub` and therefore not necessarily `new`-validated: it is
    /// sanitised at the point of rendering (ex-review r2 P2-11, sibling class) so
    /// a ref carrying a newline cannot forge a line in a line-oriented log, nor a
    /// `U+202E` forge the reading of the diagnostic (ex-review r4 L1-R4-E1).
    #[error(
        "HITL approval decides capability {}, not {requested}",
        sanitise_untrusted(approved)
    )]
    CapabilityMismatch {
        /// Capability the approval is bound to.
        approved: String,
        /// Capability the caller is trying to gate.
        requested: String,
    },
    /// The effect tuple could not be canonicalised.
    #[error(transparent)]
    Hash(#[from] HashError),
}

/// Render a capability ref that may NOT have gone through the validation gate.
///
/// `HitlBinding`'s fields are `pub`, and the upstream source of capability refs
/// is a pack's `requested_capabilities: Vec<String>` — author-controlled content
/// that no layer validates. So a ref reaching an error message can carry
/// anything the pack author typed, and it is rendered to a human or to a log.
///
/// Two families matter, and the guard must cover BOTH — this is the RENDERING
/// side of the domain [`is_forbidden_in_identity`] closes at the ENTRY side:
///
/// - a CONTROL character forges a line in a line-oriented log ("… WARN:
///   approval granted by the operator");
/// - an invisible or bidi FORMAT character forges the READING: `U+202E` makes
///   everything after it render right-to-left, so the ref an operator sees in a
///   diagnostic is not the ref the engine refused.
///
/// Guarding on `char::is_control` alone missed the second family entirely
/// (ex-review r4 L1-R4-E1): `U+202E` is `Cf`, not `Cc`, so `escape_debug` — which
/// would have escaped it correctly — was never reached. Sharing the predicate is
/// what keeps the two sides from drifting apart again.
fn sanitise_untrusted(capability_ref: &str) -> String {
    if capability_ref.chars().any(is_forbidden_in_identity) {
        return capability_ref.escape_debug().to_string();
    }
    capability_ref.to_string()
}

/// Verify a host-verified approval against the effect's ACTUAL tuple.
///
/// `approval` carries the digest whose HOST SIGNATURE was already verified
/// (see [`HostVerifiedApproval`]). This function performs the ENGINE half,
/// keyless: it recomputes the digest from `effect` — the tuple the effect will
/// actually run with — and requires:
///
/// 1. `hitl_binding_digest(effect) == approval.digest()` — any mutation of any
///    field (destination, outgoing digest, cost, verdict, …) breaks this;
/// 2. `now_ms < effect.expires_at_ms` — the approval has not expired;
/// 3. `effect.decision == Granted` — a refusal never authorises.
///
/// A refusal means the capability's ref does NOT become decided, so layer 7
/// drops the capability fail-closed.
pub fn verify_binding(
    effect: &HitlBinding,
    approval: &HostVerifiedApproval,
    now_ms: i64,
) -> Result<(), HitlError> {
    let effect_digest = hitl_binding_digest(effect)?;
    if effect_digest != approval.digest() {
        return Err(HitlError::BindingMismatch {
            effect: effect_digest,
            approved: approval.digest().to_string(),
        });
    }
    if now_ms >= effect.expires_at_ms {
        return Err(HitlError::Expired {
            expires_at_ms: effect.expires_at_ms,
            now_ms,
        });
    }
    if effect.decision != ApprovalDecision::Granted {
        return Err(HitlError::Refused);
    }
    Ok(())
}

/// Decide whether `cap_ref` may proceed past a `Human`-approval gate for the
/// effect described by `effect`, given a host-verified approval. `Ok` (a
/// [`DecidedRef`]) when the approval binds, is fresh, GRANTS, and decides THAT
/// capability; `Err` (dropped fail-closed) otherwise.
///
/// This is the only LEGITIMATE producer of [`DecidedRef`], and
/// [`crate::capability::HitlValidity::from_decided`] is the only way for another
/// CRATE to build a non-empty layer-7 set — so an out-of-crate host cannot
/// assert clearance it never obtained (ex-review r2 R2). Inside this crate the
/// constructor is reachable (`pub(crate)`), which is a scope, not a hole: see
/// [`DecidedRef`]'s own doc for what the wiring phase must not do.
pub fn decide_ref(
    effect: &HitlBinding,
    cap_ref: &str,
    approval: &HostVerifiedApproval,
    now_ms: i64,
) -> Result<DecidedRef, HitlError> {
    // The ref is part of a digest preimage and of the layer-7 key set: hold it
    // to the same domain as every other identity component (ex-review r2 P2-11).
    validate_identity_component("capability_ref", cap_ref)?;
    if effect.capability_ref != cap_ref {
        return Err(HitlError::CapabilityMismatch {
            approved: effect.capability_ref.clone(),
            requested: cap_ref.to_string(),
        });
    }
    verify_binding(effect, approval, now_ms)?;
    Ok(DecidedRef::new(cap_ref))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{
        ApprovalLevel, BoundedCapability, CapabilityClass, CapabilityGrant, CapabilityPolicy,
        Ceiling, DataPolicy, DestinationBound, EffectNeed, EffectiveCapabilities, GrantAudience,
        HitlValidity, OpenPolicy, effective_capabilities,
    };
    use std::collections::{BTreeMap, BTreeSet};

    const CAP: &str = "cloud_token";

    fn binding() -> HitlBinding {
        HitlBinding::new(
            "run-1",
            "plan-digest-abc",
            "root/call#1",
            "eff-1",
            CAP,
            "http.post",
            "https://api.example.com/v1/chat",
            "outgoing-digest-xyz",
            "NetworkFixedDestination",
            1000,
            "operator@example.com",
            "wait-nonce-1",
            1,
            10_000,
            ApprovalDecision::Granted,
        )
        .expect("valid binding")
    }

    /// The witness a VERIFIED host would produce — in one place, so the tests
    /// below read as "the host did its half".
    ///
    /// HONEST SCOPE (ex-Codex r1). This helper does NOT verify anything: it
    /// computes the digest and attests it. It cannot do otherwise — this crate
    /// is KEYLESS and does not depend on `nexus-core-rs`, so no signature check
    /// is reachable from here. An earlier comment claimed "verify the signature,
    /// THEN attest ... never fabricated from a digest the host did not check",
    /// which is exactly what the body does. That mattered: the sentence made the
    /// self-attestation path look like it was under test, when what is under
    /// test is the ENGINE half only.
    ///
    /// The residue is a real design bound, not a test artifact:
    /// `attest_verified_signature` is public and takes any string, so a caller
    /// CAN mint a witness from a digest it computed itself. The constructor is
    /// named after its precondition, which closes the OVERSIGHT; it cannot close
    /// a host that lies about having checked. Bound 2 of §19.4 says so, and the
    /// binding phase is what must call `verify_hitl_approval` first.
    fn host_attests(binding: &HitlBinding) -> HostVerifiedApproval {
        HostVerifiedApproval::attest_verified_signature(binding.digest().unwrap())
    }

    #[test]
    fn digest_is_stable_and_injective() {
        let b = binding();
        assert_eq!(b.digest().unwrap(), b.digest().unwrap());
        let mut other = b.clone();
        other.node_instance_id = "root/call#2".to_string();
        assert_ne!(b.digest().unwrap(), other.digest().unwrap());
    }

    #[test]
    fn matching_binding_verifies_while_fresh() {
        let b = binding();
        let approval = host_attests(&b);
        assert!(verify_binding(&b, &approval, 9_999).is_ok());
        assert_eq!(decide_ref(&b, CAP, &approval, 9_999).unwrap().as_str(), CAP);
    }

    #[test]
    fn expiry_is_enforced() {
        let b = binding();
        let approval = host_attests(&b);
        match verify_binding(&b, &approval, 10_000) {
            Err(HitlError::Expired { .. }) => {}
            other => panic!("expected Expired, got {other:?}"),
        }
        assert!(decide_ref(&b, CAP, &approval, 10_001).is_err());
    }

    /// A REFUSAL never authorises — and it is not confusable with an approval,
    /// because the verdict is inside the digest (ex-review P1-8).
    #[test]
    fn a_refusal_never_decides_a_ref() {
        let mut refused = binding();
        refused.decision = ApprovalDecision::Refused;
        let approval = host_attests(&refused);

        // The host signed a REFUSAL: the binding matches, but it does not grant.
        match verify_binding(&refused, &approval, 0) {
            Err(HitlError::Refused) => {}
            other => panic!("a refusal must not authorise, got {other:?}"),
        }
        assert!(decide_ref(&refused, CAP, &approval, 0).is_err());

        // And a refusal's digest differs from the same tuple granted, so an
        // approval cannot be swapped in for a refusal (or vice versa).
        let mut granted = refused.clone();
        granted.decision = ApprovalDecision::Granted;
        assert_ne!(refused.digest().unwrap(), granted.digest().unwrap());
    }

    /// An approval obtained for one capability cannot decide another
    /// (ex-review D2).
    #[test]
    fn approval_is_bound_to_one_capability() {
        let b = binding();
        let approval = host_attests(&b);
        match decide_ref(&b, "another_cap", &approval, 0) {
            Err(HitlError::CapabilityMismatch {
                approved,
                requested,
            }) => {
                assert_eq!(approved, CAP);
                assert_eq!(requested, "another_cap");
            }
            other => panic!("expected CapabilityMismatch, got {other:?}"),
        }
    }

    /// G-HITL. Mutating ANY field of the tuple after signing invalidates the
    /// approval — proven field by field.
    ///
    /// Exhaustiveness is enforced BY THE COMPILER, not by a hand-kept count:
    /// the destructuring below has no `..`, so adding a field to
    /// [`HitlBinding`] fails to compile until a mutator is added here
    /// (ex-review P1-8: an `assert_eq!(mutators.len(), 13)` froze an
    /// incomplete tuple instead of guarding it).
    #[test]
    fn mutation_invalidates_approval() {
        let approved = binding();
        let approval = host_attests(&approved);
        let now = 9_000;

        // Positive control: the un-mutated tuple verifies. Without it, a
        // `verify_binding` that ALWAYS failed would satisfy every assertion
        // below vacuously.
        assert!(
            verify_binding(&approved, &approval, now).is_ok(),
            "un-mutated tuple must verify (non-vacuity control)"
        );

        // Compile-time exhaustiveness gate: every field is named here.
        let HitlBinding {
            run_id: _,
            plan_digest: _,
            node_instance_id: _,
            effect_id: _,
            capability_ref: _,
            operation: _,
            destination: _,
            outgoing_digest: _,
            effect_class: _,
            max_cost: _,
            principal: _,
            nonce: _,
            seq: _,
            expires_at_ms: _,
            decision: _,
        } = approved.clone();

        type Mutator = fn(&mut HitlBinding);
        let mutators: Vec<(&str, Mutator)> = vec![
            ("run_id", |b| b.run_id = "run-2".to_string()),
            ("plan_digest", |b| {
                b.plan_digest = "plan-digest-def".to_string()
            }),
            ("node_instance_id", |b| {
                b.node_instance_id = "root/call#9".to_string()
            }),
            ("effect_id", |b| b.effect_id = "eff-2".to_string()),
            ("capability_ref", |b| {
                b.capability_ref = "other_cap".to_string()
            }),
            ("operation", |b| b.operation = "http.delete".to_string()),
            ("destination", |b| {
                b.destination = "https://evil.example.com/exfil".to_string()
            }),
            ("outgoing_digest", |b| {
                b.outgoing_digest = "outgoing-digest-TAMPERED".to_string()
            }),
            ("effect_class", |b| {
                b.effect_class = "NetworkOpen".to_string()
            }),
            ("max_cost", |b| b.max_cost = 1_000_000),
            ("principal", |b| {
                b.principal = "attacker@example.com".to_string()
            }),
            ("nonce", |b| b.nonce = "wait-nonce-2".to_string()),
            ("seq", |b| b.seq = 99),
            ("expires_at_ms", |b| b.expires_at_ms = 1_000_000_000),
            ("decision", |b| b.decision = ApprovalDecision::Refused),
        ];

        // EXACT power of these two controls, stated because an earlier comment
        // overstated it (ex-review r3 D2-1). The destructuring above catches a
        // field ADDED to the struct — it stops compiling. This count catches a
        // mutator REMOVED. Neither catches the third case: a field added and
        // then silenced with `new_field: _,`, which is a one-keystroke reflex.
        // That residual hole is assumed, and it is narrow: an ordinary added
        // field still goes red twice elsewhere (the golden digest vector, and the
        // HITL-PARITY leg of check-frontier-contracts). Only a `#[serde(skip)]`
        // field would slip through — and that is precisely the field for which
        // "any mutation invalidates the approval" would be false.
        assert_eq!(
            mutators.len(),
            15,
            "a mutator was removed: every field of the signed tuple needs one"
        );

        for (field, mutate) in mutators {
            let mut effect = approved.clone();
            mutate(&mut effect);
            match verify_binding(&effect, &approval, now) {
                Err(HitlError::BindingMismatch { .. }) => {}
                other => panic!("mutating {field} must invalidate the approval, got {other:?}"),
            }
            assert!(
                decide_ref(&effect, CAP, &approval, now).is_err(),
                "mutating {field} must drop the decided ref"
            );
        }
    }

    /// Layer-7 COMPOSITION (ex-review D5): the decided ref actually changes the
    /// intersection's verdict. A `Human`-approval capability survives when its
    /// ref is decided, and is DROPPED when it is not — including when the only
    /// available approval is bound to a mutated tuple.
    #[test]
    fn decided_ref_gates_layer_seven() {
        let cap = BoundedCapability {
            class: CapabilityClass::NetworkFixedDestination,
            quota: Ceiling::Unbounded,
            timeout_ms: Ceiling::Unbounded,
            destinations: DestinationBound::Any,
            approval: ApprovalLevel::Human,
            data_policy: DataPolicy::UNRESTRICTED,
        };
        let mut declared_map = BTreeMap::new();
        declared_map.insert(CAP.to_string(), cap.clone());
        let declared = EffectiveCapabilities::from_declared(declared_map);
        let closure: BTreeSet<String> = [CAP.to_string()].into_iter().collect();
        let policy: &dyn CapabilityPolicy = &OpenPolicy;

        // Layers 3 and 4 are AUTHORITY layers: an absent/empty grant audience
        // REFUSES the ref (it is not the attenuation identity). So a
        // composition test must confer the capability in BOTH audiences,
        // otherwise the ref is already gone before layer 7 is consulted — which
        // is exactly how the first version of this test failed.
        let grant_for = |audience| {
            let mut granted = BTreeMap::new();
            granted.insert(CAP.to_string(), cap.clone());
            vec![CapabilityGrant {
                audience,
                granted,
                expires_at_ms: None,
                revoked: false,
            }]
        };
        let release = grant_for(GrantAudience::Release);
        let pack = grant_for(GrantAudience::Pack);

        let compute = |hitl: &HitlValidity| {
            effective_capabilities(
                &declared,
                policy,
                &release,
                &pack,
                &closure,
                &EffectNeed::Any,
                hitl,
                0,
            )
        };

        // No decision → the `Human` capability is dropped fail-closed.
        assert!(
            compute(&HitlValidity::none()).get(CAP).is_none(),
            "a Human capability must not survive without a decision"
        );

        // A verified, granted approval for THIS ref → it survives.
        let b = binding();
        let approval = host_attests(&b);
        let decided = HitlValidity::from_decided([decide_ref(&b, CAP, &approval, 0).unwrap()]);
        assert!(
            compute(&decided).get(CAP).is_some(),
            "a decided ref must let the Human capability through"
        );

        // A MUTATED tuple yields no decided ref → dropped again.
        let mut mutated = b.clone();
        mutated.destination = "https://evil.example.com".to_string();
        let refused = HitlValidity::from_decided(decide_ref(&mutated, CAP, &approval, 0).ok());
        assert!(refused.is_empty(), "a mutated tuple must not decide");
        assert!(
            compute(&refused).get(CAP).is_none(),
            "a mutated approval must leave the capability dropped"
        );
    }

    /// A capability ref carrying a control character must be REFUSED by
    /// `decide_ref`, and must not appear raw in any diagnostic (ex-review r2
    /// P2-11 and its sibling class).
    ///
    /// The attack is upstream and real: a pack's `requested_capabilities` is
    /// author-controlled and validated for no such thing, so a signed pack can
    /// declare `"cloud_token\nWARN: approval granted by the operator"`. Echoed
    /// raw into a line-oriented log, that IS a forged log line.
    ///
    /// Bar (c) covers the OTHER family of the same class: `U+202E` is `Cf`, not
    /// `Cc`, so the guard that only knew `char::is_control` let it through
    /// untouched (ex-review r4 L1-R4-E1). Without that bar, sharing the
    /// predicate between the entry side and the rendering side is untested.
    #[test]
    fn a_capability_ref_with_a_control_character_is_refused_and_never_echoed_raw() {
        let forged = "cloud_token\nWARN: approbation accordee par l'operateur";
        let b = binding();
        let approval = host_attests(&b);

        // (a) as the REQUESTED ref: refused by the validation gate.
        let err = decide_ref(&b, forged, &approval, 0)
            .expect_err("a ref with a newline must not be accepted");
        // Pin WHICH mechanism refused. Without this the bar passes by side
        // effect: drop the `validate_identity_component` call and the flow falls
        // through to `CapabilityMismatch`, whose `{requested}` is rendered RAW —
        // so the newline assertion below would still fire, for the wrong reason
        // (ex-review r3 D2-7).
        assert!(
            matches!(err, HitlError::Hash(HashError::InvalidIdentity { .. })),
            "the refusal must come from the validation gate, not from a \
             downstream mismatch: {err:?}"
        );
        assert!(
            !err.to_string().contains('\n'),
            "the diagnostic must not carry the injected newline: {err:?}"
        );

        // (b) as the APPROVED ref reached through the `pub` fields, which bypass
        // `new` — the sibling half of the class. The mismatch branch renders the
        // approved ref, so it must be escaped there too.
        let mut tampered = binding();
        tampered.capability_ref = forged.to_string();
        let tampered_approval = host_attests(&tampered);
        let err = decide_ref(&tampered, CAP, &tampered_approval, 0)
            .expect_err("a mismatching ref must not decide");
        let rendered = err.to_string();
        assert!(
            !rendered.contains('\n'),
            "the mismatch diagnostic must escape the approved ref: {rendered}"
        );
        assert!(
            rendered.contains("\\n"),
            "the injected newline must be VISIBLE as an escape, not dropped: {rendered}"
        );

        // (c) the INVISIBLE half of the same class: a right-to-left override
        // mutates nothing and forges the READING of the diagnostic.
        let mut spoofed = binding();
        spoofed.capability_ref = "cloud_token\u{202E}gol.rekcatta//:sptth".to_string();
        let spoofed_approval = host_attests(&spoofed);
        let err = decide_ref(&spoofed, CAP, &spoofed_approval, 0)
            .expect_err("a mismatching ref must not decide");
        let rendered = err.to_string();
        assert!(
            !rendered.contains('\u{202E}'),
            "the mismatch diagnostic must escape the bidi override: {rendered}"
        );
        assert!(
            rendered.contains("\\u{202e}"),
            "the override must be VISIBLE as an escape, not dropped: {rendered}"
        );
    }

    /// `HitlBinding::new` is documented as THE validation gate of the authority
    /// tuple, and it was gated by nothing (ex-review r3 L1-R3-03): its only
    /// callers were this module's own happy-path helper and a doc line, so
    /// deleting the whole validation loop — or one entry of it — left the entire
    /// workspace green. Every adversarial test here builds the tuple through the
    /// `pub` fields, which BYPASS the gate.
    ///
    /// Asserting the `kind` is what makes this discriminant BY POSITION: a loop
    /// that validated the same field eleven times would pass a mere `is_err()`.
    #[test]
    fn new_validates_every_string_component_by_position() {
        // The 11 string positions, in the order `new` takes them.
        let positions = [
            "run_id",
            "plan_digest",
            "node_instance_id",
            "effect_id",
            "capability_ref",
            "operation",
            "destination",
            "outgoing_digest",
            "effect_class",
            "principal",
            "nonce",
        ];
        let bad = "value\nWARN: approval granted by the operator";

        for (index, kind) in positions.iter().enumerate() {
            // Build the argument list with exactly ONE poisoned position.
            let mut args: Vec<String> = vec![
                "run-1".into(),
                "plan-digest-abc".into(),
                "root/call#1".into(),
                "eff-1".into(),
                CAP.into(),
                "http.post".into(),
                "https://api.example.com/v1/chat".into(),
                "outgoing-digest-xyz".into(),
                "NetworkFixedDestination".into(),
                "operator@example.com".into(),
                "wait-nonce-1".into(),
            ];
            args[index] = bad.to_string();

            let err = HitlBinding::new(
                &args[0],
                &args[1],
                &args[2],
                &args[3],
                &args[4],
                &args[5],
                &args[6],
                &args[7],
                &args[8],
                1000,
                &args[9],
                &args[10],
                1,
                10_000,
                ApprovalDecision::Granted,
            )
            .expect_err(&format!("position {kind} must be validated"));

            match err {
                HashError::InvalidIdentity { kind: reported, .. } => assert_eq!(
                    &reported, kind,
                    "the refusal must name the OFFENDING position, not another one"
                ),
                other => panic!("expected InvalidIdentity for {kind}, got {other:?}"),
            }
        }

        // Non-vacuity: the same call with every position clean SUCCEEDS.
        binding();
    }

    /// GOLDEN VECTOR for the binding digest: the only control that pins the
    /// PREIMAGE rather than a self-consistent recomputation.
    ///
    /// Without it, `HITL_BINDING_CONTEXT` could be set to `"x"`, or the field
    /// separator could be removed from `canonical_digest`, and every other test
    /// here would stay green — they all compute the digest with the same code
    /// they check it against. Since this digest is what the HOST SIGNS, a silent
    /// change of preimage silently invalidates every stored approval.
    #[test]
    fn binding_digest_matches_its_golden_vector() {
        assert_eq!(
            binding().digest().expect("digest"),
            "ec923fb5a86a0e0d317d88164e19fba7b1e1ba82f88bbd226cd7bdce3ba129c6",
            "the binding digest preimage changed — if intended, re-mint the \
             vector AND note that every stored approval is invalidated"
        );
    }

    /// TWIN parity with `nexus_core_rs::vault::validate_ref`
    /// (`SECRET_REF_MAX_LEN`), pinned symmetrically on the vault side. The two
    /// crates do not depend on one another, so nothing but these two tests keeps
    /// the seam's domain single: the vault refuses a name this side accepts and
    /// the pack would name a slot the vault cannot store.
    #[test]
    fn identity_component_domain_is_the_twin_of_the_vaults() {
        assert_eq!(
            crate::hash::IDENTITY_MAX_LEN,
            512,
            "twin of nexus_core_rs::vault::SECRET_REF_MAX_LEN"
        );
        assert!(validate_identity_component("ref", "").is_err());
        assert!(
            validate_identity_component("ref", &"a".repeat(crate::hash::IDENTITY_MAX_LEN + 1))
                .is_err()
        );
        for bad in [
            "with\nnewline",
            "with\u{1f}separator",
            "with\u{0}nul",
            // Invisible / bidi FORMAT characters (ex-review r3 L1-R3-01): a
            // `U+202E` in a rendered field spoofs the READING of a tuple that is
            // byte-for-byte unmutated, so every downstream control stays green.
            "with\u{202E}override", // = borne HAUTE de `202A..202E`
            "with\u{200B}zero-width",
            "with\u{2066}isolate",
            "with\u{FEFF}bom",
            // One witness per range added in ex-review r4 L1-R4-E2, so the
            // predicate stays a SUPERSET of `output_filter::is_invisible_char`.
            // `U+2060` is the one that mattered: it fell in the gap between
            // `202E` and `2066`, i.e. between two ranges that were already here.
            "with\u{2060}word-joiner",
            "with\u{00AD}soft-hyphen",
            "with\u{2028}line-separator",
            "with\u{E000}private-use",
            "with\u{E0041}tag-smuggling",
            "with\u{F0000}supplementary-private-use",
            // Plane 16. Without it the range is UNWITNESSED on both twins:
            // removing it from either predicate left this test and the vault
            // twin GREEN (mutation measured, review r5 L1-R5-D1-03).
            "with\u{100000}plane16-private-use",
            // BORNES des plages multi-points. Un seul temoin ne prouve que SON
            // point : mesure Codex r1 — reduire `200B..200F` a `200B` laissait
            // la suite entiere VERTE. Le predicat compte NEUF plages
            // multi-points ; chacune porte desormais son point BAS et son point
            // HAUT, ci-dessous ou plus haut dans cette liste. Enumeration tenue
            // a jour avec `hash.rs::is_forbidden_in_identity` — ajouter une
            // plage sans ses deux bornes rouvre la classe (review r7).
            //   200B..200F : bas `200B` (plus haut), haut ici
            "with\u{200F}right-to-left-mark",
            //   202A..202E : haut `202E` (plus haut), bas ici
            "with\u{202A}left-to-right-embedding",
            //   2028..2029 : bas `2028` (plus haut), haut ici
            "with\u{2029}paragraph-separator",
            //   2060..2064 : bas `2060` (plus haut), haut ici
            "with\u{2064}invisible-plus",
            //   2066..2069 : bas `2066` (plus haut), haut ici
            "with\u{2069}pop-directional-isolate",
            //   E000..F8FF : bas `E000` (plus haut), haut ici
            "with\u{F8FF}private-use-end",
            //   F0000..FFFFD : bas `F0000` (plus haut), haut ici
            "with\u{FFFFD}supplementary-private-use-end",
            //   100000..10FFFD : bas `100000` (plus haut), haut ici
            "with\u{10FFFD}plane16-private-use-end",
            //   E0020..E007F : `E0041` (plus haut) est au MILIEU, donc les DEUX
            //   bornes manquaient — c'est la plage du smuggling ASCII invisible
            "with\u{E0020}tag-space",
            "with\u{E007F}cancel-tag",
            // C0 et C1 : `is_control()` couvre la categorie Cc ENTIERE, pas
            // seulement l'ASCII bas. Sans ces deux temoins, remplacer
            // `c.is_control()` par une comparaison `c < ' '` passait.
            "with\ttab",
            "with\u{0085}next-line",
            // Bornes de la categorie Cc, les DEUX blocs : `007F` DEL ferme C0,
            // `0080`..`009F` est C1 en entier. Sans eux, restreindre
            // `is_control()` a `c < ' '` — la forme la plus naturelle d'une
            // reecriture — passait sur DEL et sur tout C1.
            "with\u{7F}delete",
            "with\u{0080}c1-start",
            "with\u{009F}c1-end",
        ] {
            assert!(
                validate_identity_component("ref", bad).is_err(),
                "{bad:?} must be refused"
            );
        }
        validate_identity_component("ref", &"a".repeat(crate::hash::IDENTITY_MAX_LEN)).unwrap();
        validate_identity_component("ref", "cloud_provider_token").unwrap();
        // ACCEPTES, et c'est la moitie qui manquait : le domaine n'est PAS
        // borne a l'ASCII. Un identifiant non-ASCII VISIBLE passe — c'est
        // exactement ce qui laisse ouvert le spoof d'affichage par homoglyphes
        // (borne 24 de §19.4). Sans ces temoins, restreindre le predicat a
        // `c.is_ascii()` — un durcissement plausible — passait sans un rouge,
        // en cassant silencieusement tout identifiant non anglophone.
        // Liste byte-identique au jumeau `nexus-core-rs/src/vault.rs`.
        validate_identity_component("ref", "caf\u{00E9}_token").unwrap();
        validate_identity_component("ref", "\u{65E5}\u{672C}_token").unwrap();
    }
}
