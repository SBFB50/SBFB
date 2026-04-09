"""
NEXUS GOV -- Government Event Types and Manager.

Defines government-specific event types and a GovManager that
orchestrates government workers independently from the cold case
InvestigationManager.
"""

from __future__ import annotations

import asyncio
from enum import Enum
from typing import Any, Optional

from loguru import logger

from nexus.events.bus import EventBus
from nexus.events.types import NexusEvent


class GovEventType(str, Enum):
    """Government-specific event types."""

    # Data ingestion
    GOV_POLITICIAN_ADDED = "gov_politician_added"
    GOV_POSITION_ADDED = "gov_position_added"
    GOV_AFFAIR_ADDED = "gov_affair_added"
    GOV_PRESS_ADDED = "gov_press_added"
    GOV_SOCIAL_POST_ADDED = "gov_social_post_added"
    GOV_DECLARATION_ADDED = "gov_declaration_added"
    GOV_LAW_ADDED = "gov_law_added"
    GOV_FACTCHECK_ADDED = "gov_factcheck_added"

    # Media processing
    GOV_VIDEO_DOWNLOADED = "gov_video_downloaded"
    GOV_TRANSCRIPTION_READY = "gov_transcription_ready"
    GOV_IMAGE_ADDED = "gov_image_added"

    # Analysis results
    GOV_CONTRADICTION_FOUND = "gov_contradiction_found"
    GOV_PATTERN_DETECTED = "gov_pattern_detected"
    GOV_SENTIMENT_ANALYZED = "gov_sentiment_analyzed"

    # Alerts
    GOV_ALERT_CREATED = "gov_alert_created"

    # Periodic ticks (for scheduled sync workers)
    TICK_HOURLY = "gov_tick_hourly"
    TICK_DAILY = "gov_tick_daily"
    TICK_WEEKLY = "gov_tick_weekly"
    TICK_MONTHLY = "gov_tick_monthly"


class GovDatabaseProxy:
    """Proxy that opens a fresh GovernmentDatabase connection per method call.

    Long-lived workers use this instead of holding a single connection open.
    Supports all read/write methods from ``GovernmentDatabase``.
    """

    def __getattr__(self, name: str) -> Any:
        from nexus.db.sqlite_db import get_db
        from nexus.gov.db import GovernmentDatabase

        async def _method_proxy(*args: Any, **kwargs: Any) -> Any:
            async with get_db() as conn:
                db = GovernmentDatabase(conn)
                method = getattr(db, name)
                return await method(*args, **kwargs)

        return _method_proxy


