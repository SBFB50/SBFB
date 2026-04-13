# SPDX-License-Identifier: AGPL-3.0-or-later
"""TabView pre-render — convert a TabView descriptor to self-contained HTML.

Sprint 12 Phase B (D5 impl). When a coordinator publishes a project
with ``visibility=public``, every TabView tab is pre-rendered to a
standalone HTML page that the daemon serves via
``GET /blob-serve/{hash}/{path}``. Remote nodes render this HTML
inside a sandboxed iframe (``sandbox="allow-scripts"``, CSP
``connect-src 'none'``).

The renderer mirrors the React ``TabBlockRenderer`` switch and the
Tailwind classes used in ``web/src/components/app/tabview/blocks/``
— except that here every style is inlined via a ``<style>`` block
(no Tailwind runtime, no external CSS). The goal is pixel-parity
with the React shell for view-only blocks; interactive blocks
(``button``, ``file_upload``) degrade to a static placeholder
because the iframe cannot reach the coordinator API.

The renderer handles both v1 and v2 TabView descriptors: the block
union is determined by ``schema_version`` in the descriptor dict.
Unknown block kinds are silently ignored (forward-compat).
"""

from __future__ import annotations

import html
import math
from typing import Any

# ===================================================================
# Inline CSS — mirrors Tailwind tokens from web/src/index.css
# ===================================================================

