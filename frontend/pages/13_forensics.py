"""
NEXUS -- Page d'analyse forensique physique.

5 onglets:
1. BPA (Blood Pattern Analysis) -- classification + calculs geometriques
2. Acoustique -- transcription + analyse forensique + timeline
3. Traces -- classification + analyse de traces physiques
4. Auto-analyse -- analyse forensique automatique sur toutes les preuves
5. Simulations -- simulation physique (balistique sang, cast-off, son, origine)
"""

from __future__ import annotations
import sys; from pathlib import Path; sys.path.insert(0, str(Path(__file__).resolve().parent.parent)) if str(Path(__file__).resolve().parent.parent) not in sys.path else None  # noqa: E402

import json

import streamlit as st

from frontend.api_client import api

# ---------------------------------------------------------------------------
# Guard
# ---------------------------------------------------------------------------

st.title("Analyse Forensique")

case_id = st.session_state.get("case_id")
if not case_id:
    st.info("Selectionnez ou creez un dossier dans la barre laterale.")
    st.stop()

# ---------------------------------------------------------------------------
# Tabs
# ---------------------------------------------------------------------------

tab_bpa, tab_audio, tab_traces, tab_auto, tab_sim = st.tabs([
    "BPA (Projections de sang)",
    "Acoustique",
    "Traces physiques",
    "Auto-analyse",
    "Simulations",
])


# ===========================================================================
# Tab 1: Blood Pattern Analysis
# ===========================================================================

