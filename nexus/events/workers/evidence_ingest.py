"""
NEXUS -- EvidenceIngestWorker.

Subscribes to MONITORING_RESULT events, filters by relevance >= 50,
and calls EvidenceProcessor.process_text_input to ingest the result
as new evidence.  Emits EVIDENCE_ADDED on success.
"""

from __future__ import annotations

import logging
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)

_RELEVANCE_THRESHOLD = 70


class EvidenceIngestWorker(ReactiveWorker):
    """Ingests high-relevance monitoring results as evidence."""

    name = "evidence_ingest"
    subscriptions = [EventType.MONITORING_RESULT]

    def __init__(
        self,
        bus: EventBus,
        evidence_processor: Any,
    ) -> None:
        super().__init__(bus)
        self._processor = evidence_processor

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        payload = event.payload
        relevance = payload.get("relevance_score", 0)

        if relevance < _RELEVANCE_THRESHOLD:
            logger.debug(
                "EvidenceIngest: skipping result with relevance %s (< %s)",
                relevance, _RELEVANCE_THRESHOLD,
            )
            return []

        title = payload.get("title", "Monitoring result")
        url = payload.get("url", payload.get("source", "monitoring"))
        snippet = payload.get("snippet", "") or payload.get("raw_text", "")

        if not snippet.strip():
            logger.debug("EvidenceIngest: empty text, skipping")
            return []

        # Fetch full page content when URL is available
        text = snippet
        if url and url.startswith("http"):
            try:
                import trafilatura
                downloaded = trafilatura.fetch_url(url)
                if downloaded:
                    extracted = trafilatura.extract(
                        downloaded,
                        favor_precision=True,
                        include_comments=False,
                        include_tables=False,
                        include_links=False,
                    )
                    if extracted and len(extracted) > len(snippet):
                        text = extracted[:8000]  # Cap at 8K chars
                        logger.info("EvidenceIngest: fetched full page (%d chars) for '%s'", len(text), title[:40])
            except ImportError:
                pass  # trafilatura not installed, use snippet
            except Exception as exc:
                logger.debug("EvidenceIngest: full page fetch failed for %s: %s", url[:50], exc)

        # Arquivo.pt: fetch full text via dedicated API
        if not text or len(text) < 100:
            text_url = payload.get("text_url")
            if text_url:
                try:
                    from nexus.monitoring.arquivo_monitor import ArquivoMonitor
                    arquivo = ArquivoMonitor()
                    full_text = await arquivo.fetch_full_text(text_url)
                    if full_text and len(full_text) > len(snippet):
                        text = full_text[:8000]
                        logger.info("EvidenceIngest: fetched Arquivo.pt full text (%d chars)", len(text))
                except Exception as exc:
                    logger.debug("EvidenceIngest: Arquivo.pt text fetch failed: %s", exc)

        # Content quality gate (2 layers):
        # Layer 1: jusText — reject pages without real article content
        # Layer 2: entity keywords — reject articles about wrong subject
        if len(text) > 50:
            try:
                import justext
                paragraphs = justext.justext(text, justext.get_stoplist("French"))
                good = [p for p in paragraphs if p.class_type == "good"]
                good_len = sum(len(p.text) for p in good)
                if good_len < 200:
                    logger.info(
                        "EvidenceIngest: REJECTED '%s' — not article content (good_text=%d chars)",
                        title[:40], good_len,
                    )
                    return []
            except Exception:
                pass

            # Layer 2: case entity keyword check
            try:
                from nexus.db.sqlite_db import get_db, Database
                async with get_db() as conn:
                    db = Database(conn)
                    case_ents = await db.list_entities_by_case(event.case_id)
                    keywords = []
                    for e in case_ents:
                        if e.get("entity_type") in ("person", "location"):
                            for part in e["name"].lower().split():
                                if len(part) >= 3:
                                    keywords.append(part)
                    keywords = list(dict.fromkeys(keywords))[:20]
                    if keywords:
                        text_lower = text.lower()
                        matches = sum(1 for kw in keywords if kw in text_lower)
                        if matches < 2:
                            logger.info(
                                "EvidenceIngest: REJECTED '%s' — off-topic (%d/%d case keywords)",
                                title[:40], matches, len(keywords),
                            )
                            return []
            except Exception:
                pass

        logger.info(
            "EvidenceIngest: ingesting '%s' (relevance=%s, %d chars) for case %s",
            title[:60], relevance, len(text), event.case_id,
        )

        evidence = await self._processor.process_text_input(
            case_id=event.case_id,
            title=title,
            text=text,
            source=url,
        )

        return [NexusEvent(
            event_type=EventType.EVIDENCE_ADDED,
            case_id=event.case_id,
            payload={
                "evidence_id": evidence.id,
                "title": evidence.title,
                "evidence_type": evidence.evidence_type,
            },
            source_worker=self.name,
            parent_event_id=event.event_id,
        )]