_INLINE_CSS = """\
:root {
  --background: #0a0a0f;
  --foreground: #e2e4f0;
  --muted: #1a1a2e;
  --muted-foreground: #8b8fa3;
  --border: #2a2a3e;
  --primary: #6366f1;
  --destructive: #ef4444;
  --emerald: #34d399;
  --amber: #fbbf24;
}
*,*::before,*::after{box-sizing:border-box}
body{
  margin:0;padding:24px;
  background:var(--background);color:var(--foreground);
  font-family:ui-sans-serif,system-ui,sans-serif;
  font-size:14px;line-height:1.6;
}
.space-y>*+*{margin-top:12px}
h2,h3,h4{margin:0}
h2{font-size:1.25rem;font-weight:700;letter-spacing:-0.025em}
h3{font-size:0.875rem;font-weight:600}
h4{font-size:0.75rem;font-weight:600;text-transform:uppercase;color:var(--muted-foreground)}
p{margin:0;font-size:0.875rem;line-height:1.625}
.muted{color:var(--muted-foreground)}
.section{
  border:1px solid var(--border);border-radius:8px;
  background:color-mix(in srgb,var(--muted) 20%,transparent);
  padding:16px;
}
.section-title{
  margin-bottom:12px;font-size:0.75rem;font-weight:600;
  text-transform:uppercase;letter-spacing:0.05em;
  color:var(--muted-foreground);
}
.kv-grid{
  display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));
  gap:8px;
}
.kv-item{
  display:flex;flex-direction:column;
  border:1px solid var(--border);border-radius:6px;
  background:color-mix(in srgb,var(--background) 60%,transparent);
  padding:8px 12px;
}
.kv-label{
  font-size:10px;text-transform:uppercase;letter-spacing:0.05em;
  color:var(--muted-foreground);
}
.kv-value{font-family:ui-monospace,monospace;font-size:0.875rem}
.kv-hint{font-size:11px;color:var(--muted-foreground)}
.metric{
  border:1px solid var(--border);border-radius:8px;
  background:color-mix(in srgb,var(--background) 60%,transparent);
  padding:12px 16px;display:inline-block;min-width:140px;
}
.metric-label{
  font-size:10px;text-transform:uppercase;letter-spacing:0.05em;
  color:var(--muted-foreground);
}
.metric-value{font-size:1.5rem;font-weight:600}
.metric-unit{font-size:0.75rem;color:var(--muted-foreground);margin-left:4px}
.metric-delta{font-size:0.75rem;margin-left:8px}
.tone-neutral{color:var(--foreground)}
.tone-ok{color:var(--emerald)}
.tone-warn{color:var(--amber)}
.tone-danger{color:var(--destructive)}
.fill-neutral{fill:color-mix(in srgb,var(--primary) 70%,transparent)}
.fill-ok{fill:color-mix(in srgb,var(--emerald) 80%,transparent)}
.fill-warn{fill:color-mix(in srgb,var(--amber) 80%,transparent)}
.fill-danger{fill:color-mix(in srgb,var(--destructive) 80%,transparent)}
table{
  width:100%;border-collapse:collapse;font-size:0.875rem;
  border:1px solid var(--border);border-radius:6px;
  overflow:hidden;
}
thead{background:color-mix(in srgb,var(--muted) 40%,transparent)}
th{
  padding:8px 12px;text-align:left;
  font-size:10px;text-transform:uppercase;letter-spacing:0.05em;
  color:var(--muted-foreground);font-weight:600;
}
td{padding:8px 12px;border-top:1px solid color-mix(in srgb,var(--border) 60%,transparent)}
tr:nth-child(odd) td{background:color-mix(in srgb,var(--background) 40%,transparent)}
tr:nth-child(even) td{background:color-mix(in srgb,var(--background) 20%,transparent)}
th.right,td.right{text-align:right}
th.center,td.center{text-align:center}
.empty-text{
  font-size:0.75rem;font-style:italic;color:var(--muted-foreground);
  padding:24px 16px;text-align:center;
}
.badge-list{display:flex;flex-wrap:wrap;gap:8px}
.badge{
  display:inline-block;padding:2px 10px;border-radius:9999px;
  font-size:0.75rem;font-weight:500;
  border:1px solid var(--border);
}
.badge-ok{background:color-mix(in srgb,var(--emerald) 15%,transparent);color:var(--emerald);border-color:var(--emerald)}
.badge-warn{background:color-mix(in srgb,var(--amber) 15%,transparent);color:var(--amber);border-color:var(--amber)}
.badge-danger{background:color-mix(in srgb,var(--destructive) 15%,transparent);color:var(--destructive);border-color:var(--destructive)}
.btn{
  display:inline-block;padding:6px 16px;border-radius:6px;
  font-size:0.875rem;font-weight:500;cursor:default;
  border:1px solid var(--border);color:var(--foreground);
  background:var(--muted);opacity:0.6;
}
.chart-container{
  border:1px solid var(--border);border-radius:8px;
  background:color-mix(in srgb,var(--background) 60%,transparent);
  padding:12px;
}
.chart-label{
  font-size:0.75rem;font-weight:600;color:var(--muted-foreground);
  margin-bottom:8px;
}
.empty-box{
  border:1px dashed var(--border);border-radius:6px;
  background:color-mix(in srgb,var(--background) 30%,transparent);
  padding:24px 16px;text-align:center;
}
.file-upload-placeholder{
  border:2px dashed var(--border);border-radius:6px;
  padding:24px;text-align:center;min-height:80px;
  display:flex;align-items:center;justify-content:center;
  color:var(--muted-foreground);font-size:0.75rem;
}
svg text{font-family:ui-monospace,monospace}
"""

# ===================================================================
# Tone helpers
# ===================================================================

_TONE_CLASS = {
    "neutral": "tone-neutral",
    "ok": "tone-ok",
    "warn": "tone-warn",
    "danger": "tone-danger",
}

_FILL_CLASS = {
    "neutral": "fill-neutral",
    "ok": "fill-ok",
    "warn": "fill-warn",
    "danger": "fill-danger",
}


def _esc(value: Any) -> str:
    """HTML-escape a value, coercing to string first."""
    return html.escape(str(value))


# ===================================================================
# SVG chart helpers
# ===================================================================

_SVG_W = 400
_SVG_H = 120
_SVG_PAD_L = 32
_SVG_PAD_R = 32
_SVG_PAD_T = 16
_SVG_PAD_B = 16
# Bar charts use slightly larger top/bottom padding for label room.
_SVG_BAR_PAD_T = 24
_SVG_BAR_PAD_B = 24


