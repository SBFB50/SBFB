# Technology Stack

**Analysis Date:** 2026-04-06

## Languages

**Primary:**
- Python 3.13 - Backend (FastAPI), Streamlit frontend, all core logic (`nexus/`)
- TypeScript ~5.9.3 - React frontend (`web/src/`)

**Secondary:**
- Cypher - Neo4j graph queries (`nexus/db/neo4j_db.py`)
- SQL - SQLite DDL and queries (`nexus/db/sqlite_db.py`)
- YAML - Docker Compose, SearXNG config (`docker-compose.yml`, `searxng/settings.yml`)

## Runtime

**Environment:**
- Python 3.13 via Conda (environment name: `nexus`)
- Node.js (version unspecified, required for React frontend in `web/`)
- Docker Desktop (Windows 11, WSL2) for Neo4j, ChromaDB, Robin containers
- Ollama (local, port 11434) with CUDA/RTX 5080 16GB VRAM
- Windows 11 Pro (10.0.26200)

**Package Manager:**
- pip (`requirements.txt`) - Python dependencies
- npm (`package-lock.json` present in `web/`) - Node.js dependencies

**Lockfiles:**
- `web/package-lock.json` - present
- No Python lockfile (no `pip freeze` output, no `poetry.lock`, no `conda-lock.yml`)

## Frameworks

**Core:**
- FastAPI >=0.115,<0.116 - REST API backend (115+ endpoints across 22 routers)
- React 19.2.4 - Primary frontend SPA (`web/`)
- Streamlit >=1.44,<2 - Legacy frontend (`frontend/`), 16 pages

**Testing:**
- pytest (with `pytest.ini`, asyncio_mode=auto) - 233 tests in `tests/`

**Build/Dev:**
- Vite 8.0.1 - Frontend dev server and bundler (`web/vite.config.ts`)
- uvicorn[standard] >=0.34,<0.35 - ASGI server for FastAPI
- ESLint 9.39.4 - Frontend linting (`web/`)
- TypeScript ~5.9.3 - Type checking (`web/tsconfig.json`)

## Key Dependencies

**Critical (Backend):**
- `ollama` >=0.4,<1 - Official Python SDK for Ollama LLM interactions (`nexus/llm/ollama_client.py`)
- `neo4j` >=5.28,<6 - Async Neo4j driver for graph database (`nexus/db/neo4j_db.py`)
- `chromadb` >=0.6,<1 - Vector store client for RAG embeddings (`nexus/db/chroma_db.py`)
- `aiosqlite` >=0.21,<1 - Async SQLite3 driver (`nexus/db/sqlite_db.py`)
- `pydantic` >=2.11,<3 - Data validation, API schemas (`nexus/db/models.py`)
- `pydantic-settings` >=2.7,<3 - Environment configuration (`nexus/config.py`)
- `httpx` >=0.28,<1 - Async HTTP client for SearXNG, Nominatim, OSRM

**Critical (Frontend):**
- `@tanstack/react-query` 5.96.2 - Data fetching and caching (`web/src/api/client.ts`)
- `axios` 1.14.0 - HTTP client to FastAPI backend (`web/src/api/client.ts`)
- `zustand` 5.0.12 - State management (`web/src/stores/`)
- `react-router-dom` 7.14.0 - Client-side routing (`web/src/App.tsx`)
- `tailwindcss` 4.2.2 - Utility-first CSS framework

**AI/ML:**
- `transformers` >=4.46,<5 - DINOv2 + CLIP visual embeddings (`nexus/vision/embeddings.py`)
- `torch` >=2.5 - PyTorch backend for vision models
- `torchvision` >=0.20 - Image processing for visual embeddings
- GLiNER (`urchade/gliner_multi-v2.1`) - NER extraction, CPU-only (`nexus/core/entity_extractor.py`)
- RapidFuzz - Entity deduplication via Jaro-Winkler (threshold 82%)

**Infrastructure:**
- `APScheduler` >=3.11,<4 - Async job scheduling for monitoring (`nexus/monitoring/scheduler.py`)
- `tenacity` >=9.0,<10 - Retry logic with exponential backoff (`nexus/llm/ollama_client.py`)
- `loguru` >=0.7,<1 - Structured logging throughout backend
- `python-dotenv` >=1.1,<2 - Environment variable loading

**Ingestion/Export:**
- `PyMuPDF` (fitz) >=1.25,<2 - PDF text and image extraction (`nexus/ingest/pdf_parser.py`)
- `weasyprint` >=63,<64 - HTML-to-PDF report generation (`nexus/export/pdf_export.py`)
- `Jinja2` >=3.1,<4 - HTML templating for reports (`nexus/export/pdf_export.py`)
- `Pillow` >=11,<12 - Image processing

**OSINT:**
- `holehe` >=1.61,<2 - Email existence check across 120+ services (`nexus/recon/holehe_recon.py`)
- `python-whois` >=0.9,<1 - WHOIS domain lookups (`nexus/recon/domain_recon.py`)

**Science:**
- `scipy` >=1.14,<2 - Agglomerative clustering for RAPTOR summaries (`nexus/core/summary_tree.py`), forensic physics (`nexus/forensics/physics_sim.py`)
- `numpy` - Numerical operations (transitive via scipy, torch)

**Visualization (Frontend React):**
- `recharts` 3.8.1 - Charts and graphs (`web/src/pages/Dashboard.tsx`)
- `react-force-graph-2d` 1.29.1 / `react-force-graph-3d` 1.29.1 - Knowledge graph visualization
- `leaflet` 1.9.4 + `react-leaflet` 5.0.0 - Interactive maps

