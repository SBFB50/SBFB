"""
NEXUS Compute -- Compute Event Types and Database Proxy.

Defines event types for the distributed GPU computing system
and a proxy for long-lived compute workers.
"""

from __future__ import annotations

from enum import Enum
from typing import Any

from loguru import logger


class ComputeEventType(str, Enum):
    """Distributed computing event types."""

    # Node lifecycle
    COMPUTE_NODE_REGISTERED = "compute_node_registered"
    COMPUTE_NODE_CONNECTED = "compute_node_connected"
    COMPUTE_NODE_DISCONNECTED = "compute_node_disconnected"
    COMPUTE_NODE_BANNED = "compute_node_banned"

    # Task lifecycle
    COMPUTE_TASK_CREATED = "compute_task_created"
    COMPUTE_TASK_ASSIGNED = "compute_task_assigned"
    COMPUTE_TASK_COMPLETED = "compute_task_completed"
    COMPUTE_TASK_FAILED = "compute_task_failed"
    COMPUTE_TASK_EXPIRED = "compute_task_expired"

    # Validation
    COMPUTE_RESULT_VALIDATED = "compute_result_validated"
    COMPUTE_RESULT_REJECTED = "compute_result_rejected"
    COMPUTE_SPOT_CHECK_NEEDED = "compute_spot_check_needed"

    # Model management
    COMPUTE_MODEL_CHANGED = "compute_model_changed"

    # Network
    COMPUTE_NETWORK_STATS_UPDATED = "compute_network_stats_updated"

    # Periodic ticks
    COMPUTE_TICK_HEARTBEAT = "compute_tick_heartbeat"
    COMPUTE_TICK_REAPER = "compute_tick_reaper"


class ComputeDatabaseProxy:
    """Proxy that opens a fresh ComputeDatabase connection per method call.

    Long-lived workers use this instead of holding a single connection open.
    """

    def __getattr__(self, name: str) -> Any:
        from nexus.engine import get_db
        from nexus.compute.db import ComputeDatabase

        async def _method_proxy(*args: Any, **kwargs: Any) -> Any:
            async with get_db() as conn:
                db = ComputeDatabase(conn)
                method = getattr(db, name)
                return await method(*args, **kwargs)

        return _method_proxy
