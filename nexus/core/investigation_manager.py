"""
NEXUS -- Investigation Manager.

Manages one AutonomousInvestigator per active case.
Started/stopped via the FastAPI lifespan.
"""

from __future__ import annotations

import asyncio
from typing import Any

from loguru import logger

from nexus.config import settings
from nexus.core.autonomous_loop import AutonomousInvestigator
from nexus.db.sqlite_db import Database, get_db
from nexus.llm.router import LLMRouter


class InvestigationManager:
    """Manages autonomous investigation loops for all active cases.

    Usage::

        manager = InvestigationManager(router, chroma, neo4j)
        await manager.start()       # Start investigators for all active cases
        await manager.stop_all()    # Graceful shutdown
    """

    def __init__(
        self,
        router: LLMRouter,
        chroma: Any,
        neo4j: Any,
        entity_extractor: Any = None,
    ) -> None:
        self._router = router
        self._chroma = chroma
        self._neo4j = neo4j
        self._entity_extractor = entity_extractor
        self._investigators: dict[str, AutonomousInvestigator] = {}
        self._tasks: dict[str, asyncio.Task] = {}

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    async def start(self) -> None:
        """Start investigators for all active cases in the database."""
        async with get_db() as conn:
            db = Database(conn)
            cases = await db.list_cases(status="active")

        for case in cases:
            await self.start_investigation(case["id"])

        logger.info(
            "InvestigationManager started: {} active investigations",
            len(self._investigators),
        )

    async def start_investigation(self, case_id: str) -> bool:
        """Start autonomous investigation for a specific case.

        Returns True if a new investigation was started, False if already running.
        """
        if case_id in self._investigators and self._investigators[case_id].is_running:
            logger.debug(
                "Investigation already running for case {}", case_id
            )
            return False

        # Clean up any stale references
        if case_id in self._tasks:
            task = self._tasks[case_id]
            if not task.done():
                task.cancel()
                try:
                    await task
                except (asyncio.CancelledError, Exception):
                    pass

        investigator = AutonomousInvestigator(
            case_id=case_id,
            router=self._router,
            chroma=self._chroma,
            neo4j=self._neo4j,
            entity_extractor=self._entity_extractor,
        )

        self._investigators[case_id] = investigator
        self._tasks[case_id] = asyncio.create_task(
            investigator.run(),
            name=f"investigation-{case_id[:8]}",
        )

        logger.info("Started autonomous investigation for case {}", case_id)
        return True

    async def stop_investigation(self, case_id: str) -> bool:
        """Stop investigation for a specific case.

        Returns True if an investigation was stopped, False if none was running.
        """
        if case_id not in self._investigators:
            return False

        investigator = self._investigators[case_id]
        await investigator.stop()

        if case_id in self._tasks:
            task = self._tasks[case_id]
            if not task.done():
                task.cancel()
                try:
                    await task
                except (asyncio.CancelledError, Exception):
                    pass
            del self._tasks[case_id]

        del self._investigators[case_id]
        logger.info("Stopped autonomous investigation for case {}", case_id)
        return True

    async def stop_all(self) -> None:
        """Stop all investigations gracefully."""
        case_ids = list(self._investigators.keys())
        for case_id in case_ids:
            await self.stop_investigation(case_id)
        logger.info("InvestigationManager stopped all investigations")

    # ------------------------------------------------------------------
    # Status
    # ------------------------------------------------------------------

    def get_status(self) -> dict[str, Any]:
        """Return status of all running investigations."""
        return {
            "active_count": len(self._investigators),
            "investigations": {
                case_id: inv.get_status()
                for case_id, inv in self._investigators.items()
            },
        }

    def get_investigation_status(self, case_id: str) -> dict[str, Any] | None:
        """Return detailed status for a specific case investigation."""
        inv = self._investigators.get(case_id)
        if inv is None:
            return None
        return inv.get_status()
