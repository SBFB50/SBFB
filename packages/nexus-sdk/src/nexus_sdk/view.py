"""TabView — schema-driven tab rendering vocabulary.

Sprint 6 D1: every ``@nexus_tab`` descriptor returns a
:class:`TabView` v1 instead of an app-specific dict. The React
shell renders the returned blocks via a fixed switch over
``kind``, so apps never ship custom React.

The vocabulary covers view-centric blocks (dashboards, tables,
metrics, minimal SVG charts) not forms. RJSF-style form
rendering stays out of scope for v1 — if an app needs forms it
can route the user to a native coordinator endpoint.

Versioning: :attr:`TabView.schema_version` is a frozen literal
``1``. Any vocabulary extension is a breaking change that must
bump the literal in lockstep with the Zod mirror in
``web/src/components/app/tabview/schema.ts``.

Helpers: prefer the constructor helpers (``section``, ``metric``,
``table_``) over building ``TabBlock*`` dicts by hand — they
type-check the arguments at construction time and produce a
validated instance.
"""

from __future__ import annotations

from typing import Annotated, Any, Literal, Union

from pydantic import BaseModel, ConfigDict, Field

BlockTone = Literal["neutral", "ok", "warn", "danger"]
TableAlign = Literal["left", "right", "center"]
HeadingLevel = Literal[1, 2, 3]


class _Block(BaseModel):
    """Common base for block models: frozen, strict, forbid extras."""

    model_config = ConfigDict(frozen=True, extra="forbid")


class TabBlockSection(_Block):
    kind: Literal["section"] = "section"
    title: str | None = None
    blocks: list["TabBlock"] = Field(default_factory=list)


class TabBlockHeading(_Block):
    kind: Literal["heading"] = "heading"
    level: HeadingLevel
    text: str


class TabBlockText(_Block):
    kind: Literal["text"] = "text"
    text: str
    muted: bool = False


