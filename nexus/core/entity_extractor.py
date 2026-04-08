"""
NEXUS -- Entity Extractor.

Hybrid NER extraction:
1. GLiNER (fast, CPU, specialized) for primary entity extraction
2. LLM fallback (gemma4:e4b) for relation extraction and edge cases

GLiNER runs on CPU in ~0.08s per text, freeing GPU for analysis models.
"""

from __future__ import annotations

import re
import unicodedata
from typing import Any, Optional

from loguru import logger

from nexus.llm.parsers import parse_entities, parse_relations
from nexus.llm.prompts import (
    ENTITY_EXTRACTION_PROMPT,
    RELATION_EXTRACTION_PROMPT,
)
from nexus.llm.router import LLMRouter, TaskType


# GLiNER entity labels mapped to NEXUS entity types
_GLINER_LABELS = {
    "personne": "person",
    "lieu": "location",
    "adresse": "location",
    "code postal": "location",
    "vehicule": "vehicle",
    "immatriculation": "vehicle",
    "telephone": "phone",
    "email": "email",
    "date": "date",
    "heure": "date",
    "numero de procedure": "other",
    "organisation": "organization",
    "entreprise": "organization",
    "arme": "weapon",
    "drogue": "drug",
    "somme d'argent": "money",
    "compte bancaire": "account",
    "adresse ip": "ip",
}

_GLINER_LABEL_DESCRIPTIONS = list(_GLINER_LABELS.keys())


