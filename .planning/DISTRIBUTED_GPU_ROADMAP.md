# NEXUS GOV — Partage de Puissance GPU Distribuee

## Vision
Les citoyens partagent leur GPU pour rendre l'IA politique plus puissante.
Un serveur central orchestre, les contributeurs calculent, tout le monde voit.
Le modele LLM s'adapte automatiquement a la puissance disponible.

---

## Architecture

```
                 SERVEUR CENTRAL (M5 Ultra ou VPS)
                 ┌──────────────────────────────────┐
                 │  NEXUS Master                     │
                 │  - Base de donnees (SQLite/PG)    │
                 │  - 31 workers (scraping+analyse)  │
                 │  - API REST + SSE + Frontend      │
                 │  - Task Queue (jobs LLM)          │
                 │  - GPU Registry (noeuds connectes)│
                 │  - Model Selector (auto-scaling)  │
                 │  - Result Validator               │
                 └───────────────┬──────────────────┘
                                 │
              WebSocket/gRPC (persistent connections)
                                 │
          ┌──────────────────────┼──────────────────────┐
          │                      │                      │
   Contributeur A          Contributeur B         Contributeur C
   ┌──────────────┐       ┌──────────────┐       ┌──────────────┐
   │ RTX 5080     │       │ RTX 4090     │       │ Mac M4 Pro   │
   │ 16 GB VRAM   │       │ 24 GB VRAM   │       │ 36 GB unified│
   │              │       │              │       │              │
   │ nexus-worker │       │ nexus-worker │       │ nexus-worker │
   │ (Python CLI) │       │ (Python CLI) │       │ (Python CLI) │
   │              │       │              │       │              │
   │ Ollama local │       │ Ollama local │       │ Ollama local │
   │ (modele      │       │ (modele      │       │ (modele      │
   │  assigne)    │       │  assigne)    │       │  assigne)    │
   └──────────────┘       └──────────────┘       └──────────────┘

   Chaque contributeur:
   1. Installe nexus-worker (pip install nexus-worker)
   2. Lance: nexus-worker connect nexusgov.fr
   3. Son GPU recoit des taches LLM du serveur
   4. Il voit son dashboard contributeur
   5. Il peut consulter nexusgov.fr comme tout le monde
```

---

## Phase 1 — Infrastructure de base (Sprint 1, ~3 jours)

### 1.1 GPU Registry (serveur)
Table `gpu_nodes` dans la BDD:
```sql
CREATE TABLE gpu_nodes (
    id TEXT PRIMARY KEY,
    name TEXT,                    -- pseudo du contributeur
    gpu_model TEXT,               -- "NVIDIA RTX 5080"
    vram_mb INTEGER,              -- 16384
    status TEXT DEFAULT 'idle',   -- idle, busy, offline, banned
    connected_at DATETIME,
    last_heartbeat DATETIME,
    tasks_completed INTEGER DEFAULT 0,
    tasks_errored INTEGER DEFAULT 0,
    avg_tokens_per_sec REAL,
    api_key_hash TEXT,            -- auth token hashe
    ip_hash TEXT,                 -- IP hashee (pas stockee en clair)
);
```

### 1.2 Task Queue (serveur)
Table `gpu_tasks`:
```sql
CREATE TABLE gpu_tasks (
    id TEXT PRIMARY KEY,
    task_type TEXT,               -- contradiction_detection, sentiment, summary, etc.
    prompt TEXT,                  -- le texte a analyser
    model TEXT,                   -- modele requis (auto-selectionne)
    status TEXT DEFAULT 'pending',-- pending, assigned, completed, failed, expired
    assigned_to TEXT,             -- gpu_node.id
    assigned_at DATETIME,
    completed_at DATETIME,
    result TEXT,                  -- JSON reponse du LLM
    result_validated BOOLEAN,     -- le serveur a valide le resultat
    priority INTEGER DEFAULT 5,  -- 1=urgent, 10=batch
    timeout_seconds INTEGER DEFAULT 300,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
);
```

### 1.3 API Endpoints (serveur)
```
POST   /api/compute/register     -- GPU node s'enregistre (GPU model, VRAM, name)
                                    → retourne api_key

POST   /api/compute/heartbeat    -- GPU node signale qu'il est vivant
                                    → retourne status (idle/busy)

GET    /api/compute/task          -- GPU node pull la prochaine tache
                                    → retourne {task_id, type, prompt, model}
                                    → ou 204 No Content si pas de tache

POST   /api/compute/result        -- GPU node soumet le resultat
                                    → serveur valide et stocke

GET    /api/compute/stats          -- Stats publiques du reseau
                                    → {nodes: 5, vram_total: 100GB, tasks_today: 342}

GET    /api/compute/leaderboard    -- Top contributeurs
                                    → [{name, tasks, uptime, gpu_model}]
```

### 1.4 Securite
- Chaque node recoit un `api_key` unique a l'enregistrement
- Toutes les requetes authentifiees par header `Authorization: Bearer <api_key>`
- IP hashee (pas stockee en clair — on ne traque pas les contributeurs)
- Rate limiting: max 100 tasks/minute par node
- Verification croisee: 5% des taches envoyees a 2 nodes differents
  - Si resultats divergent → les deux sont marques "a verifier"
  - Node avec >10% de divergences → banni automatiquement
- Le prompt envoye ne contient JAMAIS de donnees personnelles
  - Juste le texte politique public a analyser

---

## Phase 2 — Auto-scaling du modele LLM (Sprint 2, ~2 jours)

