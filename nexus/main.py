"""
NEXUS -- FastAPI application entry point.

Starts the investigation system with:
- SQLite database initialisation
- Ollama LLM router (shared singleton)
- CORS middleware for dashboard access
- All API routers (cases, evidence, entities, analysis, reports)
- Loguru intercepting uvicorn logs
- X-Process-Time header middleware
- Exception handler for Ollama connection errors (503)

Run with::

    uvicorn nexus.main:app --host 0.0.0.0 --port 8000 --reload
"""

from __future__ import annotations

import logging
import sys
import time
from contextlib import asynccontextmanager

from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from loguru import logger
from starlette.middleware.base import BaseHTTPMiddleware

from nexus.config import settings


# ============================================================================
# Loguru intercept handler for uvicorn logs
# ============================================================================

class _InterceptHandler(logging.Handler):
    """Redirect standard logging to Loguru.

    Installed on uvicorn's loggers so all output goes through the
    same structured Loguru pipeline.
    """

    def emit(self, record: logging.LogRecord) -> None:
        # Get the corresponding Loguru level
        try:
            level = logger.level(record.levelname).name
        except ValueError:
            level = record.levelno

        # Find caller from where the logged message originated
        frame, depth = logging.currentframe(), 2
        while frame and frame.f_code.co_filename == logging.__file__:
            frame = frame.f_back  # type: ignore[assignment]
            depth += 1

        logger.opt(depth=depth, exception=record.exc_info).log(
            level, record.getMessage()
        )


def _setup_loguru_intercept() -> None:
    """Install Loguru as the handler for uvicorn and other stdlib loggers."""
    intercept = _InterceptHandler()

    for name in ("uvicorn", "uvicorn.error", "uvicorn.access", "fastapi"):
        target = logging.getLogger(name)
        target.handlers = [intercept]
        target.propagate = False


# ============================================================================
# X-Process-Time middleware
# ============================================================================

class ProcessTimeMiddleware(BaseHTTPMiddleware):
    """Add X-Process-Time header to every response."""

    async def dispatch(self, request: Request, call_next):
        start = time.perf_counter()
        response = await call_next(request)
        elapsed = time.perf_counter() - start
        response.headers["X-Process-Time"] = f"{elapsed:.4f}"
        return response


# ============================================================================
# Lifespan
# ============================================================================

