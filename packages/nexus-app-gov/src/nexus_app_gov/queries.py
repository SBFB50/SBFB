"""Read-only SQL queries backing the Sprint 8 gov tabs for
:class:`nexus_app_gov.app.GovApp`.

Each function takes an :class:`nexus_sdk.AppDatabaseClient` and
returns plain Python data (lists of dicts, a single dict, or
primitives) that the tab handler in ``app.py`` wraps into a
:class:`nexus_sdk.view.TabView`.

Design notes
------------

- **Read-only**: every query is a ``SELECT``. Sprint 8 is
  strictly read-heavy — mutations through the gov tabs are
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
is prefixed ``gov_*``; tables consumed by the Sprint 8 tabs:

- ``gov_politicians`` — Dashboard count, Politicians list,
  PoliticianDetail, Biography
- ``gov_positions`` — Dashboard count, Positions list, Subjects
  aggregate
- ``gov_contradictions`` — Dashboard count, PoliticianDetail,
  Contradictions (Phase C upgrade)
- ``gov_mandates`` — Biography chronology
- ``gov_parties`` + ``gov_party_memberships`` — Biography party
  history
- ``gov_scan_log`` — Scan list, Workers aggregate, Pipeline
  chronology (Phase C)
- ``gov_press`` / ``gov_social_posts`` / ``gov_transcriptions``
  — Press / Social / Transcriptions tabs (Phase C)
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


# ---------------------------------------------------------------------------
# Sprint 8 Phase C — Batch 2 queries
# ---------------------------------------------------------------------------


async def contradictions_overview_query(db: AppDatabaseClient, *, limit: int = 50) -> dict[str, Any]:
    """Full Contradictions tab payload: paginated list joined
    with politician names, per-subject aggregate, and summary
    counts (total / high severity / verified).

    A missing ``gov_contradictions`` table returns ``None`` slots
    so the tab handler can render an empty state.
    """
    try:
        rows = await db.fetchall(
            """
            SELECT c.id,
                   c.subject,
                   c.severity,
                   c.description,
                   c.source_verified,
                   c.detected_at,
                   pol.name AS politician_name
            FROM gov_contradictions c
            LEFT JOIN gov_politicians pol ON pol.id = c.politician_id
            ORDER BY COALESCE(c.detected_at, '') DESC
            LIMIT ?
            """,
            (limit,),
        )
    except DatabaseError:
        rows = []

    try:
        by_subject = await db.fetchall(
            """
            SELECT subject, COUNT(*) AS count
            FROM gov_contradictions
            GROUP BY subject
            ORDER BY count DESC, subject
            LIMIT 10
            """
        )
    except DatabaseError:
        by_subject = []

    try:
        summary_row = await db.fetchone(
            """
            SELECT COUNT(*) AS total,
                   SUM(CASE WHEN severity = 'high' THEN 1 ELSE 0 END) AS high,
                   SUM(CASE WHEN source_verified = 1 THEN 1 ELSE 0 END) AS verified
            FROM gov_contradictions
            """
        )
    except DatabaseError:
        summary_row = None

    if summary_row is None:
        summary = {"total": 0, "high": 0, "verified": 0}
    else:
        summary = {
            "total": int(summary_row.get("total") or 0),
            "high": int(summary_row.get("high") or 0),
            "verified": int(summary_row.get("verified") or 0),
        }

    return {"rows": rows, "by_subject": by_subject, "summary": summary}


async def scan_log_recent_query(db: AppDatabaseClient, *, limit: int = 50) -> list[dict[str, Any]]:
    """Return the ``limit`` most recent ``gov_scan_log`` entries
    ordered by ``started_at`` desc.

    Columns projected: ``id``, ``scan_type``, ``status``,
    ``items_found``, ``items_new``, ``started_at``,
    ``completed_at``, ``error_message``.
    """
    try:
        return await db.fetchall(
            """
            SELECT id,
                   scan_type,
                   status,
                   items_found,
                   items_new,
                   started_at,
                   completed_at,
                   error_message
            FROM gov_scan_log
            ORDER BY COALESCE(started_at, '') DESC
            LIMIT ?
            """,
            (limit,),
        )
    except DatabaseError:
        return []


async def workers_state_query(db: AppDatabaseClient) -> list[dict[str, Any]]:
    """Aggregate ``gov_scan_log`` rows by ``scan_type`` to produce
    a synthetic Workers view.

    Each row describes one legacy worker module (``press_sync``,
    ``senat_sync``, ``facebook_sync``, ...) as reflected by the
    scan_log audit trail:

    - ``scan_type``   — routing key
    - ``total_runs``  — total scan_log rows for this type
    - ``successes``   — rows with ``status = 'completed'``
    - ``failures``    — rows with ``status = 'failed'``
    - ``last_status`` — most recent scan's status
    - ``last_run``    — most recent ``started_at``
    - ``items_new_total`` — cumulative ``items_new``

    Rows are ordered by ``last_run`` desc so the most active
    workers appear first.
    """
    try:
        return await db.fetchall(
            """
            SELECT scan_type,
                   COUNT(*) AS total_runs,
                   SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END) AS successes,
                   SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) AS failures,
                   MAX(started_at) AS last_run,
                   (
                       SELECT s2.status
                       FROM gov_scan_log s2
                       WHERE s2.scan_type = s1.scan_type
                       ORDER BY COALESCE(s2.started_at, '') DESC
                       LIMIT 1
                   ) AS last_status,
                   COALESCE(SUM(items_new), 0) AS items_new_total
            FROM gov_scan_log s1
            GROUP BY scan_type
            ORDER BY COALESCE(last_run, '') DESC, scan_type
            """
        )
    except DatabaseError:
        return []


async def pipeline_state_query(db: AppDatabaseClient) -> dict[str, Any]:
    """Pipeline tab payload: status distribution, running scans
    with their current phase, and a short chronological tail.

    Returns a dict with:

    - ``status_counts`` — list of ``{status, count}`` for the
      chart_bar over all scan_log rows
    - ``running``       — rows currently ``status = 'running'``
      with their current_phase + phase_offset + items counts
    - ``recent``        — last 10 scans of any status
    """
    try:
        status_counts = await db.fetchall(
            """
            SELECT status, COUNT(*) AS count
            FROM gov_scan_log
            GROUP BY status
            ORDER BY count DESC
            """
        )
    except DatabaseError:
        status_counts = []

    try:
        running = await db.fetchall(
            """
            SELECT scan_type,
                   current_phase,
                   phase_offset,
                   items_found,
                   items_new,
                   started_at
            FROM gov_scan_log
            WHERE status = 'running'
            ORDER BY COALESCE(started_at, '') DESC
            LIMIT 20
            """
        )
    except DatabaseError:
        running = []

    try:
        recent = await db.fetchall(
            """
            SELECT scan_type,
                   status,
                   items_new,
                   started_at,
                   completed_at
            FROM gov_scan_log
            ORDER BY COALESCE(started_at, '') DESC
            LIMIT 10
            """
        )
    except DatabaseError:
        recent = []

    return {
        "status_counts": status_counts,
        "running": running,
        "recent": recent,
    }


async def social_posts_query(db: AppDatabaseClient, *, limit: int = 50) -> list[dict[str, Any]]:
    """Return the ``limit`` most recent social posts joined with
    their politician's name.

    Columns: ``platform``, ``politician_name``, ``content``
    (truncated to 160 chars by the handler, not here), ``url``,
    ``posted_at``, ``likes``, ``shares``, ``comments``.
    """
    try:
        return await db.fetchall(
            """
            SELECT sp.platform,
                   sp.content,
                   sp.url,
                   sp.posted_at,
                   sp.likes,
                   sp.shares,
                   sp.comments,
                   pol.name AS politician_name
            FROM gov_social_posts sp
            LEFT JOIN gov_politicians pol ON pol.id = sp.politician_id
            ORDER BY COALESCE(sp.posted_at, '') DESC
            LIMIT ?
            """,
            (limit,),
        )
    except DatabaseError:
        return []


async def social_platform_breakdown_query(
    db: AppDatabaseClient,
) -> list[dict[str, Any]]:
    """Return ``{platform, count}`` rows for the Social tab
    chart_bar. A missing table yields an empty list.
    """
    try:
        return await db.fetchall(
            """
            SELECT platform, COUNT(*) AS count
            FROM gov_social_posts
            GROUP BY platform
            ORDER BY count DESC, platform
            """
        )
    except DatabaseError:
        return []


async def press_list_query(db: AppDatabaseClient, *, limit: int = 50) -> list[dict[str, Any]]:
    """Return the ``limit`` most recent press entries ordered by
    ``published_at`` desc.

    Columns: ``title``, ``source_name``, ``published_at``,
    ``sentiment``, ``url``, ``summary``.
    """
    try:
        return await db.fetchall(
            """
            SELECT title,
                   source_name,
                   published_at,
                   sentiment,
                   url,
                   summary
            FROM gov_press
            ORDER BY COALESCE(published_at, '') DESC
            LIMIT ?
            """,
            (limit,),
        )
    except DatabaseError:
        return []


async def transcriptions_list_query(db: AppDatabaseClient, *, limit: int = 50) -> list[dict[str, Any]]:
    """Return the ``limit`` most recent transcriptions joined
    with their politician name when available.

    Columns: ``title``, ``politician_name`` (or ``None``),
    ``source_type``, ``duration_seconds``, ``language``,
    ``model_used``, ``created_at``, ``source_url``.
    """
    try:
        return await db.fetchall(
            """
            SELECT t.title,
                   t.source_type,
                   t.duration_seconds,
                   t.language,
                   t.model_used,
                   t.created_at,
                   t.source_url,
                   pol.name AS politician_name
            FROM gov_transcriptions t
            LEFT JOIN gov_politicians pol ON pol.id = t.politician_id
            ORDER BY COALESCE(t.created_at, '') DESC
            LIMIT ?
            """,
            (limit,),
        )
    except DatabaseError:
        return []


# ---------------------------------------------------------------------------
# Sprint 8 Phase D — Batch 3 queries (Alerts/Affairs/Laws/Factchecks)
# ---------------------------------------------------------------------------


async def alerts_overview_query(db: AppDatabaseClient, *, limit: int = 50) -> dict[str, Any]:
    """Alerts tab payload: the ``limit`` most recent alerts
    joined with their politician name (when any), a severity
    chart, and three summary metrics (total / unread / high
    severity).

    A missing ``gov_alerts`` table returns the empty-shape
    payload (``rows=[]``, ``by_severity=[]``, zeros) so the tab
    handler can render an empty state.
    """
    try:
        rows = await db.fetchall(
            """
            SELECT a.id,
                   a.alert_type,
                   a.title,
                   a.description,
                   a.severity,
                   a.is_read,
                   a.created_at,
                   pol.name AS politician_name
            FROM gov_alerts a
            LEFT JOIN gov_politicians pol ON pol.id = a.politician_id
            ORDER BY COALESCE(a.created_at, '') DESC
            LIMIT ?
            """,
            (limit,),
        )
    except DatabaseError:
        rows = []

    try:
        by_severity = await db.fetchall(
            """
            SELECT severity, COUNT(*) AS count
            FROM gov_alerts
            GROUP BY severity
            ORDER BY count DESC, severity
            """
        )
    except DatabaseError:
        by_severity = []

    try:
        summary_row = await db.fetchone(
            """
            SELECT COUNT(*) AS total,
                   SUM(CASE WHEN is_read = 0 THEN 1 ELSE 0 END) AS unread,
                   SUM(CASE WHEN severity = 'high' THEN 1 ELSE 0 END) AS high
            FROM gov_alerts
            """
        )
    except DatabaseError:
        summary_row = None

    if summary_row is None:
        summary = {"total": 0, "unread": 0, "high": 0}
    else:
        summary = {
            "total": int(summary_row.get("total") or 0),
            "unread": int(summary_row.get("unread") or 0),
            "high": int(summary_row.get("high") or 0),
        }

    return {"rows": rows, "by_severity": by_severity, "summary": summary}


async def affairs_list_query(db: AppDatabaseClient, *, limit: int = 50) -> dict[str, Any]:
    """Affairs tab payload: the ``limit`` most recent affairs
    joined with their politician name plus a status breakdown
    for the ``chart_bar`` block.

    Returns a dict ``{rows, by_status}``. A missing table yields
    empty slots.
    """
    try:
        rows = await db.fetchall(
            """
            SELECT a.id,
                   a.title,
                   a.description,
                   a.status,
                   a.category,
                   a.involvement,
                   a.date_start,
                   a.date_end,
                   pol.name AS politician_name
            FROM gov_affairs a
            LEFT JOIN gov_politicians pol ON pol.id = a.politician_id
            ORDER BY COALESCE(a.date_start, a.created_at, '') DESC
            LIMIT ?
            """,
            (limit,),
        )
    except DatabaseError:
        rows = []

    try:
        by_status = await db.fetchall(
            """
            SELECT status, COUNT(*) AS count
            FROM gov_affairs
            GROUP BY status
            ORDER BY count DESC, status
            """
        )
    except DatabaseError:
        by_status = []

    return {"rows": rows, "by_status": by_status}


async def laws_list_query(db: AppDatabaseClient, *, limit: int = 50) -> dict[str, Any]:
    """Laws tab payload: the ``limit`` most recent laws plus a
    status breakdown for the ``chart_bar`` block and three
    summary metrics (total / promulgated / average duration in
    days).

    The legacy schema tracks ``duration_days`` per law; the
    summary averages the non-zero values to avoid skewing the
    figure with in-flight proposals that have no final duration.
    """
    try:
        rows = await db.fetchall(
            """
            SELECT id,
                   uid,
                   title,
                   short_title,
                   procedure,
                   status,
                   date_initial,
                   date_promulgation,
                   legislature,
                   amendments_count,
                   amendments_adopted,
                   duration_days,
                   source_url
            FROM gov_laws
            ORDER BY COALESCE(date_initial, '') DESC
            LIMIT ?
            """,
            (limit,),
        )
    except DatabaseError:
        rows = []

    try:
        by_status = await db.fetchall(
            """
            SELECT status, COUNT(*) AS count
            FROM gov_laws
            GROUP BY status
            ORDER BY count DESC, status
            """
        )
    except DatabaseError:
        by_status = []

    try:
        summary_row = await db.fetchone(
            """
            SELECT COUNT(*) AS total,
                   SUM(CASE WHEN date_promulgation IS NOT NULL
                              AND date_promulgation != ''
                            THEN 1 ELSE 0 END) AS promulgated,
                   AVG(CASE WHEN duration_days > 0
                            THEN duration_days ELSE NULL END) AS avg_duration
            FROM gov_laws
            """
        )
    except DatabaseError:
        summary_row = None

    if summary_row is None:
        summary = {"total": 0, "promulgated": 0, "avg_duration": 0}
    else:
        avg_raw = summary_row.get("avg_duration")
        summary = {
            "total": int(summary_row.get("total") or 0),
            "promulgated": int(summary_row.get("promulgated") or 0),
            "avg_duration": int(round(float(avg_raw))) if avg_raw is not None else 0,
        }

    return {"rows": rows, "by_status": by_status, "summary": summary}


async def factchecks_list_query(db: AppDatabaseClient, *, limit: int = 50) -> dict[str, Any]:
    """Factchecks tab payload: the ``limit`` most recent
    factchecks joined with their politician name plus a rating
    breakdown (``true`` / ``mostly_true`` / ``false`` / ...) for
    the ``chart_bar``.

    Returns ``{rows, by_rating}``. Missing table yields empty
    slots so the tab degrades to an empty state.
    """
    try:
        rows = await db.fetchall(
            """
            SELECT f.id,
                   f.claim,
                   f.claim_date,
                   f.claimant,
                   f.rating,
                   f.review_url,
                   f.reviewer,
                   f.review_date,
                   pol.name AS politician_name
            FROM gov_factchecks f
            LEFT JOIN gov_politicians pol ON pol.id = f.politician_id
            ORDER BY COALESCE(f.review_date, f.claim_date, '') DESC
            LIMIT ?
            """,
            (limit,),
        )
    except DatabaseError:
        rows = []

    try:
        by_rating = await db.fetchall(
            """
            SELECT rating, COUNT(*) AS count
            FROM gov_factchecks
            GROUP BY rating
            ORDER BY count DESC, rating
            """
        )
    except DatabaseError:
        by_rating = []

    return {"rows": rows, "by_rating": by_rating}


__all__ = [
    "affairs_list_query",
    "alerts_overview_query",
    "biography_query",
    "contradictions_overview_query",
    "dashboard_stats_query",
    "factchecks_list_query",
    "laws_list_query",
    "pipeline_state_query",
    "politician_detail_query",
    "politicians_list_query",
    "positions_list_query",
    "press_list_query",
    "scan_log_recent_query",
    "social_platform_breakdown_query",
    "social_posts_query",
    "subjects_aggregate_query",
    "transcriptions_list_query",
    "workers_state_query",
]
