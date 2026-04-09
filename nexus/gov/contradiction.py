"""
NEXUS GOV -- Political Contradiction Detector.

Detects factual contradictions between a politician's positions
(votes vs declarations, statements over time) using LLM analysis.

Follows the same pattern as ``contradiction_detector.py`` but
operates on the government monitoring tables (gov_positions)
rather than case evidence.

Usage::

    from nexus.gov.db import GovernmentDatabase
    from nexus.llm.router import LLMRouter

    detector = PoliticalContradictionDetector(gov_db, router)
    new_contradictions = await detector.detect_all()
"""

from __future__ import annotations

from collections import defaultdict
from typing import Any

from loguru import logger

from nexus.config import settings
from nexus.llm.parsers import parse_json_safe
from nexus.llm.prompts import POLITICAL_CONTRADICTION_PROMPT
from nexus.llm.router import LLMRouter, TaskType


class PoliticalContradictionDetector:
    """Detect contradictions in politician positions using LLM analysis."""

    def __init__(self, gov_db: Any, router: LLMRouter) -> None:
        self._db = gov_db
        self._router = router

    # ==================================================================
    # detect_all
    # ==================================================================

    async def detect_all(
        self,
        politician_id: str | None = None,
    ) -> list[dict[str, Any]]:
        """Detect contradictions across all positions.

        Steps:
          1. Load positions grouped by politician + subject
          2. For each group with 2+ positions: find pairs to compare
          3. Send pairs to LLM with POLITICAL_CONTRADICTION_PROMPT
          4. Parse JSON response, store new contradictions in DB
          5. Return list of new contradictions found

        Args:
            politician_id: If provided, restrict analysis to this
                politician only.

        Returns:
            List of newly detected contradiction dicts.
        """
        logger.info(
            "Detecting political contradictions{}",
            f" for politician {politician_id}" if politician_id else " (all)",
        )

        # 1. Load positions
        positions = await self._db.list_positions(politician_id=politician_id)
        if len(positions) < 2:
            logger.info("Less than 2 positions — no contradiction detection possible")
            return []

        # 2. Group by (politician_id, subject) and find comparable pairs
        pairs = self._find_comparable_pairs(positions)
        if not pairs:
            logger.info("No comparable position pairs found")
            return []

        logger.info("Analysing {} position pairs for contradictions", len(pairs))

        # Load existing contradictions to skip duplicates
        existing_contradictions = await self._db.list_contradictions(
            politician_id=politician_id,
        )
        existing_pairs: set[tuple[str, str]] = set()
        for c in existing_contradictions:
            pair = tuple(sorted([c.get("position_a_id", ""), c.get("position_b_id", "")]))
            existing_pairs.add(pair)

        # 3. Analyse each pair
        new_contradictions: list[dict[str, Any]] = []

        for pos_a, pos_b in pairs:
            # Skip if contradiction already exists for this pair
            pair_key = tuple(sorted([pos_a.get("id", ""), pos_b.get("id", "")]))
            if pair_key in existing_pairs:
                logger.debug(
                    "Skipping already-analysed pair ({}, {})",
                    pair_key[0][:8],
                    pair_key[1][:8],
                )
                continue

            try:
                found = await self._analyze_pair(pos_a, pos_b)
                for contradiction in found:
                    # 4. Store in DB
                    try:
                        await self._db.create_contradiction({
                            "politician_id": pos_a["politician_id"],
                            "position_a_id": pos_a["id"],
                            "position_b_id": pos_b["id"],
                            "subject": pos_a.get("subject", ""),
                            "description": contradiction.get("description", ""),
                            "severity": contradiction.get("severity", "medium"),
                        })
                        new_contradictions.append(contradiction)
                        existing_pairs.add(pair_key)
                    except Exception as exc:
                        logger.warning(
                            "Failed to store contradiction for pair ({}, {}): {}",
                            pair_key[0][:8],
                            pair_key[1][:8],
                            exc,
                        )

            except Exception as exc:
                logger.error(
                    "Failed to analyse pair ({}, {}): {}",
                    pos_a.get("id", "?")[:8],
                    pos_b.get("id", "?")[:8],
                    exc,
                )

        logger.info(
            "Political contradiction detection complete: {} new contradictions",
            len(new_contradictions),
        )
        return new_contradictions

    # ==================================================================
    # _find_comparable_pairs
    # ==================================================================

    def _find_comparable_pairs(
        self,
        positions: list[dict[str, Any]],
    ) -> list[tuple[dict[str, Any], dict[str, Any]]]:
        """Find position pairs worth comparing.

        Groups positions by (politician_id, subject). For each group
        with 2+ entries, creates pairs sorted by date (earliest vs
        latest). Limits total pairs to ``settings.gov_contradiction_max_pairs``.
        """
        max_pairs = getattr(settings, "gov_contradiction_max_pairs", 30)

        # Group by (politician_id, subject)
        groups: dict[tuple[str, str], list[dict[str, Any]]] = defaultdict(list)
        for pos in positions:
            key = (pos.get("politician_id", ""), pos.get("subject", ""))
            groups[key].append(pos)

        pairs: list[tuple[dict[str, Any], dict[str, Any]]] = []

        for (_pid, _subj), group in groups.items():
            if len(group) < 2:
                continue

            # Sort by date (None dates go last)
            sorted_group = sorted(
                group,
                key=lambda p: p.get("date") or "9999-99-99",
            )

            # Compare earliest with latest, and also consecutive pairs
            # if the group is large enough
            if len(sorted_group) == 2:
                pairs.append((sorted_group[0], sorted_group[1]))
            else:
                # Earliest vs latest
                pairs.append((sorted_group[0], sorted_group[-1]))
                # Consecutive pairs for richer analysis
                for i in range(len(sorted_group) - 1):
                    pair = (sorted_group[i], sorted_group[i + 1])
                    if pair not in pairs:
                        pairs.append(pair)

            if len(pairs) >= max_pairs:
                break

        # Enforce hard limit
        if len(pairs) > max_pairs:
            logger.info(
                "Limiting from {} to {} position pairs",
                len(pairs),
                max_pairs,
            )
            pairs = pairs[:max_pairs]

        return pairs

    # ==================================================================
    # _analyze_pair
    # ==================================================================

    async def _analyze_pair(
        self,
        pos_a: dict[str, Any],
        pos_b: dict[str, Any],
    ) -> list[dict[str, Any]]:
        """Send a pair to LLM for contradiction analysis.

        Returns list of contradiction dicts (usually 0 or 1).
        """
        prompt = POLITICAL_CONTRADICTION_PROMPT.format(
            date_a=pos_a.get("date", "date inconnue"),
            type_a=pos_a.get("position_type", "inconnu"),
            subject=pos_a.get("subject", "N/A"),
            text_a=pos_a.get("position_text", "(pas de contenu)"),
            source_a=pos_a.get("source_url", "N/A"),
            date_b=pos_b.get("date", "date inconnue"),
            type_b=pos_b.get("position_type", "inconnu"),
            text_b=pos_b.get("position_text", "(pas de contenu)"),
            source_b=pos_b.get("source_url", "N/A"),
        )

        raw = await self._router.route_json(TaskType.CONTRADICTION_DETECTION, prompt)

        if not raw or "contradictions" not in raw:
            return []

        contradictions = raw["contradictions"]
        if not isinstance(contradictions, list):
            return []

        # Filter and normalise results
        results: list[dict[str, Any]] = []
        for c in contradictions:
            if not isinstance(c, dict):
                continue
            description = c.get("description", "").strip()
            if not description:
                continue

            severity = c.get("severity", "medium").lower()
            if severity not in ("low", "medium", "high"):
                severity = "medium"

            results.append({
                "politician_id": pos_a.get("politician_id", ""),
                "position_a_id": pos_a.get("id", ""),
                "position_b_id": pos_b.get("id", ""),
                "subject": pos_a.get("subject", ""),
                "description": description,
                "severity": severity,
            })

        return results
