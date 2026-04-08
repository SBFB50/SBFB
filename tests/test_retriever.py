"""
Tests for the InvestigationRetriever (nexus/core/retriever.py).

Focuses on reranking, deduplication, recency scoring, and FTS query
sanitization — pure computation that doesn't need external services.
"""

from datetime import datetime, timedelta, timezone
from unittest.mock import AsyncMock, MagicMock

import pytest

from nexus.core.retriever import (
    InvestigationRetriever,
    _SEMANTIC_WEIGHT,
    _GRAPH_WEIGHT,
    _FTS_WEIGHT,
    _RECENCY_WEIGHT,
    _RECENCY_HORIZON_DAYS,
)


def _make_retriever(db=None):
    """Create a retriever with all external deps mocked."""
    return InvestigationRetriever(
        chroma=None,
        neo4j=None,
        router=MagicMock(),
        db=db or AsyncMock(),
    )


def _make_chunk(
    evidence_id="ev-1",
    text="some chunk text",
    semantic=0.0,
    graph=0.0,
    fts=0.0,
    recency=0.0,
    source="test",
):
    return {
        "chunk_text": text,
        "evidence_id": evidence_id,
        "title": f"Title {evidence_id}",
        "source": source,
        "metadata": {},
        "_semantic_score": semantic,
        "_graph_score": graph,
        "_fts_score": fts,
        "_recency_score": recency,
        "_source": source,
    }


# ===================================================================
# TestReranking
# ===================================================================

class TestReranking:

    def test_rerank_by_composite_score(self):
        r = _make_retriever()
        chunks = [
            _make_chunk("ev-1", semantic=0.9),
            _make_chunk("ev-2", graph=0.9),
            _make_chunk("ev-3", semantic=0.5, graph=0.5),
        ]
        ranked = r._rerank(chunks, n=3)

        # ev-1: 0.9*0.50 = 0.45
        # ev-2: 0.9*0.25 = 0.225
        # ev-3: 0.5*0.50 + 0.5*0.25 = 0.375
        assert ranked[0]["evidence_id"] == "ev-1"
        assert ranked[1]["evidence_id"] == "ev-3"
        assert ranked[2]["evidence_id"] == "ev-2"

    def test_rerank_limits_to_n(self):
        r = _make_retriever()
        chunks = [_make_chunk(f"ev-{i}", semantic=1.0 - i * 0.1) for i in range(10)]
        ranked = r._rerank(chunks, n=3)
        assert len(ranked) == 3

    def test_rerank_weights_applied(self):
        r = _make_retriever()
        chunk = _make_chunk(semantic=1.0, graph=1.0, fts=1.0, recency=1.0)
        ranked = r._rerank([chunk], n=1)
        expected = _SEMANTIC_WEIGHT + _GRAPH_WEIGHT + _FTS_WEIGHT + _RECENCY_WEIGHT
        assert abs(ranked[0]["_composite_score"] - expected) < 0.01

    def test_rerank_empty_list(self):
        r = _make_retriever()
        assert r._rerank([], n=5) == []


# ===================================================================
# TestDeduplication
# ===================================================================

class TestDeduplication:

    def test_dedup_merges_same_evidence(self):
        r = _make_retriever()
        chunks = [
            _make_chunk("ev-1", text="same text", semantic=0.8),
            _make_chunk("ev-1", text="same text", graph=0.9),
        ]
        deduped = r._deduplicate(chunks)
        assert len(deduped) == 1
        assert deduped[0]["_semantic_score"] == 0.8
        assert deduped[0]["_graph_score"] == 0.9
        assert deduped[0]["_source"] == "hybrid"

    def test_dedup_keeps_different_evidence(self):
        r = _make_retriever()
        chunks = [
            _make_chunk("ev-1", text="text A"),
            _make_chunk("ev-2", text="text B"),
        ]
        deduped = r._deduplicate(chunks)
        assert len(deduped) == 2

    def test_dedup_different_text_same_evidence(self):
        r = _make_retriever()
        chunks = [
            _make_chunk("ev-1", text="first chunk about cats"),
            _make_chunk("ev-1", text="second chunk about dogs"),
        ]
        deduped = r._deduplicate(chunks)
        assert len(deduped) == 2


# ===================================================================
# TestRecencyScoring
# ===================================================================

class TestRecencyScoring:

    def test_recent_item_high_score(self):
        now = datetime.now(timezone.utc).isoformat()
        score = InvestigationRetriever._compute_recency_score({"created_at": now})
        assert score > 0.9

    def test_old_item_zero_score(self):
        old = (datetime.now(timezone.utc) - timedelta(days=_RECENCY_HORIZON_DAYS + 10)).isoformat()
        score = InvestigationRetriever._compute_recency_score({"created_at": old})
        assert score == 0.0

    def test_midpoint_item(self):
        mid = (datetime.now(timezone.utc) - timedelta(days=_RECENCY_HORIZON_DAYS // 2)).isoformat()
        score = InvestigationRetriever._compute_recency_score({"created_at": mid})
        assert 0.3 < score < 0.7

    def test_no_timestamp_zero_score(self):
        assert InvestigationRetriever._compute_recency_score({}) == 0.0

    def test_invalid_timestamp_zero_score(self):
        assert InvestigationRetriever._compute_recency_score({"created_at": "not-a-date"}) == 0.0


# ===================================================================
# TestFTSQuerySanitization
# ===================================================================

class TestFTSQuerySanitization:

    def test_sanitize_wraps_words_in_quotes(self):
        result = InvestigationRetriever._sanitize_fts_query("hello world")
        assert result == '"hello" "world"'

    def test_sanitize_strips_fts_operators(self):
        result = InvestigationRetriever._sanitize_fts_query("hello AND world OR NOT")
        assert "AND" not in result
        assert "OR" not in result
        assert "NOT" not in result
        assert '"hello"' in result
        assert '"world"' in result

    def test_sanitize_strips_special_chars(self):
        result = InvestigationRetriever._sanitize_fts_query('test"quote*star^caret')
        assert '"' not in result.replace('"', '')  # only wrapping quotes
        assert "*" not in result
        assert "^" not in result

    def test_sanitize_empty_query(self):
        assert InvestigationRetriever._sanitize_fts_query("") == ""

    def test_sanitize_short_words_filtered(self):
        result = InvestigationRetriever._sanitize_fts_query("a b hello")
        assert '"hello"' in result
        # Single-char words should be filtered out
        assert result.count('"') == 2  # only "hello"
