"""
NEXUS -- Wayback Machine monitor.

Searches the Internet Archive's Wayback Machine CDX API for archived
web pages matching a query within a date range. This allows OSINT
searches on historical content that no longer exists on the live web.

The CDX API indexes every snapshot stored in the Wayback Machine.
Date ranges are specified as YYYYMMDD (from/to parameters).
No API key required.

Usage::

    monitor = WaybackMonitor()
    results = await monitor.search("Elodie Kulik", before_date="2002-06-01")
"""

from __future__ import annotations

import asyncio
from typing import Any, List, Optional

import httpx
from loguru import logger


_CDX_API = "https://web.archive.org/cdx/search/cdx"
_WAYBACK_PREFIX = "https://web.archive.org/web"


class WaybackMonitor:
    """Search archived web pages via Internet Archive CDX API."""

    async def search(
        self,
        query: str,
        max_results: int = 15,
        before_date: Optional[str] = None,
        after_date: Optional[str] = None,
    ) -> List[dict[str, Any]]:
        """Search Wayback Machine for pages matching query terms.

        Strategy: use the CDX API to find archived pages from news sites
        that contain the query terms in their URL. Then fetch the archived
        snapshot to extract title and snippet.

        Args:
            query: Search terms (e.g. "Elodie Kulik Cartigny")
            max_results: Max results to return
            before_date: YYYY-MM-DD — only pages archived before this date
            after_date: YYYY-MM-DD — only pages archived after this date
        """
        # Build URL patterns from query keywords
        keywords = [w.strip().lower() for w in query.split() if len(w.strip()) >= 3]
        if not keywords:
            return []

        # Convert dates to CDX format (YYYYMMDD)
        from_ts = after_date.replace("-", "") if after_date else None
        to_ts = before_date.replace("-", "") if before_date else None

        # Search multiple news domains for URL matches
        domains = [
            "leparisien.fr", "lefigaro.fr", "lemonde.fr",
            "liberation.fr", "20minutes.fr", "france3-regions.francetvinfo.fr",
            "courrier-picard.fr", "aisnenouvelle.fr",
        ]

        all_results: list[dict[str, Any]] = []

        async with httpx.AsyncClient(timeout=15.0) as client:
            for domain in domains:
                if len(all_results) >= max_results:
                    break
                try:
                    results = await self._search_domain(
                        client, domain, keywords, from_ts, to_ts, max_results=5
                    )
                    all_results.extend(results)
                except Exception as exc:
                    logger.debug("Wayback search failed for {}: {}", domain, exc)

            # Also search with wildcard URL matching for the primary keyword
            if keywords and len(all_results) < max_results:
                try:
                    primary = keywords[0]
                    results = await self._search_wildcard(
                        client, primary, from_ts, to_ts, max_results=10
                    )
                    # Dedup by URL
                    seen = {r["url"] for r in all_results}
                    for r in results:
                        if r["url"] not in seen and len(all_results) < max_results:
                            all_results.append(r)
                            seen.add(r["url"])
                except Exception as exc:
                    logger.debug("Wayback wildcard search failed: {}", exc)

        logger.info(
            "Wayback search '{}': {} results (before={})",
            query[:40], len(all_results), before_date,
        )
        return all_results[:max_results]

    async def _search_domain(
        self,
        client: httpx.AsyncClient,
        domain: str,
        keywords: list[str],
        from_ts: str | None,
        to_ts: str | None,
        max_results: int = 5,
    ) -> list[dict[str, Any]]:
        """Search a specific domain's archived pages via CDX API."""
        # Build URL filter: match URLs containing keywords
        url_pattern = f"{domain}/*"

        params: dict[str, Any] = {
            "url": url_pattern,
            "matchType": "prefix",
            "output": "json",
            "limit": max_results * 3,  # Over-fetch for filtering
            "fl": "timestamp,original,statuscode,length",
            "filter": "statuscode:200",
        }
        if from_ts:
            params["from"] = from_ts
        if to_ts:
            params["to"] = to_ts

        resp = await client.get(_CDX_API, params=params)
        resp.raise_for_status()
        rows = resp.json()

        if not rows or len(rows) < 2:
            return []

        # First row is header
        header = rows[0]
        results: list[dict[str, Any]] = []

        for row in rows[1:]:
            if len(results) >= max_results:
                break
            entry = dict(zip(header, row))
            url = entry.get("original", "")
            url_lower = url.lower()

            # Filter: URL must contain at least one keyword
            if not any(kw in url_lower for kw in keywords):
                continue

            timestamp = entry.get("timestamp", "")
            wayback_url = f"{_WAYBACK_PREFIX}/{timestamp}/{url}"

            # Format date
            date_str = ""
            if len(timestamp) >= 8:
                date_str = f"{timestamp[:4]}-{timestamp[4:6]}-{timestamp[6:8]}"

            results.append({
                "url": wayback_url,
                "original_url": url,
                "title": f"[Archive {date_str}] {url.split('/')[-1][:60]}",
                "snippet": f"Page archivee le {date_str} sur {domain}",
                "engine": "wayback",
                "source": "wayback",
                "published_date": date_str,
                "archived_at": timestamp,
            })

        return results

    async def _search_wildcard(
        self,
        client: httpx.AsyncClient,
        keyword: str,
        from_ts: str | None,
        to_ts: str | None,
        max_results: int = 10,
    ) -> list[dict[str, Any]]:
        """Wildcard search across all domains for a keyword in URLs."""
        params: dict[str, Any] = {
            "url": f"*.fr/*{keyword}*",
            "matchType": "domain",
            "output": "json",
            "limit": max_results * 2,
            "fl": "timestamp,original,statuscode",
            "filter": "statuscode:200",
        }
        if from_ts:
            params["from"] = from_ts
        if to_ts:
            params["to"] = to_ts

        resp = await client.get(_CDX_API, params=params)
        resp.raise_for_status()
        rows = resp.json()

        if not rows or len(rows) < 2:
            return []

        header = rows[0]
        results: list[dict[str, Any]] = []

        for row in rows[1:]:
            if len(results) >= max_results:
                break
            entry = dict(zip(header, row))
            url = entry.get("original", "")
            timestamp = entry.get("timestamp", "")
            wayback_url = f"{_WAYBACK_PREFIX}/{timestamp}/{url}"
            date_str = f"{timestamp[:4]}-{timestamp[4:6]}-{timestamp[6:8]}" if len(timestamp) >= 8 else ""

            results.append({
                "url": wayback_url,
                "original_url": url,
                "title": f"[Archive {date_str}] {url.split('/')[-1][:60]}",
                "snippet": f"Page archivee le {date_str}",
                "engine": "wayback",
                "source": "wayback",
                "published_date": date_str,
            })

        return results

    async def fetch_archived_text(
        self,
        wayback_url: str,
        max_chars: int = 8000,
    ) -> str | None:
        """Fetch the text content of an archived page."""
        try:
            async with httpx.AsyncClient(timeout=20.0, follow_redirects=True) as client:
                resp = await client.get(wayback_url)
                resp.raise_for_status()
                # Strip HTML tags (basic)
                import re
                text = re.sub(r'<[^>]+>', ' ', resp.text)
                text = re.sub(r'\s+', ' ', text).strip()
                return text[:max_chars] if text else None
        except Exception as exc:
            logger.debug("Wayback fetch failed for {}: {}", wayback_url[:60], exc)
            return None
