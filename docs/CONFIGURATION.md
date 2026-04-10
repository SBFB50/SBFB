# NEXUS -- Configuration Reference

Complete reference for every configurable setting in the NEXUS system.

Settings are loaded by `pydantic-settings` from environment variables and/or a `.env` file at project root. The singleton `settings` object is importable as:

```python
from nexus.config import settings
```

Worker nodes use a separate JSON config stored at `~/.nexus-worker/config.json`.

---

## 1. Server Settings (`nexus/config.py` -- `Settings` class)

### FastAPI

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `nexus_host` | `str` | `"0.0.0.0"` | Bind address for the FastAPI server | `NEXUS_HOST` |
| `nexus_port` | `int` | `8000` | Port for the FastAPI server | `NEXUS_PORT` |
| `nexus_debug` | `bool` | `True` | Enable debug mode (verbose logging, auto-reload) | `NEXUS_DEBUG` |

### Ollama

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `ollama_base_url` | `str` | `"http://localhost:11434"` | Ollama API base URL | `OLLAMA_BASE_URL` |

### Models

All model slots point to the same MoE 26B model by default (zero VRAM swap). Override individual slots via env vars.

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `model_fast` | `str` | `"juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m"` | Fast inference model (summaries, extraction) | `MODEL_FAST` |
| `model_reasoning` | `str` | `"juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m"` | Reasoning model (analysis, hypotheses) | `MODEL_REASONING` |
| `model_deep` | `str` | `"juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m"` | Deep analysis model (complex reasoning) | `MODEL_DEEP` |
| `model_embedding` | `str` | `"nomic-embed-text"` | Embedding model for RAG (137MB, VRAM bypass) | `MODEL_EMBEDDING` |
| `model_audio` | `str` | `"faster-whisper"` | Audio transcription (Python lib, not Ollama) | `MODEL_AUDIO` |
| `model_vision` | `str` | `"juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m"` | Vision model (image analysis) | `MODEL_VISION` |
| `model_vision_deep` | `str` | `"juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m"` | Deep vision model (detailed image analysis) | `MODEL_VISION_DEEP` |

### Neo4j

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `neo4j_uri` | `str` | `"bolt://localhost:7687"` | Neo4j Bolt protocol URI | `NEO4J_URI` |
| `neo4j_user` | `str` | `"neo4j"` | Neo4j username | `NEO4J_USER` |
| `neo4j_password` | `str` | `""` | Neo4j password (set in `.env`!) | `NEO4J_PASSWORD` |

### ChromaDB

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `chroma_host` | `str` | `"localhost"` | ChromaDB server hostname | `CHROMA_HOST` |
| `chroma_port` | `int` | `8100` | ChromaDB server port | `CHROMA_PORT` |

### Search Engines

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `searxng_url` | `str` | `"http://localhost:8888"` | SearXNG clearweb search URL | `SEARXNG_URL` |
| `robin_url` | `str` | `"http://localhost:9090"` | Robin dark web / Tor search URL | `ROBIN_URL` |

### Storage Paths

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `data_dir` | `Path` | `"./data"` | Root data directory | `DATA_DIR` |
| `upload_dir` | `Path` | `"./data/uploads"` | Uploaded evidence files | `UPLOAD_DIR` |
| `sqlite_path` | `Path` | `"./data/nexus.db"` | SQLite database file (FTS5 + WAL) | `SQLITE_PATH` |

### Monitoring Intervals

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `clearweb_interval` | `int` | `21600` (6h) | Seconds between clearweb monitoring sweeps | `CLEARWEB_INTERVAL` |
| `darkweb_interval` | `int` | `86400` (24h) | Seconds between dark web monitoring sweeps | `DARKWEB_INTERVAL` |