### 2.1 Paliers de modele
```python
MODEL_TIERS = [
    {"min_vram_gb": 0,   "model": "gemma-4-12b-q4",          "label": "Basique"},
    {"min_vram_gb": 14,  "model": "gemma-4-26b-q4",          "label": "Standard"},
    {"min_vram_gb": 40,  "model": "llama-3.1-70b-q4",        "label": "Avance"},
    {"min_vram_gb": 80,  "model": "qwen-2.5-110b-q4",        "label": "Pro"},
    {"min_vram_gb": 150, "model": "llama-3.1-405b-q2",       "label": "Ultra"},
    {"min_vram_gb": 300, "model": "llama-3.1-405b",           "label": "Maximum"},
]
```

### 2.2 Logique de selection automatique
```python
class ModelSelector:
    def __init__(self):
        self._current_model = None
        self._check_interval = 60  # secondes

    async def select_best_model(self, gpu_registry) -> str:
        """Selectionne le meilleur modele selon la VRAM totale disponible."""
        nodes = gpu_registry.get_online_nodes()
        total_vram_gb = sum(n.vram_mb for n in nodes) / 1024

        best = MODEL_TIERS[0]
        for tier in MODEL_TIERS:
            if total_vram_gb >= tier["min_vram_gb"]:
                best = tier

        new_model = best["model"]
        if new_model != self._current_model:
            logger.info(
                "Model auto-switch: {} -> {} ({:.0f} GB VRAM, {} nodes)",
                self._current_model, new_model, total_vram_gb, len(nodes)
            )
            # Notifier tous les nodes de pull le nouveau modele
            await self._notify_model_change(nodes, new_model)
            self._current_model = new_model

        return new_model
```

### 2.3 Notification de changement de modele
Quand la VRAM totale change (node se connecte/deconnecte):
1. ModelSelector recalcule le meilleur modele
2. Si changement → envoie a tous les nodes: `{"action": "pull_model", "model": "llama-3.1-70b-q4"}`
3. Chaque node fait `ollama pull <model>` en background
4. Quand pret → signale au serveur
5. Serveur commence a assigner des taches avec le nouveau modele
6. Graceful transition: taches en cours finissent avec l'ancien modele

### 2.4 Mode mixte (pendant la transition)
```
Situation: passage de 26B a 70B, 2 nodes sur 3 ont pull le 70B

  Tache urgente (contradiction)  → assigner aux 2 nodes avec 70B
  Tache batch (sentiment)        → assigner au node encore en 26B
  
  Quand le 3eme node a fini le pull → tout passe en 70B
```

---

## Phase 3 — Client contributeur (Sprint 3, ~2 jours)

### 3.1 Package Python installable
```bash
pip install nexus-worker
```

### 3.2 CLI simple
```bash
# S'enregistrer
nexus-worker register --server nexusgov.fr --name "MonPseudo"
  → Enregistre le GPU, recoit un api_key
  → Stocke dans ~/.nexus-worker/config.json

# Lancer le worker
nexus-worker start
  → Se connecte au serveur
  → Pull le modele assigne
  → Commence a traiter des taches
  → Affiche un dashboard Rich en direct

# Voir les stats
nexus-worker stats
  → Tasks completed, uptime, leaderboard position
```

### 3.3 Dashboard contributeur (Rich TUI)
```
┌─────────────────────────────────────────────┐
│  NEXUS GPU Contributor — MonPseudo          │
│                                             │
│  GPU: NVIDIA RTX 5080 (16 GB)              │
│  Model: llama-3.1-70b-q4 (partage)         │
│  Status: Processing task #4521             │
│                                             │
│  Session: 342 tasks | 4h 23m uptime         │
│  Speed: 45 tokens/s                         │
│  Total: 1,203 tasks contributed             │
│                                             │
│  Current task:                              │
│    Type: contradiction_detection             │
│    Progress: generating... (12s)             │
│                                             │
│  Network:                                   │
│    Nodes online: 5 (76 GB VRAM total)       │
│    Model actif: llama-3.1-70b-q4            │
│    Tasks/hour: 1,200                        │
│                                             │
│  Leaderboard:                               │
│    1. FlowUP         1,203 tasks            │
│    2. CitoyenXY        987 tasks            │
│    3. → MonPseudo      342 tasks ←          │
│                                             │
│  [Q] Quit  [P] Pause  [S] Stats            │
└─────────────────────────────────────────────┘
```

### 3.4 Mode Ollama local
Chaque contributeur a Ollama installe localement:
```
nexus-worker start
  1. Detecte GPU (nvidia-smi / system_profiler)
  2. Se connecte a nexusgov.fr/api/compute/register
  3. Recoit le modele a charger: "llama-3.1-70b-q4"
  4. ollama pull llama-3.1-70b-q4 (si pas deja present)
  5. Boucle:
     a. GET /api/compute/task → recoit un prompt
     b. ollama.generate(model, prompt) → resultat
     c. POST /api/compute/result → envoie au serveur
     d. Retour a (a)
```

Avantage: le modele tourne EN LOCAL sur le GPU du contributeur.
Le serveur n'envoie que le prompt (quelques KB) et recoit le resultat (quelques KB).
Pas besoin de bande passante enorme.

---

## Phase 4 — Mode exo (modele splitte, Sprint 4, ~3 jours)

### 4.1 Quand utiliser exo vs Ollama local
```
Ollama local (Phase 3):
  - Chaque node a le modele COMPLET en VRAM
  - Fonctionne si le modele tient dans la VRAM du node
  - 26B q4 (14GB) → OK pour une 5080 (16GB)
  - 70B q4 (40GB) → NE TIENT PAS dans une seule 5080

exo distribue (Phase 4):
  - Le modele est SPLITTE entre plusieurs machines
  - 70B q4 (40GB) → split sur 3 machines de 16GB
  - Necessite une connexion reseau entre les nodes
  - Plus lent (latence reseau) mais permet les gros modeles
```

