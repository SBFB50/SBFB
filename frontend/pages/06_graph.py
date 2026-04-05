import sys; from pathlib import Path; sys.path.insert(0, str(Path(__file__).resolve().parent.parent)) if str(Path(__file__).resolve().parent.parent) not in sys.path else None
"""
NEXUS -- Knowledge graph visualisation page.

Interactive Neo4j graph viewer with type-based filtering,
shortest path computation, and community cluster detection.
"""

import streamlit as st
from frontend.api_client import api
from frontend.components.system_stats import render_system_stats; render_system_stats()
from frontend.components.graph_viewer import (
    render_graph,
    render_node_details,
    TYPE_COLORS,
)

st.header("Graphe de connaissances")

case_id = st.session_state.get("case_id")
if not case_id:
    st.warning("Selectionnez un dossier dans la barre laterale.")
    st.stop()

# ------------------------------------------------------------------
# Load graph data
# ------------------------------------------------------------------

graph_data = api.get_graph(case_id)
if not graph_data:
    st.info(
        "Le graphe est vide. Ajoutez des preuves et lancez une analyse "
        "pour peupler le graphe d'entites et de relations."
    )
    st.stop()

all_nodes = graph_data.get("nodes", [])
all_edges = graph_data.get("edges", [])

# Collect unique node types present in the graph
node_types_present = sorted({n.get("type", "other").lower() for n in all_nodes})

# ------------------------------------------------------------------
# Sidebar filters
# ------------------------------------------------------------------

st.sidebar.markdown("### Filtres du graphe")

selected_types = st.sidebar.multiselect(
    "Types de noeuds",
    options=node_types_present,
    default=node_types_present,
    format_func=lambda t: f"{t.capitalize()} ({sum(1 for n in all_nodes if n.get('type','other').lower()==t)})",
)

physics_enabled = st.sidebar.checkbox("Simulation physique", value=True)
graph_height = st.sidebar.slider("Hauteur (px)", 300, 1000, 600, step=50)

# ------------------------------------------------------------------
# Stats row
# ------------------------------------------------------------------

stats = api.get_graph_stats(case_id)

if stats:
    c1, c2, c3 = st.columns(3)
    c1.metric("Noeuds", stats.get("node_count", len(all_nodes)))
    c2.metric("Relations", stats.get("edge_count", len(all_edges)))
    c3.metric("Types", stats.get("type_count", len(node_types_present)))
else:
    c1, c2 = st.columns(2)
    c1.metric("Noeuds", len(all_nodes))
    c2.metric("Relations", len(all_edges))

# ------------------------------------------------------------------
# Colour legend
# ------------------------------------------------------------------

with st.expander("Legende des couleurs", expanded=False):
    legend_cols = st.columns(min(len(node_types_present), 6) or 1)
    for i, ntype in enumerate(node_types_present):
        color = TYPE_COLORS.get(ntype, "#BDC3C7")
        legend_cols[i % len(legend_cols)].markdown(
            f"<span style='display:inline-block;width:12px;height:12px;"
            f"background:{color};border-radius:50%;margin-right:6px'></span>"
            f"{ntype.capitalize()}",
            unsafe_allow_html=True,
        )

# ------------------------------------------------------------------
# Main graph
# ------------------------------------------------------------------

st.subheader("Graphe interactif")

selected_node_id = render_graph(
    graph_data,
    height=graph_height,
    physics=physics_enabled,
    node_filter=selected_types if selected_types != node_types_present else None,
)

# Show details of selected node
if selected_node_id:
    node_map = {n["id"]: n for n in all_nodes}
    if selected_node_id in node_map:
        st.markdown("---")
        st.subheader("Details du noeud selectionne")
        render_node_details(node_map[selected_node_id])

        # Neighbours
        with st.expander("Voisins directs"):
            neighbors = api.get_neighbors(case_id, selected_node_id, depth=1)
            if neighbors:
                nb_nodes = neighbors.get("nodes", [])
                if nb_nodes:
                    for nb in nb_nodes:
                        if nb["id"] != selected_node_id:
                            ntype = nb.get("type", "other")
                            st.markdown(
                                f"- **{nb.get('label', nb['id'])}** ({ntype})"
                            )
                else:
                    st.caption("Aucun voisin direct.")
            else:
                st.caption("Impossible de charger les voisins.")

# ------------------------------------------------------------------
# Shortest path
# ------------------------------------------------------------------

st.markdown("---")
st.subheader("Plus court chemin")

node_labels = {n["id"]: n.get("label", n["id"]) for n in all_nodes}
node_ids = list(node_labels.keys())

if len(node_ids) >= 2:
    path_col1, path_col2 = st.columns(2)
    with path_col1:
        from_id = st.selectbox(
            "Noeud de depart",
            options=node_ids,
            format_func=lambda nid: node_labels.get(nid, nid),
            key="sp_from",
        )
    with path_col2:
        to_id = st.selectbox(
            "Noeud d'arrivee",
            options=node_ids,
            format_func=lambda nid: node_labels.get(nid, nid),
            key="sp_to",
        )

    if st.button("Calculer le plus court chemin", type="primary"):
        if from_id == to_id:
            st.warning("Selectionnez deux noeuds differents.")
        else:
            with st.spinner("Calcul du chemin..."):
                path_result = api.get_shortest_path(case_id, from_id, to_id)
            if path_result:
                path_nodes = path_result.get("nodes", [])
                path_edges = path_result.get("edges", [])
                if path_nodes:
                    st.success(
                        f"Chemin trouve : {len(path_nodes)} noeuds, "
                        f"{len(path_edges)} relations."
                    )
                    # Display path as a chain
                    chain_parts = []
                    for pn in path_nodes:
                        chain_parts.append(
                            f"**{pn.get('label', pn['id'])}** ({pn.get('type', '?')})"
                        )
                    st.markdown(" -> ".join(chain_parts))

                    # Render the path sub-graph
                    render_graph(
                        {"nodes": path_nodes, "edges": path_edges},
                        height=350,
                        physics=True,
                    )
                else:
                    st.warning("Aucun chemin trouve entre ces deux noeuds.")
            else:
                st.error("Erreur lors du calcul du chemin.")
else:
    st.info("Il faut au moins 2 noeuds dans le graphe pour calculer un chemin.")

# ------------------------------------------------------------------
# Clusters
# ------------------------------------------------------------------

st.markdown("---")
st.subheader("Clusters (communautes)")

with st.expander("Afficher les clusters detectes", expanded=False):
    clusters = api.get_clusters(case_id)
    if clusters:
        cluster_list = clusters.get("clusters", [])
        if cluster_list:
            for i, cluster in enumerate(cluster_list):
                members = cluster.get("members", cluster.get("nodes", []))
                st.markdown(f"**Cluster {i + 1}** ({len(members)} membres)")
                member_labels = []
                for m in members:
                    if isinstance(m, dict):
                        member_labels.append(m.get("label", m.get("id", "?")))
                    else:
                        member_labels.append(node_labels.get(m, str(m)))
                st.markdown(", ".join(member_labels))
        else:
            st.info("Aucun cluster detecte.")
    else:
        st.info(
            "Impossible de charger les clusters. "
            "Lancez une analyse pour detecter les communautes."
        )
