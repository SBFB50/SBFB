"""
NEXUS -- Hypotheses API router.

CRUD endpoints for hypotheses, evaluation pipelines (background tasks),
snapshot history, time-series evolution, contradiction detection
and testimony comparison.

Heavy operations (generate, evaluate, evaluate-all) run as
BackgroundTasks to avoid blocking the HTTP response.
"""

from __future__ import annotations

from typing import Optional

from fastapi import APIRouter, BackgroundTasks, Depends, HTTPException, Query, Request
from loguru import logger
from pydantic import BaseModel, Field

from nexus.db.models import (
    Hypothesis,
    HypothesisCreate,
    HypothesisSnapshot,
    HypothesisUpdate,
)
from nexus.db.sqlite_db import Database, get_db

from nexus.api.deps import get_database, paginated_response

router = APIRouter(prefix="/api", tags=["hypotheses"])


# ------------------------------------------------------------------
# Request bodies
# ------------------------------------------------------------------

class CompareTestimoniesRequest(BaseModel):
    """Body for POST /cases/{case_id}/compare-testimonies."""
    evidence_ids: list[str] = Field(
        ..., min_length=2, description="At least 2 evidence IDs to compare"
    )


class MergeHypothesesRequest(BaseModel):
    """Body for POST /cases/{case_id}/hypotheses/merge."""
    hypothesis_ids: list[str] = Field(
        ..., min_length=2, description="At least 2 hypothesis IDs to merge"
    )
    new_title: str
    new_description: str


# ------------------------------------------------------------------
# Background task helpers
# ------------------------------------------------------------------

async def _generate_hypotheses_bg(case_id: str, request_app) -> None:
    """Run hypothesis generation in the background with its own DB connection."""
    try:
        from nexus.core.hypothesis_engine import HypothesisEngine

        async with get_db() as conn:
            db = Database(conn)
            engine = HypothesisEngine(
                db,
                request_app.state.router,
                chroma=getattr(request_app.state, "chroma", None),
                neo4j=getattr(request_app.state, "neo4j", None),
            )
            results = await engine.generate_hypotheses(case_id)
            logger.info(
                "Background hypothesis generation completed for case {} ({} hypotheses)",
                case_id, len(results),
            )
    except Exception as exc:
        logger.exception("Background hypothesis generation FAILED for case {}: {}", case_id, exc)


async def _evaluate_hypothesis_bg(hypothesis_id: str, request_app) -> None:
    """Run single hypothesis evaluation in the background."""
    try:
        from nexus.core.hypothesis_engine import HypothesisEngine

        async with get_db() as conn:
            db = Database(conn)
            engine = HypothesisEngine(
                db,
                request_app.state.router,
                chroma=getattr(request_app.state, "chroma", None),
                neo4j=getattr(request_app.state, "neo4j", None),
            )
            snapshot = await engine.evaluate_hypothesis(hypothesis_id, trigger="manual")
            logger.info(
                "Background evaluation completed for hypothesis {} (score={:.1f})",
                hypothesis_id[:8], snapshot.get("score", 0),
            )
    except Exception as exc:
        logger.exception("Background evaluation FAILED for hypothesis {}: {}", hypothesis_id[:8], exc)


async def _evaluate_all_bg(case_id: str, request_app) -> None:
    """Run evaluate-all in the background."""
    try:
        from nexus.core.hypothesis_engine import HypothesisEngine

        async with get_db() as conn:
            db = Database(conn)
            engine = HypothesisEngine(
                db,
                request_app.state.router,
                chroma=getattr(request_app.state, "chroma", None),
                neo4j=getattr(request_app.state, "neo4j", None),
            )
            snapshots = await engine.evaluate_all(case_id)
            logger.info(
                "Background evaluate-all completed for case {} ({} snapshots)",
                case_id, len(snapshots),
            )
    except Exception as exc:
        logger.exception("Background evaluate-all FAILED for case {}: {}", case_id, exc)


