"""
NEXUS -- Hypothesis Engine.

Moteur d'hypotheses evolutives pour les cold cases.
Genere, evalue, re-evalue et fusionne des hypotheses en utilisant
une pipeline multi-modeles:

  - [nexus 26B]      Generation et scoring des hypotheses
  - [deepseek-r1 14B] Verification logique et ajustement du score

Les modeles sont appeles SEQUENTIELLEMENT pour respecter la contrainte
de 16 GB VRAM partagee.

Usage::

    async with get_db() as conn:
        db = Database(conn)
        router = LLMRouter()
        engine = HypothesisEngine(db, router)
        hypotheses = await engine.generate_hypotheses(case_id)
"""

from __future__ import annotations

from typing import Any

from loguru import logger
from rapidfuzz import fuzz

from nexus.core.audit import AuditService
from nexus.db.sqlite_db import Database
from nexus.llm.parsers import parse_hypothesis_score, parse_json_safe, parse_verification
from nexus.llm.prompts import (
    HYPOTHESIS_GENERATION_PROMPT,
    HYPOTHESIS_SCORING_PROMPT,
    LOGIC_VERIFICATION_PROMPT,
)
from nexus.llm.router import LLMRouter, TaskType


# Score delta above which we flag a significant shift.
_SCORE_SHIFT_THRESHOLD = 15.0


