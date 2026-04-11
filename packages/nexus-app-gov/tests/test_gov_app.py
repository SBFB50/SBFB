"""Tests for the gov app — Sprint 4 regression baseline plus
Sprint 8 Phase B Batch 1 and Phase C Batch 2 tab handlers.

The Batch 1 + Batch 2 tests build an on-disk SQLite fixture
(a real file under ``tmp_path`` — SQLite shared in-memory DBs
are awkward across aiosqlite connections) seeded with a
miniature gov schema so the handler code runs end-to-end
against real SQL.
"""

from __future__ import annotations

import sqlite3
from pathlib import Path
from typing import Any

import pytest
from nexus_app_gov import POLITICAL_CONTRADICTION_PROMPT, GovApp
from nexus_sdk import AppContext, AppDatabaseClient, ComputeClient

# ---------------------------------------------------------------------------
# Legacy Sprint 4 regression baseline
# ---------------------------------------------------------------------------


def test_gov_app_manifest_and_descriptors() -> None:
    """Sprint 8 Phase C brings gov to thirteen tabs (six Batch 1
    read-only browse tabs + the Contradictions tab upgraded from
    the Sprint 4 stub + six Batch 2 operational/content tabs).
    The route + worker surfaces are unchanged: one
    ``/statements`` route and one ``contradiction_detector``
    worker — Phase D will add the RAG workers.
    """
    app = GovApp()
    assert app.manifest.name == "gov"
    # Sprint 8 Phase B bumps the package version to 0.2.0.
    assert app.manifest.version == "0.2.0"

    routes = app.routes()
    workers = app.workers()
    tabs = app.tabs()

    assert len(routes) == 1
    assert routes[0].path == "/statements"

    assert len(workers) == 1
    assert workers[0].name == "contradiction_detector"
    assert workers[0].model == "stub-model:latest"

    # Thirteen tabs: six Batch 1 tabs + Contradictions (upgraded
    # in place from the Sprint 4 stub) + six Batch 2 tabs.
    tab_names = {t.name for t in tabs}
    assert tab_names == {
        "Contradictions",
        "Dashboard",
        "Politiciens",
        "Politicien",
        "Biographie",
        "Positions",
        "Sujets",
        "Scan",
        "Workers",
        "Pipeline",
        "Social",
        "Presse",
        "Transcriptions",
    }


def test_political_contradiction_prompt_is_present() -> None:
    assert "contradiction" in POLITICAL_CONTRADICTION_PROMPT.lower()
    assert "{statements}" in POLITICAL_CONTRADICTION_PROMPT


@pytest.mark.asyncio
async def test_on_start_and_list_statements() -> None:
    """Sprint 4 regression + Sprint 8 Phase B: ``on_start``
    leaves ``ctx.db`` in place when the legacy govdata.db file
    does not exist under the repo root (which is the case in
    CI)."""
    app = GovApp()
    ctx = AppContext(
        compute=ComputeClient("http://127.0.0.1:65500"),
        project_name="gov-test",
    )
    await app.on_start(ctx)
    body = await app.list_statements()
    assert body["app"] == "gov"
    assert body["status"] == "ready"
    await app.on_stop()


# ---------------------------------------------------------------------------
# Sprint 8 Phase B — shared fixture + helpers
# ---------------------------------------------------------------------------


