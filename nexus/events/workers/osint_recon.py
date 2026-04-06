"""
NEXUS -- OSINTReconWorker.

Subscribes to ENTITY_DISCOVERED, filtering for person and email types.
Runs passive OSINT reconnaissance (HoleheRecon for emails,
SocialRecon for person names) and emits ENTITY_ENRICHED.
"""

from __future__ import annotations

import logging
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)

_OSINT_ENTITY_TYPES = {"person", "email"}


class OSINTReconWorker(ReactiveWorker):
    """Runs passive OSINT recon on person/email entities."""

    name = "osint_recon"
    subscriptions = [EventType.ENTITY_DISCOVERED]

    def __init__(self, bus: EventBus) -> None:
        super().__init__(bus)
        # Lazy-import to avoid pulling heavy deps at module level
        self._holehe = None
        self._social = None

    def _get_holehe(self):
        if self._holehe is None:
            from nexus.recon.holehe_recon import HoleheRecon
            self._holehe = HoleheRecon()
        return self._holehe

    def _get_social(self):
        if self._social is None:
            from nexus.recon.social_recon import SocialRecon
            self._social = SocialRecon()
        return self._social

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        entity_type = event.payload.get("entity_type", "")
        if entity_type not in _OSINT_ENTITY_TYPES:
            return []

        entity_id = event.payload.get("entity_id", "")
        name = event.payload.get("name", "")

        if not name:
            return []

        results: list[dict] = []

        if entity_type == "email":
            logger.info("OSINTRecon: checking email '%s'", name)
            try:
                hits = await self._get_holehe().check_email(name)
                results.extend(hits)
            except Exception as exc:
                logger.warning("OSINTRecon: holehe failed for '%s': %s", name, exc)

            # Also search the username part on social platforms
            try:
                social_hits = await self._get_social().search_email_username(name)
                found = [h for h in social_hits if h.get("exists")]
                results.extend(found)
            except Exception as exc:
                logger.warning("OSINTRecon: social recon failed for '%s': %s", name, exc)

        elif entity_type == "person":
            # Try social platform search with the person name as username
            username = name.replace(" ", "").lower()
            if len(username) >= 3:
                logger.info("OSINTRecon: searching social for '%s'", username)
                try:
                    social_hits = await self._get_social().search_username(username)
                    found = [h for h in social_hits if h.get("exists")]
                    results.extend(found)
                except Exception as exc:
                    logger.warning(
                        "OSINTRecon: social recon failed for '%s': %s",
                        username, exc,
                    )

        if not results:
            logger.debug("OSINTRecon: no hits for entity '%s'", name)
            return []

        logger.info(
            "OSINTRecon: found %d hits for entity '%s' (%s)",
            len(results), name, entity_type,
        )

        return [NexusEvent(
            event_type=EventType.ENTITY_ENRICHED,
            case_id=event.case_id,
            payload={
                "entity_id": entity_id,
                "name": name,
                "entity_type": entity_type,
                "enrichment": "osint_recon",
                "hits": results,
                "hit_count": len(results),
            },
            source_worker=self.name,
            parent_event_id=event.event_id,
        )]
