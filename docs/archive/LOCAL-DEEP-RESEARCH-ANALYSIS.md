# Local Deep Research (LDR) -- Comprehensive Analysis for NEXUS Cold Case Integration

**Repository:** https://github.com/LearningCircuit/local-deep-research
**Version analyzed:** Latest as of 2026-04-05
**Stars:** ~4.3k | **Forks:** ~408 | **License:** MIT
**Python:** >=3.11, <3.15 | **Build system:** PDM

---

## 1. Full Project Architecture

### Top-Level Structure

```
local-deep-research/
├── .github/                          # CI/CD workflows (57 total)
├── .pre-commit-hooks/                # Pre-commit hook scripts
├── .semgrep/rules/                   # Semgrep security rules
├── .zap/                             # OWASP ZAP config
├── community_benchmark_results/      # Benchmark data + COMMON_ISSUES.md
├── cookiecutter-docker/              # Docker project template
├── docs/                             # Full documentation suite
├── examples/                         # Usage examples
├── scripts/                          # CI and utility scripts
├── src/local_deep_research/          # Main Python package
├── tests/                            # Test suite (809+ test classes)
├── unraid-templates/                 # Unraid deployment
├── Dockerfile                        # Container build
├── docker-compose.yml                # 3-service stack (LDR + Ollama + SearXNG)
├── docker-compose.gpu.override.yml   # NVIDIA GPU override for Ollama
├── docker-compose.unraid.yml         # Unraid-specific compose
├── pyproject.toml                    # PDM build config + all dependencies
├── package.json / vite.config.js     # Frontend asset pipeline (Vite)
├── playwright.config.js              # E2E test config
└── eslint.config.js / lighthouserc.json  # Frontend quality tools
```

### Core Package: `src/local_deep_research/`

