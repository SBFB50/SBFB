# Guide nexus-worker — Client GPU contributeur

**Version**: 0.1.0  
**Licence**: AGPL-3.0  
**Python requis**: >= 3.10

Le client `nexus-worker` permet a tout detenteur de GPU de contribuer sa puissance de calcul au reseau NEXUS GOV. Le worker tourne en arriere-plan, recoit des taches LLM du serveur central, les execute via Ollama local, et renvoie les resultats signes cryptographiquement.

---

## Table des matieres

1. [Installation](#1-installation)
2. [Demarrage rapide](#2-demarrage-rapide)
3. [Commandes CLI](#3-commandes-cli)
4. [Configuration](#4-configuration)
5. [Detection GPU](#5-detection-gpu)
6. [Modes de fonctionnement](#6-modes-de-fonctionnement)
7. [Dashboard Rich TUI](#7-dashboard-rich-tui)
8. [Signature Ed25519](#8-signature-ed25519)
9. [Etats du moteur](#9-etats-du-moteur)
10. [Depannage](#10-depannage)

---

## 1. Installation

### Installation de base

```bash
pip install nexus-worker
```

Ceci installe les 3 dependances obligatoires :

| Dependance | Version | Role |
|---|---|---|
| `httpx` | >= 0.27 | Client HTTP async (communication serveur) |
| `loguru` | >= 0.7 | Logging structure |
| `rich` | >= 13.0 | Dashboard TUI et sortie formatee |

### Extras optionnels

```bash
# Detection GPU precise via pynvml (recommande pour NVIDIA)
pip install nexus-worker[nvidia]
# Installe: nvidia-ml-py >= 12.0

# Signature cryptographique Ed25519 des resultats
pip install cryptography
```

### Prerequis systeme

- **Ollama** installe et accessible sur `http://localhost:11434` (telecharger sur [ollama.com](https://ollama.com))
- **Un GPU** avec VRAM suffisante (le worker refuse de s'enregistrer si `vram_mb == 0`)
- **Docker** (uniquement si mode `--exo` ou `--petals`)

---

## 2. Demarrage rapide

Deux commandes suffisent :

```bash
# 1. Enregistrement aupres du serveur NEXUS
nexus-worker register --server https://nexusgov.fr --name "MonPseudo"

# 2. Demarrage du worker (avec dashboard Rich)
nexus-worker start
```

L'enregistrement :
- Detecte automatiquement votre GPU et la VRAM
- Verifie que Ollama est accessible
- Genere une paire de cles Ed25519 (si `cryptography` est installe)
- Envoie les informations au serveur et recoit un `node_id` + `api_key`
- Sauvegarde la configuration dans `~/.nexus-worker/config.json`

Le demarrage :
- Lance la boucle heartbeat (toutes les 15s)
- Attend les instructions de modele du serveur
- Pull automatiquement le modele requis via Ollama
- Commence a traiter les taches LLM
- Affiche le dashboard Rich en temps reel

---

## 3. Commandes CLI

### Syntaxe globale

```
nexus-worker [--version] [-v|--verbose] <commande> [options]
```

| Flag global | Description |
|---|---|
| `--version` | Affiche la version (`nexus-worker 0.1.0`) |
| `-v`, `--verbose` | Active le logging DEBUG (defaut : INFO) |

### 3.1 `register` — Enregistrement

```bash
nexus-worker register --server URL --name NOM
```

| Argument | Requis | Description |
|---|---|---|
| `--server` | Oui | URL du serveur NEXUS (ex: `https://nexusgov.fr`) |
| `--name` | Oui | Nom d'affichage du contributeur |

**Processus d'enregistrement** :

1. Detection GPU (voir [section 5](#5-detection-gpu))
2. Detection Ollama (requete `GET http://localhost:11434/api/version`)
3. Generation keypair Ed25519 (si `cryptography` est installe)
4. Appel `POST /api/compute/register` avec : `name`, `gpu_model`, `vram_mb`, `platform`, `ollama_version`, `public_key_pem`
5. Le serveur retourne `node_id` et `api_key`
6. Sauvegarde dans `~/.nexus-worker/config.json`

**Sortie** :

```
NEXUS Worker Registration

  Detecting GPU...
  GPU: NVIDIA GeForce RTX 5080 (16 GB)
  Platform: windows
  Ollama: v0.6.2
  Ed25519: keypair generated

  Registering with https://nexusgov.fr...

  Registration successful!
  Node ID: a1b2c3d4e5f6...
  Config saved to: ~/.nexus-worker/config.json

  Run nexus-worker start to begin contributing.
```

### 3.2 `start` — Demarrer le worker

```bash
nexus-worker start [--no-dashboard] [--exo] [--petals] [--sync]
```

| Flag | Description |
|---|---|
| `--no-dashboard` | Desactive le dashboard Rich TUI. Le worker tourne en mode simple (logs uniquement) jusqu'a Ctrl+C |
| `--exo` | Active le mode exo peer : contribue des couches GPU a un modele distribue au lieu d'executer Ollama localement. Requiert `pip install exo` |
| `--petals` | Active le mode Petals server : heberge des blocs transformer pour un modele distribue (port 31330). Requiert `pip install petals` |
| `--sync` | Active la synchronisation DB locale en temps reel via cr-sqlite WebSocket. Cree une copie locale dans `~/.nexus-worker/nexus_local.db` |

**Prerequis** : le worker doit etre enregistre (`api_key` presente dans la config). Sinon, le message suivant s'affiche :

```
Not registered. Run: nexus-worker register --server URL --name NAME
```

### 3.3 `stats` — Statistiques reseau

```bash
nexus-worker stats
```

Aucun argument. Affiche :

- **Statistiques reseau** : nombre de noeuds en ligne, VRAM totale, modele actif, taches du jour, taches en attente/assignees
- **Etat de transition** de modele (si une migration est en cours, avec pourcentage de readiness)
- **Leaderboard** : top 20 contributeurs avec rang, nom, GPU, nombre de taches, vitesse moyenne (tokens/s). Votre nom est surligne en vert

**Endpoints appeles** :
- `GET /api/compute/stats`
- `GET /api/compute/leaderboard?limit=20`
- `GET /api/compute/model/status`

### 3.4 `config` — Afficher la configuration

```bash
nexus-worker config
```

Aucun argument. Affiche les champs principaux de la configuration locale :

```
NEXUS Worker Config
  Path: ~/.nexus-worker/config.json
  Server: https://nexusgov.fr
  Name: MonPseudo
  Node ID: a1b2c3d4e5f6...
  API Key: ***
  GPU: NVIDIA GeForce RTX 5080 (16 GB)
  Ollama URL: http://localhost:11434
```

La cle API est masquee dans l'affichage.

---

## 4. Configuration

### Emplacement

```
~/.nexus-worker/config.json
```

Le repertoire `~/.nexus-worker/` est cree automatiquement lors du premier `register`.

### Structure complete

```json
{
  "server_url": "https://nexusgov.fr",
  "node_id": "uuid-du-noeud",
  "api_key": "cle-api-secrete",
  "name": "MonPseudo",
  "gpu_model": "NVIDIA GeForce RTX 5080",
  "vram_mb": 16384,
  "platform": "windows",
  "ollama_url": "http://localhost:11434",
  "poll_interval": 2.0,
  "heartbeat_interval": 15.0,
  "private_key_pem": "-----BEGIN PRIVATE KEY-----\n..."
}
```

### Champs

| Champ | Type | Defaut | Description |
|---|---|---|---|
| `server_url` | string | `""` | URL du serveur NEXUS (defini lors du `register`) |
| `node_id` | string | `""` | UUID unique attribue par le serveur |
| `api_key` | string | `""` | Cle API pour l'authentification Bearer |
| `name` | string | `""` | Nom d'affichage du contributeur |
| `gpu_model` | string | `""` | Nom du GPU detecte (ex: `NVIDIA GeForce RTX 5080`) |
| `vram_mb` | int | `0` | VRAM totale en megaoctets |
| `platform` | string | `""` | Systeme d'exploitation (`windows`, `linux`, `darwin`) |
| `ollama_url` | string | `"http://localhost:11434"` | URL de l'instance Ollama locale |
| `poll_interval` | float | `2.0` | Intervalle en secondes entre les tentatives de recuperation de tache |
| `heartbeat_interval` | float | `15.0` | Intervalle en secondes entre les heartbeats envoyes au serveur |
| `private_key_pem` | string | `""` | Cle privee Ed25519 PEM (reste locale, jamais envoyee au serveur) |

### Modification manuelle

Le fichier peut etre edite manuellement. Les valeurs modifiables les plus utiles :

- `ollama_url` : si Ollama tourne sur un autre port ou une autre machine
- `poll_interval` : reduire pour une reaction plus rapide (min recommande : 1.0), augmenter pour moins de charge reseau
- `heartbeat_interval` : reduire pour un meilleur suivi (min recommande : 5.0)

Les champs `node_id`, `api_key`, et `private_key_pem` ne doivent pas etre modifies manuellement.

### Detection d'enregistrement

Le worker est considere comme enregistre si **les deux** champs `api_key` et `server_url` sont non vides dans la config.

---

## 5. Detection GPU

La detection s'effectue automatiquement lors du `register` et suit une cascade de 3 methodes, par ordre de fiabilite.

### Methode 1 : pynvml (nvidia-ml-py) — preferee

```python
import pynvml
pynvml.nvmlInit()
handle = pynvml.nvmlDeviceGetHandleByIndex(0)
name = pynvml.nvmlDeviceGetName(handle)
mem_info = pynvml.nvmlDeviceGetMemoryInfo(handle)
vram_mb = mem_info.total // (1024 * 1024)
```

- Utilise la librairie NVIDIA Management Library en Python
- Requiert `pip install nvidia-ml-py` (ou `pip install nexus-worker[nvidia]`)
- Acces direct au driver NVIDIA, valeurs exactes
- Detecte le GPU a l'index 0

### Methode 2 : nvidia-smi CLI — fallback NVIDIA

```bash
nvidia-smi --query-gpu=name,memory.total --format=csv,noheader,nounits
```

- Fallback si pynvml n'est pas installe
- Requiert que `nvidia-smi` soit dans le PATH
- Parse la sortie CSV (timeout : 10 secondes)
- Premiere ligne uniquement (GPU 0)

### Methode 3 : Apple Silicon — memoire unifiee

- Uniquement sur macOS (`platform.system() == "darwin"`)
- Lit la memoire totale via `sysctl -n hw.memsize`
- Estime la VRAM GPU a 75% de la memoire unifiee (`total * 0.75`)
- Lit le nom du chip via `sysctl -n machdep.cpu.brand_string`

### Aucun GPU detecte

Si les 3 methodes echouent, le resultat est :

```json
{"gpu_model": "Unknown GPU", "vram_mb": 0, "platform": "..."}
```

L'enregistrement est alors **refuse** avec le message :

```
No GPU detected. A GPU is required to contribute.
```

### Format d'affichage VRAM

- `>= 1024 MB` : affiche en GB (ex: `16 GB`)
- `< 1024 MB` : affiche en MB (ex: `512 MB`)

---

## 6. Modes de fonctionnement

### Mode par defaut : Ollama local

C'est le mode standard. Le worker :
1. Recoit les instructions de modele via heartbeat
2. Pull le modele via Ollama (`POST /api/pull`)
3. Execute les taches via `POST /api/generate` (mode non-streaming)
4. Renvoie les resultats au serveur

Aucun flag requis. L'affichage indique `Mode: Ollama local`.

### Mode exo peer (`--exo`)

```bash
nexus-worker start --exo
```

Le worker contribue des couches GPU a un modele distribue sur plusieurs machines via [exo](https://github.com/exo-explore/exo).

- **Prerequis** : `pip install exo` (le binaire `exo` doit etre dans le PATH)
- **Port** : 31330 (par defaut)
- **Fonctionnement** : lance un processus `exo run --port 31330 --initial-peers <server_url>`
- **Monitoring** : le processus exo est surveille en arriere-plan; un crash est detecte et log
- **Arret** : le processus est termine proprement (SIGTERM, puis SIGKILL apres 10s timeout)

L'affichage indique `Mode: exo peer`.

### Mode Petals server (`--petals`)

```bash
nexus-worker start --petals
```

Heberge des blocs transformer pour un modele distribue via [Petals](https://github.com/bigscience-workshop/petals).

- **Prerequis** : `pip install petals`
- **Port** : 31330
- **Modele** : par defaut `meta-llama/Meta-Llama-3.1-405B` (configurable via le champ `petals_model` dans config.json)
- **Fonctionnement** : lance `python -m petals.cli.run_server <model> --port 31330`
- **Arret** : termine proprement (SIGTERM, puis SIGKILL apres 10s timeout)

L'affichage indique `Mode: Petals server`.

### Mode sync (`--sync`)

```bash
nexus-worker start --sync
```

Active la synchronisation en temps reel de la base de donnees NEXUS vers une copie locale.

- **Transport** : WebSocket (`wss://<server>/ws/sync`)
- **Snapshot initial** : `GET /api/sync/snapshot`
- **Base locale** : `~/.nexus-worker/nexus_local.db`
- **Protocole** : cr-sqlite (CRDT-based SQLite replication)

Ce mode peut etre combine avec les autres flags (`--exo --sync`, etc.).

### Combinaisons

Les flags sont combinables :

```bash
nexus-worker start --no-dashboard --sync
nexus-worker start --exo --sync
nexus-worker start --petals --no-dashboard
```

---

## 7. Dashboard Rich TUI

Par defaut, `nexus-worker start` affiche un dashboard en temps reel dans le terminal, rafraichi 2 fois par seconde.

### Sections affichees

```
+--------------------------------------------+
|           NEXUS Worker                      |
|                                             |
| NEXUS GPU Contributor -- MonPseudo          |
|                                             |
|   GPU: NVIDIA GeForce RTX 5080 (16 GB)     |
|   Model: gemma-4-26B-A4B-it-heretic:q4_k_m |
|   Status: processing                        |
|                                             |
|   Current task:                             |
|     Type: summarize                         |
|     ID: a1b2c3d4e5f6...                     |
|                                             |
|   Session: 42 tasks | 2h 15m uptime         |
|   Speed: 28.3 tokens/s                      |
|   Total tokens: 156,832                     |
|                                             |
|   Network:                                  |
|     Nodes online: 12 (94 GB VRAM)           |
|     Model actif: gemma-4:q4_k_m             |
|     Tasks today: 1,234                      |
|                                             |
|   Leaderboard:                              |
|    -> 3. MonPseudo         42 tasks          |
|       1. GPUKing          128 tasks          |
|       2. CryptoMiner       87 tasks          |
|       4. DataCruncher      38 tasks          |
|       5. NeuralNinja       25 tasks          |
|                                             |
|   [Q] Quit  [P] Pause/Resume               |
+--------------------------------------------+
```

| Section | Contenu |
|---|---|
| **En-tete** | Nom du contributeur |
| **GPU info** | Modele GPU, VRAM, modele Ollama actif, etat actuel |
| **Current task** | Type et ID de la tache en cours (visible uniquement en etat `processing`) |
| **Pulling model** | Affiche "Pulling model..." (visible uniquement en etat `pulling_model`) |
| **Session** | Nombre de taches completees, uptime (format `Xh Ym`), vitesse derniere tache, tokens totaux, erreurs |
| **Network** | Noeuds en ligne, VRAM totale, modele actif du reseau, taches du jour |
| **Leaderboard** | Top 5 contributeurs avec fleche `->` sur votre position |
| **Controls** | Raccourcis clavier disponibles |

### Couleurs d'etat

| Etat | Couleur |
|---|---|
| `idle` | Jaune |
| `pulling_model` | Cyan |
| `processing` | Vert |
| `paused` | Gris (dim) |
| `error` | Rouge |
| `stopped` | Rouge |

### Raccourcis clavier

| Touche | Action |
|---|---|
| `Q` | Arrete proprement le worker (termine la tache en cours, puis quitte) |
| `P` | Bascule pause/resume. En pause, aucune nouvelle tache n'est acceptee mais la tache en cours se termine |

La detection clavier fonctionne sur Windows (via `msvcrt.kbhit()`) et Linux/macOS (via `select` sur stdin).

### Mode sans dashboard

```bash
nexus-worker start --no-dashboard
```

Le worker tourne en mode simple : logs uniquement, arret via `Ctrl+C`.

---

## 8. Signature Ed25519

Chaque resultat de tache est signe cryptographiquement pour prouver l'identite du contributeur (non-repudiation).

### Generation de cles

Lors du `register`, si le package `cryptography` est installe :

1. Une paire de cles **Ed25519** est generee (`Ed25519PrivateKey.generate()`)
2. La **cle publique** (PEM, format SubjectPublicKeyInfo) est envoyee au serveur
3. La **cle privee** (PEM, format PKCS8, sans chiffrement) est stockee localement dans `config.json` (champ `private_key_pem`)

Si `cryptography` n'est pas installe, la signature est desactivee et le worker affiche :

```
Ed25519: cryptography not installed (signing disabled)
```

### Processus de signature

A chaque soumission de resultat, le worker :

1. Construit un **payload canonique** JSON avec cles triees :
   ```json
   {
     "model_digest": "<sha256 du modele Ollama>",
     "node_id": "<uuid du noeud>",
     "result": "<premiers 2000 caracteres du resultat>",
     "task_id": "<uuid de la tache>"
   }
   ```
2. Signe le payload avec la cle privee Ed25519
3. Encode la signature en **base64**
4. Envoie la signature dans le champ `signature` du resultat

### Verification (cote serveur)

Le serveur verifie la signature avec la cle publique enregistree. Si `cryptography` n'est pas installe cote serveur, la verification est bypassee (degradation gracieuse).

### Installer cryptography

```bash
pip install cryptography
```

Puis re-enregistrer le worker pour generer une paire de cles :

```bash
nexus-worker register --server https://nexusgov.fr --name "MonPseudo"
```

---

## 9. Etats du moteur

Le `WorkerEngine` suit une machine a etats avec 6 etats possibles.

### Diagramme d'etats

```
                    +----------+
                    |   IDLE   |<-----------+
                    +----------+            |
                     /    |    \            |
                    /     |     \           |
     heartbeat:   /      |      \ pull     |
     pull_model  /       |       \ task    |
                v        |        v        |
  +--------------+       |   +------------+|
  |PULLING_MODEL |       |   | PROCESSING ||
  +--------------+       |   +------------+|
         |               |        |        |
         | done          |        | done   |
         +-------->------+--------+--------+
                         |
                    [P] pause
                         |
                         v
                    +----------+
                    |  PAUSED  |
                    +----------+
                         |
                    [P] resume
                         |
                         v
                      (etat precedent, ou IDLE)


  Erreurs:               Arret:
  +-------+            +---------+
  | ERROR |            | STOPPED |
  +-------+            +---------+
```

### Description des etats

| Etat | Valeur | Description |
|---|---|---|
| `IDLE` | `"idle"` | En attente d'une tache. Le worker poll le serveur a intervalles reguliers (`poll_interval`). |
| `PULLING_MODEL` | `"pulling_model"` | Telechargement d'un modele Ollama en cours (declenche par le serveur via heartbeat). Aucune tache n'est traitee pendant le pull. |
| `PROCESSING` | `"processing"` | Execution d'une tache LLM via Ollama (`POST /api/generate`). |
| `PAUSED` | `"paused"` | Le worker est en pause (touche P ou appel `engine.pause()`). La tache en cours se termine mais aucune nouvelle tache n'est acceptee. |
| `ERROR` | `"error"` | Erreur lors de l'execution. Le worker applique un backoff exponentiel avant de reessayer. |
| `STOPPED` | `"stopped"` | Arret complet. Le worker ne peut plus etre redemarre. |

### Backoff exponentiel

En cas d'erreurs consecutives, le delai avant la prochaine tentative suit la formule :

```
delai = min(2 ^ nombre_erreurs_consecutives, 60.0)
```

| Erreurs consecutives | Delai |
|---|---|
| 1 | 2s |
| 2 | 4s |
| 3 | 8s |
| 4 | 16s |
| 5 | 32s |
| 6+ | 60s (max) |

Le compteur d'erreurs est remis a zero des qu'un heartbeat ou une tache reussit.

### Boucle principale

La boucle `_main_loop` du moteur suit ce cycle :

1. Attendre 2 secondes (laisser le premier heartbeat passer)
2. Si en pause (`PAUSED`) : attendre 1s et recommencer
3. Si en pull de modele (`PULLING_MODEL`) : attendre 2s et recommencer
4. Si aucun modele assigne (`current_model == ""`) : attendre `poll_interval` et recommencer
5. Appeler `GET /api/compute/task` pour recuperer une tache
6. Si aucune tache (204) : repasser en `IDLE`, attendre `poll_interval`
7. Passer en `PROCESSING`, executer via Ollama
8. Soumettre le resultat via `POST /api/compute/result`
9. Si accepte : incrementer compteur, reset erreurs
10. Si rejete ou echec : incrementer erreurs, appliquer backoff
11. Repasser en `IDLE`, recommencer

### Boucle heartbeat

Toutes les `heartbeat_interval` secondes (defaut : 15s), le worker :

1. Envoie `POST /api/compute/heartbeat` avec le modele actuel et le statut (`idle` ou `busy`)
2. Si le serveur repond `message: "pull_model:<model>"` et que le modele est different du modele actuel : lance le pull en arriere-plan
3. Recupere les statistiques reseau et le leaderboard (pour le dashboard)

---

## 10. Depannage

### Le worker refuse de demarrer

| Symptome | Cause | Solution |
|---|---|---|
| `Not registered. Run: nexus-worker register...` | Pas de config valide | Executer `nexus-worker register --server URL --name NOM` |
| `No GPU detected. A GPU is required.` | Aucun GPU detecte | Installer les drivers GPU. Pour NVIDIA : `pip install nvidia-ml-py`. Verifier que `nvidia-smi` fonctionne |
| `Ollama: not detected` | Ollama n'est pas lance | Installer Ollama depuis [ollama.com](https://ollama.com), puis lancer `ollama serve` |

### Problemes de connexion au serveur

| Symptome | Cause | Solution |
|---|---|---|
| `Registration failed: ...` | Serveur inaccessible | Verifier l'URL du serveur. Verifier votre connexion internet |
| `Failed to fetch stats: ...` | API key invalide ou serveur down | Re-enregistrer avec `nexus-worker register` |
| Heartbeat echoue en boucle | Firewall ou proxy | Verifier que le port 443 (HTTPS) est ouvert. Configurer `ollama_url` si Ollama est sur une autre machine |

### Problemes GPU/Ollama

| Symptome | Cause | Solution |
|---|---|---|
| VRAM affichee a 0 MB | Driver GPU non installe | Installer/mettre a jour les drivers NVIDIA. Verifier avec `nvidia-smi` |
| `nvidia-ml-py` non detecte | Package optionnel manquant | `pip install nvidia-ml-py` ou `pip install nexus-worker[nvidia]` |
| Pull de modele echoue | Ollama non lance ou espace disque insuffisant | Lancer `ollama serve`. Verifier l'espace disque (`ollama list` pour voir les modeles existants) |
| Tache timeout (300s) | Modele trop lourd pour le GPU | Le serveur attribue les taches selon la VRAM; ce probleme est rare. Verifier que `OLLAMA_FLASH_ATTENTION=1` est defini |
| Erreurs d'execution en boucle | Ollama sature ou crash | Verifier `ollama ps`. Redemarrer Ollama. Le worker applique un backoff exponentiel automatique (max 60s) |

### Problemes de mode exo/Petals

| Symptome | Cause | Solution |
|---|---|---|
| `exo not found` | exo non installe | `pip install exo` |
| `Petals server failed` | petals non installe | `pip install petals` |
| exo peer crash au demarrage | Port 31330 deja utilise | Arreter le processus utilisant le port, ou modifier le port dans la config exo |

### Problemes de signature

| Symptome | Cause | Solution |
|---|---|---|
| `Ed25519: cryptography not installed` | Package manquant | `pip install cryptography` puis re-enregistrer |
| `Result rejected` | Signature invalide | Re-enregistrer (`nexus-worker register`) pour regenerer la paire de cles |

### Verification de la configuration

```bash
# Afficher la config actuelle
nexus-worker config

# Verifier manuellement le fichier
cat ~/.nexus-worker/config.json

# Tester la detection GPU
python -c "from worker.gpu_detect import detect_gpu; print(detect_gpu())"

# Verifier Ollama
curl http://localhost:11434/api/version
```

### Logs detailles

Pour activer le mode debug :

```bash
nexus-worker -v start
nexus-worker -v stats
```

Le flag `-v` / `--verbose` passe le niveau de log de INFO a DEBUG, ce qui affiche les details de chaque heartbeat, pull de tache, et erreur reseau.

---

## Reference rapide des endpoints API

Le worker communique avec le serveur via ces endpoints :

| Endpoint | Methode | Auth | Description |
|---|---|---|---|
| `/api/compute/register` | POST | Non | Enregistrement du noeud |
| `/api/compute/heartbeat` | POST | Bearer | Heartbeat periodique |
| `/api/compute/task` | GET | Bearer | Recuperer une tache (204 si aucune) |
| `/api/compute/result` | POST | Bearer | Soumettre un resultat |
| `/api/compute/model/ready` | POST | Bearer | Reporter qu'un modele est pull |
| `/api/compute/stats` | GET | Non | Statistiques reseau |
| `/api/compute/leaderboard` | GET | Non | Classement contributeurs |
| `/api/compute/model/status` | GET | Non | Etat du modele actif |
