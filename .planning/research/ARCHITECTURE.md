# Architecture Patterns: Reactive Event-Driven NEXUS

**Domain:** Event-driven investigation system
**Researched:** 2026-04-06

## Recommended Architecture

### High-Level Flow

```
                          +------------------+
                          |   InvestigationBus  |  (one per case_id)
                          |   asyncio.Queue     |
                          +--------+---------+
                                   |
                    +--------------+--------------+
                    |              |              |
              +-----v-----+ +-----v-----+ +-----v-----+
              | Dispatcher | | Dispatcher | | Dispatcher |  (concurrent workers)
              +-----+-----+ +-----+-----+ +-----+-----+
                    |              |              |
         +----------+---------+   |   +----------+---------+
         |                    |   |   |                    |
   +-----v------+  +-----v---v-+ |  +-----v------+  +----v-------+
   | Evidence    |  | Hypothesis| |  | Contradiction|  | Suspect    |
   | Processor   |  | Engine    | |  | Detector     |  | Scorer     |
   +-----+------+  +-----------+ |  +--------------+  +------------+
         |                        |
   +-----v------+          +-----v------+
   | GPU Queue   |          | GPU Queue   |
   | (Priority)  |          | (Priority)  |
   +-----+------+          +-----+------+
         |                        |
   +-----v--------------------------v-----+
   |         VRAM Scheduler               |
   |   asyncio.PriorityQueue             |
   |   (one heavy model at a time)        |
   +--------------------------------------+
```

### Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| **InvestigationBus** | Per-case event queue, dispatches to registered handlers | All modules via typed events |
| **BusManager** | Creates/destroys per-case buses, lifecycle management | InvestigationManager, FastAPI lifespan |
| **EventDispatcher** | Pulls events from bus, routes to matching handlers | InvestigationBus, all modules |
| **VRAMScheduler** | PriorityQueue for GPU-bound tasks, one heavy model at a time | LLMRouter, all GPU-using modules |
| **ModuleBase** | Abstract base class for all reactive modules | InvestigationBus (subscribe/publish) |
| **EvidenceModule** | Watches: MONITORING_RESULT_FOUND. Produces: EVIDENCE_INGESTED, ENTITIES_EXTRACTED | Bus, VRAMScheduler |
| **HypothesisModule** | Watches: EVIDENCE_INGESTED, ENTITIES_EXTRACTED. Produces: HYPOTHESIS_SCORED | Bus, VRAMScheduler |
| **ContradictionModule** | Watches: EVIDENCE_INGESTED. Produces: CONTRADICTION_FOUND | Bus, VRAMScheduler |
| **SuspectModule** | Watches: HYPOTHESIS_SCORED, EVIDENCE_INGESTED. Produces: SUSPECT_SCORED | Bus, VRAMScheduler |
| **OSINTModule** | Watches: ENTITIES_EXTRACTED. Produces: OSINT_ENRICHED | Bus (no GPU) |
| **GeoModule** | Watches: ENTITIES_EXTRACTED. Produces: LOCATION_GEOCODED | Bus (no GPU) |
| **ImageModule** | Watches: EVIDENCE_INGESTED (type=image). Produces: IMAGE_ANALYZED | Bus, VRAMScheduler |
| **ForensicsModule** | Watches: EVIDENCE_INGESTED. Produces: FORENSIC_RESULT | Bus, VRAMScheduler |
| **TimelineModule** | Watches: EVIDENCE_INGESTED, ENTITIES_EXTRACTED. Produces: TIMELINE_UPDATED | Bus (no GPU) |
| **ReportModule** | Watches: HYPOTHESIS_SCORED (periodic). Produces: REPORT_GENERATED | Bus, VRAMScheduler |
| **Neo4jSyncModule** | Watches: EVIDENCE_INGESTED, ENTITIES_EXTRACTED, HYPOTHESIS_SCORED. Produces: GRAPH_SYNCED | Bus (no GPU) |
| **MonitoringTrigger** | External: APScheduler fires MONITORING_RESULT_FOUND when new results arrive | Bus |

