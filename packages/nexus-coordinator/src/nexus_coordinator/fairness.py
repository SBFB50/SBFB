# SPDX-License-Identifier: AGPL-3.0-or-later
"""Fairness observability metrics — pure math, no DB access.

Sprint 23 Phase E (D4). Three functions compute distribution
metrics from a list of per-worker contributions:

- :func:`compute_gini` — Gini coefficient (0 = perfect equality,
  1 = maximum inequality).
- :func:`compute_top_k_share` — fraction of total held by top k%.
- :func:`compute_churn_rate` — fraction of workers active in the
  previous window that are absent in the current window.
"""

from __future__ import annotations


def compute_gini(contributions: list[float]) -> float:
    """Gini coefficient of the contribution distribution.

    Empty or uniform lists return 0.0. Single-element lists return
    0.0 (trivially equal).
    """
    n = len(contributions)
    if n <= 1:
        return 0.0
    total = sum(contributions)
    if total == 0.0:
        return 0.0
    sorted_c = sorted(contributions)
    weighted_sum = 0.0
    for i, c in enumerate(sorted_c):
        weighted_sum += (2 * (i + 1) - n - 1) * c
    return weighted_sum / (n * total)


def compute_top_k_share(contributions: list[float], k: int = 5) -> float:
    """Fraction of total contributions held by the top ``k`` percent.

    Returns 0.0 if there are no contributions or total is 0.
    """
    if not contributions:
        return 0.0
    total = sum(contributions)
    if total == 0.0:
        return 0.0
    count = max(1, len(contributions) * k // 100)
    top = sorted(contributions, reverse=True)[:count]
    return sum(top) / total


def compute_churn_rate(
    previous_workers: set[str],
    current_workers: set[str],
) -> float:
    """Fraction of workers from the previous window absent in the current window.

    Returns 0.0 if the previous window is empty (no baseline to churn from).
    """
    if not previous_workers:
        return 0.0
    departed = previous_workers - current_workers
    return len(departed) / len(previous_workers)
