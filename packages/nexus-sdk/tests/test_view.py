"""Unit tests for the Sprint 6 TabView schema and helpers."""

from __future__ import annotations

import json
from pathlib import Path

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
# Cross-language snapshot — must match web/src/components/app/tabview/schema.ts
# ---------------------------------------------------------------------------


def test_view_schema_stable_snapshot() -> None:
    """Guard against accidental drift in the Python side.

    The snapshot captures Pydantic's JSON-schema dump of
    :class:`TabView`. Any intentional schema change must (a)
    bump ``schema_version``, (b) be mirrored in the Zod schema
    on the frontend, and (c) regenerate this snapshot explicitly.
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