with tab_bpa:
    st.subheader("Analyse de projections de sang (BPA)")

    # --- Sub-section: Upload + Classification ---
    st.markdown("#### Classification de pattern")
    st.caption(
        "Uploadez une photo de projections de sang pour identifier "
        "le type de pattern (spatter, transfer, drip, pool, etc.)."
    )

    bpa_file = st.file_uploader(
        "Photo de projections de sang",
        type=["png", "jpg", "jpeg", "webp", "tiff"],
        key="bpa_upload",
    )

    if bpa_file is not None:
        st.image(bpa_file, caption=bpa_file.name, use_container_width=True)

        col_classify, col_full = st.columns(2)
        with col_classify:
            if st.button("Classifier le pattern", key="btn_bpa_classify"):
                with st.spinner("Classification VLM en cours..."):
                    result = api.forensic_bpa_classify(bpa_file)
                if result:
                    st.success("Classification terminee")
                    primary = result.get("primary_type", result.get("classification", "?"))
                    st.metric("Type principal", primary)
                    if "confidence" in result:
                        st.progress(
                            float(result["confidence"]),
                            text=f"Confiance: {result['confidence']:.0%}",
                        )
                    if "description" in result:
                        st.write("**Description:**")
                        st.write(result["description"])
                    if "mechanism" in result:
                        st.write("**Mecanisme:**")
                        st.write(result["mechanism"])
                    if "implications" in result:
                        st.write("**Implications forensiques:**")
                        for imp in result["implications"]:
                            st.write(f"- {imp}")
                else:
                    st.error("La classification a echoue.")

        with col_full:
            case_ctx = st.text_area(
                "Contexte de l'enquete (optionnel)",
                key="bpa_context",
                height=100,
            )
            if st.button("Analyse BPA complete", key="btn_bpa_full"):
                with st.spinner("Analyse BPA complete en cours..."):
                    result = api.forensic_bpa_analyze(
                        bpa_file, case_context=case_ctx
                    )
                if result:
                    st.success("Analyse BPA terminee")
                    if "classification" in result:
                        st.write("**Classification:**")
                        st.json(result["classification"])
                    if "spatter_analysis" in result:
                        st.write("**Analyse de spatter:**")
                        st.json(result["spatter_analysis"])
                    if "interpretation" in result:
                        st.write("**Interpretation:**")
                        st.write(result["interpretation"])
                else:
                    st.error("L'analyse a echoue.")

    st.divider()

    # --- Sub-section: Geometric calculations ---
    st.markdown("#### Calculs geometriques")

    st.markdown("##### Angle d'impact")
    st.caption("sin(angle) = largeur / longueur de la tache")
    col_w, col_l = st.columns(2)
    with col_w:
        width = st.number_input(
            "Largeur (mm)", min_value=0.1, value=3.0, step=0.1, key="bpa_width"
        )
    with col_l:
        length = st.number_input(
            "Longueur (mm)", min_value=0.1, value=6.0, step=0.1, key="bpa_length"
        )

    if st.button("Calculer l'angle", key="btn_calc_angle"):
        if width > length:
            st.error("La largeur ne peut pas depasser la longueur.")
        else:
            result = api.forensic_bpa_calculate_angle(width, length)
            if result:
                angle = result.get("impact_angle_degrees", 0)
                st.metric("Angle d'impact", f"{angle:.1f} deg")
                st.caption(result.get("formula", ""))

    st.markdown("##### Zone de convergence")
    st.caption(
        "Entrez les coordonnees et directions de plusieurs taches "
        "pour calculer le point de convergence."
    )

    num_stains = st.number_input(
        "Nombre de taches",
        min_value=2,
        max_value=20,
        value=3,
        key="bpa_num_stains",
    )

    stains_data = []
    for i in range(int(num_stains)):
        with st.expander(f"Tache {i + 1}", expanded=(i < 2)):
            c1, c2, c3 = st.columns(3)
            with c1:
                sx = st.number_input(
                    f"X (mm)", value=0.0, key=f"stain_x_{i}"
                )
            with c2:
                sy = st.number_input(
                    f"Y (mm)", value=0.0, key=f"stain_y_{i}"
                )
            with c3:
                sd = st.number_input(
                    f"Direction (deg)", value=0.0,
                    min_value=0.0, max_value=360.0,
                    key=f"stain_dir_{i}",
                )
            c4, c5 = st.columns(2)
            with c4:
                sw = st.number_input(
                    f"Largeur (mm, optionnel)", value=0.0,
                    min_value=0.0, key=f"stain_w_{i}",
                )
            with c5:
                sl = st.number_input(
                    f"Longueur (mm, optionnel)", value=0.0,
                    min_value=0.0, key=f"stain_l_{i}",
                )
            stain: dict = {
                "x": sx,
                "y": sy,
                "direction_degrees": sd,
            }
            if sw > 0 and sl > 0:
                stain["width"] = sw
                stain["length"] = sl
            stains_data.append(stain)

    if st.button("Calculer la convergence", key="btn_calc_convergence"):
        result = api.forensic_bpa_convergence(stains_data)
        if result:
            conv = result.get("convergence", {})
            if "error" not in conv:
                c1, c2, c3 = st.columns(3)
                with c1:
                    st.metric("Centre X", f"{conv.get('center_x', 0):.1f} mm")
                with c2:
                    st.metric("Centre Y", f"{conv.get('center_y', 0):.1f} mm")
                with c3:
                    st.metric(
                        "Confiance",
                        f"{conv.get('confidence', 0):.0%}",
                    )

                origin = result.get("area_of_origin")
                if origin and "error" not in origin:
                    st.write("**Point d'origine estime (3D):**")
                    o1, o2, o3 = st.columns(3)
                    with o1:
                        st.metric("X", f"{origin.get('x', 0):.1f} mm")
                    with o2:
                        st.metric("Y", f"{origin.get('y', 0):.1f} mm")
                    with o3:
                        st.metric(
                            "Hauteur Z",
                            f"{origin.get('z_height', 0):.1f} mm",
                        )
            else:
                st.error(conv.get("error", "Erreur inconnue"))
        else:
            st.error("Le calcul a echoue.")


# ===========================================================================
# Tab 2: Acoustic Analysis
# ===========================================================================

