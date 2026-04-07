"""
NEXUS -- ReactiveInvestigationManager.

Replaces the old InvestigationManager + AutonomousInvestigator.
For each active case:
  - Creates an EventBus
  - Instantiates all 17 workers
  - Starts MonitoringLoop + PeriodicTimer
  - Runs everything as asyncio tasks

The manager provides the SAME interface as the old InvestigationManager
so the API endpoints continue to work unchanged.
"""

from __future__ import annotations

import asyncio
from datetime import datetime, timezone
from typing import Any

from loguru import logger

from nexus.config import settings
from nexus.db.sqlite_db import Database, get_db
from nexus.events.bus import EventBus
from nexus.events.db_proxy import DatabaseProxy
from nexus.events.monitoring_loop import MonitoringLoop
from nexus.events.timer import PeriodicTimer
from nexus.events.worker import ReactiveWorker
from nexus.llm.router import LLMRouter


# ---------------------------------------------------------------------------
# Per-case context: everything needed to run one investigation
# ---------------------------------------------------------------------------

class _CaseContext:
    """All resources for a single case investigation."""

    __slots__ = (
        "case_id",
        "bus",
        "monitoring_loop",
        "timer",
        "workers",
        "tasks",
        "started_at",
    )

    def __init__(self, case_id: str) -> None:
        self.case_id = case_id
        self.bus: EventBus | None = None
        self.monitoring_loop: MonitoringLoop | None = None
        self.timer: PeriodicTimer | None = None
        self.workers: list[ReactiveWorker] = []
        self.tasks: list[asyncio.Task] = []
        self.started_at: str | None = None


# ---------------------------------------------------------------------------
# ReactiveInvestigationManager
# ---------------------------------------------------------------------------

