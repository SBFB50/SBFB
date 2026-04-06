"""
NEXUS -- Advanced Wayback Machine & Historical Archive Monitor.

Multi-strategy search for historical web content:

1. **CDX keyword search**: Find archived URLs containing keywords (fast)
2. **Reverse discovery**: Fetch ALL pages from news domains in date range,
   extract text, grep for case keywords (slow but finds everything)
3. **Common Crawl**: Search CC indexes for additional coverage (2008+)
4. **Gallica BnF**: Search digitized French newspaper OCR text (pre-1956)
5. **Availability check**: Find archived versions of known URLs

No API keys required. All sources are free and open.

Usage::

    monitor = WaybackMonitor()
    results = await monitor.search("Elodie Kulik", before_date="2012-01-01")
    results = await monitor.reverse_discover("courrier-picard.fr", "2002", "2012", ["kulik", "cartigny"])
"""

from __future__ import annotations

import asyncio
import re
from typing import Any, List, Optional

import httpx
from loguru import logger


_CDX_API = "https://web.archive.org/cdx/search/cdx"
_CC_INDEX_API = "https://index.commoncrawl.org"
_CC_INDEX_LIST = "https://index.commoncrawl.org/collinfo.json"
_WAYBACK_PREFIX = "https://web.archive.org/web"
_AVAILABILITY_API = "https://archive.org/wayback/available"
_GALLICA_SRU = "https://gallica.bnf.fr/SRU"

# French news domains for cold case reverse discovery
_NEWS_DOMAINS_FR = [
    "courrier-picard.fr", "lavoixdunord.fr",
    "leparisien.fr", "lefigaro.fr", "lemonde.fr",
    "liberation.fr", "20minutes.fr",
    "france3-regions.francetvinfo.fr", "francetvinfo.fr",
    "francebleu.fr", "ladepeche.fr",
]

_JUNK_RE = re.compile(
    r'robots\.txt|\.css|\.js|\.png|\.jpe?g|\.gif|\.ico|\.svg|\.woff|\.ttf'
    r'|/wp-content/|/wp-admin/|/feed/|/rss|/sitemap|/ads/|/pub/'
    r'|\.pdf$|\.zip$|\.xml$',
    re.IGNORECASE,
)

# Rate limit: max 1 req/sec to Wayback CDX
_RATE_LIMIT = 1.0


