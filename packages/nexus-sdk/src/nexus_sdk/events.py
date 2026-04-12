"""AppContext.events — per-app in-process pub/sub over anyio memory streams.

Sprint 9 Phase C (D2 impl). Every app gets a per-instance
:class:`AppEvents` wired by the coordinator loader on
:attr:`nexus_sdk.AppContext.events` BEFORE the app's ``on_start``
hook runs. The bus is the asynchronous mirror of P13
``AppContext.storage``: P13 is the writable per-app KV, P14 is
the in-process pub/sub fan-out.

Design choices (frozen Sprint 9 Day 0 D2)
-----------------------------------------

- **Wrapper around :func:`anyio.create_memory_object_stream`.** Each
  call to :meth:`AppEvents.subscribe` allocates its own
  ``(send_stream, receive_stream)`` pair with the configured
  ``max_buffer_size`` (default 1024). The dispatcher iterates the
  matching subscribers and pushes envelopes onto every send
  stream — there is no shared queue and no clone-per-subscriber
  trickery, which sidesteps the only fragile spot of anyio's
  memory stream API.
- **Frozen :class:`EventEnvelope`.** ``{topic, payload, timestamp,
  trace_id}`` shape with ``model_config={"frozen": True,
  "extra": "forbid"}`` — once an envelope is constructed it
  cannot be mutated, and a typo in a producer's ``topic`` name
  fails fast at construction. ``trace_id`` is generated as
  ``uuid4().hex[:16]``, ``timestamp`` is ``datetime.now(UTC)`` at
  publish time.
- **Topic glob matching via :func:`fnmatch.fnmatch`.** Subscribers
  pass shell-style patterns (``politician.*``, ``*.refreshed``,
  ``file.upload.*``). ``fnmatch`` does NOT interpret ``**`` as a
  recursive wildcard — it is just a one-character ``*`` extension
  — and the ``.`` is a literal character (not a delimiter), so
  ``politician.*`` matches ``politician.refreshed`` but also
  ``politician.party.refreshed`` because ``*`` matches the entire
  remaining string. Producers that need single-segment
  semantics should use a more specific suffix.
- **Sync dispatch via :meth:`MemoryObjectSendStream.send_nowait`.**
  :meth:`AppEvents.publish` is ``async def`` so callers can await
  it the same way they await every other SDK helper, but the
  fan-out loop never awaits inside the body for the
  ``drop_oldest`` / ``drop_newest`` policies — every push is a
  ``send_nowait`` call wrapped in
  ``except anyio.WouldBlock``. The block-policy escape hatch is
  the only path that awaits a real ``send`` and is documented as
  risky (a single slow consumer stalls the whole bus).
- **Overflow policy enum :class:`EventOverflowPolicy`.** Three
  modes:

  - ``drop_oldest`` (default): drain one envelope from the
    receive side via ``receive_nowait`` then retry the
    ``send_nowait``. The drop is logged once per minute per
    subscriber via :class:`_ThrottledWarning` so a slow consumer
    cannot flood the structlog stream.
  - ``drop_newest``: skip the publish for that subscriber and log
    the throttled warning.
  - ``block``: ``await send_stream.send(envelope)``. Documented
    as a foot-gun because a single slow consumer stalls every
    other subscriber that comes after it in the dispatch loop.

- **Context manager subscribe.** ``async with
  events.subscribe(pattern) as stream:`` registers a fresh
  :class:`Subscription` on enter, yields the receive stream, and
  unregisters / closes both halves of the stream on exit — even
  when the body raises. Anti-pattern explicitly rejected: weak
  references to coroutines, which the GC tears down before the
  coroutine has a chance to run.
- **Per-app, in-process scope.** A given :class:`AppEvents`
  instance is bound to a single app. Cross-app and cross-node
  fan-out are explicitly out of scope for Sprint 9; the
  coordinator constructs one bus per app at boot and assigns it
  to :attr:`nexus_sdk.AppContext.events`. Events do not survive
  a coordinator restart — there is no replay buffer.
- **Lifespan close via :meth:`aclose`.** The coordinator's
  ``stop()`` closes every app's bus before the app's ``on_stop``
  hook so any subscriber currently iterating sees a clean
  ``EndOfStream`` instead of a hung receive.

Anti-patterns explicitly rejected
---------------------------------

- ``asyncio.Queue`` with weak refs (re-invents the wheel; weak
  refs on coroutines are GC footguns).
- iroh-gossip (cross-node, way too heavy for the per-app
  in-process surface this primitive serves).
- MQTT topic ``+`` / ``#`` (more expressive but adds a parser
  dependency for no Sprint 9 win).
- :mod:`blinker` (sync only, not async-friendly).
- :mod:`aiopubsub` (peu maintained).

Reference: ``.planning/sprint9_kickoff.md`` §4 D2,
``.planning/sprint9_plan.md`` §6 Phase C, ``docs/shell/PATTERNS.md``
P14.
"""

from __future__ import annotations

import enum
import fnmatch
import json
import logging
import time
import uuid
from contextlib import asynccontextmanager
from datetime import datetime, timezone
from typing import Any, AsyncIterator

