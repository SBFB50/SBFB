// SPDX-License-Identifier: AGPL-3.0-or-later
//! Canonical digests: JCS (RFC 8785) bytes hashed with BLAKE3 under a
//! context string.
//!
//! Two digests carry the spike's guarantees:
//!
//! - the **step request hash**, journaled when a step is first executed and
//!   re-computed on replay. A divergence means the replayed run is asking a
//!   different question than the one whose answer was memoised, so the run
//!   fails closed instead of returning a stale answer as if it were fresh.
//! - the **effect id**, DERIVED from `(run_id, node_instance_id,
//!   request_hash)` engine-side. It is never taken from caller-supplied
//!   data: an idempotency key an untrusted author could choose would let a
//!   pack collide with, or replay, another run's effect.
//!
//! Canonicalisation is `serde_jcs`, never `serde_json::to_vec` — see the
//! rationale on the dependency in `Cargo.toml`. JCS sorts OBJECT MEMBERS at
//! serialization time; it does NOT sort arrays, so ordered collections stay
//! semantically ordered while map key order becomes irrelevant.

use std::collections::BTreeMap;

use serde::Serialize;

/// Context string mixed into every step-request digest. Local to this
/// crate: not a signing domain, never gossiped, so it deliberately does not
/// follow the `DOMAIN_*_V1` naming of the signed-wire families.
pub const STEP_REQUEST_HASH_CONTEXT: &str = "sbfb.workflow.engine.step-request.v0";

/// Context string mixed into every derived effect id.
pub const EFFECT_ID_CONTEXT: &str = "sbfb.workflow.engine.effect-id.v0";

/// Context string mixed into the receipt digest journaled for a completed
/// effect. Only the digest reaches the journal; the receipt itself stays in
/// the outbox row.
pub const RECEIPT_DIGEST_CONTEXT: &str = "sbfb.workflow.engine.receipt.v0";

/// Separator between the context string and the canonical bytes. Mirrors the
/// `0x1f` field separator already used by the seed-registry sampling key so
/// a context can never be confused with the payload it prefixes.
const CONTEXT_SEPARATOR: u8 = 0x1f;

/// A value could not be canonicalised.
#[derive(Debug, thiserror::Error)]
pub enum HashError {
    /// The serializer refused the value.
    #[error("canonicalisation failed: {0}")]
    Canonicalisation(String),
    /// The value canonicalises to a JSON `null`, which this crate refuses to
    /// hash. See [`canonical_digest`] for why.
    #[error("value at {path} canonicalises to null, which is not hashable here")]
    NonRepresentable {
        /// JSON path of the offending member.
        path: String,
    },
    /// An identity component is outside the accepted domain. See
    /// [`validate_identity_component`].
    #[error("invalid {kind}: {reason}")]
    InvalidIdentity {
        /// Which component (`run_id`, `node_instance_id`, `target`, …).
        kind: String,
        /// Why it was refused.
        reason: String,
    },
}

/// Longest accepted identity component. Bounded so a pathological id cannot
/// bloat every journal row and every side-output line.
pub const IDENTITY_MAX_LEN: usize = 512;

/// Validate one identity component (`run_id`, `node_instance_id`).
///
/// ## Why this exists (it is not defensive decoration)
///
/// Identity components are concatenated into two flat encodings: the digest
/// preimage of [`derive_effect_id`], and the `key\treceipt` line an effect
/// target writes to deduplicate. Both were originally documented as safe "by
/// construction (run and node ids are ASCII paths)" — an invariant NOTHING
/// enforced, on `String` fields anyone can fill. Measured consequences of
/// leaving it unenforced:
///
/// - a TAB in `run_id` splits the side-output line early, so the key read
///   back never matches the key written: deduplication silently stops working
///   and the counter meant to detect the double effect reports zero;
/// - a LINE FEED makes one write produce two physical lines, which either
///   forges a receipt for ANOTHER run's key (the engine then journals a
///   success that never happened) or corrupts the log for EVERY run;
/// - a `0x1f` moves the boundary between components, so
///   `("a\u{1f}b", "c")` and `("a", "b\u{1f}c")` collapse to the same key.
///
/// The component domain is therefore CHECKED, not assumed. Control characters
/// are the only thing refused: ids stay free-form paths otherwise.
pub fn validate_identity_component(kind: &str, value: &str) -> Result<(), HashError> {
    if value.is_empty() {
        return Err(HashError::InvalidIdentity {
            kind: kind.to_string(),
            reason: "empty".to_string(),
        });
    }
    if value.len() > IDENTITY_MAX_LEN {
        return Err(HashError::InvalidIdentity {
            kind: kind.to_string(),
            reason: format!("longer than {IDENTITY_MAX_LEN} bytes"),
        });
    }
    if let Some(offending) = value.chars().find(|c| c.is_control()) {
        return Err(HashError::InvalidIdentity {
            kind: kind.to_string(),
            reason: format!("contains control character U+{:04X}", offending as u32),
        });
    }
    Ok(())
}

