"""
NEXUS -- Evidence API router.

Upload files (multipart), submit text evidence, list/get/update/delete.
"""

from __future__ import annotations

from fastapi import APIRouter, Depends, HTTPException, UploadFile, File, Form
from pydantic import BaseModel
from typing import Optional

from nexus.db.models import Evidence, EvidenceUpdate
from nexus.db.sqlite_db import Database

from nexus.api.deps import get_database, get_evidence_processor

router = APIRouter(tags=["evidence"])


# ------------------------------------------------------------------
# Request body for text evidence submission
# ------------------------------------------------------------------

class TextEvidenceInput(BaseModel):
    title: str
    text: str
    source: Optional[str] = None


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
    file: UploadFile = File(...),
    title: str = Form(...),
    source: str | None = Form(default=None),
    evidence_type: str | None = Form(default=None),
    processor=Depends(get_evidence_processor),
) -> Evidence:
    """Upload a file as evidence (multipart/form-data).

    The EvidenceProcessor handles file storage, text extraction,
    and initial processing.
    """
    return await processor.process_upload(
        case_id=case_id,
        file=file,
        title=title,
        source=source,
        evidence_type=evidence_type,
    )


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
    body: TextEvidenceInput,
    processor=Depends(get_evidence_processor),
) -> Evidence:
    """Submit text evidence (copy-pasted notes, transcripts, etc.)."""
    return await processor.process_text_input(
        case_id=case_id,
        title=body.title,
        text=body.text,
        source=body.source,
    )


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/evidence
# ------------------------------------------------------------------

@router.get("/api/cases/{case_id}/evidence", response_model=list[Evidence])
async def list_evidence(
    case_id: str,
    status: str | None = None,
    evidence_type: str | None = None,
    db: Database = Depends(get_database),
) -> list[Evidence]:
    """List all evidence for a case, with optional filters."""
    rows = await db.list_evidence_by_case(case_id, status=status)

    # The DB method only filters by status; apply evidence_type in-memory
    if evidence_type:
        rows = [r for r in rows if r.get("evidence_type") == evidence_type]

    return [Evidence(**r) for r in rows]


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
