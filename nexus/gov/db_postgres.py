"""
NEXUS GOV -- PostgreSQL Database Backend.

Drop-in replacement for GovernmentDatabase (SQLite) using asyncpg.
Activated when GOV_DATABASE_URL is set in config.
Supports pgvector for embeddings (replaces ChromaDB for gov module).
"""
from __future__ import annotations

import json
import re
import uuid
from collections import defaultdict
from datetime import datetime, timezone
from itertools import combinations
from typing import Any, Dict, List, Optional

from loguru import logger

try:
    import asyncpg
except ImportError:
    asyncpg = None


# ============================================================================
# Helpers
# ============================================================================

def _new_id() -> str:
    return str(uuid.uuid4())


def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def _slugify(name: str) -> str:
    """Turn a politician name into a URL-safe slug."""
    slug = name.lower().strip()
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


def _json_dumps(obj: Any) -> str:
    """Serialize to JSON string (null-safe)."""
    if obj is None:
        return "{}"
    if isinstance(obj, str):
        return obj
    return json.dumps(obj, ensure_ascii=False, default=str)


def _json_loads(val: Any) -> Any:
    """Deserialize JSON string (null-safe). asyncpg returns JSONB as dicts already."""
    if val is None:
        return {}
    if isinstance(val, (dict, list)):
        return val
    try:
        return json.loads(val)
    except (json.JSONDecodeError, TypeError):
        return {}


def _row_to_dict(row: Any) -> Dict[str, Any]:
    """Convert asyncpg Record to dict."""
    if row is None:
        return {}
    return dict(row)


def _dict_with_json_fields(d: Dict[str, Any], *fields: str) -> Dict[str, Any]:
    """Parse JSON string fields in a dict into Python objects."""
    for f in fields:
        if f in d:
            d[f] = _json_loads(d[f])
    return d


# ============================================================================
# DDL for PostgreSQL (all 16 tables)
# ============================================================================

