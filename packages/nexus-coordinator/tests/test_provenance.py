# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 14 Phase A — tests for provenance generation and verification."""

from __future__ import annotations

import json

import nexus_core
from nexus_coordinator.provenance import (
    generate_provenance,
    provenance_blake3_hex,
    provenance_to_json,
    verify_provenance,
)


def _make_keypair() -> tuple[bytes, bytes]:
    """Generate a fresh Ed25519 keypair via nexus_core."""
    kp = nexus_core.generate_secret()
    return kp["secret"], kp["public"]


class TestGenerateProvenance:
    def test_produces_valid_record(self) -> None:
        secret, public = _make_keypair()
        rec = generate_provenance(
            repo_url="https://github.com/alice/app",
            commit_sha="abc123",
            artifact_hash="de" * 32,
            node_id_hex=public.hex(),
            secret=secret,
        )
        assert rec.schema_version == 1
        assert rec.repo_url == "https://github.com/alice/app"
        assert rec.commit_sha == "abc123"
        assert rec.artifact_hash == "de" * 32
        assert rec.node_id == public.hex()
        assert len(rec.signature) == 128  # 64 bytes hex

    def test_provenance_to_json_is_valid_json(self) -> None:
        secret, public = _make_keypair()
        rec = generate_provenance(
            repo_url="https://github.com/alice/app",
            commit_sha="abc123",
            artifact_hash="de" * 32,
            node_id_hex=public.hex(),
            secret=secret,
        )
        data = json.loads(provenance_to_json(rec))
        assert data["repo_url"] == "https://github.com/alice/app"
        assert "signature" in data


class TestVerifyProvenance:
    def test_accepts_valid_signature(self) -> None:
        secret, public = _make_keypair()
        rec = generate_provenance(
            repo_url="https://github.com/alice/app",
            commit_sha="abc123",
            artifact_hash="de" * 32,
            node_id_hex=public.hex(),
            secret=secret,
        )
        assert verify_provenance(provenance_to_json(rec), public) is True

    def test_rejects_tampered_hash(self) -> None:
        secret, public = _make_keypair()
        rec = generate_provenance(
            repo_url="https://github.com/alice/app",
            commit_sha="abc123",
            artifact_hash="de" * 32,
            node_id_hex=public.hex(),
            secret=secret,
        )
        # Tamper with the artifact_hash.
        j = json.loads(provenance_to_json(rec))
        j["artifact_hash"] = "ff" * 32
        assert verify_provenance(json.dumps(j), public) is False

    def test_rejects_wrong_key(self) -> None:
        secret1, public1 = _make_keypair()
        _secret2, public2 = _make_keypair()
        rec = generate_provenance(
            repo_url="https://github.com/alice/app",
            commit_sha="abc123",
            artifact_hash="de" * 32,
            node_id_hex=public1.hex(),
            secret=secret1,
        )
        # Verify with a different key.
        assert verify_provenance(provenance_to_json(rec), public2) is False

    def test_rejects_garbage_json(self) -> None:
        _, public = _make_keypair()
        assert verify_provenance("not json", public) is False


class TestProvenanceBlake3:
    def test_hash_is_deterministic(self) -> None:
        secret, public = _make_keypair()
        rec = generate_provenance(
            repo_url="https://github.com/alice/app",
            commit_sha="abc123",
            artifact_hash="de" * 32,
            node_id_hex=public.hex(),
            secret=secret,
        )
        h1 = provenance_blake3_hex(rec)
        h2 = provenance_blake3_hex(rec)
        assert h1 == h2
        assert len(h1) == 64  # 32 bytes hex
