"""
NEXUS LLM layer -- client Ollama, routeur multi-modeles, prompts, parsers.
"""

from nexus.llm.ollama_client import OllamaClient
from nexus.llm.router import LLMRouter, TaskType
from nexus.llm.parsers import (
    clean_llm_response,
    parse_entities,
    parse_json_safe,
    parse_relations,
    parse_hypothesis_score,
    parse_verification,
)

__all__ = [
    "OllamaClient",
    "LLMRouter",
    "TaskType",
    "clean_llm_response",
    "parse_entities",
    "parse_json_safe",
    "parse_relations",
    "parse_hypothesis_score",
    "parse_verification",
]