with tab_audio:
    st.subheader("Analyse acoustique forensique")

    audio_file = st.file_uploader(
        "Fichier audio",
        type=["wav", "mp3", "ogg", "m4a", "flac"],
        key="audio_upload",
    )

    if audio_file is not None:
        st.audio(audio_file)

        col_trans, col_full, col_events = st.columns(3)

        with col_trans:
            if st.button("Transcrire", key="btn_audio_transcribe"):
                with st.spinner("Transcription en cours (voxtral)..."):
                    result = api.forensic_audio_transcribe(audio_file)
                if result:
                    st.write("**Transcription:**")
                    st.text_area(
                        "Resultat",
                        value=result.get("transcription", ""),
                        height=300,
                        key="audio_transcript_result",
                        disabled=True,
                    )
                else:
                    st.error("La transcription a echoue.")

        with col_full:
            if st.button("Analyse forensique", key="btn_audio_analyze"):
                with st.spinner("Analyse forensique audio en cours..."):
                    result = api.forensic_audio_analyze(audio_file)
                if result:
                    st.success("Analyse terminee")
                    if result.get("transcription"):
                        with st.expander("Transcription"):
                            st.write(result["transcription"])
                    if result.get("forensic_analysis"):
                        st.write("**Analyse forensique:**")
                        st.write(result["forensic_analysis"])
                    events = result.get("events", [])
                    if events:
                        st.write(f"**Evenements detectes ({len(events)}):**")
                        for ev in events[:20]:
                            ts = ev.get("timestamp_sec", 0)
                            etype = ev.get("type", "?")
                            dur = ev.get("duration_sec", 0)
                            amp_db = ev.get("amplitude_db", "?")
                            st.write(
                                f"- **{ts:.1f}s** [{etype}] "
                                f"duree={dur:.2f}s, amplitude={amp_db} dB"
                            )
                else:
                    st.error("L'analyse a echoue.")

        with col_events:
            if st.button("Detecter evenements", key="btn_audio_events"):
                with st.spinner("Detection d'evenements audio..."):
                    result = api.forensic_audio_events(audio_file)
                if result:
                    events = result.get("events", [])
                    st.write(f"**{len(events)} evenement(s) detecte(s)**")
                    if events:
                        table_data = []
                        for ev in events:
                            table_data.append({
                                "Temps (s)": f"{ev.get('timestamp_sec', 0):.2f}",
                                "Type": ev.get("type", "?"),
                                "Duree (s)": f"{ev.get('duration_sec', 0):.3f}",
                                "Amplitude (dB)": ev.get("amplitude_db", "?"),
                            })
                        st.dataframe(
                            table_data,
                            use_container_width=True,
                            hide_index=True,
                        )
                else:
                    st.error("La detection a echoue.")


# ===========================================================================
# Tab 3: Trace Analysis
# ===========================================================================

