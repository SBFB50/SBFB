"""
NEXUS -- Async SQLite database layer.

Provides:
- init_db()  : creates all tables + indexes (idempotent)
- get_db()   : async context-manager yielding a connection
- Database   : full CRUD for every table
"""

from __future__ import annotations

import json
import uuid
from contextlib import asynccontextmanager
from datetime import datetime
from typing import Any, AsyncIterator, Dict, List, Optional

import aiosqlite

from nexus.config import settings

# ============================================================================
# SQL DDL
# ============================================================================

_CREATE_TABLES = """
CREATE TABLE IF NOT EXISTS cases (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    reference TEXT,
    description TEXT,
    status TEXT DEFAULT 'active',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS evidence (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    title TEXT NOT NULL,
    evidence_type TEXT NOT NULL,
    source TEXT,
    source_date DATETIME,
    ingestion_date DATETIME DEFAULT CURRENT_TIMESTAMP,
    reliability INTEGER DEFAULT 50,
    file_path TEXT,
    raw_text TEXT,
    summary TEXT,
    metadata TEXT,
    status TEXT DEFAULT 'pending',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS entities (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    name TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    aliases TEXT,
    description TEXT,
    first_seen DATETIME,
    metadata TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS entity_mentions (
    id TEXT PRIMARY KEY,
    entity_id TEXT NOT NULL REFERENCES entities(id),
    evidence_id TEXT NOT NULL REFERENCES evidence(id),
    context TEXT,
    confidence REAL DEFAULT 0.8,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS hypotheses (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT DEFAULT 'active',
    current_score REAL DEFAULT 50.0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS hypothesis_snapshots (
    id TEXT PRIMARY KEY,
    hypothesis_id TEXT NOT NULL REFERENCES hypotheses(id),
    score REAL NOT NULL,
    supporting TEXT,
    contradicting TEXT,
    reasoning TEXT,
    trigger TEXT,
    model_used TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS analysis_runs (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    run_type TEXT NOT NULL,
    trigger TEXT,
    status TEXT DEFAULT 'running',
    model_used TEXT,
    input_summary TEXT,
    output_summary TEXT,
    duration_sec REAL,
    tokens_used INTEGER,
    started_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME
);

CREATE TABLE IF NOT EXISTS monitoring_jobs (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    job_type TEXT NOT NULL,
    query TEXT NOT NULL,
    entity_id TEXT REFERENCES entities(id),
    interval_hours INTEGER DEFAULT 24,
    is_active BOOLEAN DEFAULT 1,
    last_run DATETIME,
    next_run DATETIME,
    results_count INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS monitoring_results (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES monitoring_jobs(id),
    case_id TEXT NOT NULL REFERENCES cases(id),
    url TEXT,
    title TEXT,
    snippet TEXT,
    source_engine TEXT,
    relevance_score REAL,
    is_new BOOLEAN DEFAULT 1,
    is_duplicate BOOLEAN DEFAULT 0,
    reviewed BOOLEAN DEFAULT 0,
    found_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS alerts (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    alert_type TEXT NOT NULL,
    severity TEXT DEFAULT 'info',
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    related_id TEXT,
    is_read BOOLEAN DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS reports (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    report_type TEXT NOT NULL,
    status TEXT DEFAULT 'generating',
    file_path TEXT,
    file_size INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME
);

CREATE TABLE IF NOT EXISTS locations (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    entity_id TEXT REFERENCES entities(id),
    name TEXT NOT NULL,
    address TEXT,
    lat REAL,
    lon REAL,
    location_type TEXT DEFAULT 'other',
    metadata TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS audit_log (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    target_type TEXT,
    target_id TEXT,
    summary TEXT NOT NULL,
    details TEXT,
    cycle_number INTEGER,
    entry_hash TEXT,
    previous_hash TEXT
);
"""

