# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 22 Phase C — contributor attestation registry API (Couche 2).

Exposes three endpoints :

- ``GET /api/contributor/verify/{project_id}/{node_id_hex}`` —
  loopback lookup returning whether the pair is a verified
  contributor for the project. Consumed by the shell-daemon
  via the loopback proxy at the same path.
- ``GET /api/contributor/project/{project_id}`` — enumerate every
  contributor recorded for a project, oldest-first by
  ``first_deploy_ts`` anchor. Used by the future curator UI
  surfacing "verified contributors" badges.
- ``GET /api/contributor/envelope/{project_id}/{node_id_hex}`` —
  return the full in-toto v1.0 envelope JSON for offline
  auditor replay (`nexus_core.verify_contributor_attestation`
  client-side).

The router has no authentication beyond the loopback bearer
already enforced by ``LoopbackAuthMiddleware`` (Sprint 16).
"""

from __future__ import annotations

import json
from typing import TYPE_CHECKING, Any

import structlog
from fastapi import APIRouter, HTTPException, Request

if TYPE_CHECKING:
    from nexus_coordinator.coordinator import Coordinator

_log = structlog.get_logger(__name__)

router = APIRouter()


def _coordinator(request: Request) -> "Coordinator":
    return request.app.state.coordinator  # type: ignore[no-any-return]


def _validate_hex(value: str, *, expected_len: int, label: str) -> None:
    if len(value) != expected_len:
        raise HTTPException(
            status_code=400,
            detail=f"{label}: expected {expected_len} chars, got {len(value)}",
        )
    if not all(c in "0123456789abcdef" for c in value):
        raise HTTPException(
            status_code=400,
            detail=f"{label}: must be lowercase hex",
        )


@router.get("/api/contributor/verify/{project_id}/{node_id_hex}")
async def verify_contributor(
    project_id: str,
    node_id_hex: str,
    request: Request,
) -> dict[str, Any]:
    """Return ``{"verified": bool}`` for the
    ``(project_id, contributor_node_id)`` pair.

    The daemon proxies this call over loopback when the curator
    list Couche 2 governance-strong gate fires. Sub-millisecond
    SQLite indexed lookup ; the endpoint is meant to be called
    per curator-list entry under the
    ``CURATOR_LIST_MAX_ENTRIES = 256`` cap.
    """
    _validate_hex(project_id, expected_len=64, label="project_id")
    _validate_hex(node_id_hex, expected_len=64, label="node_id_hex")
    coord = _coordinator(request)
    verified = coord.contributor_registry.is_verified_contributor(project_id, node_id_hex)
    return {
        "project_id": project_id,
        "contributor_node_id": node_id_hex,
        "verified": verified,
    }


@router.get("/api/contributor/project/{project_id}")
async def list_contributors(
    project_id: str,
    request: Request,
) -> dict[str, Any]:
    """Enumerate every contributor attestation for a project.

    Results ordered by ``first_deploy_ts`` ascending so the
    earliest (anchor) contributor appears first. Useful for the
    curator UI and for partnership audit flows that want a
    chronological ledger of who has deployed.
    """
    _validate_hex(project_id, expected_len=64, label="project_id")
    coord = _coordinator(request)
    rows = coord.contributor_registry.list_for_project(project_id)
    return {
        "project_id": project_id,
        "count": len(rows),
        "contributors": [
            {
                "contributor_node_id": row.contributor_node_id,
                "first_deploy_ts": row.first_deploy_ts,
                "commit_sha": row.commit_sha,
                "repo_url": row.repo_url,
            }
            for row in rows
        ],
    }


@router.get("/api/contributor/envelope/{project_id}/{node_id_hex}")
async def envelope(
    project_id: str,
    node_id_hex: str,
    request: Request,
) -> dict[str, Any]:
    """Return the full in-toto v1.0 envelope JSON for audit replay.

    Third-party auditors (or the trust-web integration reserved
    S27) call this endpoint through the daemon proxy to obtain a
    verifiable envelope and re-run the Ed25519 signature check
    offline via ``nexus_core.verify_contributor_attestation``.
    """
    _validate_hex(project_id, expected_len=64, label="project_id")
    _validate_hex(node_id_hex, expected_len=64, label="node_id_hex")
    coord = _coordinator(request)
    record = coord.contributor_registry.get(project_id, node_id_hex)
    if record is None:
        raise HTTPException(status_code=404, detail="attestation not found")
    try:
        envelope_obj = json.loads(record.attestation_json)
    except json.JSONDecodeError as exc:
        _log.error(
            "contributor_registry.envelope_parse_failed",
            project_id=project_id,
            contributor_node_id=node_id_hex,
            error=str(exc),
        )
        raise HTTPException(status_code=500, detail="stored envelope is corrupt") from exc
    return envelope_obj
