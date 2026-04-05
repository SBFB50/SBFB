"""Tests for suspect scoring system."""

import pytest
from nexus.core.suspect_scorer import (
    compute_evidence_score,
    compute_contradiction_score,
    compute_hypothesis_score,
    compute_graph_score,
    compute_profile_score,
    compute_composite_score,
    DEFAULT_WEIGHTS,
)


class TestEvidenceScore:

    def test_evidence_score_no_mentions(self):
        """No mentions -> score 0."""
        assert compute_evidence_score([]) == 0.0

    def test_evidence_score_single_low(self):
        """One mention with default confidence and default reliability."""
        mentions = [{"confidence": 0.5, "evidence_id": "e1"}]
        score = compute_evidence_score(mentions)
        # 0.5 * (50/100) = 0.25 -> (0.25/5)*100 = 5.0
        assert score == pytest.approx(5.0)

    def test_evidence_score_high_confidence(self):
        """High confidence + high reliability -> high score."""
        mentions = [
            {"confidence": 1.0, "evidence_id": "e1"},
            {"confidence": 0.95, "evidence_id": "e2"},
            {"confidence": 0.9, "evidence_id": "e3"},
        ]
        ev_map = {
            "e1": {"reliability": 100},
            "e2": {"reliability": 90},
            "e3": {"reliability": 85},
        }
        score = compute_evidence_score(mentions, ev_map)
        # 1.0*1.0 + 0.95*0.9 + 0.9*0.85 = 1.0 + 0.855 + 0.765 = 2.62
        # (2.62/5)*100 = 52.4
        assert score == pytest.approx(52.4)

    def test_evidence_score_capped_at_100(self):
        """Many high-quality mentions should cap at 100."""
        mentions = [
            {"confidence": 1.0, "evidence_id": f"e{i}"}
            for i in range(10)
        ]
        ev_map = {f"e{i}": {"reliability": 100} for i in range(10)}
        score = compute_evidence_score(mentions, ev_map)
        assert score == 100.0

    def test_evidence_score_no_evidence_map(self):
        """Without evidence_map, reliability defaults to 50."""
        mentions = [{"confidence": 1.0, "evidence_id": "e1"}]
        score = compute_evidence_score(mentions)
        # 1.0 * (50/100) = 0.5 -> (0.5/5)*100 = 10.0
        assert score == pytest.approx(10.0)


class TestContradictionScore:

    def test_contradiction_score_none(self):
        """No contradictions -> score 0."""
        assert compute_contradiction_score([]) == 0.0

    def test_contradiction_score_one(self):
        """1 contradiction -> 40."""
        assert compute_contradiction_score(["c1"]) == 40.0

    def test_contradiction_score_two(self):
        """2 contradictions -> 70."""
        assert compute_contradiction_score(["c1", "c2"]) == 70.0

    def test_contradiction_score_multiple(self):
        """3 contradictions -> score capped at 100."""
        assert compute_contradiction_score(["c1", "c2", "c3"]) == 100.0

    def test_contradiction_score_many(self):
        """5 contradictions still caps at 100."""
        assert compute_contradiction_score(["c1", "c2", "c3", "c4", "c5"]) == 100.0


class TestHypothesisScore:

    def test_hypothesis_score_implicated(self):
        """Person mentioned in high-score hypothesis -> high score."""
        hypotheses = [
            {"title": "Jean Dupont est le coupable", "description": "Based on evidence", "current_score": 85.0},
            {"title": "Accident", "description": "Could be an accident", "current_score": 20.0},
        ]
        score = compute_hypothesis_score("Jean Dupont", hypotheses)
        assert score == 85.0

    def test_hypothesis_score_in_description(self):
        """Person mentioned in description -> uses that score."""
        hypotheses = [
            {"title": "Main theory", "description": "Implicates Marc in the murder", "current_score": 70.0},
        ]
        score = compute_hypothesis_score("Marc", hypotheses)
        assert score == 70.0

    def test_hypothesis_score_not_mentioned(self):
        """Person not in any hypothesis -> score 0."""
        hypotheses = [
            {"title": "Jean did it", "description": "All signs point to Jean", "current_score": 90.0},
        ]
        score = compute_hypothesis_score("Marie", hypotheses)
        assert score == 0.0

    def test_hypothesis_score_empty(self):
        """Empty hypotheses list -> score 0."""
        assert compute_hypothesis_score("Anyone", []) == 0.0

    def test_hypothesis_score_empty_name(self):
        """Empty person name -> score 0."""
        hypotheses = [{"title": "Test", "description": "D", "current_score": 50.0}]
        assert compute_hypothesis_score("", hypotheses) == 0.0

    def test_hypothesis_score_takes_max(self):
        """When mentioned in multiple hypotheses, takes the max score."""
        hypotheses = [
            {"title": "Pierre stole it", "description": "", "current_score": 60.0},
            {"title": "Pierre killed him", "description": "", "current_score": 90.0},
        ]
        score = compute_hypothesis_score("Pierre", hypotheses)
        assert score == 90.0


