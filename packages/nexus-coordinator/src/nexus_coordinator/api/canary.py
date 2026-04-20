# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 20 Phase E.3 — federated warrant canary registry API.

Exposes (warrant canary, S20 Phase E.3 + S21 Phase E):

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

Sprint 22 Phase E — watermark canari-input primitive — adds two
more endpoints on the same router (distinct primitive, same URL
prefix for operator clarity):

- ``POST /api/canary/inject-rate`` — live update of the
  :class:`~nexus_coordinator.canary_input.CanaryInputManager`
  ``inject_rate`` (1/N sampling frequency) without a coordinator
  restart.
- ``GET  /api/canary/observed-divergence`` — recent divergence
  records emitted by the Observer when a worker answer failed a
  known-answer probe. Delivers the primitive only; durable
  alerting lands in Sprint 23 B1 Guardrails refactor.

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


# ---------------------------------------------------------------------------
# Sprint 22 Phase E — watermark canari-input primitive endpoints
# ---------------------------------------------------------------------------


@router.post("/api/canary/inject-rate")
async def set_inject_rate(request: Request) -> dict[str, object]:
    """Update the canari-input 1/N sampling frequency live.

    Body shape: ``{"inject_rate": <positive int>}``. A value ``<= 1``
    forces injection on every task — useful for integration tests
    but a terrible production default (workers would catch on).
    The ``CanaryInputManager`` clamps to ``max(1, new_rate)`` so a
    zero or negative value is coerced to ``1`` without erroring.
    """
    coord = _coordinator(request)
    manager = getattr(coord, "canary_input", None)
    if manager is None:
        raise HTTPException(status_code=503, detail="canary_input manager not initialised")
    body = await request.json()
    if not isinstance(body, dict) or "inject_rate" not in body:
        raise HTTPException(status_code=400, detail="body must contain 'inject_rate' integer field")
    try:
        new_rate = int(body["inject_rate"])
    except (TypeError, ValueError) as exc:
        raise HTTPException(status_code=400, detail=f"inject_rate must be an integer: {exc}") from exc
    manager.update_inject_rate(new_rate)
    return {
        "status": "updated",
        "inject_rate": manager.policy.inject_rate,
    }


@router.get("/api/canary/observed-divergence")
async def observed_divergence(request: Request, limit: int = 50) -> dict[str, object]:
    """Return the recent divergence ring-buffer contents.

    ``limit`` caps the number of records returned (default 50, max
    ring capacity is set by the Observer — currently 100). Each
    record carries ``(prompt_id, observed_at_unix, similarity,
    expected_answer, observed_answer, worker_pubkey_hex)``. The
    response also bundles injector + observer counters so operators
    can eyeball the "did anything trigger" signal at a glance.
    """
    coord = _coordinator(request)
    manager = getattr(coord, "canary_input", None)
    if manager is None:
        raise HTTPException(status_code=503, detail="canary_input manager not initialised")
    capped = max(0, min(int(limit), 100))
    divergences = manager.observer.recent_divergences(limit=capped)
    return {
        "divergences": [d.to_dict() for d in divergences],
        "count": len(divergences),
        "injector_stats": manager.injector.stats,
        "observer_stats": manager.observer.stats,
    }
