# Research: Distributed GPU Task Queue for NEXUS GOV

**Domain:** Distributed volunteer GPU computing for LLM inference
**Researched:** 2026-04-09
**Overall confidence:** HIGH (patterns well-established, libraries mature)

---

## Executive Summary

Building a distributed GPU task queue for NEXUS GOV is a well-trodden problem space. The core pattern -- a central server maintaining a registry of volunteer GPU nodes, a priority task queue, and workers pulling tasks over persistent connections -- is validated by BOINC (20+ years), Celery (pull-based workers), and the recently published Hive framework (purpose-built for distributed Ollama inference).

The recommended approach is a **hybrid WebSocket + REST pull** architecture: WebSocket for real-time task push and heartbeats, REST endpoints as fallback for unreliable connections. This avoids the complexity of gRPC while matching NEXUS GOV's existing FastAPI stack. The existing VRAMScheduler and ReactiveWorker patterns in NEXUS core provide a strong foundation to extend.

The key risk is **result validation** -- volunteer nodes can return garbage, malicious output, or time out. BOINC solves this with majority voting (redundant computation). For LLM inference where outputs are non-deterministic, a lightweight validation approach (format checks + spot-check sampling at 5%) is more practical than full redundancy.

---

## 1. Task Queue Patterns

### 1.1 Pull-Based vs Push-Based

| Pattern | How It Works | Pros | Cons |
|---------|-------------|------|------|
| **Pull (worker polls)** | Worker requests next task when idle | Natural load balancing, workers self-pace, simple | Polling overhead, latency between task availability and pickup |
| **Push (server assigns)** | Server pushes task to idle worker | Lower latency, server controls assignment | Server must track worker state, risk of overloading slow workers |
| **Hybrid (recommended)** | WebSocket push with REST pull fallback | Best of both worlds | Slightly more complex |

**Recommendation: Hybrid push-pull.** Use WebSocket to push task notifications to idle workers. Worker confirms readiness and receives task details. If WebSocket disconnects, worker falls back to REST polling every 5-10 seconds.

**Confidence: HIGH** -- This is exactly how BOINC operates (client polls server, but with adaptive intervals), and how Celery workers operate (pull from broker queue). The Hive framework for distributed Ollama uses the same pattern.

### 1.2 Priority Queue Design

Based on research into Celery, Azure Priority Queue Pattern, and NEXUS's existing VRAMScheduler:

```
Priority Levels (lower = more urgent):
  1 = CRITICAL   -- Breaking change detection, urgent contradiction
  2 = HIGH       -- Active analysis pipeline tasks
  3 = NORMAL     -- Standard sentiment, summary, NER tasks
  4 = LOW        -- Batch processing, historical re-analysis
  5 = BACKGROUND -- Data enrichment, non-urgent embedding
```

**Key patterns to implement:**

1. **Priority with aging:** Tasks waiting too long get priority bumped (+1 per 5 minutes). Prevents starvation of low-priority tasks.
2. **Model affinity batching:** Same pattern as existing VRAMScheduler -- if a worker has model X loaded, prefer sending model X tasks to avoid expensive model swaps. This is already battle-tested in NEXUS core.
3. **Visibility timeout:** When a task is assigned, it becomes invisible to other workers for `timeout_seconds`. If the worker doesn't complete in time, the task returns to the queue. This prevents double-execution while handling dead workers.
4. **Back-pressure:** Reject new tasks when queue depth exceeds threshold (e.g., 1000). Return HTTP 503 to callers. Same pattern as VRAMScheduler's `_MAX_HEAVY_QUEUE = 50`.

**Confidence: HIGH** -- These are standard distributed queue patterns documented in Azure Architecture Center and implemented in every production task queue (Celery, SQS, Bull).

### 1.3 Task State Machine

```
pending -> assigned -> completed
                   \-> failed -> pending (retry, max 3)
                   \-> expired (timeout) -> pending (reassign)
pending -> cancelled (manual)
```

The state machine prevents double-execution. A background reaper task runs every 60 seconds to:
- Move `assigned` tasks past their timeout back to `pending`
- Mark workers that haven't heartbeated in 90 seconds as `offline`
- Reassign tasks from offline workers

