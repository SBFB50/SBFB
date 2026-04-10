"""
NEXUS Compute — Distributed GPU task queue for citizen-contributed computing.

Contributors share their GPU power to run LLM inference tasks.
The server orchestrates: GPU registry, task queue, result validation.
"""

from nexus.compute.db import ComputeDatabase, init_compute_db
from nexus.compute.dispatcher import TaskDispatcher
from nexus.compute.manager import ComputeManager
from nexus.compute.model_selector import ModelSelector, MODEL_TIERS
from nexus.compute.hybrid import HybridRouter, ExoBackend, ExecutionMode
from nexus.compute.events import ComputeEventType, ComputeDatabaseProxy

__all__ = [
    "ComputeDatabase",
    "init_compute_db",
    "TaskDispatcher",
    "ComputeManager",
    "ModelSelector",
    "MODEL_TIERS",
    "HybridRouter",
    "ExoBackend",
    "ExecutionMode",
    "ComputeEventType",
    "ComputeDatabaseProxy",
]