```
src/local_deep_research/
│
├── __init__.py
├── __version__.py
├── constants.py
├── exceptions.py
├── search_system.py                  # AdvancedSearchSystem orchestrator
├── search_system_factory.py          # Factory for strategies + search systems
├── report_generator.py               # IntegratedReportGenerator
├── citation_handler.py               # Citation management
│
├── config/                           # Configuration layer
│   ├── constants.py
│   ├── llm_config.py                 # LLM provider setup (get_llm)
│   ├── search_config.py              # Search engine config resolution
│   ├── paths.py                      # File path management
│   ├── thread_settings.py            # Concurrency settings
│   └── default_settings/             # Built-in defaults
│
├── defaults/                         # Default config files
│   ├── default_settings.json         # Master settings with ALL defaults
│   ├── .env.template                 # Environment variable template
│   ├── settings_*.json               # Per-engine/feature settings
│   ├── llm_providers/                # Provider-specific defaults
│   ├── research_library/             # Library defaults
│   └── settings/search_engines/      # Engine-specific defaults
│
├── llm/                              # LLM abstraction layer
│   ├── llm_registry.py              # Model registry
│   └── providers/
│       ├── base.py                   # Base provider interface
│       ├── openai_base.py            # OpenAI-compatible base
│       ├── auto_discovery.py         # Provider auto-detection
│       └── implementations/
│           ├── ollama.py             # Ollama provider <-- KEY FILE
│           ├── openai.py             # OpenAI
│           ├── anthropic.py          # Anthropic
│           ├── google.py             # Google Gemini
│           ├── openrouter.py         # OpenRouter
│           ├── lmstudio.py           # LM Studio
│           ├── custom_openai_endpoint.py  # Custom endpoints
│           ├── ionos.py              # IONOS
│           └── xai.py               # X.AI (Grok)
│
├── web_search_engines/               # Search engine abstraction
│   ├── search_engine_base.py         # Base class for all engines
│   ├── search_engine_factory.py      # Engine instantiation factory
│   ├── engine_registry.py            # Hardcoded engine->class mapping
│   ├── search_engines_config.py      # Runtime config resolution
│   ├── retriever_registry.py         # LangChain retriever registry
│   ├── default_search_engines.py     # Default engine configs
│   ├── engines/                      # 35 engine implementations
│   │   ├── search_engine_searxng.py  # SearXNG <-- KEY FILE
│   │   ├── search_engine_arxiv.py
│   │   ├── search_engine_pubmed.py
│   │   ├── search_engine_wikipedia.py
│   │   ├── search_engine_brave.py
│   │   ├── search_engine_github.py
│   │   ├── search_engine_wayback.py
│   │   ├── search_engine_guardian.py
│   │   ├── search_engine_semantic_scholar.py
│   │   ├── search_engine_ddg.py
│   │   ├── search_engine_tavily.py
│   │   ├── search_engine_elasticsearch.py
│   │   ├── search_engine_library.py      # Local document library
│   │   ├── search_engine_retriever.py    # LangChain retriever wrapper
│   │   ├── meta_search_engine.py         # "auto" aggregator
│   │   ├── parallel_search_engine.py     # Parallel multi-engine
│   │   ├── full_search.py                # Full content extraction
│   │   ├── local_embedding_manager.py    # Vector embedding manager
│   │   ├── search_engine_nasa_ads.py
│   │   ├── search_engine_openalex.py
│   │   ├── search_engine_pubchem.py
│   │   ├── search_engine_stackexchange.py
│   │   ├── search_engine_zenodo.py
│   │   ├── search_engine_gutenberg.py
│   │   ├── search_engine_openlibrary.py
│   │   ├── search_engine_wikinews.py
│   │   ├── search_engine_paperless.py
│   │   ├── search_engine_exa.py
│   │   ├── search_engine_mojeek.py
│   │   ├── search_engine_google_pse.py
│   │   ├── search_engine_serpapi.py
│   │   ├── search_engine_serper.py
│   │   ├── search_engine_scaleserp.py
│   │   └── search_engine_collection.py
│   └── rate_limiting/                # Adaptive rate limiter
│
├── advanced_search_system/           # Research strategy engine
│   ├── strategies/                   # 34 strategy implementations
│   │   ├── base_strategy.py
│   │   ├── source_based_strategy.py
│   │   ├── focused_iteration_strategy.py
│   │   ├── evidence_based_strategy.py
│   │   ├── langgraph_agent_strategy.py
│   │   ├── iterative_reasoning_strategy.py
│   │   ├── constrained_search_strategy.py
│   │   ├── parallel_search_strategy.py
│   │   ├── rapid_search_strategy.py
│   │   ├── mcp_strategy.py               # ReAct agentic strategy
│   │   ├── ... (24 more strategies)
│   │   └── followup/                     # Follow-up research
│   ├── answer_decoding/
│   ├── candidate_exploration/
│   ├── candidates/
│   ├── constraint_checking/
│   ├── constraints/
│   ├── evidence/
│   ├── filters/
│   ├── findings/
│   ├── knowledge/
│   ├── query_generation/
│   ├── questions/
│   ├── repositories/
│   ├── search_optimization/
│   ├── source_management/
│   └── tools/
│
├── web/                              # Flask Web UI
│   ├── app.py                        # Entry point (ldr-web)
│   ├── app_factory.py                # Flask app factory
│   ├── api.py                        # API endpoints
│   ├── server_config.py              # Server settings
│   ├── exceptions.py
│   ├── auth/                         # Authentication + CSRF
│   ├── database/                     # Per-user DB management
│   ├── models/                       # Data models
│   ├── queue/                        # Research job queue
│   ├── routes/                       # Route blueprints
│   ├── services/                     # Business logic
│   ├── static/                       # CSS/JS/images (Vite-built)
│   ├── templates/                    # Jinja2 HTML templates
│   ├── themes/                       # UI theme system
│   ├── utils/                        # Vite helper, etc.
│   └── warning_checks/              # Validation
│
├── api/                              # REST API layer (/api/v1)
├── mcp/                              # MCP server (ldr-mcp)
├── database/                         # SQLAlchemy + SQLCipher models
├── embeddings/                       # Embedding providers + splitters
├── research_library/                 # Document download + storage
├── library/download_management/      # PDF/web download managers
├── document_loaders/                 # File format loaders
├── content_fetcher/                  # Web content extraction
├── text_processing/                  # Text cleaning/chunking
├── text_optimization/                # Content optimization
├── citation_handlers/                # Citation formatting
├── exporters/                        # PDF/MD/LaTeX/Quarto/RIS export
├── domain_classifier/                # Query domain detection
├── followup_research/                # Follow-up research logic
├── news/                             # News subscription scheduler
├── notifications/                    # Webhook/email notifications
├── research_scheduler/               # Scheduled research tasks
├── metrics/                          # Usage analytics
├── security/                         # URL validation, SSRF protection
├── settings/                         # Settings management + logger
├── storage/                          # File storage abstraction
└── utilities/                        # LLM utils, general helpers
```

---

## 2. Dependencies and Requirements

### System Requirements

| Resource       | Minimum         | Recommended       |
|---------------|-----------------|-------------------|
| Python        | 3.11            | 3.11-3.14        |
| RAM           | 8 GB            | 16 GB             |
| GPU VRAM      | 4 GB (7B model) | 16-48 GB (26B+)  |
| Disk (LDR)    | 100 MB          | 100 MB            |
| Disk (SearXNG)| 1-2 GB          | 1-2 GB            |
| Disk (Model)  | 5-15 GB each    | 5-15 GB each      |

### Core Python Dependencies (from pyproject.toml)

**LLM Framework:**
- `langchain~=1.2`, `langchain-community~=0.4`, `langchain-core~=1.2`
- `langchain-ollama~=1.0` -- Ollama integration
- `langchain-openai~=1.1` -- OpenAI / compatible endpoints
- `langchain-anthropic~=1.3` -- Anthropic
- `langchain-experimental~=0.4`

**Search & Content Extraction:**
- `duckduckgo-search~=8.1`
- `beautifulsoup4~=4.14`, `trafilatura~=2.0`, `newspaper4k~=0.9`
- `crawl4ai~=0.8` -- AI-optimized web crawling
- `playwright~=1.58` -- Browser automation
- `arxiv~=2.4`, `wikipedia~=1.4`
- `google-search-results~=2.4` -- SerpAPI
- `elasticsearch~=9.3`

