"""
NEXUS -- Analysis Pipeline.

Orchestrates multi-model sequential analysis of a case:
  1. [gemma4:e4b]       Summarise un-summarised evidence
  2. [nexus 26B]        Deep analysis of the full dossier
  3. [nexus 26B]        Re-score existing hypotheses
  4. [deepseek-r1 14B]  Logic verification
  5. Save results, create alerts for significant score changes

Models are called SEQUENTIALLY to respect the 16 GB shared VRAM
constraint (the LLMRouter heavy-lock already enforces this, but the
pipeline also avoids building parallel call chains).

Usage::

    async with get_db() as conn:
        db = Database(conn)
        router = LLMRouter()
        pipeline = AnalysisPipeline(db, router)
        run = await pipeline.run_full_analysis(case_id)
"""

from __future__ import annotations

import time
from datetime import datetime, timezone
from typing import Any

from loguru import logger

from nexus.db.models import AnalysisRun
from nexus.db.sqlite_db import Database
from nexus.llm.parsers import parse_hypothesis_score, parse_verification
from nexus.llm.prompts import (
    DEEP_ANALYSIS_PROMPT,
    EVIDENCE_SUMMARY_PROMPT,
    HYPOTHESIS_SCORING_PROMPT,
    LOGIC_VERIFICATION_PROMPT,
)
from nexus.llm.router import LLMRouter, TaskType


# Score delta above which we create an alert.
_SCORE_SHIFT_THRESHOLD = 15.0


