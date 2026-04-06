# Codebase Structure

**Analysis Date:** 2026-04-06

## Directory Layout

```
nexus/                          # Project root
├── nexus/                      # Backend Python package
│   ├── __init__.py
│   ├── main.py                 # FastAPI app entry point + lifespan
│   ├── config.py               # Centralized settings (pydantic-settings)
│   ├── api/                    # 21 FastAPI routers + dependency injection
│   │   ├── __init__.py
│   │   ├── deps.py             # DI helpers (get_database, get_*_service)
│   │   ├── cases.py            # CRUD /api/cases
│   │   ├── evidence.py         # Upload/text /api/cases/{id}/evidence
│   │   ├── entities.py         # Entity endpoints
│   │   ├── hypotheses.py       # Hypothesis CRUD + generate/evaluate
│   │   ├── analysis.py         # Trigger analysis runs
│   │   ├── graph.py            # Neo4j graph endpoints
│   │   ├── search.py           # Search endpoints
│   │   ├── monitoring.py       # Monitoring jobs management
│   │   ├── alerts.py           # Alert CRUD
│   │   ├── reports.py          # Report generation
│   │   ├── timeline.py         # Timeline endpoints
│   │   ├── geo.py              # Geocoding endpoints
│   │   ├── recon.py            # OSINT recon endpoints
│   │   ├── image_search.py     # Visual similarity search
│   │   ├── vision.py           # VLM image analysis
│   │   ├── forensics.py        # Forensic analysis endpoints
│   │   ├── physics_sim_api.py  # Physics simulation
│   │   ├── investigation.py    # Start/stop autonomous loop
│   │   ├── audit.py            # Audit log endpoints
│   │   ├── benchmark.py        # Benchmark execution + results
│   │   └── suspects.py         # Suspect scoring + profiles
│   ├── core/                   # Business logic (18 modules)
│   │   ├── __init__.py
│   │   ├── autonomous_loop.py  # OODA loop daemon (1439 lines)
│   │   ├── investigation_manager.py  # Lifecycle for all loops
│   │   ├── analysis_pipeline.py     # Multi-model analysis orchestrator
│   │   ├── hypothesis_engine.py     # Hypothesis gen/eval/merge
│   │   ├── contradiction_detector.py # Pairwise contradiction detection
│   │   ├── evidence_processor.py    # Full ingestion pipeline
│   │   ├── entity_extractor.py      # GLiNER + LLM hybrid NER
│   │   ├── suspect_scorer.py        # 5-factor suspect scoring
│   │   ├── retriever.py             # Hybrid RAG (semantic+graph+recency)
│   │   ├── chunker.py               # Semantic text chunking (512 tokens)
│   │   ├── embedding_store.py       # ChromaDB evidence_chunks manager
│   │   ├── summary_tree.py          # RAPTOR hierarchical summaries
│   │   ├── timeline_builder.py      # Chronological timeline extraction
│   │   ├── geo_mapper.py            # Nominatim + OSRM geocoding
│   │   ├── image_analyzer.py        # VLM image analysis
│   │   ├── case_manager.py          # Case CRUD logic
│   │   ├── audit.py                 # 3-layer audit trail
│   │   └── backup.py                # Database backup management
│   ├── db/                     # Database clients
│   │   ├── __init__.py
│   │   ├── sqlite_db.py        # Async SQLite (17 tables, FTS5, WAL)
│   │   ├── neo4j_db.py         # Async Neo4j (11 labels, 17 rel types)
│   │   ├── chroma_db.py        # ChromaDB (7 collections)
│   │   └── models.py           # Pydantic v2 schemas (all tables)
│   ├── llm/                    # LLM abstraction
│   │   ├── __init__.py
│   │   ├── router.py           # Task routing (21 TaskTypes, VRAM lock)
│   │   ├── ollama_client.py    # Async Ollama SDK wrapper + retry
│   │   ├── prompts.py          # 25+ French prompts
│   │   └── parsers.py          # Robust JSON parsing (GLiNER/LLM)
│   ├── monitoring/             # Web surveillance
│   │   ├── __init__.py
│   │   ├── scheduler.py        # APScheduler orchestration
│   │   ├── searxng_monitor.py  # Clearweb search (SearXNG)
│   │   ├── robin_monitor.py    # Dark web search (Robin/Tor)
│   │   └── alert_manager.py    # Alert creation + management
│   ├── forensics/              # Forensic analysis modules
│   │   ├── __init__.py
│   │   ├── blood_pattern.py    # Blood pattern analysis (VLM)
│   │   ├── trace_analyzer.py   # Physical trace analysis (VLM)
│   │   ├── acoustic_analysis.py # Audio forensic analysis
│   │   ├── physics_sim.py      # Physics simulation engine
│   │   └── the_well_loader.py  # The Well data loader
│   ├── recon/                  # OSINT reconnaissance
│   │   ├── __init__.py
│   │   ├── holehe_recon.py     # Email existence on 120+ services
│   │   ├── social_recon.py     # Username lookup across platforms
│   │   └── domain_recon.py     # WHOIS + DNS reconnaissance
│   ├── vision/                 # Computer vision
│   │   ├── __init__.py
│   │   ├── embeddings.py       # DINOv2 + CLIP embedding generation
│   │   └── image_search.py     # Visual similarity search engine
│   ├── ingest/                 # File parsing
│   │   ├── __init__.py
│   │   ├── pdf_parser.py       # PDF text extraction (PyMuPDF)
│   │   └── text_parser.py      # Plain text/HTML/CSV parsing
│   └── export/                 # Report generation + export
│       ├── __init__.py
│       ├── report_generator.py # LLM-generated summary reports
│       ├── pdf_export.py       # PDF rendering (WeasyPrint + Jinja2)
│       ├── timeline_export.py  # Timeline data export
│       └── templates/          # Jinja2 HTML templates for PDF
├── web/                        # React frontend
│   ├── src/
│   │   ├── main.tsx            # React entry point
│   │   ├── App.tsx             # Router (9 routes)
│   │   ├── index.css           # Global styles (Tailwind)
│   │   ├── api/
│   │   │   └── client.ts       # Axios client + ~30 API functions
│   │   ├── pages/              # 9 page components
│   │   │   ├── Dashboard.tsx
│   │   │   ├── Evidence.tsx
│   │   │   ├── Entities.tsx
│   │   │   ├── Hypotheses.tsx
│   │   │   ├── Graph.tsx
│   │   │   ├── Timeline.tsx
│   │   │   ├── Investigation.tsx
│   │   │   ├── Suspects.tsx
│   │   │   └── Benchmark.tsx
│   │   ├── components/         # 9 shared components
│   │   │   ├── Layout.tsx      # App shell with sidebar
│   │   │   ├── Sidebar.tsx     # Navigation sidebar
│   │   │   ├── TopBar.tsx      # Top bar with case selector
│   │   │   ├── Card.tsx
│   │   │   ├── Badge.tsx
│   │   │   ├── DataTable.tsx
│   │   │   ├── LoadingSpinner.tsx
│   │   │   ├── MetricCard.tsx
│   │   │   └── ScoreBar.tsx
│   │   ├── hooks/              # 3 custom hooks
│   │   │   ├── useApi.ts       # Generic API hook helpers
│   │   │   ├── useCase.ts      # Active case hook (useActiveCase)
│   │   │   └── useSystemStats.ts
│   │   ├── stores/             # 2 Zustand stores
│   │   │   ├── caseStore.ts    # Active case ID (persisted)
│   │   │   └── systemStore.ts  # System-wide state
│   │   └── assets/
│   └── vite.config.ts          # Vite + React + Tailwind + /api proxy
├── frontend/                   # Streamlit frontend (legacy)
│   ├── app.py                  # Main entry (16 pages)
│   ├── pages/                  # Streamlit page files
│   └── components/             # Streamlit component files
├── tests/                      # 233 pytest tests
├── data/                       # Runtime data directory
│   ├── nexus.db                # SQLite database
│   ├── uploads/                # Uploaded evidence files
│   ├── reports/                # Generated PDF reports
│   ├── backups/                # Database backups
│   ├── audit/                  # Audit logs (JSONL + git repo)
│   ├── chroma/                 # ChromaDB data (Docker volume)
│   ├── neo4j/                  # Neo4j data (Docker volume)
│   ├── robin/                  # Robin data
│   └── benchmark/              # 3 benchmark cold cases
│       ├── kulik/              # Elodie Kulik (14 evidence files)
│       ├── golden-state-killer/ # GSK (13 evidence files)
│       └── affaire-moreau/     # Fictional (15 files, 7 planted contradictions)
├── docs/                       # Documentation (468 KB)
├── prompts/                    # Additional prompt files
├── models/                     # Model configuration files
├── searxng/                    # SearXNG configuration
├── docker-compose.yml          # Neo4j + ChromaDB + Robin containers
├── requirements.txt            # Python dependencies
├── pytest.ini                  # Pytest configuration
├── Modelfile                   # Ollama Modelfile for nexus model
├── Modelfile.gemma4-heretic    # Gemma 4 Heretic Modelfile
├── Modelfile.qwen3-30b         # Qwen3 30B Modelfile
└── CLAUDE.md                   # Project instructions for Claude
```

