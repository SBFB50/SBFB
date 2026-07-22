// SPDX-License-Identifier: AGPL-3.0-or-later
//! `SBFBPromptAgentPack.v1` — the signed, portable prompt+agent pack (D2).
//!
//! A pack is the distribution unit of the Workflow Agents chantier: it names a
//! workflow, pins every one of its files by BLAKE3 digest (`PACK.lock`), and
//! carries in-toto-style provenance. It is DATA, never control flow — it grants
//! nothing. Capabilities inside a pack are *requests*; the authority that turns a
//! request into a permission lives outside this crate (Sprint 83 Phase C
//! installed the host-side intersection for the bridge).
//!
//! # What the signature proves, and what it does not
//!
//! A valid signature proves that the holder of one Ed25519 key SIGNED exactly
//! these canonical bytes under the pack domain — not an identity, not a
//! publication event. It proves nothing about the
//! trustworthiness of the prompts inside, and it is not an identity: this crate
//! verifies a signature against a key the CALLER supplies. Deciding which keys
//! are acceptable is a trust-model question this crate deliberately does not
//! answer.
//!
//! # Verification order (the property this module exists to hold)
//!
//! 1. **Size bound before any deserialisation** — [`MAX_PACK_JSON_BYTES`] is
//!    checked on the raw slice, so a decompression bomb never reaches serde.
//! 2. **Structural parse only** — the envelope is read as a
//!    [`serde_json::Value`]: two members, `payload` and `signature`. No typed
//!    field of the pack is interpreted yet.
//! 3. **Shape bounds** — depth, collection cardinality and string length are
//!    enforced on the payload value ([`MAX_JSON_DEPTH`], [`MAX_COLLECTION_LEN`],
//!    [`MAX_STRING_LEN`]). The structural parse of step 2 also refuses
//!    duplicate object keys anywhere in the envelope: `serde_json` would
//!    silently keep the last one, so a first-wins parser elsewhere would read
//!    a DIFFERENT pack out of the very byte stream this function verified.
//!    Refusing the bytes closes that cross-parser differential (byte streams
//!    that differ only in whitespace or member order still verify alike —
//!    that is JCS working as intended, not a gap).
//! 4. **Signature** — Ed25519 over [`canonical_bytes`] of the payload value.
//! 5. **Rich typed parse** — and only now does the payload become an
//!    [`SbfbPromptAgentPack`], with `deny_unknown_fields` on every type.
//! 6. **Semantic validation** — six well-formed digests with `content` bound to
//!    the lock, subjects named and unique, lock paths in the narrow admissible
//!    shape, and `secret_refs` that keep the SHAPE of opaque references (a
//!    heuristic refusal of pasted credentials — see
//!    `looks_like_materialised_secret` for what it does and does not catch).
//!
//! Steps 4 and 5 are ordered on purpose, and the gate test
//! `import_hostile::signature_before_parse` is written so that swapping them
//! turns it red. The honest phrasing is "signature before the RICH parse": a
//! bounded structural parse necessarily precedes it, because the bytes that get
//! signed are the canonical form of a JSON value.
//!
//! # Canonicalisation
//!
//! [`canonical_bytes`] is the single canonicalisation door of this module, in the
//! shape `<domain> 0x00 <JCS>` shared with `nexus_core_rs::canonical`. Every
//! digest and every signature pre-image in this module is built through it, so
//! the `null` rejection cannot be bypassed by a second serialisation path — the
//! same discipline as `sbfb-workflow-engine`'s `hash::canonical_bytes`. (The
//! plain byte helpers `hex_encode` / `blake3_hex` are not canonicalisation:
//! they never see a JSON value.)
//!
//! # Media content
//!
//! The pack ROOT embeds no file content at all — prompts, SVG, Markdown and
//! every other asset exist here only as `PACK.lock` entries (path + digest +
//! size), so refusal of active media CONTENT is by construction of the root
//! shape, not by a filter. What the root DOES carry is free-text metadata
//! (`name`, `description`, `builder_id`, subject names): those are
//! attacker-controlled strings, and any UI that renders them owns the
//! escaping. Whatever later materialises or imports the pinned files owns the
//! content-level hostile surface.

use std::collections::BTreeMap;

use ed25519_dalek::{Signature, VerifyingKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Format version of the pack envelope, carried inside the signed payload.
///
/// Versioned from v1 on purpose: n8n ships an unversioned workflow schema and
/// pays for it at every migration (kickoff D2). Pre-launch policy applies — the
/// canonical form of v1 is edited freely until the `v1.0` tag rather than bumped.
pub const PACK_FORMAT_VERSION: u16 = 1;

/// Ed25519 domain separation tag for pack signatures.
///
/// These bytes MIRROR `nexus_core_rs::canonical::DOMAIN_WORKFLOW_PACK_V1`. This
/// crate cannot import that constant: its charter forbids Tokio, network and
/// database, and `nexus-core-rs` pulls iroh. The mirror is not taken on trust —
/// `tests::pack_signing_domain_mirrors_canonical_rs` reads the declaring file
/// from disk, asserts the declaration exists EXACTLY once and uncommented, and
/// fails if it drifts. (A lexical check, not a compiler link: it binds the
/// declaration in that file, which is where the constant is canonically
/// defined.)
pub const PACK_SIGNING_DOMAIN: &[u8] = b"nexus-workflow-pack-v1";

/// Hard cap on the raw envelope, enforced BEFORE deserialisation.
///
/// This is also the honest memory-amplification bound of the structural
/// parse: the duplicate-key visitor materialises the full value tree before
/// the finer shape bounds run, so a hostile envelope can cost up to this many
/// bytes of parsed JSON (times a small constant) — bounded, and bounded HERE.
pub const MAX_PACK_JSON_BYTES: usize = 1 << 20;

/// Hard cap on JSON nesting depth inside the PAYLOAD (`enforce_bounds` runs
/// on the payload value; a refused envelope member dies at the parser).
pub const MAX_JSON_DEPTH: usize = 32;

/// Hard cap on the length of any string in the PAYLOAD (the signature string
/// and refused envelope members are bounded by [`MAX_PACK_JSON_BYTES`] alone).
pub const MAX_STRING_LEN: usize = 8 * 1024;

/// Hard cap on one `PACK.lock` path, total bytes.
pub const MAX_LOCK_PATH_BYTES: usize = 512;

/// Hard cap on the number of segments of one `PACK.lock` path.
pub const MAX_PATH_SEGMENTS: usize = 32;

/// Hard cap on one `PACK.lock` path segment, bytes (POSIX NAME_MAX).
pub const MAX_SEGMENT_BYTES: usize = 255;

/// Hard cap on one pinned file's declared size.
pub const MAX_FILE_SIZE_BYTES: u64 = 256 * 1024 * 1024;

/// Hard cap on the declared total size of a pack's pinned files.
pub const MAX_TOTAL_SIZE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// Hard cap on the cardinality of any array or object in the PAYLOAD (same
/// scope as [`MAX_JSON_DEPTH`]).
pub const MAX_COLLECTION_LEN: usize = 512;

/// Length in hex characters of a BLAKE3-256 digest.
const BLAKE3_HEX_LEN: usize = 64;

/// How much of a detected endpoint a scan finding quotes back.
const MAX_REPORTED_ENDPOINT_CHARS: usize = 256;

/// The largest integer that survives a round trip through an IEEE-754 double,
/// and therefore the largest one a JCS consumer in another language can read
/// back unchanged (I-JSON, RFC 7493 §2.2).
const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

/// Everything that can make a pack unacceptable.
///
/// Every variant is TERMINAL. There is no degraded "verified anyway" outcome:
/// a pack that carries a valid signature over a body that fails any later check
/// is refused, never downgraded (the failure mode that bit cosign).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackError {
    /// The raw envelope exceeds [`MAX_PACK_JSON_BYTES`].
    TooLarge { size_bytes: usize, cap: usize },
    /// The envelope is not the expected `{payload, signature}` object.
    MalformedEnvelope(String),
    /// A shape bound was exceeded, at `path`.
    Unbounded { path: String, detail: String },
    /// A `null` reached the canonicalisation door, at `path`.
    NonRepresentable { path: String },
    /// Canonicalisation itself failed.
    Canonicalisation(String),
    /// The signature is absent, malformed, or does not verify under the domain.
    SignatureInvalid(String),
    /// The payload does not match the typed schema (this is the RICH parse).
    SchemaViolation(String),
    /// The payload is well typed but semantically unacceptable.
    Semantic(String),
}

impl std::fmt::Display for PackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLarge { size_bytes, cap } => {
                write!(f, "pack envelope is {size_bytes} bytes, cap is {cap}")
            }
            Self::MalformedEnvelope(detail) => write!(f, "malformed pack envelope: {detail}"),
            Self::Unbounded { path, detail } => write!(f, "bound exceeded at {path}: {detail}"),
            Self::NonRepresentable { path } => {
                write!(f, "null is not canonically representable at {path}")
            }
            Self::Canonicalisation(detail) => write!(f, "canonicalisation failed: {detail}"),
            Self::SignatureInvalid(detail) => write!(f, "signature rejected: {detail}"),
            Self::SchemaViolation(detail) => write!(f, "pack schema violation: {detail}"),
            Self::Semantic(detail) => write!(f, "pack semantically refused: {detail}"),
        }
    }
}

impl std::error::Error for PackError {}

/// The six digest claims of a pack, one field per facet.
///
/// Six SIGNED digest CLAIMS, not six independently verified facts. What this
/// crate checks about them is uneven, and the split is not a partition:
/// `content` is RECOMPUTED from the lock and therefore covers every pinned
/// byte — so a presentation-only change moves both `presentation` AND
/// `content`; the two are not disjoint. The other four (`graph_semantic`,
/// `presentation`, `compiled_plan`, `evaluation`, `publication`) are
/// validated for SHAPE only here — binding them to the artefacts they name
/// needs those artefacts, which this pure crate never receives (it would be
/// a second, materialising verification pass). The separation exists so that
/// pass CAN check one facet without the others, not because this pass does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PackDigestKind {
    /// Every byte of every file listed in `PACK.lock`.
    Content,
    /// The meaning of the workflow graph, free of presentation: moving a node
    /// on a canvas must not change it. Array ORDER stays significant until the
    /// typed graph contract (a later phase) pins node identity — this pass does
    /// not reorder nodes or edges by id.
    GraphSemantic,
    /// Presentation only: positions, colours, collapsed state.
    Presentation,
    /// The plan a compiler produced from the graph.
    CompiledPlan,
    /// The evaluation suite bound to the pack.
    Evaluation,
    /// Publication metadata (naming, listing, discovery).
    Publication,
}

impl PackDigestKind {
    /// The canonical enumeration, in order. Iterating THIS — never a
    /// deserialised map — is what makes digest completeness checkable.
    pub const ALL: [Self; 6] = [
        Self::Content,
        Self::GraphSemantic,
        Self::Presentation,
        Self::CompiledPlan,
        Self::Evaluation,
        Self::Publication,
    ];

    /// Field name of this digest in the canonical JSON form.
    pub const fn field(self) -> &'static str {
        match self {
            Self::Content => "content",
            Self::GraphSemantic => "graph_semantic",
            Self::Presentation => "presentation",
            Self::CompiledPlan => "compiled_plan",
            Self::Evaluation => "evaluation",
            Self::Publication => "publication",
        }
    }
}

/// The six digests as a signed struct. No member is optional: a missing digest
/// is a refusal, not a default.
///
/// What each one binds, from this crate's point of view: `content` is
/// RECOMPUTED by the verifier from the lock ([`content_digest_of_lock`]), so it
/// cannot drift from what the pack pins; `graph_semantic` and `presentation`
/// are recomputable from a graph document through [`graph_semantic_view`] /
/// [`presentation_view`]; `compiled_plan`, `evaluation` and `publication` are
/// validated for SHAPE only here — the documents they cover are lock-pinned
/// files this crate never opens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "PackDigests.v1")]
pub struct PackDigests {
    /// BLAKE3 hex of every byte pinned by `PACK.lock`.
    pub content: String,
    /// BLAKE3 hex of the presentation-free graph meaning.
    pub graph_semantic: String,
    /// BLAKE3 hex of the presentation layer.
    pub presentation: String,
    /// BLAKE3 hex of the compiled plan.
    pub compiled_plan: String,
    /// BLAKE3 hex of the evaluation suite.
    pub evaluation: String,
    /// BLAKE3 hex of the publication metadata.
    pub publication: String,
}

impl PackDigests {
    /// Read one digest by its canonical kind.
    pub fn get(&self, kind: PackDigestKind) -> &str {
        match kind {
            PackDigestKind::Content => &self.content,
            PackDigestKind::GraphSemantic => &self.graph_semantic,
            PackDigestKind::Presentation => &self.presentation,
            PackDigestKind::CompiledPlan => &self.compiled_plan,
            PackDigestKind::Evaluation => &self.evaluation,
            PackDigestKind::Publication => &self.publication,
        }
    }