### Data Flow: Evidence Ingestion Cascade

```
1. MonitoringScheduler detects new result
   -> emit MONITORING_RESULT_FOUND(result_id, case_id, relevance, title)

2. EvidenceModule handles MONITORING_RESULT_FOUND
   - Downloads/parses content
   - Creates Evidence record in SQLite
   - Runs GLiNER entity extraction (CPU, no GPU)
   - Chunks text + embeds in ChromaDB (light GPU: nomic-embed-text)
   -> emit EVIDENCE_INGESTED(evidence_id, case_id, evidence_type)
   -> emit ENTITIES_EXTRACTED(entity_ids, case_id)

3a. HypothesisModule handles EVIDENCE_INGESTED
    - Runs incremental analysis (heavy GPU: nexus 26B)
    - Re-scores existing hypotheses
    -> emit HYPOTHESIS_SCORED(hypothesis_id, old_score, new_score)

3b. ContradictionModule handles EVIDENCE_INGESTED
    - Checks new evidence against existing evidence (heavy GPU: deepseek-r1 14B)
    -> emit CONTRADICTION_FOUND(evidence_ids, description) [if any]

3c. OSINTModule handles ENTITIES_EXTRACTED
    - Runs holehe on email entities (no GPU)
    - Runs social recon on person entities (no GPU)
    -> emit OSINT_ENRICHED(entity_id, results)

3d. GeoModule handles ENTITIES_EXTRACTED
    - Geocodes location entities via Nominatim (no GPU)
    -> emit LOCATION_GEOCODED(entity_id, lat, lon)

3e. ImageModule handles EVIDENCE_INGESTED (filters type=image)
    - Runs VLM analysis (heavy GPU: qwen3-vl)
    - Indexes in DINOv2/CLIP
    -> emit IMAGE_ANALYZED(evidence_id, description)

3f. Neo4jSyncModule handles EVIDENCE_INGESTED + ENTITIES_EXTRACTED
    - Syncs evidence node + entity nodes + links to Neo4j
    -> emit GRAPH_SYNCED(node_count)

3g. TimelineModule handles EVIDENCE_INGESTED + ENTITIES_EXTRACTED
    - Extracts dates, builds chronological timeline
    -> emit TIMELINE_UPDATED(event_count)

4. SuspectModule handles HYPOTHESIS_SCORED
   - Re-scores suspects based on new hypothesis scores (heavy GPU: nexus 26B)
   -> emit SUSPECT_SCORED(suspect_id, score)

5. ReportModule handles HYPOTHESIS_SCORED (batched, periodic)
   - Generates updated investigation report
   -> emit REPORT_GENERATED(report_id)
```

## Patterns to Follow

### Pattern 1: Module Base Class (SpiderFoot-inspired)

**What:** Every reactive module inherits from ModuleBase and declares its event dependencies.
**When:** Every module in the system.
**Why:** Self-documenting dependencies, enables automatic wiring, enables dependency graph visualization.

