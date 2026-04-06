"""
NEXUS -- FastAPI dependency injection helpers.

Provides request-scoped dependencies for database connections and
service objects.  Each request gets its own aiosqlite connection
(via ``get_db()``) and a fresh ``Database`` wrapper around it.

Higher-level services (CaseManager, EvidenceProcessor, etc.) are
built per-request on top of that connection.  Shared singletons
(LLMRouter, OllamaClient, Neo4jClient, ChromaClient) live on
``app.state`` and are injected from there.
"""

from __future__ import annotations

from typing import AsyncIterator

from fastapi import Depends, Request

from nexus.config import settings
from nexus.core.audit import AuditService
from nexus.core.case_manager import CaseManager
from nexus.core.entity_extractor import EntityExtractor
from nexus.core.geo_mapper import GeoMapper
from nexus.db.chroma_db import ChromaClient
from nexus.db.neo4j_db import Neo4jClient
from nexus.db.sqlite_db import Database, get_db
from nexus.llm.router import LLMRouter


# ------------------------------------------------------------------
# Database (request-scoped connection)
# ------------------------------------------------------------------

async def get_database() -> AsyncIterator[Database]:
    """Yield a ``Database`` bound to a fresh connection, closed on exit."""
    async with get_db() as conn:
        yield Database(conn)


# ------------------------------------------------------------------
# Core services
# ------------------------------------------------------------------

def get_audit_service(
    db: Database = Depends(get_database),
) -> AuditService:
    """Build an AuditService with per-request DB connection."""
    return AuditService(db)


def get_case_manager(
    db: Database = Depends(get_database),
) -> CaseManager:
    return CaseManager(db)


def get_evidence_processor(
    request: Request,
    db: Database = Depends(get_database),
):
    """Build an EvidenceProcessor with per-request DB and shared LLMRouter."""
    from nexus.core.evidence_processor import EvidenceProcessor

    return EvidenceProcessor(
        db=db,
        router=request.app.state.router,
        upload_dir=settings.upload_dir,
        neo4j=getattr(request.app.state, "neo4j", None),
        chroma=getattr(request.app.state, "chroma", None),
        entity_extractor=getattr(request.app.state, "entity_extractor", None),
    )


def get_analysis_pipeline(
    request: Request,
    db: Database = Depends(get_database),
):
    """Build an AnalysisPipeline with per-request DB and shared LLMRouter.

    Injects ChromaDB and Neo4j from app.state when available so the
    pipeline can use the RAG retriever instead of loading all evidence.
    """
    from nexus.core.analysis_pipeline import AnalysisPipeline

    return AnalysisPipeline(
        db=db,
        router=request.app.state.router,
        chroma=getattr(request.app.state, "chroma", None),
        neo4j=getattr(request.app.state, "neo4j", None),
    )


def get_entity_extractor(request: Request) -> EntityExtractor:
    """Return the pre-loaded EntityExtractor singleton, or create one."""
    extractor = getattr(request.app.state, "entity_extractor", None)
    if extractor is not None:
        return extractor
    return EntityExtractor(request.app.state.router)


def get_geo_mapper(
    db: Database = Depends(get_database),
) -> GeoMapper:
    """Build a GeoMapper with a per-request DB connection."""
    return GeoMapper(db)


# ------------------------------------------------------------------
# Image Analyzer (request-scoped)
# ------------------------------------------------------------------

def get_image_analyzer(
    request: Request,
    db: Database = Depends(get_database),
):
    """Build an ImageAnalyzer with per-request DB and shared LLMRouter + ChromaDB."""
    from nexus.core.image_analyzer import ImageAnalyzer

    return ImageAnalyzer(
        router=request.app.state.router,
        db=db,
        chroma=getattr(request.app.state, "chroma", None),
    )


# ------------------------------------------------------------------
# Neo4j (singleton on app.state)
# ------------------------------------------------------------------

def get_neo4j(request: Request) -> Neo4jClient:
    """Return the shared Neo4jClient from app.state."""
    return request.app.state.neo4j


# ------------------------------------------------------------------
# ChromaDB (singleton on app.state)
# ------------------------------------------------------------------

def get_chroma(request: Request) -> ChromaClient:
    """Return the shared ChromaClient from app.state."""
    return request.app.state.chroma


# ------------------------------------------------------------------
# LLM Router (singleton on app.state)
# ------------------------------------------------------------------

def get_llm_router(request: Request) -> LLMRouter:
    """Return the shared LLMRouter from app.state."""
    return request.app.state.router


# ------------------------------------------------------------------
# Hypothesis Engine (request-scoped)
# ------------------------------------------------------------------

def get_hypothesis_engine(
    request: Request,
    db: Database = Depends(get_database),
):
    """Build a HypothesisEngine with per-request DB and shared LLMRouter.

    Injects ChromaDB and Neo4j from app.state when available so the
    engine can use the RAG retriever instead of loading all evidence.
    """
    from nexus.core.hypothesis_engine import HypothesisEngine

    return HypothesisEngine(
        db=db,
        router=request.app.state.router,
        chroma=getattr(request.app.state, "chroma", None),
        neo4j=getattr(request.app.state, "neo4j", None),
    )


# ------------------------------------------------------------------
# Suspect Scorer (request-scoped)
# ------------------------------------------------------------------

def get_suspect_scorer(
    request: Request,
    db: Database = Depends(get_database),
):
    """Build a SuspectScorer with per-request DB and shared LLMRouter.

    Injects Neo4j from app.state when available for graph scoring.
    """
    from nexus.core.suspect_scorer import SuspectScorer

    return SuspectScorer(
        db=db,
        router=request.app.state.router,
        neo4j=getattr(request.app.state, "neo4j", None),
    )


# ------------------------------------------------------------------
# Contradiction Detector (request-scoped)
# ------------------------------------------------------------------

def get_contradiction_detector(
    request: Request,
    db: Database = Depends(get_database),
):
    """Build a ContradictionDetector with per-request DB and shared LLMRouter."""
    from nexus.core.contradiction_detector import ContradictionDetector

    return ContradictionDetector(db=db, router=request.app.state.router)


# ------------------------------------------------------------------
# Forensic analyzers (request-scoped, only need the LLMRouter)
# ------------------------------------------------------------------

def get_bpa_analyzer(request: Request):
    """Return a BloodPatternAnalyzer using the shared LLMRouter."""
    from nexus.forensics.blood_pattern import BloodPatternAnalyzer

    return BloodPatternAnalyzer(request.app.state.router)


def get_acoustic_analyzer(request: Request):
    """Return an AcousticAnalyzer using the shared LLMRouter."""
    from nexus.forensics.acoustic_analysis import AcousticAnalyzer

    return AcousticAnalyzer(request.app.state.router)


def get_trace_analyzer(request: Request):
    """Return a TraceAnalyzer using the shared LLMRouter."""
    from nexus.forensics.trace_analyzer import TraceAnalyzer

    return TraceAnalyzer(request.app.state.router)
