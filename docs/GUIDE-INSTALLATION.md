# NEXUS -- Guide d'installation

**Version :** 0.1.0
**Date :** 2026-04-05
**Plateforme :** Windows 11, GPU NVIDIA (16 GB+ VRAM recommande)

---

## Table des matieres

1. [Prerequis](#1-prerequis)
2. [Installation d'Ollama et des modeles](#2-installation-dollama-et-des-modeles)
3. [Installation des services Docker](#3-installation-des-services-docker)
4. [Installation de SearXNG (separe)](#4-installation-de-searxng-separe)
5. [Installation de Python et des dependances](#5-installation-de-python-et-des-dependances)
6. [Configuration du fichier .env](#6-configuration-du-fichier-env)
7. [Lancement de NEXUS](#7-lancement-de-nexus)
8. [Verification de l'installation](#8-verification-de-linstallation)
9. [Troubleshooting](#9-troubleshooting)

---

## 1. Prerequis

### Materiel requis

| Composant | Minimum | Recommande |
|-----------|---------|------------|
| OS | Windows 11 64-bit (23H2+) | Windows 11 Pro |
| GPU | NVIDIA avec 8 GB VRAM | NVIDIA RTX 5080 (16 GB VRAM) |
| RAM | 16 GB | 32 GB |
| Stockage | 50 GB libres | 100 GB+ SSD NVMe |
| CPU | 64-bit avec SLAT | 8+ coeurs |

### Logiciels a installer

Avant de commencer, installez les logiciels suivants dans cet ordre :

#### 1.1 Git

Telecharger et installer depuis https://git-scm.com/downloads/win

Verifier l'installation :

```bash
git --version
```

#### 1.2 Docker Desktop

1. Activer la virtualisation dans le BIOS/UEFI (Intel VT-x ou AMD-V)
2. Activer WSL 2 (Windows Subsystem for Linux) :

```powershell
# Dans PowerShell en administrateur
wsl --install
```

3. Telecharger Docker Desktop depuis https://www.docker.com/products/docker-desktop/
4. Installer et redemarrer Windows si demande
5. Verifier :

```bash
docker --version
docker compose version
```

#### 1.3 Conda (Miniconda recommande)

Telecharger Miniconda depuis https://docs.conda.io/en/latest/miniconda.html

Choisir l'installeur Windows 64-bit. Suivre les instructions de l'installeur.

Verifier :

```bash
conda --version
```

#### 1.4 Ollama

Telecharger et installer depuis https://ollama.com/download

L'installeur Windows configure automatiquement le service Ollama sur le port 11434.

Verifier :

```bash
ollama --version
```

#### 1.5 Drivers NVIDIA

S'assurer d'avoir les derniers drivers NVIDIA installes depuis https://www.nvidia.com/fr-fr/drivers/

Ollama detecte automatiquement le GPU NVIDIA si les drivers CUDA sont presents.

---

## 2. Installation d'Ollama et des modeles

NEXUS utilise plusieurs modeles LLM avec des roles distincts. Chaque modele doit etre telecharge avant utilisation.

### 2.1 Telecharger les modeles depuis le registre Ollama

Ouvrir un terminal et executer ces commandes. Chaque telechargement peut prendre plusieurs minutes selon la connexion.

```bash
# Modele rapide -- extraction d'entites, filtrage, reformulation (~3 GB)
ollama pull gemma4:e4b

# Modele de raisonnement -- verification logique, chain-of-thought (~8 GB)
ollama pull huihui_ai/deepseek-r1-abliterated:14b

# Embeddings vectoriels (~274 MB)
ollama pull nomic-embed-text

# Transcription audio/video (~2.5 GB)
ollama pull voxtral-mini:4b

# Vision avancee (~5 GB)
ollama pull qwen3-vl:8b
```

### 2.2 Telecharger le modele de base pour nexus

Le modele principal de NEXUS est base sur Gemma 4 26B Heretic (uncensored). Le fichier GGUF est stocke dans `models/`.

Si le fichier GGUF n'est pas deja present, il faut le telecharger manuellement depuis la source HuggingFace et le placer dans `models/gemma-4-26b-a4b-it-heretic.q4_k_s.gguf`.

### 2.3 Creer le modele personnalise nexus

Le modele `nexus` est un modele personnalise avec un prompt systeme specialise pour l'analyse de cold cases. Il est defini dans le fichier `Modelfile.gemma4-heretic` a la racine du projet.

```bash
# Depuis la racine du projet NEXUS
cd C:\Users\<votre_user>\Documents\Code\nexus

# Creer le modele nexus a partir du Modelfile
ollama create nexus -f Modelfile.gemma4-heretic
```

Cette commande lit le fichier `Modelfile.gemma4-heretic` qui :
- Pointe vers le fichier GGUF local (`models/gemma-4-26b-a4b-it-heretic.q4_k_s.gguf`)
- Configure la temperature a 0.3, le contexte a 32768 tokens
- Injecte le prompt systeme d'analyste criminel impartial

### 2.4 Verifier les modeles installes

```bash
ollama list
```

Vous devez voir au minimum :

```
NAME                                       ID           SIZE    MODIFIED
nexus:latest                               ...          ~15 GB  ...
gemma4:e4b                                 ...          ~3 GB   ...
huihui_ai/deepseek-r1-abliterated:14b      ...          ~8 GB   ...
nomic-embed-text:latest                    ...          ~274 MB ...
voxtral-mini:4b                            ...          ~2.5 GB ...
qwen3-vl:8b                               ...          ~5 GB   ...
```

### 2.5 Tester un modele

```bash
ollama run gemma4:e4b "Bonjour, reponds en une phrase."
```

Si le modele repond, Ollama fonctionne correctement avec le GPU.

---

## 3. Installation des services Docker

NEXUS utilise Docker Compose pour faire tourner trois services : Neo4j, ChromaDB et Robin.

### 3.1 Demarrer les services

Depuis la racine du projet :

```bash
cd C:\Users\<votre_user>\Documents\Code\nexus

docker compose up -d
```

Cette commande telecharge les images Docker et demarre les conteneurs en arriere-plan.

### 3.2 Verification des services

Attendre 30 secondes apres le lancement, puis verifier :

```bash
docker compose ps
```

Les trois conteneurs doivent etre en etat `running` :

```
NAME             IMAGE                    STATUS          PORTS
nexus-neo4j      neo4j:5-community        Up (healthy)    7474->7474, 7687->7687
nexus-chromadb   chromadb/chroma:latest   Up (healthy)    8100->8000
nexus-robin      apurvsg/robin:latest     Up              8502->8501
```

### 3.3 Verification individuelle

#### Neo4j (graphe de connaissances)

Ouvrir dans le navigateur : http://localhost:7474

- Interface : Neo4j Browser
- Identifiants par defaut : `neo4j` / `nexus2026`
- Le plugin APOC est installe automatiquement via `NEO4J_PLUGINS='["apoc"]'`
- La memoire heap est configuree : 512 MB initial, 1 GB max

Se connecter avec les identifiants et verifier que le prompt Cypher apparait.

#### ChromaDB (base vectorielle)

Tester le endpoint heartbeat :

```bash
curl http://localhost:8100/api/v2/heartbeat
```

Reponse attendue (JSON avec un timestamp) :

```json
{"nanosecond heartbeat": 1712345678901234567}
```

L'API ChromaDB v2 est exposee sur le port 8100 (le port interne 8000 est reserve a FastAPI).

#### Robin (recherche dark web via Tor)

Ouvrir dans le navigateur : http://localhost:8502

Robin expose une interface Streamlit sur le port 8502 (8501 interne, remape pour eviter le conflit avec le Streamlit de NEXUS).

**Note :** Robin n'a pas d'API REST. L'interaction se fait soit via son UI Streamlit, soit via `docker exec` en CLI.

### 3.4 Volumes de donnees

Les donnees des services Docker sont persistees dans `data/` :

```
data/
  neo4j/data/   -- Donnees Neo4j
  neo4j/logs/   -- Logs Neo4j
  chroma/       -- Donnees ChromaDB
  robin/        -- Investigations Robin
```

Ces dossiers sont crees automatiquement au premier lancement.

---

## 4. Installation de SearXNG (separe)

SearXNG est le moteur de recherche clearweb utilise par NEXUS pour le monitoring automatise. Il tourne separement de Docker Compose car il est souvent deja installe sur la machine.

### 4.1 Si SearXNG n'est pas encore installe

La methode recommandee est d'utiliser Docker :

```bash
docker run -d \
  --name searxng \
  -p 8888:8888 \
  -v "C:\Users\<votre_user>\Documents\Code\nexus\searxng\settings.yml:/etc/searxng/settings.yml" \
  -v "C:\Users\<votre_user>\Documents\Code\nexus\searxng\limiter.toml:/etc/searxng/limiter.toml" \
  --restart unless-stopped \
  searxng/searxng:latest
```

### 4.2 Verification

Ouvrir dans le navigateur : http://localhost:8888

L'interface de recherche SearXNG doit s'afficher. Les moteurs configures dans `searxng/settings.yml` incluent :
- Google, DuckDuckGo, Brave, Bing (moteurs principaux)
- Wikipedia, Wikidata (encyclopedies)
- Archive.is, Reddit (archives et communautes)

### 4.3 Tester l'API JSON

NEXUS utilise l'API JSON de SearXNG, pas l'interface web :

```bash
curl "http://localhost:8888/search?q=test&format=json"
```

La reponse doit etre un JSON contenant des resultats de recherche.

---

## 5. Installation de Python et des dependances

### 5.1 Creer l'environnement conda

```bash
conda create -n nexus python=3.13 -y
conda activate nexus
```

### 5.2 Installer les dependances Python

Depuis la racine du projet :

```bash
cd C:\Users\<votre_user>\Documents\Code\nexus

pip install -r requirements.txt
```

L'installation peut prendre plusieurs minutes. Les dependances principales sont :

| Categorie | Bibliotheques |
|-----------|---------------|
| Backend | FastAPI, uvicorn, pydantic, httpx |
| Bases de donnees | neo4j, chromadb, aiosqlite |
| LLM | ollama (client Python) |
| Ingestion | PyMuPDF (PDF), Pillow (images) |
| Frontend | streamlit, streamlit-agraph, streamlit-folium, plotly |
| Export | weasyprint (PDF), Jinja2 |
| OSINT | python-whois, holehe |
| Vision | transformers, torch, torchvision |
| Physique | scipy |
| Planification | APScheduler |
| Utilitaires | loguru, tenacity |

### 5.3 Verifier l'installation

```bash
python -c "import fastapi; import streamlit; import ollama; import neo4j; import chromadb; print('OK')"
```

### 5.4 Note sur PyTorch et CUDA

Pour beneficier de l'acceleration GPU sur les modeles de vision (DINOv2, CLIP) :

```bash
# Si pip a installe la version CPU de PyTorch, reinstaller avec CUDA
pip install torch torchvision --index-url https://download.pytorch.org/whl/cu124
```

Verifier :

```bash
python -c "import torch; print(f'CUDA: {torch.cuda.is_available()}, GPU: {torch.cuda.get_device_name(0) if torch.cuda.is_available() else None}')"
```

---

## 6. Configuration du fichier .env

Le fichier `.env` a la racine du projet contient la configuration de NEXUS. Un fichier `.env` par defaut est fourni.

### 6.1 Contenu du fichier .env

```env
# === NEXUS Configuration ===

# ---- FastAPI ----
NEXUS_HOST=0.0.0.0          # Adresse d'ecoute (0.0.0.0 = toutes les interfaces)
NEXUS_PORT=8000              # Port du backend API
NEXUS_DEBUG=true             # Mode debug (true en dev, false en prod)

# ---- Ollama ----
OLLAMA_BASE_URL=http://localhost:11434  # URL du serveur Ollama

# ---- Modeles (optionnel, valeurs par defaut dans config.py) ----
# MODEL_FAST=gemma4:e4b                              # Taches rapides
# MODEL_REASONING=huihui_ai/deepseek-r1-abliterated:14b  # Raisonnement
# MODEL_DEEP=nexus                                   # Analyse profonde
# MODEL_EMBEDDING=nomic-embed-text                   # Embeddings
# MODEL_AUDIO=voxtral-mini:4b                        # Transcription audio
# MODEL_VISION=gemma4:e4b                            # Vision rapide
# MODEL_VISION_DEEP=qwen3-vl:8b                      # Vision avancee

# ---- Neo4j ----
NEO4J_URI=bolt://localhost:7687      # Protocole Bolt pour les requetes
NEO4J_USER=neo4j                     # Utilisateur Neo4j
NEO4J_PASSWORD=nexus2026             # Mot de passe (changer en production)

# ---- ChromaDB ----
CHROMA_HOST=localhost                # Host ChromaDB
CHROMA_PORT=8100                     # Port externe ChromaDB

# ---- SearXNG (clearweb) ----
SEARXNG_URL=http://localhost:8888    # URL de l'instance SearXNG

# ---- Robin (dark web / Tor) ----
ROBIN_URL=http://localhost:9090      # URL de Robin

# ---- Stockage ----
DATA_DIR=./data                      # Repertoire racine des donnees
UPLOAD_DIR=./data/uploads            # Fichiers uploades
SQLITE_PATH=./data/nexus.db          # Base SQLite

# ---- Intervalles de monitoring (secondes) ----
# CLEARWEB_INTERVAL=21600            # 6 heures (defaut)
# DARKWEB_INTERVAL=86400             # 24 heures (defaut)
```

### 6.2 Variables importantes a personnaliser

| Variable | Quand la changer |
|----------|-----------------|
| `NEO4J_PASSWORD` | Toujours changer en production |
| `NEXUS_DEBUG` | Mettre `false` en production |
| `OLLAMA_BASE_URL` | Si Ollama tourne sur une autre machine |
| `CLEARWEB_INTERVAL` | Ajuster la frequence du monitoring clearweb |
| `DARKWEB_INTERVAL` | Ajuster la frequence du monitoring dark web |

### 6.3 Fichier robin.env

Le fichier `robin.env` configure Robin pour communiquer avec Ollama via Docker :

```env
OLLAMA_BASE_URL=http://host.docker.internal:11434
```

`host.docker.internal` permet au conteneur Robin d'acceder a Ollama sur la machine hote.

---

## 7. Lancement de NEXUS

### 7.1 Ordre de demarrage

NEXUS necessite que les services suivants soient demarres dans cet ordre :

1. **Ollama** (demarre automatiquement au demarrage de Windows)
2. **Docker** (Neo4j, ChromaDB, Robin)
3. **SearXNG** (conteneur Docker separe)
4. **FastAPI** (backend NEXUS)
5. **Streamlit** (frontend NEXUS)

### 7.2 Demarrer le backend FastAPI

Ouvrir un premier terminal :

```bash
conda activate nexus
cd C:\Users\<votre_user>\Documents\Code\nexus

uvicorn nexus.main:app --host 0.0.0.0 --port 8000 --reload
```

Le flag `--reload` active le rechargement automatique en cas de modification du code (mode developpement uniquement).

Au demarrage, NEXUS :
- Initialise la base SQLite (`data/nexus.db`)
- Cree les repertoires necessaires (`data/uploads/`, `data/reports/`, `data/backups/`)
- Se connecte a Ollama, Neo4j et ChromaDB
- Demarre le scheduler de monitoring (APScheduler)
- Demarre le gestionnaire d'investigation autonome

Les logs s'affichent via Loguru dans le terminal.

### 7.3 Demarrer le frontend Streamlit

Ouvrir un second terminal :

```bash
conda activate nexus
cd C:\Users\<votre_user>\Documents\Code\nexus

streamlit run frontend/app.py --server.port 8501
```

Streamlit detecte automatiquement le dossier `frontend/pages/` et charge les 15 pages du dashboard.

### 7.4 Acceder a l'interface

Ouvrir dans le navigateur : http://localhost:8501

Le dashboard NEXUS s'affiche avec :
- La barre laterale (selection de dossier, indicateur de sante API)
- La page d'accueil (creation de dossier, apercu du systeme)

---

## 8. Verification de l'installation

### 8.1 Health check API

```bash
curl http://localhost:8000/api/health
```

Reponse attendue :

```json
{
  "status": "ok",
  "version": "0.1.0",
  "sqlite": "data/nexus.db",
  "ollama": "http://localhost:11434"
}
```

### 8.2 Verification complete des services

| Service | URL / Commande | Resultat attendu |
|---------|----------------|------------------|
| Ollama | `ollama list` | Liste des modeles installes |
| Neo4j Browser | http://localhost:7474 | Interface web Neo4j |
| ChromaDB | `curl http://localhost:8100/api/v2/heartbeat` | JSON avec timestamp |
| SearXNG | http://localhost:8888 | Interface de recherche |
| Robin | http://localhost:8502 | Interface Streamlit Robin |
| FastAPI | http://localhost:8000/api/health | JSON status ok |
| Streamlit | http://localhost:8501 | Dashboard NEXUS |

### 8.3 Tester la chaine complete

1. Ouvrir http://localhost:8501
2. Creer un dossier test ("Test d'installation")
3. Le dossier doit apparaitre dans le selecteur de la barre laterale
4. L'indicateur API doit etre vert ("API : ok")

Si tout fonctionne, NEXUS est pret a l'emploi.

---

## 9. Troubleshooting

### 9.1 Neo4j ne demarre pas

**Symptome :** Le conteneur `nexus-neo4j` redemarre en boucle.

**Causes possibles :**

1. **Port deja utilise :**
   ```bash
   netstat -ano | findstr :7474
   netstat -ano | findstr :7687
   ```
   Si un autre processus utilise ces ports, l'arreter ou modifier les ports dans `docker-compose.yml`.

2. **Memoire insuffisante :**
   Neo4j est configure avec 512 MB - 1 GB de heap. Si Docker Desktop a une limite memoire trop basse, augmenter la memoire allouee dans Docker Desktop > Settings > Resources.

3. **Plugin APOC non telechargeable :**
   Verifier la connexion Internet du conteneur. Alternativement, telecharger le JAR APOC manuellement et le monter dans `/plugins`.

4. **Donnees corrompues :**
   ```bash
   docker compose down
   rm -rf data/neo4j/data/*
   docker compose up -d neo4j
   ```

**Voir les logs :**
```bash
docker logs nexus-neo4j --tail 50
```

### 9.2 ChromaDB : erreur API v1 vs v2

**Symptome :** Erreur `404 Not Found` en accedant a ChromaDB.

**Cause :** Les versions recentes de ChromaDB utilisent l'API v2 (`/api/v2/`). Le healthcheck dans `docker-compose.yml` est deja configure pour v2.

**Verification :**
```bash
# API v2 (correct)
curl http://localhost:8100/api/v2/heartbeat

# API v1 (obsolete, ne pas utiliser)
# curl http://localhost:8100/api/v1/heartbeat
```

Si vous avez une ancienne version du client Python chromadb, mettre a jour :
```bash
pip install --upgrade chromadb
```

### 9.3 Ollama OOM (Out of Memory)

**Symptome :** Erreur `CUDA error: out of memory` ou le modele se fige.

**Causes et solutions :**

1. **Modele trop gros pour la VRAM :**
   Le modele `nexus` (26B Q4) necessite environ 15 GB de VRAM. Sur un GPU 16 GB, il laisse peu de marge.

   **Solution :** Ne pas charger plusieurs modeles en parallele. NEXUS serialise les appels GPU automatiquement.

2. **Contexte trop long :**
   Ollama pre-alloue la memoire KV cache pour le contexte declare (32768 tokens). Reduire si necessaire :
   ```bash
   # Dans le Modelfile, reduire num_ctx
   PARAMETER num_ctx 16384
   # Puis recreer le modele
   ollama create nexus -f Modelfile.gemma4-heretic
   ```

3. **Modele precedent encore en memoire :**
   Ollama garde les modeles en VRAM par defaut. Forcer le dechargement :
   ```bash
   # Verifier les modeles en memoire
   ollama ps

   # Arreter un modele specifique (utiliser son nom)
   ollama stop nexus
   ```

4. **Applications GPU en arriere-plan :**
   Fermer les navigateurs avec acceleration materielle, les jeux, les autres applications GPU.

5. **Verifier l'utilisation VRAM :**
   ```bash
   nvidia-smi
   ```

### 9.4 Ports deja utilises

**Symptome :** Erreur `address already in use` au lancement.

**Ports utilises par NEXUS :**

| Port | Service |
|------|---------|
| 8000 | FastAPI (backend) |
| 8501 | Streamlit (frontend) |
| 11434 | Ollama |
| 7474 | Neo4j Browser |
| 7687 | Neo4j Bolt |
| 8100 | ChromaDB |
| 8502 | Robin |
| 8888 | SearXNG |

**Trouver le processus qui occupe un port :**
```bash
netstat -ano | findstr :<PORT>
# Puis identifier le processus
tasklist | findstr <PID>
```

**Tuer le processus :**
```bash
taskkill /PID <PID> /F
```

### 9.5 Streamlit ne voit pas l'API

**Symptome :** Message "API injoignable" dans la barre laterale Streamlit.

**Causes :**
1. Le backend FastAPI n'est pas demarre. Le lancer d'abord.
2. Le backend ecoute sur un port different. Verifier le port dans `.env` (`NEXUS_PORT`).
3. Le frontend cherche l'API sur `http://localhost:8000` par defaut (code dans `frontend/api_client.py`).

### 9.6 Erreurs torch / CUDA

**Symptome :** `RuntimeError: CUDA error` ou `torch.cuda.is_available()` retourne `False`.

**Solutions :**
1. Verifier les drivers NVIDIA :
   ```bash
   nvidia-smi
   ```
2. Reinstaller PyTorch avec support CUDA :
   ```bash
   pip install torch torchvision --index-url https://download.pytorch.org/whl/cu124
   ```
3. Redemarrer apres mise a jour des drivers.

### 9.7 WeasyPrint (export PDF) ne s'installe pas

**Symptome :** Erreur a l'installation de `weasyprint` sur Windows.

WeasyPrint necessite GTK sur Windows. Installer via :
```bash
# Option 1: via conda (recommande)
conda install -c conda-forge weasyprint

# Option 2: installer les dependances GTK manuellement
# Telecharger MSYS2 depuis https://www.msys2.org/
# Puis: pacman -S mingw-w64-x86_64-gtk3
```

### 9.8 Robin ne se connecte pas a Ollama

**Symptome :** Robin ne peut pas utiliser les LLMs.

Verifier le fichier `robin.env` :
```env
OLLAMA_BASE_URL=http://host.docker.internal:11434
```

`host.docker.internal` est un alias Docker qui pointe vers la machine hote. Si Ollama ecoute bien sur `0.0.0.0:11434`, la connexion doit fonctionner.

---

## Annexe : Resume des commandes

```bash
# === Demarrage complet de NEXUS ===

# 1. Verifier que Docker Desktop est lance
docker compose ps

# 2. Demarrer les services Docker (si pas encore fait)
docker compose up -d

# 3. Verifier SearXNG
curl http://localhost:8888/search?q=test&format=json

# 4. Demarrer le backend (terminal 1)
conda activate nexus
uvicorn nexus.main:app --host 0.0.0.0 --port 8000 --reload

# 5. Demarrer le frontend (terminal 2)
conda activate nexus
streamlit run frontend/app.py --server.port 8501

# === Arret propre ===

# Ctrl+C dans les terminaux FastAPI et Streamlit
docker compose down     # Arrete Neo4j, ChromaDB, Robin
docker stop searxng     # Arrete SearXNG
```
