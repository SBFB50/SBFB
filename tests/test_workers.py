"""
Tests for representative reactive workers.

Tests 5 workers end-to-end with real EventBus and mocked DB/LLM:
- EntityExtractorWorker: emits ENTITY_DISCOVERED per entity
- SummarizerWorker: bridge that emits EVIDENCE_PROCESSED
- AlertWorker: sink that creates DB alerts
"""

import asyncio
from unittest.mock import AsyncMock, MagicMock

import pytest

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.workers.entity_extractor import EntityExtractorWorker
from nexus.events.workers.summarizer import SummarizerWorker
from nexus.events.workers.alert import AlertWorker


def _make_event(
    event_type: EventType = EventType.EVIDENCE_ADDED,
    case_id: str = "case-1",
    **kwargs,
) -> NexusEvent:
    return NexusEvent(event_type=event_type, case_id=case_id, **kwargs)


# ===================================================================
# TestEntityExtractorWorker
# ===================================================================

class TestEntityExtractorWorker:

    @pytest.mark.asyncio
    async def test_emits_entity_discovered_per_mention(self, bus):
        db = AsyncMock()
        db.list_mentions_by_evidence = AsyncMock(return_value=[
            {"entity_id": "e-1"},
            {"entity_id": "e-2"},
        ])
        db.get_entity = AsyncMock(side_effect=[
            {"id": "e-1", "name": "John Doe", "entity_type": "personne", "description": "suspect"},
            {"id": "e-2", "name": "Paris", "entity_type": "lieu", "description": "city"},
        ])

        worker = EntityExtractorWorker(bus, db)
        event = _make_event(payload={"evidence_id": "ev-1"})
        output = await worker.handle(event)

        assert len(output) == 2
        assert output[0].event_type == EventType.ENTITY_DISCOVERED
        assert output[0].payload["entity_id"] == "e-1"
        assert output[0].payload["name"] == "John Doe"
        assert output[1].payload["entity_id"] == "e-2"

    @pytest.mark.asyncio
    async def test_no_mentions_returns_empty(self, bus):
        db = AsyncMock()
        db.list_mentions_by_evidence = AsyncMock(return_value=[])

        worker = EntityExtractorWorker(bus, db)
        output = await worker.handle(_make_event(payload={"evidence_id": "ev-1"}))
        assert output == []

    @pytest.mark.asyncio
    async def test_idempotency_skips_duplicate(self, bus):
        db = AsyncMock()
        db.list_mentions_by_evidence = AsyncMock(return_value=[
            {"entity_id": "e-1"},
        ])
        db.get_entity = AsyncMock(return_value={
            "id": "e-1", "name": "John", "entity_type": "personne", "description": "",
        })

        worker = EntityExtractorWorker(bus, db)
        event = _make_event(payload={"evidence_id": "ev-1"})

        output1 = await worker.handle(event)
        output2 = await worker.handle(event)

        assert len(output1) == 1
        assert len(output2) == 0  # Idempotency guard

    @pytest.mark.asyncio
    async def test_dedup_entities_within_evidence(self, bus):
        db = AsyncMock()
        db.list_mentions_by_evidence = AsyncMock(return_value=[
            {"entity_id": "e-1"},
            {"entity_id": "e-1"},  # Duplicate mention
            {"entity_id": "e-1"},
        ])
        db.get_entity = AsyncMock(return_value={
            "id": "e-1", "name": "John", "entity_type": "personne", "description": "",
        })

        worker = EntityExtractorWorker(bus, db)
        output = await worker.handle(_make_event(payload={"evidence_id": "ev-1"}))

        # Only one ENTITY_DISCOVERED for e-1 despite 3 mentions
        assert len(output) == 1

    @pytest.mark.asyncio
    async def test_missing_evidence_id_returns_empty(self, bus):
        db = AsyncMock()
        worker = EntityExtractorWorker(bus, db)
        output = await worker.handle(_make_event(payload={}))
        assert output == []


# ===================================================================
# TestSummarizerWorker
# ===================================================================

