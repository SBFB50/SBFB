"""
NEXUS -- Graph API router (Neo4j).

Exposes the investigation knowledge graph for a given case:
- Full graph retrieval (nodes + edges)
- Neighborhood exploration around a node
- Shortest path between two nodes
- Cluster detection (connected components)
- Node statistics by label
"""

from __future__ import annotations

from typing import Any, Dict, List

from fastapi import APIRouter, Depends, HTTPException, Query

from nexus.api.deps import get_neo4j
from nexus.db.neo4j_db import Neo4jClient

router = APIRouter(prefix="/api", tags=["graph"])


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/graph
# ------------------------------------------------------------------

@router.get("/cases/{case_id}/graph")
async def get_full_graph(
    case_id: str,
    neo4j: Neo4jClient = Depends(get_neo4j),
) -> Dict[str, Any]:
    """Return the entire graph for a case (nodes + edges).

    Ready for front-end visualisation (e.g. streamlit-agraph).
    """
    graph = await neo4j.get_full_graph(case_id)
    return graph


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/graph/neighbors/{node_id}
# ------------------------------------------------------------------

@router.get("/cases/{case_id}/graph/neighbors/{node_id}")
async def get_neighbors(
    case_id: str,
    node_id: str,
    depth: int = Query(default=1, ge=1, le=5),
    neo4j: Neo4jClient = Depends(get_neo4j),
) -> Dict[str, Any]:
    """Return the sub-graph around a node up to *depth* hops."""
    subgraph = await neo4j.get_neighbors(node_id, depth=depth)
    if not subgraph["nodes"]:
        raise HTTPException(
            status_code=404,
            detail=f"Node '{node_id}' not found in case '{case_id}'",
        )
    return subgraph


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/graph/path/{from_id}/{to_id}
# ------------------------------------------------------------------

@router.get("/cases/{case_id}/graph/path/{from_id}/{to_id}")
async def find_shortest_path(
    case_id: str,
    from_id: str,
    to_id: str,
    neo4j: Neo4jClient = Depends(get_neo4j),
) -> Dict[str, Any]:
    """Find the shortest path between two nodes.

    Returns the ordered list of nodes along the path, or 404 if no
    path exists.
    """
    path = await neo4j.find_shortest_path(from_id, to_id)
    if not path:
        raise HTTPException(
            status_code=404,
            detail=f"No path found between '{from_id}' and '{to_id}'",
        )
    return {"path": path, "length": len(path) - 1}


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/graph/clusters
# ------------------------------------------------------------------

@router.get("/cases/{case_id}/graph/clusters")
async def find_clusters(
    case_id: str,
    neo4j: Neo4jClient = Depends(get_neo4j),
) -> Dict[str, Any]:
    """Detect connected components within a case's sub-graph.

    Returns a list of clusters, each being a list of node ids.
    """
    clusters = await neo4j.find_clusters(case_id)
    return {
        "clusters": clusters,
        "count": len(clusters),
    }


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/graph/stats
# ------------------------------------------------------------------

@router.get("/cases/{case_id}/graph/stats")
async def get_node_stats(
    case_id: str,
    neo4j: Neo4jClient = Depends(get_neo4j),
) -> Dict[str, int]:
    """Return the count of nodes per label for a given case."""
    return await neo4j.get_node_stats(case_id)