with tab_traces:
    st.subheader("Analyse de traces physiques")
    st.caption(
        "Analysez des empreintes digitales, marques d'outils, "
        "traces de pneus, empreintes de chaussures, fractures de verre, etc."
    )

    trace_col1, trace_col2 = st.columns([2, 1])

    with trace_col1:
        trace_file = st.file_uploader(
            "Photo de la trace",
            type=["png", "jpg", "jpeg", "webp", "tiff"],
            key="trace_upload",
        )

    with trace_col2:
        trace_type = st.selectbox(
            "Type de trace",
            options=[
                "auto",
                "fingerprint",
                "tool_mark",
                "tire_track",
                "shoe_print",
                "glass_fracture",
                "fabric",
                "hair",
                "fiber",
            ],
            format_func=lambda x: {
                "auto": "Automatique",
                "fingerprint": "Empreinte digitale",
                "tool_mark": "Marque d'outil",
                "tire_track": "Trace de pneu",
                "shoe_print": "Empreinte de chaussure",
                "glass_fracture": "Fracture de verre",
                "fabric": "Tissu",
                "hair": "Cheveu / Poil",
                "fiber": "Fibre",
            }.get(x, x),
            key="trace_type_select",
        )

    if trace_file is not None:
        st.image(trace_file, caption=trace_file.name, use_container_width=True)

        if st.button("Analyser la trace", key="btn_trace_analyze"):
            with st.spinner("Analyse de trace en cours..."):
                result = api.forensic_trace_analyze(trace_file, trace_type)
            if result:
                st.success("Analyse terminee")

                # Display key metrics
                c1, c2, c3 = st.columns(3)
                with c1:
                    detected_type = result.get(
                        "trace_type", result.get("analysis", "?")
                    )
                    st.metric("Type detecte", detected_type)
                with c2:
                    quality = result.get("quality", "?")
                    st.metric("Qualite", quality)
                with c3:
                    fv = result.get("forensic_value", "?")
                    st.metric("Valeur forensique", fv)

                if "description" in result:
                    st.write("**Description:**")
                    st.write(result["description"])

                chars = result.get("characteristics", [])
                if chars:
                    st.write("**Caracteristiques:**")
                    for c in chars:
                        st.write(f"- {c}")

                features = result.get("identifying_features", [])
                if features:
                    st.write("**Elements distinctifs:**")
                    for f in features:
                        st.write(f"- {f}")

                recs = result.get("recommendations", [])
                if recs:
                    st.write("**Recommandations:**")
                    for r in recs:
                        st.write(f"- {r}")

                if "confidence" in result:
                    st.progress(
                        float(result["confidence"]),
                        text=f"Confiance: {result['confidence']:.0%}",
                    )
            else:
                st.error("L'analyse a echoue.")

    st.divider()

    # --- Comparison ---
    st.markdown("#### Comparaison de traces")
    st.caption("Comparez deux traces pour evaluer si elles proviennent de la meme source.")

    cmp_col1, cmp_col2 = st.columns(2)
    with cmp_col1:
        trace_file_1 = st.file_uploader(
            "Trace 1",
            type=["png", "jpg", "jpeg", "webp", "tiff"],
            key="trace_cmp_1",
        )
        if trace_file_1:
            st.image(trace_file_1, use_container_width=True)

    with cmp_col2:
        trace_file_2 = st.file_uploader(
            "Trace 2",
            type=["png", "jpg", "jpeg", "webp", "tiff"],
            key="trace_cmp_2",
        )
        if trace_file_2:
            st.image(trace_file_2, use_container_width=True)

    if trace_file_1 and trace_file_2:
        if st.button("Comparer les traces", key="btn_trace_compare"):
            with st.spinner("Comparaison en cours..."):
                result = api.forensic_trace_compare(trace_file_1, trace_file_2)
            if result:
                st.success("Comparaison terminee")

                score = result.get("similarity_score", 0)
                prob = result.get("same_source_probability", "?")
                st.metric("Score de similarite", f"{score:.0%}")
                st.metric("Meme source", prob)

                matching = result.get("matching_features", [])
                if matching:
                    st.write("**Elements communs:**")
                    for m in matching:
                        st.write(f"- {m}")

                differing = result.get("differing_features", [])
                if differing:
                    st.write("**Differences:**")
                    for d in differing:
                        st.write(f"- {d}")

                if "conclusion" in result:
                    st.write("**Conclusion:**")
                    st.write(result["conclusion"])
            else:
                st.error("La comparaison a echoue.")


# ===========================================================================
# Tab 4: Auto-analysis
# ===========================================================================

with tab_auto:
    st.subheader("Analyse forensique automatique")
    st.caption(
        "Lance automatiquement les analyses BPA, acoustique et traces "
        "sur toutes les preuves du dossier actif."
    )

    evidence_list = api.list_evidence(case_id)
    if evidence_list:
        image_count = sum(
            1 for e in evidence_list if e.get("evidence_type") == "image"
        )
        audio_count = sum(
            1 for e in evidence_list if e.get("evidence_type") == "audio"
        )
        st.write(
            f"**Preuves dans le dossier:** "
            f"{len(evidence_list)} total, "
            f"{image_count} images, "
            f"{audio_count} audio"
        )
    else:
        st.info("Aucune preuve dans ce dossier.")

    if st.button(
        "Lancer l'analyse forensique automatique",
        key="btn_forensic_auto",
        type="primary",
    ):
        with st.spinner("Analyse forensique automatique en cours..."):
            result = api.forensic_auto_analyze(case_id)

        if result:
            processed = result.get("evidence_processed", 0)
            errors = result.get("errors_count", 0)

            if processed > 0:
                st.success(f"{processed} analyse(s) effectuee(s).")
            if errors > 0:
                st.warning(f"{errors} erreur(s) rencontree(s).")

            results_list = result.get("results", [])
            for r in results_list:
                ev_id = r.get("evidence_id", "?")[:8]
                a_type = r.get("analysis_type", "?")
                with st.expander(f"{a_type} -- Evidence {ev_id}..."):
                    st.json(r.get("result", {}))

            errors_list = result.get("errors", [])
            for e in errors_list:
                st.error(
                    f"Erreur ({e.get('analysis_type', '?')}) "
                    f"sur {e.get('evidence_id', '?')[:8]}...: "
                    f"{e.get('error', '')}"
                )
        else:
            st.error("L'analyse automatique a echoue.")


