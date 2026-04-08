"""
Tests for the ContradictionDetector (nexus/core/contradiction_detector.py).

Uses mocked LLM router and DB to test pairing logic, deduplication,
severity handling, and edge cases.
"""

import json
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from nexus.core.contradiction_detector import ContradictionDetector


def _make_evidence(eid: str, title: str = "Evidence", summary: str = "Some text"):
    return {
        "id": eid,
        "case_id": "case-1",
        "title": title,
        "summary": summary,
        "raw_text": summary,
        "source": "test",
        "reliability": 80,
    }


def _make_detector(llm_response=None, mentions=None):
    """Create a detector with mocked deps."""
    db = AsyncMock()
    router = AsyncMock()

    # Default LLM response
    if llm_response is None:
        llm_response = json.dumps({"contradictions": []})
    router.route = AsyncMock(return_value=llm_response)

    # Default mentions (no entity overlap)
    if mentions is None:
        db.list_mentions_by_evidence = AsyncMock(return_value=[])
    else:
        db.list_mentions_by_evidence = AsyncMock(side_effect=mentions)

    return ContradictionDetector(db, router), db, router


# ===================================================================
# TestDetectContradictions
# ===================================================================

class TestDetectContradictions:

    @pytest.mark.asyncio
    async def test_detects_contradiction_between_pair(self):
        llm_response = json.dumps({
            "contradictions": [
                {
                    "type": "factual",
                    "description": "Witness A says blue car, witness B says red car",
                    "severity": "high",
                }
            ]
        })
        detector, db, _ = _make_detector(llm_response=llm_response)
        db.list_evidence_by_case = AsyncMock(return_value=[
            _make_evidence("ev-1", "Witness A", "The car was blue"),
            _make_evidence("ev-2", "Witness B", "The car was red"),
        ])

        results = await detector.detect_contradictions("case-1")
        assert len(results) == 1
        assert results[0]["type"] == "factual"
        assert results[0]["evidence_1_id"] == "ev-1"
        assert results[0]["evidence_2_id"] == "ev-2"

    @pytest.mark.asyncio
    async def test_no_contradiction_in_consistent_evidence(self):
        detector, db, _ = _make_detector()
        db.list_evidence_by_case = AsyncMock(return_value=[
            _make_evidence("ev-1", "Report A"),
            _make_evidence("ev-2", "Report B"),
        ])

        results = await detector.detect_contradictions("case-1")
        assert results == []

    @pytest.mark.asyncio
    async def test_empty_evidence_returns_empty(self):
        detector, db, _ = _make_detector()
        db.list_evidence_by_case = AsyncMock(return_value=[])
        results = await detector.detect_contradictions("case-1")
        assert results == []

    @pytest.mark.asyncio
    async def test_single_evidence_returns_empty(self):
        detector, db, _ = _make_detector()
        db.list_evidence_by_case = AsyncMock(return_value=[
            _make_evidence("ev-1"),
        ])
        results = await detector.detect_contradictions("case-1")
        assert results == []

    @pytest.mark.asyncio
    async def test_entity_based_pair_selection(self):
        """Evidence sharing entities should be paired together."""
        detector, db, router = _make_detector()
        db.list_evidence_by_case = AsyncMock(return_value=[
            _make_evidence("ev-1"),
            _make_evidence("ev-2"),
            _make_evidence("ev-3"),
        ])
        # ev-1 and ev-2 share entity "e-1", ev-3 has different entity
        db.list_mentions_by_evidence = AsyncMock(side_effect=[
            [{"entity_id": "e-1"}],  # ev-1
            [{"entity_id": "e-1"}],  # ev-2
            [{"entity_id": "e-99"}],  # ev-3
        ])

        await detector.detect_contradictions("case-1")

        # Only one pair should be analysed (ev-1, ev-2)
        assert router.route.call_count == 1

    @pytest.mark.asyncio
    async def test_max_pairs_limit_respected(self):
        """Should not exceed contradiction_max_evidence_pairs."""
        detector, db, router = _make_detector()
        # Create many evidence items
        evidence_list = [_make_evidence(f"ev-{i}") for i in range(10)]
        db.list_evidence_by_case = AsyncMock(return_value=evidence_list)
        # All share entity "e-1" so all pairs are relevant
        db.list_mentions_by_evidence = AsyncMock(return_value=[{"entity_id": "e-1"}])

        with patch("nexus.core.contradiction_detector.settings") as mock_settings:
            mock_settings.contradiction_max_evidence_pairs = 3
            mock_settings.contradiction_max_fallback_pairs = 3
            mock_settings.text_truncation_short = 2000
            await detector.detect_contradictions("case-1")

        assert router.route.call_count <= 3


# ===================================================================
# TestDeduplication
# ===================================================================

class TestDeduplication:

    def test_dedup_removes_duplicate_pairs(self):
        detector, _, _ = _make_detector()
        contradictions = [
            {"evidence_1_id": "ev-1", "evidence_2_id": "ev-2", "type": "factual", "desc": "A"},
            {"evidence_1_id": "ev-2", "evidence_2_id": "ev-1", "type": "factual", "desc": "B"},
        ]
        deduped = detector._deduplicate_contradictions(contradictions)
        assert len(deduped) == 1

    def test_dedup_keeps_different_types(self):
        detector, _, _ = _make_detector()
        contradictions = [
            {"evidence_1_id": "ev-1", "evidence_2_id": "ev-2", "type": "factual"},
            {"evidence_1_id": "ev-1", "evidence_2_id": "ev-2", "type": "temporal"},
        ]
        deduped = detector._deduplicate_contradictions(contradictions)
        assert len(deduped) == 2


# ===================================================================
# TestHypothesisConsistency
# ===================================================================

class TestHypothesisConsistency:

    @pytest.mark.asyncio
    async def test_hypothesis_consistency_check(self):
        llm_response = json.dumps({
            "contradictions": [
                {"type": "hypothesis_conflict", "description": "H1 and H2 are mutually exclusive"}
            ]
        })
        detector, db, _ = _make_detector(llm_response=llm_response)
        db.list_hypotheses_by_case = AsyncMock(return_value=[
            {"id": "h-1", "title": "H1: suspect A did it", "description": "...", "current_score": 80},
            {"id": "h-2", "title": "H2: suspect B did it", "description": "...", "current_score": 70},
        ])

        results = await detector.check_hypothesis_consistency("case-1")
        assert len(results) == 1
        assert results[0]["hypothesis_a_id"] == "h-1"
        assert results[0]["hypothesis_b_id"] == "h-2"

    @pytest.mark.asyncio
    async def test_single_hypothesis_no_check(self):
        detector, db, _ = _make_detector()
        db.list_hypotheses_by_case = AsyncMock(return_value=[
            {"id": "h-1", "title": "Only one", "description": "...", "current_score": 50},
        ])
        results = await detector.check_hypothesis_consistency("case-1")
        assert results == []
