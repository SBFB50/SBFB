"""
NEXUS GOV -- Government Monitoring database layer.

DDL + CRUD for French government monitoring tables:
- gov_politicians
- gov_positions
- gov_contradictions
- gov_scan_log
- gov_mandates
- gov_parties
- gov_party_memberships
- gov_affairs
- gov_declarations
- gov_laws
- gov_press
- gov_social_posts
- gov_transcriptions
- gov_factchecks
- gov_external_ids
- gov_alerts

Uses the same helpers and connection management as sqlite_db.py.
"""

from __future__ import annotations

import re
import uuid
from collections import defaultdict
from itertools import combinations
from typing import Any, Dict, List, Optional

import aiosqlite
from loguru import logger
from sqlite3 import IntegrityError

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

_GOV_CREATE_TABLES = """
CREATE TABLE IF NOT EXISTS gov_politicians (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    chamber TEXT NOT NULL,
    party TEXT,
    role TEXT,
    constituency TEXT,
    photo_url TEXT,
    official_url TEXT,
    hatvp_url TEXT,
    active INTEGER DEFAULT 1,
    metadata TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS gov_positions (
    id TEXT PRIMARY KEY,
    politician_id TEXT NOT NULL REFERENCES gov_politicians(id),
    subject TEXT NOT NULL,
    position_type TEXT NOT NULL,
    position_text TEXT NOT NULL,
    stance TEXT,
    source_url TEXT NOT NULL,
    source_type TEXT,
    date DATE,
    session TEXT,
    metadata TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS gov_contradictions (
    id TEXT PRIMARY KEY,
    politician_id TEXT NOT NULL REFERENCES gov_politicians(id),
    position_a_id TEXT NOT NULL REFERENCES gov_positions(id),
    position_b_id TEXT NOT NULL REFERENCES gov_positions(id),
    subject TEXT NOT NULL,
    description TEXT NOT NULL,
    severity TEXT DEFAULT 'medium',
    source_verified INTEGER DEFAULT 0,
    metadata TEXT,
    detected_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS gov_scan_log (
    id TEXT PRIMARY KEY,
    scan_type TEXT NOT NULL,
    status TEXT DEFAULT 'running',
    items_found INTEGER DEFAULT 0,
    items_new INTEGER DEFAULT 0,
    error_message TEXT,
    started_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME
);

CREATE TABLE IF NOT EXISTS gov_mandates (
    id TEXT PRIMARY KEY,
    politician_id TEXT NOT NULL REFERENCES gov_politicians(id),
    type TEXT NOT NULL,
    title TEXT,
    institution TEXT,
    constituency TEXT,
    start_date DATE,
    end_date DATE,
    is_current INTEGER DEFAULT 0,
    parliamentary_group TEXT,
    metadata TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS gov_parties (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    short_name TEXT UNIQUE,
    color TEXT,
    description TEXT,
    leader TEXT,
    metadata TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS gov_party_memberships (
    id TEXT PRIMARY KEY,
    politician_id TEXT NOT NULL REFERENCES gov_politicians(id),
    party_id TEXT NOT NULL REFERENCES gov_parties(id),
    start_date DATE,
    end_date DATE,
    is_current INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS gov_affairs (
    id TEXT PRIMARY KEY,
    politician_id TEXT NOT NULL REFERENCES gov_politicians(id),
    title TEXT NOT NULL,
    description TEXT,
    status TEXT DEFAULT 'enquete',
    category TEXT,
    involvement TEXT DEFAULT 'direct',
    source_url TEXT,
    date_start DATE,
    date_end DATE,
    metadata TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS gov_declarations (
    id TEXT PRIMARY KEY,
    politician_id TEXT NOT NULL REFERENCES gov_politicians(id),
    type TEXT NOT NULL,
    qualite TEXT,
    departement TEXT,
    date_publication DATE,
    date_depot DATE,
    url TEXT,
    status TEXT,
    metadata TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS gov_laws (
    id TEXT PRIMARY KEY,
    uid TEXT UNIQUE,
    title TEXT NOT NULL,
    short_title TEXT,
    procedure TEXT,
    status TEXT,
    initiator_ref TEXT,
    date_initial DATE,
    date_promulgation DATE,
    legislature TEXT,
    amendments_count INTEGER DEFAULT 0,
    amendments_adopted INTEGER DEFAULT 0,
    articles_initial INTEGER DEFAULT 0,
    articles_final INTEGER DEFAULT 0,
    duration_days INTEGER DEFAULT 0,
    source_url TEXT,
    jo_url TEXT,
    metadata TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS gov_press (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    url TEXT UNIQUE,
    source_name TEXT,
    published_at DATETIME,
    summary TEXT,
    sentiment TEXT,
    politicians_mentioned TEXT,
    subjects TEXT,
    metadata TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS gov_social_posts (
    id TEXT PRIMARY KEY,
    politician_id TEXT NOT NULL REFERENCES gov_politicians(id),
    platform TEXT NOT NULL,
    post_id TEXT,
    content TEXT,
    url TEXT,
    media_type TEXT,
    media_url TEXT,
    posted_at DATETIME,
    likes INTEGER DEFAULT 0,
    shares INTEGER DEFAULT 0,
    comments INTEGER DEFAULT 0,
    metadata TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(platform, post_id)
);

CREATE TABLE IF NOT EXISTS gov_transcriptions (
    id TEXT PRIMARY KEY,
    source_type TEXT NOT NULL,
    source_url TEXT,
    politician_id TEXT REFERENCES gov_politicians(id),
    title TEXT,
    transcription TEXT,
    timestamped_text TEXT,
    duration_seconds INTEGER,
    language TEXT DEFAULT 'fr',
    model_used TEXT,
    metadata TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS gov_factchecks (
    id TEXT PRIMARY KEY,
    claim TEXT NOT NULL,
    claim_date DATE,
    claimant TEXT,
    politician_id TEXT REFERENCES gov_politicians(id),
    rating TEXT,
    review_url TEXT,
    reviewer TEXT,
    review_date DATE,
    metadata TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS gov_external_ids (
    id TEXT PRIMARY KEY,
    politician_id TEXT NOT NULL REFERENCES gov_politicians(id),
    source TEXT NOT NULL,
    external_id TEXT NOT NULL,
    confidence REAL DEFAULT 1.0,
    metadata TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(source, external_id)
);

CREATE TABLE IF NOT EXISTS gov_alerts (
    id TEXT PRIMARY KEY,
    alert_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    severity TEXT DEFAULT 'info',
    politician_id TEXT REFERENCES gov_politicians(id),
    event_id TEXT,
    is_read INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
"""

_GOV_CREATE_INDEXES = """
CREATE INDEX IF NOT EXISTS idx_gov_positions_politician ON gov_positions(politician_id);
CREATE INDEX IF NOT EXISTS idx_gov_positions_date ON gov_positions(date);
CREATE INDEX IF NOT EXISTS idx_gov_positions_subject ON gov_positions(subject);
CREATE INDEX IF NOT EXISTS idx_gov_positions_type ON gov_positions(position_type);
CREATE INDEX IF NOT EXISTS idx_gov_contradictions_politician ON gov_contradictions(politician_id);
CREATE INDEX IF NOT EXISTS idx_gov_contradictions_subject ON gov_contradictions(subject);
CREATE INDEX IF NOT EXISTS idx_gov_scan_log_type ON gov_scan_log(scan_type);
CREATE INDEX IF NOT EXISTS idx_gov_politicians_chamber ON gov_politicians(chamber);
CREATE INDEX IF NOT EXISTS idx_gov_politicians_party ON gov_politicians(party);
CREATE INDEX IF NOT EXISTS idx_gov_politicians_slug ON gov_politicians(slug);
CREATE INDEX IF NOT EXISTS idx_gov_mandates_politician ON gov_mandates(politician_id);
CREATE INDEX IF NOT EXISTS idx_gov_mandates_current ON gov_mandates(is_current);
CREATE INDEX IF NOT EXISTS idx_gov_mandates_type ON gov_mandates(type);
CREATE INDEX IF NOT EXISTS idx_gov_party_memberships_politician ON gov_party_memberships(politician_id);
CREATE INDEX IF NOT EXISTS idx_gov_affairs_politician ON gov_affairs(politician_id);
CREATE INDEX IF NOT EXISTS idx_gov_affairs_status ON gov_affairs(status);
CREATE INDEX IF NOT EXISTS idx_gov_declarations_politician ON gov_declarations(politician_id);
CREATE INDEX IF NOT EXISTS idx_gov_laws_uid ON gov_laws(uid);
CREATE INDEX IF NOT EXISTS idx_gov_laws_status ON gov_laws(status);
CREATE INDEX IF NOT EXISTS idx_gov_press_url ON gov_press(url);
CREATE INDEX IF NOT EXISTS idx_gov_press_published ON gov_press(published_at);
CREATE INDEX IF NOT EXISTS idx_gov_social_politician ON gov_social_posts(politician_id);
CREATE INDEX IF NOT EXISTS idx_gov_social_platform ON gov_social_posts(platform);
CREATE INDEX IF NOT EXISTS idx_gov_social_posted ON gov_social_posts(posted_at);
CREATE INDEX IF NOT EXISTS idx_gov_transcriptions_politician ON gov_transcriptions(politician_id);
CREATE INDEX IF NOT EXISTS idx_gov_transcriptions_source ON gov_transcriptions(source_type);
CREATE INDEX IF NOT EXISTS idx_gov_factchecks_politician ON gov_factchecks(politician_id);
CREATE INDEX IF NOT EXISTS idx_gov_external_ids_politician ON gov_external_ids(politician_id);
CREATE INDEX IF NOT EXISTS idx_gov_external_ids_source ON gov_external_ids(source, external_id);
CREATE INDEX IF NOT EXISTS idx_gov_alerts_type ON gov_alerts(alert_type);
CREATE INDEX IF NOT EXISTS idx_gov_alerts_read ON gov_alerts(is_read);
"""

