# NEXUS — Cold Case Investigation System

## Projet
Systeme d'investigation AUTONOME et PERSISTANT pour cold cases. Pas un chatbot — un investigateur qui tourne 24/7, cherche, raisonne, et converge vers la verite.

## Architecture
- **Backend**: FastAPI (port 8000) — 115+ endpoints REST
- **Frontend React**: Vite + TypeScript + Tailwind (port 3002) — 9 pages, dark theme pro
- **Frontend Streamlit**: (port 8501) — 16 pages, legacy
- **LLMs**: Ollama (port 11434)
  - `nexus` (Gemma 4 26B Heretic) — analyse profonde, hypotheses, rapports, scoring suspects
  - `huihui_ai/deepseek-r1-abliterated:14b` — raisonnement CoT, verification logique, contradictions
  - `gemma4:e4b` — resume, reformulation (NER remplace par GLiNER)
  - `nomic-embed-text` — embeddings vectoriels RAG
  - `qwen3-vl:8b` — analyse d'images (VLM)
  - `voxtral-mini:4b` — transcription audio
- **NER**: GLiNER (urchade/gliner_multi-v2.1) — CPU, 0.08s, zero VRAM
- **Entity Resolution**: RapidFuzz (Jaro-Winkler, threshold 82%)
- **Search**: SearXNG clearweb (port 8888) + Robin dark web/Tor (port 8502)
- **Storage**: SQLite (FTS5+WAL) + Neo4j (graphe, port 7474) + ChromaDB (vecteurs, port 8100)
- **Docker**: Neo4j + ChromaDB + Robin (docker-compose.yml)

## Stack
- Windows 11, RTX 5080 16GB VRAM
- Python 3.13 + conda (env nexus)
- Node.js (frontend React dans web/)
- Docker Desktop
- Ollama (20.4 tok/s sur nexus 26B Q4_K_S)

## Structure du code (41K lignes)
```
nexus/                    # Backend Python
  api/                    # 20 routers FastAPI (cases, evidence, entities, hypotheses,
                          #   graph, search, monitoring, alerts, analysis, reports,
                          #   timeline, geo, recon, vision, forensics, physics_sim,
                          #   investigation, audit, benchmark, suspects)
  core/                   # Logique metier
    autonomous_loop.py    # Boucle OODA (observe-orient-decide-act-question)
    investigation_manager.py # Gere un investigateur par case active
    analysis_pipeline.py  # Pipeline multi-modeles (RAG-powered)
    hypothesis_engine.py  # Generation + scoring + snapshots
    contradiction_detector.py
    evidence_processor.py # Ingestion: parse → GLiNER → resume → chunk → embed → Neo4j
    entity_extractor.py   # GLiNER + RapidFuzz dedup
    suspect_scorer.py     # 5 facteurs: graph, evidence, contradiction, profile, hypothesis
    retriever.py          # RAG hybride 4 sources (semantic + graph + FTS5 + recency)
    chunker.py            # Decoupe semantique 512 tokens
    embedding_store.py    # ChromaDB evidence_chunks
    summary_tree.py       # Resumes hierarchiques RAPTOR (preuve → cluster → case)
    audit.py              # 3 couches: SQLite hash chain + JSONL + Git
    geo_mapper.py         # Nominatim + OSRM
    image_analyzer.py     # VLM (gemma4/qwen3-vl)
    backup.py
  db/
    sqlite_db.py          # 15 tables, FTS5, WAL, 23 index, batch queries
    neo4j_db.py           # 20+ methodes (centralite, betweenness, communautes, temporal)
    chroma_db.py          # 7 collections, unified search cross-collection
    models.py             # Pydantic v2
  llm/
    router.py             # 20 TaskTypes, serialisation VRAM (asyncio.Lock)
    ollama_client.py      # AsyncClient, retry tenacity
    prompts.py            # 25+ prompts FR
    parsers.py            # JSON robuste (GLiNER/LLM)
  monitoring/             # SearXNG + Robin + APScheduler + AlertManager
  forensics/              # BPA sang, acoustique, traces, physique sim, the_well
  recon/                  # holehe, social, domain (WHOIS/DNS)
  vision/                 # DINOv2 + CLIP embeddings, image search
  ingest/                 # PDF (PyMuPDF) + text parsers
  export/                 # PDF (WeasyPrint + Jinja2) + timeline
web/                      # Frontend React
  src/pages/              # Dashboard, Evidence, Entities, Hypotheses, Graph,
                          #   Timeline, Investigation, Benchmark, Suspects
tests/                    # 233 tests (pytest)
data/benchmark/           # 3 cold cases (Kulik 14 pieces, GSK 13, Moreau 15)
docs/                     # 468 KB documentation
```

