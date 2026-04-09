"""
NEXUS GOV -- Alert Worker.

Creates alerts for significant events:
- New contradiction detected
- New judicial affair
- Significant voting pattern change
"""

from __future__ import annotations

from typing import Any

from loguru import logger

from nexus.engine import get_db, _new_id, _now_iso, NexusEvent, ReactiveWorker
from nexus.gov.events import GovEventType


class GovAlertWorker(ReactiveWorker):
    name = "gov_alert"
    subscriptions = [
        GovEventType.GOV_CONTRADICTION_FOUND,
        GovEventType.GOV_AFFAIR_ADDED,
        GovEventType.GOV_PATTERN_DETECTED,
    ]

    def __init__(self, bus: Any, db: Any) -> None:
        super().__init__(bus)
        self._db = db

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        etype = event.event_type
        payload = event.payload

        alert_type = ""
        title = ""
        description = ""
        severity = "info"
        politician_id = payload.get("politician_id")

        if etype == GovEventType.GOV_CONTRADICTION_FOUND:
            alert_type = "contradiction"
            title = "Contradiction detectee"
            description = payload.get("description", "")[:500]
            severity = payload.get("severity", "medium")

        elif etype == GovEventType.GOV_AFFAIR_ADDED:
            alert_type = "affair"
            title = "Nouvelle affaire judiciaire"
            description = payload.get("title", "")[:500]
            severity = "high"

        elif etype == GovEventType.GOV_PATTERN_DETECTED:
            alert_type = "pattern"
            title = "Pattern de vote detecte"
            description = (
                f"Type: {payload.get('type', '')}, "
                f"{payload.get('count', 0)} politiciens"
            )
            severity = "info"

        if not alert_type:
            return []

        # Store alert in gov_alerts table
        try:
            from nexus.gov.db import GovernmentDatabase

            alert_id = _new_id()
            now = _now_iso()
            async with get_db() as conn:
                db = GovernmentDatabase(conn)
                await conn.execute(
                    """INSERT INTO gov_alerts
                       (id, alert_type, title, description, severity,
                        politician_id, event_id, created_at)
                       VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
                    (
                        alert_id,
                        alert_type,
                        title,
                        description,
                        severity,
                        politician_id,
                        event.event_id,
                        now,
                    ),
                )
                await conn.commit()

            logger.info(
                "Alert created: [{}] {} — {}",
                severity,
                title,
                description[:60],
            )

            return [
                NexusEvent(
                    event_type=GovEventType.GOV_ALERT_CREATED,
                    case_id="gov",
                    payload={
                        "alert_id": alert_id,
                        "alert_type": alert_type,
                        "severity": severity,
                        "title": title,
                    },
                    source_worker=self.name,
                    parent_event_id=event.event_id,
                )
            ]
        except Exception as exc:
            logger.debug("Alert creation failed: {}", exc)
            return []