### Autonomous Investigation Loop

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `investigation_cycle_minutes` | `int` | `30` | Minutes per autonomous investigation cycle | `INVESTIGATION_CYCLE_MINUTES` |
| `auto_ingest_relevance_threshold` | `float` | `70.0` | Minimum relevance score to auto-ingest evidence | `AUTO_INGEST_RELEVANCE_THRESHOLD` |
| `full_reevaluation_every_n_cycles` | `int` | `6` | Full re-evaluation every N cycles | `FULL_REEVALUATION_EVERY_N_CYCLES` |
| `max_auto_ingest_per_cycle` | `int` | `5` | Max evidence pieces auto-ingested per cycle | `MAX_AUTO_INGEST_PER_CYCLE` |
| `max_new_queries_per_cycle` | `int` | `3` | Max new search queries generated per cycle | `MAX_NEW_QUERIES_PER_CYCLE` |

### RAG Settings

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `rag_chunk_size` | `int` | `512` | Tokens per chunk for embedding | `RAG_CHUNK_SIZE` |
| `rag_chunk_overlap` | `int` | `128` | Overlap tokens between chunks | `RAG_CHUNK_OVERLAP` |
| `rag_top_k` | `int` | `20` | Number of chunks retrieved per RAG query | `RAG_TOP_K` |
| `rag_min_reliability` | `int` | `0` | Minimum evidence reliability score for retrieval | `RAG_MIN_RELIABILITY` |

### Autonomous Module Activation

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `auto_osint_recon` | `bool` | `True` | Scan emails/usernames automatically | `AUTO_OSINT_RECON` |
| `auto_geocode` | `bool` | `True` | Geocode location entities automatically | `AUTO_GEOCODE` |
| `auto_image_analysis` | `bool` | `True` | Analyse images automatically (VLM) | `AUTO_IMAGE_ANALYSIS` |
| `auto_forensic_analysis` | `bool` | `True` | Forensic analysis (blood, traces) auto | `AUTO_FORENSIC_ANALYSIS` |
| `auto_visual_embeddings` | `bool` | `True` | Index images in DINOv2/CLIP automatically | `AUTO_VISUAL_EMBEDDINGS` |
| `auto_domain_recon` | `bool` | `True` | WHOIS/DNS recon on email domains | `AUTO_DOMAIN_RECON` |
| `auto_timeline_rebuild` | `bool` | `True` | Rebuild timeline each DECIDE phase | `AUTO_TIMELINE_REBUILD` |
| `auto_suspect_scoring` | `bool` | `True` | Score suspects each DECIDE phase | `AUTO_SUSPECT_SCORING` |
| `auto_suspect_profile_every_n_cycles` | `int` | `3` | LLM profile evaluation every N cycles | `AUTO_SUSPECT_PROFILE_EVERY_N_CYCLES` |
| `auto_report_every_n_cycles` | `int` | `12` | Generate report every N cycles (12 x 30min = 6h) | `AUTO_REPORT_EVERY_N_CYCLES` |
| `auto_backup_every_n_cycles` | `int` | `24` | Database backup every N cycles (24 x 30min = 12h) | `AUTO_BACKUP_EVERY_N_CYCLES` |
| `auto_recon_rate_limit` | `float` | `2.0` | Seconds between OSINT recon API calls | `AUTO_RECON_RATE_LIMIT` |

### Text Truncation Limits

Controls maximum character count passed to the LLM for different task types.

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `text_truncation_short` | `int` | `2000` | Contradiction detector pair text | `TEXT_TRUNCATION_SHORT` |
| `text_truncation_medium` | `int` | `3000` | Testimony text, red-team facts | `TEXT_TRUNCATION_MEDIUM` |
| `text_truncation_long` | `int` | `4000` | Evidence content, analysis context | `TEXT_TRUNCATION_LONG` |
| `text_truncation_summary` | `int` | `8000` | Evidence summary input | `TEXT_TRUNCATION_SUMMARY` |
| `text_truncation_verification` | `int` | `10000` | Logic verification input | `TEXT_TRUNCATION_VERIFICATION` |
| `text_truncation_llm_extract` | `int` | `12000` | LLM entity extraction input | `TEXT_TRUNCATION_LLM_EXTRACT` |
| `text_truncation_deep_analysis` | `int` | `20000` | Deep analysis dossier input | `TEXT_TRUNCATION_DEEP_ANALYSIS` |

