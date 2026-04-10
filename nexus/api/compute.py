"""
NEXUS Compute -- API endpoints for distributed GPU computing.

Endpoints:
  POST   /api/compute/register          — GPU node registers (returns api_key)
  POST   /api/compute/heartbeat         — Node heartbeat + model instructions
  GET    /api/compute/task              — Node pulls next task
  POST   /api/compute/result            — Node submits result
  GET    /api/compute/stats             — Public network statistics
  GET    /api/compute/leaderboard       — Public contributor leaderboard
  POST   /api/compute/model/ready       — Node reports model pulled successfully
  GET    /api/compute/model/status      — Model selection + transition state
  GET    /api/compute/model/assignments — Per-node model assignments
  GET    /api/compute/model/transitions — Model transition history
  POST   /api/compute/tasks             — Internal: create a compute task
  GET    /api/compute/nodes             — Internal: list registered nodes
"""

from __future__ import annotations

from collections import defaultdict
from datetime import datetime, timezone
from typing import Optional

from fastapi import APIRouter, Depends, Header, HTTPException, Request
from fastapi.responses import Response
from loguru import logger

from nexus.compute.db import ComputeDatabase, hash_ip
from nexus.compute.dispatcher import TaskDispatcher
from nexus.compute.model_selector import ModelSelector
from nexus.compute.models import (
    HybridStatusResponse,
    LeaderboardEntry,
    LeaderboardResponse,
    ModelReadyRequest,
    ModelReadyResponse,
    ModelStatusResponse,
    ModelTransitionEntry,
    NetworkStatsResponse,
    NodeAssignment,
    NodeHeartbeatRequest,
    NodeHeartbeatResponse,
    NodePublic,
    NodeRegisterRequest,
    NodeRegisterResponse,
    TaskCreateRequest,
    TaskPullResponse,
    TaskResultRequest,
    TaskResultResponse,
)
from nexus.db.sqlite_db import get_db


router = APIRouter(prefix="/api/compute", tags=["compute"])


# ============================================================================
# Rate limiting (in-memory, per IP hash)
# ============================================================================

_rate_limits: dict[str, list[float]] = defaultdict(list)
_RATE_LIMIT_PER_MINUTE = 100


def _check_rate_limit(request: Request) -> None:
    """Enforce rate limit per IP (100 requests/minute)."""
    ip = request.client.host if request.client else "unknown"
    ip_hash = hash_ip(ip)
    now = datetime.now(timezone.utc).timestamp()

    # Clean old entries (> 60s ago)
    recent = [t for t in _rate_limits[ip_hash] if now - t < 60]
    if not recent:
        _rate_limits.pop(ip_hash, None)
        recent = []
    recent.append(now)
    _rate_limits[ip_hash] = recent

    if len(recent) > _RATE_LIMIT_PER_MINUTE:
        raise HTTPException(status_code=429, detail="Rate limit exceeded (100 req/min)")


# ============================================================================
# Auth helper
# ============================================================================

async def _get_authenticated_node(
    authorization: str = Header(..., description="Bearer <api_key>"),
) -> dict:
    """Extract and validate API key from Authorization header.

    Returns the authenticated node dict or raises 401/403.
    """
    if not authorization.lower().startswith("bearer "):
        raise HTTPException(status_code=401, detail="Invalid Authorization header (expected: Bearer <api_key>)")

    api_key = authorization[7:].strip()
    if not api_key:
        raise HTTPException(status_code=401, detail="Missing API key")

    async with get_db() as conn:
        db = ComputeDatabase(conn)
        node = await db.get_node_by_api_key(api_key)

    if not node:
        raise HTTPException(status_code=401, detail="Invalid API key")

    if node.get("status") == "banned":
        raise HTTPException(status_code=403, detail="Node is banned")

    return node


def _get_dispatcher(request: Request) -> TaskDispatcher:
    """Get the TaskDispatcher from app state."""
    compute_mgr = getattr(request.app.state, "compute_manager", None)
    if not compute_mgr or not compute_mgr.dispatcher:
        raise HTTPException(status_code=503, detail="Compute system not initialized")
    return compute_mgr.dispatcher


