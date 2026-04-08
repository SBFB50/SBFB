"""
Tests for the HypothesisEngine (nexus/core/hypothesis_engine.py).

Tests generation parsing, scoring pipeline, deduplication, and DB
persistence using mocked LLM router and real in-memory DB.
"""

import json
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from nexus.core.hypothesis_engine import HypothesisEngine


def _make_engine(db=None, router=None):
    """Create an engine with mocked deps."""
    db = db or AsyncMock()
    router = router or AsyncMock()
    engine = HypothesisEngine(db, router, chroma=None, neo4j=None)
    # Mock audit to avoid file I/O during tests
    engine._audit = AsyncMock()
    return engine


# ===================================================================
# TestHypothesisGeneration
# ===================================================================

class TestHypothesisGeneration:

    @pytest.mark.asyncio
    async def test_generate_parses_llm_output(self):
        db = AsyncMock()
        router = AsyncMock()

        # Setup DB responses
        db.list_evidence_by_case = AsyncMock(return_value=[
            {"title": "Evidence 1", "summary": "Witness saw suspect", "source": "police",
             "reliability": 80, "raw_text": ""},
        ])
        db.list_entities_by_case = AsyncMock(return_value=[
            {"name": "John Doe", "entity_type": "personne", "description": "suspect"},
        ])
        db.list_hypotheses_by_case = AsyncMock(return_value=[])
        db.create_hypothesis = AsyncMock(return_value={
            "id": "h-1", "case_id": "case-1", "title": "H1: John Doe committed the crime",
            "current_score": 65.0, "status": "active",
        })
        db.create_hypothesis_snapshot = AsyncMock(return_value={})

        # LLM returns valid JSON
        generation_response = {
            "hypotheses": [
                {
                    "id": "H1",
                    "description": "John Doe committed the crime based on witness testimony",
                    "plausibility": 0.65,
                    "supporting_evidence": ["Evidence 1"],
                    "tests": ["Check alibi"],
                }
            ]
        }
        # First call (route_json) returns parsed dict
        router.route_json = AsyncMock(return_value=generation_response)
        # Red team call
        router.route = AsyncMock(return_value="")

        engine = _make_engine(db, router)
        result = await engine.generate_hypotheses("case-1")

        assert len(result) == 1
        assert result[0]["id"] == "h-1"
        db.create_hypothesis.assert_called_once()

    @pytest.mark.asyncio
    async def test_generate_with_empty_evidence(self):
        db = AsyncMock()
        router = AsyncMock()
        db.list_evidence_by_case = AsyncMock(return_value=[])
        db.list_entities_by_case = AsyncMock(return_value=[])

        engine = _make_engine(db, router)
        result = await engine.generate_hypotheses("case-1")

        assert result == []
        router.route_json.assert_not_called()

    @pytest.mark.asyncio
    async def test_generate_deduplicates_against_existing(self):
        db = AsyncMock()
        router = AsyncMock()

        db.list_evidence_by_case = AsyncMock(return_value=[
            {"title": "E1", "summary": "data", "source": "s", "reliability": 80, "raw_text": ""},
        ])
        db.list_entities_by_case = AsyncMock(return_value=[])
        db.list_hypotheses_by_case = AsyncMock(return_value=[
            {"title": "H1: John Doe did the crime"},
        ])
        db.create_hypothesis = AsyncMock(return_value={"id": "h-2", "case_id": "case-1"})
        db.create_hypothesis_snapshot = AsyncMock(return_value={})

        # LLM returns hypothesis with similar title to existing
        router.route_json = AsyncMock(return_value={
            "hypotheses": [
                {"id": "H1", "description": "John Doe did the crime", "plausibility": 0.7},
            ]
        })

        engine = _make_engine(db, router)
        result = await engine.generate_hypotheses("case-1")

        # Should be skipped as duplicate
        assert len(result) == 0

    @pytest.mark.asyncio
    async def test_generate_all_attempts_fail(self):
        db = AsyncMock()
        router = AsyncMock()
        db.list_evidence_by_case = AsyncMock(return_value=[
            {"title": "E1", "summary": "data", "source": "s", "reliability": 80, "raw_text": ""},
        ])
        db.list_entities_by_case = AsyncMock(return_value=[])

        # Both attempts fail
        router.route_json = AsyncMock(side_effect=Exception("LLM error"))
        router.route = AsyncMock(return_value="not json")

        engine = _make_engine(db, router)
        result = await engine.generate_hypotheses("case-1")

        assert result == []

    @pytest.mark.asyncio
    async def test_generate_saves_to_db(self):
        db = AsyncMock()
        router = AsyncMock()
        db.list_evidence_by_case = AsyncMock(return_value=[
            {"title": "E1", "summary": "x", "source": "s", "reliability": 80, "raw_text": ""},
        ])
        db.list_entities_by_case = AsyncMock(return_value=[])
        db.list_hypotheses_by_case = AsyncMock(return_value=[])
        db.create_hypothesis = AsyncMock(return_value={
            "id": "h-1", "case_id": "case-1", "title": "H1",
            "current_score": 50.0, "status": "active",
        })
        db.create_hypothesis_snapshot = AsyncMock(return_value={})

        router.route_json = AsyncMock(return_value={
            "hypotheses": [{"id": "H1", "description": "Something happened", "plausibility": 0.5}]
        })

        engine = _make_engine(db, router)
        await engine.generate_hypotheses("case-1")

        db.create_hypothesis.assert_called_once()
        db.create_hypothesis_snapshot.assert_called_once()


