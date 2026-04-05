"""
NEXUS -- Hypotheses page.

Multi-line score evolution chart, hypothesis list with actions,
detailed view with snapshots and supporting/contradicting elements.
"""

from __future__ import annotations
import sys; from pathlib import Path; sys.path.insert(0, str(Path(__file__).resolve().parent.parent)) if str(Path(__file__).resolve().parent.parent) not in sys.path else None  # noqa: E402

import streamlit as st

from frontend.api_client import api
from frontend.components.hypothesis_chart import (
    render_evolution_chart,
    render_single_hypothesis_chart,
)

# ---------------------------------------------------------------------------
# Guard
# ---------------------------------------------------------------------------

st.title("Hypotheses")

case_id = st.session_state.get("case_id")
if not case_id:
    st.info("Selectionnez ou creez un dossier dans la barre laterale.")
    st.stop()

# ---------------------------------------------------------------------------
# Fetch data
# ---------------------------------------------------------------------------

hypotheses = api.list_hypotheses(case_id)

# ---------------------------------------------------------------------------
# Evolution chart (multi-line, all hypotheses)
# ---------------------------------------------------------------------------

st.subheader("Evolution des scores")

if hypotheses:
    # Build data for the multi-line chart
    chart_data = []
    for hyp in hypotheses:
        evolution = api.get_hypothesis_evolution(hyp["id"])
        if evolution:
            chart_data.append({
                "hypothesis_id": hyp["id"],
                "title": hyp.get("title", "Sans titre"),
                "snapshots": [
                    {
                        "date": pt.get("date", ""),
                        "score": pt.get("score", 0),
                        "trigger": pt.get("trigger", "N/A"),
                    }
                    for pt in evolution
                ],
            })

    render_evolution_chart(chart_data)
else:
    st.caption("Aucune hypothese pour ce dossier.")

# ---------------------------------------------------------------------------
# Action buttons
# ---------------------------------------------------------------------------

st.markdown("---")

btn_col1, btn_col2, btn_col3 = st.columns(3)

with btn_col1:
    if st.button("Generer via IA", type="primary", use_container_width=True):
        result = api.generate_hypotheses(case_id)
        if result:
            st.success("Generation d'hypotheses lancee en arriere-plan.")
            st.rerun()

with btn_col2:
    if st.button("Re-evaluer toutes", use_container_width=True):
        result = api.evaluate_all_hypotheses(case_id)
        if result:
            st.success("Re-evaluation lancee en arriere-plan.")

with btn_col3:
    contradictions = None
    if st.button("Detecter contradictions", use_container_width=True):
        contradictions = api.get_contradictions(case_id)

if contradictions:
    st.subheader("Contradictions detectees")
    if not contradictions:
        st.caption("Aucune contradiction trouvee.")
    else:
        for c in contradictions:
            with st.expander(c.get("title", "Contradiction")):
                st.markdown(c.get("description", ""))
                if c.get("evidence_ids"):
                    st.markdown(f"**Preuves impliquees:** {', '.join(c['evidence_ids'])}")

# ---------------------------------------------------------------------------
# Create hypothesis form
# ---------------------------------------------------------------------------

st.markdown("---")
st.subheader("Creer une hypothese")

with st.form("create_hypothesis_form", clear_on_submit=True):
    new_title = st.text_input("Titre *")
    new_description = st.text_area("Description *", height=100)
    new_score = st.slider("Score initial", 0.0, 100.0, 50.0, 1.0)
    created = st.form_submit_button("Creer")

    if created:
        if not new_title or not new_description:
            st.warning("Le titre et la description sont requis.")
        else:
            result = api.create_hypothesis(case_id, {
                "case_id": case_id,
                "title": new_title,
                "description": new_description,
                "current_score": new_score,
            })
            if result:
                st.success(f"Hypothese creee: {result.get('title', '')}")
                st.rerun()

# ---------------------------------------------------------------------------
# Hypothesis list with status filter
# ---------------------------------------------------------------------------

st.markdown("---")
st.subheader("Liste des hypotheses")

