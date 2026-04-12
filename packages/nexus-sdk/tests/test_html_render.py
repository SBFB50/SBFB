# SPDX-License-Identifier: AGPL-3.0-or-later
"""Tests for :mod:`nexus_sdk.html_render` (Sprint 12 Phase B).

12 scenarios covering every block kind in the TabView HTML
pre-renderer. Each test verifies that the generated HTML contains
the expected structural elements and content.
"""

from __future__ import annotations

from nexus_sdk.html_render import render_tabview_to_html

# ---------------------------------------------------------------
# Helper — build a minimal descriptor dict
# ---------------------------------------------------------------


def _descriptor(blocks: list) -> dict:
    return {"schema_version": 1, "tab_name": "test", "blocks": blocks}


# ---------------------------------------------------------------
# 1. Heading levels
# ---------------------------------------------------------------


def test_render_heading_levels():
    desc = _descriptor(
        [
            {"kind": "heading", "level": 1, "text": "Titre principal"},
            {"kind": "heading", "level": 2, "text": "Sous-titre"},
            {"kind": "heading", "level": 3, "text": "Section"},
        ]
    )
    html = render_tabview_to_html(desc)
    assert "<h2>Titre principal</h2>" in html
    assert "<h3>Sous-titre</h3>" in html
    assert "<h4>Section</h4>" in html


# ---------------------------------------------------------------
# 2. Text with muted
# ---------------------------------------------------------------


def test_render_text_muted():
    desc = _descriptor(
        [
            {"kind": "text", "text": "Normal text"},
            {"kind": "text", "text": "Muted text", "muted": True},
        ]
    )
    html = render_tabview_to_html(desc)
    assert "<p>Normal text</p>" in html
    assert 'class="muted"' in html
    assert "Muted text" in html


# ---------------------------------------------------------------
# 3. KV items
# ---------------------------------------------------------------


def test_render_kv_items():
    desc = _descriptor(
        [
            {
                "kind": "kv",
                "items": [
                    {"label": "Version", "value": "1.0.0"},
                    {"label": "Status", "value": "ok", "hint": "All green"},
                ],
            }
        ]
    )
    html = render_tabview_to_html(desc)
    assert "Version" in html
    assert "1.0.0" in html
    assert "Status" in html
    assert "All green" in html
    assert "kv-grid" in html


# ---------------------------------------------------------------
# 4. Metric with tones
# ---------------------------------------------------------------


def test_render_metric_tones():
    desc = _descriptor(
        [
            {"kind": "metric", "label": "CPU", "value": "42%", "tone": "ok", "delta": 5},
            {"kind": "metric", "label": "Errors", "value": 12, "tone": "danger", "unit": "/s"},
        ]
    )
    html = render_tabview_to_html(desc)
    assert "tone-ok" in html
    assert "tone-danger" in html
    assert "CPU" in html
    assert "42%" in html
    assert "+5" in html
    assert "/s" in html


# ---------------------------------------------------------------
# 5. Table with columns and rows
# ---------------------------------------------------------------


def test_render_table():
    desc = _descriptor(
        [
            {
                "kind": "table",
                "columns": [
                    {"key": "name", "label": "Nom"},
                    {"key": "score", "label": "Score", "align": "right"},
                ],
                "rows": [
                    {"name": "Alice", "score": 95},
                    {"name": "Bob", "score": 87},
                ],
            }
        ]
    )
    html = render_tabview_to_html(desc)
    assert "<table>" in html
    assert "Nom" in html
    assert "Score" in html
    assert "Alice" in html
    assert "87" in html
    assert 'class="right"' in html


# ---------------------------------------------------------------
# 6. Table empty_text fallback
# ---------------------------------------------------------------


def test_render_table_empty_text():
    desc = _descriptor(
        [
            {
                "kind": "table",
                "columns": [{"key": "x", "label": "X"}],
                "rows": [],
                "empty_text": "Aucune donnee",
            }
        ]
    )
    html = render_tabview_to_html(desc)
    assert "Aucune donnee" in html
    assert "<table>" not in html


