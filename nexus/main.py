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
import os
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


def _setup_loguru_sinks() -> None:
    """Configure Loguru sinks: console + rotating file output."""
    from pathlib import Path

    # Remove default handler (avoids duplicate console output)
    logger.remove()

    # Console output
    logger.add(
        sys.stderr,
        level="INFO",
        format="{time:HH:mm:ss} | {level} | {name}:{function}:{line} | {message}",
    )

    # Ensure logs directory exists
    logs_dir = Path("logs")
    logs_dir.mkdir(parents=True, exist_ok=True)

    # File output with rotation
    logger.add(
        "logs/nexus.log",
        rotation="50 MB",
        retention="30 days",
        compression="gz",
        level="DEBUG",
        format="{time:YYYY-MM-DD HH:mm:ss} | {level} | {name}:{function}:{line} | {message}",
    )


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
    from nexus.events.vram_scheduler import VRAMScheduler

    # -- Startup --------------------------------------------------------
    _setup_loguru_sinks()
    _setup_loguru_intercept()
    logger.info("NEXUS starting up... (7 steps)")
    app.state.startup_time = time.time()

    # Database schema
    logger.info("[1/7] Initialising SQLite database...")
    await init_db()
    from nexus.db.government_db import init_government_db
    await init_government_db()
    from nexus.compute.db import init_compute_db
    await init_compute_db()
    logger.info("[1/7] SQLite + GOV + Compute tables ready at {}", settings.sqlite_path)

    # Ensure required directories exist
    settings.upload_dir.mkdir(parents=True, exist_ok=True)
    (settings.data_dir / "reports").mkdir(parents=True, exist_ok=True)
    (settings.data_dir / "backups").mkdir(parents=True, exist_ok=True)

    # Shared singletons -- these are stateless or internally locked,
    # so they can safely be shared across requests.
    app.state.ollama = OllamaClient()
    app.state.vram_scheduler = VRAMScheduler()
    app.state.router = LLMRouter(
        app.state.ollama,
        vram_scheduler=app.state.vram_scheduler,
    )

    # Neo4j graph database (optional -- degraded mode if unavailable)
    app.state.neo4j = None
    try:
        logger.info("[2/7] Connecting to Neo4j...")
        neo4j_client = Neo4jClient()
        logger.info("[2/7] Neo4j connected, initialising constraints (this takes ~20s)...")
        await neo4j_client.init_constraints()
        app.state.neo4j = neo4j_client
        logger.info("[2/7] Neo4j ready at {}", settings.neo4j_uri)
    except Exception as exc:
        logger.warning(
            "Neo4j unavailable -- running in degraded mode (no graph): {}",
            exc,
        )

    # ChromaDB vector store (optional -- degraded mode if unavailable)
    app.state.chroma = None
    try:
        logger.info("[3/7] Connecting to ChromaDB...")
        chroma_client = ChromaClient()
        chroma_client.init_collections()
        app.state.chroma = chroma_client
        logger.info("[3/7] ChromaDB ready at {}:{}", settings.chroma_host, settings.chroma_port)
    except Exception as exc:
        logger.warning(
            "ChromaDB unavailable -- running in degraded mode (no vectors): {}",
            exc,
        )

    # Pre-load GLiNER entity extractor (CPU, avoids VRAM conflicts with Ollama)
    app.state.entity_extractor = None
    try:
        logger.info("[4/7] Loading GLiNER entity extractor (CPU)...")
        from nexus.core.entity_extractor import EntityExtractor

        entity_extractor = EntityExtractor(app.state.router)
        if entity_extractor.preload():
            app.state.entity_extractor = entity_extractor
            logger.info("[4/7] GLiNER ready (CPU singleton)")
        else:
            logger.warning("GLiNER pre-load failed — will use LLM fallback")
    except Exception as exc:
        logger.warning("GLiNER pre-load skipped: {}", exc)

    # Reactive investigation manager (replaces APScheduler + old InvestigationManager)
    # MonitoringLoop is created per-case inside the manager -- no separate scheduler.
    app.state.monitoring_scheduler = None  # Keep attr for backward compat with monitoring API
    app.state.investigation_manager = None
    try:
        logger.info("[5/7] Starting investigation manager + cold case workers...")
        from nexus.events.manager import ReactiveInvestigationManager

        inv_manager = ReactiveInvestigationManager(
            router=app.state.router,
            chroma=app.state.chroma,
            neo4j=app.state.neo4j,
            entity_extractor=app.state.entity_extractor,
        )
        await inv_manager.start()
        app.state.investigation_manager = inv_manager
        logger.info("[5/7] Investigation manager ready")
    except Exception as exc:
        logger.warning(
            "Investigation manager failed to start -- reactive pipeline will be unavailable: {}",
            exc,
        )

    # Government autonomous investigation (auto-starts on boot)
    app.state.gov_case_id = None
    app.state.gov_manager = None
    try:
        logger.info("[6/7] Starting GOV module (31 workers)...")
        from nexus.core.government_bootstrap import bootstrap_government

        case_id, gov_manager = await bootstrap_government(
            app.state.investigation_manager,
            neo4j=getattr(app.state, "neo4j", None),
            chroma=getattr(app.state, "chroma", None),
        )
        app.state.gov_case_id = case_id
        app.state.gov_manager = gov_manager
        logger.info("[6/7] GOV module ready (31 workers)")
    except Exception as exc:
        logger.warning("[6/7] Government bootstrap skipped: {}", exc)

    # Distributed GPU compute system
    app.state.compute_manager = None
    if settings.compute_enabled:
        try:
            logger.info("[7/7] Starting Compute distributed GPU system...")
            from nexus.compute.manager import ComputeManager

            compute_mgr = ComputeManager()
            await compute_mgr.start()
            app.state.compute_manager = compute_mgr
            logger.info("[7/7] Compute system ready (dispatcher active)")
        except Exception as exc:
            logger.warning("[7/7] Compute system skipped: {}", exc)
    else:
        logger.info("[7/7] Compute system disabled (compute_enabled=false)")

    # Real-time sync broadcaster (cr-sqlite WebSocket)
    app.state.sync_broadcaster = None
    if settings.sync_enabled:
        try:
            from nexus.sync.broadcaster import SyncBroadcaster

            sync = SyncBroadcaster()
            await sync.start()
            app.state.sync_broadcaster = sync
            logger.info("Sync broadcaster started (cr-sqlite: {})", sync.crsqlite_available)
        except Exception as exc:
            logger.warning("Sync broadcaster skipped: {}", exc)

    # Source health monitor (resilience layer)
    app.state.gov_health_monitor = None
    try:
        from nexus.gov.resilience import SourceHealthMonitor

        health_monitor = SourceHealthMonitor()
        await health_monitor.start()
        app.state.gov_health_monitor = health_monitor
    except Exception as exc:
        logger.warning("Source health monitor skipped: {}", exc)

    elapsed = round(time.time() - app.state.startup_time, 1)
    logger.info("NEXUS ready in {}s -- http://{}:{}", elapsed, settings.nexus_host, settings.nexus_port)

    try:
        yield
    finally:
        # -- Shutdown ---------------------------------------------------
        logger.info("NEXUS shutting down")
        if getattr(app.state, "sync_broadcaster", None) is not None:
            await app.state.sync_broadcaster.stop()
        if getattr(app.state, "compute_manager", None) is not None:
            await app.state.compute_manager.stop()
        if getattr(app.state, "gov_health_monitor", None) is not None:
            await app.state.gov_health_monitor.stop()
        if getattr(app.state, "gov_manager", None) is not None:
            await app.state.gov_manager.stop()
        if app.state.investigation_manager is not None:
            await app.state.investigation_manager.stop_all()
        # MonitoringLoop is stopped inside investigation_manager.stop_all()
        if app.state.neo4j is not None:
            await app.state.neo4j.close()
        if app.state.chroma is not None:
            app.state.chroma.close()
        logger.info("NEXUS shutdown complete")