### 4.2 Integration exo
```python
# Sur le serveur
# exo cree un endpoint compatible OpenAI
# NEXUS pointe dessus au lieu d'Ollama local

OLLAMA_HOST = "http://localhost:11434"          # mode local (defaut)
EXO_HOST = "http://localhost:52415"             # mode distribue

# Le ModelSelector choisit:
if total_vram > 40 and model_requires > single_node_vram:
    # Mode distribue necessaire
    use_exo = True
    # Lancer exo avec tous les nodes
else:
    # Mode local suffit
    use_exo = False
    # Chaque node utilise son Ollama local
```

### 4.3 Hybrid mode
```
Taches legeres (sentiment, classification):
  → Ollama local sur chaque node (rapide, pas de latence reseau)
  
Taches lourdes (contradiction profonde, resume long):
  → exo distribue sur tous les nodes (modele 70B+ splitte)
  
Le serveur decide automatiquement selon:
  - Type de tache
  - Taille du contexte
  - Urgence
```

---

## Phase 5 — Dashboard public + gamification (Sprint 5, ~2 jours)

### 5.1 Page publique /network sur nexusgov.fr
```
NEXUS Network — Puissance Citoyenne

  Contributeurs actifs:  5
  VRAM totale:          76 GB
  Modele actif:         Llama 70B q4
  Tasks aujourd'hui:    1,234
  Uptime reseau:        99.2%

  ┌─ Leaderboard ──────────────────────────┐
  │  #  Pseudo        GPU       Tasks     │
  │  1  FlowUP       5080      12,403    │
  │  2  CitoyenXY    4090       8,721    │
  │  3  DemocrateZ   Mac M4     5,432    │
  │  4  JusteJean    3090       3,211    │
  │  5  LibreFR      5070 Ti    2,876    │
  └────────────────────────────────────────┘

  Puissance dans le temps:
  [graph: VRAM totale par jour sur 30 jours]

  Comment contribuer:
  pip install nexus-worker
  nexus-worker register --server nexusgov.fr
  nexus-worker start
```

### 5.2 Badges contributeurs
- "Premiere tache" — 1 tache completee
- "Centurion" — 100 taches
- "Millionnaire" — 1,000 taches
- "Pilier" — 10,000 taches
- "24/7" — uptime > 7 jours consecutifs
- "Early Adopter" — parmi les 10 premiers contributeurs
- "Power Node" — VRAM > 24 GB

---

## Phase 6 — Securite avancee : Proof-of-Computation (Sprint 6, ~3 jours)

### 6.0 Pourquoi pas zkML (zero-knowledge proofs) ?

```
zkML (EZKL, DeepProve, Giza) = prouver cryptographiquement qu'un modele a tourne.
Probleme: 180x overhead. Un 70B prendrait des HEURES a prouver.
Fonctionne uniquement pour des petits modeles (<1M params).
Inutilisable pour des LLM sur hardware consommateur.

Sources:
  - EZKL: github.com/zkonduit/ezkl (1.2K stars) — max ~1M params
  - DeepProve: github.com/Lagrange-Labs/deep-prove (3.3K stars) — CNNs/MLPs
  - NANOZK (arxiv 2603.18046) — prometteur mais paper only
  
Verdict: zkML est a 5+ ans de pouvoir prouver un LLM 70B.
On utilise une approche plus pragmatique ci-dessous.
```

### 6.1 Protocole de verification 3 couches

Le systeme combine 3 niveaux de verification independants.
Un attaquant doit tromper les 3 simultanement pour passer.

```
┌─────────────────────────────────────────────────────────────┐
│ COUCHE 1: Signature Ed25519 (qui a produit le resultat)     │
│   → Prouve l'IDENTITE du contributeur                       │
│   → Non-repudiation (impossible de nier)                    │
│                                                             │
│ COUCHE 2: Ollama digest (quel modele est charge)            │
│   → Prouve que le MODELE est present sur la machine         │
│   → SHA256 du fichier de poids = unique par modele          │
│                                                             │
│ COUCHE 3: Logprob fingerprinting (le modele a-t-il tourne)  │
│   → Prouve que le BON MODELE a genere la reponse            │
│   → Distribution de probabilites unique par modele          │
│   → >95% accuracy (paper LLMmap, USENIX Security 2025)     │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 Couche 1 — Signature cryptographique Ed25519

```python
# A l'enregistrement du contributeur:
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

# Contributeur genere sa paire de cles
private_key = Ed25519PrivateKey.generate()
public_key = private_key.public_key()

# Envoie la cle publique au serveur lors du register
# Garde la cle privee en local (~/.nexus-worker/key.pem)


# A chaque resultat:
import json, time

result_payload = {
    "task_id": "abc123",
    "result": {"contradiction": True, "description": "..."},
    "model_digest": "sha256:a4f8c2e3...",
    "timestamp": time.time(),
    "node_id": "monpseudo",
}

# Signer le payload
message = json.dumps(result_payload, sort_keys=True).encode()
signature = private_key.sign(message)

# Envoyer au serveur: payload + signature