    /// Refuse unless all six digests are well-formed LOWERCASE BLAKE3 hex.
    ///
    /// Lowercase-only on purpose: hex has two spellings per digest, and a
    /// format that accepted both would let two canonically DIFFERENT packs pin
    /// the same bytes — the digest-level cousin of the duplicate-key problem.
    /// [`blake3_hex`] only ever emits lowercase.
    ///
    /// Iterates [`PackDigestKind::ALL`], so a digest that were dropped from the
    /// struct could not slip through unchecked.
    fn validate(&self) -> Result<(), PackError> {
        for kind in PackDigestKind::ALL {
            if !is_lower_hex_digest(self.get(kind)) {
                return Err(PackError::Semantic(format!(
                    "digest '{}' is not a 64-char lowercase BLAKE3 hex string",
                    kind.field()
                )));
            }
        }
        Ok(())
    }
}

/// A single-algorithm digest set, in-toto's `digest` shape.
///
/// in-toto allows several algorithms per subject; v1 admits exactly one, BLAKE3,
/// because a multi-algorithm set invites the weakest-link question ("which one
/// did the verifier actually check?"). BLAKE3 is not a standardised in-toto
/// algorithm — the borrowing is of the SHAPE, hence "in-toto style".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "PackDigestSet.v1")]
pub struct PackDigestSet {
    /// BLAKE3 hex digest.
    pub blake3: String,
}

/// One named artefact this pack attests to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "PackSubject.v1")]
pub struct PackSubject {
    /// Name of the artefact, unique within the pack.
    pub name: String,
    /// Its digest set.
    pub digest: PackDigestSet,
}

/// Provenance of the pack, in-toto style.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "PackProvenance.v1")]
pub struct PackProvenance {
    /// The attested artefacts. At least one, names unique.
    pub subject: Vec<PackSubject>,
    /// Free-form builder identifier, e.g. the tool that assembled the pack.
    pub builder_id: String,
}

/// One file pinned by `PACK.lock`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "PackLockEntry.v1")]
pub struct PackLockEntry {
    /// Pack-relative path, forward slashes, no `.` or `..` segment, no drive
    /// letter, no leading separator.
    pub path: String,
    /// BLAKE3 hex of the file bytes.
    pub blake3: String,
    /// Size of the file in bytes.
    pub size_bytes: u64,
}

/// `PACK.lock` — every file of the pack pinned by digest.
///
/// The signature covers the lock, and the lock names the digest of every byte —
/// that chain is the rug-pull resistance of the FORMAT. What THIS crate can
/// enforce ends at the lock: it never sees file bytes, so checking that a
/// materialised file matches its pinned digest belongs to whoever extracts the
/// pack. A verifier that skipped that check would re-open the rug-pull.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "PackLock.v1")]
pub struct PackLock {
    /// Pinned files, paths unique and sorted by the producer.
    pub entries: Vec<PackLockEntry>,
}

/// `SBFBPromptAgentPack.v1` — root of the signed, portable prompt+agent pack
/// (D2). NAMES AND PINS — by digest, never by embedding — the workflow graph,
/// agent definitions, prompts, pinned schemas and evals, and carries requested
/// (never granted) capabilities and provenance. Documented v1 limitation: the
/// root does not yet REQUIRE any particular artefact set in its lock (no
/// role→path rule says "a workflow graph must be pinned") — those requirements
/// belong to the graph/agent contracts, and until they land a syntactically
/// valid pack may pin any file set.
/// Presentation is kept out of the semantic digest by the
/// [`PRESENTATION_KEYS`] reservation (the kickoff's "`layout/` outside the
/// semantic digest" intent, realised as reserved member names).
///
/// The graph and agent bodies are pinned here BY DIGEST rather than embedded:
/// `SBFBWorkflowGraph.v1` and `SBFBAgentDefinition.v1` are reserved identities
/// whose fields are authored in the workflow-language phase, and a root that
/// embedded them would have to change shape when they land. Pinning by digest
/// is what lets this root be frozen first.
// FRONTIER: SbfbPromptAgentPack domain=DOMAIN_WORKFLOW_PACK_V1 version=PACK_FORMAT_VERSION
// Sprint 83 Phase F · doctrine §7 — its generated JSON schema is pinned by
// `contracts.schema.json` (schema_snapshot_stub); gated by scripts/check-frontier-contracts.sh.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "SBFBPromptAgentPack.v1")]
pub struct SbfbPromptAgentPack {
    /// Envelope format version; must equal [`PACK_FORMAT_VERSION`].
    pub format_version: u16,
    /// Monotonic revision of THIS pack identity, minted by the publisher.
    ///
    /// Reserved in v1 so that anti-rollback enforcement has a field to stand on
    /// without a later wire bump. Enforcing it is stateful (a floor per pack
    /// identity, the `iroh_runtime` pattern) and belongs to whoever installs a
    /// pack; this crate is pure and holds no state, so it validates the field's
    /// presence and range, never its history.
    pub pack_revision: u64,
    /// Stable identity of the pack, unique per publisher.
    pub name: String,
    /// Human description.
    pub description: String,
    /// The six semantic digests.
    pub digests: PackDigests,
    /// Every file of the pack, pinned by digest.
    pub lock: PackLock,
    /// in-toto style provenance.
    pub provenance: PackProvenance,
    /// Capabilities the pack REQUESTS. Requesting is not obtaining: an
    /// authority intersects these against a grant and a policy elsewhere.
    pub requested_capabilities: Vec<String>,
    /// Opaque LOGICAL secret references, e.g. `"llm.api_key"` — the format
    /// RULE is that a value here names a slot, never a secret or a
    /// deployment's credential name. Enforcement is shape-heuristic only
    /// (`looks_like_materialised_secret`): a 64-hex secret or one split
    /// across fields walks past it, and the other free-text fields are not
    /// checked at all — the rule stops the honest-mistake paste, nothing
    /// stronger.
    pub secret_refs: Vec<String>,
}

/// A pack together with the signature over its canonical bytes.
///
/// Signature and payload stay separable so that a DSSE envelope can wrap the
/// same payload later without breaking anything already published.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "SignedPack.v1")]
pub struct SignedPack {
    /// The signed payload.
    pub payload: SbfbPromptAgentPack,
    /// Ed25519 signature over [`canonical_bytes`] of the payload, hex encoded.
    pub signature: String,
}

/// The single canonicalisation door: `<domain> 0x00 <JCS>`, with `null`
/// rejected everywhere.
///
/// The `0x00` separator is a hard boundary, so a crafted value cannot smuggle
/// the domain into its own JSON payload (the `nexus_core_rs::canonical` rule).
///
/// `null` is rejected for two measured reasons. First, `serde_jcs` collapses a
/// NESTED non-finite float to `null` instead of erroring — silent information
/// loss inside a signature pre-image (demonstrated in
/// `tests::nested_non_finite_float_is_refused_not_collapsed`). Second, an
/// explicit `null` member and an absent member are a classic consumer
/// ambiguity: readers that treat them as equivalent disagree with readers that
/// do not, over bytes both consider verified. Refusing `null` outright removes
/// both classes. This crate can hold that line because every signed struct here
/// declares required fields only; the rule is deliberately NOT hoisted into
/// `nexus-core-rs`, where `Task::required_runtime` signs over an explicit
/// `null` today and would stop verifying.
///
/// Number caveat, inherited from JSON itself, and NORMATIVE for this format:
/// distinct decimal SPELLINGS of the same IEEE-754 double (`1e2` / `100.0`),
/// and `-0.0` versus `0`, are ONE value after JCS canonicalisation in every
/// RFC 8785 implementation (the parity golden itself shows `-0.0` becoming
/// `0`). This door guarantees one parsed value has one canonical form — not
/// one byte spelling per number — so a consumer of a pack MUST NOT rely on a
/// distinction JCS erases (the sign of zero, integer-vs-float of an equal
/// value). Freezing that as a rule now is cheaper than a later graph contract
/// discovering it.
///
/// # Errors
///
/// [`PackError::NonRepresentable`] if a `null` is reachable,
/// [`PackError::Canonicalisation`] if JCS serialisation fails.
pub fn canonical_bytes<T: Serialize + ?Sized>(
    value: &T,
    domain: &[u8],
) -> Result<Vec<u8>, PackError> {
    let as_value = serde_json::to_value(value)
        .map_err(|e| PackError::Canonicalisation(format!("to_value: {e}")))?;
    reject_nulls(&as_value, "$")?;
    let body = serde_jcs::to_vec(&as_value)
        .map_err(|e| PackError::Canonicalisation(format!("JCS: {e}")))?;
    let mut out = Vec::with_capacity(domain.len() + 1 + body.len());
    out.extend_from_slice(domain);
    out.push(0);
    out.extend_from_slice(&body);
    Ok(out)
}