def _get_model_selector(request: Request) -> ModelSelector:
    """Get the ModelSelector from app state."""
    compute_mgr = getattr(request.app.state, "compute_manager", None)
    if not compute_mgr or not compute_mgr.model_selector:
        raise HTTPException(status_code=503, detail="Compute system not initialized")
    return compute_mgr.model_selector


# ============================================================================
# Public endpoints (no auth required)
# ============================================================================

@router.get("/stats", response_model=NetworkStatsResponse)
async def get_network_stats(request: Request) -> NetworkStatsResponse:
    """Public network statistics — nodes, VRAM, tasks."""
    dispatcher = _get_dispatcher(request)

    async with get_db() as conn:
        db = ComputeDatabase(conn)
        stats = await db.get_network_stats()

    return NetworkStatsResponse(
        **stats,
        current_model=dispatcher.current_model,
        model_tier=dispatcher.current_tier,
    )


@router.get("/leaderboard", response_model=LeaderboardResponse)
async def get_leaderboard(
    request: Request,
    limit: int = 20,
) -> LeaderboardResponse:
    """Public contributor leaderboard."""
    async with get_db() as conn:
        db = ComputeDatabase(conn)
        entries = await db.get_leaderboard(limit=min(limit, 100))
        all_nodes = await db.list_nodes()

    return LeaderboardResponse(
        entries=[LeaderboardEntry(**e) for e in entries],
        total_contributors=len(all_nodes),
    )


# ============================================================================
# Node registration (no auth — this IS the auth creation)
# ============================================================================

@router.post("/register", response_model=NodeRegisterResponse, status_code=201)
async def register_node(
    request: Request,
    body: NodeRegisterRequest,
) -> NodeRegisterResponse:
    """Register a new GPU contributor. Returns an API key (shown once)."""
    _check_rate_limit(request)

    ip = request.client.host if request.client else "unknown"

    async with get_db() as conn:
        db = ComputeDatabase(conn)
        node, api_key = await db.register_node(
            name=body.name,
            gpu_model=body.gpu_model,
            vram_mb=body.vram_mb,
            ip=ip,
            platform=body.platform,
            ollama_version=body.ollama_version,
            public_key_pem=body.public_key_pem,
        )

    # Recalculate model tier with new node
    dispatcher = _get_dispatcher(request)
    await dispatcher.recalculate_model()

    logger.info(
        "New compute node registered: {} ({}, {} MB VRAM)",
        body.name, body.gpu_model, body.vram_mb,
    )

    return NodeRegisterResponse(
        node_id=node["id"],
        api_key=api_key,
        name=node["name"],
        gpu_model=node["gpu_model"],
        vram_mb=node["vram_mb"],
        status=node["status"],
    )


# ============================================================================
# Authenticated node endpoints
# ============================================================================

@router.post("/heartbeat", response_model=NodeHeartbeatResponse)
async def node_heartbeat(
    request: Request,
    body: NodeHeartbeatRequest,
    node: dict = Depends(_get_authenticated_node),
) -> NodeHeartbeatResponse:
    """Node heartbeat — keeps the node alive and returns model instructions.

    The response tells the node which model it should have loaded.
    If model_required differs from the node's current model, the node
    should pull the new model and report readiness via POST /model/ready.
    """
    _check_rate_limit(request)
    selector = _get_model_selector(request)

    async with get_db() as conn:
        db = ComputeDatabase(conn)
        await db.heartbeat(node["id"], body.current_model)

        if body.status == "idle" and node.get("status") != "idle":
            await db.update_node_status(node["id"], "idle")

    # Per-node model assignment based on individual VRAM
    assigned_model = selector.get_model_for_node(node.get("vram_mb", 0))
    needs_pull = body.current_model != assigned_model

    message = ""
    if needs_pull:
        message = f"pull_model:{assigned_model}"

    return NodeHeartbeatResponse(
        status=node.get("status", "idle"),
        model_required=assigned_model,
        message=message,
    )


