"""
NEXUS -- Forensic analysis API router.

Endpoints for:
- Blood Pattern Analysis (BPA): classification, spatter analysis,
  geometric calculations (impact angle, convergence, origin)
- Acoustic forensics: transcription, forensic analysis, event detection
- Trace analysis: classification, comparison
- Auto-analysis: run all forensic analyses on case evidence
"""

from __future__ import annotations

import json
import tempfile
from pathlib import Path
from typing import Any, Dict, List, Optional

from fastapi import APIRouter, Depends, File, Form, HTTPException, UploadFile
from loguru import logger
from pydantic import BaseModel, Field

from nexus.api.deps import (
    get_acoustic_analyzer,
    get_bpa_analyzer,
    get_database,
    get_trace_analyzer,
)
from nexus.db.sqlite_db import Database
from nexus.forensics.acoustic_analysis import AcousticAnalyzer
from nexus.forensics.blood_pattern import BloodPatternAnalyzer
from nexus.forensics.trace_analyzer import TraceAnalyzer

router = APIRouter(prefix="/api/forensics", tags=["forensics"])


# =====================================================================
# Request / Response models
# =====================================================================

class ImpactAngleRequest(BaseModel):
    width: float = Field(..., gt=0, description="Width of the stain (mm)")
    length: float = Field(..., gt=0, description="Length of the stain (mm)")


class StainMeasurement(BaseModel):
    x: float = Field(..., description="X position on surface (mm)")
    y: float = Field(..., description="Y position on surface (mm)")
    direction_degrees: float = Field(
        ..., description="Direction blood came from (degrees, 0=right)"
    )
    width: Optional[float] = Field(None, gt=0, description="Stain width (mm)")
    length: Optional[float] = Field(None, gt=0, description="Stain length (mm)")


class ConvergenceRequest(BaseModel):
    stains: List[StainMeasurement] = Field(
        ..., min_length=2, description="At least 2 stain measurements"
    )


class BPAAnalyzeRequest(BaseModel):
    measurements: Optional[List[StainMeasurement]] = None
    case_context: str = ""


class SoundPropagationRequest(BaseModel):
    source_x: float
    source_y: float
    listeners: List[Dict[str, float]] = Field(
        ..., description="List of {x, y} listener positions"
    )
    speed_of_sound: float = Field(default=343.0, gt=0)


# =====================================================================
# BPA endpoints
# =====================================================================

@router.post("/bpa/classify")
async def classify_blood_pattern(
    file: UploadFile = File(...),
    bpa: BloodPatternAnalyzer = Depends(get_bpa_analyzer),
) -> Dict[str, Any]:
    """Classify a bloodstain pattern from an uploaded image.

    Upload a photo of a bloodstain pattern and get its classification
    (spatter, transfer, drip, pool, cast-off, arterial, etc.).
    """
    tmp_path = await _save_upload(file)
    try:
        result = await bpa.classify_pattern(tmp_path)
        result["filename"] = file.filename
        return result
    finally:
        _cleanup(tmp_path)


@router.post("/bpa/analyze")
async def analyze_bpa(
    file: UploadFile = File(...),
    measurements: Optional[str] = Form(None),
    case_context: str = Form(""),
    bpa: BloodPatternAnalyzer = Depends(get_bpa_analyzer),
) -> Dict[str, Any]:
    """Full BPA analysis: VLM classification + geometric calculations.

    Upload a blood pattern image. Optionally provide stain measurements
    as a JSON string and case context for interpretation.

    Measurements JSON format:
    [{"x": 0, "y": 0, "direction_degrees": 45, "width": 3, "length": 6}, ...]
    """
    tmp_path = await _save_upload(file)

    # Parse measurements if provided
    parsed_measurements: Optional[List[Dict[str, Any]]] = None
    if measurements:
        try:
            parsed_measurements = json.loads(measurements)
        except json.JSONDecodeError:
            raise HTTPException(
                status_code=400,
                detail="Invalid measurements JSON",
            )

    try:
        result = await bpa.full_bpa_analysis(
            image_path=tmp_path,
            measurements=parsed_measurements,
            case_context=case_context,
        )
        result["filename"] = file.filename
        return result
    finally:
        _cleanup(tmp_path)


@router.post("/bpa/calculate-angle")
async def calculate_impact_angle(
    body: ImpactAngleRequest,
    bpa: BloodPatternAnalyzer = Depends(get_bpa_analyzer),
) -> Dict[str, Any]:
    """Calculate the angle of impact from stain width and length.

    Uses sin(angle) = width / length. Returns angle in degrees.
    """
    try:
        angle = bpa.calculate_impact_angle(body.width, body.length)
        return {
            "width": body.width,
            "length": body.length,
            "impact_angle_degrees": round(angle, 2),
            "formula": "sin(angle) = width / length",
        }
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))