/// Walk a canonical form and reject every value with no faithful canonical
/// spelling: `null`, and integers outside the I-JSON exact range.
///
/// Applied to a `serde_json::Value` rather than to the JCS output on purpose:
/// `serde_jcs` does not reject a non-finite float once it is NESTED — it emits
/// `null` for it — so scanning the value tree is what actually closes that
/// hole. The integer rule is equally MEASURED, not theoretical: `serde_jcs`
/// serialises `9007199254740993` as `9007199254740992` (RFC 8785 goes through
/// an IEEE double), so two DISTINCT values would otherwise share one canonical
/// form — a silent collision inside a signature pre-image. Enforcing it in the
/// door (not only in the verify pipeline's `enforce_bounds`) is what protects
/// direct callers of [`digest_for`] / [`content_digest_of_lock`] too.
fn reject_nulls(value: &serde_json::Value, path: &str) -> Result<(), PackError> {
    match value {
        serde_json::Value::Null => Err(PackError::NonRepresentable {
            path: path.to_string(),
        }),
        serde_json::Value::Number(n) => {
            let out_of_range = match (n.as_i64(), n.as_u64()) {
                (Some(i), _) => i.unsigned_abs() > MAX_SAFE_INTEGER as u64,
                (None, Some(u)) => u > MAX_SAFE_INTEGER as u64,
                _ => false,
            };
            if out_of_range {
                Err(PackError::Canonicalisation(format!(
                    "integer at {path} outside the I-JSON exact range collapses under JCS"
                )))
            } else {
                Ok(())
            }
        }
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

/// Enforce the shape bounds on a parsed value, before anything typed is read.
///
/// Depth, cardinality and string length are three DISTINCT bounds; each has its
/// own test, because a single "it's bounded" claim hides which one actually
/// fired. Integer range is enforced here too: JSON has no integer type, so a
/// value beyond IEEE-754's exact range canonicalises to something a TypeScript
/// verifier reads back as a different number.
fn enforce_bounds(value: &serde_json::Value, path: &str, depth: usize) -> Result<(), PackError> {
    if depth > MAX_JSON_DEPTH {
        return Err(PackError::Unbounded {
            path: path.to_string(),
            detail: format!("nesting deeper than {MAX_JSON_DEPTH}"),
        });
    }
    match value {
        serde_json::Value::String(s) => {
            if s.len() > MAX_STRING_LEN {
                return Err(PackError::Unbounded {
                    path: path.to_string(),
                    detail: format!("string of {} bytes, cap is {MAX_STRING_LEN}", s.len()),
                });
            }
            Ok(())
        }
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                // `unsigned_abs`, not `abs`: `i64::MIN.abs()` panics, and a
                // hostile corpus is exactly where that value shows up.
                if i.unsigned_abs() > MAX_SAFE_INTEGER as u64 {
                    return Err(PackError::Unbounded {
                        path: path.to_string(),
                        detail: format!("integer {i} outside the I-JSON exact range"),
                    });
                }
                Ok(())
            } else if n.as_u64().is_some_and(|u| u > MAX_SAFE_INTEGER as u64) {
                Err(PackError::Unbounded {
                    path: path.to_string(),
                    detail: format!("integer {n} outside the I-JSON exact range"),
                })
            } else {
                Ok(())
            }
        }
        serde_json::Value::Array(items) => {
            if items.len() > MAX_COLLECTION_LEN {
                return Err(PackError::Unbounded {
                    path: path.to_string(),
                    detail: format!(
                        "array of {} entries, cap is {MAX_COLLECTION_LEN}",
                        items.len()
                    ),
                });
            }
            for (index, item) in items.iter().enumerate() {
                enforce_bounds(item, &format!("{path}[{index}]"), depth + 1)?;
            }
            Ok(())
        }
        serde_json::Value::Object(members) => {
            if members.len() > MAX_COLLECTION_LEN {
                return Err(PackError::Unbounded {
                    path: path.to_string(),
                    detail: format!(
                        "object of {} members, cap is {MAX_COLLECTION_LEN}",
                        members.len()
                    ),
                });
            }
            for (key, member) in members {
                if key.len() > MAX_STRING_LEN {
                    return Err(PackError::Unbounded {
                        path: format!("{path}.{key}"),
                        detail: "member name longer than the string cap".to_string(),
                    });
                }
                enforce_bounds(member, &format!("{path}.{key}"), depth + 1)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Verify a signed pack from its raw bytes, under [`PACK_SIGNING_DOMAIN`].
///
/// The steps run in the order documented at the module level, and that order is
/// the security property: nothing typed is interpreted before the signature has
/// been checked, and nothing at all is parsed before the size bound.
///
/// The domain is NOT a parameter: binding it inside the function is what makes
/// domain separation a property of this crate rather than a discipline asked of
/// every caller. `verifying_key` IS supplied by the caller — this function
/// answers "were these bytes signed by THAT key under the pack domain", never
/// "should that key be trusted".
///
/// # Errors
///
/// One of [`PackError`], all terminal. A signature that verifies over a body
/// which later fails validation yields the LATER error and no pack — there is no
/// partially-accepted outcome.
pub fn verify_signed_pack(
    raw: &[u8],
    verifying_key: &VerifyingKey,
) -> Result<SbfbPromptAgentPack, PackError> {
    verify_signed_pack_under_domain(raw, PACK_SIGNING_DOMAIN, verifying_key)
}

/// [`verify_signed_pack`] with an explicit domain — private on purpose, so the
/// only way to verify under a foreign domain is from this module's own tests
/// (which need it to PROVE cross-domain rejection).
fn verify_signed_pack_under_domain(
    raw: &[u8],
    domain: &[u8],
    verifying_key: &VerifyingKey,
) -> Result<SbfbPromptAgentPack, PackError> {
    // (1) size bound, before serde sees anything.
    if raw.len() > MAX_PACK_JSON_BYTES {
        return Err(PackError::TooLarge {
            size_bytes: raw.len(),
            cap: MAX_PACK_JSON_BYTES,
        });
    }

    // (2) structural parse only — two members, nothing typed yet. The parse
    // refuses duplicate object keys anywhere: `serde_json` alone keeps the
    // last one silently, and a first-wins parser elsewhere would read a
    // different pack out of the same verified bytes.
    let envelope = parse_rejecting_duplicate_keys(raw)?;
    let members = envelope
        .as_object()
        .ok_or_else(|| PackError::MalformedEnvelope("envelope is not an object".to_string()))?;
    for key in members.keys() {
        if key != "payload" && key != "signature" {
            return Err(PackError::MalformedEnvelope(format!(
                "unknown envelope member '{key}'"
            )));
        }
    }
    let payload_value = members
        .get("payload")
        .ok_or_else(|| PackError::MalformedEnvelope("no 'payload' member".to_string()))?;
    let signature_hex = members
        .get("signature")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| PackError::MalformedEnvelope("no 'signature' string".to_string()))?;

    // (3) shape bounds on the untyped tree. (The signature string needs no
    // bound of its own: the whole envelope is already under the step-1 cap.)
    enforce_bounds(payload_value, "$.payload", 0)?;

    // (4) signature over the canonical payload. The pre-image is built FIRST:
    // a payload that cannot be canonicalised has no signable form at all, and
    // reporting that as a signature failure would misname the defect.
    let signed_bytes = canonical_bytes(payload_value, domain)?;
    if signature_hex.len() != 128 {
        return Err(PackError::SignatureInvalid(format!(
            "signature is {} chars, an Ed25519 signature is 128 hex chars",
            signature_hex.len()
        )));
    }
    let signature_bytes = hex_decode(signature_hex)
        .ok_or_else(|| PackError::SignatureInvalid("signature is not hex".to_string()))?;
    let signature_array: [u8; 64] = signature_bytes
        .try_into()
        .map_err(|_| PackError::SignatureInvalid("signature is not 64 bytes".to_string()))?;
    let signature = Signature::from_bytes(&signature_array);
    // STRICT verification, and an explicit weak-key refusal before it. Plain
    // `verify` accepts small-order public keys, and for the identity-point
    // key the forged signature (R = identity, S = 0) satisfies the basic
    // equation for EVERY message — no private key involved, which would make
    // "the holder of the key signed these bytes" a false sentence (found by
    // external cross-verification round 4; the boundary test below the module
    // exercises exactly that key). `verify_strict` rejects the small-order
    // class; `is_weak` names the refusal instead of leaving it to the
    // equation.
    if verifying_key.is_weak() {
        return Err(PackError::SignatureInvalid(
            "verifying key is a small-order (weak) point".to_string(),
        ));
    }
    verifying_key
        .verify_strict(&signed_bytes, &signature)
        .map_err(|e| PackError::SignatureInvalid(format!("Ed25519: {e}")))?;

    // (5) rich typed parse — the first moment a pack field is interpreted.
    // Deserialised BY REFERENCE from the already-parsed value: no second pass
    // over the raw bytes, no clone of a payload that may be a mebibyte.
    let pack = SbfbPromptAgentPack::deserialize(payload_value)
        .map_err(|e| PackError::SchemaViolation(e.to_string()))?;

    // (6) semantic validation.
    validate_pack(&pack)?;
    Ok(pack)
}

/// Parse raw JSON into a [`serde_json::Value`], refusing duplicate object keys
/// at every depth.
///
/// `serde_json` streams every key/value pair of the source text to the
/// visitor, so duplicates are observable here even though `serde_json`'s own
/// `Value` visitor would silently keep the last.
fn parse_rejecting_duplicate_keys(raw: &[u8]) -> Result<serde_json::Value, PackError> {
    struct DupFree(serde_json::Value);

    impl<'de> Deserialize<'de> for DupFree {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct V;
            impl<'de> serde::de::Visitor<'de> for V {
                type Value = serde_json::Value;

                fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str("a JSON value without duplicate object keys")
                }

                fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
                    Ok(serde_json::Value::Bool(v))
                }
                fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
                    Ok(serde_json::Value::from(v))
                }
                fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
                    Ok(serde_json::Value::from(v))
                }
                fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
                    serde_json::Number::from_f64(v)
                        .map(serde_json::Value::Number)
                        .ok_or_else(|| E::custom("non-finite number"))
                }
                fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                    Ok(serde_json::Value::String(v.to_string()))
                }
                fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
                    Ok(serde_json::Value::String(v))
                }
                fn visit_unit<E>(self) -> Result<Self::Value, E> {
                    Ok(serde_json::Value::Null)
                }
                fn visit_seq<A: serde::de::SeqAccess<'de>>(
                    self,
                    mut seq: A,
                ) -> Result<Self::Value, A::Error> {
                    let mut items = Vec::new();
                    while let Some(DupFree(item)) = seq.next_element()? {
                        items.push(item);
                    }
                    Ok(serde_json::Value::Array(items))
                }
                fn visit_map<A: serde::de::MapAccess<'de>>(
                    self,
                    mut map: A,
                ) -> Result<Self::Value, A::Error> {
                    let mut members = serde_json::Map::new();
                    while let Some(key) = map.next_key::<String>()? {
                        let DupFree(value) = map.next_value()?;
                        if members.insert(key.clone(), value).is_some() {
                            return Err(serde::de::Error::custom(format!(
                                "duplicate object key '{key}'"
                            )));
                        }
                    }
                    Ok(serde_json::Value::Object(members))
                }
            }
            deserializer.deserialize_any(V).map(DupFree)
        }
    }

    serde_json::from_slice::<DupFree>(raw)
        .map(|v| v.0)
        .map_err(|e| PackError::MalformedEnvelope(format!("not JSON: {e}")))
}

/// Semantic acceptance rules, applied after the typed parse.
fn validate_pack(pack: &SbfbPromptAgentPack) -> Result<(), PackError> {
    if pack.format_version != PACK_FORMAT_VERSION {
        return Err(PackError::Semantic(format!(
            "format_version {} is not {PACK_FORMAT_VERSION}",
            pack.format_version
        )));
    }
    if pack.name.trim().is_empty() {
        return Err(PackError::Semantic("name is empty".to_string()));
    }
    pack.digests.validate()?;

    if pack.provenance.subject.is_empty() {
        return Err(PackError::Semantic(
            "provenance carries no subject".to_string(),
        ));
    }
    let mut seen_subjects = BTreeMap::new();
    for subject in &pack.provenance.subject {
        if subject.name.trim().is_empty() {
            return Err(PackError::Semantic("subject with empty name".to_string()));
        }
        if seen_subjects.insert(subject.name.as_str(), ()).is_some() {
            return Err(PackError::Semantic(format!(
                "duplicate subject name '{}'",
                subject.name
            )));
        }
        if !is_lower_hex_digest(&subject.digest.blake3) {
            return Err(PackError::Semantic(format!(
                "subject '{}' digest is not lowercase BLAKE3 hex",
                subject.name
            )));
        }
    }

    if pack.lock.entries.is_empty() {
        return Err(PackError::Semantic(
            "PACK.lock pins no file — an empty pack attests to nothing".to_string(),
        ));
    }
    // Uniqueness is checked case-INSENSITIVELY: two paths differing only by
    // case are distinct strings here and one file on a Windows or macOS
    // filesystem, which would make the pinned digests ambiguous exactly like a
    // literal duplicate. Declared sizes are capped per file and in total
    // (checked_add — a u64 sum of hostile sizes may otherwise wrap), so a
    // syntactically valid pack cannot promise exabytes to a materialiser.
    let mut seen_paths = BTreeMap::new();
    let mut total_size: u64 = 0;
    for entry in &pack.lock.entries {
        validate_lock_path(&entry.path)?;
        if seen_paths
            .insert(entry.path.to_ascii_lowercase(), ())
            .is_some()
        {
            return Err(PackError::Semantic(format!(
                "duplicate lock path (case-insensitive) '{}'",
                entry.path
            )));
        }
        if !is_lower_hex_digest(&entry.blake3) {
            return Err(PackError::Semantic(format!(
                "lock entry '{}' digest is not lowercase BLAKE3 hex",
                entry.path
            )));
        }
        if entry.size_bytes > MAX_FILE_SIZE_BYTES {
            return Err(PackError::Semantic(format!(
                "lock entry '{}' declares {} bytes, per-file cap is {MAX_FILE_SIZE_BYTES}",
                entry.path, entry.size_bytes
            )));
        }
        total_size = total_size
            .checked_add(entry.size_bytes)
            .filter(|t| *t <= MAX_TOTAL_SIZE_BYTES)
            .ok_or_else(|| {
                PackError::Semantic(format!(
                    "declared total size exceeds the {MAX_TOTAL_SIZE_BYTES}-byte cap"
                ))
            })?;
    }
    // Strict byte-lexicographic order: a producer that could emit one file set
    // in several orders would mint several canonical forms (and several
    // content digests) for one pack. Checked AFTER per-entry rules so a
    // duplicate dies as a duplicate, not as an ordering error.
    for pair in pack.lock.entries.windows(2) {
        if pair[0].path.as_bytes() >= pair[1].path.as_bytes() {
            return Err(PackError::Semantic(format!(
                "lock entries are not sorted byte-lexicographically ('{}' >= '{}')",
                pair[0].path, pair[1].path
            )));
        }
    }

    // The content digest is RECOMPUTED, never taken at its word: a pack whose
    // `digests.content` does not match its own lock is refused, which is what
    // ties the six-digest struct to the thing it claims to cover (A10).
    let expected_content = content_digest_of_lock(&pack.lock)?;
    if pack.digests.content != expected_content {
        return Err(PackError::Semantic(format!(
            "content digest does not match PACK.lock (expected {expected_content})"
        )));
    }

    for (index, secret_ref) in pack.secret_refs.iter().enumerate() {
        if looks_like_materialised_secret(secret_ref) {
            // The refused VALUE is deliberately withheld: echoing a detected
            // secret into an error would copy it into every log that prints
            // the refusal.
            return Err(PackError::Semantic(format!(
                "secret_refs[{index}] must stay an opaque logical reference (value withheld)"
            )));
        }
    }
    Ok(())
}

/// The `content` digest a verifier expects: [`digest_for`] over the canonical
/// form of the lock itself.
///
/// The lock names the BLAKE3 of every file, so digesting the LOCK covers every
/// pinned byte transitively — without this crate ever opening a file.
///
/// # Errors
///
/// Propagates the canonicalisation errors of [`canonical_bytes`].
pub fn content_digest_of_lock(lock: &PackLock) -> Result<String, PackError> {
    let value = serde_json::to_value(lock)
        .map_err(|e| PackError::Canonicalisation(format!("lock to_value: {e}")))?;
    digest_for(PackDigestKind::Content, &value)
}

