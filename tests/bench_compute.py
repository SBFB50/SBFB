#!/usr/bin/env python
"""
NEXUS Compute — Synthetic dispatcher benchmark.

Measures the throughput and latency of the distributed compute stack
in isolation from LLM, Ollama, Neo4j and ChromaDB. The benchmark runs
entirely against an in-memory SQLite database with the real compute
schema, exercising the same code paths as production for:

- Task creation (ComputeDatabase.create_task)
- Task pulling (ComputeDatabase.pull_next_task — optimistic lock)
- Result storage + validation flow
- Spot-check coordinator event handling

This is the Sprint 0 reference for the SBFB pivot. The real
2-machine LLM benchmark (tokens/sec, heartbeat latency, crash/OOM)
is deferred until a second GPU machine is available. The numbers
produced here measure dispatcher/DB throughput, which is what the
upcoming Rust port needs to match or beat.

Usage::

    python tests/bench_compute.py                       # default 1000 tasks
    python tests/bench_compute.py --tasks 5000          # heavier run
    python tests/bench_compute.py --write-doc           # update docs/BENCHMARK_COMPUTE.md

Output: a markdown-formatted report to stdout. With --write-doc, the
report is also written to docs/BENCHMARK_COMPUTE.md with a timestamp
header.
"""
from __future__ import annotations

import argparse
import asyncio
import contextlib
import statistics
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

import aiosqlite

# Ensure repo root is importable when launched as a plain script
REPO_ROOT = Path(__file__).resolve().parent.parent
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from nexus.compute.db import (  # noqa: E402
    ComputeDatabase,
    _COMPUTE_CREATE_INDEXES,
    _COMPUTE_CREATE_TABLES,
)
from nexus.compute.dispatcher import SpotCheckCoordinator  # noqa: E402
from nexus.compute.events import ComputeEventType  # noqa: E402
from nexus.engine import NexusEvent  # noqa: E402


# ---------------------------------------------------------------------------
# Fake bus (same pattern as tests/test_compute.py)
# ---------------------------------------------------------------------------


class _FakeBus:
    def __init__(self) -> None:
        self.subscriptions: dict = {}
        self.published: list = []

    def subscribe(self, event_type, queue):
        self.subscriptions.setdefault(event_type, []).append(queue)

    def unsubscribe(self, event_type, queue):
        try:
            self.subscriptions.get(event_type, []).remove(queue)
        except ValueError:
            pass

    async def publish(self, event):
        self.published.append(event)
        return True


@contextlib.asynccontextmanager
async def _fake_get_db(conn):
    yield conn


# ---------------------------------------------------------------------------
# Benchmark fixtures
# ---------------------------------------------------------------------------


async def _mkdb() -> tuple[aiosqlite.Connection, ComputeDatabase]:
    conn = await aiosqlite.connect(":memory:")
    conn.row_factory = aiosqlite.Row
    await conn.execute("PRAGMA journal_mode = WAL")
    await conn.execute("PRAGMA foreign_keys = ON")
    await conn.execute("PRAGMA synchronous = NORMAL")
    await conn.executescript(_COMPUTE_CREATE_TABLES)
    await conn.executescript(_COMPUTE_CREATE_INDEXES)
    await conn.commit()
    return conn, ComputeDatabase(conn)


async def _register_fleet(db: ComputeDatabase, count: int) -> list[str]:
    """Register `count` synthetic nodes with varied trust scores."""
    node_ids: list[str] = []
    for i in range(count):
        node, _ = await db.register_node(
            name=f"bench-node-{i:03d}",
            gpu_model="RTX 5080",
            vram_mb=16000,
            ip=f"10.0.0.{i + 1}",
            platform="linux",
        )
        # Spread trust scores: some trusted, some default, some suspect
        if i % 10 == 0:
            await db.update_node_trust(node["id"], +40)  # 50 -> 90 (trusted)
        elif i % 7 == 0:
            await db.update_node_trust(node["id"], -20)  # 50 -> 30 (suspect)
        node_ids.append(node["id"])
    return node_ids


# ---------------------------------------------------------------------------
# Benchmark steps
# ---------------------------------------------------------------------------


async def bench_task_creation(db: ComputeDatabase, n: int) -> dict:
    """Measure throughput of ComputeDatabase.create_task."""
    latencies: list[float] = []
    t0 = time.perf_counter()
    for i in range(n):
        ts = time.perf_counter()
        await db.create_task(
            task_type="bench",
            prompt=f"Synthetic prompt {i}",
            model="bench-model",
            priority=5,
        )
        latencies.append((time.perf_counter() - ts) * 1000.0)
    elapsed = time.perf_counter() - t0
    return {
        "operation": "create_task",
        "count": n,
        "total_s": elapsed,
        "throughput_ops_s": n / elapsed,
        "p50_ms": statistics.median(latencies),
        "p95_ms": _percentile(latencies, 95),
        "p99_ms": _percentile(latencies, 99),
    }


