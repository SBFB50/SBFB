"""
NEXUS -- AlertWorker.

Subscribes to significant events and creates alerts for the
investigation dashboard.  Handles:
- CONTRADICTION_FOUND  -> contradiction alert
- SUSPECT_SCORED       -> alert when score > 60
- FORENSIC_RESULT      -> forensic finding alert
- ENTITY_ENRICHED      -> OSINT discovery alert
"""

from __future__ import annotations

import logging
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)

_SUSPECT_ALERT_THRESHOLD = 60.0


class AlertWorker(ReactiveWorker):
    """Creates alerts for significant investigation events."""

    name = "alert_manager"
    subscriptions = [
        EventType.CONTRADICTION_FOUND,
        EventType.SUSPECT_SCORED,
        EventType.FORENSIC_RESULT,
        EventType.ENTITY_ENRICHED,
    ]

    def __init__(self, bus: EventBus, db: Any) -> None:
        super().__init__(bus)
        self._db = db
        self._manager = None

    def _get_manager(self):
        if self._manager is None:
            from nexus.monitoring.alert_manager import AlertManager
            self._manager = AlertManager(self._db)
        return self._manager

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        mgr = self._get_manager()

        if event.event_type == EventType.CONTRADICTION_FOUND:
            await self._alert_contradiction(mgr, event)

        elif event.event_type == EventType.SUSPECT_SCORED:
            await self._alert_suspect(mgr, event)

        elif event.event_type == EventType.FORENSIC_RESULT:
            await self._alert_forensic(mgr, event)

        elif event.event_type == EventType.ENTITY_ENRICHED:
            await self._alert_osint(mgr, event)

        # Alert worker is a sink -- no output events
        return []

    async def _alert_contradiction(self, mgr: Any, event: NexusEvent) -> None:
        """Create alert for a detected contradiction."""
        desc = event.payload.get("description", "Contradiction detectee")
        ev1 = event.payload.get("evidence_1_title", "?")
        ev2 = event.payload.get("evidence_2_title", "?")

        details = f"Entre '{ev1}' et '{ev2}': {desc}"

        await mgr.create_contradiction_alert(
            case_id=event.case_id,
            details=details,
        )
        logger.info("AlertWorker: contradiction alert created for case %s", event.case_id)

    async def _alert_suspect(self, mgr: Any, event: NexusEvent) -> None:
        """Create alert for a high-scoring suspect."""
        score = event.payload.get("score", 0)
        if score < _SUSPECT_ALERT_THRESHOLD:
            return

        name = event.payload.get("name", "Inconnu")
        severity = "critical" if score >= 80 else "warning"

        await mgr.create_alert(
            case_id=event.case_id,
            alert_type="suspect_high_score",
            severity=severity,
            title=f"Suspect a score eleve: {name} ({score:.0f}%)",
            message=(
                f"Le suspect {name} a atteint un score de suspicion de {score:.1f}/100.\n"
                f"Facteurs: {event.payload.get('factors', {})}"
            ),
            related_id=event.payload.get("suspect_id"),
        )
        logger.info("AlertWorker: suspect alert for '%s' (score=%.1f)", name, score)

    async def _alert_forensic(self, mgr: Any, event: NexusEvent) -> None:
        """Create alert for a forensic analysis result."""
        analysis_type = event.payload.get("analysis_type", "unknown")
        evidence_id = event.payload.get("evidence_id", "")

        await mgr.create_alert(
            case_id=event.case_id,
            alert_type="forensic_result",
            severity="info",
            title=f"Analyse forensique terminee: {analysis_type}",
            message=(
                f"Analyse {analysis_type} completee pour evidence {evidence_id[:8]}.\n"
                f"Detail: {event.payload.get('detail', 'N/A')}"
            ),
            related_id=evidence_id,
        )
        logger.info("AlertWorker: forensic alert for %s", analysis_type)

    async def _alert_osint(self, mgr: Any, event: NexusEvent) -> None:
        """Create alert for significant OSINT discoveries."""
        enrichment = event.payload.get("enrichment", "")
        hit_count = event.payload.get("hit_count", 0)

        if enrichment != "osint_recon" or hit_count == 0:
            return

        name = event.payload.get("name", "?")
        await mgr.create_alert(
            case_id=event.case_id,
            alert_type="osint_discovery",
            severity="info",
            title=f"OSINT: {hit_count} resultat(s) pour '{name}'",
            message=(
                f"La reconnaissance OSINT a trouve {hit_count} comptes/profils "
                f"pour l'entite '{name}'."
            ),
            related_id=event.payload.get("entity_id"),
        )
        logger.info("AlertWorker: OSINT alert for '%s' (%d hits)", name, hit_count)
