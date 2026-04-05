"""
NEXUS -- Hypothesis evolution charts (Plotly).

Provides multi-line and single-hypothesis score evolution charts
with coloured confidence zones and hover details.
"""

from __future__ import annotations

from typing import Any, Dict, List

import plotly.graph_objects as go
import streamlit as st


# ---------------------------------------------------------------
# Colour palette for hypothesis traces
# ---------------------------------------------------------------

TRACE_COLORS = [
    "#4A90D9", "#E74C3C", "#27AE60", "#E67E22", "#8E44AD",
    "#1ABC9C", "#F1C40F", "#C0392B", "#2980B9", "#D35400",
]

# Zone thresholds & colours
ZONE_HIGH = 70   # >= 70: strong hypothesis
ZONE_MED = 40    # >= 40: moderate
# < 40: weak


def render_evolution_chart(hypotheses_data: List[Dict[str, Any]]) -> None:
    """Multi-line chart showing score evolution for several hypotheses.

    Parameters
    ----------
    hypotheses_data : list[dict]
        Each entry: ``{hypothesis_id, title, snapshots: [{date, score, trigger}]}``.
    """
    if not hypotheses_data:
        st.info("Aucune donnee d'evolution disponible.")
        return

    fig = go.Figure()

    for idx, hyp in enumerate(hypotheses_data):
        snapshots = hyp.get("snapshots", [])
        if not snapshots:
            continue

        dates = [s["date"] for s in snapshots]
        scores = [s["score"] for s in snapshots]
        triggers = [s.get("trigger", "N/A") for s in snapshots]
        color = TRACE_COLORS[idx % len(TRACE_COLORS)]

        hover_text = [
            f"Score: {s:.1f}<br>Trigger: {t}<br>Date: {d}"
            for s, t, d in zip(scores, triggers, dates)
        ]

        fig.add_trace(go.Scatter(
            x=dates,
            y=scores,
            mode="lines+markers",
            name=_truncate(hyp.get("title", f"Hypothese {idx+1}"), 40),
            line=dict(color=color, width=2),
            marker=dict(size=6),
            hovertext=hover_text,
            hoverinfo="text",
        ))

    # Coloured zones
    fig.add_hrect(y0=ZONE_HIGH, y1=100, fillcolor="#27AE60", opacity=0.08,
                  line_width=0, annotation_text="Forte", annotation_position="top left")
    fig.add_hrect(y0=ZONE_MED, y1=ZONE_HIGH, fillcolor="#F39C12", opacity=0.08,
                  line_width=0, annotation_text="Moderee", annotation_position="top left")
    fig.add_hrect(y0=0, y1=ZONE_MED, fillcolor="#E74C3C", opacity=0.08,
                  line_width=0, annotation_text="Faible", annotation_position="top left")

    fig.update_layout(
        title="Evolution des scores d'hypotheses",
        xaxis_title="Date",
        yaxis_title="Score",
        yaxis=dict(range=[0, 100], dtick=10),
        legend=dict(orientation="h", yanchor="bottom", y=-0.3),
        height=450,
        template="plotly_white",
        margin=dict(l=50, r=20, t=50, b=80),
    )

    st.plotly_chart(fig, use_container_width=True)


def render_single_hypothesis_chart(
    evolution_data: List[Dict[str, Any]],
    title: str,
) -> None:
    """Chart for a single hypothesis with coloured confidence zones.

    Parameters
    ----------
    evolution_data : list[dict]
        ``[{date, score, trigger, model_used}]`` sorted chronologically.
    title : str
        Hypothesis title shown above the chart.
    """
    if not evolution_data:
        st.info("Aucun historique de score pour cette hypothese.")
        return

    dates = [s["date"] for s in evolution_data]
    scores = [s["score"] for s in evolution_data]
    triggers = [s.get("trigger", "N/A") for s in evolution_data]
    models = [s.get("model_used", "N/A") for s in evolution_data]

    hover_text = [
        f"Score: {sc:.1f}<br>Trigger: {tr}<br>Modele: {mo}<br>Date: {dt}"
        for sc, tr, mo, dt in zip(scores, triggers, models, dates)
    ]

    fig = go.Figure()

    # Coloured background zones
    fig.add_hrect(y0=ZONE_HIGH, y1=100, fillcolor="#27AE60", opacity=0.10, line_width=0)
    fig.add_hrect(y0=ZONE_MED, y1=ZONE_HIGH, fillcolor="#F39C12", opacity=0.10, line_width=0)
    fig.add_hrect(y0=0, y1=ZONE_MED, fillcolor="#E74C3C", opacity=0.10, line_width=0)

    # Main score line
    fig.add_trace(go.Scatter(
        x=dates,
        y=scores,
        mode="lines+markers",
        name="Score",
        line=dict(color="#4A90D9", width=3),
        marker=dict(size=8, color=_score_colors(scores)),
        hovertext=hover_text,
        hoverinfo="text",
        fill="tozeroy",
        fillcolor="rgba(74, 144, 217, 0.08)",
    ))

    # Annotations for trigger events
    for i, (d, s, t) in enumerate(zip(dates, scores, triggers)):
        if t and t not in ("N/A", "manual"):
            fig.add_annotation(
                x=d, y=s,
                text=t,
                showarrow=True,
                arrowhead=2,
                arrowsize=0.8,
                ax=0, ay=-30,
                font=dict(size=9, color="#555"),
            )

    fig.update_layout(
        title=_truncate(title, 60),
        xaxis_title="Date",
        yaxis_title="Score",
        yaxis=dict(range=[0, 100], dtick=10),
        height=380,
        template="plotly_white",
        showlegend=False,
        margin=dict(l=50, r=20, t=50, b=40),
    )

    st.plotly_chart(fig, use_container_width=True)


# ---------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------

def _score_colors(scores: List[float]) -> List[str]:
    """Map each score to a colour based on zone thresholds."""
    colors = []
    for s in scores:
        if s >= ZONE_HIGH:
            colors.append("#27AE60")
        elif s >= ZONE_MED:
            colors.append("#F39C12")
        else:
            colors.append("#E74C3C")
    return colors


def _truncate(text: str, max_len: int) -> str:
    """Truncate text with ellipsis if too long."""
    if len(text) <= max_len:
        return text
    return text[: max_len - 3] + "..."
