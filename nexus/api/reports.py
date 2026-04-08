"""
NEXUS -- Reports API router.

Endpoints for generating, listing, and downloading investigation reports.
"""

from __future__ import annotations

from pathlib import Path
from typing import Literal, Optional

from fastapi import APIRouter, BackgroundTasks, Depends, HTTPException, Request
from fastapi.responses import FileResponse
from loguru import logger
from pydantic import BaseModel

from nexus.config import settings
from nexus.db.sqlite_db import Database
from nexus.api.deps import get_database, get_llm_router
from nexus.llm.router import LLMRouter

router = APIRouter(tags=["reports"])


# ------------------------------------------------------------------
# Request / Response models
# ------------------------------------------------------------------

ReportType = Literal["full", "summary", "timeline"]


class GenerateReportRequest(BaseModel):
    report_type: ReportType = "full"


class ReportResponse(BaseModel):
    id: str
    case_id: str
    report_type: str
    status: str
    file_path: Optional[str] = None
    file_size: Optional[int] = None
    created_at: str
    completed_at: Optional[str] = None


# ------------------------------------------------------------------
# Background task: generate report
# ------------------------------------------------------------------

async def _generate_report_task(
    report_id: str,
    case_id: str,
    report_type: str,
) -> None:
    """Background task that generates the report and updates the DB row."""
    from nexus.db.sqlite_db import get_db, Database
    from nexus.llm.ollama_client import OllamaClient
    from nexus.llm.router import LLMRouter
    from nexus.export.report_generator import ReportGenerator
    from nexus.export.pdf_export import PDFExporter
    from datetime import datetime, timezone

    try:
        async with get_db() as conn:
            db = Database(conn)
            llm_router = LLMRouter(OllamaClient())

            generator = ReportGenerator(db, llm_router)

            # Generate report data
            if report_type == "full":
                report_data = await generator.generate_full_report(case_id)
                template_name = "full_report.html"
            elif report_type == "summary":
                report_data = await generator.generate_summary_report(case_id)
                template_name = "summary.html"
            elif report_type == "timeline":
                report_data = await generator.generate_timeline_report(case_id)
                # Timeline uses full_report template but could have its own
                template_name = "full_report.html"
            else:
                raise ValueError(f"Unknown report type: {report_type}")

            # Export to PDF
            reports_dir = settings.data_dir / "reports"
            reports_dir.mkdir(parents=True, exist_ok=True)

            timestamp = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
            filename = f"nexus_{report_type}_{case_id[:8]}_{timestamp}.pdf"
            output_path = reports_dir / filename

            exporter = PDFExporter()
            exporter.export_report(
                report_data,
                output_path,
                template_name=template_name,
            )

            file_size = output_path.stat().st_size
            now = datetime.now(timezone.utc).isoformat()

            await db.update_report(
                report_id,
                status="completed",
                file_path=str(output_path),
                file_size=file_size,
                completed_at=now,
            )

            logger.info(
                "Report {} generated: {} ({} bytes)",
                report_id,
                output_path,
                file_size,
            )

    except Exception as exc:
        logger.error("Report generation failed for {}: {}", report_id, exc)
        try:
            async with get_db() as conn:
                db = Database(conn)
                await db.update_report(report_id, status="error")
        except Exception as exc:
            logger.error("Failed to update report status to error: {}", exc)


# ------------------------------------------------------------------
# POST /api/cases/{case_id}/reports/generate
# ------------------------------------------------------------------

@router.post(
    "/api/cases/{case_id}/reports/generate",
    response_model=ReportResponse,
    status_code=202,
)
async def generate_report(
    case_id: str,
    body: GenerateReportRequest,
    background_tasks: BackgroundTasks,
    db: Database = Depends(get_database),
) -> ReportResponse:
    """Start generating a report in the background.

    Returns immediately with status 'generating'. Poll the report
    endpoint to check completion.
    """
    # Verify case exists
    case = await db.get_case(case_id)
    if not case:
        raise HTTPException(status_code=404, detail=f"Case not found: {case_id}")

    # Create report row
    report = await db.create_report(
        case_id=case_id,
        report_type=body.report_type,
    )

    # Schedule background generation
    background_tasks.add_task(
        _generate_report_task,
        report["id"],
        case_id,
        body.report_type,
    )

    logger.info(
        "Report generation queued: {} (type={}, case={})",
        report["id"],
        body.report_type,
        case_id,
    )

    return ReportResponse(**report)


# ------------------------------------------------------------------
# GET /api/reports/{report_id}
# ------------------------------------------------------------------

@router.get(
    "/api/reports/{report_id}",
    response_model=ReportResponse,
)
async def get_report(
    report_id: str,
    db: Database = Depends(get_database),
) -> ReportResponse:
    """Get the status and metadata of a report."""
    report = await db.get_report(report_id)
    if not report:
        raise HTTPException(status_code=404, detail=f"Report not found: {report_id}")
    return ReportResponse(**report)


# ------------------------------------------------------------------
# GET /api/reports/{report_id}/download
# ------------------------------------------------------------------

@router.get("/api/reports/{report_id}/download")
async def download_report(
    report_id: str,
    db: Database = Depends(get_database),
) -> FileResponse:
    """Download the generated PDF report file."""
    report = await db.get_report(report_id)
    if not report:
        raise HTTPException(status_code=404, detail=f"Report not found: {report_id}")

    if report["status"] != "completed":
        raise HTTPException(
            status_code=409,
            detail=f"Report is not ready yet (status: {report['status']})",
        )

    file_path = report.get("file_path")
    if not file_path or not Path(file_path).exists():
        raise HTTPException(
            status_code=404,
            detail="Report file not found on disk",
        )

    return FileResponse(
        path=file_path,
        media_type="application/pdf",
        filename=Path(file_path).name,
    )


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/reports
# ------------------------------------------------------------------

@router.get(
    "/api/cases/{case_id}/reports",
    response_model=list[ReportResponse],
)
async def list_reports(
    case_id: str,
    db: Database = Depends(get_database),
) -> list[ReportResponse]:
    """List all reports for a case."""
    reports = await db.list_reports_by_case(case_id)
    return [ReportResponse(**r) for r in reports]
