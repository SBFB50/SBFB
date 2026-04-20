# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 23 Phase E — diagnostic endpoints for fairness observability.

Exposes ``GET /diagnostic/fairness`` under the same loopback bearer
auth middleware enforced app-wide (Sprint 16). Returns Gini
coefficient, top-5% compute share, and churn rate derived from the
kudos ledger.

No new wire format — these are local diagnostic endpoints for the
operator shell, not P2P-visible data.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import aiosqlite
import structlog
from fastapi import APIRouter, Request

from nexus_coordinator.fairness import compute_churn_rate, compute_gini, compute_top_k_share

if TYPE_CHECKING:
    from nexus_coordinator.coordinator import Coordinator

_log = structlog.get_logger(__name__)

router = APIRouter()


def _coordinator(request: Request) -> "Coordinator":
    return request.app.state.coordinator  # type: ignore[no-any-return]


async def _worker_contributions(db_path: str) -> list[float]:
    """Query per-worker total kudos from the ledger DB."""
    async with aiosqlite.connect(db_path) as db:
        async with db.execute("SELECT COALESCE(SUM(amount), 0.0) FROM kudos_ledger GROUP BY worker_pubkey") as cursor:
            rows = await cursor.fetchall()
    return [float(row[0]) for row in rows]


async def _active_workers(db_path: str, since_epoch: float) -> set[str]:
    """Return hex pubkeys of workers with ledger entries since ``since_epoch``."""
    async with aiosqlite.connect(db_path) as db:
        async with db.execute(
            "SELECT DISTINCT hex(worker_pubkey) FROM kudos_ledger WHERE awarded_at >= ?",
            (since_epoch,),
        ) as cursor:
            rows = await cursor.fetchall()
    return {row[0].lower() for row in rows}


@router.get("/diagnostic/fairness")
async def fairness_metrics(request: Request) -> dict:
    """Return fairness metrics derived from the kudos ledger.

    Response shape::

        {
            "gini": 0.42,
            "top_5_pct_share": 0.65,
            "churn_rate": 0.12,
            "worker_count": 25
        }

    Returns zeroed metrics if no kudos ledger is available.
    """
    coord = _coordinator(request)
    if coord.kudos_ledger is None:
        return {
            "gini": 0.0,
            "top_5_pct_share": 0.0,
            "churn_rate": 0.0,
            "worker_count": 0,
        }

    db_path = str(coord.kudos_ledger._db_path)
    contributions = await _worker_contributions(db_path)

    import time

    now = time.time()
    day_seconds = 86400
    current_workers = await _active_workers(db_path, now - day_seconds)
    previous_workers = await _active_workers(db_path, now - 2 * day_seconds)

    return {
        "gini": round(compute_gini(contributions), 4),
        "top_5_pct_share": round(compute_top_k_share(contributions, k=5), 4),
        "churn_rate": round(compute_churn_rate(previous_workers, current_workers), 4),
        "worker_count": len(contributions),
    }
