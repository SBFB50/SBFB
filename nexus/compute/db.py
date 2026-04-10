"""
NEXUS Compute -- Distributed GPU database layer.

DDL + CRUD for distributed computing tables:
- compute_nodes     (GPU contributor registry)
- compute_tasks     (LLM inference task queue)
- compute_results   (validated task results)

Uses the same helpers and connection management as sqlite_db.py.
"""

from __future__ import annotations

import hashlib
import json
import secrets
import uuid
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional

import aiosqlite
from loguru import logger

from nexus.engine import (
    get_db,
    _new_id,
    _now_iso,
    _json_dumps,
    _json_loads,
    _row_to_dict,
    _dict_with_json_fields,
)


# ============================================================================
# SQL DDL
# ============================================================================

_COMPUTE_CREATE_TABLES = """
CREATE TABLE IF NOT EXISTS compute_nodes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    gpu_model TEXT NOT NULL,
    vram_mb INTEGER NOT NULL,
    platform TEXT DEFAULT '',
    ollama_version TEXT DEFAULT '',
    status TEXT DEFAULT 'idle',
    connected_at DATETIME,
    last_heartbeat DATETIME,
    tasks_completed INTEGER DEFAULT 0,
    tasks_errored INTEGER DEFAULT 0,
    avg_tokens_per_sec REAL DEFAULT 0.0,
    trust_score INTEGER DEFAULT 50,
    api_key_hash TEXT NOT NULL,
    ip_hash TEXT NOT NULL,
    public_key TEXT DEFAULT '',
    current_model TEXT DEFAULT '',
    assigned_model TEXT DEFAULT '',
    model_status TEXT DEFAULT '',
    model_pull_started_at DATETIME,
    metadata TEXT DEFAULT '{}',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS compute_model_transitions (
    id TEXT PRIMARY KEY,
    old_model TEXT DEFAULT '',
    new_model TEXT DEFAULT '',
    old_tier TEXT DEFAULT '',
    new_tier TEXT DEFAULT '',
    total_vram_gb REAL DEFAULT 0.0,
    nodes_online INTEGER DEFAULT 0,
    nodes_ready INTEGER DEFAULT 0,
    transition_state TEXT DEFAULT 'transitioning',
    started_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME
);

CREATE TABLE IF NOT EXISTS compute_tasks (
    id TEXT PRIMARY KEY,
    task_type TEXT NOT NULL,
    prompt TEXT NOT NULL,
    system_prompt TEXT DEFAULT '',
    model TEXT DEFAULT '',
    status TEXT DEFAULT 'pending',
    priority INTEGER DEFAULT 5,
    assigned_to TEXT REFERENCES compute_nodes(id),
    assigned_at DATETIME,
    completed_at DATETIME,
    result TEXT,
    result_validated INTEGER DEFAULT 0,
    validation_score REAL DEFAULT 0.0,
    timeout_seconds INTEGER DEFAULT 300,
    require_logprobs INTEGER DEFAULT 0,
    calibration_prompt TEXT DEFAULT '',
    source_worker TEXT DEFAULT '',
    parent_task_id TEXT DEFAULT '',
    error_message TEXT DEFAULT '',
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    execution_mode TEXT DEFAULT 'local',
    metadata TEXT DEFAULT '{}',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS compute_results (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES compute_tasks(id),
    node_id TEXT NOT NULL REFERENCES compute_nodes(id),
    result_text TEXT NOT NULL,
    tokens_generated INTEGER DEFAULT 0,
    generation_time_ms INTEGER DEFAULT 0,
    model_digest TEXT DEFAULT '',
    logprobs TEXT DEFAULT '',
    signature TEXT DEFAULT '',
    validated INTEGER DEFAULT 0,
    validation_method TEXT DEFAULT '',
    metadata TEXT DEFAULT '{}',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS compute_badges (
    id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL REFERENCES compute_nodes(id),
    badge_id TEXT NOT NULL,
    badge_name TEXT NOT NULL,
    awarded_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(node_id, badge_id)
);

CREATE TABLE IF NOT EXISTS compute_uptime_log (
    id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL REFERENCES compute_nodes(id),
    connected_at DATETIME NOT NULL,
    disconnected_at DATETIME,
    duration_seconds INTEGER DEFAULT 0
);
"""