**Embeddings & RAG:**
- `sentence-transformers~=5.2` -- Local embedding models
- `faiss-cpu~=1.13` -- Vector similarity search
- `tiktoken~=0.12` -- Token counting

**Web Framework:**
- `flask~=3.1.3`, `flask-cors~=6.0`, `flask-socketio~=5.6.1`
- `flask-login~=0.6`, `flask-limiter~=4.1`, `flask-wtf~=1.2`

**Database:**
- `sqlalchemy~=2.0`, `alembic~=1.17`
- `sqlcipher3-binary~=0.6` / `sqlcipher3~=0.6` -- Encrypted databases

**Document Processing:**
- `pypdf~=6.9.1`, `pdfplumber~=0.11`, `unstructured~=0.18`
- `weasyprint~=68.1` -- PDF generation
- `pypandoc-binary~=1.16` -- Document conversion

**Data & Visualization:**
- `pandas~=3.0`, `matplotlib~=3.10`, `plotly~=6.5`

**Other Key Libraries:**
- `pydantic~=2.12`, `dynaconf~=3.2`, `loguru~=0.7`
- `aiohttp~=3.13`, `tenacity~=9.1` -- Async HTTP + retry
- `apscheduler~=3.11` -- Task scheduling
- `apprise~=1.9` -- Notifications (50+ services)
- `optuna~=4.7` -- Hyperparameter optimization
- `cryptography>=46.0.5` -- Encryption

**Optional:**
- `mcp[cli]~=1.2` -- MCP server support (install with `pip install "local-deep-research[mcp]"`)

### Entry Points

```
ldr       = local_deep_research.main:main        # CLI
ldr-web   = local_deep_research.web.app:main     # Web UI (port 5000)
ldr-mcp   = local_deep_research.mcp:run_server   # MCP server (STDIO)
```

---

## 3. How LDR Connects to Ollama

### Connection Architecture

LDR uses `langchain-ollama` (`ChatOllama`) as the bridge to Ollama. The implementation lives in:
- `src/local_deep_research/llm/providers/implementations/ollama.py` -- OllamaProvider class
- `src/local_deep_research/config/llm_config.py` -- get_llm() factory function

### Configuration

| Setting                              | Env Variable                          | Default               |
|--------------------------------------|---------------------------------------|----------------------|
| Provider selection                   | `LDR_LLM_PROVIDER`                   | `ollama`             |
| Model name                           | `LDR_LLM_MODEL`                      | `gemma3:12b`         |
| Ollama base URL                      | `LDR_LLM_OLLAMA_URL`                 | `http://localhost:11434` |
| Ollama API key (optional)            | `LDR_LLM_OLLAMA_API_KEY`             | (none)               |
| Context window (local models)        | `LDR_LLM_LOCAL_CONTEXT_WINDOW_SIZE`  | `18432`              |
| Temperature                          | `LDR_LLM_TEMPERATURE`                | `0.7`                |
| Enable thinking/reasoning            | `LDR_LLM_OLLAMA_ENABLE_THINKING`     | `true`               |

### Connection Flow

```
1. LDR reads provider setting --> "ollama"
2. Reads model name --> e.g., "nexus"
3. Reads Ollama URL --> e.g., "http://localhost:11434"
4. Validates model exists via GET /api/tags
   - Supports both new and legacy Ollama API response formats
   - If model not found, logs warning but continues
5. Creates ChatOllama instance with:
   - model=<model_name>
   - base_url=<ollama_url>
   - num_ctx=<context_window_size>
   - max_tokens=<80% of context window>
   - temperature=<temperature>
   - Optional: auth headers for proxied Ollama
6. Wraps with think-tag removal + rate limiting + token counting
```

### Docker Compose Integration

In `docker-compose.yml`, the Ollama service is pre-configured:

```yaml
ollama:
  image: ollama/ollama:latest
  entrypoint: "/scripts/ollama_entrypoint.sh ${MODEL:-gemma3:12b}"
  healthcheck:
    test: ["CMD", "ollama", "show", "${MODEL:-gemma3:12b}"]
  environment:
    OLLAMA_KEEP_ALIVE: '30m'
  volumes:
    - ollama_data:/root/.ollama

local-deep-research:
  environment:
    - LDR_LLM_OLLAMA_URL=http://ollama:11434
  depends_on:
    ollama:
      condition: service_healthy
```

The `MODEL` environment variable controls which model Ollama pulls on startup. The entrypoint script handles automatic model download.

---

## 4. Search Sources Supported

### Complete Engine Inventory (31 registered + runtime engines)

#### Free -- No API Key Required

