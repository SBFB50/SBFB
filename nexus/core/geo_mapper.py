"""
NEXUS -- Geospatial investigation mapper.

Provides geocoding (Nominatim / OSM), routing (OSRM), travel-time
verification, and map-data assembly for a given case.
"""

from __future__ import annotations

import asyncio
import re
from typing import Any, Dict, List, Optional

import httpx
from loguru import logger

from nexus.db.sqlite_db import Database

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

_NOMINATIM_URL = "https://nominatim.openstreetmap.org/search"
_OSRM_URL = "http://router.project-osrm.org/route/v1/driving"
_USER_AGENT = "NEXUS-Investigation/0.1"
_RATE_LIMIT_SEC = 1.1  # Nominatim asks for <= 1 req/sec


class GeoMapper:
    """Geospatial utilities backed by free OSM/OSRM services."""

    def __init__(self, db: Database) -> None:
        self._db = db

    # ------------------------------------------------------------------
    # Geocoding
    # ------------------------------------------------------------------

    async def geocode_address(
        self, address: str, country_hint: str = "France"
    ) -> Optional[Dict[str, Any]]:
        """Convert a free-text address to GPS coordinates via Nominatim.

        Returns ``{"lat": float, "lon": float, "display_name": str}`` or
        *None* when no result is found.

        Addresses that look like bare route numbers (e.g. "D44") or postal
        codes (e.g. "80400") are skipped because Nominatim cannot resolve
        them reliably without more context.

        Results outside metropolitan France are rejected.
        """
        # Skip bare route numbers (D44, N7, A6) and standalone postal codes
        stripped = address.strip()
        if re.match(r'^[A-Z]?\d+$', stripped, re.IGNORECASE):
            logger.info(
                "Skipping geocoding for ambiguous code '{}' (route/postal)",
                stripped,
            )
            return None

        # Add country hint to improve Nominatim accuracy
        query = f"{stripped}, {country_hint}"

        async with httpx.AsyncClient(timeout=15) as client:
            try:
                resp = await client.get(
                    _NOMINATIM_URL,
                    params={"q": query, "format": "json", "limit": 1},
                    headers={"User-Agent": _USER_AGENT},
                )
                resp.raise_for_status()
                data = resp.json()
            except (httpx.HTTPError, ValueError) as exc:
                logger.warning("Geocoding failed for '{}': {}", address, exc)
                return None

        if not data:
            logger.info("No geocoding result for '{}'", address)
            return None

        hit = data[0]
        lat = float(hit["lat"])
        lon = float(hit["lon"])

        # Reject results far outside metropolitan France
        # (lat ~42-51, lon ~-5 to 8)
        if not (42.0 <= lat <= 51.5 and -5.5 <= lon <= 8.5):
            logger.warning(
                "Geocoding result for '{}' is outside France "
                "(lat={:.2f}, lon={:.2f}) -- rejected",
                address, lat, lon,
            )
            return None

        return {
            "lat": lat,
            "lon": lon,
            "display_name": hit.get("display_name", address),
        }

    # ------------------------------------------------------------------
    # Batch geocode all location entities of a case
    # ------------------------------------------------------------------

    async def geocode_entities(self, case_id: str) -> List[Dict[str, Any]]:
        """Geocode every entity of type ``location`` for *case_id*.

        Creates / updates rows in the ``locations`` table and returns a
        list of ``{entity_id, name, lat, lon}`` dicts.
        """
        entities = await self._db.list_entities_by_case(
            case_id, entity_type="location"
        )
        results: List[Dict[str, Any]] = []

        for ent in entities:
            entity_id = ent["id"]
            name = ent["name"]

            # Check if already geocoded
            existing = await self._db.get_location_by_entity(entity_id)
            if existing and existing.get("lat") is not None:
                results.append({
                    "entity_id": entity_id,
                    "name": name,
                    "lat": existing["lat"],
                    "lon": existing["lon"],
                    "location_id": existing["id"],
                    "status": "cached",
                })
                continue

            # Rate-limit
            await asyncio.sleep(_RATE_LIMIT_SEC)

            geo = await self.geocode_address(name)
            if geo is None:
                results.append({
                    "entity_id": entity_id,
                    "name": name,
                    "lat": None,
                    "lon": None,
                    "location_id": None,
                    "status": "not_found",
                })
                continue

            # Determine location_type from entity description / metadata
            loc_type = _guess_location_type(ent)

            if existing:
                # Update existing location row
                await self._db.update_location(
                    existing["id"],
                    lat=geo["lat"],
                    lon=geo["lon"],
                    address=geo["display_name"],
                )
                loc_id = existing["id"]
            else:
                loc = await self._db.create_location(
                    case_id=case_id,
                    entity_id=entity_id,
                    name=name,
                    address=geo["display_name"],
                    lat=geo["lat"],
                    lon=geo["lon"],
                    location_type=loc_type,
                )
                loc_id = loc["id"]

            results.append({
                "entity_id": entity_id,
                "name": name,
                "lat": geo["lat"],
                "lon": geo["lon"],
                "location_id": loc_id,
                "status": "geocoded",
            })

        return results

    # ------------------------------------------------------------------
    # Routing (OSRM)
    # ------------------------------------------------------------------

    async def calculate_route(
        self, origin: str, destination: str
    ) -> Optional[Dict[str, Any]]:
        """Calculate driving route between two addresses via OSRM.

        Returns ``{distance_km, duration_min, geometry_geojson}`` or
        *None* on failure.
        """
        origin_geo = await self.geocode_address(origin)
        if not origin_geo:
            return None

        await asyncio.sleep(_RATE_LIMIT_SEC)

        dest_geo = await self.geocode_address(destination)
        if not dest_geo:
            return None

        return await self._route_coords(
            origin_geo["lat"], origin_geo["lon"],
            dest_geo["lat"], dest_geo["lon"],
        )

    async def _route_coords(
        self,
        lat1: float, lon1: float,
        lat2: float, lon2: float,
    ) -> Optional[Dict[str, Any]]:
        """Low-level OSRM call using raw coordinates."""
        url = f"{_OSRM_URL}/{lon1},{lat1};{lon2},{lat2}"
        async with httpx.AsyncClient(timeout=15) as client:
            try:
                resp = await client.get(
                    url,
                    params={"overview": "full", "geometries": "geojson"},
                )
                resp.raise_for_status()
                data = resp.json()
            except (httpx.HTTPError, ValueError) as exc:
                logger.warning("OSRM routing failed: {}", exc)
                return None

        if data.get("code") != "Ok" or not data.get("routes"):
            logger.info("OSRM returned no route")
            return None

        route = data["routes"][0]
        return {
            "distance_km": round(route["distance"] / 1000, 2),
            "duration_min": round(route["duration"] / 60, 1),
            "geometry_geojson": route["geometry"],
        }

    # ------------------------------------------------------------------
    # Travel-time verification
    # ------------------------------------------------------------------

    async def verify_travel_time(
        self,
        origin: str,
        destination: str,
        claimed_minutes: float,
    ) -> Optional[Dict[str, Any]]:
        """Check whether a claimed travel time is realistic.

        Returns ``{plausible, actual_minutes, claimed_minutes, difference,
        distance_km}`` or *None* if the route cannot be computed.
        """
        route = await self.calculate_route(origin, destination)
        if route is None:
            return None

        actual = route["duration_min"]
        diff = round(claimed_minutes - actual, 1)
        # Allow a 20 % margin
        plausible = actual * 0.8 <= claimed_minutes <= actual * 1.5

        return {
            "plausible": plausible,
            "actual_minutes": actual,
            "claimed_minutes": claimed_minutes,
            "difference_minutes": diff,
            "distance_km": route["distance_km"],
        }

    # ------------------------------------------------------------------
    # Build full map data for a case
    # ------------------------------------------------------------------

    async def build_case_map_data(self, case_id: str) -> Dict[str, Any]:
        """Assemble everything the frontend needs to draw the case map.

        Returns::

            {
                "locations": [ {id, name, lat, lon, location_type, entity_id, address} ],
                "entities_at_locations": [ {location_id, entity_id, entity_name, entity_type} ],
            }
        """
        locations = await self._db.list_locations_by_case(case_id)

        # Build entity lookup for popup enrichment
        entities_at: List[Dict[str, Any]] = []
        for loc in locations:
            if loc.get("entity_id"):
                ent = await self._db.get_entity(loc["entity_id"])
                if ent:
                    entities_at.append({
                        "location_id": loc["id"],
                        "entity_id": ent["id"],
                        "entity_name": ent["name"],
                        "entity_type": ent["entity_type"],
                    })

        return {
            "locations": locations,
            "entities_at_locations": entities_at,
        }


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _guess_location_type(entity: Dict[str, Any]) -> str:
    """Heuristic to classify a location entity by its description."""
    text = (
        (entity.get("description") or "")
        + " "
        + (entity.get("name") or "")
    ).lower()

    if any(w in text for w in ("crime", "meurtre", "agression", "scene")):
        return "crime_scene"
    if any(w in text for w in ("domicile", "maison", "appartement", "residence", "habitation")):
        return "home"
    if any(w in text for w in ("travail", "bureau", "entreprise", "emploi", "societe")):
        return "work"
    if any(w in text for w in ("hopital", "clinique", "hospital")):
        return "hospital"
    if any(w in text for w in ("bar", "restaurant", "cafe", "hotel", "club")):
        return "establishment"
    return "other"
