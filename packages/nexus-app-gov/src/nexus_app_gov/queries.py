"""Read-only SQL queries backing the Sprint 8 Phase B Batch 1
tabs for :class:`nexus_app_gov.app.GovApp`.

Each function takes an :class:`nexus_sdk.AppDatabaseClient` and
returns plain Python data (lists of dicts, a single dict, or
primitives) that the tab handler in ``app.py`` wraps into a
:class:`nexus_sdk.view.TabView`.

Design notes
------------

- **Read-only**: every query is a ``SELECT``. Sprint 8 Phase B
  is strictly read-heavy — mutations through the gov tabs are
  deferred to Sprint 9+.
- **Graceful degradation**: if the legacy SQLite schema is
  missing a table (fresh install without a prior scrape), the
  underlying :class:`nexus_sdk.DatabaseError` bubbles up and the
  tab handler catches it to render an empty state instead of
  surfacing a 500 to the shell.
- **No joins where counts suffice**: the Dashboard tab uses
  independent ``COUNT(*)`` queries rather than a single aggregate
  — this keeps the schema coupling loose so missing tables
  degrade independently.
- **LIMIT 50**: list views paginate implicitly to 50 rows. Sprint
  9 polish will add proper pagination; until then 50 is a
  reasonable dev sample.

Legacy schema reference
-----------------------

The gov SQLite schema lives in ``nexus/gov/db.py``. Every table
is prefixed ``gov_*``; the tables consumed by Batch 1 are:

- ``gov_politicians`` — Dashboard count, Politicians list,
  PoliticianDetail, Biography
- ``gov_positions`` — Dashboard count, Positions list, Subjects
  aggregate
- ``gov_contradictions`` — Dashboard count, PoliticianDetail
- ``gov_mandates`` — Biography chronology
- ``gov_parties`` + ``gov_party_memberships`` — Biography party
  history
"""

from __future__ import annotations

from typing import Any

from nexus_sdk import AppDatabaseClient, DatabaseError


async def _safe_count(db: AppDatabaseClient, table: str) -> int:
    """Return ``COUNT(*) FROM table`` or ``0`` on missing-table.

    The Dashboard tab calls this for several independent tables
    so a fresh DB (zero or one of the expected tables present)
    still renders gracefully.
    """
    try:
        row = await db.fetchone(f"SELECT COUNT(*) AS n FROM {table}")
    except DatabaseError:
        return 0
    return int(row["n"]) if row is not None else 0


async def dashboard_stats_query(db: AppDatabaseClient) -> dict[str, Any]:
    """Aggregate counts for the Dashboard tab.

    Returns a dict with:

    - ``politicians`` — total count (int)
    - ``active_politicians`` — count with ``active = 1``
    - ``positions`` — total count
    - ``contradictions`` — total count
    - ``parties`` — total count
    - ``top_subjects`` — up to 5 subject/count pairs sorted desc
    """
    politicians = await _safe_count(db, "gov_politicians")
    positions = await _safe_count(db, "gov_positions")
    contradictions = await _safe_count(db, "gov_contradictions")
    parties = await _safe_count(db, "gov_parties")

    try:
        active_row = await db.fetchone("SELECT COUNT(*) AS n FROM gov_politicians WHERE active = 1")
        active_politicians = int(active_row["n"]) if active_row is not None else 0
    except DatabaseError:
        active_politicians = 0

    try:
        top_subjects = await db.fetchall(
            """
            SELECT subject, COUNT(*) AS n
            FROM gov_positions
            GROUP BY subject
            ORDER BY n DESC
            LIMIT 5
            """
        )
    except DatabaseError:
        top_subjects = []

    return {
        "politicians": politicians,
        "active_politicians": active_politicians,
        "positions": positions,
        "contradictions": contradictions,
        "parties": parties,
        "top_subjects": top_subjects,
    }


async def politicians_list_query(db: AppDatabaseClient, *, limit: int = 50) -> list[dict[str, Any]]:
    """Return up to ``limit`` politicians sorted by name.

    Columns projected: ``id``, ``name``, ``chamber``, ``party``,
    ``role``, ``constituency``, ``active``. A fresh DB missing
    the table returns ``[]`` rather than raising — the caller
    decides whether to render an empty state.
    """
    try:
        return await db.fetchall(
            """
            SELECT id, name, chamber, party, role, constituency, active
            FROM gov_politicians
            ORDER BY name
            LIMIT ?
            """,
            (limit,),
        )
    except DatabaseError:
        return []


