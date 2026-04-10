"""
NEXUS GOV -- Identity Resolution Engine.

Reconciles politician identities across 9+ official data sources:
  - Assemblee Nationale (acteurRef like "PA842279")
  - Senat (matricule like "08061X")
  - HATVP (nom/prenom)
  - Wikidata (QID like "Q3052772")
  - PoliGraph (slug like "marine-le-pen")
  - nosdeputes.fr (slug)
  - nossenateurs.fr (slug)
  - data.gouv.fr (nom/prenom CSV)
  - La Fabrique de la Loi (no direct IDs)

Uses fuzzy matching with confidence scoring:
  >= 0.95 → auto-link (no human review)
  0.70 - 0.95 → flagged for review
  < 0.70 → no match
"""

from __future__ import annotations

import re
import unicodedata
from typing import Any, Optional

from loguru import logger

try:
    from rapidfuzz import fuzz, process
    from rapidfuzz.distance import JaroWinkler
except ImportError:
    fuzz = None
    process = None
    JaroWinkler = None  # type: ignore[assignment,misc]


# French name particles to strip for matching
_PARTICLES = {"de", "du", "des", "le", "la", "les", "d", "l"}


def normalize_name(name: str) -> str:
    """Normalize a French politician name for matching.

    - Lowercase
    - Strip accents (é→e, ç→c, etc.)
    - Remove particles (de, du, le, la, d', l')
    - Remove hyphens, extra spaces
    - Strip titles (M., Mme, Dr)
    """
    if not name or not isinstance(name, str):
        return ""
    # Lowercase
    name = name.lower().strip()
    # Remove titles
    for title in ("m.", "mme", "mme.", "dr.", "pr."):
        if name.startswith(title + " "):
            name = name[len(title):].strip()
    # Strip accents
    nfkd = unicodedata.normalize("NFKD", name)
    name = "".join(c for c in nfkd if not unicodedata.combining(c))
    # Remove d', l' prefixes
    name = re.sub(r"\b[dl]'", " ", name)
    # Remove particles
    words = name.split()
    words = [w for w in words if w not in _PARTICLES]
    # Remove hyphens within words
    name = " ".join(words).replace("-", " ")
    # Collapse spaces
    return re.sub(r"\s+", " ", name).strip()


def compute_similarity(name_a: str, name_b: str) -> float:
    """Compute similarity score (0.0-1.0) between two politician names.

    Uses RapidFuzz Jaro-Winkler similarity if available, falls back to basic
    word overlap.  Jaro-Winkler is preferred over WRatio for person names
    because it rewards matching prefixes (common in French names).
    """
    norm_a = normalize_name(name_a)
    norm_b = normalize_name(name_b)
    if not norm_a or not norm_b:
        return 0.0
    if norm_a == norm_b:
        return 1.0
    if JaroWinkler is not None:
        return JaroWinkler.similarity(norm_a, norm_b)
    # Fallback: basic word overlap
    set_a, set_b = set(norm_a.split()), set(norm_b.split())
    if not set_a or not set_b:
        return 0.0
    return len(set_a & set_b) / max(len(set_a), len(set_b))


