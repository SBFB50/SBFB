# Architecture

**Analysis Date:** 2026-04-06

## Pattern Overview

**Overall:** Autonomous Agent System with OODA Loop + Layered Service Architecture

**Key Characteristics:**
- Persistent autonomous investigation daemon running one OODA loop per active case
- 3-database polyglot persistence: SQLite (relational), Neo4j (graph), ChromaDB (vectors)
- Multi-model LLM routing with VRAM serialization (asyncio.Lock) to prevent GPU OOM on a single RTX 5080 16GB
- Request-scoped dependency injection via FastAPI `Depends()` for all API handlers
- Shared singletons for expensive resources (LLMRouter, Neo4j driver, ChromaDB client) on `app.state`
- Graceful degradation: Neo4j and ChromaDB are optional; system runs in degraded mode if unavailable

## Layers

**API Layer (FastAPI Routers):**
- Purpose: HTTP interface exposing all system capabilities as REST endpoints
- Location: `nexus/api/`
- Contains: 21 router files, each defining endpoints for one domain
- Depends on: `nexus/api/deps.py` for dependency injection, `nexus/db/models.py` for Pydantic schemas
- Used by: React frontend (`web/`), Streamlit frontend (`frontend/`), direct API calls

**Dependency Injection Layer:**
- Purpose: Build request-scoped service instances with proper DB connections and shared singletons
- Location: `nexus/api/deps.py`
- Contains: ~20 FastAPI dependency functions
- Pattern: Each request gets its own `aiosqlite` connection (via `get_db()` context manager) and a fresh `Database` wrapper. Shared singletons (LLMRouter, Neo4j, ChromaDB) are pulled from `request.app.state`. Higher-level services (EvidenceProcessor, AnalysisPipeline, HypothesisEngine) are assembled per-request on top.
- Depends on: `app.state` singletons set in lifespan, `nexus/db/sqlite_db.py` for connection management
- Used by: All API router handlers

**Core Business Logic Layer:**
- Purpose: All investigation intelligence -- OODA loop, evidence processing, hypothesis generation, contradiction detection, suspect scoring, retrieval, forensics
- Location: `nexus/core/`
- Contains: 18 modules (see detailed breakdown below)
- Depends on: `nexus/db/` for persistence, `nexus/llm/` for model access, `nexus/monitoring/` for search
- Used by: API layer (via deps.py), autonomous loop (directly)

**LLM Abstraction Layer:**
- Purpose: Route tasks to the right Ollama model, manage VRAM contention, provide retry/timeout
- Location: `nexus/llm/`
- Contains: `router.py` (task routing), `ollama_client.py` (async SDK wrapper), `prompts.py` (25+ French prompts), `parsers.py` (robust JSON extraction)
- Depends on: Ollama server (localhost:11434), `nexus/config.py` for model names
- Used by: Every core module that needs LLM capabilities

**Database Layer:**
- Purpose: Polyglot persistence -- relational data, graph relationships, vector embeddings
- Location: `nexus/db/`
- Contains: `sqlite_db.py` (async SQLite with 17 tables + FTS5), `neo4j_db.py` (async graph client), `chroma_db.py` (vector store client), `models.py` (Pydantic v2 schemas)
- Depends on: aiosqlite, neo4j Python driver, chromadb SDK
- Used by: Core layer, API layer (via deps.py)

**Monitoring Layer:**
- Purpose: Automated web surveillance -- clearweb (SearXNG) and dark web (Robin/Tor)
- Location: `nexus/monitoring/`
- Contains: `scheduler.py` (APScheduler orchestration), `searxng_monitor.py`, `robin_monitor.py`, `alert_manager.py`
- Depends on: SearXNG (port 8888), Robin (port 8502), LLMRouter for relevance filtering
- Used by: Autonomous loop OBSERVE phase, API layer

