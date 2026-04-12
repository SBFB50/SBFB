# NEXUS GOV -- Intelligence Politique Autonome

> Systeme citoyen open source qui detecte automatiquement les contradictions
> entre ce que les politiciens disent et ce qu'ils votent.
> 100% local. Zero cloud. Zero censure.

[Francais](#fr) | [English](#en)

---

## Quick start (SBFB / nexus-grid pivot 2026-04)

Since the 2026-04 pivot, the primary project is **SBFB** — a P2P LLM
compute network that now hosts the legacy NEXUS GOV as one of its apps.
See `CLAUDE.md` and `docs/claude/README.md` for the full workflow.

### First-time setup on a fresh checkout

Sprint 9 Phase A ships two helper scripts under `scripts/` that
encapsulate the install + verification ceremony so a brand-new
clone can get to a green `verify.sh` in three commands:

```bash
# 1. Install Rust + Python toolchains:
#    - Rust 1.94+   (via rustup or winget on Windows)
#    - uv 0.5+      (Python workspace manager)
#    - maturin 1.13 (PyO3 wheel builder, via uv)
#    - Node 20+     (frontend)

# 2. Build the nexus_core PyO3 wheel into .venv/ and sync Python deps.
./scripts/setup.sh

# 3. Run the full fail-fast verification suite (cargo + ruff + pytest
#    + tsc + eslint + vitest + size-limit + playwright + npm audit).
./scripts/verify.sh

# For faster iteration during a phase, skip Playwright:
./scripts/verify.sh --quick
```

`setup.sh` is idempotent: it hashes `Cargo.lock` +
`crates/nexus-core-{rs,py}/src` into `.venv/.nexus-core-hash` and
skips the maturin rebuild when the sources have not changed. Pass
`--force` to rebuild unconditionally.

Opt-in git hook to remind you after pulls that touched the Rust
core crates:

```bash
git config core.hooksPath .githooks
```

---

<a id="fr"></a>

## FR -- Qu'est-ce que NEXUS GOV ?

Un outil autonome qui tourne 24/7 pour :

- Collecter les votes, tweets, interviews, declarations HATVP de 1145+ politiciens francais et eurodeputes
- Transcrire automatiquement les videos parlementaires et interviews TV (faster-whisper)
- Detecter les contradictions via IA locale (LLM uncensored, zero filtre)
- Visualiser le tout : hemicycle interactif, graphe d'influence, timeline cross-source, carte des elus

### Pourquoi ?

La transparence politique ne devrait pas dependre d'une entreprise privee, d'un cloud americain, ou de la bonne volonte d'un gouvernement. NEXUS GOV est un outil de souverainete numerique : le code est ouvert, les donnees sont publiques, l'IA tourne sur votre machine. Personne ne peut le censurer, le fermer, ou le monetiser.

Chaque citoyen devrait pouvoir verifier si un elu vote comme il le promet. NEXUS GOV automatise cette verification 24h/24, sur toutes les sources publiques disponibles, sans opinion ni jugement -- uniquement des faits sources.

### Screenshots

> A venir -- hemicycle interactif, graphe d'influence, detection de contradictions, sidebar navigation

### Fonctionnalites

**Collecte autonome (31 workers)**

- 8 sources institutionnelles : Assemblee Nationale, Senat, HATVP, data.gouv.fr, La Fabrique de la Loi, Wikidata, PoliGraph, Parlement Europeen
- 5 reseaux sociaux : Twitter/X, Facebook, Instagram, TikTok, YouTube
- Presse : 7+ flux RSS + recherche SearXNG temps reel
- Fact-checks : Google Fact Check API + AFP Factuel
- Legislation europeenne : EUR-Lex

**Analyse IA locale**

- Detection de contradictions cross-source (tweet vs vote, interview TV vs declaration patrimoine)
- Transcription video timestampee (faster-whisper large-v3)
- Analyse visuelle : OCR bandeaux TV (PaddleOCR), detection de scenes, classification objets (YOLO)
- Biographies generees automatiquement par politicien
- Resume de scrutins avec impact citoyen
- Detection d'affaires dans la presse
- Analyse de sentiment mediatique
- Score de coherence factuel (positions coherentes / total positions)
- Embeddings vectoriels pour recherche semantique (RAG)

**19 onglets d'analyse**

- Hemicycle interactif avec positions des deputes
- 3 moteurs de graphe : G6 WebGL analytique, Sigma.js avec clustering Louvain, Reagraph 3D
- Contradictions detectees avec sources et timeline
- Timeline cross-source (votes + tweets + interviews + presse)
- Recherche semantique RAG dans tout le corpus
- Carte Leaflet des elus par circonscription
- Comparateur : 2 politiciens cote a cote sur tous les axes
- Legislation en cours avec stats La Fabrique
- Declarations patrimoine HATVP avec evolution temporelle
- Affaires judiciaires (timeline, statut, categorie)
- Presse agregee avec tracking de sentiment
- Reseaux sociaux integres
- Videos avec transcriptions et recherche dans le contenu
- Alertes temps reel
- Newsletter automatique et publication reseaux sociaux
- Pipeline : statut temps reel des 31 workers
- Stats globales et graphiques analytiques

**Publication automatique**

- Newsletter hebdomadaire "Alerte Politique" (SMTP)
- Publication Bluesky/Twitter des contradictions detectees
- Resume hebdomadaire genere par LLM

### Architecture

```
Sources publiques (AN, Senat, HATVP, Twitter, YouTube, Presse...)
                            |
                    31 Workers autonomes
                 (collecte + analyse + publication)
                            |
                    EventBus (pub/sub + persistence SQLite)
                            |
            +---------------+---------------+
            |               |               |
    COLLECTE (16)     ANALYSE (10)    PUBLICATION (5)
    Votes, Presse     Contradictions  Newsletter
    Social, HATVP     Sentiment       Bluesky/Twitter
    Lois, Wikidata    Patterns vote   Alertes
    EU Parlement      Biographies     Recap hebdo
    Transcription     Embeddings RAG  Vote impact
                            |
                    FastAPI (180+ endpoints REST + SSE)
                            |
                    React Dashboard (19 onglets)
```

### Quick Start

```bash
# 1. Cloner le depot
git clone https://github.com/FlowUP/nexus-gov.git
cd nexus-gov

# 2. Creer l'environnement Python
conda create -n nexus python=3.13 -y
conda activate nexus
pip install -r requirements.txt

# 3. Lancer les services Docker (Neo4j + ChromaDB)
docker compose up -d

# 4. Installer les modeles Ollama
ollama pull juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m
ollama pull nomic-embed-text

# 5. Lancer le backend
uvicorn nexus.main:app --host 0.0.0.0 --port 8000

# 6. Lancer le frontend (dans un autre terminal)
cd web && npm install && npx vite --host 0.0.0.0 --port 3002

# 7. Ouvrir le navigateur
# http://localhost:3002
```

**Pre-requis materiel :** GPU NVIDIA avec 16 GB+ VRAM (RTX 4090, 5080 ou equivalent). Le LLM et les modeles de vision tournent localement.

### Stack technique

| Composant | Technologie | Role |
|-----------|-------------|------|
| Backend | FastAPI + Python 3.13 | API REST, 180+ endpoints, 24 routers |
| Frontend | React 19 + Vite + TypeScript + Tailwind 4 | Dashboard 19 onglets, dark theme |
| UI | shadcn/ui + Radix | Composants accessibles |
| Graphe | G6 WebGL + Sigma.js + Reagraph 3D | 3 moteurs de visualisation reseau |
| Charts | Recharts + Nivo (heatmap, sankey, chord, radar) | Statistiques et analytique |
| Carte | Leaflet + React-Leaflet | Carte interactive des elus |
| Timeline | react-calendar-timeline | Chronologie cross-source |
| LLM | Ollama -- Gemma 4 26B heretic (MoE, uncensored) | Analyse, resume, contradictions, vision |
| Transcription | faster-whisper large-v3 | Transcription video/audio timestampee |
| Vision | PaddleOCR + OpenCV + YOLO + CLIP | OCR bandeaux TV, detection scenes |
| NER | GLiNER v2.1 (CPU, 0.08s) | Extraction d'entites nommees |
| Entity resolution | RapidFuzz (Jaro-Winkler, seuil 78%) | Deduplication politiciens cross-source |
| Embeddings | nomic-embed-text (137 MB) | Recherche semantique RAG |
| Base vectorielle | ChromaDB | Stockage et recherche d'embeddings |
| Base graphe | Neo4j 5 Community + APOC | Relations politiciens-votes-partis-affaires |
| Base relationnelle | SQLite (FTS5 + WAL) / PostgreSQL | Donnees structurees, full-text search |
| Recherche web | SearXNG | Meta-moteur de recherche, zero tracking |
| Social scraping | twikit, instagrapi, yt-dlp | Collecte reseaux sociaux |
| State management | Zustand | Etat frontend reactif |
| Data fetching | TanStack Query | Cache et synchronisation API |

### Configuration

Copier `.env.example` vers `.env` et adapter les valeurs :

```bash
cp .env.example .env
```

Voir [`.env.example`](.env.example) pour la liste complete des variables.

### Tests

```bash
# Backend (476 tests)
python -m pytest tests/ -v

# Frontend type-check
cd web && npx tsc --noEmit
```

### Documentation

| Document | Contenu |
|----------|---------|
| [CONTRIBUTING.md](CONTRIBUTING.md) | Comment contribuer au projet |
| [SECURITY.md](SECURITY.md) | Politique de donnees et vie privee |
| [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) | Code de conduite des contributeurs |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Architecture detaillee, event system |
| [docs/API_REFERENCE.md](docs/API_REFERENCE.md) | 180+ endpoints documentes |

---

<a id="en"></a>

## EN -- What is NEXUS GOV?

An autonomous tool running 24/7 to:

- Collect votes, tweets, interviews, and asset declarations from 1,145+ French politicians and MEPs
- Automatically transcribe parliamentary videos and TV interviews (faster-whisper)
- Detect contradictions using local AI (uncensored LLM, zero filtering)
- Visualize everything: interactive hemicycle, influence graph, cross-source timeline, elected officials map

### Why?

Political transparency should not depend on a private company, an American cloud, or government goodwill. NEXUS GOV is a digital sovereignty tool: the code is open, the data is public, the AI runs on your machine. Nobody can censor it, shut it down, or monetize it.

Every citizen should be able to check if an elected official votes as promised. NEXUS GOV automates this verification 24/7, across all available public sources, without opinion or judgment -- only sourced facts.

### Features

**Autonomous collection (31 workers)**

- 8 institutional sources: National Assembly, Senate, HATVP (asset declarations), data.gouv.fr, La Fabrique de la Loi, Wikidata, PoliGraph, European Parliament
- 5 social networks: Twitter/X, Facebook, Instagram, TikTok, YouTube
- Press: 7+ RSS feeds + real-time SearXNG search
- Fact-checks: Google Fact Check API + AFP Factuel
- European legislation: EUR-Lex

**Local AI analysis**

- Cross-source contradiction detection (tweet vs vote, TV interview vs asset declaration)
- Timestamped video transcription (faster-whisper large-v3)
- Visual analysis: TV chyron OCR (PaddleOCR), scene detection, object classification (YOLO)
- Auto-generated biographies per politician
- Vote summaries with citizen impact
- Press affair detection
- Media sentiment analysis
- Factual coherence score (consistent positions / total positions)
- Vector embeddings for semantic search (RAG)

**19 analysis tabs**

- Interactive hemicycle with deputy positions
- 3 graph engines: G6 WebGL analytical, Sigma.js with Louvain clustering, Reagraph 3D
- Detected contradictions with sources and timeline
- Cross-source timeline (votes + tweets + interviews + press)
- Semantic RAG search across the entire corpus
- Leaflet map of elected officials by constituency
- Comparator: 2 politicians side-by-side across all axes
- Current legislation with La Fabrique stats
- HATVP asset declarations with temporal evolution
- Judicial affairs (timeline, status, category)
- Aggregated press with sentiment tracking
- Integrated social media
- Videos with transcriptions and content search
- Real-time alerts
- Automatic newsletter and social media publishing
- Pipeline: real-time status of all 31 workers
- Global stats and analytical charts

### Quick Start

```bash
# 1. Clone the repository
git clone https://github.com/FlowUP/nexus-gov.git
cd nexus-gov

# 2. Create the Python environment
conda create -n nexus python=3.13 -y
conda activate nexus
pip install -r requirements.txt

# 3. Start Docker services (Neo4j + ChromaDB)
docker compose up -d

# 4. Install Ollama models
ollama pull juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m
ollama pull nomic-embed-text

# 5. Start the backend
uvicorn nexus.main:app --host 0.0.0.0 --port 8000

# 6. Start the frontend (in another terminal)
cd web && npm install && npx vite --host 0.0.0.0 --port 3002

# 7. Open your browser
# http://localhost:3002
```

**Hardware requirements:** NVIDIA GPU with 16 GB+ VRAM (RTX 4090, 5080 or equivalent). The LLM and vision models run locally.

### Tech Stack

| Component | Technology | Role |
|-----------|-----------|------|
| Backend | FastAPI + Python 3.13 | REST API, 180+ endpoints, 24 routers |
| Frontend | React 19 + Vite + TypeScript + Tailwind 4 | 19-tab dashboard, dark theme |
| UI | shadcn/ui + Radix | Accessible components |
| Graph | G6 WebGL + Sigma.js + Reagraph 3D | 3 network visualization engines |
| Charts | Recharts + Nivo (heatmap, sankey, chord, radar) | Statistics and analytics |
| Map | Leaflet + React-Leaflet | Interactive elected officials map |
| LLM | Ollama -- Gemma 4 26B heretic (MoE, uncensored) | Analysis, summarization, contradictions, vision |
| Transcription | faster-whisper large-v3 | Timestamped video/audio transcription |
| Vision | PaddleOCR + OpenCV + YOLO + CLIP | TV chyron OCR, scene detection |
| NER | GLiNER v2.1 (CPU, 0.08s) | Named entity recognition |
| Entity resolution | RapidFuzz (Jaro-Winkler, 78% threshold) | Cross-source politician deduplication |
| Embeddings | nomic-embed-text (137 MB) | Semantic RAG search |
| Vector DB | ChromaDB | Embedding storage and search |
| Graph DB | Neo4j 5 Community + APOC | Politician-vote-party-affair relationships |
| Relational DB | SQLite (FTS5 + WAL) / PostgreSQL | Structured data, full-text search |
| Web search | SearXNG | Meta search engine, zero tracking |
| Social scraping | twikit, instagrapi, yt-dlp | Social network data collection |

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute.

## License

AGPL-3.0 -- see [LICENSE](LICENSE)

This license guarantees that NEXUS GOV and all derivative works remain free and open source. If you deploy a modified version as a network service, you must share your source code.

## Financement / Funding

NEXUS GOV est un projet citoyen independant. Aucune entreprise, aucun parti politique, aucun interet prive.

NEXUS GOV is an independent citizen project. No company, no political party, no private interest.

- **Open Collective :** [opencollective.com/nexus-gov](https://opencollective.com/nexus-gov)
- **GitHub Sponsors :** [github.com/sponsors/FlowUP](https://github.com/sponsors/FlowUP)

Les fonds servent exclusivement a l'infrastructure (serveur dedie, nom de domaine) et sont geres de maniere transparente.

Funds are used exclusively for infrastructure (dedicated server, domain name) and managed transparently.