class TestSummarizerWorker:

    @pytest.mark.asyncio
    async def test_emits_evidence_processed(self, bus):
        db = AsyncMock()
        db.get_evidence = AsyncMock(return_value={
            "id": "ev-1", "status": "processed", "summary": "A witness statement",
            "title": "Witness A", "evidence_type": "testimony",
        })

        worker = SummarizerWorker(bus, db)
        event = _make_event(payload={"evidence_id": "ev-1"})
        output = await worker.handle(event)

        assert len(output) == 1
        assert output[0].event_type == EventType.EVIDENCE_PROCESSED
        assert output[0].payload["evidence_id"] == "ev-1"
        assert output[0].payload["has_summary"] is True

    @pytest.mark.asyncio
    async def test_skips_unprocessed_evidence(self, bus):
        db = AsyncMock()
        db.get_evidence = AsyncMock(return_value={
            "id": "ev-1", "status": "pending", "summary": "",
            "title": "Witness A", "evidence_type": "testimony",
        })

        worker = SummarizerWorker(bus, db)
        output = await worker.handle(_make_event(payload={"evidence_id": "ev-1"}))
        assert output == []

    @pytest.mark.asyncio
    async def test_idempotency_guard(self, bus):
        db = AsyncMock()
        db.get_evidence = AsyncMock(return_value={
            "id": "ev-1", "status": "processed", "summary": "text",
            "title": "E1", "evidence_type": "text",
        })

        worker = SummarizerWorker(bus, db)
        event = _make_event(payload={"evidence_id": "ev-1"})

        out1 = await worker.handle(event)
        out2 = await worker.handle(event)

        assert len(out1) == 1
        assert len(out2) == 0

    @pytest.mark.asyncio
    async def test_missing_evidence_returns_empty(self, bus):
        db = AsyncMock()
        db.get_evidence = AsyncMock(return_value=None)

        worker = SummarizerWorker(bus, db)
        output = await worker.handle(_make_event(payload={"evidence_id": "ev-missing"}))
        assert output == []


# ===================================================================
# TestAlertWorker
# ===================================================================

class TestAlertWorker:

    @pytest.mark.asyncio
    async def test_contradiction_creates_alert(self, bus):
        db = AsyncMock()
        worker = AlertWorker(bus, db)

        # Mock the AlertManager
        mock_mgr = AsyncMock()
        worker._manager = mock_mgr

        event = _make_event(
            event_type=EventType.CONTRADICTION_FOUND,
            payload={
                "description": "Timeline conflict",
                "evidence_1_title": "Witness A",
                "evidence_2_title": "Witness B",
            },
        )
        output = await worker.handle(event)

        assert output == []  # Alert worker is a sink
        mock_mgr.create_contradiction_alert.assert_called_once()

    @pytest.mark.asyncio
    async def test_suspect_high_score_creates_alert(self, bus):
        db = AsyncMock()
        worker = AlertWorker(bus, db)
        mock_mgr = AsyncMock()
        worker._manager = mock_mgr

        event = _make_event(
            event_type=EventType.SUSPECT_SCORED,
            payload={"name": "John Doe", "score": 85.0, "suspect_id": "s-1", "factors": {}},
        )
        await worker.handle(event)

        mock_mgr.create_alert.assert_called_once()
        call_args = mock_mgr.create_alert.call_args
        assert call_args[1]["severity"] == "critical"  # score >= 80

    @pytest.mark.asyncio
    async def test_suspect_low_score_no_alert(self, bus):
        db = AsyncMock()
        worker = AlertWorker(bus, db)
        mock_mgr = AsyncMock()
        worker._manager = mock_mgr

        event = _make_event(
            event_type=EventType.SUSPECT_SCORED,
            payload={"name": "Jane", "score": 30.0},
        )
        await worker.handle(event)

        mock_mgr.create_alert.assert_not_called()

    @pytest.mark.asyncio
    async def test_forensic_creates_info_alert(self, bus):
        db = AsyncMock()
        worker = AlertWorker(bus, db)
        mock_mgr = AsyncMock()
        worker._manager = mock_mgr

        event = _make_event(
            event_type=EventType.FORENSIC_RESULT,
            payload={"analysis_type": "blood_pattern", "evidence_id": "ev-1"},
        )
        await worker.handle(event)

        mock_mgr.create_alert.assert_called_once()
        assert mock_mgr.create_alert.call_args[1]["severity"] == "info"