**Confidence: HIGH** -- This is exactly how Celery/Redis and SQS handle task lifecycle. NEXUS's existing event_log table uses a similar `pending -> processed` flow.

### 1.4 Existing Frameworks Evaluated

| Framework | Relevance | Verdict |
|-----------|-----------|---------|
| **BOINC** | Gold standard for volunteer computing | Too heavyweight (C++, custom protocols). Useful for architectural patterns only. |
| **Celery** | Mature Python task queue | Requires Redis/RabbitMQ broker. Overkill for NEXUS GOV which only needs SQLite. Workers are co-located, not remote. |
| **Taskiq** | Modern async Python task queue | Async-native, FastAPI integration. But still broker-dependent (Redis/NATS). |
| **Ray Serve** | Distributed ML serving | Designed for cluster environments, not volunteer/home GPUs over internet. |
| **Hive** | Distributed Ollama inference | Closest match! But written in Java, and is a proxy (forwards Ollama API), not a task queue. |
| **Petals** | BitTorrent-style LLM inference | Model parallelism (splits layers across GPUs). Different problem -- NEXUS GOV needs task parallelism (each worker runs full model). |

**Recommendation: Build custom, inspired by BOINC pull model + Celery state machine + Hive's Ollama integration pattern.** The problem is specific enough (SQLite-backed, volunteer GPUs, Ollama-only) that no existing framework fits without adding unnecessary dependencies.

**Confidence: HIGH** -- Validated by examining all major frameworks. None fit the specific constraints (SQLite, no broker, volunteer GPUs, Ollama).

---

## 2. GPU Registry

### 2.1 GPU Detection on Worker Side

**Recommended library: `nvidia-ml-py`** (official NVIDIA Python bindings)

| Library | Status | Performance | Notes |
|---------|--------|-------------|-------|
| `nvidia-ml-py` | Active, NVIDIA-maintained | Direct NVML C API via ctypes | **Use this.** `pip install nvidia-ml-py` |
| `pynvml` | **Deprecated** | Same API | Redirects to nvidia-ml-py |
| `py3nvml` | Deprecated | Same API | Redirects to nvidia-ml-py |
| `GPUtil` | Maintained | Parses nvidia-smi (slower) | Fallback only if NVML unavailable |
| `subprocess nvidia-smi` | Always works | Slowest (spawns process) | Last resort fallback |

**Detection code pattern:**
```python
import pynvml  # pip install nvidia-ml-py (imports as pynvml)

def detect_gpu() -> dict:
    """Detect GPU info via NVML. Falls back to nvidia-smi parsing."""
    try:
        pynvml.nvmlInit()
        handle = pynvml.nvmlDeviceGetHandleByIndex(0)
        name = pynvml.nvmlDeviceGetName(handle)
        mem = pynvml.nvmlDeviceGetMemoryInfo(handle)
        return {
            "gpu_model": name,
            "vram_mb": mem.total // (1024 * 1024),
            "vram_free_mb": mem.free // (1024 * 1024),
            "driver_version": pynvml.nvmlSystemGetDriverVersion(),
        }
    except Exception:
        return _fallback_nvidia_smi()
    finally:
        try:
            pynvml.nvmlShutdown()
        except Exception:
            pass
```

**For Apple Silicon (Mac M-series):** No NVML. Detect via `sysctl hw.memsize` (unified memory) or `system_profiler SPDisplaysDataType`. Detect platform with `platform.system()`.

**Confidence: HIGH** -- nvidia-ml-py is NVIDIA's official library. The import-as-pynvml pattern is documented on PyPI.

### 2.2 Heartbeat / Keepalive Strategy

**Dual-layer heartbeat:**

1. **WebSocket ping/pong (transport layer):** The `websockets` library sends automatic pings every 20 seconds by default. If no pong within 20 seconds, connection is considered dead. This handles network-level failures.

2. **Application heartbeat (business layer):** Worker sends a JSON heartbeat every 30 seconds with:
   ```json
   {
     "type": "heartbeat",
     "gpu_status": "idle|busy",
     "vram_free_mb": 8192,
     "current_task_id": null,
     "tokens_per_sec": 45.2,
     "uptime_s": 3600
   }
   ```

**Server-side timeout logic:**
- No heartbeat for 90 seconds -> mark node `offline`
- No heartbeat for 300 seconds -> reassign all tasks from that node
- Reconnection resets the node to `idle` (not `offline`)

