"""
NEXUS -- Evidence API router.

Upload files (multipart), submit text evidence, list/get/update/delete.
"""

from __future__ import annotations

from fastapi import APIRouter, Depends, HTTPException, Query, Request, UploadFile, File, Form
from loguru import logger
from pydantic import BaseModel
from typing import Optional

from nexus.db.models import Evidence, EvidenceUpdate
from nexus.db.sqlite_db import Database
from nexus.events.types import EventType, NexusEvent

from nexus.api.deps import get_database, get_evidence_processor, paginated_response

router = APIRouter(tags=["evidence"])


# ------------------------------------------------------------------
# Request body for text evidence submission
# ------------------------------------------------------------------

class TextEvidenceInput(BaseModel):
    title: str
    text: str
    source: Optional[str] = None


# ------------------------------------------------------------------
# Helper: notify reactive workers after synchronous processing
# ------------------------------------------------------------------

async def _notify_reactive_workers(
    request: Request,
    case_id: str,
    evidence: Evidence,
) -> None:
    """Publish EVIDENCE_ADDED to the EventBus so downstream workers react.

    EvidenceProcessor already handles entity extraction, summarisation,
    chunking, embeddings, and Neo4j sync synchronously.  We only emit
    EVIDENCE_ADDED -- the reactive chain handles the rest:

      EVIDENCE_ADDED
        -> EntityExtractorWorker -> ENTITY_DISCOVERED (reads existing mentions)
        -> SummarizerWorker      -> EVIDENCE_PROCESSED (confirms status=processed)
           -> ChunkerEmbedWorker -> EVIDENCE_CHUNKED (detects existing chunks)
           -> ContradictionWorker, TimelineWorker, WikiCompilerWorker, Neo4jSync
              -> AnalysisPipelineWorker, HypothesisWorker

    We must NOT emit EVIDENCE_PROCESSED or EVIDENCE_CHUNKED directly here,
    because the workers that bridge EVIDENCE_ADDED -> EVIDENCE_PROCESSED
    and EVIDENCE_PROCESSED -> EVIDENCE_CHUNKED would also emit them,
    causing double-processing of LLM-heavy downstream workers.
    """
    inv_mgr = getattr(request.app.state, "investigation_manager", None)
    if inv_mgr is None:
        return

    bus = inv_mgr.get_event_bus(case_id)
    if bus is None:
        return

    try:
        await bus.publish(NexusEvent(
            event_type=EventType.EVIDENCE_ADDED,
            case_id=case_id,
            payload={
                "evidence_id": evidence.id,
                "title": evidence.title,
                "evidence_type": evidence.evidence_type,
                "source": "user_upload",
            },
            source_worker="api.evidence",
        ))
        logger.info(
            "Evidence upload: published EVIDENCE_ADDED "
            "for {} ({}) on case {}",
            evidence.id[:8], evidence.title[:40], case_id[:8],
        )
    except Exception as exc:
        logger.warning(
            "Evidence upload: failed to publish event for {}: {}",
            evidence.id[:8], exc,
        )


# ------------------------------------------------------------------
# POST /api/cases/{case_id}/evidence  (file upload)
# ------------------------------------------------------------------

@router.post(
    "/api/cases/{case_id}/evidence",
    response_model=Evidence,
    status_code=201,
)
async def upload_evidence(
    case_id: str,
    request: Request,
    file: UploadFile = File(...),
    title: str = Form(...),
    source: str | None = Form(default=None),
    evidence_type: str | None = Form(default=None),
    processor=Depends(get_evidence_processor),
) -> Evidence:
    """Upload a file as evidence (multipart/form-data).

    The EvidenceProcessor handles file storage, text extraction,
    and initial processing.  After processing completes, events are
    published to the EventBus to trigger downstream reactive workers.
    """
    evidence = await processor.process_upload(
        case_id=case_id,
        file=file,
        title=title,
        source=source,
        evidence_type=evidence_type,
    )
    await _notify_reactive_workers(request, case_id, evidence)
    return evidence


# ------------------------------------------------------------------
# POST /api/cases/{case_id}/evidence/text  (JSON body)
# ------------------------------------------------------------------

@router.post(
    "/api/cases/{case_id}/evidence/text",
    response_model=Evidence,
    status_code=201,
)
async def submit_text_evidence(
    case_id: str,
    request: Request,
    body: TextEvidenceInput,
    processor=Depends(get_evidence_processor),
) -> Evidence:
    """Submit text evidence (copy-pasted notes, transcripts, etc.)."""
    evidence = await processor.process_text_input(
        case_id=case_id,
        title=body.title,
        text=body.text,
        source=body.source,
    )
    await _notify_reactive_workers(request, case_id, evidence)
    return evidence


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/evidence
# ------------------------------------------------------------------

@router.get("/api/cases/{case_id}/evidence")
async def list_evidence(
    case_id: str,
    status: str | None = None,
    evidence_type: str | None = None,
    limit: int = Query(100, ge=1, le=1000),
    offset: int = Query(0, ge=0),
    db: Database = Depends(get_database),
):
    """List evidence for a case, with optional filters and pagination."""
    # Fetch all matching rows (DB already supports limit/offset but we need
    # to apply evidence_type filter in-memory, so fetch a large set first)
    all_rows = await db.list_evidence_by_case(case_id, status=status, limit=100_000)

    # The DB method only filters by status; apply evidence_type in-memory
    if evidence_type:
        all_rows = [r for r in all_rows if r.get("evidence_type") == evidence_type]

    return paginated_response(
        all_rows, offset, limit,
        serializer=lambda r: Evidence(**r).model_dump(mode="json"),
    )


# ------------------------------------------------------------------
# GET /api/evidence/{evidence_id}
# ------------------------------------------------------------------

@router.get("/api/evidence/{evidence_id}", response_model=Evidence)
async def get_evidence(
    evidence_id: str,
    db: Database = Depends(get_database),
) -> Evidence:
    """Retrieve a single evidence item by ID."""
    row = await db.get_evidence(evidence_id)
    if row is None:
        raise HTTPException(status_code=404, detail=f"Evidence not found: {evidence_id}")
    return Evidence(**row)


# ------------------------------------------------------------------
# PUT /api/evidence/{evidence_id}
# ------------------------------------------------------------------

@router.put("/api/evidence/{evidence_id}", response_model=Evidence)
async def update_evidence(
    evidence_id: str,
    data: EvidenceUpdate,
    db: Database = Depends(get_database),
) -> Evidence:
    """Update evidence metadata."""
    update_fields = data.model_dump(exclude_unset=True)
    if not update_fields:
        # Nothing to update -- return current state
        row = await db.get_evidence(evidence_id)
        if row is None:
            raise HTTPException(status_code=404, detail=f"Evidence not found: {evidence_id}")
        return Evidence(**row)

    row = await db.update_evidence(evidence_id, **update_fields)
    if row is None:
        raise HTTPException(status_code=404, detail=f"Evidence not found: {evidence_id}")
    return Evidence(**row)


# ------------------------------------------------------------------
# DELETE /api/evidence/{evidence_id}
# ------------------------------------------------------------------

@router.delete("/api/evidence/{evidence_id}", status_code=204)
async def delete_evidence(
    evidence_id: str,
    db: Database = Depends(get_database),
) -> None:
    """Delete an evidence item."""
    deleted = await db.delete_evidence(evidence_id)
    if not deleted:
        raise HTTPException(status_code=404, detail=f"Evidence not found: {evidence_id}")
