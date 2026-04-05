"""
NEXUS -- Dashboard page.

Overview of the active case: key metrics, recent alerts, latest analysis,
leading hypothesis, and quick-action buttons.
"""

from __future__ import annotations
import sys; from pathlib import Path; sys.path.insert(0, str(Path(__file__).resolve().parent.parent)) if str(Path(__file__).resolve().parent.parent) not in sys.path else None  # noqa: E402

import streamlit as st

from frontend.api_client import api
from frontend.components.system_stats import render_system_stats; render_system_stats()

# ---------------------------------------------------------------------------
# Guard: a case must be selected
# ---------------------------------------------------------------------------

st.title("Tableau de bord")

case_id = st.session_state.get("case_id")
if not case_id:
    st.info("Selectionnez ou creez un dossier dans la barre laterale.")
    st.stop()

# ---------------------------------------------------------------------------
# Fetch data
# ---------------------------------------------------------------------------

stats = api.get_case_stats(case_id) or {}
alerts = api.list_alerts(case_id)
hypotheses = api.list_hypotheses(case_id)
runs = api.list_analysis_runs(case_id)
unread = api.get_unread_count(case_id)

# ---------------------------------------------------------------------------
# Key metrics (4 columns)
# ---------------------------------------------------------------------------

c1, c2, c3, c4 = st.columns(4)

c1.metric("Preuves", stats.get("evidence_count", 0))
c2.metric("Entites", stats.get("entity_count", 0))

active_hyps = [h for h in hypotheses if h.get("status") == "active"]
c3.metric("Hypotheses actives", len(active_hyps))
c4.metric("Alertes non lues", unread)

st.markdown("---")

# ---------------------------------------------------------------------------
# Two-column layout: Alerts + Analysis / Hypothesis
# ---------------------------------------------------------------------------

left, right = st.columns(2)

# -- Recent alerts ----------------------------------------------------------

with left:
    st.subheader("Alertes recentes")

    recent_alerts = sorted(
        alerts,
        key=lambda a: a.get("created_at", ""),
        reverse=True,
    )[:5]

    if not recent_alerts:
        st.caption("Aucune alerte.")
    else:
        for alert in recent_alerts:
            severity = alert.get("severity", "info")
            css_class = f"severity-{severity}"
            icon = {"critical": "🔴", "warning": "🟠", "info": "🔵"}.get(severity, "⚪")
            is_read = alert.get("is_read", False)
            opacity = "opacity:0.6;" if is_read else ""

            st.markdown(
                f"<div style='{opacity}margin-bottom:0.5rem'>"
                f"{icon} <span class='{css_class}'>[{severity.upper()}]</span> "
                f"<b>{alert.get('title', '')}</b>"
                f"<br><span style='color:#888;font-size:0.85em'>"
                f"{alert.get('message', '')}</span></div>",
                unsafe_allow_html=True,
            )

            if not is_read:
                btn_key = f"read_{alert['id']}"
                if st.button("Marquer comme lu", key=btn_key, type="secondary"):
                    api.mark_alert_read(alert["id"])
                    st.rerun()

# -- Latest analysis + leading hypothesis -----------------------------------

with right:
    # Latest analysis run
    st.subheader("Derniere analyse")

    completed_runs = [
        r for r in runs if r.get("status") == "completed"
    ]
    completed_runs.sort(key=lambda r: r.get("completed_at") or "", reverse=True)

    if completed_runs:
        last_run = completed_runs[0]
        run_date = last_run.get("completed_at", "N/A")
        if isinstance(run_date, str) and len(run_date) > 19:
            run_date = run_date[:19]
        duration = last_run.get("duration_sec")
        duration_str = f"{duration:.1f}s" if duration else "N/A"
        summary = last_run.get("output_summary") or "Pas de resume"

        st.markdown(f"**Date:** {run_date}")
        st.markdown(f"**Duree:** {duration_str} | **Type:** {last_run.get('run_type', 'N/A')}")
        st.markdown(f"**Resume:** {summary[:300]}")
    else:
        st.caption("Aucune analyse terminee.")

    st.markdown("---")

    # Leading hypothesis
    st.subheader("Hypothese principale")

    if active_hyps:
        best = max(active_hyps, key=lambda h: h.get("current_score", 0))
        score = best.get("current_score", 0)

        if score >= 70:
            score_color = "#27AE60"
        elif score >= 40:
            score_color = "#F39C12"
        else:
            score_color = "#E74C3C"

        st.markdown(
            f"**{best.get('title', 'Sans titre')}** "
            f"<span style='color:{score_color};font-weight:700;font-size:1.2em'>"
            f"{score:.1f}/100</span>",
            unsafe_allow_html=True,
        )
        st.caption(best.get("description", "")[:200])
    else:
        st.caption("Aucune hypothese active.")

# ---------------------------------------------------------------------------
# Action buttons
# ---------------------------------------------------------------------------

st.markdown("---")

act1, act2, act3 = st.columns(3)

with act1:
    if st.button("Lancer une analyse", type="primary", use_container_width=True):
        result = api.trigger_analysis(case_id)
        if result:
            st.success(
                f"Analyse lancee (run_id: {result.get('run_id', '?')[:8]}...). "
                f"Type: {result.get('run_type', 'N/A')}"
            )

with act2:
    if st.button("Generer des hypotheses", use_container_width=True):
        result = api.generate_hypotheses(case_id)
        if result:
            st.success("Generation d'hypotheses lancee en arriere-plan.")

with act3:
    if st.button("Re-evaluer toutes les hypotheses", use_container_width=True):
        result = api.evaluate_all_hypotheses(case_id)
        if result:
            st.success("Re-evaluation de toutes les hypotheses lancee.")

# ---------------------------------------------------------------------------
# Running analyses indicator
# ---------------------------------------------------------------------------

running = [r for r in runs if r.get("status") == "running"]
if running:
    st.info(f"{len(running)} analyse(s) en cours d'execution...")
