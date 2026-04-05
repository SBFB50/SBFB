import sys; from pathlib import Path; sys.path.insert(0, str(Path(__file__).resolve().parent.parent)) if str(Path(__file__).resolve().parent.parent) not in sys.path else None  # noqa: E402
"""
NEXUS -- Centre de commande de l'investigation autonome.

Page tout-en-un pour le systeme autonome:
- Controle (start/stop/status)
- Boucle OODA en temps reel
- Evolution des hypotheses
- Journal d'audit immutable
- Preuves et entites
- Contradictions detectees
- Alertes
- Requetes auto-generees
- Auto-questionnement
- Verification d'integrite
"""

import json
import streamlit as st
import pandas as pd
from datetime import datetime
from frontend.api_client import api
from frontend.components.system_stats import render_system_stats; render_system_stats()


# ------------------------------------------------------------------
# Helpers
# ------------------------------------------------------------------

def _ts(iso_str) -> str:
    if not iso_str:
        return "---"
    try:
        return datetime.fromisoformat(str(iso_str)).strftime("%d/%m %H:%M:%S")
    except (ValueError, TypeError):
        return str(iso_str)[:19]


ACTION_ICONS = {
    "evidence_added": "📄", "evidence_ingested_auto": "🤖📄",
    "entity_discovered": "👤", "hypothesis_created": "💡",
    "hypothesis_scored": "📊", "contradiction_found": "⚡",
    "monitoring_result": "🔍", "query_generated": "🔎",
    "self_questioning": "🧠", "analysis_completed": "✅",
    "analysis_started": "⏳", "analysis_running": "⏳",
    "investigation_started": "▶️", "investigation_stopped": "⏹️",
}

ACTOR_COLORS = {
    "autonomous_loop": "#ff6b6b",
    "system": "#4ecdc4",
    "user": "#45b7d1",
    "monitoring": "#f9ca24",
}


# ------------------------------------------------------------------
# Page header
# ------------------------------------------------------------------

st.set_page_config(page_title="NEXUS — Autonome", layout="wide") if False else None

case_id = st.session_state.get("case_id")
case_name = st.session_state.get("case_name", "")

st.title(f"Centre de commande — {case_name}" if case_name else "Centre de commande")

if not case_id:
    st.warning("Selectionnez un dossier dans la barre laterale.")
    st.stop()


# ------------------------------------------------------------------
# 1. STATUS + CONTROLS
# ------------------------------------------------------------------

status = api.get_investigation_status(case_id) or {}
is_running = status.get("running", False)

col1, col2, col3, col4, col_btn = st.columns([1, 1, 1, 1, 1.5])

col1.metric("Statut", "🟢 Actif" if is_running else "🔴 Arrete")
col2.metric("Cycles", status.get("cycle_count", 0))
col3.metric("Phase", status.get("last_action", "---"))
col4.metric("Dernier cycle", _ts(status.get("last_cycle_at")))

with col_btn:
    if is_running:
        if st.button("⏹️ Arreter", type="secondary", use_container_width=True):
            api.stop_investigation(case_id)
            st.rerun()
    else:
        if st.button("▶️ Demarrer l'investigation", type="primary", use_container_width=True):
            api.start_investigation(case_id)
            st.rerun()

st.caption(
    "Boucle OODA: Observe (monitoring) → Orient (ingestion) → "
    "Decide (analyse + hypotheses) → Act (nouvelles requetes) → "
    "Question (pensee adversariale)"
)

st.markdown("---")


# ------------------------------------------------------------------
# 2. STATS OVERVIEW
# ------------------------------------------------------------------

stats = api.get_case_stats(case_id) or {}
audit_summary = api.get_audit_summary(case_id) or {}
by_action = audit_summary.get("by_action", {})

s1, s2, s3, s4, s5, s6 = st.columns(6)
s1.metric("Preuves", stats.get("evidence", 0))
s2.metric("Entites", stats.get("entities", 0))
s3.metric("Hypotheses", stats.get("hypotheses", 0))
s4.metric("Contradictions", by_action.get("contradiction_found", 0))
s5.metric("Requetes auto", by_action.get("query_generated", 0))
s6.metric("Auto-questionnements", by_action.get("self_questioning", 0))

st.markdown("---")


# ------------------------------------------------------------------
# 3. HYPOTHESES EVOLUTION
# ------------------------------------------------------------------

st.subheader("Evolution des hypotheses")

hypotheses = api.list_hypotheses(case_id) or []

