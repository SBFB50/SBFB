# SPDX-License-Identifier: AGPL-3.0-or-later
"""Provenance attestation for verified deploy (SLSA L1).

Sprint 14 Phase A — generates and verifies ``provenance.json``
attestations. Each attestation is signed by the coordinator's
Ed25519 keypair and proves that:

1. The artifact was built from a specific repo + commit.
2. The signing coordinator is identified by its ``node_id``.
3. The artifact content matches the recorded BLAKE3 hash.

Signature uses JCS canonical bytes with domain separation tag
``nexus-provenance-v1\\x00`` to prevent cross-domain replay.
"""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from typing import Any

import nexus_core
import structlog

_log = structlog.get_logger(__name__)

PROVENANCE_SCHEMA_VERSION = 1

# Domain separation tag for provenance signing.
# Must match DOMAIN_PROVENANCE_V1 in canonical.rs (reserved Sprint 14).
_DOMAIN_PROVENANCE_V1 = b"nexus-provenance-v1"


@dataclass(frozen=True, slots=True)
class ProvenanceRecord:
    """A signed provenance attestation (SLSA L1 auto-attestation)."""

    schema_version: int
    repo_url: str
    commit_sha: str
    artifact_hash: str
    node_id: str
    timestamp: str
    signature: str


def generate_provenance(
    *,
    repo_url: str,
    commit_sha: str,
    artifact_hash: str,
    node_id_hex: str,
    secret: bytes,
) -> ProvenanceRecord:
    """Generate a signed provenance attestation.

    Parameters
    ----------
    repo_url:
        URL of the source repository.
    commit_sha:
        Git commit SHA the artifact was built from.
    artifact_hash:
        BLAKE3 hex hash of the zip artifact (before provenance is added).
    node_id_hex:
        Ed25519 public key hex of the signing coordinator.
    secret:
        32-byte Ed25519 secret key for signing.

    Returns
    -------
    ProvenanceRecord
        The complete signed attestation.
    """
    timestamp = datetime.now(UTC).isoformat()

    # Build the signable payload (everything except the signature).
    payload = _signable_payload(
        schema_version=PROVENANCE_SCHEMA_VERSION,
        repo_url=repo_url,
        commit_sha=commit_sha,
        artifact_hash=artifact_hash,
        node_id=node_id_hex,
        timestamp=timestamp,
    )

    # Sign with domain separation.
    canonical = _canonical_bytes(payload)
    sig_bytes: bytes = nexus_core.sign_bytes(canonical, secret)
    signature_hex = sig_bytes.hex()

    return ProvenanceRecord(
        schema_version=PROVENANCE_SCHEMA_VERSION,
        repo_url=repo_url,
        commit_sha=commit_sha,
        artifact_hash=artifact_hash,
        node_id=node_id_hex,
        timestamp=timestamp,
        signature=signature_hex,
    )


def verify_provenance(record_json: str, public_key: bytes) -> bool:
    """Verify a provenance attestation signature.

    Parameters
    ----------
    record_json:
        JSON string of the provenance record.
    public_key:
        32-byte Ed25519 public key of the expected signer.

    Returns
    -------
    bool
        ``True`` if the signature is valid, ``False`` otherwise.
    """
    try:
        data = json.loads(record_json)
        signature_hex = data.get("signature", "")
        sig_bytes = bytes.fromhex(signature_hex)

        payload = _signable_payload(
            schema_version=data["schema_version"],
            repo_url=data["repo_url"],
            commit_sha=data["commit_sha"],
            artifact_hash=data["artifact_hash"],
            node_id=data["node_id"],
            timestamp=data["timestamp"],
        )
        canonical = _canonical_bytes(payload)

        nexus_core.verify_bytes(canonical, sig_bytes, public_key)
        return True
    except Exception:
        _log.debug("provenance verification failed", exc_info=True)
        return False


def provenance_to_json(record: ProvenanceRecord) -> str:
    """Serialize a provenance record to JSON.

    Uses sorted keys for human readability. The signature is computed
    from canonical bytes, not from this serialized form.
    """
    return json.dumps(asdict(record), sort_keys=True, indent=2)


def provenance_blake3_hex(record: ProvenanceRecord) -> str:
    """Compute the BLAKE3 hex hash of a provenance record's JSON.

    This hash is propagated in the gossip announcement so receivers
    can verify the provenance without downloading the full zip.
    """
    json_bytes = provenance_to_json(record).encode("utf-8")
    hash_bytes: bytes = nexus_core.blake3_digest(json_bytes)
    return hash_bytes.hex()


# ------------------------------------------------------------------
# Internal helpers
# ------------------------------------------------------------------


def _signable_payload(**fields: Any) -> dict[str, Any]:
    """Build the dict that gets canonicalized for signing.

    The ``signature`` field is excluded — it's the output.
    """
    return {
        "schema_version": fields["schema_version"],
        "repo_url": fields["repo_url"],
        "commit_sha": fields["commit_sha"],
        "artifact_hash": fields["artifact_hash"],
        "node_id": fields["node_id"],
        "timestamp": fields["timestamp"],
    }


def _canonical_bytes(payload: dict[str, Any]) -> bytes:
    """Produce JCS-like canonical bytes with domain separation.

    Format: ``<domain>\\x00<json-sorted-keys-compact>``

    We use ``json.dumps(sort_keys=True, separators=(',', ':'))``
    which is equivalent to JCS for our flat string/int schema.
    """
    json_bytes = json.dumps(
        payload,
        sort_keys=True,
        ensure_ascii=False,
        separators=(",", ":"),
    ).encode("utf-8")
    return _DOMAIN_PROVENANCE_V1 + b"\x00" + json_bytes