class AnalysisPipeline:
    """Orchestrate multi-model analysis on a case."""

    def __init__(
        self,
        db: Database,
        router: LLMRouter,
        chroma=None,
        neo4j=None,
    ) -> None:
        self._db = db
        self._router = router
        self._chroma = chroma
        self._neo4j = neo4j
        self._retriever = None  # lazily built

    # ==================================================================
    # Retriever access
    # ==================================================================

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
    # Full analysis
    # ==================================================================

    async def run_full_analysis(self, case_id: str) -> AnalysisRun:
        """Run the complete multi-model analysis pipeline on a case.

        Steps:
          1. Create an AnalysisRun (status='running', run_type='full')
          2. [gemma4:e4b]   Summarise any evidence not yet summarised
          3. Build analysis context via RAG retriever (or legacy fallback)
          4. [nexus 26B]    Deep analysis of the context
          5. [nexus 26B]    Re-score existing hypotheses (RAG per hypothesis)
          6. [deepseek-r1]  Logic verification of the analysis
          7. Save results (output_summary, snapshots)
          8. Update status='completed' + duration
          9. Create alerts for significant score shifts (> 15 points)
        """
        t0 = time.monotonic()

        # 1. Create analysis run
        run_row = await self._db.create_analysis_run(
            case_id=case_id,
            run_type="full",
            trigger="manual",
            model_used="multi",
            input_summary="Full multi-model analysis pipeline",
        )
        run_id = run_row["id"]
        logger.info("Starting full analysis run {} for case {}", run_id, case_id)

        try:
            # 2. [gemma4:e4b] Summarise un-summarised evidence
            evidence_list = await self._db.list_evidence_by_case(case_id)
            await self._summarise_pending_evidence(evidence_list)

            hypotheses_list = await self._db.list_hypotheses_by_case(case_id)

            # 3. Build analysis context — RAG or legacy fallback
            retriever = self._get_retriever()
            if retriever is not None:
                dossier_text = await retriever.build_analysis_context(
                    case_id, max_tokens=4000
                )
                logger.info(
                    "Built RAG context for full analysis ({} chars)",
                    len(dossier_text),
                )
            else:
                # Legacy fallback: load everything and truncate
                evidence_list = await self._db.list_evidence_by_case(case_id)
                entities_list = await self._db.list_entities_by_case(case_id)
                dossier_text = self._build_dossier_text(
                    evidence_list, entities_list, hypotheses_list
                )
                logger.info(
                    "Built legacy dossier text ({} chars, no RAG)",
                    len(dossier_text),
                )

            # 4. [nexus 26B] Deep analysis
            deep_analysis = await self._run_deep_analysis(dossier_text)

            # 5. [nexus 26B] Re-score hypotheses if any exist
            score_results = []
            if hypotheses_list:
                score_results = await self._rescore_hypotheses(
                    hypotheses_list, case_id, run_id
                )

            # 6. [deepseek-r1] Logic verification
            verification = await self._run_logic_verification(deep_analysis)

            # 7. Build output summary
            output_summary = self._build_output_summary(
                deep_analysis, score_results, verification
            )

            # 8. Complete the run
            duration = time.monotonic() - t0
            completed_row = await self._db.update_analysis_run(
                run_id,
                status="completed",
                output_summary=output_summary,
                duration_sec=round(duration, 2),
                completed_at=datetime.now(timezone.utc).isoformat(),
            )

            # 9. Create alerts for significant changes
            await self._create_score_shift_alerts(case_id, score_results)

            logger.info(
                "Full analysis run {} completed in {:.1f}s", run_id, duration
            )
            return AnalysisRun(**completed_row)

        except Exception as exc:
            # Mark run as failed
            duration = time.monotonic() - t0
            logger.error("Analysis run {} failed: {}", run_id, exc)
            failed_row = await self._db.update_analysis_run(
                run_id,
                status="failed",
                output_summary=f"Pipeline failed: {exc}",
                duration_sec=round(duration, 2),
                completed_at=datetime.now(timezone.utc).isoformat(),
            )
            return AnalysisRun(**failed_row)

    # ==================================================================
    # Incremental analysis
    # ==================================================================

    async def run_incremental_analysis(
        self,
        case_id: str,
        trigger: str,
        new_evidence_id: str | None = None,
    ) -> AnalysisRun:
        """Run an incremental analysis focused on new data.

        Lighter than full analysis: only processes the new evidence and
        re-evaluates hypotheses against it.  When a retriever is available
        the context is focused on the new evidence via RAG rather than
        loading the entire dossier.

        Parameters:
            case_id: The case being analysed.
            trigger: What triggered this run (e.g. 'new_evidence',
                     'monitoring', 'scheduled').
            new_evidence_id: If a specific piece of evidence triggered
                             this run, its ID.
        """
        t0 = time.monotonic()

        input_desc = f"Incremental analysis (trigger={trigger})"
        if new_evidence_id:
            input_desc += f", new_evidence_id={new_evidence_id}"

        run_row = await self._db.create_analysis_run(
            case_id=case_id,
            run_type="incremental",
            trigger=trigger,
            model_used="multi",
            input_summary=input_desc,
        )
        run_id = run_row["id"]
        logger.info(
            "Starting incremental analysis run {} for case {} (trigger={})",
            run_id, case_id, trigger,
        )

        try:
            hypotheses_list = await self._db.list_hypotheses_by_case(case_id)

            # Build focus text from new evidence if provided
            new_evidence_text = ""
            focus_query: str | None = None
            if new_evidence_id:
                ev = await self._db.get_evidence(new_evidence_id)
                if ev:
                    # Summarise the new evidence if not yet done
                    if not ev.get("summary") and ev.get("raw_text"):
                        summary = await self._generate_single_summary(ev["raw_text"])
                        await self._db.update_evidence(new_evidence_id, summary=summary)
                        ev["summary"] = summary
                    new_evidence_text = (
                        f"NOUVELLE PREUVE:\n"
                        f"Titre: {ev.get('title', 'N/A')}\n"
                        f"Contenu: {ev.get('raw_text', '')[:4000]}\n"
                        f"Resume: {ev.get('summary', 'N/A')}\n"
                    )
                    # Use the new evidence title + summary as focus for RAG
                    focus_query = (
                        f"{ev.get('title', '')}. {ev.get('summary', '')}"
                    )
            else:
                # Summarise any pending evidence
                evidence_list = await self._db.list_evidence_by_case(case_id)
                await self._summarise_pending_evidence(evidence_list)

            # Build context — RAG or legacy fallback
            retriever = self._get_retriever()
            if retriever is not None:
                context = await retriever.build_analysis_context(
                    case_id, focus=focus_query, max_tokens=4000
                )
                if new_evidence_text:
                    dossier_text = (
                        new_evidence_text
                        + "\n\nCONTEXTE PERTINENT DU DOSSIER:\n"
                        + context
                    )
                else:
                    dossier_text = context
                logger.info(
                    "Built RAG context for incremental analysis ({} chars)",
                    len(dossier_text),
                )
            else:
                # Legacy fallback
                evidence_list = await self._db.list_evidence_by_case(case_id)
                entities_list = await self._db.list_entities_by_case(case_id)
                dossier_text = self._build_dossier_text(
                    evidence_list, entities_list, hypotheses_list
                )
                if new_evidence_text:
                    dossier_text = (
                        new_evidence_text
                        + "\n\nCONTEXTE COMPLET DU DOSSIER:\n"
                        + dossier_text
                    )

            # Deep analysis
            deep_analysis = await self._run_deep_analysis(dossier_text)

            # Re-score hypotheses
            score_results = []
            if hypotheses_list:
                score_results = await self._rescore_hypotheses(
                    hypotheses_list, case_id, run_id
                )

            # Logic verification
            verification = await self._run_logic_verification(deep_analysis)

            # Build output
            output_summary = self._build_output_summary(
                deep_analysis, score_results, verification
            )

            # Complete
            duration = time.monotonic() - t0
            completed_row = await self._db.update_analysis_run(
                run_id,
                status="completed",
                output_summary=output_summary,
                duration_sec=round(duration, 2),
                completed_at=datetime.now(timezone.utc).isoformat(),
            )

            # Alerts
            await self._create_score_shift_alerts(case_id, score_results)

            logger.info(
                "Incremental analysis run {} completed in {:.1f}s", run_id, duration
            )
            return AnalysisRun(**completed_row)

        except Exception as exc:
            duration = time.monotonic() - t0
            logger.error("Incremental analysis run {} failed: {}", run_id, exc)
            failed_row = await self._db.update_analysis_run(
                run_id,
                status="failed",
                output_summary=f"Pipeline failed: {exc}",
                duration_sec=round(duration, 2),
                completed_at=datetime.now(timezone.utc).isoformat(),
            )
            return AnalysisRun(**failed_row)

    # ==================================================================
    # Pipeline stages (private)
    # ==================================================================

    async def _summarise_pending_evidence(
        self,
        evidence_list: list[dict[str, Any]],
    ) -> None:
        """[gemma4:e4b] Generate summaries for evidence that lacks one."""
        pending = [
            e for e in evidence_list
            if not e.get("summary") and e.get("raw_text")
        ]
        if not pending:
            logger.debug("No pending evidence to summarise")
            return

        logger.info("Summarising {} un-summarised evidence items", len(pending))
        for ev in pending:
            try:
                summary = await self._generate_single_summary(ev["raw_text"])
                await self._db.update_evidence(ev["id"], summary=summary)
                logger.debug("Summarised evidence {}", ev["id"])
            except Exception as exc:
                logger.warning(
                    "Failed to summarise evidence {}: {}", ev["id"], exc
                )

    async def _generate_single_summary(self, raw_text: str) -> str:
        """Call the fast model to produce a factual summary."""
        truncated = raw_text[:8_000] if len(raw_text) > 8_000 else raw_text
        prompt = EVIDENCE_SUMMARY_PROMPT.format(evidence=truncated)
        return (await self._router.route(TaskType.EVIDENCE_SUMMARY, prompt)).strip()

    async def _run_deep_analysis(self, dossier_text: str) -> str:
        """[nexus 26B] Run the deep analysis prompt."""
        truncated = dossier_text[:20_000] if len(dossier_text) > 20_000 else dossier_text
        prompt = DEEP_ANALYSIS_PROMPT.format(dossier=truncated)

        logger.info("Running deep analysis ({} chars of dossier)", len(truncated))
        result = await self._router.route(TaskType.DEEP_ANALYSIS, prompt)
        return result.strip()

    async def _rescore_hypotheses(
        self,
        hypotheses: list[dict[str, Any]],
        case_id: str,
        run_id: str,
    ) -> list[dict[str, Any]]:
        """[nexus 26B] Re-score each active hypothesis.

        When the retriever is available, fetches per-hypothesis relevant
        evidence via RAG.  Otherwise falls back to loading all evidence.

        Returns a list of scoring result dicts (one per hypothesis),
        each containing 'hypothesis_id', 'previous_score', 'new_score',
        'delta', etc.
        """
        active = [h for h in hypotheses if h.get("status") == "active"]
        if not active:
            logger.debug("No active hypotheses to re-score")
            return []

        retriever = self._get_retriever()

        # If no retriever, prepare a single evidence_text for all hypotheses
        fallback_evidence_text: str | None = None
        if retriever is None:
            evidence_list = await self._db.list_evidence_by_case(case_id)
            fallback_evidence_text = "\n".join(
                f"- [{e.get('title', 'N/A')}]: "
                f"{e.get('summary') or (e.get('raw_text', '')[:500])}"
                for e in evidence_list
            )

        logger.info("Re-scoring {} active hypotheses", len(active))
        results: list[dict[str, Any]] = []

        for h in active:
            try:
                # Build per-hypothesis evidence context
                if retriever is not None:
                    hyp_results = await retriever.retrieve_for_hypothesis(
                        h, case_id, n_results=10
                    )
                    supporting_chunks = hyp_results.get("supporting", [])
                    contradicting_chunks = hyp_results.get("contradicting", [])
                    # Format chunks into text for the prompt
                    sup_lines = [
                        f"- [{c.get('title', '?')}]: {c.get('chunk_text', '')[:500]}"
                        for c in supporting_chunks
                    ]
                    contra_lines = [
                        f"- [CONTRA] [{c.get('title', '?')}]: {c.get('chunk_text', '')[:500]}"
                        for c in contradicting_chunks
                    ]
                    evidence_text = "\n".join(sup_lines + contra_lines)
                else:
                    evidence_text = fallback_evidence_text or "(aucune preuve)"

                prompt = HYPOTHESIS_SCORING_PROMPT.format(
                    hypothesis=f"{h['title']}: {h['description']}",
                    current_score=h.get("current_score", 50.0),
                    new_evidence=evidence_text,
                    hypothesis_id=h["id"],
                )
                raw = await self._router.route(TaskType.HYPOTHESIS_SCORING, prompt)
                parsed = parse_hypothesis_score(raw)

                if not parsed:
                    logger.warning("Could not parse score for hypothesis {}", h["id"])
                    continue

                new_score = parsed.get("new_score", h.get("current_score", 50.0))

                # Ensure score is in 0-100 range (prompt asks 0-1, we store 0-100)
                if new_score <= 1.0:
                    new_score = new_score * 100.0
                new_score = max(0.0, min(100.0, new_score))

                previous_score = h.get("current_score", 50.0)
                delta = new_score - previous_score

                # Update hypothesis score in DB
                await self._db.update_hypothesis(
                    h["id"],
                    current_score=new_score,
                )

                # Create a snapshot
                await self._db.create_hypothesis_snapshot(
                    hypothesis_id=h["id"],
                    score=new_score,
                    supporting=parsed.get("supporting", []),
                    contradicting=parsed.get("contradicting", []),
                    reasoning=parsed.get("reasoning", ""),
                    trigger="full_analysis",
                    model_used="nexus",
                )

                result_entry = {
                    "hypothesis_id": h["id"],
                    "hypothesis_title": h.get("title", ""),
                    "previous_score": previous_score,
                    "new_score": new_score,
                    "delta": delta,
                    "reasoning": parsed.get("reasoning", ""),
                }
                results.append(result_entry)

                logger.info(
                    "Hypothesis {} rescored: {:.1f} -> {:.1f} (delta={:+.1f})",
                    h["id"][:8], previous_score, new_score, delta,
                )

            except Exception as exc:
                logger.error("Failed to re-score hypothesis {}: {}", h["id"], exc)

        return results

    async def _run_logic_verification(self, reasoning: str) -> dict[str, Any]:
        """[deepseek-r1 14B] Verify the logical soundness of the analysis."""
        if not reasoning.strip():
            return {}

        truncated = reasoning[:10_000] if len(reasoning) > 10_000 else reasoning
        prompt = LOGIC_VERIFICATION_PROMPT.format(reasoning=truncated)

        logger.info("Running logic verification")
        raw = await self._router.route(TaskType.LOGIC_VERIFICATION, prompt)
        return parse_verification(raw)

    # ==================================================================
    # Helpers
    # ==================================================================

    def _build_dossier_text(
        self,
        evidence_list: list[dict[str, Any]],
        entities_list: list[dict[str, Any]],
        hypotheses_list: list[dict[str, Any]],
    ) -> str:
        """Build a structured text representation of the case for prompts."""
        sections: list[str] = []

        # Evidence section
        if evidence_list:
            lines = ["=== PREUVES ==="]
            for e in evidence_list:
                status = e.get("status", "?")
                title = e.get("title", "N/A")
                summary = e.get("summary") or "(pas de resume)"
                source = e.get("source") or "inconnue"
                lines.append(
                    f"[{title}] (source: {source}, status: {status})\n  {summary}"
                )
            sections.append("\n".join(lines))

        # Entities section
        if entities_list:
            lines = ["=== ENTITES ==="]
            for ent in entities_list:
                etype = ent.get("entity_type", "?")
                name = ent.get("name", "?")
                desc = ent.get("description") or ""
                lines.append(f"- {name} ({etype}) {desc}")
            sections.append("\n".join(lines))

        # Hypotheses section
        if hypotheses_list:
            lines = ["=== HYPOTHESES ==="]
            for h in hypotheses_list:
                score = h.get("current_score", "?")
                status = h.get("status", "?")
                lines.append(
                    f"- [{status}] {h.get('title', 'N/A')} "
                    f"(score: {score})\n  {h.get('description', '')}"
                )
            sections.append("\n".join(lines))

        return "\n\n".join(sections) if sections else "(dossier vide)"

    def _build_output_summary(
        self,
        deep_analysis: str,
        score_results: list[dict[str, Any]],
        verification: dict[str, Any],
    ) -> str:
        """Combine pipeline results into a single output summary string."""
        parts: list[str] = []

        # Deep analysis (truncated for storage)
        if deep_analysis:
            truncated = deep_analysis[:4_000] if len(deep_analysis) > 4_000 else deep_analysis
            parts.append(f"=== ANALYSE PROFONDE ===\n{truncated}")

        # Score changes
        if score_results:
            lines = ["=== CHANGEMENTS DE SCORES ==="]
            for sr in score_results:
                lines.append(
                    f"- {sr.get('hypothesis_title', '?')}: "
                    f"{sr.get('previous_score', '?'):.1f} -> {sr.get('new_score', '?'):.1f} "
                    f"(delta: {sr.get('delta', 0):+.1f})"
                )
            parts.append("\n".join(lines))

        # Logic verification
        if verification:
            soundness = verification.get("soundness_score", "N/A")
            validity = verification.get("logical_validity", "N/A")
            critique = verification.get("critique", "")
            fallacies = verification.get("fallacies", [])
            lines = [
                f"=== VERIFICATION LOGIQUE ===",
                f"Validite: {validity} | Solidite: {soundness}",
            ]
            if fallacies:
                lines.append(f"Sophismes detectes: {len(fallacies)}")
                for f in fallacies[:5]:
                    lines.append(f"  - {f.get('type', '?')}: {f.get('description', '')}")
            if critique:
                lines.append(f"Critique: {critique[:1000]}")
            parts.append("\n".join(lines))

        return "\n\n".join(parts) if parts else "(aucun resultat)"

    async def _create_score_shift_alerts(
        self,
        case_id: str,
        score_results: list[dict[str, Any]],
    ) -> None:
        """Create alerts for hypothesis score shifts exceeding the threshold."""
        for sr in score_results:
            delta = abs(sr.get("delta", 0))
            if delta >= _SCORE_SHIFT_THRESHOLD:
                direction = "renforce" if sr.get("delta", 0) > 0 else "affaibli"
                severity = "critical" if delta >= 30 else "warning"

                await self._db.create_alert(
                    case_id=case_id,
                    alert_type="score_shift",
                    severity=severity,
                    title=f"Hypothese {direction}: {sr.get('hypothesis_title', '?')}",
                    message=(
                        f"Score modifie de {sr.get('previous_score', 0):.1f} "
                        f"a {sr.get('new_score', 0):.1f} "
                        f"(delta: {sr.get('delta', 0):+.1f}). "
                        f"Raison: {sr.get('reasoning', 'N/A')[:300]}"
                    ),
                    related_id=sr.get("hypothesis_id"),
                )
                logger.warning(
                    "Alert created: hypothesis {} score shift {:.1f}",
                    sr.get("hypothesis_id", "?")[:8],
                    sr.get("delta", 0),
                )
