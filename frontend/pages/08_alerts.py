import sys; from pathlib import Path; sys.path.insert(0, str(Path(__file__).resolve().parent.parent)) if str(Path(__file__).resolve().parent.parent) not in sys.path else None
"""
NEXUS -- Alerts page.

Lists system alerts with severity-based filtering, unread badge,
and mark-as-read capability.
"""

import streamlit as st
from frontend.api_client import api
from frontend.components.system_stats import render_system_stats; render_system_stats()

st.header("Alertes")

case_id = st.session_state.get("case_id")
if not case_id:
    st.warning("Selectionnez un dossier dans la barre laterale.")
    st.stop()

# ------------------------------------------------------------------
# Unread count badge
# ------------------------------------------------------------------

unread_count = api.get_unread_count(case_id)

if unread_count > 0:
    st.markdown(
        f"<div style='display:inline-block;background:#ff4b4b;color:white;"
        f"padding:4px 14px;border-radius:16px;font-weight:700;"
        f"font-size:1rem;margin-bottom:1rem'>"
        f"{unread_count} non lue{'s' if unread_count > 1 else ''}"
        f"</div>",
        unsafe_allow_html=True,
    )
else:
    st.success("Aucune alerte non lue.")

# ------------------------------------------------------------------
# Filters
# ------------------------------------------------------------------

filter_col1, filter_col2 = st.columns(2)

with filter_col1:
    severity_filter = st.selectbox(
        "Severite",
        options=["Toutes", "critical", "warning", "info"],
        format_func=lambda s: {
            "Toutes": "Toutes les severites",
            "critical": "Critique",
            "warning": "Avertissement",
            "info": "Information",
        }.get(s, s),
    )

with filter_col2:
    unread_only = st.checkbox("Non lues uniquement", value=False)

# ------------------------------------------------------------------
# Fetch alerts
# ------------------------------------------------------------------

severity_param = None if severity_filter == "Toutes" else severity_filter
alerts = api.list_alerts(case_id, severity=severity_param, unread_only=unread_only)

if not alerts:
    st.info("Aucune alerte pour les filtres selectionnes.")
    st.stop()

# ------------------------------------------------------------------
# Severity helpers
# ------------------------------------------------------------------

SEVERITY_CONFIG = {
    "critical": {
        "icon": "🔴",
        "css_class": "severity-critical",
        "label": "CRITIQUE",
        "bg": "rgba(255, 75, 75, 0.08)",
        "border": "rgba(255, 75, 75, 0.3)",
    },
    "warning": {
        "icon": "🟠",
        "css_class": "severity-warning",
        "label": "AVERTISSEMENT",
        "bg": "rgba(255, 166, 43, 0.08)",
        "border": "rgba(255, 166, 43, 0.3)",
    },
    "info": {
        "icon": "🔵",
        "css_class": "severity-info",
        "label": "INFO",
        "bg": "rgba(28, 131, 225, 0.08)",
        "border": "rgba(28, 131, 225, 0.3)",
    },
}

# ------------------------------------------------------------------
# Bulk actions
# ------------------------------------------------------------------

unread_alert_ids = [a["id"] for a in alerts if not a.get("read", a.get("is_read", False))]

if unread_alert_ids:
    if st.button(
        f"Marquer toutes comme lues ({len(unread_alert_ids)})",
        type="secondary",
    ):
        with st.spinner("Marquage en cours..."):
            for aid in unread_alert_ids:
                api.mark_alert_read(aid)
        st.success("Toutes les alertes visibles ont ete marquees comme lues.")
        st.rerun()

st.markdown("---")

# ------------------------------------------------------------------
# Alert list
# ------------------------------------------------------------------

for alert in alerts:
    alert_id = alert.get("id", "?")
    severity = alert.get("severity", "info").lower()
    title = alert.get("title", "Alerte sans titre")
    message = alert.get("message", "")
    created_at = alert.get("created_at", "?")
    is_read = alert.get("read", alert.get("is_read", False))

    cfg = SEVERITY_CONFIG.get(severity, SEVERITY_CONFIG["info"])

    # Container with severity-coloured border
    with st.container():
        st.markdown(
            f"<div style='border-left:4px solid {cfg['border']};"
            f"background:{cfg['bg']};padding:12px 16px;"
            f"border-radius:0 8px 8px 0;margin-bottom:4px;"
            f"opacity:{'0.6' if is_read else '1'}'>"
            f"<div style='display:flex;align-items:center;gap:8px;margin-bottom:4px'>"
            f"<span>{cfg['icon']}</span>"
            f"<span class='{cfg['css_class']}' style='font-size:0.75rem'>"
            f"{cfg['label']}</span>"
            f"<span style='font-size:0.75rem;color:#808495;margin-left:auto'>"
            f"{created_at}</span>"
            f"</div>"
            f"<div style='font-weight:600;margin-bottom:4px'>{title}</div>"
            f"<div style='font-size:0.9rem;color:#ccc'>{message}</div>"
            f"</div>",
            unsafe_allow_html=True,
        )

        if not is_read:
            if st.button("Marquer comme lue", key=f"read_{alert_id}"):
                api.mark_alert_read(alert_id)
                st.rerun()
        else:
            st.caption("Lue")

    st.markdown("")  # spacing
