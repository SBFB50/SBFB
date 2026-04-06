# NEXUS -- Reference API (115+ endpoints)

**Version :** 0.2.0
**Date :** 2026-04-06
**Base URL :** `http://localhost:8000`

---

## Table des matieres

1. [Cases](#1-cases)
2. [Evidence](#2-evidence)
3. [Entities](#3-entities)
4. [Hypotheses](#4-hypotheses)
5. [Graph (Neo4j)](#5-graph-neo4j)
6. [Search](#6-search)
7. [Monitoring](#7-monitoring)
8. [Alerts](#8-alerts)
9. [Analysis](#9-analysis)
10. [Reports](#10-reports)
11. [Timeline](#11-timeline)
12. [Geo](#12-geo)
13. [Recon (OSINT)](#13-recon-osint)
14. [Vision](#14-vision)
15. [Image Search](#15-image-search)
16. [Forensics](#16-forensics)
17. [Physics Sim](#17-physics-sim)
18. [Investigation](#18-investigation)
19. [Audit](#19-audit)
20. [Benchmark](#20-benchmark)
21. [Suspects](#21-suspects)

---

## 1. Cases

**Fichier :** `nexus/api/cases.py`
**Prefix :** `/api/cases`

| Methode | Endpoint | Description |
|---------|----------|-------------|
| POST | `/api/cases` | Creer un nouveau dossier d'investigation |
| GET | `/api/cases` | Lister tous les dossiers (filtre optionnel par `status`) |
| GET | `/api/cases/{case_id}` | Recuperer un dossier par ID |
| PUT | `/api/cases/{case_id}` | Mettre a jour un dossier (update partiel) |
| DELETE | `/api/cases/{case_id}` | Supprimer un dossier et toutes ses donnees (cascade) |
| GET | `/api/cases/{case_id}/stats` | Statistiques agregees du dossier |

**POST /api/cases** -- Body :
```json
{
  "name": "Affaire KULIK",
  "reference": "#2002-PER-0011",
  "description": "Meurtre d'Elodie Kulik, 24 ans..."
}
```

**GET /api/cases/{case_id}/stats** -- Reponse :
```json
{
  "evidence_count": 14,
  "entity_count": 42,
  "hypothesis_count": 5,
  "alert_count": 3,
  "monitoring_jobs": 8
}
```

---

## 2. Evidence

**Fichier :** `nexus/api/evidence.py`
**Prefix :** aucun (routes completes)

| Methode | Endpoint | Description |
|---------|----------|-------------|
| POST | `/api/cases/{case_id}/evidence` | Upload fichier (multipart/form-data) |
| POST | `/api/cases/{case_id}/evidence/text` | Soumettre du texte brut (JSON) |
| GET | `/api/cases/{case_id}/evidence` | Lister les preuves (filtres : `status`, `evidence_type`) |
| GET | `/api/evidence/{evidence_id}` | Recuperer une preuve par ID |
| PUT | `/api/evidence/{evidence_id}` | Mettre a jour les metadonnees |
| DELETE | `/api/evidence/{evidence_id}` | Supprimer une preuve |

**POST /api/cases/{id}/evidence** -- multipart :
- `file` : fichier (obligatoire)
- `title` : titre (obligatoire)
- `source` : provenance (optionnel)
- `evidence_type` : override du type auto-detecte (optionnel)

**POST /api/cases/{id}/evidence/text** -- Body :
```json
{
  "title": "Temoignage LEFEBVRE",
  "text": "Le 10 janvier 2002, vers 23h30...",
  "source": "Audition gendarmerie"
}
```

---

## 3. Entities

**Fichier :** `nexus/api/entities.py`
**Prefix :** aucun

| Methode | Endpoint | Description |
|---------|----------|-------------|
| GET | `/api/cases/{case_id}/entities` | Lister les entites d'un dossier (filtre : `entity_type`) |
| GET | `/api/entities/{entity_id}` | Recuperer une entite par ID |
| GET | `/api/entities/{entity_id}/mentions` | Lister les mentions (entite <-> preuves) |

---

## 4. Hypotheses

**Fichier :** `nexus/api/hypotheses.py`
**Prefix :** `/api`

| Methode | Endpoint | Description |
|---------|----------|-------------|
| POST | `/api/cases/{case_id}/hypotheses/generate` | Generer des hypotheses (background, nexus 26B) |
| GET | `/api/cases/{case_id}/hypotheses` | Lister les hypotheses (filtre : `status`) |
| GET | `/api/hypotheses/{hypothesis_id}` | Recuperer une hypothese par ID |
| PUT | `/api/hypotheses/{hypothesis_id}` | Mettre a jour une hypothese |
| DELETE | `/api/hypotheses/{hypothesis_id}` | Supprimer une hypothese |
| POST | `/api/hypotheses/{hypothesis_id}/evaluate` | Evaluer une hypothese (background) |
| GET | `/api/hypotheses/{hypothesis_id}/snapshots` | Historique des snapshots de scoring |
| GET | `/api/hypotheses/{hypothesis_id}/evolution` | Series temporelles pour graphique |
| POST | `/api/cases/{case_id}/hypotheses/evaluate-all` | Re-evaluer toutes les hypotheses (background) |
| POST | `/api/cases/{case_id}/hypotheses/merge` | Fusionner 2+ hypotheses |
| POST | `/api/cases/{case_id}/contradictions` | Detecter les contradictions (background) |
| GET | `/api/cases/{case_id}/contradictions` | Lister les contradictions detectees |
| POST | `/api/cases/{case_id}/compare-testimonies` | Comparer 2+ temoignages |

**POST /api/cases/{id}/hypotheses/merge** -- Body :
```json
{
  "hypothesis_ids": ["uuid1", "uuid2"],
  "new_title": "Agression preméditée par proche",
  "new_description": "..."
}
```

**POST /api/cases/{id}/compare-testimonies** -- Body :
```json
{
  "evidence_ids": ["ev1", "ev2", "ev3"]
}
```

---

## 5. Graph (Neo4j)

**Fichier :** `nexus/api/graph.py`
**Prefix :** `/api`

| Methode | Endpoint | Description |
|---------|----------|-------------|
| GET | `/api/cases/{case_id}/graph` | Graphe complet (noeuds + aretes) |
| GET | `/api/cases/{case_id}/graph/neighbors/{node_id}` | Voisins d'un noeud (profondeur configurable) |
| GET | `/api/cases/{case_id}/graph/path/{from_id}/{to_id}` | Plus court chemin entre deux noeuds |
| GET | `/api/cases/{case_id}/graph/clusters` | Detection de communautes (Louvain) |
| GET | `/api/cases/{case_id}/graph/stats` | Statistiques du graphe |
| GET | `/api/cases/{case_id}/graph/central-entities` | Entites centrales (degree centrality) |
| GET | `/api/cases/{case_id}/graph/importance` | Entites importantes (betweenness) |
| GET | `/api/cases/{case_id}/graph/communities` | Communautes detectees |
| GET | `/api/cases/{case_id}/graph/connections/{id1}/{id2}` | Connexions entre deux entites |
| GET | `/api/cases/{case_id}/graph/temporal` | Graphe temporel (evolution dans le temps) |
| GET | `/api/cases/{case_id}/graph/evidence-matrix` | Matrice preuve-entite |

---

## 6. Search

**Fichier :** `nexus/api/search.py`
**Prefix :** `/api`

| Methode | Endpoint | Description |
|---------|----------|-------------|
| POST | `/api/cases/{case_id}/search` | Recherche semantique (ChromaDB evidence_chunks) |
| POST | `/api/cases/{case_id}/search/fts` | Recherche full-text (SQLite FTS5) |
| POST | `/api/cases/{case_id}/search/hybrid` | Recherche hybride 4 sources |
| POST | `/api/cases/{case_id}/search/unified` | Recherche unifiee cross-collection |
| GET | `/api/cases/{case_id}/similar/{evidence_id}` | Preuves similaires a une preuve donnee |
| GET | `/api/cases/{case_id}/duplicates` | Detection de doublons |
| GET | `/api/search/stats` | Statistiques des collections ChromaDB |
| POST | `/api/cases/{case_id}/search/reindex` | Re-indexer toutes les preuves |

**POST /api/cases/{id}/search/hybrid** -- Body :
```json
{
  "query": "telephone portable de la victime",
  "n_results": 20,
  "strategy": "hybrid"
}
```

---

## 7. Monitoring

**Fichier :** `nexus/api/monitoring.py`
**Prefix :** `/api`

| Methode | Endpoint | Description |
|---------|----------|-------------|
| POST | `/api/cases/{case_id}/monitoring` | Creer un job de surveillance |
| GET | `/api/cases/{case_id}/monitoring` | Lister les jobs d'un dossier |
| PUT | `/api/monitoring/{job_id}` | Mettre a jour un job |
| DELETE | `/api/monitoring/{job_id}` | Supprimer un job |
| POST | `/api/monitoring/{job_id}/run` | Executer un job immediatement (background, 202) |
| GET | `/api/monitoring/{job_id}/results` | Resultats d'un job specifique |
| GET | `/api/cases/{case_id}/monitoring/results` | Tous les resultats d'un dossier |
| POST | `/api/monitoring/results/{result_id}/ingest` | Ingerer un resultat comme preuve |

**POST /api/cases/{id}/monitoring** -- Body :
```json
{
  "job_type": "searxng",
  "query": "\"Elodie Kulik\" cold case",
  "entity_id": null,
  "interval_hours": 24
}
```

---

## 8. Alerts

**Fichier :** `nexus/api/alerts.py`
**Prefix :** `/api`

| Methode | Endpoint | Description |
|---------|----------|-------------|
| GET | `/api/cases/{case_id}/alerts` | Lister les alertes (filtres : `severity`, `is_read`) |
| PUT | `/api/alerts/{alert_id}/read` | Marquer une alerte comme lue |
| GET | `/api/alerts/unread-count` | Nombre total d'alertes non lues |

---

## 9. Analysis

**Fichier :** `nexus/api/analysis.py`
**Prefix :** `/api`

| Methode | Endpoint | Description |
|---------|----------|-------------|
| POST | `/api/cases/{case_id}/analyze` | Lancer une analyse complete (background, 202) |
| GET | `/api/analysis/{run_id}` | Recuperer le statut/resultat d'une analyse |
| GET | `/api/cases/{case_id}/analysis` | Historique des analyses du dossier |

---

## 10. Reports

**Fichier :** `nexus/api/reports.py`
**Prefix :** `/api`

| Methode | Endpoint | Description |
|---------|----------|-------------|
| POST | `/api/cases/{case_id}/reports` | Generer un rapport PDF (background) |
| GET | `/api/cases/{case_id}/reports` | Lister les rapports du dossier |
| GET | `/api/reports/{report_id}/download` | Telecharger un rapport PDF |
| GET | `/api/reports/{report_id}` | Metadonnees d'un rapport |

---

## 11. Timeline

**Fichier :** `nexus/api/timeline.py`
**Prefix :** `/api`

| Methode | Endpoint | Description |
|---------|----------|-------------|
| GET | `/api/cases/{case_id}/timeline` | Timeline complete du dossier |
| GET | `/api/cases/{case_id}/timeline/range` | Timeline filtree par dates |

---

## 12. Geo

**Fichier :** `nexus/api/geo.py`
**Prefix :** `/api`

| Methode | Endpoint | Description |
|---------|----------|-------------|
| POST | `/api/cases/{case_id}/geocode` | Geocoder toutes les entites location |
| GET | `/api/cases/{case_id}/map` | Donnees cartographiques (lieux + liens) |
| POST | `/api/cases/{case_id}/route` | Calculer un itineraire entre deux lieux |
| POST | `/api/cases/{case_id}/verify-travel` | Verifier un temps de trajet (alibi) |

**POST /api/cases/{id}/verify-travel** -- Body :
```json
{
  "from_location": "Ham",
  "to_location": "Cartigny",
  "departure_time": "2002-01-10T23:45:00",
  "arrival_time": "2002-01-11T00:19:00"
}
```

---

## 13. Recon (OSINT)

**Fichier :** `nexus/api/recon.py`
**Prefix :** `/api`

| Methode | Endpoint | Description |
|---------|----------|-------------|
| POST | `/api/recon/email/{email}` | Holehe + social recon sur email |
| POST | `/api/recon/username/{username}` | Recherche username sur les plateformes |
| POST | `/api/recon/domain/{domain}` | WHOIS + DNS d'un domaine |
| GET | `/api/cases/{case_id}/recon` | Resultats OSINT de toutes les entites |
| POST | `/api/cases/{case_id}/recon/auto` | Lancer un scan OSINT automatise |

---

## 14. Vision

**Fichier :** `nexus/api/vision.py`
**Prefix :** `/api`

| Methode | Endpoint | Description |
|---------|----------|-------------|
| POST | `/api/evidence/{evidence_id}/analyze-image` | Analyser une preuve image (VLM complet) |
| POST | `/api/cases/{case_id}/analyze-images` | Analyser toutes les images d'un dossier |
| POST | `/api/vision/describe` | Decrire une image (upload direct) |
| POST | `/api/vision/compare` | Comparer deux images |
| GET | `/api/cases/{case_id}/visual-entities` | Entites extraites d'images |

---

## 15. Image Search

**Fichier :** `nexus/api/image_search.py`
**Prefix :** `/api`

| Methode | Endpoint | Description |
|---------|----------|-------------|
| POST | `/api/cases/{case_id}/image-search/text` | Recherche texte -> image (CLIP) |
| POST | `/api/cases/{case_id}/image-search/similar` | Recherche image -> image (DINOv2) |
| GET | `/api/cases/{case_id}/image-search/gallery` | Galerie de toutes les images indexees |
| POST | `/api/cases/{case_id}/image-search/index` | Indexer une image dans DINOv2/CLIP |

---

## 16. Forensics

**Fichier :** `nexus/api/forensics.py`
**Prefix :** `/api/forensics`

### Blood Pattern Analysis (BPA)

| Methode | Endpoint | Description |
|---------|----------|-------------|
| POST | `/api/forensics/bpa/classify` | Classifier un pattern sanguin |
| POST | `/api/forensics/bpa/analyze` | Analyse complete de taches de sang |
| POST | `/api/forensics/bpa/calculate-angle` | Calculer l'angle d'impact |
| POST | `/api/forensics/bpa/convergence` | Trouver le point de convergence |

### Acoustique

| Methode | Endpoint | Description |
|---------|----------|-------------|
| POST | `/api/forensics/audio/transcribe` | Transcrire un fichier audio |
| POST | `/api/forensics/audio/analyze` | Analyse forensique audio |
| POST | `/api/forensics/audio/events` | Detecter les evenements sonores |
| POST | `/api/forensics/audio/propagation` | Modeliser la propagation sonore |

### Traces

| Methode | Endpoint | Description |
|---------|----------|-------------|
| POST | `/api/forensics/trace/analyze` | Analyser une trace (empreinte, pneu...) |
| POST | `/api/forensics/trace/compare` | Comparer deux traces |

### Auto

| Methode | Endpoint | Description |
|---------|----------|-------------|
| POST | `/api/forensics/cases/{case_id}/auto` | Analyse forensique automatique du dossier |

---

## 17. Physics Sim

**Fichier :** `nexus/api/physics_sim_api.py`
**Prefix :** `/api/physics`

| Methode | Endpoint | Description |
|---------|----------|-------------|
| POST | `/api/physics/blood-drop` | Simulation trajectoire goutte de sang |
| POST | `/api/physics/cast-off` | Simulation projection centrifuge |
| POST | `/api/physics/sound` | Simulation propagation sonore |
| POST | `/api/physics/origin` | Estimation point d'origine |
| GET | `/api/physics/datasets` | Lister les datasets de simulation |
| GET | `/api/physics/datasets/{name}` | Recuperer un dataset specifique |

---

## 18. Investigation

**Fichier :** `nexus/api/investigation.py`
**Prefix :** `/api`

| Methode | Endpoint | Description |
|---------|----------|-------------|
| GET | `/api/investigations` | Lister les investigations actives |
| POST | `/api/cases/{case_id}/investigation/start` | Demarrer la boucle autonome |
| POST | `/api/cases/{case_id}/investigation/stop` | Arreter la boucle autonome |
| GET | `/api/cases/{case_id}/investigation/status` | Statut detaille (cycle, outils, derniere action) |
| GET | `/api/cases/{case_id}/investigation/log` | Journal de l'investigation |

**GET /api/cases/{id}/investigation/status** -- Reponse :
```json
{
  "case_id": "uuid",
  "running": true,
  "cycle_count": 7,
  "last_action": "DECIDE",
  "last_cycle_at": "2026-04-06T14:30:00",
  "started_at": "2026-04-06T12:00:00",
  "tools": {
    "monitoring": {"status": "done", "detail": "3 resultats pertinents"},
    "evidence_processor": {"status": "done", "detail": "2 preuves ingerees"},
    "hypothesis_engine": {"status": "running", "detail": "Evaluation..."},
    "..."
  }
}
```

---

## 19. Audit

**Fichier :** `nexus/api/audit.py`
**Prefix :** `/api`

| Methode | Endpoint | Description |
|---------|----------|-------------|
| GET | `/api/cases/{case_id}/audit` | Journal d'audit (filtres : `action`, `limit`) |
| GET | `/api/cases/{case_id}/audit/summary` | Resume de l'activite du dossier |
| GET | `/api/cases/{case_id}/audit/timeline` | Timeline des actions |
| GET | `/api/audit/{audit_id}` | Detail d'une entree d'audit |
| GET | `/api/cases/{case_id}/audit/verify` | Verification d'integrite du hash chain |

---

## 20. Benchmark

**Fichier :** `nexus/api/benchmark.py`
**Prefix :** `/api/benchmark`

| Methode | Endpoint | Description |
|---------|----------|-------------|
| GET | `/api/benchmark/available` | Lister les benchmarks disponibles |
| POST | `/api/benchmark/launch/{bench_key}` | Lancer un benchmark complet (pipeline complete) |
| POST | `/api/benchmark/inject/{case_id}/{bench_key}/wave/{wave}` | Injecter une vague specifique |

**Cles de benchmark :** `kulik`, `gsk`, `moreau`

**POST /api/benchmark/launch/{key}** -- Reponse :
```json
{
  "case_id": "uuid",
  "name": "Affaire KULIK",
  "status": "running_full_pipeline",
  "total_evidence": 14
}
```

---

## 21. Suspects

**Fichier :** `nexus/api/suspects.py`
**Prefix :** `/api`

| Methode | Endpoint | Description |
|---------|----------|-------------|
| GET | `/api/cases/{case_id}/suspects` | Lister les suspects avec scores |
| POST | `/api/cases/{case_id}/suspects/score-all` | Scorer tous les suspects (background) |
| POST | `/api/suspects/{suspect_id}/evaluate-profile` | Evaluer le profil LLM (background) |
| GET | `/api/suspects/{suspect_id}` | Detail d'un suspect |
| PUT | `/api/suspects/{suspect_id}` | Mettre a jour (notes, relation victime, etc.) |
| GET | `/api/suspects/{suspect_id}/evolution` | Evolution temporelle du score |
| GET | `/api/suspects/{suspect_id}/snapshots` | Historique des snapshots |

---

## Notes techniques

### Authentification

Aucune authentification requise (systeme local).

### Operations en arriere-plan

Les endpoints retournant HTTP 202 executent des taches lourdes en `BackgroundTasks` FastAPI. Le client doit interroger le statut via les endpoints GET correspondants.

### Serialisation VRAM

Les operations utilisant des modeles lourds (26B, 14B) sont serialisees via `asyncio.Lock`. Les appels simultanees a ces endpoints seront mis en file d'attente.

### Pagination

La plupart des endpoints GET listent tous les resultats sans pagination. Pour les dossiers avec beaucoup de donnees, les filtres (`status`, `entity_type`, `action`) limitent les resultats.
