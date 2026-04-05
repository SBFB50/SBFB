"""
NEXUS -- Graph API router (Neo4j).

Exposes the investigation knowledge graph for a given case:
- Full graph retrieval (nodes + edges)
- Neighborhood exploration around a node
- Shortest path between two nodes
- Cluster detection (connected components)
- Node statistics by label
- Central entities (degree centrality)
- Entity importance (betweenness / bridge detection)
- Community detection
- Indirect connection discovery
- Temporal graph view
- Evidence-entity matrix
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


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/graph/central-entities
# ------------------------------------------------------------------

@router.get("/cases/{case_id}/graph/central-entities")
async def get_central_entities(
    case_id: str,
    limit: int = Query(default=10, ge=1, le=50),
    neo4j: Neo4jClient = Depends(get_neo4j),
) -> Dict[str, Any]:
    """Return the most connected entities using degree centrality.

    Useful for identifying key suspects, locations, or evidence nodes
    that are at the center of the investigation graph.
    """
    entities = await neo4j.get_central_entities(case_id, limit=limit)
    return {
        "entities": entities,
        "count": len(entities),
    }


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/graph/importance
# ------------------------------------------------------------------

@router.get("/cases/{case_id}/graph/importance")
async def get_entity_importance(
    case_id: str,
    limit: int = Query(default=20, ge=1, le=50),
    neo4j: Neo4jClient = Depends(get_neo4j),
) -> Dict[str, Any]:
    """Return entities ranked by betweenness / bridge importance.

    Entities that connect otherwise separate groups are likely
    important to the investigation (intermediaries, shared locations, etc.).
    """
    entities = await neo4j.get_entity_importance(case_id, limit=limit)
    return {
        "entities": entities,
        "count": len(entities),
    }


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/graph/communities
# ------------------------------------------------------------------

@router.get("/cases/{case_id}/graph/communities")
async def detect_communities(
    case_id: str,
    neo4j: Neo4jClient = Depends(get_neo4j),
) -> Dict[str, Any]:
    """Detect communities / clusters of related entities.

    Returns groups of entities that are tightly connected.
    Each community is a list of enriched node dicts (id, name, labels).
    Communities are sorted by size (largest first).
    """
    communities = await neo4j.detect_communities(case_id)
    return {
        "communities": communities,
        "count": len(communities),
        "sizes": [len(c) for c in communities],
    }


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/graph/connections/{id1}/{id2}
# ------------------------------------------------------------------

@router.get("/cases/{case_id}/graph/connections/{id1}/{id2}")
async def find_indirect_connections(
    case_id: str,
    id1: str,
    id2: str,
    max_hops: int = Query(default=4, ge=1, le=6),
    neo4j: Neo4jClient = Depends(get_neo4j),
) -> Dict[str, Any]:
    """Find all paths between two entities up to max_hops.

    Useful for discovering hidden connections between suspects
    or understanding how two entities are indirectly related.
    """
    paths = await neo4j.find_indirect_connections(id1, id2, max_hops=max_hops)
    if not paths:
        raise HTTPException(
            status_code=404,
            detail=f"No paths found between '{id1}' and '{id2}' within {max_hops} hops",
        )
    return {
        "paths": paths,
        "count": len(paths),
        "shortest": paths[0]["length"] if paths else None,
    }


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/graph/temporal
# ------------------------------------------------------------------

@router.get("/cases/{case_id}/graph/temporal")
async def get_temporal_graph(
    case_id: str,
    neo4j: Neo4jClient = Depends(get_neo4j),
) -> Dict[str, Any]:
    """Return a temporal view of the knowledge graph.

    Events and time-stamped entities sorted chronologically,
    useful for building investigation timelines.
    """
    events = await neo4j.get_temporal_graph(case_id)
    return {
        "events": events,
        "count": len(events),
    }


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/graph/evidence-matrix
# ------------------------------------------------------------------

@router.get("/cases/{case_id}/graph/evidence-matrix")
async def get_evidence_matrix(
    case_id: str,
    neo4j: Neo4jClient = Depends(get_neo4j),
) -> Dict[str, Any]:
    """Build a matrix of which entities appear in which evidence.

    Returns the full matrix plus co-occurrence patterns showing
    entities that frequently appear together or never together.
    Useful for pattern detection in investigations.
    """
    return await neo4j.get_evidence_entity_matrix(case_id)
