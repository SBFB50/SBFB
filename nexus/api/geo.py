"""
NEXUS -- Geospatial API router.

Exposes geocoding, routing, travel-time verification, and map-data
endpoints for the investigation dashboard.
"""

from __future__ import annotations

from typing import Optional

from fastapi import APIRouter, Depends
from pydantic import BaseModel

from nexus.api.deps import get_database, get_geo_mapper
from nexus.core.geo_mapper import GeoMapper
from nexus.db.sqlite_db import Database

router = APIRouter(prefix="/api", tags=["geo"])


# ---------------------------------------------------------------------------
# Request bodies
# ---------------------------------------------------------------------------

class RouteRequest(BaseModel):
    origin: str
    destination: str


class VerifyTravelRequest(BaseModel):
    origin: str
    destination: str
    claimed_minutes: float


# ---------------------------------------------------------------------------
# Endpoints
# ---------------------------------------------------------------------------

@router.post("/cases/{case_id}/geocode")
async def geocode_case_entities(
    case_id: str,
    mapper: GeoMapper = Depends(get_geo_mapper),
) -> dict:
    """Geocode all location-type entities for *case_id*.

    Returns a summary list with geocoding status for each entity.
    """
    results = await mapper.geocode_entities(case_id)
    geocoded = sum(1 for r in results if r["status"] == "geocoded")
    cached = sum(1 for r in results if r["status"] == "cached")
    not_found = sum(1 for r in results if r["status"] == "not_found")
    return {
        "total": len(results),
        "geocoded": geocoded,
        "cached": cached,
        "not_found": not_found,
        "results": results,
    }


@router.get("/cases/{case_id}/map")
async def get_case_map(
    case_id: str,
    mapper: GeoMapper = Depends(get_geo_mapper),
) -> dict:
    """Return all data needed to render the investigation map."""
    return await mapper.build_case_map_data(case_id)


@router.post("/cases/{case_id}/route")
async def calculate_route(
    case_id: str,
    body: RouteRequest,
    mapper: GeoMapper = Depends(get_geo_mapper),
) -> dict:
    """Calculate a driving route between two addresses."""
    result = await mapper.calculate_route(body.origin, body.destination)
    if result is None:
        return {"error": "Impossible de calculer le trajet."}
    return result


@router.post("/cases/{case_id}/verify-travel")
async def verify_travel_time(
    case_id: str,
    body: VerifyTravelRequest,
    mapper: GeoMapper = Depends(get_geo_mapper),
) -> dict:
    """Verify whether a claimed travel time between two addresses is plausible."""
    result = await mapper.verify_travel_time(
        body.origin, body.destination, body.claimed_minutes
    )
    if result is None:
        return {"error": "Impossible de verifier le trajet."}
    return result
