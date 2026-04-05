"""
NEXUS -- Evidence Processor.

Full ingestion pipeline for uploaded files and manual text input:
save file, detect type, extract text, create DB entry, run entity
extraction via LLM, generate summary, and return the completed
Evidence record.

Usage::

    async with get_db() as conn:
        db = Database(conn)
        router = LLMRouter()
        proc = EvidenceProcessor(db, router, settings.upload_dir)
        evidence = await proc.process_upload(case_id, file, "Report.pdf")
"""

from __future__ import annotations

import hashlib
import shutil
import uuid
from pathlib import Path
from typing import Any, BinaryIO

from loguru import logger

from nexus.config import settings
from nexus.core.audit import AuditService
from nexus.core.chunker import TextChunker
from nexus.core.embedding_store import EmbeddingStore
from nexus.core.entity_extractor import EntityExtractor
from nexus.core.image_analyzer import ImageAnalyzer
from nexus.db.models import Evidence, Entity
from nexus.db.sqlite_db import Database
from nexus.ingest.pdf_parser import PDFParser, compute_file_hash, detect_mime_type
from nexus.ingest.text_parser import TextParser
from nexus.llm.prompts import EVIDENCE_SUMMARY_PROMPT
from nexus.llm.router import LLMRouter, TaskType


# Map MIME types to evidence_type values understood by the schema.
_MIME_TO_EVIDENCE_TYPE: dict[str, str] = {
    "application/pdf": "pdf",
    "image/jpeg": "image",
    "image/png": "image",
    "image/gif": "image",
    "image/webp": "image",
    "image/tiff": "image",
    "text/plain": "text",
    "text/html": "text",
    "text/csv": "text",
    "text/markdown": "text",
    "audio/mpeg": "audio",
    "audio/wav": "audio",
    "audio/ogg": "audio",
    "audio/flac": "audio",
    "video/mp4": "audio",
    "video/webm": "audio",
}