_GOV_MINI_SCHEMA = """
CREATE TABLE gov_politicians (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    chamber TEXT NOT NULL,
    party TEXT,
    role TEXT,
    constituency TEXT,
    photo_url TEXT,
    official_url TEXT,
    active INTEGER DEFAULT 1
);

CREATE TABLE gov_positions (
    id TEXT PRIMARY KEY,
    politician_id TEXT NOT NULL,
    subject TEXT NOT NULL,
    position_type TEXT NOT NULL,
    position_text TEXT NOT NULL,
    stance TEXT,
    source_url TEXT NOT NULL,
    date DATE
);

CREATE TABLE gov_contradictions (
    id TEXT PRIMARY KEY,
    politician_id TEXT NOT NULL,
    position_a_id TEXT NOT NULL,
    position_b_id TEXT NOT NULL,
    subject TEXT NOT NULL,
    description TEXT NOT NULL,
    severity TEXT DEFAULT 'medium',
    source_verified INTEGER DEFAULT 0,
    detected_at DATETIME
);

CREATE TABLE gov_parties (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    short_name TEXT
);

CREATE TABLE gov_mandates (
    id TEXT PRIMARY KEY,
    politician_id TEXT NOT NULL,
    type TEXT NOT NULL,
    title TEXT,
    institution TEXT,
    constituency TEXT,
    start_date DATE,
    end_date DATE,
    is_current INTEGER DEFAULT 0,
    parliamentary_group TEXT
);

CREATE TABLE gov_party_memberships (
    id TEXT PRIMARY KEY,
    politician_id TEXT NOT NULL,
    party_id TEXT NOT NULL,
    start_date DATE,
    end_date DATE,
    is_current INTEGER DEFAULT 0
);

CREATE TABLE gov_scan_log (
    id TEXT PRIMARY KEY,
    scan_type TEXT NOT NULL,
    status TEXT DEFAULT 'running',
    items_found INTEGER DEFAULT 0,
    items_new INTEGER DEFAULT 0,
    error_message TEXT,
    started_at DATETIME,
    completed_at DATETIME,
    current_phase TEXT DEFAULT '',
    phase_offset INTEGER DEFAULT 0,
    checkpoint_data TEXT DEFAULT '{}'
);

CREATE TABLE gov_press (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    url TEXT,
    source_name TEXT,
    published_at DATETIME,
    summary TEXT,
    sentiment TEXT,
    politicians_mentioned TEXT,
    subjects TEXT
);

CREATE TABLE gov_social_posts (
    id TEXT PRIMARY KEY,
    politician_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    post_id TEXT,
    content TEXT,
    url TEXT,
    media_type TEXT,
    media_url TEXT,
    posted_at DATETIME,
    likes INTEGER DEFAULT 0,
    shares INTEGER DEFAULT 0,
    comments INTEGER DEFAULT 0
);

CREATE TABLE gov_transcriptions (
    id TEXT PRIMARY KEY,
    source_type TEXT NOT NULL,
    source_url TEXT,
    politician_id TEXT,
    title TEXT,
    transcription TEXT,
    timestamped_text TEXT,
    duration_seconds INTEGER,
    language TEXT DEFAULT 'fr',
    model_used TEXT,
    created_at DATETIME
);
"""


