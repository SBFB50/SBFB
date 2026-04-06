# External Integrations

**Analysis Date:** 2026-04-06

## APIs & External Services

**LLM Inference (Ollama):**
- Ollama local server - All LLM tasks (text generation, JSON, vision, embeddings)
  - SDK: `ollama` Python SDK (`AsyncClient`)
  - Client: `nexus/llm/ollama_client.py`
  - Router: `nexus/llm/router.py` (dispatches 20 TaskTypes to 7 models)
  - Connection: `settings.ollama_base_url` (default `http://localhost:11434`)
  - Auth: None (local only)
  - Retry: tenacity, 3 attempts, exponential backoff 1s-8s on `httpx.ConnectError`/`TimeoutException`
  - VRAM lock: `asyncio.Lock` serializes heavy model calls

**Clearweb Search (SearXNG):**
- SearXNG metasearch engine - Web monitoring for active cases
  - Client: `nexus/monitoring/searxng_monitor.py`
  - HTTP client: `httpx` async
  - Connection: `settings.searxng_url` (default `http://localhost:8888`)
  - Config: `searxng/settings.yml` (Google, DuckDuckGo, Brave, Wikipedia enabled)
  - Auth: None
  - Format: JSON API (`/search?format=json`)
  - Rate limiting: configurable via `searxng/limiter.toml`

**Dark Web Search (Robin/Tor):**
- Robin (`apurvsg/robin`) - Dark web search via Tor
  - Client: `nexus/monitoring/robin_monitor.py`
  - Interface: CLI via `docker exec` subprocess (no REST API)
  - Container: `nexus-robin` (Docker)
  - Connection: `settings.robin_url` (default `http://localhost:9090`)
  - Searches 15+ .onion search engines
  - Output: Markdown investigation reports, parsed to structured data

**Geocoding (Nominatim/OSM):**
- OpenStreetMap Nominatim - Address to GPS coordinates
  - Client: `nexus/core/geo_mapper.py`
  - HTTP client: `httpx` async
  - URL: `https://nominatim.openstreetmap.org/search`
  - Auth: None (public API)
  - User-Agent: `NEXUS-Investigation/0.1`
  - Rate limit: 1.1s between requests (Nominatim policy)

**Routing (OSRM):**
- Project OSRM - Driving distance/time calculations
  - Client: `nexus/core/geo_mapper.py`
  - URL: `http://router.project-osrm.org/route/v1/driving`
  - Auth: None (public API)
  - Used for travel-time verification in investigations

