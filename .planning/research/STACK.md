# Technology Stack: Reactive Event-Driven Architecture

**Project:** NEXUS Reactive Migration
**Researched:** 2026-04-06

## Recommended Stack

### Core: Custom EventBus (no external dependency)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Python asyncio.Queue | stdlib 3.13 | Event dispatch backbone | Zero dependency, native async, proven in production |
| Python asyncio.PriorityQueue | stdlib 3.13 | VRAM-aware GPU task scheduling | Built-in priority ordering, heapq-based, efficient |
| Python dataclasses | stdlib 3.13 | Typed event definitions | Lightweight, frozen=True for immutability, no Pydantic needed for events |
| Python asyncio.Event/Condition | stdlib 3.13 | Synchronization primitives | Coordinate module lifecycle, shutdown signals |

### Supporting Libraries (already in project)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| loguru | existing | Structured event logging | Every event emission and handler invocation |
| pydantic | existing | Event payload validation (optional) | Complex events with nested data, API-facing events |
| tenacity | existing | Retry logic for failed handlers | OSINT calls, LLM calls that may timeout |

### Libraries Evaluated But NOT Recommended

| Library | Why Not |
|---------|---------|
| **bubus** (browser-use) | Promising (Pydantic events, WAL, SQLite middleware) but v0.x, 165 commits, 99 stars. Too young for a 41K-line production system. Re-evaluate in 6 months. |
| **aiopubsub** (Quantlane) | Decent but key-based subscription is awkward for typed events. Moved to GitLab, low maintenance signal. |
| **blinker** (Pallets) | Signal/slot pattern is too simple -- no queue, no priority, no persistence. Good for Flask, wrong for pipeline orchestration. |
| **RxPY / reactivex** | 120+ operators but steep learning curve, functional-reactive paradigm is foreign to the codebase. Overkill for "react when input changes." |
| **Faust (faust-streaming)** | Requires Kafka. NEXUS is single-process on one machine. Kafka adds deployment complexity for zero benefit. |
| **Dramatiq** | Task queue, not event bus. Requires Redis/RabbitMQ broker. Right pattern for distributed systems, wrong for in-process reactive flow. |
| **Celery** | Same as Dramatiq but heavier. External broker dependency is unnecessary. |
| **Temporal.io** | Workflow orchestration for distributed systems. Massive overhead for single-process Python. |
| **fastapi-events** | Request-scoped event dispatch (ASGI middleware). NEXUS events are background-task-scoped, not request-scoped. |
| **Redis pub/sub** | Adds an external service dependency for pub/sub that asyncio.Queue does natively in-process. Would only matter if NEXUS became multi-process. |

## Rationale: Why Custom Over Library

1. **NEXUS is single-process.** No need for cross-process message passing (Redis, RabbitMQ, Kafka).
2. **The codebase already uses asyncio.Lock for VRAM.** Evolving to PriorityQueue is natural.
3. **41K lines of existing code.** A library that imposes its own patterns (RxPY, Faust) would require massive refactoring.
4. **SpiderFoot proves the pattern works.** 207 modules, event-driven, all custom Python, no external broker.
5. **Event bus is ~200 lines of code.** The complexity is in the module migration, not the bus itself.

## Installation

```bash
# No new dependencies needed for core EventBus.
# Everything uses Python stdlib asyncio.

# If event payload validation is desired (optional, Pydantic already in project):
# Already installed: pydantic >= 2.0
```

## Sources

- [asyncio.Queue docs](https://docs.python.org/3/library/asyncio-queue.html) -- stdlib, HIGH confidence
- [asyncio.PriorityQueue](https://superfastpython.com/asyncio-priorityqueue/) -- stdlib, HIGH confidence
- [bubus GitHub](https://github.com/browser-use/bubus) -- evaluated, MEDIUM confidence (young project)
- [aiopubsub GitHub](https://github.com/qntln/aiopubsub) -- evaluated, MEDIUM confidence
- [blinker docs](https://blinker.readthedocs.io/) -- evaluated, HIGH confidence
- [RxPY GitHub](https://github.com/ReactiveX/RxPY) -- evaluated, HIGH confidence
- [faust-streaming GitHub](https://github.com/faust-streaming/faust) -- evaluated, HIGH confidence
- [SpiderFoot DeepWiki](https://deepwiki.com/smicallef/spiderfoot) -- architecture reference, HIGH confidence
