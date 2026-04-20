# SPDX-License-Identifier: AGPL-3.0-or-later
"""Contributor attestation registry — Sprint 22 Phase C (Couche 2).

Coordinator-side SQLite-backed registry that records signed
:class:`~nexus_core_rs::attestations::ContributorAttestation`
envelopes at verified-deploy time and answers
``is_verified_contributor`` queries for the daemon's curator-list
Couche 2 governance-strong gate.

Design targets :

- **Source of truth** : the SQLite ``contributor_attestations``
  table lives on the coordinator's disk ; the daemon proxies
  queries over loopback HTTP.
- **Idempotent record** : if the same
  ``(project_id, contributor_node_id)`` pair is already present,
  ``record()`` returns the stored anchor timestamp rather than
  minting a new attestation. This preserves the "first deploy" anchor
  semantics (§4 of
  ``docs/security/CONTRIBUTOR_ATTESTATION_PREDICATE.md``).
- **Offline verify** : ``is_verified_contributor`` only queries the
  DB — it does not re-fetch the repo, does not re-check Keyoxide,
  does not reach the network. Sub-millisecond lookup under the
  curator-list cap (256 entries × a few DB reads).

.. warning::

    Interim Sybil-resistance S22. Contributor selection is still
    biased toward high-kudos workers (Matthew effect one layer
    deeper — high-kudos workers publish more projects and earn
    more attestations). Post-v1.0 LT-1 Kudos-v2 reform will
    introduce log-utility + DRF + EMA trust to break this cycle.
    See :

    - ``docs/FAIRNESS_VISION.md §7`` "Design-conflict S22"
    - ``docs/release/ROADMAP_COMMITMENTS.md §LT-1``
"""

from __future__ import annotations

import json
import sqlite3
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import nexus_core
import structlog

_log = structlog.get_logger(__name__)

#: Name of the SQLite file inside the coordinator's state dir. Kept
#: distinct from the warrant-canary and quarantine DBs so a schema
#: evolution on one does not require migrating the others.
CONTRIBUTOR_REGISTRY_DB_NAME: str = "contributor_registry.sqlite"


@dataclass(frozen=True, slots=True)
class ContributorRecord:
    """A single row in the ``contributor_attestations`` table."""

    project_id: str
    contributor_node_id: str
    first_deploy_ts: int
    commit_sha: str
    repo_url: str
    coord_sig_b64: str
    attestation_json: str


