# NEXUS GOV

**Cold Case Investigation + Political Intelligence System**

An autonomous, persistent investigation system that runs 24/7 -- analyzing cold cases, monitoring French political activity, detecting contradictions, and sharing GPU power across a distributed citizen network. Not a chatbot. An investigator.

---

## What It Does

NEXUS is two systems in one, sharing a common AI backbone:

**Cold Case Investigation** -- Ingest evidence (PDFs, images, audio, text), extract entities with GLiNER NER, build knowledge graphs in Neo4j, detect contradictions between testimonies, generate and score hypotheses using ACH (Analysis of Competing Hypotheses), run OSINT recon (WHOIS, social, Holehe), search clearweb (SearXNG) and dark web (Robin/Tor), and converge toward the truth autonomously.

**Political Intelligence (GOV)** -- Monitor all 925 French deputies and 348 senators in real time. Scrape parliamentary votes, positions, press articles, social media (Twitter, Instagram, Facebook, TikTok, YouTube), detect contradictions between public statements and voting records, track affairs and asset declarations (HATVP), sync EU Parliament and EUR-Lex legislation, generate weekly recaps, and publish newsletters -- all autonomously with 31 specialized workers.

**Distributed GPU Network** -- Citizens contribute their GPU power. The server orchestrates LLM tasks, contributors compute them locally via Ollama. Auto-scaling model selection adapts to total available VRAM (from 12B solo to 405B Petals swarm with 50+ GPUs). Results are cryptographically verified (Ed25519 + model digest + logprob fingerprinting).

---

## Key Numbers

| Metric | Value |
|---|---|
| Lines of code | ~71K |
| GOV workers | 31 (scraping, analysis, social media, EU, vision, newsletters) |
| Cold case workers | 20 (event-driven reactive pipeline) |
| Frontend pages | 14 (Dashboard, Evidence, Entities, Hypotheses, Graph, Timeline, Investigation, Suspects, Wiki, Reports, Images, Benchmark, Government, Network) |
| GOV tabs | 18 (Politicians, Hemicycle, Map, Comparator, Network, Contradictions, Press, Social, Videos, Statistics, Alerts, Affairs, Declarations, Legislation, Timeline, Search, Recap, Pipeline) |
| Network tabs | 6 (Statistics, Leaderboard, Nodes, Swarm Petals, Contribute, Badges) |
| API routers | 27 |
| Tests | 756 test functions across 29 files |
| Distributed GPU phases | 9 (registry, auto-scaling, worker client, exo split, dashboard, proof-of-computation, Petals 405B, public swarm, cr-sqlite sync) |

---

## Architecture

```
                    ┌─────────────────────────────────────┐
                    │          React Frontend              │
                    │   Vite + TypeScript + Tailwind       │
                    │   14 pages, 24 tabs, dark theme      │
                    │   Port 3002                          │
                    └──────────────┬──────────────────────┘
                                   │ REST + SSE
                    ┌──────────────┴──────────────────────┐
                    │          FastAPI Backend              │
                    │   27 routers, 7-step startup          │
                    │   EventBus + VRAMScheduler            │
                    │   Port 8000                           │
                    ├──────────────────────────────────────┤
                    │  Cold Case    │  GOV Module           │
                    │  20 workers   │  31 workers           │
                    │  5 benchmarks │  925 deputies          │
                    │               │  348 senators          │
                    ├──────────────────────────────────────┤
                    │  Distributed GPU Compute              │
                    │  Task queue, model selector,          │
                    │  result validator, leaderboard        │
                    └──┬─────────┬─────────┬──────────┬───┘
                       │         │         │          │
                  ┌────┴──┐ ┌───┴───┐ ┌───┴───┐ ┌───┴────┐
                  │Ollama │ │Neo4j  │ │Chroma │ │SearXNG │
                  │LLM    │ │Graph  │ │Vector │ │Search  │
                  │:11434 │ │:7474  │ │:8100  │ │:8888   │
                  └───────┘ └───────┘ └───────┘ └────────┘
                       │
              ┌────────┼────────┐
              │        │        │
         Contributor  Contributor  Contributor
         RTX 5080    RTX 4090    Mac M4 Pro
         nexus-worker nexus-worker nexus-worker
         (Ollama)    (Ollama)    (Ollama)
```