# Le serveur verifie:
public_key.verify(signature, message)  # Leve une exception si invalide
# → Prouve que "monpseudo" a produit ce resultat
# → Impossible de falsifier sans la cle privee
```

**Ce que ca prouve:** QUI a envoye le resultat.
**Ce que ca ne prouve PAS:** Que le bon modele a tourne.

### 6.3 Couche 2 — Ollama model digest

```python
# Ollama expose le SHA256 du modele charge:
# GET http://localhost:11434/api/show
# Body: {"name": "llama-3.1-70b-q4"}
#
# Response:
# {
#   "digest": "sha256:a4f8c2e3b7d9...",
#   "details": {
#     "family": "llama",
#     "parameter_size": "70B",
#     "quantization_level": "Q4_K_M"
#   }
# }

# Le serveur maintient une whitelist de digests valides:
VALID_DIGESTS = {
    "llama-3.1-70b-q4": "sha256:a4f8c2e3b7d9...",
    "gemma-4-26b-q4":   "sha256:7b2e1f4a8c6d...",
    # ... etc
}

# A chaque tache, le contributeur inclut son digest
# Le serveur verifie: digest == VALID_DIGESTS[model_expected]
```

**Ce que ca prouve:** Le modele est CHARGE dans Ollama.
**Ce que ca ne prouve PAS:** Que ce modele a ete UTILISE pour cette tache specifique.
(Un attaquant pourrait charger le 70B pour le digest mais executer un 7B)

### 6.4 Couche 3 — Logprob fingerprinting (la vraie preuve)

```python
# Chaque modele a une distribution de probabilites UNIQUE.
# Meme prompt → logprobs differents selon le modele.
#
# Reference: LLMmap (github.com/pasquini-dario/LLMmap)
# 8 queries suffisent pour identifier un modele avec >95% accuracy
# Paper: USENIX Security 2025

# Le serveur maintient des "profils logprob" de reference:
# (calibres une fois sur un GPU trusted)

CALIBRATION_PROMPTS = [
    "La capitale de la France est",
    "Le president de la Republique en 2026 est",
    "L'article 49.3 permet au gouvernement de",
    "Le nombre de deputes a l'Assemblee nationale est",
]

# Pour chaque prompt, le profil du 70B est:
LOGPROB_PROFILES = {
    "llama-3.1-70b-q4": {
        "La capitale de la France est": {
            "Paris": -0.03,    # logprob ~0.97
            "Lyon": -4.6,
            "Marseille": -5.3,
        },
        # ... etc pour chaque prompt
    },
    "llama-3.1-7b-q4": {
        "La capitale de la France est": {
            "Paris": -0.12,    # logprob ~0.89 — DIFFERENT du 70B
            "Lyon": -3.2,      # DIFFERENT
            "Marseille": -3.9, # DIFFERENT
        },
    },
}

# Verification:
# 1. Serveur envoie 1-2 prompts de calibration (meles aux vraies taches)
# 2. Contributeur retourne les logprobs (Ollama les expose nativement)
# 3. Serveur compare avec le profil de reference
# 4. Distance KL-divergence < seuil → modele verifie

import numpy as np

def verify_logprobs(expected_profile, actual_logprobs, threshold=0.1):
    """Kullback-Leibler divergence entre profils attendus et recus."""
    for token, expected_logp in expected_profile.items():
        actual_logp = actual_logprobs.get(token, -10.0)
        if abs(expected_logp - actual_logp) > threshold:
            return False  # Divergence trop grande → mauvais modele
    return True
```

**Ce que ca prouve:** Le VRAI MODELE a genere la reponse.
**Precision:** >95% (LLMmap) avec seulement 8 queries.
**Overhead:** Quasi-zero — les logprobs sont deja calcules pendant l'inference.
**Reference:** arxiv.org/abs/2512.03816 — "1 token suffit pour detecter un changement de modele"

### 6.5 Protocole complet par tache

```
1. SERVEUR assigne une tache au contributeur:
   {
     "task_id": "abc123",
     "prompt": "Analyse cette contradiction entre...",
     "calibration_prompt": "La capitale de la France est",  // 10% des taches
     "require_logprobs": true,
   }

2. CONTRIBUTEUR execute:
   a. Ollama genere la reponse + logprobs
   b. Lit le digest via /api/show
   c. Signe le tout: Ed25519(response + logprobs + digest + timestamp)
   d. Envoie au serveur

3. SERVEUR verifie (3 checks, <1ms total):
   ✓ Check 1: Ed25519 signature valide (cle publique connue)
   ✓ Check 2: Model digest == whitelist[model_attendu]
   ✓ Check 3: Logprobs matchent le profil de reference (si calibration_prompt)
   
   Si les 3 passent → resultat accepte, trust_score +1
   Si check 3 echoue → node suspecte, spot-check sur GPU trusted
   Si check 1 ou 2 echoue → node banni immediatement
```

### 6.6 BOINC-style spot-checking (filet de securite)

```python
# En complement des 3 couches, 5% des taches sont aussi executees
# sur le GPU trusted du serveur (ou un node de confiance).
# Si le resultat diverge → le node distant est flagge.

# Ref: BOINC (github.com/BOINC/boinc, 2.3K stars)
# Utilise depuis 20 ans, prouve que ca marche a grande echelle.

async def spot_check(task, remote_result, node):
    if random.random() < 0.05:  # 5% spot-check
        trusted_result = await run_on_trusted_gpu(task)
        similarity = compare_results(remote_result, trusted_result)
        if similarity < 0.8:
            node.trust_score -= 20
            if node.trust_score < 20:
                ban_node(node)
            return False
    return True
```

### 6.7 Score de confiance par node

```python
# Chaque node a un trust_score evolue au fil du temps
#
# Debut: 50 (neutre)
# +1 par tache acceptee (signature + digest + logprobs OK)
# +5 par spot-check reussi (resultat identique au trusted GPU)
# -20 par spot-check echoue (resultat divergent)
# -50 par signature invalide
# < 20 → banni automatiquement
# > 80 → "trusted node" (spot-checks reduits a 1%)
#
# Les nodes trusted sont moins verifies → incentive a etre honnete

