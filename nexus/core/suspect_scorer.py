"""
NEXUS -- Suspect Scorer.

Calculates a composite suspicion score for each person entity in a case.
5 factors weighted:
  - graph_score   (20%): centrality in Neo4j + proximity to victim
  - evidence_score (25%): mention frequency * confidence * reliability
  - contradiction_score (20%): contradictions in their statements
  - profile_score  (20%): motive, alibi, criminal record (LLM-evaluated)
  - hypothesis_score (15%): hypotheses that implicate them

Usage::

    async with get_db() as conn:
        db = Database(conn)
        router = LLMRouter()
        scorer = SuspectScorer(db, router, neo4j=neo4j_client)
        results = await scorer.score_all_suspects(case_id)
"""

from __future__ import annotations

from typing import Any, Dict, List, Optional

from loguru import logger

from nexus.core.audit import AuditService
from nexus.db.sqlite_db import Database
from nexus.llm.parsers import parse_json_safe
from nexus.llm.prompts import SUSPECT_PROFILE_PROMPT
from nexus.llm.router import LLMRouter, TaskType


# ==================================================================
# Standalone utility functions (kept for backward compatibility)
# ==================================================================

# Default weights for each factor (sum = 1.0)
DEFAULT_WEIGHTS: Dict[str, float] = {
    "graph": 0.20,
    "evidence": 0.25,
    "contradiction": 0.20,
    "profile": 0.20,
    "hypothesis": 0.15,
}


def compute_evidence_score(
    mentions: List[Dict[str, Any]],
    evidence_map: Optional[Dict[str, Dict[str, Any]]] = None,
) -> float:
    """Score based on how often a person appears in evidence.

    Each mention contributes:  confidence * (reliability / 100)
    The raw sum is scaled so that ~5 high-quality mentions = 100.

    Args:
        mentions: list of entity_mention rows (need ``confidence`` key).
        evidence_map: optional dict mapping evidence_id -> evidence row
            (needs ``reliability`` key). When absent, reliability defaults to 50.
    """
    if not mentions:
        return 0.0

    total = 0.0
    for m in mentions:
        conf = m.get("confidence", 0.8)
        ev_id = m.get("evidence_id")
        reliability = 50
        if evidence_map and ev_id and ev_id in evidence_map:
            reliability = evidence_map[ev_id].get("reliability", 50)
        total += conf * (reliability / 100.0)

    # Normalise: 5 perfect mentions (conf=1.0, rel=100) -> score 100
    score = (total / 5.0) * 100.0
    return min(score, 100.0)


def compute_contradiction_score(contradictions: List[Any]) -> float:
    """Score based on the number of contradictions involving the person.

    1 contradiction = 40, 2 = 70, 3+ = 100.
    """
    n = len(contradictions)
    if n == 0:
        return 0.0
    if n == 1:
        return 40.0
    if n == 2:
        return 70.0
    return 100.0


def compute_hypothesis_score(
    person_name: str,
    hypotheses: List[Dict[str, Any]],
) -> float:
    """Score based on how strongly the person is implicated in hypotheses.

    For each hypothesis whose title or description mentions the person,
    we take its current_score. The final result is the max of these scores.
    """
    if not hypotheses or not person_name:
        return 0.0

    name_lower = person_name.lower()
    best = 0.0
    for h in hypotheses:
        title = (h.get("title") or "").lower()
        desc = (h.get("description") or "").lower()
        if name_lower in title or name_lower in desc:
            score = h.get("current_score", 0.0) or 0.0
            if score > best:
                best = score
    return min(best, 100.0)


def compute_graph_score(degree: int, max_degree: int = 10) -> float:
    """Score based on the number of graph connections.

    Linear from 0 (no connections) to 100 (max_degree connections).
    """
    if max_degree <= 0 or degree <= 0:
        return 0.0
    return min((degree / max_degree) * 100.0, 100.0)


def compute_profile_score(
    alibi_status: str = "unknown",
    has_motive: bool = False,
    has_criminal_record: bool = False,
) -> float:
    """Score based on qualitative profile factors.

    - alibi: none=40, weak=30, partial=15, strong/verified/unknown=0
    - motive: +30
    - criminal record: +30
    """
    score = 0.0

    alibi_scores = {
        "none": 40.0,
        "weak": 30.0,
        "partial": 15.0,
        "strong": 0.0,
        "verified": 0.0,
        "unknown": 0.0,
    }
    score += alibi_scores.get(alibi_status, 0.0)

    if has_motive:
        score += 30.0
    if has_criminal_record:
        score += 30.0

    return min(score, 100.0)


