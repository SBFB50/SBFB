"""
NEXUS -- Monitoring API router.

CRUD for monitoring jobs, access to results, and manual trigger
for immediate execution.
"""

from __future__ import annotations

from fastapi import APIRouter, Depends, HTTPException, Request

from nexus.db.models import (
    MonitoringJob,
    MonitoringJobCreate,
    MonitoringJobUpdate,
    MonitoringResult,
)
from nexus.db.sqlite_db import Database

from nexus.api.deps import get_database

router = APIRouter(tags=["monitoring"])


# ------------------------------------------------------------------
# POST /api/cases/{case_id}/monitoring — create a monitoring job
# ------------------------------------------------------------------

@router.post(
    "/api/cases/{case_id}/monitoring",
    response_model=MonitoringJob,
    status_code=201,
)
async def create_monitoring_job(
    case_id: str,
    data: MonitoringJobCreate,
    request: Request,
    db: Database = Depends(get_database),
) -> MonitoringJob:
    """Create a new monitoring job for a case."""
    # Verify case exists
    case = await db.get_case(case_id)
    if case is None:
        raise HTTPException(status_code=404, detail=f"Case not found: {case_id}")

    row = await db.create_monitoring_job(
        case_id=case_id,
        job_type=data.job_type,
        query=data.query,
        entity_id=data.entity_id,
        interval_hours=data.interval_hours,
    )

    # Register the job in the scheduler if available
    scheduler = getattr(request.app.state, "monitoring_scheduler", None)
    if scheduler is not None:
        scheduler.add_job(row["id"], data.interval_hours)

    return MonitoringJob(**row)


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/monitoring — list jobs for a case
# ------------------------------------------------------------------

@router.get(
    "/api/cases/{case_id}/monitoring",
    response_model=list[MonitoringJob],
)
async def list_monitoring_jobs(
    case_id: str,
    active_only: bool = False,
    db: Database = Depends(get_database),
) -> list[MonitoringJob]:
    """List all monitoring jobs for a case."""
    rows = await db.list_jobs_by_case(case_id, active_only=active_only)
    return [MonitoringJob(**r) for r in rows]


# ------------------------------------------------------------------
# PUT /api/monitoring/{job_id} — update a job
# ------------------------------------------------------------------

@router.put(
    "/api/monitoring/{job_id}",
    response_model=MonitoringJob,
)
async def update_monitoring_job(
    job_id: str,
    data: MonitoringJobUpdate,
    request: Request,
    db: Database = Depends(get_database),
) -> MonitoringJob:
    """Update a monitoring job's parameters."""
    existing = await db._get_monitoring_job(job_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Monitoring job not found: {job_id}")

    update_fields = data.model_dump(exclude_unset=True)
    if not update_fields:
        return MonitoringJob(**existing)

    row = await db.update_job(job_id, **update_fields)
    if row is None:
        raise HTTPException(status_code=404, detail=f"Monitoring job not found: {job_id}")

    # Sync scheduler with updated settings
    scheduler = getattr(request.app.state, "monitoring_scheduler", None)
    if scheduler is not None:
        if "is_active" in update_fields and not update_fields["is_active"]:
            scheduler.remove_job(job_id)
        elif "is_active" in update_fields and update_fields["is_active"]:
            scheduler.add_job(job_id, row.get("interval_hours", 24))
        if "interval_hours" in update_fields:
            scheduler.update_job_interval(job_id, update_fields["interval_hours"])

    return MonitoringJob(**row)


# ------------------------------------------------------------------
# DELETE /api/monitoring/{job_id} — delete a job
# ------------------------------------------------------------------

@router.delete("/api/monitoring/{job_id}", status_code=204)
async def delete_monitoring_job(
    job_id: str,
    request: Request,
    db: Database = Depends(get_database),
) -> None:
    """Delete a monitoring job and remove it from the scheduler."""
    existing = await db._get_monitoring_job(job_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Monitoring job not found: {job_id}")

    # Remove from scheduler first
    scheduler = getattr(request.app.state, "monitoring_scheduler", None)
    if scheduler is not None:
        scheduler.remove_job(job_id)

    deleted = await db.delete_job(job_id)
    if not deleted:
        raise HTTPException(status_code=404, detail=f"Monitoring job not found: {job_id}")


# ------------------------------------------------------------------
# POST /api/monitoring/{job_id}/run — force immediate execution
# ------------------------------------------------------------------

@router.post("/api/monitoring/{job_id}/run", status_code=202)
async def trigger_monitoring_job(
    job_id: str,
    request: Request,
    db: Database = Depends(get_database),
) -> dict:
    """Force an immediate execution of a monitoring job."""
    existing = await db._get_monitoring_job(job_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Monitoring job not found: {job_id}")

    scheduler = getattr(request.app.state, "monitoring_scheduler", None)
    if scheduler is None:
        raise HTTPException(
            status_code=503,
            detail="Monitoring scheduler not available",
        )

    scheduler.trigger_job(job_id)
    return {"status": "triggered", "job_id": job_id}


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/monitoring/results — list results for a case
# ------------------------------------------------------------------

@router.get(
    "/api/cases/{case_id}/monitoring/results",
    response_model=list[MonitoringResult],
)
async def list_monitoring_results(
    case_id: str,
    limit: int = 200,
    db: Database = Depends(get_database),
) -> list[MonitoringResult]:
    """List all monitoring results for a case, newest first."""
    rows = await db.list_results_by_case(case_id, limit=limit)
    return [MonitoringResult(**r) for r in rows]


# ------------------------------------------------------------------
# GET /api/monitoring/results/{result_id} — get a single result
# ------------------------------------------------------------------

@router.get(
    "/api/monitoring/results/{result_id}",
    response_model=MonitoringResult,
)
async def get_monitoring_result(
    result_id: str,
    db: Database = Depends(get_database),
) -> MonitoringResult:
    """Get a single monitoring result by ID."""
    row = await db.get_monitoring_result(result_id)
    if row is None:
        raise HTTPException(
            status_code=404,
            detail=f"Monitoring result not found: {result_id}",
        )
    return MonitoringResult(**row)


# ------------------------------------------------------------------
# POST /api/monitoring/results/{result_id}/ingest — convert to evidence
# ------------------------------------------------------------------

@router.post("/api/monitoring/results/{result_id}/ingest", status_code=201)
async def ingest_monitoring_result(
    result_id: str,
    db: Database = Depends(get_database),
) -> dict:
    """Convert a monitoring result into a piece of evidence.

    Marks the result as reviewed and creates an evidence record
    linked to the same case.
    """
    result = await db.get_monitoring_result(result_id)
    if result is None:
        raise HTTPException(
            status_code=404,
            detail=f"Monitoring result not found: {result_id}",
        )

    # Create evidence from the monitoring result
    evidence = await db.create_evidence(
        case_id=result["case_id"],
        title=result.get("title") or "Resultat de monitoring",
        evidence_type="url",
        source=result.get("url"),
        raw_text=result.get("snippet"),
        reliability=int(result.get("relevance_score") or 30),
        metadata={
            "from_monitoring": True,
            "monitoring_result_id": result_id,
            "source_engine": result.get("source_engine"),
        },
        status="pending",
    )

    # Mark the monitoring result as reviewed
    await db.update_monitoring_result(result_id, reviewed=1)

    return {
        "evidence_id": evidence["id"],
        "monitoring_result_id": result_id,
        "status": "ingested",
    }
