# NEXUS -- Plan de developpement detaille

**Projet :** NEXUS -- Systeme d'investigation persistant pour cold cases
**Date :** 2026-04-05
**Environnement :** Windows 11, RTX 5080 16GB, Ollama, Docker Desktop, Python 3.13

---

## Table des matieres

1. [Architecture globale](#1-architecture-globale)
2. [Structure du projet](#2-structure-du-projet)
3. [Schemas de base de donnees](#3-schemas-de-base-de-donnees)
4. [Logique de routage multi-modeles](#4-logique-de-routage-multi-modeles)
5. [Docker Compose](#5-docker-compose)
6. [Phase 1 -- MVP : Stockage + Ingestion + Analyse de base](#phase-1--mvp--stockage--ingestion--analyse-de-base)
7. [Phase 2 -- Graphe relationnel + Recherche vectorielle](#phase-2--graphe-relationnel--recherche-vectorielle)
8. [Phase 3 -- Monitoring automatise (SearXNG + Tor)](#phase-3--monitoring-automatise-searxng--tor)
9. [Phase 4 -- Hypotheses evolutives + Snapshots](#phase-4--hypotheses-evolutives--snapshots)
10. [Phase 5 -- Dashboard web complet](#phase-5--dashboard-web-complet)
11. [Phase 6 -- Rapports + Export + Hardening](#phase-6--rapports--export--hardening)

---

## 1. Architecture globale

```
                    +------------------+
                    |   FRONTEND       |
                    |   Streamlit      |
                    |   (port 8501)    |
                    +--------+---------+
                             |
                             | HTTP
                             v
                    +------------------+
                    |   BACKEND        |
                    |   FastAPI        |
                    |   (port 8000)    |
                    +--------+---------+
                             |
              +--------------+--------------+------------------+
              |              |              |                  |
              v              v              v                  v
     +--------+--+   +------+-----+  +-----+------+   +------+------+
     |  SQLite   |   |   Neo4j    |  |  ChromaDB  |   |   Ollama    |
     |  (local)  |   |  (Docker)  |  |  (Docker)  |   | (localhost) |
     |           |   | port 7474  |  | port 8100  |   | port 11434  |
     +-----------+   | port 7687  |  +------------+   +------+------+
                     +------------+                          |
                                                   +---------+---------+
                                                   |         |         |
                                                   v         v         v
                                              gemma4:e4b  deepseek  nexus
                                              (4B)        r1 (14B)  (26B)
              +------------------+
              |   APScheduler    |
              |   (in-process)   |
              +--------+---------+
                       |
              +--------+---------+
              |    SearXNG       |        +-------------+
              |  (port 8888)     |        |   Robin     |
              |  (deja actif)    |        |   Tor/DW    |
              +------------------+        | (port 9090) |
                                          +-------------+
```

**Flux de donnees principal :**

```
Evidence (upload)
    |
    v
[FastAPI] --> SQLite (metadonnees)
    |
    +--> [gemma4:e4b] --> Extraction d'entites (noms, lieux, dates, numeros)
    |                         |
    |                         +--> Neo4j (noeuds + aretes)
    |                         +--> ChromaDB (embeddings via nomic-embed-text)
    |
    +--> [nexus/26B] --> Analyse profonde + Hypotheses
    |                         |
    |                         +--> SQLite (hypotheses + scores)
    |
    +--> [deepseek-r1] --> Verification logique des hypotheses
                              |
                              +--> SQLite (verification_results)
```

---

## 2. Structure du projet

```
cold-case-analyst/
|
|-- docker-compose.yml              # Stack complete (Neo4j, ChromaDB, Robin)
|-- .env                            # Variables d'environnement
|-- requirements.txt                # Dependances Python
|
|-- models/                         # Fichiers GGUF (existant)
|-- Modelfile.gemma4-heretic        # (existant)
|-- Modelfile                       # (existant)
|
|-- searxng/                        # Config SearXNG (existant)
|   |-- settings.yml
|   |-- limiter.toml
|
|-- prompts/                        # Prompts specialises (existant)
|   |-- analyse-cold-case.md
|   |-- contre-enquete.md
|   |-- modus-operandi.md
|   |-- recherche-osint.md
|   |-- reinvestigation-continue.md
|
|-- docs/                           # Documentation (existant)
|   |-- DEVELOPMENT-PLAN.md         # << CE FICHIER
|   |-- LOCAL-DEEP-RESEARCH-ANALYSIS.md
|   |-- OSINT-with-LLM-research.md
|
|-- nexus/                          # ** PACKAGE PRINCIPAL **
|   |-- __init__.py
|   |-- main.py                     # Point d'entree FastAPI (uvicorn)
|   |-- config.py                   # Configuration centralisee
|   |
|   |-- api/                        # Couche API (FastAPI routers)
|   |   |-- __init__.py
|   |   |-- cases.py                # CRUD dossiers
|   |   |-- evidence.py             # Upload + gestion preuves
|   |   |-- entities.py             # Entites extraites
|   |   |-- hypotheses.py           # Hypotheses + scoring
|   |   |-- analysis.py             # Declenchement d'analyses
|   |   |-- monitoring.py           # Gestion jobs de surveillance
|   |   |-- alerts.py               # Alertes
|   |   |-- graph.py                # Requetes Neo4j
|   |   |-- search.py               # Recherche vectorielle
|   |   |-- reports.py              # Generation de rapports
|   |
|   |-- core/                       # Logique metier
|   |   |-- __init__.py
|   |   |-- case_manager.py         # Gestion des dossiers
|   |   |-- evidence_processor.py   # Pipeline d'ingestion
|   |   |-- entity_extractor.py     # Extraction NER via LLM
|   |   |-- hypothesis_engine.py    # Moteur d'hypotheses
|   |   |-- analysis_pipeline.py    # Orchestration multi-modeles
|   |   |-- contradiction_detector.py  # Detection de contradictions
|   |   |-- timeline_builder.py     # Construction de timelines
|   |
|   |-- llm/                        # Couche d'abstraction LLM
|   |   |-- __init__.py
|   |   |-- router.py               # Routage vers le bon modele
|   |   |-- ollama_client.py        # Client Ollama unifie
|   |   |-- prompts.py              # Templates de prompts
|   |   |-- parsers.py              # Parsing des reponses LLM (JSON, etc.)
|   |
|   |-- db/                         # Couche donnees
|   |   |-- __init__.py
|   |   |-- sqlite_db.py            # Connexion + migrations SQLite
|   |   |-- neo4j_db.py             # Client Neo4j
|   |   |-- chroma_db.py            # Client ChromaDB
|   |   |-- models.py               # Modeles Pydantic
|   |
|   |-- monitoring/                 # Surveillance automatisee
|   |   |-- __init__.py
|   |   |-- scheduler.py            # Config APScheduler
|   |   |-- searxng_monitor.py      # Recherche clearweb
|   |   |-- robin_monitor.py        # Recherche dark web
|   |   |-- alert_manager.py        # Gestion des alertes
|   |
|   |-- ingest/                     # Ingestion de fichiers
|   |   |-- __init__.py
|   |   |-- pdf_parser.py           # Extraction texte PDF
|   |   |-- image_ocr.py            # OCR sur images
|   |   |-- audio_transcript.py     # Transcription audio
|   |   |-- text_parser.py          # Fichiers texte brut
|   |
|   |-- export/                     # Export de rapports
|   |   |-- __init__.py
|   |   |-- report_generator.py     # Generation de rapports
|   |   |-- pdf_export.py           # Export PDF
|   |   |-- timeline_export.py      # Export timeline
|
|-- frontend/                       # ** INTERFACE WEB **
|   |-- app.py                      # Point d'entree Streamlit
|   |-- pages/
|   |   |-- 01_dashboard.py         # Vue d'ensemble du dossier
|   |   |-- 02_evidence.py          # Gestion des preuves
|   |   |-- 03_entities.py          # Entites + graphe
|   |   |-- 04_hypotheses.py        # Hypotheses + evolution
|   |   |-- 05_timeline.py          # Timeline interactive
|   |   |-- 06_monitoring.py        # Jobs de surveillance
|   |   |-- 07_alerts.py            # Centre d'alertes
|   |   |-- 08_reports.py           # Generation de rapports
|   |-- components/
|   |   |-- graph_viewer.py         # Composant visualisation graphe
|   |   |-- timeline_viewer.py      # Composant timeline
|   |   |-- hypothesis_chart.py     # Graphique d'evolution des hypotheses
|
|-- data/                           # Donnees persistantes (gitignored)
|   |-- nexus.db                    # Base SQLite
|   |-- uploads/                    # Fichiers uploades
|   |-- neo4j/                      # Volume Neo4j
|   |-- chroma/                     # Volume ChromaDB
|
|-- tests/                          # Tests
|   |-- test_evidence_processor.py
|   |-- test_entity_extractor.py
|   |-- test_hypothesis_engine.py
|   |-- test_ollama_client.py
|   |-- test_api_cases.py
```

---

## 3. Schemas de base de donnees

### 3.1 SQLite -- Donnees structurees

```sql
-- ============================================================
-- TABLE : cases (Dossiers d'investigation)
-- ============================================================
CREATE TABLE cases (
    id              TEXT PRIMARY KEY,        -- UUID v4
    name            TEXT NOT NULL,           -- Nom du dossier
    reference       TEXT,                    -- Reference officielle (ex: #2019-4472)
    description     TEXT,                    -- Description du cas
    status          TEXT DEFAULT 'active',   -- active | archived | closed
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================
-- TABLE : evidence (Pieces a conviction / preuves)
-- ============================================================
CREATE TABLE evidence (
    id              TEXT PRIMARY KEY,        -- UUID v4
    case_id         TEXT NOT NULL REFERENCES cases(id),
    title           TEXT NOT NULL,           -- Titre descriptif
    evidence_type   TEXT NOT NULL,           -- pdf | image | text | audio | url | manual
    source          TEXT,                    -- Provenance (police, temoin, OSINT, etc.)
    source_date     DATETIME,               -- Date de la source originale
    ingestion_date  DATETIME DEFAULT CURRENT_TIMESTAMP,
    reliability     INTEGER DEFAULT 50,     -- Score de fiabilite 0-100
    file_path       TEXT,                    -- Chemin vers le fichier original
    raw_text        TEXT,                    -- Texte extrait
    summary         TEXT,                    -- Resume genere par LLM
    metadata        TEXT,                    -- JSON blob (taille, format, hash SHA256, etc.)
    status          TEXT DEFAULT 'pending',  -- pending | processed | error
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================
-- TABLE : entities (Entites extraites)
-- ============================================================
CREATE TABLE entities (
    id              TEXT PRIMARY KEY,        -- UUID v4
    case_id         TEXT NOT NULL REFERENCES cases(id),
    name            TEXT NOT NULL,           -- Nom/valeur de l'entite
    entity_type     TEXT NOT NULL,           -- person | location | phone | vehicle |
                                             -- organization | date | money | ip | email |
                                             -- account | weapon | drug | other
    aliases         TEXT,                    -- JSON array d'alias connus
    description     TEXT,                    -- Description contextuelle
    first_seen      DATETIME,               -- Premiere apparition dans les preuves
    metadata        TEXT,                    -- JSON blob (donnees specifiques au type)
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================
-- TABLE : entity_mentions (Liens entite <-> preuve)
-- ============================================================
CREATE TABLE entity_mentions (
    id              TEXT PRIMARY KEY,
    entity_id       TEXT NOT NULL REFERENCES entities(id),
    evidence_id     TEXT NOT NULL REFERENCES evidence(id),
    context         TEXT,                    -- Extrait du texte ou l'entite est mentionnee
    confidence      REAL DEFAULT 0.8,       -- Confiance de l'extraction (0.0 - 1.0)
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================
-- TABLE : hypotheses
-- ============================================================
CREATE TABLE hypotheses (
    id              TEXT PRIMARY KEY,        -- UUID v4
    case_id         TEXT NOT NULL REFERENCES cases(id),
    title           TEXT NOT NULL,           -- Titre court (ex: "Implication du conjoint")
    description     TEXT NOT NULL,           -- Description detaillee
    status          TEXT DEFAULT 'active',   -- active | refuted | confirmed | merged
    current_score   REAL DEFAULT 50.0,      -- Score actuel (0-100)
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================
-- TABLE : hypothesis_snapshots (Evolution dans le temps)
-- ============================================================
CREATE TABLE hypothesis_snapshots (
    id              TEXT PRIMARY KEY,
    hypothesis_id   TEXT NOT NULL REFERENCES hypotheses(id),
    score           REAL NOT NULL,           -- Score a cet instant
    supporting      TEXT,                    -- JSON array d'elements supportant
    contradicting   TEXT,                    -- JSON array d'elements contredisant
    reasoning       TEXT,                    -- Raisonnement du LLM a cet instant
    trigger         TEXT,                    -- Ce qui a declenche la reevaluation
    model_used      TEXT,                    -- Quel modele a fait l'analyse
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================
-- TABLE : analysis_runs (Historique des analyses)
-- ============================================================
CREATE TABLE analysis_runs (
    id              TEXT PRIMARY KEY,
    case_id         TEXT NOT NULL REFERENCES cases(id),
    run_type        TEXT NOT NULL,           -- full | incremental | verification | extraction
    trigger         TEXT,                    -- manual | new_evidence | monitoring | scheduled
    status          TEXT DEFAULT 'running',  -- running | completed | failed
    model_used      TEXT,                    -- Modele(s) utilise(s)
    input_summary   TEXT,                    -- Resume de l'input
    output_summary  TEXT,                    -- Resume du resultat
    duration_sec    REAL,                    -- Duree en secondes
    tokens_used     INTEGER,                -- Tokens consommes
    started_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at    DATETIME
);

-- ============================================================
-- TABLE : monitoring_jobs (Jobs de surveillance)
-- ============================================================
CREATE TABLE monitoring_jobs (
    id              TEXT PRIMARY KEY,
    case_id         TEXT NOT NULL REFERENCES cases(id),
    job_type        TEXT NOT NULL,           -- searxng | robin | both
    query           TEXT NOT NULL,           -- Requete de recherche
    entity_id       TEXT REFERENCES entities(id),  -- Entite surveillee (optionnel)
    interval_hours  INTEGER DEFAULT 24,     -- Frequence en heures
    is_active       BOOLEAN DEFAULT 1,
    last_run        DATETIME,
    next_run        DATETIME,
    results_count   INTEGER DEFAULT 0,      -- Nombre total de resultats trouves
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================
-- TABLE : monitoring_results (Resultats de surveillance)
-- ============================================================
CREATE TABLE monitoring_results (
    id              TEXT PRIMARY KEY,
    job_id          TEXT NOT NULL REFERENCES monitoring_jobs(id),
    case_id         TEXT NOT NULL REFERENCES cases(id),
    url             TEXT,
    title           TEXT,
    snippet         TEXT,
    source_engine   TEXT,                    -- google | duckduckgo | brave | robin
    relevance_score REAL,                    -- Score de pertinence (0-100)
    is_new          BOOLEAN DEFAULT 1,       -- Nouveau depuis la derniere fois
    is_duplicate    BOOLEAN DEFAULT 0,       -- Doublon detecte via embeddings
    reviewed        BOOLEAN DEFAULT 0,       -- Lu par l'utilisateur
    found_at        DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================
-- TABLE : alerts
-- ============================================================
CREATE TABLE alerts (
    id              TEXT PRIMARY KEY,
    case_id         TEXT NOT NULL REFERENCES cases(id),
    alert_type      TEXT NOT NULL,           -- new_evidence | score_shift | monitoring_hit |
                                             -- contradiction | new_entity
    severity        TEXT DEFAULT 'info',     -- info | warning | critical
    title           TEXT NOT NULL,
    message         TEXT NOT NULL,
    related_id      TEXT,                    -- ID de l'objet concerne (evidence, hypothesis, etc.)
    is_read         BOOLEAN DEFAULT 0,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Index pour les requetes frequentes
CREATE INDEX idx_evidence_case ON evidence(case_id);
CREATE INDEX idx_entities_case ON entities(case_id);
CREATE INDEX idx_hypotheses_case ON hypotheses(case_id);
CREATE INDEX idx_snapshots_hyp ON hypothesis_snapshots(hypothesis_id);
CREATE INDEX idx_monitoring_case ON monitoring_jobs(case_id);
CREATE INDEX idx_alerts_case_read ON alerts(case_id, is_read);
CREATE INDEX idx_analysis_case ON analysis_runs(case_id);
```

### 3.2 Neo4j -- Graphe relationnel

**Types de noeuds :**

```cypher
-- Noeuds principaux
(:Person {id, name, aliases, role, description, case_id})
(:Location {id, name, address, lat, lon, location_type, case_id})
(:Phone {id, number, carrier, case_id})
(:Vehicle {id, plate, make, model, color, case_id})
(:Organization {id, name, org_type, case_id})
(:Account {id, identifier, platform, account_type, case_id})
(:Event {id, description, datetime, location, case_id})
(:Evidence {id, title, evidence_type, reliability, case_id})
(:Money {id, amount, currency, description, case_id})

-- Noeuds speciaux
(:Hypothesis {id, title, current_score, case_id})
(:Case {id, name, reference})
```

**Types de relations :**

```cypher
-- Relations entre personnes
(:Person)-[:KNOWS {since, context, strength}]->(:Person)
(:Person)-[:RELATED_TO {relationship}]->(:Person)          -- famille, collegue, etc.
(:Person)-[:COMMUNICATED_WITH {count, last_date, channel}]->(:Person)
(:Person)-[:FINANCIAL_LINK {type, amount, dates}]->(:Person)

-- Relations personne <-> lieu
(:Person)-[:LIVES_AT {since, until}]->(:Location)
(:Person)-[:WAS_AT {datetime, source, confirmed}]->(:Location)
(:Person)-[:WORKS_AT {since, until}]->(:Location)
(:Person)-[:FREQUENTS {frequency}]->(:Location)

-- Relations personne <-> objet
(:Person)-[:OWNS {since}]->(:Vehicle)
(:Person)-[:OWNS {since}]->(:Phone)
(:Person)-[:OWNS {since}]->(:Account)
(:Person)-[:MEMBER_OF {role, since}]->(:Organization)

-- Relations temporelles
(:Event)-[:OCCURRED_AT]->(:Location)
(:Event)-[:INVOLVES]->(:Person)
(:Event)-[:PRECEDED_BY {gap_hours}]->(:Event)

-- Relations d'evidence
(:Evidence)-[:MENTIONS]->(:Person)
(:Evidence)-[:MENTIONS]->(:Location)
(:Evidence)-[:MENTIONS]->(:Event)
(:Evidence)-[:SUPPORTS]->(:Hypothesis)
(:Evidence)-[:CONTRADICTS]->(:Hypothesis)

-- Relations financieres
(:Person)-[:SENT_MONEY {amount, date, method}]->(:Person)
(:Person)-[:RECEIVED_MONEY {amount, date, method}]->(:Person)
(:Account)-[:TRANSACTION {amount, date, description}]->(:Account)
```

### 3.3 ChromaDB -- Collections vectorielles

```python
# Collection 1 : Embeddings des preuves (texte complet)
collection_evidence = {
    "name": "evidence_texts",
    "embedding_function": "nomic-embed-text",  # via Ollama
    "metadata_schema": {
        "case_id": "string",
        "evidence_id": "string",
        "evidence_type": "string",
        "source": "string",
        "reliability": "integer",
        "date": "string"
    }
}

# Collection 2 : Embeddings des entites (description + contexte)
collection_entities = {
    "name": "entity_contexts",
    "embedding_function": "nomic-embed-text",
    "metadata_schema": {
        "case_id": "string",
        "entity_id": "string",
        "entity_type": "string",
        "name": "string"
    }
}

# Collection 3 : Resultats de monitoring (deduplication)
collection_monitoring = {
    "name": "monitoring_results",
    "embedding_function": "nomic-embed-text",
    "metadata_schema": {
        "case_id": "string",
        "job_id": "string",
        "url": "string",
        "source_engine": "string",
        "found_at": "string"
    }
}

# Collection 4 : Snapshots d'hypotheses (recherche semantique)
collection_hypotheses = {
    "name": "hypothesis_reasoning",
    "embedding_function": "nomic-embed-text",
    "metadata_schema": {
        "case_id": "string",
        "hypothesis_id": "string",
        "score": "float",
        "snapshot_date": "string"
    }
}
```

---

## 4. Logique de routage multi-modeles

### 4.1 Tableau de routage

| Tache                          | Modele               | Raison                                  | Timeout |
| ------------------------------ | -------------------- | --------------------------------------- | ------- |
| Extraction d'entites           | `gemma4:e4b` (4B)    | Rapide, structuree, JSON                | 30s     |
| Reformulation de requetes      | `gemma4:e4b` (4B)    | Rapide, tache simple                    | 15s     |
| Filtrage de resultats          | `gemma4:e4b` (4B)    | Classification binaire rapide           | 20s     |
| Structuration JSON             | `gemma4:e4b` (4B)    | Formatage mecanique                     | 15s     |
| Resume de preuve               | `gemma4:e4b` (4B)    | Resume factuel                          | 30s     |
| Embeddings                     | `nomic-embed-text`    | Modele dedie aux embeddings             | 10s     |
| Verification logique           | `deepseek-r1` (14B)  | Chain-of-thought, detection d'erreurs   | 120s    |
| Detection de contradictions    | `deepseek-r1` (14B)  | Raisonnement adversarial                | 120s    |
| Comparaison de temoignages     | `deepseek-r1` (14B)  | Analyse comparative precise             | 120s    |
| Analyse profonde               | `nexus` (26B)         | Raisonnement complexe multi-facteurs    | 300s    |
| Scoring d'hypotheses           | `nexus` (26B)         | Ponderation nuancee, contexte large     | 300s    |
| Rapport final                  | `nexus` (26B)         | Synthese complete, qualite redactionnelle| 600s   |
| Re-evaluation incrementale     | `nexus` (26B)         | Necessite le contexte complet           | 300s    |

### 4.2 Implementation du routeur

```python
# nexus/llm/router.py -- Schema conceptuel

from enum import Enum

class TaskType(Enum):
    # Taches legeres -> gemma4:e4b
    ENTITY_EXTRACTION = "entity_extraction"
    QUERY_REFORMULATION = "query_reformulation"
    RESULT_FILTERING = "result_filtering"
    JSON_STRUCTURING = "json_structuring"
    EVIDENCE_SUMMARY = "evidence_summary"

    # Embeddings -> nomic-embed-text
    EMBEDDING = "embedding"

    # Raisonnement -> deepseek-r1
    LOGIC_VERIFICATION = "logic_verification"
    CONTRADICTION_DETECTION = "contradiction_detection"
    TESTIMONY_COMPARISON = "testimony_comparison"

    # Analyse profonde -> nexus (26B)
    DEEP_ANALYSIS = "deep_analysis"
    HYPOTHESIS_SCORING = "hypothesis_scoring"
    FINAL_REPORT = "final_report"
    INCREMENTAL_REEVAL = "incremental_reeval"

MODEL_ROUTING = {
    # gemma4:e4b -- taches rapides
    TaskType.ENTITY_EXTRACTION:     {"model": "gemma4:e4b",              "timeout": 30},
    TaskType.QUERY_REFORMULATION:   {"model": "gemma4:e4b",              "timeout": 15},
    TaskType.RESULT_FILTERING:      {"model": "gemma4:e4b",              "timeout": 20},
    TaskType.JSON_STRUCTURING:      {"model": "gemma4:e4b",              "timeout": 15},
    TaskType.EVIDENCE_SUMMARY:      {"model": "gemma4:e4b",              "timeout": 30},

    # nomic-embed-text -- embeddings
    TaskType.EMBEDDING:             {"model": "nomic-embed-text",        "timeout": 10},

    # deepseek-r1 -- raisonnement
    TaskType.LOGIC_VERIFICATION:    {"model": "deepseek-r1-abliterated:14b", "timeout": 120},
    TaskType.CONTRADICTION_DETECTION:{"model": "deepseek-r1-abliterated:14b","timeout": 120},
    TaskType.TESTIMONY_COMPARISON:  {"model": "deepseek-r1-abliterated:14b", "timeout": 120},

    # nexus -- analyse profonde
    TaskType.DEEP_ANALYSIS:         {"model": "nexus",                   "timeout": 300},
    TaskType.HYPOTHESIS_SCORING:    {"model": "nexus",                   "timeout": 300},
    TaskType.FINAL_REPORT:          {"model": "nexus",                   "timeout": 600},
    TaskType.INCREMENTAL_REEVAL:    {"model": "nexus",                   "timeout": 300},
}
```

### 4.3 Pipeline d'analyse multi-modeles

```
Nouvelle preuve uploadee
        |
        v
[1] gemma4:e4b -- Extraction d'entites (JSON)
        |
        +--> Sauvegarde entites dans SQLite + Neo4j
        +--> Embeddings via nomic-embed-text --> ChromaDB
        |
        v
[2] gemma4:e4b -- Resume de la preuve
        |
        +--> Sauvegarde resume dans SQLite
        |
        v
[3] nexus (26B) -- Analyse profonde
        |   Input : resume + entites + hypotheses existantes + contexte du dossier
        |
        +--> Nouvelles hypotheses ou mise a jour des scores
        |
        v
[4] deepseek-r1 (14B) -- Verification
        |   Input : hypotheses + raisonnement du nexus + preuves contradictoires
        |
        +--> Validation ou correction des scores
        +--> Detection de contradictions
        |
        v
[5] Sauvegarde des snapshots dans SQLite
        |
        +--> Si ecart de score > 15 points --> Alerte
```

**Important : gestion VRAM (RTX 5080 = 16GB)**

Un seul modele lourd en memoire a la fois. Le routeur doit :
1. Executer toutes les taches `gemma4:e4b` en premier (modele leger, ~3GB VRAM)
2. Decharger et charger `nexus` (26B, ~15GB VRAM en Q4)
3. Decharger et charger `deepseek-r1` (14B, ~9GB VRAM)

Ollama gere le chargement/dechargement automatiquement, mais il faut eviter les appels
concurrents a des modeles differents (OOM). Le routeur doit serialiser les appels par
taille de modele.

```python
# Strategie de sequencement VRAM :
# 1. Batch toutes les taches e4b (petit modele, rapide)
# 2. Puis taches nexus 26B (gros modele, lent)
# 3. Puis taches deepseek-r1 14B (moyen)
# Jamais deux gros modeles en parallele.
```

---

## 5. Docker Compose

```yaml
# docker-compose.yml
# Lance : neo4j, chromadb, robin (searxng est deja actif separement)

version: "3.8"

services:
  # ============================================================
  # Neo4j -- Base de donnees graphe
  # ============================================================
  neo4j:
    image: neo4j:5-community
    container_name: nexus-neo4j
    ports:
      - "7474:7474"   # Interface web Neo4j Browser
      - "7687:7687"   # Bolt protocol
    environment:
      NEO4J_AUTH: neo4j/changeme
      NEO4J_PLUGINS: '["apoc"]'
      NEO4J_server_memory_heap_initial__size: "512m"
      NEO4J_server_memory_heap_max__size: "1g"
    volumes:
      - ./data/neo4j/data:/data
      - ./data/neo4j/logs:/logs
    restart: unless-stopped
    healthcheck:
      test: ["CMD-SHELL", "wget --no-verbose --tries=1 --spider http://localhost:7474 || exit 1"]
      interval: 15s
      timeout: 10s
      retries: 5

  # ============================================================
  # ChromaDB -- Base vectorielle
  # ============================================================
  chromadb:
    image: chromadb/chroma:latest
    container_name: nexus-chromadb
    ports:
      - "8100:8000"   # Port 8100 en externe (8000 est pris par FastAPI)
    volumes:
      - ./data/chroma:/chroma/chroma
    environment:
      ANONYMIZED_TELEMETRY: "false"
      IS_PERSISTENT: "true"
    restart: unless-stopped
    healthcheck:
      test: ["CMD-SHELL", "curl -f http://localhost:8000/api/v1/heartbeat || exit 1"]
      interval: 15s
      timeout: 10s
      retries: 5

  # ============================================================
  # Robin -- Recherche dark web via Tor
  # (SearXNG clearweb est deja actif sur port 8888, pas inclus ici)
  # ============================================================
  robin:
    image: pwnfoo/robin:latest
    container_name: nexus-robin
    ports:
      - "9090:9090"
    restart: unless-stopped
    # Robin utilise Tor en interne pour les recherches .onion
    # API REST sur port 9090

# Note : SearXNG tourne deja sur le host en Docker sur port 8888.
# Si besoin de l'integrer ici plus tard, ajouter le service depuis
# la config existante dans searxng/settings.yml

volumes:
  neo4j_data:
  chroma_data:
```

**.env (a creer a la racine) :**

```bash
# === NEXUS Configuration ===

# FastAPI
NEXUS_HOST=0.0.0.0
NEXUS_PORT=8000
NEXUS_DEBUG=true

# Ollama
OLLAMA_BASE_URL=http://localhost:11434

# Neo4j
NEO4J_URI=bolt://localhost:7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=changeme

# ChromaDB
CHROMA_HOST=localhost
CHROMA_PORT=8100

# SearXNG (deja actif)
SEARXNG_URL=http://localhost:8888

# Robin (dark web)
ROBIN_URL=http://localhost:9090

# Stockage
DATA_DIR=./data
UPLOAD_DIR=./data/uploads
SQLITE_PATH=./data/nexus.db
```

---

## Phase 1 -- MVP : Stockage + Ingestion + Analyse de base

### Objectif

Systeme fonctionnel minimum : creer un dossier, uploader des preuves (PDF, texte, images),
extraire les entites, lancer une analyse de base, voir les resultats.
**Pas de monitoring, pas de Neo4j, pas de ChromaDB.** Uniquement SQLite + Ollama.

### Dependances

- Python 3.13
- FastAPI, uvicorn, python-multipart
- SQLite (inclus dans Python)
- Ollama avec les modeles : `gemma4:e4b`, `nexus`
- PyMuPDF (extraction PDF), Pillow (images)

### Fichiers a creer

```
nexus/__init__.py
nexus/main.py                     # App FastAPI + startup
nexus/config.py                   # Chargement .env, constantes
nexus/api/__init__.py
nexus/api/cases.py                # POST/GET/PUT/DELETE /cases
nexus/api/evidence.py             # POST /cases/{id}/evidence (upload)
                                  # GET  /cases/{id}/evidence
nexus/api/entities.py             # GET  /cases/{id}/entities
nexus/api/analysis.py             # POST /cases/{id}/analyze
nexus/core/__init__.py
nexus/core/case_manager.py        # CRUD cases
nexus/core/evidence_processor.py  # Pipeline : upload -> parse -> extract -> store
nexus/core/entity_extractor.py    # Appel gemma4:e4b pour NER
nexus/core/analysis_pipeline.py   # Orchestration : e4b -> nexus
nexus/llm/__init__.py
nexus/llm/router.py               # Routage par TaskType
nexus/llm/ollama_client.py        # Client HTTP pour Ollama /api/generate et /api/embed
nexus/llm/prompts.py              # Templates de prompts
nexus/llm/parsers.py              # JSON parsing des reponses LLM
nexus/db/__init__.py
nexus/db/sqlite_db.py             # Init DB, migrations, helpers
nexus/db/models.py                # Modeles Pydantic (Case, Evidence, Entity, etc.)
nexus/ingest/__init__.py
nexus/ingest/pdf_parser.py        # PyMuPDF -> texte
nexus/ingest/text_parser.py       # Lecture fichiers texte
requirements.txt                  # Dependances Phase 1
.env                              # Configuration
```

### Endpoints API -- Phase 1

```
POST   /api/cases                          # Creer un dossier
GET    /api/cases                          # Lister les dossiers
GET    /api/cases/{case_id}                # Details d'un dossier
PUT    /api/cases/{case_id}                # Modifier un dossier
DELETE /api/cases/{case_id}                # Supprimer un dossier

POST   /api/cases/{case_id}/evidence       # Uploader une preuve (multipart/form-data)
GET    /api/cases/{case_id}/evidence       # Lister les preuves d'un dossier
GET    /api/evidence/{evidence_id}         # Details d'une preuve
PUT    /api/evidence/{evidence_id}         # Modifier metadonnees d'une preuve
DELETE /api/evidence/{evidence_id}         # Supprimer une preuve

GET    /api/cases/{case_id}/entities       # Lister les entites extraites
GET    /api/entities/{entity_id}           # Details d'une entite

POST   /api/cases/{case_id}/analyze        # Lancer une analyse complete
GET    /api/analysis/{run_id}              # Statut d'une analyse en cours
GET    /api/cases/{case_id}/analysis-runs  # Historique des analyses

GET    /api/health                         # Health check (DB + Ollama)
```

### Complexite estimee

| Composant                | Effort     | Notes                                        |
| ------------------------ | ---------- | -------------------------------------------- |
| Config + structure       | 2h         | Boilerplate FastAPI, .env, SQLite init        |
| CRUD cases               | 2h         | Standard REST                                |
| Upload + parsing         | 4h         | Multipart upload, PDF extraction, stockage    |
| Client Ollama            | 3h         | generate + embed, gestion erreurs, timeouts   |
| Extraction entites       | 4h         | Prompt engineering, parsing JSON, dedup       |
| Pipeline d'analyse       | 6h         | Orchestration e4b -> nexus, gestion VRAM      |
| Modeles Pydantic         | 2h         | Validation, serialisation                     |
| Tests unitaires          | 4h         | Minimum : client Ollama, parsers, CRUD        |
| **Total Phase 1**        | **~27h**   | ~3-4 jours de travail                         |

### Details d'implementation

**Prompt d'extraction d'entites (gemma4:e4b) :**

```
Tu es un extracteur d'entites. Analyse le texte suivant et extrais TOUTES les entites
dans un format JSON strict.

TEXTE :
{text}

Reponds UNIQUEMENT avec un JSON valide dans ce format exact :
{
  "entities": [
    {
      "name": "...",
      "type": "person|location|phone|vehicle|organization|date|money|email|other",
      "context": "phrase ou l'entite apparait",
      "confidence": 0.0-1.0
    }
  ]
}
```

**Pipeline d'ingestion d'une preuve :**

```
1. Reception du fichier (POST multipart)
2. Sauvegarde dans data/uploads/{case_id}/{uuid}.{ext}
3. Extraction du texte :
   - PDF -> PyMuPDF (fitz)
   - Image -> placeholder (Phase 1 : texte alternatif seulement)
   - Texte -> lecture directe
4. Hash SHA256 du fichier (deduplication)
5. Insertion dans SQLite (table evidence, status='processing')
6. Envoi du texte a gemma4:e4b pour extraction d'entites
7. Sauvegarde des entites dans SQLite (table entities + entity_mentions)
8. Envoi du texte a gemma4:e4b pour resume
9. Mise a jour evidence.summary + evidence.status='processed'
```

### Validation Phase 1

La phase est terminee quand :
- [ ] On peut creer un dossier via l'API
- [ ] On peut uploader un PDF et un fichier texte
- [ ] Le texte est extrait automatiquement
- [ ] Les entites sont extraites par gemma4:e4b et stockees
- [ ] On peut lancer une analyse qui produit un rapport textuel via nexus (26B)
- [ ] L'historique des analyses est conserve
- [ ] `GET /api/health` repond avec le statut de SQLite et Ollama

---

## Phase 2 -- Graphe relationnel + Recherche vectorielle

### Objectif

Ajouter Neo4j pour visualiser les relations entre entites, et ChromaDB pour la recherche
semantique (trouver des preuves similaires, detecter les doublons).

### Dependances

- Phase 1 complete
- Docker Compose (Neo4j + ChromaDB)
- neo4j (driver Python)
- chromadb (client Python)
- nomic-embed-text (modele Ollama pour embeddings)

### Fichiers a creer / modifier

```
docker-compose.yml                  # CREER -- Neo4j + ChromaDB
nexus/db/neo4j_db.py               # CREER -- Client Neo4j
nexus/db/chroma_db.py              # CREER -- Client ChromaDB
nexus/api/graph.py                 # CREER -- Endpoints graphe
nexus/api/search.py                # CREER -- Endpoints recherche semantique
nexus/core/evidence_processor.py   # MODIFIER -- Ajouter etapes Neo4j + ChromaDB
nexus/core/entity_extractor.py     # MODIFIER -- Pousser entites vers Neo4j
nexus/core/timeline_builder.py     # CREER -- Construction timeline depuis Neo4j
nexus/main.py                      # MODIFIER -- Ajouter startup Neo4j + ChromaDB
nexus/config.py                    # MODIFIER -- Ajouter config Neo4j + ChromaDB
```

### Endpoints API -- Phase 2 (nouveaux)

```
GET    /api/cases/{case_id}/graph                  # Graphe complet (noeuds + aretes)
GET    /api/cases/{case_id}/graph/neighbors/{id}   # Voisins d'un noeud
GET    /api/cases/{case_id}/graph/path/{from}/{to}  # Plus court chemin entre deux entites
GET    /api/cases/{case_id}/graph/clusters          # Clusters detectes

POST   /api/cases/{case_id}/search                 # Recherche semantique dans les preuves
GET    /api/cases/{case_id}/similar/{evidence_id}  # Preuves similaires a une preuve donnee
GET    /api/cases/{case_id}/duplicates             # Doublons detectes

GET    /api/cases/{case_id}/timeline               # Timeline des evenements
```

### Complexite estimee

| Composant                  | Effort     | Notes                                     |
| -------------------------- | ---------- | ----------------------------------------- |
| Docker Compose             | 2h         | Neo4j + ChromaDB, volumes, health checks  |
| Client Neo4j               | 4h         | CRUD noeuds/aretes, requetes Cypher       |
| Client ChromaDB            | 3h         | Collections, embed via Ollama, queries    |
| Integration pipeline       | 4h         | Modifier ingestion pour pousser partout   |
| Endpoints graphe           | 4h         | Serialisation graphe, pathfinding         |
| Recherche semantique       | 3h         | Similarity search, deduplication          |
| Timeline builder           | 3h         | Extraction dates, ordonnancement          |
| Tests                      | 3h         | Neo4j mock, ChromaDB tests                |
| **Total Phase 2**          | **~26h**   | ~3-4 jours de travail                     |

### Details d'implementation

**Synchronisation entites -> Neo4j :**

A chaque extraction d'entite, le systeme :
1. Cree ou met a jour le noeud correspondant dans Neo4j
2. Cree les relations deduites du contexte (ex: "Jean appelle Marie" -> COMMUNICATED_WITH)
3. Lie le noeud Evidence au noeud entite via MENTIONS

**Deduction de relations via LLM (gemma4:e4b) :**

```
Prompt : "Voici des entites extraites d'un meme document. Identifie les RELATIONS entre
elles. Reponds en JSON."

Input : [{"name": "Jean Dupont", "type": "person"}, {"name": "Marie Martin", "type": "person"},
         {"name": "06 12 34 56 78", "type": "phone"}]

Output attendu :
{
  "relations": [
    {"from": "Jean Dupont", "to": "Marie Martin", "type": "COMMUNICATED_WITH", "context": "..."},
    {"from": "Jean Dupont", "to": "06 12 34 56 78", "type": "OWNS", "context": "..."}
  ]
}
```

**Recherche semantique ChromaDB :**

Quand l'utilisateur cherche "temoignage sur la voiture rouge", le systeme :
1. Embede la requete avec nomic-embed-text
2. Query ChromaDB collection `evidence_texts` avec filtre `case_id`
3. Retourne les N preuves les plus proches avec score de similarite

### Validation Phase 2

- [ ] `docker compose up` lance Neo4j et ChromaDB sans erreur
- [ ] L'upload d'une preuve cree des noeuds dans Neo4j
- [ ] Le graphe est requetable via l'API
- [ ] La recherche semantique retourne des resultats pertinents
- [ ] Les doublons sont detectes par similarite vectorielle (seuil > 0.92)

---

## Phase 3 -- Monitoring automatise (SearXNG + Tor)

### Objectif

Surveillance continue des entites d'un dossier sur le clearweb (SearXNG) et le dark web
(Robin/Tor). Le systeme cherche periodiquement de nouvelles mentions et notifie l'utilisateur.

### Dependances

- Phase 2 complete
- SearXNG actif sur port 8888 (deja en place)
- Robin (Docker) sur port 9090
- APScheduler
- ChromaDB (pour deduplication des resultats)

### Fichiers a creer / modifier

```
nexus/monitoring/__init__.py            # CREER
nexus/monitoring/scheduler.py           # CREER -- Config APScheduler
nexus/monitoring/searxng_monitor.py     # CREER -- Client SearXNG JSON API
nexus/monitoring/robin_monitor.py       # CREER -- Client Robin API
nexus/monitoring/alert_manager.py       # CREER -- Creation + envoi d'alertes
nexus/api/monitoring.py                 # CREER -- CRUD jobs de surveillance
nexus/api/alerts.py                     # CREER -- Lecture + gestion alertes
nexus/core/analysis_pipeline.py        # MODIFIER -- Integrer resultats monitoring
nexus/main.py                          # MODIFIER -- Demarrer APScheduler au startup
```

### Endpoints API -- Phase 3 (nouveaux)

```
POST   /api/cases/{case_id}/monitoring              # Creer un job de surveillance
GET    /api/cases/{case_id}/monitoring              # Lister les jobs actifs
PUT    /api/monitoring/{job_id}                     # Modifier un job (frequence, requete)
DELETE /api/monitoring/{job_id}                     # Supprimer un job
POST   /api/monitoring/{job_id}/run                 # Forcer une execution immediate

GET    /api/cases/{case_id}/monitoring/results      # Resultats de surveillance
GET    /api/monitoring/results/{result_id}          # Detail d'un resultat
POST   /api/monitoring/results/{result_id}/ingest   # Convertir un resultat en preuve

GET    /api/cases/{case_id}/alerts                  # Lister les alertes
PUT    /api/alerts/{alert_id}/read                  # Marquer comme lue
GET    /api/alerts/unread-count                     # Nombre d'alertes non lues
```

### Complexite estimee

| Composant                  | Effort     | Notes                                      |
| -------------------------- | ---------- | ------------------------------------------ |
| APScheduler setup          | 2h         | Integration FastAPI, persistance jobs      |
| Client SearXNG             | 3h         | API JSON, parsing resultats, pagination    |
| Client Robin               | 4h         | API Tor, gestion latence, retry            |
| Deduplication resultats    | 3h         | Embeddings + seuil similarite ChromaDB     |
| Filtrage pertinence        | 3h         | gemma4:e4b pour scorer la pertinence       |
| Systeme d'alertes          | 3h         | Creation, stockage, compteurs              |
| Endpoints API              | 3h         | CRUD monitoring + alertes                  |
| Integration pipeline       | 3h         | Resultats monitoring -> preuves -> analyse |
| Tests                      | 3h         | Mocks SearXNG/Robin, tests scheduler       |
| **Total Phase 3**          | **~27h**   | ~3-4 jours de travail                      |

### Details d'implementation

**Flux de monitoring :**

```
APScheduler declenche un job toutes les N heures
        |
        v
[1] Recuperer les requetes associees au job (entites surveillees)
        |
        v
[2] Reformuler chaque requete via gemma4:e4b (variations, synonymes)
        |   Ex: "Jean Dupont disparu" -> ["Jean Dupont missing", "Dupont disparition 2019", ...]
        |
        v
[3] Envoyer chaque variante a SearXNG (JSON API) et/ou Robin
        |
        +--> SearXNG: GET http://localhost:8888/search?q={query}&format=json
        +--> Robin:   GET http://localhost:9090/search?q={query}
        |
        v
[4] Pour chaque resultat :
        +--> Embedding via nomic-embed-text
        +--> Comparaison avec monitoring_results dans ChromaDB
        +--> Si similarite > 0.92 --> marquer is_duplicate=True, ignorer
        +--> Si nouveau :
              +--> gemma4:e4b evalue la pertinence (0-100)
              +--> Si pertinence > 40 --> sauvegarder dans monitoring_results
              +--> Si pertinence > 70 --> creer une alerte
```

**Conversion resultat -> preuve :**

L'utilisateur peut transformer un resultat de monitoring en preuve officielle du dossier.
Endpoint `POST /api/monitoring/results/{result_id}/ingest` :
1. Cree une entree dans la table `evidence` avec `evidence_type='url'`
2. Tente de recuperer le contenu de la page (requests ou Robin pour .onion)
3. Lance le pipeline d'ingestion standard (extraction entites, embeddings, etc.)

### Validation Phase 3

- [ ] On peut creer un job de surveillance pour une entite
- [ ] Le job s'execute automatiquement selon l'intervalle defini
- [ ] Les resultats SearXNG sont recuperes et filtres
- [ ] Les doublons sont detectes et ignores
- [ ] Les resultats pertinents generent des alertes
- [ ] Un resultat peut etre converti en preuve du dossier

---

## Phase 4 -- Hypotheses evolutives + Snapshots

### Objectif

Systeme d'hypotheses qui evoluent dans le temps. Chaque re-evaluation cree un snapshot.
L'utilisateur voit l'evolution des scores sur un graphique temporel. Le systeme detecte
automatiquement les contradictions et les changements significatifs.

### Dependances

- Phase 3 complete (monitoring fournit de nouvelles donnees)
- deepseek-r1-abliterated:14b (verification logique)
- nexus 26B (scoring)

### Fichiers a creer / modifier

```
nexus/core/hypothesis_engine.py         # CREER -- Moteur d'hypotheses complet
nexus/core/contradiction_detector.py    # CREER -- Detection contradictions via deepseek-r1
nexus/api/hypotheses.py                 # CREER -- CRUD + evolution hypotheses
nexus/core/analysis_pipeline.py        # MODIFIER -- Integrer re-evaluation automatique
nexus/monitoring/alert_manager.py      # MODIFIER -- Alertes sur changements de score
nexus/llm/prompts.py                   # MODIFIER -- Ajouter prompts hypotheses + verification
```

### Endpoints API -- Phase 4 (nouveaux)

```
POST   /api/cases/{case_id}/hypotheses              # Creer une hypothese (manuelle ou LLM)
GET    /api/cases/{case_id}/hypotheses              # Lister les hypotheses
GET    /api/hypotheses/{hyp_id}                     # Details d'une hypothese
PUT    /api/hypotheses/{hyp_id}                     # Modifier une hypothese
DELETE /api/hypotheses/{hyp_id}                     # Supprimer/archiver

POST   /api/hypotheses/{hyp_id}/evaluate            # Forcer une re-evaluation
GET    /api/hypotheses/{hyp_id}/snapshots           # Historique des snapshots
GET    /api/hypotheses/{hyp_id}/evolution            # Donnees pour graphique d'evolution

POST   /api/cases/{case_id}/hypotheses/generate     # Generer des hypotheses via LLM
POST   /api/cases/{case_id}/evaluate-all            # Re-evaluer TOUTES les hypotheses

GET    /api/cases/{case_id}/contradictions          # Contradictions detectees
```

### Complexite estimee

| Composant                   | Effort     | Notes                                     |
| --------------------------- | ---------- | ----------------------------------------- |
| Moteur d'hypotheses         | 6h         | Scoring, snapshots, merge                 |
| Detection contradictions    | 5h         | Prompts deepseek-r1, parsing resultats    |
| Generation d'hypotheses     | 4h         | nexus 26B genere des hypotheses initiales |
| Re-evaluation automatique   | 4h         | Trigger sur nouvelle preuve/monitoring    |
| Alertes score shifts        | 2h         | Seuil configurable (defaut: 15 points)   |
| Endpoints API               | 3h         | CRUD + evolution + contradictions         |
| Tests                       | 3h         | Tests scoring, snapshot, detection        |
| **Total Phase 4**           | **~27h**   | ~3-4 jours de travail                     |

### Details d'implementation

**Pipeline de re-evaluation :**

```
Trigger : nouvelle preuve OU resultat monitoring OU demande manuelle
        |
        v
[1] Charger toutes les hypotheses actives du dossier
        |
        v
[2] Pour chaque hypothese :
        |
        +--> [nexus 26B] Scoring avec contexte complet :
        |       - Description de l'hypothese
        |       - TOUTES les preuves (resumes)
        |       - Entites et relations (depuis Neo4j)
        |       - Score precedent et son raisonnement
        |       - Nouvelles donnees depuis le dernier snapshot
        |
        |       Prompt :
        |       "Re-evalue cette hypothese. Ancien score : {old_score}%.
        |        Nouvelles donnees : {new_data}.
        |        Reponds en JSON : {score, supporting: [...], contradicting: [...], reasoning}"
        |
        +--> [deepseek-r1] Verification du raisonnement :
        |       "Verifie ce raisonnement pour des erreurs logiques,
        |        biais de confirmation ou contradictions internes.
        |        Raisonnement : {nexus_reasoning}
        |        Preuves : {evidence_summaries}"
        |
        +--> Sauvegarde du snapshot :
                - Score
                - Elements supportant/contredisant
                - Raisonnement
                - Resultat de verification
                - Modeles utilises
                - Trigger de la re-evaluation
        |
        v
[3] Comparer ancien et nouveau score
        +--> Si |delta| > 15 --> alerte "score_shift"
        +--> Si score < 10 et etait > 50 --> suggerer status "refuted"
        +--> Si score > 90 et confirm par deepseek --> suggerer status "confirmed"
```

**Format de snapshot (stocke dans hypothesis_snapshots) :**

```json
{
  "score": 72.5,
  "supporting": [
    {"evidence_id": "abc-123", "summary": "Temoin confirme la presence", "weight": 0.8},
    {"evidence_id": "def-456", "summary": "Releve telephonique coherent", "weight": 0.7}
  ],
  "contradicting": [
    {"evidence_id": "ghi-789", "summary": "Alibi partiel confirme", "weight": 0.5}
  ],
  "reasoning": "Le score augmente de 65 a 72.5 suite au nouveau temoignage...",
  "verification": {
    "model": "deepseek-r1-abliterated:14b",
    "issues_found": [],
    "confidence": 0.85
  },
  "trigger": "new_evidence:abc-123"
}
```

### Validation Phase 4

- [ ] On peut creer des hypotheses manuellement et via LLM
- [ ] Chaque re-evaluation cree un snapshot horodate
- [ ] L'endpoint /evolution retourne des donnees de serie temporelle
- [ ] Les contradictions sont detectees par deepseek-r1
- [ ] Les changements de score > 15 points generent des alertes
- [ ] Le scoring utilise le pipeline complet e4b -> nexus -> deepseek-r1

---

## Phase 5 -- Dashboard web complet

### Objectif

Interface web Streamlit permettant d'interagir avec tout le systeme sans toucher l'API
directement. Visualisation du graphe, timeline, evolution des hypotheses, alertes.

### Dependances

- Phases 1-4 completes (toute l'API est disponible)
- Streamlit
- streamlit-agraph (visualisation graphe)
- plotly (graphiques interactifs)
- streamlit-timeline (composant timeline)

### Fichiers a creer

```
frontend/app.py                          # Point d'entree Streamlit + navigation
frontend/api_client.py                   # Client HTTP vers FastAPI
frontend/pages/01_dashboard.py           # Vue d'ensemble : stats, alertes recentes, dossiers
frontend/pages/02_evidence.py            # Upload, liste, details des preuves
frontend/pages/03_entities.py            # Liste des entites + filtre par type
frontend/pages/04_hypotheses.py          # Hypotheses + graphique d'evolution
frontend/pages/05_timeline.py            # Timeline interactive
frontend/pages/06_graph.py               # Visualisation du graphe relationnel
frontend/pages/07_monitoring.py          # Gestion des jobs de surveillance
frontend/pages/08_alerts.py              # Centre d'alertes
frontend/pages/09_analysis.py            # Historique d'analyses + lancement manuel
frontend/components/graph_viewer.py      # Composant graphe (streamlit-agraph)
frontend/components/hypothesis_chart.py  # Composant Plotly pour l'evolution des scores
frontend/components/timeline_viewer.py   # Composant timeline
frontend/components/evidence_card.py     # Carte de preuve (affichage compact)
```

### Pages du dashboard

**01 - Dashboard (vue d'ensemble)**
```
+-------------------------------------------------------------------+
|  NEXUS -- Cold Case Analyst                    [Dossier actif: v]  |
+-------------------------------------------------------------------+
|                                                                     |
|  Statistiques                    Alertes recentes                   |
|  +---------------------------+  +-------------------------------+   |
|  | Preuves : 47              |  | [!] Score H2 : 65% -> 42%    |   |
|  | Entites : 123             |  | [+] Nouveau resultat SearXNG  |   |
|  | Hypotheses actives : 5    |  | [i] Analyse terminee           |   |
|  | Jobs monitoring : 3       |  +-------------------------------+   |
|  +---------------------------+                                      |
|                                                                     |
|  Derniere analyse : il y a 2h                                       |
|  Hypothese principale : H1 (Implication du conjoint) -- 78%        |
|                                                                     |
+-------------------------------------------------------------------+
```

**04 - Hypotheses (evolution temporelle)**
```
+-------------------------------------------------------------------+
|  Hypotheses -- Dossier #2019-4472                                  |
+-------------------------------------------------------------------+
|                                                                     |
|  Score (%)                                                          |
|  100|                                                               |
|   80|  H1----*---*------*---*                                       |
|   60|     H2--*-----*                                               |
|   40|              \                                                |
|   20|               *---*---*  H2                                   |
|    0+----+----+----+----+----+----> Temps                          |
|     S1   S2   S3   S4   S5   S6                                    |
|                                                                     |
|  [H1] Implication du conjoint    78%  [Evaluer] [Details]          |
|  [H2] Crime opportuniste         23%  [Evaluer] [Details]          |
|  [H3] Lien professionnel         61%  [Evaluer] [Details]          |
|                                                                     |
|  [+ Nouvelle hypothese]  [Re-evaluer toutes]  [Generer via IA]    |
+-------------------------------------------------------------------+
```

**06 - Graphe relationnel**
```
+-------------------------------------------------------------------+
|  Graphe relationnel -- Dossier #2019-4472                          |
+-------------------------------------------------------------------+
|  Filtres : [x] Personnes [x] Lieux [ ] Telephones [x] Vehicules  |
|                                                                     |
|  +-------------------------------------------------------------+  |
|  |                                                               |  |
|  |          [Jean]---KNOWS---[Marie]                            |  |
|  |            |                  |                               |  |
|  |         OWNS              WAS_AT                              |  |
|  |            |                  |                               |  |
|  |        [06 12..]          [Cafe X]---NEAR---[Parking Y]      |  |
|  |            |                                    |             |  |
|  |      COMMUNICATED_WITH                       WAS_AT          |  |
|  |            |                                    |             |  |
|  |          [Pierre]----------OWNS-----------[Peugeot 308]      |  |
|  |                                                               |  |
|  +-------------------------------------------------------------+  |
|                                                                     |
|  Noeud selectionne : Jean Dupont                                   |
|  Connexions : 4 | Mentions dans 12 preuves | Vu dans 3 hypotheses |
+-------------------------------------------------------------------+
```

### Complexite estimee

| Composant                   | Effort     | Notes                                     |
| --------------------------- | ---------- | ----------------------------------------- |
| Structure Streamlit + nav   | 2h         | Multi-page app, sidebar, theme            |
| Client API                  | 3h         | Wrapper requests vers FastAPI             |
| Dashboard overview          | 3h         | Stats, alertes, resume                    |
| Page preuves                | 4h         | Upload, liste, details, filtres           |
| Page entites                | 2h         | Liste, filtres par type, recherche        |
| Page hypotheses             | 5h         | Graphique Plotly, CRUD, evolution         |
| Page timeline               | 4h         | Timeline interactive, filtres             |
| Page graphe                 | 6h         | streamlit-agraph, filtres, interaction    |
| Page monitoring             | 3h         | CRUD jobs, resultats, conversion          |
| Page alertes                | 2h         | Liste, filtres, mark as read              |
| Page analyse                | 2h         | Historique, lancement                     |
| Composants reutilisables    | 3h         | Cards, viewers, charts                    |
| **Total Phase 5**           | **~39h**   | ~5-6 jours de travail                     |

### Validation Phase 5

- [ ] Le dashboard affiche les stats du dossier actif
- [ ] On peut uploader des preuves via l'interface
- [ ] Le graphe relationnel est interactif (zoom, clic sur noeuds)
- [ ] Le graphique d'evolution des hypotheses est fonctionnel
- [ ] La timeline affiche les evenements chronologiquement
- [ ] Les alertes sont visibles et marquables comme lues
- [ ] On peut lancer une analyse depuis l'interface

---

## Phase 6 -- Rapports + Export + Hardening

### Objectif

Generation de rapports complets exportables en PDF et Markdown. Consolidation du systeme :
gestion d'erreurs robuste, logs, rate limiting, sauvegarde automatique.

### Dependances

- Phase 5 complete
- WeasyPrint ou reportlab (generation PDF)
- Jinja2 (templates de rapports)

### Fichiers a creer / modifier

```
nexus/export/__init__.py                 # CREER
nexus/export/report_generator.py         # CREER -- Generation du rapport complet
nexus/export/pdf_export.py               # CREER -- Conversion en PDF
nexus/export/timeline_export.py          # CREER -- Export timeline (image/PDF)
nexus/export/templates/                  # CREER -- Templates Jinja2
nexus/export/templates/full_report.html  # Template rapport complet
nexus/export/templates/summary.html      # Template resume executif
nexus/export/templates/timeline.html     # Template timeline
nexus/api/reports.py                     # CREER -- Endpoints generation/download
frontend/pages/08_reports.py             # MODIFIER -- Interface de generation
nexus/main.py                           # MODIFIER -- Logging, error handling, CORS
nexus/core/backup.py                    # CREER -- Backup SQLite + export Neo4j
```

### Endpoints API -- Phase 6 (nouveaux)

```
POST   /api/cases/{case_id}/reports/generate    # Generer un rapport (type: full|summary|timeline)
GET    /api/reports/{report_id}                 # Statut de generation
GET    /api/reports/{report_id}/download         # Telecharger le rapport (PDF ou MD)
GET    /api/cases/{case_id}/reports              # Lister les rapports generes

POST   /api/backup                              # Lancer une sauvegarde
GET    /api/backup/list                         # Lister les sauvegardes
POST   /api/backup/restore/{backup_id}          # Restaurer une sauvegarde
```

### Complexite estimee

| Composant                   | Effort     | Notes                                     |
| --------------------------- | ---------- | ----------------------------------------- |
| Rapport complet (MD)        | 5h         | Template Jinja2 + donnees de tous les modules |
| Export PDF                  | 4h         | WeasyPrint rendering, mise en page        |
| Resume executif             | 3h         | nexus 26B genere un resume synthetique    |
| Export timeline             | 2h         | Image ou PDF de la timeline               |
| Interface rapports          | 3h         | Choix du type, preview, download          |
| Backup/restore              | 3h         | SQLite copy, Neo4j export, ChromaDB dump  |
| Logging structure           | 2h         | Loguru/structlog, rotation fichiers       |
| Gestion d'erreurs           | 3h         | Middleware FastAPI, retry Ollama, fallbacks|
| Rate limiting Ollama        | 2h         | Queue avec priorites, backpressure        |
| Tests integration           | 4h         | Tests end-to-end : upload -> analyse -> rapport |
| **Total Phase 6**           | **~31h**   | ~4-5 jours de travail                     |

### Structure d'un rapport complet

```markdown
# RAPPORT D'INVESTIGATION
## Dossier : {case.name} ({case.reference})
## Date du rapport : {date}
## Genere par NEXUS

---

### 1. Resume executif
{resume genere par nexus 26B : 1-2 paragraphes}

### 2. Donnees ingeres
- {n} preuves analysees
- Types : {repartition par type}
- Fiabilite moyenne : {score moyen}
- Sources : {liste des sources}

### 3. Entites identifiees
#### Personnes ({count})
| Nom | Role | Premiere mention | Connexions |
| ... | ...  | ...              | ...        |

#### Lieux ({count})
...

### 4. Chronologie des evenements
{timeline textuelle ordonnee}

### 5. Graphe relationnel
{description textuelle des relations cles}
{si PDF : image du graphe embedded}

### 6. Hypotheses
#### H1 : {titre} -- Score actuel : {score}%
- Evolution : {historique des scores}
- Elements supportant : {liste}
- Elements contredisant : {liste}
- Verification logique : {resultat deepseek}

#### H2 : ...

### 7. Contradictions detectees
{liste des contradictions avec references aux preuves}

### 8. Surveillance active
- {n} jobs actifs
- {n} resultats trouves (dont {m} pertinents)
- Dernieres alertes : {liste}

### 9. Pistes d'investigation recommandees
{generees par nexus 26B, ordonnees par impact potentiel}

### 10. Angles morts
{informations manquantes qui pourraient changer l'analyse}

---
Rapport genere automatiquement par NEXUS v1.0
Modeles utilises : gemma4:e4b, deepseek-r1-abliterated:14b, nexus (Gemma 4 26B)
```

### Validation Phase 6

- [ ] Un rapport complet PDF est genereable en un clic
- [ ] Le rapport inclut toutes les sections (resume, entites, hypotheses, timeline)
- [ ] Le resume executif est genere par nexus 26B
- [ ] Les sauvegardes fonctionnent (SQLite + Neo4j + ChromaDB)
- [ ] Les logs sont structures et persistes dans un fichier
- [ ] Les erreurs Ollama (timeout, OOM) sont gerees gracieusement avec retry

---

## Recapitulatif des phases

| Phase | Objectif                              | Effort estime | Dependances          |
| ----- | ------------------------------------- | ------------- | -------------------- |
| 1     | MVP : stockage + ingestion + analyse  | ~27h          | Aucune               |
| 2     | Graphe Neo4j + recherche vectorielle  | ~26h          | Phase 1              |
| 3     | Monitoring SearXNG + Tor              | ~27h          | Phase 2              |
| 4     | Hypotheses evolutives + snapshots     | ~27h          | Phase 3              |
| 5     | Dashboard Streamlit                   | ~39h          | Phases 1-4           |
| 6     | Rapports + export + hardening         | ~31h          | Phase 5              |
| **Total** |                                   | **~177h**     | ~22-25 jours         |

**Note :** Chaque phase est independamment utilisable via l'API.
Le dashboard (Phase 5) est un bonus ergonomique mais n'est pas requis pour que le
systeme fonctionne. On peut utiliser les Phases 1-4 entierement via curl/Postman.

---

## Dependances Python (requirements.txt complet)

```
# === Backend ===
fastapi==0.115.*
uvicorn[standard]==0.34.*
python-multipart==0.0.18
python-dotenv==1.1.*
pydantic==2.11.*
httpx==0.28.*

# === Base de donnees ===
neo4j==5.28.*
chromadb==0.6.*
aiosqlite==0.21.*

# === LLM ===
ollama==0.4.*                   # Client Python officiel Ollama

# === Ingestion ===
PyMuPDF==1.25.*                 # Extraction texte PDF
Pillow==11.*                    # Traitement images
python-magic==0.4.*             # Detection type MIME

# === Scheduling ===
APScheduler==3.11.*

# === Frontend ===
streamlit==1.44.*
streamlit-agraph==0.0.45        # Visualisation graphe
plotly==6.0.*                   # Graphiques interactifs

# === Export ===
weasyprint==63.*                # Generation PDF
Jinja2==3.1.*                   # Templates

# === Utils ===
loguru==0.7.*                   # Logging structure
tenacity==9.0.*                 # Retry logic
```

---

## Commandes de demarrage

```bash
# 1. Creer l'environnement conda
conda create -n nexus python=3.13 -y
conda activate nexus

# 2. Installer les dependances
pip install -r requirements.txt

# 3. Verifier les modeles Ollama
ollama list
# Doit montrer : nexus, gemma4:e4b, deepseek-r1-abliterated:14b, nomic-embed-text
# Si manquant :
ollama pull gemma4:e4b
ollama pull nomic-embed-text

# 4. Lancer les services Docker (Phase 2+)
docker compose up -d

# 5. Lancer le backend
cd C:\Users\FlowUP\Desktop\cold-case-analyst
uvicorn nexus.main:app --host 0.0.0.0 --port 8000 --reload

# 6. Lancer le frontend (Phase 5+)
streamlit run frontend/app.py --server.port 8501

# 7. Acceder aux interfaces
# FastAPI docs  : http://localhost:8000/docs
# Streamlit     : http://localhost:8501
# Neo4j Browser : http://localhost:7474
# SearXNG       : http://localhost:8888
```