| Engine              | Class                       | Category     | Specialization                                    |
|--------------------|-----------------------------|-------------|---------------------------------------------------|
| **SearXNG**        | SearXNGSearchEngine         | Web/Meta    | Meta-search aggregator (Google, Bing, etc.)       |
| **arXiv**          | ArxivSearchEngine           | Academic    | Physics, math, CS, biology preprints              |
| **PubMed**         | PubMedSearchEngine          | Academic    | Biomedical/life science literature                |
| **Semantic Scholar**| SemanticScholarSearchEngine | Academic    | Cross-disciplinary with citation data             |
| **Wikipedia**      | WikipediaSearchEngine       | Reference   | Encyclopedic background information               |
| **DuckDuckGo**     | DDGSearchEngine             | Web         | Privacy-focused (strict rate limits)              |
| **GitHub**         | GitHubSearchEngine          | Technical   | Code repositories and documentation               |
| **Wayback Machine**| WaybackSearchEngine         | Archive     | Historical web snapshots (Internet Archive)       |
| **The Guardian**   | GuardianSearchEngine        | News        | News articles                                     |
| **Wikinews**       | WikinewsSearchEngine        | News        | Community news                                    |
| **OpenAlex**       | OpenAlexSearchEngine        | Academic    | Open scholarly metadata                           |
| **NASA ADS**       | NasaAdsSearchEngine         | Academic    | Astrophysics data system                          |
| **PubChem**        | PubChemSearchEngine         | Scientific  | Chemistry compound data                           |
| **Stack Exchange** | StackExchangeSearchEngine   | Technical   | Q&A communities                                   |
| **Zenodo**         | ZenodoSearchEngine          | Academic    | Open research data repository                     |
| **Project Gutenberg**| GutenbergSearchEngine     | Reference   | Public domain books                               |
| **Open Library**   | OpenLibrarySearchEngine     | Reference   | Book metadata and full texts                      |

#### Premium -- API Key Required

| Engine          | Class                    | Category | Notes                        |
|----------------|--------------------------|----------|------------------------------|
| **Tavily**     | TavilySearchEngine       | Web      | AI-optimized for LLM apps    |
| **Brave**      | BraveSearchEngine        | Web      | Privacy-focused web search   |
| **SerpAPI**    | SerpApiSearchEngine      | Web      | Google results proxy         |
| **Google PSE** | GooglePSESearchEngine    | Web      | Google Programmable Search   |
| **Serper**     | SerperSearchEngine       | Web      | Google SERP API              |
| **ScaleSERP**  | ScaleSerpSearchEngine    | Web      | SERP data at scale           |
| **Exa**        | ExaSearchEngine          | Web      | Neural search engine         |
| **Mojeek**     | MojeekSearchEngine       | Web      | Independent crawler          |

#### Infrastructure / Self-Hosted

| Engine           | Class                        | Notes                              |
|-----------------|------------------------------|------------------------------------|
| **Elasticsearch**| ElasticsearchSearchEngine   | Self-hosted full-text search       |
| **Paperless**   | PaperlessSearchEngine        | Document management system         |

#### Meta / Composite Engines

| Engine              | Class                    | Notes                                |
|--------------------|--------------------------|--------------------------------------|
| **auto**           | MetaSearchEngine         | Intelligent multi-engine aggregation |
| **parallel**       | ParallelSearchEngine     | Concurrent multi-engine search       |
| **parallel_scientific** | ParallelSearchEngine | Academic-focused parallel search   |
| **collection_***   | CollectionSearchEngine   | User document collections (RAG)     |
| **library**        | LibrarySearchEngine      | Research library index               |

#### LangChain Retriever Integration

Any LangChain-compatible retriever can be registered at runtime:
- FAISS, Chroma, Pinecone, Weaviate, Elasticsearch
- Custom retrievers via `retriever_registry`
- Wrapped in `RetrieverSearchEngine` for unified interface

### SearXNG Integration Details

The `SearXNGSearchEngine` class connects via HTTP:

```
Configuration:
  - instance_url: base URL (e.g., http://localhost:8080 or http://searxng:8080)
  - categories: ["general"] by default
  - language: "en" by default
  - safesearch: configurable (0=off, 1=moderate, 2=strict)
  - engines: optional comma-separated list of backend engines
  - time_range: optional (day/week/month/year)
  - delay_between_requests: rate limiting

Query flow:
  1. Validates instance accessibility via safe_get (allows private IPs)
  2. Sends search request to {instance_url}/search with HTML format
  3. Parses results, filters out internal/error pages
  4. Respects rate limits between requests
  5. Returns structured results (title, URL, snippet)
```

---

## 5. RAG Capabilities

### Document Ingestion Pipeline

```
Upload (PDF/TXT/MD/HTML)
    |
    v
Text Extraction (pypdf, pdfplumber, unstructured)
    |
    v
Text Chunking (configurable: default 1000 chars, 200 overlap)
    |
    v
Embedding Generation (one of 3 providers)
    |
    v
FAISS Indexing (cosine/L2/dot product similarity)
    |
    v
Encrypted Storage (SQLCipher AES-256)
```

### Embedding Providers

| Provider             | Model                          | Requires    |
|---------------------|-------------------------------|-------------|
| Sentence Transformers| `all-MiniLM-L6-v2` (default) | Local only  |
| Ollama              | `nomic-embed-text` etc.       | Ollama running |
| OpenAI              | `text-embedding-3-small`      | API key     |

Configuration via settings:
- `LDR_SETTINGS_OLLAMA_EMBEDDINGS_*` for Ollama embedding model

### Collection System

- Documents organized into named collections
- Each collection gets its own FAISS index
- Collections can be used as search engines in research workflows
- Web UI at `http://localhost:5000/library` for upload and management
- Incremental indexing (add documents without rebuilding entire index)

