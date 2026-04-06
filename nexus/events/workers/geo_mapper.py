"""
NEXUS -- GeoMapperWorker.

Subscribes to ENTITY_DISCOVERED, filtering for location entities.
Geocodes the location name via GeoMapper and emits LOCATION_GEOCODED.
"""

from __future__ import annotations

import logging
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)


class GeoMapperWorker(ReactiveWorker):
    """Geocodes location entities via Nominatim/OSM."""

    name = "geo_mapper"
    subscriptions = [EventType.ENTITY_DISCOVERED]

    def __init__(self, bus: EventBus, db: Any) -> None:
        super().__init__(bus)
        self._db = db
        self._mapper = None

    def _get_mapper(self):
        if self._mapper is None:
            from nexus.core.geo_mapper import GeoMapper
            self._mapper = GeoMapper(self._db)
        return self._mapper

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        entity_type = event.payload.get("entity_type", "")
        if entity_type != "location":
            return []

        entity_id = event.payload.get("entity_id", "")
        name = event.payload.get("name", "")

        if not name:
            return []

        logger.info("GeoMapper: geocoding location '%s'", name)

        geo = await self._get_mapper().geocode_address(name)
        if not geo:
            logger.debug("GeoMapper: no result for '%s'", name)
            return []

        # Store in DB if not already present
        existing = await self._db.get_location_by_entity(entity_id)
        if not existing:
            try:
                await self._db.create_location(
                    case_id=event.case_id,
                    entity_id=entity_id,
                    name=name,
                    address=geo.get("display_name", name),
                    lat=geo["lat"],
                    lon=geo["lon"],
                    location_type="other",
                )
            except Exception as exc:
                logger.warning("GeoMapper: DB store failed for '%s': %s", name, exc)

        logger.info(
            "GeoMapper: geocoded '%s' -> lat=%.4f, lon=%.4f",
            name, geo["lat"], geo["lon"],
        )

        return [NexusEvent(
            event_type=EventType.LOCATION_GEOCODED,
            case_id=event.case_id,
            payload={
                "entity_id": entity_id,
                "name": name,
                "lat": geo["lat"],
                "lon": geo["lon"],
                "display_name": geo.get("display_name", ""),
            },
            source_worker=self.name,
            parent_event_id=event.event_id,
        )]
