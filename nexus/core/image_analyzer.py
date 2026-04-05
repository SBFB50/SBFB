"""
NEXUS -- Image Analyzer.

Complete visual analysis pipeline for evidence images:
- Quick description (gemma4:e4b)
- Visual entity extraction (gemma4:e4b)
- Deep scene analysis (qwen3-vl:8b)
- Image comparison (qwen3-vl:8b)
- Embedding via text description (nomic-embed-text)
- Full evidence processing pipeline

Usage::

    analyzer = ImageAnalyzer(router, db, chroma)
    result = await analyzer.process_evidence_image(case_id, evidence_id, path)
"""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any, Dict, List, Optional

from loguru import logger

from nexus.db.chroma_db import ChromaClient
from nexus.db.sqlite_db import Database
from nexus.llm.prompts import (
    IMAGE_COMPARISON_PROMPT,
    IMAGE_DESCRIPTION_PROMPT,
    IMAGE_ENTITY_EXTRACTION_PROMPT,
    IMAGE_SCENE_ANALYSIS_PROMPT,
)
from nexus.llm.router import LLMRouter, TaskType


class ImageAnalyzer:
    """Full visual analysis pipeline for NEXUS investigation images."""

    def __init__(
        self,
        router: LLMRouter,
        db: Database,
        chroma: Optional[ChromaClient] = None,
    ) -> None:
        self._router = router
        self._db = db
        self._chroma = chroma

    # ==================================================================
    # Quick description
    # ==================================================================

    async def describe_image(self, image_path: str | Path) -> str:
        """Generate a detailed textual description of an image.

        Uses the fast vision model (gemma4:e4b) for quick turnaround.
        """
        logger.info("Describing image: {}", Path(image_path).name)
        description = await self._router.route_vision(
            TaskType.IMAGE_DESCRIPTION,
            IMAGE_DESCRIPTION_PROMPT,
            image_path,
        )
        return description.strip()

    # ==================================================================
    # Visual entity extraction
    # ==================================================================

    async def extract_entities_from_image(
        self, image_path: str | Path
    ) -> List[Dict[str, Any]]:
        """Extract visual entities from an image via VLM.

        Returns a list of entity dicts with keys:
        name, type, description, position, confidence.
        """
        logger.info("Extracting visual entities from: {}", Path(image_path).name)
        raw = await self._router.route_vision(
            TaskType.IMAGE_ENTITY_EXTRACTION,
            IMAGE_ENTITY_EXTRACTION_PROMPT,
            image_path,
        )
        return self._parse_entity_json(raw)

    # ==================================================================
    # Deep scene analysis
    # ==================================================================

    async def analyze_scene(
        self,
        image_path: str | Path,
        case_context: str = "",
    ) -> Dict[str, Any]:
        """Perform deep scene analysis using the advanced VLM (qwen3-vl).

        Returns a dict with keys: description, key_elements,
        spatial_relations, anomalies, potential_clues, questions, raw.
        """
        logger.info("Deep scene analysis: {}", Path(image_path).name)
        prompt = IMAGE_SCENE_ANALYSIS_PROMPT.format(
            case_context=case_context or "Aucun contexte fourni."
        )
        raw = await self._router.route_vision(
            TaskType.IMAGE_SCENE_ANALYSIS,
            prompt,
            image_path,
        )
        return self._parse_scene_analysis(raw)

    # ==================================================================
    # Image comparison
    # ==================================================================

    async def compare_images(
        self,
        image_path_1: str | Path,
        image_path_2: str | Path,
    ) -> Dict[str, Any]:
        """Compare two images and identify similarities/differences.

        Since VLMs process one image at a time, we describe both images
        independently and then ask the reasoning model to compare them.
        """
        logger.info(
            "Comparing images: {} vs {}",
            Path(image_path_1).name,
            Path(image_path_2).name,
        )

        # Step 1: Describe both images
        desc_1 = await self.describe_image(image_path_1)
        desc_2 = await self.describe_image(image_path_2)

        # Step 2: Send comparison prompt with second image + both descriptions
        comparison_prompt = (
            f"{IMAGE_COMPARISON_PROMPT}\n\n"
            f"DESCRIPTION IMAGE 1:\n{desc_1}\n\n"
            f"DESCRIPTION IMAGE 2:\n{desc_2}"
        )

        raw = await self._router.route_vision(
            TaskType.IMAGE_COMPARISON,
            comparison_prompt,
            image_path_2,
        )

        return {
            "description_1": desc_1,
            "description_2": desc_2,
            "comparison": raw.strip(),
        }

    # ==================================================================
    # Embed image (via text description)
    # ==================================================================

    async def embed_image(self, image_path: str | Path) -> List[float]:
        """Embed an image by first describing it then embedding the text.

        Uses nomic-embed-text on the generated description so that
        images become searchable alongside textual evidence.
        """
        description = await self.describe_image(image_path)
        embedding = await self._router.embed(description)
        return embedding

    # ==================================================================
    # Full evidence processing pipeline
    # ==================================================================

    async def process_evidence_image(
        self,
        case_id: str,
        evidence_id: str,
        image_path: str | Path,
    ) -> Dict[str, Any]:
        """Complete image evidence pipeline.

        Steps:
          1. Quick description (gemma4:e4b)
          2. Extract visual entities (gemma4:e4b)
          3. Deep scene analysis (qwen3-vl:8b)
          4. Embed description in ChromaDB
          5. Save visual entities to SQLite
          6. Update evidence.summary + evidence.raw_text
          7. Return aggregated result
        """
        image_path = Path(image_path)
        logger.info(
            "Processing evidence image: case={} evidence={} file={}",
            case_id[:8],
            evidence_id[:8],
            image_path.name,
        )

        result: Dict[str, Any] = {
            "evidence_id": evidence_id,
            "case_id": case_id,
            "image_path": str(image_path),
        }

        # --- 1. Quick description ---
        try:
            description = await self.describe_image(image_path)
            result["description"] = description
            logger.info("Image described ({} chars)", len(description))
        except Exception as exc:
            logger.error("Image description failed: {}", exc)
            result["description"] = ""
            description = ""

        # --- 2. Entity extraction ---
        entities: List[Dict[str, Any]] = []
        try:
            entities = await self.extract_entities_from_image(image_path)
            result["entities"] = entities
            logger.info("Extracted {} visual entities", len(entities))
        except Exception as exc:
            logger.error("Visual entity extraction failed: {}", exc)
            result["entities"] = []

        # --- 3. Deep scene analysis ---
        try:
            # Get case context for the analysis
            case = await self._db.get_case(case_id)
            case_context = (
                f"{case.get('name', '')} -- {case.get('description', '')}"
                if case
                else ""
            )
            scene = await self.analyze_scene(image_path, case_context)
            result["scene_analysis"] = scene
            logger.info("Scene analysis complete")
        except Exception as exc:
            logger.error("Scene analysis failed: {}", exc)
            result["scene_analysis"] = {}

        # --- 4. Embed description in ChromaDB ---
        if self._chroma and description:
            try:
                embedding = await self._router.embed(description)
                self._chroma.add_evidence(
                    evidence_id=evidence_id,
                    case_id=case_id,
                    text=description,
                    embedding=embedding,
                    metadata={
                        "case_id": case_id,
                        "evidence_id": evidence_id,
                        "source_type": "image_description",
                    },
                )
                logger.info("Image embedding stored in ChromaDB")
            except Exception as exc:
                logger.error("ChromaDB embedding failed: {}", exc)

        # --- 5. Save visual entities to SQLite ---
        saved_entities = 0
        for ent in entities:
            try:
                # Map visual entity types to the DB schema entity types
                ent_type = self._map_visual_entity_type(ent.get("type", "other"))
                entity_row = await self._db.create_entity(
                    case_id=case_id,
                    name=ent.get("name", "inconnu"),
                    entity_type=ent_type,
                    description=ent.get("description", ""),
                    metadata={
                        "source": "visual_extraction",
                        "evidence_id": evidence_id,
                        "position": ent.get("position", ""),
                        "confidence": ent.get("confidence", 0.5),
                    },
                )
                # Link entity to evidence via mention
                await self._db.create_entity_mention(
                    entity_id=entity_row["id"],
                    evidence_id=evidence_id,
                    context=ent.get("description", ""),
                    confidence=float(ent.get("confidence", 0.5)),
                )
                saved_entities += 1
            except Exception as exc:
                logger.warning("Failed to save visual entity '{}': {}", ent.get("name"), exc)

        result["saved_entities_count"] = saved_entities
        logger.info("Saved {} visual entities to SQLite", saved_entities)

        # --- 6. Update evidence record ---
        try:
            await self._db.update_evidence(
                evidence_id,
                raw_text=description,
                summary=description[:500] if description else "",
                status="processed",
            )
            logger.info("Evidence record updated with image description")
        except Exception as exc:
            logger.error("Failed to update evidence record: {}", exc)

        # --- 7. Return ---
        result["status"] = "processed"
        return result

    # ==================================================================
    # Internal helpers
    # ==================================================================

    @staticmethod
    def _parse_entity_json(raw: str) -> List[Dict[str, Any]]:
        """Parse the VLM entity extraction JSON response.

        Handles common issues: markdown code blocks, trailing commas,
        partial JSON.  Returns an empty list on failure.
        """
        # Strip markdown code fences
        cleaned = raw.strip()
        if cleaned.startswith("```"):
            cleaned = re.sub(r"^```(?:json)?\s*", "", cleaned)
            cleaned = re.sub(r"\s*```\s*$", "", cleaned)

        try:
            data = json.loads(cleaned)
            return data.get("entities", [])
        except json.JSONDecodeError:
            # Try to find JSON object in the text
            match = re.search(r"\{[\s\S]*\}", cleaned)
            if match:
                try:
                    data = json.loads(match.group())
                    return data.get("entities", [])
                except json.JSONDecodeError:
                    pass
            logger.warning("Failed to parse entity JSON from VLM response")
            return []

    @staticmethod
    def _parse_scene_analysis(raw: str) -> Dict[str, Any]:
        """Parse the VLM scene analysis into structured sections.

        Extracts numbered sections from the text output.
        """
        result: Dict[str, Any] = {"raw": raw.strip()}

        # Try to extract each section by number or title
        section_map = {
            "description": r"(?:1\.|DESCRIPTION GENERALE)[:\s]*(.+?)(?=(?:\d\.|[A-Z]{3,}|$))",
            "key_elements": r"(?:2\.|ELEMENTS CLES)[:\s]*(.+?)(?=(?:\d\.|[A-Z]{3,}|$))",
            "spatial_relations": r"(?:3\.|RELATIONS SPATIALES)[:\s]*(.+?)(?=(?:\d\.|[A-Z]{3,}|$))",
            "anomalies": r"(?:4\.|ANOMALIES)[:\s]*(.+?)(?=(?:\d\.|[A-Z]{3,}|$))",
            "potential_clues": r"(?:5\.|INDICES POTENTIELS)[:\s]*(.+?)(?=(?:\d\.|[A-Z]{3,}|$))",
            "questions": r"(?:6\.|QUESTIONS)[:\s]*(.+?)$",
        }

        for key, pattern in section_map.items():
            match = re.search(pattern, raw, re.DOTALL | re.IGNORECASE)
            result[key] = match.group(1).strip() if match else ""

        return result

    @staticmethod
    def _map_visual_entity_type(visual_type: str) -> str:
        """Map visual entity types to the DB schema EntityType values."""
        mapping = {
            "person": "person",
            "vehicle": "vehicle",
            "location": "location",
            "object": "other",
            "weapon": "weapon",
            "other": "other",
        }
        return mapping.get(visual_type.lower(), "other")
