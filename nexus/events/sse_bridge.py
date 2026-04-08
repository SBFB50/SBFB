"""
NEXUS -- SSE bridge: EventBus -> Server-Sent Events.

Creates one asyncio.Queue per SSE client connection, subscribes it
to requested EventTypes, yields events as SSE data dicts, and
unsubscribes on disconnect.
"""

from __future__ import annotations

import asyncio
import json
import logging
from collections.abc import AsyncGenerator
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent

logger = logging.getLogger(__name__)

_CLIENT_QUEUE_SIZE = 200


class SSEBridge:
    """Bridges EventBus events to SSE client streams."""

    def __init__(self, bus: EventBus) -> None:
        self._bus = bus

    async def stream(
        self,
        event_types: list[EventType],
        case_id: str | None = None,
    ) -> AsyncGenerator[dict[str, Any], None]:
        """Yield SSE-formatted dicts for the given event types.

        Each yielded dict has the shape expected by ``sse-starlette``::

            {"event": "<type>", "id": "<uuid>", "data": "<json>"}

        Filters by *case_id* when provided.  Unsubscribes from the bus
        in the ``finally`` block so disconnected clients are cleaned up.
        """
        queue: asyncio.Queue[NexusEvent | None] = asyncio.Queue(
            maxsize=_CLIENT_QUEUE_SIZE,
        )

        for etype in event_types:
            self._bus.subscribe(etype, queue)

        logger.debug(
            "SSE client connected (types=%d, case=%s)",
            len(event_types),
            case_id or "*",
        )

        try:
            while True:
                event = await queue.get()

                # Sentinel None = bus shutdown
                if event is None:
                    break

                # Filter by case if requested
                if case_id and event.case_id != case_id:
                    continue

                yield {
                    "event": event.event_type.value,
                    "id": event.event_id,
                    "data": json.dumps(
                        {
                            "case_id": event.case_id,
                            "payload": event.payload,
                            "source_worker": event.source_worker,
                            "timestamp": event.timestamp,
                        },
                        ensure_ascii=False,
                        default=str,
                    ),
                }
        finally:
            for etype in event_types:
                self._bus.unsubscribe(etype, queue)
            logger.debug(
                "SSE client disconnected (case=%s)",
                case_id or "*",
            )