def _seed_gov_db(db_path: Path) -> None:
    """Seed a miniature gov schema with two politicians, a handful
    of positions, a contradiction, and some mandates.

    Uses :mod:`sqlite3` synchronously because we only need the
    data to be present before the async handlers run — the
    aiosqlite wrapper under test reopens the file from scratch.
    """
    conn = sqlite3.connect(db_path)
    try:
        conn.executescript(_GOV_MINI_SCHEMA)
        conn.executemany(
            "INSERT INTO gov_politicians VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            [
                (
                    "p-alice",
                    "Alice Martin",
                    "alice-martin",
                    "assemblee",
                    "PS",
                    "Députée",
                    "Paris 1",
                    None,
                    None,
                    1,
                ),
                (
                    "p-bob",
                    "Bob Durand",
                    "bob-durand",
                    "senat",
                    "LR",
                    "Sénateur",
                    "Rhône",
                    None,
                    None,
                    0,
                ),
            ],
        )
        conn.executemany(
            "INSERT INTO gov_positions VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            [
                (
                    "pos-1",
                    "p-alice",
                    "climat",
                    "vote",
                    "Pour la loi climat.",
                    "favor",
                    "https://example.org/1",
                    "2025-10-01",
                ),
                (
                    "pos-2",
                    "p-alice",
                    "climat",
                    "declaration",
                    "Le climat n'est pas urgent.",
                    "against",
                    "https://example.org/2",
                    "2025-11-15",
                ),
                (
                    "pos-3",
                    "p-bob",
                    "fiscalite",
                    "vote",
                    "Contre la hausse fiscale.",
                    "against",
                    "https://example.org/3",
                    "2025-09-20",
                ),
            ],
        )
        conn.execute(
            "INSERT INTO gov_contradictions VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                "ctr-1",
                "p-alice",
                "pos-1",
                "pos-2",
                "climat",
                "Pour puis contre la loi climat.",
                "high",
                1,
                "2025-12-01T08:00:00",
            ),
        )
        conn.execute(
            "INSERT INTO gov_parties VALUES (?, ?, ?)",
            ("party-ps", "Parti socialiste", "PS"),
        )
        conn.executemany(
            "INSERT INTO gov_mandates VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            [
                (
                    "m-alice-1",
                    "p-alice",
                    "depute",
                    "Députée 1ère circ. Paris",
                    "Assemblée nationale",
                    "Paris 1",
                    "2022-06-01",
                    None,
                    1,
                    "Socialistes",
                ),
            ],
        )
        conn.execute(
            "INSERT INTO gov_party_memberships VALUES (?, ?, ?, ?, ?, ?)",
            ("mb-alice-1", "p-alice", "party-ps", "2022-06-01", None, 1),
        )
        # ------------------------------------------------------------
        # Sprint 8 Phase C — operational + content tables
        # ------------------------------------------------------------
        conn.executemany(
            "INSERT INTO gov_scan_log "
            "(id, scan_type, status, items_found, items_new, "
            " error_message, started_at, completed_at, "
            " current_phase, phase_offset, checkpoint_data) "
            "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            [
                (
                    "sc-1",
                    "press_sync",
                    "completed",
                    42,
                    12,
                    None,
                    "2025-11-30T08:00:00",
                    "2025-11-30T08:04:00",
                    "done",
                    0,
                    "{}",
                ),
                (
                    "sc-2",
                    "press_sync",
                    "failed",
                    0,
                    0,
                    "429 rate limit",
                    "2025-12-01T08:00:00",
                    "2025-12-01T08:00:30",
                    "fetch",
                    1,
                    "{}",
                ),
                (
                    "sc-3",
                    "depute_sync",
                    "running",
                    100,
                    25,
                    None,
                    "2025-12-01T09:30:00",
                    None,
                    "politicians",
                    150,
                    "{}",
                ),
                (
                    "sc-4",
                    "facebook_sync",
                    "completed",
                    18,
                    18,
                    None,
                    "2025-12-01T10:00:00",
                    "2025-12-01T10:02:00",
                    "done",
                    0,
                    "{}",
                ),
            ],
        )
        conn.executemany(
            "INSERT INTO gov_press "
            "(id, title, url, source_name, published_at, summary, "
            " sentiment, politicians_mentioned, subjects) "
            "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            [
                (
                    "pr-1",
                    "Alice Martin défend la loi climat",
                    "https://lemonde.fr/article-1",
                    "Le Monde",
                    "2025-11-29T18:00:00",
                    "Prise de parole à l'Assemblée.",
                    "positive",
                    "p-alice",
                    "climat",
                ),
                (
                    "pr-2",
                    "Bob Durand s'oppose à la hausse fiscale",
                    "https://lefigaro.fr/article-2",
                    "Le Figaro",
                    "2025-11-28T09:00:00",
                    "Intervention au Sénat.",
                    "neutral",
                    "p-bob",
                    "fiscalite",
                ),
            ],
        )
        conn.executemany(
            "INSERT INTO gov_social_posts "
            "(id, politician_id, platform, post_id, content, url, "
            " media_type, media_url, posted_at, likes, shares, comments) "
            "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            [
                (
                    "sp-1",
                    "p-alice",
                    "twitter",
                    "t-1",
                    "Oui à la loi climat #climat",
                    "https://twitter.com/alice/status/1",
                    "text",
                    None,
                    "2025-11-29T19:00:00",
                    120,
                    34,
                    12,
                ),
                (
                    "sp-2",
                    "p-alice",
                    "facebook",
                    "fb-1",
                    "Communiqué sur le climat",
                    "https://facebook.com/alice/posts/1",
                    "text",
                    None,
                    "2025-11-30T08:00:00",
                    45,
                    8,
                    3,
                ),
                (
                    "sp-3",
                    "p-bob",
                    "twitter",
                    "t-2",
                    "Non à la hausse fiscale",
                    "https://twitter.com/bob/status/1",
                    "text",
                    None,
                    "2025-11-28T10:00:00",
                    80,
                    20,
                    5,
                ),
            ],
        )
        conn.executemany(
            "INSERT INTO gov_transcriptions "
            "(id, source_type, source_url, politician_id, title, "
            " transcription, timestamped_text, duration_seconds, "
            " language, model_used, created_at) "
            "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            [
                (
                    "tr-1",
                    "youtube",
                    "https://youtube.com/watch?v=abc",
                    "p-alice",
                    "Discours Alice au Parlement",
                    "Mes chers collègues, aujourd'hui…",
                    None,
                    540,
                    "fr",
                    "whisper-large",
                    "2025-11-29T19:30:00",
                ),
                (
                    "tr-2",
                    "podcast",
                    "https://podcast.fm/ep-12",
                    None,
                    "Analyse budget 2026",
                    "Le budget est marqué par…",
                    None,
                    1800,
                    "fr",
                    "whisper-medium",
                    "2025-11-27T15:00:00",
                ),
            ],
        )
        conn.commit()
    finally:
        conn.close()


