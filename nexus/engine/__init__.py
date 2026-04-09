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
from nexus.db.sqlite_db import Database, get_db, init_db

__all__ = [
    "EventBus", "ReactiveWorker", "EventType", "NexusEvent",
    "VRAMScheduler", "LLMRouter", "TaskType", "OllamaClient",
    "Database", "get_db", "init_db",
]