async def bench_task_pull(
    db: ComputeDatabase, node_ids: list[str], n: int,
) -> dict:
    """Measure throughput of atomic pull_next_task with round-robin nodes."""
    latencies: list[float] = []
    t0 = time.perf_counter()
    fleet = node_ids
    pulled = 0
    i = 0
    while pulled < n:
        node_id = fleet[i % len(fleet)]
        ts = time.perf_counter()
        task = await db.pull_next_task(node_id, model="bench-model")
        latencies.append((time.perf_counter() - ts) * 1000.0)
        if task is None:
            break
        pulled += 1
        i += 1
    elapsed = time.perf_counter() - t0
    return {
        "operation": "pull_next_task",
        "count": pulled,
        "total_s": elapsed,
        "throughput_ops_s": pulled / elapsed if elapsed > 0 else 0.0,
        "p50_ms": statistics.median(latencies),
        "p95_ms": _percentile(latencies, 95),
        "p99_ms": _percentile(latencies, 99),
    }


async def bench_task_complete(
    db: ComputeDatabase, pulled_tasks: list[str], node_ids: list[str],
) -> dict:
    """Measure throughput of store_result + complete_task + increment_stats."""
    latencies: list[float] = []
    t0 = time.perf_counter()
    for i, task_id in enumerate(pulled_tasks):
        node_id = node_ids[i % len(node_ids)]
        ts = time.perf_counter()
        await db.store_result(
            task_id=task_id,
            node_id=node_id,
            result_text=f"Synthetic result {i}",
            tokens_generated=100,
            generation_time_ms=1000,
        )
        await db.complete_task(task_id, f"Synthetic result {i}", validated=True)
        await db.increment_node_stats(node_id, completed=1, tokens_per_sec=100.0)
        latencies.append((time.perf_counter() - ts) * 1000.0)
    elapsed = time.perf_counter() - t0
    return {
        "operation": "store_result + complete + stats",
        "count": len(pulled_tasks),
        "total_s": elapsed,
        "throughput_ops_s": len(pulled_tasks) / elapsed if elapsed > 0 else 0.0,
        "p50_ms": statistics.median(latencies) if latencies else 0.0,
        "p95_ms": _percentile(latencies, 95) if latencies else 0.0,
        "p99_ms": _percentile(latencies, 99) if latencies else 0.0,
    }


async def bench_spot_check(
    db: ComputeDatabase, node_ids: list[str], n: int,
) -> dict:
    """Measure throughput of the spot-check coordinator handling N events."""
    from nexus.compute import dispatcher as disp_mod

    # Create N original tasks to be spot-checked
    original_tasks = []
    for i in range(n):
        t = await db.create_task(
            task_type="analysis",
            prompt=f"Spot-check prompt {i}",
            model="bench-model",
            priority=5,
        )
        original_tasks.append(t)

    # Use the first suspect node as "original" and trusted nodes as verifiers
    original_node = node_ids[7]  # likely suspect (i % 7 == 0 rule)
    bus = _FakeBus()
    coordinator = SpotCheckCoordinator(bus=bus)

    latencies: list[float] = []
    t0 = time.perf_counter()
    with _patch_get_db(disp_mod, db._conn):
        for t in original_tasks:
            event = NexusEvent(
                event_type=ComputeEventType.COMPUTE_SPOT_CHECK_NEEDED,
                case_id="compute",
                payload={
                    "task_id": t["id"],
                    "node_id": original_node,
                    "result_text": "synthetic",
                    "prompt": t["prompt"],
                },
                source_worker="bench",
            )
            ts = time.perf_counter()
            await coordinator.handle_event(event)
            latencies.append((time.perf_counter() - ts) * 1000.0)
    elapsed = time.perf_counter() - t0
    return {
        "operation": "spot_check_coordinator.handle_event",
        "count": n,
        "total_s": elapsed,
        "throughput_ops_s": n / elapsed if elapsed > 0 else 0.0,
        "created": coordinator.spot_checks_created,
        "skipped": coordinator.spot_checks_skipped,
        "p50_ms": statistics.median(latencies),
        "p95_ms": _percentile(latencies, 95),
        "p99_ms": _percentile(latencies, 99),
    }


@contextlib.contextmanager
def _patch_get_db(disp_mod, conn):
    """Monkey-patch nexus.compute.dispatcher.get_db to return our conn."""
    original = disp_mod.get_db
    disp_mod.get_db = lambda: _fake_get_db(conn)
    try:
        yield
    finally:
        disp_mod.get_db = original


def _percentile(data: list[float], p: int) -> float:
    if not data:
        return 0.0
    srt = sorted(data)
    k = int(round((p / 100.0) * (len(srt) - 1)))
    return srt[k]


# ---------------------------------------------------------------------------
# Report formatter
# ---------------------------------------------------------------------------