**Forensics Layer:**
- Purpose: Specialized forensic analysis -- blood patterns, traces, acoustics, physics simulation
- Location: `nexus/forensics/`
- Contains: `blood_pattern.py`, `trace_analyzer.py`, `acoustic_analysis.py`, `physics_sim.py`, `the_well_loader.py`
- Depends on: LLMRouter (VLM models for image analysis, audio transcription)
- Used by: Autonomous loop DECIDE phase

**OSINT Recon Layer:**
- Purpose: Open-source intelligence gathering -- email existence, social profiles, domain WHOIS/DNS
- Location: `nexus/recon/`
- Contains: `holehe_recon.py` (email on 120+ services), `social_recon.py` (username across platforms), `domain_recon.py` (WHOIS + DNS)
- Depends on: External services (holehe library, HTTP requests)
- Used by: Autonomous loop ORIENT and ACT phases

**Vision Layer:**
- Purpose: Image analysis and similarity search using DINOv2 and CLIP embeddings
- Location: `nexus/vision/`
- Contains: `embeddings.py` (DINOv2/CLIP embedding generation), `image_search.py` (similarity search engine)
- Depends on: PyTorch, ChromaDB for storage
- Used by: Autonomous loop ORIENT phase, API layer

**Ingest Layer:**
- Purpose: Parse uploaded files into raw text
- Location: `nexus/ingest/`
- Contains: `pdf_parser.py` (PyMuPDF), `text_parser.py` (plain text/HTML/CSV)
- Depends on: PyMuPDF (fitz)
- Used by: EvidenceProcessor

**Export Layer:**
- Purpose: Generate reports and export data
- Location: `nexus/export/`
- Contains: `report_generator.py` (summary reports via LLM), `pdf_export.py` (WeasyPrint + Jinja2), `timeline_export.py`, `templates/` (Jinja2 HTML templates)
- Depends on: LLMRouter, WeasyPrint, Jinja2
- Used by: Autonomous loop QUESTION phase, API layer

**Frontend Layer (React):**
- Purpose: Professional dark-themed investigation dashboard
- Location: `web/src/`
- Contains: 9 pages, 9 components, 3 hooks, 2 Zustand stores
- Depends on: Backend API (proxied via Vite dev server)
- Used by: End users (investigators)

## Data Flow

**Evidence Ingestion Pipeline (the core data flow):**

1. **Input**: File upload (PDF, image, text, audio) or text submission via API (`nexus/api/evidence.py`)
2. **Save**: File saved to `data/uploads/{case_id}/{uuid}.{ext}` (`nexus/core/evidence_processor.py`)
3. **Detect**: MIME type detection, evidence_type classification
4. **Parse**: Extract raw text -- PDFParser for PDFs, TextParser for text, VLM for images, Voxtral for audio (`nexus/ingest/`)
5. **Hash**: SHA-256 file hash computed for deduplication
6. **Store**: Evidence record created in SQLite (status='processing') (`nexus/db/sqlite_db.py`)
7. **NER**: Entity extraction via GLiNER (CPU, ~0.08s) with LLM fallback (`nexus/core/entity_extractor.py`)
8. **Dedup Entities**: RapidFuzz Jaro-Winkler matching (threshold 82%) against existing entities
9. **Summarize**: LLM summary generation via gemma4:e4b (`nexus/llm/router.py` -> TaskType.EVIDENCE_SUMMARY)
10. **Chunk**: Recursive semantic chunking at 512 tokens with 128 token overlap (`nexus/core/chunker.py`)
11. **Embed**: Batch embedding via nomic-embed-text (`nexus/core/embedding_store.py`)
12. **Vector Store**: Chunks stored in ChromaDB `evidence_chunks` collection (`nexus/db/chroma_db.py`)
13. **Graph Sync**: Entities synced to Neo4j as typed nodes (Person, Location, Phone, etc.) (`nexus/db/neo4j_db.py`)
14. **Status Update**: Evidence marked as 'processed' in SQLite

**Autonomous OODA Loop (per active case):**

