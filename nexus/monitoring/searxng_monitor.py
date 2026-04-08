"""
NEXUS -- SearXNG clearweb search monitor.

Provides async search against a local SearXNG instance for
monitoring jobs.  Handles timeouts, rate-limiting pauses,
and result normalisation.
"""

from __future__ import annotations

import asyncio
from typing import Any, Dict, List, Optional

import httpx
from loguru import logger

from nexus.config import settings


class SearXNGMonitor:
    """Async client for the local SearXNG metasearch engine.

    Usage::

        monitor = SearXNGMonitor()
        results = await monitor.search("John Doe disparition 2019")
    """

    def __init__(self, searxng_url: str | None = None) -> None:
        self._base_url = (searxng_url or settings.searxng_url).rstrip("/")

    # ------------------------------------------------------------------
    # Single search
    # ------------------------------------------------------------------

    async def search(
        self,
        query: str,
        *,
        categories: str = "general",
        language: str = "fr",
        max_results: int = 20,
        time_range: Optional[str] = None,
        before_date: Optional[str] = None,
    ) -> List[Dict[str, Any]]:
        """Execute a single search query against SearXNG.

        Args:
            query: The search string.
            categories: SearXNG categories (e.g. "general", "news", "social media").
            language: Language code for results.
            max_results: Maximum number of results to return.
            time_range: SearXNG time_range filter ("day", "week", "month", "year").
            before_date: If set, appends "before:YYYY-MM-DD" to the query
                         to filter out results published after this date.
                         Useful for cold case benchmarks to avoid spoilers.

        Returns:
            List of dicts with keys: url, title, snippet, engine, score, publishedDate.
        """
        # Append date filter to query if specified
        effective_query = query
        if before_date:
            effective_query = f"{query} before:{before_date}"

        params = {
            "q": effective_query,
            "format": "json",
            "categories": categories,
            "language": language,
        }
        if time_range:
            params["time_range"] = time_range

        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.get(
                    f"{self._base_url}/search",
                    params=params,
                )
                response.raise_for_status()
                data = response.json()

        except httpx.TimeoutException:
            logger.error(
                "SearXNG timeout searching '{}' (url={})", query, self._base_url
            )
            return []
        except httpx.ConnectError:
            logger.error(
                "SearXNG connection refused at {}", self._base_url
            )
            return []
        except httpx.HTTPStatusError as exc:
            logger.error(
                "SearXNG HTTP error {}: {}", exc.response.status_code, exc
            )
            return []
        except Exception as exc:
            logger.exception("SearXNG unexpected error for query '{}': {}", query, exc)
            return []

        raw_results = data.get("results", [])
        results: List[Dict[str, Any]] = []

        for item in raw_results[:max_results]:
            published = item.get("publishedDate", "")

            # Post-filter: skip articles published after before_date
            if before_date and published:
                try:
                    from datetime import datetime as _dt
                    pub_str = published[:10]  # "YYYY-MM-DD"
                    if pub_str > before_date:
                        continue  # Skip — article is after the cutoff
                except Exception as exc:
                    logger.debug("SearXNG date parse failed for '{}': {}", published[:20], exc)
                    pass  # Can't parse date, keep the result

            results.append({
                "url": item.get("url", ""),
                "title": item.get("title", ""),
                "snippet": item.get("content", ""),
                "engine": ", ".join(item.get("engines", [])) if isinstance(item.get("engines"), list) else item.get("engine", ""),
                "score": item.get("score", 0.0),
                "published_date": published,
            })

        logger.debug(
            "SearXNG search '{}': {} results (from {} raw)",
            query[:60],
            len(results),
            len(raw_results),
        )
        return results

    # ------------------------------------------------------------------
    # Multi-query search with deduplication
    # ------------------------------------------------------------------

    async def search_multiple(
        self,
        queries: List[str],
        *,
        categories: str = "general",
        language: str = "fr",
        max_results: int = 20,
        pause_between: float = 2.0,
    ) -> List[Dict[str, Any]]:
        """Run multiple queries sequentially and deduplicate by URL.

        A pause is inserted between queries to avoid overloading SearXNG
        or triggering upstream rate limits.

        Args:
            queries: List of search strings.
            pause_between: Seconds to wait between queries.

        Returns:
            Deduplicated list of results.
        """
        seen_urls: set[str] = set()
        all_results: List[Dict[str, Any]] = []

        for i, query in enumerate(queries):
            if i > 0:
                await asyncio.sleep(pause_between)

            results = await self.search(
                query,
                categories=categories,
                language=language,
                max_results=max_results,
            )

            for r in results:
                url = r.get("url", "")
                if url and url not in seen_urls:
                    seen_urls.add(url)
                    all_results.append(r)

        logger.debug(
            "SearXNG multi-search: {} queries, {} unique results",
            len(queries),
            len(all_results),
        )
        return all_results
