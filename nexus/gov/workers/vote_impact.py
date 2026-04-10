"""
NEXUS GOV -- Vote/Law Impact Analyzer.

For each new law, LLM explains the concrete citizen impact.
Runs daily. Processes laws without an existing impact explanation.
Stores result in law metadata and creates an impact_analysis position.
"""

from __future__ import annotations

import asyncio
import json
from typing import Any

from loguru import logger

from nexus.engine import NexusEvent, ReactiveWorker
from nexus.gov.events import GovEventType


class GovVoteImpactWorker(ReactiveWorker):
    name = "gov_vote_impact"
    subscriptions = [GovEventType.TICK_DAILY]

    def __init__(self, bus: Any, db: Any, router: Any = None) -> None:
        super().__init__(bus)
        self._db = db
        self._router = router

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        if not self._router:
            return []

        from nexus.engine import TaskType

        # Fetch recent laws
        laws = await self._db.list_laws(limit=200)
        if not laws:
            return []

        processed = 0

        for law in laws:
            # Check if already has citizen_impact in metadata
            metadata = law.get("metadata") or {}
            if isinstance(metadata, str):
                try:
                    metadata = json.loads(metadata)
                except (json.JSONDecodeError, TypeError):
                    metadata = {}

            if metadata.get("citizen_impact"):
                continue  # Already analyzed

            # Limit per run
            if processed >= 10:
                break

            law_id = law["id"]
            title = law.get("title", "")
            short_title = law.get("short_title", "")
            procedure = law.get("procedure", "N/A")
            status = law.get("status", "N/A")
            amendments_count = law.get("amendments_count", 0) or 0
            amendments_adopted = law.get("amendments_adopted", 0) or 0

            if not title:
                continue

            display_title = short_title or title

            prompt = (
                "Tu es un analyste politique. Explique en 2-3 phrases l'impact concret "
                "de cette loi sur les citoyens francais. Sois factuel et precis.\n\n"
                f"Loi: {display_title}\n"
                f"Procedure: {procedure}\n"
                f"Status: {status}\n"
                f"Amendements: {amendments_count} deposes, {amendments_adopted} adoptes\n\n"
                "Reponds UNIQUEMENT avec l'explication d'impact."
            )

            try:
                result = await self._router.route(TaskType.SUMMARIZE, prompt)
                impact_text = result.strip() if isinstance(result, str) else ""

                if not impact_text or len(impact_text) < 20:
                    continue

                # Store in law metadata
                metadata["citizen_impact"] = impact_text
                metadata["citizen_impact_generated_at"] = event.timestamp

                from nexus.engine import get_db
                from nexus.gov.db import GovernmentDatabase

                async with get_db() as conn:
                    db = GovernmentDatabase(conn)
                    await conn.execute(
                        "UPDATE gov_laws SET metadata = ? WHERE id = ?",
                        (json.dumps(metadata), law_id),
                    )
                    await conn.commit()

                # Create a position record for the impact analysis.
                # We need a politician (initiator) — use initiator_ref if available.
                initiator_ref = law.get("initiator_ref", "")
                if initiator_ref:
                    try:
                        await self._db.create_position(
                            politician_id=initiator_ref,
                            subject=display_title[:500],
                            position_type="impact_analysis",
                            position_text=impact_text[:2000],
                            source_url=law.get("source_url", "") or "",
                            source_type="law_impact",
                            date=law.get("date_initial", ""),
                            metadata={"law_id": law_id},
                        )
                    except Exception as exc:
                        # initiator_ref may not be a valid politician_id — skip
                        logger.debug(
                            "Impact position creation failed for law {}: {}",
                            law_id[:8], exc,
                        )

                processed += 1
                logger.debug(
                    "Impact analyzed for '{}': {}",
                    display_title[:40], impact_text[:60],
                )

            except Exception as exc:
                logger.debug("Vote impact analysis failed for {}: {}", law_id[:8], exc)

            await asyncio.sleep(0)  # Yield for cancellation

        if processed:
            logger.info("Vote impact analyses generated: {}", processed)

        return []