**Reconnection with exponential backoff + jitter:**
```python
# Worker-side reconnection
delay = min(30, 2 ** attempt) + random.uniform(0, 2)
# Cap at 30 seconds, add 0-2s jitter to prevent thundering herd
```

**Confidence: HIGH** -- `websockets` library v16.0 handles ping/pong automatically. The 20s/20s defaults are production-proven. Application-level heartbeats on top are standard in BOINC and Celery.

---

## 3. Communication: WebSocket vs Polling

### 3.1 Decision Matrix

| Factor | WebSocket | HTTP Polling | Hybrid (recommended) |
|--------|-----------|-------------|---------------------|
| **Latency** | ~0ms (push) | 5-10s (poll interval) | ~0ms normally, 5-10s fallback |
| **Server complexity** | Medium (connection manager) | Low | Medium |
| **Network reliability** | Fragile (NAT, proxies, firewalls) | Robust | Robust |
| **Bandwidth** | Minimal (no HTTP headers per message) | Higher (full HTTP request/response) | Minimal |
| **Scalability** | ~10K connections per uvicorn worker | Higher (stateless) | Good |
| **Volunteer-friendly** | Some corporate firewalls block WS | Works everywhere | Works everywhere |

### 3.2 Recommended Architecture: WebSocket Primary + REST Fallback

```
Worker connects:
  1. Try WebSocket to wss://nexusgov.fr/ws/compute
  2. If WS connection succeeds:
     - Receive task push notifications
     - Send heartbeats as JSON messages
     - Bidirectional: server can push task, worker can push results
  3. If WS fails (firewall, proxy):
     - Fall back to REST polling:
       GET /api/compute/task every 5-10 seconds
       POST /api/compute/heartbeat every 30 seconds
       POST /api/compute/result when done
  4. Worker auto-detects which mode works
```

**Why not pure WebSocket?**
- Some corporate/university networks block WebSocket upgrades
- NAT timeout can silently kill connections
- HTTP polling is more resilient and debuggable
- Volunteer environments are inherently unreliable

**Why not pure polling?**
- 5-10 second latency for task assignment is wasteful
- More bandwidth (HTTP headers on every poll)
- Can create thundering herd if many workers poll simultaneously

**FastAPI implementation note:** FastAPI supports WebSocket natively. Use `@app.websocket("/ws/compute")` endpoint. The existing SSE bridge in NEXUS shows the pattern already works. For a single uvicorn process (which is the NEXUS GOV deployment), no Redis pub/sub needed -- in-memory connection manager suffices.

**Confidence: HIGH** -- Hybrid approach is standard in production systems (Socket.IO, Ably, etc.). FastAPI WebSocket support is mature.

### 3.3 FastAPI WebSocket Connection Manager Pattern

```python
class ComputeConnectionManager:
    """Manages WebSocket connections to GPU worker nodes."""

    def __init__(self):
        self._connections: dict[str, WebSocket] = {}  # node_id -> ws
        self._node_status: dict[str, str] = {}         # node_id -> idle/busy

    async def connect(self, node_id: str, ws: WebSocket):
        await ws.accept()
        self._connections[node_id] = ws
        self._node_status[node_id] = "idle"

    async def disconnect(self, node_id: str):
        self._connections.pop(node_id, None)
        self._node_status.pop(node_id, None)

    async def push_task(self, node_id: str, task: dict):
        ws = self._connections.get(node_id)
        if ws:
            await ws.send_json({"type": "task", **task})
            self._node_status[node_id] = "busy"

    def get_idle_nodes(self) -> list[str]:
        return [nid for nid, s in self._node_status.items() if s == "idle"]
```

---

## 4. Security

### 4.1 API Key Generation and Storage

**Use `secrets.token_urlsafe(32)`** -- generates 32 bytes of randomness, URL-safe base64 encoded (43 characters). This is cryptographically secure and the Python standard library recommendation.

```python
import secrets
import hashlib

def generate_api_key() -> tuple[str, str]:
    """Generate API key and its hash for storage."""
    key = secrets.token_urlsafe(32)
    key_hash = hashlib.sha256(key.encode()).hexdigest()
    return key, key_hash

def verify_api_key(provided_key: str, stored_hash: str) -> bool:
    """Verify an API key against its stored hash."""
    return hashlib.sha256(provided_key.encode()).hexdigest() == stored_hash
```