@router.get("/task", response_model=TaskPullResponse, responses={204: {"description": "No task available"}})
async def pull_task(
    request: Request,
    node: dict = Depends(_get_authenticated_node),
):
    """Pull the next available task for this node.

    Returns 200 with task data, or 204 No Content if queue is empty.
    """
    _check_rate_limit(request)

    async with get_db() as conn:
        db = ComputeDatabase(conn)
        task = await db.pull_next_task(
            node_id=node["id"],
            model=node.get("current_model", ""),
        )

    if not task:
        return Response(status_code=204)

    # Mark node as busy
    async with get_db() as conn:
        db = ComputeDatabase(conn)
        await db.update_node_status(node["id"], "busy")

    return TaskPullResponse(
        task_id=task["id"],
        task_type=task["task_type"],
        prompt=task["prompt"],
        system_prompt=task.get("system_prompt", ""),
        model=task.get("model", ""),
        timeout_seconds=task.get("timeout_seconds", 300),
        require_logprobs=bool(task.get("require_logprobs", 0)),
        calibration_prompt=task.get("calibration_prompt", ""),
    )


@router.post("/result", response_model=TaskResultResponse)
async def submit_result(
    request: Request,
    body: TaskResultRequest,
    node: dict = Depends(_get_authenticated_node),
) -> TaskResultResponse:
    """Submit a task result. Server validates and stores it."""
    _check_rate_limit(request)
    dispatcher = _get_dispatcher(request)

    result = await dispatcher.validate_result(
        task_id=body.task_id,
        node_id=node["id"],
        result_text=body.result_text,
        tokens_generated=body.tokens_generated,
        generation_time_ms=body.generation_time_ms,
        model_digest=body.model_digest,
        logprobs=body.logprobs,
        signature=body.signature,
    )

    return TaskResultResponse(
        accepted=result["accepted"],
        task_id=body.task_id,
        message=result.get("message", ""),
        trust_delta=result.get("trust_delta", 0),
    )


# ============================================================================
# Model management endpoints (Phase 2)
# ============================================================================

@router.post("/model/ready", response_model=ModelReadyResponse)
async def report_model_ready(
    request: Request,
    body: ModelReadyRequest,
    node: dict = Depends(_get_authenticated_node),
) -> ModelReadyResponse:
    """Node reports that it has finished pulling a model.

    After receiving a pull_model instruction via heartbeat, the node
    downloads the model and calls this endpoint when done.
    """
    _check_rate_limit(request)
    selector = _get_model_selector(request)

    result = await selector.report_model_ready(node["id"], body.model)

    return ModelReadyResponse(
        accepted=result.get("accepted", False),
        message=result.get("message", ""),
        transition_state=result.get("transition_state", ""),
        readiness_pct=result.get("readiness_pct", 0.0),
    )


@router.get("/model/status", response_model=ModelStatusResponse)
async def get_model_status(request: Request) -> ModelStatusResponse:
    """Current model selection and transition status (public)."""
    selector = _get_model_selector(request)
    status = await selector.recalculate()

    return ModelStatusResponse(
        target_model=status.get("target_model", ""),
        target_tier=status.get("target_tier", ""),
        previous_model=status.get("previous_model", ""),
        transition_state=status.get("transition_state", "stable"),
        transition_started_at=selector.get_status().get("transition_started_at"),
        execution_mode=status.get("execution_mode", "local"),
        total_vram_gb=status.get("total_vram_gb", 0.0),
        max_single_node_vram_gb=status.get("max_single_node_vram_gb", 0.0),
        nodes_online=status.get("nodes_total", 0),
        nodes_ready=status.get("nodes_ready", 0),
        nodes_compatible=status.get("nodes_compatible", 0),
        nodes_pulling=status.get("nodes_pulling", 0),
        readiness_pct=status.get("readiness_pct", 100.0),
    )


