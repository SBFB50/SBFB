"""
NEXUS -- Vision API router.

Endpoints for visual analysis of evidence images:
- Analyze a single evidence image
- Analyze all images in a case
- Direct image upload and description
- Compare two evidence images
- List visual entities for a case
"""

from __future__ import annotations

import tempfile
from pathlib import Path
from typing import Any, Dict, List

from fastapi import APIRouter, Depends, HTTPException, UploadFile, File, Form
from loguru import logger

from nexus.api.deps import get_database, get_image_analyzer
from nexus.core.image_analyzer import ImageAnalyzer
from nexus.db.sqlite_db import Database

router = APIRouter(prefix="/api", tags=["vision"])


# ------------------------------------------------------------------
# POST /api/evidence/{evidence_id}/analyze-image
# ------------------------------------------------------------------

@router.post("/evidence/{evidence_id}/analyze-image")
async def analyze_evidence_image(
    evidence_id: str,
    db: Database = Depends(get_database),
    analyzer: ImageAnalyzer = Depends(get_image_analyzer),
) -> Dict[str, Any]:
    """Analyze the image attached to an evidence item.

    Runs the full visual pipeline: description, entity extraction,
    scene analysis, embedding, and saves results.
    """
    evidence = await db.get_evidence(evidence_id)
    if evidence is None:
        raise HTTPException(status_code=404, detail=f"Evidence not found: {evidence_id}")

    if evidence.get("evidence_type") != "image":
        raise HTTPException(
            status_code=400,
            detail=f"Evidence '{evidence_id}' is type '{evidence.get('evidence_type')}', not 'image'",
        )

    file_path = evidence.get("file_path")
    if not file_path or not Path(file_path).exists():
        raise HTTPException(
            status_code=400,
            detail=f"Image file not found at: {file_path}",
        )

    result = await analyzer.process_evidence_image(
        case_id=evidence["case_id"],
        evidence_id=evidence_id,
        image_path=file_path,
    )
    return result


# ------------------------------------------------------------------
# POST /api/cases/{case_id}/analyze-images
# ------------------------------------------------------------------

@router.post("/cases/{case_id}/analyze-images")
async def analyze_all_case_images(
    case_id: str,
    db: Database = Depends(get_database),
    analyzer: ImageAnalyzer = Depends(get_image_analyzer),
) -> Dict[str, Any]:
    """Analyze ALL image evidence in a case.

    Iterates through all evidence items of type 'image' and runs
    the full visual pipeline on each.
    """
    # Verify case exists
    case = await db.get_case(case_id)
    if case is None:
        raise HTTPException(status_code=404, detail=f"Case not found: {case_id}")

    all_evidence = await db.list_evidence_by_case(case_id)
    image_evidence = [
        e for e in all_evidence
        if e.get("evidence_type") == "image" and e.get("file_path")
    ]

    if not image_evidence:
        return {
            "case_id": case_id,
            "images_found": 0,
            "results": [],
            "message": "Aucune preuve image trouvee dans ce dossier.",
        }

    results: List[Dict[str, Any]] = []
    errors: List[Dict[str, Any]] = []

    for ev in image_evidence:
        file_path = ev["file_path"]
        if not Path(file_path).exists():
            errors.append({
                "evidence_id": ev["id"],
                "error": f"Fichier introuvable: {file_path}",
            })
            continue

        try:
            result = await analyzer.process_evidence_image(
                case_id=case_id,
                evidence_id=ev["id"],
                image_path=file_path,
            )
            results.append(result)
        except Exception as exc:
            logger.error(
                "Failed to analyze image evidence {}: {}",
                ev["id"],
                exc,
            )
            errors.append({
                "evidence_id": ev["id"],
                "error": str(exc),
            })

    return {
        "case_id": case_id,
        "images_found": len(image_evidence),
        "images_processed": len(results),
        "results": results,
        "errors": errors,
    }


# ------------------------------------------------------------------
# POST /api/vision/describe  (direct upload)
# ------------------------------------------------------------------

@router.post("/vision/describe")
async def describe_uploaded_image(
    file: UploadFile = File(...),
    analyzer: ImageAnalyzer = Depends(get_image_analyzer),
) -> Dict[str, Any]:
    """Upload an image and get a direct description.

    This endpoint does NOT store the image as evidence; it just
    returns the visual analysis.  Useful for quick checks.
    """
    # Save upload to a temp file
    suffix = Path(file.filename or "image.jpg").suffix or ".jpg"
    with tempfile.NamedTemporaryFile(delete=False, suffix=suffix) as tmp:
        content = await file.read()
        tmp.write(content)
        tmp_path = Path(tmp.name)

    try:
        description = await analyzer.describe_image(tmp_path)
        entities = await analyzer.extract_entities_from_image(tmp_path)

        return {
            "filename": file.filename,
            "description": description,
            "entities": entities,
        }
    finally:
        # Cleanup temp file
        try:
            tmp_path.unlink()
        except OSError:
            pass


# ------------------------------------------------------------------
# POST /api/vision/compare
# ------------------------------------------------------------------

@router.post("/vision/compare")
async def compare_evidence_images(
    evidence_id_1: str = Form(...),
    evidence_id_2: str = Form(...),
    db: Database = Depends(get_database),
    analyzer: ImageAnalyzer = Depends(get_image_analyzer),
) -> Dict[str, Any]:
    """Compare two evidence images side by side.

    Both evidence items must be of type 'image' with valid file paths.
    """
    ev1 = await db.get_evidence(evidence_id_1)
    ev2 = await db.get_evidence(evidence_id_2)

    if ev1 is None:
        raise HTTPException(status_code=404, detail=f"Evidence not found: {evidence_id_1}")
    if ev2 is None:
        raise HTTPException(status_code=404, detail=f"Evidence not found: {evidence_id_2}")

    for ev, eid in [(ev1, evidence_id_1), (ev2, evidence_id_2)]:
        if ev.get("evidence_type") != "image":
            raise HTTPException(
                status_code=400,
                detail=f"Evidence '{eid}' is not an image (type: {ev.get('evidence_type')})",
            )
        fp = ev.get("file_path")
        if not fp or not Path(fp).exists():
            raise HTTPException(
                status_code=400,
                detail=f"Image file not found for evidence '{eid}'",
            )

    result = await analyzer.compare_images(
        ev1["file_path"],
        ev2["file_path"],
    )
    result["evidence_id_1"] = evidence_id_1
    result["evidence_id_2"] = evidence_id_2
    return result


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/visual-entities
# ------------------------------------------------------------------

@router.get("/cases/{case_id}/visual-entities")
async def list_visual_entities(
    case_id: str,
    db: Database = Depends(get_database),
) -> List[Dict[str, Any]]:
    """List all entities extracted from images for a case.

    Filters entities whose metadata contains
    ``"source": "visual_extraction"``.
    """
    all_entities = await db.list_entities_by_case(case_id)
    visual = []
    for ent in all_entities:
        meta = ent.get("metadata")
        if isinstance(meta, dict) and meta.get("source") == "visual_extraction":
            visual.append(ent)
    return visual