_GOV_CREATE_FTS = """
-- FTS5 virtual tables for full-text search on gov content
CREATE VIRTUAL TABLE IF NOT EXISTS gov_positions_fts USING fts5(
    subject, position_text, content=gov_positions, content_rowid=rowid
);

CREATE VIRTUAL TABLE IF NOT EXISTS gov_press_fts USING fts5(
    title, summary, content=gov_press, content_rowid=rowid
);

CREATE VIRTUAL TABLE IF NOT EXISTS gov_transcriptions_fts USING fts5(
    title, transcription, content=gov_transcriptions, content_rowid=rowid
);

-- Triggers to keep FTS indexes in sync
CREATE TRIGGER IF NOT EXISTS gov_positions_ai AFTER INSERT ON gov_positions BEGIN
    INSERT INTO gov_positions_fts(rowid, subject, position_text)
    VALUES (new.rowid, new.subject, new.position_text);
END;

CREATE TRIGGER IF NOT EXISTS gov_positions_ad AFTER DELETE ON gov_positions BEGIN
    INSERT INTO gov_positions_fts(gov_positions_fts, rowid, subject, position_text)
    VALUES ('delete', old.rowid, old.subject, old.position_text);
END;

CREATE TRIGGER IF NOT EXISTS gov_positions_au AFTER UPDATE ON gov_positions BEGIN
    INSERT INTO gov_positions_fts(gov_positions_fts, rowid, subject, position_text)
    VALUES ('delete', old.rowid, old.subject, old.position_text);
    INSERT INTO gov_positions_fts(rowid, subject, position_text)
    VALUES (new.rowid, new.subject, new.position_text);
END;

CREATE TRIGGER IF NOT EXISTS gov_press_ai AFTER INSERT ON gov_press BEGIN
    INSERT INTO gov_press_fts(rowid, title, summary)
    VALUES (new.rowid, new.title, new.summary);
END;

CREATE TRIGGER IF NOT EXISTS gov_press_ad AFTER DELETE ON gov_press BEGIN
    INSERT INTO gov_press_fts(gov_press_fts, rowid, title, summary)
    VALUES ('delete', old.rowid, old.title, old.summary);
END;

CREATE TRIGGER IF NOT EXISTS gov_press_au AFTER UPDATE ON gov_press BEGIN
    INSERT INTO gov_press_fts(gov_press_fts, rowid, title, summary)
    VALUES ('delete', old.rowid, old.title, old.summary);
    INSERT INTO gov_press_fts(rowid, title, summary)
    VALUES (new.rowid, new.title, new.summary);
END;

CREATE TRIGGER IF NOT EXISTS gov_transcriptions_ai AFTER INSERT ON gov_transcriptions BEGIN
    INSERT INTO gov_transcriptions_fts(rowid, title, transcription)
    VALUES (new.rowid, new.title, new.transcription);
END;

CREATE TRIGGER IF NOT EXISTS gov_transcriptions_ad AFTER DELETE ON gov_transcriptions BEGIN
    INSERT INTO gov_transcriptions_fts(gov_transcriptions_fts, rowid, title, transcription)
    VALUES ('delete', old.rowid, old.title, old.transcription);
END;

CREATE TRIGGER IF NOT EXISTS gov_transcriptions_au AFTER UPDATE ON gov_transcriptions BEGIN
    INSERT INTO gov_transcriptions_fts(gov_transcriptions_fts, rowid, title, transcription)
    VALUES ('delete', old.rowid, old.title, old.transcription);
    INSERT INTO gov_transcriptions_fts(rowid, title, transcription)
    VALUES (new.rowid, new.title, new.transcription);
END;
"""


# ============================================================================
# Slug helper
# ============================================================================

def _slugify(name: str) -> str:
    """Turn a politician name into a URL-safe slug."""
    slug = name.lower().strip()
    # Normalize common French accented characters
    replacements = {
        "é": "e", "è": "e", "ê": "e", "ë": "e",
        "à": "a", "â": "a", "ä": "a",
        "ù": "u", "û": "u", "ü": "u",
        "î": "i", "ï": "i",
        "ô": "o", "ö": "o",
        "ç": "c", "ñ": "n",
    }
    for src, dst in replacements.items():
        slug = slug.replace(src, dst)
    slug = re.sub(r"[^a-z0-9]+", "-", slug)
    return slug.strip("-")


# ============================================================================
# Init
# ============================================================================

async def init_government_db() -> None:
    """Create government monitoring tables and indexes (idempotent)."""
    async with get_db() as conn:
        # Ensure WAL mode, FK enforcement, and sync pragmas on this connection
        await conn.execute("PRAGMA journal_mode = WAL")
        await conn.execute("PRAGMA foreign_keys = ON")
        await conn.execute("PRAGMA synchronous = NORMAL")
        await conn.executescript(_GOV_CREATE_TABLES)
        await conn.executescript(_GOV_CREATE_INDEXES)
        await conn.executescript(_GOV_CREATE_FTS)
        await conn.commit()
    logger.info("Government monitoring tables initialised")


# ============================================================================
# Database CRUD class
# ============================================================================

