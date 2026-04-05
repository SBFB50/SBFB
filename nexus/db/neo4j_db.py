"""
NEXUS -- Async Neo4j graph database client.

Provides the full graph layer for the investigation system:
- Node CRUD (Person, Location, Phone, Vehicle, Organization, etc.)
- Relation CRUD with typed relationships
- Graph queries (neighbors, shortest path, clusters, stats)
- Synchronization bridge from SQLite entities to Neo4j nodes
- Schema constraints initialization

All Cypher queries use parameterized $variables -- never f-strings.
"""

from __future__ import annotations

import uuid
from collections import Counter
from datetime import datetime
from typing import Any, Dict, List, Optional

from loguru import logger
from neo4j import AsyncGraphDatabase, AsyncManagedTransaction

from nexus.config import settings

# ---------------------------------------------------------------------------
# Valid node labels and relationship types for this project
# ---------------------------------------------------------------------------

NODE_LABELS = frozenset({
    "Person",
    "Location",
    "Phone",
    "Vehicle",
    "Organization",
    "Account",
    "Event",
    "Evidence",
    "Money",
    "Hypothesis",
    "Case",
})

RELATIONSHIP_TYPES = frozenset({
    # Person <-> Person
    "KNOWS",
    "RELATED_TO",
    "COMMUNICATED_WITH",
    "FINANCIAL_LINK",
    "SENT_MONEY",
    "RECEIVED_MONEY",
    # Person <-> Location
    "LIVES_AT",
    "WAS_AT",
    "WORKS_AT",
    "FREQUENTS",
    # Person <-> Vehicle/Phone/Account
    "OWNS",
    # Person <-> Organization
    "MEMBER_OF",
    # Event <-> *
    "OCCURRED_AT",
    "INVOLVES",
    "PRECEDED_BY",
    # Evidence <-> *
    "MENTIONS",
    "SUPPORTS",
    "CONTRADICTS",
    # Account <-> Account
    "TRANSACTION",
    # Generic
    "BELONGS_TO",
})

# Map entity_type from SQLite to Neo4j label
_ENTITY_TYPE_TO_LABEL: Dict[str, str] = {
    "person": "Person",
    "location": "Location",
    "phone": "Phone",
    "vehicle": "Vehicle",
    "organization": "Organization",
    "account": "Account",
    "date": "Event",
    "money": "Money",
    "ip": "Account",
    "email": "Account",
    "weapon": "Evidence",
    "drug": "Evidence",
    "other": "Evidence",
}


def _new_id() -> str:
    """Generate a new UUID4 string."""
    return str(uuid.uuid4())


def _now_iso() -> str:
    """Current UTC timestamp in ISO-8601."""
    return datetime.utcnow().isoformat()


# ============================================================================
# Neo4jClient
# ============================================================================


