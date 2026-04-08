"""
NEXUS -- Contradiction Detector.

Detecte les contradictions entre preuves, temoignages et hypotheses
en utilisant deepseek-r1 14B (raisonnement chain-of-thought).

Le detecteur travaille par paires pertinentes pour eviter l'explosion
combinatoire: seules les preuves mentionnant des entites communes sont
comparees.

Usage::

    async with get_db() as conn:
        db = Database(conn)
        router = LLMRouter()
        detector = ContradictionDetector(db, router)
        contradictions = await detector.detect_contradictions(case_id)
"""

from __future__ import annotations

from itertools import combinations
from typing import Any

from loguru import logger

from nexus.config import settings
from nexus.db.sqlite_db import Database
from nexus.llm.parsers import parse_json_safe
from nexus.llm.prompts import (
    CONTRADICTION_DETECTION_PROMPT,
    TESTIMONY_COMPARISON_PROMPT,
)
from nexus.llm.router import LLMRouter, TaskType


class ContradictionDetector:
    """Detect contradictions between evidence, testimonies and hypotheses."""

    def __init__(self, db: Database, router: LLMRouter) -> None:
        self._db = db
        self._router = router

    # ==================================================================
    # detect_contradictions
    # ==================================================================

    async def detect_contradictions(self, case_id: str) -> list[dict[str, Any]]:
        """Detect contradictions between evidence in a case.

        Steps:
          1. Load all evidence with summaries
          2. Group into relevant pairs (shared entities)
          3. [deepseek-r1] Analyse each pair with CONTRADICTION_DETECTION_PROMPT
          4. Deduplicate results
          5. Return consolidated contradiction list
        """
        logger.info("Detecting contradictions for case {}", case_id)

        # 1. Load evidence
        evidence_list = await self._db.list_evidence_by_case(case_id)
        if len(evidence_list) < 2:
            logger.info("Less than 2 evidence items for case {} — no contradictions possible", case_id)
            return []

        # 2. Group into relevant pairs
        pairs = await self._find_relevant_pairs(case_id, evidence_list)

        if not pairs:
            # Fallback: if no entity-based pairing, compare all pairs (up to a limit)
            logger.info("No entity-based pairs found, comparing all pairs (up to {})", settings.contradiction_max_fallback_pairs)
            all_pairs = list(combinations(evidence_list, 2))
            pairs = all_pairs[:settings.contradiction_max_fallback_pairs]

        logger.info("Analysing {} evidence pairs for contradictions", len(pairs))

        # 3. Analyse each pair
        all_contradictions: list[dict[str, Any]] = []
        for ev_a, ev_b in pairs:
            try:
                contradictions = await self._analyse_pair(ev_a, ev_b)
                all_contradictions.extend(contradictions)
            except Exception as exc:
                logger.error(
                    "Failed to analyse pair ({}, {}): {}",
                    ev_a["id"][:8], ev_b["id"][:8], exc,
                )

        # 4. Deduplicate
        deduped = self._deduplicate_contradictions(all_contradictions)

        logger.info(
            "Found {} contradictions ({} before dedup) for case {}",
            len(deduped), len(all_contradictions), case_id,
        )

        return deduped

    # ==================================================================
    # compare_testimonies
    # ==================================================================

    async def compare_testimonies(
        self,
        case_id: str,
        evidence_ids: list[str],
    ) -> dict[str, Any]:
        """Compare specific testimonies using deepseek-r1.

        Parameters:
            case_id: The case ID (for validation).
            evidence_ids: List of evidence IDs to compare as testimonies.

        Returns:
            {convergences: [...], divergences: [...], reliability_ranking: [...]}
        """
        if len(evidence_ids) < 2:
            raise ValueError("At least 2 evidence IDs required for testimony comparison")

        logger.info("Comparing {} testimonies for case {}", len(evidence_ids), case_id)

        # 1. Load the specified evidence items
        testimonies: list[dict[str, Any]] = []
        for eid in evidence_ids:
            ev = await self._db.get_evidence(eid)
            if ev is None:
                raise ValueError(f"Evidence not found: {eid}")
            if ev["case_id"] != case_id:
                raise ValueError(f"Evidence {eid} does not belong to case {case_id}")
            testimonies.append(ev)

        # 2. Build testimonies text
        testimonies_text = self._build_testimonies_text(testimonies)

        # 3. [deepseek-r1] Call TESTIMONY_COMPARISON_PROMPT
        prompt = TESTIMONY_COMPARISON_PROMPT.format(testimonies=testimonies_text)

        logger.info("Calling deepseek-r1 for testimony comparison")
        raw_response = await self._router.route(TaskType.TESTIMONY_COMPARISON, prompt)

        # 4. Parse response
        parsed = parse_json_safe(raw_response)
        if not parsed:
            logger.error("Failed to parse testimony comparison response")
            return {
                "convergences": [],
                "divergences": [],
                "reliability_ranking": [],
                "error": "Echec du parsing de la reponse LLM",
            }

        # Normalise the output
        result = {
            "convergences": parsed.get("convergences", []),
            "divergences": parsed.get("divergences", []),
            "reliability_ranking": [],
            "synthesis": parsed.get("synthesis", ""),
        }

        # Build reliability ranking from reliability_scores if present
        reliability_scores = parsed.get("reliability_scores", [])
        if isinstance(reliability_scores, list):
            result["reliability_ranking"] = sorted(
                reliability_scores,
                key=lambda x: x.get("score", 0) if isinstance(x, dict) else 0,
                reverse=True,
            )

        return result

    # ==================================================================
    # check_hypothesis_consistency
    # ==================================================================

    async def check_hypothesis_consistency(self, case_id: str) -> list[dict[str, Any]]:
        """Check that hypotheses do not contradict each other.

        Compares active hypotheses pairwise to find logical conflicts.
        """
        hypotheses = await self._db.list_hypotheses_by_case(case_id, status="active")

        if len(hypotheses) < 2:
            logger.info("Less than 2 active hypotheses — no consistency check needed")
            return []

        pairs = list(combinations(hypotheses, 2))
        # Limit to avoid excessive LLM calls
        pairs = pairs[:settings.contradiction_max_hypothesis_pairs]

        logger.info(
            "Checking consistency between {} hypothesis pairs for case {}",
            len(pairs), case_id,
        )

        contradictions: list[dict[str, Any]] = []

        for hyp_a, hyp_b in pairs:
            try:
                # Build elements text for the pair
                elements_text = (
                    f"HYPOTHESE A:\n"
                    f"Titre: {hyp_a.get('title', 'N/A')}\n"
                    f"Description: {hyp_a.get('description', 'N/A')}\n"
                    f"Score: {hyp_a.get('current_score', '?')}/100\n\n"
                    f"HYPOTHESE B:\n"
                    f"Titre: {hyp_b.get('title', 'N/A')}\n"
                    f"Description: {hyp_b.get('description', 'N/A')}\n"
                    f"Score: {hyp_b.get('current_score', '?')}/100"
                )

                prompt = CONTRADICTION_DETECTION_PROMPT.format(elements=elements_text)

                raw = await self._router.route(TaskType.CONTRADICTION_DETECTION, prompt)
                parsed = parse_json_safe(raw)

                if parsed and "contradictions" in parsed:
                    for c in parsed["contradictions"]:
                        if not isinstance(c, dict):
                            continue
                        c["hypothesis_a_id"] = hyp_a["id"]
                        c["hypothesis_b_id"] = hyp_b["id"]
                        c["type"] = "hypothesis_conflict"
                        contradictions.append(c)

            except Exception as exc:
                logger.error(
                    "Failed to check consistency between {} and {}: {}",
                    hyp_a["id"][:8], hyp_b["id"][:8], exc,
                )

        logger.info(
            "Found {} inter-hypothesis contradictions for case {}",
            len(contradictions), case_id,
        )

        return contradictions

    # ==================================================================
    # Private helpers
    # ==================================================================

    async def _find_relevant_pairs(
        self,
        case_id: str,
        evidence_list: list[dict[str, Any]],
    ) -> list[tuple[dict[str, Any], dict[str, Any]]]:
        """Find evidence pairs that share at least one entity mention.

        This avoids comparing unrelated evidence (e.g. a financial
        record with an audio transcript about a different person).
        """
        # Build a mapping: evidence_id -> set of entity_ids
        evidence_entities: dict[str, set[str]] = {}
        for ev in evidence_list:
            mentions = await self._db.list_mentions_by_evidence(ev["id"])
            entity_ids = {m["entity_id"] for m in mentions}
            evidence_entities[ev["id"]] = entity_ids

        # Build evidence lookup
        ev_by_id = {ev["id"]: ev for ev in evidence_list}

        # Find pairs with shared entities
        pairs: list[tuple[dict[str, Any], dict[str, Any]]] = []
        seen: set[tuple[str, str]] = set()

        ev_ids = list(evidence_entities.keys())
        for i, eid_a in enumerate(ev_ids):
            for eid_b in ev_ids[i + 1:]:
                shared = evidence_entities[eid_a] & evidence_entities[eid_b]
                if shared:
                    pair_key = tuple(sorted([eid_a, eid_b]))
                    if pair_key not in seen:
                        seen.add(pair_key)
                        pairs.append((ev_by_id[eid_a], ev_by_id[eid_b]))

        # Limit the number of pairs to avoid excessive LLM calls
        max_pairs = settings.contradiction_max_evidence_pairs
        if len(pairs) > max_pairs:
            logger.info("Limiting from {} to {} pairs", len(pairs), max_pairs)
            pairs = pairs[:max_pairs]

        return pairs

    async def _analyse_pair(
        self,
        ev_a: dict[str, Any],
        ev_b: dict[str, Any],
    ) -> list[dict[str, Any]]:
        """Analyse a single evidence pair for contradictions using deepseek-r1."""
        # Build elements text
        text_a = ev_a.get("summary") or (ev_a.get("raw_text", "")[:settings.text_truncation_short]) or "(pas de contenu)"
        text_b = ev_b.get("summary") or (ev_b.get("raw_text", "")[:settings.text_truncation_short]) or "(pas de contenu)"

        elements_text = (
            f"ELEMENT A — {ev_a.get('title', 'N/A')} "
            f"(source: {ev_a.get('source', 'inconnue')}, "
            f"fiabilite: {ev_a.get('reliability', '?')}/100):\n"
            f"{text_a}\n\n"
            f"ELEMENT B — {ev_b.get('title', 'N/A')} "
            f"(source: {ev_b.get('source', 'inconnue')}, "
            f"fiabilite: {ev_b.get('reliability', '?')}/100):\n"
            f"{text_b}"
        )

        prompt = CONTRADICTION_DETECTION_PROMPT.format(elements=elements_text)

        raw = await self._router.route(TaskType.CONTRADICTION_DETECTION, prompt)
        parsed = parse_json_safe(raw)

        if not parsed or "contradictions" not in parsed:
            return []

        contradictions = parsed["contradictions"]
        if not isinstance(contradictions, list):
            return []

        # Enrich each contradiction with evidence IDs
        results: list[dict[str, Any]] = []
        for c in contradictions:
            if not isinstance(c, dict):
                continue
            c["evidence_1_id"] = ev_a["id"]
            c["evidence_2_id"] = ev_b["id"]
            c["evidence_1_title"] = ev_a.get("title", "N/A")
            c["evidence_2_title"] = ev_b.get("title", "N/A")
            results.append(c)

        return results

    def _deduplicate_contradictions(
        self,
        contradictions: list[dict[str, Any]],
    ) -> list[dict[str, Any]]:
        """Remove duplicate contradictions based on description similarity.

        Uses a simple approach: deduplicate by (evidence pair, type).
        """
        seen: set[tuple[str, str, str]] = set()
        unique: list[dict[str, Any]] = []

        for c in contradictions:
            # Build dedup key from evidence IDs and contradiction type
            ev1 = c.get("evidence_1_id", c.get("element_a", ""))
            ev2 = c.get("evidence_2_id", c.get("element_b", ""))
            ctype = c.get("type", "unknown")

            # Normalise order
            pair = tuple(sorted([str(ev1), str(ev2)]))
            key = (pair[0], pair[1], ctype)

            if key not in seen:
                seen.add(key)
                unique.append(c)

        return unique

    def _build_testimonies_text(self, testimonies: list[dict[str, Any]]) -> str:
        """Build formatted text for the testimony comparison prompt."""
        parts: list[str] = []
        for i, ev in enumerate(testimonies, 1):
            title = ev.get("title", f"Temoignage {i}")
            source = ev.get("source") or "source inconnue"
            text = ev.get("summary") or ev.get("raw_text", "")[:settings.text_truncation_medium] or "(pas de contenu)"
            source_date = ev.get("source_date") or "date inconnue"

            parts.append(
                f"TEMOIGNAGE {i}: {title}\n"
                f"Source: {source}\n"
                f"Date: {source_date}\n"
                f"Contenu:\n{text}\n"
            )

        return "\n---\n\n".join(parts)
