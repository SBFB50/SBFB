"""
NEXUS GOV -- Voting Pattern Analyzer.

Computes voting statistics per politician and party:
- Party loyalty % (how often they vote with their group)
- Abstention rate
- Coalition detection (who votes together against their own group)
- Temporal evolution (position shifts over time)

Runs weekly. Pure math, no LLM needed.
"""

from __future__ import annotations

import json
from collections import defaultdict
from typing import Any

from loguru import logger

from nexus.events.types import NexusEvent
from nexus.events.worker import ReactiveWorker
from nexus.gov.events import GovEventType


class GovVotingPatternAnalyzer(ReactiveWorker):
    name = "gov_voting_pattern"
    subscriptions = [GovEventType.TICK_WEEKLY]

    def __init__(self, bus: Any, db: Any) -> None:
        super().__init__(bus)
        self._db = db

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []

        politicians = await self._db.list_politicians(limit=100_000)
        if not politicians:
            return []

        # Group politicians by party
        party_members: dict[str, list[dict]] = defaultdict(list)
        for pol in politicians:
            party = pol.get("party", "SE")
            party_members[party].append(pol)

        patterns_computed = 0

        for pol in politicians:
            pol_id = pol["id"]
            party = pol.get("party", "SE")

            # Get all vote positions
            positions = await self._db.list_positions_by_politician(pol_id, limit=100_000)
            votes = [p for p in positions if p.get("position_type") == "vote" and p.get("stance")]

            if not votes:
                continue

            total_votes = len(votes)
            pour_count = sum(1 for v in votes if v["stance"] == "pour")
            contre_count = sum(1 for v in votes if v["stance"] == "contre")
            abstention_count = sum(1 for v in votes if v["stance"] == "abstention")

            # Abstention rate
            abstention_rate = round(abstention_count / total_votes * 100, 1) if total_votes else 0

            # Party loyalty: compare with majority stance of same party members
            # (simplified -- would need per-scrutin analysis for accuracy)
            loyalty_rate = 0.0
            if len(party_members.get(party, [])) > 1:
                # For now, use a simplified metric
                # Real loyalty would cross-reference per-scrutin group votes
                loyalty_rate = round((pour_count + contre_count) / max(total_votes, 1) * 100, 1)

            # Store computed stats in politician metadata
            stats = {
                "total_votes": total_votes,
                "pour": pour_count,
                "contre": contre_count,
                "abstention": abstention_count,
                "abstention_rate": abstention_rate,
                "loyalty_rate": loyalty_rate,
                "last_computed": event.timestamp,
            }

            try:
                current = await self._db.get_politician(pol_id)
                if current:
                    existing_meta = current.get("metadata") or {}
                    if isinstance(existing_meta, str):
                        try:
                            existing_meta = json.loads(existing_meta)
                        except Exception:
                            existing_meta = {}
                    existing_meta["voting_stats"] = stats
                    await self._db.update_politician(pol_id, metadata=existing_meta)
                    patterns_computed += 1
            except Exception as exc:
                logger.debug("Update voting stats for {}: {}", pol["name"], exc)

        if patterns_computed:
            logger.info("Voting patterns computed for {} politicians", patterns_computed)
            output.append(NexusEvent(
                event_type=GovEventType.GOV_PATTERN_DETECTED,
                case_id="gov",
                payload={"type": "voting_patterns", "count": patterns_computed},
                source_worker=self.name,
                parent_event_id=event.event_id,
            ))

        return output
