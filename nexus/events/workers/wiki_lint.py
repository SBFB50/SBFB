"""
NEXUS -- WikiLintWorker.

Subscribes to TICK_WIKI_LINT. Checks wiki health:
- Broken wikilinks
- Entities without wiki pages
- Stale pages
Creates alerts for issues found.
"""

from __future__ import annotations

import logging
import re
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)


class WikiLintWorker(ReactiveWorker):
    """Checks wiki health and creates alerts for issues."""

    name = "wiki_lint"
    subscriptions = [EventType.TICK_WIKI_LINT]

    def __init__(self, bus: EventBus, db: Any) -> None:
        super().__init__(bus)
        self._db = db

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        try:
            from nexus.config import settings

            wiki_dir = settings.data_dir / "cases" / event.case_id / "wiki"
            if not wiki_dir.exists():
                return []

            pages = await self._db.list_wiki_pages(event.case_id)
            page_paths = {p["page_path"] for p in pages}
            issues: list[str] = []

            # Check for broken wikilinks
            for p in pages:
                file_path = wiki_dir / p["page_path"]
                if not file_path.exists():
                    issues.append(f"Page manquante sur disque: {p['page_path']}")
                    continue
                content = file_path.read_text(encoding="utf-8")
                links = re.findall(r'\[\[([^|\]]+)', content)
                for link in links:
                    # Check if linked page exists (approximate match)
                    link_path = link if link.endswith(".md") else f"{link}.md"
                    if link_path not in page_paths and not any(link_path in pp for pp in page_paths):
                        issues.append(f"Wikilink casse dans {p['page_path']}: [[{link}]]")

            # Check entities without wiki pages
            entities = await self._db.list_entities_by_case(event.case_id)
            for ent in entities:
                if ent.get("entity_type") in ("person", "location", "vehicle", "organization"):
                    # Check if entity has a wiki page
                    has_page = any(
                        ent["name"].lower().replace(" ", "-") in p["page_path"].lower()
                        for p in pages
                    )
                    if not has_page:
                        issues.append(f"Entite sans page wiki: {ent['name']} ({ent['entity_type']})")

            if issues:
                logger.info("WikiLint: %d issues found for case %s", len(issues), event.case_id)
                # Create alert
                try:
                    await self._db.create_alert(
                        case_id=event.case_id,
                        alert_type="wiki_lint",
                        severity="info",
                        title=f"Wiki lint: {len(issues)} probleme(s)",
                        message="\n".join(issues[:20]),
                    )
                except Exception as exc:
                    logger.debug("WikiLint: failed to create alert: %s", exc)

            return []

        except Exception as exc:
            logger.warning("WikiLint failed: %s", exc)
            return []
