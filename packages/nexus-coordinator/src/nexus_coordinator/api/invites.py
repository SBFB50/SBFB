"""``/invite`` endpoints (create, list, revoke)."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from fastapi import APIRouter, HTTPException, Request
from pydantic import BaseModel, Field

if TYPE_CHECKING:
    from nexus_coordinator.coordinator import Coordinator

router = APIRouter(prefix="/invite", tags=["invite"])


class CreateInviteBody(BaseModel):
    scope: str = Field("worker", pattern="^(worker|observer)$")
    expiry_secs: int = Field(7 * 24 * 3600, ge=60)
    max_uses: int | None = Field(None, ge=1)
    note: str | None = None


def _coord(request: Request) -> "Coordinator":
    return request.app.state.coordinator  # type: ignore[no-any-return]


@router.post("/create")
async def create_invite(request: Request, body: CreateInviteBody) -> dict[str, Any]:
    coord = _coord(request)
    ledger = coord.invite_ledger
    if ledger is None:
        raise HTTPException(status_code=503, detail="invite ledger not yet initialised")
    try:
        record = await ledger.mint(
            project_id=coord.state.doc_id or "",
            project_name=coord.project_name,
            scope=body.scope,
            tasks_doc_ticket=coord.state.tasks_doc_ticket,
            expiry_secs=body.expiry_secs,
            max_uses=body.max_uses,
            note=body.note,
        )
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e)) from e
    return {
        "id": record.id,
        "wire": record.wire,
        "scope": record.scope,
        "expires_at": record.expires_at,
        "max_uses": record.max_uses,
        "note": record.note,
    }


@router.get("")
async def list_invites(request: Request) -> dict[str, Any]:
    coord = _coord(request)
    ledger = coord.invite_ledger
    if ledger is None:
        raise HTTPException(status_code=503, detail="invite ledger not yet initialised")
    records = await ledger.list_invites()
    return {
        "invites": [
            {
                "id": r.id,
                "scope": r.scope,
                "project_id": r.project_id,
                "expires_at": r.expires_at,
                "max_uses": r.max_uses,
                "uses_count": r.uses_count,
                "revoked_at": r.revoked_at,
                "note": r.note,
                "created_at": r.created_at,
            }
            for r in records
        ],
        "count": len(records),
    }


@router.delete("/{invite_id}")
async def revoke_invite(request: Request, invite_id: str) -> dict[str, Any]:
    coord = _coord(request)
    ledger = coord.invite_ledger
    if ledger is None:
        raise HTTPException(status_code=503, detail="invite ledger not yet initialised")
    ok = await ledger.revoke(invite_id)
    if not ok:
        raise HTTPException(status_code=404, detail=f"invite {invite_id} not found or already revoked")
    return {"id": invite_id, "revoked": True}
