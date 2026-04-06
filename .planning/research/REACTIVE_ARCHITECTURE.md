# Reactive Architecture Research: NEXUS Event-Driven Migration

**Researched:** 2026-04-06
**Confidence:** HIGH (stdlib patterns) / MEDIUM (Ollama-specific behavior)

---

## 1. Event Bus Patterns in Python asyncio

### 1a. asyncio.Queue-Based Pub/Sub -- RECOMMENDED

**Confidence:** HIGH (stdlib, battle-tested)

The simplest pattern that works. An `asyncio.Queue` acts as the event channel. Publishers `put()` events, a dispatcher loop `get()`s them and routes to registered handlers.

**Strengths:**
- Zero dependencies -- Python stdlib since 3.4
- Native async -- no thread bridging, no sync/async mismatch
- Backpressure via `maxsize` -- blocks producers when queue is full
- `PriorityQueue` subclass for priority-based dispatch
- Well-understood by any Python developer

**Weaknesses:**
- In-memory only -- events lost on crash
- Single-process only -- no cross-process communication
- No built-in persistence, replay, or dead letter queue

**For NEXUS:** This is the right foundation. NEXUS is single-process, events are transient (database state is the source of truth), and the existing OODA loop serves as a crash-recovery mechanism.

**Implementation pattern (from multiple verified sources):**
```python
class InvestigationBus:
    def __init__(self, case_id: str):
        self._case_id = case_id
        self._queue: asyncio.Queue[Event] = asyncio.Queue(maxsize=500)
        self._handlers: dict[EventType, list[Callable]] = {}
        self._running = False

    def subscribe(self, event_type: EventType, handler: Callable):
        self._handlers.setdefault(event_type, []).append(handler)

    async def emit(self, event: Event):
        await self._queue.put(event)

    async def run(self):
        self._running = True
        while self._running:
            event = await self._queue.get()
            handlers = self._handlers.get(event.type, [])
            for handler in handlers:
                asyncio.create_task(handler(event))  # Non-blocking dispatch
            self._queue.task_done()
```

### 1b. Python blinker Library (Signal/Slot)

**Confidence:** HIGH (Pallets ecosystem, Flask uses it)

Blinker provides named signals with `connect()` / `send()`. Since v1.8+, supports `send_async()` for coroutine receivers.

**Strengths:**
- Simple API: `signal('evidence-ingested').connect(handler)`
- Async support via `send_async()`
- Part of Pallets ecosystem (Flask, Jinja2) -- well maintained

**Weaknesses:**
- No queue -- signals are dispatched immediately (synchronous fan-out)
- No priority, no ordering, no backpressure
- No persistence or replay
- Signal names are strings -- no type safety

**For NEXUS:** Too simple. Blinker is great for "notify X when Y happens" but has no queue, no priority, no batching. NEXUS needs queued dispatch for VRAM serialization.

### 1c. aio-pika / RabbitMQ

**Confidence:** HIGH (well-established)

Full AMQP message broker. Persistent queues, routing, acknowledgments, dead letter exchanges.

**For NEXUS:** Overkill. Adds an external service (RabbitMQ) for a single-process Python application. Would only make sense if NEXUS became a distributed multi-service architecture.

### 1d. Redis Pub/Sub via aioredis

**Confidence:** HIGH (well-established)

Redis provides pub/sub channels with O(1) publish and subscribe. `aioredis` (now part of `redis-py` async) is the standard async client.

**For NEXUS:** Unnecessary. Redis pub/sub adds an external dependency (Docker container) for something asyncio.Queue does natively in-process. Redis would only be useful if multiple NEXUS instances needed to share events.

### 1e. Custom EventBus -- RECOMMENDED APPROACH

**Confidence:** HIGH (verified via SpiderFoot, bubus, and multiple tutorials)

Build a custom `InvestigationBus` class that combines:
- `asyncio.Queue` for event dispatch
- Type-based handler registration (dict of EventType -> handlers)
- Per-case isolation (one bus per case_id)
- Immutable typed events (frozen dataclass)
- Integration with existing audit trail

This is approximately 200 lines of code. The complexity is in the modules, not the bus.

