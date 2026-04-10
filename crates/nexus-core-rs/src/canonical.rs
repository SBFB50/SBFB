//! Canonical byte serialization for signing and verifying.
//!
//! All signed payloads in the nexus-grid system go through
//! [`canonical_bytes`], which produces RFC 8785 JCS output with a
//! type-specific **domain separation prefix**.
//!
//! # Why JCS?
//!
//! The Sprint 2 audit identified a cross-language footgun: Rust's
//! `serde_json::to_vec` emits struct fields in declaration order,
//! while Python's `json.dumps(sort_keys=True)` emits them
//! alphabetically. A Python coordinator and a Rust worker signing
//! the same struct would disagree on the wire bytes and every
//! signature would silently fail to verify.
//!
//! [RFC 8785 JSON Canonicalization Scheme (JCS)][rfc] fixes this
//! by construction: it standardizes key ordering (lexicographic at
//! every nesting level), number formatting, string escaping, and
//! whitespace. A JCS-compliant implementation in any language
//! produces byte-identical output for the same input.
//!
//! Rust side: [`serde_jcs`] crate.
//! Python side: `jcs` PyPI package.
//!
//! [rfc]: https://datatracker.ietf.org/doc/html/rfc8785
//! [`serde_jcs`]: https://docs.rs/serde_jcs
//!
//! # Why a domain prefix?
//!
//! Without domain separation, a valid signature over
//! `canonical_bytes(&task)` would also be a valid signature over
//! the same byte string interpreted as a [`Claim`] or a
//! [`ResultPayload`] — if a hypothetical future schema extension
//! makes two types coincidentally serialize to similar JSON, a
//! signed message of one type could be replayed as the other.
//!
//! To prevent this we prepend a type-tagged prefix followed by a
//! null byte:
//!
//! ```text
//! <domain bytes> <0x00> <serde_jcs::to_vec(value)>
//! ```
//!
//! The `0x00` separator is a hard boundary so a crafted value
//! cannot smuggle the domain into its own JSON payload.
//!
//! Each struct family has its own constant:
//!
//! - [`DOMAIN_TASK_V1`]   — [`crate::task::Task`]
//! - [`DOMAIN_RESULT_V1`] — [`crate::task::ResultPayload`]
//! - [`DOMAIN_CLAIM_V1`]  — [`crate::task::Claim`]
//! - [`DOMAIN_INVITE_V1`] — invite payloads in
//!   `nexus_worker_core::invite::InvitePayload`
//! - [`DOMAIN_KUDOS_V1`]  — kudos ledger entries in the Python
//!   coordinator (reserved here so both sides agree on the tag)
//!
//! The `v1` suffix is the domain version, independent from any
//! struct version field. Bumping it changes the signature surface
//! and invalidates every existing signature — reserved for hard
//! schema migrations.

use serde::Serialize;

use crate::error::{NexusError, Result};

/// Domain separation tag for [`crate::task::Task`] canonical bytes.
pub const DOMAIN_TASK_V1: &[u8] = b"nexus-task-v1";

/// Domain separation tag for [`crate::task::ResultPayload`] canonical bytes.
pub const DOMAIN_RESULT_V1: &[u8] = b"nexus-result-v1";

/// Domain separation tag for [`crate::task::Claim`] canonical bytes.
pub const DOMAIN_CLAIM_V1: &[u8] = b"nexus-claim-v1";

/// Domain separation tag for invite payloads signed by the
/// coordinator (the struct lives in `nexus-worker-core`).
pub const DOMAIN_INVITE_V1: &[u8] = b"nexus-invite-v1";

/// Domain separation tag for kudos ledger entries. The entries
/// themselves live in the Python coordinator (SQLite), but the tag
/// is defined here so both sides use the same bytes.
pub const DOMAIN_KUDOS_V1: &[u8] = b"nexus-kudos-v1";

/// Produce the canonical byte representation of any serializable
/// value for signing.
///
/// The output is
///
/// ```text
/// <domain> <0x00> <serde_jcs::to_vec(value)>
/// ```
///
/// which is deterministic, cross-language reproducible (RFC 8785),
/// and type-tagged so a signature valid for one domain cannot be
/// replayed as a signature for another.
///
/// # Errors
///
/// Returns [`NexusError::Other`] only if `serde_jcs` fails to
/// serialize `value` — in practice this only happens if `T`'s
/// `Serialize` impl rejects the input (e.g. a `HashMap` with
/// non-string keys), which none of the SBFB payload types do.
pub fn canonical_bytes<T: Serialize + ?Sized>(value: &T, domain: &[u8]) -> Result<Vec<u8>> {
    let body = serde_jcs::to_vec(value)
        .map_err(|e| NexusError::Other(format!("canonical JCS serialization failed: {e}")))?;
    let mut out = Vec::with_capacity(domain.len() + 1 + body.len());
    out.extend_from_slice(domain);
    out.push(0);
    out.extend_from_slice(&body);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use std::collections::BTreeMap;

    #[derive(Serialize)]
    struct Sample {
        b: u32,
        a: String,
        c: BTreeMap<String, String>,
    }

    fn sample() -> Sample {
        let mut c = BTreeMap::new();
        c.insert("z".into(), "1".into());
        c.insert("a".into(), "2".into());
        Sample {
            b: 42,
            a: "hello".into(),
            c,
        }
    }

    #[test]
    fn canonical_bytes_starts_with_domain_prefix_and_null() {
        let out = canonical_bytes(&sample(), DOMAIN_TASK_V1).unwrap();
        assert!(out.starts_with(DOMAIN_TASK_V1));
        assert_eq!(out[DOMAIN_TASK_V1.len()], 0);
    }

    #[test]
    fn canonical_bytes_json_body_is_lexicographically_ordered() {
        // JCS sorts keys at every level, regardless of struct
        // declaration order or BTreeMap insertion order.
        let out = canonical_bytes(&sample(), DOMAIN_TASK_V1).unwrap();
        let body = &out[DOMAIN_TASK_V1.len() + 1..];
        let text = std::str::from_utf8(body).unwrap();
        // "a" appears before "b" which appears before "c"; inside
        // the c object "a" appears before "z".
        let idx_a = text.find("\"a\"").unwrap();
        let idx_b = text.find("\"b\"").unwrap();
        let idx_c = text.find("\"c\"").unwrap();
        assert!(idx_a < idx_b && idx_b < idx_c, "got: {text}");
        let idx_ca = text.find("\"a\":\"2\"").unwrap();
        let idx_cz = text.find("\"z\":\"1\"").unwrap();
        assert!(idx_ca < idx_cz, "got: {text}");
    }

    #[test]
    fn different_domains_yield_different_bytes_for_same_value() {
        let as_task = canonical_bytes(&sample(), DOMAIN_TASK_V1).unwrap();
        let as_result = canonical_bytes(&sample(), DOMAIN_RESULT_V1).unwrap();
        assert_ne!(
            as_task, as_result,
            "domain separation must produce distinct byte strings"
        );
    }

    #[test]
    fn canonical_bytes_is_deterministic_across_calls() {
        let a = canonical_bytes(&sample(), DOMAIN_TASK_V1).unwrap();
        let b = canonical_bytes(&sample(), DOMAIN_TASK_V1).unwrap();
        assert_eq!(a, b);
    }
}
