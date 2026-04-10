# NEXUS Compute -- Reference API (21 endpoints)

**Version :** 0.3.0
**Date :** 2026-04-09
**Base URL :** `http://localhost:8000`
**Fichiers :** `nexus/api/compute.py`, `nexus/sync/api.py`
**Modeles :** `nexus/compute/models.py`

---

## Table des matieres

1. [Authentification](#authentification)
2. [Rate Limiting](#rate-limiting)
3. [Public (pas d'auth)](#3-public-pas-dauth)
4. [Enregistrement node](#4-enregistrement-node)
5. [Authentifie (Bearer token)](#5-authentifie-bearer-token)
6. [Gestion des modeles](#6-gestion-des-modeles)
7. [Hybrid / Swarm](#7-hybrid--swarm)
8. [Self-Worker](#8-self-worker)
9. [Health / Uptime / Badges](#9-health--uptime--badges)
10. [Interne / Admin](#10-interne--admin)
11. [Sync (synchronisation DB)](#11-sync-synchronisation-db)

---

## Authentification

Les endpoints authentifies exigent un header `Authorization: Bearer <api_key>`.
La cle API est obtenue une seule fois lors de `POST /api/compute/register` et ne peut plus etre recuperee ensuite.

```
Authorization: Bearer nxc_a1b2c3d4e5f6...
```

Un noeud banni (status `banned`) recevra un **403 Forbidden**.
Un token invalide ou absent recevra un **401 Unauthorized**.

---

## Rate Limiting

Tous les endpoints authentifies et `POST /register` sont rate-limited a **100 requetes par minute par IP** (in-memory, basee sur le hash de l'IP client).

Reponse si depasse :
```
HTTP 429 Too Many Requests
{"detail": "Rate limit exceeded (100 req/min)"}
```

---

## 3. Public (pas d'auth)

### GET /api/compute/stats

Statistiques publiques du reseau de calcul distribue.

**Response model :** `NetworkStatsResponse`

| Champ | Type | Description |
|-------|------|-------------|
| `nodes_online` | int | Noeuds actuellement connectes |
| `nodes_total` | int | Total de noeuds enregistres |
| `vram_total_gb` | float | VRAM cumulee du reseau (GB) |
| `tasks_pending` | int | Taches en attente |
| `tasks_assigned` | int | Taches en cours |
| `tasks_completed` | int | Taches terminees (total) |
| `tasks_failed` | int | Taches echouees (total) |
| `tasks_today` | int | Taches completees aujourd'hui |
| `current_model` | string | Modele LLM actif sur le reseau |
| `model_tier` | string | Tier du modele (e.g. "26B", "13B") |

```bash
curl http://localhost:8000/api/compute/stats
```

```json
{
  "nodes_online": 3,
  "nodes_total": 5,
  "vram_total_gb": 40.0,
  "tasks_pending": 2,
  "tasks_assigned": 1,
  "tasks_completed": 847,
  "tasks_failed": 12,
  "tasks_today": 34,
  "current_model": "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m",
  "model_tier": "26B"
}
```

---

### GET /api/compute/leaderboard

Classement public des contributeurs GPU.

**Query params :**

| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `limit` | int | 20 | Nombre d'entrees (max 100) |

**Response model :** `LeaderboardResponse`

| Champ | Type | Description |
|-------|------|-------------|
| `entries` | list[LeaderboardEntry] | Liste classee |
| `total_contributors` | int | Total de contributeurs enregistres |

**LeaderboardEntry :**

| Champ | Type | Description |
|-------|------|-------------|
| `rank` | int | Rang dans le classement |
| `name` | string | Nom du contributeur |
| `gpu_model` | string | Modele GPU |
| `vram_mb` | int | VRAM en MB |
| `tasks_completed` | int | Taches completees |
| `avg_tokens_per_sec` | float | Vitesse moyenne de generation |
| `trust_score` | int | Score de confiance (0-100, defaut 50) |
| `status` | string | Statut actuel du noeud |

```bash
curl "http://localhost:8000/api/compute/leaderboard?limit=5"
```

```json
{
  "entries": [
    {
      "rank": 1,
      "name": "FlowUP-RTX5080",
      "gpu_model": "NVIDIA RTX 5080",
      "vram_mb": 16384,
      "tasks_completed": 312,
      "avg_tokens_per_sec": 45.2,
      "trust_score": 92,
      "status": "idle"
    }
  ],
  "total_contributors": 5
}
```

---

### GET /api/compute/health

Endpoint de sante leger pour monitoring externe (style health.petals.dev). Reponse cachee 5 secondes.

**Response :** `dict` (pas de modele Pydantic dedie)

| Champ | Type | Description |
|-------|------|-------------|
| `status` | string | `"healthy"` si nodes_online > 0, sinon `"offline"` |
| `model` | string | Modele cible actif |
| `tier` | string | Tier du modele |
| `execution_mode` | string | `"local"`, `"exo"`, ou `"petals"` |
| `nodes_online` | int | Noeuds connectes |
| `nodes_total` | int | Total noeuds |
| `vram_total_gb` | float | VRAM totale reseau |
| `tasks_today` | int | Taches aujourd'hui |
| `tasks_completed` | int | Taches completees (total) |
| `uptime_pct` | float | Pourcentage uptime reseau |
| `total_node_hours_30d` | float | Heures-noeuds sur 30 jours |
| `swarm_health` | string | Sante du swarm Petals |
| `swarm_blocks` | string | Blocs couverts (e.g. `"24/32"`) |

```bash
curl http://localhost:8000/api/compute/health
```

```json
{
  "status": "healthy",
  "model": "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m",
  "tier": "26B",
  "execution_mode": "local",
  "nodes_online": 3,
  "nodes_total": 5,
  "vram_total_gb": 40.0,
  "tasks_today": 34,
  "tasks_completed": 847,
  "uptime_pct": 98.7,
  "total_node_hours_30d": 2160.5,
  "swarm_health": "offline",
  "swarm_blocks": "0/0"
}
```

---

### GET /api/compute/model/status

Etat actuel de la selection de modele et des transitions.

**Response model :** `ModelStatusResponse`

| Champ | Type | Description |
|-------|------|-------------|
| `target_model` | string | Modele cible actuel |
| `target_tier` | string | Tier cible |
| `previous_model` | string | Modele precedent |
| `transition_state` | string | `"stable"`, `"pulling"`, `"transitioning"` |
| `transition_started_at` | string? | ISO timestamp du debut de transition |
| `execution_mode` | string | `"local"`, `"exo"`, `"petals"` |
| `total_vram_gb` | float | VRAM totale disponible |
| `max_single_node_vram_gb` | float | VRAM max d'un seul noeud |
| `nodes_online` | int | Noeuds en ligne |
| `nodes_ready` | int | Noeuds prets avec le bon modele |
| `nodes_compatible` | int | Noeuds compatibles avec le modele cible |
| `nodes_pulling` | int | Noeuds en train de telecharger le modele |
| `readiness_pct` | float | Pourcentage de readiness (0-100) |

```bash
curl http://localhost:8000/api/compute/model/status
```

---

### GET /api/compute/model/assignments

Assignation modele par noeud (quel noeud doit charger quel modele).

**Response :** `list[NodeAssignment]`

| Champ | Type | Description |
|-------|------|-------------|
| `node_id` | string | ID du noeud |
| `name` | string | Nom du noeud |
| `vram_mb` | int | VRAM du noeud |
| `assigned_model` | string | Modele assigne |
| `current_model` | string | Modele actuellement charge |
| `ready` | bool | Modele assigne est pret |
| `needs_pull` | bool | Doit telecharger le modele |

```bash
curl http://localhost:8000/api/compute/model/assignments
```

```json
[
  {
    "node_id": "abc123",
    "name": "FlowUP-RTX5080",
    "vram_mb": 16384,
    "assigned_model": "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m",
    "current_model": "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m",
    "ready": true,
    "needs_pull": false
  }
]
```

---

### GET /api/compute/model/transitions

Historique des transitions de modele sur le reseau.

**Query params :**

| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `limit` | int | 20 | Nombre d'entrees (max 100) |

**Response :** `list[ModelTransitionEntry]`

| Champ | Type | Description |
|-------|------|-------------|
| `id` | string | ID de la transition |
| `old_model` | string | Ancien modele |
| `new_model` | string | Nouveau modele |
| `old_tier` | string | Ancien tier |
| `new_tier` | string | Nouveau tier |
| `total_vram_gb` | float | VRAM totale au moment de la transition |
| `nodes_online` | int | Noeuds en ligne |
| `nodes_ready` | int | Noeuds prets |
| `transition_state` | string | Etat de la transition |
| `started_at` | string? | ISO timestamp debut |
| `completed_at` | string? | ISO timestamp fin |

```bash
curl "http://localhost:8000/api/compute/model/transitions?limit=5"
```

---

### GET /api/compute/uptime

Statistiques d'uptime du reseau.

**Response :** `dict`

| Champ | Type | Description |
|-------|------|-------------|
| `uptime_pct` | float | Pourcentage d'uptime |
| `total_node_hours_30d` | float | Heures-noeuds cumulees sur 30 jours |

```bash
curl http://localhost:8000/api/compute/uptime
```

---

### GET /api/compute/badges

Badges et recompenses. Sans `node_id` : resume global. Avec `node_id` : badges d'un noeud specifique.

**Query params :**

| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `node_id` | string? | null | ID du noeud (optionnel) |

**Response (avec node_id) :**
```json
{
  "node_id": "abc123",
  "badges": ["early_adopter", "gpu_veteran", "speed_demon"]
}
```

**Response (sans node_id) :**
```json
{
  "summary": {
    "early_adopter": 3,
    "gpu_veteran": 1,
    "speed_demon": 2
  }
}
```

```bash
# Resume global
curl http://localhost:8000/api/compute/badges

# Badges d'un noeud
curl "http://localhost:8000/api/compute/badges?node_id=abc123"
```

---

### GET /api/compute/self-worker/status

Statut du self-worker integre (contribution GPU du serveur lui-meme).

**Response :** `dict`

| Champ | Type | Description |
|-------|------|-------------|
| `running` | bool | Self-worker actif |
| `message` | string | Message d'etat (si non initialise) |
| *(autres)* | | Champs supplementaires depuis `self_worker.get_status()` |

```bash
curl http://localhost:8000/api/compute/self-worker/status
```

---

## 4. Enregistrement node

### POST /api/compute/register

Enregistrer un nouveau noeud contributeur GPU. Retourne une cle API unique (affichee une seule fois).

**Rate limited :** Oui (100 req/min par IP)

**Request model :** `NodeRegisterRequest`

| Champ | Type | Requis | Description |
|-------|------|--------|-------------|
| `name` | string | Oui | Nom d'affichage (1-64 chars) |
| `gpu_model` | string | Oui | Modele GPU (1-128 chars, e.g. `"NVIDIA RTX 5080"`) |
| `vram_mb` | int | Oui | VRAM totale en MB (> 0, max 1048576) |
| `platform` | string | Non | OS (`"windows"`, `"linux"`, `"darwin"`) |
| `ollama_version` | string | Non | Version d'Ollama |
| `public_key_pem` | string | Non | Cle publique Ed25519 PEM pour signature des resultats |

**Response model :** `NodeRegisterResponse` (status 201)

| Champ | Type | Description |
|-------|------|-------------|
| `node_id` | string | ID unique du noeud |
| `api_key` | string | Cle API -- a conserver, affichee une seule fois |
| `name` | string | Nom confirme |
| `gpu_model` | string | GPU confirme |
| `vram_mb` | int | VRAM confirmee |
| `status` | string | Statut initial (`"idle"`) |

```bash
curl -X POST http://localhost:8000/api/compute/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "FlowUP-RTX5080",
    "gpu_model": "NVIDIA RTX 5080",
    "vram_mb": 16384,
    "platform": "windows",
    "ollama_version": "0.6.2"
  }'
```

```json
{
  "node_id": "d4e5f6a1-b2c3-4d5e-6f7a-8b9c0d1e2f3a",
  "api_key": "nxc_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6",
  "name": "FlowUP-RTX5080",
  "gpu_model": "NVIDIA RTX 5080",
  "vram_mb": 16384,
  "status": "idle"
}
```

---

## 5. Authentifie (Bearer token)

Tous les endpoints de cette section exigent : `Authorization: Bearer <api_key>`

### POST /api/compute/heartbeat

Heartbeat du noeud -- maintient la connexion et recoit les instructions de modele.

Si `model_required` dans la reponse differe du modele actuel du noeud, celui-ci doit telecharger le nouveau modele et signaler sa readiness via `POST /api/compute/model/ready`.

**Rate limited :** Oui

**Request model :** `NodeHeartbeatRequest`

| Champ | Type | Default | Description |
|-------|------|---------|-------------|
| `current_model` | string | `""` | Modele Ollama actuellement charge |
| `status` | string | `"idle"` | Statut auto-reporte du noeud |

**Response model :** `NodeHeartbeatResponse`

| Champ | Type | Description |
|-------|------|-------------|
| `status` | string | Statut assigne par le serveur |
| `model_required` | string | Modele que le noeud devrait avoir charge |
| `message` | string | Instructions (e.g. `"pull_model:gemma-4..."`) |

```bash
curl -X POST http://localhost:8000/api/compute/heartbeat \
  -H "Authorization: Bearer nxc_a1b2c3d4e5f6..." \
  -H "Content-Type: application/json" \
  -d '{"current_model": "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m", "status": "idle"}'
```

```json
{
  "status": "idle",
  "model_required": "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m",
  "message": ""
}
```

---

### GET /api/compute/task

Tirer la prochaine tache disponible pour ce noeud. Retourne 200 avec les donnees de la tache, ou **204 No Content** si la queue est vide.

**Rate limited :** Oui

**Response model :** `TaskPullResponse` (200) ou vide (204)

| Champ | Type | Description |
|-------|------|-------------|
| `task_id` | string | ID de la tache |
| `task_type` | string | Type de tache (e.g. `"summary"`, `"analysis"`) |
| `prompt` | string | Prompt a envoyer au LLM |
| `system_prompt` | string | System prompt (optionnel) |
| `model` | string | Modele a utiliser |
| `timeout_seconds` | int | Timeout en secondes (defaut 300) |
| `require_logprobs` | bool | Si le noeud doit retourner les logprobs |
| `calibration_prompt` | string | Prompt de calibration (anti-triche) |

```bash
curl http://localhost:8000/api/compute/task \
  -H "Authorization: Bearer nxc_a1b2c3d4e5f6..."
```

**Reponse 200 :**
```json
{
  "task_id": "task_001",
  "task_type": "summary",
  "prompt": "Resume le temoignage suivant...",
  "system_prompt": "Tu es un analyste forensique.",
  "model": "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m",
  "timeout_seconds": 300,
  "require_logprobs": false,
  "calibration_prompt": ""
}
```

**Reponse 204 :** corps vide, pas de tache disponible.

---

### POST /api/compute/result

Soumettre le resultat d'une tache. Le serveur valide, stocke et met a jour le trust score.

**Rate limited :** Oui

**Request model :** `TaskResultRequest`

| Champ | Type | Requis | Description |
|-------|------|--------|-------------|
| `task_id` | string | Oui | ID de la tache completee |
| `result_text` | string | Oui | Sortie du LLM (min 1 char) |
| `tokens_generated` | int | Non | Nombre de tokens generes (>= 0) |
| `generation_time_ms` | int | Non | Temps de generation en ms (>= 0) |
| `model_digest` | string | Non | SHA256 digest du fichier modele Ollama |
| `logprobs` | string | Non | Logprobs serialises en JSON (si demande) |
| `signature` | string | Non | Signature Ed25519 du payload |

**Response model :** `TaskResultResponse`

| Champ | Type | Description |
|-------|------|-------------|
| `accepted` | bool | Resultat accepte par le serveur |
| `task_id` | string | ID de la tache |
| `message` | string | Message explicatif |
| `trust_delta` | int | Variation du trust score (+/-) |

```bash
curl -X POST http://localhost:8000/api/compute/result \
  -H "Authorization: Bearer nxc_a1b2c3d4e5f6..." \
  -H "Content-Type: application/json" \
  -d '{
    "task_id": "task_001",
    "result_text": "Le temoignage revele trois points cles...",
    "tokens_generated": 256,
    "generation_time_ms": 5800,
    "model_digest": "sha256:abc123def456..."
  }'
```

```json
{
  "accepted": true,
  "task_id": "task_001",
  "message": "Result accepted",
  "trust_delta": 1
}
```

---

## 6. Gestion des modeles

### POST /api/compute/model/ready

**Auth :** Bearer token
**Rate limited :** Oui

Le noeud signale qu'il a fini de telecharger un modele (suite a une instruction `pull_model` via heartbeat).

**Request model :** `ModelReadyRequest`

| Champ | Type | Requis | Description |
|-------|------|--------|-------------|
| `model` | string | Oui | Nom du modele telecharge (min 1 char) |
| `model_digest` | string | Non | SHA256 digest du fichier modele |

**Response model :** `ModelReadyResponse`

| Champ | Type | Description |
|-------|------|-------------|
| `accepted` | bool | Readiness acceptee |
| `message` | string | Message du serveur |
| `transition_state` | string | Etat de la transition globale |
| `readiness_pct` | float | Pourcentage de noeuds prets |

```bash
curl -X POST http://localhost:8000/api/compute/model/ready \
  -H "Authorization: Bearer nxc_a1b2c3d4e5f6..." \
  -H "Content-Type: application/json" \
  -d '{"model": "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m", "model_digest": "sha256:abc..."}'
```

```json
{
  "accepted": true,
  "message": "Node ready",
  "transition_state": "stable",
  "readiness_pct": 100.0
}
```

---

## 7. Hybrid / Swarm

### GET /api/compute/hybrid/status

Statut du mode d'execution hybride (Ollama local vs exo distribue).

**Response model :** `HybridStatusResponse`

| Champ | Type | Description |
|-------|------|-------------|
| `execution_mode` | string | `"local"`, `"exo"`, ou `"petals"` |
| `exo_enabled` | bool | Exo est active dans la config |
| `exo_available` | bool | Exo est disponible (endpoint repond) |
| `exo_url` | string | Toujours vide (securite: URL interne jamais exposee) |
| `exo_model` | string | Modele exo configure |
| `max_single_node_vram_gb` | float | VRAM max d'un noeud unique |
| `target_model` | string | Modele cible |
| `target_tier` | string | Tier cible |

```bash
curl http://localhost:8000/api/compute/hybrid/status
```

```json
{
  "execution_mode": "local",
  "exo_enabled": false,
  "exo_available": false,
  "exo_url": "",
  "exo_model": "",
  "max_single_node_vram_gb": 16.0,
  "target_model": "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m",
  "target_tier": "26B"
}
```

---

### GET /api/compute/swarm/status

Statut du swarm Petals (Phase 7) : sante, couverture des blocs, noeuds.

**Response :** `dict`

| Champ | Type | Description |
|-------|------|-------------|
| `health` | string | `"online"`, `"degraded"`, ou `"offline"` |
| `model` | string | Modele Petals |
| `nodes_online` | int | Noeuds dans le swarm |
| `blocks_total` | int | Blocs totaux du modele |
| `blocks_covered` | int | Blocs couverts par le swarm |
| `coverage_pct` | float | Pourcentage de couverture (0-100) |
| `is_ready` | bool | Swarm pret pour l'inference |
| `throughput_tok_s` | float | Debit en tokens/seconde |

```bash
curl http://localhost:8000/api/compute/swarm/status
```

---

## 8. Self-Worker

Le self-worker est la contribution GPU integree du serveur NEXUS lui-meme.

### GET /api/compute/self-worker/status

Voir section [Public](#get-apicomputeself-workerstatus) ci-dessus.

### POST /api/compute/self-worker/pause

Mettre en pause le self-worker (arrete le traitement des taches, noeud reste en ligne).

**Response :**
```json
{"ok": true, "paused": true}
```

```bash
curl -X POST http://localhost:8000/api/compute/self-worker/pause
```

### POST /api/compute/self-worker/resume

Reprendre le self-worker (recommence a traiter les taches).

**Response :**
```json
{"ok": true, "paused": false}
```

```bash
curl -X POST http://localhost:8000/api/compute/self-worker/resume
```

---

## 9. Health / Uptime / Badges

### GET /api/compute/health

Voir section [Public](#get-apicomputehealth) ci-dessus.

### GET /api/compute/uptime

Voir section [Public](#get-apicomputeuptime) ci-dessus.

### GET /api/compute/badges

Voir section [Public](#get-apicomputebadges) ci-dessus.

### GET /api/compute/nodes/{node_id}/impact

Statistiques d'impact detaillees pour un noeud contributeur specifique.

**Path params :**

| Param | Type | Description |
|-------|------|-------------|
| `node_id` | string | ID du noeud |

**Response :** `dict` (champs depuis `db.get_node_impact()`)

**Erreur 404** si le noeud n'existe pas.

```bash
curl http://localhost:8000/api/compute/nodes/abc123/impact
```

---

## 10. Interne / Admin

Ces endpoints ne requierent pas d'auth Bearer mais sont destines a un usage interne (serveur NEXUS lui-meme ou admin).

### POST /api/compute/tasks

Creer une tache de calcul distribue (usage interne/admin).

**Request model :** `TaskCreateRequest`

| Champ | Type | Default | Description |
|-------|------|---------|-------------|
| `task_type` | string | *requis* | Type de tache |
| `prompt` | string | *requis* | Prompt LLM |
| `system_prompt` | string | `""` | System prompt |
| `model` | string | `""` | Modele force (sinon auto) |
| `priority` | int | 5 | Priorite 1 (basse) a 10 (haute) |
| `timeout_seconds` | int | 300 | Timeout (30-3600 sec) |
| `source_worker` | string | `""` | Worker source de la tache |
| `require_logprobs` | bool | false | Exiger les logprobs |
| `max_retries` | int | 3 | Max tentatives (0-10) |

**Response :** status 201

```json
{
  "task_id": "task_002",
  "status": "pending",
  "model": "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m"
}
```

```bash
curl -X POST http://localhost:8000/api/compute/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "analysis",
    "prompt": "Analyse les contradictions dans ce temoignage...",
    "system_prompt": "Tu es un analyste forensique expert.",
    "priority": 8,
    "timeout_seconds": 600
  }'
```

---

### GET /api/compute/nodes

Lister tous les noeuds de calcul enregistres.

**Query params :**

| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `status` | string? | null | Filtrer par statut (`"idle"`, `"busy"`, `"offline"`, `"banned"`) |

**Response :** `list[NodePublic]`

| Champ | Type | Description |
|-------|------|-------------|
| `id` | string | ID du noeud |
| `name` | string | Nom d'affichage |
| `gpu_model` | string | Modele GPU |
| `vram_mb` | int | VRAM en MB |
| `status` | string | Statut actuel |
| `tasks_completed` | int | Taches completees |
| `tasks_errored` | int | Taches echouees |
| `avg_tokens_per_sec` | float | Vitesse moyenne |
| `trust_score` | int | Score de confiance (0-100) |
| `connected_at` | string? | ISO timestamp de connexion |

```bash
# Tous les noeuds
curl http://localhost:8000/api/compute/nodes

# Noeuds actifs seulement
curl "http://localhost:8000/api/compute/nodes?status=idle"
```

```json
[
  {
    "id": "d4e5f6a1-b2c3-4d5e-6f7a-8b9c0d1e2f3a",
    "name": "FlowUP-RTX5080",
    "gpu_model": "NVIDIA RTX 5080",
    "vram_mb": 16384,
    "status": "idle",
    "tasks_completed": 312,
    "tasks_errored": 2,
    "avg_tokens_per_sec": 45.2,
    "trust_score": 92,
    "connected_at": "2026-04-09T10:30:00Z"
  }
]
```

---

## 11. Sync (synchronisation DB)

**Fichier :** `nexus/sync/api.py`
**Prefix :** `/api/sync` (REST) + `/ws/sync` (WebSocket)

Ces endpoints permettent la synchronisation en temps reel de la base de donnees SQLite entre le serveur NEXUS et les noeuds contributeurs.

---

### GET /api/sync/version

Version actuelle de la base de donnees pour la synchronisation.

**Response :**

| Champ | Type | Description |
|-------|------|-------------|
| `version` | int | Numero de version DB (0 si sync desactive) |
| `sync_enabled` | bool | Sync active sur ce serveur |
| `crsqlite` | bool | Extension cr-sqlite disponible |

```bash
curl http://localhost:8000/api/sync/version
```

```json
{
  "version": 847,
  "sync_enabled": true,
  "crsqlite": true
}
```

---

### GET /api/sync/tables

Liste des tables disponibles pour la synchronisation.

**Response :**

| Champ | Type | Description |
|-------|------|-------------|
| `tables` | list[string] | Noms des tables synchronisees |
| `sync_enabled` | bool | Sync active |

```bash
curl http://localhost:8000/api/sync/tables
```

```json
{
  "tables": ["cases", "evidence", "entities", "hypotheses", "alerts"],
  "sync_enabled": true
}
```

---

### GET /api/sync/status

Statut du systeme de synchronisation (broadcaster, clients connectes, version).

**Response :** `dict` (depuis `broadcaster.get_status()`)

| Champ | Type | Description |
|-------|------|-------------|
| `running` | bool | Broadcaster actif |
| `sync_enabled` | bool | Sync active |
| *(autres)* | | Champs supplementaires du broadcaster |

```bash
curl http://localhost:8000/api/sync/status
```

---

### GET /api/sync/snapshot

Telecharger le fichier base de donnees complet (sync initiale). Supporte le header `Range` pour reprise apres interruption.

**Response :** Fichier binaire `application/x-sqlite3` (nomme `nexus.db`)

**Erreur 404** si le fichier n'existe pas.

```bash
# Telecharger la DB complete
curl -o nexus.db http://localhost:8000/api/sync/snapshot

# Reprendre un download interrompu
curl -C - -o nexus.db http://localhost:8000/api/sync/snapshot
```

---

### WS /ws/sync

WebSocket pour la synchronisation en temps reel des changesets.

**Protocole :**

1. Le serveur envoie a la connexion : `{"type": "version", "version": N, "tables": [...]}`
2. A chaque modification DB : `{"type": "changes", "version": N, "changes": [[...]]}`
3. Keepalive toutes les 30s : `{"type": "ping", "version": N}`
4. Le client est read-only (aucun message attendu)

**Fermeture :** Code 1013 si le systeme de sync n'est pas disponible.

```javascript
const ws = new WebSocket("ws://localhost:8000/ws/sync");
ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  if (msg.type === "changes") {
    // Appliquer les changesets localement
    applyChanges(msg.changes);
  }
};
```

---

## Resume des endpoints

| # | Methode | Endpoint | Auth | Rate Limited | Description |
|---|---------|----------|------|--------------|-------------|
| 1 | GET | `/api/compute/stats` | Non | Non | Statistiques reseau |
| 2 | GET | `/api/compute/leaderboard` | Non | Non | Classement contributeurs |
| 3 | GET | `/api/compute/health` | Non | Non | Sante reseau (cache 5s) |
| 4 | GET | `/api/compute/model/status` | Non | Non | Etat modele + transition |
| 5 | GET | `/api/compute/model/assignments` | Non | Non | Assignations par noeud |
| 6 | GET | `/api/compute/model/transitions` | Non | Non | Historique transitions |
| 7 | GET | `/api/compute/hybrid/status` | Non | Non | Mode hybride Ollama/exo |
| 8 | GET | `/api/compute/swarm/status` | Non | Non | Swarm Petals |
| 9 | GET | `/api/compute/uptime` | Non | Non | Uptime reseau |
| 10 | GET | `/api/compute/badges` | Non | Non | Badges contributeurs |
| 11 | GET | `/api/compute/self-worker/status` | Non | Non | Statut self-worker |
| 12 | POST | `/api/compute/register` | Non* | Oui | Enregistrer un noeud GPU |
| 13 | POST | `/api/compute/heartbeat` | Bearer | Oui | Heartbeat noeud |
| 14 | GET | `/api/compute/task` | Bearer | Oui | Tirer une tache |
| 15 | POST | `/api/compute/result` | Bearer | Oui | Soumettre un resultat |
| 16 | POST | `/api/compute/model/ready` | Bearer | Oui | Signaler modele pret |
| 17 | POST | `/api/compute/tasks` | Interne | Non | Creer une tache |
| 18 | GET | `/api/compute/nodes` | Interne | Non | Lister les noeuds |
| 19 | GET | `/api/compute/nodes/{node_id}/impact` | Interne | Non | Impact d'un noeud |
| 20 | POST | `/api/compute/self-worker/pause` | Interne | Non | Pause self-worker |
| 21 | POST | `/api/compute/self-worker/resume` | Interne | Non | Resume self-worker |
| 22 | GET | `/api/sync/version` | Non | Non | Version DB sync |
| 23 | GET | `/api/sync/tables` | Non | Non | Tables synchronisees |
| 24 | GET | `/api/sync/status` | Non | Non | Statut sync |
| 25 | GET | `/api/sync/snapshot` | Non | Non | Download DB complete |
| 26 | WS | `/ws/sync` | Non | Non | WebSocket changesets |

\* `POST /register` ne requiert pas d'auth car c'est l'endpoint de creation d'auth.

---

## Codes d'erreur communs

| Code | Signification |
|------|---------------|
| 200 | Succes |
| 201 | Cree (register, tasks) |
| 204 | Pas de contenu (task queue vide) |
| 401 | API key invalide ou manquante |
| 403 | Noeud banni |
| 404 | Noeud ou ressource non trouve |
| 429 | Rate limit depasse (100 req/min) |
| 503 | Systeme compute non initialise |
