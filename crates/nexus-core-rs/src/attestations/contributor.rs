// SPDX-License-Identifier: AGPL-3.0-or-later
//! ContributorAttestation in-toto v1.0 predicate (Couche 2).
//!
//! Sprint 22 Phase C. The coordinator signs a binary attestation at
//! verified-deploy time asserting "node_id has completed at least
//! one successful verified-deploy for subject project at
//! commit_sha". Consumed by curator list verification under a
//! governance-strong flag (cf. [`crate::curator`]), and by the
//! federated trust-web Amnesty integration reserved S27.
//!
//! Wire shape follows the in-toto v1.0 Statement envelope :
//!
//! ```text
//! {
//!   _type:          "https://in-toto.io/Statement/v1",
//!   subject:        [ { name, digest } ],   // exactly 1 entry in v1
//!   predicateType:  "https://nexus-grid.org/contributor-attestation/v1",
//!   predicate:      { contributor_node_id, first_deploy_ts,
//!                     commit_sha, repo_url, attestation_coord_sig }
//! }
//! ```
//!
//! The signable bytes are produced by replacing
//! `predicate.attestation_coord_sig` with the empty string, serializing
//! the whole envelope via JCS + [`DOMAIN_CONTRIBUTOR_ATTESTATION_V1`],
//! and signing with the coordinator's Ed25519 key.
//!
//! ## Matthew-effect caveat (LT-1)
//!
//! This attestation is **binary**. It does not quantify contribution
//! weight, and the Matthew effect reappears one layer deeper —
//! high-kudos workers publish more projects and earn more
//! attestations. Fairness reform is scheduled post-`v1.0` as
//! [`docs/release/ROADMAP_COMMITMENTS.md §LT-1`](
//! ../../../docs/release/ROADMAP_COMMITMENTS.md).
//!
//! The docs-side specification lives in
//! [`docs/security/CONTRIBUTOR_ATTESTATION_PREDICATE.md`](
//! ../../../docs/security/CONTRIBUTOR_ATTESTATION_PREDICATE.md).

use std::collections::BTreeMap;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64_STANDARD;
use serde::{Deserialize, Serialize};

use crate::canonical::{DOMAIN_CONTRIBUTOR_ATTESTATION_V1, canonical_bytes};
use crate::crypto::{KeyPair, PUBLIC_KEY_LENGTH};
use crate::error::{NexusError, Result};

/// The stable in-toto v1.0 Statement `_type` URI. Verifiers reject
/// envelopes whose `_type` does not match.
pub const CONTRIBUTOR_ATTESTATION_STATEMENT_TYPE: &str = "https://in-toto.io/Statement/v1";

/// The stable SBFB predicate type URI. Pre-launch redefinition
/// policy : any breaking change before the first `v1.0` tag edits
/// the v1 semantics in place. After `v1.0`, this URI becomes
/// immutable — any semantic change bumps the path segment to `/v2`.
pub const CONTRIBUTOR_ATTESTATION_PREDICATE_TYPE: &str =
    "https://nexus-grid.org/contributor-attestation/v1";

/// Algorithm identifier used inside `subject[].digest` for the
/// enclosed project artifact hash. Matches the BLAKE3 algorithm
/// used by [`crate::crypto::blake3_hash`] and the coordinator's
/// `ProvenanceRecord.artifact_hash` (Sprint 14).
pub const SUBJECT_DIGEST_ALGO_BLAKE3: &str = "blake3";

/// Error type for contributor attestation operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContributorAttestationError {
    /// Envelope `_type` did not match [`CONTRIBUTOR_ATTESTATION_STATEMENT_TYPE`].
    BadStatementType(String),
    /// Envelope `predicateType` did not match
    /// [`CONTRIBUTOR_ATTESTATION_PREDICATE_TYPE`].
    BadPredicateType(String),
    /// `subject[]` did not have exactly 1 entry.
    BadSubjectCount(usize),
    /// A required field failed basic pattern validation (hex
    /// encoding, expected byte length, non-empty string, ...).
    BadField {
        /// Name of the field that failed validation (e.g.
        /// `"predicate.contributor_node_id"`).
        field: String,
        /// Human-readable reason the validation failed.
        reason: String,
    },
    /// Ed25519 signature verification failed.
    BadSignature(String),
    /// JCS serialization failed.
    CanonicalFailed(String),
}

