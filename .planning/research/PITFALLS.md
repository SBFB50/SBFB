# Domain Pitfalls: Reactive Event-Driven Migration

**Domain:** Migrating a sequential investigation system to event-driven
**Researched:** 2026-04-06

## Critical Pitfalls

Mistakes that cause rewrites or major system failures.

### Pitfall 1: Event Storm Cascade

**What goes wrong:** One evidence ingest triggers entity extraction, which triggers OSINT for each entity, which discovers new data, which triggers more entity extraction, which triggers more OSINT -- exponential growth.
**Why it happens:** In a sequential system, you process one thing at a time. In an event-driven system, each output can trigger N handlers, each of which produces more events.
**Consequences:** CPU/VRAM saturation, thousands of queued events, Ollama OOM, system becomes unresponsive. Potentially infinite loops if OSINT results feed back into evidence ingestion.
**Prevention:**
- Circuit breaker: max events per type per cycle (start with 50)
- Depth tracking: events carry `depth` counter, reject events deeper than 5
- Dedup: hash-based deduplication on event payloads within a cycle window
- Rate limiting: OSINT modules already have `auto_recon_rate_limit`, enforce it
**Detection:** Monitor queue depth. If >100 events queued, something is looping. Log `parent_event_id` chain to trace cascades.

### Pitfall 2: VRAM Deadlock Under Priority Inversion

**What goes wrong:** A low-priority batch job (RAPTOR summary, nexus 26B) holds the GPU. A high-priority task (contradiction detection on new evidence) waits. Meanwhile, the batch job itself is waiting for a result from the contradiction detector (which needs GPU access). Deadlock.
**Why it happens:** Priority queues prevent starvation but not circular dependencies. The current `asyncio.Lock` is simple FIFO -- it cannot deadlock because there is only one lock. A priority system introduces ordering that can create circular waits.
**Consequences:** System hangs forever. Investigation stops. Requires manual restart.
**Prevention:**
- Never let a GPU-holding task await another GPU task. Design modules so GPU work is atomic (call LLM, get result, release GPU, THEN process result and emit events).
- Timeout on GPU acquisition: if a task cannot get GPU in 5 minutes, log error and skip.
- No re-entrant GPU access: a module currently using GPU cannot submit another GPU task.
**Detection:** Watchdog timer on GPU queue. If no task completes in 10 minutes, log full queue state and force-release.

### Pitfall 3: Big-Bang Migration Breaks Everything

**What goes wrong:** Trying to migrate all 21 modules to event-driven at once. The new system has bugs. The old system is removed. Nothing works.
**Why it happens:** Enthusiasm to "do it right" + underestimating the complexity of 41K lines of working code.
**Consequences:** Days of debugging, lost investigation progress, user (FlowUP) loses trust in the system.
**Prevention:**
- Strangler Fig pattern: run event bus alongside OODA loop. Migrate one module at a time.
- Each migrated module has a feature flag: `use_event_bus_for_X: bool = True`.
- OODA loop remains as a periodic "sweep" that catches anything the event bus missed.
- Only remove OODA after 2+ weeks of clean event bus operation.
**Detection:** Compare event bus results vs OODA sweep results. If OODA catches things the bus missed, the migration is not complete.

### Pitfall 4: Lost Events on Crash

**What goes wrong:** Events are in-memory (asyncio.Queue). Process crashes. Events are lost. Evidence was partially processed. On restart, the system does not know what was already done.
**Why it happens:** In-memory queues are fast but volatile. The current OODA loop does not have this problem because it re-scans everything each cycle.
**Consequences:** Missing evidence ingestion, duplicate processing, inconsistent state between SQLite/Neo4j/ChromaDB.
**Prevention:**
- Keep the OODA sweep as a periodic consistency check (every 10 cycles instead of every cycle).
- Track processing state in SQLite: each evidence has a `processing_state` column (pending, ingesting, ingested, analyzed, etc.).
- On startup, scan for `processing_state = ingesting` and re-process those items.
- Events themselves do not need persistence -- the database state is the source of truth.
**Detection:** On startup, count items where `processing_state` is not a terminal state. If >0, log warning and re-queue.

## Moderate Pitfalls

### Pitfall 5: Handler Ordering Assumptions

**What goes wrong:** Module A assumes it runs before Module B because that was the order in the OODA loop. In the event bus, handlers for the same event type run concurrently. Module B finishes first. Module A reads stale data.
**Prevention:** Modules must be independent for the same event type. If Module A depends on Module B's output, Module A should watch Module B's output event, not the same input event.

