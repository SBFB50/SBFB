"""
NEXUS -- Monitoring scheduler (APScheduler).

Manages recurring monitoring jobs that search clearweb (SearXNG)
and dark web (Robin/Tor) for new information related to active cases.

Jobs are persisted in SQLite.  The scheduler loads all active jobs
on startup and executes them at their configured interval.
"""

from __future__ import annotations

import asyncio
from datetime import datetime, timedelta
from typing import Optional

from apscheduler.schedulers.asyncio import AsyncIOScheduler
from apscheduler.triggers.interval import IntervalTrigger
from loguru import logger

from nexus.config import settings
from nexus.db.chroma_db import ChromaClient
from nexus.db.sqlite_db import Database, get_db
from nexus.llm.router import LLMRouter, TaskType
from nexus.llm.prompts import QUERY_REFORMULATION_PROMPT, RESULT_FILTERING_PROMPT
from nexus.monitoring.searxng_monitor import SearXNGMonitor
from nexus.monitoring.robin_monitor import RobinMonitor
from nexus.monitoring.alert_manager import AlertManager


class MonitoringScheduler:
    """Orchestrates recurring monitoring jobs via APScheduler.

    Each monitoring job performs:
    1. Query reformulation via LLM (gemma4:e4b)
    2. Search execution (SearXNG and/or Robin)
    3. Deduplication via ChromaDB embeddings
    4. Relevance filtering via LLM (gemma4:e4b)
    5. Alert creation for high-relevance hits
    """

    def __init__(
        self,
        router: LLMRouter,
        chroma: ChromaClient,
    ) -> None:
        self._router = router
        self._chroma = chroma
        self._searxng = SearXNGMonitor()
        self._robin = RobinMonitor()
        self._scheduler = AsyncIOScheduler(
            timezone="UTC",
            job_defaults={
                "coalesce": True,       # Merge missed runs into one
                "max_instances": 1,     # No parallel runs of the same job
                "misfire_grace_time": 3600,  # Allow 1h late
            },
        )

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    async def start(self) -> None:
        """Start the scheduler and load all active jobs from the DB."""
        async with get_db() as conn:
            db = Database(conn)
            # Fetch all active monitoring jobs across all cases
            cursor = await conn.execute(
                "SELECT * FROM monitoring_jobs WHERE is_active = 1"
            )
            rows = [dict(r) for r in await cursor.fetchall()]

        for job_row in rows:
            self._add_scheduler_job(job_row["id"], job_row["interval_hours"])

        self._scheduler.start()
        logger.info(
            "MonitoringScheduler started with {} active jobs",
            len(rows),
        )

    async def stop(self) -> None:
        """Shut down the scheduler gracefully."""
        self._scheduler.shutdown(wait=False)
        logger.info("MonitoringScheduler stopped")

    # ------------------------------------------------------------------
    # Job management
    # ------------------------------------------------------------------

    def add_job(self, job_id: str, interval_hours: int) -> None:
        """Register a new recurring job in the scheduler."""
        self._add_scheduler_job(job_id, interval_hours)
        logger.info(
            "Scheduler: added job {} (every {}h)", job_id, interval_hours
        )

    def remove_job(self, job_id: str) -> None:
        """Remove a job from the scheduler."""
        scheduler_id = f"monitoring_{job_id}"
        try:
            self._scheduler.remove_job(scheduler_id)
            logger.info("Scheduler: removed job {}", job_id)
        except Exception:
            logger.warning("Scheduler: job {} not found for removal", job_id)

    def update_job_interval(self, job_id: str, interval_hours: int) -> None:
        """Reschedule an existing job with a new interval."""
        scheduler_id = f"monitoring_{job_id}"
        try:
            self._scheduler.reschedule_job(
                scheduler_id,
                trigger=IntervalTrigger(hours=interval_hours),
            )
            logger.info(
                "Scheduler: rescheduled job {} to every {}h",
                job_id,
                interval_hours,
            )
        except Exception:
            # Job not found -- add it fresh
            self._add_scheduler_job(job_id, interval_hours)

    def trigger_job(self, job_id: str) -> None:
        """Force an immediate execution of a monitoring job."""
        scheduler_id = f"monitoring_{job_id}"
        try:
            self._scheduler.modify_job(
                scheduler_id,
                next_run_time=datetime.utcnow(),
            )
            logger.info("Scheduler: triggered immediate run for job {}", job_id)
        except Exception:
            logger.warning(
                "Scheduler: job {} not found for trigger, running directly",
                job_id,
            )
            # Schedule a one-off run directly
            asyncio.ensure_future(self._execute_monitoring_job(job_id))

    # ------------------------------------------------------------------
    # Internal: scheduler wiring
    # ------------------------------------------------------------------

    def _add_scheduler_job(self, job_id: str, interval_hours: int) -> None:
        """Wire an APScheduler job to ``_execute_monitoring_job``."""
        scheduler_id = f"monitoring_{job_id}"
        # Remove if already exists (idempotent add)
        try:
            self._scheduler.remove_job(scheduler_id)
        except Exception:
            pass
        self._scheduler.add_job(
            self._execute_monitoring_job,
            trigger=IntervalTrigger(hours=interval_hours),
            id=scheduler_id,
            args=[job_id],
            name=f"Monitor {job_id[:8]}...",
        )

    # ------------------------------------------------------------------
    # Core execution logic
    # ------------------------------------------------------------------

    async def _execute_monitoring_job(self, job_id: str) -> None:
        """Execute a single monitoring cycle for a job.

        Opens its own DB connection (scheduler runs outside request scope).
        """
        logger.info("Executing monitoring job {}", job_id)

        try:
            async with get_db() as conn:
                db = Database(conn)
                alert_mgr = AlertManager(db)

                # 1. Load the job definition
                job = await db._get_monitoring_job(job_id)
                if job is None:
                    logger.error("Monitoring job {} not found -- removing from scheduler", job_id)
                    self.remove_job(job_id)
                    return

                if not job["is_active"]:
                    logger.info("Monitoring job {} is inactive -- skipping", job_id)
                    return

                case_id = job["case_id"]
                query = job["query"]
                job_type = job["job_type"]

                # Check for date filter (cold case benchmark: avoid spoilers)
                # Stored in case description as "before:YYYY-MM-DD" or in job metadata
                before_date = None
                try:
                    import json as _json
                    case = await db.get_case(case_id)
                    if case:
                        desc = case.get("description", "")
                        # Extract before:YYYY-MM-DD from case description
                        import re
                        m = re.search(r"before:(\d{4}-\d{2}-\d{2})", desc)
                        if m:
                            before_date = m.group(1)
                except Exception:
                    pass

                # 2. Search
                raw_results: list[dict] = []

                if job_type in ("searxng", "both"):
                    try:
                        searxng_results = await self._searxng.search(
                            query=query,
                            max_results=20,
                            before_date=before_date,
                        )
                        raw_results.extend(searxng_results)
                        logger.debug(
                            "SearXNG returned {} results for job {}",
                            len(searxng_results),
                            job_id,
                        )
                    except Exception:
                        logger.exception(
                            "SearXNG search failed for job {}", job_id
                        )

                if job_type in ("robin", "both"):
                    try:
                        if await self._robin.is_available():
                            robin_results = await self._robin.search(
                                query=query,
                                max_results=10,
                            )
                            raw_results.extend(robin_results)
                            logger.debug(
                                "Robin returned {} results for job {}",
                                len(robin_results),
                                job_id,
                            )
                        else:
                            logger.warning("Robin unavailable for job {}", job_id)
                    except Exception:
                        logger.exception(
                            "Robin search failed for job {}", job_id
                        )

                if not raw_results:
                    logger.info("No results for job {} -- updating timestamps", job_id)
                    now = datetime.utcnow().isoformat()
                    next_run = (
                        datetime.utcnow() + timedelta(hours=job["interval_hours"])
                    ).isoformat()
                    await db.update_job(job_id, last_run=now, next_run=next_run)
                    return

                # 3. Deduplicate & filter each result
                stored_count = 0
                for result in raw_results:
                    text_for_embed = f"{result.get('title', '')} {result.get('snippet', '')}"
                    if not text_for_embed.strip():
                        continue

                    # Compute embedding for deduplication
                    try:
                        embedding = await self._router.embed(text_for_embed)
                    except Exception:
                        logger.exception("Embedding failed for result: {}", result.get("url"))
                        continue

                    # Check semantic duplicate in ChromaDB
                    is_dup = False
                    try:
                        is_dup = self._chroma.is_duplicate_result(
                            case_id=case_id,
                            embedding=embedding,
                            threshold=0.92,
                        )
                    except Exception:
                        logger.warning("Duplicate check failed, treating as new")

                    # Relevance scoring via LLM
                    relevance_score: Optional[float] = None
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
                        relevance_score = filter_resp.get("relevance_score")
                        if isinstance(relevance_score, (int, float)):
                            # Scale 0-1 to 0-100 for the DB field
                            relevance_score = round(float(relevance_score) * 100, 1)
                        else:
                            relevance_score = None
                    except Exception:
                        logger.warning(
                            "Relevance filtering failed for {}", result.get("url")
                        )

                    # 4. Store result in SQLite
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

                    # 5. Store embedding in ChromaDB for future dedup
                    if not is_dup:
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
                                "Failed to store embedding for result {}",
                                db_result["id"],
                            )

                    stored_count += 1

                    # 6. Create alert for high-relevance, non-duplicate results
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
                                "Alert creation failed for result {}",
                                db_result["id"],
                            )

                # 7. Update job timestamps
                now = datetime.utcnow().isoformat()
                next_run = (
                    datetime.utcnow() + timedelta(hours=job["interval_hours"])
                ).isoformat()
                await db.update_job(job_id, last_run=now, next_run=next_run)

                logger.info(
                    "Monitoring job {} completed: {} results stored ({} total raw)",
                    job_id,
                    stored_count,
                    len(raw_results),
                )

        except Exception:
            logger.exception("Monitoring job {} FAILED", job_id)
