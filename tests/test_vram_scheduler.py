"""Tests for VRAM-aware priority scheduler (nexus.events.vram_scheduler).

Tests cover:
- Priority ordering (VRAMPriority enum)
- Embedding bypass (no queueing)
- Light model lock serialisation
- Heavy model priority queue ordering
- Model affinity batching
- Keep-alive differentiation
- Stats tracking
- Backward compatibility (LLMRouter without scheduler)
"""

import asyncio
from unittest.mock import AsyncMock, patch, MagicMock

import pytest
import pytest_asyncio

from nexus.events.vram_scheduler import (
    VRAMPriority,
    VRAMScheduler,
    get_keep_alive,
    _is_embedding_model,
    _is_light_model,
    _QueueEntry,
    _next_sequence,
)


# =====================================================================
# VRAMPriority enum
# =====================================================================


class TestVRAMPriority:

    def test_priority_ordering(self):
        """Lower numeric value = higher priority."""
        assert VRAMPriority.EMBEDDING < VRAMPriority.FAST_LLM
        assert VRAMPriority.FAST_LLM < VRAMPriority.VISION
        assert VRAMPriority.VISION < VRAMPriority.REASONING
        assert VRAMPriority.REASONING < VRAMPriority.DEEP

    def test_priority_values(self):
        assert VRAMPriority.EMBEDDING == 10
        assert VRAMPriority.FAST_LLM == 20
        assert VRAMPriority.VISION == 30
        assert VRAMPriority.REASONING == 40
        assert VRAMPriority.DEEP == 50

    def test_priority_is_int(self):
        """VRAMPriority values are usable as ints for comparison."""
        assert VRAMPriority.EMBEDDING + 10 == VRAMPriority.FAST_LLM


# =====================================================================
# Model classification
# =====================================================================


class TestModelClassification:

    def test_embedding_model_detected(self):
        assert _is_embedding_model("nomic-embed-text") is True

    def test_embedding_model_by_name(self):
        """Any model with 'embed' in the name should be treated as embedding."""
        assert _is_embedding_model("some-custom-embed-model") is True

    def test_non_embedding_model(self):
        assert _is_embedding_model("nexus") is False
        assert _is_embedding_model("gemma4:e4b") is False

    def test_light_model_detected(self):
        from nexus.config import settings
        assert _is_light_model(settings.model_fast) is True

    def test_heavy_model_not_light(self):
        assert _is_light_model("nexus") is False
        assert _is_light_model("huihui_ai/deepseek-r1-abliterated:14b") is False
        assert _is_light_model("qwen3-vl:8b") is False


# =====================================================================
# Keep-alive
# =====================================================================


class TestKeepAlive:

    def test_embedding_keep_alive(self):
        assert get_keep_alive("nomic-embed-text") == "30m"

    def test_fast_model_keep_alive(self):
        assert get_keep_alive("gemma4:e4b") == "10m"

    def test_deep_model_keep_alive(self):
        assert get_keep_alive("nexus") == "3m"

    def test_reasoning_keep_alive(self):
        assert get_keep_alive("huihui_ai/deepseek-r1-abliterated:14b") == "3m"

    def test_vision_keep_alive(self):
        assert get_keep_alive("qwen3-vl:8b") == "5m"

    def test_audio_keep_alive(self):
        assert get_keep_alive("voxtral-mini:4b") == "5m"

    def test_unknown_model_default(self):
        assert get_keep_alive("some-unknown-model") == "5m"


# =====================================================================
# QueueEntry ordering
# =====================================================================


class TestQueueEntry:

    def test_priority_ordering(self):
        """Lower priority number should sort first."""
        high = _QueueEntry(priority=10, sequence=2, model="a", label="high")
        low = _QueueEntry(priority=50, sequence=1, model="b", label="low")
        assert high < low

    def test_fifo_within_same_priority(self):
        """Same priority: lower sequence (earlier arrival) sorts first."""
        first = _QueueEntry(priority=40, sequence=1, model="a", label="first")
        second = _QueueEntry(priority=40, sequence=2, model="a", label="second")
        assert first < second

    def test_sorting(self):
        """Queue entries should sort correctly."""
        entries = [
            _QueueEntry(priority=50, sequence=3, model="c", label="deep"),
            _QueueEntry(priority=10, sequence=1, model="a", label="embed"),
            _QueueEntry(priority=40, sequence=2, model="b", label="reason"),
        ]
        entries.sort()
        assert entries[0].label == "embed"
        assert entries[1].label == "reason"
        assert entries[2].label == "deep"


