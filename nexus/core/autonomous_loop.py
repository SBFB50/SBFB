"""
NEXUS -- Autonomous Investigation Loop.

The brain of NEXUS. For each active case, this loop runs continuously:

1. OBSERVE  -- Check for new monitoring results, new evidence
2. ORIENT   -- Analyze new data, extract entities, update graph
3. DECIDE   -- Re-evaluate hypotheses, detect contradictions, identify gaps
4. ACT      -- Generate new search queries, adjust monitoring
5. QUESTION -- Challenge top hypothesis, ask "what would disprove this?"

This implements the OODA loop (Observe-Orient-Decide-Act) adapted for
criminal investigation, with an added self-questioning step.

The loop runs as a background task in the FastAPI lifespan.
"""

from __future__ import annotations

import asyncio
from datetime import datetime
from typing import Any

from loguru import logger

from nexus.config import settings
from nexus.core.audit import AuditService
from nexus.db.sqlite_db import Database, get_db
from nexus.llm.prompts import ADAPTIVE_QUERY_PROMPT, SELF_QUESTIONING_PROMPT
from nexus.llm.router import LLMRouter, TaskType
from nexus.monitoring.alert_manager import AlertManager


class AutonomousInvestigator:
    """Autonomous investigation daemon for a single case.

    Runs continuously, connecting all NEXUS engines together:
    - MonitoringScheduler finds new information
    - EvidenceProcessor ingests it
    - AnalysisPipeline analyzes it
    - HypothesisEngine re-evaluates theories
    - ContradictionDetector finds inconsistencies
    - Self-questioning generates new search directions

    IMPORTANT: This class opens its own DB connections per-operation
    via ``get_db()`` because it runs as a long-lived background task,
    outside any request scope.
    """

    def __init__(
        self,
        case_id: str,
        router: LLMRouter,
        chroma: Any,
        neo4j: Any,
    ) -> None:
        self._case_id = case_id
        self._router = router
        self._chroma = chroma
        self._neo4j = neo4j
        self._running = False
        self._cycle_count = 0
        self._last_action: str | None = None
        self._last_cycle_at: str | None = None
        self._started_at: str | None = None

    # ------------------------------------------------------------------
    # Public interface
    # ------------------------------------------------------------------

    @property
    def case_id(self) -> str:
        return self._case_id

    @property
    def cycle_count(self) -> int:
        return self._cycle_count

    @property
    def is_running(self) -> bool:
        return self._running

    @property
    def last_action(self) -> str | None:
        return self._last_action

    @property
    def last_cycle_at(self) -> str | None:
        return self._last_cycle_at

    @property
    def started_at(self) -> str | None:
        return self._started_at

    def get_status(self) -> dict[str, Any]:
        """Return a status dict for the API."""
        return {
            "case_id": self._case_id,
            "running": self._running,
            "cycle_count": self._cycle_count,
            "last_action": self._last_action,
            "last_cycle_at": self._last_cycle_at,
            "started_at": self._started_at,
        }

    async def _audit_log(self, actor, action, summary, **kwargs):
        """Fire-and-forget audit log via a fresh DB connection."""
        try:
            async with get_db() as conn:
                audit = AuditService(Database(conn))
                await audit.log(
                    case_id=self._case_id,
                    actor=actor,
                    action=action,
                    summary=summary,
                    **kwargs,
                )
        except Exception as exc:
            logger.warning("Audit log failed (non-blocking): {}", exc)

    async def run(self) -> None:
        """Main investigation loop -- runs until stopped."""
        self._running = True
        self._started_at = datetime.utcnow().isoformat()
        logger.info("Autonomous investigator STARTED for case {}", self._case_id)

        await self._audit_log(
            "autonomous_loop", "investigation_started",
            "Boucle autonome demarree",
        )

        while self._running:
            try:
                self._cycle_count += 1
                self._last_cycle_at = datetime.utcnow().isoformat()
                logger.info(
                    "=== Case {} -- OODA Cycle {} ===",
                    self._case_id,
                    self._cycle_count,
                )

                # PHASE 1: OBSERVE -- What's new?
                self._last_action = "OBSERVE"
                new_results = await self._observe()

                # PHASE 2: ORIENT -- Ingest and understand new data
                self._last_action = "ORIENT"
                new_evidence_ids = await self._orient(new_results)

                # PHASE 3: DECIDE -- Re-evaluate everything
                self._last_action = "DECIDE"
                decisions = await self._decide(new_evidence_ids)

                # PHASE 4: ACT -- Take actions based on decisions
                self._last_action = "ACT"
                await self._act(decisions)

                # PHASE 5: QUESTION -- Challenge ourselves
                self._last_action = "QUESTION"
                await self._question()

                self._last_action = "SLEEPING"
                logger.info(
                    "=== Case {} -- Cycle {} complete, sleeping {}min ===",
                    self._case_id,
                    self._cycle_count,
                    settings.investigation_cycle_minutes,
                )

                # Sleep between cycles
                await asyncio.sleep(settings.investigation_cycle_minutes * 60)

            except asyncio.CancelledError:
                logger.info(
                    "Autonomous investigator CANCELLED for case {}",
                    self._case_id,
                )
                break
            except Exception as e:
                logger.error(
                    "Investigation cycle error for case {}: {}",
                    self._case_id,
                    e,
                )
                self._last_action = f"ERROR: {e}"
                # Wait 5 minutes before retrying on error
                await asyncio.sleep(300)

        self._running = False
        await self._audit_log(
            "autonomous_loop", "investigation_stopped",
            f"Boucle autonome arretee apres {self._cycle_count} cycles",
        )
        logger.info(
            "Autonomous investigator STOPPED for case {} after {} cycles",
            self._case_id,
            self._cycle_count,
        )

    async def stop(self) -> None:
        """Signal the loop to stop after current cycle."""
        self._running = False

    # ================================================================
    # PHASE 1: OBSERVE -- Check for new monitoring results
    # ================================================================

    async def _observe(self) -> list[dict[str, Any]]:
        """Check for unreviewed monitoring results with high relevance."""
        async with get_db() as conn:
            db = Database(conn)
            results = await db.list_results_by_case(self._case_id)

        new_results = [
            r
            for r in results
            if not r.get("reviewed") and not r.get("is_duplicate")
        ]

        # Filter: only auto-ingest if relevance >= threshold
        threshold = settings.auto_ingest_relevance_threshold
        high_relevance = [
            r
            for r in new_results
            if (r.get("relevance_score") or 0) >= threshold
        ]

        if high_relevance:
            logger.info(
                "OBSERVE: {} new relevant results (of {} unreviewed) for case {}",
                len(high_relevance),
                len(new_results),
                self._case_id,
            )
            # Audit: log each observed result
            for r in high_relevance:
                await self._audit_log(
                    "autonomous_loop", "monitoring_result",
                    f"Resultat observe: {(r.get('title') or 'N/A')[:100]} "
                    f"(pertinence: {r.get('relevance_score', 0):.0f}%)",
                    target_type="monitoring_result",
                    target_id=r.get("id"),
                    details={"title": r.get("title"), "relevance": r.get("relevance_score")},
                    cycle_number=self._cycle_count,
                )
        else:
            logger.debug(
                "OBSERVE: No new relevant results for case {} ({} unreviewed below threshold)",
                self._case_id,
                len(new_results),
            )

        return high_relevance

    # ================================================================
    # PHASE 2: ORIENT -- Ingest new data into the system
    # ================================================================

    async def _orient(self, new_results: list[dict[str, Any]]) -> list[str]:
        """Auto-ingest promising monitoring results as evidence."""
        new_evidence_ids: list[str] = []
        max_ingest = settings.max_auto_ingest_per_cycle

        for result in new_results[:max_ingest]:
            try:
                async with get_db() as conn:
                    db = Database(conn)

                    # Import here to avoid circular imports
                    from nexus.core.evidence_processor import EvidenceProcessor

                    processor = EvidenceProcessor(
                        db=db,
                        router=self._router,
                        upload_dir=settings.upload_dir,
                        neo4j=self._neo4j,
                        chroma=self._chroma,
                    )

                    # Build text from monitoring result
                    text = (
                        f"Source: {result.get('url', 'unknown')}\n"
                        f"Titre: {result.get('title', '')}\n"
                        f"Contenu: {result.get('snippet', '')}\n"
                        f"Moteur: {result.get('source_engine', '')}\n"
                        f"Date: {result.get('found_at', '')}"
                    )

                    title_raw = result.get("title", "Resultat monitoring")
                    title = f"[AUTO-MONITORING] {title_raw[:100]}"

                    evidence = await processor.process_text_input(
                        case_id=self._case_id,
                        title=title,
                        text=text,
                        source=(
                            f"Monitoring automatique -- "
                            f"{result.get('source_engine', 'SearXNG')}"
                        ),
                    )

                    new_evidence_ids.append(evidence.id)

                    # Mark monitoring result as reviewed
                    await db.update_monitoring_result(
                        result["id"], reviewed=True
                    )

                    # Audit: log auto-ingestion
                    audit = AuditService(db)
                    await audit.log_auto_ingest(
                        case_id=self._case_id,
                        result_id=result["id"],
                        evidence_id=evidence.id,
                        title=title,
                        cycle=self._cycle_count,
                    )

                    logger.info(
                        "ORIENT: Auto-ingested monitoring result {} -> evidence {}",
                        result["id"][:8],
                        evidence.id[:8],
                    )

            except Exception as e:
                logger.error(
                    "ORIENT: Failed to ingest result {}: {}",
                    result.get("id", "?")[:8],
                    e,
                )

        return new_evidence_ids

    # ================================================================
    # PHASE 3: DECIDE -- Re-evaluate hypotheses, detect contradictions
    # ================================================================

    async def _decide(self, new_evidence_ids: list[str]) -> dict[str, Any]:
        """Re-analyze the case if new evidence was added."""
        decisions: dict[str, Any] = {
            "analysis_run": None,
            "contradictions": [],
            "score_shifts": [],
        }

        if not new_evidence_ids:
            # Even without new evidence, periodically re-evaluate
            if self._cycle_count % settings.full_reevaluation_every_n_cycles != 0:
                logger.debug(
                    "DECIDE: No new evidence and not a re-evaluation cycle, skipping"
                )
                return decisions
            logger.info(
                "DECIDE: Periodic full re-evaluation (cycle {})",
                self._cycle_count,
            )

        # Run incremental analysis for new evidence
        if new_evidence_ids:
            from nexus.core.analysis_pipeline import AnalysisPipeline

            for ev_id in new_evidence_ids:
                try:
                    async with get_db() as conn:
                        db = Database(conn)
                        pipeline = AnalysisPipeline(
                            db=db, router=self._router
                        )
                        run = await pipeline.run_incremental_analysis(
                            case_id=self._case_id,
                            trigger="autonomous_loop",
                            new_evidence_id=ev_id,
                        )
                        decisions["analysis_run"] = {
                            "id": run.id,
                            "status": run.status,
                        }
                        # Audit: log analysis completed
                        audit = AuditService(db)
                        await audit.log_analysis(
                            case_id=self._case_id,
                            run_id=run.id,
                            run_type="incremental",
                            status=run.status,
                            actor="autonomous_loop",
                        )
                        logger.info(
                            "DECIDE: Incremental analysis completed for evidence {}",
                            ev_id[:8],
                        )
                except Exception as e:
                    logger.error(
                        "DECIDE: Analysis failed for evidence {}: {}",
                        ev_id[:8],
                        e,
                    )

        # Re-evaluate ALL hypotheses
        try:
            async with get_db() as conn:
                db = Database(conn)
                from nexus.core.hypothesis_engine import HypothesisEngine

                engine = HypothesisEngine(db=db, router=self._router)

                hypotheses = await db.list_hypotheses_by_case(
                    self._case_id, status="active"
                )
                if not hypotheses:
                    # No hypotheses yet -- generate initial ones
                    logger.info(
                        "DECIDE: No hypotheses exist, generating initial set"
                    )
                    await engine.generate_hypotheses(self._case_id)
                    hypotheses = await db.list_hypotheses_by_case(
                        self._case_id, status="active"
                    )

                if hypotheses:
                    snapshots = await engine.evaluate_all(self._case_id)
                    decisions["score_shifts"] = [
                        s
                        for s in snapshots
                        if abs(s.get("delta", 0)) > 15
                    ]
                    # Audit: log each significant score shift
                    audit = AuditService(db)
                    for s in snapshots:
                        if abs(s.get("delta", 0)) > 5:
                            hyp = await db.get_hypothesis(s.get("hypothesis_id", ""))
                            hyp_title = hyp.get("title", "?") if hyp else "?"
                            await audit.log_hypothesis_scored(
                                case_id=self._case_id,
                                hyp_id=s.get("hypothesis_id", ""),
                                title=hyp_title,
                                old_score=s.get("previous_score", 0),
                                new_score=s.get("score", 0),
                                actor="autonomous_loop",
                            )
                    logger.info(
                        "DECIDE: Re-evaluated {} hypotheses",
                        len(snapshots),
                    )
        except Exception as e:
            logger.error("DECIDE: Hypothesis evaluation failed: {}", e)

        # Detect contradictions
        try:
            async with get_db() as conn:
                db = Database(conn)
                from nexus.core.contradiction_detector import (
                    ContradictionDetector,
                )

                detector = ContradictionDetector(
                    db=db, router=self._router
                )
                contradictions = await detector.detect_contradictions(
                    self._case_id
                )
                decisions["contradictions"] = contradictions

                if contradictions:
                    logger.info(
                        "DECIDE: Found {} contradictions",
                        len(contradictions),
                    )
                    alert_mgr = AlertManager(db)
                    audit = AuditService(db)
                    for c in contradictions:
                        await alert_mgr.create_contradiction_alert(
                            case_id=self._case_id,
                            details=c.get("description", str(c)),
                        )
                        # Audit: log each contradiction
                        await audit.log_contradiction_found(
                            case_id=self._case_id,
                            description=c.get("description", str(c)),
                            actor="autonomous_loop",
                        )
        except Exception as e:
            logger.error("DECIDE: Contradiction detection failed: {}", e)

        return decisions

    # ================================================================
    # PHASE 4: ACT -- Generate new search queries, adjust monitoring
    # ================================================================

    async def _act(self, decisions: dict[str, Any]) -> None:
        """Adapt monitoring based on what we learned.

        This is KEY: the system learns what to search for next.
        """
        try:
            async with get_db() as conn:
                db = Database(conn)

                # Get current state
                hypotheses = await db.list_hypotheses_by_case(
                    self._case_id, status="active"
                )
                entities = await db.list_entities_by_case(self._case_id)
                existing_jobs = await db.list_jobs_by_case(self._case_id)
                existing_queries = {j["query"] for j in existing_jobs}

                if not hypotheses:
                    logger.debug("ACT: No hypotheses, skipping query generation")
                    return

                # Ask the LLM: "Based on current hypotheses and evidence,
                # what should we search for next?"
                hypotheses_text = "\n".join(
                    [
                        f"- {h['title']} (score: {h['current_score']})"
                        for h in hypotheses
                    ]
                )
                entities_text = "\n".join(
                    [
                        f"- {e['name']} ({e['entity_type']})"
                        for e in entities[:20]
                    ]
                )
                queries_text = "\n".join(
                    [f"- {q}" for q in existing_queries]
                )
                contradictions_text = "\n".join(
                    [
                        str(c)
                        for c in decisions.get("contradictions", [])[:5]
                    ]
                )

                prompt = ADAPTIVE_QUERY_PROMPT.format(
                    hypotheses=hypotheses_text or "(aucune)",
                    entities=entities_text or "(aucune)",
                    existing_queries=queries_text or "(aucune)",
                    contradictions=contradictions_text or "(aucune)",
                )

                response = await self._router.route_json(
                    TaskType.QUERY_REFORMULATION, prompt
                )

                new_queries = response.get("queries", [])
                max_new = settings.max_new_queries_per_cycle

                created_count = 0
                for q in new_queries:
                    if created_count >= max_new:
                        break

                    query_text = (
                        q.get("query", q)
                        if isinstance(q, dict)
                        else str(q)
                    )

                    if not query_text or query_text in existing_queries:
                        continue

                    await db.create_monitoring_job(
                        case_id=self._case_id,
                        job_type="searxng",
                        query=query_text,
                        interval_hours=12,
                    )
                    created_count += 1
                    # Audit: log query generation
                    audit = AuditService(db)
                    await audit.log_query_generated(
                        case_id=self._case_id,
                        query=query_text,
                        cycle=self._cycle_count,
                    )
                    logger.info(
                        "ACT: New monitoring job created: '{}'",
                        query_text[:60],
                    )

                if created_count:
                    logger.info(
                        "ACT: Created {} new monitoring jobs for case {}",
                        created_count,
                        self._case_id,
                    )

        except Exception as e:
            logger.error(
                "ACT: Adaptive query generation failed: {}", e
            )

    # ================================================================
    # PHASE 5: QUESTION -- Challenge our own conclusions
    # ================================================================

    async def _question(self) -> None:
        """Self-questioning: adversarial thinking against top hypothesis.

        The system asks itself:
        - What would DISPROVE my top hypothesis?
        - What evidence am I MISSING?
        - Am I suffering from confirmation bias?
        - What alternative explanations haven't I considered?
        """
        try:
            async with get_db() as conn:
                db = Database(conn)

                hypotheses = await db.list_hypotheses_by_case(
                    self._case_id, status="active"
                )
                if not hypotheses:
                    logger.debug(
                        "QUESTION: No hypotheses to question for case {}",
                        self._case_id,
                    )
                    return

                # Get top hypothesis
                top = max(
                    hypotheses,
                    key=lambda h: h.get("current_score", 0),
                )

                evidence = await db.list_evidence_by_case(self._case_id)
                evidence_summaries = "\n".join(
                    [
                        f"- [{e.get('title', '?')}]: "
                        f"{(e.get('summary') or '')[:200]}"
                        for e in evidence[:15]
                    ]
                )

                all_hyps_text = "\n".join(
                    [
                        f"- {h['title']} ({h['current_score']}%)"
                        for h in hypotheses
                    ]
                )

                prompt = SELF_QUESTIONING_PROMPT.format(
                    top_hypothesis=top["title"],
                    top_score=top["current_score"],
                    top_description=top.get("description", ""),
                    all_hypotheses=all_hyps_text,
                    evidence_summaries=evidence_summaries or "(aucune preuve)",
                )

                response = await self._router.route(
                    TaskType.DEEP_ANALYSIS, prompt
                )

                # Store the questioning result as a special analysis run
                await db.create_analysis_run(
                    case_id=self._case_id,
                    run_type="self_questioning",
                    trigger="autonomous_loop",
                    status="completed",
                    model_used="nexus",
                    input_summary=(
                        f"Self-questioning cycle {self._cycle_count} -- "
                        f"Top hypothesis: {top['title']} ({top['current_score']}%)"
                    ),
                    output_summary=response[:2000],
                )

                # Audit: log self-questioning
                audit = AuditService(db)
                await audit.log_self_questioning(
                    case_id=self._case_id,
                    top_hypothesis=top["title"],
                    summary=response,
                    cycle=self._cycle_count,
                )

                logger.info(
                    "QUESTION: Self-questioning completed for case {}",
                    self._case_id,
                )

        except Exception as e:
            logger.error(
                "QUESTION: Self-questioning failed for case {}: {}",
                self._case_id,
                e,
            )
