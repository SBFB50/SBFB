"""
NEXUS -- Report generator.

Loads all case data (evidence, entities, hypotheses, alerts, monitoring)
and uses the nexus 26B model to produce structured investigation reports.
"""

from __future__ import annotations

from datetime import datetime
from typing import Any, Dict, List, Optional

from loguru import logger

from nexus.db.sqlite_db import Database
from nexus.llm.router import LLMRouter, TaskType
from nexus.llm.prompts import FINAL_REPORT_PROMPT


class ReportGenerator:
    """Generate investigation reports from case data + LLM analysis.

    Usage::

        async with get_db() as conn:
            db = Database(conn)
            gen = ReportGenerator(db, router)
            report = await gen.generate_full_report("case-uuid")
    """

    def __init__(self, db: Database, router: LLMRouter) -> None:
        self._db = db
        self._router = router

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    async def generate_full_report(self, case_id: str) -> Dict[str, Any]:
        """Generate a comprehensive investigation report.

        Loads all case data, builds a dossier string, sends it to
        nexus 26B for deep analysis, and returns structured sections.
        """
        logger.info("Generating full report for case {}", case_id)

        case = await self._db.get_case(case_id)
        if not case:
            raise ValueError(f"Case not found: {case_id}")

        # Load all data in parallel-ish (sequential but fast on SQLite)
        evidence = await self._db.list_evidence_by_case(case_id)
        entities = await self._db.list_entities_by_case(case_id)
        hypotheses = await self._db.list_hypotheses_by_case(case_id)
        alerts = await self._db.list_alerts_by_case(case_id, limit=200)
        monitoring_results = await self._load_monitoring_results(case_id)
        analysis_runs = await self._db.list_runs_by_case(case_id, limit=20)

        # Build the dossier text
        dossier = self._build_dossier_text(case, evidence, entities)
        hypotheses_text = self._build_hypotheses_text(hypotheses)
        key_evidence_text = self._build_key_evidence_text(evidence)

        # Generate executive summary via nexus 26B
        prompt = FINAL_REPORT_PROMPT.format(
            dossier=dossier,
            hypotheses=hypotheses_text,
            key_evidence=key_evidence_text,
        )

        try:
            llm_report = await self._router.route(
                TaskType.FINAL_REPORT,
                prompt,
            )
        except Exception as exc:
            logger.error("LLM report generation failed: {}", exc)
            llm_report = f"[Erreur de generation LLM: {exc}]"

        # Build timeline entries
        timeline = self._build_timeline_entries(evidence, entities)

        now = datetime.utcnow().isoformat()

        return {
            "case_info": {
                "id": case["id"],
                "name": case["name"],
                "reference": case.get("reference"),
                "status": case["status"],
                "created_at": case["created_at"],
            },
            "generated_at": now,
            "sections": {
                "executive_summary": llm_report,
                "evidence": [
                    {
                        "id": ev["id"],
                        "title": ev["title"],
                        "type": ev["evidence_type"],
                        "source": ev.get("source"),
                        "source_date": ev.get("source_date"),
                        "reliability": ev.get("reliability"),
                        "summary": ev.get("summary"),
                        "status": ev["status"],
                    }
                    for ev in evidence
                ],
                "entities": [
                    {
                        "id": ent["id"],
                        "name": ent["name"],
                        "type": ent["entity_type"],
                        "description": ent.get("description"),
                        "aliases": ent.get("aliases"),
                        "first_seen": ent.get("first_seen"),
                    }
                    for ent in entities
                ],
                "hypotheses": [
                    {
                        "id": hyp["id"],
                        "title": hyp["title"],
                        "description": hyp["description"],
                        "status": hyp["status"],
                        "score": hyp["current_score"],
                    }
                    for hyp in hypotheses
                ],
                "timeline": timeline,
                "alerts": [
                    {
                        "id": a["id"],
                        "type": a["alert_type"],
                        "severity": a["severity"],
                        "title": a["title"],
                        "message": a["message"],
                        "created_at": a["created_at"],
                    }
                    for a in alerts[:50]  # Cap for report size
                ],
                "monitoring_results": monitoring_results[:30],
                "analysis_runs": [
                    {
                        "id": r["id"],
                        "type": r["run_type"],
                        "status": r["status"],
                        "model": r.get("model_used"),
                        "started_at": r["started_at"],
                        "duration_sec": r.get("duration_sec"),
                    }
                    for r in analysis_runs
                ],
            },
        }

    async def generate_summary_report(self, case_id: str) -> Dict[str, Any]:
        """Generate a short summary report with key highlights.

        Lighter than full report: only the top hypothesis, recent
        alerts, and a short LLM summary.
        """
        logger.info("Generating summary report for case {}", case_id)

        case = await self._db.get_case(case_id)
        if not case:
            raise ValueError(f"Case not found: {case_id}")

        evidence = await self._db.list_evidence_by_case(case_id)
        hypotheses = await self._db.list_hypotheses_by_case(case_id)
        alerts = await self._db.list_alerts_by_case(case_id, unread_only=True, limit=10)

        # Build a compact dossier for the summary
        summary_prompt = (
            "Tu es un analyste d'investigation. Redige un resume executif "
            "CONCIS (10 phrases maximum) du dossier suivant.\n\n"
            f"Affaire: {case['name']}\n"
            f"Reference: {case.get('reference', 'N/A')}\n"
            f"Nombre de preuves: {len(evidence)}\n"
            f"Nombre d'hypotheses: {len(hypotheses)}\n\n"
        )

        if hypotheses:
            top = hypotheses[0]
            summary_prompt += (
                f"Hypothese principale: {top['title']} "
                f"(score: {top['current_score']}/100)\n"
                f"Description: {top['description']}\n\n"
            )

        # Add summaries of top 5 evidence pieces
        for ev in evidence[:5]:
            summary_prompt += (
                f"- Preuve: {ev['title']} ({ev['evidence_type']})\n"
                f"  Resume: {ev.get('summary', 'Pas de resume')}\n"
            )

        try:
            llm_summary = await self._router.route(
                TaskType.DEEP_ANALYSIS,
                summary_prompt,
            )
        except Exception as exc:
            logger.error("LLM summary generation failed: {}", exc)
            llm_summary = f"[Erreur de generation: {exc}]"

        # Next steps from alerts
        next_steps = [
            a["message"] for a in alerts
            if a["severity"] in ("warning", "critical")
        ]

        now = datetime.utcnow().isoformat()

        return {
            "case_info": {
                "id": case["id"],
                "name": case["name"],
                "reference": case.get("reference"),
                "status": case["status"],
            },
            "generated_at": now,
            "sections": {
                "summary": llm_summary,
                "top_hypothesis": (
                    {
                        "id": hypotheses[0]["id"],
                        "title": hypotheses[0]["title"],
                        "description": hypotheses[0]["description"],
                        "score": hypotheses[0]["current_score"],
                    }
                    if hypotheses
                    else None
                ),
                "evidence_count": len(evidence),
                "hypotheses_count": len(hypotheses),
                "unread_alerts": len(alerts),
                "next_steps": next_steps,
            },
        }

    async def generate_timeline_report(self, case_id: str) -> Dict[str, Any]:
        """Generate a timeline-focused report.

        Builds a chronological view from evidence, entities, and
        hypothesis snapshots.
        """
        logger.info("Generating timeline report for case {}", case_id)

        case = await self._db.get_case(case_id)
        if not case:
            raise ValueError(f"Case not found: {case_id}")

        evidence = await self._db.list_evidence_by_case(case_id)
        entities = await self._db.list_entities_by_case(case_id)
        hypotheses = await self._db.list_hypotheses_by_case(case_id)

        # Collect all dated events
        events: List[Dict[str, Any]] = []

        for ev in evidence:
            if ev.get("source_date"):
                events.append({
                    "date": ev["source_date"],
                    "type": "evidence",
                    "title": ev["title"],
                    "description": ev.get("summary") or ev["evidence_type"],
                    "related_id": ev["id"],
                })

        for ent in entities:
            if ent.get("first_seen"):
                events.append({
                    "date": ent["first_seen"],
                    "type": "entity",
                    "title": f"{ent['entity_type']}: {ent['name']}",
                    "description": ent.get("description") or "First appearance",
                    "related_id": ent["id"],
                })

        # Hypothesis snapshot evolution
        snapshot_events: List[Dict[str, Any]] = []
        for hyp in hypotheses:
            snapshots = await self._db.list_snapshots_by_hypothesis(hyp["id"])
            for snap in snapshots:
                snapshot_events.append({
                    "date": snap["created_at"],
                    "type": "hypothesis_update",
                    "title": f"{hyp['title']} -> {snap['score']:.0f}/100",
                    "description": snap.get("reasoning") or "",
                    "related_id": snap["id"],
                })

        events.extend(snapshot_events)

        # Sort chronologically
        events.sort(key=lambda e: e.get("date") or "9999-12-31")

        now = datetime.utcnow().isoformat()

        return {
            "case_info": {
                "id": case["id"],
                "name": case["name"],
                "reference": case.get("reference"),
            },
            "generated_at": now,
            "sections": {
                "events": events,
                "total_events": len(events),
                "date_range": {
                    "earliest": events[0]["date"] if events else None,
                    "latest": events[-1]["date"] if events else None,
                },
            },
        }

    # ------------------------------------------------------------------
    # Private helpers
    # ------------------------------------------------------------------

    async def _load_monitoring_results(
        self,
        case_id: str,
    ) -> List[Dict[str, Any]]:
        """Load monitoring results for a case."""
        results = await self._db.list_results_by_case(case_id, limit=50)
        return [
            {
                "title": r.get("title"),
                "url": r.get("url"),
                "snippet": r.get("snippet"),
                "source_engine": r.get("source_engine"),
                "relevance_score": r.get("relevance_score"),
                "found_at": r.get("found_at"),
            }
            for r in results
        ]

    @staticmethod
    def _build_dossier_text(
        case: Dict[str, Any],
        evidence: List[Dict[str, Any]],
        entities: List[Dict[str, Any]],
    ) -> str:
        """Build a textual dossier for LLM consumption."""
        parts = [
            f"# Dossier: {case['name']}",
            f"Reference: {case.get('reference', 'N/A')}",
            f"Status: {case['status']}",
            f"Cree le: {case['created_at']}",
            "",
            "## Preuves",
        ]

        for i, ev in enumerate(evidence, 1):
            parts.append(
                f"{i}. [{ev['evidence_type']}] {ev['title']}\n"
                f"   Source: {ev.get('source', 'N/A')} | "
                f"Date: {ev.get('source_date', 'N/A')} | "
                f"Fiabilite: {ev.get('reliability', '?')}/100\n"
                f"   Resume: {ev.get('summary', 'Pas de resume')}"
            )

        parts.append("")
        parts.append("## Entites")

        for ent in entities:
            aliases_str = (
                f" (alias: {', '.join(ent['aliases'])})"
                if ent.get("aliases")
                else ""
            )
            parts.append(
                f"- [{ent['entity_type']}] {ent['name']}{aliases_str}: "
                f"{ent.get('description', 'N/A')}"
            )

        return "\n".join(parts)

    @staticmethod
    def _build_hypotheses_text(
        hypotheses: List[Dict[str, Any]],
    ) -> str:
        """Build hypotheses text for the LLM prompt."""
        if not hypotheses:
            return "Aucune hypothese formulee."

        parts = []
        for i, hyp in enumerate(hypotheses, 1):
            parts.append(
                f"{i}. {hyp['title']} (score: {hyp['current_score']:.0f}/100, "
                f"status: {hyp['status']})\n"
                f"   {hyp['description']}"
            )
        return "\n".join(parts)

    @staticmethod
    def _build_key_evidence_text(
        evidence: List[Dict[str, Any]],
    ) -> str:
        """Build key evidence summary for the LLM prompt."""
        if not evidence:
            return "Aucune preuve enregistree."

        # Take the top 10 by reliability
        sorted_ev = sorted(
            evidence,
            key=lambda e: e.get("reliability", 0),
            reverse=True,
        )[:10]

        parts = []
        for ev in sorted_ev:
            parts.append(
                f"- [{ev['evidence_type']}] {ev['title']} "
                f"(fiabilite: {ev.get('reliability', '?')}/100)\n"
                f"  {ev.get('summary', 'Pas de resume')}"
            )
        return "\n".join(parts)

    @staticmethod
    def _build_timeline_entries(
        evidence: List[Dict[str, Any]],
        entities: List[Dict[str, Any]],
    ) -> List[Dict[str, Any]]:
        """Build a simple timeline from evidence and entity dates."""
        entries = []

        for ev in evidence:
            if ev.get("source_date"):
                entries.append({
                    "date": ev["source_date"],
                    "type": "evidence",
                    "title": ev["title"],
                    "related_id": ev["id"],
                })

        for ent in entities:
            if ent.get("first_seen"):
                entries.append({
                    "date": ent["first_seen"],
                    "type": "entity",
                    "title": f"{ent['entity_type']}: {ent['name']}",
                    "related_id": ent["id"],
                })

        entries.sort(key=lambda e: e.get("date") or "9999-12-31")
        return entries