### Comparison Matrix

| Criterion | asyncio.Queue | blinker | RabbitMQ | Redis Pub/Sub | Custom Bus |
|-----------|--------------|---------|----------|---------------|------------|
| Dependencies | stdlib | pip | Docker + pip | Docker + pip | stdlib |
| Async native | Yes | Since 1.8 | Via aio-pika | Via redis-py | Yes |
| Queue/buffer | Yes | No | Yes | No | Yes |
| Priority | PriorityQueue | No | Yes (priority queue) | No | PriorityQueue |
| Persistence | No | No | Yes | No | Optional (audit trail) |
| Type safety | With dataclass | No (strings) | No | No | With dataclass |
| Complexity to add | Zero | pip install | Docker compose + config | Docker compose + config | ~200 lines |
| Cross-process | No | No | Yes | Yes | No |
| NEXUS fit | Good base | Too simple | Overkill | Overkill | Best fit |

**Decision:** Custom EventBus built on asyncio.Queue. No external dependencies.

---

## 2. Reactive Patterns for Data Pipelines

### 2a. RxPY (ReactiveX for Python)

**Confidence:** MEDIUM (library maintained but niche in Python)

RxPY v4.x provides 120+ operators for composing asynchronous data streams. Latest release November 2025, repo active as of March 2026.

**Strengths:**
- Rich operator library (map, filter, debounce, throttle, merge, combine_latest)
- Well-suited for complex data transformation pipelines
- Good for "react to change" patterns

**Weaknesses:**
- Steep learning curve -- functional-reactive paradigm is foreign to imperative Python
- 41K lines of existing imperative code would need significant refactoring
- Community is smaller in Python than JavaScript (RxJS)
- Debugging observable chains is notoriously difficult
- Not a natural fit for "module A produces events for module B" -- it is designed for stream transformations

**For NEXUS:** Not recommended. The investigation pipeline is not a stream transformation problem. It is a "when X happens, do Y" problem. RxPY solves a different class of problem.

### 2b. Faust (faust-streaming)

**Confidence:** MEDIUM (community fork active, original Robinhood repo deprecated)

Stream processing library inspired by Kafka Streams. Requires Kafka as a backend. Active community fork at `faust-streaming/faust`, updated March 2026.

**For NEXUS:** Not recommended. Requires Kafka (another Docker container, JVM dependency). NEXUS processes events at ~50/cycle, not millions/second. Faust solves a scale problem NEXUS does not have.

### 2c. Temporal.io

**Confidence:** HIGH (well-funded, production-ready)

Workflow orchestration platform. Durable execution, automatic retries, saga patterns.

**For NEXUS:** Not recommended. Temporal requires a server (Go/Java), adds massive operational complexity. Designed for distributed microservices, not a single Python process. The investigation workflow is not complex enough to justify Temporal's overhead.

### 2d. Dramatiq / Celery

**Confidence:** HIGH (well-established)

Task queues with broker backends (Redis, RabbitMQ). Dramatiq is lighter and faster than Celery. Both require an external broker.

**For NEXUS:** Not recommended as the primary pattern. These are task queues (fire-and-forget work), not event buses (react-to-change). However, the VRAMScheduler pattern is conceptually similar to a single-worker task queue -- and we can build that with asyncio.PriorityQueue.

### 2e. Plain asyncio Primitives -- RECOMMENDED

**Confidence:** HIGH (stdlib, well-documented)

`asyncio.Queue`, `asyncio.PriorityQueue`, `asyncio.Event`, `asyncio.Condition`, `asyncio.Semaphore`.

**For NEXUS:** This is the right level of abstraction. The codebase already uses `asyncio.Lock` for VRAM serialization. Evolving to PriorityQueue is natural and requires no new dependencies.

---

## 3. VRAM-Aware Task Scheduling

### 3a. Current State in NEXUS

The `LLMRouter` uses a single `asyncio.Lock` (`_heavy_lock`) to serialize all heavy model calls. This ensures only one heavy model occupies VRAM at a time. It works but has no priority -- a batch RAPTOR summary blocks an urgent contradiction detection.

