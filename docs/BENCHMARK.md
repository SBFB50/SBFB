# NEXUS -- Guide de benchmarking

**Version :** 0.2.0
**Date :** 2026-04-06
**Fichiers sources :** `nexus/api/benchmark.py`, `data/benchmark/`

---

## Table des matieres

1. [Les 3 cold cases](#1-les-3-cold-cases)
2. [Systeme de scoring](#2-systeme-de-scoring)
3. [Lancer un benchmark](#3-lancer-un-benchmark)
4. [Pipeline de benchmark](#4-pipeline-de-benchmark)
5. [Interpreter les resultats](#5-interpreter-les-resultats)
6. [Structure des fichiers](#6-structure-des-fichiers)

---

## 1. Les 3 cold cases

### 1.1 Affaire Elodie Kulik (2002) -- `kulik`

**Repertoire :** `data/benchmark/kulik/`
**Pieces :** 14 preuves en 4 vagues
**Verite terrain :** Gregory Wiart (agresseur principal, ADN) + Willy Bardon (complice, voix sur l'appel au 18)

| Vague | Nom | Pieces | Contenu |
|-------|-----|--------|---------|
| 1 | Dossier initial | P-01 a P-05 | Rapport police, constatations scene, temoignages (Delattre, Lefebvre, collegues CIC) |
| 2 | Forensique | P-06 a P-08 | Autopsie (Dr. Charlet), resultats ADN + prelevements, expertise vehicule incendie (IRCGN) |
| 3 | Numerique | P-09 a P-11 | Transcription appel au 18, releves telephoniques victime, analyse bornes-relais Cartigny |
| 4 | Contexte | P-12 a P-14 | Environnement local, profil victime, note geographique lieux et trajets |

**Contradictions attendues (5) :**
- C1 : Deux voix masculines sur l'appel au 18 mais un seul profil ADN
- C2 : Un vehicule entendu par le voisin mais traces de pneus de deux vehicules distincts
- C3 : Victime nerveuse au travail mais detendue avec ses amis le soir meme
- C4 : Telephone prepaye inconnu borne a Cartigny, sans correspondance FNAEG
- C5 : Demi-tour inexplique de la victime entre Peronne-Sud et Cartigny

**Hypotheses attendues (4) :**
- H1 : Crime opportuniste par inconnu(s) de passage (score attendu : 30-50)
- H2 : Crime lie a l'entourage professionnel/personnel (score attendu : 10-20)
- H3 : Crime sexuel en reunion par individus locaux connaissant la zone (score attendu : 50-70)
- H4 : Crime passionnel (ex-petit ami, admirateur) (score attendu : 15-25)

**Entites attendues :**
- 11 personnes (Elodie Kulik, Jacqueline Kulik, Delattre, Lefebvre, etc.)
- 5 lieux (Cartigny, Peronne, Ham, Amiens, D44)
- 2 vehicules (VW Polo bleue, vehicule inconnu)
- 2 telephones

---

### 1.2 Golden State Killer (1974-86) -- `gsk`

**Repertoire :** `data/benchmark/golden-state-killer/`
**Pieces :** 13 preuves en 4 vagues
**Verite terrain :** Joseph James DeAngelo, ex-policier d'Exeter puis Auburn PD

| Vague | Nom | Pieces | Contenu |
|-------|-----|--------|---------|
| 1 | Dossier initial | P-01 a P-05 | Synthese crimes ONS, profil FBI, depositions survivantes, voisin |
| 2 | Forensique | P-06 a P-08 | Resultats ADN, analyse armes, MO compare |
| 3 | Numerique/OSINT | P-09 a P-11 | Carte geographic, analyse genealogique ADN, piste policiere |
| 4 | Contexte | P-12 a P-13 | Profil psychologique, chronologie complete |

**Points cles du benchmark :**
- L'hypothese "ex-policier" doit etre generee par le systeme
- Les patterns MO (methodologie) doivent etre detectes
- Le lien EAR/ONS doit etre etabli par l'ADN
- La genealogie genetique (GEDmatch) doit apparaitre dans l'analyse

---

### 1.3 Affaire Moreau (fictif) -- `moreau`

**Repertoire :** `data/benchmark/affaire-moreau/`
**Pieces :** 15 preuves en 4 vagues
**7 contradictions plantees** pour tester la detection

| Vague | Nom | Pieces | Contenu |
|-------|-----|--------|---------|
| 1 | Dossier initial | P-01 a P-06 | Rapport police, 4 temoignages (dont Karim Belhadj fiabilite 45%), autopsie |
| 2 | Numerique | P-07 a P-08 | Telephonie (5 lignes), transactions bancaires |
| 3 | OSINT | P-10 a P-14 | Assurance-vie AXA, Instagram, Kbis Webcraft, main courante, casier judiciaire |
| 4 | Photos + analyse | P-09, P-15 | Descriptions scene, note analytique correlations |

**Points cles du benchmark :**
- 7 contradictions explicitement plantees dans les preuves
- Temoignage de Karim Belhadj en 2 versions (fiabilite basse)
- Piste assurance-vie (mobile financier)
- Main courante anterieure (violence conjugale)
- Casier judiciaire d'un suspect

---

## 2. Systeme de scoring

Le benchmark est evalue sur **100 points** repartis en 5 categories :

```
Score total = Entites /20 + Hypotheses /20 + Contradictions /20
            + Suspects /20 + Timeline+Geo /20
```

### 2.1 Entites (/20)

Mesure la capacite d'extraction d'entites nommees.

| Critere | Points |
|---------|--------|
| >= 80% des entites attendues extraites | 10 |
| >= 60% et < 80% | 6 |
| >= 40% et < 60% | 3 |
| < 40% | 0 |
| Deduplication correcte (pas de doublons significatifs) | 5 |
| Types corrects (person, location, vehicle, phone) | 5 |

**Evaluation :** Comparaison des entites extraites avec `manifest.json > expected_entities`.

### 2.2 Hypotheses (/20)

Mesure la qualite des hypotheses generees.

| Critere | Points |
|---------|--------|
| L'hypothese "correcte" est dans le top 3 par score | 10 |
| Les scores des hypotheses sont dans les fourchettes attendues | 5 |
| Au moins 3 hypotheses distinctes generees | 5 |

**Evaluation :** Comparaison avec `manifest.json > expected_hypotheses` (titres et fourchettes de scores).

Pour Kulik : H3 (crime en reunion par locaux) doit etre dans le top 3.
Pour GSK : l'hypothese "ex-policier" doit etre generee.
Pour Moreau : les pistes financieres doivent apparaitre.

### 2.3 Contradictions (/20)

Mesure la detection de contradictions.

| Critere | Points |
|---------|--------|
| >= 80% des contradictions attendues detectees | 10 |
| >= 50% et < 80% | 6 |
| >= 30% et < 50% | 3 |
| < 30% | 0 |
| Pas de faux positifs majeurs (< 3 fausses contradictions) | 5 |
| Description coherente des contradictions | 5 |

**Evaluation :** Comparaison avec `manifest.json > expected_contradictions` (par mots-cles).

### 2.4 Suspects (/20)

Mesure le scoring des suspects.

| Critere | Points |
|---------|--------|
| Le suspect correct a un score > 40% | 10 |
| Le suspect correct est dans le top 3 | 5 |
| Les 5 facteurs sont non-nuls pour au moins 1 suspect | 5 |

**Evaluation :** Verification que les perpetrators de `manifest.json > ground_truth` sont correctement identifies.

### 2.5 Timeline + Geo (/20)

Mesure la reconstruction chronologique et geographique.

| Critere | Points |
|---------|--------|
| Timeline non-vide avec dates parsees | 5 |
| Evenements dans le bon ordre chronologique | 5 |
| Lieux geocodes avec coordonnees valides | 5 |
| Verification de trajet coherente | 5 |

---

## 3. Lancer un benchmark

### Via l'interface React

1. Ouvrir `http://localhost:3002`
2. Naviguer vers la page **Benchmark**
3. Cliquer **Nouveau benchmark**
4. Choisir le case : Kulik, GSK ou Moreau
5. Le benchmark demarre en arriere-plan (pipeline complete)
6. Suivre la progression via la page Investigation

### Via l'API

```bash
# Lancer le benchmark complet (recommande)
curl -X POST http://localhost:8000/api/benchmark/launch/kulik

# Reponse :
# {"case_id": "uuid", "name": "Affaire KULIK", "status": "running_full_pipeline", "total_evidence": 14}

# Suivre le statut
curl http://localhost:8000/api/cases/{case_id}/investigation/status

# Injecter une vague specifique (optionnel, pour debug)
curl -X POST http://localhost:8000/api/benchmark/inject/{case_id}/kulik/wave/1
```

### Pre-requis

Avant de lancer un benchmark, verifier :

1. **Ollama** : tous les modeles charges
   ```bash
   ollama list  # Doit montrer: juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m, nomic-embed-text
   ```

2. **Docker** : Neo4j et ChromaDB en cours
   ```bash
   docker compose up -d
   ```

3. **Backend** : FastAPI en cours
   ```bash
   uvicorn nexus.main:app --host 0.0.0.0 --port 8000
   ```

4. **Frontend** (optionnel, pour suivi visuel)
   ```bash
   cd web && npx vite --host 0.0.0.0 --port 3002
   ```

---

## 4. Pipeline de benchmark

Quand on lance `POST /api/benchmark/launch/{key}`, la fonction `_run_full_benchmark()` execute 7 etapes sequentiellement en arriere-plan :

```
Etape 1 : Creation du dossier SQLite
    |
    v
Etape 2 : Injection des preuves (toutes les vagues, sequentiellement)
    |       Pour chaque vague :
    |         Pour chaque preuve :
    |           - Lire le fichier texte depuis data/benchmark/{case}/
    |           - EvidenceProcessor.process_text_input()
    |           - Pipeline complete (GLiNER, resume, Neo4j, ChromaDB, RAPTOR)
    |           - Pause 2 secondes (GPU breathe)
    |         Pause 3 secondes entre vagues
    |
    v
Etape 3 : Analyse complete (AnalysisPipeline.run_full_analysis)
    |       - Resume des preuves non-resumees
    |       - Contexte RAG
    |       - Analyse profonde (gemma-4-26B-A4B)
    |       - Re-scoring hypotheses
    |       - Verification logique (gemma-4-26B-A4B)
    |
    v
Etape 4 : Generation d'hypotheses (HypothesisEngine.generate_hypotheses)
    |       - Contexte RAG
    |       - gemma-4-26B-A4B genere 3-6 hypotheses
    |       - Scoring initial
    |       - Snapshots
    |
    v
Etape 5 : Detection des contradictions (ContradictionDetector)
    |       - Paires d'evidence par entites communes
    |       - gemma-4-26B-A4B analyse chaque paire
    |       - Deduplication
    |
    v
Etape 6 : Scoring des suspects (SuspectScorer.score_all_suspects)
    |       - 5 facteurs pour chaque person entity
    |       - Snapshots
    |
    v
Etape 7 : Synchronisation Neo4j (final pass)
    |       - Sync toutes les preuves processed
    |       - Sync toutes les entites
    |       - Liens evidence <-> entity via mentions
    |
    v
Etape 8 : Demarrage investigation reactive (via ReactiveInvestigationManager)
          - Les 20 workers reactifs ecoutent les evenements
          - Le MonitoringLoop cherche de nouvelles informations (30s sweep)
          - Les hypotheses sont re-evaluees a chaque analysis_completed
```

**Serialisation VRAM :** Chaque etape utilise le `_INJECT_LOCK` global + le `_heavy_lock` du LLMRouter pour eviter la saturation GPU. Un seul modele lourd en VRAM a la fois.

**Singletons partages :** Le benchmark reutilise `app.state.router`, `app.state.chroma`, `app.state.neo4j`, `app.state.entity_extractor` pour eviter de dupliquer les connexions et le modele GLiNER.

---

## 5. Interpreter les resultats

### Via l'interface React

Apres le benchmark, consulter :

| Page | Ce qu'on y trouve |
|------|-------------------|
| **Dashboard** | Nombre de preuves, entites, hypotheses, alertes |
| **Evidence** | Toutes les preuves injectees avec statut (pending/processing/processed/error) |
| **Entities** | Entites extraites -- comparer avec `expected_entities` du manifest |
| **Hypotheses** | Hypotheses generees avec scores -- comparer avec `expected_hypotheses` |
| **Suspects** | Classement des suspects avec breakdown 5 facteurs |
| **Graph** | Graphe de connaissances -- verifier les connexions |
| **Timeline** | Chronologie des evenements |
| **Investigation** | Statut des 20 workers reactifs, outils actifs/en erreur |

### Via l'API

```bash
# Preuves
curl http://localhost:8000/api/cases/{id}/evidence | jq '. | length'

# Entites
curl http://localhost:8000/api/cases/{id}/entities | jq '.[].name'

# Hypotheses
curl http://localhost:8000/api/cases/{id}/hypotheses | jq '.[] | {title, current_score}'

# Contradictions (dans l'audit log)
curl "http://localhost:8000/api/cases/{id}/audit?action=contradiction_found" | jq '.[].summary'

# Suspects
curl http://localhost:8000/api/cases/{id}/suspects | jq '.[] | {name: .entity_name, score: .suspicion_score}'

# Graphe
curl http://localhost:8000/api/cases/{id}/graph | jq '.nodes | length, .edges | length'
```

### Diagnostic des problemes courants

| Symptome | Cause probable | Solution |
|----------|---------------|----------|
| Preuves en status "error" | Crash LLM (timeout ou OOM) | Verifier les logs Ollama, reduire la taille des textes |
| Peu d'entites extraites | GLiNER non charge ou texte trop court | Verifier que GLiNER est pre-charge au startup |
| Hypotheses absentes | Pas assez de preuves processed | Attendre la fin de l'injection des 4 vagues |
| Contradictions nulles | Pas assez de paires d'evidence avec entites communes | Verifier que l'extraction d'entites a fonctionne |
| Suspects tous a 0 | Neo4j non connecte ou pas d'entites person | Verifier Docker + Neo4j sync |
| Neo4j vide | Sync echouee silencieusement | Relancer via `/api/cases/{id}/investigation/start` (resync reactive via events) |

---

## 6. Structure des fichiers

### Arborescence d'un benchmark

```
data/benchmark/kulik/
    manifest.json           # Metadonnees, preuves, contradictions attendues, verite terrain
    police/
        P-01_rapport-initial.txt
        P-02_proces-verbal-scene.txt
    temoignages/
        P-03_amis-ham.txt
        P-04_voisin-cartigny.txt
        P-05_collegues-cic.txt
    forensique/
        P-06_autopsie.txt
        P-07_adn-resultats.txt
        P-08_vehicule-expertise.txt
    numerique/
        P-09_appel-18.txt
        P-10_telephonie.txt
        P-11_bornes-relais.txt
    contexte/
        P-12_environnement.txt
        P-13_profil-victime.txt
    geo/
        P-14_lieux-trajets.txt
```

### Format du manifest.json

```json
{
  "case": {
    "name": "Nom de l'affaire",
    "reference": "#REF",
    "description": "Description detaillee"
  },
  "evidence": [
    {
      "id": "P-01",
      "title": "Titre",
      "type": "text",
      "source": "Source",
      "reliability": 85,
      "source_date": "2002-01-11",
      "file": "police/P-01_rapport.txt",
      "wave": 1
    }
  ],
  "waves": {
    "1": {"name": "Nom vague", "description": "...", "evidence_ids": ["P-01"]}
  },
  "expected_contradictions": [
    {"id": "C1", "description": "...", "evidence": ["P-09", "P-06"], "keywords": [...]}
  ],
  "expected_hypotheses": [
    {"id": "H1", "title": "...", "expected_final_score_range": [30, 50]}
  ],
  "expected_entities": {
    "persons": [...],
    "locations": [...],
    "vehicles": [...],
    "phones": [...]
  },
  "ground_truth": {
    "perpetrators": [
      {"name": "Nom", "role": "Description du role"}
    ]
  }
}
```

### Comparaison des 3 benchmarks

| Critere | Kulik | GSK | Moreau |
|---------|-------|-----|--------|
| Pieces | 14 | 13 | 15 |
| Vagues | 4 | 4 | 4 |
| Nature | Reel (2002) | Reel (1974-86) | Fictif |
| Contradictions attendues | 5 | ~3 | 7 |
| Difficulte NER | Moyenne (francais) | Elevee (anglophone + old dates) | Moyenne |
| Difficulte hypotheses | Elevee (2 agresseurs) | Elevee (policier) | Moyenne (indices plantes) |
| Verite terrain | Wiart + Bardon | DeAngelo | Definie dans manifest |
| Temps estime | ~30-45 min | ~25-40 min | ~30-45 min |
