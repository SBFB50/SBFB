"""Sprint 9 Phase C tests for :class:`nexus_sdk.AppEvents`.

The 25 contracts listed in ``.planning/sprint9_plan.md`` §6.2.
"""

from __future__ import annotations

import asyncio
import logging
from datetime import datetime, timezone

import anyio
import pytest
from nexus_sdk import AppEvents, EventEnvelope, EventOverflowPolicy

_RECEIVE_TIMEOUT = 0.5


async def _receive_one(stream) -> EventEnvelope:
    """Pull a single envelope from ``stream`` with a hard timeout
    so a missing dispatch fails the test loudly instead of
    deadlocking the runner."""
    return await asyncio.wait_for(stream.receive(), _RECEIVE_TIMEOUT)


# 1
async def test_publish_without_subscribers_is_noop() -> None:
    bus = AppEvents()
    await bus.publish("party.refreshed", {"count": 0})
    assert bus.stats() == {"subscribers": 0}


# 2
async def test_subscribe_receives_matching_event() -> None:
    bus = AppEvents()
    async with bus.subscribe("party.refreshed") as stream:
        await bus.publish("party.refreshed", {"count": 3})
        envelope = await _receive_one(stream)
        assert envelope.topic == "party.refreshed"
        assert envelope.payload == {"count": 3}


# 3
async def test_subscribe_filters_non_matching_event() -> None:
    bus = AppEvents()
    async with bus.subscribe("party.refreshed") as stream:
        await bus.publish("politician.created", {"id": 1})
        with pytest.raises(asyncio.TimeoutError):
            await _receive_one(stream)


# 4
async def test_glob_pattern_star_matches_single_segment() -> None:
    bus = AppEvents()
    async with bus.subscribe("politician.*") as stream:
        await bus.publish("politician.refreshed", {})
        envelope = await _receive_one(stream)
        assert envelope.topic == "politician.refreshed"


# 5
async def test_glob_pattern_prefix_wildcard() -> None:
    bus = AppEvents()
    async with bus.subscribe("*.refreshed") as stream:
        await bus.publish("party.refreshed", {})
        envelope = await _receive_one(stream)
        assert envelope.topic == "party.refreshed"
        await bus.publish("politician.created", {})
        with pytest.raises(asyncio.TimeoutError):
            await _receive_one(stream)


# 6
async def test_multi_subscribers_receive_fanout() -> None:
    bus = AppEvents()
    async with bus.subscribe("party.refreshed") as stream_a:
        async with bus.subscribe("*.refreshed") as stream_b:
            await bus.publish("party.refreshed", {"count": 7})
            env_a = await _receive_one(stream_a)
            env_b = await _receive_one(stream_b)
            assert env_a.payload == {"count": 7}
            assert env_b.payload == {"count": 7}
            assert env_a.trace_id == env_b.trace_id


# 7
async def test_envelope_has_topic_payload_timestamp_trace_id() -> None:
    bus = AppEvents()
    async with bus.subscribe("party.refreshed") as stream:
        await bus.publish("party.refreshed", {"count": 1})
        envelope = await _receive_one(stream)
        assert envelope.topic == "party.refreshed"
        assert envelope.payload == {"count": 1}
        assert isinstance(envelope.timestamp, datetime)
        assert isinstance(envelope.trace_id, str)
        assert len(envelope.trace_id) == 16


# 8
async def test_envelope_trace_id_is_unique_per_publish() -> None:
    bus = AppEvents()
    async with bus.subscribe("party.refreshed") as stream:
        await bus.publish("party.refreshed", {})
        await bus.publish("party.refreshed", {})
        env_a = await _receive_one(stream)
        env_b = await _receive_one(stream)
        assert env_a.trace_id != env_b.trace_id


# 9
async def test_envelope_timestamp_is_utc_iso8601() -> None:
    bus = AppEvents()
    async with bus.subscribe("party.refreshed") as stream:
        await bus.publish("party.refreshed", {})
        envelope = await _receive_one(stream)
        assert envelope.timestamp.tzinfo is not None
        assert envelope.timestamp.utcoffset() == timezone.utc.utcoffset(None)
        # round-trips through model_dump_json -> ISO 8601
        json_str = envelope.model_dump_json()
        assert "+00:00" in json_str or "Z" in json_str


