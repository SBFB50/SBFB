"""
NEXUS -- Configuration centralisee.

Charge les variables depuis .env et expose un singleton `settings`.
Utilise pydantic-settings pour la validation typee.
"""

from pathlib import Path
from typing import Dict

from pydantic_settings import BaseSettings


# ---------------------------------------------------------------------------
# Model routing constants
# ---------------------------------------------------------------------------
# Maps task types to the appropriate Ollama model.
# Used by nexus.llm.router to dispatch requests.

MODEL_ROUTING: Dict[str, str] = {
    # Mechanical tasks: entity extraction, filtering, JSON formatting
    "extract": "gemma4:e4b",
    "filter": "gemma4:e4b",
    "format": "gemma4:e4b",
    "summarize": "gemma4:e4b",
    # Chain-of-thought reasoning: contradiction detection, logic verification
    "reason": "huihui_ai/deepseek-r1-abliterated:14b",
    "verify": "huihui_ai/deepseek-r1-abliterated:14b",
    "contradict": "huihui_ai/deepseek-r1-abliterated:14b",
    # Deep analysis: hypotheses, scoring, reports
    "analyze": "nexus",
    "hypothesize": "nexus",
    "report": "nexus",
    # Embeddings
    "embed": "nomic-embed-text",
    # Audio / video transcription
    "transcribe": "voxtral-mini:4b",
}


# ---------------------------------------------------------------------------
# Settings
# ---------------------------------------------------------------------------

class Settings(BaseSettings):
    """Typed application settings loaded from environment / .env file."""

    # -- FastAPI --
    nexus_host: str = "0.0.0.0"
    nexus_port: int = 8000
    nexus_debug: bool = True

    # -- Ollama --
    ollama_base_url: str = "http://localhost:11434"

    # -- Models (overridable via env) --
    model_fast: str = "gemma4:e4b"
    model_reasoning: str = "huihui_ai/deepseek-r1-abliterated:14b"
    model_deep: str = "nexus"
    model_embedding: str = "nomic-embed-text"
    model_audio: str = "voxtral-mini:4b"
    model_vision: str = "gemma4:e4b"  # VLM pour analyse de photos (supporte images)
    model_vision_deep: str = "qwen3-vl:8b"  # VLM avance pour analyse approfondie

    # -- Neo4j --
    neo4j_uri: str = "bolt://localhost:7687"
    neo4j_user: str = "neo4j"
    neo4j_password: str = "nexus2026"

    # -- ChromaDB --
    chroma_host: str = "localhost"
    chroma_port: int = 8100

    # -- SearXNG (clearweb) --
    searxng_url: str = "http://localhost:8888"

    # -- Robin (dark web / Tor) --
    robin_url: str = "http://localhost:9090"

    # -- Storage paths --
    data_dir: Path = Path("./data")
    upload_dir: Path = Path("./data/uploads")
    sqlite_path: Path = Path("./data/nexus.db")

    # -- Monitoring intervals (seconds) --
    clearweb_interval: int = 6 * 3600    # 6 hours
    darkweb_interval: int = 24 * 3600    # 24 hours

    # -- Autonomous investigation loop --
    investigation_cycle_minutes: int = 30
    auto_ingest_relevance_threshold: float = 50.0
    full_reevaluation_every_n_cycles: int = 6
    max_auto_ingest_per_cycle: int = 5
    max_new_queries_per_cycle: int = 3

    # -- Autonomous loop: module activation --
    auto_osint_recon: bool = True            # Scan emails/usernames automatically
    auto_geocode: bool = True                # Geocode location entities automatically
    auto_image_analysis: bool = True         # Analyse images automatically (VLM)
    auto_forensic_analysis: bool = True      # Forensic analysis (blood, traces) auto
    auto_visual_embeddings: bool = True      # Index images in DINOv2/CLIP automatically
    auto_domain_recon: bool = True           # WHOIS/DNS recon on email domains
    auto_timeline_rebuild: bool = True       # Rebuild timeline each DECIDE phase
    auto_report_every_n_cycles: int = 12     # Report every 6h (12 cycles * 30min)
    auto_backup_every_n_cycles: int = 24     # Backup every 12h (24 cycles * 30min)
    auto_recon_rate_limit: float = 2.0       # Seconds between OSINT recon calls

    model_config = {
        "env_file": ".env",
        "env_file_encoding": "utf-8",
    }


# Singleton -- import as `from nexus.config import settings`
settings = Settings()
