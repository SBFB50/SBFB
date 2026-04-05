"""
NEXUS -- Entities page.

Browse extracted entities filtered by type, view details and mentions
across evidence items.
"""

from __future__ import annotations
import sys; from pathlib import Path; sys.path.insert(0, str(Path(__file__).resolve().parent.parent)) if str(Path(__file__).resolve().parent.parent) not in sys.path else None  # noqa: E402

import streamlit as st
import pandas as pd

from frontend.api_client import api

# ---------------------------------------------------------------------------
# Guard
# ---------------------------------------------------------------------------

st.title("Entites")

case_id = st.session_state.get("case_id")
if not case_id:
    st.info("Selectionnez ou creez un dossier dans la barre laterale.")
    st.stop()

# ---------------------------------------------------------------------------
# Filters
# ---------------------------------------------------------------------------

entity_types = [
    "Tous", "person", "location", "phone", "vehicle", "organization",
    "date", "money", "ip", "email", "account", "weapon", "drug", "other",
]

TYPE_LABELS = {
    "person": "Personne",
    "location": "Lieu",
    "phone": "Telephone",
    "vehicle": "Vehicule",
    "organization": "Organisation",
    "date": "Date",
    "money": "Argent",
    "ip": "Adresse IP",
    "email": "Email",
    "account": "Compte",
    "weapon": "Arme",
    "drug": "Drogue",
    "other": "Autre",
}

TYPE_ICONS = {
    "person": "👤",
    "location": "📍",
    "phone": "📞",
    "vehicle": "🚗",
    "organization": "🏢",
    "date": "📅",
    "money": "💰",
    "ip": "🌐",
    "email": "📧",
    "account": "🔑",
    "weapon": "🔫",
    "drug": "💊",
    "other": "❓",
}

selected_type = st.selectbox(
    "Filtrer par type",
    entity_types,
    index=0,
    format_func=lambda t: t if t == "Tous" else f"{TYPE_ICONS.get(t, '')} {TYPE_LABELS.get(t, t)}",
)

# ---------------------------------------------------------------------------
# Fetch entities
# ---------------------------------------------------------------------------

entities = api.list_entities(
    case_id=case_id,
    entity_type=selected_type if selected_type != "Tous" else None,
)

if not entities:
    st.caption("Aucune entite extraite pour ce dossier.")
    st.stop()

st.caption(f"{len(entities)} entite(s) trouvee(s)")

# ---------------------------------------------------------------------------
# Summary table
# ---------------------------------------------------------------------------

table_data = []
for ent in entities:
    first_seen = ent.get("first_seen", "")
    if isinstance(first_seen, str) and len(first_seen) > 10:
        first_seen = first_seen[:10]

    table_data.append({
        "Nom": ent.get("name", "?"),
        "Type": TYPE_LABELS.get(ent.get("entity_type", "other"), ent.get("entity_type", "?")),
        "Description": (ent.get("description") or "")[:80],
        "Premiere apparition": first_seen or "N/A",
        "id": ent.get("id", ""),
    })

df = pd.DataFrame(table_data)

# Display table without the hidden id column
st.dataframe(
    df[["Nom", "Type", "Description", "Premiere apparition"]],
    use_container_width=True,
    hide_index=True,
)

# ---------------------------------------------------------------------------
# Entity detail viewer
# ---------------------------------------------------------------------------

st.markdown("---")
st.subheader("Detail d'une entite")

entity_options = {ent["id"]: f"{TYPE_ICONS.get(ent.get('entity_type','other'),'')} {ent.get('name', '?')}" for ent in entities}
selected_entity_id = st.selectbox(
    "Selectionner une entite",
    options=list(entity_options.keys()),
    format_func=lambda eid: entity_options[eid],
)

if selected_entity_id:
    entity = api.get_entity(selected_entity_id)

    if entity:
        col1, col2 = st.columns([2, 1])

        with col1:
            etype = entity.get("entity_type", "other")
            st.markdown(
                f"### {TYPE_ICONS.get(etype, '')} {entity.get('name', '?')}"
            )
            st.markdown(f"**Type:** {TYPE_LABELS.get(etype, etype)}")

            description = entity.get("description")
            if description:
                st.markdown(f"**Description:** {description}")

            aliases = entity.get("aliases")
            if aliases:
                st.markdown(f"**Alias:** {', '.join(aliases)}")

            first_seen = entity.get("first_seen")
            if first_seen:
                display_date = first_seen[:10] if isinstance(first_seen, str) else str(first_seen)
                st.markdown(f"**Premiere apparition:** {display_date}")

            metadata = entity.get("metadata")
            if metadata and isinstance(metadata, dict):
                st.markdown("**Metadonnees:**")
                for k, v in metadata.items():
                    st.markdown(f"- **{k}:** {v}")

        with col2:
            st.markdown(f"**ID:** `{entity['id'][:12]}...`")
            created = entity.get("created_at", "")
            if isinstance(created, str) and len(created) > 19:
                created = created[:19]
            st.markdown(f"**Cree le:** {created}")

        # Mentions
        st.markdown("---")
        st.subheader("Mentions dans les preuves")

        mentions = api.get_entity_mentions(selected_entity_id)

        if not mentions:
            st.caption("Aucune mention trouvee.")
        else:
            st.caption(f"{len(mentions)} mention(s)")

            for mention in mentions:
                confidence = mention.get("confidence", 0)
                conf_pct = f"{confidence * 100:.0f}%"

                if confidence >= 0.8:
                    conf_color = "#27AE60"
                elif confidence >= 0.5:
                    conf_color = "#F39C12"
                else:
                    conf_color = "#E74C3C"

                with st.expander(
                    f"Evidence: {mention.get('evidence_id', '?')[:12]}... "
                    f"(confiance: {conf_pct})"
                ):
                    st.markdown(
                        f"**Confiance:** "
                        f"<span style='color:{conf_color};font-weight:700'>{conf_pct}</span>",
                        unsafe_allow_html=True,
                    )

                    context = mention.get("context")
                    if context:
                        st.markdown(f"**Contexte:** {context}")

                    st.markdown(f"**Evidence ID:** `{mention.get('evidence_id', '?')}`")
                    st.markdown(f"**Mention ID:** `{mention.get('id', '?')[:12]}...`")
