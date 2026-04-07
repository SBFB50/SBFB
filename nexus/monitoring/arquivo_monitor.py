"""
NEXUS -- Arquivo.pt Monitor.

Full-text search across the Portuguese Web Archive (arquivo.pt).
Arquivo.pt indexes billions of web pages including French news sites,
with guaranteed crawl dates — solving the date detection problem.

API: https://arquivo.pt/textsearch
No API key required. Dates are archive timestamps, always available.
"""

from __future__ import annotations

import asyncio
import html
import re
from datetime import datetime
from typing import Any, List, Optional

import httpx
from loguru import logger

_TEXTSEARCH_API = "https://arquivo.pt/textsearch"


class ArquivoMonitor:
    """Search archived web pages via Arquivo.pt full-text search API."""

    async def search(
        self,
        query: str,
        max_results: int = 20,
        before_date: Optional[str] = None,
        after_date: Optional[str] = None,
        site: Optional[str] = None,
    ) -> List[dict[str, Any]]:
        """Full-text search across archived web pages.

        Unlike Wayback CDX (URL-only), this searches PAGE CONTENT.
        Dates are guaranteed (archive crawl timestamps).
        """
        # Build date params (format: YYYYMMDDHHmmss)
        from_ts = after_date.replace("-", "") + "000000" if after_date else None
        to_ts = before_date.replace("-", "") + "235959" if before_date else None

        params: dict[str, Any] = {
            "q": query,
            "maxItems": min(max_results, 200),
            "fields": "title,originalURL,tstamp,snippet,linkToExtractedText,date,mimeType",
            "prettyPrint": "false",
            "dedupField": "title",
            "dedupValue": 2,
        }
        if from_ts:
            params["from"] = from_ts
        if to_ts:
            params["to"] = to_ts
        if site:
            params["siteSearch"] = site

        results: list[dict[str, Any]] = []

        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                resp = await client.get(_TEXTSEARCH_API, params=params)
                resp.raise_for_status()
                data = resp.json()

            for item in data.get("response_items", []):
                # Clean snippet (contains HTML entities and tags)
                snippet_raw = item.get("snippet", "")
                snippet_clean = html.unescape(re.sub(r"<[^>]+>", "", snippet_raw)).strip()

                # Parse crawl timestamp
                tstamp = item.get("tstamp", "")
                date_str = ""
                if len(tstamp) >= 8:
                    date_str = f"{tstamp[:4]}-{tstamp[4:6]}-{tstamp[6:8]}"

                results.append({
                    "url": item.get("originalURL", ""),
                    "title": item.get("title", ""),
                    "snippet": snippet_clean[:500],
                    "engine": "arquivo.pt",
                    "source": "arquivo",
                    "published_date": date_str,  # Guaranteed crawl date
                    "archived_at": tstamp,
                    "text_url": item.get("linkToExtractedText"),
                    "mime_type": item.get("mimeType", ""),
                })

            logger.info(
                "Arquivo.pt search '{}': {} results (before={}, after={})",
                query[:40], len(results), before_date, after_date,
            )

        except httpx.TimeoutException:
            logger.warning("Arquivo.pt timeout for query '{}'", query[:40])
        except Exception as exc:
            logger.error("Arquivo.pt search failed: {}", exc)

        return results[:max_results]

    async def search_french_press(
        self,
        query: str,
        before_date: Optional[str] = None,
        after_date: Optional[str] = None,
        max_results: int = 30,
    ) -> List[dict[str, Any]]:
        """Search across known French news domains on Arquivo.pt.

        Searches multiple French press sites in parallel for better coverage.
        """
        french_domains = [
            "www.leparisien.fr", "www.lefigaro.fr", "www.lemonde.fr",
            "www.liberation.fr", "www.20minutes.fr", "www.lunion.fr",
            "www.lavoixdunord.fr", "www.europe1.fr", "www.lexpress.fr",
            "www.ladepeche.fr", "www.paris-match.com", "www.francetvinfo.fr",
        ]

        all_results: list[dict[str, Any]] = []
        seen_urls: set[str] = set()

        # Search without site filter first (broadest coverage)
        broad = await self.search(
            query, max_results=max_results,
            before_date=before_date, after_date=after_date,
        )
        for r in broad:
            if r["url"] not in seen_urls:
                all_results.append(r)
                seen_urls.add(r["url"])

        # Then search specific French domains for additional coverage
        tasks = []
        for domain in french_domains[:6]:  # limit to 6 parallel
            tasks.append(self.search(
                query, max_results=5,
                before_date=before_date, after_date=after_date,
                site=domain,
            ))

        if tasks:
            await asyncio.sleep(0.5)  # rate limit courtesy
            domain_results = await asyncio.gather(*tasks, return_exceptions=True)
            for res in domain_results:
                if isinstance(res, list):
                    for r in res:
                        if r["url"] not in seen_urls:
                            all_results.append(r)
                            seen_urls.add(r["url"])

        logger.info(
            "Arquivo.pt French press search '{}': {} results",
            query[:40], len(all_results),
        )
        return all_results[:max_results]

    async def fetch_full_text(
        self,
        text_url: str,
        max_chars: int = 8000,
    ) -> str | None:
        """Fetch full extracted text from an archived page."""
        try:
            async with httpx.AsyncClient(timeout=20.0) as client:
                resp = await client.get(text_url)
                resp.raise_for_status()
                text = resp.text.strip()
                return text[:max_chars] if text else None
        except Exception as exc:
            logger.debug("Arquivo.pt text fetch failed: {}", exc)
            return None