# ====================================================================
# CRUD: Hypotheses
# ====================================================================

# ------------------------------------------------------------------
# POST /api/cases/{case_id}/hypotheses — create manually
# ------------------------------------------------------------------

@router.post(
    "/cases/{case_id}/hypotheses",
    response_model=Hypothesis,
    status_code=201,
)
async def create_hypothesis(
    case_id: str,
    data: HypothesisCreate,
    db: Database = Depends(get_database),
) -> Hypothesis:
    """Create a hypothesis manually."""
    # Verify the case exists
    case = await db.get_case(case_id)
    if case is None:
        raise HTTPException(status_code=404, detail=f"Case not found: {case_id}")

    # Override case_id from path
    row = await db.create_hypothesis(
        case_id=case_id,
        title=data.title,
        description=data.description,
        status=data.status,
        current_score=data.current_score,
    )

    # Create initial snapshot
    await db.create_hypothesis_snapshot(
        hypothesis_id=row["id"],
        score=data.current_score,
        reasoning="Creation manuelle",
        trigger="manual",
        model_used="user",
    )

    return Hypothesis(**row)


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/hypotheses — list
# ------------------------------------------------------------------

@router.get(
    "/cases/{case_id}/hypotheses",
)
async def list_hypotheses(
    case_id: str,
    status: Optional[str] = Query(default=None, description="Filter by status"),
    limit: int = Query(100, ge=1, le=1000),
    offset: int = Query(0, ge=0),
    db: Database = Depends(get_database),
):
    """List hypotheses for a case, optionally filtered by status, with pagination."""
    all_rows = await db.list_hypotheses_by_case(case_id, status=status, limit=100_000)

    return paginated_response(
        all_rows, offset, limit,
        serializer=lambda r: Hypothesis(**r).model_dump(mode="json"),
    )


# ------------------------------------------------------------------
# GET /api/hypotheses/{hyp_id} — details
# ------------------------------------------------------------------

@router.get(
    "/hypotheses/{hyp_id}",
    response_model=Hypothesis,
)
async def get_hypothesis(
    hyp_id: str,
    db: Database = Depends(get_database),
) -> Hypothesis:
    """Get details of a single hypothesis."""
    row = await db.get_hypothesis(hyp_id)
    if row is None:
        raise HTTPException(status_code=404, detail=f"Hypothesis not found: {hyp_id}")
    return Hypothesis(**row)


# ------------------------------------------------------------------
# PUT /api/hypotheses/{hyp_id} — update
# ------------------------------------------------------------------