class ContributorRegistry:
    """SQLite-backed registry of contributor attestations.

    Single-writer, single-reader contract : the coordinator process
    owns the file and never concurrently spawns a second writer.
    Thread safety is delegated to the ``sqlite3`` module's default
    per-connection locking (the registry opens a fresh connection
    per public call, which is the simplest correct pattern for the
    FastAPI request-scoped access pattern and matches
    :mod:`nexus_coordinator.canary_registry`).
    """

    def __init__(self, db_path: Path) -> None:
        self._db_path = db_path
        self._db_path.parent.mkdir(parents=True, exist_ok=True)
        self._init_schema()

    # ------------------------------------------------------------------
    # Schema
    # ------------------------------------------------------------------

    def _init_schema(self) -> None:
        """Create the table + indices idempotently.

        WAL mode is enabled so a read (e.g. ``/api/contributor/verify``)
        never blocks behind an in-flight deploy's write. The
        ``UNIQUE (project_id, contributor_node_id)`` constraint is
        what makes ``record()`` idempotent without a SELECT-then-
        INSERT race.
        """
        with sqlite3.connect(self._db_path) as conn:
            conn.execute("PRAGMA journal_mode = WAL")
            conn.execute("PRAGMA synchronous = NORMAL")
            conn.execute(
                """
                CREATE TABLE IF NOT EXISTS contributor_attestations (
                    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
                    project_id              TEXT NOT NULL,
                    contributor_node_id     TEXT NOT NULL,
                    first_deploy_ts         INTEGER NOT NULL,
                    commit_sha              TEXT NOT NULL,
                    repo_url                TEXT NOT NULL,
                    coord_sig_b64           TEXT NOT NULL,
                    attestation_json        TEXT NOT NULL,
                    recorded_at_epoch_s     INTEGER NOT NULL,
                    UNIQUE (project_id, contributor_node_id)
                )
                """
            )
            conn.execute("CREATE INDEX IF NOT EXISTS idx_contrib_project ON contributor_attestations(project_id)")
            conn.execute("CREATE INDEX IF NOT EXISTS idx_contrib_node ON contributor_attestations(contributor_node_id)")
            conn.commit()

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def record(
        self,
        *,
        project_id: str,
        contributor_node_id: str,
        artifact_hash: str,
        commit_sha: str,
        repo_url: str,
        coord_secret: bytes,
        now_ts: int | None = None,
    ) -> ContributorRecord:
        """Record (or return existing) attestation for this pair.

        On first record, mints a fresh
        :class:`~nexus_core_rs::ContributorAttestation` via the
        Rust ``nexus_core.build_contributor_attestation`` binding,
        stores it, and returns the row. On re-record (same
        ``(project_id, contributor_node_id)``), returns the stored
        row unchanged — the ``first_deploy_ts`` anchor is
        preserved across subsequent deploys (cf. predicate spec §4).

        Idempotency is enforced by the ``UNIQUE`` constraint +
        ``INSERT OR IGNORE`` ; the caller does not need to
        pre-check.

        ``coord_secret`` is the coordinator's 32-byte Ed25519 secret
        key ; never logged, never persisted in the DB. Only the
        signature and envelope JSON land on disk.
        """
        existing = self._fetch_one(project_id, contributor_node_id)
        if existing is not None:
            return existing

        now = now_ts if now_ts is not None else int(time.time())
        envelope_json = nexus_core.build_contributor_attestation(
            project_id_hex=project_id,
            artifact_hash_hex=artifact_hash,
            contributor_node_id_hex=contributor_node_id,
            first_deploy_ts=now,
            commit_sha_hex=commit_sha,
            repo_url=repo_url,
            secret=coord_secret,
        )
        envelope = json.loads(envelope_json)
        coord_sig_b64 = envelope["predicate"]["attestation_coord_sig"]

        with sqlite3.connect(self._db_path) as conn:
            conn.execute(
                "INSERT OR IGNORE INTO contributor_attestations "
                "(project_id, contributor_node_id, first_deploy_ts, commit_sha, "
                " repo_url, coord_sig_b64, attestation_json, recorded_at_epoch_s) "
                "VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    project_id,
                    contributor_node_id,
                    now,
                    commit_sha,
                    repo_url,
                    coord_sig_b64,
                    envelope_json,
                    now,
                ),
            )
            conn.commit()

        _log.info(
            "contributor_registry.recorded",
            project_id=project_id,
            contributor_node_id=contributor_node_id,
            first_deploy_ts=now,
            commit_sha=commit_sha,
        )
        # Re-fetch to get the stored value (handles the race where
        # a concurrent writer already inserted before us ; the
        # IGNORE above means our INSERT was a no-op and the stored
        # first_deploy_ts is the earlier one — which is the correct
        # anchor).
        stored = self._fetch_one(project_id, contributor_node_id)
        assert stored is not None, "INSERT followed by fetch must find the row"
        return stored

    def is_verified_contributor(
        self,
        project_id: str,
        contributor_node_id: str,
    ) -> bool:
        """Return ``True`` iff the pair has an attestation stored.

        Sub-millisecond lookup (indexed). Called by the daemon's
        loopback proxy when the curator-list Couche 2 governance-
        strong gate is active.
        """
        return self._fetch_one(project_id, contributor_node_id) is not None

    def get(
        self,
        project_id: str,
        contributor_node_id: str,
    ) -> ContributorRecord | None:
        """Return the full row for inspection / audit replay, or
        ``None`` if the pair is not registered."""
        return self._fetch_one(project_id, contributor_node_id)

    def list_for_project(self, project_id: str) -> list[ContributorRecord]:
        """Return every contributor attestation recorded for a
        project, ordered by ``first_deploy_ts`` ascending (oldest
        contributor first)."""
        with sqlite3.connect(self._db_path) as conn:
            conn.row_factory = sqlite3.Row
            cur = conn.execute(
                "SELECT project_id, contributor_node_id, first_deploy_ts, "
                "       commit_sha, repo_url, coord_sig_b64, attestation_json "
                "FROM contributor_attestations "
                "WHERE project_id = ? "
                "ORDER BY first_deploy_ts ASC",
                (project_id,),
            )
            return [_row_to_record(row) for row in cur.fetchall()]

    # ------------------------------------------------------------------
    # Internal
    # ------------------------------------------------------------------

    def _fetch_one(
        self,
        project_id: str,
        contributor_node_id: str,
    ) -> ContributorRecord | None:
        with sqlite3.connect(self._db_path) as conn:
            conn.row_factory = sqlite3.Row
            cur = conn.execute(
                "SELECT project_id, contributor_node_id, first_deploy_ts, "
                "       commit_sha, repo_url, coord_sig_b64, attestation_json "
                "FROM contributor_attestations "
                "WHERE project_id = ? AND contributor_node_id = ?",
                (project_id, contributor_node_id),
            )
            row = cur.fetchone()
        if row is None:
            return None
        return _row_to_record(row)


def _row_to_record(row: Any) -> ContributorRecord:
    return ContributorRecord(
        project_id=row["project_id"],
        contributor_node_id=row["contributor_node_id"],
        first_deploy_ts=int(row["first_deploy_ts"]),
        commit_sha=row["commit_sha"],
        repo_url=row["repo_url"],
        coord_sig_b64=row["coord_sig_b64"],
        attestation_json=row["attestation_json"],
    )
