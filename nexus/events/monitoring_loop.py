"""
NEXUS -- Continuous monitoring loop (replaces APScheduler).

Sweeps the DB every 30 seconds for monitoring jobs whose next_run
has arrived.  Executes SearXNG / Robin searches, scores relevance,
deduplicates via ChromaDB embeddings, and publishes MONITORING_RESULT
events for each hit.

Reuses the existing SearXNGMonitor and RobinMonitor classes.
"""

from __future__ import annotations

import asyncio
import re
from datetime import datetime, timedelta, timezone
from typing import Any, Optional

from loguru import logger

from nexus.config import settings
from nexus.db.chroma_db import ChromaClient
from nexus.db.sqlite_db import Database, get_db
from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.llm.prompts import RESULT_FILTERING_PROMPT
from nexus.llm.router import LLMRouter, TaskType
from nexus.monitoring.alert_manager import AlertManager
from nexus.monitoring.robin_monitor import RobinMonitor
from nexus.monitoring.searxng_monitor import SearXNGMonitor
from nexus.monitoring.wayback_monitor import WaybackMonitor


# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

_DEFAULT_SWEEP_INTERVAL = 30.0    # seconds between DB sweeps
_DEFAULT_RATE_LIMIT = 2.0         # seconds between searches


class MonitoringLoop:
    """Continuous async loop that replaces APScheduler for monitoring jobs.

    Design:
    - Sweeps every *sweep_interval* seconds.
    - Queries ``monitoring_jobs`` for rows where ``next_run <= now()``
      or ``last_run IS NULL``.
    - Executes the search (SearXNG / Robin).
    - For each result: score relevance, check dedup, publish
      MONITORING_RESULT event.
    - Updates the job's ``last_run`` and ``next_run``.
    - Rate-limits between searches (default 2s).

    Parameters:
        bus:            EventBus to publish MONITORING_RESULT events.
        router:         LLMRouter for relevance scoring / embeddings.
        chroma:         ChromaClient for dedup embeddings.
        case_id:        Scope the loop to a single case.
        sweep_interval: Seconds between DB sweeps (default 30).
        rate_limit:     Seconds between individual searches (default 2).
    """

    def __init__(
        self,
        bus: EventBus,
        router: LLMRouter,
        chroma: Optional[ChromaClient],
        case_id: str,
        *,
        sweep_interval: float = _DEFAULT_SWEEP_INTERVAL,
        rate_limit: float = _DEFAULT_RATE_LIMIT,
    ) -> None:
        self._bus = bus
        self._router = router
        self._chroma = chroma
        self._case_id = case_id
        self._sweep_interval = sweep_interval
        self._rate_limit = rate_limit

        self._searxng = SearXNGMonitor()
        self._robin = RobinMonitor()
        self._wayback = WaybackMonitor()

        self._running = False
        self._task: asyncio.Task | None = None

        # Stats
        self._sweeps = 0
        self._jobs_executed = 0
        self._results_stored = 0

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    async def start(self) -> None:
        """Start the monitoring sweep loop as a background task."""
        if self._running:
            return
        self._running = True
        self._task = asyncio.create_task(
            self._sweep_loop(),
            name=f"monitoring-loop-{self._case_id[:8]}",
        )
        logger.info(
            "MonitoringLoop started for case {} (sweep every {}s)",
            self._case_id[:8],
            self._sweep_interval,
        )

    async def stop(self) -> None:
        """Stop the sweep loop gracefully."""
        self._running = False
        if self._task and not self._task.done():
            self._task.cancel()
            try:
                await self._task
            except (asyncio.CancelledError, Exception):
                pass
        logger.info(
            "MonitoringLoop stopped for case {} (sweeps={}, jobs={}, results={})",
            self._case_id[:8],
            self._sweeps,
            self._jobs_executed,
            self._results_stored,
        )

    def get_stats(self) -> dict[str, Any]:
        """Return runtime statistics."""
        return {
            "running": self._running,
            "case_id": self._case_id,
            "sweeps": self._sweeps,
            "jobs_executed": self._jobs_executed,
            "results_stored": self._results_stored,
        }

    # ------------------------------------------------------------------
    # Main sweep loop
    # ------------------------------------------------------------------

    async def _sweep_loop(self) -> None:
        """Periodically check for due monitoring jobs and execute them."""
        try:
            while self._running:
                await asyncio.sleep(self._sweep_interval)
                if not self._running:
                    break
                try:
                    await self._sweep_once()
                except Exception as exc:
                    logger.error(
                        "MonitoringLoop sweep error for case {}: {}",
                        self._case_id[:8],
                        exc,
                    )
                self._sweeps += 1
        except asyncio.CancelledError:
            pass

    async def _sweep_once(self) -> None:
        """One sweep: find due jobs, execute them sequentially."""
        now_iso = datetime.now(timezone.utc).isoformat()

        async with get_db() as conn:
            db = Database(conn)

            # Find jobs for this case that are due or never run
            cursor = await conn.execute(
                """
                SELECT * FROM monitoring_jobs
                 WHERE case_id = ?
                   AND is_active = 1
                   AND (last_run IS NULL OR next_run <= ?)
                """,
                (self._case_id, now_iso),
            )
            due_jobs = [dict(r) for r in await cursor.fetchall()]

        if not due_jobs:
            return

        logger.debug(
            "MonitoringLoop: {} due jobs for case {}",
            len(due_jobs),
            self._case_id[:8],
        )

        for job in due_jobs:
            if not self._running:
                break
            try:
                await self._execute_job(job)
                self._jobs_executed += 1
            except Exception as exc:
                logger.error(
                    "MonitoringLoop: job {} failed: {}",
                    job["id"][:8],
                    exc,
                )
            # Rate-limit between jobs
            await asyncio.sleep(self._rate_limit)

    # ------------------------------------------------------------------
    # Job execution (ported from MonitoringScheduler)
    # ------------------------------------------------------------------

    async def _execute_job(self, job: dict[str, Any]) -> None:
        """Execute a single monitoring job: search, dedup, score, publish."""
        job_id = job["id"]
        case_id = job["case_id"]
        query = job["query"]
        job_type = job["job_type"]

        logger.info("MonitoringLoop: executing job {} ({})", job_id[:8], query[:60])

        # Check for before-date filter from case description
        before_date = await self._get_before_date(case_id)

        # 1. Search
        raw_results: list[dict] = []

        if job_type in ("searxng", "both"):
            try:
                searxng_results = await self._searxng.search(
                    query=query,
                    max_results=20,
                    before_date=before_date,
                )
                raw_results.extend(searxng_results)
            except Exception:
                logger.exception("SearXNG search failed for job {}", job_id[:8])

        if job_type in ("robin", "both"):
            try:
                if await self._robin.is_available():
                    robin_results = await self._robin.search(
                        query=query,
                        max_results=10,
                    )
                    raw_results.extend(robin_results)
            except Exception:
                logger.exception("Robin search failed for job {}", job_id[:8])

        # Wayback Machine: search archived pages when date filter is active
        if before_date:
            try:
                wayback_results = await self._wayback.search(
                    query=query,
                    max_results=10,
                    before_date=before_date,
                )
                raw_results.extend(wayback_results)
            except Exception:
                logger.exception("Wayback search failed for job {}", job_id[:8])

        # 2. Update timestamps regardless of results
        now = datetime.now(timezone.utc).isoformat()
        interval_hours = job.get("interval_hours", 6)
        next_run = (
            datetime.now(timezone.utc) + timedelta(hours=interval_hours)
        ).isoformat()

        async with get_db() as conn:
            db = Database(conn)
            await db.update_job(job_id, last_run=now, next_run=next_run)

        if not raw_results:
            return

        # 2b. Post-filter by date if before_date is set
        if before_date:
            raw_results = await self._filter_by_date(raw_results, before_date)

        # 3. Process each result: dedup, score, store, publish
        async with get_db() as conn:
            db = Database(conn)
            alert_mgr = AlertManager(db)

            for result in raw_results:
                if not self._running:
                    break
                try:
                    stored = await self._process_result(
                        db, alert_mgr, job_id, case_id, query, result
                    )
                    if stored:
                        self._results_stored += 1
                except Exception as exc:
                    logger.warning(
                        "MonitoringLoop: result processing failed: {}", exc
                    )

        logger.info(
            "MonitoringLoop: job {} done ({} raw results)",
            job_id[:8],
            len(raw_results),
        )

    async def _process_result(
        self,
        db: Database,
        alert_mgr: AlertManager,
        job_id: str,
        case_id: str,
        query: str,
        result: dict[str, Any],
    ) -> bool:
        """Process a single search result: dedup, score, store, publish.

        Returns True if the result was stored (not dropped).
        """
        text_for_embed = f"{result.get('title', '')} {result.get('snippet', '')}"
        if not text_for_embed.strip():
            return False

        # Compute embedding for deduplication
        try:
            embedding = await self._router.embed(text_for_embed)
        except Exception:
            logger.warning("Embedding failed for result: {}", result.get("url"))
            return False

        # Check semantic duplicate in ChromaDB
        is_dup = False
        if self._chroma:
            try:
                is_dup = self._chroma.is_duplicate_result(
                    case_id=case_id,
                    embedding=embedding,
                    threshold=0.92,
                )
            except Exception:
                logger.warning("Duplicate check failed, treating as new")

        # Relevance scoring via LLM
        relevance_score: float | None = None
        try:
            filter_prompt = RESULT_FILTERING_PROMPT.format(
                investigation_context=query,
                title=result.get("title", ""),
                url=result.get("url", ""),
                snippet=result.get("snippet", ""),
            )
            filter_resp = await self._router.route_json(
                TaskType.RESULT_FILTERING,
                filter_prompt,
            )
            raw_score = filter_resp.get("relevance_score")
            if isinstance(raw_score, (int, float)):
                relevance_score = round(float(raw_score) * 100, 1)
        except Exception:
            logger.warning("Relevance filtering failed for {}", result.get("url"))

        # Store result in SQLite
        db_result = await db.create_monitoring_result(
            job_id=job_id,
            case_id=case_id,
            url=result.get("url"),
            title=result.get("title"),
            snippet=result.get("snippet"),
            source_engine=result.get("engine") or result.get("source"),
            relevance_score=relevance_score,
            is_new=not is_dup,
            is_duplicate=is_dup,
        )

        # Store embedding in ChromaDB for future dedup
        if not is_dup and self._chroma:
            try:
                self._chroma.add_monitoring_result(
                    result_id=db_result["id"],
                    case_id=case_id,
                    text=text_for_embed,
                    embedding=embedding,
                    metadata={
                        "job_id": job_id,
                        "url": result.get("url", ""),
                    },
                )
            except Exception:
                logger.warning(
                    "Failed to store embedding for result {}", db_result["id"]
                )

        # Create alert for high-relevance, non-duplicate results
        if (
            not is_dup
            and relevance_score is not None
            and relevance_score >= 60.0
        ):
            try:
                await alert_mgr.create_monitoring_alert(
                    case_id=case_id,
                    result=db_result,
                )
            except Exception:
                logger.warning(
                    "Alert creation failed for result {}", db_result["id"]
                )

        # Publish event so workers can react
        if not is_dup and relevance_score is not None and relevance_score >= settings.auto_ingest_relevance_threshold:
            event = NexusEvent(
                event_type=EventType.MONITORING_RESULT,
                case_id=case_id,
                payload={
                    "result_id": db_result["id"],
                    "url": result.get("url", ""),
                    "title": result.get("title", ""),
                    "snippet": result.get("snippet", ""),
                    "relevance_score": relevance_score,
                    "source_engine": result.get("engine") or result.get("source"),
                },
                source_worker="MonitoringLoop",
            )
            await self._bus.publish(event)

        return True

    # ------------------------------------------------------------------
    # Helpers
    # ------------------------------------------------------------------

    async def _filter_by_date(
        self,
        results: list[dict[str, Any]],
        before_date: str,
    ) -> list[dict[str, Any]]:
        """Post-filter results by publication date using htmldate.

        For each result that has a URL, fetch the page and extract
        the real publication date. Reject results published after
        before_date. Results without a detectable date are kept
        (benefit of the doubt).
        """
        try:
            from htmldate import find_date
        except ImportError:
            logger.warning("htmldate not installed — skipping date post-filter")
            return results

        filtered: list[dict[str, Any]] = []
        for r in results:
            url = r.get("url", "")
            # Check SearXNG-provided published_date first
            pub_date = r.get("published_date") or r.get("publishedDate")
            if not pub_date and url:
                try:
                    # htmldate extracts date from the actual page
                    pub_date = await asyncio.to_thread(
                        find_date, url, extensive_search=False
                    )
                except Exception:
                    pub_date = None

            if pub_date:
                # Normalize to YYYY-MM-DD string for comparison
                date_str = str(pub_date)[:10]
                if date_str > before_date:
                    logger.debug(
                        "MonitoringLoop: REJECTED (date={} > {}): {}",
                        date_str, before_date, r.get("title", "?")[:50],
                    )
                    continue

            filtered.append(r)

        rejected = len(results) - len(filtered)
        if rejected > 0:
            logger.info(
                "MonitoringLoop: date filter rejected {}/{} results (before:{})",
                rejected, len(results), before_date,
            )
        return filtered

    async def _get_before_date(self, case_id: str) -> str | None:
        """Extract ``before:YYYY-MM-DD`` from case description if present."""
        try:
            async with get_db() as conn:
                db = Database(conn)
                case = await db.get_case(case_id)
                if case:
                    desc = case.get("description", "")
                    m = re.search(r"before:(\d{4}-\d{2}-\d{2})", desc)
                    if m:
                        return m.group(1)
        except Exception:
            pass
        return None