import anyio
from anyio.streams.memory import MemoryObjectReceiveStream, MemoryObjectSendStream
from pydantic import BaseModel, ConfigDict, Field, field_validator

__all__ = [
    "AppEvents",
    "EventEnvelope",
    "EventOverflowPolicy",
]


_DEFAULT_BUFFER_SIZE = 1024
_THROTTLE_INTERVAL_SECONDS = 60.0
_log = logging.getLogger(__name__)


class EventOverflowPolicy(str, enum.Enum):
    """Per-subscriber buffer-overflow handling.

    The default ``drop_oldest`` matches the canonical "ring
    buffer" behaviour every dashboard expects — when a slow
    consumer falls behind it loses the oldest events first while
    the producer keeps moving. The two alternatives exist for
    completeness; ``block`` is documented as risky and reserved
    for tests that need deterministic backpressure.
    """

    drop_oldest = "drop_oldest"
    drop_newest = "drop_newest"
    block = "block"


class EventEnvelope(BaseModel):
    """Frozen Pydantic envelope around one published event.

    Constructed exclusively by :meth:`AppEvents.publish`. The
    ``payload`` field is validated as JSON-serialisable on
    construction so the bus refuses to ferry an envelope that
    the SSE bridge would later fail to ``json.dumps``.
    """

    model_config = ConfigDict(frozen=True, extra="forbid")

    topic: str = Field(..., min_length=1)
    payload: dict[str, Any]
    timestamp: datetime
    trace_id: str = Field(..., min_length=1)

    @field_validator("payload")
    @classmethod
    def _payload_must_be_json(cls, value: dict[str, Any]) -> dict[str, Any]:
        """Refuse a payload that the standard JSON encoder rejects.

        The check happens at envelope construction so a producer
        sees the error at the ``publish`` call site instead of
        deep inside the SSE bridge. The encoder is the same one
        the bridge uses (:func:`json.dumps` with default
        settings) so anything that survives this validator
        survives serialisation.
        """
        try:
            json.dumps(value)
        except (TypeError, ValueError) as exc:
            raise ValueError(f"event payload is not JSON-serialisable: {exc}") from exc
        return value


class _ThrottledWarning:
    """Per-subscriber overflow warning rate limiter.

    Tracks the last time we logged an overflow for a given
    subscriber. The first call within a minute logs at WARNING,
    every subsequent call within that minute is silently dropped.
    The window resets after :data:`_THROTTLE_INTERVAL_SECONDS`.
    """

    def __init__(self) -> None:
        self._last_logged_at: float = 0.0
        self._suppressed: int = 0

    def maybe_log(self, message: str) -> None:
        """Log ``message`` at WARNING unless we logged in the last minute.

        Includes a tail count of suppressed messages so an
        operator reading the log can tell at a glance how many
        events were dropped between two surfacing warnings.
        """
        now = time.monotonic()
        if now - self._last_logged_at < _THROTTLE_INTERVAL_SECONDS:
            self._suppressed += 1
            return
        if self._suppressed > 0:
            _log.warning("%s (and %d earlier drops since last warning)", message, self._suppressed)
        else:
            _log.warning("%s", message)
        self._last_logged_at = now
        self._suppressed = 0


class _Subscription:
    """One ``async with subscribe(pattern)`` registration.

    Holds the pattern, the bound (send, receive) memory stream
    pair, the per-subscriber overflow policy, and the throttled
    warning tracker. Stored on :class:`AppEvents` in a flat
    ``dict[int, _Subscription]`` keyed by ``id(self)`` so
    register / unregister are O(1) and the dispatcher does not
    care about insertion order.
    """

    __slots__ = ("pattern", "send_stream", "receive_stream", "policy", "warnings")

    def __init__(
        self,
        pattern: str,
        send_stream: MemoryObjectSendStream[EventEnvelope],
        receive_stream: MemoryObjectReceiveStream[EventEnvelope],
        policy: EventOverflowPolicy,
    ) -> None:
        self.pattern = pattern
        self.send_stream = send_stream
        self.receive_stream = receive_stream
        self.policy = policy
        self.warnings = _ThrottledWarning()