_COMPUTE_CREATE_INDEXES = """
CREATE INDEX IF NOT EXISTS idx_compute_nodes_status ON compute_nodes(status);
CREATE INDEX IF NOT EXISTS idx_compute_nodes_trust ON compute_nodes(trust_score);
CREATE INDEX IF NOT EXISTS idx_compute_nodes_api_key ON compute_nodes(api_key_hash);
CREATE INDEX IF NOT EXISTS idx_compute_tasks_status ON compute_tasks(status);
CREATE INDEX IF NOT EXISTS idx_compute_tasks_priority ON compute_tasks(priority, created_at);
CREATE INDEX IF NOT EXISTS idx_compute_tasks_assigned ON compute_tasks(assigned_to, status);
CREATE INDEX IF NOT EXISTS idx_compute_tasks_type ON compute_tasks(task_type);
CREATE INDEX IF NOT EXISTS idx_compute_results_task ON compute_results(task_id);
CREATE INDEX IF NOT EXISTS idx_compute_results_node ON compute_results(node_id);
CREATE INDEX IF NOT EXISTS idx_compute_nodes_model_status ON compute_nodes(model_status);
CREATE INDEX IF NOT EXISTS idx_compute_nodes_assigned_model ON compute_nodes(assigned_model);
CREATE INDEX IF NOT EXISTS idx_compute_transitions_state ON compute_model_transitions(transition_state);
CREATE INDEX IF NOT EXISTS idx_compute_badges_node ON compute_badges(node_id);
CREATE INDEX IF NOT EXISTS idx_compute_uptime_node ON compute_uptime_log(node_id);
CREATE INDEX IF NOT EXISTS idx_compute_uptime_connected ON compute_uptime_log(connected_at);
"""


# ============================================================================
# Auth helpers
# ============================================================================

def _generate_api_key() -> str:
    """Generate a secure API key for a compute node."""
    return secrets.token_urlsafe(32)


def _hash_api_key(api_key: str) -> str:
    """SHA-256 hash of an API key for storage."""
    return hashlib.sha256(api_key.encode()).hexdigest()


def _hash_ip(ip: str) -> str:
    """SHA-256 hash of an IP address (privacy: never store raw IP)."""
    return hashlib.sha256(ip.encode()).hexdigest()


# ============================================================================
# Init
# ============================================================================

async def init_compute_db() -> None:
    """Create distributed computing tables and indexes (idempotent)."""
    async with get_db() as conn:
        await conn.execute("PRAGMA journal_mode = WAL")
        await conn.execute("PRAGMA foreign_keys = ON")
        await conn.execute("PRAGMA synchronous = NORMAL")
        await conn.executescript(_COMPUTE_CREATE_TABLES)
        await conn.executescript(_COMPUTE_CREATE_INDEXES)

        # -- Phase 2 migration: add model tracking columns if missing
        try:
            cursor = await conn.execute("PRAGMA table_info(compute_nodes)")
            existing_cols = {row[1] for row in await cursor.fetchall()}
            for col, default in [
                ("assigned_model", "''"),
                ("model_status", "''"),
                ("model_pull_started_at", "NULL"),
            ]:
                if col not in existing_cols:
                    await conn.execute(
                        f"ALTER TABLE compute_nodes ADD COLUMN {col} TEXT DEFAULT {default}"
                    )
        except Exception as exc:
            logger.debug("compute_nodes migration check: {}", exc)

        # -- Phase 4 migration: add execution_mode to tasks if missing
        try:
            cursor = await conn.execute("PRAGMA table_info(compute_tasks)")
            existing_cols = {row[1] for row in await cursor.fetchall()}
            if "execution_mode" not in existing_cols:
                await conn.execute(
                    "ALTER TABLE compute_tasks ADD COLUMN execution_mode TEXT DEFAULT 'local'"
                )
        except Exception as exc:
            logger.debug("compute_tasks migration check: {}", exc)
        await conn.commit()
    logger.info("Compute distributed GPU tables initialised")


# ============================================================================
# Database CRUD class
# ============================================================================