# 10
async def test_bounded_queue_blocks_when_full_policy_block() -> None:
    bus = AppEvents(buffer_size=1)
    async with bus.subscribe("topic", policy=EventOverflowPolicy.block) as stream:
        await bus.publish("topic", {"i": 0})  # fills the buffer

        async def producer() -> None:
            await bus.publish("topic", {"i": 1})

        task = asyncio.create_task(producer())
        await asyncio.sleep(0.05)
        # the second publish should still be blocked because no
        # one has drained the buffer yet.
        assert not task.done()
        first = await _receive_one(stream)
        assert first.payload == {"i": 0}
        await task
        second = await _receive_one(stream)
        assert second.payload == {"i": 1}


# 11
async def test_bounded_queue_drops_oldest_by_default(caplog) -> None:
    bus = AppEvents(buffer_size=2)
    caplog.set_level(logging.WARNING, logger="nexus_sdk.events")
    async with bus.subscribe("topic") as stream:  # default drop_oldest
        for i in range(5):
            await bus.publish("topic", {"i": i})
        # buffer holds the last 2 (3, 4)
        first = await _receive_one(stream)
        second = await _receive_one(stream)
        assert (first.payload["i"], second.payload["i"]) == (3, 4)
    assert any("drop_oldest" in r.message for r in caplog.records)


# 12
async def test_bounded_queue_drops_newest_policy(caplog) -> None:
    bus = AppEvents(buffer_size=2)
    caplog.set_level(logging.WARNING, logger="nexus_sdk.events")
    async with bus.subscribe("topic", policy=EventOverflowPolicy.drop_newest) as stream:
        for i in range(5):
            await bus.publish("topic", {"i": i})
        first = await _receive_one(stream)
        second = await _receive_one(stream)
        assert (first.payload["i"], second.payload["i"]) == (0, 1)
    assert any("drop_newest" in r.message for r in caplog.records)


# 13
async def test_context_manager_registers_on_enter() -> None:
    bus = AppEvents()
    assert bus.stats()["subscribers"] == 0
    async with bus.subscribe("topic"):
        assert bus.stats()["subscribers"] == 1


# 14
async def test_context_manager_unregisters_on_exit() -> None:
    bus = AppEvents()
    async with bus.subscribe("topic"):
        pass
    assert bus.stats()["subscribers"] == 0


# 15
async def test_context_manager_unregisters_on_exception() -> None:
    bus = AppEvents()
    with pytest.raises(RuntimeError, match="kaboom"):
        async with bus.subscribe("topic"):
            assert bus.stats()["subscribers"] == 1
            raise RuntimeError("kaboom")
    assert bus.stats()["subscribers"] == 0


# 16
async def test_multiple_context_managers_coexist() -> None:
    bus = AppEvents()
    async with bus.subscribe("a.*"):
        async with bus.subscribe("b.*"):
            async with bus.subscribe("c.*"):
                assert bus.stats()["subscribers"] == 3
        assert bus.stats()["subscribers"] == 1
    assert bus.stats()["subscribers"] == 0


# 17
async def test_fnmatch_pattern_validation_on_subscribe_raises_on_invalid() -> None:
    bus = AppEvents()
    # fnmatch itself accepts almost anything (a stray "[" is
    # treated as a literal), so the validation surface that
    # actually has teeth is the type/empty guard. Both must
    # raise ValueError so a producer typo or accidental ``None``
    # never silently subscribes to a no-op pattern.
    with pytest.raises(ValueError, match="non-empty string"):
        async with bus.subscribe(""):
            pass
    with pytest.raises(ValueError, match="non-empty string"):
        async with bus.subscribe(None):  # type: ignore[arg-type]
            pass