## Problemes connus (priorite)
1. ~~**Crash VRAM**~~ — FIXE: GLiNER pre-charge en singleton CPU au startup FastAPI
2. ~~**Neo4j rarement synced**~~ — FIXE: re-sync periodique tous les 3 cycles avec counters ok/fail
3. ~~**Suspects scoring incomplet**~~ — FIXE: 5 facteurs connectes dans la boucle DECIDE + profil LLM auto
4. ~~**Pipeline end-to-end jamais complete**~~ — FIXE: benchmark utilise app.state singletons + RAG active
5. ~~**FTS5 cree mais jamais appele**~~ — FIXE: endpoints /fts + /hybrid + integre dans retriever hybride
6. **Timeline vide** — dates extraites pas parsees en datetime
7. **Hypotheses en doublon** — partiellement fixe (RapidFuzz) mais pas parfait
8. **Benchmark Kulik incomplet** — seules 3/14 pieces injectees lors du dernier test (waves 2-3 pas lancees)
9. **Chargement front benchmark** — pas de progress bar avec timer pendant l'injection

## Boucle autonome OODA (completee)
```
OBSERVE  → monitoring results (filtre pertinence >= 50%)
ORIENT   → 2a ingest + 2b neo4j resync (3 cycles) + 2c OSINT + 2d geocode + 2e VLM + 2f DINOv2/CLIP
DECIDE   → 3a analyse + 3b hypotheses + 3c contradictions + 3d suspects scoring + 3e forensics + 3f timeline + 3g RAPTOR
ACT      → 4a queries LLM + 4b enrichment OSINT + 4c domain recon
QUESTION → 5a self-questioning + 5b rapports + 5c backups
```

## Retriever hybride (4 sources)
- Semantic (ChromaDB unified) × 0.50
- Graph (Neo4j traversal) × 0.25
- FTS5 (SQLite lexical) × 0.15
- Recency × 0.10

## ChromaDB collections (7, toutes peuplees)
- evidence_chunks ← EmbeddingStore (RAG principal)
- entity_contexts ← evidence_processor (extraction entites)
- monitoring_results ← scheduler (dedup OSINT)
- hypothesis_reasoning ← hypothesis_engine (generation + reevaluation)
- image_dinov2 + image_clip ← ImageSearchEngine
- evidence_texts ← DEPRECATED (plus d'ecriture)

## Priorite prochaine session
1. **Timeline** — parser les dates extraites en datetime
2. **Benchmark Kulik complet** — 14 pieces sans crash, scorer /100
3. **Progress bar front** — chargement avec timer pendant le bench
4. **Tests unitaires** — couvrir les nouvelles features (FTS5, suspect scoring, retriever)
5. **Hypotheses Neo4j** — sync_hypothesis() pour le graphe (constraint existe, sync manquante)

## Benchmarking
3 cold cases reels:
- `data/benchmark/kulik/` — Affaire Elodie Kulik (14 pieces, verite: Wiart+Bardon)
- `data/benchmark/golden-state-killer/` — GSK (13 pieces, verite: DeAngelo ex-policier)
- `data/benchmark/affaire-moreau/` — Fictif (15 pieces, 7 contradictions plantees)

Scoring /100: entites /20 + hypothese top3 /20 + contradictions /20 + score>40% /20 + timeline+geo /20

Lancement: React http://localhost:3002 → page Benchmark → Nouveau benchmark → choisir case

## Commandes
```bash
# Backend
uvicorn nexus.main:app --host 0.0.0.0 --port 8000

# Frontend React
cd web && npx vite --host 0.0.0.0 --port 3002

# Frontend Streamlit (legacy)
streamlit run frontend/app.py --server.port 8501

# Docker
docker compose up -d

# Tests
python -m pytest tests/ -v

# Ollama optimisation
set OLLAMA_FLASH_ATTENTION=1
```

## Langue
Francophone. Repondre en francais. Prompts LLM en francais. Code/commentaires en anglais.