# ============================================================================
# Application
# ============================================================================

app = FastAPI(
    title="NEXUS API",
    description="Cold Case Investigation + Political Intelligence -- persistent, incremental, multi-source.",
    version="2.0",
    lifespan=lifespan,
    docs_url="/docs",
    redoc_url="/redoc",
    openapi_url="/openapi.json",
    openapi_tags=[
        {
            "name": "GOV Public API",
            "description": "Endpoints publics en lecture seule pour les donnees politiques (rate-limited, pagine).",
        },
        {
            "name": "GOV Internal",
            "description": "Endpoints internes du module gouvernement (workers, sync, RAG).",
        },
        {
            "name": "Investigation",
            "description": "Endpoints d'investigation cold case (evidence, entities, analysis).",
        },
        {
            "name": "compute",
            "description": "Distributed GPU compute — register nodes, pull tasks, submit results, leaderboard.",
        },
        {
            "name": "system",
            "description": "Health checks et monitoring systeme.",
        },
    ],
)


# ============================================================================
# Middleware
# ============================================================================

app.add_middleware(ProcessTimeMiddleware)

app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:3002",
        "http://localhost:8501",
        "http://127.0.0.1:3002",
        "http://127.0.0.1:8501",
    ],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
    expose_headers=["X-Total-Count", "X-Process-Time"],
)


