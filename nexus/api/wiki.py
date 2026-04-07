"""
NEXUS -- Wiki API.

Endpoints for browsing and managing the case investigation wiki.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Request

from nexus.api.deps import get_database
from nexus.config import settings
from nexus.db.sqlite_db import Database

router = APIRouter(prefix="/api", tags=["wiki"])


@router.get("/cases/{case_id}/wiki")
async def list_wiki_pages(
    case_id: str,
    page_type: str | None = None,
    db: Database = Depends(get_database),
) -> list[dict[str, Any]]:
    """List all wiki pages for a case."""
    case = await db.get_case(case_id)
    if not case:
        raise HTTPException(404, f"Case not found: {case_id}")
    return await db.list_wiki_pages(case_id, page_type=page_type)


@router.get("/cases/{case_id}/wiki/read/{page_path:path}")
async def read_wiki_page(
    case_id: str,
    page_path: str,
    db: Database = Depends(get_database),
) -> dict[str, Any]:
    """Read a wiki page's markdown content."""
    case = await db.get_case(case_id)
    if not case:
        raise HTTPException(404, f"Case not found: {case_id}")

    wiki_dir = settings.data_dir / "cases" / case_id / "wiki"
    file_path = wiki_dir / page_path

    if not file_path.exists() or not str(file_path.resolve()).startswith(str(wiki_dir.resolve())):
        raise HTTPException(404, f"Page not found: {page_path}")

    content = file_path.read_text(encoding="utf-8")
    page_meta = await db.get_wiki_page(case_id, page_path)

    return {
        "page_path": page_path,
        "content": content,
        "metadata": page_meta,
    }


@router.post("/cases/{case_id}/wiki/rebuild")
async def rebuild_wiki(
    case_id: str,
    request: Request,
    db: Database = Depends(get_database),
) -> dict[str, Any]:
    """Force a full wiki recompilation for a case."""
    case = await db.get_case(case_id)
    if not case:
        raise HTTPException(404, f"Case not found: {case_id}")

    from nexus.core.wiki_compiler import WikiCompiler

    compiler = WikiCompiler(db, request.app.state.router)

    # Compile all evidence
    evidence_list = await db.list_evidence_by_case(case_id, status="processed")
    total_pages = 0
    for ev in evidence_list:
        pages = await compiler.compile_evidence(case_id, ev["id"])
        total_pages += len(pages)

    # Compile hypotheses
    await compiler.compile_hypothesis_update(case_id)
    total_pages += 1

    # Rebuild index
    await compiler.rebuild_index(case_id)

    return {"status": "rebuilt", "pages_compiled": total_pages}