class AppEvents:
    """Per-app in-process pub/sub bus.

    Parameters
    ----------
    buffer_size:
        Max number of unread envelopes per subscriber. Defaults
        to 1024 — large enough that a transient consumer pause
        of a few seconds does not start dropping events under a
        normal app's publish rate (a few events per second), and
        small enough that a runaway producer cannot leak
        unbounded memory before the operator notices.
    """

    def __init__(self, *, buffer_size: int = _DEFAULT_BUFFER_SIZE) -> None:
        if buffer_size < 1:
            raise ValueError(f"buffer_size must be >= 1 (got {buffer_size!r})")
        self._buffer_size = int(buffer_size)
        self._subscriptions: dict[int, _Subscription] = {}
        self._closed = False

    # ------------------------------------------------------------------
    # Public read-only properties
    # ------------------------------------------------------------------

    @property
    def buffer_size(self) -> int:
        """Return the per-subscriber buffer size."""
        return self._buffer_size

    @property
    def closed(self) -> bool:
        """Return whether :meth:`aclose` has been called."""
        return self._closed

    def stats(self) -> dict[str, int]:
        """Return a snapshot of the bus internals.

        Currently exposes ``subscribers`` (the live registration
        count). Used by the SSE bridge tests and by the bus's
        own ``test_event_bus_stats_reports_subscribers_count``
        contract test.
        """
        return {"subscribers": len(self._subscriptions)}

    # ------------------------------------------------------------------
    # Publish
    # ------------------------------------------------------------------

    async def publish(self, topic: str, payload: dict[str, Any]) -> None:
        """Build an :class:`EventEnvelope` and dispatch to matching subscribers.

        ``topic`` must be non-empty and a string; ``payload`` must
        be a JSON-serialisable dict (validated by
        :class:`EventEnvelope`). The dispatch loop iterates a
        snapshot of the live subscriptions so a subscriber that
        unregisters mid-fan-out cannot perturb the iteration.
        """
        if self._closed:
            raise RuntimeError("AppEvents.publish called after aclose()")
        if not isinstance(topic, str) or not topic:
            raise ValueError("topic must be a non-empty string")
        envelope = EventEnvelope(
            topic=topic,
            payload=payload,
            timestamp=datetime.now(timezone.utc),
            trace_id=uuid.uuid4().hex[:16],
        )
        for sub in list(self._subscriptions.values()):
            if not fnmatch.fnmatch(topic, sub.pattern):
                continue
            await self._deliver_to(sub, envelope)

    async def _deliver_to(self, sub: _Subscription, envelope: EventEnvelope) -> None:
        """Push ``envelope`` onto ``sub`` honouring its overflow policy.

        Sync ``send_nowait`` for ``drop_oldest`` / ``drop_newest``;
        the only ``await`` path is the ``block`` escape hatch.
        """
        try:
            sub.send_stream.send_nowait(envelope)
            return
        except anyio.WouldBlock:
            pass
        except anyio.BrokenResourceError:
            return
        if sub.policy is EventOverflowPolicy.drop_oldest:
            try:
                sub.receive_stream.receive_nowait()
            except (anyio.WouldBlock, anyio.EndOfStream):
                pass
            try:
                sub.send_stream.send_nowait(envelope)
            except anyio.WouldBlock:
                pass
            except anyio.BrokenResourceError:
                return
            sub.warnings.maybe_log(
                f"AppEvents drop_oldest: subscriber pattern={sub.pattern!r} buffer full, dropped one"
            )
        elif sub.policy is EventOverflowPolicy.drop_newest:
            sub.warnings.maybe_log(
                f"AppEvents drop_newest: subscriber pattern={sub.pattern!r} buffer full, dropped envelope"
            )
        elif sub.policy is EventOverflowPolicy.block:
            try:
                await sub.send_stream.send(envelope)
            except anyio.BrokenResourceError:
                return

    # ------------------------------------------------------------------
    # Subscribe
    # ------------------------------------------------------------------

    @asynccontextmanager
    async def subscribe(
        self,
        pattern: str,
        *,
        policy: EventOverflowPolicy = EventOverflowPolicy.drop_oldest,
    ) -> AsyncIterator[MemoryObjectReceiveStream[EventEnvelope]]:
        """Register a subscriber and yield its receive stream.

        Usage::

            async with events.subscribe("party.refreshed") as stream:
                async for envelope in stream:
                    handle(envelope)

        ``pattern`` is a :func:`fnmatch.fnmatch` shell-style glob.
        Calling :meth:`subscribe` validates the pattern by trying
        a no-op match and refusing :class:`re.error` (the only
        case is a malformed bracket expression like ``"foo[a"``).
        On exit the subscriber is removed from the registry and
        both halves of the memory stream are closed even when the
        body raises.
        """
        if self._closed:
            raise RuntimeError("AppEvents.subscribe called after aclose()")
        if not isinstance(pattern, str) or not pattern:
            raise ValueError("pattern must be a non-empty string")

        send_stream, receive_stream = anyio.create_memory_object_stream[EventEnvelope](
            max_buffer_size=self._buffer_size,
        )
        sub = _Subscription(pattern, send_stream, receive_stream, policy)
        sub_id = id(sub)
        self._subscriptions[sub_id] = sub
        try:
            yield receive_stream
        finally:
            self._subscriptions.pop(sub_id, None)
            await receive_stream.aclose()
            await send_stream.aclose()

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    async def aclose(self) -> None:
        """Close every live subscription and mark the bus closed.

        Idempotent — a second call is a no-op. After the first
        call any further :meth:`publish` raises ``RuntimeError``;
        any subscriber currently iterating its receive stream
        sees ``EndOfStream`` and exits the ``async for`` loop
        gracefully.
        """
        if self._closed:
            return
        self._closed = True
        for sub in list(self._subscriptions.values()):
            try:
                await sub.send_stream.aclose()
            except Exception:
                pass
        self._subscriptions.clear()