status_filter = st.selectbox(
    "Filtrer par statut",
    ["Tous", "active", "refuted", "confirmed", "merged"],
    index=0,
)

displayed = hypotheses
if status_filter != "Tous":
    displayed = [h for h in hypotheses if h.get("status") == status_filter]

if not displayed:
    st.caption("Aucune hypothese correspond aux filtres.")
else:
    # Sort by score descending
    displayed.sort(key=lambda h: h.get("current_score", 0), reverse=True)

    for hyp in displayed:
        score = hyp.get("current_score", 0)
        status = hyp.get("status", "active")

        if score >= 70:
            score_color = "#27AE60"
        elif score >= 40:
            score_color = "#F39C12"
        else:
            score_color = "#E74C3C"

        status_icon = {
            "active": "🟢",
            "refuted": "🔴",
            "confirmed": "✅",
            "merged": "🔗",
        }.get(status, "⚪")

        with st.expander(
            f"{status_icon} {hyp.get('title', 'Sans titre')} — "
            f"Score: {score:.1f}/100 [{status}]"
        ):
            st.markdown(f"**Description:** {hyp.get('description', '')}")
            st.markdown(
                f"**Score:** "
                f"<span style='color:{score_color};font-weight:700;font-size:1.1em'>"
                f"{score:.1f}/100</span>",
                unsafe_allow_html=True,
            )
            st.markdown(f"**Statut:** <span class='status-{status}'>{status}</span>",
                        unsafe_allow_html=True)

            updated = hyp.get("updated_at", "")
            if isinstance(updated, str) and len(updated) > 19:
                updated = updated[:19]
            st.caption(f"Derniere mise a jour: {updated}")

            # Action buttons for individual hypothesis
            act1, act2 = st.columns(2)

            with act1:
                eval_key = f"eval_{hyp['id']}"
                if st.button("Re-evaluer", key=eval_key, use_container_width=True):
                    result = api.evaluate_hypothesis(hyp["id"])
                    if result:
                        st.success("Evaluation lancee en arriere-plan.")

            with act2:
                detail_key = f"detail_{hyp['id']}"
                show_detail = st.button(
                    "Voir detail",
                    key=detail_key,
                    use_container_width=True,
                )

            # Detailed view: snapshots + evolution chart
            if show_detail:
                st.markdown("---")

                # Single hypothesis evolution chart
                evolution = api.get_hypothesis_evolution(hyp["id"])
                if evolution:
                    render_single_hypothesis_chart(evolution, hyp.get("title", ""))

                # Snapshots history
                snapshots = api.get_hypothesis_snapshots(hyp["id"])

                if snapshots:
                    st.markdown("**Historique des evaluations:**")

                    # Show newest first
                    for snap in reversed(snapshots):
                        snap_date = snap.get("created_at", "?")
                        if isinstance(snap_date, str) and len(snap_date) > 19:
                            snap_date = snap_date[:19]

                        snap_score = snap.get("score", 0)
                        trigger = snap.get("trigger", "N/A")
                        model = snap.get("model_used", "N/A")

                        st.markdown(
                            f"- **{snap_date}** — Score: {snap_score:.1f} | "
                            f"Trigger: {trigger} | Modele: {model}"
                        )

                        reasoning = snap.get("reasoning")
                        if reasoning:
                            st.caption(f"  Raisonnement: {reasoning[:300]}")

                        # Supporting elements
                        supporting = snap.get("supporting")
                        if supporting:
                            if isinstance(supporting, list):
                                st.markdown("  **Elements supportant:**")
                                for s in supporting:
                                    if isinstance(s, dict):
                                        st.markdown(f"  - {s.get('text', s)}")
                                    else:
                                        st.markdown(f"  - {s}")

                        # Contradicting elements
                        contradicting = snap.get("contradicting")
                        if contradicting:
                            if isinstance(contradicting, list):
                                st.markdown("  **Elements contredisant:**")
                                for c in contradicting:
                                    if isinstance(c, dict):
                                        st.markdown(f"  - {c.get('text', c)}")
                                    else:
                                        st.markdown(f"  - {c}")
                else:
                    st.caption("Aucun snapshot disponible.")
