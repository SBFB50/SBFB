# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 9 Phase C — SSE bridge over :class:`AppEvents`.

Three tests pinning the contracts of
``GET /app/{name}/events?pattern=...`` (R7 mitigation included).

The streaming generator is exercised directly via
:func:`nexus_coordinator.api.events.render_sse_stream` rather
than through :class:`httpx.ASGITransport`. ``ASGITransport`` has
a long-standing buffering quirk on
:class:`fastapi.responses.StreamingResponse` that makes
in-process SSE assertions deadlock — the ``render_sse_stream``
helper isolates the protocol logic that R7 actually pins
(envelope framing, heartbeat scheduling, ``finally:``
cleanup), and a separate ``test_sse_route_returns_event_stream``
asserts the HTTP layer mounts the route with the right
``content-type`` against a real :class:`fastapi.testclient.TestClient`.
"""

from __future__ import annotations

import asyncio
import json

import pytest
from nexus_coordinator.api.events import render_sse_stream
from nexus_sdk import AppEvents


def _parse_sse_data(frame: str) -> dict:
    """Pull the JSON payload out of a single SSE message frame."""
    for line in frame.splitlines():
        if line.startswith("data: "):
            return json.loads(line[len("data: ") :])
    raise AssertionError(f"no data line in SSE frame: {frame!r}")


async def _drain_one_envelope(generator) -> dict:
    """Pull bytes from ``generator`` until the first ``data:`` lands.

    Comment-only frames (heartbeats) are skipped silently. Used
    by the bus tests so a missing publish surfaces as a clean
    timeout instead of a deadlock.
    """
    buffer = ""
    async for chunk in generator:
        buffer += chunk.decode("utf-8")
        while "\n\n" in buffer:
            frame, buffer = buffer.split("\n\n", 1)
            if not frame.strip() or frame.startswith(":"):
                continue
            return _parse_sse_data(frame)
    raise AssertionError("generator exhausted without yielding a data frame")


@pytest.mark.asyncio
async def test_events_sse_streams_envelope_on_publish() -> None:
    """A publish on the bus emits one ``data:`` SSE line carrying
    the JSON-encoded envelope (topic, payload, timestamp,
    trace_id)."""
    bus = AppEvents()
    try:
        gen = render_sse_stream(bus, "party.refreshed", heartbeat_interval_seconds=2.0)

        async def producer() -> None:
            # Wait until the generator has entered the
            # ``async with bus.subscribe(...)`` block.
            for _ in range(50):
                if bus.stats()["subscribers"] == 1:
                    break
                await asyncio.sleep(0.01)
            await bus.publish("party.refreshed", {"count": 7})

        producer_task = asyncio.create_task(producer())
        try:
            envelope = await asyncio.wait_for(_drain_one_envelope(gen), timeout=2.0)
            assert envelope["topic"] == "party.refreshed"
            assert envelope["payload"] == {"count": 7}
            assert isinstance(envelope["trace_id"], str) and len(envelope["trace_id"]) == 16
            assert "timestamp" in envelope
        finally:
            await producer_task
            await gen.aclose()
    finally:
        await bus.aclose()


@pytest.mark.asyncio
async def test_events_sse_filters_by_pattern() -> None:
    """Non-matching topics are filtered out by the bus before they
    reach the SSE generator — only the matching one shows up."""
    bus = AppEvents()
    try:
        gen = render_sse_stream(bus, "party.*", heartbeat_interval_seconds=2.0)

        async def producer() -> None:
            for _ in range(50):
                if bus.stats()["subscribers"] == 1:
                    break
                await asyncio.sleep(0.01)
            # politician.created does not match party.*
            await bus.publish("politician.created", {"id": 1})
            # party.refreshed does match
            await bus.publish("party.refreshed", {"count": 3})

        producer_task = asyncio.create_task(producer())
        try:
            envelope = await asyncio.wait_for(_drain_one_envelope(gen), timeout=2.0)
            assert envelope["topic"] == "party.refreshed"
            assert envelope["payload"] == {"count": 3}
        finally:
            await producer_task
            await gen.aclose()
    finally:
        await bus.aclose()


@pytest.mark.asyncio
async def test_events_sse_disconnect_unregisters_subscriber() -> None:
    """R7 mitigation: when the streaming generator is closed
    (e.g. on a brutal client disconnect Starlette translates
    into a CancelledError) the ``async with bus.subscribe(...)``
    finally: path runs and the subscriber count drops to zero.
    """
    bus = AppEvents()
    try:
        gen = render_sse_stream(bus, "party.refreshed", heartbeat_interval_seconds=2.0)

        # Drive the generator past its ``async with`` enter.
        async def consume() -> dict:
            return await _drain_one_envelope(gen)

        consumer_task = asyncio.create_task(consume())
        # Wait for the subscription to register.
        for _ in range(50):
            if bus.stats()["subscribers"] == 1:
                break
            await asyncio.sleep(0.01)
        assert bus.stats()["subscribers"] == 1

        # Push one envelope so the consumer drains and exits.
        await bus.publish("party.refreshed", {"count": 1})
        envelope = await asyncio.wait_for(consumer_task, timeout=2.0)
        assert envelope["topic"] == "party.refreshed"

        # Close the generator (the SSE protocol path Starlette
        # walks on a client disconnect). The async generator's
        # finally: must drop the subscriber off the bus.
        await gen.aclose()
        assert bus.stats()["subscribers"] == 0
    finally:
        await bus.aclose()