## Directory Purposes

**`nexus/api/`:**
- Purpose: HTTP REST interface for the entire system
- Contains: 21 FastAPI router files, each for one domain (cases, evidence, entities, etc.)
- Key files: `deps.py` (all dependency injection functions), `cases.py` (case CRUD pattern to follow)
- Pattern: Each router uses `APIRouter(prefix="/api/{domain}", tags=["{domain}"])`. Endpoints use `Depends()` to get request-scoped services.

**`nexus/core/`:**
- Purpose: All business logic and intelligence
- Contains: 18 Python modules implementing investigation algorithms
- Key files: `autonomous_loop.py` (1439 lines, the brain), `evidence_processor.py` (ingestion pipeline), `retriever.py` (hybrid RAG)
- Pattern: Each module is a class with `__init__` taking `db` + `router` + optional `chroma`/`neo4j`. Methods are async.

**`nexus/db/`:**
- Purpose: All database interaction
- Contains: 3 database clients + Pydantic models
- Key files: `sqlite_db.py` (Database class with full CRUD for 17 tables), `models.py` (Pydantic v2 schemas)
- Pattern: `Database` wraps an `aiosqlite.Connection`. Methods return `dict` (row data) or `None`. `get_db()` async context manager provides connections.

**`nexus/llm/`:**
- Purpose: LLM abstraction layer
- Contains: 4 files -- routing, client, prompts, parsers
- Key files: `router.py` (LLMRouter with 21 TaskTypes), `prompts.py` (25+ French prompt templates)
- Pattern: All LLM access goes through `LLMRouter.route()` or `route_json()`. Never call OllamaClient directly.