1. **OBSERVE** (`_observe`): Query SQLite for unreviewed monitoring results above relevance threshold (default 50%)
2. **ORIENT** (`_orient`): 
   - 2a. Auto-ingest high-relevance monitoring results through evidence pipeline
   - 2b. OSINT recon on new email/account entities (holehe + social)
   - 2c. Geocode location entities via Nominatim
   - 2d. VLM analysis of unprocessed image evidence
   - 2e. Index images in DINOv2/CLIP for visual similarity
3. **DECIDE** (`_decide`):
   - 3a. Run incremental analysis on each new evidence (AnalysisPipeline)
   - 3b. Re-evaluate ALL active hypotheses (HypothesisEngine + RAG retriever)
   - 3c. Detect contradictions between evidence pairs (ContradictionDetector via deepseek-r1)
   - 3d. Forensic analysis on image/audio evidence (blood patterns, traces, acoustics)
   - 3e. Rebuild chronological timeline (TimelineBuilder)
   - 3f. Rebuild RAPTOR summary tree every 3 cycles (SummaryTree)
4. **ACT** (`_act`):
   - 4a. LLM generates new search queries based on current hypotheses and contradictions
   - 4b. Create monitoring jobs from OSINT recon discoveries (holehe hits -> SearXNG jobs)
   - 4c. Domain recon (WHOIS/DNS) on email entity domains
5. **QUESTION** (`_question`):
   - 5a. Adversarial self-questioning: challenge the top-scored hypothesis
   - 5b. Periodic report generation (every N cycles, default 12 = ~6 hours)
   - 5c. Automated database backup (every N cycles, default 24 = ~12 hours)
6. **SLEEP**: Wait `investigation_cycle_minutes` (default 30 min) then repeat

**RAG Retrieval Flow (used by AnalysisPipeline, HypothesisEngine):**

1. Query received (natural language or hypothesis text)
2. Embed query via nomic-embed-text
3. Semantic search: ChromaDB cross-collection search (evidence_chunks + entity_contexts + monitoring_results)
4. Graph search: Find entities in query text, traverse Neo4j neighbors, fetch connected evidence
5. Merge + deduplicate results by evidence_id + chunk_text prefix
6. Rerank: weighted score = semantic(0.6) + graph(0.3) + recency(0.1)
7. Return top-K chunks as context for LLM prompt

**State Management:**
- **Backend**: All state in SQLite (source of truth), mirrored to Neo4j (relationships) and ChromaDB (embeddings)
- **Frontend**: Zustand store (`web/src/stores/caseStore.ts`) persists active case ID to localStorage. React Query (`@tanstack/react-query`) handles server state with 5s stale time and 10s refetch interval.

## Key Abstractions

**LLMRouter (VRAM Serialization):**
- Purpose: Route any LLM task to the optimal model while preventing GPU OOM
- Location: `nexus/llm/router.py`
- Pattern: 21 TaskType enum values mapped to (model_name, timeout, heavy_flag). Heavy tasks (26B nexus, 14B deepseek-r1) acquire an `asyncio.Lock` before calling Ollama. Light tasks (4B gemma4:e4b, nomic-embed-text) run concurrently without the lock.
- Methods: `route()` (text), `route_json()` (structured JSON), `route_vision()` (image + text), `embed()` / `embed_batch()` (embeddings)

**InvestigationRetriever (Hybrid RAG):**
- Purpose: Unified retrieval combining semantic, graph, and recency signals
- Location: `nexus/core/retriever.py`
- Pattern: Three retrieval strategies executed in parallel, results merged and reranked with configurable weights
- Constants: `_SEMANTIC_WEIGHT = 0.6`, `_GRAPH_WEIGHT = 0.3`, `_RECENCY_WEIGHT = 0.1`

