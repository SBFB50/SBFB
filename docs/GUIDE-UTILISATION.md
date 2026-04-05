# NEXUS -- Guide d'utilisation

**Version :** 0.1.0
**Date :** 2026-04-05

Ce guide explique comment utiliser NEXUS pour mener une investigation sur un cold case. Il couvre toutes les fonctionnalites du systeme, de la creation d'un dossier a la generation de rapports.

---

## Table des matieres

1. [Vue d'ensemble](#1-vue-densemble)
2. [Creer un dossier d'investigation](#2-creer-un-dossier-dinvestigation)
3. [Ajouter des preuves](#3-ajouter-des-preuves)
4. [Lancer une analyse](#4-lancer-une-analyse)
5. [Investigation autonome (boucle OODA)](#5-investigation-autonome-boucle-ooda)
6. [Hypotheses](#6-hypotheses)
7. [Monitoring OSINT](#7-monitoring-osint)
8. [Graphe relationnel (Neo4j)](#8-graphe-relationnel-neo4j)
9. [Outils forensiques](#9-outils-forensiques)
10. [Carte d'investigation](#10-carte-dinvestigation)
11. [Rapports](#11-rapports)
12. [Journal d'audit](#12-journal-daudit)
13. [Recherche semantique](#13-recherche-semantique)
14. [OSINT Recon](#14-osint-recon)
15. [Vision et analyse d'images](#15-vision-et-analyse-dimages)
16. [Reference API](#16-reference-api)

---

## 1. Vue d'ensemble

### Qu'est-ce que NEXUS ?

NEXUS est un systeme d'investigation **persistant** et **incremental** concu pour l'analyse de cold cases. Contrairement a un chatbot classique, NEXUS :

- **Stocke tout** -- chaque preuve, chaque hypothese, chaque resultat de recherche est sauvegarde de maniere permanente
- **Evolue dans le temps** -- les nouvelles preuves declenchent une reevaluation automatique des hypotheses
- **Surveille en continu** -- des recherches automatiques sur le clearweb (toutes les 6h) et le dark web (toutes les 24h)
- **Pense de maniere adversariale** -- chaque hypothese est systematiquement challengee

### Architecture multi-modeles

NEXUS ne repose pas sur un seul LLM. Les taches sont routees vers le modele le plus adapte :

| Modele | Role | Vitesse |
|--------|------|---------|
| `gemma4:e4b` (4B) | Extraction d'entites, filtrage, reformulation | Rapide (~80 t/s) |
| `huihui_ai/deepseek-r1-abliterated:14b` | Raisonnement logique, detection de contradictions | Moyen |
| `nexus` (Gemma 4 26B Heretic) | Analyse profonde, hypotheses, rapports | Lent mais precis |
| `nomic-embed-text` | Embeddings vectoriels (recherche semantique) | Instantane |
| `voxtral-mini:4b` | Transcription audio/video | Moyen |
| `qwen3-vl:8b` | Analyse d'images avancee | Moyen |

Les modeles partagent les 16 GB de VRAM du GPU. NEXUS serialise les appels : un seul modele tourne a la fois.

### Interface

NEXUS s'utilise principalement via le **dashboard Streamlit** (http://localhost:8501), qui propose 15 pages accessibles depuis le menu lateral. Toutes les operations sont aussi disponibles via l'**API REST** FastAPI (http://localhost:8000).

---

## 2. Creer un dossier d'investigation

Un dossier (case) est le conteneur principal d'une investigation. Toutes les preuves, entites, hypotheses et resultats de monitoring sont lies a un dossier.

### Via l'interface Streamlit

1. Ouvrir http://localhost:8501
2. Sur la page d'accueil, remplir le formulaire **"Creer un nouveau dossier"** :
   - **Nom du dossier** (obligatoire) : ex. "Disparition de Jean Dupont"
   - **Reference** (optionnel) : ex. "COLD-2024-001"
   - **Description** (optionnel) : contexte general de l'affaire
3. Cliquer sur **"Creer"**
4. Le dossier apparait dans le selecteur de la barre laterale

### Via l'API

```bash
curl -X POST http://localhost:8000/api/cases \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Disparition de Jean Dupont",
    "reference": "COLD-2024-001",
    "description": "Homme de 45 ans disparu le 15 mars 2019 a Lyon",
    "status": "active"
  }'
```

Reponse :

```json
{
  "id": "a1b2c3d4-...",
  "name": "Disparition de Jean Dupont",
  "reference": "COLD-2024-001",
  "description": "Homme de 45 ans disparu le 15 mars 2019 a Lyon",
  "status": "active",
  "created_at": "2026-04-05T14:30:00",
  "updated_at": "2026-04-05T14:30:00"
}
```

### Gestion des dossiers

- **Statuts possibles :** `active`, `archived`, `closed`
- **Suppression :** Supprime le dossier et toutes les donnees liees (cascade). Irreversible.
- **Plusieurs dossiers :** Vous pouvez travailler sur plusieurs dossiers simultanement. Le dossier actif se selectionne dans la barre laterale.

---

## 3. Ajouter des preuves

Les preuves (evidence) sont les donnees brutes de l'investigation. NEXUS accepte plusieurs formats.

### Types de preuves supportes

| Type | Formats | Traitement automatique |
|------|---------|----------------------|
| PDF | `.pdf` | Extraction de texte via PyMuPDF |
| Image | `.png`, `.jpg`, `.jpeg`, `.webp`, `.tiff` | Analyse VLM, embeddings visuels |
| Audio | `.wav`, `.mp3`, `.ogg` | Transcription via voxtral |
| Texte | Saisie directe | Pret a analyser |
| URL | Lien web | Stockage de metadonnees |

### Uploader un fichier (Streamlit)

1. Aller sur la page **"Preuves"** (menu lateral)
2. Selectionner un fichier via le bouton d'upload
3. Renseigner :
   - **Titre** (obligatoire) : description de la preuve
   - **Source** (optionnel) : provenance (ex. "Rapport de police", "Temoin X")
4. Cliquer sur **"Uploader"**

### Soumettre du texte (Streamlit)

1. Aller sur la page **"Preuves"**
2. Utiliser l'onglet texte
3. Renseigner le titre, coller le texte (temoignage, notes, rapport), indiquer la source
4. Valider

### Via l'API -- upload de fichier

```bash
curl -X POST http://localhost:8000/api/cases/<case_id>/evidence \
  -F "file=@rapport_police.pdf" \
  -F "title=Rapport de police du 15/03/2019" \
  -F "source=Police nationale"
```

### Via l'API -- soumission de texte

```bash
curl -X POST http://localhost:8000/api/cases/<case_id>/evidence/text \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Temoignage de Marie Martin",
    "text": "Je l ai vu pour la derniere fois le 14 mars vers 19h devant le supermarche...",
    "source": "Audition temoin n3"
  }'
```

### Ce qui se passe automatiquement apres l'ajout

Quand une preuve est ajoutee, NEXUS declenche un pipeline de traitement :

1. **Extraction de texte** -- le contenu textuel est extrait (PDF, image OCR, transcription audio)
2. **Resume** -- `gemma4:e4b` genere un resume du contenu
3. **Extraction d'entites** -- `gemma4:e4b` identifie les personnes, lieux, dates, numeros, organisations, etc.
4. **Synchronisation Neo4j** -- les entites et leurs relations sont ajoutees au graphe de connaissances
5. **Indexation ChromaDB** -- le texte est converti en embeddings via `nomic-embed-text` pour la recherche semantique
6. **Statut** -- la preuve passe de `pending` a `processed`

### Consulter les preuves

La page "Preuves" liste toutes les preuves du dossier actif avec :
- Le titre et le type
- Le statut de traitement (pending, processed, error)
- La source et la date
- Le score de fiabilite (0-100, modifiable manuellement)
- Le resume genere par le LLM

---

## 4. Lancer une analyse

L'analyse est le processus central de NEXUS. Elle passe les preuves a travers le pipeline multi-modeles pour generer des hypotheses et detecter des anomalies.

### Analyse manuelle

1. Aller sur la page **"Analyse"** (menu lateral)
2. Cliquer sur **"Lancer une analyse"**
3. L'analyse est lancee en arriere-plan. Son statut apparait dans la page.

### Via l'API

```bash
curl -X POST http://localhost:8000/api/cases/<case_id>/analyze
```

### Ce que le pipeline fait

L'analyse se deroule en trois etapes sequentielles (un seul modele GPU a la fois) :

**Etape 1 : Extraction (gemma4:e4b)**
- Extraction d'entites (noms, lieux, dates, numeros de telephone, adresses, vehicules)
- Filtrage et deduplication
- Structuration en JSON

**Etape 2 : Analyse profonde (nexus 26B)**
- Ingestion de toutes les preuves et entites
- Construction d'une chronologie
- Cartographie des relations
- Detection d'anomalies et d'incoherences
- Generation d'hypotheses avec scores de plausibilite (0-100)
- Identification des pistes d'investigation prioritaires

**Etape 3 : Verification logique (deepseek-r1 14B)**
- Verification de la coherence logique des hypotheses
- Detection de contradictions entre preuves
- Construction de contre-arguments (pensee adversariale)
- Ajustement des scores en consequence

### Historique des analyses

Chaque analyse est enregistree avec :
- Le type (full, incremental, verification)
- Le declencheur (manual, new_evidence, monitoring, scheduled)
- La duree et les tokens consommes
- Le resume du resultat

---

## 5. Investigation autonome (boucle OODA)

La fonctionnalite la plus puissante de NEXUS est la **boucle d'investigation autonome**. Une fois activee, NEXUS enquete de maniere continue sans intervention humaine.

### Qu'est-ce que la boucle OODA ?

OODA (Observe - Orient - Decide - Act) est un cycle decisional militaire adapte a l'investigation :

```
   OBSERVE  -->  Monitoring : collecte de nouveaux resultats (SearXNG, Robin)
      |
   ORIENT   -->  Ingestion : analyse des resultats pertinents, extraction d'entites
      |
   DECIDE   -->  Analyse : reevaluation des hypotheses, detection de contradictions
      |
    ACT     -->  Action : generation de nouvelles requetes de recherche
      |
  QUESTION  -->  Pensee adversariale : auto-questionnement, remise en cause
      |
   (retour a OBSERVE)
```

Chaque cycle dure environ 30 minutes (configurable via `INVESTIGATION_CYCLE_MINUTES` dans le `.env`).

### Demarrer l'investigation autonome

#### Via le Centre de commande (Streamlit)

1. Aller sur la page **"Centre de commande"** (derniere page du menu)
2. Selectionner le dossier dans la barre laterale
3. Cliquer sur le bouton **"Demarrer l'investigation"**
4. Le statut passe a "Actif" avec un indicateur vert

#### Via l'API

```bash
# Demarrer
curl -X POST http://localhost:8000/api/cases/<case_id>/investigation/start

# Verifier le statut
curl http://localhost:8000/api/cases/<case_id>/investigation/status
```

### Ce que fait chaque cycle

A chaque cycle (toutes les 30 minutes), la boucle autonome execute :

1. **Observe** -- Lance les jobs de monitoring actifs (SearXNG, Robin)
2. **Orient** -- Evalue la pertinence des nouveaux resultats, ingere automatiquement ceux au-dessus du seuil (par defaut : 50/100)
3. **Decide** -- Reevalue toutes les hypotheses avec les nouvelles donnees, detecte les contradictions, reconstruit la timeline
4. **Act** -- Genere de nouvelles requetes de recherche basees sur les lacunes identifiees
5. **Question** -- Auto-questionnement : remet en cause les hypotheses dominantes, cherche les angles morts

Modules automatiques actives par defaut :
- Scan OSINT (emails, usernames) des entites
- Geocodage des entites de type lieu
- Analyse d'images via VLM
- Analyse forensique automatique (sang, traces)
- Indexation des images (DINOv2/CLIP)
- Reconnaissance WHOIS/DNS sur les domaines emails
- Reconstruction de timeline

### Surveiller l'investigation

Le **Centre de commande** affiche en temps reel :

- **Metriques** : statut (actif/arrete), nombre de cycles, phase courante, horodatage du dernier cycle
- **Statistiques** : nombre de preuves, entites, hypotheses, contradictions, requetes auto-generees, auto-questionnements
- **Evolution des hypotheses** : barres de progression et graphique temporel
- **Journal d'audit immutable** : chaque action est enregistree avec un hash chaine (tamper-proof)
- **Pensee adversariale** : derniers auto-questionnements du systeme
- **Requetes de recherche** : queries auto-generees vs manuelles
- **Alertes** : alertes non lues classees par severite
- **Contradictions detectees** : incoherences entre preuves

### Arreter l'investigation

#### Via Streamlit

Cliquer sur le bouton **"Arreter"** dans le Centre de commande.

#### Via l'API

```bash
curl -X POST http://localhost:8000/api/cases/<case_id>/investigation/stop
```

L'arret est propre : le cycle en cours termine avant l'arret effectif.

### Parametres de la boucle autonome

Ces parametres sont configurables dans `.env` ou `nexus/config.py` :

| Parametre | Defaut | Description |
|-----------|--------|-------------|
| `INVESTIGATION_CYCLE_MINUTES` | 30 | Duree d'un cycle complet |
| `AUTO_INGEST_RELEVANCE_THRESHOLD` | 50.0 | Score minimum pour ingerer automatiquement un resultat |
| `FULL_REEVALUATION_EVERY_N_CYCLES` | 6 | Reevaluation complete toutes les N cycles (3h par defaut) |
| `MAX_AUTO_INGEST_PER_CYCLE` | 5 | Maximum de resultats ingeres par cycle |
| `MAX_NEW_QUERIES_PER_CYCLE` | 3 | Maximum de nouvelles requetes generees par cycle |
| `AUTO_REPORT_EVERY_N_CYCLES` | 12 | Rapport automatique toutes les 6h |
| `AUTO_BACKUP_EVERY_N_CYCLES` | 24 | Backup automatique toutes les 12h |

---

## 6. Hypotheses

Les hypotheses sont au coeur de l'analyse NEXUS. Chaque hypothese a un score de plausibilite (0-100) qui evolue au fil des preuves.

### Generation automatique

1. Aller sur la page **"Hypotheses"**
2. Cliquer sur **"Generer des hypotheses"**
3. Le modele `nexus` (26B) analyse toutes les preuves et genere des hypotheses
4. L'operation tourne en arriere-plan -- les resultats apparaissent au rafraichissement de la page

Via l'API :
```bash
curl -X POST http://localhost:8000/api/cases/<case_id>/hypotheses/generate
```

### Creation manuelle

Vous pouvez aussi creer une hypothese manuellement :

```bash
curl -X POST http://localhost:8000/api/cases/<case_id>/hypotheses \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Implication du conjoint",
    "description": "Le conjoint de la victime pourrait etre implique en raison des contradictions dans son alibi",
    "status": "active",
    "current_score": 60.0
  }'
```

### Evaluation des hypotheses

#### Evaluation individuelle

Cliquer sur **"Re-evaluer"** a cote d'une hypothese pour forcer une reevaluation avec les dernieres preuves.

#### Evaluation globale

Cliquer sur **"Re-evaluer toutes"** pour relancer l'evaluation de toutes les hypotheses actives en une seule operation.

Le processus d'evaluation :
1. `nexus` (26B) analyse l'hypothese par rapport a TOUTES les preuves
2. Il identifie les elements supportant et contredisant
3. Il calcule un score de plausibilite
4. `deepseek-r1` (14B) verifie la coherence logique du raisonnement
5. Un snapshot est cree pour tracer l'evolution

### Comprendre les scores

| Score | Signification |
|-------|--------------|
| 80-100 | Fortement supporte par les preuves |
| 60-79 | Probablement vrai, elements solides mais incomplets |
| 40-59 | Neutre -- autant d'elements pour que contre |
| 20-39 | Peu probable, elements contradictoires forts |
| 0-19 | Quasi-refute par les preuves |

### Snapshots et evolution

Chaque evaluation cree un **snapshot** qui enregistre :
- Le score a cet instant
- Les elements supportant (JSON)
- Les elements contredisant (JSON)
- Le raisonnement complet du LLM
- Ce qui a declenche l'evaluation (manual, new_evidence, autonomous_loop)
- Le modele utilise

La page "Hypotheses" affiche un **graphique d'evolution** montrant comment les scores changent au fil du temps.

### Statuts des hypotheses

| Statut | Signification |
|--------|--------------|
| `active` | En cours d'investigation |
| `refuted` | Contredite par les preuves (archivee) |
| `confirmed` | Supportee de maniere conclusive |
| `merged` | Fusionnee avec une autre hypothese |

### Fusion d'hypotheses

Si deux hypotheses convergent, vous pouvez les fusionner :

```bash
curl -X POST http://localhost:8000/api/cases/<case_id>/hypotheses/merge \
  -H "Content-Type: application/json" \
  -d '{
    "hypothesis_ids": ["<hyp_id_1>", "<hyp_id_2>"],
    "new_title": "Implication du couple",
    "new_description": "Les deux conjoints pourraient etre impliques conjointement"
  }'
```

### Detection de contradictions

NEXUS detecte automatiquement les contradictions entre les preuves :

1. Aller sur la page **"Hypotheses"**
2. La section "Contradictions" liste les incoherences detectees
3. Chaque contradiction indique les preuves concernees et la nature de l'incoherence

Via l'API :
```bash
curl http://localhost:8000/api/cases/<case_id>/contradictions
```

### Comparaison de temoignages

Pour comparer specifiquement des temoignages :

```bash
curl -X POST http://localhost:8000/api/cases/<case_id>/compare-testimonies \
  -H "Content-Type: application/json" \
  -d '{
    "evidence_ids": ["<evidence_id_1>", "<evidence_id_2>"]
  }'
```

Le systeme identifie les points de convergence et de divergence entre les temoignages.

---

## 7. Monitoring OSINT

Le monitoring automatise surveille le web pour trouver de nouvelles informations pertinentes a l'enquete.

### Creer un job de surveillance

#### Via Streamlit

1. Aller sur la page **"Monitoring"**
2. Remplir le formulaire :
   - **Requete** : termes de recherche (ex. "Jean Dupont Lyon disparition")
   - **Type** : `searxng` (clearweb), `robin` (dark web), ou `both`
   - **Intervalle** : frequence en heures (6h par defaut pour clearweb, 24h pour dark web)
3. Cliquer sur **"Creer"**

#### Via l'API

```bash
curl -X POST http://localhost:8000/api/cases/<case_id>/monitoring \
  -H "Content-Type: application/json" \
  -d '{
    "job_type": "searxng",
    "query": "Jean Dupont Lyon disparition 2019",
    "interval_hours": 6
  }'
```

### SearXNG vs Robin

| Moteur | Type | Couverture | Frequence recommandee |
|--------|------|------------|----------------------|
| SearXNG | Clearweb | Google, DuckDuckGo, Brave, Bing, Wikipedia, Reddit, Archive.is | Toutes les 6h |
| Robin | Dark web / Tor | Sites .onion, marches, forums | Toutes les 24h |

SearXNG est un meta-moteur qui agrege les resultats de multiples moteurs de recherche. Il est configure via `searxng/settings.yml`.

Robin utilise le reseau Tor pour acceder au dark web. Il n'a pas d'API REST -- NEXUS interagit avec lui via Docker exec.

### Resultats de monitoring

Les resultats apparaissent dans la page **"Monitoring"** avec :
- Le titre et l'URL
- Le moteur source
- Un score de pertinence (0-100)
- L'indicateur "nouveau" (premiere apparition) ou "doublon" (deja vu via embeddings)

### Convertir un resultat en preuve

Quand un resultat de monitoring semble pertinent, vous pouvez le convertir en preuve :

1. Cliquer sur **"Ingerer"** a cote du resultat
2. Le resultat est transforme en preuve de type `url` avec :
   - Le titre et le snippet comme contenu
   - Le score de pertinence comme fiabilite initiale
   - La reference au job de monitoring d'origine

Via l'API :
```bash
curl -X POST http://localhost:8000/api/monitoring/results/<result_id>/ingest
```

### Execution forcee

Pour lancer un job immediatement sans attendre l'intervalle :

```bash
curl -X POST http://localhost:8000/api/monitoring/<job_id>/run
```

---

## 8. Graphe relationnel (Neo4j)

NEXUS construit un graphe de connaissances reliant toutes les entites extraites des preuves.

### Visualisation du graphe

1. Aller sur la page **"Graphe"** dans le menu lateral
2. Le graphe interactif s'affiche avec :
   - Les noeuds colores par type (personne, lieu, telephone, vehicule, etc.)
   - Les aretes representant les relations
   - La possibilite de cliquer sur un noeud pour voir ses details

### Filtrage par type

Dans la barre laterale de la page Graphe :
- Selectionner/deselectionner les types de noeuds a afficher
- Activer/desactiver la simulation physique (layout dynamique)
- Ajuster la hauteur du graphe

### Plus court chemin

Pour trouver la connexion la plus courte entre deux entites :

1. Selectionner un noeud de depart et un noeud d'arrivee
2. NEXUS calcule le plus court chemin via Neo4j et surligne la route

Via l'API :
```bash
curl http://localhost:8000/api/cases/<case_id>/graph/path/<from_id>/<to_id>
```

### Detection de clusters

NEXUS detecte les communautes (groupes fortement connectes) dans le graphe :

```bash
curl http://localhost:8000/api/cases/<case_id>/graph/clusters
```

Les clusters revelent des groupes d'entites etroitement liees qui pourraient constituer des reseaux.

### Voisinage d'un noeud

Pour explorer les connexions d'une entite specifique :

```bash
curl http://localhost:8000/api/cases/<case_id>/graph/neighbors/<node_id>?depth=2
```

Le parametre `depth` controle le nombre de sauts (1 = voisins directs, 2 = voisins des voisins).

### Types de noeuds dans Neo4j

| Type | Exemples | Couleur |
|------|----------|---------|
| Person | Noms de suspects, temoins, victimes | Configurable |
| Location | Adresses, villes, lieux-dits | Configurable |
| Phone | Numeros de telephone | Configurable |
| Vehicle | Plaques, modeles de vehicules | Configurable |
| Organization | Entreprises, associations | Configurable |
| Email | Adresses email | Configurable |
| Account | Comptes bancaires, reseaux sociaux | Configurable |
| Date | Dates mentionnees | Configurable |
| Weapon | Armes mentionnees | Configurable |
| Drug | Substances mentionnees | Configurable |

### Acces direct a Neo4j Browser

Pour des requetes Cypher avancees, utiliser Neo4j Browser a http://localhost:7474 :

```cypher
// Exemple : toutes les personnes liees a un lieu
MATCH (p:Person)-[r]-(l:Location {name: "Lyon"})
WHERE p.case_id = "<case_id>"
RETURN p, r, l

// Exemple : trouver les entites les plus connectees
MATCH (n)
WHERE n.case_id = "<case_id>"
RETURN n.name, n.entity_type, size([(n)-[]-() | 1]) AS connections
ORDER BY connections DESC
LIMIT 10
```

Identifiants : `neo4j` / `nexus2026`

---

## 9. Outils forensiques

La page **"Analyse Forensique"** donne acces a cinq modules specialises.

### 9.1 BPA -- Analyse de projections de sang (Blood Pattern Analysis)

#### Classification de pattern

1. Uploader une photo de projections de sang
2. Cliquer sur **"Classifier le pattern"**
3. Le VLM identifie le type de pattern :
   - Spatter (eclaboussure), Transfer (transfert), Drip (goutte), Pool (mare)
   - Avec score de confiance et implications forensiques

#### Analyse complete

1. Uploader la photo
2. Optionnel : ajouter des mesures (largeur/longueur des taches) et le contexte de l'enquete
3. Cliquer sur **"Analyse complete"**
4. Le systeme produit une analyse detaillee avec mecanisme probable, angle d'impact, implications

#### Calculs geometriques

- **Angle d'impact** : saisir la largeur et la longueur d'une tache de sang. NEXUS calcule l'angle d'impact via `arcsin(largeur/longueur)`.
- **Zone de convergence** : saisir les coordonnees et angles de plusieurs taches. NEXUS calcule le point d'origine.

### 9.2 Acoustique

#### Transcription

1. Uploader un fichier audio (`.wav`, `.mp3`, `.ogg`)
2. Cliquer sur **"Transcrire"**
3. `voxtral-mini:4b` transcrit le contenu audio en texte

#### Analyse forensique audio

1. Uploader le fichier audio
2. Cliquer sur **"Analyse forensique"**
3. Le systeme identifie : environnement sonore, bruits de fond, voix, evenements acoustiques

#### Detection d'evenements

Detecte et horodate les evenements sonores (coups, cris, tirs, portes, etc.) dans l'enregistrement.

### 9.3 Traces physiques

#### Analyse de trace

1. Uploader une photo de trace (empreinte, tire, outil, chaussure, etc.)
2. Selectionner le type de trace (`auto` pour detection automatique)
3. Le VLM analyse la trace et fournit une classification et des observations

#### Comparaison de traces

Uploader deux photos de traces pour les comparer et evaluer leur similarite.

### 9.4 Auto-analyse forensique

Lance une analyse forensique automatique sur **toutes** les preuves d'un dossier :
- Identifie les preuves susceptibles d'une analyse forensique
- Execute les analyses appropriees (BPA, audio, traces)
- Enregistre les resultats

### 9.5 Simulations physiques

NEXUS integre des simulations de physique pour verifier des scenarios :

#### Trajectoire de goutte de sang

Parametres : vitesse, angle, hauteur, inclinaison de la surface, proprietes du sang.
Resultat : trajectoire 3D, point d'impact, distance parcourue.

#### Pattern de cast-off (projection d'arme)

Simule les projections de sang produites par le balancement d'une arme (couteau, batte, etc.).
Parametres : rayon du mouvement, vitesse, nombre de gouttes, longueur de sang sur l'arme.

#### Propagation sonore

Simule la propagation du son d'une source (ex. coup de feu) vers des positions de temoins.
Parametres : position de la source, positions des auditeurs, puissance (dB), frequence, conditions meteorologiques, terrain.
Resultat : niveaux sonores percus par chaque temoin, delais de propagation.

#### Estimation du point d'origine

A partir de mesures de taches de sang (position, angle d'impact, direction), estime le point d'origine 3D de l'impact.

---

## 10. Carte d'investigation

La page **"Carte"** affiche une carte interactive des lieux lies a l'enquete.

### Geocodage des lieux

1. Aller sur la page **"Carte"**
2. Cliquer sur **"Geocoder les lieux"**
3. NEXUS geocode toutes les entites de type `location` via une API gratuite (1 lieu/seconde pour respecter les limites)
4. Les resultats sont mis en cache pour eviter les appels redondants

### Affichage sur la carte

Les lieux geocodes apparaissent sur une carte Folium avec :
- Des marqueurs colores selon le type de lieu :
  - Rouge : scene de crime
  - Bleu : domicile
  - Vert : lieu de travail
  - Orange : hopital
  - Violet : etablissement
  - Gris : autre
- Des popups avec les details de chaque lieu

### Verification de trajets

Pour verifier si un alibi temporel est plausible :

1. Saisir l'adresse de depart et l'adresse d'arrivee
2. Indiquer le temps declare par le temoin (en minutes)
3. Cliquer sur **"Verifier le trajet"**
4. NEXUS calcule le temps reel et compare avec le temps declare

Via l'API :
```bash
curl -X POST http://localhost:8000/api/cases/<case_id>/verify-travel \
  -H "Content-Type: application/json" \
  -d '{
    "origin": "15 rue de la Republique, Lyon",
    "destination": "Gare de Lyon Part-Dieu",
    "claimed_minutes": 10
  }'
```

Le systeme retourne si le temps declare est plausible, trop court, ou trop long.

### Calcul de route

```bash
curl -X POST http://localhost:8000/api/cases/<case_id>/route \
  -H "Content-Type: application/json" \
  -d '{
    "origin": "15 rue de la Republique, Lyon",
    "destination": "Gare de Lyon Part-Dieu"
  }'
```

---

## 11. Rapports

NEXUS genere des rapports PDF complets a partir de l'analyse d'un dossier.

### Types de rapports

| Type | Contenu |
|------|---------|
| `full` | Rapport complet : preuves, entites, hypotheses, timeline, graphe, contradictions |
| `summary` | Resume executif : hypotheses principales, pistes prioritaires |
| `timeline` | Rapport centre sur la chronologie des evenements |

### Generer un rapport (Streamlit)

1. Aller sur la page **"Rapports"** (menu lateral, ou depuis le Centre de commande)
2. Selectionner le type de rapport
3. Cliquer sur **"Generer"**
4. Le rapport est genere en arriere-plan. Rafraichir la page pour suivre l'avancement.
5. Quand le statut passe a `completed`, cliquer sur **"Telecharger"**

### Via l'API

```bash
# Lancer la generation
curl -X POST http://localhost:8000/api/cases/<case_id>/reports/generate \
  -H "Content-Type: application/json" \
  -d '{"report_type": "full"}'

# Verifier le statut
curl http://localhost:8000/api/reports/<report_id>

# Telecharger le PDF
curl -o rapport.pdf http://localhost:8000/api/reports/<report_id>/download
```

### Contenu du rapport complet

Le rapport PDF inclut :
- Page de garde avec reference du dossier et date
- Resume executif
- Chronologie des evenements cles
- Liste des preuves avec resume et fiabilite
- Entites et relations
- Hypotheses classees par plausibilite avec elements supportant/contredisant
- Contradictions detectees
- Pistes d'investigation prioritaires
- Angles morts identifies

Les rapports sont stockes dans `data/reports/` avec un nommage du type :
`nexus_full_a1b2c3d4_20260405_143000.pdf`

---

## 12. Journal d'audit

NEXUS maintient un journal d'audit **immutable** (chaine de hash) de toutes les actions effectuees.

### Consulter le journal

Le journal est accessible depuis la page **"Centre de commande"** ou la page **"Audit"** :
- Chaque entree affiche : horodatage, acteur, action, resume
- Les acteurs possibles : `user`, `system`, `autonomous_loop`, `monitoring`
- Les actions incluent : evidence_added, hypothesis_scored, entity_discovered, contradiction_found, monitoring_result, etc.

### Filtrage

Filtrer par :
- **Action** : type d'evenement
- **Acteur** : qui a declenche l'action
- **Nombre d'entrees** : limiter l'affichage

### Verification d'integrite

La chaine de hash garantit que le journal n'a pas ete modifie :

1. Cliquer sur **"Verifier l'integrite"**
2. NEXUS recalcule la chaine de hash de toutes les entrees
3. Resultat :
   - **"Chaine intacte"** : aucune falsification detectee
   - **"FALSIFICATION DETECTEE"** : indique l'entree ou la chaine est rompue

Via l'API :
```bash
curl http://localhost:8000/api/cases/<case_id>/audit/verify
```

### Export du journal

Deux formats d'export disponibles depuis le Centre de commande :

- **JSON** : export structurel complet (chaque entree avec tous ses champs)
- **Markdown** : format lisible pour inclusion dans un rapport

---

## 13. Recherche semantique

ChromaDB permet de rechercher dans les preuves par similarite de sens (pas seulement par mots-cles).

### Recherche par texte

Depuis la page appropriee ou via l'API :

```bash
curl -X POST http://localhost:8000/api/cases/<case_id>/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "problemes financiers avant la disparition",
    "n_results": 10,
    "collection": "evidence"
  }'
```

La recherche retourne les preuves dont le contenu est semantiquement proche de la requete, meme si les mots exacts ne sont pas presents.

### Trouver des preuves similaires

Pour trouver les preuves similaires a une preuve donnee :

```bash
curl http://localhost:8000/api/cases/<case_id>/similar/<evidence_id>?n_results=5
```

### Detection de doublons

```bash
curl http://localhost:8000/api/cases/<case_id>/duplicates?threshold=0.92
```

Identifie les preuves quasi-identiques (seuil de similarite configurable, 0.92 par defaut).

---

## 14. OSINT Recon

Le module de reconnaissance OSINT effectue des verifications sur les entites du dossier.

### Types de recon

| Type | Ce qu'il verifie |
|------|-----------------|
| Email | Comptes en ligne lies a l'adresse (via holehe) |
| Username | Presence sur les plateformes sociales |
| Domaine | WHOIS, DNS, informations d'hebergement |

### Lancer une recon manuelle

Via l'API :

```bash
# Recon sur une adresse email
curl -X POST http://localhost:8000/api/recon/email/suspect@example.com

# Recon sur un nom d'utilisateur
curl -X POST http://localhost:8000/api/recon/username/jean_dupont_69

# Recon sur un domaine
curl -X POST http://localhost:8000/api/recon/domain/example.com
```

### Recon automatique

Lancer une recon automatique sur toutes les entites email et username du dossier :

```bash
curl -X POST http://localhost:8000/api/cases/<case_id>/recon/auto
```

La boucle autonome execute aussi ces recons automatiquement si `AUTO_OSINT_RECON=true` dans la configuration.

---

## 15. Vision et analyse d'images

NEXUS analyse les images via des modeles de vision (VLM).

### Analyse d'une preuve image

```bash
curl -X POST http://localhost:8000/api/evidence/<evidence_id>/analyze-image
```

Le VLM (`gemma4:e4b` ou `qwen3-vl:8b` pour l'analyse approfondie) decrit le contenu de l'image, identifie les objets, personnes, lieux, et extrait les informations pertinentes.

### Analyse de toutes les images d'un dossier

```bash
curl -X POST http://localhost:8000/api/cases/<case_id>/analyze-images
```

### Recherche visuelle

NEXUS indexe les images avec DINOv2 (similarite visuelle) et CLIP (recherche texte-image) :

```bash
# Recherche par texte
curl -X POST http://localhost:8000/api/cases/<case_id>/images/search-by-text \
  -H "Content-Type: application/json" \
  -d '{"query": "voiture noire garee", "n_results": 5}'

# Recherche par similarite visuelle
curl -X POST http://localhost:8000/api/cases/<case_id>/images/search-by-image \
  -H "Content-Type: application/json" \
  -d '{"evidence_id": "<evidence_id>", "n_results": 5}'
```

### Comparaison d'images

Comparer visuellement deux preuves images :

```bash
curl -X POST http://localhost:8000/api/vision/compare \
  -d "evidence_id_1=<id1>&evidence_id_2=<id2>"
```

---

## 16. Reference API

Tous les endpoints de l'API REST FastAPI sont documentes automatiquement.

### Documentation interactive

Ouvrir dans le navigateur :

- **Swagger UI** : http://localhost:8000/docs
- **ReDoc** : http://localhost:8000/redoc

Ces interfaces permettent de tester chaque endpoint directement depuis le navigateur.

### Principaux groupes d'endpoints

| Prefixe | Description |
|---------|-------------|
| `/api/cases` | CRUD dossiers d'investigation |
| `/api/cases/{id}/evidence` | Gestion des preuves |
| `/api/cases/{id}/entities` | Entites extraites |
| `/api/cases/{id}/hypotheses` | Hypotheses et evaluation |
| `/api/cases/{id}/analyze` | Declenchement d'analyses |
| `/api/cases/{id}/graph` | Graphe Neo4j (noeuds, chemins, clusters) |
| `/api/cases/{id}/search` | Recherche semantique ChromaDB |
| `/api/cases/{id}/monitoring` | Jobs de surveillance |
| `/api/cases/{id}/reports` | Generation de rapports |
| `/api/cases/{id}/audit` | Journal d'audit |
| `/api/cases/{id}/investigation` | Boucle autonome |
| `/api/cases/{id}/geocode` | Geocodage des lieux |
| `/api/cases/{id}/recon` | OSINT recon |
| `/api/forensics` | Outils forensiques (BPA, audio, traces, simulations) |
| `/api/vision` | Analyse d'images VLM |
| `/api/health` | Health check systeme |

### Codes de reponse

| Code | Signification |
|------|--------------|
| 200 | Succes |
| 201 | Ressource creee |
| 202 | Tache lancee en arriere-plan |
| 204 | Suppression reussie |
| 404 | Ressource non trouvee |
| 409 | Conflit (ex. rapport pas encore pret) |
| 503 | Service indisponible (Ollama, scheduler, etc.) |

---

## Annexe : Navigation dans le dashboard

| # | Page | Fonction |
|---|------|----------|
| 01 | Tableau de bord | Vue d'ensemble du dossier actif, statistiques |
| 02 | Preuves | Upload, gestion et consultation des preuves |
| 03 | Entites | Liste et details des entites extraites |
| 04 | Hypotheses | Gestion, evaluation et evolution des hypotheses |
| 05 | Chronologie | Timeline interactive des evenements |
| 06 | Graphe | Visualisation Neo4j interactive |
| 07 | Monitoring | Jobs de surveillance et resultats |
| 08 | Alertes | Centre de notifications |
| 09 | Analyse | Declenchement et historique des analyses |
| 10 | Carte | Carte geographique des lieux |
| 11 | OSINT | Reconnaissance email/username/domaine |
| 12 | Vision | Analyse d'images et recherche visuelle |
| 13 | Forensique | BPA, acoustique, traces, simulations |
| 14 | Centre de commande | Investigation autonome, audit, controle |
| 15 | Audit | Journal d'audit detaille |