### RAG in Research Workflows

Collections can be specified as the search tool for any research mode:
- Quick summary querying your own documents
- Detailed research combining local docs + web sources
- The `analyze_documents` MCP tool does collection-scoped semantic search + LLM summarization

### Knowledge Library

LDR automatically downloads and indexes sources found during research:
- PDF papers from arXiv, journals
- Web page content extraction
- Deduplication on storage
- Builds a persistent, searchable knowledge base over time

---

## 6. How Reports Are Generated

### Report Generation Pipeline

The `IntegratedReportGenerator` class (`report_generator.py`) orchestrates:

```
Phase 1: STRUCTURE DETERMINATION
   Input: Initial research findings from search system
   Process: LLM analyzes findings and proposes section hierarchy
   Output: List of sections, each with subsections and purpose statements
   (Source-related sections are filtered out automatically)

Phase 2: SECTION RESEARCH & GENERATION
   For each subsection:
     a. Build targeted research query with:
        - Subsection purpose statement
        - Context about related sections
        - Accumulated findings from prior sections (max 3 recent, 4000 chars)
        - "DO NOT REPEAT" instructions
     b. Execute single-iteration focused search
     c. Generate section content via LLM
     d. Add to accumulated_findings for context
   Note: questions_by_iteration is preserved from initial research
         to prevent duplicate searches

Phase 3: FINAL FORMATTING
   Assembles complete document:
     - Hierarchical table of contents
     - Research summary
     - All section content
     - Formatted source links with citations
     - Metadata (timestamp, source counts)
```

### Configuration Parameters

| Setting                    | Default | Description                            |
|---------------------------|---------|----------------------------------------|
| `searches_per_section`    | 2       | Search iterations per subsection       |
| `max_context_sections`    | 3       | Recent sections kept in context        |
| `max_context_chars`       | 4000    | Max characters from prior sections     |

### Export Formats

- **Markdown** -- Primary output format
- **PDF** -- Via WeasyPrint
- **LaTeX** -- Academic formatting
- **Quarto** -- Reproducible documents
- **HTML** -- Web-ready
- **RIS / BibTeX** -- Reference manager import
- **Plain text** -- Simple output
- **JSON** -- Programmatic consumption

### Citation System

- Citations tracked via `CitationHandler`
- Sources linked to specific claims in the report
- Multiple citation format styles supported
- Source deduplication across research iterations

---

## 7. Web UI

### Technology Stack

- **Backend:** Flask with SocketIO (real-time WebSocket updates)
- **Frontend:** Vite-built JavaScript/CSS assets with theme system
- **Database:** Per-user SQLCipher encrypted databases
- **Authentication:** Flask-Login with session management

### Access Point

```
URL: http://localhost:5000
Command: ldr-web
```

### Features

**Research Interface:**
- Query input with mode selection (quick/detailed/report/document analysis)
- Real-time progress updates via WebSocket
- Interactive result exploration
- Keyboard shortcuts (Ctrl+Shift combinations for mode switching)

**Research History:**
- Full history with search/filter
- Re-run previous queries
- Export results

**Settings Panel:**
- LLM provider/model selection (dropdown populated from Ollama /api/tags)
- Search engine configuration
- Strategy selection
- All settings editable in UI (overrides env vars)
- Settings locking via `LDR_LOCKED_SETTINGS`

**Research Library:**
- Document upload (PDF/TXT/MD/HTML)
- Collection management
- Semantic search across collections
- RAG integration

**Analytics Dashboard:**
- Research metrics and usage stats
- Token consumption tracking
- Search engine performance

**News Subscriptions:**
- Scheduled research on topics
- Notification delivery (50+ channels via Apprise)

### Security

- Per-user encrypted databases (AES-256 SQLCipher)
- CSRF protection (Flask-WTF)
- Rate limiting (Flask-Limiter, moving-window)
- Secure cookie handling (Secure flag for public IPs, HTTP for LAN)
- Path traversal prevention on static files
- No telemetry or analytics
- Server header stripping

### API Endpoints

```
/research/api     -- Research-specific API
/api/v1           -- REST API (CSRF-exempt for programmatic access)
/library          -- Research library management
/news             -- News subscription API
```

---

## 8. Integration with NEXUS (Gemma 4 26B Heretic) and SearXNG for Cold Case Investigation

### Current NEXUS Setup (from your project)

Your existing architecture:
- **Model:** `nexus` -- Gemma 4 26B Heretic (Q4_K_S GGUF) via Ollama
- **Context window:** 32,768 tokens
- **Temperature:** 0.3 (conservative for factual analysis)
- **System prompt:** 5-phase cold case analysis methodology
- **SearXNG:** Running on port 8888 with Google, DDG, Brave, Wikipedia, Wikidata, Bing, Archive.is, Reddit

### Integration Approach: Docker Compose

The most practical approach is to extend LDR's docker-compose.yml to use your existing Ollama and SearXNG instances.

**Option A: Use LDR's Bundled Stack (replace model)**