class ComputeDatabase:
    """CRUD operations for distributed GPU computing tables.

    Usage::

        async with get_db() as conn:
            db = ComputeDatabase(conn)
            node = await db.register_node(name="FlowUP", gpu_model="RTX 5080", ...)
    """

    def __init__(self, conn: aiosqlite.Connection) -> None:
        self._conn = conn

    # ------------------------------------------------------------------
    # Nodes — Registration & Management
    # ------------------------------------------------------------------

    async def register_node(
        self,
        name: str,
        gpu_model: str,
        vram_mb: int,
        ip: str,
        platform: str = "",
        ollama_version: str = "",
        public_key_pem: str = "",
        metadata: Optional[dict] = None,
    ) -> tuple[dict, str]:
        """Register a new GPU contributor node.

        Returns (node_dict, raw_api_key) — the raw key is shown once and
        never stored; only the hash is persisted.
        """
        node_id = _new_id()
        api_key = _generate_api_key()
        api_key_hash = _hash_api_key(api_key)
        ip_hash = _hash_ip(ip)
        now = _now_iso()

        await self._conn.execute(
            """INSERT INTO compute_nodes
               (id, name, gpu_model, vram_mb, platform, ollama_version,
                status, connected_at, last_heartbeat,
                api_key_hash, ip_hash, public_key,
                metadata, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, 'idle', ?, ?, ?, ?, ?, ?, ?, ?)""",
            (node_id, name, gpu_model, vram_mb, platform, ollama_version,
             now, now, api_key_hash, ip_hash, public_key_pem,
             _json_dumps(metadata or {}), now, now),
        )
        await self._conn.commit()

        node = await self.get_node(node_id)
        return node, api_key

    async def get_node(self, node_id: str) -> Optional[dict]:
        """Get a single node by ID."""
        cursor = await self._conn.execute(
            "SELECT * FROM compute_nodes WHERE id = ?", (node_id,)
        )
        row = await cursor.fetchone()
        if not row:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def get_node_by_api_key(self, api_key: str) -> Optional[dict]:
        """Look up a node by its raw API key (hashed for comparison)."""
        key_hash = _hash_api_key(api_key)
        cursor = await self._conn.execute(
            "SELECT * FROM compute_nodes WHERE api_key_hash = ?", (key_hash,)
        )
        row = await cursor.fetchone()
        if not row:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def list_nodes(
        self,
        status: Optional[str] = None,
        min_trust: Optional[int] = None,
    ) -> list[dict]:
        """List all registered nodes, optionally filtered."""
        conditions = []
        params: list = []

        if status:
            conditions.append("status = ?")
            params.append(status)
        if min_trust is not None:
            conditions.append("trust_score >= ?")
            params.append(min_trust)

        where = f" WHERE {' AND '.join(conditions)}" if conditions else ""
        cursor = await self._conn.execute(
            f"SELECT * FROM compute_nodes{where} ORDER BY tasks_completed DESC",
            params,
        )
        rows = await cursor.fetchall()
        return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]

    async def get_online_nodes(self) -> list[dict]:
        """Get all nodes with status idle or busy."""
        cursor = await self._conn.execute(
            "SELECT * FROM compute_nodes WHERE status IN ('idle', 'busy') "
            "ORDER BY trust_score DESC, avg_tokens_per_sec DESC"
        )
        rows = await cursor.fetchall()
        return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]

    async def heartbeat(self, node_id: str, current_model: str = "") -> bool:
        """Update node heartbeat timestamp. Returns True if node exists."""
        now = _now_iso()
        cursor = await self._conn.execute(
            """UPDATE compute_nodes
               SET last_heartbeat = ?, current_model = ?, updated_at = ?
               WHERE id = ?""",
            (now, current_model, now, node_id),
        )
        await self._conn.commit()
        return cursor.rowcount > 0

    async def update_node_status(self, node_id: str, status: str) -> bool:
        """Update node status (idle, busy, offline, banned)."""
        now = _now_iso()
        cursor = await self._conn.execute(
            "UPDATE compute_nodes SET status = ?, updated_at = ? WHERE id = ?",
            (status, now, node_id),
        )
        await self._conn.commit()
        return cursor.rowcount > 0

    async def update_node_trust(self, node_id: str, delta: int) -> int:
        """Adjust trust score by delta. Returns new score. Clamps to [0, 100]."""
        cursor = await self._conn.execute(
            "SELECT trust_score FROM compute_nodes WHERE id = ?", (node_id,)
        )
        row = await cursor.fetchone()
        if not row:
            return 0
        new_score = max(0, min(100, row[0] + delta))
        now = _now_iso()
        await self._conn.execute(
            "UPDATE compute_nodes SET trust_score = ?, updated_at = ? WHERE id = ?",
            (new_score, now, node_id),
        )
        await self._conn.commit()
        return new_score

    async def increment_node_stats(
        self,
        node_id: str,
        completed: int = 0,
        errored: int = 0,
        tokens_per_sec: Optional[float] = None,
    ) -> None:
        """Increment task counters and optionally update avg tokens/s."""
        now = _now_iso()
        if tokens_per_sec is not None:
            # Exponential moving average (alpha=0.2)
            await self._conn.execute(
                """UPDATE compute_nodes SET
                   tasks_completed = tasks_completed + ?,
                   tasks_errored = tasks_errored + ?,
                   avg_tokens_per_sec = CASE
                     WHEN avg_tokens_per_sec = 0 THEN ?
                     ELSE avg_tokens_per_sec * 0.8 + ? * 0.2
                   END,
                   updated_at = ?
                   WHERE id = ?""",
                (completed, errored, tokens_per_sec, tokens_per_sec, now, node_id),
            )
        else:
            await self._conn.execute(
                """UPDATE compute_nodes SET
                   tasks_completed = tasks_completed + ?,
                   tasks_errored = tasks_errored + ?,
                   updated_at = ?
                   WHERE id = ?""",
                (completed, errored, now, node_id),
            )
        await self._conn.commit()

    async def ban_node(self, node_id: str) -> None:
        """Ban a node (sets status to banned, trust to 0, unassigns tasks)."""
        now = _now_iso()
        # Unassign any tasks assigned to this node
        await self._conn.execute(
            """UPDATE compute_tasks SET status = 'pending', assigned_to = NULL,
               assigned_at = NULL WHERE assigned_to = ? AND status = 'assigned'""",
            (node_id,),
        )
        await self._conn.execute(
            "UPDATE compute_nodes SET status = 'banned', trust_score = 0, updated_at = ? WHERE id = ?",
            (now, node_id),
        )
        await self._conn.commit()
        logger.warning("Compute node {} BANNED (tasks unassigned)", node_id)

    async def delete_node(self, node_id: str) -> bool:
        """Remove a node entirely (cascade: unassign tasks, delete all references)."""
        # Unassign any tasks assigned to this node
        await self._conn.execute(
            """UPDATE compute_tasks SET status = 'pending', assigned_to = NULL,
               assigned_at = NULL WHERE assigned_to = ? AND status = 'assigned'""",
            (node_id,),
        )
        # Delete all FK-dependent rows before deleting node
        await self._conn.execute("DELETE FROM compute_results WHERE node_id = ?", (node_id,))
        await self._conn.execute("DELETE FROM compute_badges WHERE node_id = ?", (node_id,))
        await self._conn.execute("DELETE FROM compute_uptime_log WHERE node_id = ?", (node_id,))
        cursor = await self._conn.execute(
            "DELETE FROM compute_nodes WHERE id = ?", (node_id,)
        )
        await self._conn.commit()
        return cursor.rowcount > 0

    # ------------------------------------------------------------------
    # Nodes — Model management (Phase 2)
    # ------------------------------------------------------------------

    async def update_node_model_status(
        self, node_id: str, model: str, status: str,
    ) -> bool:
        """Update a node's model status (pulling, ready, failed).

        When status='ready', also sets current_model to the new model.
        """
        now = _now_iso()
        if status == "ready":
            cursor = await self._conn.execute(
                """UPDATE compute_nodes
                   SET current_model = ?, assigned_model = ?, model_status = ?,
                       updated_at = ?
                   WHERE id = ?""",
                (model, model, status, now, node_id),
            )
        elif status == "pulling":
            cursor = await self._conn.execute(
                """UPDATE compute_nodes
                   SET assigned_model = ?, model_status = ?,
                       model_pull_started_at = ?, updated_at = ?
                   WHERE id = ?""",
                (model, status, now, now, node_id),
            )
        else:
            cursor = await self._conn.execute(
                """UPDATE compute_nodes
                   SET model_status = ?, updated_at = ?
                   WHERE id = ?""",
                (status, now, node_id),
            )
        await self._conn.commit()
        return cursor.rowcount > 0

    async def set_node_assigned_model(self, node_id: str, model: str) -> bool:
        """Set the model a node should be running (from ModelSelector)."""
        now = _now_iso()
        cursor = await self._conn.execute(
            "UPDATE compute_nodes SET assigned_model = ?, updated_at = ? WHERE id = ?",
            (model, now, node_id),
        )
        await self._conn.commit()
        return cursor.rowcount > 0

    async def get_nodes_by_model(self, model: str) -> list[dict]:
        """Get all online nodes that have a specific model ready."""
        cursor = await self._conn.execute(
            """SELECT * FROM compute_nodes
               WHERE status IN ('idle', 'busy')
                 AND current_model = ?
               ORDER BY trust_score DESC""",
            (model,),
        )
        rows = await cursor.fetchall()
        return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]

    async def get_nodes_needing_pull(self) -> list[dict]:
        """Get online nodes whose assigned_model differs from current_model."""
        cursor = await self._conn.execute(
            """SELECT * FROM compute_nodes
               WHERE status IN ('idle', 'busy')
                 AND assigned_model != ''
                 AND assigned_model != current_model
                 AND model_status != 'pulling'""",
        )
        rows = await cursor.fetchall()
        return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]

    # ------------------------------------------------------------------
    # Model transitions (Phase 2)
    # ------------------------------------------------------------------

    async def create_transition(
        self,
        old_model: str,
        new_model: str,
        old_tier: str,
        new_tier: str,
        total_vram_gb: float,
        nodes_online: int,
    ) -> dict:
        """Record a model transition event."""
        tid = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO compute_model_transitions
               (id, old_model, new_model, old_tier, new_tier,
                total_vram_gb, nodes_online, transition_state, started_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, 'transitioning', ?)""",
            (tid, old_model, new_model, old_tier, new_tier,
             total_vram_gb, nodes_online, now),
        )
        await self._conn.commit()
        return {"id": tid, "old_model": old_model, "new_model": new_model}

    async def complete_transition(self, transition_id: str, nodes_ready: int) -> None:
        """Mark a transition as completed."""
        now = _now_iso()
        await self._conn.execute(
            """UPDATE compute_model_transitions
               SET transition_state = 'stable', nodes_ready = ?,
                   completed_at = ?
               WHERE id = ?""",
            (nodes_ready, now, transition_id),
        )
        await self._conn.commit()

    async def get_active_transition(self) -> Optional[dict]:
        """Get the currently active (non-completed) model transition."""
        cursor = await self._conn.execute(
            """SELECT * FROM compute_model_transitions
               WHERE transition_state = 'transitioning'
               ORDER BY started_at DESC LIMIT 1""",
        )
        row = await cursor.fetchone()
        return _row_to_dict(row) if row else None

    async def list_transitions(self, limit: int = 20) -> list[dict]:
        """List recent model transitions."""
        cursor = await self._conn.execute(
            "SELECT * FROM compute_model_transitions ORDER BY started_at DESC LIMIT ?",
            (limit,),
        )
        rows = await cursor.fetchall()
        return [_row_to_dict(r) for r in rows]

    # ------------------------------------------------------------------
    # Tasks — Queue Management
    # ------------------------------------------------------------------

    async def create_task(
        self,
        task_type: str,
        prompt: str,
        system_prompt: str = "",
        model: str = "",
        priority: int = 5,
        timeout_seconds: int = 300,
        source_worker: str = "",
        parent_task_id: str = "",
        require_logprobs: bool = False,
        calibration_prompt: str = "",
        max_retries: int = 3,
        execution_mode: str = "local",
        metadata: Optional[dict] = None,
    ) -> dict:
        """Create a new LLM inference task in the queue."""
        task_id = _new_id()
        now = _now_iso()

        await self._conn.execute(
            """INSERT INTO compute_tasks
               (id, task_type, prompt, system_prompt, model, status, priority,
                timeout_seconds, source_worker, parent_task_id,
                require_logprobs, calibration_prompt, max_retries,
                execution_mode, metadata, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, 'pending', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (task_id, task_type, prompt, system_prompt, model, priority,
             timeout_seconds, source_worker, parent_task_id,
             1 if require_logprobs else 0, calibration_prompt, max_retries,
             execution_mode, _json_dumps(metadata or {}), now, now),
        )
        await self._conn.commit()
        return await self.get_task(task_id)

    async def get_task(self, task_id: str) -> Optional[dict]:
        """Get a single task by ID."""
        cursor = await self._conn.execute(
            "SELECT * FROM compute_tasks WHERE id = ?", (task_id,)
        )
        row = await cursor.fetchone()
        if not row:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def pull_next_task(self, node_id: str, model: str = "") -> Optional[dict]:
        """Atomically assign the highest-priority pending task to a node.

        Uses BEGIN IMMEDIATE to prevent race conditions where two nodes
        could be assigned the same task. Prioritizes tasks matching the
        node's current model (affinity), then falls back to any pending task.
        Returns the assigned task or None if queue is empty.
        """
        now = _now_iso()

        # BEGIN IMMEDIATE acquires a RESERVED lock immediately,
        # preventing concurrent writers from entering the critical section.
        await self._conn.execute("BEGIN IMMEDIATE")
        try:
            # Try model-affinity first
            row = None
            if model:
                cursor = await self._conn.execute(
                    """SELECT id FROM compute_tasks
                       WHERE status = 'pending' AND (model = ? OR model = '')
                       ORDER BY priority ASC, created_at ASC LIMIT 1""",
                    (model,),
                )
                row = await cursor.fetchone()

            if not row:
                # Fall back to any pending task
                cursor = await self._conn.execute(
                    """SELECT id FROM compute_tasks
                       WHERE status = 'pending'
                       ORDER BY priority ASC, created_at ASC LIMIT 1""",
                )
                row = await cursor.fetchone()

            if not row:
                await self._conn.execute("ROLLBACK")
                return None

            task_id = row[0]
            cursor = await self._conn.execute(
                """UPDATE compute_tasks
                   SET status = 'assigned', assigned_to = ?, assigned_at = ?, updated_at = ?
                   WHERE id = ? AND status = 'pending'""",
                (node_id, now, now, task_id),
            )

            if cursor.rowcount == 0:
                # Another transaction got it first (edge case)
                await self._conn.execute("ROLLBACK")
                return None

            await self._conn.execute("COMMIT")
        except Exception:
            await self._conn.execute("ROLLBACK")
            raise

        return await self.get_task(task_id)

    async def complete_task(
        self,
        task_id: str,
        result: str,
        validated: bool = False,
        validation_score: float = 0.0,
    ) -> Optional[dict]:
        """Mark a task as completed with its result."""
        now = _now_iso()
        await self._conn.execute(
            """UPDATE compute_tasks
               SET status = 'completed', result = ?,
                   result_validated = ?, validation_score = ?,
                   completed_at = ?, updated_at = ?
               WHERE id = ?""",
            (result, 1 if validated else 0, validation_score, now, now, task_id),
        )
        await self._conn.commit()
        return await self.get_task(task_id)

    async def fail_task(self, task_id: str, error_message: str) -> Optional[dict]:
        """Mark a task as failed. If retries remain, reset to pending."""
        task = await self.get_task(task_id)
        if not task:
            return None

        now = _now_iso()
        retry_count = task.get("retry_count", 0) + 1
        max_retries = task.get("max_retries", 3)

        if retry_count < max_retries:
            # Reset to pending for retry
            await self._conn.execute(
                """UPDATE compute_tasks
                   SET status = 'pending', assigned_to = NULL, assigned_at = NULL,
                       retry_count = ?, error_message = ?, updated_at = ?
                   WHERE id = ?""",
                (retry_count, error_message, now, task_id),
            )
        else:
            # Max retries reached — mark as failed permanently
            await self._conn.execute(
                """UPDATE compute_tasks
                   SET status = 'failed', retry_count = ?,
                       error_message = ?, updated_at = ?
                   WHERE id = ?""",
                (retry_count, error_message, now, task_id),
            )
        await self._conn.commit()
        return await self.get_task(task_id)

    async def expire_stale_tasks(self, timeout_seconds: int = 600) -> int:
        """Reset tasks stuck in 'assigned' state beyond their timeout.

        Returns the number of tasks reset.
        """
        now = _now_iso()
        cursor = await self._conn.execute(
            """UPDATE compute_tasks
               SET status = 'pending', assigned_to = NULL, assigned_at = NULL,
                   retry_count = retry_count + 1, updated_at = ?
               WHERE status = 'assigned'
                 AND assigned_at IS NOT NULL
                 AND (julianday(?) - julianday(assigned_at)) * 86400 > timeout_seconds
                 AND retry_count < max_retries""",
            (now, now),
        )
        await self._conn.commit()
        return cursor.rowcount

    async def list_tasks(
        self,
        status: Optional[str] = None,
        task_type: Optional[str] = None,
        assigned_to: Optional[str] = None,
        limit: int = 100,
    ) -> list[dict]:
        """List tasks with optional filters."""
        conditions = []
        params: list = []

        if status:
            conditions.append("status = ?")
            params.append(status)
        if task_type:
            conditions.append("task_type = ?")
            params.append(task_type)
        if assigned_to:
            conditions.append("assigned_to = ?")
            params.append(assigned_to)

        where = f" WHERE {' AND '.join(conditions)}" if conditions else ""
        params.append(limit)
        cursor = await self._conn.execute(
            f"SELECT * FROM compute_tasks{where} ORDER BY priority ASC, created_at ASC LIMIT ?",
            params,
        )
        rows = await cursor.fetchall()
        return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]

    async def count_tasks(self, status: Optional[str] = None) -> int:
        """Count tasks, optionally filtered by status."""
        if status:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM compute_tasks WHERE status = ?", (status,)
            )
        else:
            cursor = await self._conn.execute("SELECT COUNT(*) FROM compute_tasks")
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # Results — Storage & Validation
    # ------------------------------------------------------------------

    async def store_result(
        self,
        task_id: str,
        node_id: str,
        result_text: str,
        tokens_generated: int = 0,
        generation_time_ms: int = 0,
        model_digest: str = "",
        logprobs: str = "",
        signature: str = "",
        metadata: Optional[dict] = None,
    ) -> dict:
        """Store a compute result from a contributor node."""
        result_id = _new_id()
        now = _now_iso()

        await self._conn.execute(
            """INSERT INTO compute_results
               (id, task_id, node_id, result_text, tokens_generated,
                generation_time_ms, model_digest, logprobs, signature,
                metadata, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (result_id, task_id, node_id, result_text, tokens_generated,
             generation_time_ms, model_digest, logprobs, signature,
             _json_dumps(metadata or {}), now),
        )
        await self._conn.commit()

        cursor = await self._conn.execute(
            "SELECT * FROM compute_results WHERE id = ?", (result_id,)
        )
        row = await cursor.fetchone()
        return _dict_with_json_fields(_row_to_dict(row), "metadata") if row else {}

    async def validate_result(self, result_id: str, method: str = "spot_check") -> None:
        """Mark a result as validated."""
        await self._conn.execute(
            "UPDATE compute_results SET validated = 1, validation_method = ? WHERE id = ?",
            (method, result_id),
        )
        await self._conn.commit()

    # ------------------------------------------------------------------
    # Stats & Leaderboard
    # ------------------------------------------------------------------

    async def get_network_stats(self) -> dict:
        """Get public network statistics."""
        nodes = await self.get_online_nodes()
        total_vram_mb = sum(n.get("vram_mb", 0) for n in nodes)

        pending = await self.count_tasks("pending")
        assigned = await self.count_tasks("assigned")
        completed = await self.count_tasks("completed")
        failed = await self.count_tasks("failed")

        # Tasks completed today
        cursor = await self._conn.execute(
            """SELECT COUNT(*) FROM compute_tasks
               WHERE status = 'completed'
                 AND completed_at >= date('now')""",
        )
        row = await cursor.fetchone()
        tasks_today = row[0] if row else 0

        return {
            "nodes_online": len(nodes),
            "nodes_total": len(await self.list_nodes()),
            "vram_total_gb": round(total_vram_mb / 1024, 1),
            "tasks_pending": pending,
            "tasks_assigned": assigned,
            "tasks_completed": completed,
            "tasks_failed": failed,
            "tasks_today": tasks_today,
        }

    async def get_leaderboard(self, limit: int = 20) -> list[dict]:
        """Get top contributors by tasks completed."""
        cursor = await self._conn.execute(
            """SELECT id, name, gpu_model, vram_mb, tasks_completed,
                      tasks_errored, avg_tokens_per_sec, trust_score,
                      status, connected_at
               FROM compute_nodes
               WHERE status != 'banned'
               ORDER BY tasks_completed DESC
               LIMIT ?""",
            (limit,),
        )
        rows = await cursor.fetchall()
        result = []
        for i, row in enumerate(rows, 1):
            d = _row_to_dict(row)
            d["rank"] = i
            result.append(d)
        return result

    # ------------------------------------------------------------------
    # Badges (Phase 5)
    # ------------------------------------------------------------------

    # Badge definitions: id → (name, check_fn description)
    BADGE_DEFS: dict[str, tuple[str, str]] = {
        "first_task": ("Premiere tache", "1 tache completee"),
        "centurion": ("Centurion", "100 taches completees"),
        "millionnaire": ("Millionnaire", "1 000 taches completees"),
        "pilier": ("Pilier", "10 000 taches completees"),
        "power_node": ("Power Node", "VRAM > 24 GB"),
    }

    async def calculate_badges(self, node_id: str) -> list[dict]:
        """Calculate and award badges for a node based on its stats.

        Returns list of all badges (awarded and not-yet-awarded).
        """
        node = await self.get_node(node_id)
        if not node:
            return []

        tasks = node.get("tasks_completed", 0)
        vram_mb = node.get("vram_mb", 0)

        earned: list[tuple[str, str]] = []
        if tasks >= 1:
            earned.append(("first_task", "Premiere tache"))
        if tasks >= 100:
            earned.append(("centurion", "Centurion"))
        if tasks >= 1000:
            earned.append(("millionnaire", "Millionnaire"))
        if tasks >= 10000:
            earned.append(("pilier", "Pilier"))
        if vram_mb > 24576:
            earned.append(("power_node", "Power Node"))

        # Check early_adopter: node created_at is in the first 10
        cursor = await self._conn.execute(
            "SELECT id FROM compute_nodes ORDER BY created_at ASC LIMIT 10",
        )
        early_ids = {r[0] for r in await cursor.fetchall()}
        if node_id in early_ids:
            earned.append(("early_adopter", "Early Adopter"))

        # Check always_on: 7+ days continuous uptime (604800 seconds)
        uptime = await self.get_node_uptime(node_id)
        if uptime.get("longest_session", 0) >= 604800:
            earned.append(("always_on", "24/7"))

        # Award new badges (UPSERT)
        for badge_id, badge_name in earned:
            try:
                await self._conn.execute(
                    """INSERT OR IGNORE INTO compute_badges
                       (id, node_id, badge_id, badge_name)
                       VALUES (?, ?, ?, ?)""",
                    (_new_id(), node_id, badge_id, badge_name),
                )
            except Exception:
                pass
        await self._conn.commit()

        # Return all badges for this node
        return await self.get_node_badges(node_id)

    async def get_node_badges(self, node_id: str) -> list[dict]:
        """Get all badges awarded to a node."""
        cursor = await self._conn.execute(
            "SELECT * FROM compute_badges WHERE node_id = ? ORDER BY awarded_at ASC",
            (node_id,),
        )
        rows = await cursor.fetchall()
        return [_row_to_dict(r) for r in rows]

    async def get_all_badges_summary(self) -> list[dict]:
        """Get badge counts across all nodes."""
        cursor = await self._conn.execute(
            """SELECT badge_id, badge_name, COUNT(*) as count
               FROM compute_badges
               GROUP BY badge_id, badge_name
               ORDER BY count DESC""",
        )
        rows = await cursor.fetchall()
        return [_row_to_dict(r) for r in rows]

    # ------------------------------------------------------------------
    # Uptime tracking (Phase 8)
    # ------------------------------------------------------------------

    async def log_connect(self, node_id: str) -> str:
        """Log a node connection event. Returns uptime_log id."""
        log_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO compute_uptime_log (id, node_id, connected_at)
               VALUES (?, ?, ?)""",
            (log_id, node_id, now),
        )
        await self._conn.commit()
        return log_id

    async def log_disconnect(self, node_id: str) -> None:
        """Close the most recent open uptime session for a node."""
        now = _now_iso()
        await self._conn.execute(
            """UPDATE compute_uptime_log
               SET disconnected_at = ?,
                   duration_seconds = CAST(
                       (julianday(?) - julianday(connected_at)) * 86400 AS INTEGER
                   )
               WHERE node_id = ? AND disconnected_at IS NULL""",
            (now, now, node_id),
        )
        await self._conn.commit()

    async def get_node_uptime(self, node_id: str) -> dict:
        """Get uptime stats for a node."""
        cursor = await self._conn.execute(
            """SELECT COUNT(*) as sessions,
                      COALESCE(SUM(duration_seconds), 0) as total_seconds,
                      COALESCE(MAX(duration_seconds), 0) as longest_session
               FROM compute_uptime_log WHERE node_id = ?""",
            (node_id,),
        )
        row = await cursor.fetchone()
        total = _row_to_dict(row) if row else {"sessions": 0, "total_seconds": 0, "longest_session": 0}

        # Current open session
        cursor = await self._conn.execute(
            """SELECT connected_at FROM compute_uptime_log
               WHERE node_id = ? AND disconnected_at IS NULL
               ORDER BY connected_at DESC LIMIT 1""",
            (node_id,),
        )
        current_row = await cursor.fetchone()
        current_seconds = 0
        if current_row:
            connected_at = current_row[0]
            cursor2 = await self._conn.execute(
                "SELECT CAST((julianday('now') - julianday(?)) * 86400 AS INTEGER)",
                (connected_at,),
            )
            r = await cursor2.fetchone()
            current_seconds = r[0] if r else 0

        return {
            "total_seconds": total["total_seconds"] + current_seconds,
            "sessions": total["sessions"],
            "longest_session": max(total["longest_session"], current_seconds),
            "current_session_seconds": current_seconds,
        }

    async def get_network_uptime(self) -> dict:
        """Get network-wide uptime stats."""
        # Sum closed sessions + estimate open sessions
        cursor = await self._conn.execute(
            """SELECT COALESCE(SUM(
                   CASE WHEN disconnected_at IS NOT NULL THEN duration_seconds
                        ELSE CAST((julianday('now') - julianday(connected_at)) * 86400 AS INTEGER)
                   END
               ), 0) as total_seconds
               FROM compute_uptime_log
               WHERE connected_at >= datetime('now', '-30 days')""",
        )
        row = await cursor.fetchone()
        total_seconds_30d = row[0] if row else 0

        cursor = await self._conn.execute(
            """SELECT COUNT(DISTINCT node_id) as count
               FROM compute_uptime_log
               WHERE duration_seconds >= 604800
                  OR (disconnected_at IS NULL
                      AND CAST((julianday('now') - julianday(connected_at)) * 86400 AS INTEGER) >= 604800)""",
        )
        row = await cursor.fetchone()
        nodes_7d = row[0] if row else 0

        nodes = await self.get_online_nodes()
        all_nodes = await self.list_nodes()

        return {
            "total_node_hours_30d": round(total_seconds_30d / 3600, 1),
            "nodes_with_7d_streak": nodes_7d,
            "nodes_online": len(nodes),
            "nodes_total": len(all_nodes),
            "uptime_pct": round(len(nodes) / max(len(all_nodes), 1) * 100, 1),
        }

    # ------------------------------------------------------------------
    # Contributor impact (Phase 8)
    # ------------------------------------------------------------------

    async def get_node_impact(self, node_id: str) -> dict:
        """Get detailed impact stats for a contributor node."""
        node = await self.get_node(node_id)
        if not node:
            return {}

        cursor = await self._conn.execute(
            """SELECT task_type, COUNT(*) as count
               FROM compute_tasks
               WHERE assigned_to = ? AND status = 'completed'
               GROUP BY task_type ORDER BY count DESC""",
            (node_id,),
        )
        by_type = [_row_to_dict(r) for r in await cursor.fetchall()]

        cursor = await self._conn.execute(
            """SELECT COALESCE(SUM(tokens_generated), 0) as tokens_week
               FROM compute_results
               WHERE node_id = ? AND created_at >= date('now', '-7 days')""",
            (node_id,),
        )
        row = await cursor.fetchone()
        tokens_week = row[0] if row else 0

        tasks_completed = node.get("tasks_completed", 0)
        cursor = await self._conn.execute(
            "SELECT COUNT(*) FROM compute_nodes WHERE tasks_completed < ? AND status != 'banned'",
            (tasks_completed,),
        )
        row = await cursor.fetchone()
        below = row[0] if row else 0

        cursor = await self._conn.execute(
            "SELECT COUNT(*) FROM compute_nodes WHERE status != 'banned'",
        )
        row = await cursor.fetchone()
        total_active = row[0] if row else 1

        percentile = round(below / max(total_active, 1) * 100)
        uptime = await self.get_node_uptime(node_id)

        return {
            "node_id": node_id,
            "name": node.get("name", ""),
            "gpu_model": node.get("gpu_model", ""),
            "vram_mb": node.get("vram_mb", 0),
            "tasks_completed": tasks_completed,
            "tasks_by_type": by_type,
            "tokens_this_week": tokens_week,
            "percentile": percentile,
            "uptime": uptime,
            "trust_score": node.get("trust_score", 50),
        }