class IdentityResolver:
    """Resolves politician identities across multiple data sources."""

    # Class-level defaults (kept for backward compatibility)
    AUTO_LINK_THRESHOLD = 0.95
    REVIEW_THRESHOLD = 0.70

    def __init__(
        self,
        gov_db: Any,
        auto_threshold: float = 0.95,
        review_threshold: float = 0.70,
    ) -> None:
        self._db = gov_db
        self.AUTO_LINK_THRESHOLD = auto_threshold
        self.REVIEW_THRESHOLD = review_threshold
        self._cache: dict[str, str] = {}  # normalized_name -> politician_id

    async def build_cache(self) -> None:
        """Pre-load all politicians for fast matching."""
        politicians = await self._db.list_politicians(limit=100_000)
        self._cache.clear()
        for p in politicians:
            norm = normalize_name(p["name"])
            if norm:
                self._cache[norm] = p["id"]
        logger.info("Identity cache built: {} politicians", len(self._cache))

    async def resolve(
        self,
        name: str,
        *,
        source: str,
        external_id: str,
    ) -> Optional[dict]:
        """Try to resolve a name to an existing politician.

        Returns {"politician_id": ..., "confidence": ..., "action": "auto"|"review"|"none"}
        or None if no match found.
        """
        if not self._cache:
            await self.build_cache()

        norm = normalize_name(name)
        if not norm:
            return None

        # Exact match (fastest)
        if norm in self._cache:
            pol_id = self._cache[norm]
            await self._link(pol_id, source, external_id, confidence=1.0)
            return {"politician_id": pol_id, "confidence": 1.0, "action": "auto"}

        # Check if external_id already linked
        existing = await self._db.find_politician_by_external_id(source, external_id)
        if existing:
            return {"politician_id": existing["id"], "confidence": 1.0, "action": "auto"}

        # Fuzzy match
        if not self._cache or JaroWinkler is None:
            return None

        candidates = list(self._cache.keys())
        matches = process.extract(
            norm, candidates, scorer=JaroWinkler.similarity, limit=3,
        )

        if not matches:
            return None

        best_name, best_score, _ = matches[0]
        # JaroWinkler.similarity already returns 0.0-1.0
        confidence = best_score
        pol_id = self._cache[best_name]

        if confidence >= self.AUTO_LINK_THRESHOLD:
            await self._link(pol_id, source, external_id, confidence=confidence)
            logger.debug(
                "Identity auto-linked: '{}' -> '{}' ({:.2f})",
                name, best_name, confidence,
            )
            return {"politician_id": pol_id, "confidence": confidence, "action": "auto"}

        if confidence >= self.REVIEW_THRESHOLD:
            # Persist review items so they are not lost
            await self._link(pol_id, source, external_id, confidence=confidence)
            logger.info(
                "Identity needs review: '{}' ~ '{}' ({:.2f})",
                name, best_name, confidence,
            )
            return {"politician_id": pol_id, "confidence": confidence, "action": "review"}

        return None

    async def resolve_batch(
        self,
        entries: list[dict],
        *,
        source: str,
        name_key: str = "name",
        id_key: str = "id",
    ) -> dict[str, Any]:
        """Resolve a batch of entries.

        Returns a dict with:
          - ``resolved``: ``{external_id: politician_id}`` for auto-linked items
          - ``review``: list of ``{ext_id, politician_id, confidence, source}``
          - ``stats``: ``{auto, review, unmatched}`` counts
        """
        if not self._cache:
            await self.build_cache()

        resolved: dict[str, str] = {}
        review_items: list[dict[str, Any]] = []
        auto_count = 0
        review_count = 0

        for entry in entries:
            name = entry.get(name_key, "")
            ext_id = entry.get(id_key, "")
            if not name or not ext_id:
                continue

            result = await self.resolve(name, source=source, external_id=str(ext_id))
            if result and result["action"] == "auto":
                resolved[str(ext_id)] = result["politician_id"]
                auto_count += 1
            elif result and result["action"] == "review":
                review_items.append({
                    "ext_id": str(ext_id),
                    "politician_id": result["politician_id"],
                    "confidence": result["confidence"],
                    "source": source,
                })
                review_count += 1

        unmatched = len(entries) - auto_count - review_count
        logger.info(
            "Batch resolve ({}): {} auto-linked, {} need review, {} unmatched",
            source, auto_count, review_count, unmatched,
        )
        return {
            "resolved": resolved,
            "review": review_items,
            "stats": {"auto": auto_count, "review": review_count, "unmatched": unmatched},
        }

    async def _link(
        self, politician_id: str, source: str, external_id: str, confidence: float
    ) -> None:
        """Store an external ID link in the database."""
        try:
            existing = await self._db.get_external_ids_by_politician(politician_id)
            for eid in existing:
                if eid.get("source") == source and eid.get("external_id") == external_id:
                    return  # Already linked

            await self._db.create_external_id(
                politician_id=politician_id,
                source=source,
                external_id=external_id,
                confidence=confidence,
            )
        except Exception as exc:
            logger.debug("Failed to link external ID: {}", exc)
