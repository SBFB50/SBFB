"""
NEXUS GOV -- Voting Pattern Analyzer.

Computes voting statistics per politician and party:
- Party loyalty % (how often they vote with their group)
- Abstention rate
- Coalition detection (who votes together against their own group)
- Temporal evolution (position shifts over time)
- Coherence score (positions vs contradictions ratio)

Runs weekly. Pure math, no LLM needed.
"""

from __future__ import annotations

import json
from collections import defaultdict
from datetime import datetime, timezone
from typing import Any

from loguru import logger

from nexus.engine import NexusEvent, ReactiveWorker, get_db, _row_to_dict
from nexus.gov.events import GovEventType


def _now_iso() -> str:
    """Return current UTC time as ISO-8601 string."""
    return datetime.now(timezone.utc).isoformat()


class GovVotingPatternAnalyzer(ReactiveWorker):
    name = "gov_voting_pattern"
    subscriptions = [GovEventType.TICK_WEEKLY]

    def __init__(self, bus: Any, db: Any) -> None:
        super().__init__(bus)
        self._db = db

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    async def _build_scrutin_index(self) -> dict[tuple[str, str], dict[str, list[str]]]:
        """Build an index of (subject, date) -> {stance: [politician_ids]}.

        This lets us determine the party majority stance for each vote
        without N+1 queries.
        """
        scrutin_index: dict[tuple[str, str], dict[str, list[str]]] = defaultdict(
            lambda: defaultdict(list)
        )
        async with get_db() as conn:
            cursor = await conn.execute(
                """SELECT p.politician_id, p.subject, p.date, p.stance, pol.party
                   FROM gov_positions p
                   JOIN gov_politicians pol ON p.politician_id = pol.id
                   WHERE p.position_type = 'vote'
                     AND p.stance IN ('pour', 'contre', 'abstention')"""
            )
            rows = await cursor.fetchall()
            for r in rows:
                row = _row_to_dict(r)
                key = (row["subject"], row["date"] or "")
                stance = row["stance"]
                pol_id = row["politician_id"]
                scrutin_index[key][stance].append(pol_id)

        return scrutin_index

    async def _build_party_map(self, politicians: list[dict]) -> dict[str, str]:
        """Map politician_id -> party."""
        return {p["id"]: p.get("party", "SE") for p in politicians}

    def _party_majority_stance(
        self,
        scrutin_key: tuple[str, str],
        party: str,
        party_map: dict[str, str],
        scrutin_index: dict,
    ) -> str | None:
        """Determine the majority stance of a party on a given scrutin."""
        stances = scrutin_index.get(scrutin_key, {})
        if not stances:
            return None

        party_stance_counts: dict[str, int] = defaultdict(int)
        for stance, pol_ids in stances.items():
            for pid in pol_ids:
                if party_map.get(pid) == party:
                    party_stance_counts[stance] += 1

        if not party_stance_counts:
            return None

        # Return the stance with most votes (majority)
        return max(party_stance_counts, key=party_stance_counts.get)  # type: ignore[arg-type]

    def _compute_loyalty(
        self,
        votes: list[dict],
        party: str,
        party_map: dict[str, str],
        scrutin_index: dict,
    ) -> tuple[float, int, int]:
        """Compute loyalty rate: votes aligned with party majority / total cast.

        Returns (loyalty_rate, aligned_count, total_cast).
        """
        aligned = 0
        total_cast = 0

        for vote in votes:
            stance = vote.get("stance", "")
            if stance not in ("pour", "contre"):
                continue

            total_cast += 1
            scrutin_key = (vote.get("subject", ""), vote.get("date", "") or "")
            majority = self._party_majority_stance(
                scrutin_key, party, party_map, scrutin_index
            )
            if majority and majority == stance:
                aligned += 1

        rate = round(aligned / max(total_cast, 1) * 100, 1)
        return rate, aligned, total_cast

    def _compute_loyalty_evolution(
        self,
        votes: list[dict],
        party: str,
        party_map: dict[str, str],
        scrutin_index: dict,
    ) -> list[dict]:
        """Split votes by quarter, compute loyalty per period.

        Returns list of {period, loyalty, votes_count, shift} dicts.
        Detects significant shifts (>10% change between consecutive periods).
        """
        by_quarter: dict[str, list[dict]] = defaultdict(list)

        for vote in votes:
            date_str = vote.get("date", "") or ""
            if len(date_str) >= 7:
                year = date_str[:4]
                month = int(date_str[5:7]) if date_str[5:7].isdigit() else 1
                quarter = (month - 1) // 3 + 1
                period = f"{year}-Q{quarter}"
            else:
                period = "unknown"
            by_quarter[period].append(vote)

        # Sort periods chronologically
        periods = sorted(p for p in by_quarter if p != "unknown")
        if "unknown" in by_quarter:
            periods.append("unknown")

        evolution = []
        prev_loyalty = None

        for period in periods:
            period_votes = by_quarter[period]
            rate, aligned, total_cast = self._compute_loyalty(
                period_votes, party, party_map, scrutin_index
            )
            if total_cast == 0:
                continue

            entry: dict[str, Any] = {
                "period": period,
                "loyalty": rate,
                "votes_count": total_cast,
            }

            if prev_loyalty is not None:
                shift = round(rate - prev_loyalty, 1)
                entry["shift"] = shift
                if abs(shift) > 10:
                    entry["significant_shift"] = True

            evolution.append(entry)
            prev_loyalty = rate

        return evolution

    def _compute_voting_allies(
        self,
        pol_id: str,
        party: str,
        votes: list[dict],
        party_map: dict[str, str],
        scrutin_index: dict,
    ) -> list[dict]:
        """Find top 5 politicians from OTHER parties who most frequently
        vote the same way as this politician.
        """
        ally_counts: dict[str, int] = defaultdict(int)
        total_comparisons: dict[str, int] = defaultdict(int)

        for vote in votes:
            stance = vote.get("stance", "")
            if stance not in ("pour", "contre"):
                continue

            scrutin_key = (vote.get("subject", ""), vote.get("date", "") or "")
            stances = scrutin_index.get(scrutin_key, {})

            # Find all politicians who voted the same way
            same_stance_pols = stances.get(stance, [])
            # Find all politicians who voted differently
            other_stance_pols = []
            for s, pids in stances.items():
                if s != stance:
                    other_stance_pols.extend(pids)

            all_voters = set(same_stance_pols) | set(other_stance_pols)
            all_voters.discard(pol_id)

            for other_id in all_voters:
                other_party = party_map.get(other_id, "SE")
                if other_party == party:
                    continue  # same party, skip
                total_comparisons[other_id] += 1
                if other_id in same_stance_pols:
                    ally_counts[other_id] += 1

        # Rank by agreement rate (minimum 3 shared votes to be meaningful)
        allies = []
        for other_id, count in ally_counts.items():
            total = total_comparisons.get(other_id, 0)
            if total < 3:
                continue
            rate = round(count / total * 100, 1)
            allies.append({
                "politician_id": other_id,
                "agreement_count": count,
                "total_shared_votes": total,
                "agreement_rate": rate,
            })

        allies.sort(key=lambda x: (-x["agreement_rate"], -x["agreement_count"]))
        return allies[:5]

    # ------------------------------------------------------------------
    # Main handler
    # ------------------------------------------------------------------

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []

        all_politicians = await self._db.list_politicians(limit=100_000)
        if not all_politicians:
            return []

        # Pre-compute indexes for efficient cross-referencing
        # Use ALL politicians for party_map/scrutin (reference data)
        scrutin_index = await self._build_scrutin_index()
        party_map = await self._build_party_map(all_politicians)

        # But only process the top 200 most active politicians per run
        # to avoid unbounded memory and DB pressure.
        # Over multiple weekly runs all politicians eventually get updated.
        politicians = all_politicians[:200]

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
            votes = [
                p for p in positions
                if p.get("position_type") == "vote" and p.get("stance")
            ]

            if not votes:
                continue

            total_votes = len(votes)
            pour_count = sum(1 for v in votes if v["stance"] == "pour")
            contre_count = sum(1 for v in votes if v["stance"] == "contre")
            abstention_count = sum(1 for v in votes if v["stance"] == "abstention")

            # Abstention rate
            abstention_rate = round(abstention_count / total_votes * 100, 1) if total_votes else 0

            # Party loyalty: compare with majority stance of same party members
            loyalty_rate = 0.0
            aligned_count = 0
            total_cast = 0
            if len(party_members.get(party, [])) > 1:
                loyalty_rate, aligned_count, total_cast = self._compute_loyalty(
                    votes, party, party_map, scrutin_index
                )
            elif total_votes > 0:
                # Solo party member: 100% loyalty by definition
                loyalty_rate = 100.0
                aligned_count = pour_count + contre_count
                total_cast = aligned_count

            # Temporal evolution of loyalty
            loyalty_evolution = []
            if len(party_members.get(party, [])) > 1 and total_cast >= 4:
                loyalty_evolution = self._compute_loyalty_evolution(
                    votes, party, party_map, scrutin_index
                )

            # Cross-party voting allies
            voting_allies = self._compute_voting_allies(
                pol_id, party, votes, party_map, scrutin_index
            )

            # Store computed stats in politician metadata
            stats: dict[str, Any] = {
                "total_votes": total_votes,
                "pour": pour_count,
                "contre": contre_count,
                "abstention": abstention_count,
                "abstention_rate": abstention_rate,
                "loyalty_rate": loyalty_rate,
                "loyalty_aligned": aligned_count,
                "loyalty_total_cast": total_cast,
                "last_computed": event.timestamp,
            }

            if loyalty_evolution:
                stats["loyalty_evolution"] = loyalty_evolution

            if voting_allies:
                stats["voting_allies"] = voting_allies

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

        # --- Coherence Score ---
        # For ALL politicians (not just those with votes), compute coherence
        # as: 1 - (contradictions / positions), clamped to [0, 1].
        # Produces the "42 positions, 3 contradictions" format.
        coherence_computed = 0
        for pol in politicians:
            pol_id = pol["id"]
            try:
                all_positions = await self._db.list_positions_by_politician(
                    pol_id, limit=100_000
                )
                contradictions = await self._db.list_contradictions_by_politician(pol_id)

                total_positions = len(all_positions)
                total_contradictions = len(contradictions)

                # Coherence = 1 - (contradictions / positions), clamped to [0, 1]
                if total_positions > 0:
                    coherence = round(
                        max(0.0, 1.0 - (total_contradictions / max(total_positions, 1))),
                        3,
                    )
                else:
                    coherence = None

                # Store in politician metadata
                current = await self._db.get_politician(pol_id)
                if current:
                    existing_meta = current.get("metadata") or {}
                    if isinstance(existing_meta, str):
                        try:
                            existing_meta = json.loads(existing_meta)
                        except Exception:
                            existing_meta = {}
                    existing_meta["coherence_score"] = coherence
                    existing_meta["coherence_detail"] = (
                        f"{total_positions} positions, "
                        f"{total_contradictions} contradictions"
                    )
                    existing_meta["coherence_computed_at"] = _now_iso()
                    await self._db.update_politician(pol_id, metadata=existing_meta)
                    coherence_computed += 1
            except Exception as exc:
                logger.debug(
                    "Update coherence for {}: {}",
                    pol.get("name", pol_id), exc,
                )

        if coherence_computed:
            logger.info(
                "Coherence scores computed for {} politicians", coherence_computed
            )
            output.append(NexusEvent(
                event_type=GovEventType.GOV_PATTERN_DETECTED,
                case_id="gov",
                payload={"type": "coherence_scores", "count": coherence_computed},
                source_worker=self.name,
                parent_event_id=event.event_id,
            ))

        return output
