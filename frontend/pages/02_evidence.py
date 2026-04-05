"""
NEXUS -- Evidence management page.

Upload files, submit text evidence, browse existing evidence with
filters, and inspect details (extracted text, summary, entities,
metadata).
"""

from __future__ import annotations
import sys; from pathlib import Path; sys.path.insert(0, str(Path(__file__).resolve().parent.parent)) if str(Path(__file__).resolve().parent.parent) not in sys.path else None  # noqa: E402

import json

import streamlit as st

from frontend.api_client import api
from frontend.components.system_stats import render_system_stats; render_system_stats()

# ---------------------------------------------------------------------------
# Guard
# ---------------------------------------------------------------------------

st.title("Preuves")

case_id = st.session_state.get("case_id")
if not case_id:
    st.info("Selectionnez ou creez un dossier dans la barre laterale.")
    st.stop()

# ---------------------------------------------------------------------------
# Upload / Text input section
# ---------------------------------------------------------------------------

tab_upload, tab_text = st.tabs(["Televerser un fichier", "Texte brut"])

with tab_upload:
    with st.form("upload_evidence_form", clear_on_submit=True):
        uploaded_file = st.file_uploader(
            "Fichier (PDF, image, audio, etc.)",
            type=["pdf", "png", "jpg", "jpeg", "webp", "mp3", "wav", "ogg", "txt"],
        )
        title = st.text_input("Titre *")
        source = st.text_input("Source (optionnel)", placeholder="ex: PV audition 2024-03-15")
        submitted = st.form_submit_button("Soumettre")

        if submitted:
            if not uploaded_file:
                st.warning("Veuillez selectionner un fichier.")
            elif not title:
                st.warning("Le titre est requis.")
            else:
                result = api.upload_evidence(
                    case_id=case_id,
                    file=uploaded_file,
                    title=title,
                    source=source or None,
                )
                if result:
                    st.success(
                        f"Preuve ajoutee: {result.get('title', '')} "
                        f"(id: {result['id'][:8]}...)"
                    )

with tab_text:
    with st.form("text_evidence_form", clear_on_submit=True):
        txt_title = st.text_input("Titre *", key="txt_title")
        txt_source = st.text_input("Source (optionnel)", key="txt_source")
        txt_content = st.text_area(
            "Contenu *",
            height=200,
            placeholder="Collez ici le texte du temoignage, article, note...",
        )
        txt_submitted = st.form_submit_button("Soumettre")

        if txt_submitted:
            if not txt_title or not txt_content:
                st.warning("Le titre et le contenu sont requis.")
            else:
                result = api.submit_text_evidence(
                    case_id=case_id,
                    title=txt_title,
                    text=txt_content,
                    source=txt_source or None,
                )
                if result:
                    st.success(
                        f"Preuve texte ajoutee: {result.get('title', '')} "
                        f"(id: {result['id'][:8]}...)"
                    )

# ---------------------------------------------------------------------------
# Evidence list with filters
# ---------------------------------------------------------------------------

st.markdown("---")
st.subheader("Liste des preuves")

filter_col1, filter_col2 = st.columns(2)

with filter_col1:
    type_options = ["Tous", "pdf", "image", "text", "audio", "url", "manual"]
    selected_type = st.selectbox("Type", type_options, index=0)

with filter_col2:
    _STATUS_LABELS = {"Tous": "Tous", "pending": "En attente", "processed": "Traite", "error": "Erreur"}
    status_options = ["Tous", "pending", "processed", "error"]
    selected_status = st.selectbox(
        "Statut", status_options, index=0,
        format_func=lambda s: _STATUS_LABELS.get(s, s),
    )

evidence_list = api.list_evidence(
    case_id=case_id,
    evidence_type=selected_type if selected_type != "Tous" else None,
    status=selected_status if selected_status != "Tous" else None,
)

if not evidence_list:
    st.caption("Aucune preuve pour ce dossier.")
else:
    st.caption(f"{len(evidence_list)} preuve(s) trouvee(s)")

    for ev in evidence_list:
        status = ev.get("status", "pending")
        status_icon = {"processed": "✅", "pending": "⏳", "error": "❌"}.get(status, "❓")
        ev_type = ev.get("evidence_type", "?")

        with st.expander(
            f"{status_icon} [{ev_type.upper()}] {ev.get('title', 'Sans titre')} "
            f"— {ev.get('source', 'source inconnue') or 'source inconnue'}"
        ):
            detail_col1, detail_col2 = st.columns([2, 1])

            with detail_col1:
                # Extracted text
                raw_text = ev.get("raw_text")
                if raw_text:
                    st.markdown("**Texte extrait:**")
                    st.text_area(
                        "Texte",
                        value=raw_text[:3000],
                        height=150,
                        disabled=True,
                        key=f"raw_{ev['id']}",
                        label_visibility="collapsed",
                    )

                # Summary
                summary = ev.get("summary")
                if summary:
                    st.markdown(f"**Resume:** {summary}")

            with detail_col2:
                st.markdown(f"**ID:** `{ev['id'][:12]}...`")
                st.markdown(f"**Type:** {ev_type}")
                st.markdown(f"**Statut:** {status}")
                st.markdown(f"**Fiabilite:** {ev.get('reliability', 'N/A')}/100")

                source_date = ev.get("source_date")
                if source_date:
                    st.markdown(f"**Date source:** {source_date[:10] if isinstance(source_date, str) else source_date}")

                created = ev.get("created_at", "")
                if isinstance(created, str) and len(created) > 19:
                    created = created[:19]
                st.markdown(f"**Ajoute le:** {created}")

                # Metadata
                metadata = ev.get("metadata")
                if metadata:
                    st.markdown("**Metadonnees:**")
                    if isinstance(metadata, str):
                        try:
                            metadata = json.loads(metadata)
                        except (json.JSONDecodeError, TypeError):
                            pass
                    if isinstance(metadata, dict):
                        for k, v in metadata.items():
                            st.markdown(f"- {k}: {v}")
                    else:
                        st.caption(str(metadata)[:200])

            # Delete button
            del_key = f"del_ev_{ev['id']}"
            if st.button("Supprimer", key=del_key, type="secondary"):
                api.delete_evidence(ev["id"])
                st.success("Preuve supprimee.")
                st.rerun()
