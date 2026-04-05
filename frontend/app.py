"""
NEXUS -- Streamlit dashboard entry point.

Launch with::

    streamlit run frontend/app.py --server.port 8501
"""

import sys
from pathlib import Path

_PROJECT_ROOT = str(Path(__file__).resolve().parent.parent)
if _PROJECT_ROOT not in sys.path:
    sys.path.insert(0, _PROJECT_ROOT)

import streamlit as st  # noqa: E402

# ---------------------------------------------------------------------------
# Page configuration (MUST be the first Streamlit call)
# ---------------------------------------------------------------------------

st.set_page_config(
    page_title="NEXUS -- Investigation Cold Cases",
    page_icon="\U0001f50d",  # magnifying glass
    layout="wide",
    initial_sidebar_state="expanded",
)

# ---------------------------------------------------------------------------
# Imports (after set_page_config)
# ---------------------------------------------------------------------------

from frontend.api_client import api  # noqa: E402

# ---------------------------------------------------------------------------
# Session state defaults
# ---------------------------------------------------------------------------

if "case_id" not in st.session_state:
    st.session_state.case_id = None
if "case_name" not in st.session_state:
    st.session_state.case_name = None

# ---------------------------------------------------------------------------
# Dark-theme compatible custom CSS
# ---------------------------------------------------------------------------

st.markdown(
    """
    <style>
    /* Sidebar header */
    [data-testid="stSidebar"] > div:first-child {
        padding-top: 1rem;
    }
    /* Metric cards */
    [data-testid="stMetric"] {
        background-color: rgba(28, 131, 225, 0.08);
        border: 1px solid rgba(28, 131, 225, 0.15);
        border-radius: 0.5rem;
        padding: 0.75rem 1rem;
    }
    /* Alert severity badges */
    .severity-critical { color: #ff4b4b; font-weight: 700; }
    .severity-warning  { color: #ffa62b; font-weight: 600; }
    .severity-info     { color: #1c83e1; }
    /* Status chips */
    .status-active    { color: #21c354; }
    .status-closed    { color: #808495; }
    .status-archived  { color: #808495; font-style: italic; }
    .status-refuted   { color: #ff4b4b; text-decoration: line-through; }
    .status-confirmed { color: #21c354; font-weight: 700; }
    </style>
    """,
    unsafe_allow_html=True,
)

# ---------------------------------------------------------------------------
# Sidebar -- System title and case selector
# ---------------------------------------------------------------------------

st.sidebar.markdown("## NEXUS")
st.sidebar.caption("Systeme d'investigation pour cold cases")
st.sidebar.markdown("---")

# Load cases from API
cases = api.list_cases()

if cases:
    case_options = {c["id"]: c["name"] for c in cases}
    case_ids = list(case_options.keys())

    # Keep current selection if still valid
    default_idx = 0
    if st.session_state.case_id in case_ids:
        default_idx = case_ids.index(st.session_state.case_id)

    selected_id = st.sidebar.selectbox(
        "Dossier actif",
        options=case_ids,
        index=default_idx,
        format_func=lambda cid: case_options[cid],
    )

    st.session_state.case_id = selected_id
    st.session_state.case_name = case_options[selected_id]
else:
    st.sidebar.info("Aucun dossier. Creez-en un depuis le dashboard.")

# Sidebar -- unread alert count
if st.session_state.case_id:
    unread = api.get_unread_count(st.session_state.case_id)
    if unread > 0:
        st.sidebar.warning(f"\u26a0 {unread} alerte(s) non lue(s)")

st.sidebar.markdown("---")

# Sidebar -- navigation links (informational -- actual navigation is via pages/)
st.sidebar.markdown(
    """
**Pages**
- Tableau de bord
- Preuves
- Entites
- Hypotheses
- Chronologie
"""
)

# Sidebar -- API health indicator
health = api.check_health()
if health:
    st.sidebar.success(f"API : {health.get('status', 'ok')} (v{health.get('version', '?')})")
else:
    st.sidebar.error("API injoignable")

# ---------------------------------------------------------------------------
# Landing page (displayed when this file is the active page)
# ---------------------------------------------------------------------------

st.title("NEXUS")
st.subheader("Systeme d'investigation persistant pour cold cases")

st.markdown("---")

col1, col2 = st.columns(2)

with col1:
    st.markdown(
        """
### Fonctionnalites

- **Persistant** -- chaque donnee est stockee, rien n'est perdu
- **Incremental** -- les nouvelles preuves re-evaluent automatiquement les hypotheses
- **Multi-sources** -- clearweb, dark web, OSINT, audio, images
- **Monitoring continu** -- recherches automatiques (6h clearweb, 24h dark web)
- **Pensee adversariale** -- chaque hypothese est systematiquement challengee
"""
    )

with col2:
    st.markdown(
        """
### Architecture

| Composant | Role |
|-----------|------|
| FastAPI | API REST backend |
| SQLite | Donnees structurees |
| Neo4j | Graphe de connaissances |
| ChromaDB | Recherche semantique |
| Ollama | LLMs locaux (Gemma 4, DeepSeek-R1, etc.) |
| SearXNG | Recherche clearweb |
| Robin | Recherche dark web / Tor |
| CompreFace | Reconnaissance faciale |
"""
    )

st.markdown("---")

# Quick-create case form
st.subheader("Creer un nouveau dossier")

with st.form("create_case_form"):
    new_name = st.text_input("Nom du dossier")
    new_ref = st.text_input("Reference (optionnel)", placeholder="ex: COLD-2024-001")
    new_desc = st.text_area("Description (optionnel)")
    submitted = st.form_submit_button("Creer")

    if submitted and new_name:
        payload = {"name": new_name, "status": "active"}
        if new_ref:
            payload["reference"] = new_ref
        if new_desc:
            payload["description"] = new_desc

        result = api.create_case(payload)
        if result:
            st.success(f"Dossier cree: {result['name']} ({result['id'][:8]}...)")
            st.session_state.case_id = result["id"]
            st.session_state.case_name = result["name"]
            st.rerun()
    elif submitted:
        st.warning("Le nom du dossier est requis.")