PG_SCHEMA = """
CREATE EXTENSION IF NOT EXISTS vector;

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
    active BOOLEAN DEFAULT TRUE,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS gov_positions (
    id TEXT PRIMARY KEY,
    politician_id TEXT NOT NULL REFERENCES gov_politicians(id),
    subject TEXT NOT NULL,
    position_type TEXT NOT NULL,
    position_text TEXT NOT NULL DEFAULT '',
    stance TEXT,
    source_url TEXT NOT NULL DEFAULT '',
    source_type TEXT,
    date DATE,
    session TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    embedding vector(768),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS gov_contradictions (
    id TEXT PRIMARY KEY,
    politician_id TEXT NOT NULL REFERENCES gov_politicians(id),
    position_a_id TEXT NOT NULL,
    position_b_id TEXT NOT NULL,
    subject TEXT NOT NULL,
    description TEXT NOT NULL,
    severity TEXT DEFAULT 'medium',
    source_verified BOOLEAN DEFAULT FALSE,
    metadata JSONB DEFAULT '{}'::jsonb,
    detected_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS gov_scan_log (
    id TEXT PRIMARY KEY,
    scan_type TEXT NOT NULL,
    status TEXT DEFAULT 'running',
    items_found INTEGER DEFAULT 0,
    items_new INTEGER DEFAULT 0,
    error_message TEXT,
    started_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ
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
    is_current BOOLEAN DEFAULT FALSE,
    parliamentary_group TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS gov_parties (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    short_name TEXT UNIQUE,
    color TEXT,
    description TEXT,
    leader TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS gov_party_memberships (
    id TEXT PRIMARY KEY,
    politician_id TEXT NOT NULL REFERENCES gov_politicians(id),
    party_id TEXT NOT NULL REFERENCES gov_parties(id),
    start_date DATE,
    end_date DATE,
    is_current BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
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
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
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
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
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
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS gov_press (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    url TEXT UNIQUE,
    source_name TEXT,
    published_at TIMESTAMPTZ,
    summary TEXT,
    sentiment TEXT,
    politicians_mentioned JSONB,
    subjects JSONB,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
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
    posted_at TIMESTAMPTZ,
    likes INTEGER DEFAULT 0,
    shares INTEGER DEFAULT 0,
    comments INTEGER DEFAULT 0,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(platform, post_id)
);

CREATE TABLE IF NOT EXISTS gov_transcriptions (
    id TEXT PRIMARY KEY,
    source_type TEXT NOT NULL,
    source_url TEXT,
    politician_id TEXT REFERENCES gov_politicians(id),
    title TEXT,
    transcription TEXT,
    timestamped_text JSONB,
    duration_seconds INTEGER,
    language TEXT DEFAULT 'fr',
    model_used TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
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
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS gov_external_ids (
    id TEXT PRIMARY KEY,
    politician_id TEXT NOT NULL REFERENCES gov_politicians(id),
    source TEXT NOT NULL,
    external_id TEXT NOT NULL,
    confidence REAL DEFAULT 1.0,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW(),
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
    is_read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_gov_politicians_chamber ON gov_politicians(chamber);
CREATE INDEX IF NOT EXISTS idx_gov_politicians_party ON gov_politicians(party);
CREATE INDEX IF NOT EXISTS idx_gov_politicians_slug ON gov_politicians(slug);
CREATE INDEX IF NOT EXISTS idx_gov_positions_politician ON gov_positions(politician_id);
CREATE INDEX IF NOT EXISTS idx_gov_positions_date ON gov_positions(date);
CREATE INDEX IF NOT EXISTS idx_gov_positions_subject ON gov_positions(subject);
CREATE INDEX IF NOT EXISTS idx_gov_positions_type ON gov_positions(position_type);
CREATE INDEX IF NOT EXISTS idx_gov_contradictions_politician ON gov_contradictions(politician_id);
CREATE INDEX IF NOT EXISTS idx_gov_contradictions_subject ON gov_contradictions(subject);
CREATE INDEX IF NOT EXISTS idx_gov_scan_log_type ON gov_scan_log(scan_type);
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


# ============================================================================
# PostgreSQL Database class
# ============================================================================

class PostgresGovernmentDatabase:
    """PostgreSQL backend for NEXUS GOV.

    Drop-in replacement for GovernmentDatabase (SQLite). Same public
    interface, backed by asyncpg connection pool.

    Usage::

        pool = await init_postgres("postgresql://user:pass@host/nexusgov")
        gov = PostgresGovernmentDatabase(pool)
        pol = await gov.create_politician(name="Jean Dupont", chamber="assemblee")
    """

    def __init__(self, pool: "asyncpg.Pool") -> None:
        self._pool = pool

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    async def _fetchrow(self, query: str, *args: Any) -> Optional[Dict[str, Any]]:
        async with self._pool.acquire() as conn:
            row = await conn.fetchrow(query, *args)
            return dict(row) if row else None

    async def _fetch(self, query: str, *args: Any) -> List[Dict[str, Any]]:
        async with self._pool.acquire() as conn:
            rows = await conn.fetch(query, *args)
            return [dict(r) for r in rows]

    async def _execute(self, query: str, *args: Any) -> str:
        async with self._pool.acquire() as conn:
            return await conn.execute(query, *args)

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
        slug = _slugify(name)
        await self._execute(
            """INSERT INTO gov_politicians
               (id, name, slug, chamber, party, role, constituency,
                photo_url, official_url, hatvp_url, active, metadata)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)""",
            row_id, name, slug, chamber, party, role, constituency,
            photo_url, official_url, hatvp_url, active,
            json.dumps(metadata or {}, ensure_ascii=False, default=str),
        )
        return await self.get_politician(row_id) or {"id": row_id, "name": name}

    async def get_politician(self, politician_id: str) -> Optional[Dict[str, Any]]:
        row = await self._fetchrow(
            "SELECT * FROM gov_politicians WHERE id = $1", politician_id
        )
        if row is None:
            return None
        return _dict_with_json_fields(row, "metadata")

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
        idx = 1
        if chamber is not None:
            conditions.append(f"chamber = ${idx}")
            params.append(chamber)
            idx += 1
        if party is not None:
            conditions.append(f"party = ${idx}")
            params.append(party)
            idx += 1
        if active is not None:
            conditions.append(f"active = ${idx}")
            params.append(active)
            idx += 1
        where = ("WHERE " + " AND ".join(conditions)) if conditions else ""
        params.extend([limit, offset])
        query = f"SELECT * FROM gov_politicians {where} ORDER BY name ASC LIMIT ${idx} OFFSET ${idx + 1}"
        rows = await self._fetch(query, *params)
        return [_dict_with_json_fields(r, "metadata") for r in rows]

    async def update_politician(
        self, politician_id: str, **fields: Any
    ) -> Optional[Dict[str, Any]]:
        if not fields:
            return await self.get_politician(politician_id)
        if "name" in fields:
            fields["slug"] = _slugify(fields["name"])
        if "metadata" in fields and isinstance(fields["metadata"], dict):
            fields["metadata"] = json.dumps(
                fields["metadata"], ensure_ascii=False, default=str
            )
        if "active" in fields and isinstance(fields["active"], int):
            fields["active"] = bool(fields["active"])
        sets: List[str] = []
        params: List[Any] = []
        idx = 1
        for k, v in fields.items():
            sets.append(f"{k} = ${idx}")
            params.append(v)
            idx += 1
        params.append(politician_id)
        await self._execute(
            f"UPDATE gov_politicians SET {', '.join(sets)}, updated_at = NOW() WHERE id = ${idx}",
            *params,
        )
        return await self.get_politician(politician_id)

    async def delete_politician(self, politician_id: str) -> bool:
        """Delete a politician and all dependent rows."""
        async with self._pool.acquire() as conn:
            async with conn.transaction():
                await conn.execute(
                    "DELETE FROM gov_contradictions WHERE politician_id = $1",
                    politician_id,
                )
                await conn.execute(
                    "DELETE FROM gov_positions WHERE politician_id = $1",
                    politician_id,
                )
                await conn.execute(
                    "DELETE FROM gov_mandates WHERE politician_id = $1",
                    politician_id,
                )
                await conn.execute(
                    "DELETE FROM gov_party_memberships WHERE politician_id = $1",
                    politician_id,
                )
                await conn.execute(
                    "DELETE FROM gov_affairs WHERE politician_id = $1",
                    politician_id,
                )
                await conn.execute(
                    "DELETE FROM gov_declarations WHERE politician_id = $1",
                    politician_id,
                )
                await conn.execute(
                    "DELETE FROM gov_social_posts WHERE politician_id = $1",
                    politician_id,
                )
                await conn.execute(
                    "DELETE FROM gov_transcriptions WHERE politician_id = $1",
                    politician_id,
                )
                await conn.execute(
                    "DELETE FROM gov_factchecks WHERE politician_id = $1",
                    politician_id,
                )
                await conn.execute(
                    "DELETE FROM gov_external_ids WHERE politician_id = $1",
                    politician_id,
                )
                await conn.execute(
                    "DELETE FROM gov_alerts WHERE politician_id = $1",
                    politician_id,
                )
                result = await conn.execute(
                    "DELETE FROM gov_politicians WHERE id = $1",
                    politician_id,
                )
        return result == "DELETE 1"

    async def search_politicians(self, query: str) -> List[Dict[str, Any]]:
        """Search politicians by name (ILIKE match)."""
        pattern = f"%{query}%"
        rows = await self._fetch(
            "SELECT * FROM gov_politicians WHERE name ILIKE $1 ORDER BY name ASC LIMIT 50",
            pattern,
        )
        return [_dict_with_json_fields(r, "metadata") for r in rows]

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
        await self._execute(
            """INSERT INTO gov_positions
               (id, politician_id, subject, position_type, position_text,
                stance, source_url, source_type, date, session, metadata)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)""",
            row_id, politician_id, subject, position_type, position_text,
            stance, source_url, source_type, date, session,
            json.dumps(metadata or {}, ensure_ascii=False, default=str),
        )
        return await self.get_position(row_id) or {"id": row_id}

    async def get_position(self, position_id: str) -> Optional[Dict[str, Any]]:
        row = await self._fetchrow(
            "SELECT * FROM gov_positions WHERE id = $1", position_id
        )
        if row is None:
            return None
        return _dict_with_json_fields(row, "metadata")

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
        conditions = ["politician_id = $1"]
        params: List[Any] = [politician_id]
        idx = 2
        if position_type is not None:
            conditions.append(f"position_type = ${idx}")
            params.append(position_type)
            idx += 1
        if date_from is not None:
            conditions.append(f"date >= ${idx}")
            params.append(date_from)
            idx += 1
        if date_to is not None:
            conditions.append(f"date <= ${idx}")
            params.append(date_to)
            idx += 1
        where = "WHERE " + " AND ".join(conditions)
        params.extend([limit, offset])
        query = f"SELECT * FROM gov_positions {where} ORDER BY date DESC, created_at DESC LIMIT ${idx} OFFSET ${idx + 1}"
        rows = await self._fetch(query, *params)
        return [_dict_with_json_fields(r, "metadata") for r in rows]

    async def count_positions(self, politician_id: Optional[str] = None) -> int:
        if politician_id:
            row = await self._fetchrow(
                "SELECT COUNT(*) AS cnt FROM gov_positions WHERE politician_id = $1",
                politician_id,
            )
        else:
            row = await self._fetchrow("SELECT COUNT(*) AS cnt FROM gov_positions")
        return row["cnt"] if row else 0

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
        await self._execute(
            """INSERT INTO gov_contradictions
               (id, politician_id, position_a_id, position_b_id,
                subject, description, severity, source_verified, metadata)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)""",
            row_id, politician_id, position_a_id, position_b_id,
            subject, description, severity, source_verified,
            json.dumps(metadata or {}, ensure_ascii=False, default=str),
        )
        return await self.get_contradiction(row_id) or {
            "id": row_id, "politician_id": politician_id,
            "subject": subject, "description": description,
        }

    async def get_contradiction(self, contradiction_id: str) -> Optional[Dict[str, Any]]:
        row = await self._fetchrow(
            "SELECT * FROM gov_contradictions WHERE id = $1", contradiction_id
        )
        if row is None:
            return None
        return _dict_with_json_fields(row, "metadata")

    async def list_contradictions_by_politician(
        self,
        politician_id: str,
        *,
        limit: int = 200,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        rows = await self._fetch(
            """SELECT * FROM gov_contradictions
               WHERE politician_id = $1
               ORDER BY detected_at DESC LIMIT $2 OFFSET $3""",
            politician_id, limit, offset,
        )
        return [_dict_with_json_fields(r, "metadata") for r in rows]

    async def list_all_contradictions(
        self,
        *,
        severity: Optional[str] = None,
        limit: int = 200,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        conditions: List[str] = []
        params: List[Any] = []
        idx = 1
        if severity is not None:
            conditions.append(f"severity = ${idx}")
            params.append(severity)
            idx += 1
        where = ("WHERE " + " AND ".join(conditions)) if conditions else ""
        params.extend([limit, offset])
        query = f"SELECT * FROM gov_contradictions {where} ORDER BY detected_at DESC LIMIT ${idx} OFFSET ${idx + 1}"
        rows = await self._fetch(query, *params)
        return [_dict_with_json_fields(r, "metadata") for r in rows]

    async def count_contradictions(self, politician_id: Optional[str] = None) -> int:
        if politician_id:
            row = await self._fetchrow(
                "SELECT COUNT(*) AS cnt FROM gov_contradictions WHERE politician_id = $1",
                politician_id,
            )
        else:
            row = await self._fetchrow(
                "SELECT COUNT(*) AS cnt FROM gov_contradictions"
            )
        return row["cnt"] if row else 0

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
        await self._execute(
            "INSERT INTO gov_scan_log (id, scan_type, status) VALUES ($1, $2, $3)",
            row_id, scan_type, status,
        )
        return await self._fetchrow(
            "SELECT * FROM gov_scan_log WHERE id = $1", row_id
        ) or {"id": row_id, "scan_type": scan_type, "status": status}

    async def update_scan_log(
        self, scan_id: str, **fields: Any
    ) -> Optional[Dict[str, Any]]:
        if not fields:
            return await self._fetchrow(
                "SELECT * FROM gov_scan_log WHERE id = $1", scan_id
            )
        sets: List[str] = []
        params: List[Any] = []
        idx = 1
        for k, v in fields.items():
            sets.append(f"{k} = ${idx}")
            params.append(v)
            idx += 1
        params.append(scan_id)
        await self._execute(
            f"UPDATE gov_scan_log SET {', '.join(sets)} WHERE id = ${idx}",
            *params,
        )
        return await self._fetchrow(
            "SELECT * FROM gov_scan_log WHERE id = $1", scan_id
        )

    async def list_scan_logs(
        self,
        *,
        limit: int = 50,
        offset: int = 0,
    ) -> List[Dict[str, Any]]:
        return await self._fetch(
            "SELECT * FROM gov_scan_log ORDER BY started_at DESC LIMIT $1 OFFSET $2",
            limit, offset,
        )

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
        await self._execute(
            """INSERT INTO gov_mandates
               (id, politician_id, type, title, institution, constituency,
                start_date, end_date, is_current, parliamentary_group, metadata)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)""",
            row_id, politician_id, type, title, institution, constituency,
            start_date, end_date, is_current, parliamentary_group,
            json.dumps(metadata or {}, ensure_ascii=False, default=str),
        )
        row = await self._fetchrow(
            "SELECT * FROM gov_mandates WHERE id = $1", row_id
        )
        if row is None:
            return {"id": row_id, "politician_id": politician_id}
        return _dict_with_json_fields(row, "metadata")

    async def list_mandates_by_politician(
        self, politician_id: str
    ) -> List[Dict[str, Any]]:
        rows = await self._fetch(
            """SELECT * FROM gov_mandates
               WHERE politician_id = $1
               ORDER BY start_date DESC, created_at DESC""",
            politician_id,
        )
        return [_dict_with_json_fields(r, "metadata") for r in rows]

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
        await self._execute(
            """INSERT INTO gov_parties
               (id, name, short_name, color, description, leader, metadata)
               VALUES ($1,$2,$3,$4,$5,$6,$7)""",
            row_id, name, short_name, color, description, leader,
            json.dumps(metadata or {}, ensure_ascii=False, default=str),
        )
        return await self.get_party(row_id) or {"id": row_id, "name": name}

    async def get_party(self, party_id: str) -> Optional[Dict[str, Any]]:
        row = await self._fetchrow(
            "SELECT * FROM gov_parties WHERE id = $1", party_id
        )
        if row is None:
            return None
        return _dict_with_json_fields(row, "metadata")

    async def get_party_by_short_name(
        self, short_name: str
    ) -> Optional[Dict[str, Any]]:
        row = await self._fetchrow(
            "SELECT * FROM gov_parties WHERE short_name = $1", short_name
        )
        if row is None:
            return None
        return _dict_with_json_fields(row, "metadata")

    async def list_parties(self) -> List[Dict[str, Any]]:
        rows = await self._fetch(
            "SELECT * FROM gov_parties ORDER BY name ASC"
        )
        return [_dict_with_json_fields(r, "metadata") for r in rows]

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
        await self._execute(
            """INSERT INTO gov_party_memberships
               (id, politician_id, party_id, start_date, end_date, is_current)
               VALUES ($1,$2,$3,$4,$5,$6)""",
            row_id, politician_id, party_id, start_date, end_date, is_current,
        )
        row = await self._fetchrow(
            "SELECT * FROM gov_party_memberships WHERE id = $1", row_id
        )
        return row or {"id": row_id, "politician_id": politician_id}

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
        await self._execute(
            """INSERT INTO gov_affairs
               (id, politician_id, title, description, status, category,
                involvement, source_url, date_start, date_end, metadata)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)""",
            row_id, politician_id, title, description, status, category,
            involvement, source_url, date_start, date_end,
            json.dumps(metadata or {}, ensure_ascii=False, default=str),
        )
        row = await self._fetchrow(
            "SELECT * FROM gov_affairs WHERE id = $1", row_id
        )
        if row is None:
            return {"id": row_id, "politician_id": politician_id, "title": title}
        return _dict_with_json_fields(row, "metadata")

    async def list_affairs_by_politician(
        self, politician_id: str
    ) -> List[Dict[str, Any]]:
        rows = await self._fetch(
            """SELECT * FROM gov_affairs
               WHERE politician_id = $1
               ORDER BY date_start DESC, created_at DESC""",
            politician_id,
        )
        return [_dict_with_json_fields(r, "metadata") for r in rows]

    async def count_affairs(self) -> int:
        row = await self._fetchrow("SELECT COUNT(*) AS cnt FROM gov_affairs")
        return row["cnt"] if row else 0

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
        await self._execute(
            """INSERT INTO gov_declarations
               (id, politician_id, type, qualite, departement,
                date_publication, date_depot, url, status, metadata)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)""",
            row_id, politician_id, type, qualite, departement,
            date_publication, date_depot, url, status,
            json.dumps(metadata or {}, ensure_ascii=False, default=str),
        )
        row = await self._fetchrow(
            "SELECT * FROM gov_declarations WHERE id = $1", row_id
        )
        if row is None:
            return {"id": row_id, "politician_id": politician_id}
        return _dict_with_json_fields(row, "metadata")

    async def list_declarations_by_politician(
        self, politician_id: str
    ) -> List[Dict[str, Any]]:
        rows = await self._fetch(
            """SELECT * FROM gov_declarations
               WHERE politician_id = $1
               ORDER BY date_publication DESC, created_at DESC""",
            politician_id,
        )
        return [_dict_with_json_fields(r, "metadata") for r in rows]

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
        await self._execute(
            """INSERT INTO gov_laws
               (id, uid, title, short_title, procedure, status,
                initiator_ref, date_initial, date_promulgation, legislature,
                amendments_count, amendments_adopted, articles_initial,
                articles_final, duration_days, source_url, jo_url, metadata)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)""",
            row_id, uid, title, short_title, procedure, status,
            initiator_ref, date_initial, date_promulgation, legislature,
            amendments_count, amendments_adopted, articles_initial,
            articles_final, duration_days, source_url, jo_url,
            json.dumps(metadata or {}, ensure_ascii=False, default=str),
        )
        row = await self._fetchrow(
            "SELECT * FROM gov_laws WHERE id = $1", row_id
        )
        if row is None:
            return {"id": row_id, "title": title}
        return _dict_with_json_fields(row, "metadata")

    async def get_law_by_uid(self, uid: str) -> Optional[Dict[str, Any]]:
        row = await self._fetchrow(
            "SELECT * FROM gov_laws WHERE uid = $1", uid
        )
        if row is None:
            return None
        return _dict_with_json_fields(row, "metadata")

    async def list_laws(
        self, *, status: Optional[str] = None, limit: int = 200
    ) -> List[Dict[str, Any]]:
        conditions: List[str] = []
        params: List[Any] = []
        idx = 1
        if status is not None:
            conditions.append(f"status = ${idx}")
            params.append(status)
            idx += 1
        where = ("WHERE " + " AND ".join(conditions)) if conditions else ""
        params.append(limit)
        query = f"SELECT * FROM gov_laws {where} ORDER BY date_initial DESC, created_at DESC LIMIT ${idx}"
        rows = await self._fetch(query, *params)
        return [_dict_with_json_fields(r, "metadata") for r in rows]

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
        await self._execute(
            """INSERT INTO gov_press
               (id, title, url, source_name, published_at, summary,
                sentiment, politicians_mentioned, subjects, metadata)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)""",
            row_id, title, url, source_name, published_at, summary,
            sentiment, politicians_mentioned, subjects,
            json.dumps(metadata or {}, ensure_ascii=False, default=str),
        )
        row = await self._fetchrow(
            "SELECT * FROM gov_press WHERE id = $1", row_id
        )
        if row is None:
            return {"id": row_id, "title": title}
        return _dict_with_json_fields(row, "metadata")

    async def list_press_by_politician(
        self, politician_id: str, limit: int = 100
    ) -> List[Dict[str, Any]]:
        # politicians_mentioned is JSONB in PG, so we can use containment
        # But since the SQLite version stores as TEXT with LIKE, we use
        # a text cast for backward compatibility
        pattern = f"%{politician_id}%"
        rows = await self._fetch(
            """SELECT * FROM gov_press
               WHERE politicians_mentioned::text ILIKE $1
               ORDER BY published_at DESC, created_at DESC LIMIT $2""",
            pattern, limit,
        )
        return [_dict_with_json_fields(r, "metadata") for r in rows]

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
        await self._execute(
            """INSERT INTO gov_social_posts
               (id, politician_id, platform, post_id, content, url,
                media_type, media_url, posted_at, likes, shares, comments,
                metadata)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)""",
            row_id, politician_id, platform, post_id, content, url,
            media_type, media_url, posted_at, likes, shares, comments,
            json.dumps(metadata or {}, ensure_ascii=False, default=str),
        )
        row = await self._fetchrow(
            "SELECT * FROM gov_social_posts WHERE id = $1", row_id
        )
        if row is None:
            return {"id": row_id, "politician_id": politician_id}
        return _dict_with_json_fields(row, "metadata")

    async def list_social_by_politician(
        self,
        politician_id: str,
        *,
        platform: Optional[str] = None,
        limit: int = 100,
    ) -> List[Dict[str, Any]]:
        conditions = ["politician_id = $1"]
        params: List[Any] = [politician_id]
        idx = 2
        if platform is not None:
            conditions.append(f"platform = ${idx}")
            params.append(platform)
            idx += 1
        where = "WHERE " + " AND ".join(conditions)
        params.append(limit)
        query = f"SELECT * FROM gov_social_posts {where} ORDER BY posted_at DESC, created_at DESC LIMIT ${idx}"
        rows = await self._fetch(query, *params)
        return [_dict_with_json_fields(r, "metadata") for r in rows]

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
        await self._execute(
            """INSERT INTO gov_transcriptions
               (id, source_type, source_url, politician_id, title,
                transcription, timestamped_text, duration_seconds,
                language, model_used, metadata)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)""",
            row_id, source_type, source_url, politician_id, title,
            transcription, timestamped_text, duration_seconds,
            language, model_used,
            json.dumps(metadata or {}, ensure_ascii=False, default=str),
        )
        row = await self._fetchrow(
            "SELECT * FROM gov_transcriptions WHERE id = $1", row_id
        )
        if row is None:
            return {"id": row_id, "source_type": source_type}
        return _dict_with_json_fields(row, "metadata")

    async def list_transcriptions_by_politician(
        self, politician_id: str, limit: int = 50
    ) -> List[Dict[str, Any]]:
        rows = await self._fetch(
            """SELECT * FROM gov_transcriptions
               WHERE politician_id = $1
               ORDER BY created_at DESC LIMIT $2""",
            politician_id, limit,
        )
        return [_dict_with_json_fields(r, "metadata") for r in rows]

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
        await self._execute(
            """INSERT INTO gov_factchecks
               (id, claim, claim_date, claimant, politician_id, rating,
                review_url, reviewer, review_date, metadata)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)""",
            row_id, claim, claim_date, claimant, politician_id, rating,
            review_url, reviewer, review_date,
            json.dumps(metadata or {}, ensure_ascii=False, default=str),
        )
        row = await self._fetchrow(
            "SELECT * FROM gov_factchecks WHERE id = $1", row_id
        )
        if row is None:
            return {"id": row_id, "claim": claim}
        return _dict_with_json_fields(row, "metadata")

    async def list_factchecks_by_politician(
        self, politician_id: str, limit: int = 50
    ) -> List[Dict[str, Any]]:
        rows = await self._fetch(
            """SELECT * FROM gov_factchecks
               WHERE politician_id = $1
               ORDER BY claim_date DESC, created_at DESC LIMIT $2""",
            politician_id, limit,
        )
        return [_dict_with_json_fields(r, "metadata") for r in rows]

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
        await self._execute(
            """INSERT INTO gov_external_ids
               (id, politician_id, source, external_id, confidence, metadata)
               VALUES ($1,$2,$3,$4,$5,$6)""",
            row_id, politician_id, source, external_id, confidence,
            json.dumps(metadata or {}, ensure_ascii=False, default=str),
        )
        row = await self._fetchrow(
            "SELECT * FROM gov_external_ids WHERE id = $1", row_id
        )
        if row is None:
            return {"id": row_id, "politician_id": politician_id}
        return _dict_with_json_fields(row, "metadata")

    async def get_external_ids_by_politician(
        self, politician_id: str
    ) -> List[Dict[str, Any]]:
        rows = await self._fetch(
            """SELECT * FROM gov_external_ids
               WHERE politician_id = $1
               ORDER BY source ASC""",
            politician_id,
        )
        return [_dict_with_json_fields(r, "metadata") for r in rows]

    async def find_politician_by_external_id(
        self, source: str, external_id: str
    ) -> Optional[Dict[str, Any]]:
        row = await self._fetchrow(
            """SELECT p.* FROM gov_politicians p
               JOIN gov_external_ids e ON e.politician_id = p.id
               WHERE e.source = $1 AND e.external_id = $2""",
            source, external_id,
        )
        if row is None:
            return None
        return _dict_with_json_fields(row, "metadata")

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
            row = await self._fetchrow(f"SELECT COUNT(*) AS cnt FROM {table}")
            key = table.replace("gov_", "")
            counts[key] = row["cnt"] if row else 0

        scan_row = await self._fetchrow(
            "SELECT started_at FROM gov_scan_log ORDER BY started_at DESC LIMIT 1"
        )
        last_scan = str(scan_row["started_at"]) if scan_row else None

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
        rows = await self._fetch(
            "SELECT DISTINCT subject FROM gov_positions ORDER BY subject ASC"
        )
        return [r["subject"] for r in rows]

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
        """
        conditions = ["p.active = TRUE"]
        params: List[Any] = []
        idx = 1
        if chamber is not None:
            conditions.append(f"p.chamber = ${idx}")
            params.append(chamber)
            idx += 1
        where = "WHERE " + " AND ".join(conditions)

        rows = await self._fetch(
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
            *params,
        )

        nodes: List[Dict[str, Any]] = []
        node_ids: set[str] = set()
        for row in rows:
            d = dict(row)
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

        # Edge priority: opposition=0 > agreement=1 > party=2
        edge_priority = {"opposition": 0, "agreement": 1, "party": 2}
        best_edges: Dict[tuple, tuple] = {}

        def _maybe_add(src: str, tgt: str, edge: Dict[str, Any], etype: str) -> None:
            if src not in node_ids or tgt not in node_ids:
                return
            key = (min(src, tgt), max(src, tgt))
            pri = edge_priority[etype]
            if key not in best_edges or pri < best_edges[key][0]:
                best_edges[key] = (pri, edge)

        # 1) Opposition edges
        opp_rows = await self._fetch(
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
        for r in opp_rows:
            _maybe_add(r["source"], r["target"], {
                "id": str(uuid.uuid4()),
                "source": r["source"],
                "target": r["target"],
                "type": "opposition",
                "label": r["subject"],
                "stance_a": r["stance_a"],
                "stance_b": r["stance_b"],
            }, "opposition")

        # 2) Agreement edges
        agr_rows = await self._fetch(
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
        for r in agr_rows:
            _maybe_add(r["source"], r["target"], {
                "id": str(uuid.uuid4()),
                "source": r["source"],
                "target": r["target"],
                "type": "agreement",
                "label": r["subject"],
            }, "agreement")

        # 3) Party edges
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
        """Return the ego network (1-hop) centered on *politician_id*."""
        full = await self.get_graph_data()

        node_map = {n["id"]: n for n in full["nodes"]}
        if politician_id not in node_map:
            return {"nodes": [], "edges": []}

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
        """Return all politicians who took a position on *subject*."""
        rows = await self._fetch(
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
               WHERE gp.subject = $1 AND pol.active = TRUE
               ORDER BY pol.name ASC""",
            subject,
        )

        nodes: List[Dict[str, Any]] = []
        node_ids: set[str] = set()
        for row in rows:
            d = dict(row)
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
        opp_rows = await self._fetch(
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
               WHERE p1.subject = $1""",
            subject,
        )
        for r in opp_rows:
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
        agr_rows = await self._fetch(
            """SELECT DISTINCT p1.politician_id AS source,
                              p2.politician_id AS target
               FROM gov_positions p1
               JOIN gov_positions p2
                 ON p1.subject = p2.subject
                AND p1.politician_id < p2.politician_id
                AND p1.stance = p2.stance
                AND p1.stance IS NOT NULL
                AND p2.stance IS NOT NULL
               WHERE p1.subject = $1""",
            subject,
        )
        for r in agr_rows:
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


# ============================================================================
# Pool initialization
# ============================================================================

async def init_postgres(database_url: str) -> "asyncpg.Pool":
    """Create connection pool and initialize schema.

    Splits PG_SCHEMA into individual statements because asyncpg
    does not support multi-statement execute.
    """
    if asyncpg is None:
        raise ImportError("asyncpg not installed. pip install asyncpg")

    pool = await asyncpg.create_pool(database_url, min_size=2, max_size=10)

    async with pool.acquire() as conn:
        # asyncpg requires executing statements one at a time
        for stmt in PG_SCHEMA.split(";"):
            stmt = stmt.strip()
            if stmt and not stmt.startswith("--"):
                await conn.execute(stmt)

    logger.info("PostgreSQL gov schema initialized ({} tables)", 16)
    return pool
