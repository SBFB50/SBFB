"""
NEXUS -- Alert manager for the monitoring subsystem.

Creates typed alerts when:
- A monitoring hit exceeds the relevance threshold
- A hypothesis score shifts significantly (> 15 points)
- A new entity is discovered
- A contradiction is detected between evidence
"""

from __future__ import annotations

from typing import Any, Dict, Optional

from loguru import logger

from nexus.db.sqlite_db import Database


class AlertManager:
    """High-level alert factory backed by the Database CRUD layer.

    Usage::

        async with get_db() as conn:
            db = Database(conn)
            mgr = AlertManager(db)
            await mgr.create_monitoring_alert(case_id, result)
    """

    def __init__(self, db: Database) -> None:
        self._db = db

    # ------------------------------------------------------------------
    # Generic alert creation
    # ------------------------------------------------------------------

    async def create_alert(
        self,
        case_id: str,
        alert_type: str,
        severity: str,
        title: str,
        message: str,
        related_id: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Create an alert and return the full record."""
        alert = await self._db.create_alert(
            case_id=case_id,
            alert_type=alert_type,
            severity=severity,
            title=title,
            message=message,
            related_id=related_id,
        )
        logger.info(
            "Alert created: [{}] {} — case={}", severity, title, case_id
        )
        return alert

    # ------------------------------------------------------------------
    # Monitoring hit
    # ------------------------------------------------------------------

    async def create_monitoring_alert(
        self,
        case_id: str,
        result: Dict[str, Any],
    ) -> Dict[str, Any]:
        """Create an alert from a high-relevance monitoring result.

        Severity is determined by the relevance score:
        - >= 80: critical
        - >= 60: warning
        - < 60:  info
        """
        score = result.get("relevance_score") or 0.0
        if score >= 80:
            severity = "critical"
        elif score >= 60:
            severity = "warning"
        else:
            severity = "info"

        title = f"Monitoring: {result.get('title', 'Nouveau resultat')}"
        # Truncate title to a reasonable length
        if len(title) > 120:
            title = title[:117] + "..."

        url = result.get("url", "")
        snippet = result.get("snippet", "")[:300]
        message = (
            f"Resultat de monitoring avec score de pertinence {score:.0f}/100.\n\n"
            f"URL: {url}\n"
            f"Extrait: {snippet}"
        )

        return await self.create_alert(
            case_id=case_id,
            alert_type="monitoring_hit",
            severity=severity,
            title=title,
            message=message,
            related_id=result.get("id"),
        )

    # ------------------------------------------------------------------
    # Hypothesis score shift
    # ------------------------------------------------------------------

    async def create_score_shift_alert(
        self,
        case_id: str,
        hypothesis_id: str,
        old_score: float,
        new_score: float,
    ) -> Dict[str, Any]:
        """Create an alert when a hypothesis score changes by > 15 points."""
        delta = new_score - old_score
        abs_delta = abs(delta)

        if abs_delta < 15:
            # Should not happen if caller checks, but guard anyway
            logger.debug(
                "Score shift too small ({:.1f}) for alert", abs_delta
            )

        direction = "augmente" if delta > 0 else "diminue"

        if abs_delta >= 30:
            severity = "critical"
        elif abs_delta >= 15:
            severity = "warning"
        else:
            severity = "info"

        title = f"Score hypothese {direction} de {abs_delta:.0f} points"
        message = (
            f"Le score d'une hypothese a {direction} significativement.\n\n"
            f"Score precedent: {old_score:.1f}\n"
            f"Nouveau score: {new_score:.1f}\n"
            f"Delta: {delta:+.1f}"
        )

        return await self.create_alert(
            case_id=case_id,
            alert_type="score_shift",
            severity=severity,
            title=title,
            message=message,
            related_id=hypothesis_id,
        )

    # ------------------------------------------------------------------
    # New entity discovered
    # ------------------------------------------------------------------

    async def create_new_entity_alert(
        self,
        case_id: str,
        entity: Dict[str, Any],
    ) -> Dict[str, Any]:
        """Create an alert when a new entity is extracted from evidence."""
        entity_name = entity.get("name", "Inconnu")
        entity_type = entity.get("entity_type", entity.get("type", "other"))

        title = f"Nouvelle entite: {entity_name} ({entity_type})"
        if len(title) > 120:
            title = title[:117] + "..."

        description = entity.get("description", "")
        message = (
            f"Une nouvelle entite a ete extraite automatiquement.\n\n"
            f"Nom: {entity_name}\n"
            f"Type: {entity_type}\n"
            f"Description: {description[:200]}"
        )

        return await self.create_alert(
            case_id=case_id,
            alert_type="new_entity",
            severity="info",
            title=title,
            message=message,
            related_id=entity.get("id"),
        )

    # ------------------------------------------------------------------
    # Contradiction detected
    # ------------------------------------------------------------------

    async def create_contradiction_alert(
        self,
        case_id: str,
        details: str,
    ) -> Dict[str, Any]:
        """Create an alert when contradictions are found between evidence."""
        title = "Contradiction detectee entre preuves"
        # Truncate details for the message if very long
        truncated = details[:500] if len(details) > 500 else details
        message = (
            f"Une contradiction a ete identifiee dans les preuves du dossier.\n\n"
            f"{truncated}"
        )

        return await self.create_alert(
            case_id=case_id,
            alert_type="contradiction",
            severity="warning",
            title=title,
            message=message,
        )