impl std::fmt::Display for ContributorAttestationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContributorAttestationError::BadStatementType(got) => write!(
                f,
                "in-toto Statement _type mismatch (got {got:?}, expected {CONTRIBUTOR_ATTESTATION_STATEMENT_TYPE:?})"
            ),
            ContributorAttestationError::BadPredicateType(got) => write!(
                f,
                "predicateType mismatch (got {got:?}, expected {CONTRIBUTOR_ATTESTATION_PREDICATE_TYPE:?})"
            ),
            ContributorAttestationError::BadSubjectCount(n) => {
                write!(f, "subject[] must have exactly 1 entry in v1, got {n}")
            }
            ContributorAttestationError::BadField { field, reason } => {
                write!(f, "bad field {field:?}: {reason}")
            }
            ContributorAttestationError::BadSignature(msg) => {
                write!(f, "attestation signature invalid: {msg}")
            }
            ContributorAttestationError::CanonicalFailed(msg) => {
                write!(f, "canonical bytes failed: {msg}")
            }
        }
    }
}

impl std::error::Error for ContributorAttestationError {}

impl From<ContributorAttestationError> for NexusError {
    fn from(e: ContributorAttestationError) -> Self {
        NexusError::Crypto(e.to_string())
    }
}

/// A single entry in the in-toto v1.0 Statement `subject[]` array.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InTotoSubject {
    /// URI identifying the subject. SBFB uses
    /// `nexus-grid://project/<project_id_hex>`.
    pub name: String,

    /// DigestSet mapping algorithm identifiers to lowercase hex
    /// digests. SBFB v1 uses a single entry
    /// `"blake3" → <artifact_hash_hex>` matching
    /// `ProvenanceRecord.artifact_hash`.
    pub digest: BTreeMap<String, String>,
}

/// The full in-toto v1.0 Statement envelope for a contributor
/// attestation.
///
/// The `_type` and `predicateType` fields are fixed strings at the
/// wire format level — they are not user-configurable. The signed
/// bytes are produced by replacing
/// `predicate.attestation_coord_sig` with the empty string and
/// feeding the whole envelope through
/// [`canonical_bytes`] with [`DOMAIN_CONTRIBUTOR_ATTESTATION_V1`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContributorAttestation {
    /// in-toto v1.0 Statement type URI.
    #[serde(rename = "_type")]
    pub statement_type: String,

    /// Subject array (exactly 1 entry in v1).
    pub subject: Vec<InTotoSubject>,

    /// SBFB predicate type URI.
    #[serde(rename = "predicateType")]
    pub predicate_type: String,

    /// The SBFB-specific predicate body.
    pub predicate: ContributorPredicate,
}

/// SBFB-specific body of a [`ContributorAttestation`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContributorPredicate {
    /// Ed25519 public key of the contributor, lowercase hex (64
    /// chars = 32 bytes).
    pub contributor_node_id: String,

    /// Unix timestamp (seconds, UTC) of the first successful
    /// verified-deploy for this `(project, contributor)` pair.
    /// Subsequent deploys do NOT replace this field — it is the
    /// anchor. The coordinator looks up the
    /// `contributor_registry` SQLite table before signing to
    /// reuse the existing timestamp when applicable.
    pub first_deploy_ts: i64,

    /// Git SHA-1 of the commit the first verified-deploy was
    /// built from. Lowercase hex, 40 chars. SHA-256 object-id
    /// migration deferred until git-ecosystem consensus.
    pub commit_sha: String,

    /// Canonical source-of-truth URL of the repository (same as
    /// `ProvenanceRecord.repo_url`). Used for multi-forge
    /// cross-validation in Couche 3 (S23+).
    pub repo_url: String,

    /// Ed25519 signature over the canonical JCS bytes of the
    /// enclosing [`ContributorAttestation`] with this field
    /// **replaced by the empty string** before serialization.
    /// Base64-std-encoded (88 chars = 64 bytes + padding).
    pub attestation_coord_sig: String,
}

