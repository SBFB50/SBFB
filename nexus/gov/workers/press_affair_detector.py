"""
NEXUS GOV -- Press Affair Detector.

Analyzes press articles to detect NEW judicial affairs not yet in the database.
Pre-filters on judicial keywords, then uses LLM for structured extraction.
Uses IdentityResolver for politician matching and dedup against existing affairs.

Subscription: GOV_PRESS_ADDED
"""

from __future__ import annotations

import json
from typing import Any

from loguru import logger

from nexus.engine import NexusEvent, ReactiveWorker
from nexus.gov.events import GovEventType
from nexus.gov.identity import IdentityResolver, normalize_name, compute_similarity


# Judicial keywords for pre-filtering (lowercase, no accents needed -- we normalize)
_JUDICIAL_KEYWORDS = [
    "mis en examen", "enquete", "garde a vue", "perquisition",
    "condamne", "proces", "tribunal", "affaire", "scandale",
    "corruption", "fraude", "detournement",
]


class GovPressAffairDetector(ReactiveWorker):
    name = "gov_press_affair_detector"
    subscriptions = [GovEventType.GOV_PRESS_ADDED]

    def __init__(self, bus: Any, db: Any, router: Any = None) -> None:
        super().__init__(bus)
        self._db = db
        self._router = router
        self._resolver = IdentityResolver(db)
        self._processed: set[str] = set()  # Idempotency guard

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        if not self._router:
            return []

        article_id = event.payload.get("article_id", "")
        if not article_id:
            return []

        # Idempotency
        if article_id in self._processed:
            return []
        self._processed.add(article_id)
        self._trim_cache()

        # Fetch article
        article = await self._db.get_press_article(article_id)
        if not article:
            return []

        title = article.get("title", "")
        summary = article.get("summary", "") or ""
        content = summary or title

        if not content:
            return []

        # Pre-filter: check for judicial keywords
        content_lower = content.lower()
        title_lower = title.lower()
        search_text = f"{title_lower} {content_lower}"

        if not any(kw in search_text for kw in _JUDICIAL_KEYWORDS):
            return []

        logger.debug(
            "Judicial keyword detected in article '{}', sending to LLM",
            title[:50],
        )

        # LLM structured extraction
        from nexus.engine import TaskType

        truncated_content = content[:2000]
        prompt = (
            "Analyse cet article de presse. S'il mentionne une affaire judiciaire "
            "impliquant un politicien, extrais:\n"
            "- politician_name: nom du politicien\n"
            "- title: titre court de l'affaire\n"
            "- category: categorie (corruption, fraude, abus_de_bien, violence, "
            "conflit_interet, autre)\n"
            "- status: statut (enquete, mis_en_examen, proces, condamne, relaxe)\n"
            "- description: resume factuel en 1-2 phrases (presomption d'innocence)\n\n"
            'Reponds en JSON strict. Si pas d\'affaire judiciaire, reponds: {"affair": false}\n\n'
            f"Article: {title}\n{truncated_content}"
        )

        try:
            result = await self._router.route(TaskType.SUMMARIZE, prompt)
            result_text = result.strip() if isinstance(result, str) else ""

            parsed = self._parse_llm_json(result_text)
            if not parsed:
                return []

            # Check if LLM found no affair
            if parsed.get("affair") is False or not parsed.get("politician_name"):
                return []

            politician_name = str(parsed.get("politician_name", "")).strip()
            affair_title = str(parsed.get("title", "")).strip()
            category = str(parsed.get("category", "autre")).strip()
            status = str(parsed.get("status", "enquete")).strip()
            description = str(parsed.get("description", "")).strip()

            if not politician_name or not affair_title:
                return []

            # Enforce presomption d'innocence
            if "presomption" not in description.lower():
                description = f"{description} Presomption d'innocence s'applique."

            # Normalize category values
            valid_categories = {
                "corruption", "fraude", "abus_de_bien", "violence",
                "conflit_interet", "autre",
            }
            if category not in valid_categories:
                category = "autre"

            # Normalize status values
            valid_statuses = {
                "enquete", "mis_en_examen", "proces", "condamne", "relaxe",
            }
            if status not in valid_statuses:
                status = "enquete"

            # Resolve politician via IdentityResolver
            match = await self._resolver.resolve(
                politician_name,
                source="press_affair_detector",
                external_id=f"press_{article_id}",
            )

            if not match or match.get("action") == "none":
                logger.debug(
                    "Could not resolve politician '{}' from article '{}'",
                    politician_name, title[:40],
                )
                return []

            politician_id = match["politician_id"]

            # Dedup: check existing affairs for this politician
            existing_affairs = await self._db.list_affairs_by_politician(politician_id)
            affair_title_lower = affair_title.lower().strip()

            for existing in existing_affairs:
                existing_title = (existing.get("title", "") or "").lower().strip()
                # Exact match
                if existing_title == affair_title_lower:
                    logger.debug(
                        "Affair '{}' already exists for politician {}",
                        affair_title[:40], politician_id[:8],
                    )
                    return []
                # Fuzzy match (Jaro-Winkler > 0.85 = likely same affair)
                if compute_similarity(affair_title, existing.get("title", "")) > 0.85:
                    logger.debug(
                        "Affair '{}' fuzzy-matches existing '{}' for {}",
                        affair_title[:30],
                        existing.get("title", "")[:30],
                        politician_id[:8],
                    )
                    return []

            # Create new affair
            source_url = article.get("url", "")
            try:
                new_affair = await self._db.create_affair(
                    politician_id=politician_id,
                    title=affair_title[:500],
                    description=description[:2000],
                    status=status,
                    category=category,
                    source_url=str(source_url) if source_url else None,
                )

                logger.info(
                    "New affair detected: '{}' for politician {} (from press)",
                    affair_title[:50], politician_id[:8],
                )

                return [
                    NexusEvent(
                        event_type=GovEventType.GOV_AFFAIR_ADDED,
                        case_id="gov",
                        payload={
                            "affair_id": new_affair["id"],
                            "politician_id": politician_id,
                            "title": affair_title,
                            "category": category,
                            "status": status,
                            "source": "press_affair_detector",
                            "article_id": article_id,
                        },
                        source_worker=self.name,
                        parent_event_id=event.event_id,
                    )
                ]

            except Exception as exc:
                logger.debug(
                    "Failed to create affair '{}': {}",
                    affair_title[:40], exc,
                )
                return []

        except Exception as exc:
            logger.debug("Press affair detection failed for '{}': {}", title[:40], exc)
            return []

    def _parse_llm_json(self, text: str) -> dict | None:
        """Try to parse JSON from LLM output, handling common formatting issues."""
        if not text:
            return None

        # Strip markdown code fences if present
        cleaned = text.strip()
        if cleaned.startswith("```"):
            lines = cleaned.split("\n")
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

    def _trim_cache(self) -> None:
        """Keep the idempotency cache bounded."""
        if len(self._processed) > 10000:
            self._processed = set(list(self._processed)[-5000:])