class EntityExtractor:
    """Hybrid NER: GLiNER (primary, CPU) + LLM (fallback, relations)."""

    def __init__(self, router: LLMRouter, gliner_model=None) -> None:
        self._router = router
        self._gliner = gliner_model
        self._gliner_loaded = gliner_model is not None
        self._gliner_failed = False

    def preload(self) -> bool:
        """Eagerly load GLiNER model. Returns True if successful."""
        self._load_gliner()
        return self._gliner_loaded

    def _load_gliner(self):
        """Lazy-load GLiNER model on first use."""
        if self._gliner_loaded or self._gliner_failed:
            return
        try:
            from gliner import GLiNER
            self._gliner = GLiNER.from_pretrained("urchade/gliner_multi-v2.1")
            self._gliner_loaded = True
            logger.info("GLiNER model loaded (CPU, 205M params)")
        except ImportError:
            self._gliner_failed = True
            logger.warning("GLiNER not installed (pip install gliner) — falling back to LLM extraction")
        except Exception as exc:
            self._gliner_failed = True
            logger.error("GLiNER load failed: {} — falling back to LLM", exc)

    # ------------------------------------------------------------------
    # Entity extraction (GLiNER primary, LLM fallback)
    # ------------------------------------------------------------------

    # Regex patterns for contact info (deterministic, <1ms, no model)
    _EMAIL_RE = re.compile(r'\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Z|a-z]{2,}\b')
    _SOCIAL_HANDLE_RE = re.compile(r'(?<!\w)@([A-Za-z0-9_]{2,30})(?!\w)')
    _SOCIAL_URL_RE = re.compile(
        r'https?://(?:www\.)?'
        r'(?:facebook\.com|instagram\.com|twitter\.com|x\.com|linkedin\.com|tiktok\.com|youtube\.com|vk\.com)'
        r'/[A-Za-z0-9_.%/\-]+'
    )

    def _extract_contact_patterns(self, text: str) -> list[dict[str, Any]]:
        """Extract emails, phones, social handles via regex + phonenumbers."""
        entities: list[dict[str, Any]] = []

        for email in set(self._EMAIL_RE.findall(text)):
            entities.append({"name": email, "type": "email",
                            "context": self._get_context(text, email), "confidence": 0.99})

        try:
            import phonenumbers
            for match in phonenumbers.PhoneNumberMatcher(text, "FR"):
                formatted = phonenumbers.format_number(match.number, phonenumbers.PhoneNumberFormat.E164)
                entities.append({"name": formatted, "type": "phone",
                                "context": self._get_context(text, match.raw_string), "confidence": 0.95})
        except ImportError:
            pass

        for handle in set(self._SOCIAL_HANDLE_RE.findall(text)):
            entities.append({"name": f"@{handle}", "type": "social_handle",
                            "context": self._get_context(text, f"@{handle}"), "confidence": 0.90})

        for url in set(self._SOCIAL_URL_RE.findall(text)):
            entities.append({"name": url, "type": "social_url",
                            "context": self._get_context(text, url), "confidence": 0.95})

        if entities:
            logger.info("Contact patterns extracted: {} items", len(entities))
        return entities

    async def extract_entities(self, text: str) -> list[dict[str, Any]]:
        """Extract named entities from text.

        1. Pattern extraction (emails, phones, handles — deterministic, instant)
        2. GLiNER NER (persons, locations, dates — CPU, ~0.08s)
        3. LLM fallback if GLiNER unavailable
        """
        if not text or not text.strip():
            return []

        # 1. Contact pattern extraction (regex + phonenumbers)
        pattern_entities = self._extract_contact_patterns(text)

        # 2. Try GLiNER
        self._load_gliner()
        if self._gliner is not None:
            ner_entities = self._extract_gliner(text)
        else:
            ner_entities = await self._extract_llm(text)

        return pattern_entities + ner_entities

    def _extract_gliner(self, text: str) -> list[dict[str, Any]]:
        """Extract entities using GLiNER (CPU, ~0.08s)."""
        try:
            # GLiNER works best with chunks < 1000 chars
            chunks = self._split_for_gliner(text, max_chars=1500)
            all_entities: list[dict[str, Any]] = []
            seen: set[tuple[str, str]] = set()

            for chunk in chunks:
                raw = self._gliner.predict_entities(
                    chunk, _GLINER_LABEL_DESCRIPTIONS, threshold=0.35
                )
                for ent in raw:
                    name = ent["text"].strip()
                    label = ent["label"]
                    nexus_type = _GLINER_LABELS.get(label, "other")
                    score = ent["score"]

                    # Skip very short or generic entities
                    if len(name) < 2:
                        continue
                    # Skip time-only values detected as "date" (e.g. "0 h 21", "14h30")
                    if nexus_type == "date":
                        import re as _re
                        if _re.match(r'^(?:\d{1,2}\s*h(?:eure)?(?:\s*\d{1,2})?|\d{1,2}[h:]\d{0,2})$', name, _re.IGNORECASE):
                            continue
                    # Skip generic words that aren't real entities
                    GENERIC_WORDS = {
                        "telephone", "portable", "vehicule", "voiture", "victime",
                        "suspect", "temoin", "enqueteur", "homme", "femme",
                        "personne", "individu", "corps", "sang", "adn",
                        "police", "gendarmerie", "pompiers", "operateur",
                        "madame", "monsieur",
                    }
                    if name.lower().strip() in GENERIC_WORDS:
                        continue

                    # Deduplicate within this extraction
                    key = (self.normalize_entity_name(name), nexus_type)
                    if key in seen:
                        continue
                    seen.add(key)

                    all_entities.append({
                        "name": name,
                        "type": nexus_type,
                        "context": self._get_context(text, name),
                        "confidence": round(score, 2),
                    })

            logger.info("GLiNER extracted {} entities from {} chars", len(all_entities), len(text))
            return all_entities

        except Exception as exc:
            logger.error("GLiNER extraction failed: {}", exc)
            return []

    async def _extract_llm(self, text: str) -> list[dict[str, Any]]:
        """Fallback: extract entities using LLM (gemma4:e4b)."""
        truncated = text[:12_000] if len(text) > 12_000 else text
        prompt = ENTITY_EXTRACTION_PROMPT.format(text=truncated)

        logger.info("LLM extracting entities from {} chars", len(truncated))
        raw_response = await self._router.route(TaskType.ENTITY_EXTRACTION, prompt)
        entities = parse_entities(raw_response)
        logger.info("LLM extracted {} entities", len(entities))
        return entities

    @staticmethod
    def _split_for_gliner(text: str, max_chars: int = 1500) -> list[str]:
        """Split text into chunks for GLiNER (works best with shorter texts)."""
        if len(text) <= max_chars:
            return [text]
        chunks = []
        paragraphs = text.split("\n\n")
        current = ""
        for para in paragraphs:
            if len(current) + len(para) + 2 > max_chars:
                if current:
                    chunks.append(current)
                current = para
            else:
                current = current + "\n\n" + para if current else para
        if current:
            chunks.append(current)
        return chunks if chunks else [text[:max_chars]]

    @staticmethod
    def _get_context(text: str, entity_name: str, window: int = 100) -> str:
        """Extract context window around an entity mention."""
        idx = text.lower().find(entity_name.lower())
        if idx < 0:
            return ""
        start = max(0, idx - window)
        end = min(len(text), idx + len(entity_name) + window)
        return text[start:end].strip()

    # ------------------------------------------------------------------
    # Relation extraction (still uses LLM — needs reasoning)
    # ------------------------------------------------------------------

    async def extract_relations(
        self, entities: list[dict[str, Any]]
    ) -> list[dict[str, Any]]:
        """Deduce relations between entities via LLM.

        Relations require reasoning, so we use the LLM (not GLiNER).
        """
        if not entities:
            return []

        lines = [f"- {e['name']} ({e['type']}): {e.get('context', '')}" for e in entities]
        entity_text = "\n".join(lines)
        prompt = RELATION_EXTRACTION_PROMPT.format(text=entity_text)

        logger.info("Extracting relations from {} entities", len(entities))
        raw_response = await self._router.route(TaskType.ENTITY_EXTRACTION, prompt)
        relations = parse_relations(raw_response)
        logger.info("Extracted {} relations", len(relations))
        return relations

    # ------------------------------------------------------------------
    # Deduplication
    # ------------------------------------------------------------------

    # Fuzzy threshold for entity resolution (0-100)
    FUZZY_THRESHOLD = 78

    def deduplicate_entities(
        self,
        new_entities: list[dict[str, Any]],
        existing_entities: list[dict[str, Any]],
    ) -> list[dict[str, Any]]:
        """Entity resolution: merge duplicates using RapidFuzz.

        Uses Jaro-Winkler weighted ratio for fuzzy name matching.
        Works for all entity types, not just persons.
        """
        from rapidfuzz import fuzz

        # Build lookup of existing entities by type
        existing_by_type: dict[str, list[str]] = {}
        for e in (existing_entities or []):
            etype = e.get("entity_type", e.get("type", ""))
            norm = self.normalize_entity_name(e.get("name", ""))
            existing_by_type.setdefault(etype, []).append(norm)
            for alias in (e.get("aliases") or []):
                if isinstance(alias, str):
                    existing_by_type.setdefault(etype, []).append(self.normalize_entity_name(alias))

        unique = []
        seen_norms: dict[str, list[str]] = {}  # type → [norm_names]

        for ent in new_entities:
            name = ent.get("name", "")
            etype = ent.get("type", "")
            norm = self.normalize_entity_name(name)

            if not norm:
                continue

            # Check against existing entities
            is_dup = False
            for existing_norm in existing_by_type.get(etype, []):
                score = fuzz.WRatio(norm, existing_norm)
                if score >= self.FUZZY_THRESHOLD:
                    is_dup = True
                    break

            if is_dup:
                continue

            # Check against already-accepted new entities
            for accepted_norm in seen_norms.get(etype, []):
                score = fuzz.WRatio(norm, accepted_norm)
                if score >= self.FUZZY_THRESHOLD:
                    is_dup = True
                    break

            if is_dup:
                continue

            unique.append(ent)
            seen_norms.setdefault(etype, []).append(norm)

        logger.info("Entity resolution: {} in, {} unique, {} merged",
                     len(new_entities), len(unique), len(new_entities) - len(unique))
        return unique

    # ------------------------------------------------------------------
    # Normalisation
    # ------------------------------------------------------------------

    @staticmethod
    def normalize_entity_name(name: str) -> str:
        """Normalize entity name for comparison."""
        if not name:
            return ""
        text = name.strip().lower()
        text = unicodedata.normalize("NFD", text)
        text = "".join(ch for ch in text if unicodedata.category(ch) != "Mn")
        text = re.sub(r"\s+", " ", text).strip()
        return text
