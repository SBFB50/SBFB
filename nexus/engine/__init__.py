"""
NEXUS Engine — shared infrastructure for all NEXUS modules.

Re-exports core components used by both cold case and government modules:
EventBus, ReactiveWorker, VRAMScheduler, LLMRouter, OllamaClient,
database helpers, ChromaDB, Neo4j.
"""

from nexus.events.bus import EventBus
from nexus.events.worker import ReactiveWorker
from nexus.events.types import EventType, NexusEvent
from nexus.events.vram_scheduler import VRAMScheduler
from nexus.llm.router import LLMRouter, TaskType
from nexus.llm.ollama_client import OllamaClient
from nexus.llm.parsers import parse_json_safe
from nexus.llm.prompts import POLITICAL_CONTRADICTION_PROMPT
from nexus.db.sqlite_db import (
    Database, get_db, init_db,
    _new_id, _now_iso, _row_to_dict, _dict_with_json_fields,
    _json_dumps, _json_loads,
)

__all__ = [
    "EventBus", "ReactiveWorker", "EventType", "NexusEvent",
    "VRAMScheduler", "LLMRouter", "TaskType", "OllamaClient",
    "Database", "get_db", "init_db",
    "_new_id", "_now_iso", "_row_to_dict", "_dict_with_json_fields",
    "_json_dumps", "_json_loads",
    "parse_json_safe", "POLITICAL_CONTRADICTION_PROMPT",
]