if hypotheses:
    # Score bars
    for h in sorted(hypotheses, key=lambda x: x.get("current_score", 0), reverse=True):
        score = h.get("current_score", 50)
        title = h.get("title", "?")
        status_h = h.get("status", "active")
        color = "#4ecdc4" if status_h == "active" else "#95a5a6"
        st.markdown(
            f'<div style="margin:4px 0">'
            f'<span style="font-weight:bold">{title}</span> '
            f'<span style="color:{color}">({status_h})</span>'
            f'</div>'
            f'<div style="background:#2d2d2d;border-radius:4px;height:24px;width:100%">'
            f'<div style="background:{color};width:{score}%;height:100%;border-radius:4px;'
            f'text-align:right;padding-right:8px;color:white;font-size:13px;line-height:24px">'
            f'{score:.0f}%</div></div>',
            unsafe_allow_html=True,
        )

    # Evolution chart
    all_evo = []
    for h in hypotheses:
        evo = api.get_hypothesis_evolution(h["id"]) or []
        for p in evo:
            all_evo.append({
                "Date": p.get("date", ""),
                "Score": p.get("score", 50),
                "Hypothese": h.get("title", "?")[:40],
            })
    if all_evo:
        df_evo = pd.DataFrame(all_evo)
        try:
            df_evo["Date"] = pd.to_datetime(df_evo["Date"])
            pivot = df_evo.pivot_table(index="Date", columns="Hypothese", values="Score", aggfunc="last").ffill()
            st.line_chart(pivot, height=300)
        except Exception:
            pass
else:
    st.info("Aucune hypothese. Le systeme en generera automatiquement.")

st.markdown("---")


# ------------------------------------------------------------------
# 4. JOURNAL D'AUDIT IMMUTABLE (hash chain)
# ------------------------------------------------------------------

st.subheader("Journal d'investigation (immutable)")

# Integrity check
col_verify, col_status_v = st.columns([1, 3])
with col_verify:
    if st.button("Verifier l'integrite"):
        verify = api._request("GET", f"/api/cases/{case_id}/audit/verify")
        if verify and verify.get("valid"):
            st.success(f"Chaine intacte ({verify['entries_checked']} entrees verifiees)")
        elif verify:
            st.error(f"FALSIFICATION DETECTEE a l'entree {verify.get('broken_at')}")
        else:
            st.warning("Verification impossible")

# Filters
f1, f2, f3 = st.columns(3)
action_filter = f1.selectbox("Action", [
    "Toutes", "evidence_added", "evidence_ingested_auto", "entity_discovered",
    "hypothesis_created", "hypothesis_scored", "contradiction_found",
    "monitoring_result", "query_generated", "self_questioning",
    "analysis_completed",
], index=0)
actor_filter = f2.selectbox("Acteur", [
    "Tous", "autonomous_loop", "system", "user", "monitoring",
], index=0)
page_size = f3.number_input("Entrees", min_value=10, max_value=500, value=50)

# Fetch
audit_params = {"limit": page_size}
if action_filter != "Toutes":
    audit_params["action"] = action_filter
if actor_filter != "Tous":
    audit_params["actor"] = actor_filter

audit_entries = api.list_audit_log(case_id, **audit_params) or []

if audit_entries:
    for entry in audit_entries:
        action = entry.get("action", "?")
        actor = entry.get("actor", "?")
        icon = ACTION_ICONS.get(action, "📌")
        actor_color = ACTOR_COLORS.get(actor, "#999")
        ts = _ts(entry.get("timestamp"))
        summary = entry.get("summary", "")
        entry_hash = (entry.get("entry_hash") or "")[:12]

        # Compact display
        st.markdown(
            f'<div style="display:flex;align-items:center;gap:8px;padding:4px 0;'
            f'border-bottom:1px solid #333;font-size:14px">'
            f'<span style="font-size:18px">{icon}</span>'
            f'<span style="color:#888;min-width:110px">{ts}</span>'
            f'<span style="background:{actor_color};color:white;padding:1px 6px;'
            f'border-radius:3px;font-size:11px">{actor}</span>'
            f'<span style="flex:1">{summary}</span>'
            f'<span style="color:#555;font-family:monospace;font-size:10px">{entry_hash}</span>'
            f'</div>',
            unsafe_allow_html=True,
        )

    # Expandable details for last entries
    with st.expander("Details JSON des dernieres entrees"):
        for entry in audit_entries[:5]:
            details = entry.get("details")
            if details:
                try:
                    parsed = json.loads(details) if isinstance(details, str) else details
                    st.json(parsed)
                except Exception:
                    st.text(str(details)[:500])
            st.markdown("---")
else:
    st.info("Aucune entree dans le journal. Demarrez l'investigation.")

st.markdown("---")


# ------------------------------------------------------------------
# 5. AUTO-QUESTIONNEMENT
# ------------------------------------------------------------------

st.subheader("Pensee adversariale (auto-questionnement)")

sq_entries = api.list_audit_log(case_id, action="self_questioning", limit=5) or []