**Visualization (Streamlit Legacy):**
- `plotly` >=6.0,<7 - Interactive charts
- `streamlit-agraph` >=0.0.45 - Graph visualization
- `streamlit-folium` >=0.23,<1 + `folium` >=0.19,<1 - Maps

**UI Components (Frontend React):**
- `@radix-ui/react-dialog` 1.1.15 - Modal dialogs
- `@radix-ui/react-dropdown-menu` 2.1.16 - Dropdown menus
- `@radix-ui/react-tabs` 1.1.13 - Tab navigation
- `lucide-react` 1.7.0 - Icon set

**Optional (commented out in requirements.txt):**
- `phiflow` >=3.3,<4 - Differentiable physics simulation
- `the-well` >=0.1 - PolymathicAI physics datasets (15TB on HuggingFace)

## LLM Models (Ollama)

**Model Fleet (configured in `nexus/config.py`):**

| Setting | Model | Size | Role | VRAM | Heavy? |
|---|---|---|---|---|---|
| `model_deep` | `nexus` (Gemma 4 26B Heretic Q4_K_S) | 26B | Deep analysis, hypothesis scoring, reports, suspect profiles | High | Yes |
| `model_reasoning` | `huihui_ai/deepseek-r1-abliterated:14b` | 14B | CoT reasoning, contradiction detection, logic verification | High | Yes |
| `model_fast` | `gemma4:e4b` | 4B | Entity extraction, summaries, reformulation, JSON structuring | Low | No |
| `model_embedding` | `nomic-embed-text` | - | Vector embeddings for RAG | Low | No |
| `model_vision` | `gemma4:e4b` | 4B | Quick image description | Low | No |
| `model_vision_deep` | `qwen3-vl:8b` | 8B | Deep scene analysis, image comparison, trace analysis | Medium | Yes |
| `model_audio` | `voxtral-mini:4b` | 4B | Audio transcription | Medium | Yes |

**VRAM Management:**
- `asyncio.Lock` serializes heavy model calls (only one large model in VRAM at a time)
- Light models (`gemma4:e4b`, `nomic-embed-text`) coexist in VRAM
- `keep_alive="10m"` keeps models loaded between calls
- Context window: `num_ctx=8192` for all inference; Modelfiles set `num_ctx=32768`

**Custom Modelfiles:**
- `Modelfile` - deepseek-r1-abliterated:14b with French NEXUS system prompt
- `Modelfile.gemma4-heretic` - Custom Gemma 4 26B from local GGUF (`models/gemma-4-26b-a4b-it-heretic.q4_k_s.gguf`)
- `Modelfile.qwen3-30b` - Qwen3 30B (alternative deep model, not default)

**Performance:**
- ~20.4 tok/s on `nexus` 26B Q4_K_S (RTX 5080)
- GLiNER NER: ~0.08s per text on CPU, zero VRAM

## Configuration

**Environment:**
- Configuration via `pydantic-settings` in `nexus/config.py` (singleton `settings`)
- Loads from `.env` file (present in project root)
- All settings have defaults; `.env` overrides are optional
- Key settings: Ollama URL, model names, Neo4j credentials, ChromaDB host/port, SearXNG URL, Robin URL, storage paths, scheduling intervals, RAG parameters

**Build:**
- `web/vite.config.ts` - Vite config with React plugin, Tailwind CSS plugin, proxy `/api` to `http://localhost:8000`
- `web/tsconfig.json` - References `tsconfig.app.json` and `tsconfig.node.json`
- `pytest.ini` - asyncio_mode=auto, testpaths=tests
- `docker-compose.yml` - Neo4j 5-community, ChromaDB latest, Robin latest

## Platform Requirements

**Development:**
- Windows 11 with Docker Desktop (WSL2)
- NVIDIA RTX 5080 (16GB VRAM) or equivalent GPU
- Conda for Python 3.13 environment management
- Node.js + npm for React frontend
- Ollama installed locally with models pulled
- SearXNG running on host (port 8888, not dockerized)
- Docker Compose for Neo4j, ChromaDB, Robin

**Production:**
- No production deployment configuration exists
- System designed for single-machine local operation
- All services communicate via localhost

## Port Allocation

| Port | Service | Notes |
|---|---|---|
| 8000 | FastAPI backend | Main API |
| 3002 | Vite dev server (React) | Proxies `/api` to :8000 |
| 8501 | Streamlit (legacy) | Direct API calls to :8000 |
| 11434 | Ollama | LLM inference |
| 7474 | Neo4j Browser | Web UI |
| 7687 | Neo4j Bolt | Driver protocol |
| 8100 | ChromaDB | Remapped from container :8000 |
| 8888 | SearXNG | Clearweb search (host-native) |
| 8502 | Robin Streamlit UI | Dark web search (container, remapped from :8501) |

## Startup Sequence

Defined in `nexus/main.py` lifespan:
1. Initialize Loguru log interception
2. `init_db()` - Create SQLite schema (idempotent)
3. Create required directories (`uploads/`, `reports/`, `backups/`)
4. Create `OllamaClient` + `LLMRouter` singletons on `app.state`
5. Connect `Neo4jClient` (optional, degraded mode if unavailable)
6. Connect `ChromaClient` (optional, degraded mode if unavailable)
7. Start `MonitoringScheduler` (APScheduler, optional)
8. Start `InvestigationManager` (autonomous OODA loops, optional)

All optional services use graceful degradation -- system runs with reduced functionality if Neo4j, ChromaDB, or monitoring fail to connect.

---

*Stack analysis: 2026-04-06*