**`web/src/`:**
- Purpose: React frontend SPA
- Contains: Pages, components, hooks, stores, API client
- Key files: `api/client.ts` (all API calls), `stores/caseStore.ts` (active case persistence)
- Pattern: Page components use `useActiveCase()` hook to get current case, then TanStack React Query for data fetching.

**`data/`:**
- Purpose: All runtime data (database, uploads, reports, backups)
- Contains: SQLite database, file uploads, generated reports, audit logs, benchmark data
- Generated: Yes (at runtime)
- Committed: Only `data/benchmark/` is committed (test data)

## Key File Locations

**Entry Points:**
- `nexus/main.py`: FastAPI application (startup, middleware, routers)
- `web/src/main.tsx`: React application entry
- `web/src/App.tsx`: React router configuration (9 routes)
- `frontend/app.py`: Streamlit legacy frontend

**Configuration:**
- `nexus/config.py`: All settings (pydantic-settings, loads from .env)
- `web/vite.config.ts`: Vite build config + API proxy
- `docker-compose.yml`: Neo4j + ChromaDB + Robin containers
- `pytest.ini`: Test runner configuration
- `.env`: Environment overrides (exists but not committed)

**Core Logic:**
- `nexus/core/autonomous_loop.py`: OODA investigation daemon (1439 lines)
- `nexus/core/investigation_manager.py`: Manages all investigation loops
- `nexus/core/evidence_processor.py`: Evidence ingestion pipeline
- `nexus/core/analysis_pipeline.py`: Multi-model analysis orchestrator
- `nexus/core/hypothesis_engine.py`: Hypothesis generation and evaluation
- `nexus/core/retriever.py`: Hybrid RAG retriever
- `nexus/core/entity_extractor.py`: GLiNER + LLM hybrid NER
- `nexus/core/suspect_scorer.py`: 5-factor suspect scoring
- `nexus/core/contradiction_detector.py`: Evidence pair contradiction detection

**Database:**
- `nexus/db/sqlite_db.py`: 17 tables, FTS5, 20+ indexes, all CRUD
- `nexus/db/neo4j_db.py`: Graph operations (11 node types, 17 relationship types)
- `nexus/db/chroma_db.py`: Vector store (7 collections)
- `nexus/db/models.py`: Pydantic v2 schemas (Base/Create/Update/Full per entity)