```yaml
# docker-compose.override.yml
services:
  ollama:
    entrypoint: "/scripts/ollama_entrypoint.sh nexus"
    healthcheck:
      test: ["CMD", "ollama", "show", "nexus"]
    # If using GGUF, you need to create the model first:
    # ollama create nexus -f /path/to/Modelfile.gemma4-heretic

  local-deep-research:
    environment:
      - LDR_LLM_PROVIDER=ollama
      - LDR_LLM_MODEL=nexus
      - LDR_LLM_OLLAMA_URL=http://ollama:11434
      - LDR_LLM_LOCAL_CONTEXT_WINDOW_SIZE=32768
      - LDR_LLM_TEMPERATURE=0.3
      - LDR_SEARCH_ENGINE_WEB_SEARXNG_DEFAULT_PARAMS_INSTANCE_URL=http://searxng:8080
      - LDR_SEARCH_TOOL=searxng
      - LDR_SEARCH_SEARCH_STRATEGY=evidence
```

**Option B: Point LDR to Your Existing External Services**

If Ollama and SearXNG already run on the host:

```yaml
services:
  local-deep-research:
    image: localdeepresearch/local-deep-research:latest
    ports:
      - "5000:5000"
    extra_hosts:
      - "host.docker.internal:host-gateway"
    environment:
      - LDR_LLM_PROVIDER=ollama
      - LDR_LLM_MODEL=nexus
      - LDR_LLM_OLLAMA_URL=http://host.docker.internal:11434
      - LDR_LLM_LOCAL_CONTEXT_WINDOW_SIZE=32768
      - LDR_LLM_TEMPERATURE=0.3
      - LDR_LLM_OLLAMA_ENABLE_THINKING=true
      - LDR_SEARCH_ENGINE_WEB_SEARXNG_DEFAULT_PARAMS_INSTANCE_URL=http://host.docker.internal:8888
      - LDR_SEARCH_TOOL=searxng
      - LDR_SEARCH_SEARCH_STRATEGY=evidence
      - LDR_SEARCH_ITERATIONS=3
      - LDR_SEARCH_QUESTIONS_PER_ITERATION=4
```

**Option C: pip install for direct host integration**

```bash
pip install local-deep-research

# Set environment variables
export LDR_LLM_PROVIDER=ollama
export LDR_LLM_MODEL=nexus
export LDR_LLM_OLLAMA_URL=http://localhost:11434
export LDR_LLM_LOCAL_CONTEXT_WINDOW_SIZE=32768
export LDR_LLM_TEMPERATURE=0.3
export LDR_SEARCH_ENGINE_WEB_SEARXNG_DEFAULT_PARAMS_INSTANCE_URL=http://localhost:8888
export LDR_SEARCH_TOOL=searxng
export LDR_SEARCH_SEARCH_STRATEGY=evidence

# Launch web UI
ldr-web
```

### SearXNG Configuration Alignment

Your SearXNG (`searxng/settings.yml`) runs on port 8888 with:
- Google, DuckDuckGo, Brave, Wikipedia, Wikidata, Bing, Archive.is, Reddit
- safe_search: 0 (disabled -- appropriate for cold case investigation)
- Language: French
- JSON format enabled

LDR connects to SearXNG and sends queries with format=html for parsing. Your SearXNG already has JSON format enabled which is a plus. The key env var is:

```
LDR_SEARCH_ENGINE_WEB_SEARXNG_DEFAULT_PARAMS_INSTANCE_URL=http://localhost:8888
```

Additional SearXNG tuning for cold case work:

```
LDR_SEARCH_ENGINE_WEB_SEARXNG_DEFAULT_PARAMS_LANGUAGE=fr
LDR_SEARCH_ENGINE_WEB_SEARXNG_DEFAULT_PARAMS_CATEGORIES=general
LDR_SEARCH_ENGINE_WEB_SEARXNG_DEFAULT_PARAMS_TIME_RANGE=      # empty = all time
LDR_SEARCH_ENGINE_WEB_SEARXNG_DEFAULT_PARAMS_SAFESEARCH=0
```

### Recommended Research Strategy for Cold Cases

**Primary strategy: `evidence`**
- Best for verification-critical research
- Builds evidence chains with supporting/contradicting data
- Matches NEXUS's Phase 4 hypothesis scoring methodology
- Slower but highest rigor

**Alternative: `focused-iteration`**
- Achieves ~95% accuracy on benchmarks
- Iteratively refines understanding
- Good for initial case exploration

**For active OSINT gathering: `mcp` (ReAct)**
- Agentic reasoning loop (THOUGHT -> ACTION -> OBSERVATION)
- Can call multiple search engines dynamically
- Can download and analyze web content
- Can chain sub-research tasks (focused_research within ReAct)
- Best for multi-source OSINT correlation

### Programmatic Integration with Your Existing Python Code

Your `README.md` already shows the Ollama API pattern. LDR adds structured research on top:

```python
# Using LDR's Python SDK alongside NEXUS
from local_deep_research import quick_summary, detailed_research, generate_report

# Quick OSINT sweep
result = quick_summary(
    query="Jean Dupont disparition mars 2019 Paris",
    search_tool="searxng",
    strategy="evidence",
    llms=None,          # Uses default (nexus via Ollama)
    provider="ollama"
)

# Deep investigation
result = detailed_research(
    query="connexions financieres suspectes Jean Dupont 2018-2019",
    search_tool="searxng",
    strategy="focused-iteration",
    iterations=5,
    questions_per_iteration=4
)

# Full report with citations
report = generate_report(
    query="Analyse complete cold case disparition Jean Dupont 2019",
    search_tool="searxng",
    searches_per_section=3
)
```

