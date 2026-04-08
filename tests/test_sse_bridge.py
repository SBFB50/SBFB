"""
Tests for the SSE bridge (nexus/events/sse_bridge.py).

Covers streaming, case_id filtering, unsubscribe on close, and
multi-client scenarios.
"""

import asyncio
import json

import pytest

from nexus.events.bus import EventBus
from nexus.events.sse_bridge import SSEBridge
from nexus.events.types import EventType, NexusEvent


def _make_event(
    event_type: EventType = EventType.EVIDENCE_ADDED,
    case_id: str = "case-1",
    **kwargs,
) -> NexusEvent:
    return NexusEvent(event_type=event_type, case_id=case_id, **kwargs)


async def _publish_after(bus: EventBus, event: NexusEvent, delay: float = 0.05):
    """Publish an event after a short delay, giving the generator time to subscribe."""
    await asyncio.sleep(delay)
    await bus.publish(event)


# ===================================================================
# TestSSEBridge
# ===================================================================

class TestSSEBridge:

    @pytest.mark.asyncio
    async def test_stream_receives_published_event(self, bus):
        bridge = SSEBridge(bus)
        gen = bridge.stream([EventType.EVIDENCE_ADDED], case_id="case-1")

        event = _make_event(payload={"evidence_id": "ev-1"})
        task = asyncio.create_task(_publish_after(bus, event))

        sse_msg = await gen.__anext__()
        await task

        assert sse_msg["event"] == "evidence_added"
        assert sse_msg["id"] == event.event_id
        data = json.loads(sse_msg["data"])
        assert data["case_id"] == "case-1"
        assert data["payload"]["evidence_id"] == "ev-1"

        await gen.aclose()

    @pytest.mark.asyncio
    async def test_stream_filters_by_case_id(self, bus):
        bridge = SSEBridge(bus)
        gen = bridge.stream([EventType.EVIDENCE_ADDED], case_id="case-1")

        async def publish_both():
            await asyncio.sleep(0.05)
            # Publish for wrong case first
            await bus.publish(_make_event(case_id="case-2"))
            # Then matching case
            await bus.publish(_make_event(case_id="case-1", payload={"match": True}))

        task = asyncio.create_task(publish_both())
        sse_msg = await gen.__anext__()
        await task

        data = json.loads(sse_msg["data"])
        assert data["payload"]["match"] is True

        await gen.aclose()

    @pytest.mark.asyncio
    async def test_stream_no_case_filter_receives_all(self, bus):
        bridge = SSEBridge(bus)
        gen = bridge.stream([EventType.EVIDENCE_ADDED])  # No case_id filter

        async def publish_two():
            await asyncio.sleep(0.05)
            await bus.publish(_make_event(case_id="case-1"))
            await bus.publish(_make_event(case_id="case-2"))

        task = asyncio.create_task(publish_two())

        msg1 = await gen.__anext__()
        msg2 = await gen.__anext__()
        await task

        cases = {json.loads(msg1["data"])["case_id"], json.loads(msg2["data"])["case_id"]}
        assert cases == {"case-1", "case-2"}

        await gen.aclose()

    @pytest.mark.asyncio
    async def test_stream_unsubscribes_on_close(self, bus):
        bridge = SSEBridge(bus)
        gen = bridge.stream([EventType.EVIDENCE_ADDED], case_id="case-1")

        # Read one event to activate the generator
        task = asyncio.create_task(_publish_after(bus, _make_event()))
        await gen.__anext__()
        await task

        stats_before = bus.get_stats()
        count_before = stats_before["subscriber_count"].get("evidence_added", 0)

        # Close the generator — triggers finally -> unsubscribe
        await gen.aclose()

        stats_after = bus.get_stats()
        count_after = stats_after["subscriber_count"].get("evidence_added", 0)
        assert count_after < count_before

    @pytest.mark.asyncio
    async def test_multiple_clients_receive_same_event(self, bus):
        bridge = SSEBridge(bus)
        gen1 = bridge.stream([EventType.EVIDENCE_ADDED], case_id="case-1")
        gen2 = bridge.stream([EventType.EVIDENCE_ADDED], case_id="case-1")

        event = _make_event(payload={"shared": True})

        # Both generators must start (subscribe) before the event is published.
        # Use gather so both __anext__ calls run concurrently, then publish
        # with a delay so they have time to subscribe.
        async def start_and_publish():
            await asyncio.sleep(0.1)
            await bus.publish(event)

        pub_task = asyncio.create_task(start_and_publish())
        msg1, msg2 = await asyncio.gather(gen1.__anext__(), gen2.__anext__())
        await pub_task

        assert msg1["id"] == event.event_id
        assert msg2["id"] == event.event_id

        await gen1.aclose()
        await gen2.aclose()

    @pytest.mark.asyncio
    async def test_stream_stops_on_bus_shutdown(self, bus):
        bridge = SSEBridge(bus)
        gen = bridge.stream([EventType.EVIDENCE_ADDED], case_id="case-1")

        # Publish and consume one event
        task = asyncio.create_task(_publish_after(bus, _make_event()))
        await gen.__anext__()
        await task

        # Stop the bus — sends sentinel None to all subscribers
        await bus.stop()

        # Generator should exit
        with pytest.raises(StopAsyncIteration):
            await gen.__anext__()

    @pytest.mark.asyncio
    async def test_stream_only_subscribed_types(self, bus):
        bridge = SSEBridge(bus)
        gen = bridge.stream([EventType.ENTITY_DISCOVERED], case_id="case-1")

        async def publish_both_types():
            await asyncio.sleep(0.05)
            # Wrong type — should not be received
            await bus.publish(_make_event(event_type=EventType.EVIDENCE_ADDED))
            # Correct type
            await bus.publish(_make_event(
                event_type=EventType.ENTITY_DISCOVERED,
                payload={"entity_id": "e-1"},
            ))

        task = asyncio.create_task(publish_both_types())
        msg = await gen.__anext__()
        await task

        assert msg["event"] == "entity_discovered"

        await gen.aclose()

    @pytest.mark.asyncio
    async def test_sse_data_is_valid_json(self, bus):
        bridge = SSEBridge(bus)
        gen = bridge.stream([EventType.EVIDENCE_ADDED], case_id="case-1")

        event = _make_event(
            payload={"text": "Témoin avec des caractères spéciaux: é, à, ü"},
            source_worker="test_worker",
        )
        task = asyncio.create_task(_publish_after(bus, event))

        msg = await gen.__anext__()
        await task

        data = json.loads(msg["data"])
        assert data["source_worker"] == "test_worker"
        assert "é" in data["payload"]["text"]

        await gen.aclose()
