# NEXUS -- Cold Case Investigation System

Systeme d'investigation **autonome** et **persistant** pour cold cases.
Architecture event-driven avec 20 reactive workers, RAG hybride 4 sources, pipeline ACH multi-pass, et dashboard React temps reel.

## Quick Start

### Prerequisites

| Composant | Version | Notes |
|-----------|---------|-------|
| GPU NVIDIA | 16 GB+ VRAM | RTX 4090/5080 recommande |
| Python | 3.13+ | via conda |
| Node.js | 18+ | pour le frontend React |
| Docker Desktop | latest | pour Neo4j + ChromaDB + Robin |
| Ollama | latest | LLM local |

### 1. Clone et installation Python

```bash
git clone https://github.com/your-org/nexus.git
cd nexus

conda create -n nexus python=3.13 -y
conda activate nexus
pip install -r requirements.txt
```

### 2. Services Docker

```bash
docker compose up -d
# Demarre: Neo4j (7474) + ChromaDB (8100) + Robin dark web (8502)
```

### 3. Modeles LLM

```bash
# Modele principal (MoE 26B, 4B actifs, uncensored + multimodal)
ollama pull juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m

# Embeddings vectoriels (137 MB, RAG)
ollama pull nomic-embed-text

# Optimisation GPU
set OLLAMA_FLASH_ATTENTION=1
```

### 4. Lancer NEXUS

```bash
# Backend FastAPI (port 8000)
uvicorn nexus.main:app --host 0.0.0.0 --port 8000

# Frontend React (port 3002) — dans un autre terminal
cd web && npm install && npx vite --host 0.0.0.0 --port 3002
```

### 5. Ouvrir le dashboard

```
http://localhost:3002
```

## Architecture

```
                    Upload / OSINT
                        |
                  EvidenceProcessor
                        |
                    EventBus (pub/sub + SQLite persistence)
                        |
        +---------------+---------------+
        |               |               |
   INGEST (4)      ENRICH (4)     ANALYZE (3)    SCORE (2)
   Evidence        Neo4j Sync     Analysis        Suspect
   Entity          OSINT Recon    Hypothesis      Query Gen
   Summarizer      GeoMapper     Contradiction
   Chunker+RAG     Forensics
        |               |               |               |
        +---------------+---------------+---------------+
                        |
                   SSE Bridge
                        |
                 React Dashboard (12 pages)
```

- **20 reactive workers** communiquent via EventBus (zero cycles fixes)
- **VRAMScheduler** previent les conflits GPU (priority queue + model affinity)
- **MonitoringLoop** sweep continu 30s (SearXNG, Robin/Tor, Arquivo.pt, Wayback)
- **SSE** remplace le polling (50+ req/min -> <5)

## Stack

| Layer | Technologie |
|-------|-------------|
| Backend | FastAPI (port 8000), 133+ endpoints, 24 routers |
| Frontend | React 19 + Vite + TypeScript + Tailwind, 12 pages |
| LLM | Ollama -- Gemma 4 26B A4B heretic (single model, zero swap) |
| NER | GLiNER v2.1 (CPU, 0.08s) + RapidFuzz dedup (78% threshold) |
| Embeddings | nomic-embed-text (137 MB, coexiste via VRAM bypass) |
| Search | SearXNG + Robin/Tor + Arquivo.pt + Wayback Machine |
| Storage | SQLite (FTS5+WAL) + Neo4j (graphe) + ChromaDB (vecteurs) |
| RAG | Hybride 4 sources: semantic 50% + graph 25% + FTS5 15% + recency 10% |

## Tests

```bash
# Backend (367 tests, ~10s)
python -m pytest tests/ -v

# Frontend type-check
cd web && npx tsc --noEmit
```

## Benchmarks

5 cold cases dans `data/benchmark/` :

| Affaire | Mode | Pieces | Verite terrain |
|---------|------|--------|----------------|
| Kulik (2002) | Evidence | 14 | Wiart + Bardon |
| Golden State Killer | Evidence | 13 | DeAngelo |
| Moreau (fictif) | Evidence | 15 | 7 contradictions |
| Jubillar (2020) | OSINT | briefing | H1 Cedric 71% |
| McCann | OSINT | briefing | unsolved |

```bash
# Lancer un benchmark via API
curl -X POST http://localhost:8000/api/benchmark/start \
  -H "Content-Type: application/json" \
  -d '{"case_name": "kulik"}'
```

## Configuration

Toutes les variables sont dans `nexus/config.py` avec des valeurs par defaut.
Copier `.env.example` vers `.env` pour surcharger :

```bash
cp .env.example .env
```

## Documentation

| Document | Contenu |
|----------|---------|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Architecture detaillee, event system, RAG |
| [PIPELINE.md](docs/PIPELINE.md) | Pipeline d'ingestion 11 etapes |
| [API_REFERENCE.md](docs/API_REFERENCE.md) | 133+ endpoints documentes |
| [BENCHMARK.md](docs/BENCHMARK.md) | Systeme de scoring et cold cases |
| [GUIDE-INSTALLATION.md](docs/GUIDE-INSTALLATION.md) | Guide complet avec troubleshooting |
| [TOOLS_MATRIX.md](docs/TOOLS_MATRIX.md) | Matrice des 20 workers reactifs |