**AutonomousInvestigator (OODA Daemon):**
- Purpose: One continuous investigation loop per active case
- Location: `nexus/core/autonomous_loop.py`
- Pattern: Infinite async loop with 5 phases, opening one DB connection per cycle, sleeping between cycles. Error recovery: 5-minute wait on exception, continues to next cycle.

**InvestigationManager (Lifecycle Manager):**
- Purpose: Manage one AutonomousInvestigator per active case
- Location: `nexus/core/investigation_manager.py`
- Pattern: Dict of `case_id -> AutonomousInvestigator` with corresponding `asyncio.Task`. Started in FastAPI lifespan, auto-starts investigators for all active cases.

**EvidenceProcessor (Ingestion Pipeline):**
- Purpose: Full ingestion from file/text to processed+indexed evidence
- Location: `nexus/core/evidence_processor.py`
- Pattern: Sequential pipeline (save -> detect -> parse -> hash -> store -> NER -> summarize -> chunk -> embed -> graph sync)

**SummaryTree (RAPTOR Hierarchical Summaries):**
- Purpose: Multi-level summary hierarchy for efficient context building
- Location: `nexus/core/summary_tree.py`
- Pattern: 3 levels -- L0: individual evidence, L1: thematic clusters (agglomerative clustering on embeddings), L2: case-level summary. Rebuilt every 3 OODA cycles.

## Entry Points

**FastAPI Application:**
- Location: `nexus/main.py`
- Triggers: `uvicorn nexus.main:app --host 0.0.0.0 --port 8000`
- Responsibilities: Lifespan startup (init DB, create singletons, start monitoring scheduler, start investigation manager), register 21 routers, CORS middleware, error handlers

**React Frontend:**
- Location: `web/src/main.tsx` -> `web/src/App.tsx`
- Triggers: `cd web && npx vite --host 0.0.0.0 --port 3002`
- Responsibilities: SPA with 9 routes, Vite dev proxy routes `/api/*` to `http://localhost:8000`

**Streamlit Frontend (Legacy):**
- Location: `frontend/app.py`
- Triggers: `streamlit run frontend/app.py --server.port 8501`
- Responsibilities: Legacy 16-page investigation interface

## Error Handling

**Strategy:** Multi-layer defensive with graceful degradation

**Patterns:**
- **Lifespan startup**: Each optional service (Neo4j, ChromaDB, MonitoringScheduler, InvestigationManager) wrapped in try/except. Failure sets `app.state.{service} = None` and logs warning. System continues in degraded mode.
- **OODA loop**: Each phase and sub-phase independently wrapped. Single sub-phase failure (e.g., geocoding) does not abort the cycle. Full cycle exception triggers 5-minute sleep then retry.
- **LLM calls**: Ollama client uses `tenacity` retry (3 attempts, exponential backoff 1s->2s->4s) for transient network errors. Router catches `RequestError`/`ResponseError` and surfaces as 503.
- **API layer**: Global exception handler in `nexus/main.py` catches Ollama connection errors -> 503, all others -> 500. Individual endpoints raise `HTTPException` for business logic errors (404, 400).
- **Audit logging**: Non-blocking (`try/except` with `logger.warning`). Audit failures never disrupt main operations.
- **Entity extraction**: GLiNER failure falls back to LLM-based extraction. Both paths tolerate partial failures gracefully.

## Cross-Cutting Concerns

**Logging:**
- Framework: Loguru with stdlib intercept handler
- Pattern: All uvicorn/fastapi logs redirected to Loguru. Structured format with `{}` placeholders. Every module uses `from loguru import logger`.

**Validation:**
- Input: Pydantic v2 models in `nexus/db/models.py` with `Literal` types for enums, `Field` constraints for ranges
- Pattern: Base/Create/Update/Full schema hierarchy per entity. `model_config = {"from_attributes": True}` for SQLite row conversion.

**Authentication:**
- None. CORS allows all origins (`allow_origins=["*"]`). No auth middleware.