@router.get("/model/assignments")
async def get_model_assignments(request: Request) -> list[NodeAssignment]:
    """Per-node model assignments (which node should run which model)."""
    selector = _get_model_selector(request)
    assignments = await selector.get_all_node_assignments()
    return [NodeAssignment(**a) for a in assignments]


@router.get("/model/transitions")
async def get_model_transitions(
    request: Request,
    limit: int = 20,
) -> list[ModelTransitionEntry]:
    """History of model transitions."""
    async with get_db() as conn:
        db = ComputeDatabase(conn)
        transitions = await db.list_transitions(limit=min(limit, 100))

    return [ModelTransitionEntry(**t) for t in transitions]


# ============================================================================
# Hybrid mode endpoints (Phase 4)
# ============================================================================

@router.get("/hybrid/status", response_model=HybridStatusResponse)
async def get_hybrid_status(request: Request) -> HybridStatusResponse:
    """Current hybrid execution mode (Ollama local vs exo distributed)."""
    selector = _get_model_selector(request)
    hybrid = selector.hybrid_router

    return HybridStatusResponse(
        execution_mode=selector.execution_mode.value,
        exo_enabled=hybrid.exo_enabled,
        exo_available=hybrid.exo_available,
        exo_url="",  # Never expose internal URLs publicly
        exo_model=hybrid.exo_model,
        max_single_node_vram_gb=hybrid.get_status().get("max_single_node_vram_gb", 0.0),
        target_model=selector.target_model,
        target_tier=selector.target_tier,
    )


# ============================================================================
# Self-worker control (embedded GPU contributor)
# ============================================================================

@router.get("/self-worker/status")
async def get_self_worker_status(request: Request) -> dict:
    """Status of the embedded self-worker (this server's GPU contribution)."""
    compute_mgr = getattr(request.app.state, "compute_manager", None)
    if not compute_mgr or not compute_mgr.self_worker:
        return {"running": False, "message": "Compute system not initialized"}
    return compute_mgr.self_worker.get_status()


@router.post("/self-worker/pause")
async def pause_self_worker(request: Request) -> dict:
    """Pause the self-worker (stop processing tasks, keep node online)."""
    compute_mgr = getattr(request.app.state, "compute_manager", None)
    if not compute_mgr or not compute_mgr.self_worker:
        return {"ok": False, "message": "Self-worker not available"}
    compute_mgr.self_worker.pause()
    return {"ok": True, "paused": True}


@router.post("/self-worker/resume")
async def resume_self_worker(request: Request) -> dict:
    """Resume the self-worker (start processing tasks again)."""
    compute_mgr = getattr(request.app.state, "compute_manager", None)
    if not compute_mgr or not compute_mgr.self_worker:
        return {"ok": False, "message": "Self-worker not available"}
    compute_mgr.self_worker.resume()
    return {"ok": True, "paused": False}


# ============================================================================
# Swarm endpoints (Phase 7 — Petals)
# ============================================================================

@router.get("/swarm/status")
async def get_swarm_status(request: Request) -> dict:
    """Petals swarm health, block coverage, and node count."""
    selector = _get_model_selector(request)
    status = selector.get_status()
    swarm = status.get("swarm", {
        "health": "offline",
        "model": "",
        "nodes_online": 0,
        "blocks_total": 0,
        "blocks_covered": 0,
        "coverage_pct": 0.0,
        "is_ready": False,
        "throughput_tok_s": 0.0,
    })
    return swarm


# ============================================================================
# Public health + uptime + impact (Phase 8)
# ============================================================================

# Cache for lightweight health endpoint
_health_cache: dict = {}
_health_cache_ts: float = 0.0

