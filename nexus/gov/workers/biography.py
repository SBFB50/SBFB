"""
NEXUS GOV -- Biography Generator.

Generates factual biographies for politicians using LLM.
Based on: mandates, votes, affairs, declarations, press mentions.
Runs weekly. No opinion, purely factual, sourced.
"""

from __future__ import annotations

import asyncio
import json
from datetime import datetime, timezone, timedelta
from typing import Any

from loguru import logger

from nexus.engine import NexusEvent, ReactiveWorker
from nexus.gov.events import GovEventType


class GovBiographyWorker(ReactiveWorker):
    name = "gov_biography"
    subscriptions = [GovEventType.TICK_WEEKLY]

    def __init__(self, bus: Any, db: Any, router: Any = None) -> None:
        super().__init__(bus)
        self._db = db
        self._router = router

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        if not self._router:
            return []

        from nexus.engine import TaskType

        # Get politicians needing biography update
        politicians = await self._db.list_politicians(limit=100_000)
        generated = 0

        for pol in politicians[:100]:  # Limit per run
            pol_id = pol["id"]
            name = pol["name"]

            # Check if biography already exists and is recent
            existing_meta = pol.get("metadata") or {}
            if isinstance(existing_meta, str):
                try:
                    existing_meta = json.loads(existing_meta)
                except Exception:
                    existing_meta = {}

            bio = existing_meta.get("biography", "")
            bio_date = existing_meta.get("biography_generated_at", "")

            # Skip if biography is fresh (< 7 days old)
            if bio and len(bio) > 100 and bio_date:
                try:
                    bio_ts = datetime.fromisoformat(bio_date)
                    if datetime.now(timezone.utc) - bio_ts < timedelta(days=7):
                        continue  # still fresh
                except (ValueError, TypeError):
                    pass  # regenerate if date is invalid

            # Collect data
            positions = await self._db.list_positions_by_politician(pol_id, limit=50)
            affairs = await self._db.list_affairs_by_politician(pol_id)
            declarations = await self._db.list_declarations_by_politician(pol_id)

            if not positions and not affairs:
                continue  # Not enough data

            # Build context
            context_parts = [
                f"Nom: {name}",
                f"Parti: {pol.get('party', 'N/A')}",
                f"Chambre: {pol.get('chamber', 'N/A')}",
            ]

            if pol.get("role"):
                context_parts.append(f"Role: {pol['role']}")
            if pol.get("constituency"):
                context_parts.append(f"Circonscription: {pol['constituency']}")

            if positions:
                votes = [p for p in positions if p.get("position_type") == "vote"]
                context_parts.append(f"\nVotes enregistres: {len(votes)}")
                for v in votes[:10]:
                    context_parts.append(
                        f"  - {v.get('date', '?')}: {v.get('subject', '')[:100]} "
                        f"({v.get('stance', '?')})"
                    )

            if affairs:
                context_parts.append(f"\nAffaires judiciaires: {len(affairs)}")
                for a in affairs:
                    context_parts.append(
                        f"  - {a.get('title', '')[:100]} (statut: {a.get('status', '?')})"
                    )

            if declarations:
                context_parts.append(f"\nDeclarations HATVP: {len(declarations)}")

            context = "\n".join(context_parts)

            prompt = (
                "Redige une biographie factuelle et neutre de ce politicien francais.\n"
                "Base-toi UNIQUEMENT sur les donnees fournies. Pas d'opinion, pas de jugement.\n"
                "2-3 paragraphes maximum.\n\n"
                f"DONNEES:\n{context}\n\n"
                "BIOGRAPHIE FACTUELLE:"
            )

            try:
                biography = await self._router.route(TaskType.SUMMARIZE, prompt)
                if biography and len(biography) > 50:
                    existing_meta["biography"] = biography
                    existing_meta["biography_generated_at"] = datetime.now(
                        timezone.utc
                    ).isoformat()
                    await self._db.update_politician(pol_id, metadata=existing_meta)
                    generated += 1
                    logger.debug("Biography generated for {}", name)
            except Exception as exc:
                logger.debug("Biography failed for {}: {}", name, exc)

            await asyncio.sleep(0)  # Yield for cancellation

        if generated:
            logger.info("Biographies generated: {}", generated)
        return []