def _format_report(sections: list[dict], args: argparse.Namespace) -> str:
    now = datetime.now(timezone.utc).isoformat(timespec="seconds")
    lines = [
        "# NEXUS Compute — Synthetic dispatcher benchmark",
        "",
        f"_Generated: {now}_",
        "",
        "## Scope and limitations",
        "",
        "This benchmark measures the compute dispatcher + SQLite",
        "persistence layer in isolation from the LLM backend, Ollama,",
        "Neo4j and ChromaDB. It uses the same schema and code paths as",
        "production, on an in-memory SQLite with WAL mode.",
        "",
        "**What it does NOT measure**:",
        "",
        "- Real tokens/sec from an Ollama model (no LLM calls)",
        "- Cross-machine latency (single-machine run)",
        "- OOM / crash behavior under adversarial GPU load",
        "- End-to-end HTTP API latency (no FastAPI boot)",
        "",
        "These are deferred until the full 2-machine plan J3-J4 can",
        "run with a second physical GPU worker available.",
        "",
        "**What it DOES measure**:",
        "",
        "- create_task throughput (dispatcher ingestion)",
        "- pull_next_task throughput with BEGIN IMMEDIATE optimistic",
        "  locking under round-robin node contention",
        "- store_result + complete_task + increment_node_stats",
        "  combined throughput",
        "- SpotCheckCoordinator.handle_event throughput",
        "",
        "## Parameters",
        "",
        f"- Task count: **{args.tasks}**",
        f"- Node count: **{args.nodes}**",
        f"- Spot-check events: **{args.spot_checks}**",
        f"- Python: `{sys.version.split()[0]}`",
        f"- Platform: `{sys.platform}`",
        "",
        "## Results",
        "",
        "| Operation | Count | Total (s) | Throughput (ops/s) | p50 (ms) | p95 (ms) | p99 (ms) |",
        "|---|---:|---:|---:|---:|---:|---:|",
    ]
    for s in sections:
        lines.append(
            "| {op} | {count} | {total:.3f} | {thr:.1f} | {p50:.3f} | {p95:.3f} | {p99:.3f} |".format(
                op=s["operation"],
                count=s["count"],
                total=s["total_s"],
                thr=s["throughput_ops_s"],
                p50=s["p50_ms"],
                p95=s["p95_ms"],
                p99=s["p99_ms"],
            )
        )

    # Spot-check supplementary section
    sc = next((s for s in sections if "created" in s), None)
    if sc is not None:
        lines += [
            "",
            "### Spot-check coordinator detail",
            "",
            f"- Events handled: {sc['count']}",
            f"- Duplicates created: {sc['created']}",
            f"- Events skipped: {sc['skipped']} "
            "(no trusted verifier available or original node was the only trusted node)",
        ]

    lines += [
        "",
        "## Reference for the Rust port",
        "",
        "When the SBFB nexus-core-rs + nexus-worker rewrite reaches",
        "parity in Sprint 2-3, re-run this benchmark on the same",
        "machine and compare. Regressions vs these numbers indicate",
        "a problem in the Rust port that must be investigated before",
        "merging. Targets: match Python within ±15% on throughput,",
        "match or improve on p99 latency.",
        "",
    ]
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


async def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tasks", type=int, default=1000,
                        help="Number of tasks to create/pull/complete")
    parser.add_argument("--nodes", type=int, default=10,
                        help="Number of synthetic nodes to register")
    parser.add_argument("--spot-checks", type=int, default=100,
                        help="Number of spot-check events to process")
    parser.add_argument("--write-doc", action="store_true",
                        help="Write the report to docs/BENCHMARK_COMPUTE.md")
    args = parser.parse_args()

    print(f"[bench] Initializing in-memory compute DB...", file=sys.stderr)
    conn, db = await _mkdb()

    try:
        print(f"[bench] Registering {args.nodes} synthetic nodes...", file=sys.stderr)
        node_ids = await _register_fleet(db, args.nodes)

        print(f"[bench] create_task x {args.tasks}...", file=sys.stderr)
        create_stats = await bench_task_creation(db, args.tasks)

        print(f"[bench] pull_next_task x {args.tasks}...", file=sys.stderr)
        pull_stats = await bench_task_pull(db, node_ids, args.tasks)

        # Re-fetch the list of tasks actually pulled (status=assigned).
        # list_tasks() defaults to limit=100 — pass the task count so we
        # can observe all completions.
        assigned = await db.list_tasks(status="assigned", limit=args.tasks + 100)
        pulled_ids = [t["id"] for t in assigned]

        print(f"[bench] store_result x {len(pulled_ids)}...", file=sys.stderr)
        complete_stats = await bench_task_complete(db, pulled_ids, node_ids)

        print(f"[bench] spot_check_coordinator x {args.spot_checks}...", file=sys.stderr)
        spot_stats = await bench_spot_check(db, node_ids, args.spot_checks)

        report = _format_report(
            [create_stats, pull_stats, complete_stats, spot_stats],
            args,
        )
        print(report)

        if args.write_doc:
            doc_path = REPO_ROOT / "docs" / "BENCHMARK_COMPUTE.md"
            doc_path.parent.mkdir(parents=True, exist_ok=True)
            doc_path.write_text(report, encoding="utf-8")
            print(f"\n[bench] Report written to {doc_path}", file=sys.stderr)

    finally:
        await conn.close()

    return 0


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))