def _render_chart_line_svg(points: list[dict[str, Any]], y_unit: str | None) -> str:
    """Render a line chart as an inline SVG string."""
    if not points:
        return ""

    ys = [float(p.get("y", 0)) for p in points]
    y_min = min(ys)
    y_max = max(ys)
    if y_max == y_min:
        y_max = y_min + 1

    n = len(points)
    plot_w = _SVG_W - _SVG_PAD_L - _SVG_PAD_R
    plot_h = _SVG_H - _SVG_PAD_T - _SVG_PAD_B

    def px(i: int) -> float:
        return _SVG_PAD_L + (i / max(n - 1, 1)) * plot_w

    def py(y: float) -> float:
        return _SVG_PAD_T + (1 - (y - y_min) / (y_max - y_min)) * plot_h

    # Grid lines (3 horizontal)
    grid_lines = []
    for step in range(4):
        gy = y_min + (y_max - y_min) * step / 3
        gy_px = py(gy)
        label = f"{gy:.0f}" if y_unit is None else f"{gy:.0f}{y_unit}"
        grid_lines.append(
            f'<line x1="{_SVG_PAD_L}" y1="{gy_px:.1f}" x2="{_SVG_W - _SVG_PAD_R}" '
            f'y2="{gy_px:.1f}" stroke="var(--border)" stroke-dasharray="4"/>'
            f'<text x="{_SVG_PAD_L - 4}" y="{gy_px + 3:.1f}" text-anchor="end" '
            f'fill="var(--muted-foreground)" font-size="9">{_esc(label)}</text>'
        )

    # Polyline path
    path_points = " ".join(f"{px(i):.1f},{py(ys[i]):.1f}" for i in range(n))

    # X labels (max 6)
    step = max(1, math.ceil(n / 6))
    x_labels = []
    for i in range(0, n, step):
        x_labels.append(
            f'<text x="{px(i):.1f}" y="{_SVG_H - 4}" text-anchor="middle" '
            f'fill="var(--muted-foreground)" font-size="9">{_esc(points[i].get("x", ""))}</text>'
        )

    # Circles
    circles = [f'<circle cx="{px(i):.1f}" cy="{py(ys[i]):.1f}" r="3" fill="var(--primary)"/>' for i in range(n)]

    return (
        f'<svg viewBox="0 0 {_SVG_W} {_SVG_H}" width="100%" '
        f'preserveAspectRatio="xMidYMid meet">'
        f"{''.join(grid_lines)}"
        f'<polyline points="{path_points}" fill="none" '
        f'stroke="var(--primary)" stroke-width="2" stroke-linejoin="round"/>'
        f"{''.join(circles)}"
        f"{''.join(x_labels)}"
        f"</svg>"
    )


def _render_chart_bar_svg(bars: list[dict[str, Any]]) -> str:
    """Render a bar chart as an inline SVG string."""
    if not bars:
        return ""

    values = [float(b.get("value", 0)) for b in bars]
    v_max = max(values) if values else 1
    if v_max == 0:
        v_max = 1

    n = len(bars)
    pad_t = _SVG_BAR_PAD_T
    pad_b = _SVG_BAR_PAD_B
    plot_w = _SVG_W - _SVG_PAD_L - _SVG_PAD_R
    plot_h = _SVG_H - pad_t - pad_b
    bar_w = plot_w / max(n, 1) * 0.7
    gap = plot_w / max(n, 1) * 0.3

    # Grid lines
    grid_lines = []
    for step in range(4):
        gv = v_max * step / 3
        gy = pad_t + (1 - gv / v_max) * plot_h
        grid_lines.append(
            f'<line x1="{_SVG_PAD_L}" y1="{gy:.1f}" x2="{_SVG_W - _SVG_PAD_R}" '
            f'y2="{gy:.1f}" stroke="var(--border)" stroke-dasharray="4"/>'
            f'<text x="{_SVG_PAD_L - 4}" y="{gy + 3:.1f}" text-anchor="end" '
            f'fill="var(--muted-foreground)" font-size="9">{gv:.0f}</text>'
        )

    rects = []
    x_labels = []
    for i, bar in enumerate(bars):
        v = float(bar.get("value", 0))
        tone = bar.get("tone", "neutral")
        fill_cls = _FILL_CLASS.get(tone, "fill-neutral")
        bx = _SVG_PAD_L + i * (bar_w + gap) + gap / 2
        bh = (v / v_max) * plot_h
        by = pad_t + plot_h - bh
        rects.append(
            f'<rect x="{bx:.1f}" y="{by:.1f}" width="{bar_w:.1f}" height="{bh:.1f}" rx="3" class="{fill_cls}"/>'
        )
        x_labels.append(
            f'<text x="{bx + bar_w / 2:.1f}" y="{_SVG_H - 4}" text-anchor="middle" '
            f'fill="var(--muted-foreground)" font-size="9">{_esc(bar.get("label", ""))}</text>'
        )

    return (
        f'<svg viewBox="0 0 {_SVG_W} {_SVG_H}" width="100%" '
        f'preserveAspectRatio="xMidYMid meet">'
        f"{''.join(grid_lines)}"
        f"{''.join(rects)}"
        f"{''.join(x_labels)}"
        f"</svg>"
    )