/// Reject a lock path that could escape the pack root — or change identity —
/// when it is materialised.
///
/// The refusal is by SHAPE, decided on the string alone: this crate never
/// touches a filesystem, so it cannot resolve a path — which is precisely why
/// the admissible shape has to be narrow. The v1 alphabet is deliberately
/// ASCII-only (`A-Z a-z 0-9 . _ - /`): one rule closes Unicode confusables
/// (UTS #39), normalisation drift, percent-encoding games, spaces and every
/// exotic separator at once, at the price of expressiveness the pack format
/// does not need (pre-launch, relaxable by editing v1 if that ever changes).
/// Backslash and `:` therefore fall out of the alphabet, but keep their own
/// named refusals for a diagnosable error. All 22 reserved DOS device names
/// are refused because on Windows they resolve to devices regardless of the
/// directory, and a segment may not end in `.` because Win32 silently strips
/// trailing dots — two spellings, one file.
fn validate_lock_path(path: &str) -> Result<(), PackError> {
    const RESERVED: [&str; 22] = [
        "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
        "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    ];

    if path.is_empty() {
        return Err(PackError::Semantic("empty lock path".to_string()));
    }
    if path.len() > MAX_LOCK_PATH_BYTES {
        return Err(PackError::Semantic(format!(
            "lock path longer than {MAX_LOCK_PATH_BYTES} bytes"
        )));
    }
    if path.starts_with('/') || path.starts_with('\\') {
        return Err(PackError::Semantic(format!("absolute lock path: {path}")));
    }
    if path.contains('\\') {
        return Err(PackError::Semantic(format!(
            "backslash in lock path: {path}"
        )));
    }
    if path.contains(':') {
        return Err(PackError::Semantic(format!(
            "drive or stream separator in lock path: {path}"
        )));
    }
    if let Some(bad) = path
        .chars()
        .find(|c| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/')))
    {
        return Err(PackError::Semantic(format!(
            "character {bad:?} outside the lock-path alphabet: {path}"
        )));
    }
    if path.split('/').count() > MAX_PATH_SEGMENTS {
        return Err(PackError::Semantic(format!(
            "lock path has more than {MAX_PATH_SEGMENTS} segments: {path}"
        )));
    }
    for segment in path.split('/') {
        if segment.is_empty() {
            return Err(PackError::Semantic(format!(
                "empty segment in lock path: {path}"
            )));
        }
        if segment.len() > MAX_SEGMENT_BYTES {
            return Err(PackError::Semantic(format!(
                "segment longer than {MAX_SEGMENT_BYTES} bytes in lock path: {path}"
            )));
        }
        if segment == "." || segment == ".." {
            return Err(PackError::Semantic(format!(
                "relative segment in lock path: {path}"
            )));
        }
        if segment.ends_with('.') {
            return Err(PackError::Semantic(format!(
                "segment ends with '.' in lock path: {path}"
            )));
        }
        let stem = segment.split('.').next().unwrap_or(segment);
        if RESERVED.contains(&stem.to_ascii_lowercase().as_str()) {
            return Err(PackError::Semantic(format!(
                "reserved device name in lock path: {path}"
            )));
        }
    }
    Ok(())
}

/// Heuristic refusal of a secret that was pasted where a reference belongs.
///
/// A logical reference names a slot (`llm.api_key`). This rule catches two
/// SHAPES that are never slot names — a known credential prefix, and a long
/// run mixing upper/lower/digit — and nothing else. It is enforced, but it is
/// NOT a guarantee that no secret can ride in a pack: a secret that happens to
/// be 64 hex chars is indistinguishable from a digest by shape, and any secret
/// split across fields walks past every shape rule. The rule exists to stop
/// the honest-mistake paste, not the determined smuggler — the same honest
/// bound as [`scan_for_disclosures`].
pub(crate) fn looks_like_materialised_secret(value: &str) -> bool {
    const CREDENTIAL_PREFIXES: [&str; 8] = [
        "sk-",
        "ghp_",
        "github_pat_",
        "xoxb-",
        "aws_secret",
        "-----BEGIN",
        // AWS access key ids and JWTs, the two most-pasted shapes after API keys.
        "akia",
        "eyj",
    ];
    let lowered = value.to_ascii_lowercase();
    if CREDENTIAL_PREFIXES
        .iter()
        .any(|prefix| lowered.starts_with(&prefix.to_ascii_lowercase()))
    {
        return true;
    }
    // A digest is a SHAPE this format is made of, not a secret. The exemption
    // matters for UPPERCASE hex spellings (which the mixed-class rule below
    // would flag); lowercase digests would pass that rule anyway, and the
    // exemption records the intent for both — a scanner that cries wolf on
    // its own data model is worse than no scanner.
    if is_hex_digest(value) {
        return false;
    }
    // Otherwise: a long run that mixes character classes is credential-shaped,
    // not a slot name like `llm.api_key`. Requiring the MIX (rather than "all
    // alphanumeric") is what keeps lowercase hex and snake_case identifiers out.
    value.len() >= 40
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        && value.chars().any(|c| c.is_ascii_uppercase())
        && value.chars().any(|c| c.is_ascii_lowercase())
        && value.chars().any(|c| c.is_ascii_digit())
}

/// Is this string exactly a BLAKE3-256 hex digest, in either case?
///
/// Any-case on purpose: this feeds the SCANNER exemption, where an uppercase
/// hex string is still digest-shaped noise. Format validation is stricter and
/// goes through [`is_lower_hex_digest`].
fn is_hex_digest(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LEN && value.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Is this string exactly a LOWERCASE BLAKE3-256 hex digest — the only spelling
/// the format admits?
pub(crate) fn is_lower_hex_digest(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LEN
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// One thing the best-effort scanner noticed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanFinding {
    /// Where in the canonical tree, e.g. `$.description`.
    pub path: String,
    /// What was noticed.
    pub detail: String,
}

/// Scan a pack for secret-looking strings and outbound endpoints.
///
/// **Not authoritative, and not a gate.** It reports; it never decides. Two
/// reasons to keep it that way: a regex-shaped scanner is trivially evaded by
/// splitting a string, and treating "scan found nothing" as "pack is clean"
/// converts a hint into a false guarantee. The enforced rules are the format's
/// own — required fields, bounded shapes, opaque `secret_refs`, and the
/// capability intersection that lives outside this crate. An endpoint appearing
/// here is likewise only a REQUEST: a pack cannot reach the network, and
/// deciding what a run may contact is not this crate's call.
pub fn scan_for_disclosures(pack: &SbfbPromptAgentPack) -> Vec<ScanFinding> {
    let mut findings = Vec::new();
    match serde_json::to_value(pack) {
        Ok(value) => collect_findings(&value, "$", &mut findings),
        // An unscannable pack must not read as "nothing found" — that would be
        // the one place where a serialisation error turns into false silence.
        Err(e) => findings.push(ScanFinding {
            path: "$".to_string(),
            detail: format!("pack could not be scanned: {e}"),
        }),
    }
    findings
}

fn collect_findings(value: &serde_json::Value, path: &str, out: &mut Vec<ScanFinding>) {
    match value {
        serde_json::Value::String(s) => {
            if looks_like_materialised_secret(s) {
                out.push(ScanFinding {
                    path: path.to_string(),
                    detail: "string looks like a materialised secret".to_string(),
                });
            }
            // `contains`, not `starts_with`: an endpoint is almost never alone
            // in its string ("see https://… for setup"), and a scanner that
            // only matches at offset zero misses the realistic case entirely.
            let lowered = s.to_ascii_lowercase();
            if let Some(at) = lowered.find("http://").or_else(|| lowered.find("https://")) {
                // Report only the scheme+host, NEVER the userinfo, path, query
                // or fragment: a token rides in `?token=…` as easily as in
                // `user:pass@`, so the only safe finding echoes neither (found
                // by external cross-verification round 4). The full string is
                // still in the pack the caller already holds; the finding's
                // job is to POINT, not to reproduce.
                let endpoint: &str = s[at..]
                    .split(|c: char| c.is_whitespace())
                    .next()
                    .unwrap_or("");
                out.push(ScanFinding {
                    path: path.to_string(),
                    detail: format!("declares an endpoint at {}", scheme_and_host(endpoint)),
                });
            }
        }
        serde_json::Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                collect_findings(item, &format!("{path}[{index}]"), out);
            }
        }
        serde_json::Value::Object(members) => {
            for (key, member) in members {
                collect_findings(member, &format!("{path}.{key}"), out);
            }
        }
        _ => {}
    }
}

/// Reduce a URL to `scheme://host` — dropping userinfo, port, path, query and
/// fragment — so a finding can name WHERE without reproducing a secret carried
/// anywhere in the URL. Bounded by [`MAX_REPORTED_ENDPOINT_CHARS`] as a last
/// resort against a pathological host.
fn scheme_and_host(endpoint: &str) -> String {
    let Some(scheme_end) = endpoint.find("://") else {
        return "<non-url endpoint>".to_string();
    };
    let authority_start = scheme_end + 3;
    let authority = &endpoint[authority_start..];
    // Authority ends at the first path/query/fragment delimiter.
    let authority = authority.split(['/', '?', '#']).next().unwrap_or("");
    // Host is what follows the last '@' (drop userinfo) and precedes ':' (drop port).
    let host = authority.rsplit('@').next().unwrap_or(authority);
    let host = host.split(':').next().unwrap_or(host);
    // A hostname can itself be a smuggled credential (sk-….evil.com); a
    // credential-shaped host is withheld like any other secret would be.
    let host = if host.is_empty() {
        "<host>"
    } else if looks_like_materialised_secret(host) {
        "<credential-shaped host withheld>"
    } else {
        host
    };
    endpoint[..scheme_end]
        .chars()
        .chain("://".chars())
        .chain(host.chars())
        .take(MAX_REPORTED_ENDPOINT_CHARS)
        .collect()
}

/// Member names the pack format RESERVES for presentation, at every depth.
///
/// This is a reservation, not an observation: a document processed by the two
/// views MUST NOT use these names for meaning-bearing members, exactly as a
/// language reserves keywords. A member named `width` is stripped from the
/// semantic view wherever it appears — by contract, not by accident
/// (`tests::presentation_names_are_reserved_by_contract` pins that behaviour).
/// The alternative — a canonical graph labelling that decides semantics
/// structurally — would have to be defined, bounded and defended against
/// poison graphs; reserved names plus stable ids pinned by the pack and the
/// total member order JCS already gives make invariance trivial instead.
pub const PRESENTATION_KEYS: [&str; 10] = [
    "layout",
    "position",
    "x",
    "y",
    "width",
    "height",
    "color",
    "collapsed",
    "zoom",
    "viewport",
];

/// The meaning of a document: everything except [`PRESENTATION_KEYS`].
///
/// Feeding this to [`digest_for`] is what makes `graph_semantic` invariant to
/// moving a node on a canvas while still changing when a prompt changes.
pub fn graph_semantic_view(document: &serde_json::Value) -> serde_json::Value {
    match document {
        serde_json::Value::Object(members) => {
            let kept = members
                .iter()
                .filter(|(key, _)| !PRESENTATION_KEYS.contains(&key.as_str()))
                .map(|(key, value)| (key.clone(), graph_semantic_view(value)))
                .collect::<serde_json::Map<_, _>>();
            serde_json::Value::Object(kept)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(graph_semantic_view).collect())
        }
        other => other.clone(),
    }
}

/// The complement of [`graph_semantic_view`]: only the presentation members,
/// each keyed by its full path encoded as a JSON ARRAY of segments (object
/// keys as strings, array positions as integers).
///
/// The encoding is INJECTIVE — `["a","b","x"]` and `["a.b","x"]` are distinct
/// keys — which a dot-joined textual path is not: with dotted keys, two
/// different documents collapsed onto one path and one entry silently
/// overwrote the other, giving two distinct documents the same presentation
/// digest (found by external cross-verification, pinned by
/// `tests::presentation_view_is_injective_on_ambiguous_keys`). The keys also
/// give the projection a deterministic order for digesting; they are still
/// not an inverse mapping of the document, only a faithful index of its
/// presentation members.
pub fn presentation_view(document: &serde_json::Value) -> serde_json::Value {
    let mut found = serde_json::Map::new();
    collect_presentation(document, &mut Vec::new(), &mut found);
    serde_json::Value::Object(found)
}

fn collect_presentation(
    value: &serde_json::Value,
    path: &mut Vec<serde_json::Value>,
    out: &mut serde_json::Map<String, serde_json::Value>,
) {
    match value {
        serde_json::Value::Object(members) => {
            for (key, member) in members {
                path.push(serde_json::Value::String(key.clone()));
                if PRESENTATION_KEYS.contains(&key.as_str()) {
                    let encoded = serde_json::to_string(&serde_json::Value::Array(path.clone()))
                        .expect("path segments serialise");
                    out.insert(encoded, member.clone());
                } else {
                    collect_presentation(member, path, out);
                }
                path.pop();
            }
        }
        serde_json::Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                path.push(serde_json::Value::from(index));
                collect_presentation(item, path, out);
                path.pop();
            }
        }
        _ => {}
    }
}