async def _build_seeded_app(tmp_path: Path) -> GovApp:
    """Return a started :class:`GovApp` bound to a seeded SQLite
    fixture under ``tmp_path``. The app's internal ``_ctx`` has a
    real :class:`AppDatabaseClient` pointing at the fixture file
    so every Batch 1 handler runs the real query path."""
    db_file = tmp_path / "gov.sqlite"
    _seed_gov_db(db_file)
    app = GovApp()
    ctx = AppContext(
        compute=ComputeClient("http://127.0.0.1:65500"),
        project_name="gov-batch1",
        app_name="gov",
        db=AppDatabaseClient(db_file),
    )
    # We bypass the on_start legacy redirection because the
    # fixture file is already under tmp_path — the redirect
    # would try to point at nexus/gov/govdata.db which does
    # not exist in CI anyway. Assigning _ctx directly mirrors
    # the effect of on_start when the legacy file is absent.
    app._ctx = ctx
    return app


def _block_kinds(descriptor: dict[str, Any]) -> list[str]:
    return [block["kind"] for block in descriptor["blocks"]]


# ---------------------------------------------------------------------------
# Sprint 8 Phase B — six Batch 1 tab handler tests
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_dashboard_tab_renders_counts(tmp_path: Path) -> None:
    """Dashboard tab must surface the aggregate counts plus a
    ``chart_bar`` of the top subjects when the DB has data."""
    app = await _build_seeded_app(tmp_path)
    desc = await app.dashboard_tab()

    assert desc["schema_version"] == 1
    assert desc["tab_name"] == "dashboard"
    kinds = _block_kinds(desc)
    # Exactly one heading + one subtitle text + one inventory
    # section + one chart_bar for the top subjects.
    assert kinds[0] == "heading"
    assert "section" in kinds
    assert "chart_bar" in kinds

    # The section contains five metric blocks — one per count.
    section_block = next(b for b in desc["blocks"] if b["kind"] == "section")
    metric_labels = {b["label"] for b in section_block["blocks"]}
    assert metric_labels == {
        "Politiciens",
        "Politiciens actifs",
        "Positions",
        "Contradictions",
        "Partis",
    }
    metric_values = {b["label"]: b["value"] for b in section_block["blocks"]}
    assert metric_values["Politiciens"] == 2
    assert metric_values["Politiciens actifs"] == 1
    assert metric_values["Positions"] == 3
    assert metric_values["Contradictions"] == 1
    assert metric_values["Partis"] == 1

    # The chart_bar lists the two subjects present in the fixture.
    chart = next(b for b in desc["blocks"] if b["kind"] == "chart_bar")
    bar_subjects = {b["label"] for b in chart["bars"]}
    assert bar_subjects == {"climat", "fiscalite"}
    climat_bar = next(b for b in chart["bars"] if b["label"] == "climat")
    assert climat_bar["value"] == 2  # pos-1 and pos-2


@pytest.mark.asyncio
async def test_politicians_tab_lists_both_rows(tmp_path: Path) -> None:
    """Politiciens table must ship both seeded politicians with
    the six-column schema the shell expects."""
    app = await _build_seeded_app(tmp_path)
    desc = await app.politicians_tab()

    assert desc["tab_name"] == "politiciens"
    table_block = next(b for b in desc["blocks"] if b["kind"] == "table")
    assert [c["key"] for c in table_block["columns"]] == [
        "name",
        "chamber",
        "party",
        "role",
        "constituency",
        "active",
    ]
    assert len(table_block["rows"]) == 2
    names = sorted(row["name"] for row in table_block["rows"])
    assert names == ["Alice Martin", "Bob Durand"]