# ============================================================================
# Routers
# ============================================================================

from nexus.api import (  # noqa: E402
    cases, evidence, entities, analysis, graph, search,
    monitoring, alerts, hypotheses, reports, timeline, geo,
    recon, image_search, vision, forensics, physics_sim_api,
    investigation, audit, benchmark, suspects, wiki, sse,
    government, compute,
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
app.include_router(benchmark.router)
app.include_router(audit.router)
app.include_router(suspects.router)
app.include_router(wiki.router)
app.include_router(sse.router)
app.include_router(government.router)
app.include_router(compute.router)

from nexus.sync.api import router as sync_router  # noqa: E402

app.include_router(sync_router)

from nexus.gov.public_api import router as gov_public_router  # noqa: E402

app.include_router(gov_public_router)


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
async def health_check(request: Request) -> dict:
    """Comprehensive readiness probe.

    Returns system status including uptime, service connectivity
    (SQLite, Neo4j, ChromaDB, Ollama, SearXNG), gov worker status,
    and resource usage (memory, disk).
    """
    import shutil

    import psutil

    # -- Uptime --
    startup_time = getattr(request.app.state, "startup_time", None)
    uptime_seconds = round(time.time() - startup_time) if startup_time else 0

    # -- Service checks --
    services: dict[str, bool] = {}

    # SQLite: try a trivial query
    try:
        import aiosqlite

        async with aiosqlite.connect(str(settings.sqlite_path)) as db:
            await db.execute("SELECT 1")
        services["sqlite"] = True
    except Exception:
        services["sqlite"] = False

    # Neo4j
    neo4j_client = getattr(request.app.state, "neo4j", None)
    services["neo4j"] = neo4j_client is not None

    # ChromaDB
    chroma_client = getattr(request.app.state, "chroma", None)
    services["chromadb"] = chroma_client is not None

    # Ollama: quick HTTP ping
    try:
        import httpx

        async with httpx.AsyncClient(timeout=3.0) as client:
            resp = await client.get(f"{settings.ollama_base_url}/api/version")
            services["ollama"] = resp.status_code == 200
    except Exception:
        services["ollama"] = False

    # SearXNG: quick HTTP ping
    try:
        import httpx

        async with httpx.AsyncClient(timeout=3.0) as client:
            resp = await client.get(f"{settings.searxng_url}/healthz")
            services["searxng"] = resp.status_code == 200
    except Exception:
        services["searxng"] = False

    # -- Gov workers --
    gov_manager = getattr(request.app.state, "gov_manager", None)
    gov_info: dict = {"workers": 0, "running": False}
    if gov_manager is not None:
        gov_info["running"] = getattr(gov_manager, "running", False)
        workers = getattr(gov_manager, "workers", [])
        gov_info["workers"] = len(workers)

    # -- Resource usage --
    process = psutil.Process(os.getpid())
    mem_info = process.memory_info()
    disk_usage = shutil.disk_usage(str(settings.data_dir))

    resources = {
        "memory_mb": round(mem_info.rss / (1024 * 1024), 1),
        "cpu_percent": process.cpu_percent(interval=0),
        "disk_total_gb": round(disk_usage.total / (1024**3), 1),
        "disk_free_gb": round(disk_usage.free / (1024**3), 1),
    }

    # -- Compute distributed GPU --
    compute_mgr = getattr(request.app.state, "compute_manager", None)
    compute_info: dict = {"running": False, "dispatcher": None}
    if compute_mgr is not None:
        compute_info = compute_mgr.get_status()

    # -- Sync --
    sync_broadcaster = getattr(request.app.state, "sync_broadcaster", None)
    sync_info: dict = {"running": False}
    if sync_broadcaster is not None:
        sync_info = sync_broadcaster.get_status()

    # -- Overall status --
    all_ok = services.get("sqlite", False) and services.get("ollama", False)
    status = "ok" if all_ok else "degraded"

    return {
        "status": status,
        "version": app.version,
        "uptime_seconds": uptime_seconds,
        "services": services,
        "gov": gov_info,
        "compute": compute_info,
        "sync": sync_info,
        "resources": resources,
    }
