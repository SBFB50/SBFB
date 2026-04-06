# Feature Landscape: Reactive Event-Driven Architecture

**Domain:** Event-driven system for autonomous investigation
**Researched:** 2026-04-06

## Table Stakes

Features the reactive system MUST have. Missing = system regresses from current behavior.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Typed event definitions | Type safety prevents wiring bugs at scale | Low | dataclass with frozen=True, enum for event types |
| Async event dispatch | Core of the reactive pattern | Low | asyncio.Queue, fire-and-forget or await-result |
| Handler registration (watches/produces) | Modules declare dependencies | Low | SpiderFoot pattern, decorator-based |
| VRAM serialization | Prevent OOM on RTX 5080 16GB | Med | PriorityQueue replaces asyncio.Lock |
| Graceful shutdown | Stop investigation without losing in-flight events | Med | drain queue, cancel pending, persist unprocessed |
| Error isolation | One handler crash must not kill the bus | Low | try/except per handler, log + continue |
| Event deduplication | Prevent processing same evidence twice | Med | Content hash or event_id tracking per cycle |
| Audit trail integration | Existing 3-layer audit must continue working | Low | Emit audit events alongside domain events |
| Investigation-scoped buses | Each case_id gets its own event flow | Med | Prevents cross-case contamination |
| Status tracking per tool | Frontend shows tool status (running/done/error) | Low | Already exists via _track_tool, emit status events |

## Differentiators

Features that make the reactive system genuinely better than the current OODA loop.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Immediate reaction to evidence | No 30-min wait: new evidence triggers analysis in seconds | Med | Core value of the migration |
| Model-aware batching | Group LLM calls by model to reduce VRAM swaps | High | Accumulate tasks, batch by model, flush periodically |
| Priority-based GPU access | Deep analysis yields to urgent contradiction detection | Med | PriorityQueue with task priority levels |
| Event replay / debugging | Replay a sequence of events to reproduce bugs | Med | Append-only event log + replay function |
| Dependency graph visualization | Show which modules trigger which (for debugging) | Low | Auto-generate from watches/produces declarations |
| Circuit breakers | Stop cascading event storms (evidence -> entities -> OSINT -> entities -> ...) | Med | Max events per type per cycle, cooldown timers |
| Selective re-processing | Re-run only hypothesis evaluation without full OODA cycle | Low | Emit specific event to trigger specific module |
| WebSocket event stream | Frontend sees events in real-time, not polling | Med | Broadcast events to connected React clients |
| Ollama keep_alive optimization | Keep frequently-used models loaded, unload idle ones | Med | Track model usage patterns, set keep_alive dynamically |

## Anti-Features

Features to explicitly NOT build.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| External message broker (Redis/RabbitMQ) | Single process, adds deployment complexity, no multi-process need | In-process asyncio.Queue |
| Full CQRS separation | NEXUS reads and writes from same process, CQRS adds complexity without benefit | Single DB connection per operation, as currently done |
| Event schema versioning | Single process, no backward compatibility needed between services | Simple dataclass evolution, Python type checking |
| Distributed event store | No microservices, no multi-node deployment | SQLite append-only table, evolve from existing audit |
| Backpressure to external producers | All producers are internal modules, not external services | Simple Queue maxsize + drop-oldest or block |
| Saga pattern / compensating transactions | Investigation actions are not transactional (no rollback needed) | Simple error handling + retry via tenacity |
| Dead letter queue | Overkill for in-process events; log errors and move on | Log failed events with full context for debugging |

## Feature Dependencies

```
Typed Events -> EventBus Core -> Handler Registration
EventBus Core -> VRAM PriorityQueue
EventBus Core -> Error Isolation
EventBus Core -> Event Deduplication
Handler Registration -> Module Migration (one by one)
Module Migration -> Circuit Breakers (needed once cascading starts)
EventBus Core -> Event Replay (optional, can come later)
EventBus Core -> WebSocket Stream (optional, can come later)
VRAM PriorityQueue -> Model Batching (optimization layer)
Model Batching -> Ollama keep_alive Optimization
```

## MVP Recommendation

Prioritize:
1. **Typed event definitions + EventBus** -- foundation, everything depends on it
2. **Handler registration with watches/produces** -- enables incremental module migration
3. **VRAM PriorityQueue** -- immediate performance improvement over asyncio.Lock
4. **Error isolation + graceful shutdown** -- production safety
5. **Circuit breakers** -- prevent event storms during first real usage

Defer:
- **Event replay**: Valuable for debugging but not blocking. Add after first 3 modules are migrated.
- **Model batching**: Optimization that requires usage data to tune properly. Add after observing real workload patterns.
- **WebSocket stream**: Nice UX improvement but the frontend already polls. Add when event bus is stable.
- **Ollama keep_alive optimization**: Requires empirical benchmarking on RTX 5080. Add after basic PriorityQueue is working.

## Sources

- [SpiderFoot module pattern](https://deepwiki.com/smicallef/spiderfoot) -- watchedEvents/producedEvents architecture
- [TheHive/Cortex observable analysis](https://docs.strangebee.com/cortex/) -- investigation tool event patterns
- [Maltego transform pattern](https://www.maltego.com/blog/how-to-use-maltego-transforms-to-map-network-infrastructure-an-in-depth-guide/) -- entity-to-entity reactive transforms
- [bubus circuit breaker patterns](https://github.com/browser-use/bubus) -- event loop prevention via event_path tracking
