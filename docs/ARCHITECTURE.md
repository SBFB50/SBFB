# NEXUS -- Architecture technique complete

**Version :** 0.1.0
**Date :** 2026-04-05
**Auteur :** Documentation generee par analyse exhaustive du code source

---

## Table des matieres

1. [Vue d'ensemble](#1-vue-densemble)
2. [Modules](#2-modules)
3. [Base de donnees](#3-base-de-donnees)
4. [API REST](#4-api-rest)
5. [Boucle autonome OODA](#5-boucle-autonome-ooda)
6. [Routage LLM](#6-routage-llm)
7. [Audit trail](#7-audit-trail)
8. [Frontend](#8-frontend)
9. [Problemes connus et limitations](#9-problemes-connus-et-limitations)
10. [Roadmap](#10-roadmap)

---

## 1. Vue d'ensemble

### Description

NEXUS est un systeme d'investigation persistant concu pour les cold cases. Contrairement a un chatbot one-shot, NEXUS accumule de l'intelligence sur des semaines ou des mois en combinant ingestion de preuves, extraction automatique d'entites, monitoring continu multi-sources (clearweb + dark web), evaluation d'hypotheses evolutives, et boucle autonome d'investigation basee sur le modele OODA (Observe-Orient-Decide-Act) etendu avec auto-questionnement.

Le systeme est entierement local (aucune donnee ne quitte la machine) et utilise des modeles LLM uncensored/abliterated pour ne pas refuser l'analyse de contenu sensible lie a des affaires criminelles.

### Stack technique

| Composant | Technologie | Version | Role |
|-----------|-------------|---------|------|
| Backend | FastAPI | >=0.115 | API REST, orchestration |
| Frontend | Streamlit | >=1.44 | Dashboard interactif |
| LLMs | Ollama | local | 5 modeles specialises |
| Donnees | SQLite (aiosqlite) | >=0.21 | Stockage principal |
| Graphe | Neo4j 5 Community | Docker | Graphe de connaissances |
| Vecteurs | ChromaDB | >=0.6, Docker | Recherche semantique |
| Recherche clearweb | SearXNG | Docker/host | Monitoring web |
| Recherche dark web | Robin (apurvsg/robin) | Docker | Monitoring Tor |
| Planification | APScheduler | >=3.11 | Jobs recurrents |
| PDF parsing | PyMuPDF (fitz) | >=1.25 | Extraction texte/images |
| Export PDF | WeasyPrint + Jinja2 | >=63 / >=3.1 | Rapports PDF |
| Vision DL | transformers + torch | >=4.46 / >=2.5 | DINOv2, CLIP |
| OSINT | holehe, python-whois | >=1.61 / >=0.9 | Recon email/domaine |
| Visualisation | Plotly, Folium | >=6.0 / >=0.19 | Graphiques, cartes |
| Graphe UI | streamlit-agraph | >=0.0.45 | Visualisation Neo4j |
| Physique | SciPy | >=1.14 | Simulations forensiques |
| HTTP client | httpx | >=0.28 | Appels async |
| Retry | tenacity | >=9.0 | Retry automatique |
| Logging | loguru | >=0.7 | Logs structures |
| Validation | Pydantic v2 | >=2.11 | Modeles de donnees |

### Environnement

- **OS :** Windows 11 Pro
- **GPU :** NVIDIA RTX 5080 (16 GB VRAM partagee)
- **Python :** 3.13 (conda)
- **Container runtime :** Docker Desktop (Neo4j, ChromaDB, Robin)
- **SearXNG :** Instance separee sur le host (port 8888)
- **Ollama :** Natif Windows (port 11434)

### Diagramme d'architecture

```
                          +------------------------+
                          |       FRONTEND         |
                          |    Streamlit (8501)    |
                          |    15 pages + 4 comp.  |
                          +-----------+------------+
                                      |
                                      | HTTP (requests sync)
                                      v
                          +------------------------+
                          |       BACKEND          |
                          |    FastAPI (8000)      |
                          |  102 endpoints REST    |
                          |  19 routers + health   |
                          +-----------+------------+
                                      |
          +---------------+-----------+-----------+---------------+
          |               |           |           |               |
          v               v           v           v               v
   +-----------+   +-----------+  +--------+  +--------+  +----------+
   |  SQLite   |   |  Neo4j    |  | Chroma |  | Ollama |  |  SearXNG |
   | (aiosqlite|   |  (Bolt    |  | DB     |  | 5 LLMs |  | (8888)   |
   |  WAL mode)|   |   7687)   |  | (8100) |  |(11434) |  +----------+
   |  13 tables|   | 11 labels |  | 4 coll.|  |        |
   +-----------+   |  22 rels  |  +--------+  +--------+  +----------+
                   +-----------+                           |  Robin   |
                                                           | Tor/dark |
                                                           | (Docker) |
                                                           +----------+

   +---------------------------------------------------------------+
   |                  BOUCLE AUTONOME (par case)                   |
   |  OBSERVE -> ORIENT -> DECIDE -> ACT -> QUESTION -> SLEEP     |
   |  (30 min/cycle)          InvestigationManager                 |
   +---------------------------------------------------------------+

   +---------------------------------------------------------------+
   |                  MONITORING (APScheduler)                     |
   |  MonitoringScheduler -> SearXNG + Robin -> dedup ChromaDB    |
   |  -> filtrage LLM -> alerts                                   |
   +---------------------------------------------------------------+
```

### Ports reseau

| Service | Port | Protocole |
|---------|------|-----------|
| FastAPI backend | 8000 | HTTP |
| Streamlit frontend | 8501 | HTTP |
| Ollama | 11434 | HTTP |
| Neo4j Browser | 7474 | HTTP |
| Neo4j Bolt | 7687 | Bolt |
| ChromaDB | 8100 | HTTP |
| SearXNG | 8888 | HTTP |
| Robin (UI Streamlit) | 8502 | HTTP |
| Open WebUI | 3000 | HTTP |

---

## 2. Modules

### 2.1 `nexus/` -- Package racine

| Fichier | Role |
|---------|------|
| `__init__.py` | Package marker |
| `main.py` | Point d'entree FastAPI, lifespan, middleware, routers |
| `config.py` | Configuration centralisee (pydantic-settings), MODEL_ROUTING |

#### `main.py` -- Application FastAPI

**Responsabilite :** Demarrage de l'application avec lifespan async (init DB, singletons, scheduler, investigation manager), middleware CORS + X-Process-Time, montage des 19 routers, gestion d'erreurs Ollama.

**Classes :**
- `_InterceptHandler(logging.Handler)` -- Redirige les logs stdlib vers Loguru
- `ProcessTimeMiddleware(BaseHTTPMiddleware)` -- Header X-Process-Time

**Fonctions :**
- `lifespan(app)` -- Startup: init SQLite, Neo4j, ChromaDB, LLMRouter, MonitoringScheduler, InvestigationManager. Shutdown: arret propre.
- `health_check()` -- GET /api/health, liveness probe
- `_ollama_exception_handler()` -- 503 pour erreurs Ollama

**Dependances importees :** config, tous les routers API, tous les clients DB, OllamaClient, LLMRouter, MonitoringScheduler, InvestigationManager

**Importe par :** uvicorn (point d'entree)

**Tests :** Non

#### `config.py` -- Configuration

**Responsabilite :** Charge les variables d'environnement via pydantic-settings, expose `settings` singleton et la table `MODEL_ROUTING`.

**Classes :**
- `Settings(BaseSettings)` -- 35+ parametres types avec defauts

**Constantes :**
- `MODEL_ROUTING: Dict[str, str]` -- Table de routage task -> modele Ollama

**Importe par :** Quasi tous les modules du systeme

**Tests :** Non

---

### 2.2 `nexus/llm/` -- Couche LLM

| Fichier | Role |
|---------|------|
| `ollama_client.py` | Client async Ollama (generate, chat, embed) |
| `router.py` | Routeur multi-modeles avec VRAM lock |
| `prompts.py` | Templates de prompts systeme (francais) |
| `parsers.py` | Parsing des reponses JSON des LLMs |

#### `ollama_client.py` -- OllamaClient

**Responsabilite :** Client unifie async pour toutes les interactions Ollama. Retry automatique (tenacity), logging structure, gestion OOM.

**Classe : `OllamaClient`**

| Methode | Parametres | Retour | Description |
|---------|-----------|--------|-------------|
| `generate()` | model, prompt, system?, format?, timeout? | `str` | Generation texte |
| `generate_json()` | model, prompt, system?, timeout? | `dict` | Generation JSON (format="json") |
| `generate_with_image()` | model, prompt, image_path, system?, timeout? | `str` | Vision (chat avec images base64) |
| `embed()` | text, model? | `list[float]` | Embedding single |
| `embed_batch()` | texts, model? | `list[list[float]]` | Embedding batch |
| `check_health()` | -- | `bool` | Connectivity check |
| `list_models()` | -- | `list[str]` | Liste des modeles locaux |

**SDK Ollama utilise :** `AsyncClient(host=...)`, `client.generate()`, `client.chat()`, `client.embed()`, `client.list()`

**Pattern de retry :** tenacity avec `stop_after_attempt(3)`, `wait_exponential(1, 1, 8)`, retry sur `httpx.ConnectError` et `httpx.TimeoutException`

**Importe par :** `router.py`

**Tests :** Non

#### `router.py` -- LLMRouter

**Responsabilite :** Dispatch chaque tache vers le modele optimal. Gere la serialisation VRAM via `asyncio.Lock` pour les modeles lourds.

**Classe : `LLMRouter`**

| Methode | Description |
|---------|-------------|
| `route(task_type, prompt, system?)` | Generation texte via le bon modele |
| `route_json(task_type, prompt, system?)` | Generation JSON via le bon modele |
| `route_vision(task_type, prompt, image_path, system?)` | Vision via VLM |
| `embed(text)` | Embedding single |
| `embed_batch(texts)` | Embedding batch |

**Enum : `TaskType`** -- 20 types de taches (voir section 6)

**Table : `_ROUTE_TABLE`** -- Mapping TaskType -> (model_attr, timeout_sec, is_heavy)

**Importe par :** Tous les modules core, API deps, autonomous_loop

**Tests :** Non

#### `prompts.py` -- Templates

**Responsabilite :** Templates de prompts en francais avec placeholders `{variable}`. Couvre : vision (4), extraction (3), filtrage (2), analyse (2), hypotheses (4), contradiction (2), monitoring (2), auto-questionnement (1), rapport (1).

**Importe par :** core/, monitoring/, API routers

#### `parsers.py` -- Parsing JSON

**Responsabilite :** Extraction et nettoyage de JSON depuis les reponses LLM (regex pour blocs ```json, fallback json.loads direct).

**Fonctions :** `parse_entities()`, `parse_relations()`, `safe_json_parse()`

---

### 2.3 `nexus/db/` -- Couche donnees

| Fichier | Role |
|---------|------|
| `sqlite_db.py` | Schema DDL, get_db(), classe Database (CRUD complet) |
| `neo4j_db.py` | Client async Neo4j (CRUD noeuds/relations, graph queries) |
| `chroma_db.py` | Client ChromaDB (4 collections, search, dedup) |
| `models.py` | Modeles Pydantic v2 (schemas request/response) |

#### `sqlite_db.py` -- Database

**Responsabilite :** Couche de persistance principale. Schema 13 tables, indexes, CRUD complet async.

**Fonctions module-level :**
- `init_db()` -- CREATE TABLE IF NOT EXISTS + indexes (WAL mode, FK ON)
- `get_db()` -- asynccontextmanager yielding `aiosqlite.Connection`

**Classe : `Database(conn)`** -- CRUD pour chaque table. Pattern : `async with get_db() as conn: db = Database(conn)`

**Methodes par table :**
- Cases : create, get, list, update, delete (cascade)
- Evidence : create, get, list_by_case, update, delete
- Entities : create, get, list_by_case, update
- EntityMentions : create, list_by_entity, list_by_evidence
- Hypotheses : create, get, list_by_case, update, delete
- HypothesisSnapshots : create, list_by_hypothesis, get_latest
- AnalysisRuns : create, get, update, list_by_case
- MonitoringJobs : create, get, list_by_case, update, delete
- MonitoringResults : create, get, list_by_case, list_by_job, update
- Alerts : create, list_by_case, mark_read, count_unread
- Reports : create, get, list_by_case, update
- Locations : create, get, list_by_case
- AuditLog : create, get, list_by_case, get_timeline

**Importe par :** Tous les modules core et API

#### `neo4j_db.py` -- Neo4jClient

**Responsabilite :** Graphe de connaissances. CRUD noeuds/relations, queries de graphe, synchronisation depuis SQLite.

**Classe : `Neo4jClient`**

| Methode | Description |
|---------|-------------|
| `init_constraints()` | Contraintes d'unicite sur `id` pour chaque label |
| `create_or_update_node(label, props)` | MERGE par id |
| `get_node(node_id)` | Fetch par id |
| `delete_node(node_id)` | DETACH DELETE |
| `find_nodes_by_case(case_id, label?)` | Filtrage par case_id |
| `create_relation(from_id, to_id, rel_type, props?)` | MERGE relation |
| `get_relations(node_id, direction?)` | Relations d'un noeud |
| `delete_relation(from_id, to_id, rel_type)` | Suppression relation |
| `get_full_graph(case_id)` | Graphe complet -> {nodes, edges} |
| `get_neighbors(node_id, depth?)` | Sous-graphe voisinage |
| `find_shortest_path(from_id, to_id)` | Plus court chemin |
| `find_clusters(case_id)` | Composantes connexes (BFS) |
| `get_node_stats(case_id)` | Count par label |
| `sync_entity(entity, case_id)` | SQLite entity -> Neo4j node |
| `sync_relations(relations, case_id)` | Sync batch de relations |
| `sync_evidence(evidence_id, ...)` | Evidence -> Neo4j node |
| `link_evidence_to_entity(evidence_id, entity_id)` | Relation MENTIONS |

**SDK Neo4j utilise :** `AsyncGraphDatabase.driver()`, `session.execute_read()`, `session.execute_write()`, `AsyncManagedTransaction`

**Importe par :** main.py, deps.py, autonomous_loop.py, API graph

#### `chroma_db.py` -- ChromaClient

**Responsabilite :** Stockage vectoriel pour recherche semantique. 4 collections avec embeddings pre-calcules (nomic-embed-text via Ollama).

**Classe : `ChromaClient`**

| Methode | Collection | Description |
|---------|-----------|-------------|
| `add_evidence()` | evidence_texts | Ajout embedding preuve |
| `search_evidence()` | evidence_texts | Recherche semantique |
| `find_similar_evidence()` | evidence_texts | Preuves similaires |
| `find_duplicates()` | evidence_texts | Paires quasi-duplicats (O(n^2)) |
| `delete_evidence()` | evidence_texts | Suppression |
| `add_entity()` | entity_contexts | Ajout embedding entite |
| `search_entities()` | entity_contexts | Recherche semantique |
| `add_monitoring_result()` | monitoring_results | Stockage pour dedup |
| `is_duplicate_result()` | monitoring_results | Dedup semantique (seuil 0.92) |
| `add_hypothesis_snapshot()` | hypothesis_reasoning | Snapshot hypothese |
| `search_hypotheses()` | hypothesis_reasoning | Recherche semantique |
| `get_collection_stats()` | toutes | Comptage items |
| `clear_case_data()` | toutes | Purge par case_id |

**SDK ChromaDB utilise :** `chromadb.HttpClient(host, port)`, `get_or_create_collection(name, embedding_function=None, metadata={"hnsw:space": "cosine"})`, `collection.add(ids, documents, embeddings, metadatas)`, `collection.query(query_embeddings, n_results, where, include)`, `collection.get(ids?, where?, include?)`, `collection.delete(ids)`

**Importe par :** main.py, deps.py, monitoring/scheduler.py, autonomous_loop.py

#### `models.py` -- Schemas Pydantic v2

**Responsabilite :** 30+ modeles Pydantic v2 organises en triplets Base/Create/Response pour chaque entite.

**Entites modelisees :**

| Entite | Modeles | Champs cles |
|--------|---------|-------------|
| Case | CaseBase, CaseCreate, CaseUpdate, Case | name, reference, status |
| Evidence | EvidenceBase, EvidenceCreate, EvidenceUpdate, Evidence | case_id, title, evidence_type, reliability, raw_text |
| Entity | EntityBase, EntityCreate, EntityUpdate, Entity | case_id, name, entity_type, aliases |
| EntityMention | EntityMentionBase, EntityMentionCreate, EntityMention | entity_id, evidence_id, confidence |
| Hypothesis | HypothesisBase, HypothesisCreate, HypothesisUpdate, Hypothesis | case_id, title, description, status, current_score |
| HypothesisSnapshot | HypothesisSnapshotBase, HypothesisSnapshotCreate, HypothesisSnapshot | hypothesis_id, score, reasoning |
| AnalysisRun | AnalysisRunBase, AnalysisRunCreate, AnalysisRunUpdate, AnalysisRun | case_id, run_type, trigger, status |
| MonitoringJob | MonitoringJobBase, MonitoringJobCreate, MonitoringJobUpdate, MonitoringJob | case_id, job_type, query, interval_hours |
| MonitoringResult | MonitoringResultBase, MonitoringResultCreate, MonitoringResult | job_id, case_id, url, relevance_score |
| Alert | AlertBase, AlertCreate, Alert | case_id, alert_type, severity, message |
| Report | ReportBase, ReportCreate, Report | case_id, report_type, status, file_path |
| AuditEntry | AuditEntryBase, AuditEntry | case_id, actor, action, summary, cycle_number |

**Patterns Pydantic v2 :**
- `model_config = {"from_attributes": True}` pour la serialisation depuis sqlite3.Row
- `Field(default=50, ge=0, le=100)` pour les contraintes numeriques
- `Literal["active", "closed", "archived"]` pour les enums typees
- `Optional[Any] = None` pour les champs JSON flexibles

---

### 2.4 `nexus/core/` -- Logique metier

| Module | Responsabilite | Modeles LLM | Dependances principales |
|--------|---------------|-------------|------------------------|
| `analysis_pipeline.py` | Analyse sequentielle multi-modeles | gemma4:e4b, nexus 26B, deepseek-r1 14B | Database, LLMRouter |
| `audit.py` | Audit trail 3 couches (SQLite + JSONL + Git) | -- | Database |
| `autonomous_loop.py` | Boucle OODA par case | Tous | Tous les modules |
| `backup.py` | Backups ZIP (SQLite + metadata) | -- | config |
| `case_manager.py` | CRUD cases haut niveau | -- | Database |
| `contradiction_detector.py` | Detection contradictions par paires | deepseek-r1 14B | Database, LLMRouter |
| `entity_extractor.py` | Extraction entites + relations | gemma4:e4b | LLMRouter, parsers |
| `evidence_processor.py` | Pipeline ingestion complete | gemma4:e4b (summary, entities) | Database, LLMRouter, Neo4j, ChromaDB |
| `geo_mapper.py` | Geocoding (Nominatim), routing (OSRM), verification temps de trajet | -- | Database, httpx |
| `hypothesis_engine.py` | Generation, evaluation, fusion hypotheses | nexus 26B, deepseek-r1 14B | Database, LLMRouter |
| `image_analyzer.py` | Pipeline analyse visuelle complete | gemma4:e4b (fast), qwen3-vl:8b (deep) | LLMRouter, Database, ChromaDB |
| `investigation_manager.py` | Gestion 1 AutonomousInvestigator par case | -- | autonomous_loop |
| `timeline_builder.py` | Construction chronologie unifiee | -- | Database |

#### `autonomous_loop.py` -- AutonomousInvestigator

**Responsabilite :** Cerveau de NEXUS. Boucle OODA modifiee (5 phases) executee en continu pour chaque case actif.

**Classe : `AutonomousInvestigator(case_id, router, chroma, neo4j)`**

| Methode | Phase OODA | Description |
|---------|-----------|-------------|
| `run()` | -- | Boucle principale, cycle toutes les 30 min |
| `stop()` | -- | Arret propre apres le cycle courant |
| `_observe()` | OBSERVE | Check monitoring results non-revus a haute pertinence |
| `_orient(results)` | ORIENT | Auto-ingest, OSINT recon, geocode, image analysis, visual embeddings |
| `_decide(evidence_ids)` | DECIDE | Analyse incrementale, hypotheses, contradictions, forensics, timeline |
| `_act(decisions)` | ACT | Genere nouvelles requetes, enrichissement OSINT, recon domaine |
| `_question()` | QUESTION | Auto-questionnement, rapports periodiques, backups |
| `get_status()` | -- | Status dict pour l'API |

**Modules connectes (21 au total) :** EvidenceProcessor, AnalysisPipeline, HypothesisEngine, ContradictionDetector, AlertManager, HoleheRecon, SocialRecon, DomainRecon, GeoMapper, ImageAnalyzer, ImageSearchEngine, VisualEmbedder, BloodPatternAnalyzer, TraceAnalyzer, AcousticAnalyzer, TimelineBuilder, ReportGenerator, BackupManager, AuditService

**Importe par :** `investigation_manager.py`

#### `investigation_manager.py` -- InvestigationManager

**Responsabilite :** Gere un `AutonomousInvestigator` par case actif. Demarre/arrete via le lifespan FastAPI.

**Classe : `InvestigationManager(router, chroma, neo4j)`**

| Methode | Description |
|---------|-------------|
| `start()` | Demarre un investigateur pour chaque case actif en DB |
| `start_investigation(case_id)` | Demarre un investigateur pour un case specifique |
| `stop_investigation(case_id)` | Arrete l'investigateur d'un case |
| `stop_all()` | Arret global propre |
| `get_status()` | Status de toutes les investigations |
| `get_investigation_status(case_id)` | Status d'une investigation specifique |

#### `analysis_pipeline.py` -- AnalysisPipeline

**Responsabilite :** Analyse sequentielle en 5 etapes : (1) resumage evidence non-resumee [gemma4:e4b], (2) analyse profonde [nexus 26B], (3) re-scoring hypotheses [nexus 26B], (4) verification logique [deepseek-r1 14B], (5) sauvegarde + alertes.

#### `hypothesis_engine.py` -- HypothesisEngine

**Responsabilite :** Generation de nouvelles hypotheses, evaluation avec supporting/contradicting evidence, re-evaluation incrementale, fusion d'hypotheses.

#### `contradiction_detector.py` -- ContradictionDetector

**Responsabilite :** Compare les preuves par paires (entites communes) via deepseek-r1 14B pour detecter les contradictions.

#### `evidence_processor.py` -- EvidenceProcessor

**Responsabilite :** Pipeline d'ingestion : sauvegarde fichier, detection type, extraction texte (PDF/text), creation DB, extraction entites LLM, generation summary, sync Neo4j + ChromaDB.

---

### 2.5 `nexus/monitoring/` -- Surveillance

| Module | Responsabilite |
|--------|---------------|
| `scheduler.py` | APScheduler AsyncIOScheduler, orchestre les jobs de monitoring |
| `searxng_monitor.py` | Client async SearXNG (httpx) |
| `robin_monitor.py` | Client Robin dark web (docker exec CLI) |
| `alert_manager.py` | Creation d'alertes typees |

#### `scheduler.py` -- MonitoringScheduler

**Responsabilite :** Orchestre les jobs de monitoring recurrents. Charge les jobs actifs au demarrage, execute a l'intervalle configure.

**Pipeline d'execution par job :**
1. Charger la definition du job depuis SQLite
2. Executer la recherche (SearXNG et/ou Robin)
3. Pour chaque resultat : embedding -> dedup ChromaDB -> scoring LLM -> stockage SQLite -> embedding ChromaDB -> alerte si pertinent

**APScheduler utilise :** `AsyncIOScheduler(timezone="UTC")`, `IntervalTrigger(hours=...)`, `add_job()`, `remove_job()`, `reschedule_job()`, `modify_job(next_run_time=...)`

**Configuration :**
```python
job_defaults = {
    "coalesce": True,       # Fusionne les runs manques
    "max_instances": 1,     # Pas de runs paralleles du meme job
    "misfire_grace_time": 3600,  # Tolere 1h de retard
}
```

#### `robin_monitor.py` -- RobinMonitor

**Responsabilite :** Interface avec Robin (pas d'API REST). Utilise `docker exec` en subprocess pour lancer des recherches Tor.

#### `alert_manager.py` -- AlertManager

**Responsabilite :** Cree des alertes typees : `new_evidence`, `score_shift`, `monitoring_hit`, `contradiction`, `new_entity`. Severite : `info`, `warning`, `critical`.

---

### 2.6 `nexus/forensics/` -- Analyse forensique

| Module | Responsabilite | Modeles LLM |
|--------|---------------|-------------|
| `blood_pattern.py` | Classification BPA, calculs geometriques | gemma4:e4b, qwen3-vl:8b |
| `acoustic_analysis.py` | Transcription, analyse audio forensique, detection evenements, propagation | voxtral-mini:4b |
| `trace_analyzer.py` | Analyse de traces physiques (empreintes, outils, pneus) | qwen3-vl:8b |
| `physics_sim.py` | Simulations physiques (goutte sang, cast-off, son) | -- (numpy/scipy) |
| `the_well_loader.py` | Chargement datasets PolymathicAI/TheWell | -- |

---

### 2.7 `nexus/recon/` -- OSINT

| Module | Responsabilite | Dependances externes |
|--------|---------------|---------------------|
| `holehe_recon.py` | Check email sur 120+ services | holehe (subprocess) |
| `social_recon.py` | Check username sur plateformes majeures | httpx (HEAD requests) |
| `domain_recon.py` | WHOIS + DNS lookup | python-whois, socket |

---

### 2.8 `nexus/vision/` -- Vision par ordinateur

| Module | Responsabilite | Modeles DL |
|--------|---------------|-----------|
| `embeddings.py` | Embeddings visuels DINOv2 (768-dim) + CLIP (512-dim) | facebook/dinov2-base, openai/clip-vit-base-patch32 |
| `image_search.py` | Recherche image-to-image et text-to-image | ChromaDB (2 collections dediees) |

**Collections ChromaDB additionnelles :**
- `image_dinov2` -- 768 dimensions, image-to-image
- `image_clip` -- 512 dimensions, text-to-image

---

### 2.9 `nexus/ingest/` -- Ingestion

| Module | Responsabilite |
|--------|---------------|
| `pdf_parser.py` | Extraction texte + images depuis PDF (PyMuPDF/fitz) |
| `text_parser.py` | Lecture fichiers texte avec detection encoding |

---

### 2.10 `nexus/export/` -- Export

| Module | Responsabilite |
|--------|---------------|
| `report_generator.py` | Generation rapports via nexus 26B |
| `pdf_export.py` | Rendu HTML -> PDF (Jinja2 + WeasyPrint) |
| `timeline_export.py` | Export timeline (HTML standalone + PNG Plotly) |

---

### 2.11 `nexus/api/` -- Routers REST

| Fichier | Prefix | Tag | Nb endpoints |
|---------|--------|-----|-------------|
| `deps.py` | -- | -- | 0 (injection de dependances) |
| `cases.py` | /api/cases | cases | 6 |
| `evidence.py` | -- | evidence | 6 |
| `entities.py` | -- | entities | 3 |
| `hypotheses.py` | /api | hypotheses | 13 |
| `analysis.py` | -- | analysis | 3 |
| `graph.py` | /api | graph | 5 |
| `search.py` | /api | search | 3 |
| `monitoring.py` | -- | monitoring | 8 |
| `alerts.py` | -- | alerts | 3 |
| `reports.py` | -- | reports | 4 |
| `timeline.py` | /api | timeline | 2 |
| `geo.py` | /api | geo | 4 |
| `recon.py` | /api | recon | 5 |
| `image_search.py` | -- | image-search | 4 |
| `vision.py` | /api | vision | 5 |
| `forensics.py` | /api/forensics | forensics | 11 |
| `physics_sim_api.py` | /api/forensics/sim | physics-sim | 6 |
| `investigation.py` | -- | investigation | 5 |
| `audit.py` | -- | audit | 5 |

#### `deps.py` -- Injection de dependances

**Responsabilite :** Fournit les dependances request-scoped via `Depends()`. Pattern : Database connection par requete, singletons (LLMRouter, Neo4j, ChromaDB) depuis `app.state`.

**Dependances fournies :**

| Fonction | Retourne | Scope |
|----------|---------|-------|
| `get_database()` | `Database` | request (nouvelle connexion) |
| `get_audit_service(db)` | `AuditService` | request |
| `get_case_manager(db)` | `CaseManager` | request |
| `get_evidence_processor(request, db)` | `EvidenceProcessor` | request |
| `get_analysis_pipeline(request, db)` | `AnalysisPipeline` | request |
| `get_entity_extractor(request)` | `EntityExtractor` | shared (stateless) |
| `get_geo_mapper(db)` | `GeoMapper` | request |
| `get_image_analyzer(request, db)` | `ImageAnalyzer` | request |
| `get_neo4j(request)` | `Neo4jClient` | singleton (app.state) |
| `get_chroma(request)` | `ChromaClient` | singleton (app.state) |
| `get_llm_router(request)` | `LLMRouter` | singleton (app.state) |
| `get_hypothesis_engine(request, db)` | `HypothesisEngine` | request |
| `get_contradiction_detector(request, db)` | `ContradictionDetector` | request |
| `get_bpa_analyzer(request)` | `BloodPatternAnalyzer` | request |
| `get_acoustic_analyzer(request)` | `AcousticAnalyzer` | request |
| `get_trace_analyzer(request)` | `TraceAnalyzer` | request |

---

## 3. Base de donnees

### 3.1 SQLite -- 13 tables

Le schema SQLite est le stockage principal. Mode WAL active, foreign keys ON.

```sql
-- Configuration
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;
```

#### Schema complet

```
cases
  id              TEXT PRIMARY KEY
  name            TEXT NOT NULL
  reference       TEXT
  description     TEXT
  status          TEXT DEFAULT 'active'        -- active | closed | archived
  created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
  updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP

evidence
  id              TEXT PRIMARY KEY
  case_id         TEXT NOT NULL REFERENCES cases(id)
  title           TEXT NOT NULL
  evidence_type   TEXT NOT NULL                -- pdf | image | text | audio | url | manual
  source          TEXT
  source_date     DATETIME
  ingestion_date  DATETIME DEFAULT CURRENT_TIMESTAMP
  reliability     INTEGER DEFAULT 50           -- 0-100
  file_path       TEXT
  raw_text        TEXT
  summary         TEXT
  metadata        TEXT                         -- JSON serialise
  status          TEXT DEFAULT 'pending'       -- pending | processed | error
  created_at      DATETIME DEFAULT CURRENT_TIMESTAMP

entities
  id              TEXT PRIMARY KEY
  case_id         TEXT NOT NULL REFERENCES cases(id)
  name            TEXT NOT NULL
  entity_type     TEXT NOT NULL                -- person | location | phone | vehicle |
                                               -- organization | date | money | ip |
                                               -- email | account | weapon | drug | other
  aliases         TEXT                         -- JSON list serialise
  description     TEXT
  first_seen      DATETIME
  metadata        TEXT                         -- JSON serialise
  created_at      DATETIME DEFAULT CURRENT_TIMESTAMP

entity_mentions
  id              TEXT PRIMARY KEY
  entity_id       TEXT NOT NULL REFERENCES entities(id)
  evidence_id     TEXT NOT NULL REFERENCES evidence(id)
  context         TEXT
  confidence      REAL DEFAULT 0.8             -- 0.0-1.0
  created_at      DATETIME DEFAULT CURRENT_TIMESTAMP

hypotheses
  id              TEXT PRIMARY KEY
  case_id         TEXT NOT NULL REFERENCES cases(id)
  title           TEXT NOT NULL
  description     TEXT NOT NULL
  status          TEXT DEFAULT 'active'        -- active | refuted | confirmed | merged
  current_score   REAL DEFAULT 50.0            -- 0.0-100.0
  created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
  updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP

hypothesis_snapshots
  id              TEXT PRIMARY KEY
  hypothesis_id   TEXT NOT NULL REFERENCES hypotheses(id)
  score           REAL NOT NULL                -- 0.0-100.0
  supporting      TEXT                         -- JSON serialise
  contradicting   TEXT                         -- JSON serialise
  reasoning       TEXT
  trigger         TEXT
  model_used      TEXT
  created_at      DATETIME DEFAULT CURRENT_TIMESTAMP

analysis_runs
  id              TEXT PRIMARY KEY
  case_id         TEXT NOT NULL REFERENCES cases(id)
  run_type        TEXT NOT NULL                -- full | incremental | verification |
                                               -- extraction | self_questioning
  trigger         TEXT                         -- manual | new_evidence | monitoring |
                                               -- scheduled | autonomous_loop
  status          TEXT DEFAULT 'running'       -- running | completed | failed
  model_used      TEXT
  input_summary   TEXT
  output_summary  TEXT
  duration_sec    REAL
  tokens_used     INTEGER
  started_at      DATETIME DEFAULT CURRENT_TIMESTAMP
  completed_at    DATETIME

monitoring_jobs
  id              TEXT PRIMARY KEY
  case_id         TEXT NOT NULL REFERENCES cases(id)
  job_type        TEXT NOT NULL                -- searxng | robin | both
  query           TEXT NOT NULL
  entity_id       TEXT REFERENCES entities(id)
  interval_hours  INTEGER DEFAULT 24
  is_active       BOOLEAN DEFAULT 1
  last_run        DATETIME
  next_run        DATETIME
  results_count   INTEGER DEFAULT 0
  created_at      DATETIME DEFAULT CURRENT_TIMESTAMP

monitoring_results
  id              TEXT PRIMARY KEY
  job_id          TEXT NOT NULL REFERENCES monitoring_jobs(id)
  case_id         TEXT NOT NULL REFERENCES cases(id)
  url             TEXT
  title           TEXT
  snippet         TEXT
  source_engine   TEXT
  relevance_score REAL                         -- 0.0-100.0
  is_new          BOOLEAN DEFAULT 1
  is_duplicate    BOOLEAN DEFAULT 0
  reviewed        BOOLEAN DEFAULT 0
  found_at        DATETIME DEFAULT CURRENT_TIMESTAMP

alerts
  id              TEXT PRIMARY KEY
  case_id         TEXT NOT NULL REFERENCES cases(id)
  alert_type      TEXT NOT NULL                -- new_evidence | score_shift |
                                               -- monitoring_hit | contradiction | new_entity
  severity        TEXT DEFAULT 'info'          -- info | warning | critical
  title           TEXT NOT NULL
  message         TEXT NOT NULL
  related_id      TEXT
  is_read         BOOLEAN DEFAULT 0
  created_at      DATETIME DEFAULT CURRENT_TIMESTAMP

reports
  id              TEXT PRIMARY KEY
  case_id         TEXT NOT NULL REFERENCES cases(id)
  report_type     TEXT NOT NULL                -- full | summary | timeline
  status          TEXT DEFAULT 'generating'    -- generating | completed | error
  file_path       TEXT
  file_size       INTEGER
  created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
  completed_at    DATETIME

locations
  id              TEXT PRIMARY KEY
  case_id         TEXT NOT NULL REFERENCES cases(id)
  entity_id       TEXT REFERENCES entities(id)
  name            TEXT NOT NULL
  address         TEXT
  lat             REAL
  lon             REAL
  location_type   TEXT DEFAULT 'other'
  metadata        TEXT                         -- JSON serialise
  created_at      DATETIME DEFAULT CURRENT_TIMESTAMP

audit_log
  id              TEXT PRIMARY KEY
  case_id         TEXT NOT NULL REFERENCES cases(id)
  timestamp       DATETIME DEFAULT CURRENT_TIMESTAMP
  actor           TEXT NOT NULL                -- system | user | autonomous_loop | monitoring
  action          TEXT NOT NULL                -- evidence_added | hypothesis_scored | ...
  target_type     TEXT
  target_id       TEXT
  summary         TEXT NOT NULL
  details         TEXT                         -- JSON serialise
  cycle_number    INTEGER
  entry_hash      TEXT                         -- SHA-256 chain
  previous_hash   TEXT                         -- Hash de l'entree precedente
```

#### Indexes

```sql
idx_evidence_case      ON evidence(case_id)
idx_entities_case      ON entities(case_id)
idx_hypotheses_case    ON hypotheses(case_id)
idx_snapshots_hyp      ON hypothesis_snapshots(hypothesis_id)
idx_monitoring_case    ON monitoring_jobs(case_id)
idx_alerts_case_read   ON alerts(case_id, is_read)
idx_analysis_case      ON analysis_runs(case_id)
idx_reports_case       ON reports(case_id)
idx_locations_case     ON locations(case_id)
idx_locations_entity   ON locations(entity_id)
idx_audit_case         ON audit_log(case_id)
idx_audit_timestamp    ON audit_log(timestamp)
idx_audit_action       ON audit_log(action)
```

### 3.2 Neo4j -- Graphe de connaissances

#### Labels de noeuds (11)

| Label | Source SQLite | Description |
|-------|-------------|-------------|
| Person | entity_type=person | Personne impliquee |
| Location | entity_type=location | Lieu |
| Phone | entity_type=phone | Numero de telephone |
| Vehicle | entity_type=vehicle | Vehicule |
| Organization | entity_type=organization | Organisation |
| Account | entity_type=account/ip/email | Compte en ligne, email, IP |
| Event | entity_type=date | Evenement date |
| Evidence | entity_type=weapon/drug/other + evidence table | Preuve |
| Money | entity_type=money | Montant financier |
| Hypothesis | hypotheses table | Hypothese d'investigation |
| Case | cases table | Dossier d'investigation |

#### Types de relations (22)

| Relation | De -> Vers | Description |
|----------|-----------|-------------|
| KNOWS | Person -> Person | Se connaissent |
| RELATED_TO | Person -> Person | Lies (famille, etc.) |
| COMMUNICATED_WITH | Person -> Person | Communication |
| FINANCIAL_LINK | Person -> Person | Lien financier |
| SENT_MONEY | Person -> Person | Envoi d'argent |
| RECEIVED_MONEY | Person -> Person | Reception d'argent |
| LIVES_AT | Person -> Location | Residence |
| WAS_AT | Person -> Location | Presence a un lieu |
| WORKS_AT | Person -> Location | Lieu de travail |
| FREQUENTS | Person -> Location | Frequentation |
| OWNS | Person -> Vehicle/Phone/Account | Possession |
| MEMBER_OF | Person -> Organization | Appartenance |
| OCCURRED_AT | Event -> Location | Lieu d'evenement |
| INVOLVES | Event -> * | Implication |
| PRECEDED_BY | Event -> Event | Chronologie |
| MENTIONS | Evidence -> * | Mention dans preuve |
| SUPPORTS | Evidence -> Hypothesis | Preuve en faveur |
| CONTRADICTS | Evidence -> Hypothesis | Preuve contradictoire |
| TRANSACTION | Account -> Account | Transaction |
| BELONGS_TO | * -> Case | Appartenance a un dossier |

#### Contraintes

```cypher
CREATE CONSTRAINT IF NOT EXISTS FOR (n:Person) REQUIRE n.id IS UNIQUE
CREATE CONSTRAINT IF NOT EXISTS FOR (n:Location) REQUIRE n.id IS UNIQUE
-- ... pour chacun des 11 labels
```

### 3.3 ChromaDB -- Stockage vectoriel

#### Collections principales (4)

| Collection | Dimensions | Espace | Usage |
|-----------|-----------|--------|-------|
| `evidence_texts` | 768 (nomic-embed-text) | cosine | Recherche semantique preuves |
| `entity_contexts` | 768 (nomic-embed-text) | cosine | Recherche semantique entites |
| `monitoring_results` | 768 (nomic-embed-text) | cosine | Deduplication monitoring |
| `hypothesis_reasoning` | 768 (nomic-embed-text) | cosine | Recherche semantique hypotheses |

#### Collections vision (2, dans image_search.py)

| Collection | Dimensions | Espace | Usage |
|-----------|-----------|--------|-------|
| `image_dinov2` | 768 (DINOv2-base) | cosine | Image-to-image similarity |
| `image_clip` | 512 (CLIP-ViT-B/32) | cosine | Text-to-image search |

**Total : 6 collections ChromaDB**

**Configuration commune :**
- `embedding_function=None` (embeddings pre-calcules)
- `metadata={"hnsw:space": "cosine"}`
- Filtrage par `case_id` dans les metadatas

---

## 4. API REST

### Endpoint complet : 102 endpoints (101 routers + 1 health)

#### Systeme

| Methode | Path | Description |
|---------|------|-------------|
| GET | `/api/health` | Liveness probe |

#### Cases (6)

| Methode | Path | Description |
|---------|------|-------------|
| POST | `/api/cases` | Creer un dossier |
| GET | `/api/cases` | Lister les dossiers |
| GET | `/api/cases/{case_id}` | Detail d'un dossier |
| PUT | `/api/cases/{case_id}` | Modifier un dossier |
| DELETE | `/api/cases/{case_id}` | Supprimer (cascade) |
| GET | `/api/cases/{case_id}/stats` | Statistiques agregees |

#### Evidence (6)

| Methode | Path | Description |
|---------|------|-------------|
| POST | `/api/cases/{case_id}/evidence` | Upload fichier (multipart) |
| POST | `/api/cases/{case_id}/evidence/text` | Soumettre texte |
| GET | `/api/cases/{case_id}/evidence` | Lister preuves |
| GET | `/api/evidence/{evidence_id}` | Detail preuve |
| PUT | `/api/evidence/{evidence_id}` | Modifier preuve |
| DELETE | `/api/evidence/{evidence_id}` | Supprimer preuve |

#### Entities (3)

| Methode | Path | Description |
|---------|------|-------------|
| GET | `/api/cases/{case_id}/entities` | Lister entites |
| GET | `/api/entities/{entity_id}` | Detail entite |
| GET | `/api/entities/{entity_id}/mentions` | Mentions dans les preuves |

#### Hypotheses (13)

| Methode | Path | Description |
|---------|------|-------------|
| POST | `/api/cases/{case_id}/hypotheses` | Creer hypothese manuellement |
| GET | `/api/cases/{case_id}/hypotheses` | Lister hypotheses |
| POST | `/api/cases/{case_id}/hypotheses/generate` | Generer via LLM |
| POST | `/api/cases/{case_id}/hypotheses/merge` | Fusionner 2 hypotheses |
| POST | `/api/cases/{case_id}/evaluate-all` | Evaluer toutes les hypotheses |
| GET | `/api/cases/{case_id}/contradictions` | Detecter contradictions |
| POST | `/api/cases/{case_id}/compare-testimonies` | Comparer temoignages |
| GET | `/api/hypotheses/{hyp_id}` | Detail hypothese |
| PUT | `/api/hypotheses/{hyp_id}` | Modifier hypothese |
| DELETE | `/api/hypotheses/{hyp_id}` | Supprimer hypothese |
| POST | `/api/hypotheses/{hyp_id}/evaluate` | Evaluer une hypothese |
| GET | `/api/hypotheses/{hyp_id}/snapshots` | Historique snapshots |
| GET | `/api/hypotheses/{hyp_id}/evolution` | Time-series du score |

#### Analysis (3)

| Methode | Path | Description |
|---------|------|-------------|
| POST | `/api/cases/{case_id}/analyze` | Lancer analyse (202 Accepted, background) |
| GET | `/api/analysis/{run_id}` | Status d'un run |
| GET | `/api/cases/{case_id}/analysis-runs` | Historique des runs |

#### Graph (5)

| Methode | Path | Description |
|---------|------|-------------|
| GET | `/api/cases/{case_id}/graph` | Graphe complet |
| GET | `/api/cases/{case_id}/graph/neighbors/{node_id}` | Sous-graphe voisinage |
| GET | `/api/cases/{case_id}/graph/path/{from_id}/{to_id}` | Plus court chemin |
| GET | `/api/cases/{case_id}/graph/clusters` | Composantes connexes |
| GET | `/api/cases/{case_id}/graph/stats` | Stats par label |

#### Search (3)

| Methode | Path | Description |
|---------|------|-------------|
| POST | `/api/cases/{case_id}/search` | Recherche semantique (evidence ou entities) |
| GET | `/api/cases/{case_id}/similar/{evidence_id}` | Preuves similaires |
| GET | `/api/cases/{case_id}/duplicates` | Paires quasi-duplicats |

#### Monitoring (8)

| Methode | Path | Description |
|---------|------|-------------|
| POST | `/api/cases/{case_id}/monitoring` | Creer job monitoring |
| GET | `/api/cases/{case_id}/monitoring` | Lister jobs |
| PUT | `/api/monitoring/{job_id}` | Modifier job |
| DELETE | `/api/monitoring/{job_id}` | Supprimer job |
| POST | `/api/monitoring/{job_id}/run` | Execution immediate (202) |
| GET | `/api/cases/{case_id}/monitoring/results` | Resultats monitoring |
| GET | `/api/monitoring/results/{result_id}` | Detail resultat |
| POST | `/api/monitoring/results/{result_id}/ingest` | Ingerer comme preuve |

#### Alerts (3)

| Methode | Path | Description |
|---------|------|-------------|
| GET | `/api/cases/{case_id}/alerts` | Lister alertes |
| PUT | `/api/alerts/{alert_id}/read` | Marquer comme lu |
| GET | `/api/alerts/unread-count` | Nombre d'alertes non lues |

#### Reports (4)

| Methode | Path | Description |
|---------|------|-------------|
| POST | `/api/cases/{case_id}/reports/generate` | Generer rapport (background) |
| GET | `/api/cases/{case_id}/reports` | Lister rapports |
| GET | `/api/reports/{report_id}` | Detail rapport |
| GET | `/api/reports/{report_id}/download` | Telecharger PDF |

#### Timeline (2)

| Methode | Path | Description |
|---------|------|-------------|
| GET | `/api/cases/{case_id}/timeline` | Timeline complete |
| GET | `/api/cases/{case_id}/timeline/range` | Timeline filtree par dates |

#### Geo (4)

| Methode | Path | Description |
|---------|------|-------------|
| POST | `/api/cases/{case_id}/geocode` | Geocoder une entite location |
| GET | `/api/cases/{case_id}/map` | Donnees carte (toutes locations) |
| POST | `/api/cases/{case_id}/route` | Calcul itineraire entre 2 points |
| POST | `/api/cases/{case_id}/verify-travel` | Verification temps de trajet |

#### Recon (5)

| Methode | Path | Description |
|---------|------|-------------|
| POST | `/api/recon/email/{email}` | Check email (holehe + social) |
| POST | `/api/recon/username/{username}` | Check username multi-platforme |
| POST | `/api/recon/domain/{domain}` | WHOIS + DNS |
| GET | `/api/cases/{case_id}/recon` | Historique recon du case |
| POST | `/api/cases/{case_id}/recon/auto` | Scan automatique toutes entites |

#### Image Search (4)

| Methode | Path | Description |
|---------|------|-------------|
| POST | `/api/cases/{case_id}/images/index` | Indexer image (DINOv2 + CLIP) |
| POST | `/api/cases/{case_id}/images/search-by-text` | Recherche text-to-image (CLIP) |
| POST | `/api/cases/{case_id}/images/search-by-image` | Recherche image-to-image (DINOv2) |
| GET | `/api/cases/{case_id}/images/similar/{evidence_id}` | Images similaires |

#### Vision (5)

| Methode | Path | Description |
|---------|------|-------------|
| POST | `/api/evidence/{evidence_id}/analyze-image` | Analyser image preuve |
| POST | `/api/cases/{case_id}/analyze-images` | Analyser toutes images du case |
| POST | `/api/vision/describe` | Description libre d'image |
| POST | `/api/vision/compare` | Comparer deux images |
| GET | `/api/cases/{case_id}/visual-entities` | Entites visuelles extraites |

#### Forensics (11)

| Methode | Path | Description |
|---------|------|-------------|
| POST | `/api/forensics/bpa/classify` | Classifier pattern sang (VLM) |
| POST | `/api/forensics/bpa/analyze` | Analyse complete BPA |
| POST | `/api/forensics/bpa/calculate-angle` | Calcul angle d'impact |
| POST | `/api/forensics/bpa/convergence` | Zone de convergence |
| POST | `/api/forensics/audio/transcribe` | Transcription audio |
| POST | `/api/forensics/audio/analyze` | Analyse forensique audio |
| POST | `/api/forensics/audio/events` | Detection evenements sonores |
| POST | `/api/forensics/audio/propagation` | Calcul propagation son |
| POST | `/api/forensics/trace/analyze` | Analyser trace physique |
| POST | `/api/forensics/trace/compare` | Comparer deux traces |
| POST | `/api/forensics/cases/{case_id}/auto` | Auto-analyse forensique |

#### Physics Sim (6)

| Methode | Path | Description |
|---------|------|-------------|
| POST | `/api/forensics/sim/blood-drop` | Simulation goutte sang |
| POST | `/api/forensics/sim/cast-off` | Simulation cast-off |
| POST | `/api/forensics/sim/sound` | Simulation propagation son |
| POST | `/api/forensics/sim/origin` | Calcul point d'origine |
| GET | `/api/forensics/sim/datasets` | Lister datasets TheWell |
| GET | `/api/forensics/sim/datasets/{name}` | Detail dataset |

#### Investigation (5)

| Methode | Path | Description |
|---------|------|-------------|
| GET | `/api/investigations` | Status toutes investigations |
| POST | `/api/cases/{case_id}/investigation/start` | Demarrer boucle autonome |
| POST | `/api/cases/{case_id}/investigation/stop` | Arreter boucle autonome |
| GET | `/api/cases/{case_id}/investigation/status` | Status investigation |
| GET | `/api/cases/{case_id}/investigation/log` | Journal investigation |

#### Audit (5)

| Methode | Path | Description |
|---------|------|-------------|
| GET | `/api/cases/{case_id}/audit` | Journal d'audit |
| GET | `/api/cases/{case_id}/audit/summary` | Resume d'audit |
| GET | `/api/cases/{case_id}/audit/timeline` | Timeline d'audit |
| GET | `/api/audit/{audit_id}` | Detail entree audit |
| GET | `/api/cases/{case_id}/audit/verify` | Verification chaine de hash |

---

## 5. Boucle autonome OODA

### Architecture

La boucle autonome est le coeur de NEXUS. Pour chaque case actif, un `AutonomousInvestigator` tourne en background (asyncio.Task) et execute le cycle OODA etendu toutes les 30 minutes.

### Diagramme du cycle

```
    +------------------------------------------------------------------+
    |                    OODA + QUESTION Loop                          |
    |                    (30 min/cycle)                                |
    |                                                                  |
    |  +----------+    +---------+    +--------+    +-----+    +-----+ |
    |  | OBSERVE  |--->| ORIENT  |--->| DECIDE |--->| ACT |--->|QUEST| |
    |  | What's   |    | Ingest, |    |Analyze,|    |New  |    |Self-| |
    |  | new?     |    | recon,  |    |evaluate|    |quer-|    |crit-| |
    |  |          |    | geocode,|    |hypoth.,|    |ies, |    |ique,| |
    |  |          |    | images  |    |contra. |    |OSINT|    |rpts | |
    |  +----------+    +---------+    +--------+    +-----+    +-----+ |
    |       ^                                                     |    |
    |       |                    SLEEP (30 min)                   |    |
    |       +-----------------------------------------------------+    |
    +------------------------------------------------------------------+
```

### Detail de chaque phase

#### Phase 1 : OBSERVE -- Qu'y a-t-il de nouveau ?

**Actions :**
- Charge les `monitoring_results` non-revus pour ce case
- Filtre : garde uniquement ceux avec `relevance_score >= auto_ingest_relevance_threshold` (defaut: 50%)
- Log chaque resultat observe dans l'audit

**Modules :** Database (monitoring_results)

#### Phase 2 : ORIENT -- Ingerer et enrichir

**Actions :**
1. **Auto-ingest** -- Convertit les resultats monitoring pertinents en evidence (max `max_auto_ingest_per_cycle`, defaut: 5)
2. **OSINT recon** (si `auto_osint_recon=True`) -- Pour chaque entite email/account non-scannee, lance holehe + social_recon
3. **Geocoding** (si `auto_geocode=True`) -- Geocode les entites location sans coordonnees
4. **Image analysis** (si `auto_image_analysis=True`) -- Analyse VLM des preuves images non-analysees
5. **Visual embeddings** (si `auto_visual_embeddings=True`) -- Indexe les images dans DINOv2/CLIP

**Modules :** EvidenceProcessor, HoleheRecon, SocialRecon, GeoMapper, ImageAnalyzer, VisualEmbedder

#### Phase 3 : DECIDE -- Analyser et evaluer

**Actions :**
1. **Analyse incrementale** -- Re-analyse les preuves (si nouvelles)
2. **Re-evaluation hypotheses** -- Re-score toutes les hypotheses actives
3. **Detection contradictions** -- Compare paires de preuves
4. **Analyse forensique** (si `auto_forensic_analysis=True`) -- BPA, traces, acoustique
5. **Reconstruction timeline** (si `auto_timeline_rebuild=True`)

**Full re-evaluation :** Toutes les `full_reevaluation_every_n_cycles` (defaut: 6 = 3h)

**Modules :** AnalysisPipeline, HypothesisEngine, ContradictionDetector, BloodPatternAnalyzer, TraceAnalyzer, AcousticAnalyzer, TimelineBuilder, AlertManager

#### Phase 4 : ACT -- Agir sur les resultats

**Actions :**
1. **Generer nouvelles requetes** -- Le LLM genere `max_new_queries_per_cycle` (defaut: 3) nouvelles requetes de recherche basees sur les hypotheses et preuves
2. **Domain recon** (si `auto_domain_recon=True`) -- WHOIS/DNS sur les domaines d'email

**Modules :** LLMRouter (query generation), DomainRecon

#### Phase 5 : QUESTION -- Auto-critique

**Actions :**
1. **Self-questioning** -- Challenge l'hypothese principale via LLM (pensee adversariale)
2. **Rapport periodique** (toutes les `auto_report_every_n_cycles`, defaut: 12 = 6h)
3. **Backup automatique** (toutes les `auto_backup_every_n_cycles`, defaut: 24 = 12h)

**Modules :** LLMRouter, ReportGenerator, BackupManager, AuditService

### Configuration (settings)

| Parametre | Defaut | Description |
|-----------|--------|-------------|
| `investigation_cycle_minutes` | 30 | Duree d'un cycle complet |
| `auto_ingest_relevance_threshold` | 50.0 | Seuil de pertinence pour auto-ingestion |
| `full_reevaluation_every_n_cycles` | 6 | Re-evaluation complete toutes les N cycles |
| `max_auto_ingest_per_cycle` | 5 | Max preuves auto-ingerees par cycle |
| `max_new_queries_per_cycle` | 3 | Max nouvelles requetes par cycle |
| `auto_osint_recon` | true | Active le scan OSINT automatique |
| `auto_geocode` | true | Active le geocoding automatique |
| `auto_image_analysis` | true | Active l'analyse VLM automatique |
| `auto_forensic_analysis` | true | Active l'analyse forensique auto |
| `auto_visual_embeddings` | true | Active l'indexation visuelle auto |
| `auto_domain_recon` | true | Active le recon domaine auto |
| `auto_timeline_rebuild` | true | Reconstruit la timeline chaque DECIDE |
| `auto_report_every_n_cycles` | 12 | Rapport toutes les 6h |
| `auto_backup_every_n_cycles` | 24 | Backup toutes les 12h |
| `auto_recon_rate_limit` | 2.0 | Secondes entre appels OSINT |

---

## 6. Routage LLM

### Table de routage complete

| TaskType | Modele | Timeout | Heavy? | Usage |
|----------|--------|---------|--------|-------|
| **Taches legeres (gemma4:e4b, ~80 tok/s, 4B)** |
| ENTITY_EXTRACTION | model_fast | 30s | Non | Extraction entites depuis texte |
| QUERY_REFORMULATION | model_fast | 15s | Non | Reformulation requetes monitoring |
| RESULT_FILTERING | model_fast | 20s | Non | Scoring pertinence resultats |
| JSON_STRUCTURING | model_fast | 20s | Non | Formatage JSON |
| EVIDENCE_SUMMARY | model_fast | 30s | Non | Resume de preuves |
| **Embeddings (nomic-embed-text, 768-dim)** |
| EMBEDDING | model_embedding | 10s | Non | Embeddings vectoriels |
| **Raisonnement (deepseek-r1-abliterated:14b, CoT)** |
| LOGIC_VERIFICATION | model_reasoning | 120s | Oui | Verification logique |
| CONTRADICTION_DETECTION | model_reasoning | 120s | Oui | Detection contradictions |
| TESTIMONY_COMPARISON | model_reasoning | 120s | Oui | Comparaison temoignages |
| **Analyse profonde (nexus / Gemma 4 26B Heretic)** |
| DEEP_ANALYSIS | model_deep | 300s | Oui | Analyse complete dossier |
| HYPOTHESIS_SCORING | model_deep | 300s | Oui | Scoring hypotheses |
| FINAL_REPORT | model_deep | 600s | Oui | Generation rapport final |
| INCREMENTAL_REEVAL | model_deep | 300s | Oui | Re-evaluation incrementale |
| **Vision (gemma4:e4b fast / qwen3-vl:8b deep)** |
| IMAGE_DESCRIPTION | model_vision | 60s | Non | Description rapide |
| IMAGE_ENTITY_EXTRACTION | model_vision | 60s | Non | Extraction entites image |
| IMAGE_SCENE_ANALYSIS | model_vision_deep | 180s | Oui | Analyse scene approfondie |
| IMAGE_COMPARISON | model_vision_deep | 180s | Oui | Comparaison deux images |
| **Audio (voxtral-mini:4b)** |
| AUDIO_TRANSCRIPTION | model_audio | 180s | Oui | Transcription audio |
| **Forensics (qwen3-vl:8b)** |
| TRACE_ANALYSIS | model_vision_deep | 180s | Oui | Analyse traces physiques |

### Gestion VRAM (RTX 5080, 16 GB partagee)

**Probleme :** Un seul "gros" modele (nexus 26B, deepseek-r1 14B, qwen3-vl:8b) peut occuper la VRAM a la fois. Charger deux gros modeles cause un OOM ou un swap GPU incessant.

**Solution :** `asyncio.Lock` (`_heavy_lock`) dans `LLMRouter` qui serialise les appels aux modeles marques `heavy=True`. Les modeles legers (gemma4:e4b, nomic-embed-text) coexistent librement.

```python
if heavy:
    async with self._heavy_lock:
        return await self.client.generate(...)
return await self.client.generate(...)
```

**Parametres Ollama :**
- `options={"num_ctx": 8192}` -- Fenetre de contexte
- `keep_alive="10m"` -- Garde le modele en VRAM 10 minutes apres le dernier appel
- `format="json"` -- Force la sortie JSON (pour generate_json)

### Timeouts par tache

| Categorie | Timeout | Justification |
|-----------|---------|--------------|
| Taches legeres | 10-30s | gemma4:e4b est rapide (~80 tok/s) |
| Raisonnement | 120s | deepseek-r1 peut generer de longues chaines CoT |
| Analyse profonde | 300s | nexus 26B sur dossiers complexes |
| Rapport final | 600s | Rapport complet multi-sections |
| Vision | 60-180s | Selon complexite de l'image |
| Audio | 180s | Transcription peut etre longue |

---

## 7. Audit trail

### Architecture 3 couches

L'audit trail est concu pour la non-repudiation et la detection de falsification. Chaque action est enregistree dans 3 couches independantes.

```
Action ──> Couche 1: SQLite (rapide, requetable)
       ──> Couche 2: JSONL append-only (humain-lisible)
       ──> Couche 3: Git commit (cryptographique, distributable)
```

#### Couche 1 : SQLite (`audit_log` table)

- Stockage dans la table `audit_log` avec `entry_hash` et `previous_hash`
- Hash chain : `SHA-256(previous_hash | entry_data)` forme une chaine comme une blockchain
- Si une entree est modifiee ou supprimee, la chaine casse et la falsification est detectee
- Requetable par case_id, action, timestamp

#### Couche 2 : Fichiers JSONL append-only

- Un fichier par case : `data/audit/{case_id}.jsonl`
- Chaque ligne est un objet JSON complet avec hash
- Ecriture en mode append uniquement -- jamais d'ecrasement
- Lisible avec n'importe quel editeur de texte

#### Couche 3 : Git

- Repo git initialise dans `data/audit/`
- Chaque entree d'audit genere un `git add` + `git commit` automatique
- Message de commit : `[{action}] {summary} (hash:{entry_hash[:12]})`
- Historique complet avec hashes cryptographiques git

### Hash chain

```
GENESIS -> hash_1 = SHA-256("GENESIS | entry_1_data")
        -> hash_2 = SHA-256("hash_1 | entry_2_data")
        -> hash_3 = SHA-256("hash_2 | entry_3_data")
        -> ...
```

**entry_data format :** `{timestamp}|{case_id}|{actor}|{action}|{summary}|{target_type}|{target_id}|{details_json}`

### Verification d'integrite

Endpoint : `GET /api/cases/{case_id}/audit/verify`

**Algorithme :**
1. Charger toutes les entrees d'audit du case (triees par timestamp)
2. Verifier que chaque `previous_hash` correspond au `entry_hash` de l'entree precedente
3. Recalculer le hash attendu et comparer avec le hash stocke
4. Retourner `{valid: bool, entries_checked: int, broken_at: int | null}`

### Actions auditees

| Action | Acteur | Description |
|--------|--------|-------------|
| `evidence_added` | user/system | Preuve ajoutee |
| `hypothesis_scored` | system | Score hypothese change |
| `entity_discovered` | system | Nouvelle entite trouvee |
| `contradiction_found` | system | Contradiction detectee |
| `monitoring_result` | monitoring | Resultat monitoring |
| `query_generated` | autonomous_loop | Nouvelle requete generee |
| `analysis_started` | system | Analyse demarree |
| `analysis_completed` | system | Analyse terminee |
| `alert_created` | system | Alerte creee |
| `hypothesis_created` | system | Hypothese creee |
| `hypothesis_refuted` | system | Hypothese refutee |
| `hypothesis_confirmed` | system | Hypothese confirmee |
| `evidence_ingested_auto` | autonomous_loop | Auto-ingestion |
| `self_questioning` | autonomous_loop | Auto-questionnement |
| `investigation_started` | system | Boucle demarree |
| `investigation_stopped` | system | Boucle arretee |
| `case_created` | user | Case cree |
| `case_updated` | user | Case modifie |

### Proprietes

- **Non-bloquant :** Les echecs d'audit sont captures et logues mais ne perturbent jamais l'operation principale
- **Asynchrone :** Les couches 2 et 3 sont executees en `asyncio.create_task()` (fire-and-forget)
- **Immutable :** Append-only files, hash chain, git commits

---

## 8. Frontend

### Architecture

Le frontend est un dashboard Streamlit multi-pages avec :
- 1 page d'accueil (`app.py`) -- selecteur de case, creation rapide
- 15 pages fonctionnelles (`frontend/pages/01_*.py` a `15_*.py`)
- 4 composants reutilisables (`frontend/components/`)
- 1 client API synchrone (`frontend/api_client.py`)

### Client API (`api_client.py`)

Wrapper synchrone autour de `requests.Session()` mappe 1-to-1 sur les endpoints REST. Singleton module-level `api`. Timeout : 120s. Gere les erreurs de connexion avec affichage Streamlit.

### Pages

| # | Fichier | Nom | Description |
|---|---------|-----|-------------|
| -- | `app.py` | Accueil | Selecteur de case, creation, health check |
| 01 | `01_dashboard.py` | Tableau de bord | Metriques cles, alertes recentes, derniere analyse, hypothese principale |
| 02 | `02_evidence.py` | Preuves | Upload fichiers, soumission texte, navigation, details |
| 03 | `03_entities.py` | Entites | Navigation par type, details, mentions croisees |
| 04 | `04_hypotheses.py` | Hypotheses | Graphique evolution scores, liste, actions, snapshots |
| 05 | `05_timeline.py` | Chronologie | Scatter plot Plotly interactif, filtrage par type |
| 06 | `06_graph.py` | Graphe | Neo4j interactif (streamlit-agraph), filtrage, chemin, clusters |
| 07 | `07_monitoring.py` | Monitoring | CRUD jobs, resultats, ingestion manuelle |
| 08 | `08_alerts.py` | Alertes | Liste filtrable par severite, badge non-lu, mark-as-read |
| 09 | `09_analysis.py` | Analyse | Declenchement analyse, historique runs |
| 10 | `10_map.py` | Carte | Carte Folium interactive, geocoding, routes, verification trajet |
| 11 | `11_osint.py` | OSINT | Recon email/username/domaine, scan auto entites |
| 12 | `12_vision.py` | Vision | Upload + analyse VLM, comparaison images, entites visuelles |
| 13 | `13_forensics.py` | Forensique | 5 onglets : BPA, acoustique, traces, auto-analyse, simulations |
| 14 | `14_investigation.py` | Investigation | Centre de commande boucle autonome, OODA temps reel, journal |
| 15 | `15_audit.py` | Audit | Journal d'audit chronologique, filtrable, verification integrite |

### Composants reutilisables

| Composant | Fichier | Description |
|-----------|---------|-------------|
| Evidence Card | `evidence_card.py` | Carte preuve stylee avec badge type, barre fiabilite, resume |
| Graph Viewer | `graph_viewer.py` | Conversion API -> streamlit-agraph avec coloration par type |
| Hypothesis Chart | `hypothesis_chart.py` | Graphiques Plotly evolution scores avec zones de confiance |
| Physics Viz | `physics_viz.py` | Rendus Plotly 3D/2D pour simulations forensiques |

### Composants Streamlit utilises

| Composant | Usage |
|-----------|-------|
| `st.set_page_config()` | Configuration globale (wide layout) |
| `st.tabs()` | Onglets dans les pages (forensics, investigation) |
| `st.metric()` | KPIs sur le dashboard |
| `st.file_uploader()` | Upload preuves (images, PDF, audio) |
| `st.form()` | Formulaires creation (cases, evidence, monitoring) |
| `st.selectbox()` | Selecteur case, filtres |
| `st.columns()` | Layouts multi-colonnes |
| `st.expander()` | Details expansibles |
| `st.dataframe()` | Tableaux de donnees |
| `st.plotly_chart()` | Graphiques Plotly (timeline, hypotheses, physics) |
| `st.session_state` | Persistance case_id, case_name |
| `st.rerun()` | Rafraichissement apres action |
| `st.markdown()` | Contenu riche avec HTML/CSS |
| `st.sidebar` | Navigation laterale |
| `streamlit_agraph()` | Visualisation graphe Neo4j |
| `streamlit_folium()` | Carte interactive |

---

## 9. Problemes connus et limitations

### 9.1 Pas de RAG -- tout en memoire

**Probleme :** Lorsqu'une analyse est lancee, toutes les preuves du case sont chargees en memoire et injectees dans le prompt du LLM. Il n'y a pas de retrieval semantique prealable depuis ChromaDB pour limiter le contexte.

**Impact :** Le prompt peut depasser la fenetre de contexte (8192 tokens) pour les cases ayant beaucoup de preuves. Les preuves sont tronquees ou les informations importantes sont perdues.

**Solution prevue :** Implementer un pipeline RAG : query -> ChromaDB retrieval -> top-K chunks -> injection dans le prompt.

### 9.2 GPU 16 GB partagee

**Probleme :** La RTX 5080 a 16 GB VRAM partagee entre tous les modeles. Les modeles lourds (nexus 26B, deepseek-r1 14B) ne peuvent pas coexister en VRAM.

**Attenuation actuelle :** Le `_heavy_lock` serialise les appels aux modeles lourds. Le parametre `keep_alive="10m"` garde un modele en VRAM 10 minutes pour eviter les rechargements frequents.

**Limitation residuelle :** Meme avec la serialisation, le swap GPU (decharger un modele de 26B pour charger un 14B) prend du temps. La pipeline d'analyse est forcement sequentielle.

### 9.3 Pas de tests unitaires

**Probleme :** Le repertoire `tests/` ne contient que des scripts de benchmark, aucun test unitaire. Aucune couverture de code, aucune regression detectee automatiquement.

**Impact :** Les modifications de code sont fragiles. Les bugs peuvent passer inapercus.

### 9.4 APScheduler in-process

**Probleme :** Le scheduler tourne dans le meme processus que FastAPI. Si le processus crash, tous les jobs sont perdus (pas de persistence de l'etat du scheduler).

**Attenuation :** Les jobs sont recrees au demarrage depuis SQLite. Les runs manques sont fusionnes (`coalesce=True`). Mais le timing exact des prochaines executions n'est pas persiste.

### 9.5 Robin sans API REST

**Probleme :** Robin n'expose pas d'API REST. L'integration passe par `docker exec` en subprocess, ce qui est fragile et lent.

**Impact :** Les recherches dark web sont plus lentes et peuvent echouer silencieusement si le conteneur Robin n'est pas disponible.

### 9.6 Deduplication O(n^2)

**Probleme :** `ChromaClient.find_duplicates()` compare toutes les paires d'evidence (O(n^2)). Pour un case avec beaucoup de preuves, cela devient tres lent.

**Solution :** Utiliser l'index HNSW de ChromaDB directement (query chaque element contre la collection) au lieu de charger tous les embeddings en memoire.

### 9.7 Pas de gestion d'authentification

**Probleme :** L'API FastAPI et le dashboard Streamlit n'ont aucune authentification. Tout le monde sur le reseau local peut acceder au systeme.

**Attenuation :** Le systeme est concu pour un usage mono-utilisateur sur machine locale.

### 9.8 SQLite en mono-process

**Probleme :** SQLite ne supporte qu'un seul ecrivain a la fois. Le mode WAL ameliore la concurrence lecture/ecriture mais reste limite.

**Attenuation :** Le systeme est mono-utilisateur. Les operations d'ecriture sont courtes. La boucle autonome ouvre et ferme ses connexions rapidement.

### 9.9 Geocoding sans cle API

**Probleme :** Le geocoding utilise Nominatim (OSM) qui impose un rate-limit de 1 req/sec et n'a pas de garantie de disponibilite.

---

## 10. Roadmap

### Priorite 1 : RAG (Retrieval-Augmented Generation)

**Objectif :** Remplacer le chargement complet des preuves par un retrieval semantique via ChromaDB.

**Impact :** Permet de gerer des cases avec des centaines de preuves sans depasser la fenetre de contexte. Ameliore la qualite des analyses en ne fournissant que le contexte pertinent.

**Implementation :**
1. Avant chaque appel LLM, formuler une query de recherche
2. Query ChromaDB (evidence_texts + entity_contexts) avec la query embedee
3. Recuperer les top-K chunks les plus pertinents
4. Injecter uniquement ces chunks dans le prompt
5. Conserver la capacite de "full context" pour les re-evaluations completes

### Priorite 2 : Docling (meilleur parsing PDF)

**Objectif :** Remplacer PyMuPDF par Docling pour un parsing PDF plus intelligent (OCR, tableaux, structures complexes).

**Impact :** Meilleure extraction de texte depuis les documents judiciaires, rapports de police, proces-verbaux scannees.

### Priorite 3 : Local Deep Research

**Objectif :** Integrer Local Deep Research pour des recherches web multi-etapes pilotees par LLM.

**Impact :** Au lieu de simples queries SearXNG, le systeme pourrait mener des recherches iteratives approfondies, suivre des liens, et synthetiser des resultats complexes.

**Reference :** `docs/LOCAL-DEEP-RESEARCH-ANALYSIS.md`

### Priorite 4 : LangGraph pour la boucle OODA

**Objectif :** Remplacer la boucle OODA codee en dur par un graphe d'agents LangGraph.

**Impact :** Plus de flexibilite pour ajouter/modifier des phases, meilleur controle du flux, possibilite de branches conditionnelles et de parallelisme intelligent, meilleure observabilite.

### Priorite 5 : OSINT-with-LLM

**Objectif :** Integrer le framework OSINT-with-LLM pour enrichir automatiquement les entites.

**Reference :** `docs/OSINT-with-LLM-research.md`

### Priorite 6 : Tests et CI/CD

**Objectif :** Ajouter des tests unitaires, d'integration, et une pipeline CI.

**Actions :**
1. Tests unitaires pour les modules core (hypothesis_engine, contradiction_detector, etc.)
2. Tests d'integration pour les API endpoints
3. Mocks pour les appels Ollama, Neo4j, ChromaDB
4. Pipeline GitHub Actions ou equivalent

---

## Annexe A : Arborescence du projet

```
nexus/
  __init__.py
  main.py                        # Point d'entree FastAPI
  config.py                      # Configuration centralisee
  api/
    __init__.py
    deps.py                      # Injection de dependances
    alerts.py                    # 3 endpoints
    analysis.py                  # 3 endpoints
    audit.py                     # 5 endpoints
    cases.py                     # 6 endpoints
    entities.py                  # 3 endpoints
    evidence.py                  # 6 endpoints
    forensics.py                 # 11 endpoints
    geo.py                       # 4 endpoints
    graph.py                     # 5 endpoints
    hypotheses.py                # 13 endpoints
    image_search.py              # 4 endpoints
    investigation.py             # 5 endpoints
    monitoring.py                # 8 endpoints
    physics_sim_api.py           # 6 endpoints
    recon.py                     # 5 endpoints
    reports.py                   # 4 endpoints
    search.py                    # 3 endpoints
    timeline.py                  # 2 endpoints
    vision.py                    # 5 endpoints
  core/
    analysis_pipeline.py         # Analyse multi-modeles
    audit.py                     # Audit trail 3 couches
    autonomous_loop.py           # Boucle OODA (cerveau)
    backup.py                    # Backups ZIP
    case_manager.py              # CRUD cases
    contradiction_detector.py    # Detection contradictions
    entity_extractor.py          # Extraction entites LLM
    evidence_processor.py        # Pipeline ingestion
    geo_mapper.py                # Geocoding + routing
    hypothesis_engine.py         # Moteur hypotheses
    image_analyzer.py            # Pipeline analyse visuelle
    investigation_manager.py     # Gestion investigators
    timeline_builder.py          # Construction timeline
  db/
    sqlite_db.py                 # 13 tables, CRUD complet
    neo4j_db.py                  # Graphe Neo4j async
    chroma_db.py                 # 4+2 collections vectorielles
    models.py                    # 30+ modeles Pydantic v2
  llm/
    ollama_client.py             # Client async Ollama SDK
    router.py                    # Routeur multi-modeles + VRAM lock
    prompts.py                   # Templates prompts (francais)
    parsers.py                   # Parsing JSON LLM
  monitoring/
    scheduler.py                 # APScheduler AsyncIO
    searxng_monitor.py           # Client SearXNG async
    robin_monitor.py             # Client Robin (docker exec)
    alert_manager.py             # Alertes typees
  forensics/
    blood_pattern.py             # BPA classification + calculs
    acoustic_analysis.py         # Analyse audio forensique
    trace_analyzer.py            # Traces physiques
    physics_sim.py               # Simulations (numpy/scipy)
    the_well_loader.py           # Datasets PolymathicAI
  recon/
    holehe_recon.py              # Email recon (120+ services)
    social_recon.py              # Username recon multi-plateforme
    domain_recon.py              # WHOIS + DNS
  vision/
    embeddings.py                # DINOv2 + CLIP
    image_search.py              # Image similarity search
  ingest/
    pdf_parser.py                # PyMuPDF (texte + images)
    text_parser.py               # Texte brut
  export/
    report_generator.py          # Generation rapports LLM
    pdf_export.py                # HTML -> PDF (WeasyPrint)
    timeline_export.py           # Timeline HTML/PNG

frontend/
  __init__.py
  _path_fix.py                   # sys.path fix
  app.py                         # Point d'entree Streamlit
  api_client.py                  # Client API sync (requests)
  components/
    evidence_card.py             # Carte preuve reutilisable
    graph_viewer.py              # Visualisation graphe Neo4j
    hypothesis_chart.py          # Charts Plotly evolution scores
    physics_viz.py               # Viz simulations forensiques
  pages/
    01_dashboard.py              # Tableau de bord
    02_evidence.py               # Gestion preuves
    03_entities.py               # Navigation entites
    04_hypotheses.py             # Hypotheses evolutives
    05_timeline.py               # Chronologie interactive
    06_graph.py                  # Graphe de connaissances
    07_monitoring.py             # Surveillance automatisee
    08_alerts.py                 # Alertes systeme
    09_analysis.py               # Analyses LLM
    10_map.py                    # Carte investigation
    11_osint.py                  # OSINT recon
    12_vision.py                 # Analyse visuelle
    13_forensics.py              # Analyse forensique
    14_investigation.py          # Centre commande autonome
    15_audit.py                  # Journal d'audit

data/                            # Donnees (gitignored)
  nexus.db                       # SQLite principal
  uploads/                       # Fichiers uploades
  reports/                       # Rapports generes
  backups/                       # Backups ZIP
  audit/                         # Fichiers JSONL + repo git
  neo4j/                         # Volume Neo4j Docker
  chroma/                        # Volume ChromaDB Docker
  robin/                         # Volume Robin Docker

docs/                            # Documentation
prompts/                         # 5 templates d'analyse
models/                          # Modelfiles Ollama
tests/                           # Benchmarks (pas de tests unitaires)
searxng/                         # Config SearXNG
```

## Annexe B : Lancement rapide

```bash
# 1. Services Docker
docker-compose up -d

# 2. Backend FastAPI
uvicorn nexus.main:app --host 0.0.0.0 --port 8000 --reload

# 3. Frontend Streamlit
streamlit run frontend/app.py --server.port 8501

# 4. Verifier
curl http://localhost:8000/api/health
# -> {"status": "ok", "version": "0.1.0", ...}
```

## Annexe C : Variables d'environnement

| Variable | Defaut | Description |
|----------|--------|-------------|
| `NEXUS_HOST` | 0.0.0.0 | Bind host FastAPI |
| `NEXUS_PORT` | 8000 | Port FastAPI |
| `NEXUS_DEBUG` | true | Mode debug |
| `OLLAMA_BASE_URL` | http://localhost:11434 | URL Ollama |
| `MODEL_FAST` | gemma4:e4b | Modele rapide |
| `MODEL_REASONING` | huihui_ai/deepseek-r1-abliterated:14b | Modele raisonnement |
| `MODEL_DEEP` | nexus | Modele analyse profonde |
| `MODEL_EMBEDDING` | nomic-embed-text | Modele embeddings |
| `MODEL_AUDIO` | voxtral-mini:4b | Modele audio |
| `MODEL_VISION` | gemma4:e4b | VLM rapide |
| `MODEL_VISION_DEEP` | qwen3-vl:8b | VLM approfondi |
| `NEO4J_URI` | bolt://localhost:7687 | URI Neo4j |
| `NEO4J_USER` | neo4j | Utilisateur Neo4j |
| `NEO4J_PASSWORD` | nexus2026 | Mot de passe Neo4j |
| `CHROMA_HOST` | localhost | Host ChromaDB |
| `CHROMA_PORT` | 8100 | Port ChromaDB |
| `SEARXNG_URL` | http://localhost:8888 | URL SearXNG |
| `ROBIN_URL` | http://localhost:9090 | URL Robin |
| `DATA_DIR` | ./data | Repertoire donnees |
| `UPLOAD_DIR` | ./data/uploads | Repertoire uploads |
| `SQLITE_PATH` | ./data/nexus.db | Chemin SQLite |
| `CLEARWEB_INTERVAL` | 21600 (6h) | Intervalle monitoring clearweb |
| `DARKWEB_INTERVAL` | 86400 (24h) | Intervalle monitoring dark web |

---

## Annexe D : Verification des APIs (Mission 1)

Verification croisee de chaque dependance principale contre la documentation officielle (avril 2026).

### FastAPI (>=0.115)

| Pattern utilise dans NEXUS | Correct ? | Notes |
|---------------------------|-----------|-------|
| asynccontextmanager + async def lifespan(app) + yield | OK | Pattern officiel recommande depuis FastAPI 0.93+ |
| FastAPI(lifespan=lifespan) | OK | Parametre lifespan standard |
| Depends(get_database) pour injection | OK | Pattern standard FastAPI DI |
| BackgroundTasks (dans analysis.py) | OK | Utilise pour analyses en background |
| CORSMiddleware avec allow_origins=["*"] | OK | Standard mais a restreindre en production |
| BaseHTTPMiddleware pour X-Process-Time | OK | Pattern valide |
| Request.app.state pour singletons | OK | Pattern officiel pour partager des objets |

**Verdict : Aucune incompatibilite detectee.**

### Ollama Python SDK (>=0.4)

| Pattern utilise dans NEXUS | Correct ? | Notes |
|---------------------------|-----------|-------|
| AsyncClient(host=base_url) | OK | Constructeur standard |
| client.generate(model, prompt, system, format, options, keep_alive) | OK | Tous les parametres sont acceptes |
| response.response pour le texte genere | OK | GenerateResponse.response: Optional[str] confirme |
| client.chat(model, messages, options, keep_alive) | OK | Pattern standard pour vision (avec images dans messages) |
| response.message.content pour le texte chat | OK | ChatResponse.message.content: Optional[str] confirme |
| client.embed(model, input=text) single input | OK | input accepte str ou list |
| client.embed(model, input=texts) batch input | OK | Batch natif confirme |
| response.embeddings[0] pour single embed | OK | EmbedResponse.embeddings: Sequence[Sequence[float]] |
| format="json" pour forcer JSON | OK | Accepte json ou JsonSchema |
| options={"num_ctx": 8192} | OK | Options Ollama standard |
| keep_alive="10m" | OK | Accepte str ou float |
| RequestError, ResponseError imports | OK | Exceptions standard du SDK |

**Verdict : Aucune incompatibilite detectee.**

### Neo4j Python Driver (>=5.28)

| Pattern utilise dans NEXUS | Correct ? | Notes |
|---------------------------|-----------|-------|
| AsyncGraphDatabase.driver(uri, auth=(user, pass)) | OK | Constructeur async standard |
| async with driver.session() as session | OK | Session async context manager |
| session.execute_read(tx_func) | OK | Managed read transaction |
| session.execute_write(tx_func) | OK | Managed write transaction |
| async def _work(tx: AsyncManagedTransaction) | OK | Type correct pour callback |
| await tx.run(query, **params) | OK | Requete parametree |
| result.single(), async for r in result | OK | Fetch patterns corrects |
| driver.verify_connectivity() | OK | Method async disponible |
| Labels via f-string (pas $param) | OK | Labels Neo4j non parametrables |

**Verdict : Aucune incompatibilite detectee.**

### ChromaDB (>=0.6)

| Pattern utilise dans NEXUS | Correct ? | Notes |
|---------------------------|-----------|-------|
| chromadb.HttpClient(host, port) | OK | Constructeur HTTP standard |
| get_or_create_collection(name, embedding_function=None, metadata) | OK | Pattern correct pour embeddings pre-calcules |
| metadata={"hnsw:space": "cosine"} | OK | Configuration HNSW valide |
| collection.add(ids, documents, embeddings, metadatas) | OK | Signature standard |
| collection.query(query_embeddings, n_results, where, include) | OK | Signature standard |
| collection.get(ids, where, include) | OK | Signature standard |
| collection.delete(ids) | OK | Signature standard |
| collection.count() | OK | Methode standard |

**Verdict : Aucune incompatibilite detectee.**

### APScheduler (>=3.11)

| Pattern utilise dans NEXUS | Correct ? | Notes |
|---------------------------|-----------|-------|
| AsyncIOScheduler(timezone, job_defaults) | OK | Constructeur standard v3.x |
| scheduler.add_job(func, trigger, id, args, name) | OK | Signature standard |
| IntervalTrigger(hours=N) | OK | Trigger intervalle |
| scheduler.remove_job(id), reschedule_job, modify_job | OK | Gestion standard |
| scheduler.start(), shutdown(wait=False) | OK | Lifecycle standard |
| coalesce, max_instances, misfire_grace_time | OK | Options standard |

**Note :** NEXUS utilise APScheduler 3.x (>=3.11,<4). Correct car v4 a une API differente.

**Verdict : Aucune incompatibilite detectee.**

### Pydantic v2 (>=2.11)

| Pattern utilise dans NEXUS | Correct ? | Notes |
|---------------------------|-----------|-------|
| BaseModel, Field(ge, le, default) | OK | Pattern standard v2 |
| model_config = {"from_attributes": True} | OK | Remplace orm_mode de v1 |
| Literal[...] pour enums typees | OK | Pattern standard |
| pydantic-settings BaseSettings | OK | Package separe correct |

**Verdict : Aucune incompatibilite detectee.**

### Streamlit (>=1.44)

| Pattern utilise dans NEXUS | Correct ? | Notes |
|---------------------------|-----------|-------|
| st.set_page_config (premier appel) | OK | Contrainte respectee |
| st.tabs, st.metric, st.file_uploader, st.form | OK | Widgets standard |
| st.session_state, st.rerun() | OK | Disponible depuis 1.27+ |
| streamlit-agraph (0.0.45), streamlit-folium (0.23) | OK | Packages tiers compatibles |

**Verdict : Aucune incompatibilite detectee.**

### aiosqlite (>=0.21)

| Pattern utilise dans NEXUS | Correct ? | Notes |
|---------------------------|-----------|-------|
| aiosqlite.connect, Row, execute, fetchall | OK | API standard |
| executescript, commit, rowcount | OK | Toutes les methodes existent |
| async with, dict(row) | OK | Patterns corrects |

**Verdict : Aucune incompatibilite detectee.**

### Resume global

| Dependance | Version | APIs correctes ? | Incompatibilites |
|-----------|---------|-----------------|-----------------|
| FastAPI | >=0.115 | Oui | Aucune |
| Ollama SDK | >=0.4 | Oui | Aucune |
| Neo4j Driver | >=5.28 | Oui | Aucune |
| ChromaDB | >=0.6 | Oui | Aucune |
| APScheduler | >=3.11 | Oui | Aucune |
| Pydantic v2 | >=2.11 | Oui | Aucune |
| Streamlit | >=1.44 | Oui | Aucune |
| aiosqlite | >=0.21 | Oui | Aucune |

**Conclusion :** Toutes les APIs sont utilisees correctement. Aucune incompatibilite detectee entre le code NEXUS et les APIs officielles des 8 dependances principales.