impl ContributorAttestation {
    /// Build and sign a [`ContributorAttestation`] for the given
    /// project artifact + contributor + commit.
    ///
    /// - `project_id_hex` is the 64-char lowercase hex of the
    ///   project's 32-byte pkarr node id.
    /// - `artifact_hash_hex` is the BLAKE3 hex of the enclosing
    ///   zip artifact (matches `ProvenanceRecord.artifact_hash`).
    /// - `contributor_node_id_hex` is the contributor's Ed25519
    ///   public key, lowercase hex.
    /// - `first_deploy_ts` is the anchor timestamp. Callers should
    ///   look it up in the `contributor_registry` table and reuse
    ///   the stored value for subsequent deploys by the same
    ///   contributor to the same project.
    /// - `commit_sha_hex` is the git SHA-1 of the commit built.
    /// - `repo_url` is the canonical repository URL.
    /// - `coord_keypair` is the coordinator's signing key.
    ///
    /// The function validates field shapes (hex encoding, lengths,
    /// non-empty strings) before signing so malformed input
    /// surfaces loudly at build time rather than at verify time.
    pub fn build(
        project_id_hex: &str,
        artifact_hash_hex: &str,
        contributor_node_id_hex: &str,
        first_deploy_ts: i64,
        commit_sha_hex: &str,
        repo_url: &str,
        coord_keypair: &KeyPair,
    ) -> Result<Self> {
        let subject_name = format!("nexus-grid://project/{project_id_hex}");
        validate_hex_field("project_id_hex", project_id_hex, 64)?;
        validate_hex_field("artifact_hash_hex", artifact_hash_hex, 64)?;
        validate_hex_field("contributor_node_id_hex", contributor_node_id_hex, 64)?;
        validate_hex_field("commit_sha_hex", commit_sha_hex, 40)?;
        validate_non_empty("repo_url", repo_url)?;

        let mut digest = BTreeMap::new();
        digest.insert(
            SUBJECT_DIGEST_ALGO_BLAKE3.to_string(),
            artifact_hash_hex.to_string(),
        );
        let subject = vec![InTotoSubject {
            name: subject_name,
            digest,
        }];

        // Build unsigned shape (signature set to empty string).
        let mut unsigned = ContributorAttestation {
            statement_type: CONTRIBUTOR_ATTESTATION_STATEMENT_TYPE.to_string(),
            subject,
            predicate_type: CONTRIBUTOR_ATTESTATION_PREDICATE_TYPE.to_string(),
            predicate: ContributorPredicate {
                contributor_node_id: contributor_node_id_hex.to_string(),
                first_deploy_ts,
                commit_sha: commit_sha_hex.to_string(),
                repo_url: repo_url.to_string(),
                attestation_coord_sig: String::new(),
            },
        };

        let canonical = canonical_bytes(&unsigned, DOMAIN_CONTRIBUTOR_ATTESTATION_V1)
            .map_err(|e| NexusError::Crypto(format!("build_contributor_attestation: {e}")))?;
        let sig_bytes = coord_keypair.sign(&canonical);
        unsigned.predicate.attestation_coord_sig = B64_STANDARD.encode(sig_bytes);

        Ok(unsigned)
    }

    /// Verify an attestation. Checks envelope invariants, field
    /// shapes, and Ed25519 signature under
    /// [`DOMAIN_CONTRIBUTOR_ATTESTATION_V1`].
    ///
    /// `coord_pubkey` is the coordinator's 32-byte Ed25519 public
    /// key (recovered from the transport layer ; the predicate
    /// intentionally does not self-declare the signer).
    ///
    /// Does **not** cross-check against a local `ProvenanceRecord`
    /// — the caller is expected to do that as a separate step for
    /// full defence-in-depth.
    pub fn verify(
        &self,
        coord_pubkey: &[u8; PUBLIC_KEY_LENGTH],
    ) -> std::result::Result<(), ContributorAttestationError> {
        if self.statement_type != CONTRIBUTOR_ATTESTATION_STATEMENT_TYPE {
            return Err(ContributorAttestationError::BadStatementType(
                self.statement_type.clone(),
            ));
        }
        if self.predicate_type != CONTRIBUTOR_ATTESTATION_PREDICATE_TYPE {
            return Err(ContributorAttestationError::BadPredicateType(
                self.predicate_type.clone(),
            ));
        }
        if self.subject.len() != 1 {
            return Err(ContributorAttestationError::BadSubjectCount(
                self.subject.len(),
            ));
        }

        // Field shape checks : catch malformed predicates cheaply
        // before attempting an Ed25519 verification (which is the
        // expensive step).
        validate_hex_field(
            "predicate.contributor_node_id",
            &self.predicate.contributor_node_id,
            64,
        )
        .map_err(|e| field_err("predicate.contributor_node_id", e))?;
        validate_hex_field("predicate.commit_sha", &self.predicate.commit_sha, 40)
            .map_err(|e| field_err("predicate.commit_sha", e))?;
        validate_non_empty("predicate.repo_url", &self.predicate.repo_url)
            .map_err(|e| field_err("predicate.repo_url", e))?;

        // Reconstruct unsigned shape : clone the envelope with
        // `attestation_coord_sig` replaced by the empty string, then
        // recompute the canonical bytes.
        let sig_b64 = &self.predicate.attestation_coord_sig;
        let sig_bytes = B64_STANDARD
            .decode(sig_b64.as_bytes())
            .map_err(|e| ContributorAttestationError::BadSignature(format!("bad base64: {e}")))?;
        if sig_bytes.len() != crate::crypto::SIGNATURE_BYTES {
            return Err(ContributorAttestationError::BadSignature(format!(
                "signature length {} != {}",
                sig_bytes.len(),
                crate::crypto::SIGNATURE_BYTES
            )));
        }
        let mut sig_arr = [0u8; crate::crypto::SIGNATURE_BYTES];
        sig_arr.copy_from_slice(&sig_bytes);

        let unsigned = ContributorAttestation {
            statement_type: self.statement_type.clone(),
            subject: self.subject.clone(),
            predicate_type: self.predicate_type.clone(),
            predicate: ContributorPredicate {
                contributor_node_id: self.predicate.contributor_node_id.clone(),
                first_deploy_ts: self.predicate.first_deploy_ts,
                commit_sha: self.predicate.commit_sha.clone(),
                repo_url: self.predicate.repo_url.clone(),
                attestation_coord_sig: String::new(),
            },
        };
        let canonical = canonical_bytes(&unsigned, DOMAIN_CONTRIBUTOR_ATTESTATION_V1)
            .map_err(|e| ContributorAttestationError::CanonicalFailed(e.to_string()))?;

        crate::crypto::verify(coord_pubkey, &canonical, &sig_arr)
            .map_err(|e| ContributorAttestationError::BadSignature(e.to_string()))
    }
}