### Startup Sequence (7 steps)

1. SQLite database (FTS5 + WAL + GOV tables + Compute tables)
2. Neo4j graph database (constraints, APOC)
3. ChromaDB vector store (7 collections)
4. GLiNER entity extractor (CPU singleton, zero VRAM)
5. Investigation manager (20 reactive workers, EventBus, VRAMScheduler)
6. GOV module (31 workers, parliamentary scraping)
7. Distributed GPU compute (task dispatcher, model selector)

---

## Quick Start

### Prerequisites

- **Windows 11** (primary), Linux/macOS supported
- **Python 3.13** (conda or venv)
- **Node.js** (for React frontend)
- **Docker Desktop** (for Neo4j, ChromaDB, Robin)
- **Ollama** installed and running
- **NVIDIA GPU** with 16+ GB VRAM (RTX 4060+ recommended)

### One Command

```bash
# Windows
start.bat

# Or directly
python start_nexus.py
```

The launcher (Rich terminal UI) handles everything automatically:

1. Starts **Docker** services (Neo4j + ChromaDB + Robin)
2. Checks **SearXNG** (port 8888)
3. Verifies **Ollama** models (auto-pulls if missing)
   - `juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m` -- Main LLM (MoE 26B, 4B active params)
   - `nomic-embed-text` -- Embeddings (137MB)
4. Starts **Frontend** (Vite, port 3002)
5. Starts **Backend** (FastAPI, port 8000) with live logs

Browser opens automatically when ready.

### Manual Start

```bash
# Docker services
docker compose up -d

# Backend
uvicorn nexus.main:app --host 0.0.0.0 --port 8000

# Frontend
cd web && npx vite --host 0.0.0.0 --port 3002

# Tests
python -m pytest tests/ -v
```

### Endpoints

| Service | URL |
|---|---|
| Frontend | http://localhost:3002 |
| Backend API | http://localhost:8000 |
| Swagger docs | http://localhost:8000/docs |
| ReDoc | http://localhost:8000/redoc |
| Neo4j Browser | http://localhost:7474 |
| Network page | http://localhost:3002/network |

---

## Distributed GPU Computing

Citizens share their GPU power to make the political AI more capable. The server orchestrates, contributors compute, everyone sees the results.

### How It Works

1. **Server** maintains a task queue (LLM prompts from GOV workers: contradiction detection, sentiment analysis, summaries, etc.)
2. **Contributors** run `nexus-worker` on their machines with Ollama installed
3. Worker pulls tasks from the server, runs inference locally, submits results
4. **Server validates** every result (Ed25519 signature + model digest + logprob fingerprinting)
5. **Model auto-scales** based on total available VRAM across all connected nodes

### Auto-Scaling Model Tiers

| Total VRAM | Model | Tier |
|---|---|---|
| 0 GB | Gemma 4 12B q4 | Basic |
| 14 GB | Gemma 4 26B q4 | Standard |
| 40 GB | Llama 3.1 70B q4 | Advanced |
| 80 GB | Qwen 2.5 110B q4 | Pro |
| 150 GB | Llama 3.1 405B q2 (Petals) | Ultra |
| 300 GB | Llama 3.1 405B (Petals) | Maximum |

When contributors connect or disconnect, the system automatically recalculates the best model and notifies all nodes to pull it. Graceful transition: in-flight tasks finish on the old model.

### Security: 3-Layer Verification