### 3b. ML Pipeline GPU Serialization Patterns

**Confidence:** MEDIUM (patterns documented in talks/articles, not in formal libraries)

The standard pattern from the PyCon ZA talk "Juggling GPU Tasks with asyncio" and the Medium article "Fast GPU based PyTorch model serving":

1. Create an `asyncio.Future` for each GPU request
2. Place the request + future into an `asyncio.Queue`
3. A single worker coroutine pulls from the queue, executes on GPU, sets the future result
4. Callers `await` the future

For batching:
1. Worker waits for the queue to accumulate N items or T seconds (whichever comes first)
2. Groups items by model
3. Processes the largest model-group first (to minimize swaps)
4. Sets results on all futures in the batch

### 3c. Ollama keep_alive Parameter

**Confidence:** HIGH (verified via official Ollama docs and FAQ)

Key facts:
- Default `keep_alive` is 5 minutes -- model unloads from VRAM after 5 min idle
- Per-request: `"keep_alive": "30m"` or `"keep_alive": -1` (keep forever)
- Environment: `OLLAMA_KEEP_ALIVE=30m` sets default for all models
- If VRAM is insufficient for a new model, Ollama queues the request until an idle model is unloaded
- Multiple models can be loaded simultaneously IF they fit in VRAM

**Strategy for NEXUS (RTX 5080, 16GB VRAM):**
- Set `OLLAMA_KEEP_ALIVE=10m` globally (longer than default 5m)
- For the primary model (nexus 26B, ~14GB Q4_K_S), set `keep_alive: "30m"` per-request
- For secondary models (deepseek-r1 14B), set `keep_alive: "5m"` per-request
- For light models (gemma4:e4b, nomic-embed-text), keep_alive default is fine -- they coexist
- Monitor via `GET /api/ps` to see what is currently loaded

### 3d. Recommended VRAMScheduler Design

```
Priority levels:
  0 = URGENT    (contradiction detected, alert triggered)
  5 = HIGH      (hypothesis scoring after new evidence)
  10 = NORMAL   (standard analysis, entity extraction)
  15 = LOW      (periodic re-evaluation, RAPTOR summaries)
  20 = BATCH    (full case re-analysis, report generation)

Batching strategy:
  - Accumulate tasks for 2 seconds OR until 5 tasks queued (whichever first)
  - Sort accumulated tasks by model name (group same-model tasks)
  - Within same model, sort by priority
  - Process all tasks for model A, then model B, etc.
  - This minimizes Ollama model swaps
```

---

## 4. Event Sourcing for Investigation Systems

### 4a. Event Sourcing Fit

**Confidence:** HIGH (well-established pattern, good Python library exists)

Event sourcing stores all changes as immutable, append-only events. The current state is derived by replaying events. This is a natural fit for NEXUS because:

1. **NEXUS already has an append-only audit trail** (SQLite hash chain + JSONL + git)
2. **Investigation is inherently temporal** -- "what did we know at time T?"
3. **Reproducibility matters** -- "why did the system reach this conclusion?"
4. **Rollback is valuable** -- "the OSINT source was unreliable, undo its effects"

### 4b. How It Maps to NEXUS

The existing `AuditService` already logs every action as a hash-chained entry. To evolve this into event sourcing:

1. **Events are already being recorded** -- the audit log entries are essentially events
2. **Add event type taxonomy** -- map audit `action` strings to typed EventType enum
3. **Add replay capability** -- function that reads events from audit table and re-emits them on the bus
4. **State reconstruction** -- given events up to time T, reconstruct what the system "knew"

**NOT recommended:**
- Full CQRS (separate read/write models) -- single process, adds complexity for no benefit
- External event store (EventStore DB, Kafka) -- SQLite is sufficient
- The `eventsourcing` Python library -- it imposes its own aggregate/domain model that does not match NEXUS's existing data model

### 4c. Practical Implementation

```python
# Evolve existing audit table to serve as event store
# Current audit table already has: case_id, actor, action, summary, details, created_at, hash_chain

# Add: event_type (enum), processing_state (pending/processed/failed), replay_id

# The audit log IS the event store. No separate system needed.
# Replay = SELECT * FROM audit WHERE case_id=? ORDER BY created_at, re-emit each as Event
```