class ReactiveInvestigationManager:
    """Manages reactive investigation pipelines for all active cases.

    Drop-in replacement for the old ``InvestigationManager``.  The API
    endpoints access ``app.state.investigation_manager`` and call:
      - ``start_investigation(case_id)``
      - ``stop_investigation(case_id)``
      - ``get_status()``
      - ``get_investigation_status(case_id)``

    All four methods are preserved with compatible return types.

    Usage::

        manager = ReactiveInvestigationManager(
            router=router,
            chroma=chroma,
            neo4j=neo4j,
            entity_extractor=entity_extractor,
        )
        await manager.start()       # Start investigations for all active cases
        await manager.stop_all()    # Graceful shutdown
    """

    def __init__(
        self,
        router: LLMRouter,
        chroma: Any = None,
        neo4j: Any = None,
        entity_extractor: Any = None,
    ) -> None:
        self._router = router
        self._chroma = chroma
        self._neo4j = neo4j
        self._entity_extractor = entity_extractor
        self._cases: dict[str, _CaseContext] = {}

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    async def start(self) -> None:
        """Start investigations for all active cases in the database."""
        async with get_db() as conn:
            db = Database(conn)
            cases = await db.list_cases(status="active")

        for case in cases:
            await self.start_investigation(case["id"])

        logger.info(
            "ReactiveInvestigationManager started: {} active investigations",
            len(self._cases),
        )

    async def start_investigation(self, case_id: str) -> bool:
        """Start the reactive pipeline for a specific case.

        Returns True if a new investigation was started, False if already running.
        """
        if case_id in self._cases:
            ctx = self._cases[case_id]
            # Check if workers are still alive
            alive = any(not t.done() for t in ctx.tasks)
            if alive:
                logger.debug(
                    "Investigation already running for case {}", case_id
                )
                return False
            # Dead tasks -- clean up and restart
            await self._teardown_case(case_id)

        ctx = _CaseContext(case_id)
        ctx.started_at = datetime.now(timezone.utc).isoformat()

        # 1. Create EventBus for this case
        ctx.bus = EventBus()
        await ctx.bus.start()

        # 2. Create a DatabaseProxy for long-lived workers
        db_proxy = DatabaseProxy()

        # 3. Instantiate all workers
        ctx.workers = self._create_workers(ctx.bus, case_id, db_proxy)

        # 4. Register workers on the bus and start them
        for worker in ctx.workers:
            worker.register()
            task = worker.start()
            ctx.tasks.append(task)

        # 5. Create and start MonitoringLoop
        ctx.monitoring_loop = MonitoringLoop(
            bus=ctx.bus,
            router=self._router,
            chroma=self._chroma,
            case_id=case_id,
            sweep_interval=30.0,
            rate_limit=settings.auto_recon_rate_limit,
        )
        await ctx.monitoring_loop.start()

        # 6. Create and start PeriodicTimer
        report_interval = settings.auto_report_every_n_cycles * settings.investigation_cycle_minutes * 60
        backup_interval = settings.auto_backup_every_n_cycles * settings.investigation_cycle_minutes * 60
        summary_interval = 3 * settings.investigation_cycle_minutes * 60  # every 3 "cycles" worth

        ctx.timer = PeriodicTimer(
            bus=ctx.bus,
            case_id=case_id,
            report_interval=max(report_interval, 300),    # min 5 minutes
            backup_interval=max(backup_interval, 600),    # min 10 minutes
            summary_interval=max(summary_interval, 300),  # min 5 minutes
        )
        await ctx.timer.start()

        self._cases[case_id] = ctx
        logger.info(
            "Started reactive investigation for case {} ({} workers)",
            case_id,
            len(ctx.workers),
        )
        return True

    async def stop_investigation(self, case_id: str) -> bool:
        """Stop the reactive pipeline for a specific case.

        Returns True if an investigation was stopped, False if none was running.
        """
        if case_id not in self._cases:
            return False

        await self._teardown_case(case_id)
        logger.info("Stopped reactive investigation for case {}", case_id)
        return True

    async def stop_all(self) -> None:
        """Stop all investigations gracefully."""
        case_ids = list(self._cases.keys())
        for case_id in case_ids:
            await self._teardown_case(case_id)
        logger.info(
            "ReactiveInvestigationManager stopped all investigations"
        )

    # ------------------------------------------------------------------
    # Status (backward-compatible with old InvestigationManager)
    # ------------------------------------------------------------------

    def get_status(self) -> dict[str, Any]:
        """Return status of all running investigations.

        Same shape as old InvestigationManager.get_status().
        """
        return {
            "active_count": len(self._cases),
            "investigations": {
                case_id: self._case_status(ctx)
                for case_id, ctx in self._cases.items()
            },
        }

    def get_investigation_status(self, case_id: str) -> dict[str, Any] | None:
        """Return detailed status for a specific case.

        Same shape as old AutonomousInvestigator.get_status().
        """
        ctx = self._cases.get(case_id)
        if ctx is None:
            return None
        return self._case_status(ctx)

    # ------------------------------------------------------------------
    # Internal: worker factory
    # ------------------------------------------------------------------

    def _create_workers(
        self,
        bus: EventBus,
        case_id: str,
        db_proxy: DatabaseProxy,
    ) -> list[ReactiveWorker]:
        """Instantiate all 17 workers for one case.

        The order matters only for documentation. All workers run
        concurrently and communicate via events.
        """
        # Lazy imports to avoid circular dependencies
        from nexus.events.workers.neo4j_sync import Neo4jSyncWorker
        from nexus.events.workers.chunker_embed import ChunkerEmbedWorker
        from nexus.events.workers.geo_mapper import GeoMapperWorker
        from nexus.events.workers.osint_recon import OSINTReconWorker
        from nexus.events.workers.summarizer import SummarizerWorker
        from nexus.events.workers.entity_extractor import EntityExtractorWorker
        from nexus.events.workers.forensics import ForensicRouterWorker
        from nexus.events.workers.contradiction import ContradictionWorker
        from nexus.events.workers.analysis import AnalysisPipelineWorker
        from nexus.events.workers.hypothesis import HypothesisWorker
        from nexus.events.workers.suspect_scorer import SuspectScorerWorker
        from nexus.events.workers.query_generator import QueryGeneratorWorker
        from nexus.events.workers.evidence_ingest import EvidenceIngestWorker
        from nexus.events.workers.self_questioning import SelfQuestioningWorker
        from nexus.events.workers.alert import AlertWorker
        from nexus.events.workers.summary_tree import SummaryTreeWorker
        from nexus.events.workers.timeline import TimelineWorker
        from nexus.events.workers.memory import MemoryWorker

        workers: list[ReactiveWorker] = [
            # 1. Neo4j sync
            Neo4jSyncWorker(bus, db_proxy, self._neo4j),
            # 2. Chunker + embedder
            ChunkerEmbedWorker(bus, db_proxy, self._chroma, self._router),
            # 3. Geo mapper
            GeoMapperWorker(bus, db_proxy),
            # 4. OSINT recon
            OSINTReconWorker(bus),
            # 5. Summarizer (bridge: EVIDENCE_ADDED -> EVIDENCE_PROCESSED)
            SummarizerWorker(bus, db_proxy),
            # 6. Entity extractor (emits ENTITY_DISCOVERED per entity)
            EntityExtractorWorker(bus, db_proxy),
            # 7. Forensic router
            ForensicRouterWorker(bus, self._router),
            # 8. Contradiction detector
            ContradictionWorker(bus, db_proxy, self._router),
            # 9. Analysis pipeline (with debounce)
            AnalysisPipelineWorker(bus, db_proxy, self._router, self._chroma, self._neo4j),
            # 10. Hypothesis engine
            HypothesisWorker(bus, db_proxy, self._router, self._chroma, self._neo4j),
            # 11. Suspect scorer
            SuspectScorerWorker(bus, db_proxy, self._router, self._neo4j),
            # 12. Query generator
            QueryGeneratorWorker(bus, db_proxy, self._router),
            # 13. Evidence ingest (from monitoring results)
            EvidenceIngestWorker(bus, self._build_evidence_processor(db_proxy)),
            # 14. Self-questioning
            SelfQuestioningWorker(bus, db_proxy, self._router),
            # 15. Alert worker
            AlertWorker(bus, db_proxy),
            # 16. Summary tree (RAPTOR)
            SummaryTreeWorker(bus, db_proxy, self._router, self._chroma),
            # 17. Timeline builder
            TimelineWorker(bus, db_proxy, self._neo4j),
            # 18. Memory worker (investigation insights)
            MemoryWorker(bus, db_proxy, self._router, self._chroma),
        ]

        return workers

    def _build_evidence_processor(self, db_proxy: DatabaseProxy) -> Any:
        """Create a lazy evidence processor wrapper for EvidenceIngestWorker.

        The EvidenceIngestWorker expects an object with
        ``process_text_input(case_id, title, text, source)``.
        We wrap EvidenceProcessor to open fresh DB connections.
        """
        router = self._router
        chroma = self._chroma
        neo4j = self._neo4j
        entity_extractor = self._entity_extractor

        class _EvidenceProcessorProxy:
            """Proxy that creates EvidenceProcessor with fresh DB per call."""

            async def process_text_input(
                self,
                case_id: str,
                title: str,
                text: str,
                source: str = "",
            ) -> Any:
                from nexus.core.evidence_processor import EvidenceProcessor

                async with get_db() as conn:
                    db = Database(conn)
                    processor = EvidenceProcessor(
                        db=db,
                        router=router,
                        upload_dir=settings.upload_dir,
                        neo4j=neo4j,
                        chroma=chroma,
                        entity_extractor=entity_extractor,
                    )
                    return await processor.process_text_input(
                        case_id=case_id,
                        title=title,
                        text=text,
                        source=source,
                    )

        return _EvidenceProcessorProxy()

    # ------------------------------------------------------------------
    # Internal: teardown
    # ------------------------------------------------------------------

    async def _teardown_case(self, case_id: str) -> None:
        """Tear down all resources for a case."""
        ctx = self._cases.pop(case_id, None)
        if ctx is None:
            return

        # Stop timer
        if ctx.timer:
            await ctx.timer.stop()

        # Stop monitoring loop
        if ctx.monitoring_loop:
            await ctx.monitoring_loop.stop()

        # Stop all workers
        for worker in ctx.workers:
            try:
                await worker.stop()
            except Exception as exc:
                logger.warning(
                    "Error stopping worker {}: {}", worker.name, exc
                )

        # Cancel any lingering tasks
        for task in ctx.tasks:
            if not task.done():
                task.cancel()
        for task in ctx.tasks:
            try:
                await task
            except (asyncio.CancelledError, Exception):
                pass

        # Stop event bus
        if ctx.bus:
            await ctx.bus.stop()

    # ------------------------------------------------------------------
    # Internal: status formatting
    # ------------------------------------------------------------------

    def _case_status(self, ctx: _CaseContext) -> dict[str, Any]:
        """Build status dict compatible with old AutonomousInvestigator.get_status()."""
        # Count alive workers
        alive_workers = sum(1 for t in ctx.tasks if not t.done())

        # Build per-worker tool status (same shape as old _tool_status dict)
        tools: dict[str, dict[str, Any]] = {}
        for worker in ctx.workers:
            ws = worker.get_status()
            tools[ws["name"]] = {
                "status": ws["status"],
                "detail": f"processed={ws['events_processed']} errors={ws['events_errored']}",
                "updated_at": ws.get("last_event_at"),
                "queue_size": ws.get("queue_size", 0),
            }

        # Event bus stats
        bus_stats = ctx.bus.get_stats() if ctx.bus else {}

        # Monitoring loop stats
        monitoring_stats = (
            ctx.monitoring_loop.get_stats() if ctx.monitoring_loop else {}
        )

        return {
            "case_id": ctx.case_id,
            "running": alive_workers > 0,
            "cycle_count": bus_stats.get("events_published", 0),
            "last_action": "reactive",
            "last_cycle_at": None,
            "started_at": ctx.started_at,
            "tools": tools,
            "event_bus": bus_stats,
            "monitoring": monitoring_stats,
            "workers_alive": alive_workers,
            "workers_total": len(ctx.workers),
        }