@router.post("/bpa/convergence")
async def calculate_convergence(
    body: ConvergenceRequest,
    bpa: BloodPatternAnalyzer = Depends(get_bpa_analyzer),
) -> Dict[str, Any]:
    """Calculate the 2D area of convergence from stain measurements.

    Requires at least 2 stains with position and direction data.
    Optionally calculates area of origin if width/length are provided.
    """
    stains = [s.model_dump() for s in body.stains]

    try:
        convergence = bpa.calculate_area_of_convergence(stains)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))

    result: Dict[str, Any] = {"convergence": convergence}

    # If width/length provided, also compute impact angles and origin
    stains_with_dims = [
        s for s in stains
        if s.get("width") and s.get("length")
    ]
    if stains_with_dims:
        for s in stains_with_dims:
            try:
                s["angle_degrees"] = bpa.calculate_impact_angle(
                    s["width"], s["length"]
                )
            except ValueError:
                pass

        stains_with_angles = [
            s for s in stains_with_dims if "angle_degrees" in s
        ]
        if stains_with_angles:
            try:
                origin = bpa.estimate_area_of_origin(
                    stains_with_angles, convergence
                )
                result["area_of_origin"] = origin
            except ValueError as exc:
                result["area_of_origin"] = {"error": str(exc)}

    return result


# =====================================================================
# Acoustic endpoints
# =====================================================================

@router.post("/audio/transcribe")
async def transcribe_audio(
    file: UploadFile = File(...),
    acoustic: AcousticAnalyzer = Depends(get_acoustic_analyzer),
) -> Dict[str, Any]:
    """Transcribe an audio file using the voxtral model."""
    tmp_path = await _save_upload(file)
    try:
        transcription = await acoustic.transcribe_audio(tmp_path)
        return {
            "filename": file.filename,
            "transcription": transcription,
        }
    finally:
        _cleanup(tmp_path)


@router.post("/audio/analyze")
async def analyze_audio_forensic(
    file: UploadFile = File(...),
    acoustic: AcousticAnalyzer = Depends(get_acoustic_analyzer),
) -> Dict[str, Any]:
    """Full forensic analysis of an audio recording.

    Includes transcription, event detection, and LLM forensic assessment.
    """
    tmp_path = await _save_upload(file)
    try:
        result = await acoustic.analyze_audio_forensic(tmp_path)
        result["filename"] = file.filename
        return result
    finally:
        _cleanup(tmp_path)


@router.post("/audio/events")
async def detect_audio_events(
    file: UploadFile = File(...),
    acoustic: AcousticAnalyzer = Depends(get_acoustic_analyzer),
) -> Dict[str, Any]:
    """Detect notable events in an audio file (WAV).

    Uses RMS energy analysis for loud event and silence detection.
    Returns a list of timestamped events.
    """
    tmp_path = await _save_upload(file)
    try:
        events = acoustic.detect_audio_events(tmp_path)
        return {
            "filename": file.filename,
            "event_count": len(events),
            "events": events,
        }
    finally:
        _cleanup(tmp_path)


@router.post("/audio/propagation")
async def calculate_sound_propagation(
    body: SoundPropagationRequest,
    acoustic: AcousticAnalyzer = Depends(get_acoustic_analyzer),
) -> Dict[str, Any]:
    """Calculate sound arrival times at different listener positions.

    Useful for gunshot localization with multiple witnesses.
    """
    listener_coords = [(l["x"], l["y"]) for l in body.listeners]
    try:
        results = acoustic.calculate_sound_propagation(
            source_coords=(body.source_x, body.source_y),
            listener_coords=listener_coords,
            speed_of_sound=body.speed_of_sound,
        )
        return {
            "source": {"x": body.source_x, "y": body.source_y},
            "speed_of_sound_ms": body.speed_of_sound,
            "listeners": results,
        }
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))


# =====================================================================
# Trace endpoints
# =====================================================================

@router.post("/trace/analyze")
async def analyze_trace(
    file: UploadFile = File(...),
    trace_type: str = Form("auto"),
    trace: TraceAnalyzer = Depends(get_trace_analyzer),
) -> Dict[str, Any]:
    """Analyze a physical trace from a photo.

    Supported trace types: fingerprint, tool_mark, tire_track,
    shoe_print, glass_fracture, fabric, hair, fiber, auto.
    """
    tmp_path = await _save_upload(file)
    try:
        result = await trace.analyze_trace(tmp_path, trace_type)
        result["filename"] = file.filename
        return result
    finally:
        _cleanup(tmp_path)