@router.get("/health")
async def get_compute_health(request: Request) -> dict:
    """Lightweight public health endpoint for the compute network.

    Designed for external monitoring (like health.petals.dev).
    Response cached for 5 seconds.
    """
    import time
    global _health_cache, _health_cache_ts

    now = time.time()
    if now - _health_cache_ts < 5.0 and _health_cache:
        return _health_cache

    selector = _get_model_selector(request)
    status = selector.get_status()
    hybrid = status.get("hybrid", {})
    swarm = status.get("swarm", {})

    async with get_db() as conn:
        db = ComputeDatabase(conn)
        net_stats = await db.get_network_stats()
        uptime = await db.get_network_uptime()

    _health_cache = {
        "status": "healthy" if net_stats.get("nodes_online", 0) > 0 else "offline",
        "model": status.get("target_model", ""),
        "tier": status.get("target_tier", ""),
        "execution_mode": status.get("execution_mode", "local"),
        "nodes_online": net_stats.get("nodes_online", 0),
        "nodes_total": net_stats.get("nodes_total", 0),
        "vram_total_gb": net_stats.get("vram_total_gb", 0.0),
        "tasks_today": net_stats.get("tasks_today", 0),
        "tasks_completed": net_stats.get("tasks_completed", 0),
        "uptime_pct": uptime.get("uptime_pct", 0.0),
        "total_node_hours_30d": uptime.get("total_node_hours_30d", 0.0),
        "swarm_health": swarm.get("health", "offline"),
        "swarm_blocks": f"{swarm.get('blocks_covered', 0)}/{swarm.get('blocks_total', 0)}",
    }
    _health_cache_ts = now
    return _health_cache


@router.get("/uptime")
async def get_network_uptime(request: Request) -> dict:
    """Network-wide uptime statistics."""
    async with get_db() as conn:
        db = ComputeDatabase(conn)
        return await db.get_network_uptime()


@router.get("/nodes/{node_id}/impact")
async def get_node_impact(request: Request, node_id: str) -> dict:
    """Detailed impact stats for a specific contributor node."""
    async with get_db() as conn:
        db = ComputeDatabase(conn)
        impact = await db.get_node_impact(node_id)
    if not impact:
        from fastapi import HTTPException
        raise HTTPException(status_code=404, detail="Node not found")
    return impact


# ============================================================================
# Badges endpoints (Phase 5)
# ============================================================================

@router.get("/badges")
async def get_badges(
    request: Request,
    node_id: Optional[str] = None,
) -> dict:
    """Get badges summary or badges for a specific node."""
    async with get_db() as conn:
        db = ComputeDatabase(conn)
        if node_id:
            badges = await db.calculate_badges(node_id)
            return {"node_id": node_id, "badges": badges}
        else:
            summary = await db.get_all_badges_summary()
            return {"summary": summary}


# ============================================================================
# Internal/admin endpoints
# ============================================================================

@router.post("/tasks", status_code=201)
async def create_compute_task(
    request: Request,
    body: TaskCreateRequest,
) -> dict:
    """Create a compute task (internal/admin use)."""
    dispatcher = _get_dispatcher(request)

    task = await dispatcher.submit_task(
        task_type=body.task_type,
        prompt=body.prompt,
        system_prompt=body.system_prompt,
        priority=body.priority,
        timeout_seconds=body.timeout_seconds,
        source_worker=body.source_worker,
        require_logprobs=body.require_logprobs,
        max_retries=body.max_retries,
    )

    return {"task_id": task["id"], "status": task["status"], "model": task.get("model", "")}


@router.get("/nodes")
async def list_nodes(
    request: Request,
    status: Optional[str] = None,
) -> list[NodePublic]:
    """List all registered compute nodes."""
    async with get_db() as conn:
        db = ComputeDatabase(conn)
        nodes = await db.list_nodes(status=status)

    return [
        NodePublic(
            id=n["id"],
            name=n["name"],
            gpu_model=n["gpu_model"],
            vram_mb=n["vram_mb"],
            status=n["status"],
            tasks_completed=n.get("tasks_completed", 0),
            tasks_errored=n.get("tasks_errored", 0),
            avg_tokens_per_sec=n.get("avg_tokens_per_sec", 0.0),
            trust_score=n.get("trust_score", 50),
            connected_at=n.get("connected_at"),
        )
        for n in nodes
    ]
