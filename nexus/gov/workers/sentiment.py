"""
NEXUS GOV -- Sentiment Analyzer.

Analyzes press article tone per politician using LLM.
Tracks sentiment evolution over time.
"""

from __future__ import annotations

from typing import Any

from loguru import logger

from nexus.engine import NexusEvent, ReactiveWorker
from nexus.gov.events import GovEventType


class GovSentimentAnalyzer(ReactiveWorker):
    name = "gov_sentiment"
    subscriptions = [GovEventType.GOV_PRESS_ADDED]

    def __init__(self, bus: Any, db: Any, router: Any = None) -> None:
        super().__init__(bus)
        self._db = db
        self._router = router

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        if not self._router:
            return []

        article_id = event.payload.get("article_id", "")
        title = event.payload.get("title", "")

        if not article_id or not title:
            return []

        # Simple LLM sentiment analysis
        from nexus.engine import TaskType

        prompt = (
            "Analyse le ton de cet article de presse politique francais.\n"
            f"Titre: {title}\n\n"
            "Reponds UNIQUEMENT par un seul mot: positive, negative, ou neutral."
        )

        try:
            result = await self._router.route(TaskType.SUMMARIZE, prompt)
            sentiment = "neutral"
            result_lower = result.lower().strip() if isinstance(result, str) else ""
            if "positive" in result_lower or "positif" in result_lower:
                sentiment = "positive"
            elif (
                "negative" in result_lower
                or "negatif" in result_lower
                or "négatif" in result_lower
            ):
                sentiment = "negative"

            # Update press article with sentiment
            from nexus.engine import get_db
            from nexus.gov.db import GovernmentDatabase

            async with get_db() as conn:
                db = GovernmentDatabase(conn)
                await conn.execute(
                    "UPDATE gov_press SET sentiment = ? WHERE id = ?",
                    (sentiment, article_id),
                )
                await conn.commit()

            logger.debug("Sentiment for '{}': {}", title[:40], sentiment)

            return [
                NexusEvent(
                    event_type=GovEventType.GOV_SENTIMENT_ANALYZED,
                    case_id="gov",
                    payload={
                        "article_id": article_id,
                        "sentiment": sentiment,
                        "title": title,
                    },
                    source_worker=self.name,
                    parent_event_id=event.event_id,
                )
            ]
        except Exception as exc:
            logger.debug("Sentiment analysis failed: {}", exc)
            return []
