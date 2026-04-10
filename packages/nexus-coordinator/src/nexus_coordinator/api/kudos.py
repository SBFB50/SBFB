"""``/kudos`` and ``/kudos/verify`` endpoints."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from fastapi import APIRouter, HTTPException, Request

if TYPE_CHECKING:
    from nexus_coordinator.coordinator import Coordinator

router = APIRouter(prefix="/kudos", tags=["kudos"])


def _coord(request: Request) -> "Coordinator":
    return request.app.state.coordinator  # type: ignore[no-any-return]


@router.get("")
async def list_kudos(request: Request, worker_pubkey_hex: str | None = None) -> dict[str, Any]:
    coord = _coord(request)
    kudos = coord.kudos_ledger
    if kudos is None:
        raise HTTPException(status_code=503, detail="kudos ledger not yet initialised")
    worker_pubkey = bytes.fromhex(worker_pubkey_hex) if worker_pubkey_hex else None
    entries = await kudos.list_entries(worker_pubkey=worker_pubkey)
    return {
        "entries": [
            {
                "id": e.id,
                "worker_pubkey_hex": e.worker_pubkey.hex(),
                "task_id": e.task_id,
                "tokens": e.tokens,
                "quality_factor": e.quality_factor,
                "trust_multiplier": e.trust_multiplier,
                "amount": e.amount,
                "awarded_at": e.awarded_at,
                "entry_hash_hex": e.entry_hash.hex(),
            }
            for e in entries
        ],
        "count": len(entries),
    }


@router.get("/verify")
async def verify_kudos(request: Request) -> dict[str, Any]:
    coord = _coord(request)
    kudos = coord.kudos_ledger
    if kudos is None:
        raise HTTPException(status_code=503, detail="kudos ledger not yet initialised")
    ok, bad = await kudos.verify_chain_integrity()
    return {"ok": ok, "first_bad_row_id": bad}