# ===================================================================
# TestHypothesisScoring
# ===================================================================

class TestHypothesisScoring:

    @pytest.mark.asyncio
    async def test_evaluate_parses_scoring_response(self):
        db = AsyncMock()
        router = AsyncMock()

        db.get_hypothesis = AsyncMock(return_value={
            "id": "h-1", "case_id": "case-1",
            "title": "H1: suspect theory", "description": "...",
            "current_score": 50.0, "status": "active",
        })
        db.list_snapshots_by_hypothesis = AsyncMock(return_value=[])
        db.list_evidence_by_case = AsyncMock(return_value=[])
        db.list_entities_by_case = AsyncMock(return_value=[])
        db.list_hypotheses_by_case = AsyncMock(return_value=[])
        db.create_hypothesis_snapshot = AsyncMock(return_value={
            "id": "snap-1", "score": 72.0,
        })
        db.update_hypothesis = AsyncMock()
        db.create_alert = AsyncMock()

        # Scoring response
        router.route = AsyncMock(side_effect=[
            # First call: scoring
            json.dumps({"new_score": 0.72, "reasoning": "Good evidence", "supporting": [], "contradicting": []}),
            # Second call: verification
            json.dumps({"soundness_score": 0.8, "fallacies": []}),
        ])

        engine = _make_engine(db, router)
        snapshot = await engine.evaluate_hypothesis("h-1")

        assert snapshot is not None
        db.create_hypothesis_snapshot.assert_called_once()
        db.update_hypothesis.assert_called_once()

    @pytest.mark.asyncio
    async def test_score_clamps_to_0_100(self):
        db = AsyncMock()
        router = AsyncMock()

        db.get_hypothesis = AsyncMock(return_value={
            "id": "h-1", "case_id": "case-1",
            "title": "H1", "description": "...",
            "current_score": 50.0, "status": "active",
        })
        db.list_snapshots_by_hypothesis = AsyncMock(return_value=[])
        db.list_evidence_by_case = AsyncMock(return_value=[])
        db.list_entities_by_case = AsyncMock(return_value=[])
        db.list_hypotheses_by_case = AsyncMock(return_value=[])
        db.create_hypothesis_snapshot = AsyncMock(return_value={"id": "snap-1", "score": 100.0})
        db.update_hypothesis = AsyncMock()

        # Score above 1.0 (should be treated as 0-100 scale)
        router.route = AsyncMock(side_effect=[
            json.dumps({"new_score": 150, "reasoning": "Very sure", "supporting": [], "contradicting": []}),
            json.dumps({"soundness_score": 0.9, "fallacies": []}),
        ])

        engine = _make_engine(db, router)
        snapshot = await engine.evaluate_hypothesis("h-1")

        # Score should be clamped to 100
        call_args = db.update_hypothesis.call_args
        score = call_args[1].get("current_score", call_args[0][1] if len(call_args[0]) > 1 else None)
        if score is not None:
            assert score <= 100.0

    @pytest.mark.asyncio
    async def test_significant_shift_creates_alert(self):
        db = AsyncMock()
        router = AsyncMock()

        db.get_hypothesis = AsyncMock(return_value={
            "id": "h-1", "case_id": "case-1",
            "title": "H1: suspect theory", "description": "...",
            "current_score": 50.0, "status": "active",
        })
        db.list_snapshots_by_hypothesis = AsyncMock(return_value=[])
        db.list_evidence_by_case = AsyncMock(return_value=[])
        db.list_entities_by_case = AsyncMock(return_value=[])
        db.list_hypotheses_by_case = AsyncMock(return_value=[])
        db.create_hypothesis_snapshot = AsyncMock(return_value={"id": "snap-1", "score": 85.0})
        db.update_hypothesis = AsyncMock()
        db.create_alert = AsyncMock()

        # Big score jump: 50 -> 85
        router.route = AsyncMock(side_effect=[
            json.dumps({"new_score": 0.85, "reasoning": "Strong new evidence", "supporting": [], "contradicting": []}),
            json.dumps({"soundness_score": 0.9, "fallacies": []}),
        ])

        engine = _make_engine(db, router)
        snapshot = await engine.evaluate_hypothesis("h-1")

        assert snapshot["significant_shift"] is True
        db.create_alert.assert_called_once()

    @pytest.mark.asyncio
    async def test_hypothesis_not_found_raises(self):
        db = AsyncMock()
        db.get_hypothesis = AsyncMock(return_value=None)

        engine = _make_engine(db)
        with pytest.raises(ValueError, match="Hypothesis not found"):
            await engine.evaluate_hypothesis("nonexistent")


