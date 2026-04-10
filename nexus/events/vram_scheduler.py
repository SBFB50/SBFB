"""VRAM-aware priority scheduler for LLM calls.

Replaces the simple asyncio.Lock in LLMRouter with a priority queue
that batches same-model calls and respects GPU memory constraints.

RTX 5080 16GB VRAM constraints:
- nomic-embed-text (137MB) -- ALWAYS co-resident, bypasses queue
- gemma4:e4b (9.6GB) -- light tasks, medium priority
- gemma-4-26B-A4B heretic (MoE, 4B active) -- reasoning + deep analysis + vision deep
- nexus (legacy deepseek-r1 14B) -- kept for backward compat
- qwen3-vl:8b (5GB) -- vision, medium-low priority

Architecture:
- Embedding calls bypass the queue entirely (always fit in VRAM alongside anything).
- Light model calls (gemma4:e4b) use a dedicated light lock -- they coexist with
  the embedding model but must not overlap with heavy models.
- Heavy model calls (nexus, deepseek, qwen3-vl, voxtral) go through a priority
  queue with model affinity batching: when a heavy model is loaded, pending
  requests for the SAME model get promoted to run next.
"""

from __future__ import annotations

import asyncio
import time
from contextlib import asynccontextmanager
from dataclasses import dataclass, field
from enum import IntEnum
from typing import AsyncIterator

from loguru import logger

from nexus.config import settings


# ---------------------------------------------------------------------------
# Priority levels (lower number = higher priority = runs first)
# ---------------------------------------------------------------------------

class VRAMPriority(IntEnum):
    """Priority tiers for GPU scheduling.

    Lower values run first. Embeddings bypass the queue entirely.
    """
    EMBEDDING = 10    # nomic-embed-text -- always co-resident, no queue
    FAST_LLM = 20     # gemma4:e4b -- light model, separate lock
    VISION = 30       # gemma-4-26B-A4B heretic (vision deep) -- heavy, queued
    REASONING = 40    # gemma-4-26B-A4B heretic (reasoning) -- heavy, queued
    DEEP = 50         # gemma-4-26B-A4B heretic (deep analysis) -- heaviest, queued


# ---------------------------------------------------------------------------
# Keep-alive durations per model tier
# ---------------------------------------------------------------------------

# Expensive models should free VRAM quickly; cheap ones can stay resident.
KEEP_ALIVE_MAP: dict[str, str] = {
    "nomic-embed-text": "30m",
    # Single LLM model — keep loaded permanently (no swap needed)
    "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m": "30m",
    # Legacy models (kept for backward compat if overridden via env)
    "gemma4:e4b": "10m",
    "aratan/gemma-4-E4B-it-heretic": "10m",
    "nexus": "3m",
    "huihui_ai/deepseek-r1-abliterated:14b": "3m",
    "qwen3-vl:8b": "5m",
    "voxtral-mini:4b": "5m",
}

# Default for unknown models
_DEFAULT_KEEP_ALIVE = "5m"


def get_keep_alive(model: str) -> str:
    """Return the appropriate keep_alive duration for a model."""
    return KEEP_ALIVE_MAP.get(model, _DEFAULT_KEEP_ALIVE)


# ---------------------------------------------------------------------------
# Model classification helpers
# ---------------------------------------------------------------------------

# Models that use the light lock (simple serialization, no priority queue).
# When all LLM models are the same (single-model stack), everything is "light"
# and stays permanently loaded — zero model swaps, max GPU utilization.
_LIGHT_MODELS: frozenset[str] = frozenset({
    settings.model_fast,
    settings.model_vision,
    settings.model_reasoning,
    settings.model_deep,
    settings.model_vision_deep,
})

# The embedding model bypasses all queues.
_EMBEDDING_MODELS: frozenset[str] = frozenset({
    settings.model_embedding,  # nomic-embed-text
})


def _is_embedding_model(model: str) -> bool:
    """Return True if the model is an embedding model (bypasses queue)."""
    return model in _EMBEDDING_MODELS or "embed" in model.lower()


def _is_light_model(model: str) -> bool:
    """Return True if the model is a light model (uses light lock)."""
    return model in _LIGHT_MODELS


# ---------------------------------------------------------------------------
# Priority queue entry
# ---------------------------------------------------------------------------

# Monotonic counter for FIFO ordering within same priority
_sequence_counter = 0


def _next_sequence() -> int:
    global _sequence_counter
    _sequence_counter += 1
    return _sequence_counter


@dataclass(order=True)
class _QueueEntry:
    """A pending request in the heavy-model priority queue.

    Ordering: (priority, sequence) -- lower priority number goes first,
    ties broken by arrival order (FIFO).
    """
    priority: int
    sequence: int = field(compare=True)
    model: str = field(compare=False)
    label: str = field(compare=False)
    ready: asyncio.Event = field(compare=False, default_factory=asyncio.Event)
    enqueued_at: float = field(compare=False, default_factory=time.monotonic)