- **Layer 1 -- Ed25519 Signature**: Cryptographic proof of WHO submitted the result (non-repudiation)
- **Layer 2 -- Ollama Model Digest**: SHA256 hash proves the correct model is loaded on the contributor's machine
- **Layer 3 -- Logprob Fingerprinting**: Each model has a unique probability distribution. 8 calibration queries identify the model with >95% accuracy (based on [LLMmap, USENIX Security 2025](https://github.com/pasquini-dario/LLMmap))
- **Spot-checking**: 5% of tasks are re-executed on a trusted GPU (BOINC pattern). Divergent results flag the node. Nodes with >10% divergence are auto-banned.

Total verification overhead: <0.01% in time, ~5% in compute. Compare with zkML: 180x overhead (unusable for LLMs).

### Petals 405B Swarm (Phase 7)

With 50+ contributors on French fiber (~8ms average RTT), the system can split Llama 3.1 405B across the swarm using [Petals](https://github.com/bigscience-workshop/petals):

- Single request: ~2 tok/s (500ms per token due to network hops)
- Batch 50 requests: ~80 tok/s aggregate (pipeline parallelism)
- Full analysis of 1,145 politicians: ~12 minutes
- Quality comparable to GPT-4 class models

Fault tolerance: when contributors disconnect, Petals auto-reroutes blocks. The ModelSelector auto-downgrades the model if VRAM drops below thresholds.

---

## Contributing GPU Power

Three commands to start contributing:

```bash
# 1. Install the worker
pip install nexus-worker

# 2. Register your GPU
nexus-worker register --server nexusgov.fr --name "YourPseudo"

# 3. Start computing
nexus-worker start
```

The worker displays a Rich terminal dashboard showing your GPU stats, current task, session statistics, network status, and leaderboard position.

The server itself also auto-contributes its own GPU as a self-worker on startup -- no manual setup needed for the host machine.

### Contributor Badges

| Badge | Requirement |
|---|---|
| First Task | 1 task completed |
| Centurion | 100 tasks |
| Millionaire | 1,000 tasks |
| Pillar | 10,000 tasks |
| 24/7 | 7+ consecutive days uptime |
| Early Adopter | Among the first 10 contributors |
| Power Node | VRAM > 24 GB |

---

## Frontend Pages

### Cold Case Investigation

| Page | Description |
|---|---|
| **Dashboard** | Case overview, active workers, recent events, SSE live feed |
| **Evidence** | Upload and manage evidence (PDF, images, text, audio) |
| **Entities** | Extracted entities (persons, locations, dates, organizations) |
| **Hypotheses** | ACH matrix, hypothesis generation, scoring, snapshots |
| **Graph** | Neo4j knowledge graph visualization |
| **Timeline** | Chronological event timeline with drag-and-zoom |
| **Investigation** | Autonomous loop controls, monitoring status, cycle history |
| **Suspects** | Suspect scoring (5 factors: graph, evidence, contradiction, profile, hypothesis) |
| **Wiki** | Auto-generated case wiki from evidence and analysis |
| **Reports** | PDF export, investigation reports |
| **Images** | DINOv2 + CLIP visual similarity search |
| **Benchmark** | 5 cold case benchmarks (Kulik, GSK, Moreau, Jubillar, McCann) |

### Government (18 tabs)

| Tab | Description |
|---|---|
| **Politicians** | Full directory of 925 deputies + 348 senators with party colors |
| **Hemicycle** | Interactive hemicycle visualization of the National Assembly |
| **Map** | Geographic map of politicians by constituency (Leaflet) |
| **Comparator** | Side-by-side comparison of voting records and positions |
| **Network** | Relationship graph between politicians (alliances, oppositions) |
| **Contradictions** | Detected contradictions between statements and votes |
| **Press** | Press articles mentioning tracked politicians |
| **Social** | Social media posts (Twitter, Instagram, Facebook, TikTok) |
| **Videos** | YouTube videos with auto-transcription (Whisper) |
| **Statistics** | Aggregate stats: attendance, voting patterns, party discipline |
| **Alerts** | Real-time alerts for new contradictions, position changes, affairs |
| **Affairs** | Judicial and political affairs linked to politicians |
| **Declarations** | HATVP asset declarations |
| **Legislation** | Laws and EUR-Lex directives tracked by the system |
| **Timeline** | Chronological view of political events |
| **Search** | Full-text search across all GOV data (RAG-powered) |
| **Recap** | Weekly AI-generated summaries per politician |
| **Pipeline** | Real-time status of all 31 GOV workers |

### Network (6 tabs)

| Tab | Description |
|---|---|
| **Statistics** | Network-wide stats: nodes online, total VRAM, tasks today, active model |
| **Leaderboard** | Top GPU contributors ranked by tasks completed |
| **Nodes** | Live view of all connected GPU nodes with status |
| **Swarm Petals** | Petals 405B swarm health: block coverage, throughput, latency |
| **Contribute** | Personal contribution dashboard, self-worker controls |
| **Badges** | Achievement badges earned by contributing |

---

## GOV Workers (31)

| Category | Workers |
|---|---|
| **Parliamentary** | `depute_sync`, `senat_sync`, `vote_sync`, `law_sync`, `fabrique_sync`, `hatvp_sync`, `wikidata_sync` |
| **EU** | `eu_parliament_sync`, `eurlex_sync` |
| **Press & Facts** | `press_sync`, `factcheck_sync`, `press_affair_detector` |
| **Social Media** | `twitter_sync`, `instagram_sync`, `facebook_sync`, `tiktok_sync`, `youtube_sync`, `social_publish` |
| **Analysis** | `contradiction_analyzer`, `sentiment`, `voting_pattern`, `vote_impact`, `biography`, `affairs_sync`, `weekly_recap` |
| **Infrastructure** | `embedding`, `neo4j_sync`, `transcription`, `vision`, `alert`, `newsletter` |

---

## Cold Case Workers (20)

| Phase | Workers |
|---|---|
| **Ingest** | `evidence_ingest`, `entity_extractor`, `summarizer`, `chunker_embed` |
| **Enrich** | `neo4j_sync`, `geo_mapper`, `osint_recon`, `query_generator`, `wiki_compiler`, `wiki_lint` |
| **Analyze** | `contradiction`, `analysis`, `hypothesis`, `forensics`, `timeline`, `memory`, `self_questioning` |
| **Score** | `suspect_scorer`, `alert`, `summary_tree` |

Event-driven reactive architecture: no fixed cycles, each worker reacts immediately to events via the EventBus.

---

## Tech Stack

| Component | Technology |
|---|---|
| **Backend** | Python 3.13, FastAPI, Pydantic, Loguru, aiosqlite |
| **Frontend** | React 19, Vite, TypeScript, Tailwind CSS, TanStack Query, Recharts, Leaflet |
| **LLM** | Ollama (`gemma-4-26B-A4B-it-heretic:q4_k_m` MoE -- single model, zero VRAM swap) |
| **Embeddings** | `nomic-embed-text` (137MB, coexists via VRAM bypass) |
| **NER** | GLiNER (`gliner_multi-v2.1`) -- CPU, 0.08s inference, zero VRAM |
| **Entity Resolution** | RapidFuzz (Jaro-Winkler, 78% threshold) |
| **Graph Database** | Neo4j 5 Community (Docker) + APOC |
| **Vector Store** | ChromaDB (Docker, 7 collections) |
| **Relational DB** | SQLite (FTS5, WAL mode, 16+ tables) |
| **Clearweb Search** | SearXNG (self-hosted, port 8888) |
| **Dark Web Search** | Robin (Tor-based, Docker) |
| **Historical Search** | Wayback Machine CDX API |
| **Vision** | DINOv2 + CLIP embeddings, PaddleOCR, Ultralytics YOLO, OpenCV |
| **Audio** | faster-whisper (transcription) |
| **Social Scraping** | twikit (Twitter), instagrapi (Instagram), yt-dlp (YouTube) |
| **Distributed GPU** | Custom task queue + Ollama local + exo split + Petals swarm |
| **Security** | Ed25519 signatures, model digest verification, logprob fingerprinting |
| **Launcher** | Rich terminal UI (start.bat / start_nexus.py) |
| **PDF Export** | WeasyPrint + Jinja2 |
| **Forensics** | Blood pattern analysis, acoustic analysis, trace analysis, physics simulation |

---

## Project Structure

```
nexus/
  api/                     # 27 FastAPI routers (REST + SSE)
  core/                    # Business logic (evidence, analysis, hypotheses, suspects)
  events/                  # Event-driven architecture (EventBus, VRAMScheduler)
    workers/               # 20 reactive cold case workers
  compute/                 # Distributed GPU system (registry, tasks, dispatcher)
  db/                      # SQLite + Neo4j + ChromaDB clients
  gov/                     # Government module (API, scraper, identity, resilience)
    workers/               # 31 GOV workers
  llm/                     # Ollama client + LLM router + task types
  monitoring/              # SearXNG + Robin + Wayback monitors
  forensics/               # Blood, acoustic, trace, physics simulation
  recon/                   # OSINT (Holehe, WHOIS, DNS, social)
  vision/                  # DINOv2 + CLIP image analysis
  sync/                    # cr-sqlite real-time sync (Phase 9)
web/                       # React frontend (Vite + TypeScript + Tailwind)
  src/pages/               # 14 page components
  src/components/gov/      # 15 GOV tab components
  src/components/compute/  # 6 Network tab components
worker/                    # nexus-worker PyPI package (contributor client)
tests/                     # 756 test functions, 29 files
data/benchmark/            # 5 cold case benchmarks
docs/                      # Architecture, API reference, pipeline, benchmarks
```

---

## Benchmarks

5 cold cases for validation:

| Case | Type | Evidence | Ground Truth |
|---|---|---|---|
| Elodie Kulik (2002) | Evidence mode | 14 pieces | Wiart + Bardon |
| Golden State Killer | Evidence mode | 13 pieces | DeAngelo |
| Affaire Moreau | Evidence mode (fictional) | 15 pieces, 7 contradictions | Controlled test |
| Delphine Jubillar (2020) | OSINT mode | 1 briefing + monitoring | NEXUS searches autonomously |
| Madeleine McCann | Evidence mode | Cold case, unsolved | Open investigation |

**Evidence mode**: Pieces provided, analyzed by 20 workers.
**OSINT mode**: Single briefing, then NEXUS searches clearweb/Wayback/dark web on its own.

---

## Environment Variables

Key settings (via `.env` file or environment):

| Variable | Default | Description |
|---|---|---|
| `OLLAMA_BASE_URL` | `http://localhost:11434` | Ollama server URL |
| `NEO4J_URI` | `bolt://localhost:7687` | Neo4j connection |
| `NEO4J_PASSWORD` | (empty) | Neo4j password |
| `CHROMA_HOST` | `localhost` | ChromaDB host |
| `CHROMA_PORT` | `8100` | ChromaDB port |
| `SEARXNG_URL` | `http://localhost:8888` | SearXNG instance |
| `COMPUTE_ENABLED` | `true` | Enable distributed GPU system |
| `EXO_ENABLED` | `false` | Enable exo distributed inference |
| `PETALS_ENABLED` | `false` | Enable Petals 405B swarm |
| `SYNC_ENABLED` | `false` | Enable cr-sqlite real-time sync |
| `AUTO_GOVERNMENT_MONITORING` | `true` | Auto-start GOV module on boot |
| `OLLAMA_FLASH_ATTENTION` | `1` | Enable flash attention in Ollama |

See `nexus/config.py` for all 70+ configurable settings.

---

## GPU Roadmap (9 Phases)

| Phase | Name | Status |
|---|---|---|
| 1 | GPU Registry + Task Queue | Implemented |
| 2 | Auto-Scaling Model Selection | Implemented |
| 3 | Contributor Worker Client (`nexus-worker`) | Implemented |
| 4 | exo Distributed Inference (model splitting) | Implemented |
| 5 | Public Dashboard + Gamification (badges) | Implemented |
| 6 | Proof-of-Computation (Ed25519 + digest + logprobs) | Implemented |
| 7 | Petals 405B Swarm (50+ GPUs) | Implemented |
| 8 | Public Permanent Swarm | Planned |
| 9 | cr-sqlite Local DB Sync (zero-API reads) | Implemented |

---

## License

**AGPL-3.0** -- GNU Affero General Public License v3.0

Copyright (C) 2026 FlowUP & Contributors

This means: you can use, modify, and distribute this software, but if you run a modified version as a network service, you must release your source code under the same license.

See [LICENSE](../LICENSE) for the full text.
