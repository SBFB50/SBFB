"""NEXUS GOV -- Political RAG (Retrieval-Augmented Generation)."""
from __future__ import annotations

from typing import Any

from loguru import logger

GOV_COLLECTION = "gov_corpus"


class GovRAG:
    """Semantic search across all political data."""

    def __init__(self, chroma: Any = None, router: Any = None) -> None:
        self._chroma = chroma
        self._router = router
        self._collection = None

    def _get_collection(self):
        if self._collection is None and self._chroma is not None:
            try:
                self._collection = self._chroma._client.get_or_create_collection(
                    name=GOV_COLLECTION,
                    metadata={"hnsw:space": "cosine"},
                )
            except Exception:
                pass
        return self._collection

    async def search(self, query: str, *, n_results: int = 10) -> list[dict]:
        """Semantic search across all political data."""
        collection = self._get_collection()
        if collection is None:
            return []

        try:
            # Generate query embedding
            if self._router:
                embedding = await self._router.route_embedding(query)
                if embedding:
                    results = collection.query(
                        query_embeddings=[embedding],
                        n_results=n_results,
                    )
                else:
                    results = collection.query(
                        query_texts=[query],
                        n_results=n_results,
                    )
            else:
                results = collection.query(
                    query_texts=[query],
                    n_results=n_results,
                )

            items = []
            ids = results.get("ids", [[]])[0]
            docs = results.get("documents", [[]])[0]
            metas = results.get("metadatas", [[]])[0]
            distances = results.get("distances", [[]])[0]

            for i in range(len(ids)):
                items.append({
                    "id": ids[i],
                    "text": docs[i] if i < len(docs) else "",
                    "metadata": metas[i] if i < len(metas) else {},
                    "score": round(1 - (distances[i] if i < len(distances) else 0), 3),
                })

            return items
        except Exception as exc:
            logger.warning("Gov RAG search failed: {}", exc)
            return []

    async def ask(self, question: str, *, n_context: int = 5) -> dict:
        """Ask a question with RAG context."""
        results = await self.search(question, n_results=n_context)
        if not results:
            return {"answer": "Aucune donnee pertinente trouvee.", "sources": []}

        context = "\n\n".join(
            f"[{r['metadata'].get('type', '?')}] {r['text']}"
            for r in results
        )

        prompt = f"""Tu es un assistant factuel specialise dans la politique francaise.
Reponds a la question en te basant UNIQUEMENT sur les sources fournies.
Si les sources ne contiennent pas la reponse, dis-le.

SOURCES:
{context}

QUESTION: {question}

REPONSE (factuelle, sourcee):"""

        if self._router:
            from nexus.llm.router import TaskType
            answer = await self._router.route(TaskType.SUMMARIZE, prompt)
        else:
            answer = "LLM non disponible."

        return {
            "answer": answer,
            "sources": [{"id": r["id"], "score": r["score"], "type": r["metadata"].get("type")} for r in results],
        }