@router.post("/trace/compare")
async def compare_traces(
    file_1: UploadFile = File(...),
    file_2: UploadFile = File(...),
    trace: TraceAnalyzer = Depends(get_trace_analyzer),
) -> Dict[str, Any]:
    """Compare two trace images for similarity.

    Upload two photos of physical traces to assess whether they
    could originate from the same source.
    """
    tmp_path_1 = await _save_upload(file_1)
    tmp_path_2 = await _save_upload(file_2)
    try:
        result = await trace.compare_traces(tmp_path_1, tmp_path_2)
        result["filename_1"] = file_1.filename
        result["filename_2"] = file_2.filename
        return result
    finally:
        _cleanup(tmp_path_1)
        _cleanup(tmp_path_2)


# =====================================================================
# Auto-analysis (all forensics on case evidence)
# =====================================================================

@router.post("/cases/{case_id}/auto", tags=["forensics", "cases"])
async def auto_forensic_analysis(
    case_id: str,
    db: Database = Depends(get_database),
    bpa: BloodPatternAnalyzer = Depends(get_bpa_analyzer),
    acoustic: AcousticAnalyzer = Depends(get_acoustic_analyzer),
    trace: TraceAnalyzer = Depends(get_trace_analyzer),
) -> Dict[str, Any]:
    """Run automatic forensic analysis on all evidence in a case.

    Iterates through all evidence and applies the appropriate
    forensic analysis based on evidence type:
    - Image evidence: BPA classification + trace analysis
    - Audio evidence: transcription + forensic analysis
    """
    case = await db.get_case(case_id)
    if case is None:
        raise HTTPException(status_code=404, detail=f"Case not found: {case_id}")

    all_evidence = await db.list_evidence_by_case(case_id)
    if not all_evidence:
        return {
            "case_id": case_id,
            "message": "Aucune preuve trouvee dans ce dossier.",
            "results": [],
        }

    results: List[Dict[str, Any]] = []
    errors: List[Dict[str, Any]] = []

    for ev in all_evidence:
        ev_id = ev.get("id", "")
        ev_type = ev.get("evidence_type", "")
        file_path = ev.get("file_path", "")

        if not file_path or not Path(file_path).exists():
            continue

        # Image evidence: run BPA + trace analysis
        if ev_type == "image":
            try:
                bpa_result = await bpa.classify_pattern(file_path)
                results.append({
                    "evidence_id": ev_id,
                    "analysis_type": "bpa_classification",
                    "result": bpa_result,
                })
            except Exception as exc:
                logger.error("BPA auto-analysis failed for {}: {}", ev_id, exc)
                errors.append({
                    "evidence_id": ev_id,
                    "analysis_type": "bpa_classification",
                    "error": str(exc),
                })

            try:
                trace_result = await trace.analyze_trace(file_path)
                results.append({
                    "evidence_id": ev_id,
                    "analysis_type": "trace_analysis",
                    "result": trace_result,
                })
            except Exception as exc:
                logger.error(
                    "Trace auto-analysis failed for {}: {}", ev_id, exc
                )
                errors.append({
                    "evidence_id": ev_id,
                    "analysis_type": "trace_analysis",
                    "error": str(exc),
                })

        # Audio evidence: transcription + forensic analysis
        elif ev_type == "audio":
            try:
                audio_result = await acoustic.analyze_audio_forensic(file_path)
                results.append({
                    "evidence_id": ev_id,
                    "analysis_type": "audio_forensic",
                    "result": audio_result,
                })
            except Exception as exc:
                logger.error(
                    "Audio auto-analysis failed for {}: {}", ev_id, exc
                )
                errors.append({
                    "evidence_id": ev_id,
                    "analysis_type": "audio_forensic",
                    "error": str(exc),
                })

    return {
        "case_id": case_id,
        "evidence_processed": len(results),
        "errors_count": len(errors),
        "results": results,
        "errors": errors,
    }


# =====================================================================
# Helpers
# =====================================================================

async def _save_upload(file: UploadFile) -> Path:
    """Save an uploaded file to a temporary location."""
    suffix = Path(file.filename or "upload").suffix or ".bin"
    with tempfile.NamedTemporaryFile(delete=False, suffix=suffix) as tmp:
        content = await file.read()
        tmp.write(content)
        return Path(tmp.name)


def _cleanup(path: Path) -> None:
    """Remove a temporary file, ignoring errors."""
    try:
        path.unlink()
    except OSError:
        pass