class HypothesisEngine:
    """Generate, evaluate and manage evolving hypotheses for a case."""

    def __init__(
        self,
        db: Database,
        router: LLMRouter,
        chroma=None,
        neo4j=None,
    ) -> None:
        self._db = db
        self._router = router
        self._audit = AuditService(db)
        self._chroma = chroma
        self._neo4j = neo4j
        self._retriever = None  # lazily built

    def _get_retriever(self):
        """Lazily build and return an InvestigationRetriever.

        Returns None if ChromaDB is not available (falls back to legacy).
        """
        if self._retriever is not None:
            return self._retriever
        if self._chroma is None:
            return None
        from nexus.core.retriever import InvestigationRetriever

        self._retriever = InvestigationRetriever(
            self._chroma, self._neo4j, self._router, self._db
        )
        return self._retriever

    # ==================================================================
    # generate_hypotheses
    # ==================================================================

    async def generate_hypotheses(self, case_id: str) -> list[dict[str, Any]]:
        """Generate initial hypotheses for a case using nexus 26B.

        Steps:
          1. Build context via RAG retriever (or legacy fallback)
          2. Call nexus 26B with HYPOTHESIS_GENERATION_PROMPT
          3. Parse generated hypotheses (title, description, initial score)
          4. Save each hypothesis to SQLite
          5. Create an initial snapshot for each hypothesis
          6. Return the created hypotheses
        """
        logger.info("Generating hypotheses for case {}", case_id)

        # 1. Build context — RAG or legacy fallback
        retriever = self._get_retriever()
        if retriever is not None:
            facts_text = await retriever.build_analysis_context(
                case_id, max_tokens=4000
            )
            logger.info(
                "Built RAG context for hypothesis generation ({} chars)",
                len(facts_text),
            )
        else:
            # Legacy fallback
            evidence_list = await self._db.list_evidence_by_case(case_id)
            entities_list = await self._db.list_entities_by_case(case_id)
            facts_text = self._build_facts_context(evidence_list, entities_list)

        if not facts_text.strip() or facts_text == "(aucune donnee)":
            logger.warning("No data available to generate hypotheses for case {}", case_id)
            return []

        # 2. Call nexus 26B
        prompt = HYPOTHESIS_GENERATION_PROMPT.format(facts=facts_text)
        logger.info("Calling nexus 26B for hypothesis generation ({} chars)", len(prompt))
        raw_response = await self._router.route(TaskType.DEEP_ANALYSIS, prompt)

        # 4. Parse
        parsed = parse_json_safe(raw_response)
        if not parsed or "hypotheses" not in parsed:
            logger.error("Failed to parse hypothesis generation response")
            return []

        raw_hypotheses = parsed["hypotheses"]
        if not isinstance(raw_hypotheses, list):
            logger.error("'hypotheses' field is not a list")
            return []

        # Load existing hypotheses to detect duplicates
        existing_hyps = await self._db.list_hypotheses_by_case(case_id)
        existing_titles = [h.get("title", "") for h in existing_hyps]

        # 5 + 6. Save each hypothesis and create initial snapshot
        created: list[dict[str, Any]] = []
        for i, h in enumerate(raw_hypotheses):
            if not isinstance(h, dict):
                continue

            title = h.get("id", f"H{i + 1}")
            description = h.get("description", "")
            if not description:
                continue

            # Plausibility is 0-1 in the prompt, we store 0-100
            plausibility = h.get("plausibility", 0.5)
            try:
                plausibility = float(plausibility)
            except (ValueError, TypeError):
                plausibility = 0.5
            initial_score = max(0.0, min(100.0, plausibility * 100.0))

            # Build a full title from the hypothesis ID + first part of description
            full_title = f"{title}: {description[:80]}" if len(description) > 80 else f"{title}: {description}"

            # Deduplicate: skip if a similar hypothesis already exists
            is_duplicate = any(
                fuzz.WRatio(full_title.lower(), ex_title.lower()) > 80
                for ex_title in existing_titles
            )
            if is_duplicate:
                logger.info(
                    "Skipping duplicate hypothesis '{}' for case {}",
                    full_title[:40], case_id,
                )
                continue

            try:
                # 5. Save hypothesis
                hyp_row = await self._db.create_hypothesis(
                    case_id=case_id,
                    title=full_title,
                    description=description,
                    status="active",
                    current_score=initial_score,
                )

                # 6. Create initial snapshot
                supporting = h.get("supporting_evidence", [])
                contradicting = h.get("contradicting_evidence", [])
                tests = h.get("tests", [])

                await self._db.create_hypothesis_snapshot(
                    hypothesis_id=hyp_row["id"],
                    score=initial_score,
                    supporting=supporting,
                    contradicting=contradicting,
                    reasoning=f"Generation initiale. Tests proposes: {', '.join(tests) if tests else 'aucun'}",
                    trigger="generation",
                    model_used="nexus",
                )

                created.append(hyp_row)

                # Sync hypothesis to Neo4j graph
                if self._neo4j:
                    try:
                        await self._neo4j.sync_hypothesis(
                            hyp_row["id"], case_id, full_title, initial_score, "active",
                        )
                    except Exception as exc:
                        logger.debug("Hypothesis Neo4j sync failed: {}", exc)

                # Embed hypothesis in ChromaDB for semantic retrieval
                if self._chroma is not None:
                    try:
                        embed_text = f"{full_title}. {description}"
                        embedding = await self._router.embed(embed_text)
                        self._chroma.add_hypothesis_snapshot(
                            snapshot_id=hyp_row["id"],
                            case_id=case_id,
                            text=embed_text,
                            embedding=embedding,
                            metadata={
                                "hypothesis_id": hyp_row["id"],
                                "title": full_title,
                                "score": initial_score,
                            },
                        )
                    except Exception as embed_exc:
                        logger.debug("Hypothesis embedding failed: {}", embed_exc)

                # Track newly created title to avoid intra-batch duplicates
                existing_titles.append(full_title)

                # Audit: log hypothesis created
                await self._audit.log_hypothesis_created(
                    case_id, hyp_row["id"], full_title, initial_score,
                )

                logger.info(
                    "Created hypothesis '{}' (score={:.1f}) for case {}",
                    full_title[:40], initial_score, case_id,
                )

            except Exception as exc:
                logger.error("Failed to save hypothesis {}: {}", title, exc)

        logger.info("Generated {} hypotheses for case {}", len(created), case_id)
        return created

    # ==================================================================
    # evaluate_hypothesis
    # ==================================================================

    async def evaluate_hypothesis(
        self,
        hypothesis_id: str,
        trigger: str = "manual",
        _shared_context: str | None = None,
    ) -> dict[str, Any]:
        """Re-evaluate a single hypothesis through the multi-model pipeline.

        Pipeline:
          1. Load hypothesis + last snapshot
          2. Build evidence context via RAG retriever (or legacy fallback)
          3. [nexus 26B] Re-score with HYPOTHESIS_SCORING_PROMPT
          4. [deepseek-r1] Verify reasoning with LOGIC_VERIFICATION_PROMPT
          5. Compute final score (70% nexus + 30% deepseek adjustment)
          6. Save snapshot
          7. Flag significant shifts (|delta| > 15)
          8. Suggest "refuted" if score < 10 and was > 50
          9. Suggest "confirmed" if score > 90 and deepseek confidence > 0.8
          10. Return the snapshot with metadata

        Args:
            hypothesis_id: The hypothesis to re-evaluate.
            trigger: What triggered this evaluation.
            _shared_context: Optional pre-built context from evaluate_all()
                to avoid rebuilding the context for each hypothesis.
        """
        logger.info("Evaluating hypothesis {}", hypothesis_id)

        # 1. Load hypothesis
        hypothesis = await self._db.get_hypothesis(hypothesis_id)
        if hypothesis is None:
            raise ValueError(f"Hypothesis not found: {hypothesis_id}")

        case_id = hypothesis["case_id"]
        previous_score = hypothesis.get("current_score", 50.0)

        # Load last snapshot for context
        snapshots = await self._db.list_snapshots_by_hypothesis(hypothesis_id)
        last_snapshot = snapshots[0] if snapshots else None

        # 2. Build evidence context — RAG or legacy fallback
        retriever = self._get_retriever()
        if retriever is not None:
            # Use per-hypothesis RAG retrieval
            hyp_results = await retriever.retrieve_for_hypothesis(
                hypothesis, case_id, n_results=10
            )
            supporting_chunks = hyp_results.get("supporting", [])
            contradicting_chunks = hyp_results.get("contradicting", [])
            sup_lines = [
                f"- [{c.get('title', '?')}]: {c.get('chunk_text', '')[:500]}"
                for c in supporting_chunks
            ]
            contra_lines = [
                f"- [CONTRA] [{c.get('title', '?')}]: {c.get('chunk_text', '')[:500]}"
                for c in contradicting_chunks
            ]
            new_evidence_context = "\n".join(sup_lines + contra_lines)

            # Append shared context summary if provided by evaluate_all
            if _shared_context:
                new_evidence_context += f"\n\nCONTEXTE GENERAL:\n{_shared_context}"

            # Add other hypotheses for cross-reference
            other_hypotheses = await self._db.list_hypotheses_by_case(case_id)
            other_hyps_text = "\n".join(
                f"- [{h.get('status', '?')}] {h.get('title', 'N/A')} "
                f"(score: {h.get('current_score', '?')})"
                for h in other_hypotheses
                if h["id"] != hypothesis_id
            )
            if other_hyps_text:
                new_evidence_context += f"\n\nAUTRES HYPOTHESES:\n{other_hyps_text}"

            logger.info(
                "Built RAG context for hypothesis {} ({} chars)",
                hypothesis_id[:8],
                len(new_evidence_context),
            )
        else:
            # Legacy fallback: load everything
            evidence_list = await self._db.list_evidence_by_case(case_id)
            entities_list = await self._db.list_entities_by_case(case_id)
            other_hypotheses = await self._db.list_hypotheses_by_case(case_id)

            evidence_text = self._build_evidence_text(evidence_list)
            entities_text = self._build_entities_text(entities_list)
            new_evidence_context = f"{evidence_text}\n\nENTITES:\n{entities_text}"

            other_hyps_text = "\n".join(
                f"- [{h.get('status', '?')}] {h.get('title', 'N/A')} "
                f"(score: {h.get('current_score', '?')})"
                for h in other_hypotheses
                if h["id"] != hypothesis_id
            )
            if other_hyps_text:
                new_evidence_context += f"\n\nAUTRES HYPOTHESES:\n{other_hyps_text}"

        # 3. [nexus 26B] Re-score
        hypothesis_text = (
            f"{hypothesis.get('title', '')}: {hypothesis.get('description', '')}"
        )
        scoring_prompt = HYPOTHESIS_SCORING_PROMPT.format(
            hypothesis=hypothesis_text,
            current_score=previous_score,
            new_evidence=new_evidence_context,
            hypothesis_id=hypothesis_id,
        )

        logger.info("Calling nexus 26B for hypothesis scoring")
        raw_scoring = await self._router.route(TaskType.HYPOTHESIS_SCORING, scoring_prompt)
        scoring_result = parse_hypothesis_score(raw_scoring)

        if not scoring_result:
            logger.error("Failed to parse scoring response for hypothesis {}", hypothesis_id)
            scoring_result = {
                "new_score": previous_score,
                "supporting": [],
                "contradicting": [],
                "reasoning": "Echec du parsing de la reponse LLM",
                "status": "active",
            }

        nexus_score = scoring_result.get("new_score", previous_score)
        # Normalise: prompt uses 0-1, we store 0-100
        if nexus_score <= 1.0:
            nexus_score = nexus_score * 100.0
        nexus_score = max(0.0, min(100.0, nexus_score))

        # 4. [deepseek-r1] Logic verification
        reasoning_to_verify = scoring_result.get("reasoning", "")
        verification = {}
        deepseek_adjusted_score = nexus_score
        deepseek_confidence = 0.0

        if reasoning_to_verify.strip():
            logger.info("Calling deepseek-r1 for logic verification")
            verification_prompt = LOGIC_VERIFICATION_PROMPT.format(
                reasoning=reasoning_to_verify[:10_000],
            )
            raw_verification = await self._router.route(
                TaskType.LOGIC_VERIFICATION, verification_prompt
            )
            verification = parse_verification(raw_verification)

            if verification:
                soundness = verification.get("soundness_score", 0.5)
                deepseek_confidence = soundness
                fallacies = verification.get("fallacies", [])

                # Adjust score based on logical soundness
                # If soundness is low, pull score towards 50 (neutral)
                if soundness < 0.5 and fallacies:
                    # Penalise: move score towards 50
                    adjustment = (nexus_score - 50.0) * (1.0 - soundness)
                    deepseek_adjusted_score = nexus_score - adjustment
                else:
                    deepseek_adjusted_score = nexus_score

                deepseek_adjusted_score = max(0.0, min(100.0, deepseek_adjusted_score))

        # 5. Compute final score (70% nexus + 30% deepseek if adjustment exists)
        if verification and deepseek_adjusted_score != nexus_score:
            final_score = (0.7 * nexus_score) + (0.3 * deepseek_adjusted_score)
        else:
            final_score = nexus_score
        final_score = round(max(0.0, min(100.0, final_score)), 2)

        delta = final_score - previous_score

        # 6. Save snapshot
        snapshot = await self._db.create_hypothesis_snapshot(
            hypothesis_id=hypothesis_id,
            score=final_score,
            supporting=scoring_result.get("supporting", []),
            contradicting=scoring_result.get("contradicting", []),
            reasoning=scoring_result.get("reasoning", ""),
            trigger=trigger,
            model_used="nexus+deepseek-r1",
        )

        # Update hypothesis current_score
        update_fields: dict[str, Any] = {"current_score": final_score}

        # 7. Flag significant shift
        significant_shift = abs(delta) > _SCORE_SHIFT_THRESHOLD

        # 8. Suggest "refuted" if score < 10 and was > 50
        suggested_status = None
        if final_score < 10.0 and previous_score > 50.0:
            suggested_status = "refuted"
            logger.warning(
                "Hypothesis {} dropped from {:.1f} to {:.1f} — suggesting REFUTED",
                hypothesis_id[:8], previous_score, final_score,
            )

        # 9. Suggest "confirmed" if score > 90 and deepseek confidence > 0.8
        if final_score > 90.0 and deepseek_confidence > 0.8:
            suggested_status = "confirmed"
            logger.info(
                "Hypothesis {} at {:.1f} with high confidence — suggesting CONFIRMED",
                hypothesis_id[:8], final_score,
            )

        if suggested_status:
            update_fields["status"] = suggested_status

        await self._db.update_hypothesis(hypothesis_id, **update_fields)

        # Sync updated score to Neo4j graph
        if self._neo4j:
            try:
                await self._neo4j.sync_hypothesis(
                    hypothesis_id,
                    case_id,
                    hypothesis.get("title", ""),
                    final_score,
                    update_fields.get("status", hypothesis.get("status", "active")),
                )
            except Exception as exc:
                logger.debug("Hypothesis Neo4j sync failed: {}", exc)

        # Create alert if significant shift
        if significant_shift:
            direction = "renforce" if delta > 0 else "affaibli"
            severity = "critical" if abs(delta) >= 30 else "warning"
            await self._db.create_alert(
                case_id=case_id,
                alert_type="score_shift",
                severity=severity,
                title=f"Hypothese {direction}: {hypothesis.get('title', '?')[:60]}",
                message=(
                    f"Score modifie de {previous_score:.1f} a {final_score:.1f} "
                    f"(delta: {delta:+.1f}). "
                    f"Raison: {scoring_result.get('reasoning', 'N/A')[:300]}"
                ),
                related_id=hypothesis_id,
            )

        # Audit: log score change
        await self._audit.log_hypothesis_scored(
            case_id,
            hypothesis_id,
            hypothesis.get("title", "?"),
            previous_score,
            final_score,
        )

        logger.info(
            "Hypothesis {} evaluated: {:.1f} -> {:.1f} (delta={:+.1f}, trigger={})",
            hypothesis_id[:8], previous_score, final_score, delta, trigger,
        )

        # Update hypothesis embedding in ChromaDB
        if self._chroma is not None:
            try:
                embed_text = (
                    f"{hypothesis.get('title', '')}. "
                    f"{hypothesis.get('description', '')}. "
                    f"Score: {final_score:.0f}. "
                    f"{scoring_result.get('reasoning', '')[:500]}"
                )
                embedding = await self._router.embed(embed_text)
                self._chroma.add_hypothesis_snapshot(
                    snapshot_id=hypothesis_id,
                    case_id=case_id,
                    text=embed_text,
                    embedding=embedding,
                    metadata={
                        "hypothesis_id": hypothesis_id,
                        "title": hypothesis.get("title", ""),
                        "score": final_score,
                        "trigger": trigger,
                    },
                )
            except Exception as embed_exc:
                logger.debug("Hypothesis embedding update failed: {}", embed_exc)

        # Enrich snapshot with evaluation metadata
        snapshot["delta"] = delta
        snapshot["previous_score"] = previous_score
        snapshot["significant_shift"] = significant_shift
        snapshot["suggested_status"] = suggested_status
        snapshot["verification"] = verification

        return snapshot

    # ==================================================================
    # evaluate_all
    # ==================================================================

    async def evaluate_all(self, case_id: str) -> list[dict[str, Any]]:
        """Re-evaluate ALL active hypotheses for a case sequentially.

        Builds a shared context ONCE via the retriever and passes it to
        each evaluate_hypothesis call to avoid redundant embedding lookups.

        Returns the list of snapshots created.
        """
        hypotheses = await self._db.list_hypotheses_by_case(case_id, status="active")

        if not hypotheses:
            logger.info("No active hypotheses to evaluate for case {}", case_id)
            return []

        logger.info("Evaluating {} active hypotheses for case {}", len(hypotheses), case_id)

        # Build shared context once for the whole batch
        shared_context: str | None = None
        retriever = self._get_retriever()
        if retriever is not None:
            shared_context = await retriever.build_analysis_context(
                case_id, max_tokens=2000
            )
            logger.info(
                "Built shared RAG context for evaluate_all ({} chars)",
                len(shared_context),
            )

        snapshots: list[dict[str, Any]] = []
        for h in hypotheses:
            try:
                snapshot = await self.evaluate_hypothesis(
                    h["id"],
                    trigger="evaluate_all",
                    _shared_context=shared_context,
                )
                snapshots.append(snapshot)
            except Exception as exc:
                logger.error("Failed to evaluate hypothesis {}: {}", h["id"], exc)

        logger.info(
            "Evaluated {}/{} hypotheses for case {}",
            len(snapshots), len(hypotheses), case_id,
        )
        return snapshots

    # ==================================================================
    # get_evolution
    # ==================================================================

    async def get_evolution(self, hypothesis_id: str) -> list[dict[str, Any]]:
        """Return time-series data for hypothesis score evolution.

        Returns:
            [{date, score, trigger, model_used}] sorted by date ascending.
        """
        snapshots = await self._db.list_snapshots_by_hypothesis(hypothesis_id)

        # list_snapshots_by_hypothesis returns DESC order; reverse for chronological
        snapshots.reverse()

        evolution = [
            {
                "date": s.get("created_at"),
                "score": s.get("score"),
                "trigger": s.get("trigger"),
                "model_used": s.get("model_used"),
            }
            for s in snapshots
        ]

        return evolution

    # ==================================================================
    # merge_hypotheses
    # ==================================================================

    async def merge_hypotheses(
        self,
        hyp_ids: list[str],
        new_title: str,
        new_description: str,
    ) -> dict[str, Any]:
        """Merge multiple hypotheses into a single new one.

        Steps:
          1. Load all source hypotheses and compute average score
          2. Create new hypothesis with the average score
          3. Mark source hypotheses as "merged"
          4. Copy relevant snapshots to the new hypothesis
          5. Return the new hypothesis
        """
        if len(hyp_ids) < 2:
            raise ValueError("At least 2 hypothesis IDs required for merge")

        # 1. Load source hypotheses
        sources: list[dict[str, Any]] = []
        for hid in hyp_ids:
            h = await self._db.get_hypothesis(hid)
            if h is None:
                raise ValueError(f"Hypothesis not found: {hid}")
            sources.append(h)

        # Verify all belong to the same case
        case_ids = {h["case_id"] for h in sources}
        if len(case_ids) > 1:
            raise ValueError("Cannot merge hypotheses from different cases")

        case_id = sources[0]["case_id"]

        # Compute average score
        avg_score = sum(h.get("current_score", 50.0) for h in sources) / len(sources)
        avg_score = round(max(0.0, min(100.0, avg_score)), 2)

        # 2. Create new merged hypothesis
        new_hyp = await self._db.create_hypothesis(
            case_id=case_id,
            title=new_title,
            description=new_description,
            status="active",
            current_score=avg_score,
        )

        logger.info(
            "Created merged hypothesis {} (score={:.1f}) from {} sources",
            new_hyp["id"][:8], avg_score, len(sources),
        )

        # 3. Mark sources as "merged"
        for h in sources:
            await self._db.update_hypothesis(h["id"], status="merged")
            logger.debug("Marked hypothesis {} as merged", h["id"][:8])

        # 4. Create initial snapshot with merge context
        source_titles = [h.get("title", "?") for h in sources]
        merge_reasoning = (
            f"Fusion de {len(sources)} hypotheses: {', '.join(source_titles)}. "
            f"Score moyen initial: {avg_score:.1f}."
        )

        await self._db.create_hypothesis_snapshot(
            hypothesis_id=new_hyp["id"],
            score=avg_score,
            supporting=[
                {"source_hypothesis": h["id"], "title": h.get("title", ""), "score": h.get("current_score", 0)}
                for h in sources
            ],
            contradicting=None,
            reasoning=merge_reasoning,
            trigger="merge",
            model_used="system",
        )

        return new_hyp

    # ==================================================================
    # Private helpers
    # ==================================================================

    def _build_facts_context(
        self,
        evidence_list: list[dict[str, Any]],
        entities_list: list[dict[str, Any]],
    ) -> str:
        """Build a textual context of case facts for hypothesis generation."""
        sections: list[str] = []

        if evidence_list:
            lines = ["=== PREUVES ==="]
            for e in evidence_list:
                title = e.get("title", "N/A")
                summary = e.get("summary") or e.get("raw_text", "")[:500] or "(pas de contenu)"
                source = e.get("source") or "inconnue"
                reliability = e.get("reliability", "?")
                lines.append(
                    f"[{title}] (source: {source}, fiabilite: {reliability}/100)\n  {summary}"
                )
            sections.append("\n".join(lines))

        if entities_list:
            lines = ["=== ENTITES IDENTIFIEES ==="]
            for ent in entities_list:
                etype = ent.get("entity_type", "?")
                name = ent.get("name", "?")
                desc = ent.get("description") or ""
                aliases = ent.get("aliases") or []
                alias_str = f" (alias: {', '.join(aliases)})" if aliases else ""
                lines.append(f"- {name} ({etype}){alias_str} {desc}")
            sections.append("\n".join(lines))

        return "\n\n".join(sections) if sections else "(aucune donnee)"

    def _build_evidence_text(self, evidence_list: list[dict[str, Any]]) -> str:
        """Build compact evidence text for scoring prompts."""
        if not evidence_list:
            return "(aucune preuve)"

        lines: list[str] = []
        for e in evidence_list:
            title = e.get("title", "N/A")
            summary = e.get("summary") or (e.get("raw_text", "")[:500])
            lines.append(f"- [{title}]: {summary}")

        return "\n".join(lines)

    def _build_entities_text(self, entities_list: list[dict[str, Any]]) -> str:
        """Build compact entity list for scoring prompts."""
        if not entities_list:
            return "(aucune entite)"

        lines: list[str] = []
        for ent in entities_list:
            etype = ent.get("entity_type", "?")
            name = ent.get("name", "?")
            lines.append(f"- {name} ({etype})")

        return "\n".join(lines)
