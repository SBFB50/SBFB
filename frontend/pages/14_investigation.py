import sys; from pathlib import Path; sys.path.insert(0, str(Path(__file__).resolve().parent.parent)) if str(Path(__file__).resolve().parent.parent) not in sys.path else None
"""
NEXUS -- Investigation autonome.

Controle et visualisation de la boucle d'investigation autonome (OODA).
- Status de l'investigation (running/stopped, cycle count, derniere action)
- Boutons start/stop
- Journal des actions autonomes (derniers cycles OODA)
- Graphique de l'evolution des hypotheses en temps reel
- Liste des requetes auto-generees
- Resultats du self-questioning
"""

import streamlit as st
import pandas as pd
from datetime import datetime
from frontend.api_client import api


# ------------------------------------------------------------------
# Helpers
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


def _format_datetime(iso_str) -> str:
    """Format an ISO datetime string for display."""
    if not iso_str:
        return "---"
    try:
        dt = datetime.fromisoformat(str(iso_str))
        return dt.strftime("%Y-%m-%d %H:%M:%S")
    except (ValueError, TypeError):
        return str(iso_str)


PHASE_ICONS = {
    "OBSERVE": "1/5",
    "ORIENT": "2/5",
    "DECIDE": "3/5",
    "ACT": "4/5",
    "QUESTION": "5/5",
    "SLEEPING": "Pause",
}


# ------------------------------------------------------------------
# Page
# ------------------------------------------------------------------

st.header("Investigation autonome")

case_id = st.session_state.get("case_id")
if not case_id:
    st.warning("Selectionnez un dossier dans la barre laterale.")
    st.stop()


# ------------------------------------------------------------------
# Investigation status + controls
# ------------------------------------------------------------------

st.subheader("Controle de l'investigation")

status = api.get_investigation_status(case_id)

if status is None:
    st.error("Impossible de recuperer le statut de l'investigation.")
    st.stop()

is_running = status.get("running", False)

col_status, col_controls = st.columns([3, 1])

with col_status:
    # Status metrics
    m_cols = st.columns(4)
    m_cols[0].metric(
        "Statut",
        "En cours" if is_running else "Arrete",
    )
    m_cols[1].metric("Cycles completes", status.get("cycle_count", 0))
    m_cols[2].metric(
        "Phase actuelle",
        PHASE_ICONS.get(
            status.get("last_action", ""),
            status.get("last_action") or "---",
        ),
    )
    m_cols[3].metric(
        "Dernier cycle",
        _format_datetime(status.get("last_cycle_at")),
    )

    if status.get("started_at"):
        st.caption(f"Demarre le: {_format_datetime(status.get('started_at'))}")

with col_controls:
    if is_running:
        if st.button(
            "Arreter l'investigation",
            type="secondary",
            use_container_width=True,
        ):
            result = api.stop_investigation(case_id)
            if result and result.get("status") == "stopped":
                st.success("Investigation arretee.")
            else:
                st.info(
                    f"Statut: {result.get('status', '?')}"
                    if result else "Erreur"
                )
            st.rerun()
    else:
        if st.button(
            "Demarrer l'investigation",
            type="primary",
            use_container_width=True,
        ):
            result = api.start_investigation(case_id)
            if result and result.get("status") == "started":
                st.success("Investigation autonome demarree.")
            elif result and result.get("status") == "already_running":
                st.info("L'investigation tourne deja.")
            else:
                st.error("Echec du demarrage.")
            st.rerun()

st.markdown(
    "> La boucle OODA (Observe-Orient-Decide-Act + Question) "
    "tourne en continu, analysant les nouveaux resultats de monitoring, "
    "ingerant les preuves, re-evaluant les hypotheses, detectant les "
    "contradictions, et generant de nouvelles requetes de recherche."
)


# ------------------------------------------------------------------
# Hypothesis evolution chart
# ------------------------------------------------------------------

st.markdown("---")
st.subheader("Evolution des hypotheses")

hypotheses = api.list_hypotheses(case_id)

if hypotheses:
    # Build evolution data for all hypotheses
    all_evolution_data = []

    for h in hypotheses:
        hyp_id = h.get("id", "")
        hyp_title = h.get("title", "?")
        # Truncate title for legend readability
        short_title = hyp_title[:50] + "..." if len(hyp_title) > 50 else hyp_title

        evolution = api.get_hypothesis_evolution(hyp_id)
        if evolution:
            for point in evolution:
                all_evolution_data.append({
                    "Date": point.get("date", ""),
                    "Score": point.get("score", 50),
                    "Hypothese": short_title,
                    "Trigger": point.get("trigger", "?"),
                })

    if all_evolution_data:
        df_evo = pd.DataFrame(all_evolution_data)
        try:
            df_evo["Date"] = pd.to_datetime(df_evo["Date"])
            df_evo = df_evo.sort_values("Date")

            # Use Streamlit native line chart with multi-series via pivot
            pivot = df_evo.pivot_table(
                index="Date",
                columns="Hypothese",
                values="Score",
                aggfunc="last",
            )
            pivot = pivot.ffill()
            st.line_chart(pivot, height=350)
        except Exception:
            # Fallback: show raw data
            st.dataframe(df_evo, use_container_width=True, hide_index=True)
    else:
        st.caption("Pas encore de donnees d'evolution.")

    # Current scores summary
    st.markdown("**Scores actuels:**")
    score_cols = st.columns(min(len(hypotheses), 4))
    for i, h in enumerate(hypotheses[:4]):
        title = h.get("title", "?")
        short = title[:30] + "..." if len(title) > 30 else title
        score_cols[i].metric(
            short,
            f"{h.get('current_score', 50):.0f}%",
        )

    if len(hypotheses) > 4:
        with st.expander(f"+ {len(hypotheses) - 4} autres hypotheses"):
            for h in hypotheses[4:]:
                st.markdown(
                    f"- **{h.get('title', '?')}**: "
                    f"{h.get('current_score', 50):.0f}%"
                )