/// The deterministic input of one step.
///
/// `BTreeMap<String, String>` by TYPE, which rules out what a TYPE can rule
/// out: no floating-point value (so the `null` collapse described on
/// [`canonical_digest`] cannot arise here) and no unordered map. It does NOT
/// rule out a caller putting a formatted clock or an absolute path in a
/// `String` — that stays the caller's responsibility, and saying otherwise
/// would claim a guarantee the type does not provide.
///
/// Build it with [`StepRequest::new`]: the constructor is the validation gate
/// for the identity components.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StepRequest {
    /// Full instance path of the node, e.g. `root/echo#1`. Identity of the
    /// memoisation slot — never an ordinal counter, so inserting a step
    /// cannot silently re-point an existing memoised answer.
    node_instance_id: String,
    /// Name of the effect target the step addresses.
    target: String,
    /// Ordered input pairs.
    input: BTreeMap<String, String>,
}

impl StepRequest {
    /// Build a validated step request.
    pub fn new(
        node_instance_id: impl Into<String>,
        target: impl Into<String>,
        input: BTreeMap<String, String>,
    ) -> Result<Self, HashError> {
        let node_instance_id = node_instance_id.into();
        let target = target.into();
        validate_identity_component("node_instance_id", &node_instance_id)?;
        validate_identity_component("target", &target)?;
        Ok(Self {
            node_instance_id,
            target,
            input,
        })
    }

    /// Node instance this request addresses.
    pub fn node_instance_id(&self) -> &str {
        &self.node_instance_id
    }

    /// Target name this request addresses.
    pub fn target(&self) -> &str {
        &self.target
    }
}

/// The idempotency key of one effect: `(run_id, node_instance_id, effect_id)`.
///
/// Build it with [`IdempotencyKey::new`]: the constructor is the validation
/// gate that keeps [`IdempotencyKey::as_line`] unambiguous.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdempotencyKey {
    run_id: String,
    node_instance_id: String,
    effect_id: String,
}

impl IdempotencyKey {
    /// Build a validated idempotency key.
    pub fn new(
        run_id: impl Into<String>,
        node_instance_id: impl Into<String>,
        effect_id: impl Into<String>,
    ) -> Result<Self, HashError> {
        let run_id = run_id.into();
        let node_instance_id = node_instance_id.into();
        let effect_id = effect_id.into();
        validate_identity_component("run_id", &run_id)?;
        validate_identity_component("node_instance_id", &node_instance_id)?;
        validate_identity_component("effect_id", &effect_id)?;
        Ok(Self {
            run_id,
            node_instance_id,
            effect_id,
        })
    }

    /// Owning run.
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// Node instance inside that run.
    pub fn node_instance_id(&self) -> &str {
        &self.node_instance_id
    }

    /// Engine-derived effect id (see [`derive_effect_id`]).
    pub fn effect_id(&self) -> &str {
        &self.effect_id
    }

    /// Single-line rendering used by an effect target to deduplicate.
    ///
    /// Unambiguous because [`IdempotencyKey::new`] refuses control characters
    /// in every component — including the `0x1f` used as the separator here,
    /// and the tab and line feed the side-output log uses as ITS separators.
    pub fn as_line(&self) -> String {
        let sep = CONTEXT_SEPARATOR as char;
        format!(
            "{}{sep}{}{sep}{}",
            self.run_id, self.node_instance_id, self.effect_id
        )
    }
}

/// BLAKE3 over `context ‖ 0x1f ‖ jcs(value)`, hex-encoded.
///
/// ## Why a `null` is refused
///
/// MEASURED on `serde_jcs 0.2.0`, three paths, because the answer is not the
/// same on all three:
///
/// | value | result |
/// |---|---|
/// | `serde_jcs::to_string(&f64::NAN)` — bare scalar | `Err("invalid float value")` |
/// | `serde_jcs::to_string(&Struct { v: f64::NAN })` — NESTED | `Ok({"v":null})` |
/// | `serde_json::to_value` then `serde_jcs` | `Ok({"v":null})` |
///
/// So "the library refuses non-finite floats" is true only for a bare
/// top-level scalar. NESTED — which is the only shape this crate ever hashes
/// — `NaN`, `+Infinity` and `-Infinity` all collapse to `null`, silently,
/// with or without the `to_value` step. Four semantically distinct values,
/// plus an explicit `null`, therefore share one canonical form and one
/// digest. For a primitive whose entire job is to make a divergent replay
/// DETECTABLE, that is a collision at the worst possible place: two different
/// step requests would hash identically and the fail-closed check would wave
/// the second one through.
///
/// Rather than trust any claim about the library, this function REFUSES every
/// `null` in the canonical form. The consequence is deliberate and assumed:
/// an `Option::None` is refused too. Nothing this crate hashes carries an
/// optional member (see [`StepRequest`], whose input is
/// `BTreeMap<String, String>` by TYPE), so the strict rule costs nothing here
/// and removes a whole class of silent collisions.
pub fn canonical_digest<T: Serialize>(context: &str, value: &T) -> Result<String, HashError> {
    let canonical = canonical_bytes(value)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(context.as_bytes());
    hasher.update(&[CONTEXT_SEPARATOR]);
    hasher.update(&canonical);
    Ok(hasher.finalize().to_hex().to_string())
}