### Scoring and Entity Resolution

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `score_shift_threshold` | `float` | `15.0` | Hypothesis score delta that triggers an alert | `SCORE_SHIFT_THRESHOLD` |
| `entity_fuzzy_threshold` | `int` | `78` | RapidFuzz WRatio threshold for entity dedup | `ENTITY_FUZZY_THRESHOLD` |
| `gliner_confidence_threshold` | `float` | `0.35` | GLiNER NER prediction confidence threshold | `GLINER_CONFIDENCE_THRESHOLD` |

### Monitoring Execution Limits

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `monitoring_max_jobs_per_sweep` | `int` | `10` | Max jobs executed per 30s sweep | `MONITORING_MAX_JOBS_PER_SWEEP` |
| `monitoring_job_timeout` | `float` | `60.0` | Seconds per individual monitoring job | `MONITORING_JOB_TIMEOUT` |
| `monitoring_max_concurrent_jobs` | `int` | `3` | Parallel jobs per monitoring sweep | `MONITORING_MAX_CONCURRENT_JOBS` |

### Contradiction Detection Limits

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `contradiction_max_evidence_pairs` | `int` | `20` | Max entity-based evidence pairs to check | `CONTRADICTION_MAX_EVIDENCE_PAIRS` |
| `contradiction_max_fallback_pairs` | `int` | `15` | Max pairs when no entity overlap found | `CONTRADICTION_MAX_FALLBACK_PAIRS` |
| `contradiction_max_hypothesis_pairs` | `int` | `10` | Max hypothesis consistency pairs | `CONTRADICTION_MAX_HYPOTHESIS_PAIRS` |

### Entity Confidence Thresholds

Confidence scores assigned during contact pattern extraction.

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `entity_confidence_high` | `float` | `0.99` | Email addresses (deterministic regex) | `ENTITY_CONFIDENCE_HIGH` |
| `entity_confidence_medium` | `float` | `0.95` | Phone numbers, social URLs | `ENTITY_CONFIDENCE_MEDIUM` |
| `entity_confidence_low` | `float` | `0.90` | Social handles (@user) | `ENTITY_CONFIDENCE_LOW` |

### Distributed GPU Computing

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `compute_enabled` | `bool` | `True` | Auto-start compute system on boot | `COMPUTE_ENABLED` |
| `compute_heartbeat_timeout` | `int` | `90` | Seconds before marking a GPU node offline | `COMPUTE_HEARTBEAT_TIMEOUT` |
| `compute_task_default_timeout` | `int` | `300` | Default task timeout in seconds | `COMPUTE_TASK_DEFAULT_TIMEOUT` |
| `compute_reaper_interval` | `int` | `60` | Task reaper sweep interval in seconds | `COMPUTE_REAPER_INTERVAL` |
| `compute_spot_check_rate` | `float` | `0.05` | Fraction of tasks to spot-check (5%) | `COMPUTE_SPOT_CHECK_RATE` |
| `compute_max_retries` | `int` | `3` | Max task retries before permanent failure | `COMPUTE_MAX_RETRIES` |
| `compute_rate_limit_per_minute` | `int` | `100` | Max requests per node per minute | `COMPUTE_RATE_LIMIT_PER_MINUTE` |

### exo Distributed Mode (Phase 4)

Multi-GPU LAN inference via exo's OpenAI-compatible endpoint.

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `exo_enabled` | `bool` | `False` | Enable exo distributed inference | `EXO_ENABLED` |
| `exo_url` | `str` | `"http://localhost:52415"` | exo OpenAI-compatible endpoint URL | `EXO_URL` |
| `exo_health_interval` | `int` | `30` | Seconds between exo health checks | `EXO_HEALTH_INTERVAL` |

### Petals Swarm Mode (Phase 7)