@pytest.mark.asyncio
async def test_politician_detail_tab_picks_first_politician(tmp_path: Path) -> None:
    """The detail tab picks ``ORDER BY name LIMIT 1`` → Alice.
    Her two positions land in a nested section table."""
    app = await _build_seeded_app(tmp_path)
    desc = await app.politician_detail_tab()

    assert desc["tab_name"] == "politicien"
    heading_block = desc["blocks"][0]
    assert heading_block["kind"] == "heading"
    assert heading_block["text"] == "Alice Martin"

    kv_block = next(b for b in desc["blocks"] if b["kind"] == "kv")
    labels = {item["label"]: item["value"] for item in kv_block["items"]}
    assert labels["Chambre"] == "assemblee"
    assert labels["Parti"] == "PS"
    assert labels["Actif"] == 1

    metric_block = next(b for b in desc["blocks"] if b["kind"] == "metric")
    assert metric_block["label"] == "Contradictions"
    assert metric_block["value"] == 1  # ctr-1 is against Alice

    positions_section = next(b for b in desc["blocks"] if b["kind"] == "section" and b["title"] == "Positions récentes")
    assert positions_section["blocks"][0]["kind"] == "table"
    position_rows = positions_section["blocks"][0]["rows"]
    assert len(position_rows) == 2


@pytest.mark.asyncio
async def test_biography_tab_shows_mandates_and_memberships(tmp_path: Path) -> None:
    """Biographie tab renders mandates + party memberships
    tables for the first politician (Alice)."""
    app = await _build_seeded_app(tmp_path)
    desc = await app.biography_tab()

    assert desc["tab_name"] == "biographie"
    heading_block = desc["blocks"][0]
    assert "Alice Martin" in heading_block["text"]

    sections = [b for b in desc["blocks"] if b["kind"] == "section"]
    titles = {s["title"] for s in sections}
    assert "Mandats" in titles
    assert "Appartenances partisanes" in titles

    mandates_section = next(s for s in sections if s["title"] == "Mandats")
    mandates_table = mandates_section["blocks"][0]
    assert mandates_table["kind"] == "table"
    assert len(mandates_table["rows"]) == 1
    assert mandates_table["rows"][0]["type"] == "depute"

    memberships_section = next(s for s in sections if s["title"] == "Appartenances partisanes")
    memberships_table = memberships_section["blocks"][0]
    assert memberships_table["kind"] == "table"
    assert len(memberships_table["rows"]) == 1
    assert memberships_table["rows"][0]["party_name"] == "Parti socialiste"


@pytest.mark.asyncio
async def test_positions_tab_lists_all_three(tmp_path: Path) -> None:
    """Positions tab table must carry the three seeded positions
    joined with their politician names, sorted by date desc."""
    app = await _build_seeded_app(tmp_path)
    desc = await app.positions_tab()

    assert desc["tab_name"] == "positions"
    table_block = next(b for b in desc["blocks"] if b["kind"] == "table")
    assert len(table_block["rows"]) == 3
    dates = [row["date"] for row in table_block["rows"]]
    assert dates == sorted(dates, reverse=True)
    # The join resolved politician_name via LEFT JOIN.
    names = {row["politician_name"] for row in table_block["rows"]}
    assert "Alice Martin" in names
    assert "Bob Durand" in names


@pytest.mark.asyncio
async def test_subjects_tab_aggregates_by_count(tmp_path: Path) -> None:
    """Sujets tab surfaces a chart_bar + a table ordered by
    position count desc. ``climat`` (2 positions) beats
    ``fiscalite`` (1 position)."""
    app = await _build_seeded_app(tmp_path)
    desc = await app.subjects_tab()

    assert desc["tab_name"] == "sujets"
    chart = next(b for b in desc["blocks"] if b["kind"] == "chart_bar")
    assert [b["label"] for b in chart["bars"]] == ["climat", "fiscalite"]
    assert [b["value"] for b in chart["bars"]] == [2, 1]

    table_block = next(b for b in desc["blocks"] if b["kind"] == "table")
    assert [row["subject"] for row in table_block["rows"]] == ["climat", "fiscalite"]
    assert [row["count"] for row in table_block["rows"]] == [2, 1]