### MCP Server Integration (for Claude Desktop/Code)

If you want Claude Desktop or Claude Code to use LDR + NEXUS:

```json
{
  "mcpServers": {
    "local-deep-research": {
      "command": "ldr-mcp",
      "env": {
        "LDR_LLM_PROVIDER": "ollama",
        "LDR_LLM_MODEL": "nexus",
        "LDR_LLM_OLLAMA_URL": "http://localhost:11434",
        "LDR_LLM_LOCAL_CONTEXT_WINDOW_SIZE": "32768",
        "LDR_LLM_TEMPERATURE": "0.3",
        "LDR_SEARCH_ENGINE_WEB_SEARXNG_DEFAULT_PARAMS_INSTANCE_URL": "http://localhost:8888",
        "LDR_SEARCH_TOOL": "searxng"
      }
    }
  }
}
```

This exposes 5 research tools + 3 discovery tools to Claude, all powered by NEXUS.

### Cold Case Investigation Workflow with LDR

```
                     Cold Case Data (police reports, testimony, OSINT)
                                      |
                                      v
                    +------ LDR Web UI (localhost:5000) ------+
                    |                                          |
                    |  Strategy: evidence / mcp                |
                    |  Model: nexus (Gemma 4 26B Heretic)     |
                    |  Search: SearXNG (port 8888)            |
                    |                                          |
                    +------------------------------------------+
                         |              |              |
                         v              v              v
                    SearXNG         arXiv         Wayback Machine
                   (Google,       (forensic       (historical
                    Bing,         papers)         web snapshots)
                    Brave,
                    Wikipedia,
                    Reddit,
                    Archive.is)
                         |              |              |
                         v              v              v
                    +------------------------------------------+
                    |        NEXUS Analysis Engine              |
                    |  Phase 1: Ingestion                      |
                    |  Phase 2: Chronology & Mapping           |
                    |  Phase 3: Link Analysis                  |
                    |  Phase 4: Hypothesis Scoring             |
                    |  Phase 5: Synthesis                      |
                    +------------------------------------------+
                                      |
                         +------------+------------+
                         |            |            |
                         v            v            v
                    Report        Knowledge     Research
                    (PDF/MD)      Library       History
                                  (RAG for     (searchable
                                  future       archive)
                                  cases)
```

### Key Configuration Values for Cold Case Work

```bash
# Model settings (conservative for factual analysis)
LDR_LLM_PROVIDER=ollama
LDR_LLM_MODEL=nexus
LDR_LLM_TEMPERATURE=0.3                    # Low randomness for factual work
LDR_LLM_LOCAL_CONTEXT_WINDOW_SIZE=32768    # Match your Modelfile num_ctx
LDR_LLM_OLLAMA_ENABLE_THINKING=true        # Enable chain-of-thought

# Search settings (thorough investigation)
LDR_SEARCH_TOOL=searxng                    # Or "auto" for multi-engine
LDR_SEARCH_SEARCH_STRATEGY=evidence        # Rigorous evidence checking
LDR_SEARCH_ITERATIONS=4                    # More iterations = more thorough
LDR_SEARCH_QUESTIONS_PER_ITERATION=5       # More angles per iteration
LDR_SEARCH_REGION=FR                       # French region
LDR_SEARCH_TIME_PERIOD=                    # Empty = search all time periods

# Report settings
LDR_REPORT_SEARCHES_PER_SECTION=3          # Thorough per-section research
LDR_REPORT_MAX_CONTEXT_CHARS=6000          # Larger context for complex cases

# SearXNG connection
LDR_SEARCH_ENGINE_WEB_SEARXNG_DEFAULT_PARAMS_INSTANCE_URL=http://localhost:8888
LDR_SEARCH_ENGINE_WEB_SEARXNG_DEFAULT_PARAMS_SAFESEARCH=0
LDR_SEARCH_ENGINE_WEB_SEARXNG_DEFAULT_PARAMS_LANGUAGE=fr
```

---

## 9. Limitations and Known Issues

### Model-Related

1. **Context window accumulation:** Each research iteration accumulates context from previous searches. With 32K context on nexus, you must be careful with iteration count. 4 iterations with 5 questions each can approach context limits. LDR calculates max_tokens as 80% of context window.

2. **26B model speed:** Gemma 4 26B at Q4_K_S will be significantly slower than the default gemma3:12b. LDR has timeouts -- research operations may need patience. Each subsection of a report requires a separate LLM call.

3. **Quantization tradeoffs:** Q4_K_S quantization may produce occasional hallucinations or instruction-following failures compared to higher quantization levels. The COMMON_ISSUES.md explicitly warns about this. For cold case analysis where accuracy is paramount, Q5_K_M or Q6_K would be more reliable if VRAM allows.

4. **Thinking/reasoning overhead:** With `enable_thinking=true`, Ollama models produce internal reasoning tokens that consume context but are stripped from output. This reduces effective context for actual content.