# ===========================================================================
# Tab 5: Simulations physiques
# ===========================================================================

with tab_sim:
    st.subheader("Simulations physiques forensiques")
    st.caption(
        "Simulations scientifiques pour la reconstruction de scenes: "
        "balistique de gouttes de sang, pattern cast-off, propagation "
        "sonore, estimation du point d'origine."
    )

    sim_sub = st.radio(
        "Type de simulation",
        [
            "Trajectoire de goutte",
            "Pattern cast-off",
            "Propagation sonore",
            "Point d'origine",
            "Datasets The Well",
        ],
        horizontal=True,
        key="sim_type_radio",
    )

    st.markdown("---")

    # -------------------------------------------------------------------
    # Sim: Blood drop trajectory
    # -------------------------------------------------------------------
    if sim_sub == "Trajectoire de goutte":
        st.markdown("#### Simulation de trajectoire d'une goutte de sang")
        st.caption(
            "Modelise la trajectoire balistique d'une goutte de sang avec "
            "resistance de l'air (trainee dependante du nombre de Reynolds), "
            "et calcule la forme de la tache elliptique a l'impact."
        )

        col1, col2, col3 = st.columns(3)
        with col1:
            sim_drop_vel = st.slider(
                "Vitesse initiale (m/s)",
                0.5, 30.0, 5.0, 0.5,
                key="sim_drop_vel",
                help="Goutte passive: 1-3 m/s. Projection: 5-10 m/s. "
                     "Impact haute velocite: 15-30 m/s.",
            )
        with col2:
            sim_drop_angle = st.slider(
                "Angle de lancement (deg)",
                -89.0, 89.0, 30.0, 1.0,
                key="sim_drop_angle",
                help="Positif = vers le haut, negatif = vers le bas.",
            )
        with col3:
            sim_drop_height = st.slider(
                "Hauteur de depart (m)",
                0.1, 5.0, 1.5, 0.1,
                key="sim_drop_height",
            )

        with st.expander("Proprietes du sang (avance)"):
            sc_a, sc_b = st.columns(2)
            with sc_a:
                sim_bp_density = st.number_input(
                    "Densite (kg/m3)", value=1060.0, step=10.0, key="sim_bp_d",
                )
                sim_bp_visc = st.number_input(
                    "Viscosite (Pa.s)", value=0.004, step=0.001,
                    format="%.4f", key="sim_bp_v",
                )
            with sc_b:
                sim_bp_st = st.number_input(
                    "Tension de surface (N/m)", value=0.058, step=0.001,
                    format="%.3f", key="sim_bp_st",
                )
                sim_bp_dia = st.number_input(
                    "Diametre de goutte (mm)", value=2.0, step=0.1,
                    key="sim_bp_dia",
                )
            sim_surf_angle = st.slider(
                "Inclinaison surface (deg)", 0.0, 45.0, 0.0, 1.0,
                key="sim_surf_ang",
            )

        if st.button("Simuler la goutte", key="btn_sim_drop_tab5", type="primary"):
            blood_props = {
                "density": sim_bp_density,
                "viscosity": sim_bp_visc,
                "surface_tension": sim_bp_st,
                "drop_diameter": sim_bp_dia / 1000.0,
            }
            with st.spinner("Simulation en cours..."):
                result = api.sim_blood_drop(
                    velocity=sim_drop_vel,
                    angle=sim_drop_angle,
                    height=sim_drop_height,
                    surface_angle=sim_surf_angle,
                    blood_properties=blood_props,
                )

            if result:
                from frontend.components.physics_viz import (
                    render_blood_trajectory,
                    render_impact_angle_diagram,
                )

                cr1, cr2, cr3, cr4 = st.columns(4)
                cr1.metric("Angle d'impact", f"{result['impact_angle']:.1f} deg")
                cr2.metric("Vitesse impact", f"{result['impact_velocity']:.2f} m/s")
                cr3.metric("Temps de vol", f"{result['travel_time']*1000:.1f} ms")
                stain = result["stain_shape"]
                cr4.metric("Tache", f"{stain['width_mm']:.1f} x {stain['length_mm']:.1f} mm")

                fig_t = render_blood_trajectory(result)
                st.plotly_chart(fig_t, use_container_width=True)

                fig_a = render_impact_angle_diagram(
                    stain["width_mm"], stain["length_mm"], result["impact_angle"],
                )
                st.plotly_chart(fig_a, use_container_width=True)

                with st.expander("Donnees brutes"):
                    st.json({k: v for k, v in result.items() if k != "trajectory"})
            else:
                st.error("La simulation a echoue.")

    # -------------------------------------------------------------------
    # Sim: Cast-off pattern
    # -------------------------------------------------------------------
    elif sim_sub == "Pattern cast-off":
        st.markdown("#### Simulation de pattern cast-off")
        st.caption(
            "Simule le detachement de gouttes de sang d'une arme en mouvement. "
            "Les gouttes se detachent quand la force centripete depasse "
            "l'adhesion par tension de surface."
        )

        col1, col2 = st.columns(2)
        with col1:
            sim_co_radius = st.slider(
                "Rayon de rotation (m)", 0.3, 1.5, 0.8, 0.05,
                key="sim_co_r",
                help="Couteau: ~0.5 m. Batte: ~1.0 m.",
            )
            sim_co_speed = st.slider(
                "Vitesse angulaire (rad/s)", 5.0, 60.0, 30.0, 1.0,
                key="sim_co_spd",
                help="Frappe lente: 10-15. Frappe rapide: 30-50.",
            )
            sim_co_drops = st.slider("Nb gouttes", 5, 50, 20, 1, key="sim_co_n")
        with col2:
            sim_co_blen = st.slider(
                "Longueur ensanglantee (m)", 0.05, 0.8, 0.3, 0.05,
                key="sim_co_bl",
            )
            sim_co_h = st.slider(
                "Hauteur pivot (m)", 0.5, 2.5, 1.5, 0.1, key="sim_co_h",
            )
            sim_co_start = st.slider(
                "Angle debut (deg)", -90.0, 90.0, -30.0, 5.0, key="sim_co_sa",
            )
            sim_co_end = st.slider(
                "Angle fin (deg)", 90.0, 270.0, 150.0, 5.0, key="sim_co_ea",
            )

        if st.button("Simuler le cast-off", key="btn_sim_co_tab5", type="primary"):
            with st.spinner(f"Simulation de {sim_co_drops} gouttes..."):
                result = api.sim_cast_off(
                    swing_radius=sim_co_radius,
                    swing_speed=sim_co_speed,
                    num_drops=sim_co_drops,
                    blood_on_weapon_length=sim_co_blen,
                    swing_plane_height=sim_co_h,
                    swing_start_angle=sim_co_start,
                    swing_end_angle=sim_co_end,
                )

            if result:
                drops = result.get("drops", [])
                released = result.get("num_drops_released", 0)

                st.metric("Gouttes detachees", f"{released} / {sim_co_drops}")

                if released == 0:
                    st.warning(
                        "Aucune goutte detachee. Augmentez la vitesse angulaire."
                    )
                else:
                    from frontend.components.physics_viz import render_cast_off_pattern
                    fig = render_cast_off_pattern(drops)
                    st.plotly_chart(fig, use_container_width=True)
            else:
                st.error("La simulation a echoue.")

    # -------------------------------------------------------------------
    # Sim: Sound propagation
    # -------------------------------------------------------------------
    elif sim_sub == "Propagation sonore":
        st.markdown("#### Simulation de propagation sonore")
        st.caption(
            "Modelise la propagation du son (coup de feu, cri, explosion) "
            "en tenant compte de la vitesse du son, de l'absorption "
            "atmospherique (ISO 9613), du vent et du terrain."
        )

        col1, col2, col3 = st.columns(3)
        with col1:
            sim_snd_type = st.selectbox(
                "Type de source",
                options=[
                    ("Coup de feu (pistolet)", 160),
                    ("Coup de feu (fusil)", 170),
                    ("Cri humain", 90),
                    ("Explosion", 180),
                    ("Personnalise", 0),
                ],
                format_func=lambda x: x[0],
                key="sim_snd_type",
            )
            if sim_snd_type[1] == 0:
                sim_snd_db = st.number_input("dB a 1 m", value=140.0, step=5.0, key="sim_snd_db_c")
            else:
                sim_snd_db = float(sim_snd_type[1])
        with col2:
            sim_snd_temp = st.slider("Temperature (C)", -20.0, 45.0, 20.0, 1.0, key="sim_snd_t")
            sim_snd_hum = st.slider("Humidite (%)", 0.0, 100.0, 50.0, 5.0, key="sim_snd_h")
            sim_snd_freq = st.slider("Frequence (Hz)", 100, 8000, 2000, 100, key="sim_snd_f")
        with col3:
            sim_snd_terrain = st.selectbox(
                "Terrain", ["urban", "rural", "indoor"],
                format_func={"urban": "Urbain", "rural": "Rural",
                             "indoor": "Interieur"}.__getitem__,
                key="sim_snd_ter",
            )
            sim_snd_wind = st.slider("Vent (m/s)", 0.0, 30.0, 0.0, 1.0, key="sim_snd_w")
            sim_snd_wdir = st.slider("Dir. vent (deg)", 0.0, 359.0, 0.0, 5.0, key="sim_snd_wd")

        st.caption("Auditeurs: x,y,z ; x,y,z ; ...")
        sim_listeners_str = st.text_input(
            "Positions auditeurs",
            value="50,0,1.7 ; 100,30,1.7 ; 200,-50,1.7 ; 500,100,1.7",
            key="sim_listen_input",
        )

        sim_listeners = []
        sim_parse_err = False
        for part in sim_listeners_str.split(";"):
            part = part.strip()
            if not part:
                continue
            coords = part.split(",")
            if len(coords) != 3:
                sim_parse_err = True
                break
            try:
                sim_listeners.append([float(c.strip()) for c in coords])
            except ValueError:
                sim_parse_err = True
                break

        if sim_parse_err:
            st.error("Format invalide.")

        if not sim_parse_err and sim_listeners and st.button(
            "Simuler", key="btn_sim_sound_tab5", type="primary"
        ):
            with st.spinner("Simulation..."):
                result = api.sim_sound(
                    source=[0.0, 0.0, 1.5],
                    listeners=sim_listeners,
                    source_db=sim_snd_db,
                    frequency=float(sim_snd_freq),
                    temperature=sim_snd_temp,
                    humidity=sim_snd_hum,
                    wind_speed=sim_snd_wind,
                    wind_direction=sim_snd_wdir,
                    terrain=sim_snd_terrain,
                )

            if result:
                from frontend.components.physics_viz import render_sound_propagation

                arrivals = result.get("arrivals", [])
                c = result.get("speed_of_sound", 343)
                cm1, cm2, cm3 = st.columns(3)
                cm1.metric("Vitesse du son", f"{c:.1f} m/s")
                if arrivals:
                    cm2.metric("Delai min", f"{min(a['delay_sec'] for a in arrivals)*1000:.1f} ms")
                    cm3.metric("Delai max", f"{max(a['delay_sec'] for a in arrivals)*1000:.1f} ms")

                fig = render_sound_propagation(result)
                st.plotly_chart(fig, use_container_width=True)

                st.subheader("Detail")
                for a in arrivals:
                    audible = "AUDIBLE" if a["above_hearing_threshold"] else "INAUDIBLE"
                    st.write(
                        f"**L{a['listener_id']}** | "
                        f"{a['distance_m']:.0f} m | "
                        f"{a['delay_sec']*1000:.1f} ms | "
                        f"{a['estimated_loudness_db']:.0f} dB | "
                        f"{audible}"
                    )
            else:
                st.error("La simulation a echoue.")

    # -------------------------------------------------------------------
    # Sim: Origin of impact
    # -------------------------------------------------------------------
    elif sim_sub == "Point d'origine":
        st.markdown("#### Estimation du point d'origine")
        st.caption(
            "A partir de mesures de taches de sang (position, largeur, "
            "longueur, direction), estime le point de convergence par "
            "la methode des tangentes: alpha = arcsin(W/L)."
        )

        sim_num_stains = st.number_input(
            "Nombre de taches", min_value=2, max_value=20, value=4, step=1,
            key="sim_origin_n",
        )

        sim_stains = []
        for i in range(int(sim_num_stains)):
            with st.expander(f"Tache {i+1}", expanded=i < 3):
                c1, c2, c3, c4, c5 = st.columns(5)
                ox = c1.number_input("X (m)", value=float(i)*0.5, step=0.1, key=f"sim_ox_{i}")
                oy = c2.number_input("Y (m)", value=float(i)*0.3, step=0.1, key=f"sim_oy_{i}")
                ow = c3.number_input("W (mm)", value=2.0, step=0.1, key=f"sim_ow_{i}", min_value=0.1)
                ol = c4.number_input("L (mm)", value=4.0, step=0.1, key=f"sim_ol_{i}", min_value=0.1)
                od = c5.number_input("Dir (deg)", value=float(45+i*30), step=5.0,
                                     key=f"sim_od_{i}", min_value=0.0, max_value=359.9)
                sim_stains.append({
                    "x": ox, "y": oy,
                    "width_mm": ow, "length_mm": ol,
                    "direction": od,
                })

        if st.button("Estimer l'origine", key="btn_sim_origin_tab5", type="primary"):
            with st.spinner("Calcul..."):
                result = api.sim_origin(stains=sim_stains)

            if result and "error" not in result:
                from frontend.components.physics_viz import render_origin_convergence

                cm1, cm2, cm3, cm4 = st.columns(4)
                cm1.metric("X", f"{result['origin_x']:.2f} m")
                cm2.metric("Y", f"{result['origin_y']:.2f} m")
                cm3.metric("Hauteur Z", f"{result['origin_z']:.2f} m")
                cm4.metric("Residuel", f"{result['residual_m']:.3f} m")

                fig = render_origin_convergence(result, sim_stains)
                st.plotly_chart(fig, use_container_width=True)
            elif result and "error" in result:
                st.error(result["error"])
            else:
                st.error("L'estimation a echoue.")

    # -------------------------------------------------------------------
    # Sim: The Well datasets
    # -------------------------------------------------------------------
    elif sim_sub == "Datasets The Well":
        st.markdown("#### Datasets The Well (PolymathicAI)")
        st.caption(
            "The Well est une collection de 15 To de simulations physiques "
            "(NeurIPS 2024). Les datasets ci-dessous sont pertinents pour "
            "l'analyse forensique."
        )

        with st.spinner("Chargement..."):
            result = api.list_sim_datasets()

        if result:
            installed = result.get("the_well_installed", False)
            if installed:
                st.success("Package the-well installe.")
            else:
                st.info(
                    "Package non installe. Pour acceder aux datasets: "
                    "`pip install the-well`"
                )

            for ds in result.get("datasets", []):
                with st.expander(f"{ds['name']} ({ds['type']})"):
                    st.write(f"**Description:** {ds.get('description', '-')}")
                    st.write(f"**Pertinence:** {ds.get('relevance', '-')}")
                    st.write(f"**Taille:** ~{ds.get('size_gb', '?')} Go")
                    st.write(f"**HuggingFace:** `{ds.get('hf_path', '-')}`")
        else:
            st.error("Impossible de charger les datasets.")
