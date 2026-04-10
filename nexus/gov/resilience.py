"""
NEXUS GOV -- Source Health Monitor.

Tracks availability of each data source. Auto-fallback when primary
sources go down. Circuit breaker pattern per source.
"""
from __future__ import annotations

import asyncio
import time
from dataclasses import dataclass, field
from typing import Any, Callable, Optional

from loguru import logger

import httpx


@dataclass
class SourceHealth:
    name: str
    url: str
    status: str = "unknown"  # "healthy", "degraded", "down", "unknown"
    last_check: float = 0.0
    last_success: float = 0.0
    consecutive_failures: int = 0
    total_checks: int = 0
    total_failures: int = 0
    response_time_ms: float = 0.0
    error_message: str = ""


# All monitored sources
SOURCES = [
    SourceHealth("PoliGraph API", "https://poligraph.fr/api/politiques?limit=1"),
    SourceHealth("Assemblee Nationale", "https://data.assemblee-nationale.fr/"),
    SourceHealth("Senat API", "https://www.senat.fr/api-senat/senateurs.json"),
    SourceHealth("HATVP", "https://www.hatvp.fr/livraison/opendata/liste.csv"),
    SourceHealth(
        "Wikidata SPARQL",
        "https://query.wikidata.org/sparql?query=SELECT%20%3Fs%20WHERE%20%7B%20%3Fs%20%3Fp%20%3Fo%20%7D%20LIMIT%201&format=json",
    ),
    SourceHealth("data.gouv.fr", "https://www.data.gouv.fr/api/1/"),
    SourceHealth(
        "La Fabrique de la Loi",
        "https://www.lafabriquedelaloi.fr/api/stats/metrics.csv",
    ),
    SourceHealth("SearXNG", "http://localhost:8888/search?q=test&format=json"),
    SourceHealth(
        "EU Parliament",
        "https://data.europarl.europa.eu/api/v2/meps/show-current",
    ),
    SourceHealth(
        "EUR-Lex SPARQL",
        "https://publications.europa.eu/webapi/rdf/sparql",
    ),
    SourceHealth(
        "Google Fact Check",
        "https://factchecktools.googleapis.com/v1alpha1/claims:search",
    ),
]


class SourceHealthMonitor:
    """Monitors health of all government data sources."""

    CIRCUIT_OPEN_THRESHOLD = 5  # consecutive failures to open circuit
    CHECK_INTERVAL = 300  # 5 minutes between checks
    DOWN_BACKOFF = 900  # 15 minutes between probes for "down" sources

    def __init__(self) -> None:
        self._sources = {s.name: s for s in SOURCES}
        self._task: Optional[asyncio.Task] = None
        self._running = False

    async def start(self) -> None:
        """Start periodic health checks."""
        self._running = True
        self._task = asyncio.create_task(self._check_loop())
        logger.info(
            "Source health monitor started ({} sources)", len(self._sources)
        )

    async def stop(self) -> None:
        self._running = False
        if self._task and not self._task.done():
            self._task.cancel()
            try:
                await self._task
            except asyncio.CancelledError:
                pass

    async def _check_loop(self) -> None:
        while self._running:
            for source in self._sources.values():
                await self._check_source(source)
                await asyncio.sleep(2)  # Don't hammer all at once
            await asyncio.sleep(self.CHECK_INTERVAL)

    async def _check_source(self, source: SourceHealth) -> None:
        now = time.time()

        # Skip probing "down" sources until backoff expires
        if (
            source.status == "down"
            and source.last_check > 0
            and now - source.last_check < self.DOWN_BACKOFF
        ):
            return

        source.total_checks += 1
        source.last_check = now

        try:
            start = time.monotonic()
            async with httpx.AsyncClient(timeout=10.0) as client:
                resp = await client.get(source.url)
                elapsed = (time.monotonic() - start) * 1000
                source.response_time_ms = round(elapsed, 1)

                if resp.status_code < 400:
                    source.status = "healthy"
                    source.consecutive_failures = 0
                    source.last_success = now
                    source.error_message = ""
                else:
                    # 4xx and 5xx both count as failures
                    source.consecutive_failures += 1
                    source.total_failures += 1
                    source.error_message = f"HTTP {resp.status_code}"
                    if source.consecutive_failures >= self.CIRCUIT_OPEN_THRESHOLD:
                        source.status = "down"
                    else:
                        source.status = "degraded"

        except Exception as exc:
            source.consecutive_failures += 1
            source.total_failures += 1
            source.error_message = str(exc)[:200]
            source.response_time_ms = 0

            if source.consecutive_failures >= self.CIRCUIT_OPEN_THRESHOLD:
                source.status = "down"
            else:
                source.status = "degraded"

    def get_status(self) -> list[dict]:
        """Return health status of all sources."""
        return [
            {
                "name": s.name,
                "url": s.url,
                "status": s.status,
                "response_time_ms": s.response_time_ms,
                "consecutive_failures": s.consecutive_failures,
                "total_checks": s.total_checks,
                "total_failures": s.total_failures,
                "last_check": s.last_check,
                "last_success": s.last_success,
                "error": s.error_message,
            }
            for s in self._sources.values()
        ]

    def is_healthy(self, source_name: str) -> bool:
        s = self._sources.get(source_name)
        return s is not None and s.status in ("healthy", "degraded", "unknown")

    def get_all_health(self) -> dict:
        """Return all source health statuses as a JSON-serializable dict.

        Complements ``get_status()`` (list format) with a keyed dict that
        is easier to look up by source name in frontend dashboards.
        """
        from datetime import datetime, timezone

        def _epoch_to_iso(ts: float) -> str | None:
            if not ts:
                return None
            return datetime.fromtimestamp(ts, tz=timezone.utc).isoformat()

        return {
            name: {
                "status": s.status,
                "response_time_ms": s.response_time_ms,
                "consecutive_failures": s.consecutive_failures,
                "last_check": _epoch_to_iso(s.last_check),
                "last_success": _epoch_to_iso(s.last_success),
                "total_checks": s.total_checks,
                "total_failures": s.total_failures,
                "error": s.error_message or None,
            }
            for name, s in self._sources.items()
        }

    def get_fallback_order(self, primary: str, *fallbacks: str) -> list[str]:
        """Return sources in order of preference, healthy first."""
        all_sources = [primary] + list(fallbacks)
        healthy = [s for s in all_sources if self.is_healthy(s)]
        unhealthy = [s for s in all_sources if not self.is_healthy(s)]
        return healthy + unhealthy