---

## 5. Similar Open-Source Projects

### 5a. SpiderFoot -- Gold Standard for Investigation Event Architecture

**Confidence:** HIGH (open source, verified architecture via DeepWiki)

SpiderFoot is a 207-module OSINT automation tool built on an event-driven architecture in Python. Each module:
- Inherits from `SpiderFootPlugin`
- Implements `watchedEvents()` -- declares which event types it consumes
- Implements `producedEvents()` -- declares which event types it produces
- Implements `handleEvent(event)` -- processes incoming events

The core `SpiderFootScanner` maintains an event queue. When a module produces an event, it is placed on the queue. The scanner dispatches events to all modules that declared interest. Thread pool constrains parallel execution.

**Key lesson for NEXUS:** This is exactly the pattern NEXUS should follow. The module declarations make the system self-documenting. Dependency graph can be auto-generated from declarations.

### 5b. TheHive / Cortex -- Alert-Driven Investigation

**Confidence:** HIGH (well-documented open source)

TheHive is a Security Incident Response Platform. Cortex is its analysis engine.
- TheHive receives alerts from feeders (event sources)
- Alerts can be escalated to cases (triage)
- Observables (IP, email, hash) are analyzed by Cortex analyzers
- Cortex analyzers are Python scripts that process one observable and return structured results
- Responders can take actions (block IP, send email) based on analysis results

**Key lesson for NEXUS:** The analyzer/responder pattern maps to NEXUS modules. Each analyzer is independent, processes one input, returns structured output. No shared state between analyzers.

### 5c. Maltego -- Transform-Based Graph Expansion

**Confidence:** MEDIUM (commercial product, architecture partially documented)

Maltego uses "transforms" to expand a graph. Each transform:
- Takes one entity as input
- Queries a data source
- Returns zero or more new entities
- New entities appear connected to the input entity

Users chain transforms manually or automatically. The graph grows through entity-to-entity transformations.

**Key lesson for NEXUS:** The entity -> transform -> entity pattern maps to NEXUS's entity extraction -> OSINT enrichment -> new entity discovery flow. Maltego proves this pattern scales to complex investigations.

### 5d. Pattern Synthesis

All three tools share the same core pattern:

```
Input Event/Entity -> Module/Analyzer/Transform -> Output Events/Entities
     ^                                                    |
     |                                                    |
     +----------------------------------------------------+
                    (feedback loop)
```

The differences are in orchestration:
- SpiderFoot: automatic, event queue, all modules fire
- TheHive/Cortex: manual triage, analyst triggers analyzers
- Maltego: manual or semi-automatic, user controls transform execution

NEXUS should be like SpiderFoot (fully automatic) with TheHive-style priority (urgent findings get processed first).

---

## 6. Practical Recommendation for NEXUS

### The Simplest Approach That Works

Given:
- Python 3.13, asyncio, FastAPI
- Single GPU (RTX 5080, 16GB)
- SQLite + Neo4j + ChromaDB (already running)
- 41K lines of working code
- Solo developer (FlowUP)

**Recommended architecture: Custom In-Process EventBus + SpiderFoot Module Pattern + PriorityQueue VRAM Scheduler**

### Implementation Plan (incremental, no big-bang)

**Step 1: Define Events (~50 lines)**
```python
# nexus/core/events.py
@dataclass(frozen=True)
class Event:
    type: EventType
    case_id: str
    data: dict
    source: str
    event_id: str
    parent_id: str
    timestamp: str
```

**Step 2: Build InvestigationBus (~150 lines)**
```python
# nexus/core/event_bus.py
class InvestigationBus:
    # asyncio.Queue, handler registry, dispatch loop
    # Circuit breaker, depth tracking, dedup
```

**Step 3: Build VRAMScheduler (~100 lines)**
```python
# nexus/core/vram_scheduler.py
class VRAMScheduler:
    # asyncio.PriorityQueue, model batching
    # Replaces asyncio.Lock in LLMRouter
```