/// Decode a lowercase or uppercase hex string.
fn hex_decode(input: &str) -> Option<Vec<u8>> {
    if !input.len().is_multiple_of(2) {
        return None;
    }
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(input.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let hi = (pair[0] as char).to_digit(16)?;
        let lo = (pair[1] as char).to_digit(16)?;
        out.push((hi * 16 + lo) as u8);
    }
    Some(out)
}

/// Hex-encode bytes, lowercase.
pub fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// BLAKE3 hex digest of arbitrary bytes.
pub fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// BLAKE3 hex digest of a value's canonical form under a digest kind.
///
/// The kind is mixed in as a context prefix, so the same value under two kinds
/// yields two digests and a `presentation` digest can never be replayed as a
/// `graph_semantic` one. The prefix construction is injective because the six
/// [`PackDigestKind::field`] names are pairwise distinct and the surrounding
/// literals are fixed — no two kinds can collide on
/// `sbfb-pack-digest-<field>-v1`.
///
/// # Errors
///
/// Propagates the canonicalisation errors of [`canonical_bytes`] — including
/// the `null` rejection, which is what keeps a document with a nested
/// non-finite float from digesting at all.
pub fn digest_for(kind: PackDigestKind, value: &serde_json::Value) -> Result<String, PackError> {
    let mut domain = b"sbfb-pack-digest-".to_vec();
    domain.extend_from_slice(kind.field().as_bytes());
    domain.push(b'-');
    domain.extend_from_slice(b"v1");
    Ok(blake3_hex(&canonical_bytes(value, &domain)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey, Verifier};

    /// A deterministic signing key. Tests never need entropy, and a fixed key
    /// keeps every golden reproducible.
    pub(super) fn signing_key() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    /// Another one, for the wrong-key case.
    fn other_key() -> SigningKey {
        SigningKey::from_bytes(&[9u8; 32])
    }

    fn sample_digest(seed: u8) -> String {
        blake3_hex(&[seed])
    }

    /// A pack that passes every rule — the baseline each hostile variant
    /// mutates away from. Its `content` digest is COMPUTED from its lock, the
    /// same way the verifier recomputes it.
    pub(super) fn valid_pack() -> SbfbPromptAgentPack {
        let lock = PackLock {
            entries: vec![
                PackLockEntry {
                    path: "prompts/reviewer.md".to_string(),
                    blake3: sample_digest(11),
                    size_bytes: 64,
                },
                PackLockEntry {
                    path: "workflow/workflow.json".to_string(),
                    blake3: sample_digest(10),
                    size_bytes: 128,
                },
            ],
        };
        let content = content_digest_of_lock(&lock).expect("lock digests");
        SbfbPromptAgentPack {
            format_version: PACK_FORMAT_VERSION,
            pack_revision: 1,
            name: "sbfb.example.pack".to_string(),
            description: "A minimal well-formed pack".to_string(),
            digests: PackDigests {
                content,
                graph_semantic: sample_digest(2),
                presentation: sample_digest(3),
                compiled_plan: sample_digest(4),
                evaluation: sample_digest(5),
                publication: sample_digest(6),
            },
            lock,
            provenance: PackProvenance {
                subject: vec![PackSubject {
                    name: "sbfb.example.pack".to_string(),
                    digest: PackDigestSet {
                        blake3: sample_digest(20),
                    },
                }],
                builder_id: "sbfb-factory".to_string(),
            },
            requested_capabilities: vec!["storage_get".to_string()],
            secret_refs: vec!["llm.api_key".to_string()],
        }
    }

    /// Wrap a payload value in a signed envelope under `domain`, using `key`.
    pub(super) fn sign_envelope(
        payload: &serde_json::Value,
        domain: &[u8],
        key: &SigningKey,
    ) -> Vec<u8> {
        let signed_bytes = canonical_bytes(payload, domain).expect("payload canonicalises");
        let signature = key.sign(&signed_bytes);
        let envelope = serde_json::json!({
            "payload": payload,
            "signature": hex_encode(&signature.to_bytes()),
        });
        serde_json::to_vec(&envelope).expect("envelope serialises")
    }

    /// The valid envelope goes through the TYPED [`SignedPack`] struct, so the
    /// declared envelope type is exercised by the same path that produces the
    /// accepted control bytes (it would otherwise be dead weight in the
    /// schema).
    fn valid_envelope() -> Vec<u8> {
        let payload = valid_pack();
        let payload_value = serde_json::to_value(&payload).expect("pack to value");
        let signed_bytes =
            canonical_bytes(&payload_value, PACK_SIGNING_DOMAIN).expect("canonicalises");
        let signed = SignedPack {
            payload,
            signature: hex_encode(&signing_key().sign(&signed_bytes).to_bytes()),
        };
        serde_json::to_vec(&signed).expect("envelope serialises")
    }

    #[test]
    fn valid_pack_round_trips() {
        let raw = valid_envelope();
        let verified =
            verify_signed_pack(&raw, &signing_key().verifying_key()).expect("valid pack verifies");
        assert_eq!(verified, valid_pack());
    }

    /// The mirror of `DOMAIN_WORKFLOW_PACK_V1` is PROVEN, not trusted.
    ///
    /// This crate cannot depend on `nexus-core-rs` (charter: no Tokio, no
    /// network, no database — and that crate pulls iroh), so the domain bytes
    /// exist in two places. The test reads the declaring file from disk, the
    /// same arrangement `factory_profile` uses for its own sources: the crate
    /// stays pure in production, and the TEST is what touches the disk.
    #[test]
    fn pack_signing_domain_mirrors_canonical_rs() {
        let canonical_rs = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("nexus-core-rs")
            .join("src")
            .join("canonical.rs");
        let source = std::fs::read_to_string(&canonical_rs)
            .unwrap_or_else(|e| panic!("read {}: {e}", canonical_rs.display()));

        let needle = "pub const DOMAIN_WORKFLOW_PACK_V1: &[u8] = b\"";
        assert_eq!(
            source.matches(needle).count(),
            1,
            "the declaration must exist EXACTLY once — a second literal (or a              commented copy) would make this mirror check ambiguous"
        );
        let start = source
            .find(needle)
            .expect("DOMAIN_WORKFLOW_PACK_V1 must be declared in nexus-core-rs::canonical")
            + needle.len();
        let line_start = source[..start].rfind('\n').map_or(0, |i| i + 1);
        assert!(
            source[line_start..].trim_start().starts_with("pub const"),
            "the declaration line must be code, not a comment"
        );
        let end = start
            + source[start..]
                .find('"')
                .expect("domain literal must be terminated");
        let declared = &source[start..end];

        assert_eq!(
            declared.as_bytes(),
            PACK_SIGNING_DOMAIN,
            "PACK_SIGNING_DOMAIN drifted from nexus-core-rs::canonical::DOMAIN_WORKFLOW_PACK_V1"
        );
    }

    /// The six digests are stable against a committed golden, and each kind
    /// yields a DIFFERENT digest for the same input — digest kinds are
    /// domain-separated, so a presentation digest cannot be replayed as a
    /// semantic one.
    #[test]
    fn six_digests_stable() {
        let document = serde_json::json!({
            "nodes": [
                {"id": "n1", "prompt": "review the diff", "position": {"x": 10, "y": 20}},
                {"id": "n2", "prompt": "summarise", "position": {"x": 30, "y": 40}}
            ],
            "edges": [{"from": "n1", "to": "n2"}],
            "layout": {"zoom": 2}
        });

        let mut digests = std::collections::BTreeMap::new();
        for kind in PackDigestKind::ALL {
            let value = match kind {
                PackDigestKind::GraphSemantic => graph_semantic_view(&document),
                PackDigestKind::Presentation => presentation_view(&document),
                _ => document.clone(),
            };
            digests.insert(
                kind.field().to_string(),
                digest_for(kind, &value).expect("digest"),
            );
        }

        let distinct: std::collections::BTreeSet<_> = digests.values().collect();
        assert_eq!(
            distinct.len(),
            PackDigestKind::ALL.len(),
            "each digest kind must be domain-separated: {digests:?}"
        );

        let golden_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("pack_digests.golden.json");
        let live = serde_json::to_string_pretty(&digests).expect("serialise digests");

        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            std::fs::write(&golden_path, format!("{live}\n")).expect("write digest golden");
            return;
        }

        // STRICT: a missing golden is a FAILURE, never a self-write that
        // returns green. Deleting the file must not be a way to make this
        // drift gate quietly disappear.
        let snapshot = std::fs::read_to_string(&golden_path).unwrap_or_else(|e| {
            panic!("pack_digests.golden.json must be committed ({e}); regenerate with UPDATE_SNAPSHOTS=1")
        });
        assert_eq!(
            live.trim(),
            snapshot.trim(),
            "pack digest drift — regenerate with UPDATE_SNAPSHOTS=1 if intended"
        );
    }

    /// Moving a node does not change what the workflow MEANS; changing a
    /// prompt does. This is the property the semantic/presentation split
    /// exists for.
    #[test]
    fn graph_semantic_ignores_presentation_but_not_meaning() {
        let base = serde_json::json!({
            "nodes": [{"id": "n1", "prompt": "review the diff", "position": {"x": 10, "y": 20}}],
            "layout": {"zoom": 1}
        });
        let moved = serde_json::json!({
            "nodes": [{"id": "n1", "prompt": "review the diff", "position": {"x": 999, "y": 888}}],
            "layout": {"zoom": 3}
        });
        let reworded = serde_json::json!({
            "nodes": [{"id": "n1", "prompt": "review the diff CAREFULLY", "position": {"x": 10, "y": 20}}],
            "layout": {"zoom": 1}
        });

        let semantic = |doc: &serde_json::Value| {
            digest_for(PackDigestKind::GraphSemantic, &graph_semantic_view(doc)).expect("digest")
        };
        let presentation = |doc: &serde_json::Value| {
            digest_for(PackDigestKind::Presentation, &presentation_view(doc)).expect("digest")
        };

        assert_eq!(
            semantic(&base),
            semantic(&moved),
            "moving a node must not change the semantic digest"
        );
        assert_ne!(
            presentation(&base),
            presentation(&moved),
            "moving a node MUST change the presentation digest"
        );
        assert_ne!(
            semantic(&base),
            semantic(&reworded),
            "rewording a prompt MUST change the semantic digest"
        );
    }

    /// Cross-language parity: the digests TypeScript must reproduce byte for
    /// byte, over adverse RFC 8785 vectors. The mirror
    /// `web/src/workflow/__tests__/pack_digest.test.ts` reads THESE SAME TWO
    /// FILES and asserts the same values.
    #[test]
    fn jcs_cross_lang_parity() {
        let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let vectors: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(base.join("pack_jcs_vectors.json")).expect("read vectors"),
        )
        .expect("vectors are JSON");

        let domain = vectors["domain"].as_str().expect("domain").to_string();
        assert_eq!(
            domain.as_bytes(),
            PACK_SIGNING_DOMAIN,
            "the parity vectors must exercise the real pack domain"
        );

        let mut live = std::collections::BTreeMap::new();
        for vector in vectors["vectors"].as_array().expect("vectors array") {
            let name = vector["name"].as_str().expect("vector name").to_string();
            let bytes = canonical_bytes(&vector["value"], domain.as_bytes()).expect("canonical");
            // The golden pins the canonical TEXT as well as the digest: the
            // claim under test is byte-identical canonicalisation, and a
            // digest alone cannot show WHERE two implementations diverge.
            let jcs_text = std::str::from_utf8(&bytes[domain.len() + 1..])
                .expect("JCS output is UTF-8")
                .to_string();
            live.insert(
                name,
                serde_json::json!({ "digest": blake3_hex(&bytes), "canonical": jcs_text }),
            );
        }
        assert!(
            live.len() >= 10,
            "the parity corpus must not shrink silently"
        );

        let golden_path = base.join("pack_jcs_digests.golden.json");
        let rendered = serde_json::to_string_pretty(&live).expect("serialise");

        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            std::fs::write(&golden_path, format!("{rendered}\n")).expect("write parity golden");
            return;
        }
        let snapshot_str = std::fs::read_to_string(&golden_path).unwrap_or_else(|e| {
            panic!("pack_jcs_digests.golden.json must be committed ({e}); UPDATE_SNAPSHOTS=1")
        });
        // Value comparison, not string: `preserve_order` flips member order
        // between `-p` and `--workspace` builds (see
        // `signed_pack_schema_snapshot`). The parity-critical bytes live
        // INSIDE the "canonical" strings, which serde_jcs makes
        // build-mode-independent.
        let live_value = serde_json::to_value(&live).expect("live to value");
        let snapshot: serde_json::Value =
            serde_json::from_str(&snapshot_str).expect("parity golden is valid JSON");
        assert_eq!(
            live_value, snapshot,
            "JCS parity drift — the TypeScript mirror reads this same golden"
        );
    }

    /// The `null` guard changes real behaviour — this test shows what would be
    /// signed WITHOUT it (JCS happily emits the `null`), then shows the door
    /// refusing, with the exact path. The COLLISION story lives in the
    /// non-finite-float test below: that is where two different inputs would
    /// otherwise become one canonical `null`.
    #[test]
    fn canonical_bytes_rejects_explicit_nulls_with_path() {
        let absent = serde_json::json!({"a": 1});
        let explicit_null = serde_json::json!({"a": 1, "b": null});

        // What the guard is up against: serde_jcs itself accepts the null.
        let unguarded = serde_jcs::to_vec(&explicit_null).expect("serde_jcs accepts null");
        assert!(
            String::from_utf8(unguarded).expect("utf8").contains("null"),
            "without the guard, the null would be signed as-is"
        );

        assert!(
            canonical_bytes(&absent, PACK_SIGNING_DOMAIN).is_ok(),
            "a null-free document canonicalises"
        );
        match canonical_bytes(&explicit_null, PACK_SIGNING_DOMAIN) {
            Err(PackError::NonRepresentable { path }) => assert_eq!(path, "$.b"),
            other => panic!("explicit null must be refused, got {other:?}"),
        }
    }

    /// A non-finite float NESTED in a structure is the case `serde_jcs` does
    /// NOT reject — it collapses to `null` — which is why the guard has to run
    /// on the value tree. Measured here, not taken from documentation.
    #[test]
    fn nested_non_finite_float_is_refused_not_collapsed() {
        #[derive(Serialize)]
        struct Nested {
            ratio: f64,
        }
        let nested = Nested {
            ratio: f64::INFINITY,
        };

        // First: show the collapse really happens at the serde layer.
        let as_value = serde_json::to_value(&nested).expect("to_value tolerates it");
        assert_eq!(
            as_value["ratio"],
            serde_json::Value::Null,
            "a nested non-finite float collapses to null — this is WHY the guard exists"
        );

        // Then: show the door refuses it.
        match canonical_bytes(&nested, PACK_SIGNING_DOMAIN) {
            Err(PackError::NonRepresentable { path }) => assert_eq!(path, "$.ratio"),
            other => panic!("nested non-finite float must be refused, got {other:?}"),
        }
    }

    /// Completeness: iterating the canonical enumeration is what makes a
    /// dropped digest impossible to miss, and every kind maps to a distinct
    /// field name.
    #[test]
    fn digest_kinds_are_exhaustive_and_named() {
        let names: std::collections::BTreeSet<_> =
            PackDigestKind::ALL.iter().map(|k| k.field()).collect();
        assert_eq!(names.len(), 6, "six distinct digest field names");

        let pack = valid_pack();
        for kind in PackDigestKind::ALL {
            assert_eq!(
                pack.digests.get(kind).len(),
                BLAKE3_HEX_LEN,
                "digest {} must be readable through the canonical enumeration",
                kind.field()
            );
        }

        for kind in PackDigestKind::ALL {
            let mut broken = valid_pack();
            match kind {
                PackDigestKind::Content => broken.digests.content = "abc".to_string(),
                PackDigestKind::GraphSemantic => broken.digests.graph_semantic = "abc".to_string(),
                PackDigestKind::Presentation => broken.digests.presentation = "abc".to_string(),
                PackDigestKind::CompiledPlan => broken.digests.compiled_plan = "abc".to_string(),
                PackDigestKind::Evaluation => broken.digests.evaluation = "abc".to_string(),
                PackDigestKind::Publication => broken.digests.publication = "abc".to_string(),
            }
            let err = validate_pack(&broken).expect_err("a truncated digest is refused");
            assert!(
                matches!(err, PackError::Semantic(ref m) if m.contains(kind.field())),
                "the refusal must name the offending digest, got {err:?}"
            );
        }
    }

    /// Cross-domain, BOTH directions — the necessary and the sufficient half.
    ///
    /// Necessary: the canonical bytes differ under two domains. Sufficient: a
    /// signature minted under another domain does not verify as a pack. The
    /// first alone would not prove the verifier actually binds the domain.
    /// This is the one test that needs the domain-injectable door, which is
    /// exactly why `verify_signed_pack_under_domain` is private.
    #[test]
    fn cross_domain_signature_rejected() {
        // The corpus row for this case is exercised HERE (the in-module test
        // matrix cannot inject a foreign domain); read its declared
        // expectation mechanically so the row is data, not decoration.
        let corpus_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("pack_hostile_corpus.json");
        let corpus: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&corpus_path).expect("read corpus"))
                .expect("corpus is JSON");
        let declared_expect = corpus["post_signature_entries"]
            .as_array()
            .expect("post_signature_entries")
            .iter()
            .find(|e| e["name"] == "cross-domain-signature")
            .expect("cross-domain-signature row must exist")["expect"]
            .as_str()
            .expect("expect");
        assert_eq!(declared_expect, "SignatureInvalid");

        let payload = serde_json::to_value(valid_pack()).expect("to value");
        let other_domain: &[u8] = b"nexus-curator-list-v1";

        let ours = canonical_bytes(&payload, PACK_SIGNING_DOMAIN).expect("ours");
        let theirs = canonical_bytes(&payload, other_domain).expect("theirs");
        assert_ne!(
            ours, theirs,
            "necessary: two domains produce different pre-images"
        );

        let raw = sign_envelope(&payload, other_domain, &signing_key());
        let err = verify_signed_pack_under_domain(
            &raw,
            PACK_SIGNING_DOMAIN,
            &signing_key().verifying_key(),
        )
        .expect_err("sufficient: a foreign-domain signature must not verify");
        assert!(matches!(err, PackError::SignatureInvalid(_)), "got {err:?}");

        // And the PUBLIC door gives the same refusal — it is the same door.
        let err = verify_signed_pack(&raw, &signing_key().verifying_key())
            .expect_err("public door refuses too");
        assert!(matches!(err, PackError::SignatureInvalid(_)), "got {err:?}");
    }

    /// A pack signed by another key does not verify: this function answers
    /// "signed by THAT key", and nothing else.
    #[test]
    fn wrong_key_rejected() {
        let raw = valid_envelope();
        let err = verify_signed_pack(&raw, &other_key().verifying_key())
            .expect_err("wrong key must be refused");
        assert!(matches!(err, PackError::SignatureInvalid(_)), "got {err:?}");
    }

    /// A small-order (weak) verifying key is refused, so "the holder of the
    /// key signed these bytes" cannot be forged. For the identity point, the
    /// signature `R = identity, S = 0` satisfies the plain verification
    /// equation for ANY message with no private key — the exact class
    /// `verify_strict` + the `is_weak` guard reject (external cross-verifier,
    /// round 4). The payload is otherwise perfectly valid, so ONLY the key
    /// check stands between this and acceptance.
    #[test]
    fn weak_key_with_forged_signature_rejected() {
        // The Ed25519 identity point, compressed: y = 1, sign bit 0.
        let mut identity = [0u8; 32];
        identity[0] = 1;
        let weak_key = VerifyingKey::from_bytes(&identity).expect("identity is a valid encoding");
        assert!(weak_key.is_weak(), "the identity point must read as weak");

        // R = identity point, S = 0 — the classic universal forgery.
        let mut forged = [0u8; 64];
        forged[0] = 1;
        let signature = Signature::from_bytes(&forged);

        let payload = serde_json::to_value(valid_pack()).expect("to value");
        let envelope = serde_json::json!({
            "payload": payload,
            "signature": hex_encode(&signature.to_bytes()),
        });
        let raw = serde_json::to_vec(&envelope).expect("serialise");

        let err = verify_signed_pack(&raw, &weak_key).expect_err("weak key must be refused");
        assert!(
            matches!(err, PackError::SignatureInvalid(ref m) if m.contains("weak")),
            "the weak-key refusal must name itself, got {err:?}"
        );

        // And plain (non-strict) verify would have ACCEPTED this forgery —
        // that is what makes the guard load-bearing rather than decorative.
        assert!(
            weak_key.verify(b"anything at all", &signature).is_ok(),
            "sanity: the forgery does satisfy the non-strict equation"
        );
    }

    /// Each bound admits its cap value EXACTLY: the refusal starts at cap+1,
    /// not at the cap — otherwise every bound test above only proved "some
    /// smaller limit exists".
    #[test]
    fn bounds_admit_their_exact_cap() {
        // Depth: a chain nested exactly MAX_JSON_DEPTH deep.
        let mut at_cap = serde_json::json!(1);
        for _ in 0..MAX_JSON_DEPTH {
            at_cap = serde_json::json!({ "d": at_cap });
        }
        enforce_bounds(&at_cap, "$", 0).expect("depth == cap is admitted");

        // Cardinality: exactly MAX_COLLECTION_LEN members.
        let members: serde_json::Map<String, serde_json::Value> = (0..MAX_COLLECTION_LEN)
            .map(|i| (format!("k{i}"), serde_json::json!(1)))
            .collect();
        enforce_bounds(&serde_json::Value::Object(members), "$", 0)
            .expect("cardinality == cap is admitted");

        // String length: exactly MAX_STRING_LEN bytes.
        let s = serde_json::json!("a".repeat(MAX_STRING_LEN));
        enforce_bounds(&s, "$", 0).expect("string == cap is admitted");

        // Integers: the I-JSON limits themselves are admitted, both signs.
        let ints = serde_json::json!([MAX_SAFE_INTEGER, -MAX_SAFE_INTEGER]);
        enforce_bounds(&ints, "$", 0).expect("the exact I-JSON limits are admitted");
    }

    /// The reservation is a CONTRACT, and this is what it costs: a member
    /// named `width` is stripped from the semantic view even where it was
    /// meant semantically. A document that needs a semantic `width` must not
    /// feed these views — that is what "reserved" means, and the pinned
    /// behaviour here is what the doc of [`PRESENTATION_KEYS`] points at.
    #[test]
    fn presentation_names_are_reserved_by_contract() {
        let with_semantic_width = serde_json::json!({"node": {"width": "load-bearing"}});
        let without = serde_json::json!({"node": {}});

        let semantic = |doc: &serde_json::Value| {
            digest_for(PackDigestKind::GraphSemantic, &graph_semantic_view(doc)).expect("digest")
        };
        assert_eq!(
            semantic(&with_semantic_width),
            semantic(&without),
            "a reserved name is stripped wherever it appears — by contract"
        );
        // The stripped member is not LOST: it lands in the presentation view.
        let projected = presentation_view(&with_semantic_width);
        assert_eq!(
            projected[r#"["node","width"]"#],
            serde_json::json!("load-bearing"),
            "the reserved member surfaces in the presentation projection"
        );
    }

    /// The scanner reports; it never decides. It is not wired into
    /// [`verify_signed_pack`], and this test pins that separation.
    #[test]
    fn scanner_is_advisory_not_a_gate() {
        let mut pack = valid_pack();
        pack.description = "see https://example.com/webhook for setup".to_string();

        let findings = scan_for_disclosures(&pack);
        assert!(
            findings.iter().any(|f| f.detail.contains("endpoint")),
            "the scanner notices a declared endpoint: {findings:?}"
        );
        // …and it stays quiet about the pack's OWN digests. A scanner that
        // flags nine BLAKE3 strings in a minimal pack teaches its reader to
        // ignore it, which is how an advisory signal becomes worthless.
        assert!(
            findings
                .iter()
                .all(|f| !f.detail.contains("materialised secret")),
            "no false positive on the pack's own digests: {findings:?}"
        );

        let payload = serde_json::to_value(&pack).expect("to value");
        let raw = sign_envelope(&payload, PACK_SIGNING_DOMAIN, &signing_key());
        verify_signed_pack(&raw, &signing_key().verifying_key())
            .expect("an advisory finding must not block verification");
    }

    /// The finding must NEVER reproduce a secret carried anywhere in a URL —
    /// userinfo, port, path, query or fragment (external cross-verifier,
    /// round 4: `?token=…` leaks as easily as `user:pass@`). It reports
    /// scheme+host and nothing more.
    #[test]
    fn scanner_never_echoes_url_secrets() {
        for (field, secret) in [
            (
                "https://user:sk-userinfo-secret@example.com/x",
                "sk-userinfo-secret",
            ),
            (
                "see https://api.example/callback?token=sk-query-secret",
                "sk-query-secret",
            ),
            (
                "https://api.example/p#sk-fragment-secret",
                "sk-fragment-secret",
            ),
            (
                "https://api.example:8443/deep/path/that/is/very/long?a=sk-long-secret",
                "sk-long-secret",
            ),
        ] {
            let mut pack = valid_pack();
            pack.description = field.to_string();
            let findings = scan_for_disclosures(&pack);
            let endpoint_findings: Vec<_> = findings
                .iter()
                .filter(|f| f.detail.contains("endpoint"))
                .collect();
            assert!(
                !endpoint_findings.is_empty(),
                "the endpoint must still be noticed for '{field}'"
            );
            for f in &endpoint_findings {
                assert!(
                    !f.detail.contains(secret),
                    "finding '{}' leaked the secret from '{field}'",
                    f.detail
                );
                assert!(
                    f.detail.contains("example.com") || f.detail.contains("api.example"),
                    "finding '{}' should still name the host of '{field}'",
                    f.detail
                );
            }
        }
    }

    /// The `content ↔ lock` binding is proven against a LITERAL golden and by
    /// mutating each lock facet independently, so baseline and verifier cannot
    /// share one wrong helper (external cross-verifier, round 4): a bug in
    /// `content_digest_of_lock` would move the golden here, not hide behind a
    /// matching baseline.
    #[test]
    fn content_digest_of_lock_is_pinned_and_facet_sensitive() {
        let base = valid_pack().lock;
        let golden = content_digest_of_lock(&base).expect("digest");
        // Literal golden: if the helper changes shape, THIS line fails, not a
        // baseline computed by the same helper.
        assert_eq!(
            golden, "9da9e1350705e9619f9afcd5d2daa0fecc44c7d82e6efc3ec527eeff33a2e2c1",
            "content digest golden drift — UPDATE the literal only for an intended change"
        );

        // Each facet moves the digest, independently.
        let mut path_changed = base.clone();
        path_changed.entries[0].path = "prompts/other.md".to_string();
        assert_ne!(
            content_digest_of_lock(&path_changed).unwrap(),
            golden,
            "path"
        );

        let mut blake_changed = base.clone();
        blake_changed.entries[0].blake3 = blake3_hex(b"different");
        assert_ne!(
            content_digest_of_lock(&blake_changed).unwrap(),
            golden,
            "blake3"
        );

        let mut size_changed = base.clone();
        size_changed.entries[0].size_bytes += 1;
        assert_ne!(
            content_digest_of_lock(&size_changed).unwrap(),
            golden,
            "size"
        );

        let mut reordered = base.clone();
        reordered.entries.reverse();
        assert_ne!(content_digest_of_lock(&reordered).unwrap(), golden, "order");

        let mut extra = base.clone();
        extra.entries.push(PackLockEntry {
            path: "zzz.txt".to_string(),
            blake3: blake3_hex(b"z"),
            size_bytes: 1,
        });
        assert_ne!(content_digest_of_lock(&extra).unwrap(), golden, "added");
    }

    /// The injectivity regression found by external cross-verification: with
    /// dot-joined textual paths, these two DISTINCT documents shared one
    /// presentation view (`$.a.b.x` collided with the literal key `a.b`
    /// followed by `x`, and `Map::insert` kept one of the two), hence one
    /// presentation digest. Segment-array keys keep them apart.
    #[test]
    fn presentation_view_is_injective_on_ambiguous_keys() {
        let doc1 = serde_json::json!({"a": {"b": {"x": 1}}, "a.b": {"x": 2}});
        let doc2 = serde_json::json!({"a": {"b": {"x": 999}}, "a.b": {"x": 2}});

        let view1 = presentation_view(&doc1);
        let view2 = presentation_view(&doc2);
        assert_ne!(
            view1, view2,
            "distinct documents must not share a presentation view"
        );

        // Both members survive, under distinct keys.
        let members = view1.as_object().expect("object");
        assert_eq!(members.len(), 2, "no entry may overwrite another: {view1}");

        let d1 = digest_for(PackDigestKind::Presentation, &view1).expect("digest");
        let d2 = digest_for(PackDigestKind::Presentation, &view2).expect("digest");
        assert_ne!(d1, d2, "the collision must not reach the digest");
    }

    /// The I-JSON integer rule of the canonicalisation door is MEASURED, not
    /// assumed: first show that serde_jcs really collapses 2^53+1 onto 2^53
    /// (two distinct values, one canonical form), then show the door refuses
    /// the value that would collide.
    #[test]
    fn canonical_door_refuses_the_measured_integer_collapse() {
        // 2^53+1 and 2^53 are DISTINCT values that serde_jcs serialises to
        // the same text (RFC 8785 goes through an IEEE double); both sit
        // beyond the I-JSON exact bound of 2^53 - 1.
        let above = serde_json::json!(9_007_199_254_740_993u64);
        let collapsed_onto = serde_json::json!(9_007_199_254_740_992u64);

        let above_jcs = serde_jcs::to_string(&above).expect("serde_jcs accepts it");
        let collapsed_jcs = serde_jcs::to_string(&collapsed_onto).expect("serde_jcs accepts it");
        assert_eq!(
            above_jcs, collapsed_jcs,
            "the collapse is real — this is WHY the door refuses the range"
        );

        for out_of_range in [&above, &collapsed_onto] {
            match canonical_bytes(out_of_range, PACK_SIGNING_DOMAIN) {
                Err(PackError::Canonicalisation(msg)) => {
                    assert!(msg.contains("I-JSON"), "refusal must name the rule: {msg}")
                }
                other => panic!("out-of-range integer must be refused, got {other:?}"),
            }
        }
        let at_limit = serde_json::json!(MAX_SAFE_INTEGER);
        assert!(
            canonical_bytes(&at_limit, PACK_SIGNING_DOMAIN).is_ok(),
            "the exact I-JSON limit itself is admitted"
        );
    }

    /// The envelope type the verifier actually receives gets its own pinned
    /// schema — outside the 7-identity registry (it is transport, not a
    /// contract identity), STRICT like every golden in this crate.
    #[test]
    fn signed_pack_schema_snapshot() {
        let live =
            serde_json::to_value(schemars::schema_for!(SignedPack)).expect("schema serialises");
        let live_pretty = serde_json::to_string_pretty(&live).expect("pretty");

        let golden_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("signed_pack.schema.json");

        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            std::fs::write(&golden_path, format!("{live_pretty}\n")).expect("write golden");
            return;
        }
        let snapshot_str = std::fs::read_to_string(&golden_path).unwrap_or_else(|e| {
            panic!("signed_pack.schema.json must be committed ({e}); UPDATE_SNAPSHOTS=1")
        });
        // Compared as VALUES, not strings: `ollama-rs -> schemars` switches
        // `serde_json/preserve_order` on in `--workspace` builds and off in
        // `-p` builds, so member ORDER differs between the two build modes
        // while the schema CONTENT does not (the engine crate's Cargo.toml
        // documents the same trap for digests). A string comparison here was
        // green in one build mode and red in the other.
        let snapshot: serde_json::Value =
            serde_json::from_str(&snapshot_str).expect("signed_pack.schema.json is valid JSON");
        assert_eq!(
            live, snapshot,
            "SignedPack envelope schema drift — UPDATE_SNAPSHOTS=1 if intended"
        );
    }

    /// `PACK_FORMAT_VERSION` is what the FRONTIER tag points at; keep the tag
    /// honest by pinning the value the format actually accepts.
    #[test]
    fn format_version_is_enforced() {
        assert_eq!(PACK_FORMAT_VERSION, 1);
        let mut pack = valid_pack();
        pack.format_version = 2;
        let err = validate_pack(&pack).expect_err("a foreign format version is refused");
        assert!(matches!(err, PackError::Semantic(_)), "got {err:?}");
    }
}