Large-model inference distributed across internet contributors.

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `petals_enabled` | `bool` | `False` | Enable Petals distributed inference | `PETALS_ENABLED` |
| `petals_model` | `str` | `"meta-llama/Meta-Llama-3.1-405B"` | Model to split across swarm | `PETALS_MODEL` |
| `petals_initial_peers` | `list[str]` | `[]` | Initial DHT peers for swarm discovery | `PETALS_INITIAL_PEERS` |
| `petals_health_interval` | `int` | `60` | Seconds between swarm health checks | `PETALS_HEALTH_INTERVAL` |
| `petals_min_vram_gb` | `int` | `150` | Minimum total VRAM (GB) to activate Petals | `PETALS_MIN_VRAM_GB` |

### Real-time Sync (Phase 9 -- cr-sqlite)

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `sync_enabled` | `bool` | `False` | Enable WebSocket changeset sync | `SYNC_ENABLED` |
| `sync_poll_interval` | `float` | `0.1` | Seconds between changeset polls | `SYNC_POLL_INTERVAL` |

### Government Monitoring

| Setting | Type | Default | Description | Env Var |
|---------|------|---------|-------------|---------|
| `auto_government_monitoring` | `bool` | `True` | Auto-start government investigation on boot | `AUTO_GOVERNMENT_MONITORING` |
| `gov_scan_rate_limit` | `float` | `0.5` | Seconds between parliamentary API call chunks | `GOV_SCAN_RATE_LIMIT` |
| `gov_scan_max_pages` | `int` | `10` | Max pagination depth per scan | `GOV_SCAN_MAX_PAGES` |
| `gov_contradiction_max_pairs` | `int` | `30` | Max position pairs for LLM contradiction analysis | `GOV_CONTRADICTION_MAX_PAIRS` |
| `gov_monitoring_interval_hours` | `int` | `6` | SearXNG sweep interval per politician (hours) | `GOV_MONITORING_INTERVAL_HOURS` |
| `gov_database_url` | `str` | `""` | PostgreSQL URL (empty = use SQLite fallback) | `GOV_DATABASE_URL` |

---

## 2. Worker Settings (`worker/config.py`)

Worker configuration is stored as JSON at `~/.nexus-worker/config.json`. These settings are populated during `nexus-worker register` and can be edited manually.

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `server_url` | `str` | `""` | NEXUS server URL to connect to (e.g. `http://nexus.example.com:8000`) |
| `node_id` | `str` | `""` | Unique node identifier assigned during registration |
| `api_key` | `str` | `""` | API key for authenticating with the NEXUS server |
| `name` | `str` | `""` | Human-readable name for this worker node |
| `gpu_model` | `str` | `""` | GPU model name (e.g. `"RTX 5080"`) |
| `vram_mb` | `int` | `0` | GPU VRAM in megabytes (e.g. `16384`) |
| `platform` | `str` | `""` | OS/platform identifier (e.g. `"Windows 11"`) |
| `ollama_url` | `str` | `"http://localhost:11434"` | Local Ollama endpoint used by this worker |
| `poll_interval` | `float` | `2.0` | Seconds between task polling requests |
| `heartbeat_interval` | `float` | `15.0` | Seconds between heartbeat signals to server |
| `private_key_pem` | `str` | `""` | Ed25519 private key PEM for task result signing |

Worker registration status is determined by `is_registered()` which checks that both `api_key` and `server_url` are non-empty.

---

## 3. Environment Variables

pydantic-settings maps environment variables to `Settings` fields by uppercasing the field name. Every field in the `Settings` class can be overridden via its corresponding env var.

### `.env.example` Reference

The `.env.example` file documents the most commonly configured variables. Copy it to `.env` and adjust:

```bash
cp .env.example .env
```