# ---------------------------------------------------------------------------
# Sprint 8 Phase B — empty-state fallback
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_tabs_render_empty_state_when_db_missing(tmp_path: Path) -> None:
    """A client pointed at a file that doesn't yet contain the
    gov schema must fall back to the empty-state TabView for
    every Batch 1 + Batch 2 tab — the shell still renders a
    heading + an empty block instead of surfacing a 500."""
    empty_db = tmp_path / "empty.sqlite"
    app = GovApp()
    app._ctx = AppContext(
        compute=ComputeClient("http://127.0.0.1:65500"),
        project_name="gov-empty",
        app_name="gov",
        db=AppDatabaseClient(empty_db),
    )

    for handler in (
        app.dashboard_tab,
        app.politicians_tab,
        app.politician_detail_tab,
        app.biography_tab,
        app.positions_tab,
        app.subjects_tab,
        app.contradictions_tab,
        app.scan_tab,
        app.workers_tab,
        app.pipeline_tab,
        app.social_tab,
        app.press_tab,
        app.transcriptions_tab,
    ):
        desc = await handler()
        assert desc["schema_version"] == 1
        # Every empty-state TabView is exactly a heading followed
        # by an empty block.
        assert desc["blocks"][0]["kind"] == "heading"
        assert any(b["kind"] == "empty" for b in desc["blocks"])


# ---------------------------------------------------------------------------
# Sprint 8 Phase C — seven Batch 2 tab handler tests
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_contradictions_tab_upgrades_to_table_with_summary(tmp_path: Path) -> None:
    """Phase C rewrite: the Contradictions tab returns a TabView
    with summary metrics, a per-subject chart_bar, and the
    paginated table joined with politician names.
    """
    app = await _build_seeded_app(tmp_path)
    desc = await app.contradictions_tab()

    assert desc["tab_name"] == "contradictions"
    kinds = _block_kinds(desc)
    # heading + muted text + summary section + chart_bar + table.
    assert kinds[0] == "heading"
    assert "section" in kinds
    assert "chart_bar" in kinds
    assert "table" in kinds

    summary_section = next(b for b in desc["blocks"] if b["kind"] == "section" and b["title"] == "Résumé")
    metric_labels = {b["label"]: b["value"] for b in summary_section["blocks"]}
    assert metric_labels["Contradictions détectées"] == 1
    # ctr-1 has severity 'high' → high metric rings danger.
    high_metric = next(b for b in summary_section["blocks"] if b["label"] == "Sévérité haute")
    assert high_metric["value"] == 1
    assert high_metric["tone"] == "danger"

    chart = next(b for b in desc["blocks"] if b["kind"] == "chart_bar")
    assert chart["bars"][0]["label"] == "climat"
    assert chart["bars"][0]["value"] == 1

    table_block = next(b for b in desc["blocks"] if b["kind"] == "table")
    assert len(table_block["rows"]) == 1
    row = table_block["rows"][0]
    assert row["politician_name"] == "Alice Martin"
    assert row["subject"] == "climat"
    assert row["severity"] == "high"


@pytest.mark.asyncio
async def test_scan_tab_lists_recent_log_entries(tmp_path: Path) -> None:
    """Scan tab surfaces the four seeded scan_log rows ordered by
    started_at desc."""
    app = await _build_seeded_app(tmp_path)
    desc = await app.scan_tab()

    assert desc["tab_name"] == "scan"
    table_block = next(b for b in desc["blocks"] if b["kind"] == "table")
    assert len(table_block["rows"]) == 4
    # Most recent row first — sc-4 (facebook_sync 2025-12-01T10:00).
    assert table_block["rows"][0]["scan_type"] == "facebook_sync"
    assert table_block["rows"][0]["status"] == "completed"
    statuses = {row["status"] for row in table_block["rows"]}
    assert statuses == {"completed", "failed", "running"}


@pytest.mark.asyncio
async def test_workers_tab_aggregates_by_scan_type(tmp_path: Path) -> None:
    """Workers tab groups scan_log rows by scan_type with per-worker
    run stats and a summary section."""
    app = await _build_seeded_app(tmp_path)
    desc = await app.workers_tab()

    assert desc["tab_name"] == "workers"
    summary_section = next(b for b in desc["blocks"] if b["kind"] == "section" and b["title"] == "Synthèse")
    summary_metrics = {b["label"]: b["value"] for b in summary_section["blocks"]}
    assert summary_metrics["Workers distincts"] == 3
    # press_sync has one failed run → 1 worker flagged with failures.
    assert summary_metrics["Avec échecs"] == 1

    table_block = next(b for b in desc["blocks"] if b["kind"] == "table")
    rows_by_name = {row["scan_type"]: row for row in table_block["rows"]}
    assert set(rows_by_name) == {"press_sync", "depute_sync", "facebook_sync"}

    press_row = rows_by_name["press_sync"]
    assert press_row["total_runs"] == 2
    assert press_row["successes"] == 1
    assert press_row["failures"] == 1
    # Most recent press_sync run is sc-2 (failed) by started_at desc.
    assert press_row["last_status"] == "failed"