fn validate_hex_field(name: &str, value: &str, expected_chars: usize) -> Result<()> {
    if value.len() != expected_chars {
        return Err(NexusError::Crypto(format!(
            "{name}: expected {expected_chars} chars, got {}",
            value.len()
        )));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        return Err(NexusError::Crypto(format!("{name}: must be lowercase hex")));
    }
    Ok(())
}

fn validate_non_empty(name: &str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(NexusError::Crypto(format!("{name}: must be non-empty")));
    }
    Ok(())
}

fn field_err(field: &str, err: NexusError) -> ContributorAttestationError {
    ContributorAttestationError::BadField {
        field: field.to_string(),
        reason: err.to_string(),
    }
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const PROJECT_ID_HEX: &str = "2bf1ae3c8aa04d7a8b2e0b2e3b84f6d7c4f1a8b1e3d4c5a6b7c8d9e0f1a2b3c4";
    const ARTIFACT_HASH_HEX: &str =
        "5fabc50000000000000000000000000000000000000000000000000000000000";
    const CONTRIBUTOR_NODE_ID_HEX: &str =
        "a3b1c2d3e4f5061728394a5b6c7d8e9f0a1b2c3d4e5f60718293a4b5c6d7e8f9";
    const COMMIT_SHA_HEX: &str = "1a2b3c4d5e6f7890abcdef1234567890abcdef12";
    const REPO_URL: &str = "https://codeberg.org/alice/transLingua";

    fn build_sample_attestation(first_deploy_ts: i64, coord: &KeyPair) -> ContributorAttestation {
        ContributorAttestation::build(
            PROJECT_ID_HEX,
            ARTIFACT_HASH_HEX,
            CONTRIBUTOR_NODE_ID_HEX,
            first_deploy_ts,
            COMMIT_SHA_HEX,
            REPO_URL,
            coord,
        )
        .expect("build succeeds")
    }

    #[test]
    fn build_from_provenance_fields_valid_wire_shape() {
        let coord = KeyPair::generate();
        let att = build_sample_attestation(1_713_556_800, &coord);

        assert_eq!(att.statement_type, CONTRIBUTOR_ATTESTATION_STATEMENT_TYPE);
        assert_eq!(att.predicate_type, CONTRIBUTOR_ATTESTATION_PREDICATE_TYPE);
        assert_eq!(att.subject.len(), 1);
        assert_eq!(
            att.subject[0].name,
            format!("nexus-grid://project/{PROJECT_ID_HEX}")
        );
        assert_eq!(
            att.subject[0].digest.get(SUBJECT_DIGEST_ALGO_BLAKE3),
            Some(&ARTIFACT_HASH_HEX.to_string())
        );
        assert_eq!(att.predicate.contributor_node_id, CONTRIBUTOR_NODE_ID_HEX);
        assert_eq!(att.predicate.commit_sha, COMMIT_SHA_HEX);
        assert_eq!(att.predicate.repo_url, REPO_URL);
        assert_eq!(att.predicate.first_deploy_ts, 1_713_556_800);
        // base64 of 64-byte Ed25519 signature = 88 chars (with
        // "==" padding).
        assert_eq!(att.predicate.attestation_coord_sig.len(), 88);
    }

    #[test]
    fn verify_accepts_valid_attestation() {
        let coord = KeyPair::generate();
        let att = build_sample_attestation(1_713_556_800, &coord);
        att.verify(&coord.public_bytes()).expect("valid → ok");
    }

    #[test]
    fn verify_rejects_wrong_coord_pubkey() {
        let coord = KeyPair::generate();
        let impostor = KeyPair::generate();
        let att = build_sample_attestation(1_713_556_800, &coord);
        let err = att
            .verify(&impostor.public_bytes())
            .expect_err("wrong key must reject");
        assert!(matches!(err, ContributorAttestationError::BadSignature(_)));
    }

    #[test]
    fn verify_rejects_tampered_commit_sha() {
        let coord = KeyPair::generate();
        let mut att = build_sample_attestation(1_713_556_800, &coord);
        // Replace last hex char — still valid shape, but bytes changed.
        let mut new_sha = att.predicate.commit_sha.clone();
        new_sha.replace_range(39..40, if new_sha.ends_with('f') { "0" } else { "f" });
        att.predicate.commit_sha = new_sha;
        let err = att
            .verify(&coord.public_bytes())
            .expect_err("tampered sha must reject");
        assert!(matches!(err, ContributorAttestationError::BadSignature(_)));
    }

    #[test]
    fn verify_rejects_tampered_repo_url() {
        let coord = KeyPair::generate();
        let mut att = build_sample_attestation(1_713_556_800, &coord);
        att.predicate.repo_url = "https://attacker.example/alice/transLingua".to_string();
        let err = att
            .verify(&coord.public_bytes())
            .expect_err("tampered url must reject");
        assert!(matches!(err, ContributorAttestationError::BadSignature(_)));
    }

    #[test]
    fn verify_rejects_bad_statement_type() {
        let coord = KeyPair::generate();
        let mut att = build_sample_attestation(1_713_556_800, &coord);
        att.statement_type = "https://attacker.example/Statement/v1".into();
        let err = att
            .verify(&coord.public_bytes())
            .expect_err("bad type must reject");
        assert!(matches!(
            err,
            ContributorAttestationError::BadStatementType(_)
        ));
    }

    #[test]
    fn verify_rejects_wrong_predicate_type() {
        let coord = KeyPair::generate();
        let mut att = build_sample_attestation(1_713_556_800, &coord);
        att.predicate_type = "https://nexus-grid.org/contributor-attestation/v2".into();
        let err = att
            .verify(&coord.public_bytes())
            .expect_err("bad predicateType must reject");
        assert!(matches!(
            err,
            ContributorAttestationError::BadPredicateType(_)
        ));
    }

    #[test]
    fn verify_rejects_multi_subject_envelope() {
        let coord = KeyPair::generate();
        let mut att = build_sample_attestation(1_713_556_800, &coord);
        att.subject.push(att.subject[0].clone());
        let err = att
            .verify(&coord.public_bytes())
            .expect_err("multi-subject must reject");
        assert!(matches!(
            err,
            ContributorAttestationError::BadSubjectCount(2)
        ));
    }

    #[test]
    fn build_rejects_uppercase_hex_commit_sha() {
        let coord = KeyPair::generate();
        let result = ContributorAttestation::build(
            PROJECT_ID_HEX,
            ARTIFACT_HASH_HEX,
            CONTRIBUTOR_NODE_ID_HEX,
            1_713_556_800,
            &COMMIT_SHA_HEX.to_uppercase(),
            REPO_URL,
            &coord,
        );
        assert!(result.is_err());
    }

    #[test]
    fn predicate_format_in_toto_compat_json_has_expected_keys() {
        let coord = KeyPair::generate();
        let att = build_sample_attestation(1_713_556_800, &coord);
        let json = serde_json::to_string(&att).expect("serialize");
        // Envelope keys as specified in in-toto v1.0 spec.
        assert!(json.contains("\"_type\""), "json: {json}");
        assert!(json.contains("\"subject\""), "json: {json}");
        assert!(json.contains("\"predicateType\""), "json: {json}");
        assert!(json.contains("\"predicate\""), "json: {json}");
        // Predicate fields.
        assert!(json.contains("\"contributor_node_id\""));
        assert!(json.contains("\"first_deploy_ts\""));
        assert!(json.contains("\"commit_sha\""));
        assert!(json.contains("\"repo_url\""));
        assert!(json.contains("\"attestation_coord_sig\""));
    }
}
