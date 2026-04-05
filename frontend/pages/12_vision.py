"""
NEXUS -- Page d'analyse visuelle.

Analyse d'images via VLM (Vision Language Models):
- Upload + analyse directe
- Analyse des preuves images existantes
- Comparaison de deux images
- Tableau des entites visuelles extraites
"""

from __future__ import annotations
import sys; from pathlib import Path; sys.path.insert(0, str(Path(__file__).resolve().parent.parent)) if str(Path(__file__).resolve().parent.parent) not in sys.path else None  # noqa: E402

import json
from pathlib import Path

import streamlit as st

from frontend.api_client import api
from frontend.components.system_stats import render_system_stats; render_system_stats()

# ---------------------------------------------------------------------------
# Guard
# ---------------------------------------------------------------------------

st.title("Analyse Visuelle")

case_id = st.session_state.get("case_id")
if not case_id:
    st.info("Selectionnez ou creez un dossier dans la barre laterale.")
    st.stop()

# ---------------------------------------------------------------------------
# Tabs
# ---------------------------------------------------------------------------

tab_upload, tab_evidence, tab_compare, tab_entities = st.tabs([
    "Upload + Analyse",
    "Preuves visuelles",
    "Comparaison",
    "Entites visuelles",
])

# ===========================================================================
# Tab 1: Upload + Analyse directe
# ===========================================================================

with tab_upload:
    st.subheader("Analyser une image")
    st.caption(
        "Uploadez une image pour obtenir une description et une extraction "
        "d'entites visuelles. L'image n'est PAS sauvegardee comme preuve."
    )

    uploaded_file = st.file_uploader(
        "Image a analyser",
        type=["png", "jpg", "jpeg", "webp", "gif", "tiff"],
        key="vision_upload",
    )

    if uploaded_file is not None:
        # Show the uploaded image
        st.image(uploaded_file, caption=uploaded_file.name, use_container_width=True)

        if st.button("Analyser", key="btn_analyze_upload"):
            with st.spinner("Analyse en cours via VLM..."):
                result = api.describe_image_direct(uploaded_file)

            if result:
                st.subheader("Description")
                st.write(result.get("description", "Aucune description generee."))

                entities = result.get("entities", [])
                if entities:
                    st.subheader(f"Entites detectees ({len(entities)})")
                    for ent in entities:
                        col1, col2, col3 = st.columns([2, 1, 1])
                        with col1:
                            st.write(f"**{ent.get('name', '?')}**")
                            st.caption(ent.get("description", ""))
                        with col2:
                            st.write(f"Type: `{ent.get('type', '?')}`")
                        with col3:
                            conf = ent.get("confidence", 0)
                            st.progress(float(conf), text=f"{conf:.0%}")
                else:
                    st.info("Aucune entite extraite.")
            else:
                st.error("L'analyse a echoue.")


# ===========================================================================
# Tab 2: Preuves visuelles du dossier
# ===========================================================================