@pytest.mark.asyncio
async def test_pipeline_tab_exposes_running_scan_and_status_distribution(
    tmp_path: Path,
) -> None:
    """Pipeline tab carries a chart_bar of statuses, a running
    section with the one in-flight scan, and a historical tail."""
    app = await _build_seeded_app(tmp_path)
    desc = await app.pipeline_tab()

    assert desc["tab_name"] == "pipeline"

    chart = next(b for b in desc["blocks"] if b["kind"] == "chart_bar")
    statuses = {b["label"]: b["value"] for b in chart["bars"]}
    assert statuses["completed"] == 2
    assert statuses["failed"] == 1
    assert statuses["running"] == 1

    running_section = next(b for b in desc["blocks"] if b["kind"] == "section" and b["title"] == "En cours")
    running_table = running_section["blocks"][0]
    assert running_table["kind"] == "table"
    assert len(running_table["rows"]) == 1
    running_row = running_table["rows"][0]
    assert running_row["scan_type"] == "depute_sync"
    assert running_row["current_phase"] == "politicians"
    assert running_row["phase_offset"] == 150

    history_section = next(b for b in desc["blocks"] if b["kind"] == "section" and b["title"] == "Historique récent")
    history_table = history_section["blocks"][0]
    assert len(history_table["rows"]) == 4


@pytest.mark.asyncio
async def test_social_tab_lists_posts_with_platform_chart(tmp_path: Path) -> None:
    """Social tab lists the three seeded posts, ordered by
    posted_at desc, and exposes a chart_bar of platform counts."""
    app = await _build_seeded_app(tmp_path)
    desc = await app.social_tab()

    assert desc["tab_name"] == "social"

    chart = next(b for b in desc["blocks"] if b["kind"] == "chart_bar")
    bars_by_platform = {b["label"]: b["value"] for b in chart["bars"]}
    assert bars_by_platform["twitter"] == 2
    assert bars_by_platform["facebook"] == 1

    table_block = next(b for b in desc["blocks"] if b["kind"] == "table")
    assert len(table_block["rows"]) == 3
    # Most recent first (sp-2 facebook 2025-11-30T08:00).
    assert table_block["rows"][0]["platform"] == "facebook"
    assert table_block["rows"][0]["politician_name"] == "Alice Martin"
    # All three rows carry a politician_name via LEFT JOIN.
    names = {row["politician_name"] for row in table_block["rows"]}
    assert names == {"Alice Martin", "Bob Durand"}


@pytest.mark.asyncio
async def test_press_tab_lists_articles_sorted_by_date(tmp_path: Path) -> None:
    """Presse tab lists the two seeded press entries ordered by
    published_at desc, each with a sentiment column."""
    app = await _build_seeded_app(tmp_path)
    desc = await app.press_tab()

    assert desc["tab_name"] == "presse"
    table_block = next(b for b in desc["blocks"] if b["kind"] == "table")
    assert len(table_block["rows"]) == 2
    dates = [row["published_at"] for row in table_block["rows"]]
    assert dates == sorted(dates, reverse=True)
    # Most recent first (pr-1 Le Monde positive).
    assert table_block["rows"][0]["source_name"] == "Le Monde"
    assert table_block["rows"][0]["sentiment"] == "positive"


@pytest.mark.asyncio
async def test_transcriptions_tab_joins_politician_name(tmp_path: Path) -> None:
    """Transcriptions tab ships the two seeded rows with the
    politician_name column populated via LEFT JOIN when set."""
    app = await _build_seeded_app(tmp_path)
    desc = await app.transcriptions_tab()

    assert desc["tab_name"] == "transcriptions"
    table_block = next(b for b in desc["blocks"] if b["kind"] == "table")
    assert len(table_block["rows"]) == 2
    # tr-1 is the most recent (created_at 2025-11-29T19:30).
    assert table_block["rows"][0]["title"] == "Discours Alice au Parlement"
    assert table_block["rows"][0]["politician_name"] == "Alice Martin"
    # tr-2 has no politician_id → politician_name coerces to '—'.
    assert table_block["rows"][1]["politician_name"] == "—"