```python
from dataclasses import dataclass, field
from enum import Enum, auto
from typing import Any
import asyncio


class EventType(Enum):
    """All event types in the system."""
    MONITORING_RESULT_FOUND = auto()
    EVIDENCE_INGESTED = auto()
    ENTITIES_EXTRACTED = auto()
    HYPOTHESIS_SCORED = auto()
    CONTRADICTION_FOUND = auto()
    SUSPECT_SCORED = auto()
    OSINT_ENRICHED = auto()
    LOCATION_GEOCODED = auto()
    IMAGE_ANALYZED = auto()
    FORENSIC_RESULT = auto()
    TIMELINE_UPDATED = auto()
    GRAPH_SYNCED = auto()
    REPORT_GENERATED = auto()
    # System events
    CYCLE_STARTED = auto()
    CYCLE_COMPLETED = auto()
    MODULE_ERROR = auto()


@dataclass(frozen=True)
class Event:
    """Immutable event flowing through the bus."""
    type: EventType
    case_id: str
    data: dict[str, Any] = field(default_factory=dict)
    source_module: str = ""
    event_id: str = ""  # UUID, set by bus
    parent_event_id: str = ""  # For tracing cascades
    timestamp: str = ""  # ISO 8601, set by bus


class ModuleBase:
    """Base class for all reactive investigation modules.

    Subclasses MUST define:
    - watches: set of EventType this module consumes
    - produces: set of EventType this module may emit
    - handle(event): async method called when a watched event arrives
    """
    watches: set[EventType] = set()
    produces: set[EventType] = set()
    name: str = "unnamed"

    def __init__(self, bus: "InvestigationBus"):
        self._bus = bus

    async def handle(self, event: Event) -> None:
        """Process an event. Override in subclasses."""
        raise NotImplementedError

    async def emit(self, event_type: EventType, data: dict, parent: Event | None = None) -> None:
        """Emit a new event onto the bus."""
        await self._bus.emit(Event(
            type=event_type,
            case_id=parent.case_id if parent else "",
            data=data,
            source_module=self.name,
            parent_event_id=parent.event_id if parent else "",
        ))
```

### Pattern 2: VRAM-Aware Priority Queue

**What:** Replace asyncio.Lock with PriorityQueue that groups tasks by model and respects priority.
**When:** Any module that needs GPU (LLM calls, VLM calls, embedding).
**Why:** Reduces VRAM swaps (loading/unloading models), allows priority escalation.

```python
import asyncio
from dataclasses import dataclass, field
from typing import Any


@dataclass(order=True)
class GPUTask:
    """A task waiting for GPU access."""
    priority: int  # Lower = higher priority. 0=urgent, 10=normal, 20=batch
    model_name: str = field(compare=False)
    coroutine: Any = field(compare=False)  # The actual async work
    future: asyncio.Future = field(compare=False)  # Result goes here


class VRAMScheduler:
    """Single-GPU task scheduler with priority and model batching."""

    def __init__(self):
        self._queue = asyncio.PriorityQueue()
        self._current_model: str | None = None
        self._running = False

    async def submit(self, model: str, coro, priority: int = 10) -> Any:
        """Submit a GPU task. Returns result when complete."""
        future = asyncio.get_event_loop().create_future()
        await self._queue.put(GPUTask(
            priority=priority,
            model_name=model,
            coroutine=coro,
            future=future,
        ))
        return await future

    async def run(self):
        """Worker loop: process GPU tasks one at a time."""
        self._running = True
        while self._running:
            task = await self._queue.get()
            try:
                # If model changed, Ollama will swap automatically.
                # Track it for batching optimization later.
                if task.model_name != self._current_model:
                    self._current_model = task.model_name
                result = await task.coroutine
                task.future.set_result(result)
            except Exception as e:
                task.future.set_exception(e)
            finally:
                self._queue.task_done()
```

### Pattern 3: Circuit Breaker for Event Storms

**What:** Limit cascading events to prevent infinite loops.
**When:** Any time event A triggers module B which emits event C which triggers module D which emits event A.
**Why:** Without this, one evidence ingest could trigger thousands of events.