# 18
async def test_publish_is_sync_send_nowait_not_await(monkeypatch) -> None:
    bus = AppEvents(buffer_size=4)
    awaited_send = 0
    real_send = anyio.streams.memory.MemoryObjectSendStream.send

    async def tracking_send(self, item):  # type: ignore[no-untyped-def]
        nonlocal awaited_send
        awaited_send += 1
        return await real_send(self, item)

    monkeypatch.setattr(
        "anyio.streams.memory.MemoryObjectSendStream.send",
        tracking_send,
    )
    async with bus.subscribe("topic") as stream:
        await bus.publish("topic", {"i": 0})
        envelope = await _receive_one(stream)
        assert envelope.payload == {"i": 0}
    # publish() must use send_nowait under drop_oldest default,
    # never the awaitable send().
    assert awaited_send == 0


# 19
async def test_subscribe_after_publish_misses_event() -> None:
    bus = AppEvents()
    await bus.publish("party.refreshed", {"missed": True})
    async with bus.subscribe("party.refreshed") as stream:
        with pytest.raises(asyncio.TimeoutError):
            await _receive_one(stream)


# 20
async def test_envelope_payload_is_serializable_dict() -> None:
    bus = AppEvents()
    with pytest.raises(ValueError, match="JSON-serialisable"):
        await bus.publish("topic", {"bad": object()})


# 21
async def test_overflow_drop_oldest_logs_warning_once_per_minute(caplog) -> None:
    bus = AppEvents(buffer_size=1)
    caplog.set_level(logging.WARNING, logger="nexus_sdk.events")
    async with bus.subscribe("topic") as _stream:
        for i in range(50):
            await bus.publish("topic", {"i": i})
    drop_records = [r for r in caplog.records if "drop_oldest" in r.message]
    # 50 overflows but only one surfacing warning thanks to throttle
    assert len(drop_records) == 1


# 22
async def test_subscriber_with_slow_consumer_does_not_block_others() -> None:
    bus = AppEvents(buffer_size=1)
    async with bus.subscribe("topic") as slow_stream:
        async with bus.subscribe("topic") as fast_stream:
            # Fill the slow buffer without draining it.
            await bus.publish("topic", {"slow": True})
            # The next publish must not block — fast subscriber
            # gets it immediately even though slow has overflowed.
            await asyncio.wait_for(bus.publish("topic", {"i": 1}), 0.2)
            envelope = await _receive_one(fast_stream)
            assert envelope.payload == {"i": 1}
            # slow_stream still receives the most recent event under drop_oldest
            tail = await _receive_one(slow_stream)
            assert tail.payload == {"i": 1}


# 23
async def test_per_app_scope_isolation() -> None:
    bus_a = AppEvents()
    bus_b = AppEvents()
    async with bus_b.subscribe("topic") as stream_b:
        await bus_a.publish("topic", {"from": "a"})
        with pytest.raises(asyncio.TimeoutError):
            await _receive_one(stream_b)


# 24
async def test_shutdown_closes_all_subscribers_gracefully() -> None:
    bus = AppEvents()
    received: list[EventEnvelope] = []

    async def consumer(stream) -> None:
        async for envelope in stream:
            received.append(envelope)

    cm = bus.subscribe("topic")
    stream = await cm.__aenter__()
    try:
        task = asyncio.create_task(consumer(stream))
        await bus.publish("topic", {"i": 0})
        await asyncio.sleep(0.05)
        await bus.aclose()
        await asyncio.wait_for(task, 0.5)
    finally:
        await cm.__aexit__(None, None, None)
    assert len(received) == 1
    assert bus.closed is True
    with pytest.raises(RuntimeError, match="aclose"):
        await bus.publish("topic", {})


# 25
async def test_event_bus_stats_reports_subscribers_count() -> None:
    bus = AppEvents()
    assert bus.stats() == {"subscribers": 0}
    async with bus.subscribe("a"):
        async with bus.subscribe("b"):
            assert bus.stats() == {"subscribers": 2}
        assert bus.stats() == {"subscribers": 1}
    assert bus.stats() == {"subscribers": 0}
