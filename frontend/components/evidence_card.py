"""
NEXUS -- Reusable evidence card component.

Renders a single evidence item as a styled card with type badge,
reliability bar, and truncated summary.  Also provides a list
renderer for multiple evidence items.
"""

from __future__ import annotations

from typing import Any, Dict, List

import streamlit as st

# ---------------------------------------------------------------
# Evidence type colour palette
# ---------------------------------------------------------------

TYPE_BADGE_COLORS: Dict[str, str] = {
    "document": "#4A90D9",
    "text": "#4A90D9",
    "testimony": "#8E44AD",
    "audio": "#E67E22",
    "video": "#E74C3C",
    "image": "#27AE60",
    "photo": "#27AE60",
    "web": "#1ABC9C",
    "darkweb": "#7D3C98",
    "forensic": "#C0392B",
    "financial": "#F1C40F",
    "phone_record": "#D35400",
    "social_media": "#2980B9",
    "other": "#95A5A6",
}

# Reliability thresholds for colour coding
RELIABILITY_HIGH = 0.7
RELIABILITY_MED = 0.4


def render_evidence_card(evidence: Dict[str, Any]) -> None:
    """Render a single evidence item as a styled card.

    Parameters
    ----------
    evidence : dict
        Expected keys: ``id``, ``title``, ``type``, ``source``,
        ``reliability`` (0-1 float), ``summary`` or ``text``,
        ``created_at``, ``status``.
    """
    eid = evidence.get("id", "?")
    title = evidence.get("title", "Preuve sans titre")
    etype = evidence.get("type", evidence.get("evidence_type", "other")).lower()
    source = evidence.get("source", "Inconnue")
    reliability = evidence.get("reliability", evidence.get("reliability_score", None))
    summary = evidence.get("summary", evidence.get("text", ""))
    status = evidence.get("status", "active")
    created_at = evidence.get("created_at", "")

    badge_color = TYPE_BADGE_COLORS.get(etype, TYPE_BADGE_COLORS["other"])

    # -- Type badge
    st.markdown(
        f"<span style='display:inline-block;background:{badge_color};color:white;"
        f"padding:2px 10px;border-radius:12px;font-size:0.75rem;font-weight:600;"
        f"text-transform:uppercase;letter-spacing:0.5px'>{etype}</span>"
        f"{'  <span style=\"color:#808495;font-size:0.8rem\">' + created_at + '</span>' if created_at else ''}",
        unsafe_allow_html=True,
    )

    # -- Title
    st.markdown(f"**{title}**")

    # -- Source
    st.caption(f"Source: {source}")

    # -- Reliability bar
    if reliability is not None:
        try:
            rel_val = float(reliability)
        except (TypeError, ValueError):
            rel_val = None

        if rel_val is not None:
            # Determine colour based on value
            if rel_val >= RELIABILITY_HIGH:
                bar_color = "#27AE60"  # green
                label = "Haute"
            elif rel_val >= RELIABILITY_MED:
                bar_color = "#F39C12"  # orange
                label = "Moyenne"
            else:
                bar_color = "#E74C3C"  # red
                label = "Faible"

            percent = int(rel_val * 100)
            st.markdown(
                f"<div style='display:flex;align-items:center;gap:8px;margin:4px 0'>"
                f"<span style='font-size:0.8rem;min-width:55px'>Fiabilite</span>"
                f"<div style='flex:1;background:rgba(128,128,128,0.2);"
                f"border-radius:4px;height:8px;overflow:hidden'>"
                f"<div style='width:{percent}%;background:{bar_color};"
                f"height:100%;border-radius:4px;transition:width 0.3s'></div>"
                f"</div>"
                f"<span style='font-size:0.75rem;color:{bar_color};"
                f"min-width:60px'>{percent}% ({label})</span>"
                f"</div>",
                unsafe_allow_html=True,
            )

    # -- Summary (truncated)
    if summary:
        truncated = _truncate(str(summary), 200)
        st.markdown(
            f"<div style='font-size:0.9rem;color:#b0b0b0;margin-top:4px'>"
            f"{truncated}</div>",
            unsafe_allow_html=True,
        )

    # -- Status indicator
    if status and status != "active":
        status_colors = {
            "processing": "#F39C12",
            "processed": "#27AE60",
            "error": "#E74C3C",
            "archived": "#808495",
        }
        scolor = status_colors.get(status, "#808495")
        st.markdown(
            f"<span style='font-size:0.7rem;color:{scolor}'>"
            f"Statut: {status}</span>",
            unsafe_allow_html=True,
        )


def render_evidence_list(
    evidences: List[Dict[str, Any]],
    columns: int = 1,
) -> None:
    """Render a list of evidence items as cards.

    Parameters
    ----------
    evidences : list[dict]
        List of evidence dicts (same format as ``render_evidence_card``).
    columns : int
        Number of columns to lay out cards (1 = single column list).
    """
    if not evidences:
        st.info("Aucune preuve a afficher.")
        return

    if columns <= 1:
        for evidence in evidences:
            with st.container():
                render_evidence_card(evidence)
                st.divider()
    else:
        cols = st.columns(columns)
        for idx, evidence in enumerate(evidences):
            with cols[idx % columns]:
                with st.container():
                    render_evidence_card(evidence)
                    st.markdown("")  # spacing


# ---------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------


def _truncate(text: str, max_len: int) -> str:
    """Truncate text with ellipsis if too long."""
    if len(text) <= max_len:
        return text
    return text[: max_len - 3] + "..."
