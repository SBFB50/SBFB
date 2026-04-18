# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 20 Phase E.3 — federated warrant canary registry API.

Exposes two endpoints:

- ``GET  /api/canary/network-health`` — fleet snapshot of every
  observed maintainer's canary + duress-ack freshness.
- ``POST /api/canary/observed`` — record a freshly observed
  warrant canary or duress ack. Used by the daemon-side gossip
  subscribe path (when wired in a follow-up sprint) and by the
  ``sbfb canary publish`` CLI to seed the local registry on
  every local publication.

The router has no authentication beyond the loopback bearer
already enforced by ``LoopbackAuthMiddleware`` (Sprint 16) — the
registry is purely observational data; observing a forged canary
is harmless because the freshness diagnostic still surfaces it
as a maintainer the operator must validate, and the operator
holds the trust root (the bootstrap pubkeys in CANARY.txt at the
repo root).
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

import structlog
from fastapi import APIRouter, HTTPException, Request

from nexus_coordinator.canary_registry import (
    coerce_canary_payload,
    coerce_duress_ack_payload,
)

if TYPE_CHECKING:
    from nexus_coordinator.coordinator import Coordinator

_log = structlog.get_logger(__name__)

router = APIRouter()


def _coordinator(request: Request) -> "Coordinator":
    return request.app.state.coordinator  # type: ignore[no-any-return]


@router.get("/api/canary/network-health")
async def network_health(request: Request) -> dict[str, Any]:
    """Fleet snapshot of every observed maintainer's freshness.

    Returns the :class:`NetworkHealth` shape verbatim (FastAPI
    serializes the pydantic model). The ``summary`` field's
    counters give a one-glance fleet picture; ``maintainers`` is
    the per-pubkey detail array.
    """
    coord = _coordinator(request)
    if coord.canary_registry is None:
        raise HTTPException(status_code=503, detail="canary registry not initialised")
    return coord.canary_registry.network_health().model_dump()


@router.post("/api/canary/observed")
async def observed(request: Request) -> dict[str, str]:
    """Record a freshly observed canary or duress ack.

    Body shape:

    ::

        {
          "kind": "canary" | "duress_ack",
          "payload": <wire JSON of the signed object>
        }

    The wire JSON is whatever the Rust ``Canary`` /
    ``DuressAck`` struct serialises to (``#[serde(flatten)]``
    flattens ``signed`` into the top-level object). The handler
    coerces the ``v`` -> ``version`` rename transparently.
    """
    coord = _coordinator(request)
    if coord.canary_registry is None:
        raise HTTPException(status_code=503, detail="canary registry not initialised")

    body = await request.json()
    kind = body.get("kind")
    payload = body.get("payload")
    if not isinstance(payload, dict):
        raise HTTPException(status_code=400, detail="missing or non-object 'payload' field")

    try:
        if kind == "canary":
            obs = coerce_canary_payload(payload)
            coord.canary_registry.observe_canary(obs)
        elif kind == "duress_ack":
            obs = coerce_duress_ack_payload(payload)
            coord.canary_registry.observe_duress_ack(obs)
        else:
            raise HTTPException(
                status_code=400,
                detail="'kind' must be one of 'canary' or 'duress_ack'",
            )
    except HTTPException:
        raise
    except Exception as exc:
        # Pydantic ValidationError or any unexpected shape — surface
        # as 422 so callers can debug the wire mismatch quickly.
        raise HTTPException(status_code=422, detail=f"invalid {kind} payload: {exc}") from exc

    return {"status": "observed", "kind": kind or ""}