@router.put(
    "/hypotheses/{hyp_id}",
    response_model=Hypothesis,
)
async def update_hypothesis(
    hyp_id: str,
    data: HypothesisUpdate,
    db: Database = Depends(get_database),
) -> Hypothesis:
    """Update a hypothesis (title, description, status, score)."""
    existing = await db.get_hypothesis(hyp_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Hypothesis not found: {hyp_id}")

    fields = data.model_dump(exclude_unset=True)
    if not fields:
        return Hypothesis(**existing)

    updated = await db.update_hypothesis(hyp_id, **fields)
    return Hypothesis(**updated)


# ------------------------------------------------------------------
# DELETE /api/hypotheses/{hyp_id} — archive
# ------------------------------------------------------------------

@router.delete(
    "/hypotheses/{hyp_id}",
    status_code=200,
)
async def delete_hypothesis(
    hyp_id: str,
    db: Database = Depends(get_database),
) -> dict:
    """Archive a hypothesis by setting its status to 'archived'.

    We do not hard-delete because NEXUS is a persistent system --
    every data point matters for historical analysis.  If a true
    delete is needed, it can be done directly in SQLite.
    """
    existing = await db.get_hypothesis(hyp_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Hypothesis not found: {hyp_id}")

    # Soft-delete: mark as archived via status update
    # The HypothesisStatus type does not include 'archived', so we use 'refuted'
    # as the closest semantic match for a removed hypothesis.
    # If the status field is extended later to include 'archived', switch to that.
    await db.update_hypothesis(hyp_id, status="refuted")

    return {"detail": f"Hypothesis {hyp_id} archived (status set to 'refuted')"}


# ====================================================================
# Evaluation endpoints (background tasks)
# ====================================================================

# ------------------------------------------------------------------
# POST /api/hypotheses/{hyp_id}/evaluate — force re-evaluation
# ------------------------------------------------------------------

@router.post(
    "/hypotheses/{hyp_id}/evaluate",
    status_code=202,
)
async def evaluate_hypothesis(
    hyp_id: str,
    background_tasks: BackgroundTasks,
    request: Request,
    db: Database = Depends(get_database),
) -> dict:
    """Force re-evaluation of a single hypothesis (background task)."""
    existing = await db.get_hypothesis(hyp_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Hypothesis not found: {hyp_id}")

    background_tasks.add_task(_evaluate_hypothesis_bg, hyp_id, request.app)

    return {
        "hypothesis_id": hyp_id,
        "status": "evaluation_started",
        "message": "Re-evaluation lancee en arriere-plan",
    }


# ------------------------------------------------------------------
# GET /api/hypotheses/{hyp_id}/snapshots — snapshot history
# ------------------------------------------------------------------

@router.get(
    "/hypotheses/{hyp_id}/snapshots",
    response_model=list[HypothesisSnapshot],
)
async def list_snapshots(
    hyp_id: str,
    db: Database = Depends(get_database),
) -> list[HypothesisSnapshot]:
    """Get the full snapshot history for a hypothesis."""
    # Verify hypothesis exists
    existing = await db.get_hypothesis(hyp_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Hypothesis not found: {hyp_id}")

    rows = await db.list_snapshots_by_hypothesis(hyp_id)
    return [HypothesisSnapshot(**r) for r in rows]


# ------------------------------------------------------------------
# GET /api/hypotheses/{hyp_id}/evolution — time-series data
# ------------------------------------------------------------------

@router.get(
    "/hypotheses/{hyp_id}/evolution",
)
async def get_evolution(
    hyp_id: str,
    request: Request,
    db: Database = Depends(get_database),
) -> list[dict]:
    """Get time-series data for hypothesis score evolution.

    Returns [{date, score, trigger, model_used}] sorted chronologically.
    """
    from nexus.core.hypothesis_engine import HypothesisEngine

    existing = await db.get_hypothesis(hyp_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Hypothesis not found: {hyp_id}")

    engine = HypothesisEngine(db, request.app.state.router)
    return await engine.get_evolution(hyp_id)


# ====================================================================
# Generation and batch evaluation (background tasks)
# ====================================================================

# ------------------------------------------------------------------
# POST /api/cases/{case_id}/hypotheses/generate — generate via LLM
# ------------------------------------------------------------------

@router.post(
    "/cases/{case_id}/hypotheses/generate",
    status_code=202,
)
async def generate_hypotheses(
    case_id: str,
    background_tasks: BackgroundTasks,
    request: Request,
    db: Database = Depends(get_database),
) -> dict:
    """Generate hypotheses for a case using the LLM (background task)."""
    case = await db.get_case(case_id)
    if case is None:
        raise HTTPException(status_code=404, detail=f"Case not found: {case_id}")

    background_tasks.add_task(_generate_hypotheses_bg, case_id, request.app)

    return {
        "case_id": case_id,
        "status": "generation_started",
        "message": "Generation d'hypotheses lancee en arriere-plan",
    }


# ------------------------------------------------------------------
# POST /api/cases/{case_id}/evaluate-all — re-evaluate all
# ------------------------------------------------------------------

@router.post(
    "/cases/{case_id}/evaluate-all",
    status_code=202,
)
async def evaluate_all_hypotheses(
    case_id: str,
    background_tasks: BackgroundTasks,
    request: Request,
    db: Database = Depends(get_database),
) -> dict:
    """Re-evaluate all active hypotheses for a case (background task)."""
    case = await db.get_case(case_id)
    if case is None:
        raise HTTPException(status_code=404, detail=f"Case not found: {case_id}")

    background_tasks.add_task(_evaluate_all_bg, case_id, request.app)

    return {
        "case_id": case_id,
        "status": "evaluate_all_started",
        "message": "Re-evaluation de toutes les hypotheses lancee en arriere-plan",
    }


# ====================================================================
# Merge hypotheses
# ====================================================================

@router.post(
    "/cases/{case_id}/hypotheses/merge",
    response_model=Hypothesis,
    status_code=201,
)
async def merge_hypotheses(
    case_id: str,
    body: MergeHypothesesRequest,
    request: Request,
    db: Database = Depends(get_database),
) -> Hypothesis:
    """Merge multiple hypotheses into a single new one."""
    from nexus.core.hypothesis_engine import HypothesisEngine

    case = await db.get_case(case_id)
    if case is None:
        raise HTTPException(status_code=404, detail=f"Case not found: {case_id}")

    engine = HypothesisEngine(db, request.app.state.router)

    try:
        result = await engine.merge_hypotheses(
            hyp_ids=body.hypothesis_ids,
            new_title=body.new_title,
            new_description=body.new_description,
        )
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))

    return Hypothesis(**result)


# ====================================================================
# Contradiction detection
# ====================================================================

# ------------------------------------------------------------------
# GET /api/cases/{case_id}/contradictions
# ------------------------------------------------------------------

@router.get(
    "/cases/{case_id}/contradictions",
)
async def list_contradictions(
    case_id: str,
    db: Database = Depends(get_database),
    detect: bool = False,
    request: Request = None,
) -> list[dict]:
    """List persisted contradictions for a case.

    If ``detect=true`` query parameter is set, runs the LLM-based
    contradiction detector first, then returns all persisted results.
    """
    case = await db.get_case(case_id)
    if case is None:
        raise HTTPException(status_code=404, detail=f"Case not found: {case_id}")

    if detect and request is not None:
        from nexus.core.contradiction_detector import ContradictionDetector
        detector = ContradictionDetector(db, request.app.state.router)
        results = await detector.detect_contradictions(case_id)
        # Persist newly detected contradictions
        for c in results:
            try:
                await db.create_contradiction(
                    case_id=case_id,
                    evidence_1_id=c.get("evidence_1_id"),
                    evidence_2_id=c.get("evidence_2_id"),
                    evidence_1_title=c.get("evidence_1_title"),
                    evidence_2_title=c.get("evidence_2_title"),
                    contradiction_type=c.get("type", "factual"),
                    severity=c.get("severity", "medium"),
                    description=c.get("description", ""),
                    likely_correct=c.get("likely_correct"),
                    reasoning=c.get("reasoning"),
                )
            except Exception as exc:
                logger.debug("Contradiction storage skipped (duplicate?): {}", exc)

    return await db.list_contradictions_by_case(case_id)


# ------------------------------------------------------------------
# POST /api/cases/{case_id}/compare-testimonies
# ------------------------------------------------------------------

@router.post(
    "/cases/{case_id}/compare-testimonies",
)
async def compare_testimonies(
    case_id: str,
    body: CompareTestimoniesRequest,
    request: Request,
    db: Database = Depends(get_database),
) -> dict:
    """Compare specific testimonies for convergences and divergences."""
    from nexus.core.contradiction_detector import ContradictionDetector

    case = await db.get_case(case_id)
    if case is None:
        raise HTTPException(status_code=404, detail=f"Case not found: {case_id}")

    detector = ContradictionDetector(db, request.app.state.router)

    try:
        return await detector.compare_testimonies(case_id, body.evidence_ids)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))
