# NEXUS — Cold Case Investigation System

## Projet
Systeme d'investigation AUTONOME et PERSISTANT pour cold cases. Pas un chatbot — un investigateur qui tourne 24/7, cherche, raisonne, et converge vers la verite.

## Architecture (v2 — Event-Driven Reactive)
- **Backend**: FastAPI (port 8000) — 133 endpoints REST + 2 SSE
- **Event System**: EventBus pub/sub + 20 ReactiveWorkers + VRAMScheduler + SSE bridge
- **Frontend React**: Vite + TypeScript + Tailwind (port 3002) — 12 pages, dark theme pro
- **LLMs**: Ollama (port 11434)
  - `juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m` (MoE 26B, 4B actifs) — ALL tasks: resume, analyse, hypotheses ACH, contradictions, vision, filtering (single model, zero swap VRAM)
  - `nomic-embed-text` — embeddings vectoriels RAG (137MB, coexiste via bypass)
- **NER**: GLiNER (urchade/gliner_multi-v2.1) — CPU, 0.08s, zero VRAM, singleton pre-charge
- **Entity Resolution**: RapidFuzz (Jaro-Winkler, threshold 78%)
- **Search**: SearXNG clearweb (port 8888) + Robin dark web/Tor (port 8502) + Wayback Machine CDX API
- **Storage**: SQLite (FTS5+WAL+event_log) + Neo4j (graphe, port 7474) + ChromaDB (vecteurs, port 8100)
- **Docker**: Neo4j + ChromaDB + Robin (docker-compose.yml)

## Stack
- Windows 11, RTX 5080 16GB VRAM
- Python 3.13 + conda (env nexus)
- Node.js (frontend React dans web/)
- Docker Desktop
- Ollama (num_ctx=16384)

## Structure du code (~32K lignes)
```
nexus/
  api/                    # 24 routers FastAPI (incl. SSE)
  core/                   # Logique metier
    evidence_processor.py # Ingestion: parse -> GLiNER -> resume -> chunk -> embed -> Neo4j
    analysis_pipeline.py  # Pipeline multi-modeles (RAG-powered)
    hypothesis_engine.py  # Generation + scoring + snapshots + Neo4j sync
    contradiction_detector.py
    entity_extractor.py   # GLiNER + RapidFuzz dedup
    suspect_scorer.py     # 5 facteurs: graph, evidence, contradiction, profile, hypothesis
    retriever.py          # RAG hybride 4 sources (semantic + graph + FTS5 + recency)
    summary_tree.py       # RAPTOR (case summary deferred to avoid VRAM thrash)
  events/                 # NEW: Event-driven reactive architecture
    types.py              # 20 EventTypes + NexusEvent dataclass
    bus.py                # EventBus (pub/sub + SQLite persistence + circuit breaker)
    worker.py             # ReactiveWorker ABC
    vram_scheduler.py     # VRAMScheduler (priority queue + model affinity batching)
    manager.py            # ReactiveInvestigationManager (replaces InvestigationManager)
    monitoring_loop.py    # Continuous monitoring (replaces APScheduler)
    timer.py              # Periodic events (reports, backups)
    db_proxy.py           # DB connection proxy for long-lived workers
    workers/              # 20 reactive workers (13 core + 7 auxiliary)
  db/
    sqlite_db.py          # 16 tables (incl event_log), FTS5, WAL, 23+ index
    neo4j_db.py           # 20+ methodes + sync_hypothesis
    chroma_db.py          # 7 collections, unified search cross-collection
  llm/
    router.py             # 20 TaskTypes, VRAMScheduler priority queue
    ollama_client.py      # AsyncClient, no timeouts, num_ctx=16384
  monitoring/
    searxng_monitor.py    # SearXNG clearweb search
    robin_monitor.py      # Dark web / Tor via Docker CLI
    wayback_monitor.py    # Internet Archive CDX API (historical pages)
  forensics/              # BPA sang, acoustique, traces, physique sim
  recon/                  # holehe, social, domain (WHOIS/DNS)
  vision/                 # DINOv2 + CLIP embeddings, image search
web/                      # Frontend React
  src/components/
    PipelineTools.tsx      # 20 workers real-time status (INGEST/ENRICH/ANALYZE/SCORE)
    InvestigationMap.tsx   # Leaflet dark tiles + geocoded locations
    Toast.tsx              # Notification system
tests/                    # 367 tests (pytest)
data/benchmark/           # 5 cold cases
docs/                     # ARCHITECTURE.md, PIPELINE.md, TOOLS_MATRIX.md, API_REFERENCE.md, BENCHMARK.md
```

## Event-Driven Architecture
```
evidence_added -> EntityExtractor + Summarizer (parallel)
  -> entity_discovered -> Neo4j + GeoMapper + OSINT Recon + QueryGenerator
  -> evidence_processed -> ChunkerEmbed + ContradictionDetector + AnalysisPipeline
  -> analysis_completed -> HypothesisWorker -> hypothesis_scored -> SuspectScorer
  -> monitoring_result -> EvidenceIngestWorker -> evidence_added (LOOP)
```
- 20 event types, 20 workers, EventBus with SQLite persistence + circuit breaker
- VRAMScheduler: embedding bypass + light lock + heavy priority queue + model affinity
- MonitoringLoop: continuous 30s sweep (replaces APScheduler)
- No fixed cycles — each tool reacts immediately to changes

## Retriever hybride (4 sources)
- Semantic (ChromaDB unified) x 0.50
- Graph (Neo4j traversal) x 0.25
- FTS5 (SQLite lexical, sanitized) x 0.15
- Recency x 0.10

## Problemes connus
1. **RAPTOR case summary desactive** — deferred pour eviter LLM 26B a chaque evidence
2. **before: filter SearXNG unreliable** — Google before: operator still in beta, htmldate can't detect dates on Wikipedia
3. **ImageSearch thumbnails** — frontend placeholder icons (needs file serving endpoint for actual images)

## Benchmarking
5 cold cases:
- `data/benchmark/kulik/` — Affaire Elodie Kulik 2002 (14 pieces, verite: Wiart+Bardon)
- `data/benchmark/golden-state-killer/` — GSK (13 pieces, verite: DeAngelo)
- `data/benchmark/affaire-moreau/` — Fictif (15 pieces, 7 contradictions)
- `data/benchmark/jubillar/` — Affaire Delphine Jubillar 2020 (OSINT mode: briefing only + monitoring)
- `data/benchmark/mccann/` — Affaire Madeleine McCann (cold case, unsolved, before:2020)

Modes:
- **Evidence mode** (Kulik, GSK, Moreau, McCann): pieces fournies, analyse par les 20 workers
- **OSINT mode** (Jubillar): 1 briefing + monitoring SearXNG/Wayback, NEXUS cherche seul

## Commandes
```bash
# Backend
uvicorn nexus.main:app --host 0.0.0.0 --port 8000

# Frontend React
cd web && npx vite --host 0.0.0.0 --port 3002

# Docker
docker compose up -d

# Tests
python -m pytest tests/ -v

# Ollama optimisation
set OLLAMA_FLASH_ATTENTION=1
```

## Langue
Francophone. Repondre en francais. Prompts LLM en francais. Code/commentaires en anglais.