# ---------------------------------------------------------------
# 7. Chart line SVG
# ---------------------------------------------------------------


def test_render_chart_line_svg():
    desc = _descriptor(
        [
            {
                "kind": "chart_line",
                "label": "Tendance",
                "points": [
                    {"x": "Jan", "y": 10},
                    {"x": "Fev", "y": 20},
                    {"x": "Mar", "y": 15},
                ],
                "y_unit": "%",
            }
        ]
    )
    html = render_tabview_to_html(desc)
    assert "<svg" in html
    assert "polyline" in html
    assert "Tendance" in html
    assert "<circle" in html


# ---------------------------------------------------------------
# 8. Chart bar SVG
# ---------------------------------------------------------------


def test_render_chart_bar_svg():
    desc = _descriptor(
        [
            {
                "kind": "chart_bar",
                "label": "Repartition",
                "bars": [
                    {"label": "A", "value": 30, "tone": "ok"},
                    {"label": "B", "value": 70, "tone": "warn"},
                ],
            }
        ]
    )
    html = render_tabview_to_html(desc)
    assert "<svg" in html
    assert "<rect" in html
    assert "fill-ok" in html
    assert "fill-warn" in html


# ---------------------------------------------------------------
# 9. Section recursive
# ---------------------------------------------------------------


def test_render_section_recursive():
    desc = _descriptor(
        [
            {
                "kind": "section",
                "title": "Bloc parent",
                "blocks": [
                    {"kind": "heading", "level": 2, "text": "Enfant"},
                    {"kind": "text", "text": "Contenu enfant"},
                ],
            }
        ]
    )
    html = render_tabview_to_html(desc)
    assert "Bloc parent" in html
    assert "section-title" in html
    assert "<h3>Enfant</h3>" in html
    assert "Contenu enfant" in html


# ---------------------------------------------------------------
# 10. Full TabView roundtrip
# ---------------------------------------------------------------


def test_render_full_tabview():
    desc = _descriptor(
        [
            {"kind": "heading", "level": 1, "text": "Dashboard"},
            {"kind": "metric", "label": "Total", "value": 100, "tone": "neutral"},
            {"kind": "empty", "text": "Rien a afficher"},
        ]
    )
    html = render_tabview_to_html(desc, title="Mon App")
    assert "<!DOCTYPE html>" in html
    assert "<title>Mon App</title>" in html
    assert "Dashboard" in html
    assert "empty-box" in html
    assert "Rien a afficher" in html


# ---------------------------------------------------------------
# 11. Badge list
# ---------------------------------------------------------------


def test_render_badge_list():
    desc = _descriptor(
        [
            {
                "kind": "badge_list",
                "items": [
                    {"label": "Stable", "tone": "ok"},
                    {"label": "Beta", "tone": "warn"},
                    {"label": "Critique", "tone": "danger"},
                    {"label": "Normal"},
                ],
            }
        ]
    )
    html = render_tabview_to_html(desc)
    assert "badge-ok" in html
    assert "badge-warn" in html
    assert "badge-danger" in html
    assert "Stable" in html


# ---------------------------------------------------------------
# 12. Button placeholder
# ---------------------------------------------------------------


def test_render_button_placeholder():
    desc = _descriptor(
        [
            {
                "kind": "button",
                "label": "Soumettre",
                "action": {"kind": "route", "path": "/submit"},
            }
        ]
    )
    html = render_tabview_to_html(desc)
    assert "Soumettre" in html
    assert "btn" in html
    assert "lecture seule" in html


# ---------------------------------------------------------------
# 13. HTML escaping
# ---------------------------------------------------------------


def test_html_escaping():
    desc = _descriptor(
        [
            {"kind": "text", "text": "<script>alert('xss')</script>"},
        ]
    )
    html = render_tabview_to_html(desc)
    assert "<script>" not in html
    assert "&lt;script&gt;" in html