# =====================================================================
# VRAMScheduler -- embedding bypass
# =====================================================================


class TestEmbeddingBypass:

    @pytest.mark.asyncio
    async def test_embedding_bypasses_queue(self):
        """Embedding calls should not enter any queue or lock."""
        scheduler = VRAMScheduler()
        executed = False

        async with scheduler.gpu_access(
            VRAMPriority.EMBEDDING, "nomic-embed-text", "test-embed",
        ):
            executed = True

        assert executed is True
        # Embedding requests still count toward total
        assert scheduler.stats["total_requests"] == 1
        # No heavy queue interaction
        assert scheduler.queue_depth == 0

    @pytest.mark.asyncio
    async def test_embedding_concurrent_with_heavy(self):
        """Embedding calls should run even while a heavy model holds the slot."""
        scheduler = VRAMScheduler()
        events: list[str] = []

        async def heavy_task():
            async with scheduler.gpu_access(
                VRAMPriority.DEEP, "nexus", "heavy",
            ):
                events.append("heavy_start")
                await asyncio.sleep(0.1)
                events.append("heavy_end")

        async def embed_task():
            # Small delay to ensure heavy starts first
            await asyncio.sleep(0.02)
            async with scheduler.gpu_access(
                VRAMPriority.EMBEDDING, "nomic-embed-text", "embed",
            ):
                events.append("embed_done")

        await asyncio.gather(heavy_task(), embed_task())

        # Embedding should complete DURING the heavy task, not after
        assert events.index("embed_done") < events.index("heavy_end")


# =====================================================================
# VRAMScheduler -- light model lock
# =====================================================================


class TestLightModelLock:

    @pytest.mark.asyncio
    async def test_light_models_serialised(self):
        """Two light model calls should not overlap."""
        scheduler = VRAMScheduler()
        events: list[str] = []

        async def light_task(name: str, delay: float = 0.0):
            if delay:
                await asyncio.sleep(delay)
            async with scheduler.gpu_access(
                VRAMPriority.FAST_LLM, "gemma4:e4b", name,
            ):
                events.append(f"{name}_start")
                await asyncio.sleep(0.05)
                events.append(f"{name}_end")

        await asyncio.gather(light_task("A"), light_task("B", delay=0.01))

        # One must complete before the other starts
        a_start = events.index("A_start")
        a_end = events.index("A_end")
        b_start = events.index("B_start")
        # B_start must be after A_end (serialised)
        assert b_start > a_end


# =====================================================================
# VRAMScheduler -- heavy model queue
# =====================================================================


class TestHeavyModelQueue:

    @pytest.mark.asyncio
    async def test_single_heavy_access(self):
        """A single heavy call should work without queueing."""
        scheduler = VRAMScheduler()
        executed = False

        async with scheduler.gpu_access(
            VRAMPriority.DEEP, "nexus", "single-heavy",
        ):
            executed = True
            assert scheduler.current_heavy_model == "nexus"

        assert executed is True

    @pytest.mark.asyncio
    async def test_heavy_calls_serialised(self):
        """Two heavy model calls should not overlap."""
        scheduler = VRAMScheduler()
        events: list[str] = []

        async def heavy_task(name: str, model: str, priority: VRAMPriority, delay: float = 0.0):
            if delay:
                await asyncio.sleep(delay)
            async with scheduler.gpu_access(priority, model, name):
                events.append(f"{name}_start")
                await asyncio.sleep(0.05)
                events.append(f"{name}_end")

        await asyncio.gather(
            heavy_task("A", "nexus", VRAMPriority.DEEP),
            heavy_task("B", "nexus", VRAMPriority.DEEP, delay=0.01),
        )

        a_end = events.index("A_end")
        b_start = events.index("B_start")
        assert b_start >= a_end

    @pytest.mark.asyncio
    async def test_stats_tracking(self):
        """Stats should reflect operations performed."""
        scheduler = VRAMScheduler()

        async with scheduler.gpu_access(
            VRAMPriority.DEEP, "nexus", "test",
        ):
            pass

        stats = scheduler.stats
        assert stats["total_requests"] == 1
        assert stats["current_heavy_model"] == "nexus"
        assert stats["queue_depth"] == 0


# =====================================================================
# VRAMScheduler -- model affinity
# =====================================================================


