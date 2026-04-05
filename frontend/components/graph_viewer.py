"""
NEXUS -- Reusable graph viewer component.

Converts API graph data (nodes + edges) into a streamlit-agraph
visualisation with type-based coloring and physics simulation.
"""

from __future__ import annotations

from typing import Any, Dict, List, Optional

import streamlit as st
from streamlit_agraph import agraph, Node, Edge, Config

# ---------------------------------------------------------------
# Colour palette per entity type
# ---------------------------------------------------------------

TYPE_COLORS: Dict[str, str] = {
    "person": "#4A90D9",       # blue
    "location": "#27AE60",     # green
    "phone": "#E67E22",        # orange
    "vehicle": "#E74C3C",      # red
    "organization": "#8E44AD", # purple
    "date": "#F1C40F",         # yellow
    "money": "#1ABC9C",        # teal
    "ip": "#95A5A6",           # grey
    "email": "#2980B9",        # dark blue
    "account": "#D35400",      # burnt orange
    "weapon": "#C0392B",       # dark red
    "drug": "#7D3C98",         # dark purple
    "other": "#BDC3C7",        # light grey
}

TYPE_SIZES: Dict[str, int] = {
    "person": 30,
    "location": 25,
    "phone": 20,
    "vehicle": 22,
    "organization": 28,
    "date": 18,
    "money": 20,
    "ip": 18,
    "email": 18,
    "account": 20,
    "weapon": 22,
    "drug": 20,
    "other": 16,
}

EDGE_COLORS: Dict[str, str] = {
    "KNOWS": "#4A90D9",
    "LIVES_AT": "#27AE60",
    "OWNS": "#E67E22",
    "CALLED": "#E74C3C",
    "RELATED_TO": "#95A5A6",
    "WITNESSED": "#8E44AD",
    "EMPLOYED_BY": "#1ABC9C",
    "LOCATED_AT": "#27AE60",
}


def render_graph(
    graph_data: Dict[str, Any],
    height: int = 600,
    physics: bool = True,
    node_filter: Optional[List[str]] = None,
) -> Optional[str]:
    """Convert API graph data into an interactive agraph visualisation.

    Parameters
    ----------
    graph_data : dict
        ``{nodes: [{id, label, type, properties}], edges: [{from, to, type, properties}]}``
    height : int
        Canvas height in pixels.
    physics : bool
        Enable physics simulation (spring layout).
    node_filter : list[str] | None
        If provided, only show nodes whose ``type`` is in this list.

    Returns
    -------
    str | None
        The id of the selected node (if any click occurred), else None.
    """
    raw_nodes = graph_data.get("nodes", [])
    raw_edges = graph_data.get("edges", [])

    # Build a connection count per node for sizing
    connection_counts: Dict[str, int] = {}
    for edge in raw_edges:
        src = edge.get("from") or edge.get("source", "")
        tgt = edge.get("to") or edge.get("target", "")
        connection_counts[src] = connection_counts.get(src, 0) + 1
        connection_counts[tgt] = connection_counts.get(tgt, 0) + 1

    # Filter nodes by type if requested
    if node_filter:
        allowed_types = {t.lower() for t in node_filter}
        raw_nodes = [n for n in raw_nodes if n.get("type", "other").lower() in allowed_types]
        visible_ids = {n["id"] for n in raw_nodes}
        raw_edges = [
            e for e in raw_edges
            if (e.get("from") or e.get("source", "")) in visible_ids
            and (e.get("to") or e.get("target", "")) in visible_ids
        ]

    # Build agraph nodes
    nodes = []
    for n in raw_nodes:
        ntype = n.get("type", "other").lower()
        color = TYPE_COLORS.get(ntype, TYPE_COLORS["other"])
        base_size = TYPE_SIZES.get(ntype, 16)
        conn = connection_counts.get(n["id"], 0)
        # Scale size proportionally to connections (min=base, max=base*3)
        size = base_size + min(conn * 3, base_size * 2)

        nodes.append(Node(
            id=n["id"],
            label=n.get("label", n["id"]),
            size=size,
            color=color,
            shape="dot",
            title=_build_tooltip(n),
        ))

    # Build agraph edges
    edges = []
    for e in raw_edges:
        etype = e.get("type", "RELATED_TO")
        edge_color = EDGE_COLORS.get(etype, "#95A5A6")
        edges.append(Edge(
            source=e.get("from") or e.get("source", ""),
            target=e.get("to") or e.get("target", ""),
            label=etype,
            color=edge_color,
            width=1.5,
        ))

    # Configuration
    config = Config(
        width="100%",
        height=height,
        directed=True,
        physics=physics,
        hierarchical=False,
        nodeHighlightBehavior=True,
        highlightColor="#F1C40F",
        collapsible=False,
        node={"labelProperty": "label"},
        link={"labelProperty": "label", "renderLabel": True},
    )

    # Render and return selected node id
    selected = agraph(nodes=nodes, edges=edges, config=config)
    return selected


def render_node_details(node_data: Dict[str, Any]) -> None:
    """Display detailed information about a selected node.

    Parameters
    ----------
    node_data : dict
        A single node dict with ``{id, label, type, properties}``.
    """
    ntype = node_data.get("type", "other").lower()
    color = TYPE_COLORS.get(ntype, TYPE_COLORS["other"])

    st.markdown(
        f"<span style='background-color:{color};color:white;padding:2px 10px;"
        f"border-radius:12px;font-size:0.85em'>{ntype.upper()}</span>",
        unsafe_allow_html=True,
    )
    st.markdown(f"### {node_data.get('label', node_data['id'])}")

    props = node_data.get("properties", {})
    if props:
        for key, value in props.items():
            st.markdown(f"**{key}:** {value}")
    else:
        st.caption("Aucune propriete supplementaire.")


def _build_tooltip(node: Dict[str, Any]) -> str:
    """Build an HTML tooltip string for a node."""
    ntype = node.get("type", "other")
    label = node.get("label", node["id"])
    props = node.get("properties", {})

    lines = [f"<b>{label}</b>", f"Type: {ntype}"]
    for k, v in list(props.items())[:5]:
        lines.append(f"{k}: {v}")
    return "<br>".join(lines)
