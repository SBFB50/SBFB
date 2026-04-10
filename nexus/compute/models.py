"""
NEXUS Compute -- Pydantic models for the compute API.

Request/response models for GPU node registration, task management,
and result submission.
"""

from __future__ import annotations

from datetime import datetime
from typing import Any, Optional

from pydantic import BaseModel, Field


# ============================================================================
# Node models
# ============================================================================

class NodeRegisterRequest(BaseModel):
    """Request to register a new GPU contributor node."""
    name: str = Field(..., min_length=1, max_length=64, description="Contributor display name")
    gpu_model: str = Field(..., min_length=1, max_length=128, description="GPU model name (e.g. 'NVIDIA RTX 5080')")
    vram_mb: int = Field(..., gt=0, le=1048576, description="Total VRAM in megabytes (max 1TB)")
    platform: str = Field("", max_length=64, description="OS platform (e.g. 'windows', 'linux', 'darwin')")
    ollama_version: str = Field("", max_length=32, description="Ollama version string")
    public_key_pem: str = Field("", description="Ed25519 public key PEM for result signing")


class NodeRegisterResponse(BaseModel):
    """Response after successful node registration."""
    node_id: str
    api_key: str = Field(..., description="API key — store securely, shown only once")
    name: str
    gpu_model: str
    vram_mb: int
    status: str = "idle"


class NodeHeartbeatRequest(BaseModel):
    """Heartbeat payload from a connected node."""
    current_model: str = Field("", description="Currently loaded Ollama model")
    status: str = Field("idle", description="Node self-reported status")


class NodeHeartbeatResponse(BaseModel):
    """Server response to a heartbeat."""
    status: str = Field(..., description="Server-assigned node status")
    model_required: str = Field("", description="Model the node should have loaded")
    message: str = ""


class NodePublic(BaseModel):
    """Public representation of a compute node (no secrets)."""
    id: str
    name: str
    gpu_model: str
    vram_mb: int
    status: str
    tasks_completed: int = 0
    tasks_errored: int = 0
    avg_tokens_per_sec: float = 0.0
    trust_score: int = 50
    connected_at: Optional[str] = None


# ============================================================================
# Task models
# ============================================================================

class TaskPullResponse(BaseModel):
    """Task assigned to a contributor node for execution."""
    task_id: str
    task_type: str
    prompt: str
    system_prompt: str = ""
    model: str = ""
    timeout_seconds: int = 300
    require_logprobs: bool = False
    calibration_prompt: str = ""


class TaskResultRequest(BaseModel):
    """Result submission from a contributor node."""
    task_id: str = Field(..., description="ID of the completed task")
    result_text: str = Field(..., min_length=1, description="LLM generation output")
    tokens_generated: int = Field(0, ge=0, description="Number of tokens generated")
    generation_time_ms: int = Field(0, ge=0, description="Generation time in milliseconds")
    model_digest: str = Field("", description="SHA256 digest of the Ollama model file")
    logprobs: str = Field("", description="JSON-serialized logprobs if requested")
    signature: str = Field("", description="Ed25519 signature of the result payload")


class TaskResultResponse(BaseModel):
    """Server response after result submission."""
    accepted: bool
    task_id: str
    message: str = ""
    trust_delta: int = 0


class TaskCreateRequest(BaseModel):
    """Internal request to create a compute task (server-side only)."""
    task_type: str
    prompt: str
    system_prompt: str = ""
    model: str = ""
    priority: int = Field(5, ge=1, le=10)
    timeout_seconds: int = Field(300, ge=30, le=3600)
    source_worker: str = ""
    require_logprobs: bool = False
    max_retries: int = Field(3, ge=0, le=10)


# ============================================================================
# Stats models
# ============================================================================

class NetworkStatsResponse(BaseModel):
    """Public network statistics."""
    nodes_online: int = 0
    nodes_total: int = 0
    vram_total_gb: float = 0.0
    tasks_pending: int = 0
    tasks_assigned: int = 0
    tasks_completed: int = 0
    tasks_failed: int = 0
    tasks_today: int = 0
    current_model: str = ""
    model_tier: str = ""


class LeaderboardEntry(BaseModel):
    """Single entry in the contributor leaderboard."""
    rank: int
    name: str
    gpu_model: str
    vram_mb: int
    tasks_completed: int
    avg_tokens_per_sec: float = 0.0
    trust_score: int = 50
    status: str = "offline"


class LeaderboardResponse(BaseModel):
    """Contributor leaderboard."""
    entries: list[LeaderboardEntry] = []
    total_contributors: int = 0


# ============================================================================
# Model management models (Phase 2)
# ============================================================================

class ModelReadyRequest(BaseModel):
    """Node reports that it has finished pulling a model."""
    model: str = Field(..., min_length=1, description="Model name that was pulled successfully")
    model_digest: str = Field("", description="SHA256 digest of the model file (from Ollama)")


class ModelReadyResponse(BaseModel):
    """Server acknowledges model readiness."""
    accepted: bool
    message: str = ""
    transition_state: str = ""
    readiness_pct: float = 0.0


class ModelStatusResponse(BaseModel):
    """Current model selection and transition status."""
    target_model: str = ""
    target_tier: str = ""
    previous_model: str = ""
    transition_state: str = "stable"
    transition_started_at: Optional[str] = None
    execution_mode: str = "local"
    total_vram_gb: float = 0.0
    max_single_node_vram_gb: float = 0.0
    nodes_online: int = 0
    nodes_ready: int = 0
    nodes_compatible: int = 0
    nodes_pulling: int = 0
    readiness_pct: float = 100.0


class NodeAssignment(BaseModel):
    """Model assignment for a specific node."""
    node_id: str
    name: str
    vram_mb: int
    assigned_model: str
    current_model: str
    ready: bool
    needs_pull: bool


class HybridStatusResponse(BaseModel):
    """Hybrid execution mode status (Ollama local vs exo distributed)."""
    execution_mode: str = "local"
    exo_enabled: bool = False
    exo_available: bool = False
    exo_url: str = ""
    exo_model: str = ""
    max_single_node_vram_gb: float = 0.0
    target_model: str = ""
    target_tier: str = ""


class ModelTransitionEntry(BaseModel):
    """A model transition record."""
    id: str
    old_model: str = ""
    new_model: str = ""
    old_tier: str = ""
    new_tier: str = ""
    total_vram_gb: float = 0.0
    nodes_online: int = 0
    nodes_ready: int = 0
    transition_state: str = ""
    started_at: Optional[str] = None
    completed_at: Optional[str] = None
