"""
NEXUS -- Suspects API router.

Endpoints for suspect scoring, profile evaluation, score evolution
and manual updates.  Heavy operations (score-all, evaluate-profile)
run as BackgroundTasks to avoid blocking the HTTP response.
"""

from __future__ import annotations

from fastapi import APIRouter, BackgroundTasks, Depends, HTTPException, Request
from loguru import logger

from nexus.db.models import Suspect, SuspectCreate, SuspectSnapshot, SuspectUpdate
from nexus.db.sqlite_db import Database, get_db

from nexus.api.deps import get_database

router = APIRouter(prefix="/api", tags=["suspects"])


# ------------------------------------------------------------------
# Background task helpers
# ------------------------------------------------------------------

async def _score_all_suspects_bg(case_id: str, request_app) -> None:
    """Run suspect scoring in the background with its own DB connection."""
    try:
        from nexus.core.suspect_scorer import SuspectScorer

        async with get_db() as conn:
            db = Database(conn)
            scorer = SuspectScorer(
                db,
                request_app.state.router,
                neo4j=getattr(request_app.state, "neo4j", None),
            )
            results = await scorer.score_all_suspects(case_id, trigger="manual")
            logger.info(
                "Background suspect scoring completed for case {} ({} suspects)",
                case_id, len(results),
            )
    except Exception:
        logger.exception("Background suspect scoring FAILED for case {}", case_id)


async def _evaluate_profile_bg(suspect_id: str, request_app) -> None:
    """Run profile evaluation in the background with its own DB connection."""
    try:
        from nexus.core.suspect_scorer import SuspectScorer

        async with get_db() as conn:
            db = Database(conn)
            # Load suspect to get case_id and entity_id
            suspect = await db.get_suspect(suspect_id)
            if suspect is None:
                logger.error("Suspect not found for profile evaluation: {}", suspect_id)
                return

            scorer = SuspectScorer(
                db,
                request_app.state.router,
                neo4j=getattr(request_app.state, "neo4j", None),
            )
            result = await scorer.evaluate_profile(
                suspect["case_id"], suspect["entity_id"]
            )
            logger.info(
                "Background profile evaluation completed for suspect {} (profile_score={})",
                suspect_id[:8], result.get("profile_score", "?"),
            )
    except Exception:
        logger.exception("Background profile evaluation FAILED for suspect {}", suspect_id)


# ====================================================================
# LIST suspects by case
# ====================================================================

@router.get(
    "/cases/{case_id}/suspects",
    response_model=list[Suspect],
)
async def list_suspects(
    case_id: str,
    db: Database = Depends(get_database),
) -> list[Suspect]:
    """List all suspects for a case, sorted by suspicion_score descending."""
    case = await db.get_case(case_id)
    if case is None:
        raise HTTPException(status_code=404, detail=f"Case not found: {case_id}")

    rows = await db.list_suspects_by_case(case_id)
    return [Suspect(**r) for r in rows]


# ====================================================================
# SCORE all suspects in a case (background task)
# ====================================================================

@router.post(
    "/cases/{case_id}/suspects/score",
    status_code=202,
)
async def score_all_suspects(
    case_id: str,
    background_tasks: BackgroundTasks,
    request: Request,
    db: Database = Depends(get_database),
) -> dict:
    """Score all person entities in a case as suspects (background task).

    Creates suspect records for any person entity that does not already
    have one.  Calculates composite scores from graph, evidence,
    contradictions, profile and hypotheses.
    """
    case = await db.get_case(case_id)
    if case is None:
        raise HTTPException(status_code=404, detail=f"Case not found: {case_id}")

    background_tasks.add_task(_score_all_suspects_bg, case_id, request.app)

    return {
        "case_id": case_id,
        "status": "scoring_started",
        "message": "Scoring de tous les suspects lance en arriere-plan",
    }


# ====================================================================
# EVALUATE profile via LLM (background task)
# ====================================================================

@router.post(
    "/suspects/{suspect_id}/evaluate-profile",
    status_code=202,
)
async def evaluate_profile(
    suspect_id: str,
    background_tasks: BackgroundTasks,
    request: Request,
    db: Database = Depends(get_database),
) -> dict:
    """Evaluate suspect profile using LLM (motive, alibi, record).

    Sends evidence mentioning this person to the nexus 26B model
    for deep analysis.  Runs as a background task.
    """
    suspect = await db.get_suspect(suspect_id)
    if suspect is None:
        raise HTTPException(status_code=404, detail=f"Suspect not found: {suspect_id}")

    background_tasks.add_task(_evaluate_profile_bg, suspect_id, request.app)

    return {
        "suspect_id": suspect_id,
        "status": "evaluation_started",
        "message": "Evaluation du profil lancee en arriere-plan",
    }


# ====================================================================
# GET suspect evolution (score history)
# ====================================================================

@router.get(
    "/suspects/{suspect_id}/evolution",
)
async def get_evolution(
    suspect_id: str,
    request: Request,
    db: Database = Depends(get_database),
) -> list[dict]:
    """Get time-series data for suspect score evolution.

    Returns [{date, score, factors, trigger}] sorted chronologically.
    """
    from nexus.core.suspect_scorer import SuspectScorer

    suspect = await db.get_suspect(suspect_id)
    if suspect is None:
        raise HTTPException(status_code=404, detail=f"Suspect not found: {suspect_id}")

    scorer = SuspectScorer(
        db,
        request.app.state.router,
        neo4j=getattr(request.app.state, "neo4j", None),
    )
    return await scorer.get_evolution(suspect_id)


# ====================================================================
# UPDATE suspect (notes, alibi, motive, etc.)
# ====================================================================

@router.put(
    "/suspects/{suspect_id}",
    response_model=Suspect,
)
async def update_suspect(
    suspect_id: str,
    data: SuspectUpdate,
    db: Database = Depends(get_database),
) -> Suspect:
    """Update suspect metadata (notes, alibi_status, motive, etc.).

    Does not recalculate scores -- use POST .../score for that.
    """
    existing = await db.get_suspect(suspect_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Suspect not found: {suspect_id}")

    fields = data.model_dump(exclude_unset=True)
    if not fields:
        return Suspect(**existing)

    updated = await db.update_suspect(suspect_id, **fields)
    return Suspect(**updated)


# ====================================================================
# GET single suspect details
# ====================================================================

@router.get(
    "/suspects/{suspect_id}",
    response_model=Suspect,
)
async def get_suspect(
    suspect_id: str,
    db: Database = Depends(get_database),
) -> Suspect:
    """Get details of a single suspect."""
    row = await db.get_suspect(suspect_id)
    if row is None:
        raise HTTPException(status_code=404, detail=f"Suspect not found: {suspect_id}")
    return Suspect(**row)


# ====================================================================
# GET suspect snapshots
# ====================================================================

@router.get(
    "/suspects/{suspect_id}/snapshots",
    response_model=list[SuspectSnapshot],
)
async def list_snapshots(
    suspect_id: str,
    db: Database = Depends(get_database),
) -> list[SuspectSnapshot]:
    """Get the full snapshot history for a suspect."""
    existing = await db.get_suspect(suspect_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Suspect not found: {suspect_id}")

    rows = await db.list_suspect_snapshots(suspect_id)
    return [SuspectSnapshot(**r) for r in rows]