async def politician_detail_query(
    db: AppDatabaseClient,
) -> dict[str, Any] | None:
    """Return a fiche for the first politician (``ORDER BY name
    LIMIT 1``) together with her most recent ten positions and
    her total contradiction count.

    Sprint 8 Phase B keeps the politician selection implicit —
    Sprint 9 polish will introduce a per-tab selector. A missing
    ``gov_politicians`` table or empty table returns ``None`` so
    the tab can render an empty state.
    """
    try:
        row = await db.fetchone(
            """
            SELECT id, name, slug, chamber, party, role, constituency,
                   photo_url, official_url, active
            FROM gov_politicians
            ORDER BY name
            LIMIT 1
            """
        )
    except DatabaseError:
        return None
    if row is None:
        return None

    pol_id = row["id"]
    try:
        positions = await db.fetchall(
            """
            SELECT subject, position_type, position_text, stance, date
            FROM gov_positions
            WHERE politician_id = ?
            ORDER BY COALESCE(date, '') DESC
            LIMIT 10
            """,
            (pol_id,),
        )
    except DatabaseError:
        positions = []

    try:
        contradiction_row = await db.fetchone(
            "SELECT COUNT(*) AS n FROM gov_contradictions WHERE politician_id = ?",
            (pol_id,),
        )
        contradictions = int(contradiction_row["n"]) if contradiction_row is not None else 0
    except DatabaseError:
        contradictions = 0

    return {
        "politician": row,
        "recent_positions": positions,
        "contradictions_count": contradictions,
    }


async def biography_query(db: AppDatabaseClient) -> dict[str, Any] | None:
    """Biography tab payload for the first politician.

    Joins ``gov_mandates`` and ``gov_party_memberships`` against
    ``gov_parties`` to produce a career chronology. Returns
    ``None`` when no politician exists.
    """
    try:
        pol = await db.fetchone(
            """
            SELECT id, name, chamber, party, role, photo_url, official_url
            FROM gov_politicians
            ORDER BY name
            LIMIT 1
            """
        )
    except DatabaseError:
        return None
    if pol is None:
        return None

    pol_id = pol["id"]
    try:
        mandates = await db.fetchall(
            """
            SELECT type, title, institution, constituency,
                   start_date, end_date, is_current, parliamentary_group
            FROM gov_mandates
            WHERE politician_id = ?
            ORDER BY COALESCE(start_date, '') DESC
            """,
            (pol_id,),
        )
    except DatabaseError:
        mandates = []

    try:
        memberships = await db.fetchall(
            """
            SELECT pm.start_date, pm.end_date, pm.is_current, p.name AS party_name, p.short_name
            FROM gov_party_memberships pm
            LEFT JOIN gov_parties p ON p.id = pm.party_id
            WHERE pm.politician_id = ?
            ORDER BY COALESCE(pm.start_date, '') DESC
            """,
            (pol_id,),
        )
    except DatabaseError:
        memberships = []

    return {
        "politician": pol,
        "mandates": mandates,
        "party_memberships": memberships,
    }


async def positions_list_query(db: AppDatabaseClient, *, limit: int = 50) -> list[dict[str, Any]]:
    """Return the ``limit`` most recent positions joined with
    their politician's name.

    Columns: ``position_id``, ``politician_name``, ``subject``,
    ``position_type``, ``stance``, ``date``, ``source_url``.
    """
    try:
        return await db.fetchall(
            """
            SELECT pos.id AS position_id,
                   pol.name AS politician_name,
                   pos.subject,
                   pos.position_type,
                   pos.stance,
                   pos.date,
                   pos.source_url
            FROM gov_positions pos
            LEFT JOIN gov_politicians pol ON pol.id = pos.politician_id
            ORDER BY COALESCE(pos.date, '') DESC
            LIMIT ?
            """,
            (limit,),
        )
    except DatabaseError:
        return []


async def subjects_aggregate_query(db: AppDatabaseClient, *, limit: int = 20) -> list[dict[str, Any]]:
    """Return up to ``limit`` distinct ``gov_positions.subject``
    values ordered by frequency desc.

    Shape per row: ``{subject, count}``. The tab handler wraps
    these into a ``chart_bar`` block or a table if the list is
    empty.
    """
    try:
        return await db.fetchall(
            """
            SELECT subject, COUNT(*) AS count
            FROM gov_positions
            GROUP BY subject
            ORDER BY count DESC, subject
            LIMIT ?
            """,
            (limit,),
        )
    except DatabaseError:
        return []


__all__ = [
    "biography_query",
    "dashboard_stats_query",
    "politician_detail_query",
    "politicians_list_query",
    "positions_list_query",
    "subjects_aggregate_query",
]
