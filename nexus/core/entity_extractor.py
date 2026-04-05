"""
NEXUS -- Entity Extractor.

Uses the LLM router to extract named entities from text, deduce
relations between them, and deduplicate against already-known entities.

The extraction model (gemma4:e4b) is fast (~80 tok/s) and handles
the structured JSON output required by the entity/relation prompts.

Usage::

    router = LLMRouter()
    extractor = EntityExtractor(router)
    entities = await extractor.extract_entities("John Doe was in Paris on 12 Jan.")
"""

from __future__ import annotations

import re
import unicodedata
from typing import Any

from loguru import logger

from nexus.llm.parsers import parse_entities, parse_relations
from nexus.llm.prompts import (
    ENTITY_EXTRACTION_PROMPT,
    RELATION_EXTRACTION_PROMPT,
)
from nexus.llm.router import LLMRouter, TaskType


class EntityExtractor:
    """Extract entities and relations from text via LLM."""

    def __init__(self, router: LLMRouter) -> None:
        self._router = router

    # ------------------------------------------------------------------
    # Entity extraction
    # ------------------------------------------------------------------

    async def extract_entities(self, text: str) -> list[dict[str, Any]]:
        """Send *text* through the entity extraction prompt and parse the response.

        Returns a list of dicts with keys: ``name``, ``type``, ``context``,
        ``confidence``.  Returns an empty list if the LLM fails or produces
        unparseable output.
        """
        if not text or not text.strip():
            logger.warning("extract_entities: empty text, skipping")
            return []

        # Truncate very long texts to stay within context window
        truncated = text[:12_000] if len(text) > 12_000 else text

        prompt = ENTITY_EXTRACTION_PROMPT.format(text=truncated)

        logger.info("Extracting entities from {} chars of text", len(truncated))
        raw_response = await self._router.route(
            TaskType.ENTITY_EXTRACTION,
            prompt,
        )

        entities = parse_entities(raw_response)
        logger.info("Extracted {} entities", len(entities))
        return entities

    # ------------------------------------------------------------------
    # Relation extraction
    # ------------------------------------------------------------------

    async def extract_relations(
        self,
        entities: list[dict[str, Any]],
    ) -> list[dict[str, Any]]:
        """Deduce relations between a set of entities via LLM.

        Builds a textual description of the entities and asks the LLM to
        identify connections (knows, works_with, accused, etc.).

        Returns a list of relation dicts with keys: ``source``, ``target``,
        ``type``, ``context``, ``confidence``, ``temporal``.
        """
        if not entities:
            return []

        # Build a compact text representation the LLM can reason about
        lines: list[str] = []
        for e in entities:
            ctx = e.get("context", "")
            lines.append(f"- {e['name']} ({e['type']}): {ctx}")
        entity_text = "\n".join(lines)

        prompt = RELATION_EXTRACTION_PROMPT.format(text=entity_text)

        logger.info("Extracting relations from {} entities", len(entities))
        raw_response = await self._router.route(
            TaskType.ENTITY_EXTRACTION,  # light model, same as entity extraction
            prompt,
        )

        relations = parse_relations(raw_response)
        logger.info("Extracted {} relations", len(relations))
        return relations

    # ------------------------------------------------------------------
    # Deduplication
    # ------------------------------------------------------------------

    def deduplicate_entities(
        self,
        new_entities: list[dict[str, Any]],
        existing_entities: list[dict[str, Any]],
    ) -> list[dict[str, Any]]:
        """Merge *new_entities* against *existing_entities*.

        An entity is considered a duplicate when its normalised name AND
        type match an existing entity.  Duplicates are dropped; only truly
        new entities are returned.

        Returns the de-duplicated list of **new** entity dicts (those that
        do not already exist).
        """
        if not existing_entities:
            return new_entities

        # Build a set of (normalised_name, type) for fast lookup
        existing_keys: set[tuple[str, str]] = set()
        for e in existing_entities:
            name = e.get("name", "")
            etype = e.get("entity_type", e.get("type", ""))
            existing_keys.add((self.normalize_entity_name(name), etype))

            # Also index aliases
            aliases = e.get("aliases") or []
            if isinstance(aliases, list):
                for alias in aliases:
                    existing_keys.add((self.normalize_entity_name(alias), etype))

        unique: list[dict[str, Any]] = []
        for ent in new_entities:
            name = ent.get("name", "")
            etype = ent.get("type", "")
            key = (self.normalize_entity_name(name), etype)
            if key not in existing_keys:
                unique.append(ent)
                # Add to the set so later duplicates within new_entities are
                # also caught.
                existing_keys.add(key)
            else:
                logger.debug("Duplicate entity skipped: {} ({})", name, etype)

        logger.info(
            "Deduplication: {} new, {} duplicates removed",
            len(unique),
            len(new_entities) - len(unique),
        )
        return unique

    # ------------------------------------------------------------------
    # Normalisation
    # ------------------------------------------------------------------

    @staticmethod
    def normalize_entity_name(name: str) -> str:
        """Normalize an entity name for comparison.

        - Strip leading/trailing whitespace
        - Lowercase
        - Remove diacritics / accents  (e -> e, e -> e)
        - Collapse multiple spaces into one
        """
        if not name:
            return ""
        # Lowercase
        text = name.strip().lower()
        # Decompose unicode and strip combining characters (accents)
        text = unicodedata.normalize("NFD", text)
        text = "".join(ch for ch in text if unicodedata.category(ch) != "Mn")
        # Collapse whitespace
        text = re.sub(r"\s+", " ", text).strip()
        return text
