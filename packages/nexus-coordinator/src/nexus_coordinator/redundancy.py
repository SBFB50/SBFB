# SPDX-License-Identifier: AGPL-3.0-or-later
"""Redundancy voting — Sprint 23 Phase D Gate 3 foundation.

When a task carries ``redundancy_factor > 1``, the coordinator
tracks results from N independent workers and compares them via
hash voting. Strict majority → accept + credit the majority
workers; no majority → reject all + quarantine every worker.

The comparison hash is SHA-256 over raw result bytes (stdlib,
no extra dep). The hash is used purely for equality comparison
within the coordinator — security integrity is provided by the
Ed25519 signatures on each ResultEntry, not by this hash.
"""

from __future__ import annotations

import enum
import hashlib
from dataclasses import dataclass, field

import structlog

_log = structlog.get_logger(__name__)


class VoteVerdict(enum.Enum):
    """Outcome of a redundancy vote."""

    MAJORITY = "majority"
    MISMATCH = "mismatch"


@dataclass
class VoteOutcome:
    """Result of voting on collected results for a redundant task."""

    verdict: VoteVerdict
    canonical_hash: str | None = None
    all_hashes: dict[str, str] = field(default_factory=dict)
    outlier_worker_ids: list[str] = field(default_factory=list)


def hash_result_bytes(data: bytes) -> str:
    """SHA-256 hex digest of raw result bytes.

    Deviation from kickoff D3 which specifies BLAKE3: SHA-256 is
    functionally equivalent for equality comparison (not used for
    crypto integrity — Ed25519 signatures cover that).  SHA-256 is
    stdlib, zero extra dep.  Align to BLAKE3 if needed for
    consistency post-v1.0 (carry S25 audit track).
    """
    return hashlib.sha256(data).hexdigest()


class RedundancyDispatcher:
    """Coordinator-side redundancy voting engine.

    In-memory tracker: registers tasks that need N results, collects
    results as they arrive, and votes when quorum is reached. The
    actual multi-worker claim protocol on iroh-docs is a future
    layer — this phase delivers the voting primitive.
    """

    def __init__(self) -> None:
        self._factors: dict[str, int] = {}
        self._results: dict[str, list[tuple[str, str]]] = {}
        self._quarantined: dict[str, set[str]] = {}

    def register_task(self, task_id: str, factor: int) -> None:
        """Register a task for redundancy tracking."""
        self._factors[task_id] = factor
        self._results.setdefault(task_id, [])

    def is_redundant(self, task_id: str) -> bool:
        """True if the task is registered with factor > 1."""
        return self._factors.get(task_id, 1) > 1

    def collect_result(
        self,
        task_id: str,
        worker_id: str,
        result_bytes: bytes,
    ) -> VoteOutcome | None:
        """Store a worker's result. Returns a VoteOutcome when enough
        results have arrived (>= factor), else None."""
        h = hash_result_bytes(result_bytes)
        results = self._results.setdefault(task_id, [])
        results.append((worker_id, h))
        factor = self._factors.get(task_id, 1)
        if len(results) >= factor:
            return self.vote(task_id)
        return None

    def vote(self, task_id: str) -> VoteOutcome:
        """Tally results for *task_id* and return the vote outcome."""
        results = self._results.get(task_id, [])
        if not results:
            return VoteOutcome(
                verdict=VoteVerdict.MISMATCH,
                all_hashes={},
                outlier_worker_ids=[],
            )

        hash_counts: dict[str, int] = {}
        for _, h in results:
            hash_counts[h] = hash_counts.get(h, 0) + 1

        all_hashes = {wid: h for wid, h in results}
        total = len(results)
        majority_threshold = total // 2 + 1

        best_hash = max(hash_counts, key=lambda k: hash_counts[k])
        best_count = hash_counts[best_hash]

        if best_count >= majority_threshold:
            outliers = [wid for wid, h in results if h != best_hash]
            _log.info(
                "redundancy vote: majority",
                task_id=task_id,
                canonical_hash=best_hash,
                total=total,
                majority=best_count,
                outliers=len(outliers),
            )
            return VoteOutcome(
                verdict=VoteVerdict.MAJORITY,
                canonical_hash=best_hash,
                all_hashes=all_hashes,
                outlier_worker_ids=outliers,
            )

        all_worker_ids = [wid for wid, _ in results]
        _log.warning(
            "redundancy vote: mismatch",
            task_id=task_id,
            total=total,
            distinct_hashes=len(hash_counts),
        )
        return VoteOutcome(
            verdict=VoteVerdict.MISMATCH,
            all_hashes=all_hashes,
            outlier_worker_ids=all_worker_ids,
        )

    def quarantine_outliers(self, task_id: str, worker_ids: list[str]) -> None:
        """Record outlier workers for *task_id*."""
        q = self._quarantined.setdefault(task_id, set())
        for wid in worker_ids:
            q.add(wid)
            _log.info(
                "worker quarantined for vote outlier",
                task_id=task_id,
                worker_id=wid,
            )

    def get_quarantined(self, task_id: str) -> set[str]:
        """Return the set of quarantined worker IDs for *task_id*."""
        return self._quarantined.get(task_id, set())