with tab_evidence:
    st.subheader("Preuves images du dossier")

    evidence_list = api.list_evidence(case_id, evidence_type="image")

    if not evidence_list:
        st.info("Aucune preuve de type image dans ce dossier.")
    else:
        st.write(f"**{len(evidence_list)}** preuve(s) image trouvee(s).")

        # Bulk analysis button
        if st.button("Analyser TOUTES les images", key="btn_analyze_all"):
            with st.spinner("Analyse de toutes les images en cours..."):
                result = api.analyze_all_case_images(case_id)
            if result:
                processed = result.get("images_processed", 0)
                total = result.get("images_found", 0)
                st.success(f"{processed}/{total} images analysees.")
                errors = result.get("errors", [])
                if errors:
                    for err in errors:
                        st.warning(
                            f"Erreur sur {err.get('evidence_id', '?')[:8]}...: "
                            f"{err.get('error', '')}"
                        )
                st.rerun()

        st.divider()

        for ev in evidence_list:
            ev_id = ev.get("id", "")
            title = ev.get("title", "Sans titre")
            status = ev.get("status", "pending")
            file_path = ev.get("file_path", "")
            summary = ev.get("summary", "")

            with st.expander(
                f"{'[OK]' if status == 'processed' else '[...] '} {title} "
                f"(id: {ev_id[:8]}...)"
            ):
                # Show image if file exists
                if file_path and Path(file_path).exists():
                    st.image(file_path, caption=title, use_container_width=True)
                else:
                    st.warning(f"Fichier image introuvable: {file_path}")

                if summary:
                    st.write("**Description:**")
                    st.write(summary)

                col1, col2 = st.columns(2)
                with col1:
                    st.write(f"**Statut:** {status}")
                with col2:
                    st.write(f"**Type:** {ev.get('evidence_type', '?')}")

                if st.button(
                    "Analyser cette image",
                    key=f"btn_analyze_{ev_id}",
                ):
                    with st.spinner(f"Analyse de '{title}' en cours..."):
                        result = api.analyze_evidence_image(ev_id)

                    if result:
                        st.success("Analyse terminee!")

                        desc = result.get("description", "")
                        if desc:
                            st.write("**Description:**")
                            st.write(desc)

                        scene = result.get("scene_analysis", {})
                        if scene and scene.get("raw"):
                            st.write("**Analyse de scene:**")
                            st.write(scene["raw"])

                        ents = result.get("entities", [])
                        if ents:
                            st.write(f"**Entites ({len(ents)}):**")
                            for e in ents:
                                st.write(
                                    f"- **{e.get('name', '?')}** "
                                    f"({e.get('type', '?')}) -- "
                                    f"{e.get('description', '')}"
                                )

                        st.rerun()
                    else:
                        st.error("L'analyse a echoue.")


# ===========================================================================
# Tab 3: Comparaison
# ===========================================================================

with tab_compare:
    st.subheader("Comparer deux images")

    image_evidence = api.list_evidence(case_id, evidence_type="image")

    if len(image_evidence) < 2:
        st.info(
            "Il faut au moins 2 preuves images dans le dossier "
            "pour pouvoir comparer."
        )
    else:
        options = {
            f"{ev.get('title', 'Sans titre')} ({ev['id'][:8]}...)": ev["id"]
            for ev in image_evidence
        }

        col1, col2 = st.columns(2)
        with col1:
            label_1 = st.selectbox(
                "Image 1",
                list(options.keys()),
                key="compare_img1",
            )
        with col2:
            label_2 = st.selectbox(
                "Image 2",
                list(options.keys()),
                index=min(1, len(options) - 1),
                key="compare_img2",
            )

        if label_1 and label_2:
            id_1 = options[label_1]
            id_2 = options[label_2]

            if id_1 == id_2:
                st.warning("Selectionnez deux images differentes.")
            elif st.button("Comparer", key="btn_compare"):
                with st.spinner("Comparaison en cours..."):
                    result = api.compare_evidence_images(id_1, id_2)

                if result:
                    st.subheader("Resultats de la comparaison")

                    col_a, col_b = st.columns(2)
                    with col_a:
                        st.write("**Image 1:**")
                        st.write(result.get("description_1", ""))
                    with col_b:
                        st.write("**Image 2:**")
                        st.write(result.get("description_2", ""))

                    st.divider()
                    st.write("**Analyse comparative:**")
                    st.write(result.get("comparison", ""))
                else:
                    st.error("La comparaison a echoue.")


# ===========================================================================
# Tab 4: Entites visuelles
# ===========================================================================

with tab_entities:
    st.subheader("Entites extraites d'images")

    visual_ents = api.list_visual_entities(case_id)

    if not visual_ents:
        st.info(
            "Aucune entite visuelle extraite pour ce dossier. "
            "Analysez d'abord des images dans les onglets precedents."
        )
    else:
        st.write(f"**{len(visual_ents)}** entite(s) visuelle(s) trouvee(s).")

        # Build a table
        table_data = []
        for ent in visual_ents:
            meta = ent.get("metadata", {}) or {}
            table_data.append({
                "Nom": ent.get("name", "?"),
                "Type": ent.get("entity_type", "?"),
                "Description": ent.get("description", ""),
                "Confiance": meta.get("confidence", "?"),
                "Evidence": str(meta.get("evidence_id", ""))[:8] + "...",
            })

        st.dataframe(
            table_data,
            use_container_width=True,
            hide_index=True,
        )