/// The hostile corpus AND the verification-order guard, in their own module so
/// the gate command `cargo nextest run -p sbfb-workflow-core import_hostile`
/// selects exactly these tests (`G-IMPORT-HOSTILE`, gate matrix row 35). The
/// ordering guard lives HERE, not in `tests`, because "signature before the
/// rich parse" is the core of what a hostile import must survive — a gate that
/// skipped it would be green while the property regressed.
#[cfg(test)]
mod import_hostile {
    use super::tests::{sign_envelope, signing_key, valid_pack};
    use super::*;

    /// THE ordering test, written to be DISCRIMINANT.
    ///
    /// The envelope carries an unknown payload field (which the rich typed
    /// parse rejects) AND a wrong signature. The signature error must win.
    /// Swapping steps (4) and (5) of `verify_signed_pack_under_domain` turns
    /// this red.
    #[test]
    fn signature_before_parse() {
        let mut payload = serde_json::to_value(valid_pack()).expect("to value");
        payload["unknown_field_the_typed_parse_rejects"] = serde_json::json!("boom");

        let envelope = serde_json::json!({
            "payload": payload,
            "signature": hex_encode(&[0u8; 64]),
        });
        let raw = serde_json::to_vec(&envelope).expect("serialise");

        let err = verify_signed_pack(&raw, &signing_key().verifying_key()).expect_err("refuse");
        assert!(
            matches!(err, PackError::SignatureInvalid(_)),
            "the signature must be checked BEFORE the typed parse; got {err:?}"
        );
    }