if sq_entries:
    for i, sq in enumerate(sq_entries):
        ts = _ts(sq.get("timestamp"))
        details = sq.get("details")
        summary_text = ""
        if details:
            try:
                parsed = json.loads(details) if isinstance(details, str) else details
                summary_text = parsed.get("summary", "")
            except Exception:
                summary_text = str(details)[:1000]

        with st.expander(f"🧠 {ts} — {sq.get('summary', '')[:80]}", expanded=(i == 0)):
            if summary_text:
                st.markdown(summary_text)
            else:
                st.caption("Pas de details disponibles.")
else:
    st.caption("Aucun auto-questionnement enregistre.")

st.markdown("---")


# ------------------------------------------------------------------
# 6. REQUETES DE MONITORING AUTO-GENEREES
# ------------------------------------------------------------------

st.subheader("Requetes de recherche")

jobs = api.list_monitoring_jobs(case_id) or []

if jobs:
    tab_auto, tab_manual = st.tabs(["Auto-generees", "Manuelles"])

    auto_jobs = [j for j in jobs if j.get("interval_hours", 24) == 12]
    manual_jobs = [j for j in jobs if j not in auto_jobs]

    with tab_auto:
        if auto_jobs:
            for j in auto_jobs:
                active = "🟢" if j.get("is_active") else "🔴"
                st.markdown(
                    f"{active} **{j.get('query', '?')}** — "
                    f"{j.get('results_count', 0)} resultats — "
                    f"toutes les {j.get('interval_hours', '?')}h"
                )
        else:
            st.caption("Le systeme generera des requetes automatiquement.")

    with tab_manual:
        if manual_jobs:
            for j in manual_jobs:
                active = "🟢" if j.get("is_active") else "🔴"
                st.markdown(
                    f"{active} **{j.get('query', '?')}** — "
                    f"{j.get('results_count', 0)} resultats — "
                    f"toutes les {j.get('interval_hours', '?')}h"
                )
        else:
            st.caption("Aucune requete manuelle.")
else:
    st.info("Aucune requete de monitoring.")

st.markdown("---")


# ------------------------------------------------------------------
# 7. ALERTES RECENTES
# ------------------------------------------------------------------

st.subheader("Alertes")

alerts = api.list_alerts(case_id, unread_only=True) or []
if alerts:
    SEVERITY_STYLE = {
        "critical": ("🔴", "#e74c3c"),
        "warning": ("⚠️", "#f39c12"),
        "info": ("ℹ️", "#3498db"),
    }
    for a in alerts[:10]:
        sev = a.get("severity", "info")
        icon, color = SEVERITY_STYLE.get(sev, ("📌", "#999"))
        st.markdown(
            f'{icon} <span style="color:{color};font-weight:bold">{a.get("title", "?")}</span>'
            f' — {a.get("message", "")[:150]} '
            f'<span style="color:#888">({_ts(a.get("created_at"))})</span>',
            unsafe_allow_html=True,
        )
    unread = api.get_unread_count(case_id)
    if unread and unread > len(alerts):
        st.caption(f"... et {unread - len(alerts)} autres alertes non lues.")
else:
    st.caption("Aucune alerte non lue.")


# ------------------------------------------------------------------
# 8. CONTRADICTIONS
# ------------------------------------------------------------------

st.markdown("---")
st.subheader("Contradictions detectees")

contradiction_entries = api.list_audit_log(case_id, action="contradiction_found", limit=10) or []

if contradiction_entries:
    for c in contradiction_entries:
        details = c.get("details")
        desc = ""
        if details:
            try:
                parsed = json.loads(details) if isinstance(details, str) else details
                desc = parsed.get("description", "")
            except Exception:
                desc = str(details)[:300]
        st.markdown(f'⚡ **{_ts(c.get("timestamp"))}** — {desc[:300]}')
else:
    st.caption("Aucune contradiction detectee.")


# ------------------------------------------------------------------
# 9. EXPORT
# ------------------------------------------------------------------

st.markdown("---")

col_exp1, col_exp2 = st.columns(2)

with col_exp1:
    if st.button("Exporter le journal (JSON)"):
        timeline = api.get_audit_timeline(case_id) or []
        if timeline:
            json_str = json.dumps(timeline, ensure_ascii=False, indent=2, default=str)
            st.download_button(
                "Telecharger JSON",
                data=json_str,
                file_name=f"nexus_audit_{case_id[:8]}.json",
                mime="application/json",
            )

with col_exp2:
    if st.button("Exporter le journal (Markdown)"):
        timeline = api.get_audit_timeline(case_id) or []
        if timeline:
            lines = [f"# Journal d'investigation — {case_name}\n"]
            for e in timeline:
                icon = ACTION_ICONS.get(e.get("action", ""), "")
                lines.append(
                    f"- **{_ts(e.get('timestamp'))}** {icon} "
                    f"[{e.get('actor', '?')}] {e.get('summary', '')}"
                )
            md = "\n".join(lines)
            st.download_button(
                "Telecharger Markdown",
                data=md,
                file_name=f"nexus_audit_{case_id[:8]}.md",
                mime="text/markdown",
            )