**Key points:**
- Store only the SHA-256 hash in the database, never the plaintext key
- Show the plaintext key ONCE at registration, never again
- 32 bytes = 256 bits of entropy. Brute-force infeasible.

**Confidence: HIGH** -- This is the Python docs recommended approach. `secrets.token_urlsafe` uses `os.urandom()` under the hood.

### 4.2 IP Privacy

Hash IPs with a salt to prevent rainbow table attacks:
```python
import hashlib, os

_IP_SALT = os.environ.get("NEXUS_IP_SALT", secrets.token_hex(16))

def hash_ip(ip: str) -> str:
    return hashlib.sha256(f"{_IP_SALT}:{ip}".encode()).hexdigest()[:16]
```

Store only the hash. This lets you detect same-IP abuse without tracking contributors.

**Confidence: HIGH** -- Standard privacy pattern.

### 4.3 Rate Limiting

**Use `slowapi`** (not `fastapi-limiter`).

| Library | Status | Notes |
|---------|--------|-------|
| `slowapi` | Active, 2K+ stars | Built on Flask-Limiter, Starlette-native. **Use this.** |
| `fastapi-limiter` | Less maintained | Requires Redis. Overkill for SQLite-based NEXUS. |
| Custom middleware | N/A | Simple but reinvents the wheel |

**Configuration:**
```python
from slowapi import Limiter
from slowapi.util import get_remote_address

def get_api_key(request: Request) -> str:
    """Rate limit by API key, fallback to IP."""
    return request.headers.get("Authorization", get_remote_address(request))

limiter = Limiter(key_func=get_api_key)

@app.get("/api/compute/task")
@limiter.limit("100/minute")
async def get_task(request: Request):
    ...
```

`slowapi` uses in-memory storage by default (no Redis needed). Perfect for single-process NEXUS GOV.

**Confidence: HIGH** -- slowapi is the de facto standard for FastAPI rate limiting.

### 4.4 Result Validation

Three levels, matching the roadmap's Phase 6 vision but simplified for Phase 1:

1. **Format validation (Phase 1):** Check that the result is valid JSON, has expected fields, reasonable token count. Rejects garbage immediately.

2. **Spot-check sampling (Phase 1):** 5% of tasks are sent to 2 different workers. If results diverge significantly (cosine similarity of embeddings < 0.7), flag both for review. Cheap and effective.

3. **Reputation scoring (Phase 2+):** Track per-node error rate, divergence rate, speed consistency. Nodes with >10% anomaly rate get deprioritized, then banned.

**Confidence: HIGH** -- BOINC uses majority voting. The spot-check approach is a lightweight adaptation that works for non-deterministic LLM outputs.

---

## 5. Technology Stack Recommendation

### 5.1 Server-Side (extends existing NEXUS GOV FastAPI)

| Component | Technology | Version | Why |
|-----------|-----------|---------|-----|
| API framework | FastAPI | existing | Already the stack. Add WebSocket + REST endpoints. |
| WebSocket | FastAPI native + `websockets` | 16.x | Built-in, no extra dependency for FastAPI |
| Database | SQLite (existing) | existing | `gpu_nodes` + `gpu_tasks` tables. WAL mode already configured. |
| Rate limiting | `slowapi` | 0.1.9+ | In-memory, no Redis needed. Per-API-key limiting. |
| Task scheduling | Custom (asyncio) | N/A | Reaper task for timeouts. Extends existing event loop. |
| Auth | `secrets` + `hashlib` | stdlib | API key generation + SHA-256 hashing. |

### 5.2 Worker-Side (new `nexus-worker` package)

| Component | Technology | Version | Why |
|-----------|-----------|---------|-----|
| CLI framework | `click` or `typer` | typer 0.9+ | Type-safe CLI. `typer` is built on click with type hints. |
| TUI dashboard | `rich` | 13.x | Already used in NEXUS. Live panels, tables, progress bars. |
| GPU detection | `nvidia-ml-py` | 12.x | Official NVIDIA bindings. Falls back to subprocess nvidia-smi. |
| LLM runtime | Ollama (local) | 0.18+ | Worker runs Ollama locally. Prompts sent, results returned. |
| WebSocket client | `websockets` | 16.x | Async, production-proven, automatic ping/pong. |
| HTTP client | `httpx` | 0.27+ | Async HTTP for REST fallback. Already in NEXUS deps. |
| Config storage | JSON file | N/A | `~/.nexus-worker/config.json` for API key, server URL, preferences. |

