import sys; from pathlib import Path; sys.path.insert(0, str(Path(__file__).resolve().parent.parent)) if str(Path(__file__).resolve().parent.parent) not in sys.path else None
"""
NEXUS -- Monitoring (surveillance) page.

Create, manage, and review automated search jobs that periodically
query SearXNG (clearweb) and Robin (dark web / Tor) for new leads.
"""

import streamlit as st
import pandas as pd
from frontend.api_client import api

st.header("Surveillance")

case_id = st.session_state.get("case_id")
if not case_id:
    st.warning("Selectionnez un dossier dans la barre laterale.")
    st.stop()

# ------------------------------------------------------------------
# Create new monitoring job
# ------------------------------------------------------------------

st.subheader("Creer un job de surveillance")

with st.form("create_monitoring_job"):
    query = st.text_input(
        "Requete de recherche",
        placeholder="ex: disparition Jean Dupont Marseille 2019",
    )
    job_type = st.selectbox(
        "Source de recherche",
        options=["searxng", "robin", "both"],
        format_func=lambda t: {
            "searxng": "SearXNG (clearweb)",
            "robin": "Robin (dark web / Tor)",
            "both": "Les deux (clearweb + dark web)",
        }[t],
    )
    interval_hours = st.number_input(
        "Intervalle (heures)",
        min_value=1,
        max_value=168,
        value=6,
        step=1,
        help="SearXNG recommande: 6h, Robin recommande: 24h",
    )
    max_results = st.number_input(
        "Resultats max par execution",
        min_value=5,
        max_value=100,
        value=20,
        step=5,
    )
    submitted = st.form_submit_button("Creer le job", type="primary")

    if submitted:
        if not query.strip():
            st.warning("La requete est obligatoire.")
        else:
            payload = {
                "query": query.strip(),
                "source_type": job_type,
                "interval_hours": interval_hours,
                "max_results": max_results,
            }
            result = api.create_monitoring_job(case_id, payload)
            if result:
                st.success(
                    f"Job cree avec succes (ID: {result.get('id', '?')[:8]}...)"
                )
                st.rerun()

# ------------------------------------------------------------------
# Active jobs list
# ------------------------------------------------------------------

st.markdown("---")
st.subheader("Jobs actifs")

jobs = api.list_monitoring_jobs(case_id)

if not jobs:
    st.info("Aucun job de surveillance configure.")
else:
    for job in jobs:
        job_id = job.get("id", "?")
        job_query = job.get("query", "N/A")
        job_source = job.get("source_type", "?")
        job_interval = job.get("interval_hours", "?")
        job_active = job.get("active", job.get("is_active", True))
        job_last_run = job.get("last_run_at", "Jamais")

        source_label = {
            "searxng": "Clearweb",
            "robin": "Dark web",
            "both": "Clearweb + Dark web",
        }.get(job_source, job_source)

        with st.container():
            col1, col2, col3 = st.columns([4, 2, 2])

            with col1:
                status_dot = "🟢" if job_active else "🔴"
                st.markdown(f"{status_dot} **{job_query}**")
                st.caption(
                    f"{source_label} | Toutes les {job_interval}h | "
                    f"Dernier run: {job_last_run}"
                )

            with col2:
                if st.button(
                    "Executer maintenant",
                    key=f"run_{job_id}",
                    type="secondary",
                ):
                    with st.spinner("Execution en cours..."):
                        run_result = api.trigger_monitoring_job(job_id)
                    if run_result:
                        st.success("Execution terminee.")
                        st.rerun()

            with col3:
                st.caption(f"ID: {job_id[:8]}...")

        st.divider()

# ------------------------------------------------------------------
# Results
# ------------------------------------------------------------------

st.markdown("---")
st.subheader("Resultats de surveillance")

results = api.list_monitoring_results(case_id)

if not results:
    st.info(
        "Aucun resultat pour le moment. "
        "Creez un job et executez-le pour obtenir des resultats."
    )
else:
    # Build a dataframe for display
    rows = []
    for r in results:
        rows.append({
            "id": r.get("id", ""),
            "Titre": r.get("title", "Sans titre"),
            "URL": r.get("url", "N/A"),
            "Pertinence": r.get("relevance_score", r.get("relevance", "N/A")),
            "Source": r.get("source_type", "?"),
            "Date": r.get("found_at", r.get("created_at", "?")),
            "Ingere": "Oui" if r.get("ingested", False) else "Non",
        })

    df = pd.DataFrame(rows)

    # Filters
    filter_col1, filter_col2 = st.columns(2)
    with filter_col1:
        source_filter = st.selectbox(
            "Filtrer par source",
            options=["Toutes"] + sorted(df["Source"].unique().tolist()),
            key="monitoring_source_filter",
        )
    with filter_col2:
        ingested_filter = st.selectbox(
            "Statut d'ingestion",
            options=["Tous", "Non ingere", "Ingere"],
            key="monitoring_ingested_filter",
        )

    display_df = df.copy()
    if source_filter != "Toutes":
        display_df = display_df[display_df["Source"] == source_filter]
    if ingested_filter == "Non ingere":
        display_df = display_df[display_df["Ingere"] == "Non"]
    elif ingested_filter == "Ingere":
        display_df = display_df[display_df["Ingere"] == "Oui"]

    st.dataframe(
        display_df[["Titre", "URL", "Pertinence", "Source", "Date", "Ingere"]],
        use_container_width=True,
        hide_index=True,
    )

    # Convert to evidence
    st.markdown("#### Convertir en preuve")

    non_ingested = [r for r in results if not r.get("ingested", False)]
    if non_ingested:
        result_options = {
            r["id"]: f"{r.get('title', 'Sans titre')[:60]} ({r.get('source_type', '?')})"
            for r in non_ingested
        }
        selected_result_id = st.selectbox(
            "Resultat a convertir",
            options=list(result_options.keys()),
            format_func=lambda rid: result_options[rid],
            key="ingest_select",
        )

        if st.button("Convertir en preuve", type="primary"):
            with st.spinner("Ingestion en cours..."):
                ingest_result = api.ingest_monitoring_result(selected_result_id)
            if ingest_result:
                st.success(
                    "Resultat converti en preuve et ajoute au dossier."
                )
                st.rerun()
    else:
        st.info("Tous les resultats ont deja ete ingeres.")
