"""
NEXUS -- ForensicRouterWorker.

Subscribes to EVIDENCE_ADDED, filtering for image and audio types.
Routes to the appropriate forensic analyzer (BPA, TraceAnalyzer,
AcousticAnalyzer) based on evidence metadata keywords.
Emits FORENSIC_RESULT on completion.
"""

from __future__ import annotations

import logging
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)

_FORENSIC_EVIDENCE_TYPES = {"image", "audio"}

# Keywords that route to specific forensic modules
_BPA_KEYWORDS = {"sang", "blood", "tache", "stain", "bpa", "eclaboussure", "spatter"}
_TRACE_KEYWORDS = {
    "empreinte", "fingerprint", "trace", "pneu", "tire",
    "outil", "tool", "chaussure", "shoe", "fibre",
}
_ACOUSTIC_KEYWORDS = {
    "audio", "son", "voix", "voice", "enregistrement", "recording",
    "acoustique", "telephone", "appel",
}


class ForensicRouterWorker(ReactiveWorker):
    """Routes evidence to the appropriate forensic analyzer."""

    name = "forensic_router"
    subscriptions = [EventType.EVIDENCE_ADDED]

    def __init__(
        self,
        bus: EventBus,
        router: Any,
    ) -> None:
        super().__init__(bus)
        self._router = router

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        evidence_type = event.payload.get("evidence_type", "")
        if evidence_type not in _FORENSIC_EVIDENCE_TYPES:
            return []

        title = event.payload.get("title", "").lower()
        evidence_id = event.payload.get("evidence_id", "")

        # Determine which forensic module to invoke
        analysis_type = self._classify(title, evidence_type)
        if not analysis_type:
            logger.debug(
                "ForensicRouter: no forensic match for '%s' (type=%s)",
                title, evidence_type,
            )
            return []

        logger.info(
            "ForensicRouter: routing evidence %s to %s analysis",
            evidence_id[:8], analysis_type,
        )

        result_payload: dict[str, Any] = {
            "evidence_id": evidence_id,
            "analysis_type": analysis_type,
            "status": "completed",
        }

        try:
            if analysis_type == "bpa":
                from nexus.forensics.blood_pattern import BloodPatternAnalyzer
                analyzer = BloodPatternAnalyzer(self._router)
                # BPA requires an image path; we note the routing for now
                result_payload["detail"] = "BPA analysis routed"

            elif analysis_type == "trace":
                from nexus.forensics.trace_analyzer import TraceAnalyzer
                analyzer = TraceAnalyzer(self._router)
                result_payload["detail"] = "Trace analysis routed"

            elif analysis_type == "acoustic":
                from nexus.forensics.acoustic_analysis import AcousticAnalyzer
                analyzer = AcousticAnalyzer(self._router)
                result_payload["detail"] = "Acoustic analysis routed"

        except Exception as exc:
            logger.warning(
                "ForensicRouter: failed to route evidence %s to %s: %s",
                evidence_id[:8], analysis_type, exc,
            )
            result_payload["status"] = "error"
            result_payload["error"] = str(exc)

        return [NexusEvent(
            event_type=EventType.FORENSIC_RESULT,
            case_id=event.case_id,
            payload=result_payload,
            source_worker=self.name,
            parent_event_id=event.event_id,
        )]

    def _classify(self, title: str, evidence_type: str) -> str | None:
        """Classify evidence into a forensic analysis type by keywords."""
        words = set(title.split())

        if evidence_type == "image":
            if words & _BPA_KEYWORDS:
                return "bpa"
            if words & _TRACE_KEYWORDS:
                return "trace"
            # Default for images: trace analysis
            return "trace"

        if evidence_type == "audio":
            if words & _ACOUSTIC_KEYWORDS:
                return "acoustic"
            # Default for audio
            return "acoustic"

        return None
