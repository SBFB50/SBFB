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
7. [Boucle autonome OODA](#7-boucle-autonome-ooda)
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
- **Boucle autonome d'investigation** basee sur le modele OODA etendu (Observe-Orient-Decide-Act-Question)

Le systeme est entierement local (aucune donnee ne quitte la machine) et utilise des modeles LLM uncensored/abliterated pour analyser du contenu sensible lie a des affaires criminelles.

### Stack technique

| Composant | Technologie | Port |
|-----------|-------------|------|
| Backend | FastAPI (Python 3.13) | 8000 |
| Frontend React | Vite + TypeScript + Tailwind | 3002 |
| Frontend legacy | Streamlit | 8501 |
| LLMs | Ollama (6 modeles) | 11434 |
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
  | EvidenceProcessor|    | AutonomousLoop    |    | API Routers (21)  |
  | (ingestion)      |    | (boucle OODA)     |    | (115+ endpoints)  |
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
|Analy-|  |Hypothesis|  |Contradic-|  |Suspect   |  |Forensics |  |Investigation|
|sis   |  |Engine    |  |tion      |  |Scorer    |  |(BPA,trace|  |Manager      |
|Pipe  |  |(gen+eval)|  |Detector  |  |(5 factors)|  |acoustic) |  |(daemon mgr) |
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

| Outil | Recoit de | Alimente |
|-------|-----------|----------|
| EvidenceProcessor | API upload, monitoring auto | EntityExtractor, ChromaDB, Neo4j, SummaryTree |
| EntityExtractor | EvidenceProcessor, AutonomousLoop | SQLite entities, Neo4j nodes, ChromaDB entity_contexts |
| AnalysisPipeline | AutonomousLoop, API | HypothesisEngine (re-scoring) |
| HypothesisEngine | AnalysisPipeline, AutonomousLoop | SQLite hypotheses, ChromaDB hypothesis_reasoning |
| ContradictionDetector | AutonomousLoop | AlertManager, audit_log, SuspectScorer |
| SuspectScorer | AutonomousLoop (DECIDE) | SQLite suspects, suspect_snapshots |
| Retriever | AnalysisPipeline, HypothesisEngine | Contexte LLM (prompt injection) |
| SummaryTree | EvidenceProcessor | SQLite summary_clusters, case_summaries |
| TimelineBuilder | AutonomousLoop (DECIDE) | SQLite analysis_runs |
| GeoMapper | AutonomousLoop (ORIENT) | SQLite locations |
| ImageAnalyzer | AutonomousLoop (ORIENT) | SQLite evidence, ChromaDB |
| ImageSearchEngine | AutonomousLoop (ORIENT) | ChromaDB image_dinov2, image_clip |
| MonitoringScheduler | AutonomousLoop (OBSERVE) | SQLite monitoring_results |

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
6. Resume LLM (gemma4:e4b)
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

**Synchronisation** : Depuis SQLite via `MERGE` (idempotent). Resync periodique tous les 3 cycles OODA.

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

### 6 modeles Ollama

| Modele | Taille | Role | VRAM | Exemples de taches |
|--------|--------|------|------|--------------------|
| `nexus` (Gemma 4 26B Heretic) | 26B Q4_K_S | Analyse profonde | ~14 GB | DEEP_ANALYSIS, HYPOTHESIS_SCORING, SUSPECT_PROFILE, FINAL_REPORT |
| `huihui_ai/deepseek-r1-abliterated:14b` | 14B | Raisonnement CoT | ~10 GB | CONTRADICTION_DETECTION, LOGIC_VERIFICATION, TESTIMONY_COMPARISON |
| `gemma4:e4b` | 4B | Leger, rapide | ~3 GB | ENTITY_EXTRACTION, EVIDENCE_SUMMARY, QUERY_REFORMULATION, IMAGE_DESCRIPTION |
| `nomic-embed-text` | 137M | Embeddings | ~0.5 GB | EMBEDDING (768 dim) |
| `qwen3-vl:8b` | 8B | Vision profonde | ~6 GB | IMAGE_SCENE_ANALYSIS, IMAGE_COMPARISON, TRACE_ANALYSIS |
| `voxtral-mini:4b` | 4B | Audio | ~3 GB | AUDIO_TRANSCRIPTION |

### Taxonomie des taches (20 TaskTypes)

```python
# Leger (gemma4:e4b, ~80 tok/s)
ENTITY_EXTRACTION, QUERY_REFORMULATION, RESULT_FILTERING,
JSON_STRUCTURING, EVIDENCE_SUMMARY

# Embeddings (nomic-embed-text)
EMBEDDING

# Raisonnement (deepseek-r1, CoT)
LOGIC_VERIFICATION, CONTRADICTION_DETECTION, TESTIMONY_COMPARISON

# Analyse profonde (nexus 26B)
DEEP_ANALYSIS, HYPOTHESIS_SCORING, SUSPECT_PROFILE,
FINAL_REPORT, INCREMENTAL_REEVAL

# Vision (gemma4:e4b rapide, qwen3-vl:8b profonde)
IMAGE_DESCRIPTION, IMAGE_ENTITY_EXTRACTION,
IMAGE_SCENE_ANALYSIS, IMAGE_COMPARISON

# Audio (voxtral-mini:4b)
AUDIO_TRANSCRIPTION

# Forensique (qwen3-vl:8b)
TRACE_ANALYSIS
```

### Serialisation VRAM

```
asyncio.Lock (_heavy_lock)
     |
     +-- Tout appel "heavy=True" acquiert le lock
     |   avant d'envoyer la requete a Ollama
     |
     +-- Modeles legers (gemma4:e4b, nomic) passent sans lock
     |
     +-- Warning log si attente > 30 secondes
```

**Fichier :** `nexus/llm/router.py`, lignes 116-200

**Principe** : Un seul modele lourd (>8B params) en VRAM a la fois. Les modeles legers coexistent. Le `asyncio.Lock` serialise les appels lourds pour eviter les OOM.

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
  1. Extraction d'entites de la query (match DB ou LLM gemma4:e4b)
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

## 7. Boucle autonome OODA

### Fichier : `nexus/core/autonomous_loop.py` (1600+ lignes)

La boucle tourne en continu pour chaque case active, geree par `InvestigationManager` (`nexus/core/investigation_manager.py`).

### 5 phases, 18 sous-etapes

```
OBSERVE (Phase 1)
  1a. Filtrer monitoring_results non-reviewed + relevance >= seuil

ORIENT (Phase 2)
  2a. Auto-ingestion des resultats monitoring -> EvidenceProcessor
  2b. Re-sync Neo4j periodique (tous les 3 cycles)
  2c. OSINT recon (holehe email + social username)
  2d. Geocodage des entites location (Nominatim)
  2e. Analyse VLM des images non-traitees (qwen3-vl)
  2f. Indexation DINOv2/CLIP des images

DECIDE (Phase 3)
  3a. Analyse incrementale des nouvelles preuves (AnalysisPipeline)
  3b. Generation/evaluation des hypotheses (HypothesisEngine)
  3c. Detection des contradictions (ContradictionDetector -> deepseek-r1)
  3d. Scoring des suspects (SuspectScorer, 5 facteurs)
  3e. Analyse forensique (BPA sang + traces + acoustique)
  3f. Reconstruction chronologique (TimelineBuilder)
  3g. Reconstruction arbre RAPTOR (tous les 3 cycles)

ACT (Phase 4)
  4a. Generation adaptative de queries de recherche (LLM)
  4b. Enrichissement OSINT (jobs monitoring depuis recon)
  4c. Domain recon WHOIS/DNS (emails non-freemail)

QUESTION (Phase 5)
  5a. Auto-questionnement adversarial (challenge hypothese top)
  5b. Generation de rapports periodiques
  5c. Backups automatises
```

### Cycle de vie

```
InvestigationManager.start_investigation(case_id)
    |
    v
AutonomousInvestigator(case_id, router, chroma, neo4j)
    |
    v
while running:
    async with get_db() as conn:
        OBSERVE -> ORIENT -> DECIDE -> ACT -> QUESTION
    sleep(investigation_cycle_minutes * 60)
    |
    v
InvestigationManager.stop_investigation(case_id)
```

**Resilience** : Chaque sous-etape est wrappee dans try/except. Un echec ne bloque pas le cycle. Apres une erreur de cycle, wait 5 minutes avant retry.

**Audit** : Chaque action est loguee via `AuditService` dans les 3 couches (SQLite hash chain, JSONL, Git).

**Tracking UI** : `_track_tool()` met a jour un dict `_tool_status` consulte par l'endpoint `/investigation/status`.

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
| Investigation | `/investigation` | Controle boucle autonome, statut des 13 outils |
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
| Boucle autonome | `nexus/core/autonomous_loop.py` |
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