# ===================================================================
# Block renderers
# ===================================================================


def _render_heading(block: dict[str, Any]) -> str:
    level = block.get("level", 2)
    text = _esc(block.get("text", ""))
    tag = {1: "h2", 2: "h3", 3: "h4"}.get(level, "h3")
    return f"<{tag}>{text}</{tag}>"


def _render_text(block: dict[str, Any]) -> str:
    cls = ' class="muted"' if block.get("muted") else ""
    return f"<p{cls}>{_esc(block.get('text', ''))}</p>"


def _render_kv(block: dict[str, Any]) -> str:
    items = block.get("items", [])
    parts = []
    for item in items:
        label = _esc(item.get("label", ""))
        value = _esc(item.get("value", ""))
        hint = item.get("hint")
        hint_html = f'<span class="kv-hint">{_esc(hint)}</span>' if hint else ""
        parts.append(
            f'<div class="kv-item">'
            f'<span class="kv-label">{label}</span>'
            f'<span class="kv-value">{value}</span>'
            f"{hint_html}"
            f"</div>"
        )
    return f'<dl class="kv-grid">{"".join(parts)}</dl>'


def _render_metric(block: dict[str, Any]) -> str:
    tone = block.get("tone", "neutral")
    tone_cls = _TONE_CLASS.get(tone, "tone-neutral")
    label = _esc(block.get("label", ""))
    value = _esc(block.get("value", ""))
    unit = block.get("unit")
    delta = block.get("delta")

    unit_html = f'<span class="metric-unit">{_esc(unit)}</span>' if unit else ""
    delta_html = ""
    if delta is not None:
        d = float(delta)
        sign = "+" if d > 0 else ""
        d_cls = "tone-ok" if d > 0 else "tone-danger" if d < 0 else "muted"
        delta_html = f'<span class="metric-delta {d_cls}">{sign}{d:g}</span>'

    return (
        f'<div class="metric">'
        f'<div class="metric-label">{label}</div>'
        f'<div><span class="metric-value {tone_cls}">{value}</span>{unit_html}{delta_html}</div>'
        f"</div>"
    )


def _render_table(block: dict[str, Any]) -> str:
    columns = block.get("columns", [])
    rows = block.get("rows", [])
    empty_text = block.get("empty_text")

    if not rows and empty_text:
        return f'<div class="empty-text">{_esc(empty_text)}</div>'

    # Header
    ths = []
    for col in columns:
        align = col.get("align", "left")
        cls = f' class="{align}"' if align != "left" else ""
        ths.append(f"<th{cls}>{_esc(col.get('label', ''))}</th>")

    # Rows
    trs = []
    for row in rows:
        tds = []
        for col in columns:
            key = col.get("key", "")
            align = col.get("align", "left")
            cls = f' class="{align}"' if align != "left" else ""
            cell = row.get(key, "")
            tds.append(f"<td{cls}>{_esc(cell) if cell is not None else ''}</td>")
        trs.append(f"<tr>{''.join(tds)}</tr>")

    return (
        f'<div style="overflow-x:auto">'
        f"<table>"
        f"<thead><tr>{''.join(ths)}</tr></thead>"
        f"<tbody>{''.join(trs)}</tbody>"
        f"</table></div>"
    )


