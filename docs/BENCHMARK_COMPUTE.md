# NEXUS Compute — Synthetic dispatcher benchmark

_Generated: 2026-04-10T12:19:21+00:00_

## Scope and limitations

This benchmark measures the compute dispatcher + SQLite
persistence layer in isolation from the LLM backend, Ollama,
Neo4j and ChromaDB. It uses the same schema and code paths as
production, on an in-memory SQLite with WAL mode.

**What it does NOT measure**:

- Real tokens/sec from an Ollama model (no LLM calls)
- Cross-machine latency (single-machine run)
- OOM / crash behavior under adversarial GPU load
- End-to-end HTTP API latency (no FastAPI boot)

These are deferred until the full 2-machine plan J3-J4 can
run with a second physical GPU worker available.

**What it DOES measure**:

- create_task throughput (dispatcher ingestion)
- pull_next_task throughput with BEGIN IMMEDIATE optimistic
  locking under round-robin node contention
- store_result + complete_task + increment_node_stats
  combined throughput
- SpotCheckCoordinator.handle_event throughput

## Parameters

- Task count: **2000**
- Node count: **10**
- Spot-check events: **200**
- Python: `3.13.9`
- Platform: `win32`

## Results

| Operation | Count | Total (s) | Throughput (ops/s) | p50 (ms) | p95 (ms) | p99 (ms) |
|---|---:|---:|---:|---:|---:|---:|
| create_task | 2000 | 0.883 | 2265.5 | 0.393 | 0.755 | 1.483 |
| pull_next_task | 2000 | 1.242 | 1610.3 | 0.588 | 0.982 | 1.103 |
| store_result + complete + stats | 2000 | 1.291 | 1549.6 | 0.595 | 0.949 | 1.137 |
| spot_check_coordinator.handle_event | 200 | 0.127 | 1571.6 | 0.589 | 0.895 | 1.011 |

### Spot-check coordinator detail

- Events handled: 200
- Duplicates created: 200
- Events skipped: 0 (no trusted verifier available or original node was the only trusted node)

## Reference for the Rust port

When the SBFB nexus-core-rs + nexus-worker rewrite reaches
parity in Sprint 2-3, re-run this benchmark on the same
machine and compare. Regressions vs these numbers indicate
a problem in the Rust port that must be investigated before
merging. Targets: match Python within ±15% on throughput,
match or improve on p99 latency.
