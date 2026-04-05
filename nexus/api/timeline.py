"""
NEXUS -- Timeline API router.

Exposes the TimelineBuilder as REST endpoints.
"""

from __future__ import annotations

from datetime import datetime
from typing import Optional

from fastapi import APIRouter, Depends, Query

from nexus.api.deps import get_database, get_neo4j
from nexus.core.timeline_builder import TimelineBuilder
from nexus.db.neo4j_db import Neo4jClient
from nexus.db.sqlite_db import Database

router = APIRouter(prefix="/api", tags=["timeline"])


@router.get("/cases/{case_id}/timeline")
async def get_timeline(
    case_id: str,
    db: Database = Depends(get_database),
    neo4j: Neo4jClient = Depends(get_neo4j),
) -> list[dict]:
    """Build a chronological timeline for a case."""
    builder = TimelineBuilder(db=db, neo4j=neo4j)
    return await builder.build_timeline(case_id)


@router.get("/cases/{case_id}/timeline/range")
async def get_timeline_range(
    case_id: str,
    start: datetime = Query(...),
    end: datetime = Query(...),
    db: Database = Depends(get_database),
    neo4j: Neo4jClient = Depends(get_neo4j),
) -> list[dict]:
    """Get timeline events within a date range."""
    builder = TimelineBuilder(db=db, neo4j=neo4j)
    return await builder.get_timeline_range(case_id, start=start, end=end)
