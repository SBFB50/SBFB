# NEXUS -- Pipeline d'ingestion des preuves

**Version :** 0.2.0
**Date :** 2026-04-06
**Fichier source principal :** `nexus/core/evidence_processor.py`

---

## Table des matieres

1. [Vue d'ensemble](#1-vue-densemble)
2. [Pipeline texte/fichier (11 etapes)](#2-pipeline-textefichier-11-etapes)
3. [Branche image (VLM)](#3-branche-image-vlm)
4. [Pipeline texte manuel](#4-pipeline-texte-manuel)
5. [Gestion d'erreur et status](#5-gestion-derreur-et-status)
6. [Dependances entre etapes](#6-dependances-entre-etapes)

---

## 1. Vue d'ensemble

L'ingestion transforme un fichier brut ou un texte en une preuve completement indexee dans les 3 systemes de stockage (SQLite, Neo4j, ChromaDB) et prete a etre retrouvee par le retriever hybride.

### Points d'entree

| Methode | Declencheur | Fichier |
|---------|-------------|---------|
| `EvidenceProcessor.process_upload()` | Upload via API `/api/cases/{id}/evidence/upload` | `evidence_processor.py:89` |
| `EvidenceProcessor.process_text_input()` | Saisie texte via API `/api/cases/{id}/evidence/text` | `evidence_processor.py:279` |
| `_inject_wave()` | Benchmark automatise | `api/benchmark.py:108` |
| `_orient_ingest()` | Boucle autonome (auto-ingestion monitoring) | `autonomous_loop.py:348` |

### Statuts de progression

```
pending --> processing --> processed
                |
                +--> error (si exception globale)
```

---

## 2. Pipeline texte/fichier (11 etapes)

### Etape 1 : Sauvegarde fichier + nettoyage texte

**Fichier source :** `evidence_processor.py:122-131`

```
Fichier upload
    |
    v
data/uploads/{case_id}/{uuid}.{ext}   (mkdir parents=True)
    |
    v
detect_mime_type(dest_path)  -->  _MIME_TO_EVIDENCE_TYPE mapping
    |
    v
_extract_text(file_path, evidence_type)
    |
    +-- PDF  --> PDFParser.extract_text() (PyMuPDF)
    +-- text --> TextParser.extract_text() (nettoyage Unicode)
    +-- image/audio --> None (pas d'extraction texte directe)
```

**Types MIME reconnus :**

| MIME | evidence_type |
|------|---------------|
| application/pdf | pdf |
| image/jpeg, png, gif, webp, tiff | image |
| text/plain, html, csv, markdown | text |
| audio/mpeg, wav, ogg, flac | audio |
| video/mp4, webm | audio |

### Etape 2 : Hash SHA-256

**Fichier source :** `evidence_processor.py:148-149`

```python
file_hash = compute_file_hash(dest_path)  # nexus/ingest/pdf_parser.py
```

Le hash sert a la deduplication et a la verification d'integrite. Stocke dans `evidence.metadata.file_hash`.

Pour le texte manuel, le hash est calcule sur le contenu texte :
```python
text_hash = hashlib.sha256(cleaned.encode("utf-8")).hexdigest()
```

### Etape 3 : Creation record SQLite (status='pending')

**Fichier source :** `evidence_processor.py:154-166`

```sql
INSERT INTO evidence (id, case_id, title, evidence_type, source,
                      file_path, raw_text, status, metadata)
VALUES (uuid4(), ?, ?, ?, ?, ?, ?, 'pending', ?)
```

Colonnes renseignees : `title`, `evidence_type` (auto ou override), `source`, `file_path`, `raw_text`, `metadata` (contient file_hash + mime_type).

### Etape 4 : Passage en status='processing'

**Fichier source :** `evidence_processor.py:168`

```python
await self._db.update_evidence(evidence_id, status="processing")
```

Ce marqueur permet au front-end d'afficher un indicateur de progression.

### Etape 5 : Extraction d'entites + deduplication + ChromaDB

**Fichier source :** `evidence_processor.py:209-213` + `entity_extractor.py`

```
raw_text
    |
    v
[5a] GLiNER NER (CPU, ~0.08s)
     Labels: personne, lieu, vehicule, telephone, email, date,
             organisation, arme, drogue, somme d'argent, etc.
     Modele: urchade/gliner_multi-v2.1
    |
    v
[5b] Fallback LLM (gemma4:e4b) si GLiNER indisponible
    |
    v
[5c] Deduplication RapidFuzz (Jaro-Winkler, seuil 82%)
     Contre les entites existantes du case
    |
    v
[5d] Sauvegarde nouvelles entites -> SQLite entities
     + Creation entity_mentions (entite <-> preuve, avec confiance)
    |
    v
[5e] Embedding entites -> ChromaDB entity_contexts
     Texte: "{name} ({entity_type}): {description}"
     Vecteur: nomic-embed-text 768 dim
    |
    v
[5f] Mentions pour doublons existants
     Les entites deja presentes recoivent quand meme un entity_mention
     vers cette preuve (fuzzy match >= 82%)
```

**Detail de la deduplication :**

```python
# Pour chaque entite extraite
norm_name = normalize_entity_name(ent["name"])  # lowercase, sans accents
for existing in existing_entities:
    if existing.entity_type != ent["type"]:
        continue
    score = fuzz.WRatio(norm_name, normalize(existing.name))
    if score >= 82:
        # Doublon detecte -> pas de creation, mais mention creee
```

**Non-fatal :** Si l'extraction echoue, le pipeline continue a l'etape 6.

### Etape 6 : Resume LLM

**Fichier source :** `evidence_processor.py:221-228`

```python
# Troncature a 8000 caracteres pour le prompt
truncated = text[:8_000]
prompt = EVIDENCE_SUMMARY_PROMPT.format(evidence=truncated)
summary = await self._router.route(TaskType.EVIDENCE_SUMMARY, prompt)
```

**Modele :** gemma4:e4b (rapide, ~80 tok/s)
**Timeout :** 30 secondes
**Heavy :** Non (pas de lock VRAM)

Le prompt `EVIDENCE_SUMMARY_PROMPT` demande un resume factuel en francais, sans interpretation.

**Non-fatal :** Si la generation echoue, `summary` reste vide.

### Etape 7 : UPDATE status='processed'

**Fichier source :** `evidence_processor.py:231-235`

```python
await self._db.update_evidence(evidence_id, summary=summary, status="processed")
```

A ce stade, la preuve est consultable dans l'UI avec son resume.

### Etape 8 : Synchronisation Neo4j

**Fichier source :** `evidence_processor.py:533-598`

```
[8a] Sync noeud Evidence dans Neo4j
     MERGE (e:Evidence {id: $id})
     SET e.case_id, e.title, e.evidence_type, e.reliability
         |
         v
[8b] Sync noeuds entites MENTIONNEES dans cette preuve uniquement
     (pas toutes les entites du case)
     MERGE (n:{Label} {id: $id})
     SET n.name, n.case_id, n.entity_type
         |
         v
[8c] Liens Evidence -> Entity
     MERGE (e)-[:MENTIONS]->(n)
         |
         v
[8d] Extraction et sync des relations inter-entites
     Si >= 2 entites mentionnees:
       LLM extrait les relations (type, from, to, context)
       MERGE (a)-[:TYPE]->(b)
```

**Important :** Seules les entites mentionnees dans *cette* preuve sont synchronisees, pas toutes les entites du dossier. Cela evite de creer des liens non fondes.

**Resilience :** Si Neo4j est indisponible (`self._neo4j is None`), l'etape est silencieusement sautee.

### Etape 9 : Chunk + Embed pour RAG

**Fichier source :** `evidence_processor.py:510-531` + `chunker.py` + `embedding_store.py`

```
raw_text
    |
    v
[9a] TextChunker.chunk_evidence()
     - chunk_size = 512 tokens (~2048 chars)
     - overlap = 128 tokens (~512 chars)
     - Strategie recursive : paragraphes > lignes > phrases > mots
     - Metadata par chunk : evidence_id, case_id, title, source, chunk_index
         |
         v
[9b] EmbeddingStore.index_evidence()
     - Batch embedding via nomic-embed-text (LLMRouter.embed_batch)
     - Stockage dans ChromaDB collection "evidence_chunks"
     - Metadata : evidence_id, case_id, evidence_type, title, source, chunk_index
     - Espace : cosine (hnsw:space = cosine)
```

**Non-fatal :** Si le chunk+embed echoue, la preuve reste indexee dans SQLite (recuperable par FTS5).

### Etape 10 : Arbre de resumes RAPTOR

**Fichier source :** `evidence_processor.py:256-265` + `summary_tree.py`

```
[10a] SummaryTree.update_for_new_evidence(case_id, evidence_id)
      |
      v
[10b] Embedding du resume de la preuve (nomic-embed-text)
      |
      v
[10c] Recherche du cluster le plus proche (cosine similarity)
      - Si similarity >= 0.40 : merge dans le cluster existant
      - Sinon : creation d'un nouveau cluster
      |
      v
[10d] Regeneration du resume du cluster (gemma4:e4b)
      |
      v
[10e] Mise a jour du resume global du case (si assez de clusters)
```

**Niveaux de l'arbre :**

| Niveau | Granularite | Stockage |
|--------|-------------|----------|
| L0 | Resume individuel par preuve | SQLite evidence.summary |
| L1 | Resume de cluster thematique | SQLite summary_clusters |
| L2 | Resume global du dossier | SQLite case_summaries |

**Reconstruction complete** (`rebuild_tree()`) : Clustering agglomeratif scipy sur tous les embeddings, seuil de distance cosine 0.98.

### Etape 11 : Audit log

**Fichier source :** `evidence_processor.py:268-270` + `audit.py`

```
[11a] SQLite audit_log
      INSERT avec hash chain (SHA-256 de previous_hash + entry_data)
      |
[11b] JSONL append-only
      data/audit/{case_id}.jsonl (une ligne JSON par entree)
      |
[11c] Git commit
      git add + git commit dans data/audit/.git
```

**Non-bloquant :** Les 3 couches sont dans des try/except. Un echec d'audit ne bloque jamais le pipeline.

---

## 3. Branche image (VLM)

**Fichier source :** `evidence_processor.py:173-203` + `image_analyzer.py`

Quand `evidence_type == "image"`, le pipeline bifurque vers la pipeline visuelle AVANT l'etape 5 :

```
image file
    |
    v
ImageAnalyzer.process_evidence_image(case_id, evidence_id, path)
    |
    +---> [A] describe_image() -- gemma4:e4b (rapide)
    |     Prompt: IMAGE_DESCRIPTION_PROMPT
    |     -> raw_text de la preuve
    |
    +---> [B] extract_entities_from_image() -- gemma4:e4b
    |     Prompt: IMAGE_ENTITY_EXTRACTION_PROMPT
    |     -> creation entites + mentions
    |
    +---> [C] analyze_scene() -- qwen3-vl:8b (profond)
    |     Prompt: IMAGE_SCENE_ANALYSIS_PROMPT
    |     -> summary detaille
    |
    +---> [D] embed description (nomic-embed-text)
    |     -> ChromaDB evidence_chunks
    |
    v
UPDATE evidence (raw_text, summary, status='processed')
    |
    v
Audit log
```

**Fallback :** Si la pipeline visuelle echoue, la preuve retombe dans la pipeline texte standard (etapes 5-11). Le `raw_text` sera probablement vide pour une image.

---

## 4. Pipeline texte manuel

**Fichier source :** `evidence_processor.py:279-372`

Identique a la pipeline fichier, sauf :
- Pas de sauvegarde fichier (etape 1)
- Pas de detection MIME
- Le texte est nettoye par `TextParser.extract_from_string()`
- Le hash est calcule sur le texte (`hashlib.sha256`)
- `evidence_type = "text"` toujours

---

## 5. Gestion d'erreur et status

### Hierarchie des erreurs

```
process_upload() / process_text_input()
    |
    +-- Etape 5 (entites) : non-fatal, continue a 6
    |
    +-- Etape 6 (resume) : non-fatal, summary="" 
    |
    +-- Etape 8 (Neo4j) : non-fatal, log warning
    |
    +-- Etape 9 (chunk+embed) : non-fatal, log error
    |
    +-- Etape 10 (RAPTOR) : non-fatal, log warning
    |
    +-- Exception globale (catch ligne 272) :
    |   --> status = 'error'
    |   --> raise (l'appelant recoit l'erreur)
    |
    v
    Si aucune exception globale : status = 'processed'
```

### Tableau des statuts

| Status | Signification |
|--------|---------------|
| `pending` | Record cree, pipeline pas encore demarree |
| `processing` | Pipeline en cours d'execution |
| `processed` | Pipeline terminee avec succes (meme si certaines etapes ont echoue) |
| `error` | Exception globale dans la pipeline |

---

## 6. Dependances entre etapes

```
Etape 1 (fichier)
    |
    v
Etape 2 (hash)  -----------> necessite fichier sur disque
    |
    v
Etape 3 (SQLite) ----------> necessite raw_text + hash
    |
    v
Etape 4 (processing)
    |
    +---[IMAGE?]---> Branche VLM (ImageAnalyzer)
    |                     |
    |                     v
    |                 Return Evidence (fin)
    |
    v
Etape 5 (entites) ---------> necessite raw_text non-vide
    |
    v
Etape 6 (resume) ----------> necessite raw_text non-vide
    |
    v
Etape 7 (processed)
    |
    v
Etape 8 (Neo4j) -----------> necessite self._neo4j != None
    |                         necessite mentions (etape 5)
    v
Etape 9 (chunk+embed) -----> necessite self._chroma != None
    |                         necessite raw_text ou summary
    v
Etape 10 (RAPTOR) ----------> necessite self._chroma != None
    |                          necessite summary (etape 6)
    v
Etape 11 (audit) -----------> necessite self._db
```

### Modules impliques

| Etape | Module(s) | Modele LLM |
|-------|-----------|------------|
| 1 | PDFParser, TextParser | - |
| 2 | compute_file_hash | - |
| 3 | Database.create_evidence | - |
| 5 | EntityExtractor (GLiNER), ChromaClient | nomic-embed-text (embedding entites) |
| 6 | LLMRouter | gemma4:e4b |
| 8 | Neo4jClient | gemma4:e4b (relations optionnel) |
| 9 | TextChunker, EmbeddingStore | nomic-embed-text |
| 10 | SummaryTree | gemma4:e4b + nomic-embed-text |
| 11 | AuditService | - |