class WaybackMonitor:
    """Multi-source historical web content search."""

    # ==================================================================
    # Main search: combines CDX keyword + wildcard
    # ==================================================================

    async def search(
        self,
        query: str,
        max_results: int = 10,
        before_date: Optional[str] = None,
        after_date: Optional[str] = None,
    ) -> List[dict[str, Any]]:
        """Search Wayback Machine CDX for archived pages matching keywords in URLs."""
        keywords = [w.strip('"').lower() for w in query.split() if len(w.strip('"')) >= 3]
        if not keywords:
            return []

        from_ts = after_date.replace("-", "") if after_date else None
        to_ts = before_date.replace("-", "") if before_date else None

        all_results: list[dict[str, Any]] = []

        async with httpx.AsyncClient(timeout=10.0) as client:
            # CDX search on news domains
            tasks = []
            for domain in _NEWS_DOMAINS_FR[:6]:
                tasks.append(self._cdx_search_domain(
                    client, domain, keywords, from_ts, to_ts, max_results=3,
                ))
            domain_results = await asyncio.gather(*tasks, return_exceptions=True)
            for res in domain_results:
                if isinstance(res, list):
                    all_results.extend(res)

            # Wildcard *.fr/*keyword*
            if keywords and len(all_results) < max_results:
                primary = max(keywords, key=len)
                try:
                    wild = await asyncio.wait_for(
                        self._cdx_wildcard(client, primary, from_ts, to_ts, max_results=5),
                        timeout=10.0,
                    )
                    seen = {r["url"] for r in all_results}
                    all_results.extend(r for r in wild if r["url"] not in seen)
                except Exception:
                    pass

        filtered = [r for r in all_results if not _JUNK_RE.search(r.get("original_url", ""))]
        logger.info("Wayback CDX search '{}': {} results before={}", query[:40], len(filtered), before_date)
        return filtered[:max_results]

    # ==================================================================
    # Reverse discovery: fetch ALL pages from a domain, grep content
    # ==================================================================

    async def reverse_discover(
        self,
        domain: str,
        from_year: str,
        to_year: str,
        keywords: list[str],
        max_pages: int = 200,
        max_results: int = 20,
    ) -> list[dict[str, Any]]:
        """Find articles on a domain by fetching all archived pages and searching content.

        This is the most powerful strategy: it finds articles where keywords
        appear in the text but NOT in the URL.

        Args:
            domain: News domain (e.g., "courrier-picard.fr")
            from_year: Start year "2002"
            to_year: End year "2012"
            keywords: Case keywords to grep for (e.g., ["kulik", "cartigny", "elodie"])
            max_pages: Max CDX entries to check
            max_results: Max matching results to return
        """
        logger.info(
            "Wayback reverse discovery: {} ({}-{}) keywords={}",
            domain, from_year, to_year, keywords,
        )

        from_ts = f"{from_year}0101"
        to_ts = f"{to_year}1231"

        # Step 1: Get ALL unique URLs from this domain in date range
        urls = await self._cdx_list_all_urls(domain, from_ts, to_ts, limit=max_pages)
        logger.info("Wayback reverse: {} unique URLs from {} ({}-{})", len(urls), domain, from_year, to_year)

        if not urls:
            return []

        # Step 2: Fetch content and grep for keywords
        results: list[dict[str, Any]] = []
        keywords_lower = [k.lower() for k in keywords]
        sem = asyncio.Semaphore(3)  # max 3 concurrent fetches

        async def check_url(entry: dict) -> dict | None:
            async with sem:
                await asyncio.sleep(_RATE_LIMIT)  # rate limit
                text = await self.fetch_archived_text(entry["wayback_url"], max_chars=5000)
                if not text:
                    return None
                text_lower = text.lower()
                # Must match at least 2 keywords
                matches = sum(1 for kw in keywords_lower if kw in text_lower)
                if matches >= 2:
                    # Extract a title from the text (first line or first sentence)
                    title = text.split('\n')[0][:100].strip() or entry.get("title", "Archive")
                    return {
                        "url": entry["wayback_url"],
                        "original_url": entry["original_url"],
                        "title": title,
                        "snippet": text[:300],
                        "engine": "wayback_reverse",
                        "source": "wayback",
                        "published_date": entry.get("date_str", ""),
                        "keyword_matches": matches,
                    }
                return None

        # Process in batches of 10
        for i in range(0, len(urls), 10):
            if len(results) >= max_results:
                break
            batch = urls[i:i+10]
            batch_results = await asyncio.gather(
                *(check_url(u) for u in batch),
                return_exceptions=True,
            )
            for r in batch_results:
                if isinstance(r, dict):
                    results.append(r)

        logger.info(
            "Wayback reverse discovery: {} matches from {} pages on {}",
            len(results), len(urls), domain,
        )
        return results[:max_results]

    # ==================================================================
    # Common Crawl CDX search
    # ==================================================================

    async def search_common_crawl(
        self,
        query: str,
        from_year: str = "2008",
        to_year: str = "2012",
        max_results: int = 10,
    ) -> list[dict[str, Any]]:
        """Search Common Crawl indexes for URLs matching keywords."""
        keywords = [w.strip('"').lower() for w in query.split() if len(w.strip('"')) >= 3]
        if not keywords:
            return []

        # Get available CC indexes
        async with httpx.AsyncClient(timeout=10.0) as client:
            try:
                resp = await client.get(_CC_INDEX_LIST)
                indexes = resp.json()
            except Exception:
                logger.debug("Common Crawl index list unavailable")
                return []

        # Filter indexes by year range
        target_indexes = []
        for idx in indexes:
            idx_id = idx.get("id", "")
            # Extract year from "CC-MAIN-2011-12"
            m = re.search(r'(\d{4})', idx_id)
            if m:
                year = int(m.group(1))
                if int(from_year) <= year <= int(to_year):
                    target_indexes.append(idx)

        if not target_indexes:
            return []

        # Search top 3 most recent indexes in range
        results: list[dict[str, Any]] = []
        primary = max(keywords, key=len)

        async with httpx.AsyncClient(timeout=15.0) as client:
            for idx in target_indexes[:3]:
                try:
                    api_url = idx.get("cdx-api", "")
                    if not api_url:
                        continue
                    resp = await asyncio.wait_for(
                        client.get(api_url, params={
                            "url": f"*.fr/*{primary}*",
                            "output": "json",
                            "limit": max_results,
                            "filter": "=status:200",
                        }),
                        timeout=10.0,
                    )
                    for line in resp.text.strip().split("\n"):
                        if not line:
                            continue
                        try:
                            import json
                            entry = json.loads(line)
                            url = entry.get("url", "")
                            if _JUNK_RE.search(url):
                                continue
                            ts = entry.get("timestamp", "")
                            date_str = f"{ts[:4]}-{ts[4:6]}-{ts[6:8]}" if len(ts) >= 8 else ""
                            results.append({
                                "url": url,
                                "original_url": url,
                                "title": f"[CC {date_str}] {_title_from_url(url)}",
                                "snippet": f"Common Crawl {idx.get('id', '')}",
                                "engine": "commoncrawl",
                                "source": "commoncrawl",
                                "published_date": date_str,
                            })
                        except Exception:
                            continue
                except Exception as exc:
                    logger.debug("Common Crawl search failed for {}: {}", idx.get("id"), exc)

        logger.info("Common Crawl search '{}': {} results", query[:40], len(results))
        return results[:max_results]

    # ==================================================================
    # Gallica BnF search (French press OCR)
    # ==================================================================

    async def search_gallica(
        self,
        query: str,
        from_date: str | None = None,
        to_date: str | None = None,
        max_results: int = 10,
    ) -> list[dict[str, Any]]:
        """Search Gallica BnF for digitized French newspaper content via SRU API."""
        # Build CQL query
        cql_parts = [f'(gallica all "{query}")']
        cql_parts.append('(dc.type all "fascicule")')  # newspapers only
        if from_date:
            cql_parts.append(f'(dc.date >= "{from_date[:4]}")')
        if to_date:
            cql_parts.append(f'(dc.date <= "{to_date[:4]}")')

        cql = " and ".join(cql_parts)

        try:
            async with httpx.AsyncClient(timeout=15.0, headers={"User-Agent": "NEXUS/1.0"}) as client:
                resp = await client.get(_GALLICA_SRU, params={
                    "version": "1.2",
                    "operation": "searchRetrieve",
                    "query": cql,
                    "maximumRecords": max_results,
                    "startRecord": 1,
                })
                resp.raise_for_status()

                # Parse XML response
                import xml.etree.ElementTree as ET
                root = ET.fromstring(resp.text)
                ns = {"srw": "http://www.loc.gov/zing/srw/", "dc": "http://purl.org/dc/elements/1.1/"}

                results: list[dict[str, Any]] = []
                for record in root.findall(".//srw:record", ns):
                    data = record.find(".//srw:recordData", ns)
                    if data is None:
                        continue

                    title = ""
                    date = ""
                    identifier = ""
                    for dc_el in data.iter():
                        tag = dc_el.tag.split("}")[-1] if "}" in dc_el.tag else dc_el.tag
                        if tag == "title" and dc_el.text:
                            title = dc_el.text
                        elif tag == "date" and dc_el.text:
                            date = dc_el.text
                        elif tag == "identifier" and dc_el.text and "gallica.bnf.fr" in dc_el.text:
                            identifier = dc_el.text

                    if title:
                        results.append({
                            "url": identifier or f"https://gallica.bnf.fr/search?query={query}",
                            "original_url": identifier,
                            "title": f"[Gallica] {title[:80]}",
                            "snippet": f"BnF digitized press — {date}",
                            "engine": "gallica",
                            "source": "gallica",
                            "published_date": date,
                        })

                logger.info("Gallica search '{}': {} results", query[:40], len(results))
                return results

        except Exception as exc:
            logger.debug("Gallica search failed: {}", exc)
            return []

    # ==================================================================
    # Availability check for known URLs
    # ==================================================================

    async def check_archived_version(
        self,
        url: str,
        before_date: str | None = None,
    ) -> dict[str, Any] | None:
        """Check if an archived version of a URL exists before a given date."""
        params: dict[str, Any] = {"url": url}
        if before_date:
            params["timestamp"] = before_date.replace("-", "")

        try:
            async with httpx.AsyncClient(timeout=8.0) as client:
                resp = await client.get(_AVAILABILITY_API, params=params)
                resp.raise_for_status()
                data = resp.json()

                snapshot = data.get("archived_snapshots", {}).get("closest")
                if snapshot and snapshot.get("available"):
                    ts = snapshot.get("timestamp", "")
                    if before_date and ts:
                        snap_date = f"{ts[:4]}-{ts[4:6]}-{ts[6:8]}" if len(ts) >= 8 else ""
                        if snap_date > before_date:
                            return None
                    return {
                        "wayback_url": snapshot.get("url", ""),
                        "timestamp": ts,
                        "status": snapshot.get("status"),
                    }
        except Exception as exc:
            logger.debug("Wayback availability failed for {}: {}", url[:60], exc)
        return None

    # ==================================================================
    # Content fetcher
    # ==================================================================

    async def fetch_archived_text(
        self,
        wayback_url: str,
        max_chars: int = 8000,
    ) -> str | None:
        """Fetch and extract clean text from an archived page."""
        try:
            async with httpx.AsyncClient(timeout=12.0, follow_redirects=True) as client:
                resp = await client.get(wayback_url)
                resp.raise_for_status()

                # trafilatura for quality extraction
                try:
                    import trafilatura
                    text = trafilatura.extract(resp.text, include_comments=False, include_tables=False)
                    if text and len(text) > 50:
                        return text[:max_chars]
                except ImportError:
                    pass

                # Fallback: regex HTML strip
                text = re.sub(r'<script[^>]*>.*?</script>', ' ', resp.text, flags=re.DOTALL)
                text = re.sub(r'<style[^>]*>.*?</style>', ' ', text, flags=re.DOTALL)
                text = re.sub(r'<[^>]+>', ' ', text)
                text = re.sub(r'\s+', ' ', text).strip()
                return text[:max_chars] if len(text) > 50 else None
        except Exception as exc:
            logger.debug("Wayback fetch failed for {}: {}", wayback_url[:60], exc)
            return None

    # ==================================================================
    # Private CDX helpers
    # ==================================================================

    async def _cdx_search_domain(
        self, client: httpx.AsyncClient,
        domain: str, keywords: list[str],
        from_ts: str | None, to_ts: str | None,
        max_results: int = 3,
    ) -> list[dict[str, Any]]:
        """CDX search: find URLs containing keywords on a specific domain."""
        params: dict[str, Any] = {
            "url": f"{domain}/*",
            "matchType": "prefix",
            "output": "json",
            "limit": max_results * 5,
            "fl": "timestamp,original,statuscode,mimetype",
            "filter": ["statuscode:200", "mimetype:text/html"],
            "collapse": "urlkey",
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
        results = []
        for row in rows[1:]:
            if len(results) >= max_results:
                break
            entry = dict(zip(header, row))
            url = entry.get("original", "")
            if not any(kw in url.lower() for kw in keywords):
                continue
            if _JUNK_RE.search(url):
                continue
            ts = entry.get("timestamp", "")
            date_str = f"{ts[:4]}-{ts[4:6]}-{ts[6:8]}" if len(ts) >= 8 else ""
            results.append({
                "url": f"{_WAYBACK_PREFIX}/{ts}/{url}",
                "original_url": url,
                "title": f"[Archive {date_str}] {_title_from_url(url)}",
                "snippet": f"Archived page from {domain} ({date_str})",
                "engine": "wayback",
                "source": "wayback",
                "published_date": date_str,
            })
        return results

    async def _cdx_wildcard(
        self, client: httpx.AsyncClient,
        keyword: str,
        from_ts: str | None, to_ts: str | None,
        max_results: int = 5,
    ) -> list[dict[str, Any]]:
        """CDX wildcard search across *.fr domains."""
        params: dict[str, Any] = {
            "url": f"*.fr/*{keyword}*",
            "matchType": "domain",
            "output": "json",
            "limit": max_results * 3,
            "fl": "timestamp,original,statuscode,mimetype",
            "filter": ["statuscode:200", "mimetype:text/html"],
            "collapse": "urlkey",
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
        results = []
        for row in rows[1:]:
            if len(results) >= max_results:
                break
            entry = dict(zip(header, row))
            url = entry.get("original", "")
            if _JUNK_RE.search(url):
                continue
            ts = entry.get("timestamp", "")
            date_str = f"{ts[:4]}-{ts[4:6]}-{ts[6:8]}" if len(ts) >= 8 else ""
            results.append({
                "url": f"{_WAYBACK_PREFIX}/{ts}/{url}",
                "original_url": url,
                "title": f"[Archive {date_str}] {_title_from_url(url)}",
                "snippet": f"Archived {date_str}",
                "engine": "wayback",
                "source": "wayback",
                "published_date": date_str,
            })
        return results

    async def _cdx_list_all_urls(
        self,
        domain: str,
        from_ts: str,
        to_ts: str,
        limit: int = 500,
    ) -> list[dict[str, Any]]:
        """Get ALL unique archived HTML URLs from a domain in a date range."""
        params: dict[str, Any] = {
            "url": f"{domain}/*",
            "matchType": "prefix",
            "output": "json",
            "limit": limit,
            "fl": "timestamp,original,statuscode,mimetype",
            "filter": ["statuscode:200", "mimetype:text/html"],
            "collapse": "urlkey",
            "from": from_ts,
            "to": to_ts,
        }

        try:
            async with httpx.AsyncClient(timeout=20.0) as client:
                resp = await client.get(_CDX_API, params=params)
                resp.raise_for_status()
                rows = resp.json()

            if not rows or len(rows) < 2:
                return []

            header = rows[0]
            results = []
            for row in rows[1:]:
                entry = dict(zip(header, row))
                url = entry.get("original", "")
                if _JUNK_RE.search(url):
                    continue
                ts = entry.get("timestamp", "")
                date_str = f"{ts[:4]}-{ts[4:6]}-{ts[6:8]}" if len(ts) >= 8 else ""
                results.append({
                    "wayback_url": f"{_WAYBACK_PREFIX}/{ts}/{url}",
                    "original_url": url,
                    "title": f"[Archive {date_str}] {_title_from_url(url)}",
                    "date_str": date_str,
                    "timestamp": ts,
                })
            return results

        except Exception as exc:
            logger.debug("CDX list all URLs failed for {}: {}", domain, exc)
            return []


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _title_from_url(url: str) -> str:
    """Extract a human-readable title from a URL path."""
    path = url.rstrip("/").split("/")[-1]
    path = re.sub(r'\.\w{2,5}$', '', path)
    title = path.replace("-", " ").replace("_", " ").strip()
    return title[:80] if title else url.split("/")[2] if len(url.split("/")) > 2 else "Archive"