```python
class CircuitBreaker:
    """Prevents event storms by limiting events per type per cycle."""

    def __init__(self, max_per_type: int = 50, cooldown_seconds: float = 5.0):
        self._counts: dict[EventType, int] = {}
        self._cooldowns: dict[EventType, float] = {}
        self._max = max_per_type
        self._cooldown = cooldown_seconds

    def allow(self, event_type: EventType) -> bool:
        """Return True if this event type is allowed to proceed."""
        count = self._counts.get(event_type, 0)
        if count >= self._max:
            return False
        self._counts[event_type] = count + 1
        return True

    def reset(self):
        """Reset all counters. Call at cycle boundary."""
        self._counts.clear()
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: God Event Bus (Single Global Bus)

**What:** One global EventBus shared by all cases.
**Why bad:** Events from Case A leak to Case B handlers. Debugging becomes impossible. Shutdown of one investigation affects all.
**Instead:** One InvestigationBus per case_id. BusManager creates/destroys them.

### Anti-Pattern 2: Synchronous Event Handling

**What:** `await bus.emit(event)` blocks until ALL handlers complete.
**Why bad:** A slow LLM call (600s timeout for nexus 26B) blocks the entire bus. No concurrency between independent modules.
**Instead:** Fire-and-forget dispatch. Handlers run as independent asyncio.Tasks. Collect results via Futures if needed.

### Anti-Pattern 3: Fine-Grained Events (One per Entity)

**What:** Emitting ENTITY_EXTRACTED once per entity instead of ENTITIES_EXTRACTED with a batch.
**Why bad:** If GLiNER extracts 30 entities from one evidence, that is 30 events triggering 30 OSINT scans simultaneously.
**Instead:** Batch events (ENTITIES_EXTRACTED with list of entity_ids). Let the receiving module decide how to iterate.

### Anti-Pattern 4: Replacing the OODA Loop Entirely on Day 1

**What:** Removing the existing 30-minute cycle before the event bus is proven.
**Why bad:** 41K lines of working code. The OODA loop is tested and produces results. A new event bus will have bugs.
**Instead:** Run event bus IN PARALLEL with OODA loop. OODA becomes the fallback "sweep" that catches anything the event bus missed. Remove OODA only after 100+ cycles of event bus running clean.

### Anti-Pattern 5: Persistent Event Queue on Disk

**What:** Writing every event to SQLite before processing (write-ahead).
**Why bad:** Adds latency to every event. SQLite writes are fast but not zero-cost. Events are transient -- if the process crashes, re-running the OODA sweep catches up.
**Instead:** Log events to the audit trail AFTER processing (write-behind). The audit trail is already append-only JSONL + SQLite + git.

## Scalability Considerations

| Concern | Current (1 case) | At 5 cases | At 20 cases |
|---------|-------------------|------------|-------------|
| Event throughput | ~50 events/cycle, trivial | ~250 events/cycle, still trivial for asyncio.Queue | ~1000 events/cycle, may need queue maxsize limits |
| VRAM contention | Single Lock works | PriorityQueue with per-case fairness needed | Priority starvation risk -- add case-round-robin |
| Memory | ~100 events in flight, negligible | ~500 events, still negligible | ~2000 events, add Queue maxsize (1000) |
| SQLite writes | Single writer, WAL handles reads | WAL handles concurrent reads fine | May need write batching to avoid lock contention |
| Neo4j sync | 1 case sync at a time | May need batched sync across cases | Dedicated Neo4j sync worker with its own queue |

## Sources

- [SpiderFoot architecture](https://deepwiki.com/smicallef/spiderfoot) -- module pattern, event types, execution loop
- [asyncio.Queue](https://docs.python.org/3/library/asyncio-queue.html) -- event dispatch backbone
- [asyncio.PriorityQueue](https://docs.python.org/3/library/asyncio-queue.html#priority-queue) -- GPU scheduling
- [GPU task scheduling with asyncio](https://speakerdeck.com/pyconza/juggling-gpu-tasks-with-asyncio-by-bruce-merry) -- PyCon talk on async GPU patterns
- [Event bus patterns](https://oneuptime.com/blog/post/2026-01-25-event-bus-asyncio-python/view) -- asyncio event bus implementation guide
- [TheHive/Cortex](https://docs.strangebee.com/cortex/) -- observable analysis event model
- [Ollama keep_alive FAQ](https://docs.ollama.com/faq) -- model loading/unloading behavior
