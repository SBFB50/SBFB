import sys; from pathlib import Path; sys.path.insert(0, str(Path(__file__).resolve().parent.parent)) if str(Path(__file__).resolve().parent.parent) not in sys.path else None
"""
NEXUS -- Audit Trail / Investigation Journal page.

Displays a chronological, filterable log of every action taken
during an investigation.  Think of it as 'git log' for the case.
"""

import json
from datetime import datetime

import streamlit as st
from frontend.api_client import api
from frontend.components.system_stats import render_system_stats; render_system_stats()

st.set_page_config(page_title="Journal d'investigation", layout="wide")
st.header("Journal d'investigation")

case_id = st.session_state.get("case_id")
if not case_id:
    st.warning("Selectionnez un dossier dans la barre laterale.")
    st.stop()


# ================================================================
# Action type icons and labels
# ================================================================

_ACTION_META = {
    "evidence_added":         {"icon": "📄", "label": "Preuve ajoutee",        "color": "#2196F3"},
    "evidence_ingested_auto": {"icon": "🤖", "label": "Auto-ingestion",       "color": "#00BCD4"},
    "entity_discovered":      {"icon": "🔍", "label": "Entite decouverte",    "color": "#9C27B0"},
    "hypothesis_created":     {"icon": "💡", "label": "Hypothese creee",      "color": "#FF9800"},
    "hypothesis_scored":      {"icon": "📊", "label": "Score modifie",        "color": "#FF5722"},
    "hypothesis_refuted":     {"icon": "❌", "label": "Hypothese refutee",    "color": "#F44336"},
    "hypothesis_confirmed":   {"icon": "✅", "label": "Hypothese confirmee",  "color": "#4CAF50"},
    "contradiction_found":    {"icon": "⚠️", "label": "Contradiction",        "color": "#F44336"},
    "monitoring_result":      {"icon": "📡", "label": "Resultat monitoring",  "color": "#607D8B"},
    "query_generated":        {"icon": "🔎", "label": "Requete generee",     "color": "#795548"},
    "self_questioning":       {"icon": "🧠", "label": "Auto-questionnement", "color": "#673AB7"},
    "analysis_started":       {"icon": "⚙️", "label": "Analyse demarree",    "color": "#3F51B5"},
    "analysis_completed":     {"icon": "✔️", "label": "Analyse terminee",    "color": "#4CAF50"},
    "investigation_started":  {"icon": "▶️", "label": "Investigation lancee", "color": "#4CAF50"},
    "investigation_stopped":  {"icon": "⏹️", "label": "Investigation arretee", "color": "#9E9E9E"},
    "case_created":           {"icon": "📁", "label": "Dossier cree",        "color": "#2196F3"},
    "case_updated":           {"icon": "📝", "label": "Dossier modifie",     "color": "#2196F3"},
    "alert_created":          {"icon": "🔔", "label": "Alerte creee",        "color": "#FF9800"},
}

_ACTOR_COLORS = {
    "user":            "#2196F3",
    "system":          "#607D8B",
    "autonomous_loop": "#9C27B0",
    "monitoring":      "#00BCD4",
}


def _format_actor_badge(actor: str) -> str:
    color = _ACTOR_COLORS.get(actor, "#757575")
    label = {
        "user": "Utilisateur",
        "system": "Systeme",
        "autonomous_loop": "Boucle autonome",
        "monitoring": "Monitoring",
    }.get(actor, actor)
    return (
        f"<span style='background:{color};color:white;padding:2px 8px;"
        f"border-radius:10px;font-size:0.75rem;font-weight:600'>"
        f"{label}</span>"
    )


def _format_timestamp(ts: str) -> str:
    try:
        dt = datetime.fromisoformat(ts)
        return dt.strftime("%d/%m/%Y %H:%M:%S")
    except (ValueError, TypeError):
        return str(ts)


# ================================================================
# Summary counters
# ================================================================

summary = api.get_audit_summary(case_id)
if summary and summary.get("total", 0) > 0:
    by_action = summary.get("by_action", {})
    st.markdown(f"**{summary['total']}** actions enregistrees au total")

    # Render key metrics in columns
    metric_keys = [
        ("evidence_added", "Preuves"),
        ("entity_discovered", "Entites"),
        ("hypothesis_created", "Hypotheses"),
        ("hypothesis_scored", "Re-scorings"),
        ("contradiction_found", "Contradictions"),
        ("query_generated", "Requetes"),
    ]
    cols = st.columns(len(metric_keys))
    for col, (key, label) in zip(cols, metric_keys):
        count = by_action.get(key, 0)
        col.metric(label, count)

    st.divider()
else:
    st.info("Aucune action enregistree pour ce dossier.")
    st.stop()


# ================================================================
# Filters
# ================================================================

filter_col1, filter_col2, filter_col3 = st.columns(3)