class KVItem(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")
    label: str
    value: str | int | float
    hint: str | None = None


class TabBlockKV(_Block):
    kind: Literal["kv"] = "kv"
    items: list[KVItem]


class TabBlockMetric(_Block):
    kind: Literal["metric"] = "metric"
    label: str
    value: str | int | float
    delta: int | float | None = None
    unit: str | None = None
    tone: BlockTone = "neutral"


class TableColumn(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")
    key: str
    label: str
    align: TableAlign = "left"


class TabBlockTable(_Block):
    kind: Literal["table"] = "table"
    columns: list[TableColumn]
    rows: list[dict[str, str | int | float | None]]
    empty_text: str | None = None


class BadgeItem(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")
    label: str
    tone: BlockTone = "neutral"


class TabBlockBadgeList(_Block):
    kind: Literal["badge_list"] = "badge_list"
    items: list[BadgeItem]


class ActionRoute(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")
    kind: Literal["route"] = "route"
    path: str


class ActionTaskSubmit(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")
    kind: Literal["task_submit"] = "task_submit"
    worker: str
    payload: Any = None


ButtonAction = Annotated[
    Union[ActionRoute, ActionTaskSubmit],
    Field(discriminator="kind"),
]


class TabBlockButton(_Block):
    kind: Literal["button"] = "button"
    label: str
    action: ButtonAction
    tone: BlockTone = "neutral"


class ChartLinePoint(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")
    x: str
    y: int | float


class TabBlockChartLine(_Block):
    kind: Literal["chart_line"] = "chart_line"
    label: str
    points: list[ChartLinePoint]
    y_unit: str | None = None


class ChartBar(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")
    label: str
    value: int | float
    tone: BlockTone = "neutral"


class TabBlockChartBar(_Block):
    kind: Literal["chart_bar"] = "chart_bar"
    label: str
    bars: list[ChartBar]


class TabBlockEmpty(_Block):
    kind: Literal["empty"] = "empty"
    text: str


class TabBlockFileUpload(_Block):
    """File upload dropzone block — v2 only.

    Sprint 9 Phase E (D3 impl). Renders a drag-and-drop zone that
    posts to ``POST /app/{name}/files/upload`` and shows progress
    via the D2 SSE bridge. Only valid inside a ``schema_version: 2``
    :class:`TabViewV2` — a v1 parser will reject it via
    ``extra="forbid"`` on the v1 discriminated union.
    """

    kind: Literal["file_upload"] = "file_upload"
    label: str
    accept: list[str] = Field(default_factory=lambda: ["image/*", "application/pdf"])
    max_size_bytes: int = 50 * 1024 * 1024


# -- v1 block union (excludes file_upload) --

TabBlockV1 = Annotated[
    Union[
        TabBlockSection,
        TabBlockHeading,
        TabBlockText,
        TabBlockKV,
        TabBlockMetric,
        TabBlockTable,
        TabBlockBadgeList,
        TabBlockButton,
        TabBlockChartLine,
        TabBlockChartBar,
        TabBlockEmpty,
    ],
    Field(discriminator="kind"),
]

# -- v2 block union (includes file_upload) --

TabBlockV2 = Annotated[
    Union[
        TabBlockSection,
        TabBlockHeading,
        TabBlockText,
        TabBlockKV,
        TabBlockMetric,
        TabBlockTable,
        TabBlockBadgeList,
        TabBlockButton,
        TabBlockChartLine,
        TabBlockChartBar,
        TabBlockEmpty,
        TabBlockFileUpload,
    ],
    Field(discriminator="kind"),
]

# Backward compat alias — existing code uses ``TabBlock``
TabBlock = TabBlockV1


TabBlockSection.model_rebuild()


class TabView(BaseModel):
    """Top-level v1 descriptor returned by a ``@nexus_tab`` method.

    Kept as ``TabView`` (no ``V1`` suffix) for backward compat with
    existing app code. The ``schema_version`` literal is ``1``.
    """

    model_config = ConfigDict(frozen=True, extra="forbid")

    schema_version: Literal[1] = 1
    tab_name: str
    title: str | None = None
    blocks: list[TabBlock] = Field(default_factory=list)


# Alias for explicit v1 references.
TabViewV1 = TabView


class TabViewV2(BaseModel):
    """Top-level v2 descriptor — adds ``file_upload`` block kind.

    Sprint 9 Phase E. Backward-compatible with v1: every v1
    descriptor validates as v2 (the ``file_upload`` block is
    optional), but a v2 descriptor with ``file_upload`` blocks
    rejects under the v1 parser via ``extra="forbid"`` on block
    discriminators.
    """

    model_config = ConfigDict(frozen=True, extra="forbid")

    schema_version: Literal[2] = 2
    tab_name: str
    title: str | None = None
    blocks: list[TabBlockV2] = Field(default_factory=list)


AnyTabView = Annotated[
    Union[TabViewV1, TabViewV2],
    Field(discriminator="schema_version"),
]


# ---------------------------------------------------------------------------
# Constructor helpers
# ---------------------------------------------------------------------------


def section(*, title: str | None = None, blocks: list[TabBlock] | None = None) -> TabBlockSection:
    return TabBlockSection(title=title, blocks=list(blocks or []))


def heading(*, level: HeadingLevel, text: str) -> TabBlockHeading:
    return TabBlockHeading(level=level, text=text)


def text(*, text: str, muted: bool = False) -> TabBlockText:  # noqa: A002 — "text" is the natural keyword
    return TabBlockText(text=text, muted=muted)


def kv(*, items: list[dict[str, Any]] | list[KVItem]) -> TabBlockKV:
    validated = [item if isinstance(item, KVItem) else KVItem(**item) for item in items]
    return TabBlockKV(items=validated)


def metric(
    *,
    label: str,
    value: str | int | float,
    delta: int | float | None = None,
    unit: str | None = None,
    tone: BlockTone = "neutral",
) -> TabBlockMetric:
    return TabBlockMetric(label=label, value=value, delta=delta, unit=unit, tone=tone)


def table_(
    *,
    columns: list[dict[str, Any]] | list[TableColumn],
    rows: list[dict[str, str | int | float | None]],
    empty_text: str | None = None,
) -> TabBlockTable:
    validated_cols = [c if isinstance(c, TableColumn) else TableColumn(**c) for c in columns]
    return TabBlockTable(columns=validated_cols, rows=list(rows), empty_text=empty_text)


def badge_list(*, items: list[dict[str, Any]] | list[BadgeItem]) -> TabBlockBadgeList:
    validated = [item if isinstance(item, BadgeItem) else BadgeItem(**item) for item in items]
    return TabBlockBadgeList(items=validated)


def button_route(*, label: str, path: str, tone: BlockTone = "neutral") -> TabBlockButton:
    return TabBlockButton(label=label, action=ActionRoute(path=path), tone=tone)


def button_task(
    *,
    label: str,
    worker: str,
    payload: Any = None,
    tone: BlockTone = "neutral",
) -> TabBlockButton:
    return TabBlockButton(
        label=label,
        action=ActionTaskSubmit(worker=worker, payload=payload),
        tone=tone,
    )


def chart_line(
    *,
    label: str,
    points: list[dict[str, Any]] | list[ChartLinePoint],
    y_unit: str | None = None,
) -> TabBlockChartLine:
    validated = [p if isinstance(p, ChartLinePoint) else ChartLinePoint(**p) for p in points]
    return TabBlockChartLine(label=label, points=validated, y_unit=y_unit)


def chart_bar(
    *,
    label: str,
    bars: list[dict[str, Any]] | list[ChartBar],
) -> TabBlockChartBar:
    validated = [b if isinstance(b, ChartBar) else ChartBar(**b) for b in bars]
    return TabBlockChartBar(label=label, bars=validated)


def empty(*, text: str) -> TabBlockEmpty:  # noqa: A002
    return TabBlockEmpty(text=text)


def file_upload_block(
    *,
    label: str,
    accept: list[str] | None = None,
    max_size_bytes: int = 50 * 1024 * 1024,
) -> TabBlockFileUpload:
    """Construct a ``file_upload`` block (v2 only)."""
    return TabBlockFileUpload(
        label=label,
        accept=accept if accept is not None else ["image/*", "application/pdf"],
        max_size_bytes=max_size_bytes,
    )


__all__ = [
    "ActionRoute",
    "ActionTaskSubmit",
    "AnyTabView",
    "BadgeItem",
    "BlockTone",
    "ButtonAction",
    "ChartBar",
    "ChartLinePoint",
    "HeadingLevel",
    "KVItem",
    "TabBlock",
    "TabBlockBadgeList",
    "TabBlockButton",
    "TabBlockChartBar",
    "TabBlockChartLine",
    "TabBlockEmpty",
    "TabBlockFileUpload",
    "TabBlockHeading",
    "TabBlockKV",
    "TabBlockMetric",
    "TabBlockSection",
    "TabBlockTable",
    "TabBlockText",
    "TabBlockV1",
    "TabBlockV2",
    "TableAlign",
    "TableColumn",
    "TabView",
    "TabViewV1",
    "TabViewV2",
    "badge_list",
    "button_route",
    "button_task",
    "chart_bar",
    "chart_line",
    "empty",
    "file_upload_block",
    "heading",
    "kv",
    "metric",
    "section",
    "table_",
    "text",
]