class NodeTrustManager:
    def reward(self, node_id, points=1):
        self.scores[node_id] = min(100, self.scores.get(node_id, 50) + points)
    
    def penalize(self, node_id, points=20):
        self.scores[node_id] = max(0, self.scores.get(node_id, 50) - points)
        if self.scores[node_id] < 20:
            self.ban(node_id)
    
    def get_spot_check_rate(self, node_id):
        score = self.scores.get(node_id, 50)
        if score > 80: return 0.01  # 1% pour les trusted
        if score > 50: return 0.05  # 5% standard
        return 0.20                  # 20% pour les suspects
```

### 6.8 Isolation des donnees

- Le prompt envoye au node ne contient QUE le texte a analyser
- JAMAIS de metadata (politician_id, source_url, etc.)
- Le node ne sait PAS quel politicien il analyse
- Empeche un node malveillant de cibler un politicien specifique
- Les prompts de calibration sont indistinguables des vrais prompts

### 6.9 Cout total du systeme de verification

```
Par tache:
  Signature Ed25519:        0.1ms  (negligeable)
  Digest check:             0ms    (comparaison de string)
  Logprob check (10%):      0.5ms  (KL-divergence sur 5 tokens)
  Spot-check (5%):          30-60s (re-execution sur trusted GPU)
  
Overhead total: <0.01% en temps, ~5% en compute (spot-checks)

Comparaison avec zkML:
  zkML overhead:            180x (= 18,000%)
  Notre overhead:           5%
  Difference:               3,600x moins cher
```

### 6.10 Projets open source utilises

| Projet | GitHub | Stars | Role |
|---|---|---|---|
| LLMmap | github.com/pasquini-dario/LLMmap | 250 | Fingerprinting actif (8 queries, >95%) |
| BOINC | github.com/BOINC/boinc | 2.3K | Pattern spot-checking (20 ans de recul) |
| opML | github.com/ora-io/opml | 315 | Fraud proofs optimistes (challenge-response) |
| PyNaCl/cryptography | github.com/pyca/cryptography | 7K | Ed25519 signatures |
| Ollama | github.com/ollama/ollama | 156K | Model digest SHA256 + logprobs natifs |

---

## Phase 7 — Petals : Modele 405B sur 50 GPUs fibre (Sprint 7, ~3 jours)

### 7.1 Pourquoi Petals (et pas exo/llama.cpp RPC)

| Critere | Petals | exo | llama.cpp RPC | GPUStack |
|---|---|---|---|---|
| Fonctionne over internet | **OUI (concu pour)** | LAN seulement | Theorique | OUI |
| Fault tolerance | **OUI (dual cache, auto-reroute)** | NON | NON | Partiel |
| GPU hetero (mix 3060+4090+5080) | OUI (CUDA) | NON | OUI | OUI |
| Maturite | 10K stars, paper peer-reviewed | Immature NVIDIA | Proof-of-concept | Production |
| 405B teste | **OUI (swarm actif)** | NON | 0.7 tok/s | NON |

Petals est le seul framework qui:
- A un swarm PUBLIC actif avec Llama 405B en ce moment
- Gere les deconnexions (fault tolerance dual-cache)
- Est concu pour internet, pas juste LAN

### 7.2 Maths: 50 GPUs fibre francaise sur 405B q4

```
Ressources:
  50 GPUs NVIDIA (mix 3060/3080/4060/4070/4090/5080)
  VRAM moyenne: ~14GB par GPU
  VRAM totale: ~700 GB
  Modele: Llama 3.1 405B q4 = 230 GB
  → 230GB / 700GB = chaque GPU heberge ~33% de sa VRAM en blocs
  → ~80 blocs transformer / 50 GPUs = ~1.6 blocs par GPU (leger)

Latence fibre:
  Meme ville: ~5ms RTT
  Paris↔Lyon: ~10ms RTT
  Paris↔Marseille: ~15ms RTT
  Moyenne nationale: ~8ms RTT

Performance single-request:
  Round trips par token: ~50 (1 par GPU dans le pipeline)
  Overhead reseau: 50 × 8ms = 400ms
  Compute par token: ~100ms
  Total par token: ~500ms = ~2 tok/s