def compute_composite_score(
    sub_scores: Dict[str, float],
    weights: Optional[Dict[str, float]] = None,
) -> float:
    """Weighted average of factor scores, clamped to [0, 100].

    Args:
        sub_scores: dict with keys from DEFAULT_WEIGHTS.
        weights: optional override for factor weights.
    """
    w = weights or DEFAULT_WEIGHTS
    total = 0.0
    weight_sum = 0.0
    for factor, weight in w.items():
        total += sub_scores.get(factor, 0.0) * weight
        weight_sum += weight

    if weight_sum <= 0:
        return 0.0

    score = total / weight_sum
    return min(max(score, 0.0), 100.0)


# ==================================================================
# SuspectScorer class
# ==================================================================


class SuspectScorer:
    """Calculate composite suspicion scores for person entities in a case.

    Scoring weights are configurable via the class-level ``W`` dict.
    """

    # Configurable weights -- must sum to 1.0
    W = DEFAULT_WEIGHTS.copy()

    def __init__(
        self,
        db: Database,
        router: LLMRouter,
        neo4j=None,
    ) -> None:
        self._db = db
        self._router = router
        self._neo4j = neo4j
        self._audit = AuditService(db)

    # ==================================================================
    # score_all_suspects
    # ==================================================================

    async def score_all_suspects(
        self,
        case_id: str,
        trigger: str = "manual",
    ) -> list[dict[str, Any]]:
        """Score every person entity in the case.

        Creates suspect records on-the-fly for any person entity that
        does not already have one.  Returns a list of score dicts sorted
        by descending suspicion_score.
        """
        entities = await self._db.list_entities_by_case(
            case_id, entity_type="person"
        )

        if not entities:
            logger.info("No person entities found for case {}", case_id)
            return []

        # Filter out victims and witnesses — check description and mentions
        case = await self._db.get_case(case_id)
        case_desc = (case.get("description", "") or "").lower() if case else ""
        filtered = []
        for ent in entities:
            name_lower = ent["name"].lower()
            # Check if entity is explicitly the victim in the case description
            is_victim = False
            desc_lower = (ent.get("description") or "").lower()
            if "victime" in desc_lower or "decede" in desc_lower or "corps" in desc_lower:
                is_victim = True
            # Check suspect table for relationship_to_victim = 'victim'
            existing_suspect = await self._db.get_suspect_by_entity(case_id, ent["id"])
            if existing_suspect and existing_suspect.get("relationship_to_victim") == "victim":
                is_victim = True
            # Check if the person's name appears in case description as victim
            if name_lower in case_desc and ("victime" in case_desc or "corps" in case_desc):
                # Heuristic: if person name + "victime" both in case desc, likely victim
                for marker in ("victime", "corps retrouve", "meurtre de", "meurtre d'", "assassinat de",
                               "disparu", "disparition", "disparue", "enlevee", "enlevement"):
                    if marker in case_desc and name_lower in case_desc:
                        is_victim = True
                        break
            if is_victim:
                logger.info("Excluding victim '{}' from suspect scoring", ent["name"])
                continue
            filtered.append(ent)
        entities = filtered

        excluded_count = len([e for e in [ent for ent in await self._db.list_entities_by_case(case_id, entity_type="person")] if e not in entities]) if entities else 0

        if not entities:
            logger.info("No non-victim person entities found for case {}", case_id)
            return []

        logger.info(
            "Scoring {} person entities for case {}",
            len(entities), case_id,
        )

        # Filter entities with fewer than 2 evidence mentions (noise reduction)
        qualified = []
        for ent in entities:
            mentions = await self._db.list_mentions_by_entity(ent["id"])
            if len(mentions) >= 2:
                qualified.append(ent)
            else:
                logger.debug("Skipping '{}' — only {} mention(s)", ent["name"], len(mentions))

        if not qualified:
            logger.info("No person entities with 2+ mentions for case {}", case_id)
            return []

        logger.info(
            "Scoring {}/{} qualified entities for case {} (filtered {} with <2 mentions)",
            len(qualified), len(entities), case_id, len(entities) - len(qualified),
        )

        results: list[dict[str, Any]] = []
        for ent in qualified:
            try:
                result = await self.score_suspect(
                    case_id,
                    ent["id"],
                    ent["name"],
                    trigger=trigger,
                )
                results.append(result)
            except Exception as exc:
                logger.error(
                    "Failed to score entity {} ({}): {}",
                    ent["id"][:8], ent["name"], exc,
                )

        # Sort by descending composite score
        results.sort(key=lambda r: r.get("score", 0), reverse=True)

        logger.info(
            "Scored {}/{} suspects for case {}",
            len(results), len(entities), case_id,
        )
        return results

    # ==================================================================
    # score_suspect
    # ==================================================================

    async def score_suspect(
        self,
        case_id: str,
        entity_id: str,
        name: str,
        trigger: str = "manual",
    ) -> dict[str, Any]:
        """Calculate composite score for one suspect.

        Steps:
          1. Get or create suspect record in SQLite
          2. Calculate each factor (graph, evidence, contradiction,
             hypothesis)
          3. Keep existing profile_score unless re-evaluated via
             evaluate_profile()
          4. Compute weighted total
          5. Update suspect record + create snapshot
          6. Return score breakdown
        """
        # 1. Get or create suspect record
        suspect = await self._db.get_suspect_by_entity(case_id, entity_id)
        if not suspect:
            suspect = await self._db.create_suspect(
                case_id=case_id, entity_id=entity_id
            )

        # 2. Calculate each factor
        g = await self._calc_graph_score(case_id, entity_id)
        e = await self._calc_evidence_score(case_id, entity_id)
        c = await self._calc_contradiction_score(case_id, entity_id, name)
        # 3. Keep existing profile score unless re-evaluated
        p = suspect.get("profile_score", 0.0) or 0.0
        h = await self._calc_hypothesis_score(case_id, name)

        # 4. Composite
        total = (
            self.W["graph"] * g
            + self.W["evidence"] * e
            + self.W["contradiction"] * c
            + self.W["profile"] * p
            + self.W["hypothesis"] * h
        )

        # 5. Update suspect record
        await self._db.update_suspect(
            suspect["id"],
            suspicion_score=round(total, 1),
            graph_score=round(g, 1),
            evidence_score=round(e, 1),
            contradiction_score=round(c, 1),
            profile_score=round(p, 1),
            hypothesis_score=round(h, 1),
        )

        # Create snapshot for history
        await self._db.create_suspect_snapshot(
            suspect_id=suspect["id"],
            suspicion_score=round(total, 1),
            graph_score=round(g, 1),
            evidence_score=round(e, 1),
            contradiction_score=round(c, 1),
            profile_score=round(p, 1),
            hypothesis_score=round(h, 1),
            trigger=trigger,
        )

        # Audit trail
        await self._audit.log(
            case_id=case_id,
            actor="system",
            action="suspect_scored",
            target_type="suspect",
            target_id=suspect["id"],
            summary=(
                f"Suspect {name} scored: {total:.1f} "
                f"(G={g:.0f} E={e:.0f} C={c:.0f} P={p:.0f} H={h:.0f})"
            ),
        )

        logger.info(
            "Suspect {} ({}) scored {:.1f} -- G={:.0f} E={:.0f} C={:.0f} P={:.0f} H={:.0f}",
            entity_id[:8], name, total, g, e, c, p, h,
        )

        return {
            "suspect_id": suspect["id"],
            "entity_id": entity_id,
            "name": name,
            "score": round(total, 1),
            "factors": {
                "graph": round(g, 1),
                "evidence": round(e, 1),
                "contradiction": round(c, 1),
                "profile": round(p, 1),
                "hypothesis": round(h, 1),
            },
        }

    # ==================================================================
    # evaluate_profile
    # ==================================================================

    async def evaluate_profile(
        self,
        case_id: str,
        entity_id: str,
    ) -> dict[str, Any]:
        """Use LLM to evaluate suspect profile (motive, alibi, record).

        Sends all evidence mentioning this person to nexus 26B with
        SUSPECT_PROFILE_PROMPT.  Parses the JSON response and updates
        the suspect's profile_score, alibi_status and known_motive.

        Returns the updated suspect dict.
        """
        # Load entity
        entity = await self._db.get_entity(entity_id)
        if entity is None:
            raise ValueError(f"Entity not found: {entity_id}")

        name = entity.get("name", "Inconnu")

        # Get or create suspect record
        suspect = await self._db.get_suspect_by_entity(case_id, entity_id)
        if not suspect:
            suspect = await self._db.create_suspect(
                case_id=case_id, entity_id=entity_id
            )

        # Gather evidence mentioning this person
        mentions = await self._db.list_mentions_by_entity(entity_id)
        evidence_summaries_parts: list[str] = []
        for m in mentions:
            ev = await self._db.get_evidence(m["evidence_id"])
            if ev:
                title = ev.get("title", "N/A")
                summary = ev.get("summary") or ev.get("raw_text", "")[:500] or "(vide)"
                context = m.get("context", "")
                confidence = m.get("confidence", 0.0)
                evidence_summaries_parts.append(
                    f"- [{title}] (confiance: {confidence:.1%}): {summary}\n"
                    f"  Contexte de mention: {context}"
                )

        evidence_text = (
            "\n".join(evidence_summaries_parts)
            if evidence_summaries_parts
            else "(aucune preuve ne mentionne cette personne)"
        )

        relationship = suspect.get("relationship_to_victim") or "inconnue"

        # Build and send prompt
        prompt = SUSPECT_PROFILE_PROMPT.format(
            name=name,
            relationship=relationship,
            evidence_summaries=evidence_text,
        )

        logger.info("Evaluating profile for suspect {} ({})", entity_id[:8], name)
        raw_response = await self._router.route(TaskType.SUSPECT_PROFILE, prompt)

        # Parse response
        parsed = parse_json_safe(raw_response)
        if not parsed:
            logger.error(
                "Failed to parse suspect profile response for {} ({})",
                entity_id[:8], name,
            )
            return suspect

        # Extract scores
        mobile_score = _clamp(parsed.get("mobile_score", 0), 0, 30)
        alibi_score = _clamp(parsed.get("alibi_score", 0), 0, 40)
        danger_score = _clamp(parsed.get("danger_score", 0), 0, 30)
        profile_total = _clamp(
            parsed.get("total", mobile_score + alibi_score + danger_score),
            0, 100,
        )

        # Map alibi_status from LLM
        alibi_status_raw = parsed.get("alibi_status", "unknown")
        valid_statuses = {"none", "weak", "partial", "strong", "verified", "unknown"}
        alibi_status = alibi_status_raw if alibi_status_raw in valid_statuses else "unknown"

        motive_desc = parsed.get("mobile_description", "")
        reasoning = parsed.get("reasoning", "")

        # Update suspect
        updated = await self._db.update_suspect(
            suspect["id"],
            profile_score=round(float(profile_total), 1),
            alibi_status=alibi_status,
            known_motive=motive_desc or suspect.get("known_motive"),
        )

        # Audit
        await self._audit.log(
            case_id=case_id,
            actor="system",
            action="suspect_profile_evaluated",
            target_type="suspect",
            target_id=suspect["id"],
            summary=(
                f"Profil evalue pour {name}: mobile={mobile_score}, "
                f"alibi={alibi_score} ({alibi_status}), danger={danger_score}, "
                f"total={profile_total}"
            ),
        )

        logger.info(
            "Profile evaluated for {} ({}): profile_score={}, alibi={}",
            entity_id[:8], name, profile_total, alibi_status,
        )

        return updated  # type: ignore[return-value]

    # ==================================================================
    # get_evolution
    # ==================================================================

    async def get_evolution(
        self, suspect_id: str
    ) -> list[dict[str, Any]]:
        """Return time-series data for suspect score evolution.

        Returns:
            [{date, score, factors, trigger}] sorted by date ascending.
        """
        snapshots = await self._db.list_suspect_snapshots(suspect_id)

        # list_suspect_snapshots returns DESC order; reverse for chronological
        snapshots.reverse()

        return [
            {
                "date": s.get("created_at"),
                "score": s.get("suspicion_score"),
                "factors": {
                    "graph": s.get("graph_score"),
                    "evidence": s.get("evidence_score"),
                    "contradiction": s.get("contradiction_score"),
                    "profile": s.get("profile_score"),
                    "hypothesis": s.get("hypothesis_score"),
                },
                "trigger": s.get("trigger"),
                "reasoning": s.get("reasoning"),
            }
            for s in snapshots
        ]

    # ==================================================================
    # Private score calculators
    # ==================================================================

    async def _calc_graph_score(
        self, case_id: str, entity_id: str
    ) -> float:
        """Centrality + proximity to victim in Neo4j.

        Returns 0.0 if Neo4j is unavailable.
        Uses degree centrality from get_central_entities and
        find_shortest_path to each victim entity.
        """
        if not self._neo4j:
            return 0.0

        try:
            # Get centrality ranking
            central = await self._neo4j.get_central_entities(case_id, limit=50)
            max_degree = max((c.get("degree", 0) for c in central), default=1)
            if max_degree == 0:
                max_degree = 1

            entity_degree = 0
            for c in central:
                if c.get("id") == entity_id:
                    entity_degree = c.get("degree", 0)
                    break

            # Centrality component: 0-50 points
            centrality_score = (entity_degree / max_degree) * 50

            # Proximity to victims: 0-50 points
            proximity_score = 0.0
            try:
                relations = await self._neo4j.get_relations(entity_id)
                # Check if any neighbor is connected via VICTIM_OF or similar
                for rel in relations:
                    rel_type = rel.get("type", "")
                    if rel_type in ("VICTIM_OF", "KNOWS", "RELATED_TO"):
                        proximity_score = max(proximity_score, 25.0)

                # Also try shortest path to other central entities
                if central and len(central) >= 2:
                    # Most central entity is likely the victim or core actor
                    top_entity_id = central[0].get("id")
                    if top_entity_id and top_entity_id != entity_id:
                        path = await self._neo4j.find_shortest_path(
                            entity_id, top_entity_id
                        )
                        if path:
                            # Shorter path = higher score
                            # 1 hop = 50, 2 hops = 33, 3 hops = 25, etc.
                            path_len = max(len(path) - 1, 1)
                            proximity_score = max(
                                proximity_score, 50.0 / path_len
                            )
            except Exception as exc:
                logger.debug("Proximity calculation error: {}", exc)

            return min(100.0, centrality_score + proximity_score)

        except Exception as exc:
            logger.warning("Graph score calculation failed for {}: {}", entity_id[:8], exc)
            return 0.0

    async def _calc_evidence_score(
        self, case_id: str, entity_id: str
    ) -> float:
        """Frequency * confidence * source reliability.

        Each mention contributes confidence * reliability, normalized
        to a 0-100 scale.
        """
        mentions = await self._db.list_mentions_by_entity(entity_id)
        if not mentions:
            return 0.0

        total = 0.0
        for m in mentions:
            ev = await self._db.get_evidence(m["evidence_id"])
            reliability = (ev.get("reliability", 50) / 100.0) if ev else 0.5
            confidence = m.get("confidence", 0.5)
            total += confidence * reliability

        # Normalize: each mention with perfect scores contributes ~20 points
        return min(100.0, total * 20.0)

    async def _calc_contradiction_score(
        self, case_id: str, entity_id: str, name: str
    ) -> float:
        """Count contradictions involving this person.

        Searches the audit log for contradiction_found entries that
        mention this person's name.  Each contradiction adds 25 points.
        """
        audit = await self._db.list_audit_log(
            case_id, action="contradiction_found"
        )
        name_lower = name.lower()
        count = 0
        for a in audit:
            text = (
                (a.get("summary", "") or "")
                + " "
                + (str(a.get("details") or ""))
            ).lower()
            if name_lower in text:
                count += 1

        return min(100.0, count * 25.0)

    async def _calc_hypothesis_score(
        self, case_id: str, name: str
    ) -> float:
        """Sum of hypothesis scores that mention this person.

        Active hypotheses whose title or description mentions the
        suspect's name contribute their current_score to the total,
        normalized by the number of hypotheses.
        """
        hypotheses = await self._db.list_hypotheses_by_case(
            case_id, status="active"
        )
        if not hypotheses:
            return 0.0

        name_lower = name.lower()
        total = 0.0
        for h in hypotheses:
            text = (
                (h.get("title", "") or "")
                + " "
                + (h.get("description", "") or "")
            ).lower()
            if name_lower in text:
                total += h.get("current_score", 0.0)

        return min(100.0, total / max(len(hypotheses), 1))


# ==================================================================
# Utility
# ==================================================================

def _clamp(value: Any, lo: float, hi: float) -> float:
    """Clamp a numeric value between lo and hi."""
    try:
        v = float(value)
    except (TypeError, ValueError):
        v = 0.0
    return max(lo, min(hi, v))