**Audit Trail:**
- 3-layer immutable audit: SQLite hash chain (tamper-detectable), append-only JSONL files (`data/audit/{case_id}.jsonl`), Git commits in `data/audit/` repo
- Location: `nexus/core/audit.py`
- Every autonomous loop action, evidence ingestion, hypothesis scoring, and OSINT result is logged

**Configuration:**
- Centralized in `nexus/config.py` via pydantic-settings `BaseSettings`
- Loads from `.env` file, all settings have sensible defaults
- Singleton: `from nexus.config import settings`

## Database Architecture

**SQLite (Primary Relational Store):**
- Location: `data/nexus.db`
- Client: `nexus/db/sqlite_db.py` -- async via aiosqlite, WAL mode
- Tables (17): `cases`, `evidence`, `entities`, `entity_mentions`, `hypotheses`, `hypothesis_snapshots`, `analysis_runs`, `monitoring_jobs`, `monitoring_results`, `alerts`, `reports`, `locations`, `audit_log`, `summary_clusters`, `suspects`, `suspect_snapshots`, `case_summaries`
- FTS5: `evidence_fts` virtual table with auto-sync triggers (created but not queried by any endpoint yet)
- Indexes: 20+ indexes including composite indexes for filtered queries
- Connection pattern: `get_db()` async context manager yields a fresh connection per request/operation

**Neo4j (Graph Store):**
- Location: Docker container, bolt://localhost:7687
- Client: `nexus/db/neo4j_db.py` -- async via neo4j Python driver
- Node labels (11): Person, Location, Phone, Vehicle, Organization, Account, Event, Evidence, Money, Hypothesis, Case
- Relationship types (17): KNOWS, RELATED_TO, COMMUNICATED_WITH, FINANCIAL_LINK, LIVES_AT, WAS_AT, WORKS_AT, OWNS, MEMBER_OF, OCCURRED_AT, INVOLVES, MENTIONS, SUPPORTS, CONTRADICTS, TRANSACTION, BELONGS_TO, PRECEDED_BY
- Entity type mapping: SQLite entity_type -> Neo4j label (e.g., "person" -> "Person", "email" -> "Account")
- Capabilities: Centrality analysis, betweenness, community detection, shortest path, neighbor traversal
- Status: Often out of sync with SQLite due to pipeline crashes before sync step

**ChromaDB (Vector Store):**
- Location: Docker container, http://localhost:8100
- Client: `nexus/db/chroma_db.py` -- chromadb HTTP client
- Collections (7): `evidence_chunks` (primary RAG), `entity_contexts`, `monitoring_results`, `hypothesis_reasoning`, `evidence_texts` (deprecated), `image_dinov2`, `image_clip`
- Embedding model: nomic-embed-text via Ollama (pre-computed, ChromaDB used as pure vector store)
- HNSW distance: cosine
- Cross-collection search: Unified search across evidence_chunks + entity_contexts + monitoring_results

## Frontend-Backend Communication

**Transport:** HTTP REST via Axios with Vite dev proxy (`/api` -> `http://localhost:8000`)

**Client:** `web/src/api/client.ts` -- centralized Axios instance + ~30 typed API functions

**State Management:**
- Server state: TanStack React Query with 5s stale time, 10s auto-refetch, 1 retry
- Client state: Zustand with localStorage persistence for active case selection (`web/src/stores/caseStore.ts`)

**Data Flow Pattern:**
1. User selects a case on Dashboard (stored in Zustand -> localStorage)
2. Each page reads `caseId` from Zustand store via `useActiveCase()` hook (`web/src/hooks/useCase.ts`)
3. React Query fetches case-scoped data from `/api/cases/{caseId}/...` endpoints
4. 10-second polling interval keeps data fresh (investigation status, alerts, evidence)
5. Mutations (start investigation, upload evidence) invalidate relevant query cache

**API Prefix:** All endpoints under `/api/` prefix. Case-scoped endpoints use `/api/cases/{case_id}/...` pattern.

---

*Architecture analysis: 2026-04-06*
