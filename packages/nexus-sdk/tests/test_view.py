# SPDX-License-Identifier: AGPL-3.0-or-later
"""Unit tests for the Sprint 6 TabView schema and helpers."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import pytest
from nexus_sdk.view import (
    ActionRoute,
    ActionTaskSubmit,
    BadgeItem,
    ChartBar,
    ChartLinePoint,
    KVItem,
    TabBlockBadgeList,
    TabBlockButton,
    TabBlockChartBar,
    TabBlockChartLine,
    TabBlockEmpty,
    TabBlockHeading,
    TabBlockKV,
    TabBlockMetric,
    TabBlockSection,
    TabBlockTable,
    TabBlockText,
    TableColumn,
    TabView,
    badge_list,
    button_route,
    button_task,
    chart_bar,
    chart_line,
    empty,
    heading,
    kv,
    metric,
    section,
    table_,
    text,
)
from pydantic import ValidationError

SNAPSHOT_PATH = Path(__file__).parent / "snapshots" / "tabview_schema.json"
CANONICAL_FIXTURE_PATH = Path(__file__).parent / "snapshots" / "tabview_canonical.json"


# ---------------------------------------------------------------------------
# Top-level TabView invariants (D2 contract)
# ---------------------------------------------------------------------------


def test_tabview_defaults_schema_version_1() -> None:
    tv = TabView(tab_name="x")
    assert tv.schema_version == 1
    assert tv.blocks == []
    assert tv.title is None


def test_tabview_requires_schema_version_1() -> None:
    tv = TabView(schema_version=1, tab_name="x")
    assert tv.schema_version == 1


def test_tabview_rejects_unknown_schema_version() -> None:
    with pytest.raises(ValidationError):
        TabView(schema_version=2, tab_name="x")  # type: ignore[arg-type]


def test_tabview_forbids_extra_fields() -> None:
    with pytest.raises(ValidationError):
        TabView(tab_name="x", extra_key="nope")  # type: ignore[call-arg]


def test_tabview_requires_tab_name() -> None:
    with pytest.raises(ValidationError):
        TabView()  # type: ignore[call-arg]


# ---------------------------------------------------------------------------
# Discriminated union — every kind dumps + reloads cleanly
# ---------------------------------------------------------------------------


def _roundtrip(tv: TabView) -> TabView:
    dumped = tv.model_dump()
    reloaded = TabView.model_validate(dumped)
    assert reloaded == tv
    return reloaded


def test_section_block_recursive() -> None:
    tv = TabView(
        tab_name="demo",
        blocks=[
            section(
                title="outer",
                blocks=[section(title="inner", blocks=[empty(text="deep")])],
            )
        ],
    )
    assert isinstance(tv.blocks[0], TabBlockSection)
    inner = tv.blocks[0].blocks[0]
    assert isinstance(inner, TabBlockSection)
    assert isinstance(inner.blocks[0], TabBlockEmpty)
    _roundtrip(tv)


def test_heading_block() -> None:
    tv = TabView(tab_name="t", blocks=[heading(level=2, text="Section")])
    assert isinstance(tv.blocks[0], TabBlockHeading)
    assert tv.blocks[0].level == 2
    _roundtrip(tv)


def test_heading_rejects_invalid_level() -> None:
    with pytest.raises(ValidationError):
        heading(level=9, text="bad")  # type: ignore[arg-type]


def test_text_block_default_muted_false() -> None:
    tv = TabView(tab_name="t", blocks=[text(text="hi")])
    assert isinstance(tv.blocks[0], TabBlockText)
    assert tv.blocks[0].muted is False
    _roundtrip(tv)


def test_kv_block_with_dicts_and_models() -> None:
    tv = TabView(
        tab_name="t",
        blocks=[
            kv(
                items=[
                    {"label": "a", "value": 1},
                    KVItem(label="b", value="two", hint="note"),
                ]
            )
        ],
    )
    block = tv.blocks[0]
    assert isinstance(block, TabBlockKV)
    assert block.items[0].value == 1
    assert block.items[1].hint == "note"
    _roundtrip(tv)


def test_metric_block_defaults() -> None:
    m = metric(label="Tasks", value=42)
    assert m.tone == "neutral"
    assert m.delta is None
    tv = TabView(tab_name="t", blocks=[m])
    _roundtrip(tv)


def test_metric_tone_is_enum_constrained() -> None:
    with pytest.raises(ValidationError):
        metric(label="x", value=1, tone="purple")  # type: ignore[arg-type]


def test_table_block_with_rows() -> None:
    tv = TabView(
        tab_name="t",
        blocks=[
            table_(
                columns=[
                    {"key": "name", "label": "Name"},
                    {"key": "score", "label": "Score", "align": "right"},
                ],
                rows=[
                    {"name": "alice", "score": 10},
                    {"name": "bob", "score": None},
                ],
                empty_text="no rows",
            )
        ],
    )
    block = tv.blocks[0]
    assert isinstance(block, TabBlockTable)
    assert block.columns[1].align == "right"
    assert block.rows[1]["score"] is None
    _roundtrip(tv)


def test_badge_list_block_default_tone() -> None:
    bl = badge_list(items=[{"label": "ok"}])
    assert isinstance(bl, TabBlockBadgeList)
    assert bl.items[0].tone == "neutral"
    _roundtrip(TabView(tab_name="t", blocks=[bl]))


def test_button_route_action() -> None:
    btn = button_route(label="Go", path="/projects")
    assert isinstance(btn, TabBlockButton)
    assert isinstance(btn.action, ActionRoute)
    assert btn.action.path == "/projects"
    _roundtrip(TabView(tab_name="t", blocks=[btn]))


def test_button_task_action_with_payload() -> None:
    btn = button_task(label="Run", worker="wk", payload={"n": 42})
    assert isinstance(btn.action, ActionTaskSubmit)
    assert btn.action.worker == "wk"
    assert btn.action.payload == {"n": 42}
    _roundtrip(TabView(tab_name="t", blocks=[btn]))


def test_chart_line_block() -> None:
    cl = chart_line(
        label="7d",
        points=[{"x": "mon", "y": 1}, {"x": "tue", "y": 2.5}],
        y_unit="req",
    )
    assert isinstance(cl, TabBlockChartLine)
    assert len(cl.points) == 2
    assert cl.points[1].y == 2.5
    _roundtrip(TabView(tab_name="t", blocks=[cl]))


def test_chart_bar_block() -> None:
    cb = chart_bar(
        label="Top",
        bars=[
            {"label": "a", "value": 10},
            {"label": "b", "value": 5, "tone": "warn"},
        ],
    )
    assert isinstance(cb, TabBlockChartBar)
    assert cb.bars[1].tone == "warn"
    _roundtrip(TabView(tab_name="t", blocks=[cb]))


def test_empty_block() -> None:
    e = empty(text="nothing")
    assert isinstance(e, TabBlockEmpty)
    assert e.text == "nothing"


# ---------------------------------------------------------------------------
# Block-level models: forbid extras, frozen (D1 contract)
# ---------------------------------------------------------------------------


def test_blocks_forbid_extra_fields() -> None:
    with pytest.raises(ValidationError):
        TabBlockMetric(label="x", value=1, extra="nope")  # type: ignore[call-arg]


def test_table_column_frozen() -> None:
    col = TableColumn(key="a", label="A")
    with pytest.raises(ValidationError):
        col.key = "b"  # type: ignore[misc]


def test_chart_line_point_requires_both_x_and_y() -> None:
    with pytest.raises(ValidationError):
        ChartLinePoint(x="mon")  # type: ignore[call-arg]


def test_badge_item_default_tone() -> None:
    bi = BadgeItem(label="neutral")
    assert bi.tone == "neutral"


def test_chart_bar_default_tone() -> None:
    b = ChartBar(label="x", value=1)
    assert b.tone == "neutral"


# ---------------------------------------------------------------------------
# Python snapshot — guards against drift in the Pydantic source of truth.
# This is a SINGLE-LANGUAGE check. The cross-language guard is
# tabview_canonical.json below, which is also parsed by Vitest.
# ---------------------------------------------------------------------------


def test_view_schema_stable_snapshot() -> None:
    """Python-side guard: TabView.model_json_schema() must stay
    byte-stable across refactors that don't intentionally bump the
    schema.

    This does NOT by itself prove cross-language agreement with the
    Zod schema in ``web/src/components/app/tabview/schema.ts``. The
    real cross-language guard is
    :func:`test_canonical_fixture_roundtrip` below, which shares
    ``tabview_canonical.json`` with a Vitest test that calls
    ``TabViewSchema.safeParse``.

    Any intentional schema change must (a) bump ``schema_version``,
    (b) be mirrored in the Zod schema on the frontend, (c)
    regenerate this snapshot explicitly, and (d) regenerate the
    canonical fixture.
    """
    actual = TabView.model_json_schema()

    if not SNAPSHOT_PATH.exists():
        SNAPSHOT_PATH.parent.mkdir(parents=True, exist_ok=True)
        SNAPSHOT_PATH.write_text(json.dumps(actual, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        pytest.fail(
            f"snapshot did not exist, wrote initial one at {SNAPSHOT_PATH} — re-run tests to verify",
        )

    expected = json.loads(SNAPSHOT_PATH.read_text(encoding="utf-8"))
    assert actual == expected, (
        "TabView JSON schema drifted from snapshot. If this change is "
        "intentional, bump schema_version, update the Zod mirror, and "
        f"regenerate {SNAPSHOT_PATH} by deleting it and re-running."
    )


# ---------------------------------------------------------------------------
# Cross-language canonical fixture — shared with the Vitest suite.
# ---------------------------------------------------------------------------


def test_canonical_fixture_roundtrip() -> None:
    """Cross-language guard for the TabView contract.

    Sprint 6 audit finding A-3 fix: the Python snapshot test above
    only checks that Pydantic agrees with its own stored schema dump.
    It did NOT check that the Zod mirror in
    ``web/src/components/app/tabview/schema.ts`` still accepts the
    same payloads.

    This test pins a canonical JSON payload that exercises every
    block kind (including edge cases — unicode, nested sections,
    negative deltas, null table cells, string + numeric metrics,
    button with task_submit + nested payload, chart with floats).
    The same file is imported by
    ``web/src/components/app/tabview/__tests__/cross_lang.test.ts``,
    which calls ``TabViewSchema.safeParse`` and asserts success.

    Both tests failing together signals a real cross-language drift;
    only one failing flags which side broke.

    Round-trip contract: ``TabView.model_validate(json).model_dump()``
    must equal the original JSON byte-for-byte (minus JSON
    whitespace).
    """
    assert CANONICAL_FIXTURE_PATH.exists(), (
        f"canonical fixture missing at {CANONICAL_FIXTURE_PATH} — "
        "regenerate via the helper invocation documented in the "
        "Sprint 6 audit findings A-3"
    )

    raw = CANONICAL_FIXTURE_PATH.read_text(encoding="utf-8")
    payload = json.loads(raw)

    validated = TabView.model_validate(payload)
    dumped = validated.model_dump()

    assert dumped == payload, (
        "canonical fixture failed Pydantic round-trip. A change in "
        "view.py (field rename, default, new kind) drifts from the "
        f"committed fixture at {CANONICAL_FIXTURE_PATH}. If "
        "intentional, regenerate the fixture AND the Vitest snapshot "
        "side so both languages stay aligned."
    )

    # Sanity check: the fixture actually exercises all 11 block kinds.
    # If this assertion fails, someone edited the fixture and dropped
    # coverage — regenerate with the full set.
    def _collect_kinds(blocks: list[Any], acc: set[str]) -> None:
        for block in blocks:
            if isinstance(block, dict) and "kind" in block:
                acc.add(block["kind"])
                if block["kind"] == "section":
                    _collect_kinds(block.get("blocks", []), acc)

    kinds: set[str] = set()
    _collect_kinds(payload["blocks"], kinds)
    expected_kinds = {
        "section",
        "heading",
        "text",
        "kv",
        "metric",
        "table",
        "badge_list",
        "button",
        "chart_line",
        "chart_bar",
        "empty",
    }
    assert kinds == expected_kinds, (
        "canonical fixture must exercise every block kind exactly. "
        f"Missing: {expected_kinds - kinds}. "
        f"Unknown: {kinds - expected_kinds}."
    )