### 5.3 New Dependencies

**Server-side:**
```bash
pip install slowapi  # Rate limiting (no Redis needed)
# websockets already available via FastAPI's [standard] extras
```

**Worker-side (new package):**
```bash
pip install typer rich nvidia-ml-py websockets httpx ollama
```

**Confidence: HIGH** -- All libraries are mature, well-maintained, and compatible with the existing stack.

---

## 6. Architecture Patterns

### 6.1 Component Boundaries

```
NEXUS GOV Server (FastAPI)
 |
 +-- nexus/gov/compute/
 |    +-- __init__.py
 |    +-- registry.py        # GPURegistry: CRUD for gpu_nodes table
 |    +-- queue.py            # TaskQueue: priority queue with aging + affinity
 |    +-- dispatcher.py       # TaskDispatcher: matches tasks to idle nodes
 |    +-- validator.py        # ResultValidator: format check + spot-check
 |    +-- ws_manager.py       # ComputeConnectionManager: WebSocket connections
 |    +-- api.py              # FastAPI router: /api/compute/* + /ws/compute
 |    +-- reaper.py           # Background: timeout expired tasks, offline nodes
 |    +-- models.py           # Pydantic models for API request/response
 |
 +-- nexus/gov/db.py          # Add gpu_nodes + gpu_tasks table DDL
```

```
nexus-worker (separate package, installable via pip)
 |
 +-- nexus_worker/
 |    +-- __init__.py
 |    +-- cli.py              # typer CLI: register, start, stats, stop
 |    +-- gpu.py              # GPU detection (nvidia-ml-py + fallbacks)
 |    +-- connection.py       # WebSocket + REST hybrid connection
 |    +-- runner.py           # Ollama task execution loop
 |    +-- dashboard.py        # Rich TUI live display
 |    +-- config.py           # ~/.nexus-worker/config.json management
```

### 6.2 Data Flow

```
1. Worker starts:
   cli.py -> gpu.py (detect GPU) -> connection.py (register with server)
   Server: api.py -> registry.py (store node) -> return api_key

2. Task assignment:
   Server: workers produce tasks -> queue.py (enqueue)
   Server: dispatcher.py (find idle node) -> ws_manager.py (push task)
   Worker: connection.py (receive task) -> runner.py (ollama.generate)
   Worker: connection.py (send result) -> api.py -> validator.py -> store

3. Heartbeat:
   Worker: connection.py (every 30s) -> ws_manager.py -> registry.py (update)
   Server: reaper.py (every 60s) -> registry.py (mark offline if stale)
```

### 6.3 Integration with Existing NEXUS GOV Workers

The 31 existing workers (scraping, analysis, sentiment, etc.) currently call Ollama locally via LLMRouter. The distributed system intercepts at the LLMRouter level:

```python
# In LLMRouter (modified):
async def generate(self, task_type, prompt, model):
    if self._distributed_mode and self._has_remote_nodes():
        # Enqueue task for remote GPU workers
        task_id = await self._task_queue.enqueue(task_type, prompt, model, priority)
        result = await self._task_queue.wait_for_result(task_id, timeout=300)
        return result
    else:
        # Local Ollama (existing path)
        async with self._scheduler.gpu_access(priority, model, label):
            return await self._client.generate(model=model, prompt=prompt)
```

This is non-invasive: all 31 workers continue working unchanged. The LLMRouter decides whether to run locally or distribute.

---

## 7. Pitfalls and Risks

### 7.1 Critical Pitfalls

**Pitfall: Prompt data leakage**
- Risk: Sending sensitive data (personal info from investigations) to volunteer GPUs
- Prevention: NEXUS GOV only processes PUBLIC political data (votes, speeches, press). The roadmap already states "le prompt ne contient JAMAIS de donnees personnelles." Enforce this with a sanitization step before enqueueing.
- Detection: Regex check for PII patterns (email, phone, SSN) in prompts before sending.

