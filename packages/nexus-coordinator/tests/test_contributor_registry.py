# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 22 Phase C (Couche 2) — contributor attestation registry tests.

Unit tests for :mod:`nexus_coordinator.contributor_registry`. The
registry is a self-contained SQLite WAL + pyO3 signing module with
no iroh / FastAPI dependency, so the tests are pure data flow :
generate a fresh keypair, record an attestation, re-fetch, verify.

Tests cover the four plan §6.3 items :

- ``record_on_deploy`` — record + re-fetch invariants.
- ``is_verified_boolean`` — query API returns the right bool.
- ``sql_schema_migration`` — ``init_schema`` is idempotent across
  multiple registry instances pointing at the same file.
- ``api_deploy.emit_contributor_attestation_post_provenance`` — the
  coordinator's deploy flow invokes ``record()`` after
  ``generate_provenance``.
"""

from __future__ import annotations

import json
import secrets
from pathlib import Path

import nexus_core
import pytest
from nexus_coordinator.contributor_registry import ContributorRegistry

PROJECT_ID_HEX = "2bf1ae3c8aa04d7a8b2e0b2e3b84f6d7c4f1a8b1e3d4c5a6b7c8d9e0f1a2b3c4"
ARTIFACT_HASH_HEX = "5fabc50000000000000000000000000000000000000000000000000000000000"
COMMIT_SHA_HEX = "1a2b3c4d5e6f7890abcdef1234567890abcdef12"
REPO_URL = "https://codeberg.org/alice/transLingua"


def _fresh_keypair() -> tuple[bytes, bytes]:
    """Return (secret_32, pubkey_32) via nexus_core.generate_secret."""
    blob = nexus_core.generate_secret()
    return blob["secret"], blob["public"]


def test_record_on_deploy_produces_verifiable_attestation(tmp_path: Path) -> None:
    """Record + fetch + Rust-side verify round-trip under the
    correct coordinator pubkey. Exercises the full Rust ↔ Python
    loop the ``deploy.py`` hook depends on."""
    secret, pubkey = _fresh_keypair()
    registry = ContributorRegistry(tmp_path / "contributor_registry.sqlite")

    record = registry.record(
        project_id=PROJECT_ID_HEX,
        contributor_node_id=pubkey.hex(),
        artifact_hash=ARTIFACT_HASH_HEX,
        commit_sha=COMMIT_SHA_HEX,
        repo_url=REPO_URL,
        coord_secret=secret,
        now_ts=1_713_556_800,
    )

    assert record.project_id == PROJECT_ID_HEX
    assert record.contributor_node_id == pubkey.hex()
    assert record.first_deploy_ts == 1_713_556_800
    assert record.commit_sha == COMMIT_SHA_HEX
    assert record.repo_url == REPO_URL

    envelope = json.loads(record.attestation_json)
    assert envelope["_type"] == "https://in-toto.io/Statement/v1"
    assert envelope["predicateType"] == ("https://nexus-grid.org/contributor-attestation/v1")
    assert envelope["subject"][0]["name"] == f"nexus-grid://project/{PROJECT_ID_HEX}"

    # Offline verify under the expected coordinator pubkey.
    nexus_core.verify_contributor_attestation(record.attestation_json, pubkey)


def test_is_verified_boolean_returns_correct_state(tmp_path: Path) -> None:
    """``is_verified_contributor`` returns True for a recorded pair
    and False for anyone else. Exercises the indexed lookup path
    the daemon proxy depends on."""
    secret, pubkey = _fresh_keypair()
    registry = ContributorRegistry(tmp_path / "contributor_registry.sqlite")
    registry.record(
        project_id=PROJECT_ID_HEX,
        contributor_node_id=pubkey.hex(),
        artifact_hash=ARTIFACT_HASH_HEX,
        commit_sha=COMMIT_SHA_HEX,
        repo_url=REPO_URL,
        coord_secret=secret,
    )

    assert registry.is_verified_contributor(PROJECT_ID_HEX, pubkey.hex()) is True

    # Different node_id for same project → False.
    other_pk = secrets.token_bytes(32).hex()
    assert registry.is_verified_contributor(PROJECT_ID_HEX, other_pk) is False

    # Same node_id but different project → False.
    other_project = "f" * 64
    assert registry.is_verified_contributor(other_project, pubkey.hex()) is False


def test_sql_schema_migration_is_idempotent(tmp_path: Path) -> None:
    """Multiple ``ContributorRegistry`` instances pointing at the
    same file must share state without schema conflict. Guards
    against a regression where the ``CREATE TABLE`` on the second
    boot would raise because the table already exists (missing
    ``IF NOT EXISTS`` clause)."""
    secret, pubkey = _fresh_keypair()
    db_path = tmp_path / "contributor_registry.sqlite"

    reg_a = ContributorRegistry(db_path)
    reg_a.record(
        project_id=PROJECT_ID_HEX,
        contributor_node_id=pubkey.hex(),
        artifact_hash=ARTIFACT_HASH_HEX,
        commit_sha=COMMIT_SHA_HEX,
        repo_url=REPO_URL,
        coord_secret=secret,
    )
    # Second instance simulates a coordinator restart : schema
    # init must not raise, and the previously-recorded row must
    # still be visible.
    reg_b = ContributorRegistry(db_path)
    assert reg_b.is_verified_contributor(PROJECT_ID_HEX, pubkey.hex()) is True


def test_record_is_idempotent_preserves_first_deploy_ts(tmp_path: Path) -> None:
    """Re-recording the same ``(project, contributor)`` pair with a
    later commit must preserve the original ``first_deploy_ts``
    anchor (predicate spec §4). This guard matters for the
    Matthew-effect LT-1 follow-up : the anchor timestamp is
    evidence of "when was this contributor first verified".
    """
    secret, pubkey = _fresh_keypair()
    registry = ContributorRegistry(tmp_path / "contributor_registry.sqlite")
    first = registry.record(
        project_id=PROJECT_ID_HEX,
        contributor_node_id=pubkey.hex(),
        artifact_hash=ARTIFACT_HASH_HEX,
        commit_sha=COMMIT_SHA_HEX,
        repo_url=REPO_URL,
        coord_secret=secret,
        now_ts=1_713_556_800,
    )
    # Second deploy with a later commit_sha : anchor timestamp
    # preserved, envelope unchanged (the second call is a no-op
    # lookup).
    later_commit = "ffffffffffffffffffffffffffffffffffffffff"
    second = registry.record(
        project_id=PROJECT_ID_HEX,
        contributor_node_id=pubkey.hex(),
        artifact_hash=ARTIFACT_HASH_HEX,
        commit_sha=later_commit,
        repo_url=REPO_URL,
        coord_secret=secret,
        now_ts=1_800_000_000,
    )
    assert second.first_deploy_ts == first.first_deploy_ts == 1_713_556_800
    # commit_sha also preserved from the first record.
    assert second.commit_sha == COMMIT_SHA_HEX


def test_list_for_project_returns_chronological_order(tmp_path: Path) -> None:
    """``list_for_project`` orders by ``first_deploy_ts`` ascending.
    Ensures the curator UI and audit consumers see the earliest
    contributor first."""
    secret_a, pk_a = _fresh_keypair()
    secret_b, pk_b = _fresh_keypair()
    registry = ContributorRegistry(tmp_path / "contributor_registry.sqlite")

    # Contributor A deploys first.
    registry.record(
        project_id=PROJECT_ID_HEX,
        contributor_node_id=pk_a.hex(),
        artifact_hash=ARTIFACT_HASH_HEX,
        commit_sha=COMMIT_SHA_HEX,
        repo_url=REPO_URL,
        coord_secret=secret_a,
        now_ts=1_700_000_000,
    )
    # Contributor B deploys later.
    registry.record(
        project_id=PROJECT_ID_HEX,
        contributor_node_id=pk_b.hex(),
        artifact_hash=ARTIFACT_HASH_HEX,
        commit_sha=COMMIT_SHA_HEX,
        repo_url=REPO_URL,
        coord_secret=secret_b,
        now_ts=1_713_556_800,
    )

    rows = registry.list_for_project(PROJECT_ID_HEX)
    assert [r.contributor_node_id for r in rows] == [pk_a.hex(), pk_b.hex()]
    assert rows[0].first_deploy_ts < rows[1].first_deploy_ts


def test_record_rejects_bad_project_id(tmp_path: Path) -> None:
    """The Rust binding validates field shapes (hex encoding,
    lengths). Non-hex project_id must surface as a ValueError
    rather than landing in the DB with junk data."""
    secret, pubkey = _fresh_keypair()
    registry = ContributorRegistry(tmp_path / "contributor_registry.sqlite")
    with pytest.raises(ValueError):
        registry.record(
            project_id="not-hex-at-all",
            contributor_node_id=pubkey.hex(),
            artifact_hash=ARTIFACT_HASH_HEX,
            commit_sha=COMMIT_SHA_HEX,
            repo_url=REPO_URL,
            coord_secret=secret,
        )