    fn corpus() -> serde_json::Value {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("pack_hostile_corpus.json");
        serde_json::from_str(&std::fs::read_to_string(&path).expect("read hostile corpus"))
            .expect("hostile corpus is JSON")
    }

    /// Build the entries too large or too repetitive to store literally.
    fn generate(spec: &str) -> Vec<u8> {
        let (kind, arg) = spec
            .split_once(' ')
            .expect("generator spec is '<kind> <n>'");
        let n: usize = arg.parse().expect("generator argument is a number");
        let payload = match kind {
            "array_of_len" => serde_json::json!({ "requested_capabilities": vec![1u8; n] }),
            "object_of_len" => {
                let members: serde_json::Map<String, serde_json::Value> = (0..n)
                    .map(|i| (format!("k{i}"), serde_json::json!(1)))
                    .collect();
                serde_json::Value::Object(members)
            }
            "string_of_len" => serde_json::json!({ "description": "a".repeat(n) }),
            "nested_depth" => {
                let mut value = serde_json::json!(1);
                for _ in 0..n {
                    value = serde_json::json!({ "deeper": value });
                }
                value
            }
            "lone_surrogate" => {
                // A lone UTF-16 surrogate escape is invalid JSON text; it can
                // only exist as raw bytes (the corpus file itself must stay
                // parseable, so the entry is generated, not stored).
                return br#"{"payload": {"name": "\ud800"}, "signature": "00"}"#.to_vec();
            }
            "envelope_of_bytes" => {
                // The padding alone exceeds the cap, so the size bound fires
                // before serde is handed a byte.
                return format!(
                    "{{\"payload\": {{\"description\": \"{}\"}}, \"signature\": \"00\"}}",
                    "a".repeat(n)
                )
                .into_bytes();
            }
            other => panic!("unknown generator '{other}'"),
        };
        serde_json::to_vec(&serde_json::json!({"payload": payload, "signature": "00"}))
            .expect("serialise generated envelope")
    }

