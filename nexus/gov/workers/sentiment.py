"""
NEXUS GOV -- Sentiment Analyzer.

Analyzes press article tone per politician using LLM.
Extracts sentiment, tone, subjects, and politician mentions.
Tracks sentiment evolution over time with per-politician aggregation.
"""

from __future__ import annotations

import json
from typing import Any

from loguru import logger

from nexus.engine import NexusEvent, ReactiveWorker
from nexus.gov.events import GovEventType


def _now_iso() -> str:
    """Return current UTC timestamp in ISO format."""
    from datetime import datetime, timezone
    return datetime.now(timezone.utc).isoformat()


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
        politicians = event.payload.get("politicians", [])

        if not article_id or not title:
            return []

        # Fetch article summary for richer analysis
        summary = ""
        try:
            article = await self._db.get_press_article(article_id)
            if article:
                summary = article.get("summary", "") or ""
        except Exception:
            pass

        from nexus.engine import TaskType

        prompt = (
            "Tu es un analyste politique. Analyse le ton de cet article de presse francais.\n\n"
            f"Titre: {title}\n"
        )
        if summary:
            prompt += f"Resume: {summary[:500]}\n"
        prompt += (
            "\nReponds en JSON strict (pas de markdown, pas de commentaire) avec ce format:\n"
            "{\n"
            '  "sentiment": "positive" | "negative" | "neutral",\n'
            '  "tone": "critical" | "favorable" | "factual" | "satirical",\n'
            '  "subjects": ["sujet1", "sujet2"],\n'
            '  "politicians_mentioned": ["nom1", "nom2"]\n'
            "}\n"
            "Regles:\n"
            "- sentiment: le sentiment general de l'article\n"
            "- tone: le ton editorial (critical = attaque/denonciation, "
            "favorable = eloge/soutien, factual = neutre/informatif, satirical = moquerie/ironie)\n"
            "- subjects: les 1-3 sujets principaux (ex: retraites, immigration, budget)\n"
            "- politicians_mentioned: les noms de politiciens cites\n"
        )

        try:
            result = await self._router.route(TaskType.SUMMARIZE, prompt)
            result_text = result.strip() if isinstance(result, str) else ""

            # Parse structured response
            sentiment = "neutral"
            tone = "factual"
            subjects: list[str] = []
            mentioned_names: list[str] = []

            parsed = self._parse_llm_json(result_text)
            if parsed:
                sentiment = parsed.get("sentiment", "neutral")
                tone = parsed.get("tone", "factual")
                subjects = parsed.get("subjects", [])
                mentioned_names = parsed.get("politicians_mentioned", [])

                # Normalize sentiment value
                if sentiment not in ("positive", "negative", "neutral"):
                    sentiment = self._classify_sentiment_fallback(sentiment)
                # Normalize tone value
                valid_tones = ("critical", "favorable", "factual", "satirical")
                if tone not in valid_tones:
                    tone = "factual"
            else:
                # Fallback: extract sentiment from raw text
                sentiment = self._classify_sentiment_fallback(result_text)

            # Update press article with enriched sentiment data
            from nexus.engine import get_db
            from nexus.gov.db import GovernmentDatabase

            # Build subjects string for storage
            subjects_str = ",".join(subjects[:5]) if subjects else ""
            mentioned_str = ",".join(mentioned_names[:10]) if mentioned_names else ""

            async with get_db() as conn:
                db = GovernmentDatabase(conn)

                # Merge tone into existing metadata
                existing_meta = {}
                if article and article.get("metadata"):
                    meta_raw = article["metadata"]
                    if isinstance(meta_raw, dict):
                        existing_meta = meta_raw
                    elif isinstance(meta_raw, str):
                        try:
                            existing_meta = json.loads(meta_raw)
                        except (json.JSONDecodeError, TypeError):
                            existing_meta = {}
                existing_meta["tone"] = tone
                if mentioned_str:
                    existing_meta["politicians_llm"] = mentioned_str

                await conn.execute(
                    """UPDATE gov_press
                       SET sentiment = ?, subjects = ?, metadata = ?
                       WHERE id = ?""",
                    (
                        sentiment,
                        subjects_str or None,
                        json.dumps(existing_meta),
                        article_id,
                    ),
                )
                await conn.commit()

            logger.debug(
                "Sentiment for '{}': {} (tone={})",
                title[:40], sentiment, tone,
            )

            # Aggregate per-politician sentiment
            for pol_id in politicians:
                await self._update_politician_sentiment(pol_id)

            return [
                NexusEvent(
                    event_type=GovEventType.GOV_SENTIMENT_ANALYZED,
                    case_id="gov",
                    payload={
                        "article_id": article_id,
                        "sentiment": sentiment,
                        "tone": tone,
                        "subjects": subjects,
                        "politicians_mentioned": mentioned_names,
                        "title": title,
                    },
                    source_worker=self.name,
                    parent_event_id=event.event_id,
                )
            ]
        except Exception as exc:
            logger.debug("Sentiment analysis failed: {}", exc)
            return []

    def _parse_llm_json(self, text: str) -> dict | None:
        """Try to parse JSON from LLM output, handling common formatting issues."""
        if not text:
            return None

        # Strip markdown code fences if present
        cleaned = text.strip()
        if cleaned.startswith("```"):
            lines = cleaned.split("\n")
            # Remove first and last lines (code fences)
            lines = [l for l in lines if not l.strip().startswith("```")]
            cleaned = "\n".join(lines)

        # Try direct JSON parse
        try:
            return json.loads(cleaned)
        except json.JSONDecodeError:
            pass

        # Try to find JSON object in text
        start = cleaned.find("{")
        end = cleaned.rfind("}")
        if start != -1 and end > start:
            try:
                return json.loads(cleaned[start:end + 1])
            except json.JSONDecodeError:
                pass

        return None

    def _classify_sentiment_fallback(self, text: str) -> str:
        """Fallback sentiment classification from raw text."""
        text_lower = text.lower().strip()
        if any(w in text_lower for w in ("positive", "positif", "favorable")):
            return "positive"
        if any(w in text_lower for w in ("negative", "negatif", "négatif", "critical", "critique")):
            return "negative"
        return "neutral"

    async def _update_politician_sentiment(self, politician_id: str) -> None:
        """Compute and store per-politician sentiment aggregation.

        Updates the politician's metadata with press sentiment stats:
        positive/negative/neutral counts, total, and ratio.
        """
        try:
            articles = await self._db.list_press_by_politician(politician_id)
            sentiments = [a.get("sentiment") for a in articles if a.get("sentiment")]
            positive = sentiments.count("positive")
            negative = sentiments.count("negative")
            neutral = sentiments.count("neutral")
            total = len(sentiments)

            if total == 0:
                return

            # Fetch current metadata to merge
            pol = await self._db.get_politician(politician_id)
            if not pol:
                return

            metadata = pol.get("metadata") or {}
            if isinstance(metadata, str):
                try:
                    metadata = json.loads(metadata)
                except (json.JSONDecodeError, TypeError):
                    metadata = {}

            metadata["press_sentiment"] = {
                "positive": positive,
                "negative": negative,
                "neutral": neutral,
                "total": total,
                "ratio": round(positive / max(total, 1), 2),
                "last_computed": _now_iso(),
            }

            await self._db.update_politician(politician_id, metadata=metadata)

            logger.debug(
                "Politician {} sentiment: +{}/-{}/~{} (ratio={:.2f})",
                politician_id[:8], positive, negative, neutral,
                positive / max(total, 1),
            )
        except Exception as exc:
            logger.debug("Politician sentiment aggregation failed for {}: {}", politician_id, exc)