_CREATE_INDEXES = """
CREATE INDEX IF NOT EXISTS idx_evidence_case ON evidence(case_id);
CREATE INDEX IF NOT EXISTS idx_entities_case ON entities(case_id);
CREATE INDEX IF NOT EXISTS idx_hypotheses_case ON hypotheses(case_id);
CREATE INDEX IF NOT EXISTS idx_snapshots_hyp ON hypothesis_snapshots(hypothesis_id);
CREATE INDEX IF NOT EXISTS idx_monitoring_case ON monitoring_jobs(case_id);
CREATE INDEX IF NOT EXISTS idx_alerts_case_read ON alerts(case_id, is_read);
CREATE INDEX IF NOT EXISTS idx_analysis_case ON analysis_runs(case_id);
CREATE INDEX IF NOT EXISTS idx_reports_case ON reports(case_id);
CREATE INDEX IF NOT EXISTS idx_locations_case ON locations(case_id);
CREATE INDEX IF NOT EXISTS idx_locations_entity ON locations(entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_case ON audit_log(case_id);
CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_log(action);
"""


# ============================================================================
# Helpers
# ============================================================================

def _new_id() -> str:
    """Generate a new UUID4 string."""
    return str(uuid.uuid4())


def _now_iso() -> str:
    """Current UTC timestamp in ISO-8601."""
    return datetime.utcnow().isoformat()


def _json_dumps(obj: Any) -> Optional[str]:
    """Serialize to JSON string or return None."""
    if obj is None:
        return None
    return json.dumps(obj, ensure_ascii=False, default=str)


def _json_loads(raw: Optional[str]) -> Any:
    """Deserialize from JSON string or return None."""
    if raw is None:
        return None
    try:
        return json.loads(raw)
    except (json.JSONDecodeError, TypeError):
        return raw


def _row_to_dict(row: aiosqlite.Row) -> Dict[str, Any]:
    """Convert a sqlite3.Row to a plain dict."""
    return dict(row)


def _dict_with_json_fields(row_dict: Dict[str, Any], *fields: str) -> Dict[str, Any]:
    """Deserialize specific JSON-stored fields inside a row dict."""
    for f in fields:
        if f in row_dict:
            row_dict[f] = _json_loads(row_dict[f])
    return row_dict


# ============================================================================
# Connection management
# ============================================================================

async def init_db() -> None:
    """Create all tables and indexes. Safe to call multiple times."""
    settings.data_dir.mkdir(parents=True, exist_ok=True)
    async with aiosqlite.connect(str(settings.sqlite_path)) as db:
        await db.execute("PRAGMA journal_mode=WAL")
        await db.execute("PRAGMA foreign_keys=ON")
        await db.executescript(_CREATE_TABLES)
        await db.executescript(_CREATE_INDEXES)
        await db.commit()


@asynccontextmanager
async def get_db() -> AsyncIterator[aiosqlite.Connection]:
    """Yield an aiosqlite connection with row_factory and FK enforcement."""
    db = await aiosqlite.connect(str(settings.sqlite_path))
    db.row_factory = aiosqlite.Row
    await db.execute("PRAGMA foreign_keys=ON")
    try:
        yield db
    finally:
        await db.close()


# ============================================================================
# Database CRUD class
# ============================================================================