class TestModelAffinity:

    @pytest.mark.asyncio
    async def test_same_model_batched(self):
        """When heavy slot finishes, same-model requests should be promoted."""
        scheduler = VRAMScheduler()
        execution_order: list[str] = []

        async def task(name: str, model: str, priority: VRAMPriority, delay: float = 0.0):
            if delay:
                await asyncio.sleep(delay)
            async with scheduler.gpu_access(priority, model, name):
                execution_order.append(name)
                await asyncio.sleep(0.05)

        # Start: nexus holds the slot
        # Queue: deepseek (REASONING=40) arrives, then another nexus (DEEP=50) arrives
        # Affinity: the second nexus should run before deepseek despite lower priority
        await asyncio.gather(
            task("nexus_first", "nexus", VRAMPriority.DEEP),
            task("deepseek", "huihui_ai/deepseek-r1-abliterated:14b", VRAMPriority.REASONING, delay=0.01),
            task("nexus_second", "nexus", VRAMPriority.DEEP, delay=0.02),
        )

        # nexus_first runs first (it got the slot)
        assert execution_order[0] == "nexus_first"
        # nexus_second should be promoted by affinity (same model as just finished)
        assert execution_order[1] == "nexus_second"
        assert execution_order[2] == "deepseek"

    @pytest.mark.asyncio
    async def test_swap_counted(self):
        """Model swaps should be counted in stats."""
        scheduler = VRAMScheduler()

        async with scheduler.gpu_access(VRAMPriority.DEEP, "nexus", "first"):
            pass

        # Allow promotion to happen
        await asyncio.sleep(0.05)

        async with scheduler.gpu_access(
            VRAMPriority.REASONING,
            "huihui_ai/deepseek-r1-abliterated:14b",
            "second",
        ):
            pass

        assert scheduler.stats["total_swaps"] == 1


# =====================================================================
# LLMRouter integration (backward compatibility)
# =====================================================================


class TestLLMRouterBackwardCompat:

    def test_router_without_scheduler(self):
        """LLMRouter should work without a VRAMScheduler."""
        from nexus.llm.router import LLMRouter
        mock_client = MagicMock()
        router = LLMRouter(client=mock_client)
        assert router._vram_scheduler is None

    def test_router_with_scheduler(self):
        """LLMRouter should accept a VRAMScheduler."""
        from nexus.llm.router import LLMRouter
        mock_client = MagicMock()
        scheduler = VRAMScheduler()
        router = LLMRouter(client=mock_client, vram_scheduler=scheduler)
        assert router._vram_scheduler is scheduler


# =====================================================================
# OllamaClient -- new parameters
# =====================================================================


class TestOllamaClientParameters:

    def test_embed_accepts_keep_alive(self):
        """embed() signature should accept keep_alive parameter."""
        import inspect
        from nexus.llm.ollama_client import OllamaClient
        sig = inspect.signature(OllamaClient.embed)
        assert "keep_alive" in sig.parameters
        assert "timeout" in sig.parameters

    def test_embed_batch_accepts_keep_alive(self):
        """embed_batch() signature should accept keep_alive parameter."""
        import inspect
        from nexus.llm.ollama_client import OllamaClient
        sig = inspect.signature(OllamaClient.embed_batch)
        assert "keep_alive" in sig.parameters
        assert "timeout" in sig.parameters

    def test_generate_accepts_keep_alive(self):
        """generate() signature should accept keep_alive parameter."""
        import inspect
        from nexus.llm.ollama_client import OllamaClient
        sig = inspect.signature(OllamaClient.generate)
        assert "keep_alive" in sig.parameters

    def test_generate_with_image_accepts_keep_alive(self):
        """generate_with_image() signature should accept keep_alive parameter."""
        import inspect
        from nexus.llm.ollama_client import OllamaClient
        sig = inspect.signature(OllamaClient.generate_with_image)
        assert "keep_alive" in sig.parameters

    def test_unload_model_exists(self):
        """OllamaClient should have an unload_model method."""
        from nexus.llm.ollama_client import OllamaClient
        assert hasattr(OllamaClient, "unload_model")
        assert asyncio.iscoroutinefunction(OllamaClient.unload_model)

    @pytest.mark.asyncio
    async def test_embed_batch_empty(self):
        """embed_batch with empty list should return empty list immediately."""
        from nexus.llm.ollama_client import OllamaClient
        client = OllamaClient.__new__(OllamaClient)
        # Don't init the real client, just test the early return
        result = await client.embed_batch([])
        assert result == []