**Step 4: Define ModuleBase (~50 lines)**
```python
# nexus/core/module_base.py
class ModuleBase:
    watches: set[EventType]
    produces: set[EventType]
    async def handle(self, event: Event) -> None
    async def emit(self, ...) -> None
```

**Step 5: Migrate EvidenceProcessor first**
- Easiest module: clear input (MONITORING_RESULT_FOUND), clear output (EVIDENCE_INGESTED)
- Keep OODA fallback: if event bus misses something, OODA sweep catches it
- Test with benchmark: run Kulik case, compare results with/without event bus

**Step 6: Migrate remaining modules one by one**
- Order: Evidence -> Hypothesis -> Contradiction -> OSINT -> Geo -> Image -> Forensics -> Timeline -> Suspect -> Report -> Neo4j
- Each migration: 1 day max, with tests
- OODA loop runs in parallel throughout, as a safety net

**Step 7: Retire OODA loop (only after 2+ weeks clean operation)**
- Replace 30-minute sleep with periodic "consistency check" every 10 cycles
- The consistency check verifies that no evidence is stuck in `processing_state = pending`

### What NOT to Build

- No external message broker
- No CQRS separation
- No event schema versioning
- No saga pattern
- No distributed event store
- No dead letter queue (log errors instead)
- No RxPY operators

### Migration Risk: LOW

The event bus runs alongside the existing system. If it breaks, the OODA loop continues working. Each module can be toggled between event-driven and OODA-driven via a feature flag. The migration is fully reversible at every step.

---

## Sources

### PRIMARY (Context7 / Official Docs)
- [Python asyncio.Queue](https://docs.python.org/3/library/asyncio-queue.html) -- stdlib documentation
- [Python asyncio.PriorityQueue](https://docs.python.org/3/library/asyncio-queue.html#priority-queue) -- stdlib
- [Ollama FAQ: keep_alive](https://docs.ollama.com/faq) -- model loading behavior
- [Ollama VRAM management](https://markaicode.com/ollama-keep-alive-memory-management/) -- concurrent model rules

### VERIFIED (Multiple Sources)
- [SpiderFoot architecture](https://deepwiki.com/smicallef/spiderfoot) -- event-driven OSINT tool
- [SpiderFoot GitHub](https://github.com/smicallef/spiderfoot) -- 207 event-driven modules
- [TheHive/Cortex](https://docs.strangebee.com/cortex/) -- investigation event model
- [bubus event bus](https://github.com/browser-use/bubus) -- Pydantic events, WAL, SQLite middleware
- [blinker](https://blinker.readthedocs.io/) -- signal/slot for Python
- [RxPY](https://github.com/ReactiveX/RxPY) -- ReactiveX for Python
- [faust-streaming](https://github.com/faust-streaming/faust) -- stream processing fork
- [eventsourcing library](https://eventsourcing.readthedocs.io/) -- Python event sourcing

### RESEARCH ARTICLES (WebSearch)
- [Event bus with asyncio](https://oneuptime.com/blog/post/2026-01-25-event-bus-asyncio-python/view) -- implementation guide
- [Event-driven Python systems](https://oneuptime.com/blog/post/2026-02-02-python-event-driven-systems/view) -- patterns
- [GPU task scheduling with asyncio](https://speakerdeck.com/pyconza/juggling-gpu-tasks-with-asyncio-by-bruce-merry) -- PyCon talk
- [GPU model serving](https://medium.com/@ngoodger_7766/fast-gpu-based-pytorch-model-serving-in-100-lines-of-python-9ad3ebd0a1d9) -- batching pattern
- [Local LLM concurrency](https://mljourney.com/how-local-llm-apps-handle-concurrency-and-scaling/) -- single GPU patterns
- [Event sourcing with SQLite](https://www.sqliteforum.com/p/event-sourcing-with-sqlite) -- append-only design
- [Dramatiq vs Celery comparison](https://devproportal.com/languages/python/python-background-tasks-celery-rq-dramatiq-comparison-2025/) -- task queue evaluation
- [Event bus error handling](https://dev.to/kuba_szw/how-i-fixed-my-event-bus-before-it-could-lose-money-546i) -- production lessons
