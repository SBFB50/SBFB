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

Sprint 21 Phase E (T-NN+1 tech debt resolved): canary observations
now go through ``nexus_core.verify_canary`` Ed25519 signature
verification at ingest. A forged ``canary`` payload is rejected
with HTTP 401 before it can pollute the registry. This closes the
Sprint 20 Phase E observational-only gap (registry accepted any
shape-valid payload, deferred verify to operator inspection). The
``duress_ack`` path is intentionally NOT verified here yet — the
T-NN+1 carry only covered canary verify; ``verify_duress_ack``
binding would be a S22+ follow-up if hardened end-to-end matters
for that channel.

The router has no authentication beyond the loopback bearer
already enforced by ``LoopbackAuthMiddleware`` (Sprint 16). The
forged-canary defence above is orthogonal to the loopback bearer:
even a legitimate loopback caller (e.g. a buggy local CLI) cannot
poison the registry with garbage.
"""

from __future__ import annotations

import json
from typing import TYPE_CHECKING, Any

import nexus_core
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
            # Sprint 21 Phase E (T-NN+1 tech debt resolved) :
            # verify Ed25519 signature at ingest before letting
            # the observation reach the registry. The Rust binding
            # consumes the wire JSON directly (the same flat shape
            # produced by `canary_wire_bytes` and accepted by
            # `coerce_canary_payload` below) and raises on any
            # signature / version / hex parse error. We surface
            # those as HTTP 401 because the failure mode is
            # cryptographic, not request-shape.
            try:
                nexus_core.verify_canary(json.dumps(payload))
            except Exception as exc:
                raise HTTPException(
                    status_code=401,
                    detail=f"canary signature verification failed: {exc}",
                ) from exc
            obs = coerce_canary_payload(payload)
            coord.canary_registry.observe_canary(obs)
        elif kind == "duress_ack":
            # Note Sprint 21 Phase E: `verify_duress_ack` binding
            # is intentionally NOT yet exposed — the T-NN+1 carry
            # only mandated canary verify at ingest. Duress acks
            # remain observational-only at the registry layer
            # pending S22+ follow-up.
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
