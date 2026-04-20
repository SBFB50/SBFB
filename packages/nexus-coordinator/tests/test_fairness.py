# SPDX-License-Identifier: AGPL-3.0-or-later
"""Tests for fairness.py — pure math, no fixtures needed."""

from nexus_coordinator.fairness import compute_churn_rate, compute_gini, compute_top_k_share


class TestGini:
    def test_equal_distribution(self) -> None:
        assert compute_gini([10.0, 10.0, 10.0, 10.0]) == 0.0

    def test_maximum_inequality(self) -> None:
        # One person has everything, rest have zero
        contributions = [0.0] * 99 + [100.0]
        gini = compute_gini(contributions)
        assert gini > 0.98

    def test_realistic_distribution(self) -> None:
        contributions = [1.0, 2.0, 3.0, 4.0, 5.0, 50.0]
        gini = compute_gini(contributions)
        assert 0.3 < gini < 0.8

    def test_empty_list(self) -> None:
        assert compute_gini([]) == 0.0

    def test_single_element(self) -> None:
        assert compute_gini([42.0]) == 0.0

    def test_all_zeros(self) -> None:
        assert compute_gini([0.0, 0.0, 0.0]) == 0.0


class TestTopKShare:
    def test_top_5_of_100(self) -> None:
        # 95 workers contribute 1.0 each, 5 workers contribute 100.0 each
        contributions = [1.0] * 95 + [100.0] * 5
        share = compute_top_k_share(contributions, k=5)
        total = 95 + 500
        assert abs(share - 500 / total) < 0.01

    def test_empty(self) -> None:
        assert compute_top_k_share([]) == 0.0

    def test_all_zeros(self) -> None:
        assert compute_top_k_share([0.0, 0.0]) == 0.0

    def test_single_worker(self) -> None:
        assert compute_top_k_share([100.0], k=5) == 1.0


class TestChurnRate:
    def test_no_churn(self) -> None:
        prev = {"a", "b", "c"}
        curr = {"a", "b", "c", "d"}
        assert compute_churn_rate(prev, curr) == 0.0

    def test_full_churn(self) -> None:
        prev = {"a", "b", "c"}
        curr = {"d", "e", "f"}
        assert compute_churn_rate(prev, curr) == 1.0

    def test_partial_churn(self) -> None:
        prev = {"a", "b", "c", "d"}
        curr = {"a", "b", "e"}
        rate = compute_churn_rate(prev, curr)
        assert abs(rate - 0.5) < 0.01

    def test_empty_previous(self) -> None:
        assert compute_churn_rate(set(), {"a"}) == 0.0