**OSINT - Email Recon (Holehe):**
- Holehe - Email existence across 120+ services
  - Client: `nexus/recon/holehe_recon.py`
  - Interface: subprocess (avoids event-loop conflicts with holehe's httpx/trio)
  - Output: CSV parsed to list of `{site, domain, exists}` dicts
  - Rate limit: `settings.auto_recon_rate_limit` (2.0s default)

**OSINT - Domain Recon:**
- WHOIS via `python-whois` library
  - Client: `nexus/recon/domain_recon.py`
  - Runs in thread pool (blocking library)
  - Output: registrar, dates, name servers, registrant info

- DNS via `socket` + `nslookup` subprocess
  - Client: `nexus/recon/domain_recon.py`

**NER (GLiNER):**
- `urchade/gliner_multi-v2.1` - Named Entity Recognition
  - Client: `nexus/core/entity_extractor.py`
  - Runs on CPU (~0.08s per text, zero VRAM)
  - Loaded lazily on first use
  - 17 French entity labels (personne, lieu, vehicule, telephone, etc.)
  - Falls back to LLM (gemma4:e4b) for relation extraction

**Visual AI (DINOv2 + CLIP):**
- `facebook/dinov2-base` (86M params, ~350MB VRAM) - Image-to-image similarity
  - Client: `nexus/vision/embeddings.py`
  - Loaded lazily, unloads CLIP before loading (VRAM management)
  - Embeddings stored in ChromaDB `image_dinov2` collection

- CLIP - Text-to-image search
  - Client: `nexus/vision/embeddings.py`
  - Loaded lazily, unloads DINOv2 before loading
  - Embeddings stored in ChromaDB `image_clip` collection

**Entity Resolution (RapidFuzz):**
- Jaro-Winkler string similarity with 82% threshold
  - Used in: `nexus/core/entity_extractor.py`
  - Purpose: Deduplication of extracted entities across evidence

## Data Storage

**SQLite (Primary relational store):**
- File: `data/nexus.db` (configurable via `settings.sqlite_path`)
- Driver: `aiosqlite` (async)
- Client: `nexus/db/sqlite_db.py`
- Features: FTS5 full-text search, WAL mode, 20+ indexes
- Schema: 15 tables (cases, evidence, entities, entity_mentions, hypotheses, hypothesis_snapshots, hypothesis_evidence, contradictions, suspects, suspect_scores, monitoring_jobs, monitoring_results, alerts, audit_log, analysis_runs)
- Models: `nexus/db/models.py` (Pydantic v2, Base/Create/Update/Full pattern)

**Neo4j (Graph database):**
- Docker container: `nexus-neo4j` (image `neo4j:5-community`)
- Driver: `neo4j` Python async driver (`AsyncGraphDatabase`)
- Client: `nexus/db/neo4j_db.py`
- Connection: `bolt://localhost:7687`
- Auth: `neo4j/nexus2026` (in `docker-compose.yml` and `settings`)
- Plugins: APOC
- Memory: heap 512m-1g
- Node labels: Person, Location, Phone, Vehicle, Organization, Account, Event, Evidence, Money, Hypothesis, Case
- Relationship types: 21 typed relationships (KNOWS, WAS_AT, OWNS, MENTIONS, SUPPORTS, CONTRADICTS, etc.)
- Features: centrality, betweenness, community detection, temporal queries
- Volumes: `data/neo4j/data`, `data/neo4j/logs`
- Optional: system runs in degraded mode without it

**ChromaDB (Vector store):**
- Docker container: `nexus-chromadb` (image `chromadb/chroma:latest`)
- Client: `chromadb` Python SDK (HTTP client)
- Wrapper: `nexus/db/chroma_db.py`
- Connection: `http://localhost:8100` (remapped from container :8000)
- Collections:
  - `evidence_chunks` - Primary RAG source, chunked evidence embeddings
  - `entity_contexts` - Entity descriptions + context
  - `monitoring_results` - Monitoring hits (deduplication)
  - `hypothesis_reasoning` - Hypothesis snapshots
  - `image_dinov2` - DINOv2 visual embeddings (managed by `ImageSearchEngine`)
  - `image_clip` - CLIP text-image embeddings (managed by `ImageSearchEngine`)
  - `evidence_texts` - DEPRECATED, superseded by `evidence_chunks`
- Embedding function: None (pre-computed by Ollama `nomic-embed-text`)
- Distance metric: cosine (`hnsw:space: cosine`)
- Telemetry: disabled (`ANONYMIZED_TELEMETRY=false`)
- Volumes: `data/chroma`
- Optional: system runs in degraded mode without it

**File Storage:**
- Local filesystem only
- Uploads: `data/uploads/` (evidence files)
- Reports: `data/reports/` (generated PDF reports)
- Backups: `data/backups/` (database backups)
- Audit logs: `data/audit/` (JSONL append-only files + local git repo)
- Benchmark data: `data/benchmark/` (3 cold cases: kulik, golden-state-killer, affaire-moreau)

**Caching:**
- None (no Redis, no Memcached)
- Ollama `keep_alive="10m"` keeps models warm in VRAM
- React Query client-side caching: `staleTime: 5000ms`, `refetchInterval: 10000ms`

## Authentication & Identity

**Auth Provider:**
- None - No authentication system
  - CORS: `allow_origins=["*"]` (wide open, development mode)
  - All API endpoints are unauthenticated
  - Neo4j has hardcoded credentials in config

## Monitoring & Observability

**Error Tracking:**
- None (no Sentry, no external APM)
- FastAPI global exception handler in `nexus/main.py` catches Ollama errors (503) and unhandled exceptions (500)

**Logs:**
- Loguru (`nexus/` throughout)
  - Intercepts uvicorn/fastapi stdlib loggers
  - Structured logging with `logger.info`, `logger.debug`, `logger.error`, `logger.warning`
  - No external log aggregation

**Performance:**
- `X-Process-Time` header on every response (`ProcessTimeMiddleware` in `nexus/main.py`)
- No metrics collection (no Prometheus, no StatsD)

**Audit Trail:**
- 3-layer immutable audit in `nexus/core/audit.py`:
  1. SQLite hash chain (SHA-256 linked entries, tamper-detectable)
  2. Append-only JSONL files (`data/audit/{case_id}.jsonl`)
  3. Git commits in `data/audit/` local repo

## CI/CD & Deployment

**Hosting:**
- Local machine only (Windows 11, single workstation)
- No cloud deployment, no production infrastructure

**CI Pipeline:**
- None detected (no `.github/workflows/`, no `.gitlab-ci.yml`, no Jenkinsfile)

**Docker Compose (`docker-compose.yml`):**
- 3 services: Neo4j, ChromaDB, Robin
- All with `restart: unless-stopped`
- Health checks on Neo4j (wget to :7474) and ChromaDB (curl to heartbeat)
- SearXNG runs on host natively (not containerized)

## Environment Configuration

**Required env vars (all have defaults in `nexus/config.py`):**
- `OLLAMA_BASE_URL` - Ollama server (default: `http://localhost:11434`)
- `NEO4J_URI` / `NEO4J_USER` / `NEO4J_PASSWORD` - Neo4j connection
- `CHROMA_HOST` / `CHROMA_PORT` - ChromaDB connection
- `SEARXNG_URL` - SearXNG instance
- `ROBIN_URL` - Robin dark web search
- `MODEL_DEEP` / `MODEL_FAST` / `MODEL_REASONING` / `MODEL_EMBEDDING` / `MODEL_VISION` / `MODEL_VISION_DEEP` / `MODEL_AUDIO` - Ollama model names
- `OLLAMA_FLASH_ATTENTION=1` - Recommended env var for Ollama performance

**Secrets location:**
- `.env` file in project root (exists, not committed)
- `robin.env` - Robin container environment (mounted as volume)
- Neo4j password hardcoded in `docker-compose.yml` (`nexus2026`) and `nexus/config.py` default

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## Frontend-Backend Integration

**React Frontend (`web/`):**
- Vite dev proxy: `/api` -> `http://localhost:8000` (`web/vite.config.ts`)
- API client: `axios` with `baseURL: '/api'` (`web/src/api/client.ts`)
- Data layer: `@tanstack/react-query` with auto-refetch every 10s
- State: `zustand` stores for case selection (`web/src/stores/caseStore.ts`) and system stats (`web/src/stores/systemStore.ts`)
- 9 pages: Dashboard, Evidence, Entities, Hypotheses, Graph, Timeline, Investigation, Benchmark, Suspects

**Streamlit Legacy Frontend (`frontend/`):**
- Direct HTTP calls to FastAPI via custom `api_client.py`
- 16 pages covering all features (dashboard, evidence, entities, hypotheses, timeline, graph, monitoring, alerts, analysis, map, osint, vision, forensics, investigation, audit, benchmark)
- Independent session state management via `st.session_state`

## Data Flow Summary

```
Evidence Upload/Text → EvidenceProcessor
  → PDFParser/TextParser (text extraction)
  → GLiNER (NER, CPU) → RapidFuzz (dedup)
  → LLM gemma4:e4b (summary)
  → TextChunker (512 tokens, 128 overlap)
  → Ollama nomic-embed-text (embeddings)
  → ChromaDB evidence_chunks
  → Neo4j (entity nodes + relationships)
  → SQLite (evidence, entities, mentions)

Analysis Request → AnalysisPipeline
  → InvestigationRetriever (hybrid RAG)
    → ChromaDB semantic search (0.6 weight)
    → Neo4j graph traversal (0.3 weight)
    → Recency boost (0.1 weight)
  → LLM nexus 26B (deep analysis)
  → HypothesisEngine (generate/score)
  → ContradictionDetector (deepseek-r1 14B)
  → SuspectScorer (5-factor scoring)

Autonomous Loop (OODA per case, every 30min):
  OBSERVE  → Check monitoring results, new evidence
  ORIENT   → Auto-ingest, OSINT recon, geocode, image analysis
  DECIDE   → Re-evaluate hypotheses, contradictions, forensics, timeline
  ACT      → Generate new search queries, domain recon
  QUESTION → Challenge top hypothesis, periodic reports, backups
```

---

*Integration audit: 2026-04-06*