class EvidenceProcessor:
    """Pipeline for ingesting evidence into a case."""

    def __init__(
        self,
        db: Database,
        router: LLMRouter,
        upload_dir: Path,
        neo4j=None,
        chroma=None,
    ) -> None:
        self._db = db
        self._router = router
        self._upload_dir = Path(upload_dir)
        self._pdf_parser = PDFParser()
        self._text_parser = TextParser()
        self._entity_extractor = EntityExtractor(router)
        self._neo4j = neo4j   # Optional Neo4jClient
        self._chroma = chroma  # Optional ChromaClient
        self._audit = AuditService(db)

    # ==================================================================
    # Public API
    # ==================================================================

    async def process_upload(
        self,
        case_id: str,
        file: BinaryIO,
        title: str,
        source: str | None = None,
        evidence_type: str | None = None,
    ) -> Evidence:
        """Full pipeline for an uploaded file.

        Steps:
          1. Save file to data/uploads/{case_id}/{uuid}.{ext}
          2. Detect MIME type / evidence_type
          3. Extract text (PDF or text parser)
          4. Compute SHA-256 hash
          5. Create evidence record in SQLite (status='processing')
          6. Extract entities via LLM and save them
          7. Generate summary via LLM
          8. Update evidence (summary + status='processed')
          9. Return the completed Evidence

        Parameters:
            case_id: The case this evidence belongs to.
            file: A file-like object with a ``.read()`` method.
                  If it has a ``.filename`` attribute (e.g. Starlette
                  UploadFile) it will be used for the extension.
            title: Human-readable title for the evidence.
            source: Optional provenance description.
            evidence_type: Override the auto-detected type.
        """
        # ----------------------------------------------------------
        # 1. Save file to disk
        # ----------------------------------------------------------
        original_filename = getattr(file, "filename", None) or "upload"
        ext = Path(original_filename).suffix or ".bin"
        file_uuid = str(uuid.uuid4())
        dest_dir = self._upload_dir / case_id
        dest_dir.mkdir(parents=True, exist_ok=True)
        dest_path = dest_dir / f"{file_uuid}{ext}"

        logger.info("Saving upload to {}", dest_path)
        content = file.read()
        dest_path.write_bytes(content)

        # ----------------------------------------------------------
        # 2. Detect MIME / evidence type
        # ----------------------------------------------------------
        mime = detect_mime_type(dest_path)
        if evidence_type is None:
            evidence_type = _MIME_TO_EVIDENCE_TYPE.get(mime, "manual")
        logger.debug("Detected MIME={}, evidence_type={}", mime, evidence_type)

        # ----------------------------------------------------------
        # 3. Extract text
        # ----------------------------------------------------------
        raw_text = self._extract_text(dest_path, evidence_type)

        # ----------------------------------------------------------
        # 4. Compute hash
        # ----------------------------------------------------------
        file_hash = compute_file_hash(dest_path)

        # ----------------------------------------------------------
        # 5. Create evidence record (status='processing')
        # ----------------------------------------------------------
        ev_row = await self._db.create_evidence(
            case_id=case_id,
            title=title,
            evidence_type=evidence_type,
            source=source,
            file_path=str(dest_path),
            raw_text=raw_text,
            status="pending",
            metadata={"file_hash": file_hash, "mime_type": mime},
        )
        evidence_id = ev_row["id"]
        logger.info("Evidence record created: {}", evidence_id)

        # Mark as processing
        await self._db.update_evidence(evidence_id, status="processing")

        # ----------------------------------------------------------
        # 6–8. Branch: image evidence → visual pipeline
        # ----------------------------------------------------------
        if evidence_type == "image":
            try:
                image_analyzer = ImageAnalyzer(
                    router=self._router,
                    db=self._db,
                    chroma=self._chroma,
                )
                img_result = await image_analyzer.process_evidence_image(
                    case_id=case_id,
                    evidence_id=evidence_id,
                    image_path=dest_path,
                )
                # process_evidence_image already updates the evidence record
                # (raw_text, summary, status) and saves entities + embeddings.
                logger.info(
                    "Image evidence {} processed via visual pipeline",
                    evidence_id,
                )
                # Audit: log evidence added (image pipeline)
                await self._audit.log_evidence_added(
                    case_id, evidence_id, title, source,
                )
                updated = await self._db.get_evidence(evidence_id)
                return Evidence(**updated)
            except Exception as exc:
                logger.error(
                    "Visual pipeline failed for {}; falling back to text pipeline: {}",
                    evidence_id,
                    exc,
                )
                # Fall through to the standard text pipeline

        # ----------------------------------------------------------
        # 6. Entity extraction + save (text pipeline)
        # ----------------------------------------------------------
        try:
            entities = await self._extract_and_save_entities(
                case_id, evidence_id, raw_text or ""
            )
            logger.info("Saved {} entities for evidence {}", len(entities), evidence_id)
        except Exception as exc:
            logger.error("Entity extraction failed for {}: {}", evidence_id, exc)
            # Non-fatal: we continue to summary

        # ----------------------------------------------------------
        # 7. Generate summary
        # ----------------------------------------------------------
        summary = ""
        try:
            if raw_text:
                summary = await self._generate_summary(raw_text)
        except Exception as exc:
            logger.error("Summary generation failed for {}: {}", evidence_id, exc)

        # ----------------------------------------------------------
        # 8. Update evidence (summary + status)
        # ----------------------------------------------------------
        updated = await self._db.update_evidence(
            evidence_id,
            summary=summary,
            status="processed",
        )

        # ----------------------------------------------------------
        # 9. Sync to Neo4j + ChromaDB (Phase 2)
        # ----------------------------------------------------------
        await self._sync_to_graph_and_vectors(case_id, evidence_id, raw_text or "", summary)

        # ----------------------------------------------------------
        # 10. Chunk and embed for RAG
        # ----------------------------------------------------------
        await self._chunk_and_embed(updated)

        # ----------------------------------------------------------
        # 11. Update summary tree (RAPTOR hierarchical summaries)
        # ----------------------------------------------------------
        try:
            from nexus.core.summary_tree import SummaryTree

            tree = SummaryTree(self._db, self._router, self._chroma)
            await tree.update_for_new_evidence(case_id, evidence_id)
        except Exception as exc:
            logger.warning(
                "Summary tree update failed for evidence {} (non-blocking): {}",
                evidence_id, exc,
            )

        # Audit: log evidence added (upload pipeline)
        await self._audit.log_evidence_added(
            case_id, evidence_id, title, source,
        )

        logger.info("Evidence {} processing complete", evidence_id)
        return Evidence(**updated)

    async def process_text_input(
        self,
        case_id: str,
        title: str,
        text: str,
        source: str | None = None,
    ) -> Evidence:
        """Pipeline for manually-entered text (no file upload).

        Same as process_upload but skips file save and MIME detection.
        """
        logger.info("Processing text input for case {}: '{}'", case_id, title)

        # Clean the raw text
        cleaned = self._text_parser.extract_from_string(text)

        # Compute hash of the text content itself
        text_hash = hashlib.sha256(cleaned.encode("utf-8")).hexdigest()

        # Create evidence record
        ev_row = await self._db.create_evidence(
            case_id=case_id,
            title=title,
            evidence_type="text",
            source=source,
            raw_text=cleaned,
            status="pending",
            metadata={"text_hash": text_hash},
        )
        evidence_id = ev_row["id"]

        # Mark as processing
        await self._db.update_evidence(evidence_id, status="processing")

        # Entity extraction
        try:
            entities = await self._extract_and_save_entities(
                case_id, evidence_id, cleaned
            )
            logger.info("Saved {} entities for text evidence {}", len(entities), evidence_id)
        except Exception as exc:
            logger.error("Entity extraction failed for {}: {}", evidence_id, exc)

        # Summary
        summary = ""
        try:
            if cleaned:
                summary = await self._generate_summary(cleaned)
        except Exception as exc:
            logger.error("Summary generation failed for {}: {}", evidence_id, exc)

        # Finalise
        updated = await self._db.update_evidence(
            evidence_id,
            summary=summary,
            status="processed",
        )

        # Sync to Neo4j + ChromaDB
        await self._sync_to_graph_and_vectors(case_id, evidence_id, cleaned, summary)

        # Chunk and embed for RAG
        await self._chunk_and_embed(updated)

        # Update summary tree (RAPTOR hierarchical summaries)
        try:
            from nexus.core.summary_tree import SummaryTree

            tree = SummaryTree(self._db, self._router, self._chroma)
            await tree.update_for_new_evidence(case_id, evidence_id)
        except Exception as exc:
            logger.warning(
                "Summary tree update failed for text evidence {} (non-blocking): {}",
                evidence_id, exc,
            )

        # Audit: log evidence added (text pipeline)
        await self._audit.log_evidence_added(
            case_id, evidence_id, title, source,
        )

        logger.info("Text evidence {} processing complete", evidence_id)
        return Evidence(**updated)

    # ==================================================================
    # Internal helpers
    # ==================================================================

    async def _extract_and_save_entities(
        self,
        case_id: str,
        evidence_id: str,
        text: str,
    ) -> list[Entity]:
        """Extract entities from text, deduplicate, save to DB.

        Returns the list of Entity models that were created (new entities
        only -- duplicates are linked via mentions but not re-created).
        """
        if not text.strip():
            return []

        # 1. Extract via LLM
        raw_entities = await self._entity_extractor.extract_entities(text)
        if not raw_entities:
            return []

        # 2. Load existing entities for this case (for dedup)
        existing = await self._db.list_entities_by_case(case_id)

        # 3. Deduplicate
        new_entities = self._entity_extractor.deduplicate_entities(
            raw_entities, existing
        )

        created: list[Entity] = []

        # 4. Save new entities + create mentions
        for ent in new_entities:
            entity_row = await self._db.create_entity(
                case_id=case_id,
                name=ent["name"],
                entity_type=ent["type"],
                description=ent.get("context"),
            )
            entity = Entity(**entity_row)
            created.append(entity)

            # Audit: log entity discovered
            await self._audit.log_entity_discovered(
                case_id, entity.id, ent["name"], ent["type"],
            )

            # Create a mention linking entity <-> evidence
            await self._db.create_entity_mention(
                entity_id=entity.id,
                evidence_id=evidence_id,
                context=ent.get("context", ""),
                confidence=ent.get("confidence", 0.8),
            )

        # 5. For duplicates that already exist, still create mentions
        for ent in raw_entities:
            if ent in new_entities:
                continue  # already handled above
            # Find the matching existing entity
            norm_name = self._entity_extractor.normalize_entity_name(ent["name"])
            for ex in existing:
                ex_norm = self._entity_extractor.normalize_entity_name(ex.get("name", ""))
                ex_type = ex.get("entity_type", "")
                if ex_norm == norm_name and ex_type == ent.get("type", ""):
                    await self._db.create_entity_mention(
                        entity_id=ex["id"],
                        evidence_id=evidence_id,
                        context=ent.get("context", ""),
                        confidence=ent.get("confidence", 0.8),
                    )
                    break

        return created

    async def _generate_summary(self, text: str) -> str:
        """Generate a factual summary of the evidence text via LLM."""
        # Truncate for the summary prompt
        truncated = text[:8_000] if len(text) > 8_000 else text
        prompt = EVIDENCE_SUMMARY_PROMPT.format(evidence=truncated)

        logger.debug("Generating summary for {} chars of text", len(truncated))
        summary = await self._router.route(
            TaskType.EVIDENCE_SUMMARY,
            prompt,
        )
        # Clean up: the LLM may wrap the summary in markdown or artifacts
        summary = summary.strip()
        return summary

    async def _chunk_and_embed(self, evidence: dict) -> None:
        """Chunk evidence text and index embeddings for RAG retrieval."""
        if self._chroma is None:
            return

        evidence_id = evidence.get("id", "unknown")
        try:
            chunker = TextChunker(
                chunk_size=settings.rag_chunk_size,
                overlap=settings.rag_chunk_overlap,
            )
            chunks = chunker.chunk_evidence(evidence)
            if chunks:
                store = EmbeddingStore(self._chroma, self._router)
                n = await store.index_evidence(evidence, chunks)
                logger.info("Indexed {} RAG chunks for evidence {}", n, evidence_id)
            else:
                logger.debug("No text to chunk for evidence {}", evidence_id)
        except Exception as exc:
            logger.error(
                "RAG chunk+embed failed for evidence {}: {}", evidence_id, exc
            )

    async def _sync_to_graph_and_vectors(
        self,
        case_id: str,
        evidence_id: str,
        raw_text: str,
        summary: str,
    ) -> None:
        """Sync evidence + entities to Neo4j graph and ChromaDB vectors."""
        # --- Neo4j: sync evidence node + entity nodes + relations ---
        if self._neo4j is not None:
            try:
                # Create Evidence node
                ev = await self._db.get_evidence(evidence_id)
                if ev:
                    await self._neo4j.sync_evidence(
                        evidence_id=evidence_id,
                        case_id=case_id,
                        title=ev["title"],
                        evidence_type=ev["evidence_type"],
                        reliability=ev.get("reliability", 50),
                    )

                # Sync entities to Neo4j + link to evidence
                entities = await self._db.list_entities_by_case(case_id)
                for ent in entities:
                    await self._neo4j.sync_entity(ent, case_id)
                    await self._neo4j.link_evidence_to_entity(evidence_id, ent["id"])

                # Extract and sync relations between entities
                try:
                    ent_dicts = [{"name": e["name"], "type": e["entity_type"]} for e in entities]
                    if len(ent_dicts) >= 2:
                        relations = await self._entity_extractor.extract_relations(ent_dicts)
                        if relations:
                            # Map relation entity names to IDs
                            name_to_id = {}
                            for e in entities:
                                norm = self._entity_extractor.normalize_entity_name(e["name"])
                                name_to_id[norm] = e["id"]

                            mapped_relations = []
                            for rel in relations:
                                from_name = self._entity_extractor.normalize_entity_name(rel.get("from", ""))
                                to_name = self._entity_extractor.normalize_entity_name(rel.get("to", ""))
                                if from_name in name_to_id and to_name in name_to_id:
                                    mapped_relations.append({
                                        "from_id": name_to_id[from_name],
                                        "to_id": name_to_id[to_name],
                                        "type": rel.get("type", "RELATED_TO"),
                                        "context": rel.get("context", ""),
                                    })
                            if mapped_relations:
                                await self._neo4j.sync_relations(mapped_relations, case_id)
                                logger.info("Synced {} relations to Neo4j", len(mapped_relations))
                except Exception as exc:
                    logger.warning("Relation extraction/sync failed: {}", exc)

                logger.info("Neo4j sync complete for evidence {}", evidence_id)
            except Exception as exc:
                logger.error("Neo4j sync failed for evidence {}: {}", evidence_id, exc)

        # --- ChromaDB: embed evidence text ---
        if self._chroma is not None and raw_text:
            try:
                text_to_embed = summary if summary else raw_text[:4000]
                embedding = await self._router.embed(text_to_embed)
                self._chroma.add_evidence(
                    evidence_id=evidence_id,
                    case_id=case_id,
                    text=text_to_embed,
                    embedding=embedding,
                    metadata={
                        "case_id": case_id,
                        "evidence_id": evidence_id,
                    },
                )
                logger.info("ChromaDB embedding added for evidence {}", evidence_id)
            except Exception as exc:
                logger.error("ChromaDB sync failed for evidence {}: {}", evidence_id, exc)

    def _extract_text(self, file_path: Path, evidence_type: str) -> str | None:
        """Synchronous text extraction based on evidence type.

        Returns the extracted text, or None if the type is not supported
        for text extraction (e.g. images, audio).
        """
        try:
            if evidence_type == "pdf":
                return self._pdf_parser.extract_text(file_path)
            elif evidence_type == "text":
                return self._text_parser.extract_text(file_path)
            else:
                # image, audio, url, manual -- no direct text extraction
                logger.debug(
                    "No text extraction for evidence_type={}", evidence_type
                )
                return None
        except Exception as exc:
            logger.error(
                "Text extraction failed for {} (type={}): {}",
                file_path.name,
                evidence_type,
                exc,
            )
            return None
