# Process architecture — broker / executor split

**Ecrit** : Sprint 28 Phase C (2026-04-25)
**Status** : **design doc** — aucune ligne implementee ce sprint.
L'implementation est planifiee Sprint 29 Phase D2.
**Prerequis** : Sprint 16 loopback hardening (bearer + UDS/Named
Pipe + peer creds) + Sprint 28 Phase A watermark wiring.

---

## 1. Introduction et motivation

Le daemon shell (`nexus-shell-daemon`) cumule aujourd'hui deux
responsabilites dans un seul processus :

1. **Orchestration reseau** : gestion de la keypair Ed25519,
   souscription gossip, routage des requetes shell, authentification
   bearer, persistence de l'etat, curator pipeline, browse
   aggregator, blob-serve iframe renderer.

2. **Execution compute** : le worker (`nexus-worker`) est deja un
   binaire separe, mais le daemon proxifie les taches au
   coordinator qui les dispatche au worker. Le worker charge les
   modeles Ollama/llama.cpp, accede au GPU, et execute les taches.

Le probleme : si le worker crash (OOM GPU, segfault llama.cpp,
timeout Ollama), le daemon continue de fonctionner. C'est deja
bon. Mais le daemon lui-meme fait tourner le blob-serve (rendu
iframe de contenu untrusted) dans le meme processus que la
keypair Ed25519. Un exploit dans le parsing zip ou le rendu
HTTP pourrait atteindre la keypair.

**Objectif du split** : separer le daemon en deux processus
distincts pour obtenir :

- **Fault isolation** : un crash dans un composant ne tue pas
  l'autre. Le broker survit aux crashes executor.
- **Privilege reduction** : l'executor n'a pas acces a la
  keypair Ed25519 ni aux tokens d'authentification master.
- **Crash containment** : un executor OOM ou en segfault est
  re-spawne sans interruption du service reseau.
- **Preparation VM** : le split processus est une etape
  intermediaire avant la VM invisible
  ([`RUNTIME_ISOLATION.md`](RUNTIME_ISOLATION.md) §3) — le
  broker migre dans la VM, l'executor reste cote host avec GPU.

**Prior art OSS** : ce pattern est la norme dans les systemes
de compute distribue matures :

- **BOINC** (1999-2026, 800k+ volunteers) : architecture
  manager → client → work unit avec IPC shared-memory. Le
  client (orchestration) et les work units (compute) sont des
  processus separes. 20+ ans de production stable.
- **Golem / Yagna** (Rust) : le provider utilise un exe-unit
  qui spawne un runtime separe (`ya-runtime-sdk`). Niveaux
  d'isolation escaladables : process → container → Light VM
  (QEMU).
- **Ollama** (Go, 2023-2026) : le serveur principal spawne des
  runner subprocesses par backend (llamarunner, ollamarunner,
  MLX runner). Communication via HTTP sur port dynamique.
  `/health` polling pour readiness. Crash runner = respawn
  automatique sans perte du serveur.

---

## 2. Architecture cible

```
┌─────────────────────────────────────────────────┐
│                    HOST OS                       │
│                                                  │
│  ┌──────────────────────┐  IPC   ┌────────────┐ │
│  │      BROKER          │◄─────►│  EXECUTOR   │ │
│  │  (nexus-shell-daemon │  UDS/  │  (nexus-    │ │
│  │   refactored)        │  NP    │  executor)  │ │
│  │                      │        │             │ │
│  │  • Keypair Ed25519   │        │  • GPU      │ │
│  │  • Bearer auth       │        │  • Ollama   │ │
│  │  • Gossip subscribe  │        │  • llama.cpp│ │
│  │  • Curator pipeline  │        │  • Sampling │ │
│  │  • Browse aggregator │        │  • Watermark│ │
│  │  • State persistence │        │             │ │
│  │  • Routing table     │        │  NO keypair │ │
│  │  • Consent state     │        │  NO master  │ │
│  │                      │        │  token      │ │
│  └──────────┬───────────┘        └──────┬──────┘ │
│             │ HTTP loopback              │ GPU    │
│  ┌──────────▼───────────┐        ┌──────▼──────┐ │
│  │    COORDINATOR       │        │   NVIDIA    │ │
│  │  (FastAPI proxy)     │        │   Runtime   │ │
│  └──────────────────────┘        └─────────────┘ │
└─────────────────────────────────────────────────┘
```

### 2.1 Broker (`nexus-shell-daemon` refactore)

Long-lived, surface minimale. Responsabilites :