else:
    st.info(
        "Aucune hypothese generee. L'investigation autonome en "
        "generera automatiquement lorsque suffisamment de preuves "
        "seront disponibles."
    )


# ------------------------------------------------------------------
# Autonomous action log
# ------------------------------------------------------------------

st.markdown("---")
st.subheader("Journal des actions autonomes")

log_entries = api.get_investigation_log(case_id, limit=30)

if log_entries:
    STATUS_ICONS = {
        "completed": "[OK]",
        "running": "[...]",
        "failed": "[ERREUR]",
    }

    rows = []
    self_questioning_entries = []

    for entry in log_entries:
        run_type = entry.get("run_type", "?")
        entry_status = entry.get("status", "?")
        icon = STATUS_ICONS.get(entry_status, "?")

        rows.append({
            "Date": _format_datetime(entry.get("started_at")),
            "Type": run_type,
            "Statut": f"{icon} {entry_status}",
            "Duree": _format_duration(entry.get("duration_sec")),
            "Resume": (entry.get("input_summary") or "")[:80],
        })

        if run_type == "self_questioning":
            self_questioning_entries.append(entry)

    df_log = pd.DataFrame(rows)
    st.dataframe(df_log, use_container_width=True, hide_index=True)

    # ------------------------------------------------------------------
    # Self-questioning results
    # ------------------------------------------------------------------

    if self_questioning_entries:
        st.markdown("---")
        st.subheader("Auto-questionnement")
        st.markdown(
            "La boucle autonome s'auto-questionne de maniere adversariale "
            "a chaque cycle pour challenger ses propres conclusions."
        )

        for i, sq in enumerate(self_questioning_entries[:5]):
            cycle_info = sq.get("input_summary", "")
            output = sq.get("output_summary", "")
            date = _format_datetime(sq.get("started_at"))

            with st.expander(f"Cycle -- {date}", expanded=(i == 0)):
                if cycle_info:
                    st.caption(cycle_info)
                if output:
                    st.markdown(output)
                else:
                    st.caption("Pas de resultat disponible.")

else:
    st.info(
        "Aucune action autonome enregistree. "
        "Demarrez l'investigation pour commencer."
    )


# ------------------------------------------------------------------
# Auto-generated monitoring queries
# ------------------------------------------------------------------

st.markdown("---")
st.subheader("Requetes de monitoring")

jobs = api.list_monitoring_jobs(case_id)

if jobs:
    auto_jobs = [
        j for j in jobs
        if "[AUTO-" in (j.get("query") or "")
        or j.get("interval_hours", 24) == 12  # Auto-generated jobs use 12h interval
    ]
    manual_jobs = [j for j in jobs if j not in auto_jobs]

    if auto_jobs:
        st.markdown(f"**Requetes auto-generees** ({len(auto_jobs)})")
        auto_rows = []
        for j in auto_jobs:
            auto_rows.append({
                "Requete": j.get("query", "?"),
                "Type": j.get("job_type", "?"),
                "Intervalle": f"{j.get('interval_hours', '?')}h",
                "Resultats": j.get("results_count", 0),
                "Active": "Oui" if j.get("is_active") else "Non",
            })
        st.dataframe(
            pd.DataFrame(auto_rows),
            use_container_width=True,
            hide_index=True,
        )

    if manual_jobs:
        st.markdown(f"**Requetes manuelles** ({len(manual_jobs)})")
        manual_rows = []
        for j in manual_jobs:
            manual_rows.append({
                "Requete": j.get("query", "?"),
                "Type": j.get("job_type", "?"),
                "Intervalle": f"{j.get('interval_hours', '?')}h",
                "Resultats": j.get("results_count", 0),
                "Active": "Oui" if j.get("is_active") else "Non",
            })
        st.dataframe(
            pd.DataFrame(manual_rows),
            use_container_width=True,
            hide_index=True,
        )
else:
    st.info(
        "Aucune requete de monitoring. L'investigation autonome "
        "en creera automatiquement."
    )


# ------------------------------------------------------------------
# Global investigations overview
# ------------------------------------------------------------------

st.markdown("---")
st.subheader("Vue globale des investigations")

all_investigations = api.list_investigations()

if all_investigations and all_investigations.get("investigations"):
    inv_map = all_investigations["investigations"]
    st.metric("Investigations actives", all_investigations.get("active_count", 0))

    inv_rows = []
    for cid, inv_status in inv_map.items():
        inv_rows.append({
            "Case ID": cid[:12] + "...",
            "Cycles": inv_status.get("cycle_count", 0),
            "Phase": PHASE_ICONS.get(
                inv_status.get("last_action", ""),
                inv_status.get("last_action") or "---",
            ),
            "Dernier cycle": _format_datetime(inv_status.get("last_cycle_at")),
        })

    st.dataframe(
        pd.DataFrame(inv_rows),
        use_container_width=True,
        hide_index=True,
    )
else:
    st.caption("Aucune investigation autonome active dans le systeme.")
