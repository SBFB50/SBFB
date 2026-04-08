"""
NEXUS -- Entities API router.

List entities per case, get entity details, list entity mentions
across evidence.
"""

from __future__ import annotations

from fastapi import APIRouter, Depends, HTTPException, Query

from nexus.db.models import Entity, EntityMention
from nexus.db.sqlite_db import Database

from nexus.api.deps import get_database, paginated_response

router = APIRouter(tags=["entities"])


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/entities
# ------------------------------------------------------------------

@router.get("/api/cases/{case_id}/entities")
async def list_entities(
    case_id: str,
    entity_type: str | None = None,
    limit: int = Query(100, ge=1, le=1000),
    offset: int = Query(0, ge=0),
    db: Database = Depends(get_database),
):
    """List entities for a case, optionally filtered by type, with pagination."""
    all_rows = await db.list_entities_by_case(case_id, entity_type=entity_type, limit=100_000)

    return paginated_response(
        all_rows, offset, limit,
        serializer=lambda r: Entity(**r).model_dump(mode="json"),
    )


# ------------------------------------------------------------------
# GET /api/entities/{entity_id}
# ------------------------------------------------------------------

@router.get("/api/entities/{entity_id}", response_model=Entity)
async def get_entity(
    entity_id: str,
    db: Database = Depends(get_database),
) -> Entity:
    """Retrieve a single entity by ID."""
    row = await db.get_entity(entity_id)
    if row is None:
        raise HTTPException(status_code=404, detail=f"Entity not found: {entity_id}")
    return Entity(**row)


# ------------------------------------------------------------------
# GET /api/entities/{entity_id}/mentions
# ------------------------------------------------------------------

@router.get(
    "/api/entities/{entity_id}/mentions",
    response_model=list[EntityMention],
)
async def list_entity_mentions(
    entity_id: str,
    db: Database = Depends(get_database),
) -> list[EntityMention]:
    """List all mentions of an entity across evidence items."""
    # Verify the entity exists first
    entity = await db.get_entity(entity_id)
    if entity is None:
        raise HTTPException(status_code=404, detail=f"Entity not found: {entity_id}")

    rows = await db.list_mentions_by_entity(entity_id)
    return [EntityMention(**r) for r in rows]