| Env Var | Settings Field | `.env.example` Value | Notes |
|---------|---------------|---------------------|-------|
| `OLLAMA_BASE_URL` | `ollama_base_url` | `http://localhost:11434` | Ollama API |
| `NEO4J_URI` | `neo4j_uri` | `bolt://localhost:7687` | Docker service |
| `NEO4J_USER` | `neo4j_user` | `neo4j` | |
| `NEO4J_PASSWORD` | `neo4j_password` | `changeme` | Change in production! |
| `CHROMA_HOST` | `chroma_host` | `localhost` | Docker service |
| `CHROMA_PORT` | `chroma_port` | `8100` | Docker service |
| `SEARXNG_URL` | `searxng_url` | `http://localhost:8888` | Docker service |
| `ROBIN_URL` | `robin_url` | `http://localhost:9090` | Docker service |
| `COMPUTE_ENABLED` | `compute_enabled` | `true` | GPU compute system |
| `COMPUTE_HEARTBEAT_TIMEOUT` | `compute_heartbeat_timeout` | `90` | |
| `COMPUTE_TASK_DEFAULT_TIMEOUT` | `compute_task_default_timeout` | `300` | |
| `COMPUTE_REAPER_INTERVAL` | `compute_reaper_interval` | `60` | |
| `COMPUTE_SPOT_CHECK_RATE` | `compute_spot_check_rate` | `0.05` | |
| `COMPUTE_MAX_RETRIES` | `compute_max_retries` | `3` | |
| `COMPUTE_RATE_LIMIT_PER_MINUTE` | `compute_rate_limit_per_minute` | `100` | |
| `EXO_ENABLED` | `exo_enabled` | `false` | LAN multi-GPU |
| `EXO_URL` | `exo_url` | `http://localhost:52415` | |
| `EXO_HEALTH_INTERVAL` | `exo_health_interval` | `30` | |
| `PETALS_ENABLED` | `petals_enabled` | `false` | Internet swarm |
| `PETALS_MODEL` | `petals_model` | `meta-llama/Meta-Llama-3.1-405B` | |
| `PETALS_INITIAL_PEERS` | `petals_initial_peers` | *(commented out)* | JSON array of peer addresses |
| `PETALS_HEALTH_INTERVAL` | `petals_health_interval` | `60` | |
| `PETALS_MIN_VRAM_GB` | `petals_min_vram_gb` | `150` | |
| `SYNC_ENABLED` | `sync_enabled` | `false` | cr-sqlite sync |
| `SYNC_POLL_INTERVAL` | `sync_poll_interval` | `0.1` | |

### Ollama-specific Environment Variables

These are consumed by Ollama itself (not by NEXUS), but are set in `start.bat`:

| Env Var | Purpose |
|---------|---------|
| `OLLAMA_FLASH_ATTENTION` | Set to `1` to enable Flash Attention in Ollama (faster inference, lower VRAM) |
| `OLLAMA_HOST` | Used by `start_nexus.py` to locate the Ollama API (falls back to `http://localhost:11434`) |

---

## 4. `start.bat` Defaults

The Windows launcher `start.bat` sets the following environment variables before calling `start_nexus.py`:

| Env Var | Value | Purpose |
|---------|-------|---------|
| `COMPUTE_ENABLED` | `true` | Enable distributed GPU compute system |
| `EXO_ENABLED` | `false` | Disable exo LAN inference (not configured) |
| `PETALS_ENABLED` | `false` | Disable Petals swarm (not configured) |
| `SYNC_ENABLED` | `false` | Disable cr-sqlite sync (not configured) |
| `OLLAMA_FLASH_ATTENTION` | `1` | Enable Flash Attention in Ollama for faster inference |

It also prepends the Miniconda3 base environment to `PATH` so that `python` resolves correctly.

The launcher then delegates to `start_nexus.py` which sequentially:

1. Starts Docker Compose services (Neo4j, ChromaDB, Robin)
2. Checks/starts SearXNG container
3. Verifies Ollama is running and pulls missing models (`gemma-4-26B-A4B-it-heretic:q4_k_m`, `nomic-embed-text`)
4. Starts Vite frontend on port 3002
5. Starts FastAPI backend on port 8000 (foreground with live logs)
6. Opens browser at `http://localhost:3002` once backend is ready

---

## Quick Start

Minimal `.env` for a fresh install:

```env
NEO4J_PASSWORD=changeme
```

All other settings have working defaults. The system will bind to `0.0.0.0:8000` (backend) and expect Docker services on their standard ports.

To disable subsystems you do not need:

```env
COMPUTE_ENABLED=false
AUTO_GOVERNMENT_MONITORING=false
AUTO_IMAGE_ANALYSIS=false
AUTO_FORENSIC_ANALYSIS=false
```