class GovernmentDatabase:
    """CRUD operations for government monitoring tables.

    Usage::

        async with get_db() as conn:
            gov = GovernmentDatabase(conn)
            pol = await gov.create_politician(name="Jean Dupont", chamber="assemblee")
    """

    def __init__(self, conn: aiosqlite.Connection) -> None:
        self._conn = conn

    # ------------------------------------------------------------------
    # Politicians
    # ------------------------------------------------------------------

    async def create_politician(
        self,
        *,
        name: str,
        chamber: str,
        party: Optional[str] = None,
        role: Optional[str] = None,
        constituency: Optional[str] = None,
        photo_url: Optional[str] = None,
        official_url: Optional[str] = None,
        hatvp_url: Optional[str] = None,
        active: bool = True,
        metadata: Optional[Any] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        slug = _slugify(name)
        await self._conn.execute(
            """INSERT INTO gov_politicians
               (id, name, slug, chamber, party, role, constituency,
                photo_url, official_url, hatvp_url, active, metadata,
                created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, name, slug, chamber, party, role, constituency,
                photo_url, official_url, hatvp_url, int(active),
                _json_dumps(metadata), now, now,
            ),
        )
        await self._conn.commit()
        return await self.get_politician(row_id)  # type: ignore[return-value]

    async def get_politician(self, politician_id: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM gov_politicians WHERE id = ?", (politician_id,)
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        d = _row_to_dict(row)
        d["active"] = bool(d.get("active", 1))
        return _dict_with_json_fields(d, "metadata")

    async def list_politicians(
        self,
        *,
        chamber: Optional[str] = None,
        party: Optional[str] = None,
        active: Optional[bool] = None,
        limit: int = 200,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        conditions: List[str] = []
        params: List[Any] = []
        if chamber is not None:
            conditions.append("chamber = ?")
            params.append(chamber)
        if party is not None:
            conditions.append("party = ?")
            params.append(party)
        if active is not None:
            conditions.append("active = ?")
            params.append(int(active))
        where = ("WHERE " + " AND ".join(conditions)) if conditions else ""
        query = f"SELECT * FROM gov_politicians {where} ORDER BY name ASC LIMIT ? OFFSET ?"
        params.extend([limit, offset])
        cursor = await self._conn.execute(query, params)
        rows = await cursor.fetchall()
        result = []
        for r in rows:
            d = _row_to_dict(r)
            d["active"] = bool(d.get("active", 1))
            result.append(_dict_with_json_fields(d, "metadata"))
        return result

    async def update_politician(
        self, politician_id: str, **fields: Any
    ) -> Optional[Dict[str, Any]]:
        if not fields:
            return await self.get_politician(politician_id)
        # Auto-update slug if name changes
        if "name" in fields:
            fields["slug"] = _slugify(fields["name"])
        # Serialize metadata if provided
        if "metadata" in fields:
            fields["metadata"] = _json_dumps(fields["metadata"])
        # Convert active bool to int
        if "active" in fields and isinstance(fields["active"], bool):
            fields["active"] = int(fields["active"])
        fields["updated_at"] = _now_iso()
        set_clause = ", ".join(f"{k} = ?" for k in fields)
        values = list(fields.values()) + [politician_id]
        await self._conn.execute(
            f"UPDATE gov_politicians SET {set_clause} WHERE id = ?", values
        )
        await self._conn.commit()
        return await self.get_politician(politician_id)

    async def delete_politician(self, politician_id: str) -> bool:
        """Delete a politician and ALL dependent rows across every FK table."""
        cascade_tables = [
            "gov_contradictions",
            "gov_positions",
            "gov_affairs",
            "gov_declarations",
            "gov_mandates",
            "gov_party_memberships",
            "gov_social_posts",
            "gov_transcriptions",
            "gov_factchecks",
            "gov_external_ids",
            "gov_alerts",
        ]
        for table in cascade_tables:
            await self._conn.execute(
                f"DELETE FROM {table} WHERE politician_id = ?",
                (politician_id,),
            )
        cursor = await self._conn.execute(
            "DELETE FROM gov_politicians WHERE id = ?", (politician_id,)
        )
        await self._conn.commit()
        return cursor.rowcount > 0

    async def search_politicians(self, query: str) -> List[Dict[str, Any]]:
        """Search politicians by name (LIKE match)."""
        pattern = f"%{query}%"
        cursor = await self._conn.execute(
            "SELECT * FROM gov_politicians WHERE name LIKE ? ORDER BY name ASC LIMIT 50",
            (pattern,),
        )
        rows = await cursor.fetchall()
        result = []
        for r in rows:
            d = _row_to_dict(r)
            d["active"] = bool(d.get("active", 1))
            result.append(_dict_with_json_fields(d, "metadata"))
        return result

    async def count_politicians(self, *, active: Optional[bool] = None) -> int:
        """Count politicians, optionally filtered by active status."""
        if active is not None:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_politicians WHERE active = ?",
                (int(active),),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_politicians"
            )
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # Positions
    # ------------------------------------------------------------------

    async def create_position(
        self,
        *,
        politician_id: str,
        subject: str,
        position_type: str,
        position_text: str,
        stance: Optional[str] = None,
        source_url: str,
        source_type: Optional[str] = None,
        date: Optional[str] = None,
        session: Optional[str] = None,
        metadata: Optional[Any] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO gov_positions
               (id, politician_id, subject, position_type, position_text,
                stance, source_url, source_type, date, session, metadata,
                created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, politician_id, subject, position_type, position_text,
                stance, source_url, source_type, date, session,
                _json_dumps(metadata), now,
            ),
        )
        await self._conn.commit()
        return await self.get_position(row_id)  # type: ignore[return-value]

    async def get_position(self, position_id: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM gov_positions WHERE id = ?", (position_id,)
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def list_positions_by_politician(
        self,
        politician_id: str,
        *,
        position_type: Optional[str] = None,
        date_from: Optional[str] = None,
        date_to: Optional[str] = None,
        limit: int = 200,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        conditions = ["politician_id = ?"]
        params: List[Any] = [politician_id]
        if position_type is not None:
            conditions.append("position_type = ?")
            params.append(position_type)
        if date_from is not None:
            conditions.append("date >= ?")
            params.append(date_from)
        if date_to is not None:
            conditions.append("date <= ?")
            params.append(date_to)
        where = "WHERE " + " AND ".join(conditions)
        query = f"SELECT * FROM gov_positions {where} ORDER BY date DESC, created_at DESC LIMIT ? OFFSET ?"
        params.extend([limit, offset])
        cursor = await self._conn.execute(query, params)
        rows = await cursor.fetchall()
        return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]

    async def count_positions(self, politician_id: Optional[str] = None) -> int:
        if politician_id:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_positions WHERE politician_id = ?",
                (politician_id,),
            )
        else:
            cursor = await self._conn.execute("SELECT COUNT(*) FROM gov_positions")
        row = await cursor.fetchone()
        return row[0] if row else 0

    async def position_exists_by_url(self, source_url: str) -> bool:
        """Check if a position with this source_url already exists (SQL EXISTS)."""
        cursor = await self._conn.execute(
            "SELECT 1 FROM gov_positions WHERE source_url = ? LIMIT 1",
            (source_url,),
        )
        return (await cursor.fetchone()) is not None

    async def position_exists_by_key(
        self, politician_id: str, subject: str, date: Optional[str]
    ) -> bool:
        """Check if a position with this (politician_id, subject, date) exists."""
        if date:
            cursor = await self._conn.execute(
                "SELECT 1 FROM gov_positions"
                " WHERE politician_id = ? AND subject = ? AND date = ?"
                " LIMIT 1",
                (politician_id, subject, date),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT 1 FROM gov_positions"
                " WHERE politician_id = ? AND subject = ?"
                " AND date IS NULL LIMIT 1",
                (politician_id, subject),
            )
        return (await cursor.fetchone()) is not None

    async def declaration_exists_by_url(self, url: str) -> bool:
        """Check if a declaration with this url already exists."""
        cursor = await self._conn.execute(
            "SELECT 1 FROM gov_declarations WHERE url = ? LIMIT 1",
            (url,),
        )
        return (await cursor.fetchone()) is not None

    # ------------------------------------------------------------------
    # Contradictions
    # ------------------------------------------------------------------

    async def create_contradiction(
        self,
        *,
        politician_id: str,
        position_a_id: str,
        position_b_id: str,
        subject: str,
        description: str,
        severity: str = "medium",
        source_verified: bool = False,
        metadata: Optional[Any] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO gov_contradictions
               (id, politician_id, position_a_id, position_b_id,
                subject, description, severity, source_verified,
                metadata, detected_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, politician_id, position_a_id, position_b_id,
                subject, description, severity, int(source_verified),
                _json_dumps(metadata), now,
            ),
        )
        await self._conn.commit()
        return await self.get_contradiction(row_id)  # type: ignore[return-value]

    async def get_contradiction(self, contradiction_id: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM gov_contradictions WHERE id = ?", (contradiction_id,)
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        d = _row_to_dict(row)
        d["source_verified"] = bool(d.get("source_verified", 0))
        return _dict_with_json_fields(d, "metadata")

    async def list_contradictions_by_politician(
        self,
        politician_id: str,
        *,
        limit: int = 200,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            """SELECT * FROM gov_contradictions
               WHERE politician_id = ?
               ORDER BY detected_at DESC LIMIT ? OFFSET ?""",
            (politician_id, limit, offset),
        )
        rows = await cursor.fetchall()
        result = []
        for r in rows:
            d = _row_to_dict(r)
            d["source_verified"] = bool(d.get("source_verified", 0))
            result.append(_dict_with_json_fields(d, "metadata"))
        return result

    async def list_all_contradictions(
        self,
        *,
        severity: Optional[str] = None,
        limit: int = 200,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        conditions: List[str] = []
        params: List[Any] = []
        if severity is not None:
            conditions.append("severity = ?")
            params.append(severity)
        where = ("WHERE " + " AND ".join(conditions)) if conditions else ""
        query = f"SELECT * FROM gov_contradictions {where} ORDER BY detected_at DESC LIMIT ? OFFSET ?"
        params.extend([limit, offset])
        cursor = await self._conn.execute(query, params)
        rows = await cursor.fetchall()
        result = []
        for r in rows:
            d = _row_to_dict(r)
            d["source_verified"] = bool(d.get("source_verified", 0))
            result.append(_dict_with_json_fields(d, "metadata"))
        return result

    async def count_contradictions(self, politician_id: Optional[str] = None) -> int:
        if politician_id:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_contradictions WHERE politician_id = ?",
                (politician_id,),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_contradictions"
            )
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # Scan Log
    # ------------------------------------------------------------------

    async def create_scan_log(
        self,
        *,
        scan_type: str,
        status: str = "running",
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO gov_scan_log (id, scan_type, status, started_at)
               VALUES (?, ?, ?, ?)""",
            (row_id, scan_type, status, now),
        )
        await self._conn.commit()
        cursor = await self._conn.execute(
            "SELECT * FROM gov_scan_log WHERE id = ?", (row_id,)
        )
        row = await cursor.fetchone()
        return _row_to_dict(row)  # type: ignore[arg-type]

    async def update_scan_log(
        self, scan_id: str, **fields: Any
    ) -> Optional[Dict[str, Any]]:
        if not fields:
            cursor = await self._conn.execute(
                "SELECT * FROM gov_scan_log WHERE id = ?", (scan_id,)
            )
            row = await cursor.fetchone()
            return _row_to_dict(row) if row else None
        set_clause = ", ".join(f"{k} = ?" for k in fields)
        values = list(fields.values()) + [scan_id]
        await self._conn.execute(
            f"UPDATE gov_scan_log SET {set_clause} WHERE id = ?", values
        )
        await self._conn.commit()
        cursor = await self._conn.execute(
            "SELECT * FROM gov_scan_log WHERE id = ?", (scan_id,)
        )
        row = await cursor.fetchone()
        return _row_to_dict(row) if row else None

    async def list_scan_logs(
        self,
        *,
        limit: int = 50,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM gov_scan_log ORDER BY started_at DESC LIMIT ? OFFSET ?",
            (limit, offset),
        )
        return [_row_to_dict(r) for r in await cursor.fetchall()]

    async def get_scan_log(self, scan_id: str) -> Optional[Dict[str, Any]]:
        """Get a single scan log entry by ID."""
        cursor = await self._conn.execute(
            "SELECT * FROM gov_scan_log WHERE id = ?", (scan_id,)
        )
        row = await cursor.fetchone()
        return _row_to_dict(row) if row else None

    async def count_scan_logs(self) -> int:
        """Count total scan log entries."""
        cursor = await self._conn.execute("SELECT COUNT(*) FROM gov_scan_log")
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # Mandates
    # ------------------------------------------------------------------

    async def create_mandate(
        self,
        *,
        politician_id: str,
        type: str,
        title: Optional[str] = None,
        institution: Optional[str] = None,
        constituency: Optional[str] = None,
        start_date: Optional[str] = None,
        end_date: Optional[str] = None,
        is_current: bool = False,
        parliamentary_group: Optional[str] = None,
        metadata: Optional[Any] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO gov_mandates
               (id, politician_id, type, title, institution, constituency,
                start_date, end_date, is_current, parliamentary_group,
                metadata, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, politician_id, type, title, institution, constituency,
                start_date, end_date, int(is_current), parliamentary_group,
                _json_dumps(metadata), now,
            ),
        )
        await self._conn.commit()
        cursor = await self._conn.execute(
            "SELECT * FROM gov_mandates WHERE id = ?", (row_id,)
        )
        row = await cursor.fetchone()
        d = _row_to_dict(row)  # type: ignore[arg-type]
        d["is_current"] = bool(d.get("is_current", 0))
        return _dict_with_json_fields(d, "metadata")

    async def list_mandates_by_politician(
        self, politician_id: str
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            """SELECT * FROM gov_mandates
               WHERE politician_id = ?
               ORDER BY start_date DESC, created_at DESC""",
            (politician_id,),
        )
        rows = await cursor.fetchall()
        result = []
        for r in rows:
            d = _row_to_dict(r)
            d["is_current"] = bool(d.get("is_current", 0))
            result.append(_dict_with_json_fields(d, "metadata"))
        return result

    async def get_mandate(self, mandate_id: str) -> Optional[Dict[str, Any]]:
        """Get a single mandate by ID."""
        cursor = await self._conn.execute(
            "SELECT * FROM gov_mandates WHERE id = ?", (mandate_id,)
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        d = _row_to_dict(row)
        d["is_current"] = bool(d.get("is_current", 0))
        return _dict_with_json_fields(d, "metadata")

    async def count_mandates(self, politician_id: Optional[str] = None) -> int:
        """Count mandates, optionally filtered by politician."""
        if politician_id:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_mandates WHERE politician_id = ?",
                (politician_id,),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_mandates"
            )
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # Parties
    # ------------------------------------------------------------------

    async def create_party(
        self,
        *,
        name: str,
        short_name: Optional[str] = None,
        color: Optional[str] = None,
        description: Optional[str] = None,
        leader: Optional[str] = None,
        metadata: Optional[Any] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        try:
            await self._conn.execute(
                """INSERT INTO gov_parties
                   (id, name, short_name, color, description, leader,
                    metadata, created_at)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    row_id, name, short_name, color, description, leader,
                    _json_dumps(metadata), now,
                ),
            )
            await self._conn.commit()
        except IntegrityError:
            if short_name:
                existing = await self.get_party_by_short_name(short_name)
                if existing:
                    return existing
            raise
        return await self.get_party(row_id)  # type: ignore[return-value]

    async def get_party(self, party_id: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM gov_parties WHERE id = ?", (party_id,)
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def get_party_by_short_name(
        self, short_name: str
    ) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM gov_parties WHERE short_name = ?", (short_name,)
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def list_parties(self) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM gov_parties ORDER BY name ASC"
        )
        rows = await cursor.fetchall()
        return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]

    async def count_parties(self) -> int:
        """Count total parties."""
        cursor = await self._conn.execute("SELECT COUNT(*) FROM gov_parties")
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # Party memberships
    # ------------------------------------------------------------------

    async def create_party_membership(
        self,
        *,
        politician_id: str,
        party_id: str,
        start_date: Optional[str] = None,
        end_date: Optional[str] = None,
        is_current: bool = False,
        metadata: Optional[Any] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO gov_party_memberships
               (id, politician_id, party_id, start_date, end_date,
                is_current, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, politician_id, party_id, start_date, end_date,
                int(is_current), now,
            ),
        )
        await self._conn.commit()
        cursor = await self._conn.execute(
            "SELECT * FROM gov_party_memberships WHERE id = ?", (row_id,)
        )
        row = await cursor.fetchone()
        d = _row_to_dict(row)  # type: ignore[arg-type]
        d["is_current"] = bool(d.get("is_current", 0))
        return d

    async def get_party_membership(
        self, membership_id: str
    ) -> Optional[Dict[str, Any]]:
        """Get a single party membership by ID."""
        cursor = await self._conn.execute(
            "SELECT * FROM gov_party_memberships WHERE id = ?", (membership_id,)
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        d = _row_to_dict(row)
        d["is_current"] = bool(d.get("is_current", 0))
        return d

    async def list_party_memberships_by_politician(
        self, politician_id: str
    ) -> List[Dict[str, Any]]:
        """List all party memberships for a politician."""
        cursor = await self._conn.execute(
            """SELECT * FROM gov_party_memberships
               WHERE politician_id = ?
               ORDER BY start_date DESC, created_at DESC""",
            (politician_id,),
        )
        rows = await cursor.fetchall()
        result = []
        for r in rows:
            d = _row_to_dict(r)
            d["is_current"] = bool(d.get("is_current", 0))
            result.append(d)
        return result

    async def count_party_memberships(
        self, politician_id: Optional[str] = None
    ) -> int:
        """Count party memberships, optionally filtered by politician."""
        if politician_id:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_party_memberships WHERE politician_id = ?",
                (politician_id,),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_party_memberships"
            )
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # Affairs
    # ------------------------------------------------------------------

    async def create_affair(
        self,
        *,
        politician_id: str,
        title: str,
        description: Optional[str] = None,
        status: str = "enquete",
        category: Optional[str] = None,
        involvement: str = "direct",
        source_url: Optional[str] = None,
        date_start: Optional[str] = None,
        date_end: Optional[str] = None,
        metadata: Optional[Any] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO gov_affairs
               (id, politician_id, title, description, status, category,
                involvement, source_url, date_start, date_end, metadata,
                created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, politician_id, title, description, status, category,
                involvement, source_url, date_start, date_end,
                _json_dumps(metadata), now,
            ),
        )
        await self._conn.commit()
        cursor = await self._conn.execute(
            "SELECT * FROM gov_affairs WHERE id = ?", (row_id,)
        )
        row = await cursor.fetchone()
        return _dict_with_json_fields(_row_to_dict(row), "metadata")  # type: ignore[arg-type]

    async def get_affair(self, affair_id: str) -> Optional[Dict[str, Any]]:
        """Get a single affair by ID."""
        cursor = await self._conn.execute(
            "SELECT * FROM gov_affairs WHERE id = ?", (affair_id,)
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def list_affairs_by_politician(
        self, politician_id: str
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            """SELECT * FROM gov_affairs
               WHERE politician_id = ?
               ORDER BY date_start DESC, created_at DESC""",
            (politician_id,),
        )
        rows = await cursor.fetchall()
        return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]

    async def count_affairs(self) -> int:
        cursor = await self._conn.execute("SELECT COUNT(*) FROM gov_affairs")
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # Declarations
    # ------------------------------------------------------------------

    async def create_declaration(
        self,
        *,
        politician_id: str,
        type: str,
        qualite: Optional[str] = None,
        departement: Optional[str] = None,
        date_publication: Optional[str] = None,
        date_depot: Optional[str] = None,
        url: Optional[str] = None,
        status: Optional[str] = None,
        metadata: Optional[Any] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO gov_declarations
               (id, politician_id, type, qualite, departement,
                date_publication, date_depot, url, status, metadata,
                created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, politician_id, type, qualite, departement,
                date_publication, date_depot, url, status,
                _json_dumps(metadata), now,
            ),
        )
        await self._conn.commit()
        cursor = await self._conn.execute(
            "SELECT * FROM gov_declarations WHERE id = ?", (row_id,)
        )
        row = await cursor.fetchone()
        return _dict_with_json_fields(_row_to_dict(row), "metadata")  # type: ignore[arg-type]

    async def get_declaration(self, declaration_id: str) -> Optional[Dict[str, Any]]:
        """Get a single declaration by ID."""
        cursor = await self._conn.execute(
            "SELECT * FROM gov_declarations WHERE id = ?", (declaration_id,)
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def list_declarations_by_politician(
        self, politician_id: str
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            """SELECT * FROM gov_declarations
               WHERE politician_id = ?
               ORDER BY date_publication DESC, created_at DESC""",
            (politician_id,),
        )
        rows = await cursor.fetchall()
        return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]

    async def count_declarations(
        self, politician_id: Optional[str] = None
    ) -> int:
        """Count declarations, optionally filtered by politician."""
        if politician_id:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_declarations WHERE politician_id = ?",
                (politician_id,),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_declarations"
            )
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # Laws
    # ------------------------------------------------------------------

    async def create_law(
        self,
        *,
        title: str,
        uid: Optional[str] = None,
        short_title: Optional[str] = None,
        procedure: Optional[str] = None,
        status: Optional[str] = None,
        initiator_ref: Optional[str] = None,
        date_initial: Optional[str] = None,
        date_promulgation: Optional[str] = None,
        legislature: Optional[str] = None,
        amendments_count: int = 0,
        amendments_adopted: int = 0,
        articles_initial: int = 0,
        articles_final: int = 0,
        duration_days: int = 0,
        source_url: Optional[str] = None,
        jo_url: Optional[str] = None,
        metadata: Optional[Any] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        try:
            await self._conn.execute(
                """INSERT INTO gov_laws
                   (id, uid, title, short_title, procedure, status,
                    initiator_ref, date_initial, date_promulgation, legislature,
                    amendments_count, amendments_adopted, articles_initial,
                    articles_final, duration_days, source_url, jo_url,
                    metadata, created_at)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    row_id, uid, title, short_title, procedure, status,
                    initiator_ref, date_initial, date_promulgation, legislature,
                    amendments_count, amendments_adopted, articles_initial,
                    articles_final, duration_days, source_url, jo_url,
                    _json_dumps(metadata), now,
                ),
            )
            await self._conn.commit()
        except IntegrityError:
            if uid:
                existing = await self.get_law_by_uid(uid)
                if existing:
                    return existing
            raise
        cursor = await self._conn.execute(
            "SELECT * FROM gov_laws WHERE id = ?", (row_id,)
        )
        row = await cursor.fetchone()
        return _dict_with_json_fields(_row_to_dict(row), "metadata")  # type: ignore[arg-type]

    async def get_law(self, law_id: str) -> Optional[Dict[str, Any]]:
        """Get a single law by ID."""
        cursor = await self._conn.execute(
            "SELECT * FROM gov_laws WHERE id = ?", (law_id,)
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def get_law_by_uid(self, uid: str) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute(
            "SELECT * FROM gov_laws WHERE uid = ?", (uid,)
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def list_laws(
        self, *, status: Optional[str] = None, limit: int = 200
    ) -> List[Dict[str, Any]]:
        conditions: List[str] = []
        params: List[Any] = []
        if status is not None:
            conditions.append("status = ?")
            params.append(status)
        where = ("WHERE " + " AND ".join(conditions)) if conditions else ""
        query = f"SELECT * FROM gov_laws {where} ORDER BY date_initial DESC, created_at DESC LIMIT ?"
        params.append(limit)
        cursor = await self._conn.execute(query, params)
        rows = await cursor.fetchall()
        return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]

    async def count_laws(self) -> int:
        """Count total laws."""
        cursor = await self._conn.execute("SELECT COUNT(*) FROM gov_laws")
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # Press
    # ------------------------------------------------------------------

    async def create_press_article(
        self,
        *,
        title: str,
        url: str,
        source_name: Optional[str] = None,
        published_at: Optional[str] = None,
        summary: Optional[str] = None,
        sentiment: Optional[str] = None,
        politicians_mentioned: Optional[str] = None,
        subjects: Optional[str] = None,
        metadata: Optional[Any] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        try:
            await self._conn.execute(
                """INSERT INTO gov_press
                   (id, title, url, source_name, published_at, summary,
                    sentiment, politicians_mentioned, subjects, metadata,
                    created_at)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    row_id, title, url, source_name, published_at, summary,
                    sentiment, politicians_mentioned, subjects,
                    _json_dumps(metadata), now,
                ),
            )
            await self._conn.commit()
        except IntegrityError:
            existing = await self.get_press_article_by_url(url)
            if existing:
                return existing
            raise
        cursor = await self._conn.execute(
            "SELECT * FROM gov_press WHERE id = ?", (row_id,)
        )
        row = await cursor.fetchone()
        return _dict_with_json_fields(_row_to_dict(row), "metadata")  # type: ignore[arg-type]

    async def get_press_article(
        self, article_id: str
    ) -> Optional[Dict[str, Any]]:
        """Get a single press article by ID."""
        cursor = await self._conn.execute(
            "SELECT * FROM gov_press WHERE id = ?", (article_id,)
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def get_press_article_by_url(
        self, url: str
    ) -> Optional[Dict[str, Any]]:
        """Get a press article by its unique URL."""
        cursor = await self._conn.execute(
            "SELECT * FROM gov_press WHERE url = ?", (url,)
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def list_press(
        self, *, limit: int = 200, offset: int = 0
    ) -> List[Dict[str, Any]]:
        """List all press articles with pagination."""
        cursor = await self._conn.execute(
            "SELECT * FROM gov_press ORDER BY published_at DESC, created_at DESC LIMIT ? OFFSET ?",
            (limit, offset),
        )
        rows = await cursor.fetchall()
        return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]

    async def list_press_by_politician(
        self, politician_id: str, limit: int = 100
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            """SELECT * FROM gov_press
               WHERE politicians_mentioned LIKE ?
               ORDER BY published_at DESC, created_at DESC LIMIT ?""",
            (f"%{politician_id}%", limit),
        )
        rows = await cursor.fetchall()
        return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]

    async def count_press(self) -> int:
        """Count total press articles."""
        cursor = await self._conn.execute("SELECT COUNT(*) FROM gov_press")
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # Social posts
    # ------------------------------------------------------------------

    async def create_social_post(
        self,
        *,
        politician_id: str,
        platform: str,
        content: Optional[str] = None,
        post_id: Optional[str] = None,
        url: Optional[str] = None,
        media_type: Optional[str] = None,
        media_url: Optional[str] = None,
        posted_at: Optional[str] = None,
        likes: int = 0,
        shares: int = 0,
        comments: int = 0,
        metadata: Optional[Any] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        try:
            await self._conn.execute(
                """INSERT INTO gov_social_posts
                   (id, politician_id, platform, post_id, content, url,
                    media_type, media_url, posted_at, likes, shares, comments,
                    metadata, created_at)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    row_id, politician_id, platform, post_id, content, url,
                    media_type, media_url, posted_at, likes, shares, comments,
                    _json_dumps(metadata), now,
                ),
            )
            await self._conn.commit()
        except IntegrityError:
            if post_id:
                existing = await self.get_social_post_by_platform_id(
                    platform, post_id
                )
                if existing:
                    return existing
            raise
        cursor = await self._conn.execute(
            "SELECT * FROM gov_social_posts WHERE id = ?", (row_id,)
        )
        row = await cursor.fetchone()
        return _dict_with_json_fields(_row_to_dict(row), "metadata")  # type: ignore[arg-type]

    async def get_social_post(
        self, social_post_id: str
    ) -> Optional[Dict[str, Any]]:
        """Get a single social post by ID."""
        cursor = await self._conn.execute(
            "SELECT * FROM gov_social_posts WHERE id = ?", (social_post_id,)
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def get_social_post_by_platform_id(
        self, platform: str, post_id: str
    ) -> Optional[Dict[str, Any]]:
        """Get a social post by its unique (platform, post_id) pair."""
        cursor = await self._conn.execute(
            "SELECT * FROM gov_social_posts WHERE platform = ? AND post_id = ?",
            (platform, post_id),
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def list_social_by_politician(
        self,
        politician_id: str,
        *,
        platform: Optional[str] = None,
        limit: int = 100,
    ) -> List[Dict[str, Any]]:
        conditions = ["politician_id = ?"]
        params: List[Any] = [politician_id]
        if platform is not None:
            conditions.append("platform = ?")
            params.append(platform)
        where = "WHERE " + " AND ".join(conditions)
        query = f"SELECT * FROM gov_social_posts {where} ORDER BY posted_at DESC, created_at DESC LIMIT ?"
        params.append(limit)
        cursor = await self._conn.execute(query, params)
        rows = await cursor.fetchall()
        return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]

    async def count_social_posts(
        self, politician_id: Optional[str] = None
    ) -> int:
        """Count social posts, optionally filtered by politician."""
        if politician_id:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_social_posts WHERE politician_id = ?",
                (politician_id,),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_social_posts"
            )
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # Transcriptions
    # ------------------------------------------------------------------

    async def create_transcription(
        self,
        *,
        source_type: str,
        source_url: Optional[str] = None,
        politician_id: Optional[str] = None,
        title: Optional[str] = None,
        transcription: Optional[str] = None,
        timestamped_text: Optional[str] = None,
        duration_seconds: Optional[int] = None,
        language: str = "fr",
        model_used: Optional[str] = None,
        metadata: Optional[Any] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO gov_transcriptions
               (id, source_type, source_url, politician_id, title,
                transcription, timestamped_text, duration_seconds,
                language, model_used, metadata, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, source_type, source_url, politician_id, title,
                transcription, timestamped_text, duration_seconds,
                language, model_used, _json_dumps(metadata), now,
            ),
        )
        await self._conn.commit()
        cursor = await self._conn.execute(
            "SELECT * FROM gov_transcriptions WHERE id = ?", (row_id,)
        )
        row = await cursor.fetchone()
        return _dict_with_json_fields(_row_to_dict(row), "metadata")  # type: ignore[arg-type]

    async def get_transcription(
        self, transcription_id: str
    ) -> Optional[Dict[str, Any]]:
        """Get a single transcription by ID."""
        cursor = await self._conn.execute(
            "SELECT * FROM gov_transcriptions WHERE id = ?", (transcription_id,)
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def list_transcriptions_by_politician(
        self, politician_id: str, limit: int = 50
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            """SELECT * FROM gov_transcriptions
               WHERE politician_id = ?
               ORDER BY created_at DESC LIMIT ?""",
            (politician_id, limit),
        )
        rows = await cursor.fetchall()
        return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]

    async def count_transcriptions(
        self, politician_id: Optional[str] = None
    ) -> int:
        """Count transcriptions, optionally filtered by politician."""
        if politician_id:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_transcriptions WHERE politician_id = ?",
                (politician_id,),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_transcriptions"
            )
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # Factchecks
    # ------------------------------------------------------------------

    async def create_factcheck(
        self,
        *,
        claim: str,
        claim_date: Optional[str] = None,
        claimant: Optional[str] = None,
        politician_id: Optional[str] = None,
        rating: Optional[str] = None,
        review_url: Optional[str] = None,
        reviewer: Optional[str] = None,
        review_date: Optional[str] = None,
        metadata: Optional[Any] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO gov_factchecks
               (id, claim, claim_date, claimant, politician_id, rating,
                review_url, reviewer, review_date, metadata, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, claim, claim_date, claimant, politician_id, rating,
                review_url, reviewer, review_date,
                _json_dumps(metadata), now,
            ),
        )
        await self._conn.commit()
        cursor = await self._conn.execute(
            "SELECT * FROM gov_factchecks WHERE id = ?", (row_id,)
        )
        row = await cursor.fetchone()
        return _dict_with_json_fields(_row_to_dict(row), "metadata")  # type: ignore[arg-type]

    async def get_factcheck(
        self, factcheck_id: str
    ) -> Optional[Dict[str, Any]]:
        """Get a single factcheck by ID."""
        cursor = await self._conn.execute(
            "SELECT * FROM gov_factchecks WHERE id = ?", (factcheck_id,)
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def list_factchecks_by_politician(
        self, politician_id: str, limit: int = 50
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            """SELECT * FROM gov_factchecks
               WHERE politician_id = ?
               ORDER BY claim_date DESC, created_at DESC LIMIT ?""",
            (politician_id, limit),
        )
        rows = await cursor.fetchall()
        return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]

    async def count_factchecks(
        self, politician_id: Optional[str] = None
    ) -> int:
        """Count factchecks, optionally filtered by politician."""
        if politician_id:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_factchecks WHERE politician_id = ?",
                (politician_id,),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_factchecks"
            )
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # External IDs
    # ------------------------------------------------------------------

    async def create_external_id(
        self,
        *,
        politician_id: str,
        source: str,
        external_id: str,
        confidence: float = 1.0,
        metadata: Optional[Any] = None,
    ) -> Dict[str, Any]:
        row_id = _new_id()
        now = _now_iso()
        try:
            await self._conn.execute(
                """INSERT INTO gov_external_ids
                   (id, politician_id, source, external_id, confidence,
                    metadata, created_at)
                   VALUES (?, ?, ?, ?, ?, ?, ?)""",
                (
                    row_id, politician_id, source, external_id, confidence,
                    _json_dumps(metadata), now,
                ),
            )
            await self._conn.commit()
        except IntegrityError:
            existing = await self.get_external_id_by_source(source, external_id)
            if existing:
                return existing
            raise
        cursor = await self._conn.execute(
            "SELECT * FROM gov_external_ids WHERE id = ?", (row_id,)
        )
        row = await cursor.fetchone()
        return _dict_with_json_fields(_row_to_dict(row), "metadata")  # type: ignore[arg-type]

    async def get_external_id(
        self, ext_id: str
    ) -> Optional[Dict[str, Any]]:
        """Get a single external ID entry by its row ID."""
        cursor = await self._conn.execute(
            "SELECT * FROM gov_external_ids WHERE id = ?", (ext_id,)
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def get_external_id_by_source(
        self, source: str, external_id: str
    ) -> Optional[Dict[str, Any]]:
        """Get an external ID entry by its unique (source, external_id) pair."""
        cursor = await self._conn.execute(
            "SELECT * FROM gov_external_ids WHERE source = ? AND external_id = ?",
            (source, external_id),
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        return _dict_with_json_fields(_row_to_dict(row), "metadata")

    async def get_external_ids_by_politician(
        self, politician_id: str
    ) -> List[Dict[str, Any]]:
        cursor = await self._conn.execute(
            """SELECT * FROM gov_external_ids
               WHERE politician_id = ?
               ORDER BY source ASC""",
            (politician_id,),
        )
        rows = await cursor.fetchall()
        return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]

    async def find_politician_by_external_id(
        self, source: str, external_id: str
    ) -> Optional[Dict[str, Any]]:
        cursor = await self._conn.execute(
            """SELECT p.* FROM gov_politicians p
               JOIN gov_external_ids e ON e.politician_id = p.id
               WHERE e.source = ? AND e.external_id = ?""",
            (source, external_id),
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        d = _row_to_dict(row)
        d["active"] = bool(d.get("active", 1))
        return _dict_with_json_fields(d, "metadata")

    async def count_external_ids(
        self, politician_id: Optional[str] = None
    ) -> int:
        """Count external IDs, optionally filtered by politician."""
        if politician_id:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_external_ids WHERE politician_id = ?",
                (politician_id,),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_external_ids"
            )
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # Alerts
    # ------------------------------------------------------------------

    async def create_alert(
        self,
        *,
        alert_type: str,
        title: str,
        description: Optional[str] = None,
        severity: str = "info",
        politician_id: Optional[str] = None,
        event_id: Optional[str] = None,
        is_read: bool = False,
    ) -> Dict[str, Any]:
        """Create a new alert."""
        row_id = _new_id()
        now = _now_iso()
        await self._conn.execute(
            """INSERT INTO gov_alerts
               (id, alert_type, title, description, severity,
                politician_id, event_id, is_read, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                row_id, alert_type, title, description, severity,
                politician_id, event_id, int(is_read), now,
            ),
        )
        await self._conn.commit()
        return await self.get_alert(row_id)  # type: ignore[return-value]

    async def get_alert(self, alert_id: str) -> Optional[Dict[str, Any]]:
        """Get a single alert by ID."""
        cursor = await self._conn.execute(
            "SELECT * FROM gov_alerts WHERE id = ?", (alert_id,)
        )
        row = await cursor.fetchone()
        if row is None:
            return None
        d = _row_to_dict(row)
        d["is_read"] = bool(d.get("is_read", 0))
        return d

    async def list_alerts(
        self,
        *,
        alert_type: Optional[str] = None,
        severity: Optional[str] = None,
        is_read: Optional[bool] = None,
        politician_id: Optional[str] = None,
        limit: int = 200,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        """List alerts with optional filters."""
        conditions: List[str] = []
        params: List[Any] = []
        if alert_type is not None:
            conditions.append("alert_type = ?")
            params.append(alert_type)
        if severity is not None:
            conditions.append("severity = ?")
            params.append(severity)
        if is_read is not None:
            conditions.append("is_read = ?")
            params.append(int(is_read))
        if politician_id is not None:
            conditions.append("politician_id = ?")
            params.append(politician_id)
        where = ("WHERE " + " AND ".join(conditions)) if conditions else ""
        query = f"SELECT * FROM gov_alerts {where} ORDER BY created_at DESC LIMIT ? OFFSET ?"
        params.extend([limit, offset])
        cursor = await self._conn.execute(query, params)
        rows = await cursor.fetchall()
        result = []
        for r in rows:
            d = _row_to_dict(r)
            d["is_read"] = bool(d.get("is_read", 0))
            result.append(d)
        return result

    async def mark_alert_read(
        self, alert_id: str, *, is_read: bool = True
    ) -> Optional[Dict[str, Any]]:
        """Mark an alert as read (or unread)."""
        await self._conn.execute(
            "UPDATE gov_alerts SET is_read = ? WHERE id = ?",
            (int(is_read), alert_id),
        )
        await self._conn.commit()
        return await self.get_alert(alert_id)

    async def count_alerts(
        self, *, is_read: Optional[bool] = None
    ) -> int:
        """Count alerts, optionally filtered by read status."""
        if is_read is not None:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_alerts WHERE is_read = ?",
                (int(is_read),),
            )
        else:
            cursor = await self._conn.execute(
                "SELECT COUNT(*) FROM gov_alerts"
            )
        row = await cursor.fetchone()
        return row[0] if row else 0

    # ------------------------------------------------------------------
    # Stats
    # ------------------------------------------------------------------

    async def get_stats(self) -> Dict[str, Any]:
        """Return aggregate counts for all 16 tables + last scan timestamp."""
        tables = [
            "gov_politicians", "gov_positions", "gov_contradictions",
            "gov_scan_log", "gov_mandates", "gov_parties",
            "gov_party_memberships", "gov_affairs", "gov_declarations",
            "gov_laws", "gov_press", "gov_social_posts",
            "gov_transcriptions", "gov_factchecks", "gov_external_ids",
            "gov_alerts",
        ]
        counts: Dict[str, int] = {}
        for table in tables:
            cursor = await self._conn.execute(f"SELECT COUNT(*) FROM {table}")
            row = await cursor.fetchone()
            # Strip "gov_" prefix for cleaner keys
            key = table.replace("gov_", "")
            counts[key] = row[0] if row else 0

        scan_cursor = await self._conn.execute(
            "SELECT started_at FROM gov_scan_log ORDER BY started_at DESC LIMIT 1"
        )
        scan_row = await scan_cursor.fetchone()
        last_scan = scan_row[0] if scan_row else None

        # Keep legacy keys for backward compatibility
        return {
            "politicians": counts["politicians"],
            "positions": counts["positions"],
            "contradictions": counts["contradictions"],
            "mandates": counts["mandates"],
            "parties": counts["parties"],
            "party_memberships": counts["party_memberships"],
            "affairs": counts["affairs"],
            "declarations": counts["declarations"],
            "laws": counts["laws"],
            "press": counts["press"],
            "social_posts": counts["social_posts"],
            "transcriptions": counts["transcriptions"],
            "factchecks": counts["factchecks"],
            "external_ids": counts["external_ids"],
            "scan_logs": counts["scan_log"],
            "alerts": counts["alerts"],
            "last_scan": last_scan,
        }

    # ------------------------------------------------------------------
    # Subjects (distinct)
    # ------------------------------------------------------------------

    async def get_subjects(self) -> List[str]:
        """Return all distinct subjects from gov_positions."""
        cursor = await self._conn.execute(
            "SELECT DISTINCT subject FROM gov_positions ORDER BY subject ASC"
        )
        rows = await cursor.fetchall()
        return [r[0] for r in rows]

    # ------------------------------------------------------------------
    # Graph data
    # ------------------------------------------------------------------

    async def get_graph_data(
        self,
        *,
        chamber: Optional[str] = None,
        min_positions: int = 0,
    ) -> Dict[str, Any]:
        """Build a relationship graph of politicians.

        Returns ``{"nodes": [...], "edges": [...]}`` with 3 edge types:
        opposition (same subject, different stance), agreement (same
        subject, same stance), and party (same party membership).

        Edges are deduplicated per source-target pair, keeping the most
        significant type (opposition > agreement > party).
        """
        # -- Fetch active politicians (with optional chamber filter) ---
        conditions = ["p.active = 1"]
        params: List[Any] = []
        if chamber is not None:
            conditions.append("p.chamber = ?")
            params.append(chamber)
        where = "WHERE " + " AND ".join(conditions)

        # Aggregate position and contradiction counts in one query
        cursor = await self._conn.execute(
            f"""SELECT p.*,
                       COALESCE(pos_c.cnt, 0) AS position_count,
                       COALESCE(con_c.cnt, 0) AS contradiction_count
                FROM gov_politicians p
                LEFT JOIN (
                    SELECT politician_id, COUNT(*) AS cnt
                    FROM gov_positions GROUP BY politician_id
                ) pos_c ON pos_c.politician_id = p.id
                LEFT JOIN (
                    SELECT politician_id, COUNT(*) AS cnt
                    FROM gov_contradictions GROUP BY politician_id
                ) con_c ON con_c.politician_id = p.id
                {where}
                ORDER BY p.name ASC""",
            params,
        )
        pol_rows = await cursor.fetchall()

        # Build node list (apply min_positions filter in Python since
        # the count comes from a LEFT JOIN)
        nodes: List[Dict[str, Any]] = []
        node_ids: set[str] = set()
        for row in pol_rows:
            d = _row_to_dict(row)
            pos_count = d.pop("position_count", 0)
            con_count = d.pop("contradiction_count", 0)
            if min_positions > 0 and pos_count < min_positions:
                continue
            nodes.append({
                "id": d["id"],
                "label": d["name"],
                "party": d["party"],
                "chamber": d["chamber"],
                "role": d["role"],
                "photo_url": d["photo_url"],
                "position_count": pos_count,
                "contradiction_count": con_count,
            })
            node_ids.add(d["id"])

        if not node_ids:
            return {"nodes": [], "edges": []}

        # -- Build edges -----------------------------------------------
        # We collect edges keyed by (source, target) with a priority
        # so that we keep only the most significant type per pair.
        #   opposition=0  >  agreement=1  >  party=2
        edge_priority = {"opposition": 0, "agreement": 1, "party": 2}
        # key: (source, target) -> (priority, edge_dict)
        best_edges: Dict[tuple, tuple] = {}

        def _maybe_add(src: str, tgt: str, edge: Dict[str, Any], etype: str) -> None:
            """Keep only the highest-priority edge per pair."""
            if src not in node_ids or tgt not in node_ids:
                return
            key = (min(src, tgt), max(src, tgt))
            pri = edge_priority[etype]
            if key not in best_edges or pri < best_edges[key][0]:
                best_edges[key] = (pri, edge)

        # 1) Opposition edges (same subject, different stances)
        cursor = await self._conn.execute(
            """SELECT DISTINCT p1.politician_id AS source,
                              p2.politician_id AS target,
                              p1.subject,
                              p1.stance AS stance_a,
                              p2.stance AS stance_b
               FROM gov_positions p1
               JOIN gov_positions p2
                 ON p1.subject = p2.subject
                AND p1.politician_id < p2.politician_id
                AND p1.stance != p2.stance
                AND p1.stance IS NOT NULL
                AND p2.stance IS NOT NULL"""
        )
        for row in await cursor.fetchall():
            r = _row_to_dict(row)
            _maybe_add(r["source"], r["target"], {
                "id": str(uuid.uuid4()),
                "source": r["source"],
                "target": r["target"],
                "type": "opposition",
                "label": r["subject"],
                "stance_a": r["stance_a"],
                "stance_b": r["stance_b"],
            }, "opposition")

        # 2) Agreement edges (same subject, same stance)
        cursor = await self._conn.execute(
            """SELECT DISTINCT p1.politician_id AS source,
                              p2.politician_id AS target,
                              p1.subject
               FROM gov_positions p1
               JOIN gov_positions p2
                 ON p1.subject = p2.subject
                AND p1.politician_id < p2.politician_id
                AND p1.stance = p2.stance
                AND p1.stance IS NOT NULL
                AND p2.stance IS NOT NULL"""
        )
        for row in await cursor.fetchall():
            r = _row_to_dict(row)
            _maybe_add(r["source"], r["target"], {
                "id": str(uuid.uuid4()),
                "source": r["source"],
                "target": r["target"],
                "type": "agreement",
                "label": r["subject"],
            }, "agreement")

        # 3) Party edges (same party)
        party_groups: Dict[str, List[str]] = defaultdict(list)
        for node in nodes:
            if node["party"]:
                party_groups[node["party"]].append(node["id"])
        for party_name, members in party_groups.items():
            for a, b in combinations(sorted(members), 2):
                _maybe_add(a, b, {
                    "id": str(uuid.uuid4()),
                    "source": a,
                    "target": b,
                    "type": "party",
                    "label": party_name,
                }, "party")

        edges = [edge for _, edge in best_edges.values()]
        return {"nodes": nodes, "edges": edges}

    async def get_politician_connections(
        self, politician_id: str
    ) -> Dict[str, Any]:
        """Return the ego network (1-hop) centered on *politician_id*.

        Same structure as :meth:`get_graph_data` but limited to the
        target politician and its direct neighbours.
        """
        full = await self.get_graph_data()

        # Find the focal node
        node_map = {n["id"]: n for n in full["nodes"]}
        if politician_id not in node_map:
            return {"nodes": [], "edges": []}

        # Collect 1-hop neighbour IDs from edges
        neighbour_ids: set[str] = {politician_id}
        relevant_edges: List[Dict[str, Any]] = []
        for edge in full["edges"]:
            if edge["source"] == politician_id or edge["target"] == politician_id:
                relevant_edges.append(edge)
                neighbour_ids.add(edge["source"])
                neighbour_ids.add(edge["target"])

        relevant_nodes = [node_map[nid] for nid in neighbour_ids if nid in node_map]
        return {"nodes": relevant_nodes, "edges": relevant_edges}

    async def get_subject_graph(self, subject: str) -> Dict[str, Any]:
        """Return all politicians who took a position on *subject*.

        Edges show agreement or opposition between them on this subject.
        """
        # Fetch politicians who have positions on this subject
        cursor = await self._conn.execute(
            """SELECT DISTINCT pol.*,
                      COALESCE(pos_c.cnt, 0) AS position_count,
                      COALESCE(con_c.cnt, 0) AS contradiction_count
               FROM gov_politicians pol
               JOIN gov_positions gp ON gp.politician_id = pol.id
               LEFT JOIN (
                   SELECT politician_id, COUNT(*) AS cnt
                   FROM gov_positions GROUP BY politician_id
               ) pos_c ON pos_c.politician_id = pol.id
               LEFT JOIN (
                   SELECT politician_id, COUNT(*) AS cnt
                   FROM gov_contradictions GROUP BY politician_id
               ) con_c ON con_c.politician_id = pol.id
               WHERE gp.subject = ? AND pol.active = 1
               ORDER BY pol.name ASC""",
            (subject,),
        )
        pol_rows = await cursor.fetchall()

        nodes: List[Dict[str, Any]] = []
        node_ids: set[str] = set()
        for row in pol_rows:
            d = _row_to_dict(row)
            pos_count = d.pop("position_count", 0)
            con_count = d.pop("contradiction_count", 0)
            nodes.append({
                "id": d["id"],
                "label": d["name"],
                "party": d["party"],
                "chamber": d["chamber"],
                "role": d["role"],
                "photo_url": d["photo_url"],
                "position_count": pos_count,
                "contradiction_count": con_count,
            })
            node_ids.add(d["id"])

        if not node_ids:
            return {"nodes": [], "edges": []}

        edges: List[Dict[str, Any]] = []
        seen: set[tuple] = set()

        # Opposition edges for this subject
        cursor = await self._conn.execute(
            """SELECT DISTINCT p1.politician_id AS source,
                              p2.politician_id AS target,
                              p1.stance AS stance_a,
                              p2.stance AS stance_b
               FROM gov_positions p1
               JOIN gov_positions p2
                 ON p1.subject = p2.subject
                AND p1.politician_id < p2.politician_id
                AND p1.stance != p2.stance
                AND p1.stance IS NOT NULL
                AND p2.stance IS NOT NULL
               WHERE p1.subject = ?""",
            (subject,),
        )
        for row in await cursor.fetchall():
            r = _row_to_dict(row)
            if r["source"] in node_ids and r["target"] in node_ids:
                key = (r["source"], r["target"])
                if key not in seen:
                    seen.add(key)
                    edges.append({
                        "id": str(uuid.uuid4()),
                        "source": r["source"],
                        "target": r["target"],
                        "type": "opposition",
                        "label": subject,
                        "stance_a": r["stance_a"],
                        "stance_b": r["stance_b"],
                    })

        # Agreement edges for this subject
        cursor = await self._conn.execute(
            """SELECT DISTINCT p1.politician_id AS source,
                              p2.politician_id AS target
               FROM gov_positions p1
               JOIN gov_positions p2
                 ON p1.subject = p2.subject
                AND p1.politician_id < p2.politician_id
                AND p1.stance = p2.stance
                AND p1.stance IS NOT NULL
                AND p2.stance IS NOT NULL
               WHERE p1.subject = ?""",
            (subject,),
        )
        for row in await cursor.fetchall():
            r = _row_to_dict(row)
            if r["source"] in node_ids and r["target"] in node_ids:
                key = (r["source"], r["target"])
                if key not in seen:
                    seen.add(key)
                    edges.append({
                        "id": str(uuid.uuid4()),
                        "source": r["source"],
                        "target": r["target"],
                        "type": "agreement",
                        "label": subject,
                    })

        return {"nodes": nodes, "edges": edges}

    # ------------------------------------------------------------------
    # Batch inserts
    # ------------------------------------------------------------------

    async def batch_create_positions(self, positions: list[dict]) -> int:
        """Insert multiple positions in one transaction. Returns count inserted."""
        if not positions:
            return 0
        count = 0
        for pos in positions:
            try:
                await self.create_position(**pos)
                count += 1
            except Exception:
                logger.debug("batch_create_positions: skipped one (duplicate or error)")
        return count

    async def batch_create_social_posts(self, posts: list[dict]) -> int:
        """Insert multiple social posts in one transaction. Returns count inserted."""
        if not posts:
            return 0
        count = 0
        for post in posts:
            try:
                await self.create_social_post(**post)
                count += 1
            except Exception:
                logger.debug("batch_create_social_posts: skipped one (duplicate or error)")
        return count

    async def batch_create_press_articles(self, articles: list[dict]) -> int:
        """Insert multiple press articles in one transaction. Returns count inserted."""
        if not articles:
            return 0
        count = 0
        for article in articles:
            try:
                await self.create_press_article(**article)
                count += 1
            except Exception:
                logger.debug("batch_create_press_articles: skipped one (duplicate or error)")
        return count