Performance batch (pipeline parallelism):
  10 requetes paralleles: ~20 tok/s agrege
  50 requetes paralleles: ~80 tok/s agrege
  (les pipeline bubbles sont remplis par d'autres requetes)
```

### 7.3 Impact concret pour NEXUS

```
Avec 50 contributeurs fibre + Petals + 405B:

  Analyse de contradiction:  ~30s par paire (2 tok/s single)
  Batch 50 contradictions:   ~30s total (pipeline parallel)
  1145 politiciens:          ~12 minutes (analyse complete)
  
  Qualite:
  - 405B = comparable a GPT-4 / Claude 3 Opus
  - Comprend le jargon juridique francais
  - Detecte les nuances (ironie, double sens, contexte)
  - Analyse des documents de 50+ pages (128K context)
  
  vs setup actuel (26B seul):
  - 15x plus de parametres
  - Detection de contradictions subtiles impossible avec 26B
  - Resumes de scrutins plus precis et detailles
  - Biographies plus riches et mieux sourcees
```

### 7.4 Setup Petals pour NEXUS

```bash
# === SUR CHAQUE CONTRIBUTEUR (50 machines) ===

pip install petals

# Le contributeur lance son serveur Petals
python -m petals.cli.run_server \
    meta-llama/Meta-Llama-3.1-405B \
    --port 31330 \
    --num_blocks 2 \
    --torch_dtype float16

# Le serveur detecte automatiquement le GPU
# et heberge le nombre de blocs qui tient en VRAM


# === SUR LE SERVEUR NEXUS (orchestrateur) ===

pip install petals

# Utilisation dans le code Python:
from petals import AutoDistributedModelForCausalLM
from transformers import AutoTokenizer

model_name = "meta-llama/Meta-Llama-3.1-405B"
tokenizer = AutoTokenizer.from_pretrained(model_name)
model = AutoDistributedModelForCausalLM.from_pretrained(model_name)

# Utilise comme un modele HuggingFace standard
# Petals route automatiquement vers les contributeurs
inputs = tokenizer("Analyse cette contradiction...", return_tensors="pt")
outputs = model.generate(**inputs, max_new_tokens=500)
result = tokenizer.decode(outputs[0])
```

### 7.5 Integration avec NEXUS LLMRouter

```python
# nexus/llm/router.py — ajouter un backend Petals

class PetalsBackend:
    """Backend LLM distribue via Petals (50+ GPUs citoyens)."""
    
    def __init__(self, model_name: str):
        from petals import AutoDistributedModelForCausalLM
        from transformers import AutoTokenizer
        self.tokenizer = AutoTokenizer.from_pretrained(model_name)
        self.model = AutoDistributedModelForCausalLM.from_pretrained(model_name)
    
    async def generate(self, prompt: str, max_tokens: int = 500) -> str:
        inputs = self.tokenizer(prompt, return_tensors="pt")
        outputs = await asyncio.to_thread(
            self.model.generate, **inputs, max_new_tokens=max_tokens
        )
        return self.tokenizer.decode(outputs[0], skip_special_tokens=True)

# LLMRouter choisit automatiquement:
# - Ollama local (26B) pour les taches rapides/simples
# - Petals distribue (405B) pour les taches complexes
```

### 7.6 Fault tolerance: quand des contributeurs partent

```
Scenario: 50 contributeurs a 21h → 20 contributeurs a 3h du matin

Petals gere automatiquement:
  1. Detecte les noeuds deconnectes (heartbeat)
  2. Reroute les blocs vers les noeuds restants
  3. Les noeuds restants hebergent plus de blocs chacun

Auto-scaling modele:
  50 GPUs (700GB) → Llama 405B q4 (230GB) → OK
  30 GPUs (420GB) → Llama 405B q4 (230GB) → OK (plus lent)
  15 GPUs (210GB) → Llama 405B q4 (230GB) → LIMITE → switch 405B q2
  10 GPUs (140GB) → Auto-switch → Qwen 110B q4 (65GB)
   5 GPUs (70GB)  → Auto-switch → Llama 70B q4 (40GB)
   1 GPU  (16GB)  → Auto-switch → Gemma 26B q4 (14GB) [fallback local]

Le ModelSelector surveille le swarm toutes les 60s
et switch le modele si necessaire.
Transitions graceful: taches en cours finissent sur l'ancien modele.
```

### 7.7 Contribution par taille de GPU

```
GTX 1060 6GB:
  → NE PARTICIPE PAS au split 405B (trop petit, ralentit le pipeline)
  → Ollama local avec Gemma 12B q4
  → Taches: sentiment, classification thematique
  → Contribution: ~50 taches/heure

RTX 3060 12GB:
  → 1 bloc du 405B (~3GB par bloc)
  → OU Ollama local avec 26B q4
  → Taches: sentiment, classification, resumes courts
  → Contribution: ~80 taches/heure

RTX 3080/4060/5080 16GB:
  → 2-3 blocs du 405B
  → Noeud standard du swarm Petals
  → Contribution: ~120 taches/heure

RTX 4090 24GB:
  → 4-5 blocs du 405B
  → Noeud puissant du swarm
  → Contribution: ~200 taches/heure

Mac M4 Pro 48GB:
  → NE PARTICIPE PAS a Petals (pas CUDA)
  → Ollama local avec 70B q4 complet
  → Taches: contradictions profondes, biographies
  → Contribution: ~150 taches/heure (modele plus gros mais local)
```

---

## Phase 8 — Swarm public permanent (Sprint 8, ~2 jours)

### 8.1 Swarm NEXUS GOV toujours actif

```
Le swarm Petals NEXUS est public et permanent:
- Adresse: swarm.nexusgov.fr:31330
- Modele: Llama 405B (ou le plus gros possible)
- Monitoring: health.nexusgov.fr (comme health.petals.dev)
- N'importe qui peut rejoindre avec:
    python -m petals.cli.run_server meta-llama/Meta-Llama-3.1-405B \
        --initial_peers swarm.nexusgov.fr:31330
```

### 8.2 Monitoring public du swarm

```
swarm.nexusgov.fr/health

  Swarm NEXUS GOV
  
  Modele actif:     Llama 3.1 405B q4
  Noeuds:           47/50 en ligne
  VRAM totale:      658 GB
  Blocs couverts:   80/80 (100%)
  Throughput:       ~75 tok/s (batch)
  Uptime:           99.4% (30 jours)
  
  Carte des noeuds:
  [carte de France avec points par ville]
  Paris: 12 noeuds
  Lyon: 8 noeuds
  Marseille: 5 noeuds
  ...
```

### 8.3 Dashboard contributeur enrichi

```
nexusgov.fr/contribute

  Votre contribution:
  
  GPU: NVIDIA RTX 4060 (16 GB)
  Blocs heberges: 2/80 (blocs #34-#35)
  Status: En ligne depuis 3j 14h
  
  Statistiques:
  - 4,521 tokens generes cette semaine
  - 892 analyses de contradictions assistees
  - Top 15% des contributeurs
  
  Impact:
  - 23 contradictions detectees grace a votre GPU
  - "Depute X: tweet pro-ecologie vs vote anti-climat"
  - "Senatrice Y: promesse aide sociale vs abstention budget"
  
  [Votre GPU a contribue a rendre la democratie plus transparente]
```

---

## Phase 9 — BDD locale + sync temps reel cr-sqlite (Sprint 9, ~3 jours)

### 9.1 Le concept: zero appel API, BDD locale synchronisee

```
CHAQUE USER:
  - Telecharge nexus.db complet au premier lancement (~500MB)
  - Frontend local (localhost:3002) → queries sur SQLite local = 0ms
  - Recoit les deltas en temps reel via WebSocket (<100ms)
  - ZERO appel API vers le serveur pour consulter les donnees
  
SERVEUR:
  - SEUL a ecrire dans la BDD (source de verite)
  - Broadcast chaque changement via WebSocket (cr-sqlite changesets)
  - Users appliquent les changesets en read-only
```

### 9.2 Pourquoi PAS multi-writer

**Multi-writer = n'importe quel user peut ecrire dans la BDD de tout le monde.**

Risques factuels:
- User injecte "Macron a dit X" → fausse contradiction propagee a tous
- User supprime des contradictions genantes → disparait partout
- User flood 1M de faux records → BDD corrompue chez tout le monde
- Bot farm injecte de la desinformation → outil de transparence devient outil de manipulation

**Pour un outil de transparence politique, une seule source de verite est NON NEGOCIABLE.**

Le modele securise:
```
ECRITURE: Serveur seul (scrape + analyse + validation + ecriture)
LECTURE:  Users (BDD locale synchronisee, queries 0ms)
GPU:      Users (Petals, resultats valides par le serveur AVANT ecriture)
```

Les users donnent leur GPU, pas leur confiance en ecriture.

### 9.3 cr-sqlite : sync temps reel

```
cr-sqlite est une extension SQLite qui genere des "changesets" binaires
pour chaque modification. Ces changesets sont idempotents et ordonnables.

Serveur INSERT INTO gov_positions (...) 
  → cr-sqlite genere changeset (~200 bytes)
  → WebSocket broadcast a tous les users connectes
  → Chaque user applique le changeset sur son SQLite local
  → Resultat: <100ms de delai, BDD identique partout
```

### 9.4 Les maths

```
Changeset moyen: ~200 bytes par INSERT
Scan complet (8000 positions): 8000 × 200 = 1.6 MB par user
50 users connectes: 50 × 1.6 MB = 80 MB broadcast total
Bande passante serveur: ~2.7 Mbps pendant le scan (trivial fibre)

En regime normal (hors scan):
  ~100 nouvelles positions/jour = 20 KB/jour par user
  ~50 articles presse/jour = 10 KB/jour par user
  Total: ~30 KB/jour de sync → invisible

BDD complete: ~500 MB (premier telechargement)
Delta par jour: ~30 KB
→ Apres le premier download, quasi zero bande passante
```

### 9.5 Implementation

```python
# === SERVEUR (broadcast changesets) ===

import sqlite3
import asyncio
import websockets
import json

db = sqlite3.connect("nexus.db")
db.load_extension("crsqlite")

# Activer CRDT sur les tables GOV
for table in [
    "gov_politicians", "gov_positions", "gov_contradictions",
    "gov_laws", "gov_press", "gov_social_posts", "gov_alerts",
    "gov_transcriptions", "gov_affairs", "gov_declarations",
]:
    db.execute(f"SELECT crsql_as_crr('{table}')")

# Track la version courante
last_version = 0
connected_users = set()

async def broadcast_changes():
    """Broadcast nouveaux changesets a tous les users."""
    global last_version
    while True:
        changes = db.execute(
            "SELECT * FROM crsql_changes WHERE db_version > ?",
            [last_version]
        ).fetchall()
        
        if changes:
            payload = json.dumps({"changes": changes})
            for ws in connected_users:
                await ws.send(payload)
            last_version = max(c[5] for c in changes)  # db_version column
        
        await asyncio.sleep(0.1)  # 100ms polling

async def handle_user(ws):
    """Nouveau user connecte."""
    connected_users.add(ws)
    try:
        # Envoyer la version courante pour que le user sache ou il en est
        await ws.send(json.dumps({"version": last_version}))
        # Garder la connexion ouverte
        async for msg in ws:
            pass  # Users en read-only, pas de messages entrants
    finally:
        connected_users.discard(ws)


# === CLIENT (recoit et applique) ===

import sqlite3
import asyncio
import websockets

local_db = sqlite3.connect("nexus_local.db")
local_db.load_extension("crsqlite")

async def sync_loop():
    async with websockets.connect("wss://nexusgov.fr/sync") as ws:
        async for message in ws:
            data = json.loads(message)
            if "changes" in data:
                for change in data["changes"]:
                    local_db.execute(
                        "INSERT INTO crsql_changes VALUES (?,?,?,?,?,?)",
                        change
                    )
                local_db.commit()
                # BDD locale mise a jour — le frontend voit les nouvelles donnees
```

### 9.6 Experience utilisateur

```
Premier lancement nexus-worker:
  1. "Telechargement de la BDD NEXUS GOV... (487 MB)"
  2. Progress bar → 30s sur fibre
  3. "BDD locale prete. Frontend: http://localhost:3002"
  4. "Connexion sync temps reel... OK"
  5. "GPU connecte au swarm Petals... OK"

Utilisation quotidienne:
  - Ouvre localhost:3002
  - Donnees instantanees (SQLite local, 0ms)
  - Nouvelles contradictions apparaissent en direct (WebSocket <100ms)
  - GPU travaille en arriere-plan (Petals)
  - Aucun appel reseau pour les queries
  
  Le seul trafic: ~30 KB/jour de sync + tokens Petals
```

### 9.7 Si le serveur tombe

```
Serveur down:
  → WebSocket deconnecte
  → Users gardent leur BDD locale intacte (derniere version)
  → Queries fonctionnent toujours (tout est local)
  → Petals continue de tourner (decentralise)
  → Juste pas de NOUVELLES donnees jusqu'au retour du serveur
  
Serveur revient:
  → WebSocket reconnecte automatiquement
  → cr-sqlite envoie tous les changesets manques
  → Users rattrapent le retard en quelques secondes
  → Aucune perte de donnees
```

---

## Budget et timeline

| Phase | Livrable | Effort | Prerequis |
|---|---|---|---|
| 1 | GPU Registry + Task Queue + API | 3 jours | Rien |
| 2 | Auto-scaling modele | 2 jours | Phase 1 |
| 3 | Client contributeur (pip install nexus-worker) | 2 jours | Phase 1 |
| 4 | Mode hybride Ollama local + distribue | 3 jours | Phase 3 |
| 5 | Dashboard public + gamification + leaderboard | 2 jours | Phase 1 |
| 6 | **Securite: Ed25519 + digest + logprob fingerprint + spot-check** | 3 jours | Phase 1 |
| 7 | **Petals: split 405B sur 50 GPUs fibre** | 3 jours | Phase 3 |
| 8 | Swarm public permanent + monitoring | 2 jours | Phase 7 |
| 9 | **BDD locale + cr-sqlite sync temps reel** | 3 jours | Phase 3 |
| **Total** | **Systeme complet** | **~23 jours** | |

Phases 1-3: MVP (taches distribuees, client CLI).
Phases 4-6: Ameliorations (hybrid, dashboard, securite).
Phases 7-8: Game-changer LLM (405B Petals).
Phase 9: Game-changer data (zero API, BDD locale, sync <100ms).

---

## Modeles par palier (reference)

| VRAM totale | Modele | Params | Qualite | Tokens/s (est.) |
|---|---|---|---|---|
| 8 GB | gemma-4-12b-q4 | 12B | Basique | 60 |
| 14 GB | gemma-4-26b-q4 | 26B (MoE 4B actifs) | Bonne | 45 |
| 40 GB | llama-3.1-70b-q4 | 70B | Tres bonne | 25 |
| 80 GB | qwen-2.5-110b-q4 | 110B | Excellente | 15 |
| 150 GB | llama-3.1-405b-q2 | 405B | Top tier | 8 |
| 300 GB | llama-3.1-405b | 405B (full) | Maximum | 5 |

---

## Scenarios de puissance

```
Setup actuel (FlowUP seul):
  1x RTX 5080 (16GB) → Gemma 26B q4 → 45 tok/s
  → Bonne analyse, contradictions basiques

5 contributeurs fibre:
  5x GPU mix (80GB) → Llama 70B q4 → 25 tok/s batch
  → Tres bonne analyse, jargon juridique

20 contributeurs fibre:
  20x GPU mix (280GB) → Llama 405B q2 → 40 tok/s batch
  → Excellente analyse, comparable Claude 3

50 contributeurs fibre:
  50x GPU mix (700GB) → Llama 405B full → 80 tok/s batch
  → Maximum possible, rivalise GPT-4
  → 1145 politiciens analyses en 12 minutes
  → Detection de contradictions subtiles, ironie, contexte

100 contributeurs fibre:
  100x GPU mix (1.4TB) → Llama 405B full + modeles specialises
  → Surplus de puissance → fine-tune de modeles specialises
  → Analyse en temps reel (pas juste batch)
  → Couverture EU 27 pays possible
```

## Frameworks distribues — comparaison technique

| Framework | Internet | Fault tolerance | Mix GPU | 405B teste | Status 2026 |
|---|---|---|---|---|---|
| **Petals** | **OUI** | **OUI (dual cache)** | CUDA only | **OUI (swarm actif)** | 10K stars, mature |
| GPUStack | OUI | Partiel | TOUT | Non | Production |
| llama.cpp RPC | Risque | NON | **TOUT** | 0.7 tok/s | Proof-of-concept |
| exo | LAN | NON | Apple only | NON | Immature NVIDIA |
| Parallax | OUI | ? | NVIDIA+Apple | Non | v0.1.2, tres tot |
| vLLM | Datacenter | OUI | Homogene | OUI | Enterprise |

**Choix NEXUS:**
- Phase 1-6: Task queue custom + Ollama local par node (simple, fiable)
- Phase 7-8: Petals pour le split 405B (le seul mature pour internet + fault tolerance)
- Fallback: GPUStack si besoin de mix NVIDIA+Apple dans le meme cluster

## Le pitch

> Chaque citoyen qui contribue son GPU rend l'IA politique plus puissante.
> 5 personnes = un modele 70B.
> 50 personnes = un modele 405B qui rivalise GPT-4.
> 100% gratuit, 100% francais, 100% citoyen.
> Votre carte graphique au service de la democratie.
> 
> Meme une vieille GTX 1060 peut aider.
> Les petits GPU font les taches legeres.
> Les gros GPU forment ensemble un cerveau geant.
> Plus on est nombreux, plus l'IA est puissante.