5. **Default model assumption:** LDR defaults to `gemma3:12b`. Many internal prompts and question generation logic are tuned for this model size. A 26B model should handle these well, but an uncensored/abliterated model may occasionally produce output that doesn't match expected formatting patterns.

### Search-Related

6. **SearXNG rate limiting:** Heavy automated querying triggers SearXNG's protective rate limiting, resulting in empty results. Mitigation: reduce `questions_per_iteration`, increase delay_between_requests, or run your own SearXNG (which you already do).

7. **DuckDuckGo rate limits:** DDG aggressively rate-limits automated queries (HTTP 202). Your SearXNG config includes DDG as a backend, which may cascade these limits through SearXNG.

8. **Search result quality variance:** Academic engines (arXiv, PubMed) use keyword matching, while SearXNG uses semantic search. LDR can optionally apply LLM-based relevance filtering for academic engines, but this adds latency. For cold case work, SearXNG's semantic capability is more valuable.

9. **No direct court record / law enforcement database access:** LDR's search engines cover academic, web, and some specialized sources, but there are no integrations for PACER, law enforcement databases, or court record systems. These would need custom LangChain retrievers.

### Infrastructure-Related

10. **Memory consumption:** Running NEXUS (26B Q4_K_S needs ~16GB RAM/VRAM) + LDR + SearXNG simultaneously requires substantial resources. LDR itself uses sentence-transformers for embeddings which loads another model into RAM.

11. **No GPU acceleration for embeddings:** The default `sentence-transformers` runs on CPU. For faster RAG indexing, switch to Ollama embeddings (`nomic-embed-text`), which can use your GPU.

12. **SQLCipher on Windows:** The `sqlcipher3-binary` wheel is only pre-built for Linux x86_64. On Windows, `sqlcipher3` must be compiled from source or the encryption feature may not work. You may need to install SQLCipher development libraries manually.

13. **Docker networking on Windows:** The `host.docker.internal` bridge works differently on Windows Docker Desktop vs WSL2. If Ollama runs natively on Windows and LDR in Docker, connectivity may require additional configuration.

### Architectural Limitations

14. **No persistent conversation context:** LDR treats each research query as independent. It does not maintain case-level conversation context across multiple queries. Your NEXUS system prompt's "continuous re-injection" pattern (feeding new data back) would need to be implemented at the application level, not within LDR itself.

15. **No graph database integration:** Your architecture diagram mentions Neo4j for relationship graphs. LDR does not have native graph database support. The relationship mapping (Phase 3 of NEXUS) would need to happen outside LDR or via a custom LangChain retriever connected to Neo4j.

16. **Linear report structure:** LDR generates traditional linear reports (sections/subsections with TOC). It cannot produce the structured NEXUS analysis format (hypothesis scoring tables, relationship graphs, confidence scores) natively. The LLM (nexus) may produce that format in the content, but the report wrapper will be standard markdown.

17. **Single-language assumption in prompts:** LDR's internal question generation and synthesis prompts are in English. While the LLM can respond in French and SearXNG is configured for French, the intermediate research questions generated by LDR's question generator will be in English. This may affect French-language search quality through SearXNG.

18. **No real-time collaborative investigation:** LDR is single-user per research session. Multiple analysts cannot collaborate on the same research thread simultaneously, though the multi-user authentication system keeps separate user databases.

19. **Search cancellation issues:** Known bug (GitHub issue #324) where cancelling a search does not stop background processing. Long-running evidence-based research with many iterations may continue consuming resources after cancellation.

20. **Ollama keep-alive interaction:** LDR creates a new ChatOllama instance per research operation. Combined with `OLLAMA_KEEP_ALIVE=30m` in Docker, the model stays loaded. But if multiple research tasks queue up, they share the same Ollama instance sequentially, not in parallel (Ollama processes one request at a time per model by default).

---

## Appendix: Key File Paths for Integration Development

```
# Configuration entry points
src/local_deep_research/config/llm_config.py        -- LLM provider factory
src/local_deep_research/defaults/default_settings.json -- All default values

# Ollama provider
src/local_deep_research/llm/providers/implementations/ollama.py

# SearXNG engine
src/local_deep_research/web_search_engines/engines/search_engine_searxng.py

# Search system orchestrator
src/local_deep_research/search_system.py
src/local_deep_research/search_system_factory.py

# Strategy selection
src/local_deep_research/advanced_search_system/strategies/

# Report generation
src/local_deep_research/report_generator.py

# Web UI factory
src/local_deep_research/web/app_factory.py

# Engine registry (all engines listed)
src/local_deep_research/web_search_engines/engine_registry.py

# Docker setup
docker-compose.yml
docker-compose.gpu.override.yml

# Documentation
docs/CONFIGURATION.md          -- Full config reference
docs/CUSTOM_LLM_INTEGRATION.md -- Custom LLM guide
docs/SearXNG-Setup.md          -- SearXNG setup
docs/library-and-rag.md        -- RAG capabilities
docs/search-engines.md         -- Engine guide
docs/mcp-server.md             -- MCP server reference
docs/faq.md                    -- Troubleshooting
```
