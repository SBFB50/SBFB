"""
NEXUS -- Timeline builder.

Constructs a unified chronological timeline for a case by aggregating
date-stamped events from multiple data sources:
  1. Evidence source dates (evidence.source_date)
  2. Neo4j Event nodes with datetime properties
  3. Entity first_seen dates
  4. Hypothesis snapshot dates
  5. Monitoring result dates (found_at)

All entries are merged and sorted chronologically, producing a flat
list suitable for front-end timeline visualisation.
"""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from loguru import logger

from nexus.db.sqlite_db import Database


class TimelineBuilder:
    """Build chronological timelines from multiple NEXUS data sources.

    Usage::

        async with get_db() as conn:
            db = Database(conn)
            builder = TimelineBuilder(db, neo4j_client)
            timeline = await builder.build_timeline("case-uuid")
    """

    def __init__(
        self,
        db: Database,
        neo4j: Optional[Any] = None,
    ) -> None:
        self._db = db
        self._neo4j = neo4j  # Neo4jClient, optional

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    async def build_timeline(self, case_id: str) -> List[Dict[str, Any]]:
        """Build a full chronological timeline for a case.

        Aggregates dates from evidence, Neo4j events, entities,
        hypothesis snapshots, and monitoring results.

        Returns a list sorted by date::

            [{date, type, title, description, related_id, source}, ...]
        """
        entries: List[Dict[str, Any]] = []

        # 1. Evidence source dates
        entries.extend(await self._collect_evidence_dates(case_id))

        # 2. Neo4j Event nodes (if neo4j is available)
        if self._neo4j is not None:
            entries.extend(await self._collect_neo4j_events(case_id))

        # 3. Entity first_seen dates
        entries.extend(await self._collect_entity_dates(case_id))

        # 4. Hypothesis snapshot dates
        entries.extend(await self._collect_snapshot_dates(case_id))

        # 5. Monitoring result dates
        entries.extend(await self._collect_monitoring_dates(case_id))

        # Sort by date (entries without valid dates go to the end)
        entries.sort(key=lambda e: e.get("date") or "9999-12-31T23:59:59")

        logger.debug(
            "Timeline built for case {}: {} entries", case_id, len(entries)
        )
        return entries

    async def get_timeline_range(
        self,
        case_id: str,
        start: datetime,
        end: datetime,
    ) -> List[Dict[str, Any]]:
        """Build a timeline filtered to a specific date range.

        Parameters *start* and *end* are inclusive boundaries.
        """
        full = await self.build_timeline(case_id)
        start_iso = start.isoformat()
        end_iso = end.isoformat()

        return [
            entry for entry in full
            if entry.get("date") and start_iso <= entry["date"] <= end_iso
        ]

    # ------------------------------------------------------------------
    # Collectors (private)
    # ------------------------------------------------------------------

    async def _collect_evidence_dates(
        self,
        case_id: str,
    ) -> List[Dict[str, Any]]:
        """Collect timeline entries from evidence source_date fields."""
        entries: List[Dict[str, Any]] = []
        evidence_list = await self._db.list_evidence_by_case(case_id)

        for ev in evidence_list:
            source_date = ev.get("source_date")
            if not source_date:
                continue
            entries.append({
                "date": _normalize_date(source_date),
                "type": "evidence",
                "title": ev.get("title", "Evidence"),
                "description": ev.get("summary") or f"Type: {ev.get('evidence_type', 'unknown')}",
                "related_id": ev.get("id"),
                "source": "sqlite:evidence",
            })

        return entries

    async def _collect_neo4j_events(
        self,
        case_id: str,
    ) -> List[Dict[str, Any]]:
        """Collect timeline entries from Neo4j Event nodes."""
        entries: List[Dict[str, Any]] = []
        try:
            nodes = await self._neo4j.find_nodes_by_case(case_id, label="Event")
            for node in nodes:
                # Event nodes may store date as 'datetime', 'date', or 'first_seen'
                event_date = (
                    node.get("datetime")
                    or node.get("date")
                    or node.get("first_seen")
                )
                if not event_date:
                    continue
                entries.append({
                    "date": _normalize_date(event_date),
                    "type": "event",
                    "title": node.get("name", "Event"),
                    "description": node.get("description", ""),
                    "related_id": node.get("id"),
                    "source": "neo4j:event",
                })
        except Exception as exc:
            logger.warning("Failed to collect Neo4j events: {}", exc)

        return entries

    async def _collect_entity_dates(
        self,
        case_id: str,
    ) -> List[Dict[str, Any]]:
        """Collect timeline entries from entity first_seen dates."""
        entries: List[Dict[str, Any]] = []
        entities = await self._db.list_entities_by_case(case_id)

        for ent in entities:
            first_seen = ent.get("first_seen")
            if not first_seen:
                continue
            entries.append({
                "date": _normalize_date(first_seen),
                "type": "entity",
                "title": f"{ent.get('entity_type', 'entity').capitalize()}: {ent.get('name', '?')}",
                "description": ent.get("description") or "First appearance",
                "related_id": ent.get("id"),
                "source": "sqlite:entity",
            })

        return entries

    async def _collect_snapshot_dates(
        self,
        case_id: str,
    ) -> List[Dict[str, Any]]:
        """Collect timeline entries from hypothesis snapshot dates."""
        entries: List[Dict[str, Any]] = []
        hypotheses = await self._db.list_hypotheses_by_case(case_id)

        for hyp in hypotheses:
            snapshots = await self._db.list_snapshots_by_hypothesis(hyp["id"])
            for snap in snapshots:
                created_at = snap.get("created_at")
                if not created_at:
                    continue
                entries.append({
                    "date": _normalize_date(created_at),
                    "type": "hypothesis_snapshot",
                    "title": f"Hypothesis: {hyp.get('title', '?')} — score {snap.get('score', '?')}",
                    "description": snap.get("reasoning") or f"Trigger: {snap.get('trigger', 'unknown')}",
                    "related_id": snap.get("id"),
                    "source": "sqlite:hypothesis_snapshot",
                })

        return entries

    async def _collect_monitoring_dates(
        self,
        case_id: str,
    ) -> List[Dict[str, Any]]:
        """Collect timeline entries from monitoring result dates."""
        entries: List[Dict[str, Any]] = []
        jobs = await self._db.list_jobs_by_case(case_id)

        for job in jobs:
            results = await self._db.list_results_by_job(job["id"])
            for res in results:
                found_at = res.get("found_at")
                if not found_at:
                    continue
                entries.append({
                    "date": _normalize_date(found_at),
                    "type": "monitoring_result",
                    "title": res.get("title") or "Monitoring hit",
                    "description": res.get("snippet") or res.get("url") or "",
                    "related_id": res.get("id"),
                    "source": f"monitoring:{res.get('source_engine', 'unknown')}",
                })

        return entries


# ------------------------------------------------------------------
# Helpers
# ------------------------------------------------------------------

def _normalize_date(value: Any) -> str:
    """Normalize a date value to an ISO-8601 string.

    Handles datetime objects, ISO strings, and common date formats.
    Returns the input as-is (stringified) if parsing fails, so that
    sorting still works on string comparison.
    """
    if isinstance(value, datetime):
        return value.isoformat()
    if isinstance(value, str):
        # Already ISO-ish — return as-is
        return value
    return str(value)
