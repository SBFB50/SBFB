# SPDX-License-Identifier: AGPL-3.0-or-later
"""Full dispatcher → validator → kudos round-trip test.

Simulates 10 tasks going through the complete Phase B pipeline
**inside a single process** by using a second doc author as the
fake worker. The coordinator writes ``task:*``, the fake worker
signs and writes ``claim:*`` then ``result:*``, and the
validator's LiveEvent subscription drives the verification +
kudos credit path end-to-end.

No mocks. Every byte that goes through sign/verify is produced
by the real nexus_core Ed25519 helpers.
"""

from __future__ import annotations

import asyncio
import json
import time
from pathlib import Path

import nexus_core
import pytest
from nexus_coordinator.coordinator import Coordinator
from nexus_coordinator.dispatcher import SubmitRequest


async def _drain_validator(coord: Coordinator, expected: int, timeout_s: float = 10) -> list:
    """Pull events off the validator until ``expected`` task-related
    events (claim or result) have been observed or the timeout
    expires.
    """
    assert coord.validator is not None
    collected: list = []
    deadline = time.monotonic() + timeout_s
    while len(collected) < expected and time.monotonic() < deadline:
        try:
            batch = await asyncio.wait_for(coord.validator.run_once(1), timeout=1.0)
        except asyncio.TimeoutError:
            continue
        for ev in batch:
            if ev.kind in ("claim", "result_ok", "result_rejected"):
                collected.append(ev)
    return collected


@pytest.mark.asyncio
async def test_ten_tasks_round_trip_through_validator_and_kudos(
    nexus_grid_tmp: Path,
) -> None:
    coord = Coordinator(project_name="demo-loop")
    await coord.start()
    try:
        assert coord.dispatcher is not None
        assert coord.validator is not None
        assert coord.kudos_ledger is not None
        assert coord.state.doc is not None
        assert coord.state.node is not None

        # Simulated worker identity: fresh Ed25519 + a fresh author
        # on the same coordinator node (so the worker can write
        # claim:/result: entries to the shared doc without needing
        # to import a ticket).
        worker_kp = nexus_core.generate_secret()
        worker_secret: bytes = worker_kp["secret"]
        worker_pubkey: bytes = worker_kp["public"]
        worker_author = await coord.state.node.docs_author_create()

        # Submit 10 tasks via the dispatcher.
        task_ids: list[str] = []
        for i in range(10):
            tid = await coord.dispatcher.submit(
                SubmitRequest(
                    task_type="analysis",
                    prompt=f"Echo task {i}",
                    model="stub-model:latest",
                    priority=5,
                )
            )
            task_ids.append(tid)

        # Write a signed claim + result for each task, impersonating
        # the fake worker.
        for i, tid in enumerate(task_ids):
            now = int(time.time())
            claim_dict = {
                "version": 1,
                "task_id": tid,
                "claimed_by": list(worker_pubkey),
                "claimed_at": now,
            }
            signed_claim = nexus_core.sign_claim(
                json.dumps(claim_dict, sort_keys=True),
                worker_secret,
            )
            await coord.state.doc.set(
                worker_author,
                f"claim:{tid}".encode("utf-8"),
                signed_claim.encode("utf-8"),
            )

            result_payload = {
                "version": 1,
                "task_id": tid,
                "result_text": f"result for task {i}",
                "tokens_generated": 10 + i,
                "generation_time_ms": 100 + i,
                "model_digest": [0] * 32,
                "logprobs_hash": [0] * 32,
                "started_at": now,
                "finished_at": now + 1,
            }
            signed_result = nexus_core.sign_result(
                json.dumps(result_payload, sort_keys=True),
                worker_secret,
            )
            await coord.state.doc.set(
                worker_author,
                f"result:{tid}".encode("utf-8"),
                signed_result.encode("utf-8"),
            )

        # Drain the validator. Each task produces a claim event
        # and a result event (plus noop events for task:* writes
        # the validator should ignore), so we expect at least 20
        # task-related events.
        events = await _drain_validator(coord, expected=20, timeout_s=20)
        claim_events = [e for e in events if e.kind == "claim"]
        result_ok_events = [e for e in events if e.kind == "result_ok"]
        result_rejected = [e for e in events if e.kind == "result_rejected"]

        assert len(claim_events) == 10, (
            f"expected 10 claim validations, got {len(claim_events)} (rejected={len(result_rejected)})"
        )
        assert len(result_ok_events) == 10, (
            f"expected 10 successful results, got {len(result_ok_events)} (rejected={len(result_rejected)})"
        )

        # Every task should be in the completed state.
        completed = await coord.dispatcher.list_tasks(state="completed")
        assert len(completed) == 10

        # Kudos: 10 entries, chain valid.
        entries = await coord.kudos_ledger.list_entries(limit=100)
        assert len(entries) == 10
        ok, bad = await coord.kudos_ledger.verify_chain_integrity()
        assert ok, f"kudos chain should be valid, broke at row {bad}"

        # Every credit is attributed to the fake worker.
        assert all(e.worker_pubkey == worker_pubkey for e in entries)

        # Sum of amounts matches sum of token counts (quality=1.0,
        # trust=1.0 by default in Phase B).
        expected_total = float(sum(10 + i for i in range(10)))
        actual_total = await coord.kudos_ledger.total_for_worker(worker_pubkey)
        assert actual_total == expected_total
    finally:
        await coord.stop()