def _render_badge_list(block: dict[str, Any]) -> str:
    items = block.get("items", [])
    badges = []
    for item in items:
        tone = item.get("tone", "neutral")
        cls = f"badge badge-{tone}" if tone != "neutral" else "badge"
        badges.append(f'<span class="{cls}">{_esc(item.get("label", ""))}</span>')
    return f'<div class="badge-list">{"".join(badges)}</div>'


def _render_button(block: dict[str, Any]) -> str:
    label = _esc(block.get("label", ""))
    return f'<div><span class="btn" title="Non disponible en mode lecture seule">{label}</span></div>'


def _render_chart_line(block: dict[str, Any]) -> str:
    label = _esc(block.get("label", ""))
    points = block.get("points", [])
    y_unit = block.get("y_unit")
    svg = _render_chart_line_svg(points, y_unit)
    return f'<div class="chart-container"><div class="chart-label">{label}</div>{svg}</div>'


def _render_chart_bar(block: dict[str, Any]) -> str:
    label = _esc(block.get("label", ""))
    bars = block.get("bars", [])
    svg = _render_chart_bar_svg(bars)
    return f'<div class="chart-container"><div class="chart-label">{label}</div>{svg}</div>'


def _render_empty(block: dict[str, Any]) -> str:
    return f'<div class="empty-box"><span class="empty-text">{_esc(block.get("text", ""))}</span></div>'


def _render_file_upload(block: dict[str, Any]) -> str:
    label = _esc(block.get("label", ""))
    return f'<div class="file-upload-placeholder">{label} (upload non disponible)</div>'


def _render_section(block: dict[str, Any]) -> str:
    title = block.get("title")
    blocks = block.get("blocks", [])
    title_html = f'<div class="section-title">{_esc(title)}</div>' if title else ""
    inner = _render_blocks(blocks)
    return f'<div class="section">{title_html}{inner}</div>'


# ===================================================================
# Block dispatcher
# ===================================================================

_RENDERERS: dict[str, Any] = {
    "heading": _render_heading,
    "text": _render_text,
    "kv": _render_kv,
    "metric": _render_metric,
    "table": _render_table,
    "badge_list": _render_badge_list,
    "button": _render_button,
    "chart_line": _render_chart_line,
    "chart_bar": _render_chart_bar,
    "empty": _render_empty,
    "section": _render_section,
    "file_upload": _render_file_upload,
}


def _render_block(block: dict[str, Any]) -> str:
    """Render a single block dict to HTML."""
    kind = block.get("kind", "")
    renderer = _RENDERERS.get(kind)
    if renderer is None:
        return ""
    return renderer(block)


def _render_blocks(blocks: list[dict[str, Any]]) -> str:
    """Render a list of blocks into a space-y container."""
    parts = [_render_block(b) for b in blocks if b]
    parts = [p for p in parts if p]
    if not parts:
        return ""
    return f'<div class="space-y">{"".join(parts)}</div>'


# ===================================================================
# Public API
# ===================================================================


def render_tabview_to_html(
    descriptor: dict[str, Any],
    *,
    title: str = "SBFB App",
) -> str:
    """Render a TabView descriptor dict to a self-contained HTML page.

    Parameters
    ----------
    descriptor:
        A TabView descriptor dict (as returned by
        ``TabView.model_dump(mode="json")``). Must contain a
        ``blocks`` list.
    title:
        The ``<title>`` of the generated HTML page.

    Returns
    -------
    str:
        A complete HTML document with inline CSS, ready to be
        served by the daemon's ``GET /blob-serve/{hash}/{path}``
        endpoint.
    """
    blocks = descriptor.get("blocks", [])
    blocks_html = _render_blocks(blocks)
    escaped_title = _esc(title)

    return (
        f"<!DOCTYPE html>\n"
        f'<html lang="fr">\n'
        f"<head>\n"
        f'  <meta charset="utf-8">\n'
        f'  <meta name="viewport" content="width=device-width, initial-scale=1">\n'
        f"  <title>{escaped_title}</title>\n"
        f"  <style>{_INLINE_CSS}</style>\n"
        f"</head>\n"
        f"<body>\n"
        f"  {blocks_html}\n"
        f"</body>\n"
        f"</html>"
    )


__all__ = [
    "render_tabview_to_html",
]