**Pitfall: Model inconsistency across workers**
- Risk: Different workers running different model versions produce inconsistent results.
- Prevention: The ModelSelector assigns ONE model at a time. Tasks include the exact model tag. Worker verifies it has the right model before executing.
- Detection: Include model hash in result metadata. Server validates.

**Pitfall: Thundering herd on reconnection**
- Risk: If server restarts, all workers reconnect simultaneously, overloading the server.
- Prevention: Exponential backoff with jitter on worker side. `delay = min(30, 2^attempt) + random(0, 2)`.

### 7.2 Moderate Pitfalls

**Pitfall: SQLite contention under load**
- Risk: Many concurrent task status updates causing WAL checkpointing delays.
- Prevention: Batch heartbeat updates. Use `INSERT OR REPLACE` for heartbeats instead of `SELECT + UPDATE`. SQLite WAL mode (already configured in NEXUS) handles this well up to ~100 concurrent writers.
- Migration path: If >50 nodes, consider PostgreSQL (already supported by NEXUS GOV's db_postgres.py).

**Pitfall: Ollama model pull timeout**
- Risk: Worker told to pull a 40GB model, download takes hours on slow connection.
- Prevention: Track pull status separately. Worker reports "pulling" state. Server doesn't assign tasks until pull completes. Allow workers to opt-out of model changes.

**Pitfall: WebSocket state management complexity**
- Risk: Connection manager growing complex with reconnection, partial messages, state sync.
- Prevention: Keep WebSocket layer thin. Use it only for push notifications and heartbeats. All actual task data flows through REST POST/GET. WebSocket is a notification channel, not a data channel.

### 7.3 Minor Pitfalls

**Pitfall: Clock skew between server and workers**
- Risk: Timeout calculations wrong if clocks differ.
- Prevention: Use server-side timestamps for all timeout logic. Workers report durations, not absolute times.

**Pitfall: VRAM detection on Mac/AMD**
- Risk: nvidia-ml-py only works with NVIDIA GPUs.
- Prevention: Fallback chain: nvidia-ml-py -> subprocess nvidia-smi -> system_profiler (macOS) -> /sys/class/drm (AMD/Linux). Report "unknown" if all fail.

---

## 8. Implications for Roadmap

### Suggested Phase Structure

**Phase 1: Infrastructure (3 days)**
Core server tables, API endpoints, basic pull-based task assignment.
- gpu_nodes + gpu_tasks tables in SQLite
- REST API: register, heartbeat, task pull, result submit, stats
- API key auth + IP hashing
- slowapi rate limiting
- Task reaper background job
- This phase is pure server-side, no worker package yet. Test with curl/httpx.

**Phase 2: Worker Package (2-3 days)**
The `nexus-worker` CLI and connection logic.
- GPU detection (nvidia-ml-py + fallbacks)
- Ollama integration (pull model, generate, report)
- WebSocket connection with REST fallback
- Rich TUI dashboard
- Config file management (~/.nexus-worker/)
- Packaging with pyproject.toml

**Phase 3: WebSocket + Task Dispatcher (2 days)**
Real-time task push and intelligent assignment.
- WebSocket endpoint on server
- ComputeConnectionManager
- TaskDispatcher: match tasks to idle nodes with model affinity
- Priority queue with aging
- Integration with existing LLMRouter (distributed mode flag)

**Phase 4: Validation + Leaderboard (2 days)**
Trust and gamification.
- Result format validation
- Spot-check sampling (5% dual-send)
- Reputation scoring per node
- Public stats API + leaderboard
- Badge system

**Phase ordering rationale:**
1. Phase 1 first because REST API is the foundation everything else builds on. Can be tested independently.
2. Phase 2 second because the worker package is the user-facing deliverable. Needs the API to exist.
3. Phase 3 third because WebSocket is an optimization over polling. The system works (slower) without it.
4. Phase 4 last because validation and gamification are enhancements, not core functionality.

**Research flags:**
- Phase 3: May need deeper research on WebSocket scaling if >50 concurrent nodes.
- Phase 4: exo integration (model splitting across GPUs) is a separate, complex research topic for later.

---

## 9. Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Task queue patterns | HIGH | BOINC, Celery, Azure patterns well-documented. Pull-based with priority is standard. |
| GPU detection | HIGH | nvidia-ml-py is NVIDIA-official. Fallback chain covers edge cases. |
| WebSocket vs polling | HIGH | Hybrid approach validated by multiple production systems. FastAPI native support. |
| Security (auth) | HIGH | secrets.token_urlsafe + SHA-256 hashing is Python standard library best practice. |
| Rate limiting | HIGH | slowapi is de facto standard for FastAPI, no Redis needed. |
| Result validation | MEDIUM | Spot-check approach is pragmatic but not cryptographically proven. Sufficient for Phase 1. |
| Existing frameworks | HIGH | Evaluated BOINC, Celery, Taskiq, Ray, Hive, Petals. None fit; custom is the right call. |
| Integration with NEXUS | HIGH | LLMRouter interception point is clean. Existing VRAMScheduler patterns reusable. |

---

## 10. Gaps to Address

- **exo integration (Phase 4 of roadmap):** Model parallelism across GPUs is a separate research topic. Not needed for Phase 1-3.
- **PostgreSQL migration path:** If the network grows beyond ~50 nodes, SQLite will need migration. NEXUS GOV already has `db_postgres.py` ready.
- **Ollama model compatibility testing:** Need to verify that the same prompt produces comparable results across different Ollama versions and quantizations.
- **Cross-platform worker testing:** nvidia-ml-py on Linux, macOS unified memory detection, Windows GPU detection all need hands-on validation.
- **Network security:** HTTPS/WSS for production. Self-signed certs for development. Not researched deeply (standard DevOps).

---

## Sources

### Task Queue Patterns
- [Azure Priority Queue Pattern](https://learn.microsoft.com/en-us/azure/architecture/patterns/priority-queue) -- Priority queue design
- [Distributed Task Queue (GeeksforGeeks)](https://www.geeksforgeeks.org/system-design/distributed-task-queue-distributed-systems/) -- Task queue system design
- [Celery Routing Tasks](https://docs.celeryq.dev/en/latest/userguide/routing.html) -- Pull model, prefetch, priority routing
- [Celery Workers Guide](https://docs.celeryq.dev/en/stable/userguide/workers.html) -- Worker lifecycle
- [BOINC: A Platform for Volunteer Computing](https://boinc.berkeley.edu/boinc_a_platform_for_volunteer_computing.pdf) -- Pull-based volunteer computing architecture

### GPU Detection
- [nvidia-ml-py on PyPI](https://pypi.org/project/nvidia-ml-py/) -- Official NVIDIA Python bindings
- [pynvml on PyPI](https://pypi.org/project/pynvml/) -- Deprecated, redirects to nvidia-ml-py

### Communication
- [websockets Keepalive Docs](https://websockets.readthedocs.io/en/stable/topics/keepalive.html) -- Ping/pong configuration
- [FastAPI WebSockets](https://fastapi.tiangolo.com/advanced/websockets/) -- Native WebSocket support
- [FastAPI 45K Concurrent WebSocket](https://medium.com/@ar.aldhafeeri11/part-1-fastapi-45k-concurrent-websocket-on-single-digitalocean-droplet-1e4fce4c5a64) -- Scaling characteristics

### Security
- [Python secrets module](https://docs.python.org/3/library/secrets.html) -- Cryptographic token generation
- [slowapi GitHub](https://github.com/laurentS/slowapi) -- FastAPI rate limiting
- [SlowAPI Documentation](https://slowapi.readthedocs.io/) -- Configuration and usage

### Distributed Inference
- [Hive: Distributed Ollama Framework](https://www.sciencedirect.com/science/article/pii/S2352711025001505) -- HiveCore/HiveNode architecture
- [HiveCore GitHub](https://github.com/VakeDomen/HiveCore) -- Java implementation reference
- [Petals: BitTorrent-style LLM](https://github.com/bigscience-workshop/petals) -- Model parallelism approach
- [Taskiq: Async Task Queue](https://github.com/taskiq-python/taskiq) -- Modern Python async task queue

### Reliability
- [WebSocket Reconnection Guide](https://websocket.org/guides/reconnection/) -- State sync and recovery
- [BOINC Result Validation](https://link.springer.com/article/10.1007/s10723-019-09497-9) -- Majority voting in volunteer computing
