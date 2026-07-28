// SPDX-License-Identifier: AGPL-3.0-or-later
//! Host-authoritative HITL approval SIGNATURE (Sprint 83 Phase L1).
//!
//! The engine ([`sbfb_workflow_engine::hitl`]) owns the DIGEST binding: an
//! approval is bound to the exact effect tuple, and any mutation invalidates
//! it (keyless, pure). This module is the other half — the AUTHORITY: the HOST
//! (the daemon, sole holder of the unlocked node identity) SIGNS that binding
//! digest, proving the approval came from the trusted chrome host and not the
//! untrusted iframe. The browser shell RENDERS the tuple (WYSIWYS) and the
//! human approves; the daemon signs; a later verification (before the effect
//! runs) checks BOTH the signature (here) AND the digest binding (engine).
//!
//! ## Not a signing family (census stays at 29)
//!
//! The signature reuses the node's existing Ed25519 identity key with a LOCAL
//! context string ([`HITL_APPROVAL_SIGN_CONTEXT`]) — exactly the discipline
//! `ActivityReceipt` documents ("a LOCAL marker, not a signing domain"). It is
//! NOT a `DOMAIN_*_V1` wire family: a HITL approval is consumed on ONE node,
//! before ONE effect, and never gossiped. No census entry, no
//! `*_FORMAT_VERSION`.

use ed25519_dalek::{Signature as DalekSignature, VerifyingKey};

use crate::crypto::{KeyPair, PUBLIC_KEY_LENGTH, SIGNATURE_BYTES};
use crate::error::{NexusError, Result};

/// Context string mixed into the HITL approval signature preimage. A LOCAL
/// marker separating a HITL approval signature from any other use of the node
/// identity key; NOT a `DOMAIN_*_V1` signing family.
pub const HITL_APPROVAL_SIGN_CONTEXT: &[u8] = b"sbfb-hitl-approval-v1";

/// Field separator between the context and the digest in the preimage.
const SEP: u8 = 0x1f;

/// The exact bytes signed/verified: `context ‖ 0x1f ‖ binding_digest`. The
/// `binding_digest` is [`sbfb_workflow_engine::hitl::hitl_binding_digest`] of
/// the tuple the human approved (a hex string).
fn approval_preimage(binding_digest: &str) -> Vec<u8> {
    let digest = binding_digest.as_bytes();
    let mut msg = Vec::with_capacity(HITL_APPROVAL_SIGN_CONTEXT.len() + 1 + digest.len());
    msg.extend_from_slice(HITL_APPROVAL_SIGN_CONTEXT);
    msg.push(SEP);
    msg.extend_from_slice(digest);
    msg
}

/// Sign a HITL approval: the HOST attests, with the node identity key, that the
/// human approved the effect tuple whose binding digest is `binding_digest`.
pub fn sign_hitl_approval(keypair: &KeyPair, binding_digest: &str) -> [u8; SIGNATURE_BYTES] {
    keypair.sign(&approval_preimage(binding_digest))
}