    fn error_kind(err: &PackError) -> &'static str {
        match err {
            PackError::TooLarge { .. } => "TooLarge",
            PackError::MalformedEnvelope(_) => "MalformedEnvelope",
            PackError::Unbounded { .. } => "Unbounded",
            PackError::NonRepresentable { .. } => "NonRepresentable",
            PackError::Canonicalisation(_) => "Canonicalisation",
            PackError::SignatureInvalid(_) => "SignatureInvalid",
            PackError::SchemaViolation(_) => "SchemaViolation",
            PackError::Semantic(_) => "Semantic",
        }
    }

    /// Every committed hostile entry is refused, with the refusal it claims.
    ///
    /// Asserting the VARIANT (not merely `is_err`) is what makes this a proof
    /// of the ORDER: each of these entries carries a junk 1-byte signature, so
    /// an entry that survived its bound would come back `SignatureInvalid` and
    /// fail its expectation here. Entries that share a variant additionally
    /// pin WHICH rule fired through `expect_detail_contains` — four `Unbounded`
    /// entries would otherwise be interchangeable.
    #[test]
    fn hostile_corpus_refused() {
        let corpus = corpus();
        let entries = corpus["entries"].as_array().expect("entries array");
        assert!(entries.len() >= 17, "the corpus must not shrink silently");

        // Names must be unique across BOTH sections: duplicated rows could
        // otherwise pad the floors without adding coverage.
        let mut all_names = std::collections::BTreeSet::new();
        for section in ["entries", "post_signature_entries"] {
            for e in corpus[section].as_array().expect(section) {
                let name = e["name"].as_str().expect("name");
                assert!(
                    all_names.insert(name.to_string()),
                    "duplicate corpus name '{name}'"
                );
            }
        }

        let verifying = signing_key().verifying_key();
        for entry in entries {
            let name = entry["name"].as_str().expect("entry name");
            let expect = entry["expect"].as_str().expect("expected variant");
            let raw = match (entry.get("raw"), entry.get("raw_generator")) {
                (Some(raw), None) => raw.as_str().expect("raw is a string").as_bytes().to_vec(),
                (None, Some(spec)) => generate(spec.as_str().expect("generator is a string")),
                _ => panic!("entry '{name}' needs exactly one of raw / raw_generator"),
            };

            let err = verify_signed_pack(&raw, &verifying).unwrap_err_or_else(name);
            assert_eq!(
                error_kind(&err),
                expect,
                "hostile entry '{name}' was refused with the wrong error: {err:?}"
            );
            if let Some(needle) = entry.get("expect_detail_contains").and_then(|v| v.as_str()) {
                let rendered = err.to_string();
                assert!(
                    rendered.contains(needle),
                    "hostile entry '{name}': refusal '{rendered}' does not name '{needle}'"
                );
            }
        }
    }

    /// Small helper so a hostile entry that is ACCEPTED names itself in the
    /// panic instead of printing a bare `Ok(..)`.
    trait UnwrapErrNamed {
        fn unwrap_err_or_else(self, name: &str) -> PackError;
    }

    impl UnwrapErrNamed for Result<SbfbPromptAgentPack, PackError> {
        fn unwrap_err_or_else(self, name: &str) -> PackError {
            match self {
                Ok(_) => panic!("hostile entry '{name}' was ACCEPTED"),
                Err(e) => e,
            }
        }
    }

    /// The corpus is not vacuous: the SAME door accepts a well-formed pack.
    /// Without this, every hostile assertion above would be satisfied by a
    /// verifier that refuses everything. It intentionally repeats what
    /// `tests::valid_pack_round_trips` proves — the point is that the ACCEPT
    /// control runs INSIDE the `import_hostile` gate filter, so the gate can
    /// never go green on a refuse-everything verifier.
    #[test]
    fn hostile_corpus_is_not_vacuous() {
        let payload = serde_json::to_value(valid_pack()).expect("to value");
        let raw = sign_envelope(&payload, PACK_SIGNING_DOMAIN, &signing_key());
        verify_signed_pack(&raw, &signing_key().verifying_key())
            .expect("the control pack must be ACCEPTED by the same door");
    }

    /// The cosign failure mode: a GENUINE signature over a body that fails a
    /// later check is a terminal refusal, never a degraded acceptance.
    ///
    /// Every case is signed for real, with the real key, under the real
    /// domain. Two disciplines keep each case green for the RIGHT reason
    /// (external cross-verification found the previous version green for the
    /// wrong one): (1) lock-mutating cases RECOMPUTE `digests.content` on the
    /// mutated lock — otherwise every one of them would also fail the
    /// content-binding check, and deleting the rule under test would leave
    /// the case green; (2) each case asserts a MESSAGE FRAGMENT naming the
    /// rule that fired, not only the error variant.
    #[test]
    fn valid_signature_over_hostile_body_is_terminal() {
        let verifying = signing_key().verifying_key();

        fn rebind_content(mut pack: SbfbPromptAgentPack) -> SbfbPromptAgentPack {
            pack.digests.content =
                content_digest_of_lock(&pack.lock).expect("mutated lock digests");
            pack
        }

        let mut cases: Vec<(&str, serde_json::Value, &str, &str)> = Vec::new();

        let mut unknown = serde_json::to_value(valid_pack()).unwrap();
        unknown["extra"] = serde_json::json!(1);
        cases.push((
            "valid-signature-over-unknown-field",
            unknown,
            "SchemaViolation",
            "unknown field",
        ));

        let mut version = valid_pack();
        version.format_version = 99;
        cases.push((
            "valid-signature-over-wrong-format-version",
            serde_json::to_value(version).unwrap(),
            "Semantic",
            "format_version",
        ));

        for (name, bad_path, fragment) in [
            (
                "valid-signature-over-zip-slip-path",
                "../../etc/passwd",
                "relative segment",
            ),
            (
                "valid-signature-over-backslash-path",
                "workflow\\win.json",
                "backslash",
            ),
            (
                "valid-signature-over-absolute-path",
                "/etc/passwd",
                "absolute lock path",
            ),
            (
                "valid-signature-over-drive-letter-path",
                "C:/windows/system32",
                "drive or stream",
            ),
            (
                "valid-signature-over-reserved-device-name",
                "workflow/NUL.json",
                "reserved device name",
            ),
            (
                "valid-signature-over-trailing-dot-segment",
                "workflow./workflow.json",
                "ends with '.'",
            ),
            (
                "valid-signature-over-confusable-path",
                "workflow/p\u{0430}ck.json",
                "outside the lock-path alphabet",
            ),
        ] {
            let mut pack = valid_pack();
            pack.lock.entries[0].path = bad_path.to_string();
            cases.push((
                name,
                serde_json::to_value(rebind_content(pack)).unwrap(),
                "Semantic",
                fragment,
            ));
        }

        let mut dup_path = valid_pack();
        dup_path.lock.entries[1].path = dup_path.lock.entries[0].path.clone();
        cases.push((
            "valid-signature-over-duplicate-lock-path",
            serde_json::to_value(rebind_content(dup_path)).unwrap(),
            "Semantic",
            "duplicate lock path",
        ));

        // A third entry whose path collides case-insensitively with an
        // existing one: the dup rule fires DURING the entry loop, before the
        // ordering rule gets a look.
        let mut case_collision = valid_pack();
        let mut colliding = case_collision.lock.entries[1].clone();
        colliding.path = "WORKFLOW/workflow.json".to_string();
        case_collision.lock.entries.push(colliding);
        cases.push((
            "valid-signature-over-case-colliding-paths",
            serde_json::to_value(rebind_content(case_collision)).unwrap(),
            "Semantic",
            "duplicate lock path (case-insensitive)",
        ));

        // Two valid, distinct paths in the wrong byte order.
        let mut unsorted = valid_pack();
        unsorted.lock.entries.swap(0, 1);
        cases.push((
            "valid-signature-over-unsorted-lock",
            serde_json::to_value(rebind_content(unsorted)).unwrap(),
            "Semantic",
            "not sorted byte-lexicographically",
        ));

        let mut oversized_file = valid_pack();
        oversized_file.lock.entries[0].size_bytes = MAX_FILE_SIZE_BYTES + 1;
        cases.push((
            "valid-signature-over-oversized-file",
            serde_json::to_value(rebind_content(oversized_file)).unwrap(),
            "Semantic",
            "per-file cap",
        ));

        let mut empty_lock = valid_pack();
        empty_lock.lock.entries.clear();
        cases.push((
            "valid-signature-over-empty-lock",
            serde_json::to_value(rebind_content(empty_lock)).unwrap(),
            "Semantic",
            "pins no file",
        ));

        // The one case where content is NOT rebound — the mismatch IS the
        // subject.
        let mut mismatched = valid_pack();
        mismatched.digests.content = blake3_hex(b"not the lock");
        cases.push((
            "valid-signature-over-mismatched-content-digest",
            serde_json::to_value(mismatched).unwrap(),
            "Semantic",
            "does not match PACK.lock",
        ));

        let mut dup_subject = valid_pack();
        let first = dup_subject.provenance.subject[0].clone();
        dup_subject.provenance.subject.push(first);
        cases.push((
            "valid-signature-over-duplicate-subject-name",
            serde_json::to_value(dup_subject).unwrap(),
            "Semantic",
            "duplicate subject name",
        ));

        let mut no_subject = valid_pack();
        no_subject.provenance.subject.clear();
        cases.push((
            "valid-signature-over-empty-subject-list",
            serde_json::to_value(no_subject).unwrap(),
            "Semantic",
            "carries no subject",
        ));

        let mut secret = valid_pack();
        secret.secret_refs = vec!["sk-abcdef0123456789".to_string()];
        cases.push((
            "valid-signature-over-materialised-secret",
            serde_json::to_value(secret).unwrap(),
            "Semantic",
            "opaque logical reference",
        ));

        // Digest-syntax cases mutate `graph_semantic`, NOT `content`: a
        // content mutation would trip the binding check and mask the syntax
        // rule under test.
        let mut truncated = valid_pack();
        truncated.digests.graph_semantic = "deadbeef".to_string();
        cases.push((
            "valid-signature-over-truncated-digest",
            serde_json::to_value(truncated).unwrap(),
            "Semantic",
            "graph_semantic",
        ));

        let mut upper = valid_pack();
        upper.digests.graph_semantic = upper.digests.graph_semantic.to_ascii_uppercase();
        cases.push((
            "valid-signature-over-uppercase-digest",
            serde_json::to_value(upper).unwrap(),
            "Semantic",
            "graph_semantic",
        ));

        // The expected VARIANT of each case is read from the corpus JSON, not
        // hard-coded here (external cross-verifier, round 4): the corpus row's
        // `expect` must be the value asserted, so editing the JSON cannot
        // leave the gate green on a stale expectation. `cross-domain-signature`
        // is exercised in `tests::cross_domain_signature_rejected` (it needs
        // the domain-injectable door), so it is excluded from the coverage set
        // but its count is still floored below.
        let corpus = corpus();
        let corpus_rows: std::collections::BTreeMap<String, String> =
            corpus["post_signature_entries"]
                .as_array()
                .expect("post_signature_entries")
                .iter()
                .map(|e| {
                    (
                        e["name"].as_str().expect("name").to_string(),
                        e["expect"].as_str().expect("expect").to_string(),
                    )
                })
                .collect();
        assert!(
            corpus_rows.len() >= 21,
            "the post-signature corpus must not shrink silently (found {})",
            corpus_rows.len()
        );
        let declared: std::collections::BTreeSet<String> = corpus_rows
            .keys()
            .filter(|n| *n != "cross-domain-signature")
            .cloned()
            .collect();
        let covered: std::collections::BTreeSet<String> =
            cases.iter().map(|(n, _, _, _)| (*n).to_string()).collect();
        assert_eq!(
            declared, covered,
            "every declared post-signature corpus entry must be exercised"
        );

        for (name, payload, in_test_expect, fragment) in cases {
            // The corpus is the source of truth for the expected variant; the
            // in-test literal must agree with it (a drift here means the two
            // disagree, which is itself a bug).
            let expect = corpus_rows.get(name).expect("case is a corpus row");
            assert_eq!(
                expect, in_test_expect,
                "case '{name}': in-test expect disagrees with the corpus"
            );
            let raw = sign_envelope(&payload, PACK_SIGNING_DOMAIN, &signing_key());
            match verify_signed_pack(&raw, &verifying) {
                Ok(_) => panic!("case '{name}' was ACCEPTED despite a hostile body"),
                Err(err) => {
                    assert_eq!(
                        error_kind(&err),
                        expect,
                        "case '{name}' refused with the wrong error: {err:?}"
                    );
                    let rendered = err.to_string();
                    assert!(
                        rendered.contains(fragment),
                        "case '{name}': refusal '{rendered}' does not name '{fragment}'"
                    );
                    if name == "valid-signature-over-materialised-secret" {
                        // The refusal must WITHHOLD the detected value: an
                        // error that echoes the secret is itself the leak.
                        assert!(
                            !rendered.contains("sk-abcdef"),
                            "the refusal must not echo the secret: '{rendered}'"
                        );
                    }
                }
            }
        }
    }

    /// Duplicate JSON keys are refused at the STRUCTURAL parse, in both the
    /// envelope and the payload — before any signature is even considered.
    ///
    /// The hazard is a cross-parser differential: `serde_json` keeps the last
    /// duplicate, a first-wins parser elsewhere reads a different pack out of
    /// the byte stream a signature was checked against. Refusing the bytes
    /// outright is the only order-independent answer.
    #[test]
    fn duplicate_json_keys_are_refused_structurally() {
        let verifying = signing_key().verifying_key();

        // Envelope-level duplicate.
        let raw = br#"{"payload": {}, "payload": {}, "signature": "00"}"#;
        let err = verify_signed_pack(raw, &verifying).unwrap_err_or_else("envelope-dup");
        assert!(
            matches!(err, PackError::MalformedEnvelope(ref m) if m.contains("duplicate object key")),
            "envelope duplicate must be named as such, got {err:?}"
        );

        // Payload-level duplicate, nested.
        let raw = br#"{"payload": {"name": "a", "name": "b"}, "signature": "00"}"#;
        let err = verify_signed_pack(raw, &verifying).unwrap_err_or_else("payload-dup");
        assert!(
            matches!(err, PackError::MalformedEnvelope(ref m) if m.contains("duplicate object key")),
            "payload duplicate must be named as such, got {err:?}"
        );

        // Control: the strict parser accepts duplicate-free bytes — the
        // refusal above is the duplicate, not the parser.
        parse_rejecting_duplicate_keys(br#"{"a": 1, "b": {"a": 1}}"#)
            .expect("duplicate-free JSON parses");
    }
}