### Pitfall 6: Database Connection Per Handler Anti-Pattern

**What goes wrong:** Each handler opens its own `get_db()` connection. With 10 handlers firing for one event, that is 10 concurrent SQLite connections. WAL mode handles concurrent reads but only one writer at a time.
**Prevention:** 
- Group write operations: handlers that write to SQLite should use a shared write queue or a single-writer pattern.
- Read connections are fine concurrent (WAL mode).
- Neo4j and ChromaDB connections are already shared singletons -- keep it that way.

### Pitfall 7: Ollama Model Swap Thrashing

**What goes wrong:** Event bus processes tasks in event order, not model order. Task 1 needs nexus 26B, task 2 needs deepseek-r1 14B, task 3 needs nexus 26B. Ollama loads/unloads/loads the same model.
**Prevention:**
- The VRAMScheduler should batch tasks by model. Accumulate a small buffer (3-5 tasks or 2-second window), then sort by model before processing.
- Set `OLLAMA_KEEP_ALIVE=10m` to keep models loaded between calls (default is 5 minutes).
- Use Ollama's per-request `keep_alive` parameter: set `-1` for the primary model (nexus 26B), `5m` for others.
- Monitor model load/unload via Ollama `/api/ps` endpoint.

### Pitfall 8: Testing Event-Driven Code Is Harder

**What goes wrong:** Unit tests for the sequential OODA loop are straightforward: call function, check output. Event-driven tests need to: emit event, wait for handlers, check side effects across multiple modules.
**Prevention:**
- Each module is testable in isolation: call `handle(event)` directly, mock the bus.
- Integration tests: create a test bus, register modules, emit events, collect output events, assert.
- The bus itself is simple enough to test directly (put event, assert dispatched).

## Minor Pitfalls

### Pitfall 9: Event Payload Bloat

**What goes wrong:** Events carry full evidence text (10KB+) instead of just IDs. Queue memory grows, serialization slows.
**Prevention:** Events carry IDs. Handlers fetch data from the database. Events are notifications, not data carriers.

### Pitfall 10: Over-Granular Event Types

**What goes wrong:** Creating 50+ event types for every possible state change. Wiring becomes complex, debugging is harder.
**Prevention:** Start with ~15 event types (the ones listed in ARCHITECTURE.md). Add more only when a module genuinely needs to distinguish between subtypes.

### Pitfall 11: Forgetting to Unregister Handlers on Case Stop

**What goes wrong:** Investigation for case_id stops, but handlers remain registered on the bus. Next event for that case_id triggers zombie handlers.
**Prevention:** Per-case bus instances. When investigation stops, destroy the entire bus. No handler cleanup needed.

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| EventBus Core | Pitfall 2 (deadlock) | Atomic GPU work, no re-entrant GPU access |
| Module Migration | Pitfall 3 (big-bang), Pitfall 5 (ordering) | Strangler Fig, one module at a time |
| VRAM Optimization | Pitfall 7 (model thrashing) | Batch by model, Ollama keep_alive |
| Event Persistence | Pitfall 4 (lost events) | Database state as source of truth, not events |
| First Production Run | Pitfall 1 (event storm) | Circuit breakers from day 1, depth tracking |
| Testing | Pitfall 8 (testing difficulty) | Isolated module tests + integration test bus |

## Sources

- [SpiderFoot event flow](https://deepwiki.com/smicallef/spiderfoot) -- how 207 modules handle cascading without storms
- [bubus loop prevention](https://github.com/browser-use/bubus) -- event_path tracking to prevent cycles
- [Ollama FAQ on keep_alive](https://docs.ollama.com/faq) -- model loading/unloading behavior and VRAM management
- [Ollama VRAM management](https://markaicode.com/ollama-keep-alive-memory-management/) -- concurrent model loading rules
- [Event bus error handling](https://dev.to/kuba_szw/how-i-fixed-my-event-bus-before-it-could-lose-money-546i) -- production lessons from event bus bugs
- [GPU task scheduling](https://speakerdeck.com/pyconza/juggling-gpu-tasks-with-asyncio-by-bruce-merry) -- asyncio GPU serialization patterns
- [Local LLM concurrency](https://mljourney.com/how-local-llm-apps-handle-concurrency-and-scaling/) -- how local LLM apps handle single-GPU serialization
