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
from datetime import datetime, timezone
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

CREATE TABLE IF NOT EXISTS summary_clusters (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    title TEXT,
    summary TEXT,
    evidence_ids TEXT,
    embedding_centroid TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS suspects (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    entity_id TEXT NOT NULL REFERENCES entities(id),
    suspicion_score REAL DEFAULT 0.0,
    graph_score REAL DEFAULT 0.0,
    evidence_score REAL DEFAULT 0.0,
    contradiction_score REAL DEFAULT 0.0,
    profile_score REAL DEFAULT 0.0,
    hypothesis_score REAL DEFAULT 0.0,
    known_motive TEXT,
    alibi_status TEXT DEFAULT 'unknown',
    criminal_record TEXT,
    relationship_to_victim TEXT,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(case_id, entity_id)
);

CREATE TABLE IF NOT EXISTS suspect_snapshots (
    id TEXT PRIMARY KEY,
    suspect_id TEXT NOT NULL REFERENCES suspects(id),
    suspicion_score REAL NOT NULL,
    graph_score REAL,
    evidence_score REAL,
    contradiction_score REAL,
    profile_score REAL,
    hypothesis_score REAL,
    trigger TEXT,
    reasoning TEXT,
    model_used TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS case_summaries (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id) UNIQUE,
    summary TEXT,
    cluster_ids TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS event_log (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    case_id TEXT NOT NULL,
    payload TEXT,
    source_worker TEXT,
    parent_event_id TEXT,
    status TEXT DEFAULT 'pending',
    processed_by TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    processed_at DATETIME
);

CREATE TABLE IF NOT EXISTS contradictions (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    evidence_1_id TEXT REFERENCES evidence(id),
    evidence_2_id TEXT REFERENCES evidence(id),
    evidence_1_title TEXT,
    evidence_2_title TEXT,
    contradiction_type TEXT DEFAULT 'factual',
    severity TEXT DEFAULT 'medium',
    description TEXT NOT NULL,
    likely_correct TEXT,
    reasoning TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(case_id, evidence_1_id, evidence_2_id, contradiction_type)
);

CREATE TABLE IF NOT EXISTS wiki_pages (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    page_path TEXT NOT NULL,
    page_type TEXT NOT NULL,
    title TEXT NOT NULL,
    content_hash TEXT,
    last_compiled DATETIME,
    source_ids TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(case_id, page_path)
);

CREATE TABLE IF NOT EXISTS investigation_memory (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL REFERENCES cases(id),
    insight_type TEXT NOT NULL,
    source_event_type TEXT NOT NULL,
    importance REAL DEFAULT 0.5,
    confidence REAL DEFAULT 0.7,
    summary TEXT NOT NULL,
    full_context TEXT,
    related_entities TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
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
CREATE INDEX IF NOT EXISTS idx_clusters_case ON summary_clusters(case_id);
CREATE INDEX IF NOT EXISTS idx_suspects_case ON suspects(case_id);
CREATE INDEX IF NOT EXISTS idx_suspect_snapshots ON suspect_snapshots(suspect_id);

-- Composite indexes for common filtered queries at scale
CREATE INDEX IF NOT EXISTS idx_evidence_case_type ON evidence(case_id, evidence_type);
CREATE INDEX IF NOT EXISTS idx_evidence_case_status ON evidence(case_id, status);
CREATE INDEX IF NOT EXISTS idx_entities_case_type ON entities(case_id, entity_type);
CREATE INDEX IF NOT EXISTS idx_mentions_evidence ON entity_mentions(evidence_id);
CREATE INDEX IF NOT EXISTS idx_mentions_entity ON entity_mentions(entity_id);
CREATE INDEX IF NOT EXISTS idx_monitoring_results_job ON monitoring_results(job_id);
CREATE INDEX IF NOT EXISTS idx_monitoring_results_case ON monitoring_results(case_id);

-- Event bus indexes
CREATE INDEX IF NOT EXISTS idx_event_log_status ON event_log(status);
CREATE INDEX IF NOT EXISTS idx_event_log_type ON event_log(event_type);
CREATE INDEX IF NOT EXISTS idx_event_log_case ON event_log(case_id);

-- Contradictions indexes
CREATE INDEX IF NOT EXISTS idx_contradictions_case ON contradictions(case_id);
CREATE INDEX IF NOT EXISTS idx_contradictions_evidence ON contradictions(evidence_1_id, evidence_2_id);

-- Investigation memory indexes
CREATE INDEX IF NOT EXISTS idx_memory_case ON investigation_memory(case_id);

-- Wiki indexes
CREATE INDEX IF NOT EXISTS idx_wiki_pages_case ON wiki_pages(case_id);
CREATE INDEX IF NOT EXISTS idx_wiki_pages_path ON wiki_pages(case_id, page_path);
"""

_CREATE_FTS = """
-- FTS5 virtual table for fast full-text search across evidence
CREATE VIRTUAL TABLE IF NOT EXISTS evidence_fts USING fts5(
    title, raw_text, summary, source,
    content=evidence, content_rowid=rowid
);

-- Triggers to keep FTS index in sync with the evidence table
CREATE TRIGGER IF NOT EXISTS evidence_fts_insert AFTER INSERT ON evidence BEGIN
    INSERT INTO evidence_fts(rowid, title, raw_text, summary, source)
    VALUES (new.rowid, new.title, new.raw_text, new.summary, new.source);
END;

CREATE TRIGGER IF NOT EXISTS evidence_fts_update AFTER UPDATE ON evidence BEGIN
    INSERT INTO evidence_fts(evidence_fts, rowid, title, raw_text, summary, source)
    VALUES ('delete', old.rowid, old.title, old.raw_text, old.summary, old.source);
    INSERT INTO evidence_fts(rowid, title, raw_text, summary, source)
    VALUES (new.rowid, new.title, new.raw_text, new.summary, new.source);
END;

CREATE TRIGGER IF NOT EXISTS evidence_fts_delete AFTER DELETE ON evidence BEGIN
    INSERT INTO evidence_fts(evidence_fts, rowid, title, raw_text, summary, source)
    VALUES ('delete', old.rowid, old.title, old.raw_text, old.summary, old.source);
END;
"""


# ============================================================================
# Helpers
# ============================================================================

def _new_id() -> str:
    """Generate a new UUID4 string."""
    return str(uuid.uuid4())


def _now_iso() -> str:
    """Current UTC timestamp in ISO-8601."""
    return datetime.now(timezone.utc).isoformat()


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
        # -- Performance PRAGMAs (must come before DDL) --
        await db.execute("PRAGMA journal_mode=WAL")
        await db.execute("PRAGMA synchronous=NORMAL")
        await db.execute("PRAGMA cache_size=-64000")  # 64 MB cache
        await db.execute("PRAGMA busy_timeout=5000")  # 5 s wait on lock
        await db.execute("PRAGMA foreign_keys=ON")
        await db.executescript(_CREATE_TABLES)
        await db.executescript(_CREATE_INDEXES)
        # -- FTS5 virtual table + sync triggers --
        await db.executescript(_CREATE_FTS)
        await db.commit()


@asynccontextmanager
async def get_db() -> AsyncIterator[aiosqlite.Connection]:
    """Yield an aiosqlite connection with row_factory and FK enforcement."""
    db = await aiosqlite.connect(str(settings.sqlite_path))
    db.row_factory = aiosqlite.Row
    await db.execute("PRAGMA journal_mode=WAL")
    await db.execute("PRAGMA synchronous=NORMAL")
    await db.execute("PRAGMA cache_size=-64000")
    await db.execute("PRAGMA busy_timeout=5000")
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
            "DELETE FROM case_summaries WHERE case_id = ?", (case_id,)
        )
        await self._conn.execute(
            "DELETE FROM summary_clusters WHERE case_id = ?", (case_id,)
        )
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
            """DELETE FROM suspect_snapshots WHERE suspect_id IN
               (SELECT id FROM suspects WHERE case_id = ?)""",
            (case_id,),
        )
        await self._conn.execute(
            "DELETE FROM suspects WHERE case_id = ?", (case_id,)
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
            "DELETE FROM contradictions WHERE case_id = ?", (case_id,)
        )
        await self._conn.execute(
            "DELETE FROM event_log WHERE case_id = ?", (case_id,)
        )
        try:
            await self._conn.execute(
                "DELETE FROM investigation_memory WHERE case_id = ?", (case_id,)
            )
        except Exception:
            pass  # table may not exist on older DBs
        try:
            await self._conn.execute(
                "DELETE FROM wiki_pages WHERE case_id = ?", (case_id,)
            )
        except Exception:
            pass  # table may not exist on older DBs
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
        limit: int = 1000,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        if status:
            cursor = await self._conn.execute(
                "SELECT * FROM evidence WHERE case_id = ? AND status = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
                (case_id, status, limit, offset),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT * FROM evidence WHERE case_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
                (case_id, limit, offset),
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
        limit: int = 1000,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        if entity_type:
            cursor = await self._conn.execute(
                "SELECT * FROM entities WHERE case_id = ? AND entity_type = ? ORDER BY name LIMIT ? OFFSET ?",
                (case_id, entity_type, limit, offset),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT * FROM entities WHERE case_id = ? ORDER BY name LIMIT ? OFFSET ?",
                (case_id, limit, offset),
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
        limit: int = 100,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        if status:
            cursor = await self._conn.execute(
                "SELECT * FROM hypotheses WHERE case_id = ? AND status = ? ORDER BY current_score DESC LIMIT ? OFFSET ?",
                (case_id, status, limit, offset),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT * FROM hypotheses WHERE case_id = ? ORDER BY current_score DESC LIMIT ? OFFSET ?",
                (case_id, limit, offset),
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
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        if status:
            cursor = await self._conn.execute(
                "SELECT * FROM analysis_runs WHERE case_id = ? AND status = ? ORDER BY started_at DESC LIMIT ? OFFSET ?",
                (case_id, status, limit, offset),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT * FROM analysis_runs WHERE case_id = ? ORDER BY started_at DESC LIMIT ? OFFSET ?",
                (case_id, limit, offset),
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
               (id, case_id, job_type, query, entity_id, interval_hours, is_active,
                last_run, next_run, results_count, created_at)
               VALUES (?, ?, ?, ?, ?, ?, 1, NULL, ?, 0, ?)""",
            (row_id, case_id, job_type, query, entity_id, interval_hours, now, now),
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
        limit: int = 500,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        if active_only:
            cursor = await self._conn.execute(
                "SELECT * FROM monitoring_jobs WHERE case_id = ? AND is_active = 1 ORDER BY created_at DESC LIMIT ? OFFSET ?",
                (case_id, limit, offset),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT * FROM monitoring_jobs WHERE case_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
                (case_id, limit, offset),
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
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        if unreviewed_only:
            cursor = await self._conn.execute(
                "SELECT * FROM monitoring_results WHERE job_id = ? AND reviewed = 0 ORDER BY found_at DESC LIMIT ? OFFSET ?",
                (job_id, limit, offset),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT * FROM monitoring_results WHERE job_id = ? ORDER BY found_at DESC LIMIT ? OFFSET ?",
                (job_id, limit, offset),
            )
        return [_row_to_dict(r) for r in await cursor.fetchall()]

    async def list_results_by_case(
        self,
        case_id: str,
        *,
        limit: int = 200,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM monitoring_results WHERE case_id = ? ORDER BY found_at DESC LIMIT ? OFFSET ?",
            (case_id, limit, offset),
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
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        conditions = ["case_id = ?"]
        params: list[Any] = [case_id]
        if unread_only:
            conditions.append("is_read = 0")
        if severity:
            conditions.append("severity = ?")
            params.append(severity)
        where = " AND ".join(conditions)
        params.extend([limit, offset])
        cursor = await self._conn.execute(
            f"SELECT * FROM alerts WHERE {where} ORDER BY created_at DESC LIMIT ? OFFSET ?",
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

    async def list_reports_by_case(
        self,
        case_id: str,
        *,
        limit: int = 200,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM reports WHERE case_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
            (case_id, limit, offset),
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

    async def list_locations_by_case(
        self,
        case_id: str,
        *,
        limit: int = 1000,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM locations WHERE case_id = ? ORDER BY name LIMIT ? OFFSET ?",
            (case_id, limit, offset),
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

    # ------------------------------------------------------------------
    # Summary Clusters (RAPTOR level 1)
    # ------------------------------------------------------------------

    async def create_cluster(
        self,
        *,
        case_id: str,
        title: Optional[str] = None,
        summary: Optional[str] = None,
        evidence_ids: Optional[str] = None,
        embedding_centroid: Optional[str] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO summary_clusters
               (id, case_id, title, summary, evidence_ids,
                embedding_centroid, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, case_id, title, summary, evidence_ids,
                embedding_centroid, now, now,
            ),
        )
        await self._conn.commit()
        return await self.get_cluster(row_id)  # type: ignore[return-value]

    async def get_cluster(self, cluster_id: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM summary_clusters WHERE id = ?", (cluster_id,)
        )
        row = await cursor.fetchone()
        return _row_to_dict(row) if row else None

    async def list_clusters_by_case(
        self,
        case_id: str,
        *,
        limit: int = 500,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM summary_clusters WHERE case_id = ? ORDER BY created_at LIMIT ? OFFSET ?",
            (case_id, limit, offset),
        )
        return [_row_to_dict(r) for r in await cursor.fetchall()]

    async def update_cluster(
        self, cluster_id: str, **fields: Any
    ) -> Optional[Dict[str, Any]]:
        if not fields:
            return await self.get_cluster(cluster_id)
        fields["updated_at"] = _now_iso()
        set_clause = ", ".join(f"{k} = ?" for k in fields)
        values = list(fields.values()) + [cluster_id]
        await self._conn.execute(
            f"UPDATE summary_clusters SET {set_clause} WHERE id = ?", values
        )
        await self._conn.commit()
        return await self.get_cluster(cluster_id)

    async def delete_cluster(self, cluster_id: str) -> bool:
        cursor = await self._conn.execute(
            "DELETE FROM summary_clusters WHERE id = ?", (cluster_id,)
        )
        await self._conn.commit()
        return cursor.rowcount > 0

    # ------------------------------------------------------------------
    # Case Summaries (RAPTOR level 2)
    # ------------------------------------------------------------------

    async def create_or_update_case_summary(
        self,
        *,
        case_id: str,
        summary: str,
        cluster_ids: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Upsert the level-2 case summary (one per case)."""
        now = _now_iso()
        # Check if a summary already exists for this case
        existing = await self.get_case_summary(case_id)
        if existing:
            await self._conn.execute(
                """UPDATE case_summaries
                   SET summary = ?, cluster_ids = ?, updated_at = ?
                   WHERE case_id = ?""",
                (summary, cluster_ids, now, case_id),
            )
            await self._conn.commit()
            return await self.get_case_summary(case_id)  # type: ignore[return-value]
        else:
            row_id = _new_id()
            await self._conn.execute(
                """INSERT INTO case_summaries
                   (id, case_id, summary, cluster_ids, created_at, updated_at)
                   VALUES (?, ?, ?, ?, ?, ?)""",
                (row_id, case_id, summary, cluster_ids, now, now),
            )
            await self._conn.commit()
            return await self.get_case_summary(case_id)  # type: ignore[return-value]

    async def get_case_summary(
        self, case_id: str
    ) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM case_summaries WHERE case_id = ?", (case_id,)
        )
        row = await cursor.fetchone()
        return _row_to_dict(row) if row else None

    # ------------------------------------------------------------------
    # FTS5 Full-Text Search
    # ------------------------------------------------------------------

    async def search_evidence_fts(
        self,
        case_id: str,
        query: str,
        limit: int = 20,
    ) -> List[Dict[str, Any]]:
        """Full-text search across evidence (title, raw_text, summary, source)."""
        cursor = await self._conn.execute(
            """SELECT e.* FROM evidence e
               JOIN evidence_fts ON e.rowid = evidence_fts.rowid
               WHERE evidence_fts MATCH ? AND e.case_id = ?
               ORDER BY rank LIMIT ?""",
            (query, case_id, limit),
        )
        return [
            _dict_with_json_fields(_row_to_dict(r), "metadata")
            for r in await cursor.fetchall()
        ]

    # ------------------------------------------------------------------
    # Suspects
    # ------------------------------------------------------------------

    async def create_suspect(
        self,
        *,
        case_id: str,
        entity_id: str,
        suspicion_score: float = 0.0,
        graph_score: float = 0.0,
        evidence_score: float = 0.0,
        contradiction_score: float = 0.0,
        profile_score: float = 0.0,
        hypothesis_score: float = 0.0,
        known_motive: Optional[str] = None,
        alibi_status: str = "unknown",
        criminal_record: Optional[str] = None,
        relationship_to_victim: Optional[str] = None,
        notes: Optional[str] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO suspects
               (id, case_id, entity_id, suspicion_score, graph_score,
                evidence_score, contradiction_score, profile_score,
                hypothesis_score, known_motive, alibi_status,
                criminal_record, relationship_to_victim, notes,
                created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, case_id, entity_id, suspicion_score,
                graph_score, evidence_score, contradiction_score,
                profile_score, hypothesis_score, known_motive,
                alibi_status, criminal_record, relationship_to_victim,
                notes, now, now,
            ),
        )
        await self._conn.commit()
        return await self.get_suspect(row_id)  # type: ignore[return-value]

    async def get_suspect(self, suspect_id: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM suspects WHERE id = ?", (suspect_id,)
        )
        row = await cursor.fetchone()
        return _row_to_dict(row) if row else None

    async def get_suspect_by_entity(
        self, case_id: str, entity_id: str
    ) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM suspects WHERE case_id = ? AND entity_id = ?",
            (case_id, entity_id),
        )
        row = await cursor.fetchone()
        return _row_to_dict(row) if row else None

    async def list_suspects_by_case(
        self,
        case_id: str,
        *,
        limit: int = 200,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM suspects WHERE case_id = ? ORDER BY suspicion_score DESC LIMIT ? OFFSET ?",
            (case_id, limit, offset),
        )
        return [_row_to_dict(r) for r in await cursor.fetchall()]

    async def update_suspect(
        self, suspect_id: str, **fields: Any
    ) -> Optional[Dict[str, Any]]:
        if not fields:
            return await self.get_suspect(suspect_id)
        fields["updated_at"] = _now_iso()
        set_clause = ", ".join(f"{k} = ?" for k in fields)
        values = list(fields.values()) + [suspect_id]
        await self._conn.execute(
            f"UPDATE suspects SET {set_clause} WHERE id = ?", values
        )
        await self._conn.commit()
        return await self.get_suspect(suspect_id)

    async def delete_suspect(self, suspect_id: str) -> bool:
        # Delete child snapshots first
        await self._conn.execute(
            "DELETE FROM suspect_snapshots WHERE suspect_id = ?",
            (suspect_id,),
        )
        cursor = await self._conn.execute(
            "DELETE FROM suspects WHERE id = ?", (suspect_id,)
        )
        await self._conn.commit()
        return cursor.rowcount > 0

    # ------------------------------------------------------------------
    # Suspect Snapshots
    # ------------------------------------------------------------------

    async def create_suspect_snapshot(
        self,
        *,
        suspect_id: str,
        suspicion_score: float,
        graph_score: Optional[float] = None,
        evidence_score: Optional[float] = None,
        contradiction_score: Optional[float] = None,
        profile_score: Optional[float] = None,
        hypothesis_score: Optional[float] = None,
        trigger: Optional[str] = None,
        reasoning: Optional[str] = None,
        model_used: Optional[str] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO suspect_snapshots
               (id, suspect_id, suspicion_score, graph_score,
                evidence_score, contradiction_score, profile_score,
                hypothesis_score, trigger, reasoning, model_used,
                created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, suspect_id, suspicion_score, graph_score,
                evidence_score, contradiction_score, profile_score,
                hypothesis_score, trigger, reasoning, model_used, now,
            ),
        )
        await self._conn.commit()
        cursor = await self._conn.execute(
            "SELECT * FROM suspect_snapshots WHERE id = ?", (row_id,)
        )
        row = await cursor.fetchone()
        return _row_to_dict(row)  # type: ignore[arg-type]

    async def list_suspect_snapshots(
        self,
        suspect_id: str,
        *,
        limit: int = 200,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM suspect_snapshots WHERE suspect_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
            (suspect_id, limit, offset),
        )
        return [_row_to_dict(r) for r in await cursor.fetchall()]

    # ------------------------------------------------------------------
    # Batch / aggregate helpers (avoid N+1 queries)
    # ------------------------------------------------------------------

    async def get_evidence_batch(
        self,
        evidence_ids: List[str],
    ) -> List[Dict[str, Any]]:
        """Fetch multiple evidence rows in a single SELECT with IN clause."""
        if not evidence_ids:
            return []
        placeholders = ", ".join("?" for _ in evidence_ids)
        cursor = await self._conn.execute(
            f"SELECT * FROM evidence WHERE id IN ({placeholders})",
            evidence_ids,
        )
        return [
            _dict_with_json_fields(_row_to_dict(r), "metadata")
            for r in await cursor.fetchall()
        ]

    async def count_entities_by_type(
        self,
        case_id: str,
    ) -> Dict[str, int]:
        """Return entity counts grouped by type (single GROUP BY query)."""
        cursor = await self._conn.execute(
            "SELECT entity_type, COUNT(*) FROM entities WHERE case_id = ? GROUP BY entity_type",
            (case_id,),
        )
        return {row[0]: row[1] for row in await cursor.fetchall()}

    # ------------------------------------------------------------------
    # Contradictions
    # ------------------------------------------------------------------

    async def create_contradiction(
        self,
        *,
        case_id: str,
        evidence_1_id: str | None = None,
        evidence_2_id: str | None = None,
        evidence_1_title: str | None = None,
        evidence_2_title: str | None = None,
        contradiction_type: str = "factual",
        severity: str = "medium",
        description: str,
        likely_correct: str | None = None,
        reasoning: str | None = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        try:
            await self._conn.execute(
                """INSERT INTO contradictions
                   (id, case_id, evidence_1_id, evidence_2_id,
                    evidence_1_title, evidence_2_title,
                    contradiction_type, severity, description,
                    likely_correct, reasoning, created_at)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    row_id, case_id, evidence_1_id, evidence_2_id,
                    evidence_1_title, evidence_2_title,
                    contradiction_type, severity, description,
                    likely_correct, reasoning, now,
                ),
            )
            await self._conn.commit()
        except Exception:
            # UNIQUE constraint violation — contradiction already exists
            await self._conn.rollback()
            return {}
        cursor = await self._conn.execute(
            "SELECT * FROM contradictions WHERE id = ?", (row_id,)
        )
        row = await cursor.fetchone()
        return _row_to_dict(row) if row else {}

    async def list_contradictions_by_case(
        self,
        case_id: str,
        *,
        limit: int = 200,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM contradictions WHERE case_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
            (case_id, limit, offset),
        )
        return [_row_to_dict(r) for r in await cursor.fetchall()]

    async def count_contradictions(self, case_id: str) -> int:
        cursor = await self._conn.execute(
            "SELECT COUNT(*) FROM contradictions WHERE case_id = ?", (case_id,)
        )
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # Investigation Memory
    # ------------------------------------------------------------------

    async def create_investigation_memory(
        self,
        case_id: str,
        insight_type: str,
        source_event_type: str,
        summary: str,
        importance: float = 0.5,
        confidence: float = 0.7,
        full_context: Optional[str] = None,
        related_entities: Optional[List[str]] = None,
    ) -> Dict[str, Any]:
        mem_id = _new_id()
        now = _now_iso()
        related_json = _json_dumps(related_entities)
        await self._conn.execute(
            """INSERT INTO investigation_memory
               (id, case_id, insight_type, source_event_type, importance,
                confidence, summary, full_context, related_entities, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (mem_id, case_id, insight_type, source_event_type, importance,
             confidence, summary, full_context, related_json, now),
        )
        await self._conn.commit()
        return {
            "id": mem_id,
            "case_id": case_id,
            "insight_type": insight_type,
            "source_event_type": source_event_type,
            "importance": importance,
            "confidence": confidence,
            "summary": summary,
            "full_context": full_context,
            "related_entities": related_entities,
            "created_at": now,
        }

    async def list_memories_by_case(
        self,
        case_id: str,
        min_importance: float = 0.0,
        limit: int = 50,
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            """SELECT * FROM investigation_memory
               WHERE case_id = ? AND importance >= ?
               ORDER BY created_at DESC LIMIT ?""",
            (case_id, min_importance, limit),
        )
        return [_row_to_dict(r) for r in await cursor.fetchall()]

    async def count_memories(self, case_id: str) -> int:
        cursor = await self._conn.execute(
            "SELECT COUNT(*) FROM investigation_memory WHERE case_id = ?",
            (case_id,),
        )
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # Wiki Pages
    # ------------------------------------------------------------------

    async def upsert_wiki_page(
        self,
        *,
        case_id: str,
        page_path: str,
        page_type: str,
        title: str,
        content_hash: Optional[str] = None,
        source_ids: Optional[List[str]] = None,
    ) -> Optional[Dict[str, Any]]:
        """Insert or update a wiki page record."""
        now = _now_iso()
        existing = await self.get_wiki_page(case_id, page_path)
        if existing:
            await self._conn.execute(
                "UPDATE wiki_pages SET title=?, content_hash=?, last_compiled=?, source_ids=? WHERE id=?",
                (title, content_hash, now, _json_dumps(source_ids), existing["id"]),
            )
            await self._conn.commit()
            return await self.get_wiki_page(case_id, page_path)
        row_id = _new_id()
        await self._conn.execute(
            """INSERT INTO wiki_pages (id, case_id, page_path, page_type, title, content_hash, last_compiled, source_ids, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (row_id, case_id, page_path, page_type, title, content_hash, now, _json_dumps(source_ids), now),
        )
        await self._conn.commit()
        return await self.get_wiki_page(case_id, page_path)

    async def get_wiki_page(self, case_id: str, page_path: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM wiki_pages WHERE case_id = ? AND page_path = ?", (case_id, page_path),
        )
        row = await cursor.fetchone()
        return _dict_with_json_fields(_row_to_dict(row), "source_ids") if row else None

    async def list_wiki_pages(
        self, case_id: str, page_type: Optional[str] = None, limit: int = 500,
    ) -> List[Dict[str, Any]]:
        if page_type:
            cursor = await self._conn.execute(
                "SELECT * FROM wiki_pages WHERE case_id = ? AND page_type = ? ORDER BY page_path LIMIT ?",
                (case_id, page_type, limit),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT * FROM wiki_pages WHERE case_id = ? ORDER BY page_path LIMIT ?",
                (case_id, limit),
            )
        return [_dict_with_json_fields(_row_to_dict(r), "source_ids") for r in await cursor.fetchall()]
