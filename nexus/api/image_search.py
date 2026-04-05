"""
NEXUS -- Image search API router.

Provides endpoints for:
- Text-to-image search (CLIP)
- Image-to-image search (DINOv2)
- Similar evidence lookup
- Batch image indexing for a case
"""

from __future__ import annotations

from typing import Optional

from fastapi import APIRouter, Depends, HTTPException, Request
from pydantic import BaseModel

from nexus.api.deps import get_chroma, get_database
from nexus.db.chroma_db import ChromaClient
from nexus.db.sqlite_db import Database

router = APIRouter(tags=["image-search"])


# ------------------------------------------------------------------
# Request / Response models
# ------------------------------------------------------------------

class TextSearchRequest(BaseModel):
    """Body for text-to-image search."""
    query: str
    n_results: int = 5


class ImageSearchRequest(BaseModel):
    """Body for image-to-image search."""
    evidence_id: str
    n_results: int = 5


class ImageSearchResult(BaseModel):
    evidence_id: str
    path: str
    case_id: str
    description: str
    distance: Optional[float] = None
    similarity: Optional[float] = None


class IndexResponse(BaseModel):
    indexed: int
    total: int


# ------------------------------------------------------------------
# Dependency: ImageSearchEngine (lazy singleton on app.state)
# ------------------------------------------------------------------

def _get_image_search(request: Request):
    """Return the ImageSearchEngine, creating it lazily on first use.

    The VisualEmbedder and ImageSearchEngine are stored on app.state
    so models are only loaded once across the application lifetime.
    """
    from nexus.vision.embeddings import VisualEmbedder
    from nexus.vision.image_search import ImageSearchEngine

    if not hasattr(request.app.state, "visual_embedder"):
        request.app.state.visual_embedder = VisualEmbedder()
    if not hasattr(request.app.state, "image_search"):
        request.app.state.image_search = ImageSearchEngine(
            chroma_client=request.app.state.chroma,
            visual_embedder=request.app.state.visual_embedder,
        )
    return request.app.state.image_search


# ------------------------------------------------------------------
# POST /api/cases/{case_id}/images/search-by-text
# ------------------------------------------------------------------

@router.post(
    "/api/cases/{case_id}/images/search-by-text",
    response_model=list[ImageSearchResult],
)
async def search_images_by_text(
    case_id: str,
    body: TextSearchRequest,
    engine=Depends(_get_image_search),
):
    """Search images by natural language text query using CLIP.

    The query is embedded in CLIP's shared text-image space and compared
    against all indexed image embeddings for the given case.
    """
    results = engine.search_by_text(
        query=body.query,
        case_id=case_id,
        n_results=body.n_results,
    )
    return results


# ------------------------------------------------------------------
# POST /api/cases/{case_id}/images/search-by-image
# ------------------------------------------------------------------

@router.post(
    "/api/cases/{case_id}/images/search-by-image",
    response_model=list[ImageSearchResult],
)
async def search_images_by_image(
    case_id: str,
    body: ImageSearchRequest,
    engine=Depends(_get_image_search),
    db: Database = Depends(get_database),
):
    """Search for visually similar images using DINOv2.

    Takes an evidence_id, retrieves its file path from SQLite, then
    embeds that image and finds nearest neighbours in the DINOv2 collection.
    """
    # Look up the evidence file path
    evidence = await db.fetchone(
        "SELECT file_path FROM evidence WHERE id = ?",
        (body.evidence_id,),
    )
    if not evidence or not evidence["file_path"]:
        raise HTTPException(
            status_code=404,
            detail=f"Evidence '{body.evidence_id}' not found or has no image file",
        )

    results = engine.search_by_image(
        image_path=evidence["file_path"],
        case_id=case_id,
        n_results=body.n_results,
    )
    return results


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/images/similar/{evidence_id}
# ------------------------------------------------------------------

@router.get(
    "/api/cases/{case_id}/images/similar/{evidence_id}",
    response_model=list[ImageSearchResult],
)
async def get_similar_images(
    case_id: str,
    evidence_id: str,
    n_results: int = 5,
    engine=Depends(_get_image_search),
):
    """Find images visually similar to an already-indexed evidence image.

    Uses the stored DINOv2 embedding -- no re-computation needed.
    """
    results = engine.find_similar_evidence(
        evidence_id=evidence_id,
        case_id=case_id,
        n_results=n_results,
    )
    return results


# ------------------------------------------------------------------
# POST /api/cases/{case_id}/images/index
# ------------------------------------------------------------------

_IMAGE_EXTENSIONS = {".jpg", ".jpeg", ".png", ".bmp", ".webp", ".tiff", ".tif"}


@router.post(
    "/api/cases/{case_id}/images/index",
    response_model=IndexResponse,
)
async def index_case_images(
    case_id: str,
    engine=Depends(_get_image_search),
    db: Database = Depends(get_database),
):
    """Index all image evidence for a case into the visual search collections.

    Scans all evidence entries for the case, filters for image types,
    and embeds each one in both DINOv2 and CLIP collections.
    """
    rows = await db.fetchall(
        "SELECT id, file_path, title FROM evidence WHERE case_id = ?",
        (case_id,),
    )
    if not rows:
        raise HTTPException(
            status_code=404,
            detail=f"No evidence found for case '{case_id}'",
        )

    # Filter to image files only
    items = []
    for row in rows:
        fp = row.get("file_path") or ""
        if any(fp.lower().endswith(ext) for ext in _IMAGE_EXTENSIONS):
            items.append(
                {
                    "evidence_id": row["id"],
                    "case_id": case_id,
                    "image_path": fp,
                    "description": row.get("title", ""),
                }
            )

    if not items:
        return IndexResponse(indexed=0, total=len(rows))

    indexed = engine.index_image_batch(items)
    return IndexResponse(indexed=indexed, total=len(rows))
