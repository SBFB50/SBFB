# NEXUS Distributed GPU Computing -- Architecture

Reference documentaire exhaustive du systeme de calcul GPU distribue. Chaque nombre,
chaque seuil, chaque constante provient directement du code source.

---

## Table des matieres

1. [Vue d'ensemble systeme](#1-vue-densemble-systeme)
2. [Tables de la base de donnees](#2-tables-de-la-base-de-donnees)
3. [Cycle de vie des taches](#3-cycle-de-vie-des-taches)
4. [Auto-scaling des modeles (6 tiers)](#4-auto-scaling-des-modeles-6-tiers)
5. [Routage hybride](#5-routage-hybride)
6. [Protocole de verification 3 couches](#6-protocole-de-verification-3-couches)
7. [Regles de scoring de confiance](#7-regles-de-scoring-de-confiance)
8. [Self-Worker (GPU embarque)](#8-self-worker-gpu-embarque)
9. [Architecture Petals Swarm](#9-architecture-petals-swarm)
10. [Protocole de synchronisation cr-sqlite](#10-protocole-de-synchronisation-cr-sqlite)

---

## 1. Vue d'ensemble systeme

```
+======================================================================+
|                          NEXUS SERVER                                 |
|                                                                       |
|  +------------------+    +------------------+    +-----------------+  |
|  |  ComputeManager  |--->|  ModelSelector   |--->| HybridRouter    |  |
|  |  (orchestrator)  |    |  (auto-scaling)  |    | (LOCAL/DIST/    |  |
|  +--------+---------+    +--------+---------+    |  PETALS/OVER)   |  |
|           |                       |               +-----------------+  |
|           v                       v                                   |
|  +------------------+    +------------------+    +-----------------+  |
|  |  TaskDispatcher  |    |  ResultVerifier  |    | SwarmManager    |  |
|  |  (queue+reaper+  |    |  (3-layer proof) |    | (Petals DHT     |  |
|  |   heartbeat)     |    +------------------+    |  monitoring)    |  |
|  +--------+---------+                            +-----------------+  |
|           |                                                           |
|           v                                                           |
|  +------------------+    +------------------+    +-----------------+  |
|  |  SelfWorker      |    |  EventBus        |    | ComputeDatabase |  |
|  |  (embedded GPU   |    |  (16 event       |    | (6 tables,      |  |
|  |   contributor)   |    |   types)         |    |  17 indexes)    |  |
|  +------------------+    +------------------+    +-----------------+  |
|                                                                       |
+============================+==========================================+
                             |
            +----------------+----------------+
            |                |                |
            v                v                v
   +----------------+ +-------------+ +----------------+
   | External GPU   | | External GPU| | Petals Swarm   |
   | Contributor #1 | | Contrib. #N | | (internet,     |
   | (HTTP API +    | | (HTTP API + | |  50+ GPUs,     |
   |  Ed25519 sig)  | |  Ed25519)   | |  405B model)   |
   +----------------+ +-------------+ +----------------+
            |                |                |
            v                v                v
   +----------------+ +-------------+ +----------------+
   | SyncReceiver   | | SyncReceiver| | PetalsBackend  |
   | (WebSocket +   | | (WebSocket +| | (AutoDistrib.  |
   |  cr-sqlite)    | |  cr-sqlite) | |  ModelForCLM)  |
   +----------------+ +-------------+ +----------------+
```

### Composants principaux

| Composant | Fichier | Role |
|---|---|---|
| `ComputeManager` | `nexus/compute/manager.py` | Orchestrateur: init DB, demarre ModelSelector + TaskDispatcher + SelfWorker |
| `ModelSelector` | `nexus/compute/model_selector.py` | Selectionne le modele global selon la VRAM collective, gere les transitions |
| `HybridRouter` | `nexus/compute/hybrid.py` | Decide le mode d'execution: LOCAL, DISTRIBUTED, PETALS, OVERFLOW |
| `TaskDispatcher` | `nexus/compute/dispatcher.py` | File d'attente des taches, validation des resultats, reaper, heartbeat monitor |
| `ResultVerifier` | `nexus/compute/verification.py` | Verification 3 couches: Ed25519 + digest + logprob |
| `SelfWorker` | `nexus/compute/self_worker.py` | GPU embarque du serveur contribue automatiquement au reseau |
| `PetalsBackend` | `nexus/compute/petals_backend.py` | Client pour inference distribuee 405B via Petals |
| `SwarmManager` | `nexus/compute/swarm.py` | Surveillance de l'essaim Petals: couverture des blocs, sante |
| `ComputeDatabase` | `nexus/compute/db.py` | 6 tables, 17 index, CRUD complet pour noeuds/taches/resultats |
| `SyncBroadcaster` | `nexus/sync/broadcaster.py` | Broadcast WebSocket des changesets cr-sqlite vers les clients |
| `SyncReceiver` | `nexus/sync/receiver.py` | Client WebSocket qui applique les changesets dans un SQLite local |
| `ComputeDatabaseProxy` | `nexus/compute/events.py` | Proxy ouvrant une connexion fraiche par appel de methode |

---

## 2. Tables de la base de donnees

6 tables, 17 index. Toutes creees de maniere idempotente (`CREATE TABLE IF NOT EXISTS`).
Pragmas: `journal_mode = WAL`, `foreign_keys = ON`, `synchronous = NORMAL`.

### 2.1 `compute_nodes` -- Registre des contributeurs GPU

| Colonne | Type | Default | Description |
|---|---|---|---|
| `id` | TEXT | PK | UUID du noeud |
| `name` | TEXT | NOT NULL | Nom du contributeur |
| `gpu_model` | TEXT | NOT NULL | Modele GPU (ex: "NVIDIA GeForce RTX 5080") |
| `vram_mb` | INTEGER | NOT NULL | VRAM totale en megaoctets |
| `platform` | TEXT | `''` | Systeme d'exploitation (linux, windows, darwin) |
| `ollama_version` | TEXT | `''` | Version Ollama installee |
| `status` | TEXT | `'idle'` | `idle` / `busy` / `offline` / `banned` |
| `connected_at` | DATETIME | | Timestamp de connexion initiale |
| `last_heartbeat` | DATETIME | | Dernier heartbeat recu |
| `tasks_completed` | INTEGER | `0` | Compteur de taches reussies |
| `tasks_errored` | INTEGER | `0` | Compteur de taches en erreur |
| `avg_tokens_per_sec` | REAL | `0.0` | Moyenne mobile exponentielle (alpha=0.2) |
| `trust_score` | INTEGER | `50` | Score de confiance [0-100] |
| `api_key_hash` | TEXT | NOT NULL | SHA-256 de la cle API (jamais stockee en clair) |
| `ip_hash` | TEXT | NOT NULL | SHA-256 de l'adresse IP (jamais stockee en clair) |
| `public_key` | TEXT | `''` | Cle publique Ed25519 PEM pour verification signatures |
| `current_model` | TEXT | `''` | Modele actuellement charge sur le noeud |
| `assigned_model` | TEXT | `''` | Modele que le noeud devrait charger |
| `model_status` | TEXT | `''` | Etat du modele: `pulling` / `ready` / `failed` |
| `model_pull_started_at` | DATETIME | | Timestamp du debut de telechargement du modele |
| `metadata` | TEXT | `'{}'` | JSON libre |
| `created_at` | DATETIME | `CURRENT_TIMESTAMP` | Date de creation |
| `updated_at` | DATETIME | `CURRENT_TIMESTAMP` | Derniere mise a jour |

**Index:**
- `idx_compute_nodes_status` on `(status)`
- `idx_compute_nodes_trust` on `(trust_score)`
- `idx_compute_nodes_api_key` on `(api_key_hash)`
- `idx_compute_nodes_model_status` on `(model_status)`
- `idx_compute_nodes_assigned_model` on `(assigned_model)`

### 2.2 `compute_tasks` -- File d'attente des taches LLM

| Colonne | Type | Default | Description |
|---|---|---|---|
| `id` | TEXT | PK | UUID de la tache |
| `task_type` | TEXT | NOT NULL | Type de tache LLM |
| `prompt` | TEXT | NOT NULL | Prompt principal |
| `system_prompt` | TEXT | `''` | Prompt systeme optionnel |
| `model` | TEXT | `''` | Modele cible (auto-assigne par ModelSelector) |
| `status` | TEXT | `'pending'` | `pending` / `assigned` / `completed` / `failed` |
| `priority` | INTEGER | `5` | Priorite (1=urgent, 10=batch) |
| `assigned_to` | TEXT | FK→compute_nodes(id) | Noeud assigne |
| `assigned_at` | DATETIME | | Timestamp d'assignation |
| `completed_at` | DATETIME | | Timestamp de completion |
| `result` | TEXT | | Texte du resultat final |
| `result_validated` | INTEGER | `0` | 1 si resultat verifie |
| `validation_score` | REAL | `0.0` | Score de validation [0.0-1.0] |
| `timeout_seconds` | INTEGER | `300` | Timeout par defaut 5 minutes |
| `require_logprobs` | INTEGER | `0` | 1 si logprobs requis pour verification |
| `calibration_prompt` | TEXT | `''` | Prompt de calibration pour fingerprinting |
| `source_worker` | TEXT | `''` | Worker qui a soumis la tache |
| `parent_task_id` | TEXT | `''` | Tache parente (chainage) |
| `error_message` | TEXT | `''` | Message d'erreur en cas d'echec |
| `retry_count` | INTEGER | `0` | Nombre de tentatives effectuees |
| `max_retries` | INTEGER | `3` | Nombre maximum de tentatives |
| `execution_mode` | TEXT | `'local'` | `local` / `distributed` / `petals` / `overflow` |
| `metadata` | TEXT | `'{}'` | JSON libre |
| `created_at` | DATETIME | `CURRENT_TIMESTAMP` | Date de creation |
| `updated_at` | DATETIME | `CURRENT_TIMESTAMP` | Derniere mise a jour |

**Index:**
- `idx_compute_tasks_status` on `(status)`
- `idx_compute_tasks_priority` on `(priority, created_at)`
- `idx_compute_tasks_assigned` on `(assigned_to, status)`
- `idx_compute_tasks_type` on `(task_type)`

### 2.3 `compute_results` -- Resultats valides

| Colonne | Type | Default | Description |
|---|---|---|---|
| `id` | TEXT | PK | UUID du resultat |
| `task_id` | TEXT | FK→compute_tasks(id) | Tache associee |
| `node_id` | TEXT | FK→compute_nodes(id) | Noeud qui a produit le resultat |
| `result_text` | TEXT | NOT NULL | Texte genere |
| `tokens_generated` | INTEGER | `0` | Nombre de tokens generes |
| `generation_time_ms` | INTEGER | `0` | Temps de generation en millisecondes |
| `model_digest` | TEXT | `''` | SHA-256 des poids du modele (Couche 2) |
| `logprobs` | TEXT | `''` | Logprobs JSON pour fingerprinting (Couche 3) |
| `signature` | TEXT | `''` | Signature Ed25519 base64 (Couche 1) |
| `validated` | INTEGER | `0` | 1 si resultat verifie |
| `validation_method` | TEXT | `''` | Methode de validation utilisee |
| `metadata` | TEXT | `'{}'` | JSON libre |
| `created_at` | DATETIME | `CURRENT_TIMESTAMP` | Date de creation |

**Index:**
- `idx_compute_results_task` on `(task_id)`
- `idx_compute_results_node` on `(node_id)`

### 2.4 `compute_model_transitions` -- Historique des changements de modele

| Colonne | Type | Default | Description |
|---|---|---|---|
| `id` | TEXT | PK | UUID de la transition |
| `old_model` | TEXT | `''` | Modele precedent |
| `new_model` | TEXT | `''` | Nouveau modele cible |
| `old_tier` | TEXT | `''` | Tier precedent (ex: "Standard") |
| `new_tier` | TEXT | `''` | Nouveau tier (ex: "Avance") |
| `total_vram_gb` | REAL | `0.0` | VRAM totale du reseau au moment de la transition |
| `nodes_online` | INTEGER | `0` | Nombre de noeuds en ligne |
| `nodes_ready` | INTEGER | `0` | Nombre de noeuds prets pour le nouveau modele |
| `transition_state` | TEXT | `'transitioning'` | `transitioning` / `stable` |
| `started_at` | DATETIME | `CURRENT_TIMESTAMP` | Debut de la transition |
| `completed_at` | DATETIME | | Fin de la transition |

**Index:**
- `idx_compute_transitions_state` on `(transition_state)`

### 2.5 `compute_badges` -- Badges de contribution

| Colonne | Type | Default | Description |
|---|---|---|---|
| `id` | TEXT | PK | UUID du badge |
| `node_id` | TEXT | FK→compute_nodes(id) | Noeud recipiendaire |
| `badge_id` | TEXT | NOT NULL | Identifiant du badge |
| `badge_name` | TEXT | NOT NULL | Nom affichable du badge |
| `awarded_at` | DATETIME | `CURRENT_TIMESTAMP` | Date d'attribution |

**Contrainte:** `UNIQUE(node_id, badge_id)` -- un badge par noeud.

**Badges definis dans le code:**

| badge_id | badge_name | Condition |
|---|---|---|
| `first_task` | Premiere tache | 1 tache completee |
| `centurion` | Centurion | 100 taches completees |
| `millionnaire` | Millionnaire | 1 000 taches completees |
| `pilier` | Pilier | 10 000 taches completees |
| `power_node` | Power Node | VRAM > 24 576 MB (24 GB) |
| `early_adopter` | Early Adopter | Parmi les 10 premiers noeuds enregistres |
| `always_on` | 24/7 | Session continue >= 604 800 secondes (7 jours) |

**Index:**
- `idx_compute_badges_node` on `(node_id)`

### 2.6 `compute_uptime_log` -- Journal de disponibilite

| Colonne | Type | Default | Description |
|---|---|---|---|
| `id` | TEXT | PK | UUID de l'entree |
| `node_id` | TEXT | FK→compute_nodes(id) | Noeud concerne |
| `connected_at` | DATETIME | NOT NULL | Debut de la session |
| `disconnected_at` | DATETIME | | Fin de la session (NULL si en cours) |
| `duration_seconds` | INTEGER | `0` | Duree calculee en secondes |

**Index:**
- `idx_compute_uptime_node` on `(node_id)`
- `idx_compute_uptime_connected` on `(connected_at)`

---

## 3. Cycle de vie des taches

```
                         submit_task()
                              |
                              v
                       +------------+
                       |  PENDING   |
                       +-----+------+
                             |
                   pull_next_task()
                   (BEGIN IMMEDIATE lock,
                    model affinity first,
                    then any pending,
                    ORDER BY priority ASC,
                    created_at ASC)
                             |
                             v
                       +------------+
                       |  ASSIGNED  |-------> expire_stale_tasks()
                       +-----+------+         (timeout depassant
                             |                 timeout_seconds)
                             |                      |
                    validate_result()                |
                    (3-layer verification)           |
                             |                      v
                  +----------+----------+    +------------+
                  |                     |    |  PENDING   | (si retry_count
                  v                     v    |  (retry)   |  < max_retries)
           +------------+        +----------++------------+
           | COMPLETED  |        |  FAILED  |
           | validated  |        | (max     |
           +------------+        |  retries)|
                                 +----------+
```

### Mecanisme d'assignation atomique

`pull_next_task()` utilise `BEGIN IMMEDIATE` pour acquerir un verrou RESERVED
immediatement, empechant deux noeuds d'etre assignes a la meme tache.

Priorite d'assignation:
1. **Affinite modele**: taches dont le champ `model` correspond au `current_model` du noeud
2. **Fallback**: toute tache en `pending`, triee par `priority ASC, created_at ASC`

### Retry automatique

Quand `fail_task()` est appele:
- Si `retry_count < max_retries` (defaut: 3): la tache revient a `pending`, `assigned_to` et `assigned_at` sont remis a NULL
- Si `retry_count >= max_retries`: la tache passe a `failed` definitivement

### Reaper de taches expirees

Le `TaskDispatcher` execute `_reaper_loop()` en arriere-plan (intervalle configurable
via `settings.compute_reaper_interval`). Il reset les taches coincees en `assigned`
dont `(julianday(now) - julianday(assigned_at)) * 86400 > timeout_seconds`.

### Heartbeat monitor

Le `_heartbeat_monitor()` tourne toutes les 30 secondes. Si un noeud n'a pas envoye
de heartbeat depuis `settings.compute_heartbeat_timeout` secondes, il est marque
`offline`. Toutes ses taches assignees sont marquees en echec et le modele est
recalcule via `recalculate_model()`.

---

## 4. Auto-scaling des modeles (6 tiers)

Le `ModelSelector` recalcule le modele cible toutes les 60 secondes (configurable
via `_check_interval`). Le modele est determine par la VRAM totale collective
de tous les noeuds en ligne.

### Tiers de modeles

| min_vram_gb | Modele | Label | Description |
|---|---|---|---|
| 0 | `gemma-4-12b-q4` | Basique | Modele de base, aucun GPU requis |
| 14 | `juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m` | Standard | MoE 26B, 4B actifs, modele par defaut NEXUS |
| 40 | `llama-3.1-70b-q4` | Avance | Requiert distribution ou GPU haut de gamme |
| 80 | `qwen-2.5-110b-q4` | Pro | Multi-GPU obligatoire |
| 150 | `llama-3.1-405b-q2` | Ultra | Quantization aggressive, Petals recommande |
| 300 | `llama-3.1-405b` | Maximum | Full precision, essaim Petals large |

### Algorithme de selection

```python
# Pseudo-code simplifie de recalculate()
total_vram_gb = sum(node.vram_mb for node in online_nodes) / 1024
best_tier = max(t for t in MODEL_TIERS if total_vram_gb >= t["min_vram_gb"])
```

Exemple: 5 noeuds x 16 GB = 80 GB total --> tier `Pro` (qwen-2.5-110b-q4)

### Etats de transition

Quand le modele change, le systeme entre en mode `TransitionState`:

| Etat | Description |
|---|---|
| `STABLE` | Tous les noeuds compatibles ont le modele cible charge |
| `TRANSITIONING` | Certains noeuds sont en cours de telechargement (`pulling`) |
| `DEGRADED` | La transition est bloquee (certains noeuds ne peuvent pas telecharger) |

### Selection de modele en mode mixte (pendant transition)

`get_task_model()` determine le modele pour une tache selon sa priorite:
- **Taches urgentes** (priority <= 3): accepte n'importe quel modele disponible (retourne chaine vide = affinite)
- **Taches batch** (priority >= 7): attend le modele cible
- **Taches normales**: prefere le modele cible, fallback sur le precedent

### Assignation par noeud

`get_model_for_node(vram_mb)`:
- Si la VRAM du noeud >= VRAM minimale du modele cible --> le noeud recoit le modele cible
- Sinon --> le noeud recoit le meilleur modele individuel qu'il peut executer

---

## 5. Routage hybride

Le `HybridRouter` decide le mode d'execution pour chaque tache.
Philosophie: **toujours maximiser la qualite du modele**. Un 70B distribue
sur 3 GPU est meilleur que 3 x 26B separes.

### Modes d'execution

| Mode | Enum | Description |
|---|---|---|
| LOCAL | `ExecutionMode.LOCAL` | Ollama sur un seul noeud (le modele tient en VRAM) |
| DISTRIBUTED | `ExecutionMode.DISTRIBUTED` | exo split le modele sur plusieurs noeuds (OpenAI-compatible) |
| PETALS | `ExecutionMode.PETALS` | Petals swarm: 405B+ reparti sur 50+ GPU via internet |
| OVERFLOW | `ExecutionMode.OVERFLOW` | Petit noeud local en fallback pendant le mode distribue |

### Cascade de decision

```
1. exo desactive?                    --> LOCAL
2. Aucun noeud en ligne?             --> LOCAL
3. Modele tient sur 1 seul noeud?    --> LOCAL (plus rapide, pas d'overhead reseau)
4. Petals active + pret + VRAM >= petals_min_vram_gb?  --> PETALS
5. exo disponible?
   a. VRAM du noeud < 8 GB?          --> OVERFLOW (trop petit pour exo)
   b. Sinon                          --> DISTRIBUTED
6. Rien de disponible?               --> LOCAL (fallback)
```

### Recalcul du mode (dans `ModelSelector.recalculate()`)

Le mode d'execution global est decide selon cette logique:
1. Si Petals est pret **ET** la VRAM totale >= `settings.petals_min_vram_gb` **ET** le modele cible ne tient sur aucun noeud individuel --> `PETALS`
2. Si le modele necessite distribution (`needs_distributed()`) **ET** exo est disponible --> `DISTRIBUTED`
3. Sinon --> `LOCAL`

### ExoBackend

Client pour l'API OpenAI-compatible d'exo (endpoint par defaut: `settings.exo_url`).

- Endpoint: `POST {exo_url}/v1/chat/completions`
- Health check: `GET {exo_url}/v1/models`
- Timeout par defaut: 300 secondes
- Health check timeout: 5 secondes
- Health check periodique configurable via `settings.exo_health_interval`

Retourne: `{text, tokens, prompt_tokens, model}`.

### Noeuds Overflow

Les noeuds trop petits pour participer au cluster exo (VRAM < 8 GB) servent les taches
en attente avec leur meilleur modele local individuel. Ils absorbent les pics de charge
quand le cluster distribue est sature.

---

## 6. Protocole de verification 3 couches

Le `ResultVerifier` combine 3 couches de verification independantes, plus un
mecanisme de spot-checking BOINC-style.

```
Resultat soumis par un noeud
         |
         v
  +-------------------+
  | Couche 1: Ed25519 |  Qui a envoye le resultat?
  | (identite)        |  Verification: ~0.1ms
  +--------+----------+
           |
    echec? --> trust_delta = -50, BAN immediat
           |
           v
  +-------------------+
  | Couche 2: Digest  |  Quel modele est charge?
  | (model weights    |  SHA-256 des poids vs whitelist
  |  SHA-256)         |
  +--------+----------+
           |
    echec? --> trust_delta = -50, BAN immediat
           |
           v
  +-------------------+
  | Couche 3: Logprob |  Le bon modele a-t-il genere la reponse?
  | (fingerprinting)  |  Applique a ~10% des taches (calibration)
  |  KL-divergence    |  Reference: LLMmap, USENIX Security 2025
  +--------+----------+
           |
    echec? --> trust_delta = -5 (pas de ban, augmentation spot-checks)
           |
           v
  +-----------------------+
  | BOINC Spot-Checking   |  Re-execution sur GPU de confiance
  | (filet de securite)   |  Taux variable selon trust_score
  +-----------------------+
           |
           v
    Resultat accepte (trust_delta = +1)
```

### Couche 1 -- Ed25519 Signature (identite)

**Fichier:** `nexus/compute/crypto.py`

- Chaque contributeur genere une paire de cles Ed25519 a l'enregistrement
- La cle publique est envoyee au serveur (format PEM, SubjectPublicKeyInfo)
- La cle privee reste locale (format PEM, PKCS8, pas de chiffrement)
- Chaque soumission de resultat est signee

**Payload signe** (JSON deterministe avec `sort_keys=True`):
```json
{
    "model_digest": "<SHA-256 des poids>",
    "node_id": "<UUID du noeud>",
    "result": "<les 2000 premiers caracteres du resultat>",
    "task_id": "<UUID de la tache>"
}
```

Note: le `result` est tronque a 2000 caracteres pour la performance.

**Signature:** `Ed25519(payload_json_bytes)` --> base64-encode

**Degradation gracieuse:** si le package `cryptography` n'est pas installe, les
signatures ne sont pas requises et la verification renvoie `True` automatiquement.

### Couche 2 -- Digest du modele (SHA-256 des poids)

**Fichier:** `nexus/compute/verification.py`

Le digest est le SHA-256 du fichier de poids du modele Ollama. Le serveur
maintient une whitelist `_DIGEST_WHITELIST: dict[model_name, expected_digest]`
peuplee au runtime.

Cas de figure:
- Whitelist vide --> check ignore (accepte avec raison `no_whitelist`)
- Digest non fourni --> echec (`missing_digest`)
- Modele absent de la whitelist --> accepte avec flag (`model_not_in_whitelist`)
- Digest correspond --> succes (`digest_match`)
- Digest ne correspond pas --> echec + BAN (`digest_mismatch`)

### Couche 3 -- Logprob Fingerprinting (identite du modele)

**Fichier:** `nexus/compute/verification.py`

8 prompts de calibration en francais (contexte politique):

1. "La capitale de la France est"
2. "Le president de la Republique en 2026 est"
3. "L'article 49.3 permet au gouvernement de"
4. "Le nombre de deputes a l'Assemblee nationale est"
5. "La devise de la France est"
6. "Le Senat est compose de"
7. "La Constitution de la Cinquieme Republique date de"
8. "Le Premier ministre est nomme par"

- **Taux de calibration**: ~10% des taches (`random.random() < 0.10`)
- **Methode**: comparaison de la divergence max absolue entre logprobs attendus et reels
- **Seuil**: `threshold = 0.5` (divergence max toleree)
- **Precision declaree**: >95% avec 8 queries (reference: LLMmap, USENIX Security 2025)

Resultat si echec: `trust_delta = -5`, pas de ban immediat (augmente le taux de spot-check).

---

## 7. Regles de scoring de confiance

Score initial: **50** (sur une echelle de 0 a 100, clamp par `max(0, min(100, score + delta))`).

### Deltas de confiance

| Evenement | trust_delta | Consequence |
|---|---|---|
| Tache acceptee (toutes les couches passent) | +1 | -- |
| Spot-check reussi | +5 | -- |
| Logprob fingerprinting echoue (Couche 3) | -5 | Augmentation du taux de spot-check |
| Resultat trop court (< 5 caracteres) | -5 | Tache en echec |
| Spot-check echoue | -20 | -- |
| Signature Ed25519 invalide (Couche 1) | -50 | BAN immediat |
| Digest modele non conforme (Couche 2) | -50 | BAN immediat |
| Trust score < 20 | -- | BAN automatique |

### Taux de spot-checking

Le taux de re-execution sur GPU de confiance depend du `trust_score` du noeud:

| trust_score | Taux de spot-check |
|---|---|
| >= 80 | 1% |
| >= 50 | 5% |
| < 50 | 20% |

### Ban automatique

Quand un noeud est banni (`ban_node()`):
1. Son `status` passe a `banned`
2. Son `trust_score` est mis a 0
3. Toutes ses taches en `assigned` sont remises en `pending` (`assigned_to = NULL`)

---

## 8. Self-Worker (GPU embarque)

**Fichier:** `nexus/compute/self_worker.py`

Le serveur NEXUS contribue automatiquement son propre GPU au reseau de calcul,
sans necessiter de CLI externe. L'operateur du serveur est automatiquement le
premier contributeur.

### Cycle de vie

```
start()
  |
  +--> _detect_gpu()           # pynvml ou nvidia-smi
  |       |
  |       v
  |    vram_mb == 0?  --> desactive ("no GPU detected")
  |       |
  |       v
  +--> _ensure_registered()    # Cherche noeud name="_self_worker_"
  |       |                    # Si absent: register_node() avec ip="127.0.0.1"
  |       v
  +--> _task_loop()            # Boucle: pull_next_task() -> Ollama -> store_result()
  |       |
  +--> _heartbeat_loop()       # Heartbeat toutes les 15 secondes
```

### Detection GPU

Deux methodes, essayees dans l'ordre:
1. **pynvml**: `nvmlDeviceGetHandleByIndex(0)` --> nom + memoire totale
2. **nvidia-smi**: `nvidia-smi --query-gpu=name,memory.total --format=csv,noheader,nounits` (timeout 10s)

Retourne `{gpu_model: str, vram_mb: int}`. Si aucune methode ne fonctionne: `vram_mb = 0`, worker desactive.

### Auto-enregistrement

- Nom de convention: `"_self_worker_"`
- Si un noeud avec ce nom existe deja: reconnexion (heartbeat + status `idle`)
- Sinon: creation d'un nouveau noeud avec `ip="127.0.0.1"` et plateforme detectee

### Boucle de taches

1. Attente initiale de 3 secondes (laisser le systeme se stabiliser)
2. Boucle principale:
   - Si en pause (`_paused`): sleep 2 secondes
   - `pull_next_task()` directement via la DB (pas de round-trip HTTP)
   - Si pas de tache: sleep 2 secondes
   - Sinon: marque le noeud `busy`, execute via Ollama, stocke le resultat
3. Execution Ollama: `POST /api/generate` (base URL: `settings.ollama_base_url`)
   - Payload: `{model, prompt, system, stream: false}`
   - Calcul tokens/s: `eval_count / (eval_duration / 1e9)` (Ollama native) ou fallback sur duree totale
4. Apres execution reussie: `trust_delta = +1`
5. Pause/resume controlable via API (`pause()` / `resume()`)

### Heartbeat

Envoi toutes les 15 secondes via `db.heartbeat()` + `db.update_node_status("idle")`.

---

## 9. Architecture Petals Swarm

### PetalsBackend

**Fichier:** `nexus/compute/petals_backend.py`

Client pour inference distribuee via le reseau Petals. Chaque contributeur heberge
quelques blocs de transformers du modele. Petals gere le routage, la tolerance aux pannes
(dual-cache, auto-reroute) et le pipeline parallelism.

**Modele par defaut:** `settings.petals_model` (typiquement `meta-llama/Meta-Llama-3.1-405B`)

**Chargement paresseux:**
- Le modele n'est charge qu'au premier appel `generate()`
- `asyncio.Lock` empeche le chargement concurrent
- Utilise `AutoDistributedModelForCausalLM.from_pretrained()` + `AutoTokenizer`
- Toutes les operations synchrones de Petals sont wrappees via `asyncio.to_thread()`

**Generation:**
- Tokenization --> `model.generate()` dans un thread --> decodage
- Timeout configurable (defaut: 300 secondes)
- Retourne: `{text, tokens, prompt_tokens, model, duration_ms}`

### SwarmManager

**Fichier:** `nexus/compute/swarm.py`

Moniteur de sante de l'essaim Petals.

**Nombre de blocs par modele:**

| Modele | Blocs |
|---|---|
| `meta-llama/Meta-Llama-3.1-405B` | 126 |
| `meta-llama/Meta-Llama-3.1-70B` | 80 |
| `meta-llama/Meta-Llama-3.1-8B` | 32 |

**Etats de sante (`SwarmHealth`):**

| Etat | Description |
|---|---|
| `HEALTHY` | Tous les blocs couverts par au moins 1 noeud (chaine complete input->output) |
| `DEGRADED` | Certains blocs manquent (couverture partielle) |
| `OFFLINE` | Essaim non disponible |
| `UNKNOWN` | Pas encore verifie |

**Critere de disponibilite (`is_ready`):** `health == HEALTHY`

**Monitoring:**
- Verification periodique via `_monitor_loop()` (intervalle: `settings.petals_health_interval`, defaut 60s)
- Health check avec timeout de 30 secondes
- Interrogation du DHT Petals pour la couverture des blocs
- Couverture en pourcentage: `(blocks_covered / blocks_total) * 100`

---

## 10. Protocole de synchronisation cr-sqlite

Synchronisation en temps reel de la base de donnees via cr-sqlite (Conflict-free
Replicated Data Types pour SQLite).

### Architecture

```
  NEXUS Server                       Clients (Workers / Frontend)
  +-----------+                      +-------------------+
  | SQLite DB |                      | Local SQLite DB   |
  | (WAL)     |                      | (read-only copy)  |
  +-----+-----+                      +---------+---------+
        |                                      ^
        v                                      |
  +------------------+    WebSocket      +-----+---------+
  | SyncBroadcaster  |=================>>| SyncReceiver  |
  | polls 100ms      |  JSON changesets  | auto-reconnect|
  +------------------+                   +---------------+
```

### Tables synchronisees (15 tables GOV)

```
gov_politicians          gov_positions        gov_contradictions
gov_mandates             gov_parties          gov_party_memberships
gov_laws                 gov_press            gov_social_posts
gov_alerts               gov_transcriptions   gov_affairs
gov_declarations         gov_factchecks       gov_external_ids
```

### SyncBroadcaster (serveur)

**Fichier:** `nexus/sync/broadcaster.py`

1. **Initialisation:**
   - Ouvre une connexion SQLite dediee au path `settings.sqlite_path`
   - Charge l'extension `crsqlite`
   - Enregistre les 15 tables comme CRDT via `SELECT crsql_as_crr('<table>')`
   - Lit la version courante: `SELECT max(db_version) FROM crsql_changes`

2. **Polling (100ms):**
   - Query: `SELECT [table], [pk], [cid], [val], [col_version], [db_version] FROM crsql_changes WHERE db_version > ?`
   - Si des changements existent: broadcast JSON a tous les clients connectes
   - Met a jour `_db_version` au `max(db_version)` des changements

3. **Protocole WebSocket:**
   - A la connexion: envoie `{type: "version", version: <int>, tables: [...], crsqlite: <bool>}`
   - Changements: `{type: "changes", version: <int>, changes: [[table, pk, cid, val, col_version, db_version], ...]}`
   - Keepalive: `{type: "ping", version: <int>}` toutes les 30 secondes
   - Format des changements: tableau de 6 elements `[table, pk, cid, val, col_version, db_version]`

4. **Degradation gracieuse:** si `crsqlite` n'est pas disponible, la synchronisation
   est desactivee et le systeme fonctionne en mode API classique.

### SyncReceiver (client)

**Fichier:** `nexus/sync/receiver.py`

1. **Initialisation:**
   - Base locale par defaut: `~/.nexus-worker/nexus_local.db`
   - Si la base n'existe pas: telechargement du snapshot via HTTP (`{server}/api/sync/snapshot`)
   - Chargement de l'extension `crsqlite` localement

2. **Connexion WebSocket:**
   - Auto-reconnexion avec backoff exponentiel
   - Delay initial: 1 seconde
   - Delay maximum: 60 secondes
   - Reset du delay a la reconnexion reussie

3. **Application des changesets:**
   - `INSERT INTO crsql_changes ([table], [pk], [cid], [val], [col_version], [db_version]) VALUES (?, ?, ?, ?, ?, ?)`
   - Execution dans un thread via `asyncio.to_thread()` (SQLite synchrone)
   - Commit apres chaque batch
   - Tracking de `_local_version` pour la reprise

4. **Messages geres:**
   - `version`: log la version serveur
   - `changes`: applique les changesets
   - `ping`: ignore (keepalive)

---

## Phases du systeme

Resume des 9 phases telles qu'implementees dans le code:

| Phase | Composant | Description |
|---|---|---|
| 1 | `compute_nodes`, `compute_tasks`, `compute_results` | Tables de base + CRUD + registration |
| 2 | `ModelSelector`, `compute_model_transitions` | Auto-scaling 6 tiers + transitions gracieuses |
| 3 | `TaskDispatcher` | File d'attente prioritaire + reaper + heartbeat monitor |
| 4 | `HybridRouter`, `ExoBackend` | Routage LOCAL/DISTRIBUTED + client exo OpenAI-compatible |
| 5 | `compute_badges`, `ResultVerifier` | Badges de contribution + verification 3 couches |
| 6 | `crypto.py` | Signature Ed25519 (Couche 1 de la verification) |
| 7 | `PetalsBackend`, `SwarmManager` | Inference 405B distribuee via Petals + monitoring essaim |
| 8 | `compute_uptime_log`, `SelfWorker` | Tracking de disponibilite + GPU embarque automatique |
| 9 | `SyncBroadcaster`, `SyncReceiver` | Synchronisation temps reel cr-sqlite via WebSocket |

---

## 16 types d'evenements compute

**Fichier:** `nexus/compute/events.py`

| Categorie | Event Type | Valeur |
|---|---|---|
| Noeud | `COMPUTE_NODE_REGISTERED` | `compute_node_registered` |
| Noeud | `COMPUTE_NODE_CONNECTED` | `compute_node_connected` |
| Noeud | `COMPUTE_NODE_DISCONNECTED` | `compute_node_disconnected` |
| Noeud | `COMPUTE_NODE_BANNED` | `compute_node_banned` |
| Tache | `COMPUTE_TASK_CREATED` | `compute_task_created` |
| Tache | `COMPUTE_TASK_ASSIGNED` | `compute_task_assigned` |
| Tache | `COMPUTE_TASK_COMPLETED` | `compute_task_completed` |
| Tache | `COMPUTE_TASK_FAILED` | `compute_task_failed` |
| Tache | `COMPUTE_TASK_EXPIRED` | `compute_task_expired` |
| Validation | `COMPUTE_RESULT_VALIDATED` | `compute_result_validated` |
| Validation | `COMPUTE_RESULT_REJECTED` | `compute_result_rejected` |
| Validation | `COMPUTE_SPOT_CHECK_NEEDED` | `compute_spot_check_needed` |
| Modele | `COMPUTE_MODEL_CHANGED` | `compute_model_changed` |
| Reseau | `COMPUTE_NETWORK_STATS_UPDATED` | `compute_network_stats_updated` |
| Periodic | `COMPUTE_TICK_HEARTBEAT` | `compute_tick_heartbeat` |
| Periodic | `COMPUTE_TICK_REAPER` | `compute_tick_reaper` |

### ComputeDatabaseProxy

Proxy pour les workers long-lived qui ouvre une connexion fraiche a chaque appel de
methode via `__getattr__`. Evite de maintenir une connexion SQLite ouverte pendant
toute la duree de vie du worker.

```python
proxy = ComputeDatabaseProxy()
await proxy.get_node(node_id)  # ouvre get_db(), appelle ComputeDatabase.get_node(), ferme
```
