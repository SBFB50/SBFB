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

        # Progressive time window: advances before_date adaptively
        self._time_window_start: float | None = None  # monotonic timestamp of first sweep
        self._base_before_date: str | None = None      # original before: from case desc
        self._crime_year: int | None = None             # extracted from case dates
        self._current_window_year: int | None = None    # current simulated year
        self._dry_sweeps: int = 0                       # consecutive sweeps with 0 new results
        self._last_advance_time: float = 0              # when we last advanced

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

        events_before = self._bus.published_count if hasattr(self._bus, 'published_count') else 0
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

        # Adaptive time window: count only events published (= valuable results that
        # passed relevance threshold), not all monitoring_results stored
        events_after = self._bus.published_count if hasattr(self._bus, 'published_count') else 0
        valuable_count = events_after - events_before
        self.notify_results_found(valuable_count)

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

        # Wayback Machine: only on first sweep per job (slow, usually low value)
        if before_date and not job.get("last_run"):
            try:
                wayback_results = await asyncio.wait_for(
                    self._wayback.search(
                        query=query,
                        max_results=5,
                        before_date=before_date,
                    ),
                    timeout=15.0,
                )
                raw_results.extend(wayback_results)
            except asyncio.TimeoutError:
                logger.warning("Wayback search timed out for job {}", job_id[:8])
            except Exception:
                logger.debug("Wayback search failed for job {}", job_id[:8])

        # 2. Update timestamps regardless of results
        now = datetime.now(timezone.utc).isoformat()
        interval_hours = job.get("interval_hours", 6)
        # Minimum 2 minutes between re-runs to avoid spamming search engines
        interval_delta = max(timedelta(hours=interval_hours), timedelta(minutes=2))
        next_run = (
            datetime.now(timezone.utc) + interval_delta
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
        htmldate_calls = 0
        _MAX_HTMLDATE = 5  # limit slow HTTP fetches per batch

        for r in results:
            url = r.get("url", "")
            # Check SearXNG-provided published_date first
            pub_date = r.get("published_date") or r.get("publishedDate")

            # YouTube/video: use yt-dlp for upload date (fast, no download)
            if not pub_date and url and ("youtube.com" in url or "youtu.be" in url):
                try:
                    import yt_dlp
                    opts = {"quiet": True, "skip_download": True, "no_warnings": True}
                    info = await asyncio.wait_for(
                        asyncio.to_thread(
                            lambda: yt_dlp.YoutubeDL(opts).extract_info(url, download=False)
                        ),
                        timeout=8.0,
                    )
                    raw_date = (info or {}).get("upload_date", "")
                    if raw_date and len(raw_date) == 8:
                        pub_date = f"{raw_date[:4]}-{raw_date[4:6]}-{raw_date[6:8]}"
                except Exception:
                    pass

            if not pub_date and url and htmldate_calls < _MAX_HTMLDATE:
                try:
                    pub_date = await asyncio.wait_for(
                        asyncio.to_thread(find_date, url, extensive_search=False),
                        timeout=5.0,
                    )
                    htmldate_calls += 1
                except Exception:
                    pub_date = None

            if pub_date:
                date_str = str(pub_date)[:10]
                if date_str > before_date:
                    logger.debug(
                        "MonitoringLoop: REJECTED (date={} > {}): {}",
                        date_str, before_date, r.get("title", "?")[:50],
                    )
                    continue
                filtered.append(r)
                continue

            # No date detected → reject. No guessing.
            logger.debug(
                "MonitoringLoop: REJECTED (no date detected): {}",
                r.get("title", "?")[:50],
            )
            continue

        rejected = len(results) - len(filtered)
        if rejected > 0:
            logger.info(
                "MonitoringLoop: date filter rejected {}/{} results (before:{})",
                rejected, len(results), before_date,
            )
        return filtered

    # ------------------------------------------------------------------
    # Adaptive progressive time window
    # ------------------------------------------------------------------
    # Simulates passage of time adaptively:
    #   - Starts at crime_year + 1
    #   - If a sweep finds 0 new results → advance faster (dry streak)
    #   - If results found → hold the window to let NEXUS process them
    #   - Advances by 1 year per step, speed depends on findings
    #
    # Dry sweeps needed to advance:
    #   0-1 dry sweeps → hold (just checked, wait)
    #   2 dry sweeps   → advance 1 year (nothing here)
    #   After 3+ consecutive dry at same year → advance 2 years (skip)

    def notify_results_found(self, count: int) -> None:
        """Called after a sweep produces new ingested results."""
        if count > 0:
            self._dry_sweeps = 0
        else:
            self._dry_sweeps += 1
            self._maybe_advance_window()

    def _maybe_advance_window(self) -> None:
        """Advance the time window if we've had enough dry sweeps."""
        if self._current_window_year is None or self._base_before_date is None:
            return
        ceiling = int(self._base_before_date[:4])
        if self._current_window_year >= ceiling:
            return

        if self._dry_sweeps >= 3:
            step = 2  # skip faster — nothing interesting in this era
        elif self._dry_sweeps >= 2:
            step = 1
        else:
            return  # too early to advance

        old = self._current_window_year
        self._current_window_year = min(self._current_window_year + step, ceiling)
        self._dry_sweeps = 0

        if self._current_window_year != old:
            logger.info(
                "MonitoringLoop: TIME ADVANCE {} → {} (ceiling {}, after {} dry sweeps)",
                old, self._current_window_year, ceiling, self._dry_sweeps + 2,
            )
            # Trigger reverse discovery on the new year range
            asyncio.create_task(
                self._reverse_discover_on_advance(old, self._current_window_year),
                name=f"reverse-discover-{old}-{self._current_window_year}",
            )

    async def _reverse_discover_on_advance(self, from_year: int, to_year: int) -> None:
        """Launch reverse discovery on key news domains when the time window advances."""
        try:
            # Get case keywords from entities
            async with get_db() as conn:
                db = Database(conn)
                entities = await db.list_entities_by_case(self._case_id)
                case = await db.get_case(self._case_id)

            keywords = []
            for e in entities:
                if e.get("entity_type") in ("person", "location") and e.get("name"):
                    keywords.append(e["name"].lower().split()[0])  # first word
            # Add case name keywords
            if case:
                for w in (case.get("name", "") + " " + case.get("description", "")).split():
                    w = w.strip().lower()
                    if len(w) >= 4 and w not in ("mode", "osint", "cold", "case", "before"):
                        keywords.append(w)
            keywords = list(dict.fromkeys(keywords))[:6]  # dedup, max 6

            if not keywords:
                return

            # Reverse discover on top 3 regional news domains
            for domain in ["courrier-picard.fr", "lavoixdunord.fr", "francetvinfo.fr"]:
                try:
                    results = await self._wayback.reverse_discover(
                        domain=domain,
                        from_year=str(from_year),
                        to_year=str(to_year),
                        keywords=keywords,
                        max_pages=100,
                        max_results=5,
                    )
                    # Publish results as MONITORING_RESULT events
                    for r in results:
                        event = NexusEvent(
                            event_type=EventType.MONITORING_RESULT,
                            case_id=self._case_id,
                            payload={
                                "title": r.get("title", ""),
                                "url": r.get("url", ""),
                                "snippet": r.get("snippet", ""),
                                "source_engine": "wayback_reverse",
                                "relevance_score": 90.0,  # high relevance — content-matched
                            },
                            source_worker="monitoring_loop",
                        )
                        await self._bus.publish(event)
                        logger.info(
                            "MonitoringLoop: reverse discovery found '{}' on {}",
                            r.get("title", "?")[:50], domain,
                        )
                except Exception as exc:
                    logger.debug("Reverse discovery failed for {}: {}", domain, exc)

        except Exception as exc:
            logger.warning("Reverse discovery task failed: {}", exc)

    async def _get_before_date(self, case_id: str) -> str | None:
        """Get the current adaptive before-date for this case."""
        # Lazy init: read ceiling from case description once
        if self._base_before_date is None:
            try:
                async with get_db() as conn:
                    db = Database(conn)
                    case = await db.get_case(case_id)
                    if case:
                        desc = case.get("description", "") or ""
                        m = re.search(r"before:(\d{4}-\d{2}-\d{2})", desc)
                        if m:
                            self._base_before_date = m.group(1)
                        else:
                            self._base_before_date = ""
                            return None

                        # Extract crime year from earliest dates
                        entities = await db.list_entities_by_case(case_id, entity_type="date")
                        years = []
                        for e in entities:
                            fs = e.get("first_seen") or ""
                            if fs and len(fs) >= 4:
                                try:
                                    years.append(int(fs[:4]))
                                except ValueError:
                                    pass
                        for y in re.findall(r'\b(19\d{2}|20\d{2})\b', desc):
                            years.append(int(y))

                        self._crime_year = min(years) if years else int(self._base_before_date[:4]) - 10
                        self._current_window_year = self._crime_year + 1
                        self._dry_sweeps = 0

                        logger.info(
                            "MonitoringLoop: adaptive time window — start=%d ceiling=%s",
                            self._current_window_year, self._base_before_date,
                        )
            except Exception:
                self._base_before_date = ""
                return None

        if not self._base_before_date:
            return None

        ceiling_year = int(self._base_before_date[:4])
        current = self._current_window_year or (self._crime_year or 2002) + 1

        if current >= ceiling_year:
            current_date = self._base_before_date
        else:
            current_date = f"{current}-01-01"

        logger.info(
            "MonitoringLoop: window={} (dry_sweeps={}, ceiling={})",
            current_date, self._dry_sweeps, self._base_before_date,
        )
        return current_date
