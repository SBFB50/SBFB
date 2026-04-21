# SPDX-License-Identifier: AGPL-3.0-or-later
"""Re-run sampling — compute theft detection via statistical spot-check.

RerunSampler selects sample_rate% of completed tasks for re-dispatch
to a second worker. DivergenceScorer (hook on_result_received) compares
the BLAKE3 content hash of the original result vs the re-run. Binary
mismatch → structured log + quarantine divergent worker.

Threat chain: S22 NVML baseline → S24 re-run sampling (C-ComputeTheft).
Config: rerun_sample_rate (float 0.01–0.05, default 0.01).

1% on 100 tasks/day = 1 re-run; detection latency ~1 day for a
systematically divergent worker (BOINC/Folding@Home use full
replication for deterministic tasks; LLM tasks are stochastic,
spot-check is the only applicable pattern).
"""

from __future__ import annotations

import json
import random
import uuid
from pathlib import Path
from typing import Any

import structlog

from nexus_coordinator.hooks import DispatchHook, HookContext

_log = structlog.get_logger(__name__)


class RerunConfig:
    """Parse ``rerun_sample_rate`` from a TOML file."""

    def __init__(self, *, sample_rate: float = 0.01) -> None:
        self.sample_rate = sample_rate

    @classmethod
    def from_toml(cls, path: Path) -> RerunConfig:
        import tomllib

        with open(path, "rb") as f:
            data = tomllib.load(f)
        raw = data.get("rerun_sample_rate", 0.01)
        rate = float(raw)
        if rate < 0.0 or rate > 1.0:
            _log.warning("rerun_sample_rate out of range, clamping", original=rate)
            rate = max(0.0, min(1.0, rate))
        return cls(sample_rate=rate)


class RerunSampler:
    """Selects completed tasks for re-dispatch spot-check.

    In-memory registry maps ``rerun_task_id → original_task_id``.
    Re-run tasks are never themselves re-run (prevents infinite chains).
    """

    def __init__(self, *, sample_rate: float = 0.01) -> None:
        if sample_rate < 0.0 or sample_rate > 1.0:
            _log.warning("rerun_sample_rate clamped", original=sample_rate)
        self._sample_rate: float = max(0.0, min(1.0, sample_rate))
        self._rerun_map: dict[str, str] = {}

    @property
    def sample_rate(self) -> float:
        return self._sample_rate

    def should_rerun(self, task_id: str) -> bool:
        """Randomly decide if a completed task should be re-run."""
        if self.is_rerun(task_id):
            return False
        return random.random() < self._sample_rate

    def make_rerun_id(self, original_task_id: str) -> str:
        """Generate a distinct re-run task_id and register the mapping."""
        rerun_id = f"rerun-{uuid.uuid4().hex}"
        self.register_rerun(original_task_id, rerun_id)
        return rerun_id

    def register_rerun(self, original_task_id: str, rerun_task_id: str) -> None:
        self._rerun_map[rerun_task_id] = original_task_id

    def is_rerun(self, task_id: str) -> bool:
        return task_id in self._rerun_map

    def get_original(self, rerun_task_id: str) -> str | None:
        return self._rerun_map.get(rerun_task_id)


class DivergenceScorer(DispatchHook):
    """Hook on_result_received — detects result divergence for re-run tasks.

    Binary hash comparison: score 0.0 (identical) or 1.0 (mismatch).
    On mismatch, quarantines the divergent worker via QuarantineQueue.
    """

    def __init__(
        self,
        *,
        sampler: RerunSampler,
        db_path: Path | None = None,
        quarantine: Any | None = None,
    ) -> None:
        self._sampler = sampler
        self._db_path = db_path
        self._quarantine = quarantine

    @property
    def name(self) -> str:
        return "divergence_scorer"

    @staticmethod
    def score(original_hash: bytes, rerun_hash: bytes) -> float:
        """Binary comparison: 0.0 if identical, 1.0 if mismatch."""
        return 0.0 if original_hash == rerun_hash else 1.0

    async def __call__(self, ctx: HookContext) -> None:
        if ctx.event != "on_result_received":
            return
        original_id = self._sampler.get_original(ctx.task_id)
        if original_id is None:
            return

        rerun_hash_raw = ctx.metadata.get("result_hash")
        if rerun_hash_raw is None:
            return
        rerun_hash = bytes.fromhex(rerun_hash_raw) if isinstance(rerun_hash_raw, str) else bytes(rerun_hash_raw)

        original_hash = await self._get_result_hash(original_id)
        if original_hash is None:
            _log.warning(
                "divergence_scorer_original_hash_missing",
                original_task_id=original_id,
                rerun_task_id=ctx.task_id,
            )
            return

        divergence = self.score(original_hash, rerun_hash)
        worker = ctx.metadata.get("worker_pubkey_hex", "unknown")
        if divergence > 0.0:
            _log.warning(
                "divergence_detected",
                original_task_id=original_id,
                rerun_task_id=ctx.task_id,
                worker=worker,
                score=divergence,
            )
            if self._quarantine is not None:
                await self._quarantine.add(
                    topic="divergence",
                    sender_pubkey=bytes.fromhex(worker) if worker != "unknown" else b"\x00" * 32,
                    payload_bytes=json.dumps(
                        {
                            "original_task_id": original_id,
                            "rerun_task_id": ctx.task_id,
                            "divergence_score": divergence,
                        }
                    ).encode("utf-8"),
                    rate_strikes=0,
                    pow_status="valid",
                    task_id=original_id,
                )
        else:
            _log.info(
                "rerun_result_matches",
                original_task_id=original_id,
                rerun_task_id=ctx.task_id,
            )

    async def _get_result_hash(self, task_id: str) -> bytes | None:
        if self._db_path is None:
            return None
        import aiosqlite

        async with aiosqlite.connect(self._db_path) as db:
            async with db.execute(
                "SELECT result_hash FROM task_state WHERE task_id = ?",
                (task_id,),
            ) as cursor:
                row = await cursor.fetchone()
        if row is None or row[0] is None:
            return None
        return bytes(row[0]) if not isinstance(row[0], bytes) else row[0]


__all__ = [
    "DivergenceScorer",
    "RerunConfig",
    "RerunSampler",
]