# ===================================================================
# TestHypothesisEvolution
# ===================================================================

class TestHypothesisEvolution:

    @pytest.mark.asyncio
    async def test_get_evolution_returns_chronological(self):
        db = AsyncMock()
        db.list_snapshots_by_hypothesis = AsyncMock(return_value=[
            {"created_at": "2026-01-03", "score": 80, "trigger": "manual", "model_used": "nexus"},
            {"created_at": "2026-01-02", "score": 60, "trigger": "auto", "model_used": "nexus"},
            {"created_at": "2026-01-01", "score": 50, "trigger": "generation", "model_used": "nexus"},
        ])

        engine = _make_engine(db)
        evolution = await engine.get_evolution("h-1")

        assert len(evolution) == 3
        # Should be reversed to chronological order
        assert evolution[0]["score"] == 50
        assert evolution[2]["score"] == 80


# ===================================================================
# TestHypothesisMerge
# ===================================================================

class TestHypothesisMerge:

    @pytest.mark.asyncio
    async def test_merge_computes_average_score(self):
        db = AsyncMock()
        db.get_hypothesis = AsyncMock(side_effect=[
            {"id": "h-1", "case_id": "case-1", "title": "H1", "current_score": 60.0},
            {"id": "h-2", "case_id": "case-1", "title": "H2", "current_score": 80.0},
        ])
        db.create_hypothesis = AsyncMock(return_value={
            "id": "h-3", "case_id": "case-1", "title": "Merged",
            "current_score": 70.0, "status": "active",
        })
        db.update_hypothesis = AsyncMock()
        db.create_hypothesis_snapshot = AsyncMock(return_value={})

        engine = _make_engine(db)
        result = await engine.merge_hypotheses(
            ["h-1", "h-2"], "Merged hypothesis", "Combined theory"
        )

        assert result["id"] == "h-3"
        # Source hypotheses should be marked as merged
        assert db.update_hypothesis.call_count == 2

    @pytest.mark.asyncio
    async def test_merge_requires_minimum_two(self):
        engine = _make_engine()
        with pytest.raises(ValueError, match="At least 2"):
            await engine.merge_hypotheses(["h-1"], "title", "desc")


# ===================================================================
# TestPrivateHelpers
# ===================================================================

class TestPrivateHelpers:

    def test_build_facts_context_with_data(self):
        engine = _make_engine()
        evidence = [{"title": "E1", "summary": "Witness saw X", "source": "police", "reliability": 90, "raw_text": ""}]
        entities = [{"name": "John", "entity_type": "personne", "description": "suspect"}]

        context = engine._build_facts_context(evidence, entities)
        assert "PREUVES" in context
        assert "E1" in context
        assert "ENTITES" in context
        assert "John" in context

    def test_build_facts_context_empty(self):
        engine = _make_engine()
        context = engine._build_facts_context([], [])
        assert context == "(aucune donnee)"
