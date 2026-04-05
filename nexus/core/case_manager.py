"""
NEXUS -- Case Manager.

High-level operations on investigation cases: create, read, update,
delete (with cascade) and aggregate statistics.

Usage::

    async with get_db() as conn:
        db = Database(conn)
        mgr = CaseManager(db)
        case = await mgr.create_case(CaseCreate(name="Doe"))
"""

from __future__ import annotations

from loguru import logger

from nexus.db.models import (
    Case,
    CaseCreate,
    CaseUpdate,
)
from nexus.db.sqlite_db import Database


class CaseManager:
    """Manage investigation cases (CRUD + stats)."""

    def __init__(self, db: Database) -> None:
        self._db = db

    # ------------------------------------------------------------------
    # CRUD
    # ------------------------------------------------------------------

    async def create_case(self, data: CaseCreate) -> Case:
        """Create a new investigation case."""
        logger.info("Creating case: {}", data.name)
        row = await self._db.create_case(
            name=data.name,
            reference=data.reference,
            description=data.description,
            status=data.status,
        )
        case = Case(**row)
        logger.info("Case created: {} ({})", case.name, case.id)
        return case

    async def get_case(self, case_id: str) -> Case:
        """Retrieve a case by ID.

        Raises:
            ValueError: if the case does not exist.
        """
        row = await self._db.get_case(case_id)
        if row is None:
            raise ValueError(f"Case not found: {case_id}")
        return Case(**row)

    async def list_cases(self, status: str | None = None) -> list[Case]:
        """List all cases, optionally filtered by status."""
        rows = await self._db.list_cases(status=status)
        return [Case(**r) for r in rows]

    async def update_case(self, case_id: str, data: CaseUpdate) -> Case:
        """Update an existing case with the supplied fields.

        Only non-None fields in *data* are applied.

        Raises:
            ValueError: if the case does not exist.
        """
        # Build the kwargs dict with only the fields that were explicitly set
        update_fields = data.model_dump(exclude_unset=True)
        if not update_fields:
            return await self.get_case(case_id)

        logger.info("Updating case {} with fields: {}", case_id, list(update_fields.keys()))
        row = await self._db.update_case(case_id, **update_fields)
        if row is None:
            raise ValueError(f"Case not found: {case_id}")
        return Case(**row)

    async def delete_case(self, case_id: str) -> None:
        """Delete a case and ALL dependent data (cascade).

        Removes evidence, entities, entity_mentions, hypotheses,
        hypothesis_snapshots, analysis_runs, monitoring_jobs,
        monitoring_results and alerts linked to this case.

        Raises:
            ValueError: if the case does not exist.
        """
        logger.warning("Deleting case {} with full cascade", case_id)
        deleted = await self._db.delete_case(case_id)
        if not deleted:
            raise ValueError(f"Case not found: {case_id}")
        logger.info("Case {} deleted successfully", case_id)

    # ------------------------------------------------------------------
    # Statistics
    # ------------------------------------------------------------------

    async def get_case_stats(self, case_id: str) -> dict:
        """Return aggregate counts for a case.

        Returns a dict with keys:
            - evidence
            - entities
            - hypotheses
            - alerts (unread)
            - monitoring_jobs (active)

        Raises:
            ValueError: if the case does not exist.
        """
        # Make sure the case exists first
        row = await self._db.get_case(case_id)
        if row is None:
            raise ValueError(f"Case not found: {case_id}")

        evidence = await self._db.list_evidence_by_case(case_id)
        entities = await self._db.list_entities_by_case(case_id)
        hypotheses = await self._db.list_hypotheses_by_case(case_id)
        alerts_unread = await self._db.count_unread_alerts(case_id)
        monitoring_jobs = await self._db.list_jobs_by_case(case_id, active_only=True)

        stats = {
            "evidence": len(evidence),
            "entities": len(entities),
            "hypotheses": len(hypotheses),
            "alerts": alerts_unread,
            "monitoring_jobs": len(monitoring_jobs),
        }
        logger.debug("Stats for case {}: {}", case_id, stats)
        return stats
