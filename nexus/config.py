"""
NEXUS -- Configuration centralisee.

Charge les variables depuis .env et expose un singleton `settings`.
Utilise pydantic-settings pour la validation typee.
"""

from pathlib import Path

from pydantic_settings import BaseSettings


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
    # Single LLM: gemma-4-26B-A4B heretic for ALL tasks (MoE 26B, 4B active)
    #   → heretic ARA+EGA abliteration = zero refus, lossless quality
    #   → MMLU 82.6%, AIME 88.3%, 256K context, multimodal (text+vision)
    #   → stays permanently loaded in VRAM — zero model swaps
    #   → nomic-embed-text (137MB) coexists via VRAM bypass
    model_fast: str = "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m"
    model_reasoning: str = "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m"
    model_deep: str = "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m"
    model_embedding: str = "nomic-embed-text"
    model_audio: str = "faster-whisper"  # Python lib, not Ollama
    model_vision: str = "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m"
    model_vision_deep: str = "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m"

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
    auto_ingest_relevance_threshold: float = 70.0
    full_reevaluation_every_n_cycles: int = 6
    max_auto_ingest_per_cycle: int = 5
    max_new_queries_per_cycle: int = 3

    # -- RAG settings --
    rag_chunk_size: int = 512          # tokens per chunk
    rag_chunk_overlap: int = 128       # overlap tokens
    rag_top_k: int = 20               # chunks retrieved per query
    rag_min_reliability: int = 0       # minimum evidence reliability for retrieval

    # -- Autonomous loop: module activation --
    auto_osint_recon: bool = True            # Scan emails/usernames automatically
    auto_geocode: bool = True                # Geocode location entities automatically
    auto_image_analysis: bool = True         # Analyse images automatically (VLM)
    auto_forensic_analysis: bool = True      # Forensic analysis (blood, traces) auto
    auto_visual_embeddings: bool = True      # Index images in DINOv2/CLIP automatically
    auto_domain_recon: bool = True           # WHOIS/DNS recon on email domains
    auto_timeline_rebuild: bool = True       # Rebuild timeline each DECIDE phase
    auto_suspect_scoring: bool = True        # Score suspects each DECIDE phase
    auto_suspect_profile_every_n_cycles: int = 3  # LLM profile eval every N cycles
    auto_report_every_n_cycles: int = 12     # Report every 6h (12 cycles * 30min)
    auto_backup_every_n_cycles: int = 24     # Backup every 12h (24 cycles * 30min)
    auto_recon_rate_limit: float = 2.0       # Seconds between OSINT recon calls

    # -- Text processing truncation limits --
    text_truncation_short: int = 2000     # contradiction detector pair text
    text_truncation_medium: int = 3000    # testimony text, red-team facts
    text_truncation_long: int = 4000      # evidence content, analysis context
    text_truncation_summary: int = 8000   # evidence summary input
    text_truncation_verification: int = 10000  # logic verification input
    text_truncation_llm_extract: int = 12000   # LLM entity extraction input
    text_truncation_deep_analysis: int = 20000  # deep analysis dossier input

    # -- Score shift threshold (alerts for hypothesis score changes) --
    score_shift_threshold: float = 15.0   # |delta| above which an alert is created

    # -- Entity resolution --
    entity_fuzzy_threshold: int = 78      # RapidFuzz WRatio threshold for dedup
    gliner_confidence_threshold: float = 0.35  # GLiNER prediction threshold

    # -- Monitoring execution limits --
    monitoring_max_jobs_per_sweep: int = 10       # max jobs executed per 30s sweep
    monitoring_job_timeout: float = 60.0          # seconds per individual job
    monitoring_max_concurrent_jobs: int = 3       # parallel jobs per sweep

    # -- Contradiction detection limits --
    contradiction_max_evidence_pairs: int = 20    # max entity-based pairs
    contradiction_max_fallback_pairs: int = 15    # max pairs when no entity overlap
    contradiction_max_hypothesis_pairs: int = 10  # max hypothesis consistency pairs

    # -- Entity confidence thresholds (contact pattern extraction) --
    entity_confidence_high: float = 0.99    # email (deterministic regex)
    entity_confidence_medium: float = 0.95  # phone numbers, social URLs
    entity_confidence_low: float = 0.90     # social handles (@user)

    model_config = {
        "env_file": ".env",
        "env_file_encoding": "utf-8",
    }


# Singleton -- import as `from nexus.config import settings`
settings = Settings()