/// Verify a HITL approval signature against the node's public key and the
/// binding digest the effect will ACTUALLY run with. A mismatch (a different
/// digest, a different key, a forged signature) fails — the effect does not
/// run.
///
/// ## The prescribed composition, and what each half then proves (ex-review r2 R3)
///
/// ONE composition is prescribed: the daemon RECOMPUTES the effect digest from
/// the tuple it is about to run and passes THAT here. Under it, the digest
/// EQUALITY inside `sbfb_workflow_engine::hitl::verify_binding` is a tautology
/// at the call-site — it compares a value to itself — so it must not be counted
/// twice as evidence. What `verify_binding` still carries, and this function
/// does not, is everything the signature cannot express: the freshness window
/// (expiry vs `now_ms`), the REFUSAL verdict, and the binding to one
/// `capability_ref`. What THIS function carries, and the engine cannot, is that
/// the approval came from the node identity at all.
///
/// Both halves are therefore load-bearing, on different properties. Calling
/// only one is a hole; describing the engine's digest check as the anti-mutation
/// guarantee under this composition would be wrong — the anti-mutation property
/// lives HERE, in a signature over the recomputed digest.
///
/// ## Why NOT `crypto::verify` here (ex-review r2 R3)
///
/// `crypto::verify` uses the permissive `verify`. Sprint 83 Phase F closed that
/// exact class on the pack path and left a committed test proving it: a
/// SMALL-ORDER (weak) public key with `R = identity, S = 0` satisfies the
/// permissive equation for EVERY message, with no private key involved — so
/// "the holder of the key signed these bytes" becomes a FALSE sentence. An
/// approval is an authority artifact; it must not inherit the weaker check
/// merely because the shared helper predates the finding. `verify_strict`
/// rejects the class, and `is_weak()` NAMES the refusal instead of leaving it
/// to the equation. `crypto::verify` is deliberately left untouched (its other
/// callers are a separate decision).
pub fn verify_hitl_approval(
    public_key: &[u8; PUBLIC_KEY_LENGTH],
    binding_digest: &str,
    signature: &[u8; SIGNATURE_BYTES],
) -> Result<()> {
    let verifying_key = VerifyingKey::from_bytes(public_key)
        .map_err(|e| NexusError::Crypto(format!("bad public key: {e}")))?;
    if verifying_key.is_weak() {
        return Err(NexusError::Crypto(
            "verifying key is a small-order (weak) point".to_string(),
        ));
    }
    let sig = DalekSignature::from_bytes(signature);
    verifying_key
        .verify_strict(&approval_preimage(binding_digest), &sig)
        .map_err(|e| NexusError::Crypto(format!("signature verification failed: {e}")))
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::Verifier;

    use super::*;

    const DIGEST: &str = "abc123def456-binding-digest-hex";

    /// GOLDEN VECTOR of the HOST half of the seam (ex-review r3 D4-1).
    ///
    /// `sign_hitl_approval` and `verify_hitl_approval` both go through
    /// [`approval_preimage`], so they agree with each other under ANY change to
    /// it: setting [`HITL_APPROVAL_SIGN_CONTEXT`] to `b"x"`, changing [`SEP`], or
    /// dropping the separator entirely left all five tests of this module — and
    /// the whole workspace — green. The engine half and the vault KEK each have a
    /// pinned vector; this side had none, while the threat model claimed both
    /// halves of the seam were pinned.
    ///
    /// Pinning the BYTES, not a signature, keeps this free of any deterministic
    /// key material: it is the preimage that must not drift, because a drift
    /// silently invalidates every stored approval.
    #[test]
    fn approval_preimage_matches_its_golden_vector() {
        let expected: Vec<u8> = [HITL_APPROVAL_SIGN_CONTEXT, &[0x1f], DIGEST.as_bytes()].concat();
        assert_eq!(
            approval_preimage(DIGEST),
            expected,
            "the signed preimage changed — context, separator or field order. If \
             this is intended, every STORED approval is invalidated; say so."
        );
        // And the context is the exact frozen string, not merely non-empty.
        assert_eq!(HITL_APPROVAL_SIGN_CONTEXT, b"sbfb-hitl-approval-v1");
    }

    #[test]
    fn sign_verify_roundtrip() {
        let kp = KeyPair::generate();
        let sig = sign_hitl_approval(&kp, DIGEST);
        assert!(verify_hitl_approval(&kp.public_bytes(), DIGEST, &sig).is_ok());
    }

    #[test]
    fn a_different_digest_is_rejected() {
        // The whole point: an approval signed over one tuple's digest does not
        // verify against a MUTATED tuple's digest — the daemon recomputes the
        // effect digest and passes THAT, so a post-approval mutation fails
        // here even before the engine's binding check.
        let kp = KeyPair::generate();
        let sig = sign_hitl_approval(&kp, DIGEST);
        assert!(verify_hitl_approval(&kp.public_bytes(), "tampered-digest", &sig).is_err());
    }

    #[test]
    fn a_different_key_is_rejected() {
        // A forged approval from a key that is not the node identity (e.g. an
        // iframe that somehow produced a signature) does not verify.
        let host = KeyPair::generate();
        let forger = KeyPair::generate();
        let sig = sign_hitl_approval(&forger, DIGEST);
        assert!(verify_hitl_approval(&host.public_bytes(), DIGEST, &sig).is_err());
    }

    /// Boundary test of the class `verify_strict` closes (ex-review r2 R3),
    /// recopying the patron `sbfb_workflow_core::pack`
    /// (`weak_key_with_forged_signature_rejected`) EXACTLY — including its
    /// sanity assertion, which is what makes the test discriminant instead of
    /// merely green: the IDENTITY point with `R = identity, S = 0` satisfies the
    /// PERMISSIVE verification equation for ANY message, with no private key
    /// involved. So the last line proves that the only thing standing between
    /// this forgery and an accepted approval is the key check.
    ///
    /// The earlier version of this test used `[0u8; 32]`, which is NOT the
    /// identity (`y = 0` is a point of order 4, not order 1), and accepted a
    /// refusal reason of "verification failed" — so re-routing this function
    /// back onto the permissive `crypto::verify` could have left it GREEN.
    #[test]
    fn a_small_order_key_cannot_forge_an_approval() {
        // The Ed25519 identity point, compressed: y = 1, sign bit 0.
        let mut weak_key = [0u8; PUBLIC_KEY_LENGTH];
        weak_key[0] = 1;
        // R = identity point, S = 0 — the classic universal forgery.
        let mut forged = [0u8; SIGNATURE_BYTES];
        forged[0] = 1;

        let err = verify_hitl_approval(&weak_key, DIGEST, &forged)
            .expect_err("a small-order key must never authorise an approval");
        let rendered = err.to_string();
        assert!(
            rendered.contains("weak"),
            "the weak-key refusal must NAME itself (not fall through to a generic \
             verification failure, which a permissive verifier could also emit), \
             got: {rendered}"
        );
        // Same key, ANY other digest: still refused (the point of the class is
        // that it would otherwise verify universally).
        assert!(verify_hitl_approval(&weak_key, "another-digest", &forged).is_err());

        // Sanity — the forgery DOES satisfy the non-strict equation, so the
        // guard is load-bearing rather than decorative. Without this line the
        // test could not tell a rejected forgery from a malformed input.
        let permissive =
            VerifyingKey::from_bytes(&weak_key).expect("the identity is a valid encoding");
        assert!(permissive.is_weak(), "the identity point must read as weak");
        assert!(
            permissive
                .verify(
                    &approval_preimage(DIGEST),
                    &DalekSignature::from_bytes(&forged)
                )
                .is_ok(),
            "sanity: the forgery satisfies the permissive equation"
        );
    }

    #[test]
    fn context_separation_binds_the_signature() {
        // The signature is over the context-prefixed preimage, so a raw
        // signature of the bare digest (no HITL context) does not verify as an
        // approval — preventing a signature minted for another purpose from
        // being replayed as a HITL approval.
        let kp = KeyPair::generate();
        let raw = kp.sign(DIGEST.as_bytes());
        assert!(verify_hitl_approval(&kp.public_bytes(), DIGEST, &raw).is_err());
    }
}
