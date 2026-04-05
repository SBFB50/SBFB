"""
NEXUS -- Timeline page.

Interactive Plotly scatter plot showing chronological events across
the case, with type-based filtering and clickable point details.
"""

from __future__ import annotations
import sys; from pathlib import Path; sys.path.insert(0, str(Path(__file__).resolve().parent.parent)) if str(Path(__file__).resolve().parent.parent) not in sys.path else None  # noqa: E402

from datetime import datetime

import plotly.graph_objects as go
import streamlit as st

from frontend.api_client import api

# ---------------------------------------------------------------------------
# Guard
# ---------------------------------------------------------------------------

st.title("Timeline")

case_id = st.session_state.get("case_id")
if not case_id:
    st.info("Selectionnez ou creez un dossier dans la barre laterale.")
    st.stop()

# ---------------------------------------------------------------------------
# Fetch timeline data
# ---------------------------------------------------------------------------

timeline = api.get_timeline(case_id)

if not timeline:
    st.caption("Aucun evenement dans la timeline pour ce dossier.")
    st.stop()

# ---------------------------------------------------------------------------
# Type colours and labels
# ---------------------------------------------------------------------------

TYPE_CONFIG = {
    "evidence": {"color": "#4A90D9", "symbol": "circle", "label": "Preuve"},
    "event": {"color": "#E74C3C", "symbol": "diamond", "label": "Evenement"},
    "entity": {"color": "#27AE60", "symbol": "square", "label": "Entite"},
    "hypothesis_snapshot": {"color": "#F39C12", "symbol": "triangle-up", "label": "Hypothese (eval)"},
    "monitoring_result": {"color": "#8E44AD", "symbol": "star", "label": "Monitoring"},
}

DEFAULT_TYPE = {"color": "#95A5A6", "symbol": "circle", "label": "Autre"}

# ---------------------------------------------------------------------------
# Filters
# ---------------------------------------------------------------------------

all_types = sorted(set(e.get("type", "other") for e in timeline))
type_labels = {t: TYPE_CONFIG.get(t, DEFAULT_TYPE)["label"] for t in all_types}

selected_types = st.multiselect(
    "Filtrer par type d'evenement",
    options=all_types,
    default=all_types,
    format_func=lambda t: type_labels.get(t, t),
)

if not selected_types:
    st.warning("Selectionnez au moins un type d'evenement.")
    st.stop()

filtered = [e for e in timeline if e.get("type") in selected_types]

if not filtered:
    st.caption("Aucun evenement ne correspond aux filtres.")
    st.stop()

st.caption(f"{len(filtered)} evenement(s) affiches")

# ---------------------------------------------------------------------------
# Build Plotly scatter plot
# ---------------------------------------------------------------------------

fig = go.Figure()

# Y-axis positions per type (spread vertically for readability)
type_y_map = {t: idx for idx, t in enumerate(sorted(set(e.get("type", "other") for e in filtered)))}

for event_type in sorted(set(e.get("type") for e in filtered)):
    events_of_type = [e for e in filtered if e.get("type") == event_type]
    config = TYPE_CONFIG.get(event_type, DEFAULT_TYPE)

    dates = []
    y_values = []
    hover_texts = []
    custom_data = []

    for e in events_of_type:
        raw_date = e.get("date", "")
        if not raw_date:
            continue

        # Parse the date for proper axis rendering
        try:
            if isinstance(raw_date, str):
                # Handle various ISO formats
                parsed = raw_date[:19]  # Trim to seconds precision
            else:
                parsed = str(raw_date)
        except (ValueError, TypeError):
            parsed = str(raw_date)

        dates.append(parsed)
        y_values.append(type_y_map.get(event_type, 0))

        title = e.get("title", "Sans titre")
        description = e.get("description", "")
        if len(description) > 120:
            description = description[:120] + "..."

        hover_texts.append(
            f"<b>{title}</b><br>"
            f"Type: {config['label']}<br>"
            f"Date: {parsed[:10]}<br>"
            f"{description}"
        )

        custom_data.append({
            "title": e.get("title", ""),
            "description": e.get("description", ""),
            "related_id": e.get("related_id", ""),
            "source": e.get("source", ""),
            "date": raw_date,
            "type": event_type,
        })

    if dates:
        fig.add_trace(go.Scatter(
            x=dates,
            y=y_values,
            mode="markers",
            name=config["label"],
            marker=dict(
                color=config["color"],
                symbol=config["symbol"],
                size=12,
                line=dict(width=1, color="white"),
            ),
            hovertext=hover_texts,
            hoverinfo="text",
            customdata=custom_data,
        ))

# Layout
y_tick_labels = {v: TYPE_CONFIG.get(k, DEFAULT_TYPE)["label"] for k, v in type_y_map.items()}

fig.update_layout(
    title="Timeline chronologique",
    xaxis_title="Date",
    yaxis=dict(
        tickmode="array",
        tickvals=list(y_tick_labels.keys()),
        ticktext=list(y_tick_labels.values()),
        title="",
    ),
    height=450,
    template="plotly_white",
    legend=dict(orientation="h", yanchor="bottom", y=-0.3),
    margin=dict(l=120, r=20, t=50, b=80),
    hovermode="closest",
)

st.plotly_chart(fig, use_container_width=True)

# ---------------------------------------------------------------------------
# Event details (click-driven via selectbox)
# ---------------------------------------------------------------------------

st.markdown("---")
st.subheader("Details d'un evenement")

# Build event options for selectbox
event_options = {}
for e in filtered:
    raw_date = e.get("date", "")
    date_prefix = raw_date[:10] if isinstance(raw_date, str) and len(raw_date) >= 10 else "?"
    etype = e.get("type", "other")
    label = TYPE_CONFIG.get(etype, DEFAULT_TYPE)["label"]
    title = e.get("title", "Sans titre")
    display = f"[{date_prefix}] {label}: {title}"

    # Use index as key to handle duplicates
    idx = len(event_options)
    event_options[idx] = {"display": display, "event": e}

if event_options:
    selected_idx = st.selectbox(
        "Selectionner un evenement",
        options=list(event_options.keys()),
        format_func=lambda i: event_options[i]["display"],
    )

    if selected_idx is not None:
        event = event_options[selected_idx]["event"]

        col1, col2 = st.columns([2, 1])

        with col1:
            etype = event.get("type", "other")
            config = TYPE_CONFIG.get(etype, DEFAULT_TYPE)

            st.markdown(
                f"<span style='background-color:{config['color']};color:white;"
                f"padding:2px 10px;border-radius:12px;font-size:0.85em'>"
                f"{config['label'].upper()}</span>",
                unsafe_allow_html=True,
            )

            st.markdown(f"### {event.get('title', 'Sans titre')}")

            description = event.get("description", "")
            if description:
                st.markdown(description)

        with col2:
            raw_date = event.get("date", "N/A")
            if isinstance(raw_date, str) and len(raw_date) > 19:
                raw_date = raw_date[:19]
            st.markdown(f"**Date:** {raw_date}")
            st.markdown(f"**Source:** {event.get('source', 'N/A')}")

            related_id = event.get("related_id")
            if related_id:
                st.markdown(f"**ID lie:** `{related_id[:16]}...`")