class Neo4jClient:
    """Async Neo4j client for the NEXUS investigation graph.

    Usage::

        client = Neo4jClient()
        ok = await client.verify_connectivity()
        node_id = await client.create_or_update_node("Person", {"id": "abc", "name": "Doe"})
        await client.close()
    """

    def __init__(self) -> None:
        self._driver = AsyncGraphDatabase.driver(
            settings.neo4j_uri,
            auth=(settings.neo4j_user, settings.neo4j_password),
        )
        logger.info(
            "Neo4j async driver created for {}",
            settings.neo4j_uri,
        )

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    async def close(self) -> None:
        """Shut down the driver and release all connections."""
        await self._driver.close()
        logger.info("Neo4j driver closed")

    async def verify_connectivity(self) -> bool:
        """Return True if the database is reachable."""
        try:
            await self._driver.verify_connectivity()
            logger.info("Neo4j connectivity OK")
            return True
        except Exception as exc:
            logger.error("Neo4j connectivity check failed: {}", exc)
            return False

    # ------------------------------------------------------------------
    # Schema / constraints
    # ------------------------------------------------------------------

    async def init_constraints(self) -> None:
        """Create uniqueness constraints on `id` for every node label.

        Safe to call multiple times -- uses IF NOT EXISTS.
        """
        async with self._driver.session() as session:
            for label in NODE_LABELS:
                query = (
                    f"CREATE CONSTRAINT IF NOT EXISTS "
                    f"FOR (n:{label}) REQUIRE n.id IS UNIQUE"
                )
                await session.run(query)
                logger.debug("Constraint ensured for :{}", label)
        logger.info("All Neo4j uniqueness constraints initialized")

    # ------------------------------------------------------------------
    # Node CRUD
    # ------------------------------------------------------------------

    async def create_or_update_node(
        self,
        label: str,
        properties: Dict[str, Any],
    ) -> str:
        """MERGE a node by its ``id`` property, setting all other properties.

        If *properties* does not contain an ``id`` key, one is generated.
        Returns the node id.
        """
        if "id" not in properties:
            properties["id"] = _new_id()
        node_id = properties["id"]

        # Separate id from the rest so MERGE only matches on id
        props = {k: v for k, v in properties.items() if k != "id"}
        props["updated_at"] = _now_iso()

        async with self._driver.session() as session:

            async def _work(tx: AsyncManagedTransaction) -> str:
                query = (
                    f"MERGE (n:{label} {{id: $node_id}}) "
                    "ON CREATE SET n += $props, n.created_at = $now "
                    "ON MATCH SET n += $props "
                    "RETURN n.id AS node_id"
                )
                result = await tx.run(
                    query,
                    node_id=node_id,
                    props=props,
                    now=_now_iso(),
                )
                record = await result.single()
                return record["node_id"]  # type: ignore[index]

            created_id = await session.execute_write(_work)

        logger.debug("MERGE :{} id={}", label, created_id)
        return created_id

    async def get_node(self, node_id: str) -> Optional[Dict[str, Any]]:
        """Fetch a single node by id. Returns its properties or None."""
        async with self._driver.session() as session:

            async def _work(tx: AsyncManagedTransaction) -> Optional[Dict[str, Any]]:
                result = await tx.run(
                    "MATCH (n {id: $node_id}) "
                    "RETURN n, labels(n) AS labels",
                    node_id=node_id,
                )
                record = await result.single()
                if record is None:
                    return None
                node_data = dict(record["n"])
                node_data["_labels"] = list(record["labels"])
                return node_data

            return await session.execute_read(_work)

    async def delete_node(self, node_id: str) -> None:
        """Delete a node and all its relationships."""
        async with self._driver.session() as session:

            async def _work(tx: AsyncManagedTransaction) -> None:
                await tx.run(
                    "MATCH (n {id: $node_id}) DETACH DELETE n",
                    node_id=node_id,
                )

            await session.execute_write(_work)
        logger.debug("Deleted node id={}", node_id)

    async def find_nodes_by_case(
        self,
        case_id: str,
        label: Optional[str] = None,
    ) -> List[Dict[str, Any]]:
        """Find all nodes belonging to a case, optionally filtered by label."""
        async with self._driver.session() as session:

            async def _work(tx: AsyncManagedTransaction) -> List[Dict[str, Any]]:
                if label:
                    query = (
                        f"MATCH (n:{label} {{case_id: $case_id}}) "
                        "RETURN n, labels(n) AS labels ORDER BY n.name"
                    )
                else:
                    query = (
                        "MATCH (n {case_id: $case_id}) "
                        "RETURN n, labels(n) AS labels ORDER BY n.name"
                    )
                result = await tx.run(query, case_id=case_id)
                records = [r async for r in result]
                nodes = []
                for record in records:
                    node_data = dict(record["n"])
                    node_data["_labels"] = list(record["labels"])
                    nodes.append(node_data)
                return nodes

            return await session.execute_read(_work)

    # ------------------------------------------------------------------
    # Relation CRUD
    # ------------------------------------------------------------------

    async def create_relation(
        self,
        from_id: str,
        to_id: str,
        rel_type: str,
        properties: Optional[Dict[str, Any]] = None,
    ) -> None:
        """Create a relationship between two nodes (by id).

        Uses MERGE so calling twice with the same triple is idempotent.
        """
        props = properties or {}
        props["updated_at"] = _now_iso()

        async with self._driver.session() as session:

            async def _work(tx: AsyncManagedTransaction) -> None:
                query = (
                    "MATCH (a {id: $from_id}), (b {id: $to_id}) "
                    f"MERGE (a)-[r:{rel_type}]->(b) "
                    "ON CREATE SET r += $props, r.created_at = $now "
                    "ON MATCH SET r += $props"
                )
                await tx.run(
                    query,
                    from_id=from_id,
                    to_id=to_id,
                    props=props,
                    now=_now_iso(),
                )

            await session.execute_write(_work)
        logger.debug("MERGE relation {} -> {} [{}]", from_id, to_id, rel_type)

    async def get_relations(
        self,
        node_id: str,
        direction: str = "both",
    ) -> List[Dict[str, Any]]:
        """Return all relationships for a node.

        ``direction``: ``"out"``, ``"in"``, or ``"both"`` (default).
        Each item: ``{from, to, type, properties}``.
        """
        async with self._driver.session() as session:

            async def _work(tx: AsyncManagedTransaction) -> List[Dict[str, Any]]:
                if direction == "out":
                    query = (
                        "MATCH (a {id: $node_id})-[r]->(b) "
                        "RETURN a.id AS from_id, b.id AS to_id, "
                        "type(r) AS rel_type, properties(r) AS props"
                    )
                elif direction == "in":
                    query = (
                        "MATCH (a)-[r]->(b {id: $node_id}) "
                        "RETURN a.id AS from_id, b.id AS to_id, "
                        "type(r) AS rel_type, properties(r) AS props"
                    )
                else:
                    query = (
                        "MATCH (a)-[r]-(b) "
                        "WHERE a.id = $node_id "
                        "RETURN startNode(r).id AS from_id, "
                        "endNode(r).id AS to_id, "
                        "type(r) AS rel_type, properties(r) AS props"
                    )
                result = await tx.run(query, node_id=node_id)
                records = [r async for r in result]
                return [
                    {
                        "from": rec["from_id"],
                        "to": rec["to_id"],
                        "type": rec["rel_type"],
                        "properties": dict(rec["props"]) if rec["props"] else {},
                    }
                    for rec in records
                ]

            return await session.execute_read(_work)

    async def delete_relation(
        self,
        from_id: str,
        to_id: str,
        rel_type: str,
    ) -> None:
        """Delete a specific relationship between two nodes."""
        async with self._driver.session() as session:

            async def _work(tx: AsyncManagedTransaction) -> None:
                query = (
                    "MATCH (a {id: $from_id})-[r:" + rel_type + "]->(b {id: $to_id}) "
                    "DELETE r"
                )
                await tx.run(query, from_id=from_id, to_id=to_id)

            await session.execute_write(_work)
        logger.debug("Deleted relation {} -[{}]-> {}", from_id, rel_type, to_id)

    # ------------------------------------------------------------------
    # Graph queries
    # ------------------------------------------------------------------

    async def get_full_graph(self, case_id: str) -> Dict[str, Any]:
        """Return the entire graph for a case.

        Returns ``{nodes: [...], edges: [...]}``, ready for front-end
        visualization (e.g. streamlit-agraph).
        """
        async with self._driver.session() as session:

            async def _work(tx: AsyncManagedTransaction) -> Dict[str, Any]:
                # Nodes
                node_result = await tx.run(
                    "MATCH (n {case_id: $case_id}) "
                    "RETURN n, labels(n) AS labels",
                    case_id=case_id,
                )
                node_records = [r async for r in node_result]
                nodes = []
                node_ids = set()
                for rec in node_records:
                    nd = dict(rec["n"])
                    nd["_labels"] = list(rec["labels"])
                    nodes.append(nd)
                    node_ids.add(nd.get("id"))

                # Edges -- only between nodes in this case
                edge_result = await tx.run(
                    "MATCH (a {case_id: $case_id})-[r]->(b {case_id: $case_id}) "
                    "RETURN a.id AS from_id, b.id AS to_id, "
                    "type(r) AS rel_type, properties(r) AS props",
                    case_id=case_id,
                )
                edge_records = [r async for r in edge_result]
                edges = [
                    {
                        "from": rec["from_id"],
                        "to": rec["to_id"],
                        "type": rec["rel_type"],
                        "properties": dict(rec["props"]) if rec["props"] else {},
                    }
                    for rec in edge_records
                ]

                return {"nodes": nodes, "edges": edges}

            return await session.execute_read(_work)

    async def get_neighbors(
        self,
        node_id: str,
        depth: int = 1,
        max_nodes: int = 200,
    ) -> Dict[str, Any]:
        """Return a sub-graph around a node up to *depth* hops.

        Returns ``{nodes: [...], edges: [...]}``.
        ``max_nodes`` caps the number of paths explored to avoid
        combinatorial explosion on highly-connected graphs.
        """
        async with self._driver.session() as session:

            async def _work(tx: AsyncManagedTransaction) -> Dict[str, Any]:
                # Collect nodes reachable within `depth` hops
                # LIMIT on paths prevents Cartesian explosion
                query = (
                    "MATCH path = (start {id: $node_id})-[*1.." + str(int(depth)) + "]-(end) "
                    "WITH path LIMIT " + str(int(max_nodes)) + " "
                    "WITH nodes(path) AS ns, relationships(path) AS rs "
                    "UNWIND ns AS n "
                    "WITH collect(DISTINCT n) AS all_nodes, "
                    "collect(DISTINCT rs) AS all_rels_nested "
                    "UNWIND all_rels_nested AS rels "
                    "UNWIND rels AS r "
                    "WITH all_nodes, collect(DISTINCT r) AS all_rels "
                    "RETURN all_nodes, all_rels"
                )
                result = await tx.run(query, node_id=node_id)
                record = await result.single()

                if record is None:
                    # Return just the start node if it exists
                    solo = await tx.run(
                        "MATCH (n {id: $node_id}) "
                        "RETURN n, labels(n) AS labels",
                        node_id=node_id,
                    )
                    solo_rec = await solo.single()
                    if solo_rec is None:
                        return {"nodes": [], "edges": []}
                    nd = dict(solo_rec["n"])
                    nd["_labels"] = list(solo_rec["labels"])
                    return {"nodes": [nd], "edges": []}

                nodes = []
                seen_ids: set[str] = set()
                for n in record["all_nodes"]:
                    nd = dict(n)
                    nd["_labels"] = list(n.labels)
                    nid = nd.get("id")
                    if nid and nid not in seen_ids:
                        nodes.append(nd)
                        seen_ids.add(nid)

                edges = []
                for r in record["all_rels"]:
                    edges.append({
                        "from": dict(r.start_node).get("id"),
                        "to": dict(r.end_node).get("id"),
                        "type": r.type,
                        "properties": dict(r),
                    })

                return {"nodes": nodes, "edges": edges}

            return await session.execute_read(_work)

    async def find_shortest_path(
        self,
        from_id: str,
        to_id: str,
    ) -> List[Dict[str, Any]]:
        """Find the shortest path between two nodes.

        Returns a list of node dicts along the path (in order),
        or an empty list if no path exists.
        Uses a max depth of 10 to avoid unbounded expansion.
        """
        async with self._driver.session() as session:

            async def _work(tx: AsyncManagedTransaction) -> List[Dict[str, Any]]:
                result = await tx.run(
                    "MATCH p = shortestPath("
                    "(a {id: $from_id})-[*..10]-(b {id: $to_id})"
                    ") "
                    "RETURN nodes(p) AS path_nodes, "
                    "relationships(p) AS path_rels",
                    from_id=from_id,
                    to_id=to_id,
                )
                record = await result.single()
                if record is None:
                    return []

                path: List[Dict[str, Any]] = []
                for n in record["path_nodes"]:
                    nd = dict(n)
                    nd["_labels"] = list(n.labels)
                    path.append(nd)
                return path

            return await session.execute_read(_work)

    async def find_clusters(
        self,
        case_id: str,
    ) -> List[List[str]]:
        """Detect connected components within a case's sub-graph.

        Returns a list of clusters, each cluster being a list of node ids.
        Uses a BFS-style approach via Cypher path expansion.
        """
        async with self._driver.session() as session:

            async def _work(tx: AsyncManagedTransaction) -> List[List[str]]:
                # Get all node ids for the case
                result = await tx.run(
                    "MATCH (n {case_id: $case_id}) RETURN n.id AS nid",
                    case_id=case_id,
                )
                records = [r async for r in result]
                all_ids = {rec["nid"] for rec in records}

                if not all_ids:
                    return []

                # For each unvisited node, find its connected component
                visited: set[str] = set()
                clusters: List[List[str]] = []

                for nid in all_ids:
                    if nid in visited:
                        continue
                    # Find all nodes reachable from nid within this case
                    comp_result = await tx.run(
                        "MATCH (start {id: $nid}) "
                        "OPTIONAL MATCH (start)-[*]-(connected) "
                        "WHERE connected.case_id = $case_id "
                        "WITH collect(DISTINCT connected.id) + [start.id] AS ids "
                        "RETURN ids",
                        nid=nid,
                        case_id=case_id,
                    )
                    comp_record = await comp_result.single()
                    if comp_record is None:
                        continue
                    component_ids = [
                        i for i in comp_record["ids"] if i is not None
                    ]
                    # Intersect with case nodes to filter stray connections
                    component = [i for i in component_ids if i in all_ids]
                    visited.update(component)
                    if component:
                        clusters.append(sorted(component))

                return clusters

            return await session.execute_read(_work)

    async def get_node_stats(self, case_id: str) -> Dict[str, int]:
        """Return the count of nodes per label for a given case.

        Returns e.g. ``{"Person": 5, "Location": 3, ...}``.
        """
        async with self._driver.session() as session:

            async def _work(tx: AsyncManagedTransaction) -> Dict[str, int]:
                result = await tx.run(
                    "MATCH (n {case_id: $case_id}) "
                    "RETURN labels(n) AS labels, count(n) AS cnt",
                    case_id=case_id,
                )
                records = [r async for r in result]
                stats: Dict[str, int] = {}
                for rec in records:
                    for lbl in rec["labels"]:
                        stats[lbl] = stats.get(lbl, 0) + rec["cnt"]
                return stats

            return await session.execute_read(_work)

    # ------------------------------------------------------------------
    # Synchronization: SQLite entities -> Neo4j
    # ------------------------------------------------------------------

    async def sync_entity(
        self,
        entity: Dict[str, Any],
        case_id: str,
    ) -> str:
        """Create or update a Neo4j node from an SQLite entity dict.

        Maps ``entity_type`` to the appropriate Neo4j label and carries
        over name, aliases, description, and metadata.

        Returns the node id.
        """
        entity_type = entity.get("entity_type", "other")
        label = _ENTITY_TYPE_TO_LABEL.get(entity_type, "Evidence")

        properties: Dict[str, Any] = {
            "id": entity["id"],
            "case_id": case_id,
            "name": entity.get("name", ""),
            "entity_type": entity_type,
        }
        # Optional fields
        if entity.get("aliases"):
            # Store as comma-separated string for Neo4j compatibility
            aliases = entity["aliases"]
            if isinstance(aliases, list):
                properties["aliases"] = ", ".join(aliases)
            else:
                properties["aliases"] = str(aliases)
        if entity.get("description"):
            properties["description"] = entity["description"]
        if entity.get("first_seen"):
            properties["first_seen"] = str(entity["first_seen"])
        if entity.get("metadata"):
            # Flatten simple metadata or store as string
            meta = entity["metadata"]
            if isinstance(meta, dict):
                for k, v in meta.items():
                    properties[f"meta_{k}"] = str(v) if not isinstance(v, (int, float, bool)) else v
            else:
                properties["metadata_raw"] = str(meta)

        node_id = await self.create_or_update_node(label, properties)

        # Also link the entity to the Case node
        await self.create_relation(node_id, case_id, "BELONGS_TO")

        logger.debug("Synced entity {} as :{} id={}", entity.get("name"), label, node_id)
        return node_id

    async def sync_relations(
        self,
        relations: List[Dict[str, Any]],
        case_id: str,
    ) -> None:
        """Create relationships from a list of relation dicts.

        Each dict must have: ``from_id``, ``to_id``, ``rel_type``.
        Optional: ``properties`` dict.
        """
        for rel in relations:
            from_id = rel.get("from_id") or rel.get("from")
            to_id = rel.get("to_id") or rel.get("to")
            rel_type = rel.get("rel_type") or rel.get("type")

            if not from_id or not to_id or not rel_type:
                logger.warning(
                    "Skipping invalid relation (missing fields): {}", rel
                )
                continue

            props = rel.get("properties", {})
            props["case_id"] = case_id

            await self.create_relation(from_id, to_id, rel_type, props)

        logger.info(
            "Synced {} relations for case {}", len(relations), case_id
        )

    async def sync_evidence(
        self,
        evidence_id: str,
        case_id: str,
        title: str,
        evidence_type: str,
        reliability: int,
    ) -> None:
        """Create or update an Evidence node in Neo4j."""
        properties = {
            "id": evidence_id,
            "case_id": case_id,
            "title": title,
            "evidence_type": evidence_type,
            "reliability": reliability,
        }
        await self.create_or_update_node("Evidence", properties)

        # Link evidence to the case
        await self.create_relation(evidence_id, case_id, "BELONGS_TO")

        logger.debug("Synced evidence id={} title={}", evidence_id, title)

    async def link_evidence_to_entity(
        self,
        evidence_id: str,
        entity_id: str,
    ) -> None:
        """Create a MENTIONS relationship from an Evidence node to an entity."""
        await self.create_relation(evidence_id, entity_id, "MENTIONS")
        logger.debug(
            "Linked evidence {} -> MENTIONS -> entity {}",
            evidence_id,
            entity_id,
        )

    # ------------------------------------------------------------------
    # Advanced graph analytics
    # ------------------------------------------------------------------

    async def get_central_entities(
        self,
        case_id: str,
        limit: int = 10,
    ) -> List[Dict[str, Any]]:
        """Find the most connected entities using degree centrality.

        Returns entities sorted by number of connections (highest first).
        Each result contains: id, name, labels, degree, in_degree, out_degree.
        """
        async with self._driver.session() as session:

            async def _work(tx: AsyncManagedTransaction) -> List[Dict[str, Any]]:
                query = (
                    "MATCH (n {case_id: $case_id}) "
                    "OPTIONAL MATCH (n)-[r_out]->() "
                    "OPTIONAL MATCH (n)<-[r_in]-() "
                    "WITH n, labels(n) AS labels, "
                    "  count(DISTINCT r_out) AS out_degree, "
                    "  count(DISTINCT r_in) AS in_degree "
                    "WITH n, labels, out_degree, in_degree, "
                    "  (out_degree + in_degree) AS degree "
                    "WHERE degree > 0 "
                    "RETURN n.id AS id, n.name AS name, labels, "
                    "  degree, in_degree, out_degree "
                    "ORDER BY degree DESC "
                    "LIMIT $limit"
                )
                result = await tx.run(query, case_id=case_id, limit=limit)
                records = [r async for r in result]
                return [
                    {
                        "id": rec["id"],
                        "name": rec["name"],
                        "labels": list(rec["labels"]),
                        "degree": rec["degree"],
                        "in_degree": rec["in_degree"],
                        "out_degree": rec["out_degree"],
                    }
                    for rec in records
                ]

            return await session.execute_read(_work)

    async def get_entity_importance(
        self,
        case_id: str,
        limit: int = 20,
    ) -> List[Dict[str, Any]]:
        """Calculate betweenness-style importance to find bridge entities.

        Entities that connect otherwise separate groups are likely
        important to the investigation.

        Uses a Cypher-based approximation: for each node, counts how many
        shortest paths between other node-pairs pass through it.
        Falls back to degree centrality for large graphs (>200 nodes).
        """
        async with self._driver.session() as session:

            async def _work(tx: AsyncManagedTransaction) -> List[Dict[str, Any]]:
                # First, count nodes to decide strategy
                count_result = await tx.run(
                    "MATCH (n {case_id: $case_id}) RETURN count(n) AS cnt",
                    case_id=case_id,
                )
                count_rec = await count_result.single()
                node_count = count_rec["cnt"] if count_rec else 0

                # For large graphs, fall back to degree centrality
                # (betweenness via Cypher is O(n^3) and would be too slow)
                if node_count > 200 or node_count < 3:
                    query = (
                        "MATCH (n {case_id: $case_id})-[r]-(m) "
                        "WITH n, labels(n) AS labels, count(DISTINCT r) AS degree, "
                        "  count(DISTINCT m) AS distinct_neighbors "
                        "RETURN n.id AS id, n.name AS name, labels, "
                        "  degree, distinct_neighbors, "
                        "  0.0 AS betweenness_approx, "
                        "  toFloat(distinct_neighbors) / toFloat(degree + 1) AS bridge_score "
                        "ORDER BY degree DESC "
                        "LIMIT $limit"
                    )
                    result = await tx.run(query, case_id=case_id, limit=limit)
                    records = [r async for r in result]
                    return [
                        {
                            "id": rec["id"],
                            "name": rec["name"],
                            "labels": list(rec["labels"]),
                            "degree": rec["degree"],
                            "distinct_neighbors": rec["distinct_neighbors"],
                            "betweenness_approx": rec["betweenness_approx"],
                            "bridge_score": rec["bridge_score"],
                        }
                        for rec in records
                    ]

                # For smaller graphs: approximate betweenness using
                # shortest paths sampled between node pairs
                query = (
                    "MATCH (n {case_id: $case_id}) "
                    "WITH collect(n) AS nodes "
                    "UNWIND nodes AS a "
                    "UNWIND nodes AS b "
                    "WITH a, b WHERE a.id < b.id "
                    "MATCH p = shortestPath((a)-[*..6]-(b)) "
                    "UNWIND nodes(p) AS intermediate "
                    "WITH intermediate WHERE intermediate <> a AND intermediate <> b "
                    "WITH intermediate, count(*) AS pass_through "
                    "RETURN intermediate.id AS id, intermediate.name AS name, "
                    "  labels(intermediate) AS labels, pass_through AS betweenness_approx "
                    "ORDER BY betweenness_approx DESC "
                    "LIMIT $limit"
                )
                result = await tx.run(query, case_id=case_id, limit=limit)
                records = [r async for r in result]
                return [
                    {
                        "id": rec["id"],
                        "name": rec["name"],
                        "labels": list(rec["labels"]),
                        "betweenness_approx": rec["betweenness_approx"],
                    }
                    for rec in records
                ]

            return await session.execute_read(_work)

    async def detect_communities(
        self,
        case_id: str,
    ) -> List[List[Dict[str, Any]]]:
        """Detect communities/clusters of related entities.

        Uses connected components with enriched node data.
        Each community is a list of node dicts (id, name, labels).
        Communities are sorted by size (largest first).
        """
        async with self._driver.session() as session:

            async def _work(tx: AsyncManagedTransaction) -> List[List[Dict[str, Any]]]:
                # Get all node ids for the case
                result = await tx.run(
                    "MATCH (n {case_id: $case_id}) "
                    "RETURN n.id AS nid, n.name AS name, labels(n) AS labels",
                    case_id=case_id,
                )
                records = [r async for r in result]

                if not records:
                    return []

                # Build a lookup: id -> node info
                node_info: Dict[str, Dict[str, Any]] = {}
                all_ids: set[str] = set()
                for rec in records:
                    nid = rec["nid"]
                    all_ids.add(nid)
                    node_info[nid] = {
                        "id": nid,
                        "name": rec["name"],
                        "labels": list(rec["labels"]),
                    }

                # Find connected components using BFS via Cypher
                visited: set[str] = set()
                communities: List[List[Dict[str, Any]]] = []

                for nid in all_ids:
                    if nid in visited:
                        continue
                    comp_result = await tx.run(
                        "MATCH (start {id: $nid}) "
                        "OPTIONAL MATCH (start)-[*]-(connected) "
                        "WHERE connected.case_id = $case_id "
                        "WITH collect(DISTINCT connected.id) + [start.id] AS ids "
                        "RETURN ids",
                        nid=nid,
                        case_id=case_id,
                    )
                    comp_record = await comp_result.single()
                    if comp_record is None:
                        continue

                    component_ids = [
                        i for i in comp_record["ids"]
                        if i is not None and i in all_ids
                    ]
                    visited.update(component_ids)

                    if component_ids:
                        community = [
                            node_info[cid]
                            for cid in sorted(component_ids)
                            if cid in node_info
                        ]
                        if community:
                            communities.append(community)

                # Sort communities by size, largest first
                communities.sort(key=len, reverse=True)
                return communities

            return await session.execute_read(_work)

    async def find_indirect_connections(
        self,
        entity_id_1: str,
        entity_id_2: str,
        max_hops: int = 4,
    ) -> List[Dict[str, Any]]:
        """Find all paths between two entities up to max_hops.

        Useful for discovering hidden connections between suspects.
        Returns a list of paths, each path being a dict with:
        - nodes: list of node dicts along the path
        - relationships: list of relationship dicts
        - length: number of hops
        Paths are sorted by length (shortest first), limited to 20.
        """
        max_hops = min(max_hops, 6)  # Safety cap to avoid Cartesian explosion

        async with self._driver.session() as session:

            async def _work(tx: AsyncManagedTransaction) -> List[Dict[str, Any]]:
                query = (
                    "MATCH p = (a {id: $id1})-[*1.." + str(int(max_hops)) + "]-(b {id: $id2}) "
                    "WITH p, length(p) AS path_len "
                    "ORDER BY path_len "
                    "LIMIT 20 "
                    "RETURN nodes(p) AS path_nodes, "
                    "  relationships(p) AS path_rels, "
                    "  path_len"
                )
                result = await tx.run(
                    query, id1=entity_id_1, id2=entity_id_2,
                )
                records = [r async for r in result]

                paths: List[Dict[str, Any]] = []
                for rec in records:
                    nodes = []
                    for n in rec["path_nodes"]:
                        nd = dict(n)
                        nd["_labels"] = list(n.labels)
                        nodes.append(nd)

                    rels = []
                    for r in rec["path_rels"]:
                        rels.append({
                            "from": dict(r.start_node).get("id"),
                            "to": dict(r.end_node).get("id"),
                            "type": r.type,
                            "properties": dict(r),
                        })

                    paths.append({
                        "nodes": nodes,
                        "relationships": rels,
                        "length": rec["path_len"],
                    })

                return paths

            return await session.execute_read(_work)

    async def get_temporal_graph(
        self,
        case_id: str,
    ) -> List[Dict[str, Any]]:
        """Get events and relationships ordered by time.

        Returns a temporal view of the knowledge graph: events and
        time-stamped entities sorted chronologically.
        Each result contains the node and its relationships.
        """
        async with self._driver.session() as session:

            async def _work(tx: AsyncManagedTransaction) -> List[Dict[str, Any]]:
                # Get nodes with temporal properties, ordered by first_seen/created_at
                query = (
                    "MATCH (n {case_id: $case_id}) "
                    "WHERE n.first_seen IS NOT NULL "
                    "   OR n.created_at IS NOT NULL "
                    "   OR n:Event "
                    "OPTIONAL MATCH (n)-[r]-(m {case_id: $case_id}) "
                    "WITH n, labels(n) AS labels, "
                    "  collect({target_id: m.id, target_name: m.name, "
                    "    rel_type: type(r), direction: CASE "
                    "      WHEN startNode(r) = n THEN 'out' "
                    "      ELSE 'in' END}) AS connections, "
                    "  coalesce(n.first_seen, n.created_at, '9999') AS sort_date "
                    "ORDER BY sort_date "
                    "RETURN n.id AS id, n.name AS name, labels, "
                    "  n.first_seen AS first_seen, "
                    "  n.created_at AS created_at, "
                    "  n.description AS description, "
                    "  connections, sort_date "
                    "LIMIT 500"
                )
                result = await tx.run(query, case_id=case_id)
                records = [r async for r in result]
                return [
                    {
                        "id": rec["id"],
                        "name": rec["name"],
                        "labels": list(rec["labels"]),
                        "first_seen": rec["first_seen"],
                        "created_at": rec["created_at"],
                        "description": rec["description"],
                        "connections": [
                            c for c in rec["connections"]
                            if c.get("target_id") is not None
                        ],
                        "sort_date": rec["sort_date"],
                    }
                    for rec in records
                ]

            return await session.execute_read(_work)

    async def get_evidence_entity_matrix(
        self,
        case_id: str,
    ) -> Dict[str, Any]:
        """Build a matrix of which entities appear in which evidence.

        Returns:
        - matrix: dict mapping evidence_id -> list of entity_ids it mentions
        - entities: dict mapping entity_id -> {name, labels}
        - evidence: dict mapping evidence_id -> {title}
        - co_occurrences: list of entity pairs that appear together frequently
        """
        async with self._driver.session() as session:

            async def _work(tx: AsyncManagedTransaction) -> Dict[str, Any]:
                # Get all Evidence -> MENTIONS -> Entity links
                query = (
                    "MATCH (ev:Evidence {case_id: $case_id})-[:MENTIONS]->(ent {case_id: $case_id}) "
                    "RETURN ev.id AS evidence_id, ev.title AS evidence_title, "
                    "  ent.id AS entity_id, ent.name AS entity_name, "
                    "  labels(ent) AS entity_labels "
                    "ORDER BY ev.id, ent.name"
                )
                result = await tx.run(query, case_id=case_id)
                records = [r async for r in result]

                matrix: Dict[str, List[str]] = {}
                entities: Dict[str, Dict[str, Any]] = {}
                evidence: Dict[str, Dict[str, Any]] = {}

                for rec in records:
                    eid = rec["evidence_id"]
                    ent_id = rec["entity_id"]

                    # Build matrix
                    matrix.setdefault(eid, [])
                    if ent_id not in matrix[eid]:
                        matrix[eid].append(ent_id)

                    # Track entity info
                    if ent_id not in entities:
                        entities[ent_id] = {
                            "name": rec["entity_name"],
                            "labels": list(rec["entity_labels"]),
                        }

                    # Track evidence info
                    if eid not in evidence:
                        evidence[eid] = {"title": rec["evidence_title"]}

                # Compute co-occurrences: entity pairs appearing in same evidence
                pair_counts: Counter = Counter()
                for eid, ent_ids in matrix.items():
                    sorted_ids = sorted(ent_ids)
                    for i in range(len(sorted_ids)):
                        for j in range(i + 1, len(sorted_ids)):
                            pair_counts[(sorted_ids[i], sorted_ids[j])] += 1

                co_occurrences = [
                    {
                        "entity_1": pair[0],
                        "entity_1_name": entities.get(pair[0], {}).get("name", ""),
                        "entity_2": pair[1],
                        "entity_2_name": entities.get(pair[1], {}).get("name", ""),
                        "count": count,
                    }
                    for pair, count in pair_counts.most_common(50)
                    if count >= 2
                ]

                return {
                    "matrix": matrix,
                    "entities": entities,
                    "evidence": evidence,
                    "co_occurrences": co_occurrences,
                }

            return await session.execute_read(_work)