with filter_col1:
    action_options = ["Toutes"] + sorted(_ACTION_META.keys())
    action_filter = st.selectbox(
        "Type d'action",
        options=action_options,
        format_func=lambda a: (
            "Toutes les actions" if a == "Toutes"
            else f"{_ACTION_META.get(a, {}).get('icon', '')} {_ACTION_META.get(a, {}).get('label', a)}"
        ),
    )

with filter_col2:
    actor_options = ["Tous", "user", "system", "autonomous_loop", "monitoring"]
    actor_filter = st.selectbox(
        "Acteur",
        options=actor_options,
        format_func=lambda a: {
            "Tous": "Tous les acteurs",
            "user": "Utilisateur",
            "system": "Systeme",
            "autonomous_loop": "Boucle autonome",
            "monitoring": "Monitoring",
        }.get(a, a),
    )

with filter_col3:
    page_size = st.selectbox("Entrees par page", [25, 50, 100, 250], index=1)

# Pagination
if "audit_page" not in st.session_state:
    st.session_state.audit_page = 0

nav_col1, nav_col2, nav_col3 = st.columns([1, 2, 1])
with nav_col1:
    if st.button("Page precedente", disabled=st.session_state.audit_page == 0):
        st.session_state.audit_page = max(0, st.session_state.audit_page - 1)
        st.rerun()
with nav_col3:
    if st.button("Page suivante"):
        st.session_state.audit_page += 1
        st.rerun()

offset = st.session_state.audit_page * page_size


# ================================================================
# Fetch and display log entries
# ================================================================

entries = api.list_audit_log(
    case_id,
    action=action_filter if action_filter != "Toutes" else None,
    actor=actor_filter if actor_filter != "Tous" else None,
    limit=page_size,
    offset=offset,
)

if not entries:
    st.info("Aucune entree pour ces filtres.")
    st.stop()

with nav_col2:
    start_idx = offset + 1
    end_idx = offset + len(entries)
    st.markdown(
        f"<div style='text-align:center;padding-top:0.5rem;color:#888'>"
        f"Entrees {start_idx} - {end_idx} | Page {st.session_state.audit_page + 1}"
        f"</div>",
        unsafe_allow_html=True,
    )


# ================================================================
# Timeline rendering
# ================================================================

for entry in entries:
    action = entry.get("action", "")
    meta = _ACTION_META.get(action, {"icon": "●", "label": action, "color": "#757575"})
    icon = meta["icon"]
    color = meta["color"]
    actor_badge = _format_actor_badge(entry.get("actor", "?"))
    ts = _format_timestamp(entry.get("timestamp", ""))
    summary_text = entry.get("summary", "")
    cycle = entry.get("cycle_number")
    cycle_text = f" &mdash; Cycle {cycle}" if cycle else ""

    # Timeline entry header
    st.markdown(
        f"<div style='border-left:3px solid {color};padding-left:12px;margin-bottom:4px'>"
        f"<span style='font-size:1.1rem'>{icon}</span> "
        f"<strong>{summary_text}</strong><br>"
        f"<span style='font-size:0.8rem;color:#888'>{ts}{cycle_text}</span> "
        f"{actor_badge}"
        f"</div>",
        unsafe_allow_html=True,
    )

    # Expandable details
    details = entry.get("details")
    if details:
        with st.expander("Details", expanded=False):
            if isinstance(details, dict):
                st.json(details)
            else:
                st.code(str(details))

    st.markdown("<div style='margin-bottom:8px'></div>", unsafe_allow_html=True)


# ================================================================
# Export
# ================================================================

st.divider()
st.subheader("Export")

export_col1, export_col2 = st.columns(2)

with export_col1:
    if st.button("Exporter en Markdown"):
        timeline = api.get_audit_timeline(case_id)
        if timeline:
            lines = ["# Journal d'investigation\n"]
            for e in timeline:
                ts = _format_timestamp(e.get("timestamp", ""))
                action = e.get("action", "")
                m = _ACTION_META.get(action, {"label": action})
                actor = e.get("actor", "?")
                summary_text = e.get("summary", "")
                cycle = e.get("cycle_number")
                cycle_str = f" (cycle {cycle})" if cycle else ""
                lines.append(f"- **{ts}** [{m['label']}] ({actor}{cycle_str}): {summary_text}")
                details = e.get("details")
                if details:
                    if isinstance(details, dict):
                        lines.append(f"  ```json\n  {json.dumps(details, indent=2, default=str)}\n  ```")
                    else:
                        lines.append(f"  > {details}")
            md_content = "\n".join(lines)
            st.download_button(
                "Telecharger .md",
                data=md_content,
                file_name="journal_investigation.md",
                mime="text/markdown",
            )
        else:
            st.warning("Aucune donnee a exporter.")

with export_col2:
    if st.button("Exporter en JSON"):
        timeline = api.get_audit_timeline(case_id)
        if timeline:
            json_content = json.dumps(timeline, indent=2, ensure_ascii=False, default=str)
            st.download_button(
                "Telecharger .json",
                data=json_content,
                file_name="journal_investigation.json",
                mime="application/json",
            )
        else:
            st.warning("Aucune donnee a exporter.")
