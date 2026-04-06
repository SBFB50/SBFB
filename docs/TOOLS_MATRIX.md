# NEXUS -- Matrice d'interconnexion des 13 outils

**Version :** 0.2.0
**Date :** 2026-04-06

---

## Table des matieres

1. [Les 13 outils](#1-les-13-outils)
2. [Fiches detaillees](#2-fiches-detaillees)
3. [Matrice de connexion](#3-matrice-de-connexion)
4. [Boucles de retroaction](#4-boucles-de-retroaction)

---

## 1. Les 13 outils

| # | Outil | Fichier source | Phase OODA |
|---|-------|---------------|------------|
| 1 | EvidenceProcessor | `nexus/core/evidence_processor.py` | ORIENT |
| 2 | EntityExtractor | `nexus/core/entity_extractor.py` | ORIENT |
| 3 | AnalysisPipeline | `nexus/core/analysis_pipeline.py` | DECIDE |
| 4 | HypothesisEngine | `nexus/core/hypothesis_engine.py` | DECIDE |
| 5 | ContradictionDetector | `nexus/core/contradiction_detector.py` | DECIDE |
| 6 | SuspectScorer | `nexus/core/suspect_scorer.py` | DECIDE |
| 7 | InvestigationRetriever | `nexus/core/retriever.py` | DECIDE |
| 8 | SummaryTree | `nexus/core/summary_tree.py` | DECIDE |
| 9 | GeoMapper | `nexus/core/geo_mapper.py` | ORIENT |
| 10 | ImageAnalyzer | `nexus/core/image_analyzer.py` | ORIENT |
| 11 | MonitoringScheduler | `nexus/monitoring/scheduler.py` | OBSERVE |
| 12 | ForensicAnalyzers | `nexus/forensics/` (BPA, trace, acoustic) | DECIDE |
| 13 | TimelineBuilder | `nexus/core/timeline_builder.py` | DECIDE |

---

## 2. Fiches detaillees

### 2.1 EvidenceProcessor

**Fonction :** Pipeline complete d'ingestion des preuves (fichier ou texte) -- du parsing a l'indexation.

**Fichier :** `nexus/core/evidence_processor.py`

**Entrees :**
- Fichier upload (API `/api/cases/{id}/evidence/upload`)
- Texte brut (API `/api/cases/{id}/evidence/text`)
- Resultats monitoring (boucle autonome `_orient_ingest`)

**Sorties :**
- SQLite : `evidence` (row complete), `entities`, `entity_mentions`
- Neo4j : noeuds Evidence + Entity + liens MENTIONS + relations inter-entites
- ChromaDB : `evidence_chunks` (RAG), `entity_contexts` (entites)
- Audit : 3 couches (SQLite hash chain, JSONL, Git)

**Stockage :** SQLite evidence, Neo4j, ChromaDB evidence_chunks + entity_contexts, audit_log

**Boucles de retroaction :**
- Alimente le Retriever (via evidence_chunks)
- Declenche SummaryTree.update_for_new_evidence()
- Les entites extraites alimentent le SuspectScorer (via mentions)

---

### 2.2 EntityExtractor

**Fonction :** Extraction hybride d'entites nommees (GLiNER CPU + LLM fallback) avec deduplication RapidFuzz.

**Fichier :** `nexus/core/entity_extractor.py`

**Entrees :**
- Texte brut depuis EvidenceProcessor
- Images depuis ImageAnalyzer (via LLM)

**Sorties :**
- Liste d'entites brutes : `[{name, type, context, confidence}]`
- Entites dedupliquees apres comparaison avec existants
- Relations inter-entites (extraction LLM optionnelle)

**Stockage :** SQLite entities + entity_mentions, ChromaDB entity_contexts

**Boucles de retroaction :**
- Nouvelles entites declenchent OSINT recon dans la boucle ORIENT
- Entites email -> holehe + social recon
- Entites location -> geocodage automatique (GeoMapper)
- Entites person -> scoring suspect automatique (SuspectScorer)

**Detail technique :**
- GLiNER : `urchade/gliner_multi-v2.1`, CPU, 205M params, ~0.08s/texte
- 17 labels en francais mappes vers 12 entity_types
- Dedup : RapidFuzz `WRatio` (Jaro-Winkler), seuil 82%

---

### 2.3 AnalysisPipeline

**Fonction :** Orchestration sequentielle de l'analyse multi-modeles d'un dossier.

**Fichier :** `nexus/core/analysis_pipeline.py`

**Entrees :**
- Contexte RAG depuis le Retriever (ou fallback legacy)
- Hypotheses existantes depuis SQLite
- Declencheur : boucle autonome ou API manuelle

**Sorties :**
- Resume d'analyse profonde (nexus 26B)
- Scores mis a jour des hypotheses (nexus 26B)
- Verification logique (deepseek-r1 14B)
- Alertes si delta score > 15 points

**Stockage :** SQLite analysis_runs, hypothesis_snapshots, alerts

**Boucles de retroaction :**
- Les scores d'hypotheses mis a jour influencent le SuspectScorer (facteur H)
- Les alertes de score shift declenchent des notifications UI
- L'analyse incrementale sur nouvelles preuves utilise le Retriever focalise

**Pipeline sequentielle :**
1. gemma4:e4b -- resumer les preuves non-resumees
2. Retriever -- construire le contexte RAG
3. nexus 26B -- analyse profonde
4. nexus 26B -- re-scorer chaque hypothese (RAG par hypothese)
5. deepseek-r1 -- verification logique
6. Sauvegarder + alertes

---

### 2.4 HypothesisEngine

**Fonction :** Generation, evaluation et re-evaluation des hypotheses d'investigation.

**Fichier :** `nexus/core/hypothesis_engine.py`

**Entrees :**
- Contexte RAG (via Retriever, incluant hypothesis_reasoning pour cross-reference)
- Preuves et entites du dossier
- Hypotheses existantes (pour re-evaluation)

**Sorties :**
- Nouvelles hypotheses (titre, description, score initial)
- Snapshots de scoring (score, supporting, contradicting, reasoning)
- Deduplication des hypotheses par RapidFuzz

**Stockage :** SQLite hypotheses + hypothesis_snapshots, ChromaDB hypothesis_reasoning

**Boucles de retroaction :**
- Les hypotheses influencent les queries de recherche generees (ACT)
- Les hypotheses alimentent le SuspectScorer (facteur H)
- Le self-questioning (QUESTION) challenge l'hypothese top
- Le Retriever utilise les hypotheses pour focaliser la recherche

**Modeles utilises :**
1. nexus 26B : generation et scoring
2. deepseek-r1 14B : verification logique et ajustement

---

### 2.5 ContradictionDetector

**Fonction :** Detection des contradictions entre preuves, temoignages et hypotheses.

**Fichier :** `nexus/core/contradiction_detector.py`

**Entrees :**
- Paires de preuves partageant des entites communes
- Fallback : comparaison de toutes les paires (plafond 15)

**Sorties :**
- Liste de contradictions : `[{description, evidence_ids, severity}]`
- Alertes via AlertManager

**Stockage :** SQLite audit_log (action='contradiction_found'), alerts

**Boucles de retroaction :**
- Les contradictions alimentent le SuspectScorer (facteur C)
- Les contradictions informent les queries de recherche (ACT)
- Les contradictions declenchent des alertes UI

**Modele :** deepseek-r1 14B (raisonnement CoT, `CONTRADICTION_DETECTION_PROMPT`)

**Optimisation combinatoire :** Seules les paires d'evidence partageant au moins une entite commune sont comparees, evitant O(n^2) sur tout le dossier.

---

### 2.6 SuspectScorer

**Fonction :** Calcul d'un score composite de suspicion pour chaque entite "person" du dossier.

**Fichier :** `nexus/core/suspect_scorer.py`

**Entrees :**
- Neo4j : centralite et proximite (graph_score)
- SQLite entity_mentions : frequence et confiance (evidence_score)
- SQLite audit_log : contradictions mentionnant le suspect (contradiction_score)
- LLM nexus 26B : evaluation profil (profile_score)
- SQLite hypotheses : score des hypotheses mentionnant le suspect (hypothesis_score)

**Sorties :**
- Score composite = G*0.20 + E*0.25 + C*0.20 + P*0.20 + H*0.15
- Breakdown par facteur
- Snapshots historiques pour evolution temporelle

**Stockage :** SQLite suspects + suspect_snapshots, audit_log

**Boucles de retroaction :**
- L'evaluation de profil (LLM) est periodique (tous les N cycles)
- Les snapshots permettent de suivre l'evolution du score au fil des cycles
- Un suspect avec score eleve peut declencher des queries de recherche ciblees (ACT)

---

### 2.7 InvestigationRetriever

**Fonction :** Recuperation de contexte pertinent pour les analyses LLM via recherche hybride 4 sources.

**Fichier :** `nexus/core/retriever.py`

**Entrees :**
- Query texte (description du cas, hypothese, question)
- ChromaDB (semantique) : evidence_chunks, entity_contexts, monitoring_results, hypothesis_reasoning
- Neo4j (graphe) : traversal des entites vers les preuves
- SQLite FTS5 (lexical) : correspondance exacte de mots-cles
- Metadonnees temporelles (recency)

**Sorties :**
- Liste triee de chunks pertinents avec score composite
- Contexte structure pour injection dans les prompts LLM
- `build_analysis_context()` : bloc texte pret pour le LLM (budgetise en tokens)

**Stockage :** Aucun (lecteur pur)

**Boucles de retroaction :**
- Alimente AnalysisPipeline, HypothesisEngine, AutonomousLoop (self-questioning)
- La qualite du retrieval impacte directement la qualite des analyses LLM

---

### 2.8 SummaryTree

**Fonction :** Arbre de resumes hierarchiques RAPTOR (preuve -> cluster -> case).

**Fichier :** `nexus/core/summary_tree.py`

**Entrees :**
- Resumes de preuves (SQLite evidence.summary)
- Embeddings (nomic-embed-text via LLMRouter)
- Declencheur : fin de pipeline d'ingestion, rebuild periodique tous les 3 cycles

**Sorties :**
- Clusters thematiques (SQLite summary_clusters)
- Resume global du dossier (SQLite case_summaries)

**Stockage :** SQLite summary_clusters, case_summaries

**Boucles de retroaction :**
- Le resume global du case enrichit le contexte d'analyse
- Reconstruction complete tous les 3 cycles (clustering agglomeratif scipy)

---

### 2.9 GeoMapper

**Fonction :** Geocodage des lieux, calcul d'itineraires, verification de temps de trajet.

**Fichier :** `nexus/core/geo_mapper.py`

**Entrees :**
- Entites de type "location" depuis SQLite
- Requetes de l'API geo (adresses, coordonnees)

**Sorties :**
- Coordonnees GPS (lat, lon) via Nominatim/OSM
- Itineraires et temps de trajet via OSRM
- Verifications de compatibilite temporelle (alibi)

**Stockage :** SQLite locations (lat, lon, address, location_type)

**Boucles de retroaction :**
- Les lieux geocodes apparaissent sur la carte dans l'UI
- La verification de trajet peut alimenter le SuspectScorer (alibi)

---

### 2.10 ImageAnalyzer

**Fonction :** Pipeline visuelle complete pour les preuves images.

**Fichier :** `nexus/core/image_analyzer.py`

**Entrees :**
- Fichiers image (JPEG, PNG, GIF, WebP, TIFF)
- Declencheur : upload image, boucle autonome ORIENT

**Sorties :**
- Description textuelle (gemma4:e4b rapide)
- Entites visuelles extraites (gemma4:e4b)
- Analyse de scene detaillee (qwen3-vl:8b profond)
- Embeddings texte de la description (nomic-embed-text)

**Stockage :** SQLite evidence (raw_text, summary), ChromaDB evidence_chunks, entities

**Boucles de retroaction :**
- La description devient le raw_text de la preuve, indexable par FTS5 et le Retriever
- Les entites visuelles alimentent le graphe Neo4j et le SuspectScorer

---

### 2.11 MonitoringScheduler

**Fonction :** Execution periodique des jobs de surveillance (SearXNG clearweb + Robin dark web).

**Fichier :** `nexus/monitoring/scheduler.py`

**Entrees :**
- Jobs de monitoring crees par l'utilisateur ou par la boucle ACT
- Configuration intervalle (heures)

**Sorties :**
- Resultats de recherche avec titre, snippet, URL, score de pertinence
- Detection de doublons via ChromaDB monitoring_results

**Stockage :** SQLite monitoring_results, ChromaDB monitoring_results

**Boucles de retroaction :**
- Les resultats pertinents (relevance >= seuil) sont auto-ingeres par la boucle ORIENT
- Les resultats alimentent EvidenceProcessor -> toute la chaine d'indexation

---

### 2.12 ForensicAnalyzers

**Fonction :** Suite d'outils d'analyse forensique specialisee.

**Fichier :** `nexus/forensics/` (blood_pattern.py, trace_analyzer.py, acoustic_analysis.py, physics_sim.py)

**Sous-outils :**

| Analyseur | Modele | Type d'evidence | Fichier |
|-----------|--------|-----------------|---------|
| BloodPatternAnalyzer | qwen3-vl:8b | Images (sang, eclaboussures) | `blood_pattern.py` |
| TraceAnalyzer | qwen3-vl:8b | Images (empreintes, traces) | `trace_analyzer.py` |
| AcousticAnalyzer | voxtral-mini:4b | Audio (transcription, evenements) | `acoustic_analysis.py` |
| PhysicsSim | Calcul numerique | Simulations (trajectoire, son) | `physics_sim.py` |

**Entrees :**
- Preuves image/audio non encore analysees
- Detection par mots-cles dans le titre/resume (sang, empreinte, etc.)

**Sorties :**
- Classification de pattern (BPA)
- Analyse de traces (type, description)
- Transcription audio + evenements sonores
- Resultats de simulation physique

**Stockage :** SQLite evidence.metadata (bpa_result, trace_result, acoustic_result), evidence.summary (enrichi)

**Boucles de retroaction :**
- Les resultats forensiques sont ajoutes au summary de la preuve -> recuperables par le Retriever
- La transcription audio devient le raw_text de la preuve -> indexable
- Les patterns BPA peuvent informer les hypotheses

---

### 2.13 TimelineBuilder

**Fonction :** Reconstruction chronologique des evenements du dossier.

**Fichier :** `nexus/core/timeline_builder.py`

**Entrees :**
- Preuves avec dates (source_date, dates extraites)
- Entites temporelles (type "date")
- Evenements Neo4j

**Sorties :**
- Liste chronologique d'evenements `[{date, description, source, evidence_id}]`
- Stockage comme analysis_run (type="timeline_rebuild")

**Stockage :** SQLite analysis_runs

**Boucles de retroaction :**
- La timeline aide le self-questioning a identifier des trous temporels
- Les verifications de trajet (GeoMapper) enrichissent la timeline

---

## 3. Matrice de connexion

Lecture : Ligne = outil source, Colonne = outil destination.
"X" = l'outil source alimente l'outil destination.

```
                  EP  EE  AP  HE  CD  SS  IR  ST  GM  IA  MS  FA  TB
EvidenceProc (EP)  .   X   .   .   .   .   X   X   .   .   .   .   .
EntityExtract(EE)  .   .   .   .   .   X   .   .   X   .   X   .   .
AnalysisPipe (AP)  .   .   .   X   .   .   .   .   .   .   .   .   .
HypothesisEng(HE)  .   .   .   .   .   X   .   .   .   .   .   .   .
ContradictDet(CD)  .   .   .   .   .   X   .   .   .   .   .   .   .
SuspectScorer(SS)  .   .   .   .   .   .   .   .   .   .   .   .   .
Retriever    (IR)  .   .   X   X   .   .   .   .   .   .   .   .   .
SummaryTree  (ST)  .   .   .   .   .   .   .   .   .   .   .   .   .
GeoMapper    (GM)  .   .   .   .   .   .   .   .   .   .   .   .   X
ImageAnalyzer(IA)  X   X   .   .   .   .   X   .   .   .   .   .   .
MonitorSched (MS)  X   .   .   .   .   .   .   .   .   .   .   .   .
Forensics    (FA)  .   .   .   .   .   .   X   .   .   .   .   .   .
TimelineBuild(TB)  .   .   .   .   .   .   .   .   .   .   .   .   .
```

### Legende detaillee

| Source -> Destination | Nature de la connexion |
|-----------------------|------------------------|
| EP -> EE | EvidenceProcessor appelle EntityExtractor pour chaque preuve |
| EP -> IR | Chunks indexes dans ChromaDB, consultables par le Retriever |
| EP -> ST | Mise a jour incrementale de l'arbre RAPTOR |
| EE -> SS | Nouvelles entites person declenchent scoring suspect |
| EE -> GM | Nouvelles entites location declenchent geocodage |
| EE -> MS | Nouvelles entites email/account declenchent OSINT recon |
| AP -> HE | L'analyse re-score les hypotheses existantes |
| HE -> SS | Les scores d'hypotheses alimentent le facteur H du scoring suspect |
| CD -> SS | Les contradictions alimentent le facteur C du scoring suspect |
| IR -> AP | Le Retriever fournit le contexte RAG pour l'analyse |
| IR -> HE | Le Retriever fournit le contexte RAG par hypothese |
| IA -> EP | L'ImageAnalyzer produit raw_text et summary pour la preuve |
| IA -> EE | L'ImageAnalyzer extrait des entites visuelles |
| IA -> IR | Les descriptions sont chunked et embedees dans evidence_chunks |
| MS -> EP | Les resultats monitoring sont auto-ingeres comme preuves |
| FA -> IR | Les resultats forensiques enrichissent le summary (retrouvable par Retriever) |
| GM -> TB | Les lieux geocodes enrichissent la timeline |

---

## 4. Boucles de retroaction

### Boucle 1 : Monitoring -> Evidence -> Analyse -> Queries

```
MonitoringScheduler
    |
    v (resultats pertinents)
EvidenceProcessor
    |
    v (preuves indexees)
AnalysisPipeline + HypothesisEngine
    |
    v (nouvelles hypotheses, scores shifts)
ACT: generation de queries adaptatives (LLM)
    |
    v (nouveaux monitoring_jobs)
MonitoringScheduler (retour au debut)
```

**Delai d'un cycle complet :** `investigation_cycle_minutes` (configurable)

### Boucle 2 : Evidence -> Entites -> Suspects -> Hypotheses

```
EvidenceProcessor
    |
    v (entites extraites)
EntityExtractor
    |
    v (person entities)
SuspectScorer (5 facteurs)
    |
    v (suspects scores)
HypothesisEngine (hypotheses mentionnant suspects)
    |
    v (re-evaluation scores)
SuspectScorer (facteur H mis a jour au cycle suivant)
```

### Boucle 3 : Self-questioning adversarial

```
HypothesisEngine (hypothese top)
    |
    v
Self-questioning (nexus 26B, prompt adversarial)
    |
    v (nouvelles pistes, doutes)
ACT: queries de recherche
    |
    v
Monitoring -> Evidence -> Hypothese mise a jour
```

### Boucle 4 : Forensique -> Retriever -> Analyse

```
ForensicAnalyzers (BPA, traces, acoustique)
    |
    v (resultats ajoutes au summary de la preuve)
Retriever (retrouve ces resultats forensiques)
    |
    v (contexte enrichi pour les LLM)
AnalysisPipeline / HypothesisEngine
    |
    v (hypotheses informees par la forensique)
```

### Boucle 5 : OSINT enrichment

```
EntityExtractor (email/account entities)
    |
    v
Holehe (email -> 120+ services)
SocialRecon (username -> plateformes)
    |
    v (profils trouves)
ACT: creation de monitoring_jobs site-specifiques
    |
    v
MonitoringScheduler (recherches ciblees)
    |
    v
DomainRecon (WHOIS/DNS sur domaines non-freemail)
```
