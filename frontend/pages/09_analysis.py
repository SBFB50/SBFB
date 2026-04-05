import sys; from pathlib import Path; sys.path.insert(0, str(Path(__file__).resolve().parent.parent)) if str(Path(__file__).resolve().parent.parent) not in sys.path else None
"""
NEXUS -- Analysis page.

Trigger full case analysis and review past analysis runs
with their input/output summaries.
"""

import streamlit as st
import pandas as pd
from frontend.api_client import api
from frontend.components.system_stats import render_system_stats; render_system_stats()


# ------------------------------------------------------------------
# Helpers (must be defined before use in Streamlit top-to-bottom exec)
# ------------------------------------------------------------------


def _format_duration(seconds) -> str:
    """Format duration in seconds to a human-readable string."""
    if seconds is None:
        return "N/A"
    try:
        seconds = int(seconds)
    except (TypeError, ValueError):
        return str(seconds)

    if seconds < 60:
        return f"{seconds}s"
    minutes = seconds // 60
    remaining = seconds % 60
    if minutes < 60:
        return f"{minutes}m {remaining}s"
    hours = minutes // 60
    remaining_min = minutes % 60
    return f"{hours}h {remaining_min}m"


st.header("Analyses")

case_id = st.session_state.get("case_id")
if not case_id:
    st.warning("Selectionnez un dossier dans la barre laterale.")
    st.stop()

# ------------------------------------------------------------------
# Trigger new analysis
# ------------------------------------------------------------------

st.subheader("Lancer une analyse")

st.markdown(
    "L'analyse complete evalue toutes les preuves, met a jour le graphe, "
    "re-evalue les hypotheses et genere de nouvelles alertes si necessaire."
)

col_trigger, col_info = st.columns([1, 3])

with col_trigger:
    if st.button("Lancer l'analyse complete", type="primary", use_container_width=True):
        with st.spinner("Analyse en cours... Cela peut prendre quelques minutes."):
            result = api.trigger_analysis(case_id)
        if result:
            run_id = result.get("id", result.get("run_id", "?"))
            status = result.get("status", "lance")
            st.success(f"Analyse lancee (ID: {run_id[:8] if len(str(run_id)) > 8 else run_id})")

            if status in ("completed", "done"):
                st.info("L'analyse est terminee.")
            elif status in ("running", "pending"):
                st.info(
                    "L'analyse tourne en arriere-plan. "
                    "Rafraichissez la page pour voir les resultats."
                )
            st.rerun()

with col_info:
    # Quick stats about what will be analyzed
    case_stats = api.get_case_stats(case_id)
    if case_stats:
        stat_cols = st.columns(4)
        stat_cols[0].metric("Preuves", case_stats.get("evidence_count", 0))
        stat_cols[1].metric("Entites", case_stats.get("entity_count", 0))
        stat_cols[2].metric("Hypotheses", case_stats.get("hypothesis_count", 0))
        stat_cols[3].metric("Alertes", case_stats.get("alert_count", 0))

# ------------------------------------------------------------------
# Analysis run history
# ------------------------------------------------------------------

st.markdown("---")
st.subheader("Historique des analyses")

runs = api.list_analysis_runs(case_id)

if not runs:
    st.info(
        "Aucune analyse effectuee. Cliquez sur le bouton ci-dessus pour "
        "lancer la premiere."
    )
    st.stop()

# Status helpers
STATUS_ICONS = {
    "completed": "✅",
    "done": "✅",
    "running": "⏳",
    "pending": "🕐",
    "failed": "❌",
    "error": "❌",
}

STATUS_LABELS = {
    "completed": "Terminee",
    "done": "Terminee",
    "running": "En cours",
    "pending": "En attente",
    "failed": "Echouee",
    "error": "Erreur",
}

# Build summary table
rows = []
for run in runs:
    status = run.get("status", "?")
    rows.append({
        "id": run.get("id", "?"),
        "Statut": f"{STATUS_ICONS.get(status, '?')} {STATUS_LABELS.get(status, status)}",
        "Type": run.get("analysis_type", run.get("type", "complete")),
        "Date": run.get("created_at", run.get("started_at", "?")),
        "Duree": _format_duration(run.get("duration_seconds", run.get("duration", None))),
        "raw_status": status,
    })

df = pd.DataFrame(rows)

st.dataframe(
    df[["Statut", "Type", "Date", "Duree"]],
    use_container_width=True,
    hide_index=True,
)

# ------------------------------------------------------------------
# Analysis run details
# ------------------------------------------------------------------

st.markdown("---")
st.subheader("Details d'une analyse")

run_options = {
    r["id"]: f"{r.get('created_at', r.get('started_at', '?'))} - "
             f"{STATUS_LABELS.get(r.get('status', '?'), r.get('status', '?'))}"
    for r in runs
}

selected_run_id = st.selectbox(
    "Selectionner une analyse",
    options=list(run_options.keys()),
    format_func=lambda rid: run_options[rid],
)

if selected_run_id:
    run_detail = api.get_analysis_run(selected_run_id)
    if run_detail:
        detail_col1, detail_col2 = st.columns(2)

        with detail_col1:
            st.markdown("#### Resume des entrees")
            input_summary = run_detail.get("input_summary", None)
            if input_summary:
                if isinstance(input_summary, dict):
                    for key, value in input_summary.items():
                        st.markdown(f"- **{key}**: {value}")
                else:
                    st.markdown(str(input_summary))
            else:
                st.caption("Pas de resume des entrees disponible.")

        with detail_col2:
            st.markdown("#### Resume des resultats")
            output_summary = run_detail.get("output_summary", None)
            if output_summary:
                if isinstance(output_summary, dict):
                    for key, value in output_summary.items():
                        st.markdown(f"- **{key}**: {value}")
                else:
                    st.markdown(str(output_summary))
            else:
                st.caption("Pas de resume des resultats disponible.")

        # Errors section
        errors = run_detail.get("errors", run_detail.get("error", None))
        if errors:
            with st.expander("Erreurs", expanded=False):
                if isinstance(errors, list):
                    for err in errors:
                        st.error(str(err))
                else:
                    st.error(str(errors))

        # Raw JSON
        with st.expander("Donnees brutes (JSON)", expanded=False):
            st.json(run_detail)
    else:
        st.error("Impossible de charger les details de cette analyse.")