**LLM:**
- `nexus/llm/router.py`: Task-to-model routing with VRAM lock
- `nexus/llm/ollama_client.py`: Async Ollama SDK wrapper with retry
- `nexus/llm/prompts.py`: 25+ French prompt templates
- `nexus/llm/parsers.py`: Robust JSON/entity/score parsing

**Testing:**
- `tests/`: 233 pytest tests

## Naming Conventions

**Files:**
- Python modules: `snake_case.py` (e.g., `autonomous_loop.py`, `evidence_processor.py`)
- API routers: `snake_case.py` matching domain (e.g., `cases.py`, `evidence.py`)
- React pages: `PascalCase.tsx` (e.g., `Dashboard.tsx`, `Evidence.tsx`)
- React components: `PascalCase.tsx` (e.g., `Layout.tsx`, `Sidebar.tsx`)
- React hooks: `camelCase.ts` prefixed with `use` (e.g., `useCase.ts`)
- React stores: `camelCase.ts` suffixed with `Store` (e.g., `caseStore.ts`)

**Directories:**
- Python: `snake_case` (e.g., `nexus/core/`, `nexus/monitoring/`)
- React: `lowercase` (e.g., `web/src/pages/`, `web/src/hooks/`)

## Where to Add New Code

**New API Endpoint:**
1. Add route handler in the appropriate router file in `nexus/api/` (or create a new router file)
2. If new router file: register it in `nexus/main.py` (import + `app.include_router()`)
3. Add dependency function in `nexus/api/deps.py` if the endpoint needs a new service
4. Add Pydantic request/response models in `nexus/db/models.py`
5. Add API function in `web/src/api/client.ts`

**New Core Module:**
1. Create `nexus/core/{module_name}.py`
2. Follow the existing pattern: class with `__init__(self, db: Database, router: LLMRouter, chroma=None, neo4j=None)`
3. Add dependency injection function in `nexus/api/deps.py`
4. Wire into `nexus/core/autonomous_loop.py` if it should run autonomously

**New Database Table:**
1. Add DDL to `_CREATE_TABLES` in `nexus/db/sqlite_db.py`
2. Add indexes to `_CREATE_INDEXES` in `nexus/db/sqlite_db.py`
3. Add CRUD methods to the `Database` class in `nexus/db/sqlite_db.py`
4. Add Pydantic models (Base, Create, Update, Full) in `nexus/db/models.py`

**New React Page:**
1. Create `web/src/pages/{PageName}.tsx`
2. Add route in `web/src/App.tsx`
3. Add navigation entry in `web/src/components/Sidebar.tsx`
4. Add API functions in `web/src/api/client.ts`

**New LLM Task Type:**
1. Add enum value to `TaskType` in `nexus/llm/router.py`
2. Add routing entry to `_ROUTE_TABLE` in `nexus/llm/router.py` (model, timeout, heavy flag)
3. Add prompt template in `nexus/llm/prompts.py`
4. Add parser in `nexus/llm/parsers.py` if structured output needed

**New Forensic Module:**
1. Create `nexus/forensics/{module_name}.py`
2. Add dependency function in `nexus/api/deps.py`
3. Wire into `nexus/core/autonomous_loop.py` `_decide_forensic_analysis()` method
4. Create API router or add to existing `nexus/api/forensics.py`

**Utilities / Shared Helpers:**
- Evidence parsing: `nexus/ingest/`
- Text processing: `nexus/core/chunker.py`
- Entity resolution: `nexus/core/entity_extractor.py`

## Special Directories

**`data/`:**
- Purpose: All runtime data (database, uploads, exports, audit)
- Generated: Yes (created at startup by lifespan)
- Committed: Only `data/benchmark/` subdirectory committed (test data for 3 cold cases)

**`data/audit/`:**
- Purpose: Immutable audit trail with its own git repo
- Generated: Yes (by AuditService)
- Contains: JSONL log files per case + independent git history

**`data/benchmark/`:**
- Purpose: Test data for the 3 benchmark cold cases
- Generated: No (manually curated)
- Committed: Yes
- Subfolders: `kulik/` (14 files), `golden-state-killer/` (13 files), `affaire-moreau/` (15 files, 7 planted contradictions)

**`frontend/`:**
- Purpose: Legacy Streamlit frontend (16 pages)
- Status: Still functional but superseded by React frontend in `web/`

**`searxng/`:**
- Purpose: SearXNG search engine configuration
- Contains: Config files for the SearXNG Docker container

---

*Structure analysis: 2026-04-06*