- **Identite** : keypair Ed25519, `daemon.key`, `auth_token`
- **Reseau P2P** : iroh node, gossip subscribe, blobs client,
  DHT pkarr, relay TLS
- **Authentification** : bearer token validation, UDS/Named Pipe
  peer creds, Host/Origin allowlist
- **Orchestration** : curator runtime, browse aggregator, state
  persistence, consent state
- **Routage** : dispatch des taches vers executors via IPC,
  health monitoring executors
- **Blob-serve** : rendu iframe (a terme migre dans un 3eme
  process ou dans un executor dedie — scope S30+)

Le broker n'accede **jamais** au GPU, ne charge **aucun** modele,
et ne touche **aucun** sampling/watermark.

### 2.2 Executor (`nexus-executor` nouveau binaire)

Short-lived ou pooled, crate `crates/nexus-executor/`. Responsabilites :

- **Runtime LLM** : Ollama client, llama.cpp backend, model
  loading/unloading
- **GPU** : NVML monitoring, VRAM allocation, CUDA/Metal compute
- **Sampling** : watermark injection (PRF bias), output token
  accumulation, grammar constraints (llguidance)
- **Task execution** : recoit une requete task via IPC, execute,
  retourne le resultat via IPC

L'executor n'a **pas** acces a :
- La keypair Ed25519 (`daemon.key`)
- Le bearer token master (`auth_token`)
- Les gossip subscriptions
- Les curator lists
- L'etat de consentement GPU

---

## 3. IPC boundary

### 3.1 Canal

| Plateforme | Canal | Chemin |
|---|---|---|
| Linux / macOS | Unix domain socket | `~/.sbfb/run/executor-{pid}.sock` |
| Windows | Named Pipe | `\\.\pipe\sbfb-executor-{pid}` |

Un socket/pipe distinct par executor instance. Le broker cree le
socket avant de spawner l'executor et passe le chemin en argument
CLI (`--ipc-path`). L'executor se connecte au socket au demarrage.

### 3.2 Protocole : JSON-RPC 2.0

