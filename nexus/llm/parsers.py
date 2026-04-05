"""
NEXUS -- Parsing des reponses LLM.

Les LLMs ne produisent pas toujours du JSON parfait.  Ce module fournit
des fonctions robustes pour extraire et valider les structures attendues
meme quand la reponse contient du texte supplementaire, des blocs
Markdown, des balises ``<think>`` ou du JSON tronque.
"""

from __future__ import annotations

import json
import re
from typing import Any

from loguru import logger


# =====================================================================
# Generic helpers
# =====================================================================

def clean_llm_response(response: str) -> str:
    """Remove common LLM artifacts from a raw response.

    Handles:
    - ``<think>...</think>`` blocks (deepseek-r1 chain-of-thought)
    - Markdown code fences (```json ... ```)
    - Leading/trailing whitespace
    """
    text = response

    # Strip <think>...</think> blocks (possibly multiline)
    text = re.sub(r"<think>.*?</think>", "", text, flags=re.DOTALL)

    # Strip markdown code fences — keep only the inner content
    # Match ```json ... ``` or ``` ... ```
    fence_match = re.search(
        r"```(?:json)?\s*\n?(.*?)```", text, flags=re.DOTALL
    )
    if fence_match:
        text = fence_match.group(1)

    return text.strip()


def parse_json_safe(response: str) -> dict | None:
    """Extract the first valid JSON object from an LLM response.

    Strategy (in order):
    1. Try parsing the cleaned response directly.
    2. Look for the first ``{...}`` block and parse it.
    3. Attempt to repair truncated JSON (missing closing braces).
    4. Return ``None`` if all strategies fail.
    """
    cleaned = clean_llm_response(response)

    # Strategy 1: direct parse
    try:
        return json.loads(cleaned)
    except json.JSONDecodeError:
        pass

    # Strategy 2: locate the outermost { ... } block
    start = cleaned.find("{")
    if start != -1:
        # Walk forward to find matching close brace
        depth = 0
        end = start
        for i in range(start, len(cleaned)):
            if cleaned[i] == "{":
                depth += 1
            elif cleaned[i] == "}":
                depth -= 1
                if depth == 0:
                    end = i
                    break
        candidate = cleaned[start : end + 1]
        try:
            return json.loads(candidate)
        except json.JSONDecodeError:
            pass

    # Strategy 3: repair truncated JSON (add missing braces/brackets)
    if start != -1:
        candidate = cleaned[start:]
        candidate = _repair_json(candidate)
        try:
            return json.loads(candidate)
        except json.JSONDecodeError:
            pass

    logger.warning(
        "parse_json_safe: could not extract valid JSON (response_len={})",
        len(response),
    )
    return None


def _repair_json(text: str) -> str:
    """Attempt to close unclosed braces/brackets in truncated JSON."""
    open_braces = text.count("{") - text.count("}")
    open_brackets = text.count("[") - text.count("]")

    # Remove trailing comma if present (common truncation artefact)
    text = re.sub(r",\s*$", "", text)

    text += "]" * max(open_brackets, 0)
    text += "}" * max(open_braces, 0)
    return text


# =====================================================================
# Domain-specific parsers
# =====================================================================

def parse_entities(response: str) -> list[dict[str, Any]]:
    """Parse the entity extraction response.

    Expected structure::

        {"entities": [{"name", "type", "context", "confidence"}, ...]}

    Returns a (possibly empty) list of entity dicts.
    """
    data = parse_json_safe(response)
    if data is None:
        return []

    entities = data.get("entities", [])
    if not isinstance(entities, list):
        logger.warning("parse_entities: 'entities' is not a list")
        return []

    valid = []
    for e in entities:
        if not isinstance(e, dict):
            continue
        # Ensure required fields exist
        if "name" not in e or "type" not in e:
            continue
        # Normalise confidence
        try:
            e["confidence"] = float(e.get("confidence", 0.5))
        except (ValueError, TypeError):
            e["confidence"] = 0.5
        e.setdefault("context", "")
        valid.append(e)

    logger.debug("parse_entities: extracted {} entities", len(valid))
    return valid


def parse_relations(response: str) -> list[dict[str, Any]]:
    """Parse the relation extraction response.

    Expected structure::

        {"relations": [{"source", "target", "type", "context",
                         "confidence", "temporal"}, ...]}

    Returns a (possibly empty) list of relation dicts.
    """
    data = parse_json_safe(response)
    if data is None:
        return []

    relations = data.get("relations", [])
    if not isinstance(relations, list):
        logger.warning("parse_relations: 'relations' is not a list")
        return []

    valid = []
    for r in relations:
        if not isinstance(r, dict):
            continue
        if not all(k in r for k in ("source", "target", "type")):
            continue
        try:
            r["confidence"] = float(r.get("confidence", 0.5))
        except (ValueError, TypeError):
            r["confidence"] = 0.5
        r.setdefault("context", "")
        r.setdefault("temporal", None)
        valid.append(r)

    logger.debug("parse_relations: extracted {} relations", len(valid))
    return valid


def parse_hypothesis_score(response: str) -> dict[str, Any]:
    """Parse a hypothesis re-evaluation response.

    Expected structure::

        {
          "hypothesis_id": str,
          "previous_score": float,
          "new_score": float,
          "delta": float,
          "supporting": [...],
          "contradicting": [...],
          "reasoning": str,
          "status": str
        }

    Returns a dict with defaults for missing fields, or an empty dict on
    complete failure.
    """
    data = parse_json_safe(response)
    if data is None:
        return {}

    # Normalise numeric scores
    for key in ("previous_score", "new_score", "delta"):
        try:
            data[key] = float(data.get(key, 0.0))
        except (ValueError, TypeError):
            data[key] = 0.0

    data.setdefault("hypothesis_id", "")
    data.setdefault("supporting", [])
    data.setdefault("contradicting", [])
    data.setdefault("reasoning", "")
    data.setdefault("status", "active")

    return data


def parse_verification(response: str) -> dict[str, Any]:
    """Parse a logic-verification response.

    Expected structure::

        {
          "premises": [{"text", "explicit": bool, "valid": bool}, ...],
          "conclusion": str,
          "fallacies": [{"type", "description"}, ...],
          "logical_validity": bool,
          "soundness_score": float,
          "critique": str
        }

    Returns a dict with defaults for missing fields, or an empty dict on
    complete failure.
    """
    data = parse_json_safe(response)
    if data is None:
        return {}

    data.setdefault("premises", [])
    data.setdefault("conclusion", "")
    data.setdefault("fallacies", [])
    data.setdefault("logical_validity", False)
    data.setdefault("critique", "")

    try:
        data["soundness_score"] = float(data.get("soundness_score", 0.0))
    except (ValueError, TypeError):
        data["soundness_score"] = 0.0

    return data