class GovManager:
    """Orchestrates government monitoring workers.

    Independent from the cold case ReactiveInvestigationManager.
    Manages its own EventBus, workers, and periodic timers.
    """

    def __init__(
        self,
        router: Any = None,
        neo4j: Any = None,
        chroma: Any = None,
    ) -> None:
        self._router = router
        self._neo4j = neo4j
        self._chroma = chroma
        self._bus: Optional[EventBus] = None
        self._workers: list = []
        self._timer_task: Optional[asyncio.Task] = None
        self._running = False

    @property
    def bus(self) -> Optional[EventBus]:
        return self._bus

    @property
    def running(self) -> bool:
        return self._running

    async def start(self) -> None:
        """Start the government monitoring pipeline."""
        if self._running:
            logger.warning("GovManager already running")
            return

        logger.info("GovManager starting...")

        # Create dedicated EventBus for government events
        self._bus = EventBus(db_path=None)  # Uses default sqlite_path
        await self._bus.start()

        # Create DB proxy for long-lived workers (opens fresh connection per call)
        db_proxy = GovDatabaseProxy()

        # Import and create all 25 workers.
        # Each import is wrapped in try/except so a single broken worker
        # cannot crash the whole GovManager startup.
        _worker_specs: list[tuple[str, str, list]] = [
            # (module_path, class_name, extra_args)
            # --- Data sync workers (tick-based) ---
            ("nexus.gov.workers.vote_sync", "GovVoteSyncWorker", []),
            ("nexus.gov.workers.depute_sync", "GovDeputeSyncWorker", []),
            ("nexus.gov.workers.senat_sync", "GovSenatSyncWorker", []),
            ("nexus.gov.workers.hatvp_sync", "GovHATVPSyncWorker", []),
            ("nexus.gov.workers.law_sync", "GovLawSyncWorker", []),
            ("nexus.gov.workers.fabrique_sync", "GovFabriqueSyncWorker", []),
            ("nexus.gov.workers.wikidata_sync", "GovWikidataSyncWorker", []),
            ("nexus.gov.workers.affairs_sync", "GovAffairsSyncWorker", []),
            ("nexus.gov.workers.press_sync", "GovPressSyncWorker", []),
            ("nexus.gov.workers.factcheck_sync", "GovFactcheckSyncWorker", []),
            # --- EU sync workers (weekly tick) ---
            ("nexus.gov.workers.eu_parliament_sync", "GovEUParliamentSyncWorker", []),
            ("nexus.gov.workers.eurlex_sync", "GovEURlexSyncWorker", []),
            # --- Social media sync workers (tick-based) ---
            ("nexus.gov.workers.twitter_sync", "GovTwitterSyncWorker", []),
            ("nexus.gov.workers.facebook_sync", "GovFacebookSyncWorker", []),
            ("nexus.gov.workers.instagram_sync", "GovInstagramSyncWorker", []),
            ("nexus.gov.workers.youtube_sync", "GovYouTubeSyncWorker", []),
            # --- Transcription worker (event-based: GOV_VIDEO_DOWNLOADED) ---
            ("nexus.gov.workers.transcription", "GovTranscriptionWorker", []),
            # --- Analysis workers (event-based: need LLM router) ---
            ("nexus.gov.workers.contradiction_analyzer", "GovContradictionAnalyzer", [self._router]),
            ("nexus.gov.workers.sentiment", "GovSentimentAnalyzer", [self._router]),
            # --- Voting pattern analyzer (weekly tick, pure stats) ---
            ("nexus.gov.workers.voting_pattern", "GovVotingPatternAnalyzer", []),
            # --- Neo4j graph sync (event-based) ---
            ("nexus.gov.workers.neo4j_sync", "GovNeo4jSyncWorker", [self._neo4j]),
            # --- Alert worker (event-based: contradictions, affairs, patterns) ---
            ("nexus.gov.workers.alert", "GovAlertWorker", []),
            # --- Embedding worker (event-based: vectorizes text for RAG) ---
            ("nexus.gov.workers.embedding", "GovEmbedWorker", [self._chroma, self._router]),
            # --- Biography generator (weekly tick, LLM-based) ---
            ("nexus.gov.workers.biography", "GovBiographyWorker", [self._router]),
            # --- Weekly recap + thematic classification (weekly tick, LLM-based) ---
            ("nexus.gov.workers.weekly_recap", "GovWeeklyRecapWorker", [self._router]),
        ]

        import importlib

        for mod_path, cls_name, extra_args in _worker_specs:
            try:
                mod = importlib.import_module(mod_path)
                cls = getattr(mod, cls_name)
                worker = cls(self._bus, db_proxy, *extra_args)
                self._workers.append(worker)
            except Exception as exc:
                logger.error(
                    "Failed to load gov worker {}.{}: {}",
                    mod_path, cls_name, exc,
                )

        # Register and start all workers
        for worker in self._workers:
            worker.register()
            worker.start()
            logger.info("Gov worker started: {}", worker.name)

        # Start periodic timer
        self._timer_task = asyncio.create_task(self._periodic_timer())

        self._running = True
        logger.info("GovManager started — {} workers active", len(self._workers))

        # Trigger initial daily sync after a short delay to let workers settle
        asyncio.create_task(self._initial_tick())

    async def stop(self) -> None:
        """Stop all government workers and timers."""
        if not self._running:
            return

        logger.info("GovManager stopping...")
        self._running = False

        # Cancel timer
        if self._timer_task and not self._timer_task.done():
            self._timer_task.cancel()
            try:
                await self._timer_task
            except asyncio.CancelledError:
                pass

        # Stop workers
        for worker in self._workers:
            try:
                await worker.stop()
            except Exception as exc:
                logger.warning("Error stopping worker {}: {}", getattr(worker, 'name', '?'), exc)

        self._workers.clear()

        # Stop the EventBus (drains subscriber queues)
        if self._bus:
            try:
                await self._bus.stop()
            except Exception as exc:
                logger.warning("Error stopping GovManager EventBus: {}", exc)
            self._bus = None

        logger.info("GovManager stopped")

    async def _periodic_timer(self) -> None:
        """Emit periodic tick events for scheduled sync workers."""
        hourly_counter = 0

        while self._running:
            try:
                await asyncio.sleep(3600)  # 1 hour
                hourly_counter += 1

                if not self._bus or not self._running:
                    break

                # Hourly tick
                await self._bus.publish(NexusEvent(
                    event_type=GovEventType.TICK_HOURLY,
                    case_id="gov",
                    payload={"counter": hourly_counter},
                    source_worker="gov_timer",
                ))

                # Daily tick (every 24 hours)
                if hourly_counter % 24 == 0:
                    await self._bus.publish(NexusEvent(
                        event_type=GovEventType.TICK_DAILY,
                        case_id="gov",
                        payload={"counter": hourly_counter // 24},
                        source_worker="gov_timer",
                    ))

                # Weekly tick (every 168 hours)
                if hourly_counter % 168 == 0:
                    await self._bus.publish(NexusEvent(
                        event_type=GovEventType.TICK_WEEKLY,
                        case_id="gov",
                        payload={"counter": hourly_counter // 168},
                        source_worker="gov_timer",
                    ))

                # Monthly tick (every 720 hours ~ 30 days)
                if hourly_counter % 720 == 0:
                    await self._bus.publish(NexusEvent(
                        event_type=GovEventType.TICK_MONTHLY,
                        case_id="gov",
                        payload={"counter": hourly_counter // 720},
                        source_worker="gov_timer",
                    ))

            except asyncio.CancelledError:
                break
            except Exception as exc:
                logger.error("Gov timer error: {}", exc)
                await asyncio.sleep(60)  # Retry after 1 min

    async def _initial_tick(self) -> None:
        """Emit a TICK_DAILY shortly after startup to trigger the first sync."""
        try:
            await asyncio.sleep(5)  # Let workers settle
            if self._bus and self._running:
                await self._bus.publish(NexusEvent(
                    event_type=GovEventType.TICK_DAILY,
                    case_id="gov",
                    payload={"trigger": "startup"},
                    source_worker="gov_manager",
                ))
                logger.info("GovManager emitted initial TICK_DAILY")
        except Exception as exc:
            logger.warning("GovManager initial tick failed: {}", exc)

    def get_status(self) -> dict:
        """Return status of the government monitoring pipeline."""
        return {
            "running": self._running,
            "workers": len(self._workers),
            "worker_status": [
                w.get_status() if hasattr(w, "get_status") else {
                    "name": getattr(w, "name", "unknown"),
                    "status": getattr(w, "status", "unknown"),
                }
                for w in self._workers
            ],
            "bus_active": self._bus is not None,
            "bus_stats": self._bus.get_stats() if self._bus else None,
        }