**Choix retenu** : JSON-RPC 2.0 ([spec](https://www.jsonrpc.org/specification))

**Analyse comparative** :

| Critere | JSON-RPC 2.0 | gRPC / protobuf |
|---|---|---|
| Serialization latence (payload < 1 KB) | ~2-5 µs (`serde_json`) | ~1-2 µs (protobuf) |
| Codegen | Aucune | `.proto` + `tonic` build.rs |
| Debuggabilite | Texte lisible, `jq` compatible | Binaire, `grpcurl` requis |
| Deps supplementaires | 0 (serde_json deja en workspace) | tonic + prost + protobuf compiler |
| Streaming | Pas natif (notifications JSON-RPC) | Natif (bidirectionnel) |
| Taille binaire | ~0 KB delta | ~500 KB (tonic + prost) |

**Decision** : JSON-RPC 2.0. Sur un canal UDS/Named Pipe local,
la difference de latence est negligeable (< 5 µs). Le bottleneck
est l'inference model (100 ms+ par token). La simplicite zero-
codegen et la debuggabilite texte valent plus que les 3 µs gagnes.

Le streaming n'est pas necessaire : le broker envoie une requete
task, l'executor retourne un resultat complet. Le streaming
token-par-token est gere entre l'executor et le coordinator
(SSE existant), pas entre broker et executor.

### 3.3 Methodes JSON-RPC

```json
// Broker → Executor
{"jsonrpc": "2.0", "method": "task.execute", "params": {
    "task_id": "uuid",
    "model": "llama3.1:8b",
    "prompt": "...",
    "watermark_config": {"enabled": true, "delta": 2.0, "window_size": 4},
    "grammar": null,
    "max_tokens": 1024,
    "task_token": "ephemeral-per-task-hex"
}, "id": 1}

// Executor → Broker (result)
{"jsonrpc": "2.0", "result": {
    "task_id": "uuid",
    "output": "...",
    "output_token_ids": [123, 456, 789],
    "model_used": "llama3.1:8b",
    "duration_ms": 2340,
    "gpu_vram_peak_mb": 4200
}, "id": 1}

// Executor → Broker (notification, pas de id)
{"jsonrpc": "2.0", "method": "health.report", "params": {
    "status": "idle",
    "gpu_util_pct": 0,
    "vram_used_mb": 1200,
    "model_loaded": "llama3.1:8b",
    "uptime_s": 3600
}}

// Broker → Executor
{"jsonrpc": "2.0", "method": "executor.shutdown", "params": {
    "reason": "idle_timeout",
    "grace_period_ms": 30000
}, "id": 2}
```

**`task_token`** : token ephemere genere par le broker pour
chaque task. L'executor l'utilise pour s'authentifier aupres du
coordinator si besoin (model pull, status update). Ce n'est
**pas** le master `auth_token` — sa portee est limitee a une
seule task et expire avec elle.

---

## 4. Executor lifecycle

### 4.1 Pool mode (recommande : production)

```
Broker startup
  └─► Spawn N executors (N = nombre de models caches)
       └─► Executor se connecte au socket IPC
       └─► Executor charge le model (cold-start)
       └─► Executor envoie health.report "ready"
       └─► Idle timeout: 5 min sans task → executor s'auto-shutdown
            └─► Broker log + respawn si model demande a nouveau
```

- N = 1 pour un setup single-GPU typique
- Les executors sont pre-chauffes : le model est charge avant la
  premiere task
- Cold-start amorti sur la duree de vie du pool
- Un executor idle pendant 5 minutes se shutdown proprement, le
  broker le respawne a la prochaine demande

### 4.2 Spawn-on-demand mode (dev / test)

```
Task arrive au broker
  └─► Broker spawn executor avec --model llama3.1:8b
       └─► Executor charge model (cold-start)
       └─► Executor execute task
       └─► Executor retourne result
       └─► Grace period 30s : si pas de nouvelle task → shutdown
```

- Plus simple, moins de ressources idle
- Cold-start a chaque task si le modele n'est pas en cache Ollama
- Adapte aux tests automatises (pas de executor qui traine)

### 4.3 Cold-start budget

**Cible** : < 5 secondes entre spawn executor et premier token
genere, sur RTX 5080 avec Ollama + model 7B deja en cache.

Decomposition :

| Etape | Budget |
|---|---|
| Spawn process + connect IPC | < 100 ms |
| Ollama model load (deja cache) | ~1-3 s |
| Premier token inference | ~100-500 ms |
| **Total** | **< 5 s** |

**Prerequis S29** : benchmark reel sur RTX 5080 + Ollama 7B avant
implementation. Le budget < 5 s est une cible, pas une garantie —
si le benchmark montre > 5 s, investiguer Ollama warm-start API ou
pre-load model avant spawn executor.

---

## 5. State ownership

| State | Owner | Acces |
|---|---|---|
| Keypair Ed25519 (`daemon.key`) | Broker exclusif | Executor : **aucun** |
| Bearer token master (`auth_token`) | Broker exclusif | Executor : **aucun** |
| Gossip subscriptions | Broker exclusif | Executor : **aucun** |
| Curator lists | Broker exclusif | Executor : **aucun** |
| Consent state (GPU 4 niveaux) | Broker exclusif | Executor : read via IPC (broker envoie le niveau) |
| Routing table (peers) | Broker exclusif | Executor : **aucun** |
| Model runtime | Executor exclusif | Broker : status via IPC health.report |
| GPU memory (VRAM) | Executor exclusif | Broker : stats via IPC health.report |
| Sampling state (watermark) | Executor exclusif | Broker : config via task.execute params |
| Task request / response | Shared via IPC | Broker envoie request, executor retourne result |
| Watermark config | Broker → Executor | Envoye dans task.execute params, pas d'etat persistent executor |
| Health status | Executor → Broker | Notifications periodiques (heartbeat 10 s) |

**Invariant** : l'executor est **stateless entre les tasks**
(sauf le model charge en memoire). Toute la persistence — keypair,
gossip, state, consent — vit dans le broker.

---

## 6. Fault isolation

### 6.1 Crash executor

```
Executor crash (OOM / segfault / panic)
  └─► Broker detecte: IPC socket ferme + process exit
       └─► Log event SecurityEvent::ExecutorCrash
       └─► Re-spawn avec backoff exponentiel :
            1s → 2s → 4s → 8s → 16s → 30s (cap)
       └─► Apres 5 crashes en 5 min : alerte user via
            coordinator /api/status "executor unstable"
       └─► Tasks en cours au moment du crash :
            retournees en erreur au coordinator,
            re-dispatch possible vers un autre worker
```

Le broker continue de repondre aux requetes shell (browse,
curators, health) pendant les crashes executor. L'experience
utilisateur n'est pas interrompue.

### 6.2 Crash broker

```
Broker crash (improbable, surface reduite)
  └─► Executors detectent: IPC heartbeat manquant
       └─► Apres 60 s sans heartbeat broker → executor self-exit
            (pas d'orphaned process indefini)
       └─► Launcher detecte broker exit → restart broker
       └─► Broker respawne ses executors au redemarrage
```

### 6.3 OOM isolation

| Plateforme | Mecanisme | Effet |
|---|---|---|
| Linux | cgroup v2 memory.max (si systemd-run) | OOM killer cible l'executor, broker survit |
| macOS | Pas de cgroup, Jetsam gere par le kernel | Executor tue en premier (plus de VRAM) |
| Windows | Job Objects memory limit | Executor termine, broker intact |

**Linux** : si le broker spawne l'executor via `systemd-run
--scope -p MemoryMax=<budget>`, le noyau confine l'OOM killer
au scope de l'executor. Le broker n'est pas affecte.

**Note** : cgroup / Job Objects sont des ameliorations optionnelles.
Le split processus seul (sans cgroup) donne deja l'isolation
principale — le kernel tue le processus qui alloue, pas un
processus tiers.

---

## 7. Security implications

### 7.1 Privilege reduction

| Surface | Avant split (monolithe) | Apres split |
|---|---|---|
| Exploit blob-serve parsing | → acces keypair Ed25519 | → acces keypair (blob-serve reste broker S28) |
| Exploit LLM runtime (Ollama/llama.cpp) | → acces keypair (worker separe, OK) | → acces keypair (inchange, worker deja separe) |
| Exploit executor IPC | N/A | → pas de keypair (executor n'y a pas acces) |
| Malware user-mode lit keypair | → possible (meme user) | → possible (meme user, resolu par VM S30+) |

**Gain net S29** : le nouveau `nexus-executor` n'a pas acces a la
keypair. C'est un gain incremental — le worker etait deja separe.
Le vrai gain est la preparation pour la VM
([`RUNTIME_ISOLATION.md`](RUNTIME_ISOLATION.md)) ou le broker
migre dans la VM avec la keypair, et l'executor reste cote host.

### 7.2 Token ephemere per-task

L'executor recoit un `task_token` genere par le broker pour chaque
task :

- **Portee** : une seule task, expire a la completion ou timeout
- **Droits** : model pull + status update aupres du coordinator
- **Pas de keypair** : le task_token est un HMAC-SHA256 derive
  du master token + task_id + timestamp. L'executor ne peut pas
  reconstruire le master token a partir du task_token.
- **Stockage** : en memoire executor uniquement, jamais persiste

### 7.3 Named Pipe DACL (Windows)

Le pattern Sprint 16 Phase B (`named_pipe_server.rs`) s'applique
au pipe broker ↔ executor :

- SDDL DACL restreint au SID du user courant
- Pas de `GENERIC_ALL` — permissions minimales (read/write)
- Le pipe est cree par le broker, l'executor se connecte en
  client

**Note S29** : si `agents_sudo D3` (Windows RPC) est livre, les
Named Pipes broker ↔ executor pourraient migrer vers Windows RPC
avec SID caller authentifie automatiquement. Decision D3 a
evaluer au kickoff S29.

### 7.4 Threat model mapping

| Threat (THREAT_MODEL.md) | Impact du split |
|---|---|
| T0 (script kiddie) | Inchange — bearer + UDS suffisent |
| T1 (activist under surveillance) | Gain : exploit executor ne compromet pas identity |
| T2 (organized crime) | Gain marginal : keypair isolee du compute path |
| T3 (corporate espionage) | Idem T2 |
| T4 (state dragnet) | Gain avec VM S30+ (keypair dans guest FS isole) |
| T5 (targeted state actor) | Exige VM + TEE, split seul insuffisant |

---

## 8. Migration path

### Phase 1 : Sprint 29 Phase D2 — broker / executor split

- Refactor `nexus-shell-daemon` : extraire les responsabilites
  compute routing dans le module IPC broker
- Nouveau crate `crates/nexus-executor/` (binaire)
- IPC UDS/Named Pipe + JSON-RPC 2.0
- Pool mode (N=1 par defaut) + spawn-on-demand mode (tests)
- Benchmark cold-start RTX 5080 + Ollama 7B : < 5 s

### Phase 2 : Sprint 29 Phase C4 — task-scoped sandbox

- Chaque task s'execute dans un executor dedie (ou un executor
  pool isole) avec des permissions restreintes a la task
- Iframe renderer (blob-serve) migre potentiellement vers un
  executor dedie (sandboxed, pas d'acces keypair ni GPU)
- CSP per-task dans le broker pour le blob-serve sandbox

### Phase 3 : Sprint 30+ — VM wrapper

- Le broker migre dans la VM (WSL2 / Virtualization.framework /
  systemd-nspawn) avec la keypair
- L'executor reste cote host avec GPU passthrough
- IPC cross-VM : port-forward UDS → TCP localhost (transparent
  pour le code JSON-RPC)
- Cf. [`RUNTIME_ISOLATION.md`](RUNTIME_ISOLATION.md) pour le
  detail technologique VM par plateforme

### Sequencing

```
S28 (maintenant)  : design doc (ce fichier)
                     │
S29 Phase D2      : broker/executor split implementation
                     │
S29 Phase C4      : task-scoped sandbox (depend D2)
                     │
S30+              : VM invisible (broker dans VM, executor host)
                     │
S30 Phase TEE     : attestation hardware H100 (Gate 4 prep)
```

---

## 9. Open questions

### Q1. Cold-start Ollama 7B sur RTX 5080

Besoin : benchmark reel avant implementation S29 Phase D2.

Variables :
- Ollama model deja en cache VRAM vs cold-load depuis disk
- RTX 5080 16 GB VRAM vs models > 8 GB
- Ollama API `keep_alive` warm vs full restart

**Action** : benchmark dedie pre-S29 kickoff. Si > 5 s,
evaluer Ollama warm-start API (`keep_alive: "5m"`) pour
pre-charger le model dans le pool executor.

### Q2. cgroup isolation Windows

Windows n'a pas de cgroups. Alternative : **Job Objects**
(Win32 API `CreateJobObject` + `SetInformationJobObject` avec
`JOB_OBJECT_LIMIT_PROCESS_MEMORY`).

- Avantage : natif Windows, pas de dep externe
- Limite : granularite moins fine que cgroups v2
- Le crate `windows-rs` expose `CreateJobObjectW` +
  `AssignProcessToJobObject`
- Implementation : S29 Phase D2 (conditionnel D3 co-landing)

### Q3. Model cache partage entre executors

Scenario : pool de 2+ executors chargeant le meme model. Le model
est-il charge 2x en VRAM ?

- Avec Ollama : oui, chaque instance Ollama charge sa copie.
  Pas de shared VRAM entre processes.
- Mitigation : pool N=1 par defaut (single-GPU setup typique).
  Multi-executor = multi-GPU setup ou models differents.
- Alternative future : shared memory mapping pour les weights
  (read-only mmap). Hors scope pre-v1.0.

### Q4. Blob-serve : broker ou executor dedie ?

Actuellement blob-serve (rendu iframe) est dans le broker. C'est
une surface d'attaque (parsing zip, HTTP serving de contenu
untrusted) dans le meme processus que la keypair.

Options :
- **A.** Garder dans le broker (statu quo S28) — simple, pas de
  latence IPC supplementaire pour le rendu iframe
- **B.** Migrer dans un executor dedie "renderer" (S30+) — pas
  de GPU, pas de keypair, sandboxe

**Recommandation** : Option A court terme (S29), Option B long
terme (S30+) quand la VM isole le broker de toute facon.

---

## 10. Pointeurs

- [`RUNTIME_ISOLATION.md`](RUNTIME_ISOLATION.md) — roadmap VM
  invisible (couche au-dessus du split processus)
- [`THREAT_MODEL.md`](THREAT_MODEL.md) §5.7 — risque residuel
  keypair user-mode
- [`HARDENING_ROADMAP.md`](HARDENING_ROADMAP.md) §3 S28 ligne
  D2 — tracking du split broker/executor
- [`LOOPBACK_ENDPOINTS_TRUST_TIERS.md`](LOOPBACK_ENDPOINTS_TRUST_TIERS.md)
  — trust tiers AUTO/CONFIRM_PROMPT/BIOMETRIC_GATE
- [`CAPABILITY_TOGGLES.md`](CAPABILITY_TOGGLES.md) — capabilities
  gate-off-by-default (interagissent avec consent state broker)
- `crates/nexus-shell-daemon-core/` — code actuel du daemon
  (futur broker)
- `crates/nexus-worker-core/` — code actuel du worker (reference
  pour le pattern headless-first)

---

## 11. Revue et evolution

- **v1 (Sprint 28 Phase C, 2026-04-25)** : version initiale.
  Design doc, pas d'implementation.
- Sprint 29 kickoff evaluera ce design et l'adaptera si le
  benchmark cold-start (Q1) ou le co-landing D3 (Q2) changent
  les parametres.
- Chaque phase S29 qui implemente ce design met a jour cette
  doc : section §8 migration path marque "LIVRE Sprint 29
  Phase X" + commit hash.
