# NEXUS -- Reference API Complete

> **Version:** 0.1.0
> **Base URL:** `http://localhost:8000`
> **Derniere mise a jour:** 2026-04-05

---

## Table des matieres

1. [Informations generales](#informations-generales)
2. [Modeles Pydantic](#modeles-pydantic)
3. [Dependances FastAPI](#dependances-fastapi)
4. [Endpoints par domaine](#endpoints-par-domaine)
   - [System](#system)
   - [Cases](#cases)
   - [Evidence](#evidence)
   - [Entities](#entities)
   - [Analysis](#analysis)
   - [Hypotheses](#hypotheses)
   - [Graph (Neo4j)](#graph-neo4j)
   - [Search (ChromaDB)](#search-chromadb)
   - [Image Search (CLIP/DINOv2)](#image-search-clipdinov2)
   - [Monitoring](#monitoring)
   - [Alerts](#alerts)
   - [Reports](#reports)
   - [Timeline](#timeline)
   - [Geo](#geo)
   - [Recon (OSINT)](#recon-osint)
   - [Vision (VLM)](#vision-vlm)
   - [Forensics](#forensics)
   - [Physics Simulations](#physics-simulations)
   - [Investigation (boucle autonome)](#investigation-boucle-autonome)
   - [Audit](#audit)
5. [Codes HTTP utilises](#codes-http-utilises)
6. [Middleware et gestion d'erreurs](#middleware-et-gestion-derreurs)

---

## Informations generales

### Demarrage

```bash
uvicorn nexus.main:app --host 0.0.0.0 --port 8000 --reload
```

### Middleware

| Middleware | Description |
|---|---|
| `CORSMiddleware` | `allow_origins=["*"]`, credentials=True, tous methodes et headers |
| `ProcessTimeMiddleware` | Ajoute le header `X-Process-Time` (secondes) a chaque reponse |

### Gestion d'erreurs globale

- **`httpx.ConnectError` / `httpx.TimeoutException`** : 503, "LLM service unavailable"
- **`ollama.RequestError` / `ollama.ResponseError`** : 503, "LLM service error"
- **Toute autre exception non geree** : 500, "Internal server error"

### Lifecycle (lifespan)

Au demarrage :
1. Initialisation schema SQLite (idempotent)
2. Creation directories (`uploads`, `reports`, `backups`)
3. Singletons sur `app.state` : OllamaClient, LLMRouter, Neo4jClient, ChromaClient
4. MonitoringScheduler (APScheduler)
5. InvestigationManager (boucles autonomes)

A l'arret :
1. Arret de toutes les investigations autonomes
2. Arret du scheduler de monitoring
3. Fermeture Neo4j et ChromaDB

---

## Modeles Pydantic

Tous les modeles sont definis dans `nexus/db/models.py`. Convention :
- `*Base` : champs partages (Create + reponse)
- `*Create` : schema d'entree pour les POST
- `*Update` : mise a jour partielle (tous les champs Optional)
- `<Name>` : schema de reponse complet avec id + timestamps

### Case

```
CaseStatus = Literal["active", "closed", "archived"]
```

| Modele | Champs | Type | Defaut | Requis |
|---|---|---|---|---|
| **CaseBase** | `name` | `str` | -- | oui |
| | `reference` | `Optional[str]` | `None` | non |
| | `description` | `Optional[str]` | `None` | non |
| | `status` | `CaseStatus` | `"active"` | non |
| **CaseCreate** | *(herite de CaseBase)* | | | |
| **CaseUpdate** | `name` | `Optional[str]` | `None` | non |
| | `reference` | `Optional[str]` | `None` | non |
| | `description` | `Optional[str]` | `None` | non |
| | `status` | `Optional[CaseStatus]` | `None` | non |
| **Case** | *(herite de CaseBase)* | | | |
| | `id` | `str` | -- | auto |
| | `created_at` | `datetime` | -- | auto |
| | `updated_at` | `datetime` | -- | auto |

### Evidence

```
EvidenceType = Literal["pdf", "image", "text", "audio", "url", "manual"]
EvidenceStatus = Literal["pending", "processed", "error"]
```

| Modele | Champs | Type | Defaut | Requis |
|---|---|---|---|---|
| **EvidenceBase** | `case_id` | `str` | -- | oui |
| | `title` | `str` | -- | oui |
| | `evidence_type` | `EvidenceType` | -- | oui |
| | `source` | `Optional[str]` | `None` | non |
| | `source_date` | `Optional[datetime]` | `None` | non |
| | `reliability` | `int` | `50` | non |
| | `file_path` | `Optional[str]` | `None` | non |
| | `raw_text` | `Optional[str]` | `None` | non |
| | `summary` | `Optional[str]` | `None` | non |
| | `metadata` | `Optional[Any]` | `None` | non |
| | `status` | `EvidenceStatus` | `"pending"` | non |
| **EvidenceCreate** | *(herite de EvidenceBase)* | | | |
| **EvidenceUpdate** | *(tous champs Optional, memes types)* | | | |
| **Evidence** | *(herite de EvidenceBase)* | | | |
| | `id` | `str` | -- | auto |
| | `ingestion_date` | `datetime` | -- | auto |
| | `created_at` | `datetime` | -- | auto |

Contrainte : `reliability` entre 0 et 100 (`Field(ge=0, le=100)`).

### Entity

```
EntityType = Literal[
    "person", "location", "phone", "vehicle", "organization",
    "date", "money", "ip", "email", "account", "weapon", "drug", "other"
]
```

| Modele | Champs | Type | Defaut | Requis |
|---|---|---|---|---|
| **EntityBase** | `case_id` | `str` | -- | oui |
| | `name` | `str` | -- | oui |
| | `entity_type` | `EntityType` | -- | oui |
| | `aliases` | `Optional[List[str]]` | `None` | non |
| | `description` | `Optional[str]` | `None` | non |
| | `first_seen` | `Optional[datetime]` | `None` | non |
| | `metadata` | `Optional[Any]` | `None` | non |
| **EntityCreate** | *(herite de EntityBase)* | | | |
| **EntityUpdate** | *(tous champs Optional, memes types)* | | | |
| **Entity** | *(herite de EntityBase)* | | | |
| | `id` | `str` | -- | auto |
| | `created_at` | `datetime` | -- | auto |

### EntityMention

| Modele | Champs | Type | Defaut | Requis |
|---|---|---|---|---|
| **EntityMentionBase** | `entity_id` | `str` | -- | oui |
| | `evidence_id` | `str` | -- | oui |
| | `context` | `Optional[str]` | `None` | non |
| | `confidence` | `float` | `0.8` | non |
| **EntityMentionCreate** | *(herite de EntityMentionBase)* | | | |
| **EntityMention** | *(herite de EntityMentionBase)* | | | |
| | `id` | `str` | -- | auto |
| | `created_at` | `datetime` | -- | auto |

Contrainte : `confidence` entre 0.0 et 1.0.

### Hypothesis

```
HypothesisStatus = Literal["active", "refuted", "confirmed", "merged"]
```

| Modele | Champs | Type | Defaut | Requis |
|---|---|---|---|---|
| **HypothesisBase** | `case_id` | `str` | -- | oui |
| | `title` | `str` | -- | oui |
| | `description` | `str` | -- | oui |
| | `status` | `HypothesisStatus` | `"active"` | non |
| | `current_score` | `float` | `50.0` | non |
| **HypothesisCreate** | `title` | `str` | -- | oui |
| | `description` | `str` | -- | oui |
| | `status` | `HypothesisStatus` | `"active"` | non |
| | `current_score` | `float` | `50.0` | non |
| **HypothesisUpdate** | *(tous champs Optional)* | | | |
| **Hypothesis** | *(herite de HypothesisBase)* | | | |
| | `id` | `str` | -- | auto |
| | `created_at` | `datetime` | -- | auto |
| | `updated_at` | `datetime` | -- | auto |

Note : `HypothesisCreate` ne contient PAS `case_id` (fourni par l'URL path).
Contrainte : `current_score` entre 0.0 et 100.0.

### HypothesisSnapshot

| Modele | Champs | Type | Defaut | Requis |
|---|---|---|---|---|
| **HypothesisSnapshotBase** | `hypothesis_id` | `str` | -- | oui |
| | `score` | `float` | -- | oui |
| | `supporting` | `Optional[Any]` | `None` | non |
| | `contradicting` | `Optional[Any]` | `None` | non |
| | `reasoning` | `Optional[str]` | `None` | non |
| | `trigger` | `Optional[str]` | `None` | non |
| | `model_used` | `Optional[str]` | `None` | non |
| **HypothesisSnapshotCreate** | *(herite de HypothesisSnapshotBase)* | | | |
| **HypothesisSnapshot** | *(herite de HypothesisSnapshotBase)* | | | |
| | `id` | `str` | -- | auto |
| | `created_at` | `datetime` | -- | auto |

Contrainte : `score` entre 0.0 et 100.0.

### AnalysisRun

```
AnalysisRunType = Literal["full", "incremental", "verification", "extraction", "self_questioning"]
AnalysisTrigger = Literal["manual", "new_evidence", "monitoring", "scheduled", "evaluate_all"]
AnalysisStatus = Literal["running", "completed", "failed"]
```

| Modele | Champs | Type | Defaut | Requis |
|---|---|---|---|---|
| **AnalysisRunBase** | `case_id` | `str` | -- | oui |
| | `run_type` | `AnalysisRunType` | -- | oui |
| | `trigger` | `Optional[AnalysisTrigger]` | `None` | non |
| | `status` | `AnalysisStatus` | `"running"` | non |
| | `model_used` | `Optional[str]` | `None` | non |
| | `input_summary` | `Optional[str]` | `None` | non |
| | `output_summary` | `Optional[str]` | `None` | non |
| | `duration_sec` | `Optional[float]` | `None` | non |
| | `tokens_used` | `Optional[int]` | `None` | non |
| **AnalysisRunCreate** | `case_id` | `str` | -- | oui |
| | `run_type` | `AnalysisRunType` | -- | oui |
| | `trigger` | `Optional[AnalysisTrigger]` | `None` | non |
| | `model_used` | `Optional[str]` | `None` | non |
| | `input_summary` | `Optional[str]` | `None` | non |
| **AnalysisRun** | *(herite de AnalysisRunBase)* | | | |
| | `id` | `str` | -- | auto |
| | `started_at` | `datetime` | -- | auto |
| | `completed_at` | `Optional[datetime]` | `None` | non |

### MonitoringJob

```
MonitoringJobType = Literal["searxng", "robin", "both"]
```

| Modele | Champs | Type | Defaut | Requis |
|---|---|---|---|---|
| **MonitoringJobBase** | `case_id` | `str` | -- | oui |
| | `job_type` | `MonitoringJobType` | -- | oui |
| | `query` | `str` | -- | oui |
| | `entity_id` | `Optional[str]` | `None` | non |
| | `interval_hours` | `int` | `24` | non |
| | `is_active` | `bool` | `True` | non |
| | `last_run` | `Optional[datetime]` | `None` | non |
| | `next_run` | `Optional[datetime]` | `None` | non |
| | `results_count` | `int` | `0` | non |
| **MonitoringJobCreate** | `case_id` | `str` | -- | oui |
| | `job_type` | `MonitoringJobType` | -- | oui |
| | `query` | `str` | -- | oui |
| | `entity_id` | `Optional[str]` | `None` | non |
| | `interval_hours` | `int` | `24` | non |
| **MonitoringJob** | *(herite de MonitoringJobBase)* | | | |
| | `id` | `str` | -- | auto |
| | `created_at` | `datetime` | -- | auto |

### MonitoringResult

| Modele | Champs | Type | Defaut | Requis |
|---|---|---|---|---|
| **MonitoringResultBase** | `job_id` | `str` | -- | oui |
| | `case_id` | `str` | -- | oui |
| | `url` | `Optional[str]` | `None` | non |
| | `title` | `Optional[str]` | `None` | non |
| | `snippet` | `Optional[str]` | `None` | non |
| | `source_engine` | `Optional[str]` | `None` | non |
| | `relevance_score` | `Optional[float]` | `None` | non |
| | `is_new` | `bool` | `True` | non |
| | `is_duplicate` | `bool` | `False` | non |
| | `reviewed` | `bool` | `False` | non |
| **MonitoringResultCreate** | *(herite de MonitoringResultBase)* | | | |
| **MonitoringResult** | *(herite de MonitoringResultBase)* | | | |
| | `id` | `str` | -- | auto |
| | `found_at` | `datetime` | -- | auto |

Contrainte : `relevance_score` entre 0.0 et 100.0.

### Alert

```
AlertType = Literal["new_evidence", "score_shift", "monitoring_hit", "contradiction", "new_entity"]
AlertSeverity = Literal["info", "warning", "critical"]
```

| Modele | Champs | Type | Defaut | Requis |
|---|---|---|---|---|
| **AlertBase** | `case_id` | `str` | -- | oui |
| | `alert_type` | `AlertType` | -- | oui |
| | `severity` | `AlertSeverity` | `"info"` | non |
| | `title` | `str` | -- | oui |
| | `message` | `str` | -- | oui |
| | `related_id` | `Optional[str]` | `None` | non |
| | `is_read` | `bool` | `False` | non |
| **AlertCreate** | *(memes champs sauf is_read)* | | | |
| **Alert** | *(herite de AlertBase)* | | | |
| | `id` | `str` | -- | auto |
| | `created_at` | `datetime` | -- | auto |

### Report

```
ReportType = Literal["full", "summary", "timeline"]
ReportStatus = Literal["generating", "completed", "error"]
```

| Modele | Champs | Type | Defaut | Requis |
|---|---|---|---|---|
| **ReportBase** | `case_id` | `str` | -- | oui |
| | `report_type` | `ReportType` | -- | oui |
| | `status` | `ReportStatus` | `"generating"` | non |
| | `file_path` | `Optional[str]` | `None` | non |
| | `file_size` | `Optional[int]` | `None` | non |
| **ReportCreate** | `case_id` | `str` | -- | oui |
| | `report_type` | `ReportType` | -- | oui |
| **Report** | *(herite de ReportBase)* | | | |
| | `id` | `str` | -- | auto |
| | `created_at` | `datetime` | -- | auto |
| | `completed_at` | `Optional[datetime]` | `None` | non |

### AuditEntry

```
AuditAction = Literal[
    "evidence_added", "hypothesis_scored", "entity_discovered",
    "contradiction_found", "monitoring_result", "query_generated",
    "analysis_started", "analysis_completed", "alert_created",
    "hypothesis_created", "hypothesis_refuted", "hypothesis_confirmed",
    "evidence_ingested_auto", "self_questioning",
    "investigation_started", "investigation_stopped",
    "case_created", "case_updated"
]

AuditActor = Literal["system", "user", "worker", "monitoring"]
```

| Modele | Champs | Type | Defaut | Requis |
|---|---|---|---|---|
| **AuditEntryBase** | `case_id` | `str` | -- | oui |
| | `actor` | `str` | -- | oui |
| | `action` | `str` | -- | oui |
| | `target_type` | `Optional[str]` | `None` | non |
| | `target_id` | `Optional[str]` | `None` | non |
| | `summary` | `str` | -- | oui |
| | `details` | `Optional[Any]` | `None` | non |
| | `cycle_number` | `Optional[int]` | `None` | non |
| **AuditEntry** | *(herite de AuditEntryBase)* | | | |
| | `id` | `str` | -- | auto |
| | `timestamp` | `datetime` | -- | auto |

### Modeles specifiques aux routers (non dans models.py)

#### evidence.py -- TextEvidenceInput

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `title` | `str` | -- | oui |
| `text` | `str` | -- | oui |
| `source` | `Optional[str]` | `None` | non |

#### analysis.py -- AnalyzeRequest

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `trigger` | `str` | `"manual"` | non |
| `new_evidence_id` | `Optional[str]` | `None` | non |

#### search.py -- SearchRequest

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `query` | `str` | -- | oui |
| `n_results` | `int` | `10` | non |
| `collection` | `Literal["evidence", "entities"]` | `"evidence"` | non |

Contrainte : `n_results` entre 1 et 100.

#### hypotheses.py -- CompareTestimoniesRequest

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `evidence_ids` | `list[str]` | -- | oui |

Contrainte : `min_length=2`.

#### hypotheses.py -- MergeHypothesesRequest

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `hypothesis_ids` | `list[str]` | -- | oui |
| `new_title` | `str` | -- | oui |
| `new_description` | `str` | -- | oui |

Contrainte : `hypothesis_ids` avec `min_length=2`.

#### reports.py -- GenerateReportRequest

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `report_type` | `Literal["full", "summary", "timeline"]` | `"full"` | non |

#### reports.py -- ReportResponse

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `id` | `str` | -- | oui |
| `case_id` | `str` | -- | oui |
| `report_type` | `str` | -- | oui |
| `status` | `str` | -- | oui |
| `file_path` | `Optional[str]` | `None` | non |
| `file_size` | `Optional[int]` | `None` | non |
| `created_at` | `str` | -- | oui |
| `completed_at` | `Optional[str]` | `None` | non |

#### geo.py -- RouteRequest

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `origin` | `str` | -- | oui |
| `destination` | `str` | -- | oui |

#### geo.py -- VerifyTravelRequest

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `origin` | `str` | -- | oui |
| `destination` | `str` | -- | oui |
| `claimed_minutes` | `float` | -- | oui |

#### forensics.py -- ImpactAngleRequest

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `width` | `float` | -- | oui |
| `length` | `float` | -- | oui |

Contraintes : `width > 0`, `length > 0`.

#### forensics.py -- StainMeasurement

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `x` | `float` | -- | oui |
| `y` | `float` | -- | oui |
| `direction_degrees` | `float` | -- | oui |
| `width` | `Optional[float]` | `None` | non |
| `length` | `Optional[float]` | `None` | non |

Contraintes : `width > 0`, `length > 0` (si fournis).

#### forensics.py -- ConvergenceRequest

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `stains` | `List[StainMeasurement]` | -- | oui |

Contrainte : `min_length=2`.

#### forensics.py -- SoundPropagationRequest

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `source_x` | `float` | -- | oui |
| `source_y` | `float` | -- | oui |
| `listeners` | `List[Dict[str, float]]` | -- | oui |
| `speed_of_sound` | `float` | `343.0` | non |

Contrainte : `speed_of_sound > 0`.

#### physics_sim_api.py -- BloodDropRequest

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `velocity` | `float` | -- | oui |
| `angle` | `float` | -- | oui |
| `height` | `float` | -- | oui |
| `surface_angle` | `float` | `0.0` | non |
| `blood_properties` | `Optional[Dict[str, float]]` | `None` | non |

Contraintes : `velocity > 0`, `angle` entre -90 et 90, `height > 0`, `surface_angle` entre 0 et 90.

#### physics_sim_api.py -- CastOffRequest

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `swing_radius` | `float` | -- | oui |
| `swing_speed` | `float` | -- | oui |
| `num_drops` | `int` | `20` | non |
| `blood_on_weapon_length` | `float` | `0.3` | non |
| `swing_plane_height` | `float` | `1.5` | non |
| `swing_start_angle` | `float` | `-30.0` | non |
| `swing_end_angle` | `float` | `150.0` | non |
| `blood_properties` | `Optional[Dict[str, float]]` | `None` | non |

Contraintes : `swing_radius > 0`, `swing_speed > 0`, `num_drops` entre 1 et 100, `blood_on_weapon_length > 0`, `swing_plane_height > 0`.

#### physics_sim_api.py -- SoundRequest

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `source` | `List[float]` (3 valeurs) | -- | oui |
| `listeners` | `List[List[float]]` (3 valeurs chacun) | -- | oui |
| `source_db` | `float` | `160.0` | non |
| `frequency` | `float` | `2000.0` | non |
| `temperature` | `float` | `20.0` | non |
| `humidity` | `float` | `50.0` | non |
| `wind_speed` | `float` | `0.0` | non |
| `wind_direction` | `float` | `0.0` | non |
| `terrain` | `str` | `"urban"` | non |

Contraintes : `source_db` entre 0 et 200, `frequency > 0`, `temperature` entre -40 et 60, `humidity` entre 0 et 100, `wind_speed >= 0`, `wind_direction` entre 0 et 360, `terrain` parmi `urban|rural|indoor`.

#### physics_sim_api.py -- StainMeasurement (sim)

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `x` | `float` | -- | oui |
| `y` | `float` | -- | oui |
| `width_mm` | `float` | -- | oui |
| `length_mm` | `float` | -- | oui |
| `direction` | `float` | -- | oui |

Contraintes : `width_mm > 0`, `length_mm > 0`, `direction` entre 0 et 360.

#### physics_sim_api.py -- OriginEstimationRequest

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `stains` | `List[StainMeasurement]` | -- | oui |

Contrainte : `min_length=2`.

#### image_search.py -- TextSearchRequest

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `query` | `str` | -- | oui |
| `n_results` | `int` | `5` | non |

#### image_search.py -- ImageSearchRequest

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `evidence_id` | `str` | -- | oui |
| `n_results` | `int` | `5` | non |

#### image_search.py -- ImageSearchResult

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `evidence_id` | `str` | -- | oui |
| `path` | `str` | -- | oui |
| `case_id` | `str` | -- | oui |
| `description` | `str` | -- | oui |
| `distance` | `Optional[float]` | `None` | non |
| `similarity` | `Optional[float]` | `None` | non |

#### image_search.py -- IndexResponse

| Champ | Type | Defaut | Requis |
|---|---|---|---|
| `indexed` | `int` | -- | oui |
| `total` | `int` | -- | oui |

---

## Dependances FastAPI

Definies dans `nexus/api/deps.py`. Chaque requete HTTP obtient ses propres instances via `Depends()`.

### Connexions request-scoped

| Dependance | Fournit | Scope |
|---|---|---|
| `get_database()` | `Database` (aiosqlite) | Par requete (connection fermee a la fin) |
| `get_case_manager(db)` | `CaseManager` | Par requete |
| `get_evidence_processor(request, db)` | `EvidenceProcessor` | Par requete |
| `get_analysis_pipeline(request, db)` | `AnalysisPipeline` | Par requete |
| `get_entity_extractor(request)` | `EntityExtractor` | Par requete (stateless, seul le router est necessaire) |
| `get_geo_mapper(db)` | `GeoMapper` | Par requete |
| `get_image_analyzer(request, db)` | `ImageAnalyzer` | Par requete |
| `get_hypothesis_engine(request, db)` | `HypothesisEngine` | Par requete |
| `get_contradiction_detector(request, db)` | `ContradictionDetector` | Par requete |
| `get_audit_service(db)` | `AuditService` | Par requete |
| `get_bpa_analyzer(request)` | `BloodPatternAnalyzer` | Par requete |
| `get_acoustic_analyzer(request)` | `AcousticAnalyzer` | Par requete |
| `get_trace_analyzer(request)` | `TraceAnalyzer` | Par requete |

### Singletons sur app.state

| Dependance | Fournit | Scope |
|---|---|---|
| `get_neo4j(request)` | `Neo4jClient` | Singleton (app.state.neo4j) |
| `get_chroma(request)` | `ChromaClient` | Singleton (app.state.chroma) |
| `get_llm_router(request)` | `LLMRouter` | Singleton (app.state.router) |

### Dependances specifiques aux routers

| Router | Dependance | Fournit |
|---|---|---|
| `image_search.py` | `_get_image_search(request)` | `ImageSearchEngine` (lazy singleton sur app.state) |
| `investigation.py` | `_get_manager(request)` | `InvestigationManager` (depuis app.state, 503 si absent) |

---

## Endpoints par domaine

---

### System

#### GET /api/health
**Description :** Sonde de liveness. Ne verifie PAS la connectivite Ollama/Neo4j/ChromaDB.
**Tags :** `system`
**Parametres :** Aucun
**Response :** `200`
```json
{
  "status": "ok",
  "version": "0.1.0",
  "sqlite": "./data/nexus.db",
  "ollama": "http://localhost:11434"
}
```
**Erreurs :** Aucune specifique
**Background :** Non

---

### Cases

**Fichier :** `nexus/api/cases.py`
**Prefix :** `/api/cases`
**Tags :** `cases`

#### POST /api/cases
**Description :** Creer un nouveau dossier d'investigation.
**Body :** `CaseCreate`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `name` | `str` | oui | -- |
| `reference` | `str` | non | `null` |
| `description` | `str` | non | `null` |
| `status` | `CaseStatus` | non | `"active"` |
**Response :** `Case` (201)
**Erreurs :** Aucune specifique
**Background :** Non

#### GET /api/cases
**Description :** Lister tous les dossiers, avec filtre optionnel par statut.
**Query params :**
| Param | Type | Requis | Defaut |
|---|---|---|---|
| `status` | `str` | non | `null` |
**Response :** `list[Case]` (200)
**Erreurs :** Aucune specifique
**Background :** Non

#### GET /api/cases/{case_id}
**Description :** Recuperer un dossier par ID.
**Path params :** `case_id` (str)
**Response :** `Case` (200)
**Erreurs :** 404 (Case not found)
**Background :** Non

#### PUT /api/cases/{case_id}
**Description :** Mettre a jour un dossier (mise a jour partielle).
**Path params :** `case_id` (str)
**Body :** `CaseUpdate` (tous champs optionnels)
**Response :** `Case` (200)
**Erreurs :** 404 (Case not found)
**Background :** Non

#### DELETE /api/cases/{case_id}
**Description :** Supprimer un dossier et toutes les donnees dependantes (cascade).
**Path params :** `case_id` (str)
**Response :** 204 (No Content)
**Erreurs :** 404 (Case not found)
**Background :** Non

#### GET /api/cases/{case_id}/stats
**Description :** Statistiques agregees pour un dossier.
**Path params :** `case_id` (str)
**Response :** `dict` (200) -- contenu depend de `CaseManager.get_case_stats()`
**Erreurs :** 404 (Case not found)
**Background :** Non

---

### Evidence

**Fichier :** `nexus/api/evidence.py`
**Tags :** `evidence`

#### POST /api/cases/{case_id}/evidence
**Description :** Upload d'un fichier comme preuve (multipart/form-data).
**Path params :** `case_id` (str)
**Body :** `multipart/form-data`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `file` | `UploadFile` | oui | -- |
| `title` | `str` (Form) | oui | -- |
| `source` | `str` (Form) | non | `null` |
| `evidence_type` | `str` (Form) | non | `null` |
**Response :** `Evidence` (201)
**Erreurs :** Aucune specifique (erreurs internes via le processor)
**Background :** Non (le processing est synchrone dans cette version)

#### POST /api/cases/{case_id}/evidence/text
**Description :** Soumettre une preuve textuelle (notes, transcriptions, etc.).
**Path params :** `case_id` (str)
**Body :** `TextEvidenceInput`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `title` | `str` | oui | -- |
| `text` | `str` | oui | -- |
| `source` | `str` | non | `null` |
**Response :** `Evidence` (201)
**Erreurs :** Aucune specifique
**Background :** Non

#### GET /api/cases/{case_id}/evidence
**Description :** Lister toutes les preuves d'un dossier avec filtres optionnels.
**Path params :** `case_id` (str)
**Query params :**
| Param | Type | Requis | Defaut |
|---|---|---|---|
| `status` | `str` | non | `null` |
| `evidence_type` | `str` | non | `null` |
**Response :** `list[Evidence]` (200)
**Note :** Le filtre `evidence_type` est applique en memoire (pas en SQL).
**Erreurs :** Aucune specifique
**Background :** Non

#### GET /api/evidence/{evidence_id}
**Description :** Recuperer une preuve par ID.
**Path params :** `evidence_id` (str)
**Response :** `Evidence` (200)
**Erreurs :** 404 (Evidence not found)
**Background :** Non

#### PUT /api/evidence/{evidence_id}
**Description :** Mettre a jour les metadonnees d'une preuve.
**Path params :** `evidence_id` (str)
**Body :** `EvidenceUpdate` (tous champs optionnels)
**Response :** `Evidence` (200)
**Erreurs :** 404 (Evidence not found)
**Note :** Si aucun champ n'est modifie, retourne l'etat actuel sans erreur.
**Background :** Non

#### DELETE /api/evidence/{evidence_id}
**Description :** Supprimer une preuve.
**Path params :** `evidence_id` (str)
**Response :** 204 (No Content)
**Erreurs :** 404 (Evidence not found)
**Background :** Non

---

### Entities

**Fichier :** `nexus/api/entities.py`
**Tags :** `entities`

#### GET /api/cases/{case_id}/entities
**Description :** Lister toutes les entites d'un dossier, avec filtre optionnel par type.
**Path params :** `case_id` (str)
**Query params :**
| Param | Type | Requis | Defaut |
|---|---|---|---|
| `entity_type` | `str` | non | `null` |
**Response :** `list[Entity]` (200)
**Erreurs :** Aucune specifique
**Background :** Non

#### GET /api/entities/{entity_id}
**Description :** Recuperer une entite par ID.
**Path params :** `entity_id` (str)
**Response :** `Entity` (200)
**Erreurs :** 404 (Entity not found)
**Background :** Non

#### GET /api/entities/{entity_id}/mentions
**Description :** Lister toutes les mentions d'une entite a travers les preuves.
**Path params :** `entity_id` (str)
**Response :** `list[EntityMention]` (200)
**Erreurs :** 404 (Entity not found)
**Background :** Non

---

### Analysis

**Fichier :** `nexus/api/analysis.py`
**Tags :** `analysis`

#### POST /api/cases/{case_id}/analyze
**Description :** Lancer une analyse complete ou incrementale en arriere-plan. Retourne immediatement avec 202 Accepted.
**Path params :** `case_id` (str)
**Body :** `AnalyzeRequest` (optionnel, defauts appliques si absent)
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `trigger` | `str` | non | `"manual"` |
| `new_evidence_id` | `str` | non | `null` |
**Response :** 202
```json
{
  "run_id": "...",
  "status": "running",
  "run_type": "full|incremental"
}
```
**Logique :** Si `trigger == "manual"` et pas de `new_evidence_id` => analyse `full`. Sinon => `incremental`.
**Erreurs :** 404 (Case not found)
**Background :** **OUI** -- `_run_analysis_in_background()` ouvre sa propre connexion DB.

#### GET /api/analysis/{run_id}
**Description :** Consulter le statut d'un run d'analyse.
**Path params :** `run_id` (str)
**Response :** `AnalysisRun` (200)
**Erreurs :** 404 (Analysis run not found)
**Background :** Non

#### GET /api/cases/{case_id}/analysis-runs
**Description :** Historique des runs d'analyse pour un dossier.
**Path params :** `case_id` (str)
**Query params :**
| Param | Type | Requis | Defaut |
|---|---|---|---|
| `status` | `str` | non | `null` |
| `limit` | `int` | non | `50` |
**Response :** `list[AnalysisRun]` (200)
**Erreurs :** Aucune specifique
**Background :** Non

---

### Hypotheses

**Fichier :** `nexus/api/hypotheses.py`
**Prefix :** `/api`
**Tags :** `hypotheses`

#### POST /api/cases/{case_id}/hypotheses
**Description :** Creer une hypothese manuellement. Cree aussi un snapshot initial.
**Path params :** `case_id` (str)
**Body :** `HypothesisCreate`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `title` | `str` | oui | -- |
| `description` | `str` | oui | -- |
| `status` | `HypothesisStatus` | non | `"active"` |
| `current_score` | `float` | non | `50.0` |
**Response :** `Hypothesis` (201)
**Erreurs :** 404 (Case not found)
**Background :** Non

#### GET /api/cases/{case_id}/hypotheses
**Description :** Lister les hypotheses d'un dossier.
**Path params :** `case_id` (str)
**Query params :**
| Param | Type | Requis | Defaut |
|---|---|---|---|
| `status` | `str` | non | `null` |
**Response :** `list[Hypothesis]` (200)
**Erreurs :** Aucune specifique
**Background :** Non

#### GET /api/hypotheses/{hyp_id}
**Description :** Details d'une hypothese.
**Path params :** `hyp_id` (str)
**Response :** `Hypothesis` (200)
**Erreurs :** 404 (Hypothesis not found)
**Background :** Non

#### PUT /api/hypotheses/{hyp_id}
**Description :** Mettre a jour une hypothese (titre, description, statut, score).
**Path params :** `hyp_id` (str)
**Body :** `HypothesisUpdate` (tous champs optionnels)
**Response :** `Hypothesis` (200)
**Erreurs :** 404 (Hypothesis not found)
**Background :** Non

#### DELETE /api/hypotheses/{hyp_id}
**Description :** Archiver une hypothese (soft-delete : statut passe a "refuted"). NEXUS ne supprime jamais de donnees.
**Path params :** `hyp_id` (str)
**Response :** 200
```json
{
  "detail": "Hypothesis {hyp_id} archived (status set to 'refuted')"
}
```
**Erreurs :** 404 (Hypothesis not found)
**Background :** Non

#### POST /api/hypotheses/{hyp_id}/evaluate
**Description :** Forcer la re-evaluation d'une hypothese en arriere-plan.
**Path params :** `hyp_id` (str)
**Response :** 202
```json
{
  "hypothesis_id": "...",
  "status": "evaluation_started",
  "message": "Re-evaluation lancee en arriere-plan"
}
```
**Erreurs :** 404 (Hypothesis not found)
**Background :** **OUI** -- `_evaluate_hypothesis_bg()`

#### GET /api/hypotheses/{hyp_id}/snapshots
**Description :** Historique complet des snapshots d'une hypothese.
**Path params :** `hyp_id` (str)
**Response :** `list[HypothesisSnapshot]` (200)
**Erreurs :** 404 (Hypothesis not found)
**Background :** Non

#### GET /api/hypotheses/{hyp_id}/evolution
**Description :** Series temporelles de l'evolution du score d'une hypothese.
**Path params :** `hyp_id` (str)
**Response :** `list[dict]` (200) -- `[{date, score, trigger, model_used}]`
**Erreurs :** 404 (Hypothesis not found)
**Background :** Non

#### POST /api/cases/{case_id}/hypotheses/generate
**Description :** Generer des hypotheses via le LLM en arriere-plan.
**Path params :** `case_id` (str)
**Response :** 202
```json
{
  "case_id": "...",
  "status": "generation_started",
  "message": "Generation d'hypotheses lancee en arriere-plan"
}
```
**Erreurs :** 404 (Case not found)
**Background :** **OUI** -- `_generate_hypotheses_bg()`

#### POST /api/cases/{case_id}/evaluate-all
**Description :** Re-evaluer toutes les hypotheses actives d'un dossier en arriere-plan.
**Path params :** `case_id` (str)
**Response :** 202
```json
{
  "case_id": "...",
  "status": "evaluate_all_started",
  "message": "Re-evaluation de toutes les hypotheses lancee en arriere-plan"
}
```
**Erreurs :** 404 (Case not found)
**Background :** **OUI** -- `_evaluate_all_bg()`

#### POST /api/cases/{case_id}/hypotheses/merge
**Description :** Fusionner plusieurs hypotheses en une seule.
**Path params :** `case_id` (str)
**Body :** `MergeHypothesesRequest`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `hypothesis_ids` | `list[str]` (min 2) | oui | -- |
| `new_title` | `str` | oui | -- |
| `new_description` | `str` | oui | -- |
**Response :** `Hypothesis` (201)
**Erreurs :** 404 (Case not found), 400 (ValueError de l'engine)
**Background :** Non

#### GET /api/cases/{case_id}/contradictions
**Description :** Detecter les contradictions entre preuves d'un dossier. Appel synchrone.
**Path params :** `case_id` (str)
**Response :** `list[dict]` (200)
**Erreurs :** 404 (Case not found)
**Background :** Non
**Note :** Pour les tres gros dossiers, une version arriere-plan est envisagee.

#### POST /api/cases/{case_id}/compare-testimonies
**Description :** Comparer des temoignages specifiques pour convergences et divergences.
**Path params :** `case_id` (str)
**Body :** `CompareTestimoniesRequest`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `evidence_ids` | `list[str]` (min 2) | oui | -- |
**Response :** `dict` (200)
**Erreurs :** 404 (Case not found), 400 (ValueError)
**Background :** Non

---

### Graph (Neo4j)

**Fichier :** `nexus/api/graph.py`
**Prefix :** `/api`
**Tags :** `graph`

#### GET /api/cases/{case_id}/graph
**Description :** Retourner le graphe complet d'un dossier (noeuds + aretes). Format pret pour la visualisation front-end (streamlit-agraph).
**Path params :** `case_id` (str)
**Response :** `Dict[str, Any]` (200) -- `{nodes: [...], edges: [...]}`
**Erreurs :** Aucune specifique
**Background :** Non

#### GET /api/cases/{case_id}/graph/neighbors/{node_id}
**Description :** Retourner le sous-graphe autour d'un noeud jusqu'a N sauts.
**Path params :** `case_id` (str), `node_id` (str)
**Query params :**
| Param | Type | Requis | Defaut | Contrainte |
|---|---|---|---|---|
| `depth` | `int` | non | `1` | 1 <= depth <= 5 |
**Response :** `Dict[str, Any]` (200)
**Erreurs :** 404 (Node not found in case)
**Background :** Non

#### GET /api/cases/{case_id}/graph/path/{from_id}/{to_id}
**Description :** Trouver le chemin le plus court entre deux noeuds.
**Path params :** `case_id` (str), `from_id` (str), `to_id` (str)
**Response :** 200
```json
{
  "path": [...],
  "length": 3
}
```
**Erreurs :** 404 (No path found)
**Background :** Non

#### GET /api/cases/{case_id}/graph/clusters
**Description :** Detecter les composantes connexes dans le sous-graphe d'un dossier.
**Path params :** `case_id` (str)
**Response :** 200
```json
{
  "clusters": [[...], [...]],
  "count": 2
}
```
**Erreurs :** Aucune specifique
**Background :** Non

#### GET /api/cases/{case_id}/graph/stats
**Description :** Nombre de noeuds par label pour un dossier.
**Path params :** `case_id` (str)
**Response :** `Dict[str, int]` (200)
**Erreurs :** Aucune specifique
**Background :** Non

---

### Search (ChromaDB)

**Fichier :** `nexus/api/search.py`
**Prefix :** `/api`
**Tags :** `search`

#### POST /api/cases/{case_id}/search
**Description :** Recherche semantique sur les preuves ou entites d'un dossier. Le texte est embarque via `nomic-embed-text` puis compare dans ChromaDB.
**Path params :** `case_id` (str)
**Body :** `SearchRequest`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `query` | `str` | oui | -- |
| `n_results` | `int` (1-100) | non | `10` |
| `collection` | `"evidence" \| "entities"` | non | `"evidence"` |
**Response :** 200
```json
{
  "query": "...",
  "collection": "evidence",
  "count": 5,
  "results": [...]
}
```
**Erreurs :** Aucune specifique
**Background :** Non

#### GET /api/cases/{case_id}/similar/{evidence_id}
**Description :** Trouver les preuves semantiquement similaires a une preuve existante.
**Path params :** `case_id` (str), `evidence_id` (str)
**Query params :**
| Param | Type | Requis | Defaut | Contrainte |
|---|---|---|---|---|
| `n_results` | `int` | non | `5` | 1 <= n <= 50 |
**Response :** 200
```json
{
  "source_id": "...",
  "count": 5,
  "results": [...]
}
```
**Erreurs :** Aucune specifique
**Background :** Non

#### GET /api/cases/{case_id}/duplicates
**Description :** Detecter les paires de preuves quasi-identiques dans un dossier. Complexite O(n^2).
**Path params :** `case_id` (str)
**Query params :**
| Param | Type | Requis | Defaut | Contrainte |
|---|---|---|---|---|
| `threshold` | `float` | non | `0.92` | 0.5 <= t <= 1.0 |
**Response :** 200
```json
{
  "threshold": 0.92,
  "count": 2,
  "pairs": [
    {"id_a": "...", "id_b": "...", "similarity": 0.95}
  ]
}
```
**Erreurs :** Aucune specifique
**Background :** Non

---

### Image Search (CLIP/DINOv2)

**Fichier :** `nexus/api/image_search.py`
**Tags :** `image-search`

#### POST /api/cases/{case_id}/images/search-by-text
**Description :** Recherche d'images par texte en langage naturel via CLIP.
**Path params :** `case_id` (str)
**Body :** `TextSearchRequest`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `query` | `str` | oui | -- |
| `n_results` | `int` | non | `5` |
**Response :** `list[ImageSearchResult]` (200)
**Erreurs :** Aucune specifique
**Background :** Non

#### POST /api/cases/{case_id}/images/search-by-image
**Description :** Recherche d'images visuellement similaires via DINOv2. Prend un `evidence_id` et cherche les voisins visuels.
**Path params :** `case_id` (str)
**Body :** `ImageSearchRequest`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `evidence_id` | `str` | oui | -- |
| `n_results` | `int` | non | `5` |
**Response :** `list[ImageSearchResult]` (200)
**Erreurs :** 404 (Evidence not found or has no image file)
**Background :** Non

#### GET /api/cases/{case_id}/images/similar/{evidence_id}
**Description :** Trouver des images visuellement similaires a une image deja indexee. Utilise l'embedding DINOv2 stocke (pas de recalcul).
**Path params :** `case_id` (str), `evidence_id` (str)
**Query params :**
| Param | Type | Requis | Defaut |
|---|---|---|---|
| `n_results` | `int` | non | `5` |
**Response :** `list[ImageSearchResult]` (200)
**Erreurs :** Aucune specifique
**Background :** Non

#### POST /api/cases/{case_id}/images/index
**Description :** Indexer toutes les preuves image d'un dossier dans les collections CLIP et DINOv2.
**Path params :** `case_id` (str)
**Response :** `IndexResponse` (200)
```json
{
  "indexed": 12,
  "total": 25
}
```
**Erreurs :** 404 (No evidence found for case)
**Background :** Non

---

### Monitoring

**Fichier :** `nexus/api/monitoring.py`
**Tags :** `monitoring`

#### POST /api/cases/{case_id}/monitoring
**Description :** Creer un job de monitoring pour un dossier. L'enregistre aussi dans le scheduler APScheduler si disponible.
**Path params :** `case_id` (str)
**Body :** `MonitoringJobCreate`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `case_id` | `str` | oui | -- |
| `job_type` | `MonitoringJobType` | oui | -- |
| `query` | `str` | oui | -- |
| `entity_id` | `str` | non | `null` |
| `interval_hours` | `int` | non | `24` |
**Response :** `MonitoringJob` (201)
**Erreurs :** 404 (Case not found)
**Background :** Non

#### GET /api/cases/{case_id}/monitoring
**Description :** Lister tous les jobs de monitoring d'un dossier.
**Path params :** `case_id` (str)
**Query params :**
| Param | Type | Requis | Defaut |
|---|---|---|---|
| `active_only` | `bool` | non | `false` |
**Response :** `list[MonitoringJob]` (200)
**Erreurs :** Aucune specifique
**Background :** Non

#### PUT /api/monitoring/{job_id}
**Description :** Mettre a jour un job de monitoring. Synchronise le scheduler automatiquement.
**Path params :** `job_id` (str)
**Body :** `MonitoringJobUpdate` (tous champs optionnels)
**Response :** `MonitoringJob` (200)
**Erreurs :** 404 (Monitoring job not found)
**Note :** Si `is_active` passe a `false`, le job est retire du scheduler. Si `interval_hours` change, le scheduler est mis a jour.
**Background :** Non

#### DELETE /api/monitoring/{job_id}
**Description :** Supprimer un job de monitoring et le retirer du scheduler.
**Path params :** `job_id` (str)
**Response :** 204 (No Content)
**Erreurs :** 404 (Monitoring job not found)
**Background :** Non

#### POST /api/monitoring/{job_id}/run
**Description :** Forcer l'execution immediate d'un job de monitoring.
**Path params :** `job_id` (str)
**Response :** 202
```json
{
  "status": "triggered",
  "job_id": "..."
}
```
**Erreurs :** 404 (Monitoring job not found), 503 (Monitoring scheduler not available)
**Background :** Non (le trigger est gere par le scheduler)

#### GET /api/cases/{case_id}/monitoring/results
**Description :** Lister tous les resultats de monitoring d'un dossier (plus recents en premier).
**Path params :** `case_id` (str)
**Query params :**
| Param | Type | Requis | Defaut |
|---|---|---|---|
| `limit` | `int` | non | `200` |
**Response :** `list[MonitoringResult]` (200)
**Erreurs :** Aucune specifique
**Background :** Non

#### GET /api/monitoring/results/{result_id}
**Description :** Recuperer un resultat de monitoring par ID.
**Path params :** `result_id` (str)
**Response :** `MonitoringResult` (200)
**Erreurs :** 404 (Monitoring result not found)
**Background :** Non

#### POST /api/monitoring/results/{result_id}/ingest
**Description :** Convertir un resultat de monitoring en preuve. Marque le resultat comme `reviewed` et cree un enregistrement evidence lie au meme dossier.
**Path params :** `result_id` (str)
**Response :** 201
```json
{
  "evidence_id": "...",
  "monitoring_result_id": "...",
  "status": "ingested"
}
```
**Erreurs :** 404 (Monitoring result not found)
**Background :** Non

---

### Alerts

**Fichier :** `nexus/api/alerts.py`
**Tags :** `alerts`

#### GET /api/cases/{case_id}/alerts
**Description :** Lister les alertes d'un dossier avec filtres optionnels.
**Path params :** `case_id` (str)
**Query params :**
| Param | Type | Requis | Defaut |
|---|---|---|---|
| `severity` | `str` | non | `null` |
| `unread_only` | `bool` | non | `false` |
| `limit` | `int` | non | `100` |
**Response :** `list[Alert]` (200)
**Erreurs :** Aucune specifique
**Background :** Non

#### PUT /api/alerts/{alert_id}/read
**Description :** Marquer une alerte comme lue.
**Path params :** `alert_id` (str)
**Response :** 200
```json
{
  "alert_id": "...",
  "is_read": true
}
```
**Erreurs :** 404 (Alert not found)
**Background :** Non

#### GET /api/alerts/unread-count
**Description :** Nombre d'alertes non lues. Si `case_id` fourni, compte seulement ce dossier. Sinon, compte global.
**Query params :**
| Param | Type | Requis | Defaut |
|---|---|---|---|
| `case_id` | `str` | non | `null` |
**Response :** 200
```json
{
  "unread_count": 7,
  "case_id": null
}
```
**Erreurs :** Aucune specifique
**Background :** Non

---

### Reports

**Fichier :** `nexus/api/reports.py`
**Tags :** `reports`

#### POST /api/cases/{case_id}/reports/generate
**Description :** Demarrer la generation d'un rapport en arriere-plan. Le rapport est genere en PDF.
**Path params :** `case_id` (str)
**Body :** `GenerateReportRequest`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `report_type` | `"full" \| "summary" \| "timeline"` | non | `"full"` |
**Response :** `ReportResponse` (202)
**Erreurs :** 404 (Case not found)
**Background :** **OUI** -- `_generate_report_task()` utilise ReportGenerator + PDFExporter

#### GET /api/reports/{report_id}
**Description :** Consulter le statut et les metadonnees d'un rapport.
**Path params :** `report_id` (str)
**Response :** `ReportResponse` (200)
**Erreurs :** 404 (Report not found)
**Background :** Non

#### GET /api/reports/{report_id}/download
**Description :** Telecharger le fichier PDF du rapport genere.
**Path params :** `report_id` (str)
**Response :** `FileResponse` (200, `application/pdf`)
**Erreurs :** 404 (Report not found / file not found on disk), 409 (Report not ready -- status != "completed")
**Background :** Non

#### GET /api/cases/{case_id}/reports
**Description :** Lister tous les rapports d'un dossier.
**Path params :** `case_id` (str)
**Response :** `list[ReportResponse]` (200)
**Erreurs :** Aucune specifique
**Background :** Non

---

### Timeline

**Fichier :** `nexus/api/timeline.py`
**Prefix :** `/api`
**Tags :** `timeline`

#### GET /api/cases/{case_id}/timeline
**Description :** Construire une timeline chronologique pour un dossier.
**Path params :** `case_id` (str)
**Response :** `list[dict]` (200)
**Erreurs :** Aucune specifique
**Background :** Non

#### GET /api/cases/{case_id}/timeline/range
**Description :** Obtenir les evenements de la timeline dans un intervalle de dates.
**Path params :** `case_id` (str)
**Query params :**
| Param | Type | Requis | Defaut |
|---|---|---|---|
| `start` | `datetime` | oui | -- |
| `end` | `datetime` | oui | -- |
**Response :** `list[dict]` (200)
**Erreurs :** Aucune specifique (422 si format datetime invalide)
**Background :** Non

---

### Geo

**Fichier :** `nexus/api/geo.py`
**Prefix :** `/api`
**Tags :** `geo`

#### POST /api/cases/{case_id}/geocode
**Description :** Geocoder toutes les entites de type `location` d'un dossier.
**Path params :** `case_id` (str)
**Response :** 200
```json
{
  "total": 10,
  "geocoded": 7,
  "cached": 2,
  "not_found": 1,
  "results": [...]
}
```
**Erreurs :** Aucune specifique
**Background :** Non

#### GET /api/cases/{case_id}/map
**Description :** Retourner toutes les donnees necessaires pour rendre la carte d'investigation.
**Path params :** `case_id` (str)
**Response :** `dict` (200) -- structure dependant de `GeoMapper.build_case_map_data()`
**Erreurs :** Aucune specifique
**Background :** Non

#### POST /api/cases/{case_id}/route
**Description :** Calculer un trajet routier entre deux adresses.
**Path params :** `case_id` (str)
**Body :** `RouteRequest`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `origin` | `str` | oui | -- |
| `destination` | `str` | oui | -- |
**Response :** `dict` (200) -- ou `{"error": "Impossible de calculer le trajet."}` si echec
**Erreurs :** Aucune HTTP specifique (erreur retournee dans le body)
**Background :** Non

#### POST /api/cases/{case_id}/verify-travel
**Description :** Verifier si un temps de trajet revendique entre deux adresses est plausible.
**Path params :** `case_id` (str)
**Body :** `VerifyTravelRequest`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `origin` | `str` | oui | -- |
| `destination` | `str` | oui | -- |
| `claimed_minutes` | `float` | oui | -- |
**Response :** `dict` (200) -- ou `{"error": "Impossible de verifier le trajet."}` si echec
**Erreurs :** Aucune HTTP specifique (erreur retournee dans le body)
**Background :** Non

---

### Recon (OSINT)

**Fichier :** `nexus/api/recon.py`
**Prefix :** `/api`
**Tags :** `recon`

#### POST /api/recon/email/{email}
**Description :** Scanner un email via holehe (120+ sites) et plateformes sociales.
**Path params :** `email` (str)
**Response :** 200
```json
{
  "email": "user@example.com",
  "holehe": [...],
  "social": [...],
  "holehe_count": 5,
  "social_found": 2
}
```
**Erreurs :** Aucune specifique
**Background :** Non

#### POST /api/recon/username/{username}
**Description :** Rechercher un nom d'utilisateur sur les principales plateformes sociales.
**Path params :** `username` (str)
**Response :** 200
```json
{
  "username": "johndoe",
  "results": [...],
  "found_count": 8
}
```
**Erreurs :** Aucune specifique
**Background :** Non

#### POST /api/recon/domain/{domain}
**Description :** Effectuer un WHOIS + DNS lookup sur un domaine.
**Path params :** `domain` (str)
**Response :** 200
```json
{
  "domain": "example.com",
  "whois": {...},
  "dns": {...}
}
```
**Erreurs :** Aucune specifique
**Background :** Non

#### GET /api/cases/{case_id}/recon
**Description :** Retourner toutes les entites avec des metadonnees recon pour un dossier. Filtre les entites de type `email` ou `account` ayant un champ `metadata.recon`.
**Path params :** `case_id` (str)
**Response :** `list[dict]` (200)
**Erreurs :** 404 ("Dossier introuvable")
**Background :** Non

#### POST /api/cases/{case_id}/recon/auto
**Description :** Lancer un scan recon automatique sur toutes les entites email/account d'un dossier. Stocke les resultats dans `metadata.recon` de chaque entite.
**Path params :** `case_id` (str)
**Response :** 200
```json
{
  "case_id": "...",
  "scanned": 5,
  "errors": 1,
  "results": [...]
}
```
**Erreurs :** 404 ("Dossier introuvable")
**Background :** Non

---

### Vision (VLM)

**Fichier :** `nexus/api/vision.py`
**Prefix :** `/api`
**Tags :** `vision`

#### POST /api/evidence/{evidence_id}/analyze-image
**Description :** Analyser l'image attachee a une preuve. Pipeline complet : description, extraction d'entites, analyse de scene, embedding, sauvegarde.
**Path params :** `evidence_id` (str)
**Response :** `dict` (200)
**Erreurs :** 404 (Evidence not found), 400 (Evidence is not type 'image' / file not found)
**Background :** Non

#### POST /api/cases/{case_id}/analyze-images
**Description :** Analyser TOUTES les preuves image d'un dossier via le pipeline visuel complet.
**Path params :** `case_id` (str)
**Response :** 200
```json
{
  "case_id": "...",
  "images_found": 10,
  "images_processed": 8,
  "results": [...],
  "errors": [...]
}
```
**Erreurs :** 404 (Case not found)
**Background :** Non

#### POST /api/vision/describe
**Description :** Upload direct d'une image pour obtenir une description. Ne stocke PAS l'image comme preuve. Utile pour des verifications rapides.
**Body :** `multipart/form-data`
| Champ | Type | Requis |
|---|---|---|
| `file` | `UploadFile` | oui |
**Response :** 200
```json
{
  "filename": "photo.jpg",
  "description": "...",
  "entities": [...]
}
```
**Erreurs :** Aucune specifique
**Background :** Non

#### POST /api/vision/compare
**Description :** Comparer deux images de preuves cote a cote. Les deux doivent etre de type `image` avec des fichiers valides.
**Body :** `multipart/form-data`
| Champ | Type | Requis |
|---|---|---|
| `evidence_id_1` | `str` (Form) | oui |
| `evidence_id_2` | `str` (Form) | oui |
**Response :** `dict` (200) -- inclut `evidence_id_1` et `evidence_id_2`
**Erreurs :** 404 (Evidence not found), 400 (Evidence is not an image / file not found)
**Background :** Non

#### GET /api/cases/{case_id}/visual-entities
**Description :** Lister toutes les entites extraites d'images pour un dossier (filtrage sur `metadata.source == "visual_extraction"`).
**Path params :** `case_id` (str)
**Response :** `list[dict]` (200)
**Erreurs :** Aucune specifique
**Background :** Non

---

### Forensics

**Fichier :** `nexus/api/forensics.py`
**Prefix :** `/api/forensics`
**Tags :** `forensics`

#### POST /api/forensics/bpa/classify
**Description :** Classifier un pattern de taches de sang a partir d'une image uploadee (spatter, transfer, drip, pool, cast-off, arterial, etc.).
**Body :** `multipart/form-data`
| Champ | Type | Requis |
|---|---|---|
| `file` | `UploadFile` | oui |
**Response :** `dict` (200) -- inclut `filename`
**Erreurs :** Aucune specifique
**Background :** Non

#### POST /api/forensics/bpa/analyze
**Description :** Analyse BPA complete : classification VLM + calculs geometriques. Optionnellement fournir des mesures et un contexte.
**Body :** `multipart/form-data`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `file` | `UploadFile` | oui | -- |
| `measurements` | `str` (JSON) (Form) | non | `null` |
| `case_context` | `str` (Form) | non | `""` |
**Format measurements :**
```json
[{"x": 0, "y": 0, "direction_degrees": 45, "width": 3, "length": 6}]
```
**Response :** `dict` (200)
**Erreurs :** 400 (Invalid measurements JSON)
**Background :** Non

#### POST /api/forensics/bpa/calculate-angle
**Description :** Calculer l'angle d'impact a partir de la largeur et longueur d'une tache. Formule : `sin(angle) = width / length`.
**Body :** `ImpactAngleRequest`
| Champ | Type | Requis |
|---|---|---|
| `width` | `float` (>0) | oui |
| `length` | `float` (>0) | oui |
**Response :** 200
```json
{
  "width": 3.0,
  "length": 6.0,
  "impact_angle_degrees": 30.0,
  "formula": "sin(angle) = width / length"
}
```
**Erreurs :** 400 (ValueError)
**Background :** Non

#### POST /api/forensics/bpa/convergence
**Description :** Calculer la zone de convergence 2D a partir de mesures de taches. Calcule aussi la zone d'origine si width/length sont fournis.
**Body :** `ConvergenceRequest`
| Champ | Type | Requis |
|---|---|---|
| `stains` | `List[StainMeasurement]` (min 2) | oui |
**Response :** 200
```json
{
  "convergence": {...},
  "area_of_origin": {...}
}
```
**Erreurs :** 400 (ValueError)
**Background :** Non

#### POST /api/forensics/audio/transcribe
**Description :** Transcrire un fichier audio via le modele voxtral.
**Body :** `multipart/form-data`
| Champ | Type | Requis |
|---|---|---|
| `file` | `UploadFile` | oui |
**Response :** 200
```json
{
  "filename": "audio.wav",
  "transcription": "..."
}
```
**Erreurs :** Aucune specifique
**Background :** Non

#### POST /api/forensics/audio/analyze
**Description :** Analyse forensique complete d'un enregistrement audio : transcription, detection d'evenements, evaluation LLM.
**Body :** `multipart/form-data`
| Champ | Type | Requis |
|---|---|---|
| `file` | `UploadFile` | oui |
**Response :** `dict` (200) -- inclut `filename`
**Erreurs :** Aucune specifique
**Background :** Non

#### POST /api/forensics/audio/events
**Description :** Detecter les evenements notables dans un fichier audio (WAV). Analyse d'energie RMS pour sons forts et silences.
**Body :** `multipart/form-data`
| Champ | Type | Requis |
|---|---|---|
| `file` | `UploadFile` | oui |
**Response :** 200
```json
{
  "filename": "audio.wav",
  "event_count": 3,
  "events": [...]
}
```
**Erreurs :** Aucune specifique
**Background :** Non

#### POST /api/forensics/audio/propagation
**Description :** Calculer les temps d'arrivee du son a differentes positions d'auditeurs. Utile pour la localisation de coups de feu avec plusieurs temoins.
**Body :** `SoundPropagationRequest`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `source_x` | `float` | oui | -- |
| `source_y` | `float` | oui | -- |
| `listeners` | `List[{x, y}]` | oui | -- |
| `speed_of_sound` | `float` | non | `343.0` |
**Response :** 200
```json
{
  "source": {"x": 0, "y": 0},
  "speed_of_sound_ms": 343.0,
  "listeners": [...]
}
```
**Erreurs :** 400 (ValueError)
**Background :** Non

#### POST /api/forensics/trace/analyze
**Description :** Analyser une trace physique a partir d'une photo. Types supportes : fingerprint, tool_mark, tire_track, shoe_print, glass_fracture, fabric, hair, fiber, auto.
**Body :** `multipart/form-data`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `file` | `UploadFile` | oui | -- |
| `trace_type` | `str` (Form) | non | `"auto"` |
**Response :** `dict` (200) -- inclut `filename`
**Erreurs :** Aucune specifique
**Background :** Non

#### POST /api/forensics/trace/compare
**Description :** Comparer deux images de traces pour evaluer si elles proviennent de la meme source.
**Body :** `multipart/form-data`
| Champ | Type | Requis |
|---|---|---|
| `file_1` | `UploadFile` | oui |
| `file_2` | `UploadFile` | oui |
**Response :** `dict` (200) -- inclut `filename_1` et `filename_2`
**Erreurs :** Aucune specifique
**Background :** Non

#### POST /api/forensics/cases/{case_id}/auto
**Description :** Lancer l'analyse forensique automatique sur toutes les preuves d'un dossier. Images => BPA + traces. Audio => transcription + analyse forensique.
**Path params :** `case_id` (str)
**Tags supplementaires :** `cases`
**Response :** 200
```json
{
  "case_id": "...",
  "evidence_processed": 5,
  "errors_count": 1,
  "results": [...],
  "errors": [...]
}
```
**Erreurs :** 404 (Case not found)
**Background :** Non

---

### Physics Simulations

**Fichier :** `nexus/api/physics_sim_api.py`
**Prefix :** `/api/forensics/sim`
**Tags :** `physics-sim`

#### POST /api/forensics/sim/blood-drop
**Description :** Simuler la trajectoire d'une goutte de sang et le pattern d'impact. Mouvement projectile avec trainee dependante de Reynolds + geometrie de tache elliptique.
**Body :** `BloodDropRequest`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `velocity` | `float` (>0) | oui | -- |
| `angle` | `float` (-90 a 90) | oui | -- |
| `height` | `float` (>0) | oui | -- |
| `surface_angle` | `float` (0-90) | non | `0.0` |
| `blood_properties` | `Dict[str, float]` | non | `null` |
**Response :** `dict` (200)
**Erreurs :** 500 (Simulation failed)
**Background :** Non

#### POST /api/forensics/sim/cast-off
**Description :** Simuler un pattern de cast-off (projection) a partir d'une arme en mouvement. Modelise le detachement des gouttelettes le long de l'arc de balancement.
**Body :** `CastOffRequest`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `swing_radius` | `float` (>0) | oui | -- |
| `swing_speed` | `float` (>0) | oui | -- |
| `num_drops` | `int` (1-100) | non | `20` |
| `blood_on_weapon_length` | `float` (>0) | non | `0.3` |
| `swing_plane_height` | `float` (>0) | non | `1.5` |
| `swing_start_angle` | `float` | non | `-30.0` |
| `swing_end_angle` | `float` | non | `150.0` |
| `blood_properties` | `Dict[str, float]` | non | `null` |
**Response :** 200
```json
{
  "num_drops_released": 18,
  "num_drops_requested": 20,
  "drops": [...]
}
```
**Erreurs :** 500 (Simulation failed)
**Background :** Non

#### POST /api/forensics/sim/sound
**Description :** Simuler la propagation sonore d'une source ponctuelle vers plusieurs auditeurs. Tient compte de la propagation geometrique, l'absorption atmospherique (ISO 9613), les effets de terrain et le vent.
**Body :** `SoundRequest`
| Champ | Type | Requis | Defaut |
|---|---|---|---|
| `source` | `[x, y, z]` | oui | -- |
| `listeners` | `[[x,y,z], ...]` | oui | -- |
| `source_db` | `float` (0-200) | non | `160.0` |
| `frequency` | `float` (>0) | non | `2000.0` |
| `temperature` | `float` (-40 a 60) | non | `20.0` |
| `humidity` | `float` (0-100) | non | `50.0` |
| `wind_speed` | `float` (>=0) | non | `0.0` |
| `wind_direction` | `float` (0-360) | non | `0.0` |
| `terrain` | `urban\|rural\|indoor` | non | `"urban"` |
**Response :** `dict` (200)
**Erreurs :** 422 (Listener must have 3 coordinates), 500 (Simulation failed)
**Background :** Non

#### POST /api/forensics/sim/origin
**Description :** Estimer la zone d'origine d'impact a partir de mesures de taches de sang. Methode tangente (arcsin width/length) pour projeter les lignes de convergence.
**Body :** `OriginEstimationRequest`
| Champ | Type | Requis |
|---|---|---|
| `stains` | `List[StainMeasurement]` (min 2) | oui |
**Response :** `dict` (200)
**Erreurs :** 422 (Error in estimation), 500 (Estimation failed)
**Background :** Non

#### GET /api/forensics/sim/datasets
**Description :** Lister les datasets de simulation physique de The Well pertinents pour la forensique.
**Response :** 200
```json
{
  "the_well_installed": true,
  "datasets": [...]
}
```
**Erreurs :** Aucune specifique
**Background :** Non

#### GET /api/forensics/sim/datasets/{name}
**Description :** Obtenir les informations detaillees d'un dataset The Well.
**Path params :** `name` (str)
**Response :** `dict` (200)
**Erreurs :** 404 (Dataset not found)
**Background :** Non

---

### Investigation (boucle autonome)

**Fichier :** `nexus/api/investigation.py`
**Tags :** `investigation`

#### GET /api/investigations
**Description :** Statut de toutes les investigations autonomes actives.
**Response :** `dict` (200) -- structure dependant de `InvestigationManager.get_status()`
**Erreurs :** 503 (Investigation manager not available)
**Background :** Non

#### POST /api/cases/{case_id}/investigation/start
**Description :** Demarrer l'investigation autonome pour un dossier.
**Path params :** `case_id` (str)
**Response :** 200
```json
{"status": "started", "case_id": "..."}
// ou
{"status": "already_running", "case_id": "..."}
```
**Erreurs :** 404 (Case not found), 503 (Investigation manager not available)
**Background :** Non (le manager gere les boucles en interne)

#### POST /api/cases/{case_id}/investigation/stop
**Description :** Arreter l'investigation autonome pour un dossier.
**Path params :** `case_id` (str)
**Response :** 200
```json
{"status": "stopped", "case_id": "..."}
// ou
{"status": "not_running", "case_id": "..."}
```
**Erreurs :** 503 (Investigation manager not available)
**Background :** Non

#### GET /api/cases/{case_id}/investigation/status
**Description :** Statut detaille de l'investigation d'un dossier.
**Path params :** `case_id` (str)
**Response :** 200
```json
{
  "case_id": "...",
  "running": true,
  "cycle_count": 12,
  "last_action": "...",
  "last_cycle_at": "...",
  "started_at": "..."
}
```
**Erreurs :** 503 (Investigation manager not available)
**Note :** Si l'investigation n'est pas active, retourne un objet avec `running: false` et `cycle_count: 0`.
**Background :** Non

#### GET /api/cases/{case_id}/investigation/log
**Description :** Journal des actions pour un dossier. Filtre les analysis_runs avec `trigger='evaluate_all'` ou `run_type='self_questioning'`.
**Path params :** `case_id` (str)
**Query params :**
| Param | Type | Requis | Defaut |
|---|---|---|---|
| `limit` | `int` | non | `50` |
**Response :** `list[dict]` (200)
**Erreurs :** Aucune specifique
**Background :** Non

---

### Audit

**Fichier :** `nexus/api/audit.py`
**Tags :** `audit`

#### GET /api/cases/{case_id}/audit
**Description :** Retourner le journal d'audit d'un dossier avec filtres optionnels.
**Path params :** `case_id` (str)
**Query params :**
| Param | Type | Requis | Defaut | Contrainte |
|---|---|---|---|---|
| `action` | `str` | non | `null` | Voir `AuditAction` |
| `actor` | `str` | non | `null` | Voir `AuditActor` |
| `limit` | `int` | non | `100` | 1 <= limit <= 1000 |
| `offset` | `int` | non | `0` | offset >= 0 |
**Response :** `list[dict]` (200)
**Erreurs :** Aucune specifique
**Background :** Non

#### GET /api/cases/{case_id}/audit/summary
**Description :** Comptes groupes par type d'action pour le journal d'audit.
**Path params :** `case_id` (str)
**Response :** 200
```json
{
  "case_id": "...",
  "total": 150,
  "by_action": {
    "evidence_added": 25,
    "hypothesis_scored": 40,
    "entity_discovered": 15,
    ...
  }
}
```
**Erreurs :** Aucune specifique
**Background :** Non

#### GET /api/cases/{case_id}/audit/timeline
**Description :** Trace d'audit complete triee chronologiquement (plus ancien en premier).
**Path params :** `case_id` (str)
**Response :** `list[dict]` (200)
**Erreurs :** Aucune specifique
**Background :** Non

#### GET /api/audit/{audit_id}
**Description :** Recuperer une entree d'audit par ID.
**Path params :** `audit_id` (str)
**Response :** `dict` (200)
**Erreurs :** 404 (Audit entry not found)
**Background :** Non

#### GET /api/cases/{case_id}/audit/verify
**Description :** Verifier l'integrite de la chaine de hash du journal d'audit. Detecte la falsification.
**Path params :** `case_id` (str)
**Response :** `dict` (200) -- structure dependant de `AuditService.verify_chain()`
**Erreurs :** Aucune specifique
**Background :** Non

---

## Codes HTTP utilises

| Code | Signification | Utilise par |
|---|---|---|
| **200** | OK | La plupart des GET et PUT |
| **201** | Created | POST creation (cases, evidence, hypotheses, monitoring jobs, merge, ingest) |
| **202** | Accepted | Taches en arriere-plan (analyze, evaluate, generate, reports/generate, monitoring/run) |
| **204** | No Content | DELETE (cases, evidence, monitoring jobs) |
| **400** | Bad Request | Donnees invalides (measurements JSON, ValueError) |
| **404** | Not Found | Ressource introuvable (case, evidence, entity, hypothesis, alert, report, audit, node, path, dataset) |
| **409** | Conflict | Report pas encore pret (download avant completion) |
| **422** | Unprocessable Entity | Validation Pydantic echouee, coordonnees invalides |
| **500** | Internal Server Error | Erreur non geree, simulation echouee |
| **503** | Service Unavailable | Ollama indisponible, scheduler indisponible, investigation manager absent |

---

## Middleware et gestion d'erreurs

### Headers de reponse

| Header | Description |
|---|---|
| `X-Process-Time` | Temps de traitement en secondes (ex: `0.0234`) |

### CORS

Configuree pour accepter toutes les origines (`*`), tous les headers et toutes les methodes. Credentials autorisees.

### Exception handler global

L'application intercepte :
1. **`httpx.ConnectError` / `httpx.TimeoutException`** => 503
2. **`ollama.RequestError` / `ollama.ResponseError`** => 503
3. **Toute autre exception** => 500

### Endpoints avec BackgroundTasks

Les endpoints suivants lancent du travail en arriere-plan via `BackgroundTasks` de FastAPI. Chaque tache en arriere-plan ouvre sa propre connexion DB car la connexion request-scoped est fermee apres l'envoi de la reponse.

| Endpoint | Fonction background | Description |
|---|---|---|
| `POST /api/cases/{case_id}/analyze` | `_run_analysis_in_background()` | Analyse full ou incrementale |
| `POST /api/hypotheses/{hyp_id}/evaluate` | `_evaluate_hypothesis_bg()` | Re-evaluation d'une hypothese |
| `POST /api/cases/{case_id}/hypotheses/generate` | `_generate_hypotheses_bg()` | Generation d'hypotheses via LLM |
| `POST /api/cases/{case_id}/evaluate-all` | `_evaluate_all_bg()` | Re-evaluation de toutes les hypotheses |
| `POST /api/cases/{case_id}/reports/generate` | `_generate_report_task()` | Generation de rapport PDF |

---

## Inventaire complet des endpoints

| # | Methode | Route | Status | Background |
|---|---|---|---|---|
| 1 | GET | `/api/health` | 200 | Non |
| 2 | POST | `/api/cases` | 201 | Non |
| 3 | GET | `/api/cases` | 200 | Non |
| 4 | GET | `/api/cases/{case_id}` | 200 | Non |
| 5 | PUT | `/api/cases/{case_id}` | 200 | Non |
| 6 | DELETE | `/api/cases/{case_id}` | 204 | Non |
| 7 | GET | `/api/cases/{case_id}/stats` | 200 | Non |
| 8 | POST | `/api/cases/{case_id}/evidence` | 201 | Non |
| 9 | POST | `/api/cases/{case_id}/evidence/text` | 201 | Non |
| 10 | GET | `/api/cases/{case_id}/evidence` | 200 | Non |
| 11 | GET | `/api/evidence/{evidence_id}` | 200 | Non |
| 12 | PUT | `/api/evidence/{evidence_id}` | 200 | Non |
| 13 | DELETE | `/api/evidence/{evidence_id}` | 204 | Non |
| 14 | GET | `/api/cases/{case_id}/entities` | 200 | Non |
| 15 | GET | `/api/entities/{entity_id}` | 200 | Non |
| 16 | GET | `/api/entities/{entity_id}/mentions` | 200 | Non |
| 17 | POST | `/api/cases/{case_id}/analyze` | 202 | **OUI** |
| 18 | GET | `/api/analysis/{run_id}` | 200 | Non |
| 19 | GET | `/api/cases/{case_id}/analysis-runs` | 200 | Non |
| 20 | POST | `/api/cases/{case_id}/hypotheses` | 201 | Non |
| 21 | GET | `/api/cases/{case_id}/hypotheses` | 200 | Non |
| 22 | GET | `/api/hypotheses/{hyp_id}` | 200 | Non |
| 23 | PUT | `/api/hypotheses/{hyp_id}` | 200 | Non |
| 24 | DELETE | `/api/hypotheses/{hyp_id}` | 200 | Non |
| 25 | POST | `/api/hypotheses/{hyp_id}/evaluate` | 202 | **OUI** |
| 26 | GET | `/api/hypotheses/{hyp_id}/snapshots` | 200 | Non |
| 27 | GET | `/api/hypotheses/{hyp_id}/evolution` | 200 | Non |
| 28 | POST | `/api/cases/{case_id}/hypotheses/generate` | 202 | **OUI** |
| 29 | POST | `/api/cases/{case_id}/evaluate-all` | 202 | **OUI** |
| 30 | POST | `/api/cases/{case_id}/hypotheses/merge` | 201 | Non |
| 31 | GET | `/api/cases/{case_id}/contradictions` | 200 | Non |
| 32 | POST | `/api/cases/{case_id}/compare-testimonies` | 200 | Non |
| 33 | GET | `/api/cases/{case_id}/graph` | 200 | Non |
| 34 | GET | `/api/cases/{case_id}/graph/neighbors/{node_id}` | 200 | Non |
| 35 | GET | `/api/cases/{case_id}/graph/path/{from_id}/{to_id}` | 200 | Non |
| 36 | GET | `/api/cases/{case_id}/graph/clusters` | 200 | Non |
| 37 | GET | `/api/cases/{case_id}/graph/stats` | 200 | Non |
| 38 | POST | `/api/cases/{case_id}/search` | 200 | Non |
| 39 | GET | `/api/cases/{case_id}/similar/{evidence_id}` | 200 | Non |
| 40 | GET | `/api/cases/{case_id}/duplicates` | 200 | Non |
| 41 | POST | `/api/cases/{case_id}/images/search-by-text` | 200 | Non |
| 42 | POST | `/api/cases/{case_id}/images/search-by-image` | 200 | Non |
| 43 | GET | `/api/cases/{case_id}/images/similar/{evidence_id}` | 200 | Non |
| 44 | POST | `/api/cases/{case_id}/images/index` | 200 | Non |
| 45 | POST | `/api/cases/{case_id}/monitoring` | 201 | Non |
| 46 | GET | `/api/cases/{case_id}/monitoring` | 200 | Non |
| 47 | PUT | `/api/monitoring/{job_id}` | 200 | Non |
| 48 | DELETE | `/api/monitoring/{job_id}` | 204 | Non |
| 49 | POST | `/api/monitoring/{job_id}/run` | 202 | Non |
| 50 | GET | `/api/cases/{case_id}/monitoring/results` | 200 | Non |
| 51 | GET | `/api/monitoring/results/{result_id}` | 200 | Non |
| 52 | POST | `/api/monitoring/results/{result_id}/ingest` | 201 | Non |
| 53 | GET | `/api/cases/{case_id}/alerts` | 200 | Non |
| 54 | PUT | `/api/alerts/{alert_id}/read` | 200 | Non |
| 55 | GET | `/api/alerts/unread-count` | 200 | Non |
| 56 | POST | `/api/cases/{case_id}/reports/generate` | 202 | **OUI** |
| 57 | GET | `/api/reports/{report_id}` | 200 | Non |
| 58 | GET | `/api/reports/{report_id}/download` | 200 | Non |
| 59 | GET | `/api/cases/{case_id}/reports` | 200 | Non |
| 60 | GET | `/api/cases/{case_id}/timeline` | 200 | Non |
| 61 | GET | `/api/cases/{case_id}/timeline/range` | 200 | Non |
| 62 | POST | `/api/cases/{case_id}/geocode` | 200 | Non |
| 63 | GET | `/api/cases/{case_id}/map` | 200 | Non |
| 64 | POST | `/api/cases/{case_id}/route` | 200 | Non |
| 65 | POST | `/api/cases/{case_id}/verify-travel` | 200 | Non |
| 66 | POST | `/api/recon/email/{email}` | 200 | Non |
| 67 | POST | `/api/recon/username/{username}` | 200 | Non |
| 68 | POST | `/api/recon/domain/{domain}` | 200 | Non |
| 69 | GET | `/api/cases/{case_id}/recon` | 200 | Non |
| 70 | POST | `/api/cases/{case_id}/recon/auto` | 200 | Non |
| 71 | POST | `/api/evidence/{evidence_id}/analyze-image` | 200 | Non |
| 72 | POST | `/api/cases/{case_id}/analyze-images` | 200 | Non |
| 73 | POST | `/api/vision/describe` | 200 | Non |
| 74 | POST | `/api/vision/compare` | 200 | Non |
| 75 | GET | `/api/cases/{case_id}/visual-entities` | 200 | Non |
| 76 | POST | `/api/forensics/bpa/classify` | 200 | Non |
| 77 | POST | `/api/forensics/bpa/analyze` | 200 | Non |
| 78 | POST | `/api/forensics/bpa/calculate-angle` | 200 | Non |
| 79 | POST | `/api/forensics/bpa/convergence` | 200 | Non |
| 80 | POST | `/api/forensics/audio/transcribe` | 200 | Non |
| 81 | POST | `/api/forensics/audio/analyze` | 200 | Non |
| 82 | POST | `/api/forensics/audio/events` | 200 | Non |
| 83 | POST | `/api/forensics/audio/propagation` | 200 | Non |
| 84 | POST | `/api/forensics/trace/analyze` | 200 | Non |
| 85 | POST | `/api/forensics/trace/compare` | 200 | Non |
| 86 | POST | `/api/forensics/cases/{case_id}/auto` | 200 | Non |
| 87 | POST | `/api/forensics/sim/blood-drop` | 200 | Non |
| 88 | POST | `/api/forensics/sim/cast-off` | 200 | Non |
| 89 | POST | `/api/forensics/sim/sound` | 200 | Non |
| 90 | POST | `/api/forensics/sim/origin` | 200 | Non |
| 91 | GET | `/api/forensics/sim/datasets` | 200 | Non |
| 92 | GET | `/api/forensics/sim/datasets/{name}` | 200 | Non |
| 93 | GET | `/api/investigations` | 200 | Non |
| 94 | POST | `/api/cases/{case_id}/investigation/start` | 200 | Non |
| 95 | POST | `/api/cases/{case_id}/investigation/stop` | 200 | Non |
| 96 | GET | `/api/cases/{case_id}/investigation/status` | 200 | Non |
| 97 | GET | `/api/cases/{case_id}/investigation/log` | 200 | Non |
| 98 | GET | `/api/cases/{case_id}/audit` | 200 | Non |
| 99 | GET | `/api/cases/{case_id}/audit/summary` | 200 | Non |
| 100 | GET | `/api/cases/{case_id}/audit/timeline` | 200 | Non |
| 101 | GET | `/api/audit/{audit_id}` | 200 | Non |
| 102 | GET | `/api/cases/{case_id}/audit/verify` | 200 | Non |

**Total : 102 endpoints** repartis sur 19 routers + 1 endpoint health dans main.py.