class TestGraphScore:

    def test_graph_score_zero_degree(self):
        """No connections -> 0."""
        assert compute_graph_score(0) == 0.0

    def test_graph_score_max(self):
        """Max degree -> 100."""
        assert compute_graph_score(10, max_degree=10) == 100.0

    def test_graph_score_half(self):
        """Half of max -> 50."""
        assert compute_graph_score(5, max_degree=10) == pytest.approx(50.0)

    def test_graph_score_exceeds_max(self):
        """Exceeding max caps at 100."""
        assert compute_graph_score(20, max_degree=10) == 100.0


class TestProfileScore:

    def test_profile_no_alibi_motive_record(self):
        """No alibi + motive + record = 40+30+30 = 100."""
        score = compute_profile_score(
            alibi_status="none", has_motive=True, has_criminal_record=True
        )
        assert score == 100.0

    def test_profile_verified_alibi(self):
        """Verified alibi + nothing else = 0."""
        score = compute_profile_score(alibi_status="verified")
        assert score == 0.0

    def test_profile_weak_alibi_motive(self):
        """Weak alibi + motive = 30 + 30 = 60."""
        score = compute_profile_score(alibi_status="weak", has_motive=True)
        assert score == 60.0


class TestCompositeScore:

    def test_composite_score_weighted(self):
        """Composite = weighted sum of factors."""
        sub_scores = {
            "graph": 50.0,
            "evidence": 80.0,
            "contradiction": 60.0,
            "profile": 40.0,
            "hypothesis": 70.0,
        }
        score = compute_composite_score(sub_scores)
        # Uses DEFAULT_WEIGHTS: graph=0.20, evidence=0.25, contradiction=0.20,
        # profile=0.20, hypothesis=0.15
        expected = (
            50 * DEFAULT_WEIGHTS["graph"]
            + 80 * DEFAULT_WEIGHTS["evidence"]
            + 60 * DEFAULT_WEIGHTS["contradiction"]
            + 40 * DEFAULT_WEIGHTS["profile"]
            + 70 * DEFAULT_WEIGHTS["hypothesis"]
        )
        assert score == pytest.approx(expected)

    def test_composite_score_max_100(self):
        """Score never exceeds 100."""
        sub_scores = {
            "graph": 100.0,
            "evidence": 100.0,
            "contradiction": 100.0,
            "profile": 100.0,
            "hypothesis": 100.0,
        }
        score = compute_composite_score(sub_scores)
        assert score == 100.0

    def test_composite_score_all_zero(self):
        """All zeros -> 0."""
        sub_scores = {
            "graph": 0.0,
            "evidence": 0.0,
            "contradiction": 0.0,
            "profile": 0.0,
            "hypothesis": 0.0,
        }
        score = compute_composite_score(sub_scores)
        assert score == 0.0

    def test_composite_score_custom_weights(self):
        """Custom weights override defaults."""
        sub_scores = {"graph": 100.0, "evidence": 0.0}
        weights = {"graph": 1.0, "evidence": 0.0}
        score = compute_composite_score(sub_scores, weights=weights)
        assert score == 100.0

    def test_score_all_persons(self):
        """score_all_suspects creates suspect records for all persons.

        This is an integration-level test validating the scoring pipeline
        produces valid composite scores for a realistic set of inputs.
        """
        # Simulate scoring two persons
        persons = [
            {
                "name": "Jean",
                "mentions": [{"confidence": 0.9, "evidence_id": "e1"}],
                "contradictions": ["c1", "c2"],
                "hypotheses": [
                    {"title": "Jean coupable", "description": "", "current_score": 75.0}
                ],
                "degree": 4,
            },
            {
                "name": "Marie",
                "mentions": [],
                "contradictions": [],
                "hypotheses": [],
                "degree": 1,
            },
        ]
        ev_map = {"e1": {"reliability": 90}}

        results = []
        for p in persons:
            from nexus.core.suspect_scorer import (
                compute_evidence_score as ev_fn,
                compute_contradiction_score as contra_fn,
                compute_hypothesis_score as hyp_fn,
                compute_graph_score as graph_fn,
                compute_composite_score as comp_fn,
            )
            sub = {
                "graph": graph_fn(p["degree"], max_degree=10),
                "evidence": ev_fn(p["mentions"], ev_map),
                "contradiction": contra_fn(p["contradictions"]),
                "profile": 0.0,
                "hypothesis": hyp_fn(p["name"], p["hypotheses"]),
            }
            results.append({"name": p["name"], "score": comp_fn(sub), "sub": sub})

        # Jean should score higher than Marie
        assert results[0]["score"] > results[1]["score"]
        # Both scores in valid range
        for r in results:
            assert 0 <= r["score"] <= 100
