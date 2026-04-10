"""
NEXUS GOV -- Weekly Recap Generator.

Compiles a weekly summary of political activity:
- Important votes
- New contradictions detected
- New judicial affairs
- Press highlights
- Classification thematique (13 domaines)

Runs weekly. Stores recap in gov_alerts as type "recap".
"""

from __future__ import annotations

import json
from typing import Any

from loguru import logger

from nexus.engine import _new_id, _now_iso, _row_to_dict, get_db, NexusEvent, ReactiveWorker
from nexus.gov.events import GovEventType

THEMES = [
    "Securite et Justice",
    "Sante",
    "Education",
    "Economie et Finances",
    "Environnement",
    "Social et Travail",
    "Culture",
    "Defense",
    "Affaires etrangeres",
    "Agriculture",
    "Transport",
    "Numerique",
    "Institutions",
]


class GovWeeklyRecapWorker(ReactiveWorker):
    name = "gov_weekly_recap"
    subscriptions = [GovEventType.TICK_WEEKLY]

    def __init__(self, bus: Any, db: Any, router: Any = None) -> None:
        super().__init__(bus)
        self._db = db
        self._router = router

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []

        # Collect data from the past week
        stats = await self._db.get_stats()

        # Get recent contradictions, affairs, press
        async with get_db() as conn:
            cursor = await conn.execute(
                "SELECT * FROM gov_contradictions ORDER BY detected_at DESC LIMIT 20"
            )
            contradictions = [_row_to_dict(r) for r in await cursor.fetchall()]

            cursor = await conn.execute(
                "SELECT * FROM gov_affairs ORDER BY created_at DESC LIMIT 10"
            )
            affairs = [_row_to_dict(r) for r in await cursor.fetchall()]

            cursor = await conn.execute(
                "SELECT * FROM gov_press ORDER BY published_at DESC LIMIT 20"
            )
            press = [_row_to_dict(r) for r in await cursor.fetchall()]

        # Build recap text
        recap_parts = ["# Recap hebdomadaire politique\n"]
        recap_parts.append(f"Politiciens suivis: {stats.get('politicians', 0)}")
        recap_parts.append(f"Positions totales: {stats.get('positions', 0)}")
        recap_parts.append(f"Contradictions: {stats.get('contradictions', 0)}")

        if contradictions:
            recap_parts.append(
                f"\n## Contradictions recentes ({len(contradictions)})"
            )
            for c in contradictions[:5]:
                recap_parts.append(
                    f"- [{c.get('severity', '?')}] {c.get('subject', '')}: "
                    f"{c.get('description', '')[:150]}"
                )

        if affairs:
            recap_parts.append(f"\n## Affaires judiciaires ({len(affairs)})")
            for a in affairs[:5]:
                recap_parts.append(
                    f"- {a.get('title', '')[:150]} (statut: {a.get('status', '?')})"
                )

        if press:
            recap_parts.append(f"\n## Presse ({len(press)} articles)")
            for p in press[:5]:
                recap_parts.append(
                    f"- [{p.get('sentiment', '?')}] {p.get('title', '')[:100]} "
                    f"({p.get('source_name', '')})"
                )

        recap_text = "\n".join(recap_parts)

        # If LLM available, generate a summary
        if self._router and (contradictions or affairs):
            from nexus.engine import TaskType

            try:
                prompt = (
                    "Resume ce rapport hebdomadaire politique en 3-5 phrases.\n"
                    "Sois factuel et neutre. Mentionne les faits les plus importants.\n\n"
                    f"{recap_text}\n\n"
                    "RESUME:"
                )
                summary = await self._router.route(TaskType.SUMMARIZE, prompt)
                if summary:
                    recap_text = f"{summary}\n\n---\n\n{recap_text}"
            except Exception as exc:
                logger.debug("Recap LLM summary failed: {}", exc)

        # Store as alert
        try:
            async with get_db() as conn:
                alert_id = _new_id()
                now = _now_iso()
                await conn.execute(
                    """INSERT INTO gov_alerts
                       (id, alert_type, title, description, severity, created_at)
                       VALUES (?, ?, ?, ?, ?, ?)""",
                    (alert_id, "recap", "Recap hebdomadaire", recap_text, "info", now),
                )
                await conn.commit()

            output.append(
                NexusEvent(
                    event_type=GovEventType.GOV_ALERT_CREATED,
                    case_id="gov",
                    payload={
                        "alert_id": alert_id,
                        "alert_type": "recap",
                        "title": "Recap hebdomadaire",
                    },
                    source_worker=self.name,
                    parent_event_id=event.event_id,
                )
            )
            logger.info(
                "Weekly recap generated: {} contradictions, {} affairs, {} press",
                len(contradictions),
                len(affairs),
                len(press),
            )
        except Exception as exc:
            logger.warning("Recap storage failed: {}", exc)

        # Thematic classification of ALL unclassified positions
        if self._router:
            try:
                async with get_db() as conn:
                    cursor = await conn.execute(
                        """SELECT * FROM gov_positions
                           WHERE metadata IS NULL OR metadata NOT LIKE '%"theme"%'
                           ORDER BY created_at DESC LIMIT 20"""
                    )
                    unclassified = [_row_to_dict(r) for r in await cursor.fetchall()]

                from nexus.engine import TaskType

                classified_count = 0
                for pos in unclassified:
                    subject = pos.get("subject", "")
                    text = pos.get("position_text", "")
                    if not subject and not text:
                        continue

                    theme_prompt = (
                        "Classifie cette position politique dans UN des themes "
                        "suivants:\n"
                        f"{', '.join(THEMES)}\n\n"
                        f"Position: {subject} — {text[:200]}\n\n"
                        "Reponds UNIQUEMENT par le nom du theme."
                    )

                    try:
                        theme = await self._router.route(
                            TaskType.SUMMARIZE, theme_prompt
                        )
                        theme = theme.strip() if theme else ""
                        # Find closest matching theme
                        matched = None
                        for t in THEMES:
                            if t.lower() in theme.lower() or theme.lower() in t.lower():
                                matched = t
                                break
                        if matched:
                            meta = pos.get("metadata") or {}
                            if isinstance(meta, str):
                                try:
                                    meta = json.loads(meta)
                                except Exception:
                                    meta = {}
                            meta["theme"] = matched
                            async with get_db() as conn:
                                await conn.execute(
                                    "UPDATE gov_positions SET metadata = ? WHERE id = ?",
                                    (json.dumps(meta, ensure_ascii=False), pos["id"]),
                                )
                                await conn.commit()
                            classified_count += 1
                    except Exception as exc:
                        logger.debug("Theme classification LLM failed: {}", exc)

                if classified_count:
                    logger.info(
                        "Thematic classification: {}/{} positions classified",
                        classified_count,
                        len(unclassified),
                    )
            except Exception as exc:
                logger.debug("Thematic classification failed: {}", exc)

        return output