# ---------------------------------------------------------------------------
# VRAMScheduler
# ---------------------------------------------------------------------------

_MAX_HEAVY_QUEUE = 50  # Maximum pending heavy-model requests before rejecting


class VRAMScheduler:
    """VRAM-aware priority scheduler for GPU model access.

    Usage::

        scheduler = VRAMScheduler()

        # In LLMRouter:
        async with scheduler.gpu_access(VRAMPriority.DEEP, "nexus", "hypothesis_scoring"):
            result = await client.generate(model="nexus", prompt=..., keep_alive="3m")

    Embedding calls pass through immediately. Light model calls serialise
    via a dedicated lock. Heavy model calls enter a priority queue with
    model-affinity batching.
    """

    def __init__(self) -> None:
        # Lock for light models (gemma4:e4b) -- separate from heavy queue
        self._light_lock = asyncio.Lock()

        # Heavy model queue state
        self._heavy_lock = asyncio.Lock()
        self._heavy_queue: list[_QueueEntry] = []
        self._heavy_active: _QueueEntry | None = None
        self._current_heavy_model: str | None = None

        # --- Light/Heavy mutual exclusion ---
        # Prevents VRAM overflow: E4B (10GB) + 26B (15GB) = 25GB > 16GB VRAM.
        # Light must wait for heavy to finish and vice-versa.
        self._heavy_idle = asyncio.Event()
        self._heavy_idle.set()  # Initially no heavy model active
        self._light_idle = asyncio.Event()
        self._light_idle.set()  # Initially no light model active

        # Metrics
        self._total_requests: int = 0
        self._total_swaps: int = 0
        self._total_batched: int = 0
        self._total_wait_time: float = 0.0

        logger.info(
            "VRAMScheduler initialised | light_models={} | embedding_models={}",
            _LIGHT_MODELS,
            _EMBEDDING_MODELS,
        )

    # ------------------------------------------------------------------
    # Properties
    # ------------------------------------------------------------------

    @property
    def current_heavy_model(self) -> str | None:
        """The model currently loaded in GPU VRAM (heavy slot)."""
        return self._current_heavy_model

    @property
    def queue_depth(self) -> int:
        """Number of requests waiting in the heavy queue."""
        return len(self._heavy_queue)

    @property
    def stats(self) -> dict:
        """Return scheduler statistics."""
        return {
            "total_requests": self._total_requests,
            "total_swaps": self._total_swaps,
            "total_batched": self._total_batched,
            "total_wait_time_s": round(self._total_wait_time, 2),
            "current_heavy_model": self._current_heavy_model,
            "queue_depth": self.queue_depth,
        }

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    @asynccontextmanager
    async def gpu_access(
        self,
        priority: VRAMPriority,
        model: str,
        label: str = "",
    ) -> AsyncIterator[None]:
        """Context manager that gates GPU access by priority and model type.

        Args:
            priority: Scheduling priority (lower = more urgent).
            model: Ollama model name to load.
            label: Human-readable label for logging (e.g., task type).

        Yields control once the caller has exclusive GPU access for their
        model tier.
        """
        self._total_requests += 1

        # --- Embedding: always pass through, no queueing ---
        if _is_embedding_model(model):
            logger.debug("VRAM bypass (embedding): {} [{}]", model, label)
            yield
            return

        # --- Light model: use dedicated lock ---
        if _is_light_model(model):
            await self._acquire_light(model, label)
            try:
                yield
            finally:
                self._light_lock.release()
                self._light_idle.set()  # Signal: light model done
            return

        # --- Heavy model: priority queue with model affinity ---
        entry = _QueueEntry(
            priority=priority,
            sequence=_next_sequence(),
            model=model,
            label=label,
        )
        await self._enqueue_heavy(entry)
        try:
            yield
        finally:
            self._release_heavy(entry)

    # ------------------------------------------------------------------
    # Light model scheduling
    # ------------------------------------------------------------------

    async def _acquire_light(self, model: str, label: str) -> None:
        """Acquire the light model lock, waiting for heavy to finish first.

        Light and heavy models cannot coexist in VRAM (E4B 10GB + 26B 15GB
        = 25GB > 16GB). The light caller must wait until no heavy model is
        active before acquiring the light lock.
        """
        t0 = time.monotonic()

        # Wait for heavy model to finish (mutual exclusion)
        if not self._heavy_idle.is_set():
            logger.info(
                "VRAM light waiting for heavy to finish: {} ({}) | heavy={}",
                model, label, self._current_heavy_model,
            )
            await self._heavy_idle.wait()

        if self._light_lock.locked():
            logger.debug(
                "VRAM light lock contention: {} ({}) waiting", model, label,
            )

        await self._light_lock.acquire()
        self._light_idle.clear()  # Signal: light model now active

        wait = time.monotonic() - t0
        if wait > 1.0:
            logger.info(
                "VRAM light lock acquired: {} ({}) after {:.1f}s",
                model, label, wait,
            )
        self._total_wait_time += wait

    # ------------------------------------------------------------------
    # Heavy model scheduling (priority queue + model affinity)
    # ------------------------------------------------------------------

    async def _enqueue_heavy(self, entry: _QueueEntry) -> None:
        """Add a request to the heavy queue and wait for its turn."""
        async with self._heavy_lock:
            if self._heavy_active is None:
                # No one active -- wait for light to finish, then run
                self._heavy_active = entry
                await self._wait_light_idle_and_activate(entry)
                return
            else:
                # Reject if queue is full (back-pressure)
                if len(self._heavy_queue) >= _MAX_HEAVY_QUEUE:
                    logger.warning(
                        "VRAMScheduler heavy queue full ({}/{}), dropping: {} ({})",
                        len(self._heavy_queue), _MAX_HEAVY_QUEUE,
                        entry.model, entry.label,
                    )
                    raise RuntimeError("LLM queue full — try again later")
                # Someone active -- queue up
                self._heavy_queue.append(entry)
                self._heavy_queue.sort()  # Sort by (priority, sequence)
                logger.info(
                    "VRAM queued: {} ({}) at priority {} | queue_depth={} | "
                    "current_model={}",
                    entry.model, entry.label, entry.priority,
                    len(self._heavy_queue), self._current_heavy_model,
                )

        # Wait outside the lock until we're signalled
        await entry.ready.wait()

        wait = time.monotonic() - entry.enqueued_at
        self._total_wait_time += wait
        if wait > 5.0:
            logger.warning(
                "VRAM heavy wait: {} ({}) waited {:.1f}s",
                entry.model, entry.label, wait,
            )

    async def _wait_light_idle_and_activate(self, entry: _QueueEntry) -> None:
        """Wait for light model to finish, then activate heavy model."""
        if not self._light_idle.is_set():
            logger.info(
                "VRAM heavy waiting for light to finish: {} ({}) | light_lock={}",
                entry.model, entry.label, self._light_lock.locked(),
            )
            await self._light_idle.wait()
        self._heavy_idle.clear()  # Signal: heavy model now active
        self._activate_heavy(entry)

    def _release_heavy(self, entry: _QueueEntry) -> None:
        """Release the heavy slot and promote the next queued request.

        Uses model affinity: if pending requests want the SAME model that
        just finished, they get priority over higher-priority requests for
        a DIFFERENT model (avoids expensive model swaps).
        """
        # We must schedule the next activation outside the sync context
        # to avoid potential deadlock with _heavy_lock.
        loop = asyncio.get_event_loop()
        loop.call_soon(lambda: asyncio.ensure_future(self._promote_next(entry)))

    async def _promote_next(self, finished: _QueueEntry) -> None:
        """Find and activate the next heavy request (with affinity)."""
        async with self._heavy_lock:
            if not self._heavy_queue:
                self._heavy_active = None
                self._heavy_idle.set()  # Signal: heavy slot is free
                logger.debug(
                    "VRAM heavy slot freed: {} ({}) | no pending requests",
                    finished.model, finished.label,
                )
                return

            # Model affinity: find next request wanting the SAME model
            next_entry = self._pick_next_with_affinity(finished.model)
            self._heavy_active = next_entry

        # Wait for light outside the lock (light must finish before heavy starts)
        await self._wait_light_idle_and_activate(next_entry)
        next_entry.ready.set()

    def _pick_next_with_affinity(self, current_model: str) -> _QueueEntry:
        """Pick the next entry from the queue, preferring same-model requests.

        If there are requests for the currently loaded model, pick the
        highest-priority one among them (avoids a model swap). Otherwise,
        pick the highest-priority request overall.
        """
        # Find same-model entries
        same_model = [e for e in self._heavy_queue if e.model == current_model]

        if same_model:
            # Pick the best (lowest priority number) same-model entry
            chosen = min(same_model)
            self._heavy_queue.remove(chosen)
            self._total_batched += 1
            logger.info(
                "VRAM affinity batch: {} ({}) reuses loaded model | "
                "skipped_swap=True | remaining_queue={}",
                chosen.model, chosen.label, len(self._heavy_queue),
            )
            return chosen

        # No same-model entries -- pick highest priority overall
        # Queue is already sorted, so first element is highest priority
        chosen = self._heavy_queue.pop(0)
        return chosen

    def _activate_heavy(self, entry: _QueueEntry) -> None:
        """Mark a heavy entry as active, log model swaps."""
        prev_model = self._current_heavy_model

        if prev_model is not None and prev_model != entry.model:
            self._total_swaps += 1
            logger.info(
                "VRAM model swap: {} -> {} ({}) | total_swaps={}",
                prev_model, entry.model, entry.label, self._total_swaps,
            )
        elif prev_model is None:
            logger.debug(
                "VRAM heavy slot acquired: {} ({})",
                entry.model, entry.label,
            )

        self._current_heavy_model = entry.model