class Database:
    """Full CRUD operations for every NEXUS table.

    Usage::

        async with get_db() as conn:
            db = Database(conn)
            case = await db.create_case(name="Doe", description="Cold case")
    """

    def __init__(self, conn: aiosqlite.Connection) -> None:
        self._conn = conn

    # ------------------------------------------------------------------
    # Cases
    # ------------------------------------------------------------------

    async def create_case(
        self,
        *,
        name: str,
        reference: Optional[str] = None,
        description: Optional[str] = None,
        status: str = "active",
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO cases (id, name, reference, description, status, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?)""",
            (row_id, name, reference, description, status, now, now),
        )
        await self._conn.commit()
        return await self.get_case(row_id)  # type: ignore[return-value]

    async def get_case(self, case_id: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute("SELECT * FROM cases WHERE id = ?", (case_id,))
        row = await cursor.fetchone()
        return _row_to_dict(row) if row else None

    async def list_cases(self, *, status: Optional[str] = None) -> List[Dict[str, Any]]:
        if status:
            cursor = await self._conn.execute(
                "SELECT * FROM cases WHERE status = ? ORDER BY updated_at DESC", (status,)
            )
        else:
            cursor = await self._conn.execute("SELECT * FROM cases ORDER BY updated_at DESC")
        return [_row_to_dict(r) for r in await cursor.fetchall()]

    async def update_case(self, case_id: str, **fields: Any) -> Optional[Dict[str, Any]]:
        if not fields:
            return await self.get_case(case_id)
        fields["updated_at"] = _now_iso()
        set_clause = ", ".join(f"{k} = ?" for k in fields)
        values = list(fields.values()) + [case_id]
        await self._conn.execute(f"UPDATE cases SET {set_clause} WHERE id = ?", values)
        await self._conn.commit()
        return await self.get_case(case_id)

    async def delete_case(self, case_id: str) -> bool:
        """Delete a case and all its dependent rows (cascade)."""
        # Delete children in dependency order (leaves first)
        await self._conn.execute(
            "DELETE FROM audit_log WHERE case_id = ?", (case_id,)
        )
        await self._conn.execute(
            "DELETE FROM locations WHERE case_id = ?", (case_id,)
        )
        await self._conn.execute(
            "DELETE FROM monitoring_results WHERE case_id = ?", (case_id,)
        )
        await self._conn.execute(
            "DELETE FROM monitoring_jobs WHERE case_id = ?", (case_id,)
        )
        await self._conn.execute(
            "DELETE FROM alerts WHERE case_id = ?", (case_id,)
        )
        await self._conn.execute(
            "DELETE FROM analysis_runs WHERE case_id = ?", (case_id,)
        )
        await self._conn.execute(
            """DELETE FROM hypothesis_snapshots WHERE hypothesis_id IN
               (SELECT id FROM hypotheses WHERE case_id = ?)""",
            (case_id,),
        )
        await self._conn.execute(
            "DELETE FROM hypotheses WHERE case_id = ?", (case_id,)
        )
        await self._conn.execute(
            """DELETE FROM entity_mentions WHERE entity_id IN
               (SELECT id FROM entities WHERE case_id = ?)""",
            (case_id,),
        )
        await self._conn.execute(
            "DELETE FROM entities WHERE case_id = ?", (case_id,)
        )
        await self._conn.execute(
            "DELETE FROM evidence WHERE case_id = ?", (case_id,)
        )
        cursor = await self._conn.execute("DELETE FROM cases WHERE id = ?", (case_id,))
        await self._conn.commit()
        return cursor.rowcount > 0

    # ------------------------------------------------------------------
    # Evidence
    # ------------------------------------------------------------------

    async def create_evidence(
        self,
        *,
        case_id: str,
        title: str,
        evidence_type: str,
        source: Optional[str] = None,
        source_date: Optional[str] = None,
        reliability: int = 50,
        file_path: Optional[str] = None,
        raw_text: Optional[str] = None,
        summary: Optional[str] = None,
        metadata: Any = None,
        status: str = "pending",
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO evidence
               (id, case_id, title, evidence_type, source, source_date,
                ingestion_date, reliability, file_path, raw_text, summary,
                metadata, status, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, case_id, title, evidence_type, source, source_date,
                now, reliability, file_path, raw_text, summary,
                _json_dumps(metadata), status, now,
            ),
        )
        await self._conn.commit()
        return await self.get_evidence(row_id)  # type: ignore[return-value]

    async def get_evidence(self, evidence_id: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute("SELECT * FROM evidence WHERE id = ?", (evidence_id,))
        row = await cursor.fetchone()
        if not row:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def list_evidence_by_case(
        self,
        case_id: str,
        *,
        status: Optional[str] = None,
    ) -> List[Dict[str, Any]]:
        if status:
            cursor = await self._conn.execute(
                "SELECT * FROM evidence WHERE case_id = ? AND status = ? ORDER BY created_at DESC",
                (case_id, status),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT * FROM evidence WHERE case_id = ? ORDER BY created_at DESC", (case_id,)
            )
        return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in await cursor.fetchall()]

    async def update_evidence(self, evidence_id: str, **fields: Any) -> Optional[Dict[str, Any]]:
        if not fields:
            return await self.get_evidence(evidence_id)
        if "metadata" in fields:
            fields["metadata"] = _json_dumps(fields["metadata"])
        set_clause = ", ".join(f"{k} = ?" for k in fields)
        values = list(fields.values()) + [evidence_id]
        await self._conn.execute(f"UPDATE evidence SET {set_clause} WHERE id = ?", values)
        await self._conn.commit()
        return await self.get_evidence(evidence_id)

    async def delete_evidence(self, evidence_id: str) -> bool:
        cursor = await self._conn.execute("DELETE FROM evidence WHERE id = ?", (evidence_id,))
        await self._conn.commit()
        return cursor.rowcount > 0

    # ------------------------------------------------------------------
    # Entities
    # ------------------------------------------------------------------

    async def create_entity(
        self,
        *,
        case_id: str,
        name: str,
        entity_type: str,
        aliases: Optional[list] = None,
        description: Optional[str] = None,
        first_seen: Optional[str] = None,
        metadata: Any = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO entities
               (id, case_id, name, entity_type, aliases, description, first_seen, metadata, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, case_id, name, entity_type,
                _json_dumps(aliases), description, first_seen,
                _json_dumps(metadata), now,
            ),
        )
        await self._conn.commit()
        return await self.get_entity(row_id)  # type: ignore[return-value]

    async def get_entity(self, entity_id: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute("SELECT * FROM entities WHERE id = ?", (entity_id,))
        row = await cursor.fetchone()
        if not row:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "aliases", "metadata")

    async def list_entities_by_case(
        self,
        case_id: str,
        *,
        entity_type: Optional[str] = None,
    ) -> List[Dict[str, Any]]:
        if entity_type:
            cursor = await self._conn.execute(
                "SELECT * FROM entities WHERE case_id = ? AND entity_type = ? ORDER BY name",
                (case_id, entity_type),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT * FROM entities WHERE case_id = ? ORDER BY name", (case_id,)
            )
        return [_dict_with_json_fields(_row_to_dict(r), "aliases", "metadata") for r in await cursor.fetchall()]

    async def update_entity(self, entity_id: str, **fields: Any) -> Optional[Dict[str, Any]]:
        if not fields:
            return await self.get_entity(entity_id)
        if "aliases" in fields:
            fields["aliases"] = _json_dumps(fields["aliases"])
        if "metadata" in fields:
            fields["metadata"] = _json_dumps(fields["metadata"])
        set_clause = ", ".join(f"{k} = ?" for k in fields)
        values = list(fields.values()) + [entity_id]
        await self._conn.execute(f"UPDATE entities SET {set_clause} WHERE id = ?", values)
        await self._conn.commit()
        return await self.get_entity(entity_id)

    # ------------------------------------------------------------------
    # Entity Mentions
    # ------------------------------------------------------------------

    async def create_entity_mention(
        self,
        *,
        entity_id: str,
        evidence_id: str,
        context: Optional[str] = None,
        confidence: float = 0.8,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO entity_mentions (id, entity_id, evidence_id, context, confidence, created_at)
               VALUES (?, ?, ?, ?, ?, ?)""",
            (row_id, entity_id, evidence_id, context, confidence, now),
        )
        await self._conn.commit()
        cursor = await self._conn.execute("SELECT * FROM entity_mentions WHERE id = ?", (row_id,))
        row = await cursor.fetchone()
        return _row_to_dict(row)  # type: ignore[arg-type]

    async def list_mentions_by_entity(self, entity_id: str) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM entity_mentions WHERE entity_id = ? ORDER BY created_at DESC",
            (entity_id,),
        )
        return [_row_to_dict(r) for r in await cursor.fetchall()]

    async def list_mentions_by_evidence(self, evidence_id: str) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM entity_mentions WHERE evidence_id = ? ORDER BY confidence DESC",
            (evidence_id,),
        )
        return [_row_to_dict(r) for r in await cursor.fetchall()]

    # ------------------------------------------------------------------
    # Hypotheses
    # ------------------------------------------------------------------

    async def create_hypothesis(
        self,
        *,
        case_id: str,
        title: str,
        description: str,
        status: str = "active",
        current_score: float = 50.0,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO hypotheses
               (id, case_id, title, description, status, current_score, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
            (row_id, case_id, title, description, status, current_score, now, now),
        )
        await self._conn.commit()
        return await self.get_hypothesis(row_id)  # type: ignore[return-value]

    async def get_hypothesis(self, hypothesis_id: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute("SELECT * FROM hypotheses WHERE id = ?", (hypothesis_id,))
        row = await cursor.fetchone()
        return _row_to_dict(row) if row else None

    async def list_hypotheses_by_case(
        self,
        case_id: str,
        *,
        status: Optional[str] = None,
    ) -> List[Dict[str, Any]]:
        if status:
            cursor = await self._conn.execute(
                "SELECT * FROM hypotheses WHERE case_id = ? AND status = ? ORDER BY current_score DESC",
                (case_id, status),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT * FROM hypotheses WHERE case_id = ? ORDER BY current_score DESC", (case_id,)
            )
        return [_row_to_dict(r) for r in await cursor.fetchall()]

    async def update_hypothesis(self, hypothesis_id: str, **fields: Any) -> Optional[Dict[str, Any]]:
        if not fields:
            return await self.get_hypothesis(hypothesis_id)
        fields["updated_at"] = _now_iso()
        set_clause = ", ".join(f"{k} = ?" for k in fields)
        values = list(fields.values()) + [hypothesis_id]
        await self._conn.execute(f"UPDATE hypotheses SET {set_clause} WHERE id = ?", values)
        await self._conn.commit()
        return await self.get_hypothesis(hypothesis_id)

    # ------------------------------------------------------------------
    # Hypothesis Snapshots
    # ------------------------------------------------------------------

    async def create_hypothesis_snapshot(
        self,
        *,
        hypothesis_id: str,
        score: float,
        supporting: Any = None,
        contradicting: Any = None,
        reasoning: Optional[str] = None,
        trigger: Optional[str] = None,
        model_used: Optional[str] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO hypothesis_snapshots
               (id, hypothesis_id, score, supporting, contradicting, reasoning, trigger, model_used, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, hypothesis_id, score,
                _json_dumps(supporting), _json_dumps(contradicting),
                reasoning, trigger, model_used, now,
            ),
        )
        await self._conn.commit()
        cursor = await self._conn.execute(
            "SELECT * FROM hypothesis_snapshots WHERE id = ?", (row_id,)
        )
        row = await cursor.fetchone()
        return _dict_with_json_fields(_row_to_dict(row), "supporting", "contradicting")  # type: ignore[arg-type]

    async def list_snapshots_by_hypothesis(
        self,
        hypothesis_id: str,
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM hypothesis_snapshots WHERE hypothesis_id = ? ORDER BY created_at DESC",
            (hypothesis_id,),
        )
        return [
            _dict_with_json_fields(_row_to_dict(r), "supporting", "contradicting")
            for r in await cursor.fetchall()
        ]

    # ------------------------------------------------------------------
    # Analysis Runs
    # ------------------------------------------------------------------

    async def create_analysis_run(
        self,
        *,
        case_id: str,
        run_type: str,
        trigger: Optional[str] = None,
        status: str = "running",
        model_used: Optional[str] = None,
        input_summary: Optional[str] = None,
        output_summary: Optional[str] = None,
        duration_sec: Optional[float] = None,
        completed_at: Optional[str] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO analysis_runs
               (id, case_id, run_type, trigger, status, model_used,
                input_summary, output_summary, duration_sec, started_at, completed_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, case_id, run_type, trigger, status, model_used,
                input_summary, output_summary, duration_sec, now,
                completed_at or (now if status == "completed" else None),
            ),
        )
        await self._conn.commit()
        return await self.get_analysis_run(row_id)  # type: ignore[return-value]

    async def get_analysis_run(self, run_id: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute("SELECT * FROM analysis_runs WHERE id = ?", (run_id,))
        row = await cursor.fetchone()
        return _row_to_dict(row) if row else None

    async def update_analysis_run(self, run_id: str, **fields: Any) -> Optional[Dict[str, Any]]:
        if not fields:
            return await self.get_analysis_run(run_id)
        set_clause = ", ".join(f"{k} = ?" for k in fields)
        values = list(fields.values()) + [run_id]
        await self._conn.execute(f"UPDATE analysis_runs SET {set_clause} WHERE id = ?", values)
        await self._conn.commit()
        return await self.get_analysis_run(run_id)

    async def list_runs_by_case(
        self,
        case_id: str,
        *,
        status: Optional[str] = None,
        limit: int = 50,
    ) -> List[Dict[str, Any]]:
        if status:
            cursor = await self._conn.execute(
                "SELECT * FROM analysis_runs WHERE case_id = ? AND status = ? ORDER BY started_at DESC LIMIT ?",
                (case_id, status, limit),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT * FROM analysis_runs WHERE case_id = ? ORDER BY started_at DESC LIMIT ?",
                (case_id, limit),
            )
        return [_row_to_dict(r) for r in await cursor.fetchall()]

    # ------------------------------------------------------------------
    # Monitoring Jobs
    # ------------------------------------------------------------------

    async def create_monitoring_job(
        self,
        *,
        case_id: str,
        job_type: str,
        query: str,
        entity_id: Optional[str] = None,
        interval_hours: int = 24,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO monitoring_jobs
               (id, case_id, job_type, query, entity_id, interval_hours, is_active, results_count, created_at)
               VALUES (?, ?, ?, ?, ?, ?, 1, 0, ?)""",
            (row_id, case_id, job_type, query, entity_id, interval_hours, now),
        )
        await self._conn.commit()
        return await self._get_monitoring_job(row_id)  # type: ignore[return-value]

    async def _get_monitoring_job(self, job_id: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute("SELECT * FROM monitoring_jobs WHERE id = ?", (job_id,))
        row = await cursor.fetchone()
        return _row_to_dict(row) if row else None

    async def list_jobs_by_case(
        self,
        case_id: str,
        *,
        active_only: bool = False,
    ) -> List[Dict[str, Any]]:
        if active_only:
            cursor = await self._conn.execute(
                "SELECT * FROM monitoring_jobs WHERE case_id = ? AND is_active = 1 ORDER BY created_at DESC",
                (case_id,),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT * FROM monitoring_jobs WHERE case_id = ? ORDER BY created_at DESC",
                (case_id,),
            )
        return [_row_to_dict(r) for r in await cursor.fetchall()]

    async def update_job(self, job_id: str, **fields: Any) -> Optional[Dict[str, Any]]:
        if not fields:
            return await self._get_monitoring_job(job_id)
        set_clause = ", ".join(f"{k} = ?" for k in fields)
        values = list(fields.values()) + [job_id]
        await self._conn.execute(f"UPDATE monitoring_jobs SET {set_clause} WHERE id = ?", values)
        await self._conn.commit()
        return await self._get_monitoring_job(job_id)

    async def delete_job(self, job_id: str) -> bool:
        cursor = await self._conn.execute("DELETE FROM monitoring_jobs WHERE id = ?", (job_id,))
        await self._conn.commit()
        return cursor.rowcount > 0

    # ------------------------------------------------------------------
    # Monitoring Results
    # ------------------------------------------------------------------

    async def create_monitoring_result(
        self,
        *,
        job_id: str,
        case_id: str,
        url: Optional[str] = None,
        title: Optional[str] = None,
        snippet: Optional[str] = None,
        source_engine: Optional[str] = None,
        relevance_score: Optional[float] = None,
        is_new: bool = True,
        is_duplicate: bool = False,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO monitoring_results
               (id, job_id, case_id, url, title, snippet, source_engine,
                relevance_score, is_new, is_duplicate, reviewed, found_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?)""",
            (
                row_id, job_id, case_id, url, title, snippet,
                source_engine, relevance_score, int(is_new), int(is_duplicate), now,
            ),
        )
        await self._conn.commit()
        # Increment results_count on the parent job
        await self._conn.execute(
            "UPDATE monitoring_jobs SET results_count = results_count + 1 WHERE id = ?",
            (job_id,),
        )
        await self._conn.commit()
        cursor = await self._conn.execute(
            "SELECT * FROM monitoring_results WHERE id = ?", (row_id,)
        )
        row = await cursor.fetchone()
        return _row_to_dict(row)  # type: ignore[arg-type]

    async def get_monitoring_result(self, result_id: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM monitoring_results WHERE id = ?", (result_id,)
        )
        row = await cursor.fetchone()
        return _row_to_dict(row) if row else None

    async def list_results_by_job(
        self,
        job_id: str,
        *,
        unreviewed_only: bool = False,
        limit: int = 100,
    ) -> List[Dict[str, Any]]:
        if unreviewed_only:
            cursor = await self._conn.execute(
                "SELECT * FROM monitoring_results WHERE job_id = ? AND reviewed = 0 ORDER BY found_at DESC LIMIT ?",
                (job_id, limit),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT * FROM monitoring_results WHERE job_id = ? ORDER BY found_at DESC LIMIT ?",
                (job_id, limit),
            )
        return [_row_to_dict(r) for r in await cursor.fetchall()]

    async def list_results_by_case(
        self,
        case_id: str,
        *,
        limit: int = 200,
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM monitoring_results WHERE case_id = ? ORDER BY found_at DESC LIMIT ?",
            (case_id, limit),
        )
        return [_row_to_dict(r) for r in await cursor.fetchall()]

    async def update_monitoring_result(self, result_id: str, **fields: Any) -> Optional[Dict[str, Any]]:
        if not fields:
            return await self.get_monitoring_result(result_id)
        set_clause = ", ".join(f"{k} = ?" for k in fields)
        values = list(fields.values()) + [result_id]
        await self._conn.execute(
            f"UPDATE monitoring_results SET {set_clause} WHERE id = ?", values
        )
        await self._conn.commit()
        return await self.get_monitoring_result(result_id)

    # ------------------------------------------------------------------
    # Alerts
    # ------------------------------------------------------------------

    async def create_alert(
        self,
        *,
        case_id: str,
        alert_type: str,
        severity: str = "info",
        title: str,
        message: str,
        related_id: Optional[str] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO alerts
               (id, case_id, alert_type, severity, title, message, related_id, is_read, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?)""",
            (row_id, case_id, alert_type, severity, title, message, related_id, now),
        )
        await self._conn.commit()
        cursor = await self._conn.execute("SELECT * FROM alerts WHERE id = ?", (row_id,))
        row = await cursor.fetchone()
        return _row_to_dict(row)  # type: ignore[arg-type]

    async def list_alerts_by_case(
        self,
        case_id: str,
        *,
        unread_only: bool = False,
        severity: Optional[str] = None,
        limit: int = 100,
    ) -> List[Dict[str, Any]]:
        conditions = ["case_id = ?"]
        params: list[Any] = [case_id]
        if unread_only:
            conditions.append("is_read = 0")
        if severity:
            conditions.append("severity = ?")
            params.append(severity)
        where = " AND ".join(conditions)
        params.append(limit)
        cursor = await self._conn.execute(
            f"SELECT * FROM alerts WHERE {where} ORDER BY created_at DESC LIMIT ?",
            params,
        )
        return [_row_to_dict(r) for r in await cursor.fetchall()]

    async def mark_alert_read(self, alert_id: str) -> bool:
        cursor = await self._conn.execute(
            "UPDATE alerts SET is_read = 1 WHERE id = ?", (alert_id,)
        )
        await self._conn.commit()
        return cursor.rowcount > 0

    async def count_unread_alerts(self, case_id: str) -> int:
        cursor = await self._conn.execute(
            "SELECT COUNT(*) FROM alerts WHERE case_id = ? AND is_read = 0",
            (case_id,),
        )
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # Reports
    # ------------------------------------------------------------------

    async def create_report(
        self,
        *,
        case_id: str,
        report_type: str,
        status: str = "generating",
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO reports
               (id, case_id, report_type, status, created_at)
               VALUES (?, ?, ?, ?, ?)""",
            (row_id, case_id, report_type, status, now),
        )
        await self._conn.commit()
        return await self.get_report(row_id)  # type: ignore[return-value]

    async def get_report(self, report_id: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM reports WHERE id = ?", (report_id,)
        )
        row = await cursor.fetchone()
        return _row_to_dict(row) if row else None

    async def update_report(self, report_id: str, **fields: Any) -> Optional[Dict[str, Any]]:
        if not fields:
            return await self.get_report(report_id)
        set_clause = ", ".join(f"{k} = ?" for k in fields)
        values = list(fields.values()) + [report_id]
        await self._conn.execute(
            f"UPDATE reports SET {set_clause} WHERE id = ?", values
        )
        await self._conn.commit()
        return await self.get_report(report_id)

    async def list_reports_by_case(self, case_id: str) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM reports WHERE case_id = ? ORDER BY created_at DESC",
            (case_id,),
        )
        return [_row_to_dict(r) for r in await cursor.fetchall()]

    # ------------------------------------------------------------------
    # Locations (geo)
    # ------------------------------------------------------------------

    async def create_location(
        self,
        *,
        case_id: str,
        name: str,
        entity_id: Optional[str] = None,
        address: Optional[str] = None,
        lat: Optional[float] = None,
        lon: Optional[float] = None,
        location_type: str = "other",
        metadata: Any = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO locations
               (id, case_id, entity_id, name, address, lat, lon, location_type, metadata, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, case_id, entity_id, name, address,
                lat, lon, location_type, _json_dumps(metadata), now,
            ),
        )
        await self._conn.commit()
        return await self.get_location(row_id)  # type: ignore[return-value]

    async def get_location(self, location_id: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM locations WHERE id = ?", (location_id,)
        )
        row = await cursor.fetchone()
        if not row:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def list_locations_by_case(self, case_id: str) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM locations WHERE case_id = ? ORDER BY name",
            (case_id,),
        )
        return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in await cursor.fetchall()]

    async def get_location_by_entity(self, entity_id: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM locations WHERE entity_id = ?", (entity_id,)
        )
        row = await cursor.fetchone()
        if not row:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def update_location(self, location_id: str, **fields: Any) -> Optional[Dict[str, Any]]:
        if not fields:
            return await self.get_location(location_id)
        if "metadata" in fields:
            fields["metadata"] = _json_dumps(fields["metadata"])
        set_clause = ", ".join(f"{k} = ?" for k in fields)
        values = list(fields.values()) + [location_id]
        await self._conn.execute(
            f"UPDATE locations SET {set_clause} WHERE id = ?", values
        )
        await self._conn.commit()
        return await self.get_location(location_id)

    async def delete_locations_by_case(self, case_id: str) -> int:
        cursor = await self._conn.execute(
            "DELETE FROM locations WHERE case_id = ?", (case_id,)
        )
        await self._conn.commit()
        return cursor.rowcount

    # ------------------------------------------------------------------
    # Audit Log
    # ------------------------------------------------------------------

    async def create_audit_entry(
        self,
        *,
        case_id: str,
        actor: str,
        action: str,
        target_type: Optional[str] = None,
        target_id: Optional[str] = None,
        summary: str,
        details: Optional[str] = None,
        cycle_number: Optional[int] = None,
        entry_hash: Optional[str] = None,
        previous_hash: Optional[str] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO audit_log
               (id, case_id, timestamp, actor, action, target_type,
                target_id, summary, details, cycle_number,
                entry_hash, previous_hash)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, case_id, now, actor, action,
                target_type, target_id, summary, details, cycle_number,
                entry_hash, previous_hash,
            ),
        )
        await self._conn.commit()
        return await self.get_audit_entry(row_id)  # type: ignore[return-value]

    async def get_audit_entry(self, audit_id: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM audit_log WHERE id = ?", (audit_id,)
        )
        row = await cursor.fetchone()
        if not row:
            return None
        d = _row_to_dict(row)
        d["details"] = _json_loads(d.get("details"))
        return d

    async def list_audit_log(
        self,
        case_id: str,
        *,
        action: Optional[str] = None,
        actor: Optional[str] = None,
        limit: int = 100,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        conditions = ["case_id = ?"]
        params: list[Any] = [case_id]
        if action:
            conditions.append("action = ?")
            params.append(action)
        if actor:
            conditions.append("actor = ?")
            params.append(actor)
        where = " AND ".join(conditions)
        params.extend([limit, offset])
        cursor = await self._conn.execute(
            f"SELECT * FROM audit_log WHERE {where} ORDER BY timestamp DESC LIMIT ? OFFSET ?",
            params,
        )
        rows = await cursor.fetchall()
        result = []
        for r in rows:
            d = _row_to_dict(r)
            d["details"] = _json_loads(d.get("details"))
            result.append(d)
        return result

    async def count_audit_entries(
        self,
        case_id: str,
        *,
        action: Optional[str] = None,
    ) -> int:
        if action:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM audit_log WHERE case_id = ? AND action = ?",
                (case_id, action),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM audit_log WHERE case_id = ?",
                (case_id,),
            )
        row = await cursor.fetchone()
        return row[0] if row else 0

    async def get_investigation_timeline(
        self,
        case_id: str,
    ) -> List[Dict[str, Any]]:
        """Return the full audit log sorted chronologically (oldest first)."""
        cursor = await self._conn.execute(
            "SELECT * FROM audit_log WHERE case_id = ? ORDER BY timestamp ASC",
            (case_id,),
        )
        rows = await cursor.fetchall()
        result = []
        for r in rows:
            d = _row_to_dict(r)
            d["details"] = _json_loads(d.get("details"))
            result.append(d)
        return result
