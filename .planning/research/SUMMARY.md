# Research Summary: Reactive Event-Driven Architecture for NEXUS

**Domain:** Event-driven system design for an autonomous investigation platform
**Researched:** 2026-04-06
**Overall confidence:** HIGH

## Executive Summary

NEXUS currently runs a monolithic OODA loop that sleeps 30 minutes between cycles, executing every tool sequentially regardless of whether inputs changed. This is wasteful: if no new evidence arrives, the system still re-runs geocoding, OSINT, image analysis, forensics, and hypothesis evaluation. The solution is an event-driven architecture where each tool reacts only when its inputs change.

After researching the ecosystem, the recommended approach is a **custom in-process EventBus built on asyncio.Queue and asyncio.PriorityQueue**, combined with SpiderFoot-style module declarations (`watches` / `produces`). This avoids external dependencies (no Redis, no RabbitMQ, no Kafka) while giving NEXUS the reactive behavior it needs. The existing `asyncio.Lock` for VRAM serialization evolves into a proper priority queue with model-aware batching.

The key insight from investigating OSINT tools (SpiderFoot, TheHive/Cortex, Maltego) is that they all use the same pattern: **typed events flow between self-declaring modules**. SpiderFoot's `watchedEvents()` / `producedEvents()` / `handleEvent()` pattern is the gold standard for investigation tools and maps directly to NEXUS's 21 modules.

Event sourcing (append-only event log) is a natural fit because NEXUS already has a 3-layer audit trail. The audit log becomes the event store with minimal changes. CQRS is overkill for a single-process system.

## Key Findings

**Stack:** Custom EventBus on asyncio.Queue + PriorityQueue. No external message broker needed. Bubus library considered but too young (v0.x) for production.
**Architecture:** SpiderFoot-inspired module pattern -- each module declares what events it watches and produces. Central EventBus dispatches. VRAM access via PriorityQueue.
**Critical pitfall:** Event storms from cascading handlers. One evidence ingest triggers entities, which trigger OSINT, which trigger more entities. Need circuit breakers and per-cycle dedup.

## Implications for Roadmap

Based on research, suggested phase structure:

1. **EventBus Core** - Build the typed event system and priority queue
   - Addresses: Event definitions, bus implementation, VRAM-aware scheduling
   - Avoids: Overengineering with external brokers, premature event sourcing

2. **Module Migration** - Convert existing tools to event-driven modules
   - Addresses: EvidenceProcessor, HypothesisEngine, ContradictionDetector etc.
   - Avoids: Big-bang rewrite -- migrate one module at a time, keep OODA fallback

3. **VRAM Optimization** - Model batching and smart scheduling
   - Addresses: Group LLM calls by model, reduce model swaps, Ollama keep_alive
   - Avoids: Complex distributed scheduling, multiple GPU assumptions

4. **Event Persistence** - Evolve audit trail into event store
   - Addresses: Append-only event log, replay capability, temporal debugging
   - Avoids: Full CQRS separation, external event store

**Phase ordering rationale:**
- EventBus must come first -- everything else depends on it
- Module migration is incremental and can be done tool-by-tool alongside existing OODA loop
- VRAM optimization builds on the priority queue from Phase 1
- Event persistence is valuable but not blocking -- the system works without it

**Research flags for phases:**
- Phase 1: Standard patterns, unlikely to need deeper research
- Phase 2: Each module migration may reveal specific integration challenges
- Phase 3: Ollama keep_alive behavior under concurrent requests needs empirical testing on the RTX 5080

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Event bus pattern | HIGH | Well-established asyncio patterns, multiple implementations studied |
| SpiderFoot module pattern | HIGH | Open source, verified architecture via DeepWiki |
| VRAM scheduling | MEDIUM | Pattern is clear but Ollama-specific behavior needs empirical testing |
| Event sourcing fit | HIGH | Natural evolution of existing audit trail |
| Library recommendations | MEDIUM | bubus is promising but young; custom build is safer for 41K-line codebase |

## Gaps to Address

- Ollama `keep_alive` interaction with `asyncio.Lock` under contention needs benchmarking
- Exact event granularity (per-evidence vs per-batch) requires experimentation
- Neo4j sync timing under event-driven flow vs batch-per-cycle needs testing
- Frontend WebSocket integration for real-time event streaming to the React UI
