"""
NEXUS -- Analysis API router.

Trigger full / incremental analyses (run in background) and
query run status / history.
"""

from __future__ import annotations

from typing import Optional

from fastapi import APIRouter, BackgroundTasks, Depends, HTTPException, Request
from loguru import logger
from pydantic import BaseModel

from nexus.db.models import AnalysisRun
from nexus.db.sqlite_db import Database, get_db

from nexus.api.deps import get_database

router = APIRouter(tags=["analysis"])


# ------------------------------------------------------------------
# Request body for triggering an analysis
# ------------------------------------------------------------------

class AnalyzeRequest(BaseModel):
    """Optional parameters when triggering an analysis."""
    trigger: str = "manual"
    new_evidence_id: Optional[str] = None


# ------------------------------------------------------------------
# Background task wrapper
# ------------------------------------------------------------------

async def _run_analysis_in_background(
    case_id: str,
    trigger: str,
    new_evidence_id: str | None,
    request_app,  # starlette.applications.Starlette (for app.state)
) -> None:
    """Execute analysis in the background with its own DB connection.

    BackgroundTasks run *after* the response is sent, so the
    request-scoped DB connection is already closed.  We open a
    dedicated connection for the duration of the analysis.
    """
    try:
        from nexus.core.analysis_pipeline import AnalysisPipeline

        async with get_db() as conn:
            db = Database(conn)
            pipeline = AnalysisPipeline(
                db=db,
                router=request_app.state.router,
                chroma=getattr(request_app.state, "chroma", None),
                neo4j=getattr(request_app.state, "neo4j", None),
            )

            if trigger == "manual" and not new_evidence_id:
                await pipeline.run_full_analysis(case_id)
            else:
                await pipeline.run_incremental_analysis(
                    case_id,
                    trigger=trigger,
                    new_evidence_id=new_evidence_id,
                )

        logger.info("Background analysis completed for case {}", case_id)
    except Exception as exc:
        logger.exception("Background analysis FAILED for case {}: {}", case_id, exc)


# ------------------------------------------------------------------
# POST /api/cases/{case_id}/analyze
# ------------------------------------------------------------------

@router.post("/api/cases/{case_id}/analyze", status_code=202)
async def trigger_analysis(
    case_id: str,
    body: AnalyzeRequest | None = None,
    background_tasks: BackgroundTasks = BackgroundTasks(),
    request: Request = None,  # type: ignore[assignment]
    db: Database = Depends(get_database),
) -> dict:
    """Kick off a full or incremental analysis as a background task.

    Returns immediately with 202 Accepted and a preliminary run record
    (status='running').  Poll ``GET /api/analysis/{run_id}`` to track
    progress.
    """
    if body is None:
        body = AnalyzeRequest()

    # Verify the case exists
    case = await db.get_case(case_id)
    if case is None:
        raise HTTPException(status_code=404, detail=f"Case not found: {case_id}")

    # Create a placeholder analysis_run record so the caller can track it
    run_type = "full" if (body.trigger == "manual" and not body.new_evidence_id) else "incremental"
    row = await db.create_analysis_run(
        case_id=case_id,
        run_type=run_type,
        trigger=body.trigger,
    )

    # Schedule the actual work in the background
    background_tasks.add_task(
        _run_analysis_in_background,
        case_id,
        body.trigger,
        body.new_evidence_id,
        request.app,
    )

    return {"run_id": row["id"], "status": "running", "run_type": run_type}


# ------------------------------------------------------------------
# GET /api/analysis/{run_id}
# ------------------------------------------------------------------

@router.get("/api/analysis/{run_id}", response_model=AnalysisRun)
async def get_analysis_run(
    run_id: str,
    db: Database = Depends(get_database),
) -> AnalysisRun:
    """Get the status of an analysis run."""
    row = await db.get_analysis_run(run_id)
    if row is None:
        raise HTTPException(status_code=404, detail=f"Analysis run not found: {run_id}")
    return AnalysisRun(**row)


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/analysis-runs
# ------------------------------------------------------------------

@router.get(
    "/api/cases/{case_id}/analysis-runs",
    response_model=list[AnalysisRun],
)
async def list_analysis_runs(
    case_id: str,
    status: str | None = None,
    limit: int = 50,
    db: Database = Depends(get_database),
) -> list[AnalysisRun]:
    """List analysis run history for a case."""
    rows = await db.list_runs_by_case(case_id, status=status, limit=limit)
    return [AnalysisRun(**r) for r in rows]