/// The single canonicalisation door of this crate: JCS bytes, with the
/// `null` rejection described on [`canonical_digest`]. Everything that
/// digests, echoes or compares a value goes through here, so the guarantee
/// cannot be bypassed by a second serialization path.
pub fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, HashError> {
    let as_value =
        serde_json::to_value(value).map_err(|e| HashError::Canonicalisation(e.to_string()))?;
    reject_nulls(&as_value, "$")?;
    serde_jcs::to_vec(&as_value).map_err(|e| HashError::Canonicalisation(e.to_string()))
}

/// [`canonical_bytes`] as text. JCS output is always valid UTF-8.
pub fn canonical_string<T: Serialize>(value: &T) -> Result<String, HashError> {
    String::from_utf8(canonical_bytes(value)?)
        .map_err(|e| HashError::Canonicalisation(e.to_string()))
}

/// Walk the canonical form and reject every `null`, reporting where it is.
fn reject_nulls(value: &serde_json::Value, path: &str) -> Result<(), HashError> {
    match value {
        serde_json::Value::Null => Err(HashError::NonRepresentable {
            path: path.to_string(),
        }),
        serde_json::Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                reject_nulls(item, &format!("{path}[{index}]"))?;
            }
            Ok(())
        }
        serde_json::Value::Object(members) => {
            for (key, member) in members {
                reject_nulls(member, &format!("{path}.{key}"))?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Digest of a step request, memoised in the journal and re-checked on replay.
pub fn step_request_hash(request: &StepRequest) -> Result<String, HashError> {
    canonical_digest(STEP_REQUEST_HASH_CONTEXT, request)
}

/// Derive the effect id from the run, the node instance and the request
/// hash. Deterministic across restarts — that is what lets a recovering run
/// address the SAME effect at the target rather than minting a second one.
///
/// Each component is LENGTH-PREFIXED, not merely separator-joined. A plain
/// `a ‖ 0x1f ‖ b` preimage is prefix-ambiguous: `("r\u{1f}n", "x")` and
/// `("r", "n\u{1f}x")` hash identically. Validation already refuses a `0x1f`
/// inside a component, so the ambiguity is unreachable through
/// [`IdempotencyKey::new`] — but relying on that alone would make the digest's
/// injectivity depend on a check living in another module, and on
/// `node_instance_id` happening to also be part of the canonicalised
/// `StepRequest`. Length prefixes make the encoding unambiguous on its own.
pub fn derive_effect_id(run_id: &str, node_instance_id: &str, request_hash: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(EFFECT_ID_CONTEXT.as_bytes());
    for component in [run_id, node_instance_id, request_hash] {
        hasher.update(&[CONTEXT_SEPARATOR]);
        hasher.update(&(component.len() as u64).to_le_bytes());
        hasher.update(component.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

/// Context string mixed into every derived wait nonce (Phase J).
pub const WAIT_NONCE_CONTEXT: &str = "sbfb.workflow.engine.wait-nonce.v0";

/// Derive the mono-usage nonce of one interruption from the run, the
/// RUNTIME node instance and the awaited signal name (research §12.6).
/// Deterministic across restarts — a replayed scheduling pass re-derives
/// the SAME nonce, so `open_wait`'s INSERT OR IGNORE keeps exactly one
/// spendable nonce per interruption; and engine-DERIVED, never
/// caller-chosen, for the same reason as [`derive_effect_id`]: a nonce an
/// untrusted author could pick would let one wait's deposit answer
/// another's. Length-prefixed like the effect id — injectivity must not
/// depend on a validation living in another module.
pub fn derive_wait_nonce(run_id: &str, node_instance_id: &str, signal_name: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(WAIT_NONCE_CONTEXT.as_bytes());
    for component in [run_id, node_instance_id, signal_name] {
        hasher.update(&[CONTEXT_SEPARATOR]);
        hasher.update(&(component.len() as u64).to_le_bytes());
        hasher.update(component.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

/// Digest of a receipt. Only this reaches the journal.
pub fn receipt_digest(receipt: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(RECEIPT_DIGEST_CONTEXT.as_bytes());
    hasher.update(&[CONTEXT_SEPARATOR]);
    hasher.update(receipt.as_bytes());
    hasher.finalize().to_hex().to_string()
}
