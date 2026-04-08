# NEXUS -- Architecture technique

**Version :** 0.2.0
**Date :** 2026-04-06
**Source :** Documentation generee par analyse exhaustive du code source (41K lignes)

---

## Table des matieres

1. [Vue d'ensemble](#1-vue-densemble)
2. [Diagramme de composants](#2-diagramme-de-composants)
3. [Pipeline d'ingestion des preuves](#3-pipeline-dingestion-des-preuves)
4. [Couche de stockage](#4-couche-de-stockage)
5. [Routage LLM et gestion VRAM](#5-routage-llm-et-gestion-vram)
6. [Retriever hybride 4 sources](#6-retriever-hybride-4-sources)
7. [Architecture event-driven reactive](#7-architecture-event-driven-reactive)
8. [Scoring des suspects](#8-scoring-des-suspects)
9. [Frontend React](#9-frontend-react)

---

## 1. Vue d'ensemble

### Description

NEXUS est un systeme d'investigation persistant et autonome concu pour les cold cases. Contrairement a un chatbot one-shot, NEXUS accumule de l'intelligence sur des semaines ou des mois en combinant :

- **Ingestion multi-format** de preuves (PDF, texte, images, audio)
- **Extraction automatique d'entites** via GLiNER (CPU, zero VRAM)
- **Monitoring continu** multi-sources (clearweb via SearXNG + dark web via Robin/Tor)
- **Evaluation d'hypotheses evolutives** avec scoring multi-modeles
- **Architecture event-driven reactive** avec EventBus pub/sub + 20 ReactiveWorkers + VRAMScheduler

Le systeme est entierement local (aucune donnee ne quitte la machine) et utilise des modeles LLM uncensored/abliterated pour analyser du contenu sensible lie a des affaires criminelles.

### Stack technique

| Composant | Technologie | Port |
|-----------|-------------|------|
| Backend | FastAPI (Python 3.13) | 8000 |
| Frontend React | Vite + TypeScript + Tailwind | 3002 |
| LLMs | Ollama (1 modele + embeddings) | 11434 |
| Recherche clearweb | SearXNG | 8888 |
| Recherche dark web | Robin (Tor) | 8502 |
| Graphe | Neo4j | 7474 |
| Vecteurs | ChromaDB | 8100 |
| Base relationnelle | SQLite (FTS5 + WAL) | - |
| NER | GLiNER (CPU) | - |

### Contrainte materielle

- **GPU** : RTX 5080 16 GB VRAM, partagee entre tous les modeles
- **Serialisation** : Un seul modele lourd (>8B params) en VRAM a la fois via `asyncio.Lock`
- **Debit** : ~20.4 tok/s sur nexus 26B Q4_K_S

---

## 2. Diagramme de composants

### Les 13 outils principaux

```
                           +-------------------+
                           |   FastAPI Backend  |
                           |   (nexus/main.py)  |
                           +--------+----------+
                                    |
           +------------------------+------------------------+
           |                        |                        |
  +--------v--------+    +---------v---------+    +---------v---------+
  | EvidenceProcessor|    | EventBus +        |    | API Routers (22)  |
  | (ingestion)      |    | 20 ReactiveWorkers|    | (133+ endpoints)  |
  +---------+--------+    +----+----+----+----+    +-------------------+
            |                  |    |    |    |
   +--------+--------+        |    |    |    |
   |  EntityExtractor |        |    |    |    |
   |  (GLiNER+Fuzzy)  |        |    |    |    |
   +------------------+        |    |    |    |
                               |    |    |    |
   +---------------------------+    |    |    +----------------------------+
   |                                |    |                                 |
   v                                v    v                                 v
+--+---+  +----------+  +----------+  +----------+  +----------+  +------+-----+
|Analy-|  |Hypothesis|  |Contradic-|  |Suspect   |  |Forensics |  |ReactiveInv. |
|sis   |  |Engine    |  |tion      |  |Scorer    |  |(BPA,trace|  |Manager      |
|Pipe  |  |(gen+eval)|  |Detector  |  |(5 factors)|  |acoustic) |  |(event mgr)  |
+--+---+  +----+-----+  +----+-----+  +----+-----+  +----+-----+  +------+-----+
   |           |              |             |              |               |
   +-----+----+----+---------+------+------+--------------+               |
         |         |                |                                      |
   +-----v---+ +---v------+ +------v------+                               |
   |Retriever| |Summary   | |Timeline     |                               |
   |(4-source | |Tree      | |Builder      |                               |
   | hybrid)  | |(RAPTOR)  | |             |                               |
   +----+----+ +----+-----+ +------+------+                               |
        |           |               |                                      |
   +----v-----------v---------------v--------------------------------------v---+
   |                        COUCHE DE STOCKAGE                                  |
   |  +----------+    +----------+    +-----------+    +------------------+     |
   |  | SQLite   |    | Neo4j    |    | ChromaDB  |    | Audit (3 layers)|     |
   |  | (15 tab) |    | (graphe) |    | (7 coll.) |    | SQLite+JSONL+Git|     |
   |  +----------+    +----------+    +-----------+    +------------------+     |
   +----------------------------------------------------------------------------+
```

### Interconnexions cles

| Outil | Recoit de (event) | Alimente |
|-------|-------------------|----------|
| EvidenceProcessor | API upload, monitoring_result | EntityExtractor, ChromaDB, Neo4j, SummaryTree |
| EntityExtractor | evidence_added | SQLite entities, Neo4j nodes, ChromaDB entity_contexts |
| AnalysisPipeline | evidence_processed | HypothesisEngine (re-scoring) |
| HypothesisEngine | analysis_completed | SQLite hypotheses, ChromaDB hypothesis_reasoning |
| ContradictionDetector | evidence_processed | AlertManager, audit_log, SuspectScorer |
| SuspectScorer | hypothesis_scored | SQLite suspects, suspect_snapshots |
| Retriever | AnalysisPipeline, HypothesisEngine | Contexte LLM (prompt injection) |
| SummaryTree | evidence_added | SQLite summary_clusters, case_summaries |
| TimelineBuilder | evidence_processed | SQLite analysis_runs |
| GeoMapper | entity_discovered | SQLite locations |
| ImageAnalyzer | evidence_added (image) | SQLite evidence, ChromaDB |
| ImageSearchEngine | evidence_added (image) | ChromaDB image_dinov2, image_clip |
| MonitoringLoop | MonitoringLoop (30s sweep) | SQLite monitoring_results |

---

## 3. Pipeline d'ingestion des preuves

L'ingestion complete d'une preuve passe par 11 etapes (voir `docs/PIPELINE.md` pour le detail) :

```
Fichier/Texte
     |
     v
1. Nettoyage texte (TextParser/PDFParser)
     |
     v
2. Hash SHA-256 (dedup + integrite)
     |
     v
3. INSERT SQLite (status='pending')
     |
     v
4. UPDATE status='processing'
     |
     +---> [BRANCHE IMAGE] --> ImageAnalyzer (VLM)
     |                              |
     v                              v
5. GLiNER NER + RapidFuzz dedup  (retour au flux commun)
   + ChromaDB entity_contexts
     |
     v
6. Resume LLM (gemma-4-26B-A4B)
     |
     v
7. UPDATE status='processed'
     |
     v
8. Neo4j sync (entites mentionnees uniquement)
   + extraction relations LLM
     |
     v
9. Chunk (512 tokens, overlap 128) + embed (nomic-embed-text)
   -> ChromaDB evidence_chunks
     |
     v
10. RAPTOR summary tree (mise a jour incrementale)
     |
     v
11. Audit log (3 couches: SQLite hash chain + JSONL + Git)
```

**Fichier source :** `nexus/core/evidence_processor.py` (lignes 89-277)

**Gestion d'erreur :** Toute exception en etapes 5-11 est non-fatale et loguee. L'evidence est marquee `status='error'` uniquement si l'ensemble de la pipeline echoue (catch global ligne 272).

---

## 4. Couche de stockage

### 4.1 SQLite (16 tables, 23 index, FTS5)

**Fichier :** `nexus/db/sqlite_db.py`

**Mode :** WAL (Write-Ahead Logging) pour concurrence lecture/ecriture.

#### Tables

| Table | Description | Cles etrangeres |
|-------|-------------|-----------------|
| `cases` | Dossiers d'investigation | - |
| `evidence` | Preuves (texte, pdf, image, audio) | `case_id` -> cases |
| `entities` | Entites extraites (personne, lieu, vehicule...) | `case_id` -> cases |
| `entity_mentions` | Liens entite <-> preuve avec confiance | `entity_id`, `evidence_id` |
| `hypotheses` | Hypotheses evolutives avec score | `case_id` -> cases |
| `hypothesis_snapshots` | Historique des scores | `hypothesis_id` -> hypotheses |
| `analysis_runs` | Historique des analyses LLM | `case_id` -> cases |
| `monitoring_jobs` | Jobs de surveillance (SearXNG/Robin) | `case_id`, `entity_id` |
| `monitoring_results` | Resultats de surveillance | `job_id`, `case_id` |
| `alerts` | Alertes systeme | `case_id` -> cases |
| `reports` | Rapports generes (PDF) | `case_id` -> cases |
| `locations` | Lieux geocodes (lat/lon) | `case_id`, `entity_id` |
| `audit_log` | Journal d'audit avec hash chain | `case_id` -> cases |
| `summary_clusters` | Clusters RAPTOR niveau 1 | `case_id` -> cases |
| `suspects` | Suspects avec 5 sous-scores | `case_id`, `entity_id` (UNIQUE) |
| `suspect_snapshots` | Historique des scores suspects | `suspect_id` -> suspects |
| `case_summaries` | Resume global par dossier (RAPTOR L2) | `case_id` (UNIQUE) |

#### Index (23)

- 16 index simples sur les colonnes `case_id`, `hypothesis_id`, `suspect_id`, etc.
- 7 index composites pour les requetes frequentes :
  - `idx_evidence_case_type` : evidence(case_id, evidence_type)
  - `idx_evidence_case_status` : evidence(case_id, status)
  - `idx_entities_case_type` : entities(case_id, entity_type)
  - `idx_mentions_evidence` : entity_mentions(evidence_id)
  - `idx_mentions_entity` : entity_mentions(entity_id)
  - `idx_monitoring_results_job` : monitoring_results(job_id)
  - `idx_monitoring_results_case` : monitoring_results(case_id)

#### FTS5 (Full-Text Search)

Table virtuelle `evidence_fts` sur les colonnes `title`, `raw_text`, `summary`, `source`.

Synchronisation automatique via 3 triggers SQLite (INSERT, UPDATE, DELETE).

Expose dans le retriever hybride (poids 0.15) et les endpoints `/fts` et `/hybrid`.

### 4.2 Neo4j (graphe de connaissances)

**Fichier :** `nexus/db/neo4j_db.py`

**Labels de noeuds (11)** : Person, Location, Phone, Vehicle, Organization, Account, Event, Evidence, Money, Hypothesis, Case

**Types de relations (20)** : KNOWS, RELATED_TO, COMMUNICATED_WITH, FINANCIAL_LINK, SENT_MONEY, RECEIVED_MONEY, LIVES_AT, WAS_AT, WORKS_AT, FREQUENTS, OWNS, MEMBER_OF, OCCURRED_AT, INVOLVES, PRECEDED_BY, MENTIONS, SUPPORTS, CONTRADICTS, TRANSACTION, BELONGS_TO

**Methodes principales** : sync_entity, sync_evidence, link_evidence_to_entity, get_neighbors, find_shortest_path, get_central_entities, get_entity_importance, detect_communities, get_temporal_graph, get_evidence_matrix

**Synchronisation** : Depuis SQLite via `MERGE` (idempotent). Sync reactive via `entity_discovered` events.

### 4.3 ChromaDB (7 collections vectorielles)

**Fichier :** `nexus/db/chroma_db.py`

| Collection | Source | Usage |
|------------|--------|-------|
| `evidence_chunks` | EmbeddingStore | RAG principal (chunks 512 tokens) |
| `entity_contexts` | EvidenceProcessor | Contexte des entites extraites |
| `monitoring_results` | MonitoringScheduler | Dedup OSINT |
| `hypothesis_reasoning` | HypothesisEngine | Snapshots d'hypotheses |
| `image_dinov2` | ImageSearchEngine | Similarite image-image |
| `image_clip` | ImageSearchEngine | Recherche texte-image |
| `evidence_texts` | DEPRECATED | Plus d'ecriture |

**Recherche unifiee** : `unified_search()` interroge plusieurs collections en une seule passe, distribuant N/collections resultats par collection.

**Embeddings** : Pre-calcules par Ollama (`nomic-embed-text`, 768 dimensions) via le LLMRouter. ChromaDB est utilise uniquement comme store vectoriel (pas de fonction d'embedding interne).

---

## 5. Routage LLM et gestion VRAM

### Fichier : `nexus/llm/router.py`

### 2 modeles Ollama (single model + embeddings)

| Modele | Taille | Role | VRAM | Exemples de taches |
|--------|--------|------|------|--------------------|
| `juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m` | MoE 26B (4B actifs) | ALL tasks | ~14 GB | DEEP_ANALYSIS, HYPOTHESIS_SCORING, CONTRADICTION_DETECTION, EVIDENCE_SUMMARY, IMAGE_DESCRIPTION, RESULT_FILTERING, etc. |
| `nomic-embed-text` | 137M | Embeddings | ~0.5 GB | EMBEDDING (768 dim), coexiste via bypass |

### Taxonomie des taches (20 TaskTypes)

```python
# Light tasks (gemma-4-26B-A4B, rapides grace a MoE 4B actifs)
ENTITY_EXTRACTION, QUERY_REFORMULATION, RESULT_FILTERING,
JSON_STRUCTURING, EVIDENCE_SUMMARY

# Embeddings (nomic-embed-text, bypass VRAM lock)
EMBEDDING

# Heavy tasks (gemma-4-26B-A4B, single model for all)
LOGIC_VERIFICATION, CONTRADICTION_DETECTION, TESTIMONY_COMPARISON,
DEEP_ANALYSIS, HYPOTHESIS_SCORING, SUSPECT_PROFILE,
FINAL_REPORT, INCREMENTAL_REEVAL

# Vision (gemma-4-26B-A4B, native multimodal)
IMAGE_DESCRIPTION, IMAGE_ENTITY_EXTRACTION,
IMAGE_SCENE_ANALYSIS, IMAGE_COMPARISON

# Audio
AUDIO_TRANSCRIPTION

# Forensique
TRACE_ANALYSIS
```

### Serialisation VRAM (VRAMScheduler)

```
VRAMScheduler (nexus/events/vram_scheduler.py)
     |
     +-- embedding bypass : nomic-embed-text passe sans lock
     |
     +-- light lock : taches legeres serialisees entre elles
     |
     +-- heavy lock : taches lourdes serialisees (priority queue)
     |
     +-- light/heavy mutual exclusion : prevent GPU swap
     |
     +-- model affinity batching : regroupe les taches du meme modele
```

**Fichiers :** `nexus/events/vram_scheduler.py`, `nexus/llm/router.py`

**Principe** : Un seul modele (gemma-4-26B-A4B) pour toutes les taches. Le VRAMScheduler gere l'exclusion mutuelle light/heavy et le batching par affinite modele. `nomic-embed-text` (137MB) coexiste via bypass.

---

## 6. Retriever hybride 4 sources

### Fichier : `nexus/core/retriever.py`

Le retriever combine 4 strategies de recuperation en un score composite unique :

```
composite = semantic * 0.50 + graph * 0.25 + fts * 0.15 + recency * 0.10
```

### Source 1 : Semantique (poids 0.50)

- **Backend** : ChromaDB `unified_search()` cross-collection
- **Collections** : evidence_chunks, entity_contexts, monitoring_results, (hypothesis_reasoning optionnel)
- **Score** : `1.0 - cosine_distance` normalise [0, 1]
- **Fallback** : EmbeddingStore single-collection si unified_search echoue

### Source 2 : Graphe (poids 0.25)

- **Backend** : Neo4j traversal
- **Processus** :
  1. Extraction d'entites de la query (match DB ou LLM gemma-4-26B-A4B)
  2. Match des noms sur les noeuds Neo4j (exact + partiel)
  3. Traversal 1-2 hops pour trouver les noeuds Evidence
  4. Chargement du texte depuis SQLite
- **Boosts** : centralite (+0.3 max), bridge entities (+0.15), multi-entity (+0.1 par entite)
- **Score** : 1.0 pour 1-hop, 0.5 pour 2-hop, + bonus

### Source 3 : FTS5 (poids 0.15)

- **Backend** : SQLite FTS5 BM25
- **Role** : Capture les correspondances exactes que le semantique rate (noms propres, dates, numeros de dossier)
- **Sanitisation** : Chaque mot wrape en guillemets, operateurs FTS5 filtres
- **Score** : Decroissant par position dans le ranking BM25

### Source 4 : Recency (poids 0.10)

- **Horizon** : 30 jours
- **Score** : Decroissance lineaire de 1.0 (aujourd'hui) a 0.0 (30+ jours)

### Deduplication et reranking

1. Cle de dedup : `evidence_id + chunk_text[:200]`
2. Si doublon : merge des scores (max de chaque composante), source = "hybrid"
3. Tri descendant par score composite
4. Top N resultats retournes

---

## 7. Architecture event-driven reactive

### Fichiers : `nexus/events/` (bus.py, worker.py, manager.py, vram_scheduler.py, monitoring_loop.py, timer.py)

L'ancienne boucle OODA monolithique (`autonomous_loop.py`) a ete remplacee par une architecture event-driven reactive. Chaque outil reagit immediatement aux changements via des evenements, sans cycles fixes.

### EventBus pub/sub

Le `EventBus` (`nexus/events/bus.py`) est le coeur du systeme :
- Publication d'evenements typos (20 `EventType` dans `nexus/events/types.py`)
- Souscription par les workers via pattern matching sur le type d'evenement
- Persistance SQLite dans `event_log` pour replay et audit
- Circuit breaker integre pour isoler les workers defaillants

### 20 ReactiveWorkers (4 categories)

```
INGEST (ingestion)
  - EvidenceIngestWorker : monitoring_result -> evidence_added
  - EntityExtractorWorker : evidence_added -> entity_discovered
  - SummarizerWorker : evidence_added -> evidence_processed

ENRICH (enrichissement)
  - ChunkerEmbedWorker : evidence_processed -> embeddings indexes
  - Neo4jSyncWorker : entity_discovered -> graphe mis a jour
  - GeoMapperWorker : entity_discovered -> locations geocodees
  - OSINTReconWorker : entity_discovered -> monitoring jobs
  - QueryGeneratorWorker : entity_discovered -> queries de recherche
  - ImageAnalyzerWorker : evidence_added (image) -> description + entites

ANALYZE (analyse)
  - AnalysisPipelineWorker : evidence_processed -> analysis_completed
  - ContradictionWorker : evidence_processed -> contradictions detectees
  - HypothesisWorker : analysis_completed -> hypothesis_scored
  - SuspectScorerWorker : hypothesis_scored -> suspects scores

SCORE (evaluation)
  - TimelineWorker, ForensicsWorker, RAPTORWorker, etc.
```

### Flux d'evenements

```
evidence_added -> EntityExtractor + Summarizer (parallel)
  -> entity_discovered -> Neo4j + GeoMapper + OSINT Recon + QueryGenerator
  -> evidence_processed -> ChunkerEmbed + ContradictionDetector + AnalysisPipeline
  -> analysis_completed -> HypothesisWorker -> hypothesis_scored -> SuspectScorer
  -> monitoring_result -> EvidenceIngestWorker -> evidence_added (BOUCLE)
```

### MonitoringLoop

Le `MonitoringLoop` (`nexus/events/monitoring_loop.py`) remplace APScheduler :
- Sweep continu toutes les 30 secondes
- Execute les jobs de surveillance (SearXNG, Robin, Wayback) pour chaque case active
- Publie `monitoring_result` dans l'EventBus

### VRAMScheduler

Le `VRAMScheduler` (`nexus/events/vram_scheduler.py`) orchestre l'acces GPU :
- Embedding bypass : `nomic-embed-text` passe sans lock (137MB, coexiste)
- Light lock + Heavy lock : exclusion mutuelle pour eviter le swap GPU
- Priority queue : les taches lourdes sont priorisees
- Model affinity batching : regroupe les taches consecutives du meme modele

### Server-Sent Events (SSE)

Deux endpoints SSE permettent au frontend de recevoir les evenements en temps reel :
- `GET /api/cases/{case_id}/events` : evenements d'une investigation specifique
- `GET /api/system/events` : evenements systeme globaux

### Cycle de vie

```
ReactiveInvestigationManager.start_investigation(case_id)
    |
    v
EventBus.publish(investigation_started, case_id)
    |
    v
20 ReactiveWorkers ecoutent et reagissent aux evenements
MonitoringLoop sweep toutes les 30s
    |
    v
ReactiveInvestigationManager.stop_investigation(case_id)
```

**Resilience** : Chaque worker est isole par un circuit breaker. Un echec d'un worker n'affecte pas les autres. Les evenements non traites sont persistes dans SQLite pour replay.

**Audit** : Chaque evenement est persiste dans `event_log` (SQLite) avec timestamp et metadata.

**Tracking UI** : `PipelineTools.tsx` affiche le statut en temps reel des 20 workers (INGEST/ENRICH/ANALYZE/SCORE) via SSE.

---

## 8. Scoring des suspects

### Fichier : `nexus/core/suspect_scorer.py`

Chaque entite de type "person" dans un dossier est evaluee sur 5 facteurs ponderes :

```
suspicion_score = G * 0.20 + E * 0.25 + C * 0.20 + P * 0.20 + H * 0.15
```

### Les 5 facteurs

| Facteur | Poids | Source | Calcul |
|---------|-------|--------|--------|
| **Graph (G)** | 20% | Neo4j | Centralite de degre (0-50 pts) + proximite a la victime (0-50 pts) |
| **Evidence (E)** | 25% | SQLite mentions | `sum(confidence * reliability)` normalise (5 mentions parfaites = 100) |
| **Contradiction (C)** | 20% | audit_log | 1 contradiction = 40, 2 = 70, 3+ = 100 |
| **Profile (P)** | 20% | LLM (nexus 26B) | mobile (0-30) + alibi (0-40) + dangerosite (0-30) |
| **Hypothesis (H)** | 15% | SQLite hypotheses | Moyenne des scores des hypotheses mentionnant le suspect |

### Evaluation du profil (LLM)

L'evaluation du profil (`evaluate_profile()`) envoie au LLM nexus 26B :
- Toutes les preuves mentionnant le suspect (titre, resume, contexte)
- La relation avec la victime
- Le prompt `SUSPECT_PROFILE_PROMPT`

Le LLM retourne un JSON avec : `mobile_score`, `alibi_score`, `danger_score`, `alibi_status`, `mobile_description`, `reasoning`.

### Snapshots

Chaque scoring cree un `suspect_snapshot` pour suivre l'evolution temporelle. Les snapshots sont interrogeables via `get_evolution()` pour les graphiques de l'UI.

---

## 9. Frontend React

### Repertoire : `web/src/`

### 9 pages

| Page | Route | Description |
|------|-------|-------------|
| Dashboard | `/` | Vue d'ensemble (cases, alertes, stats) |
| Evidence | `/evidence` | Liste, upload, detail des preuves |
| Entities | `/entities` | Entites extraites, mentions, types |
| Hypotheses | `/hypotheses` | Hypotheses, scores, evolution graphique |
| Graph | `/graph` | Graphe Neo4j interactif (noeuds + aretes) |
| Timeline | `/timeline` | Chronologie des evenements |
| Investigation | `/investigation` | Controle investigation, statut des 20 workers reactifs |
| Benchmark | `/benchmark` | Lancement et suivi des benchmarks |
| Suspects | `/suspects` | Classement, 5 facteurs, evolution |

### Stack frontend

- **Build** : Vite
- **Framework** : React + TypeScript
- **Style** : Tailwind CSS, dark theme professionnel
- **Graphiques** : Recharts (evolution scores)
- **Graphe** : react-force-graph ou cytoscape (Neo4j)
- **Communication** : fetch vers FastAPI port 8000

---

## Annexe : fichiers de reference

| Module | Fichier |
|--------|---------|
| Backend entrypoint | `nexus/main.py` |
| Configuration | `nexus/config.py` |
| Event manager | `nexus/events/manager.py` |
| EventBus | `nexus/events/bus.py` |
| ReactiveWorker ABC | `nexus/events/worker.py` |
| VRAMScheduler | `nexus/events/vram_scheduler.py` |
| MonitoringLoop | `nexus/events/monitoring_loop.py` |
| Evidence processor | `nexus/core/evidence_processor.py` |
| Entity extractor | `nexus/core/entity_extractor.py` |
| Hypothesis engine | `nexus/core/hypothesis_engine.py` |
| Contradiction detector | `nexus/core/contradiction_detector.py` |
| Suspect scorer | `nexus/core/suspect_scorer.py` |
| Analysis pipeline | `nexus/core/analysis_pipeline.py` |
| Retriever hybride | `nexus/core/retriever.py` |
| Summary tree RAPTOR | `nexus/core/summary_tree.py` |
| Chunker | `nexus/core/chunker.py` |
| Embedding store | `nexus/core/embedding_store.py` |
| Image analyzer | `nexus/core/image_analyzer.py` |
| Geo mapper | `nexus/core/geo_mapper.py` |
| Timeline builder | `nexus/core/timeline_builder.py` |
| Audit service | `nexus/core/audit.py` |
| LLM router | `nexus/llm/router.py` |
| Ollama client | `nexus/llm/ollama_client.py` |
| Prompts (25+) | `nexus/llm/prompts.py` |
| Parsers JSON | `nexus/llm/parsers.py` |
| SQLite | `nexus/db/sqlite_db.py` |
| Neo4j | `nexus/db/neo4j_db.py` |
| ChromaDB | `nexus/db/chroma_db.py` |
| Models Pydantic | `nexus/db/models.py` |