@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application startup / shutdown lifecycle.

    Startup:
      - Initialise the SQLite schema (idempotent).
      - Ensure the upload directory exists.
      - Create shared singletons (OllamaClient, LLMRouter) on app.state.

    Shutdown:
      - Log a clean shutdown message.
    """
    from nexus.db.sqlite_db import init_db
    from nexus.db.neo4j_db import Neo4jClient
    from nexus.db.chroma_db import ChromaClient
    from nexus.llm.ollama_client import OllamaClient
    from nexus.llm.router import LLMRouter
    from nexus.monitoring.scheduler import MonitoringScheduler

    # -- Startup --------------------------------------------------------
    _setup_loguru_intercept()
    logger.info("NEXUS starting up...")

    # Database schema
    await init_db()
    logger.info("SQLite database initialised at {}", settings.sqlite_path)

    # Ensure required directories exist
    settings.upload_dir.mkdir(parents=True, exist_ok=True)
    (settings.data_dir / "reports").mkdir(parents=True, exist_ok=True)
    (settings.data_dir / "backups").mkdir(parents=True, exist_ok=True)

    # Shared singletons -- these are stateless or internally locked,
    # so they can safely be shared across requests.
    app.state.ollama = OllamaClient()
    app.state.router = LLMRouter(app.state.ollama)

    # Neo4j graph database (optional -- degraded mode if unavailable)
    app.state.neo4j = None
    try:
        neo4j_client = Neo4jClient()
        await neo4j_client.init_constraints()
        app.state.neo4j = neo4j_client
        logger.info("Neo4j client initialised at {}", settings.neo4j_uri)
    except Exception as exc:
        logger.warning(
            "Neo4j unavailable -- running in degraded mode (no graph): {}",
            exc,
        )

    # ChromaDB vector store (optional -- degraded mode if unavailable)
    app.state.chroma = None
    try:
        chroma_client = ChromaClient()
        chroma_client.init_collections()
        app.state.chroma = chroma_client
        logger.info("ChromaDB client initialised at {}:{}", settings.chroma_host, settings.chroma_port)
    except Exception as exc:
        logger.warning(
            "ChromaDB unavailable -- running in degraded mode (no vectors): {}",
            exc,
        )

    # Monitoring scheduler (APScheduler)
    app.state.monitoring_scheduler = None
    try:
        scheduler = MonitoringScheduler(
            router=app.state.router,
            chroma=app.state.chroma,
        )
        await scheduler.start()
        app.state.monitoring_scheduler = scheduler
        logger.info("Monitoring scheduler started")
    except Exception:
        logger.warning("Monitoring scheduler failed to start -- monitoring will be unavailable")

    # Autonomous investigation manager
    app.state.investigation_manager = None
    try:
        from nexus.core.investigation_manager import InvestigationManager

        inv_manager = InvestigationManager(
            router=app.state.router,
            chroma=app.state.chroma,
            neo4j=app.state.neo4j,
        )
        await inv_manager.start()
        app.state.investigation_manager = inv_manager
        logger.info("Autonomous investigation manager started")
    except Exception:
        logger.warning(
            "Investigation manager failed to start -- autonomous loops will be unavailable"
        )

    logger.info("NEXUS started -- listening on {}:{}", settings.nexus_host, settings.nexus_port)

    try:
        yield
    finally:
        # -- Shutdown ---------------------------------------------------
        logger.info("NEXUS shutting down")
        if app.state.investigation_manager is not None:
            await app.state.investigation_manager.stop_all()
        if app.state.monitoring_scheduler is not None:
            await app.state.monitoring_scheduler.stop()
        if app.state.neo4j is not None:
            await app.state.neo4j.close()
        if app.state.chroma is not None:
            app.state.chroma.close()
        logger.info("NEXUS shutdown complete")


# ============================================================================
# Application
# ============================================================================

app = FastAPI(
    title="NEXUS",
    description="Cold Case Investigation System -- persistent, incremental, multi-source.",
    version="0.1.0",
    lifespan=lifespan,
)


# ============================================================================
# Middleware
# ============================================================================

app.add_middleware(ProcessTimeMiddleware)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],        # Streamlit dashboard + dev
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# ============================================================================
# Routers
# ============================================================================

from nexus.api import (  # noqa: E402
    cases, evidence, entities, analysis, graph, search,
    monitoring, alerts, hypotheses, reports, timeline, geo,
    recon, image_search, vision, forensics, physics_sim_api,
    investigation, audit,
)

app.include_router(cases.router)
app.include_router(evidence.router)
app.include_router(entities.router)
app.include_router(analysis.router)
app.include_router(graph.router)
app.include_router(search.router)
app.include_router(monitoring.router)
app.include_router(alerts.router)
app.include_router(hypotheses.router)
app.include_router(reports.router)
app.include_router(timeline.router)
app.include_router(geo.router)
app.include_router(recon.router)
app.include_router(image_search.router)
app.include_router(vision.router)
app.include_router(forensics.router)
app.include_router(physics_sim_api.router)
app.include_router(investigation.router)
app.include_router(audit.router)


# ============================================================================
# Exception handlers
# ============================================================================

@app.exception_handler(Exception)
async def _ollama_exception_handler(request: Request, exc: Exception) -> JSONResponse:
    """Catch Ollama connection / response errors and return 503.

    All other unhandled exceptions get a generic 500.
    """
    import httpx
    from ollama import RequestError, ResponseError

    if isinstance(exc, (httpx.ConnectError, httpx.TimeoutException)):
        logger.error("Ollama connection error: {}", exc)
        return JSONResponse(
            status_code=503,
            content={
                "detail": "LLM service unavailable (Ollama connection error)",
                "error": str(exc),
            },
        )

    if isinstance(exc, (RequestError, ResponseError)):
        logger.error("Ollama error: {}", exc)
        return JSONResponse(
            status_code=503,
            content={
                "detail": "LLM service error",
                "error": str(exc),
            },
        )

    # Let other exceptions propagate to FastAPI's default handler
    logger.error("Unhandled exception: {} — {}", type(exc).__name__, exc)
    return JSONResponse(
        status_code=500,
        content={"detail": "Internal server error"},
    )


# ============================================================================
# Health check
# ============================================================================

@app.get("/api/health", tags=["system"])
async def health_check() -> dict:
    """Lightweight liveness probe.

    Returns basic system info. Does NOT check Ollama / Neo4j / ChromaDB
    connectivity -- use a dedicated readiness endpoint for that later.
    """
    return {
        "status": "ok",
        "version": app.version,
        "sqlite": str(settings.sqlite_path),
        "ollama": settings.ollama_base_url,
    }
