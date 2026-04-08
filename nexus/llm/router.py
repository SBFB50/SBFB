"""
NEXUS -- Routeur multi-modeles LLM.

Dispatche chaque tache vers le modele Ollama optimal en fonction de sa
nature (extraction legere, raisonnement, analyse profonde, embeddings).

VRAM Management (RTX 5080, 16 GB partagee):
- Un seul "gros" modele (nexus 26B, deepseek 14B) en VRAM a la fois.
- Les modeles legers (gemma4:e4b, nomic-embed-text) coexistent.
- VRAMScheduler provides priority-queue scheduling with model affinity.
- Falls back to simple asyncio.Lock if no scheduler is provided.
"""

from __future__ import annotations

import asyncio
import time
from contextlib import asynccontextmanager
from enum import Enum
from pathlib import Path
from typing import TYPE_CHECKING, Any, AsyncIterator

from loguru import logger

from nexus.config import settings
from nexus.llm.ollama_client import OllamaClient, _OOMError

if TYPE_CHECKING:
    from nexus.events.vram_scheduler import VRAMScheduler

# Threshold (seconds) after which we warn about lock contention.
_LOCK_WAIT_WARN_SECONDS = 30.0


# ---------------------------------------------------------------------------
# Task taxonomy
# ---------------------------------------------------------------------------

class TaskType(Enum):
    """Every task type the system can route to an LLM."""

    # Light tasks → gemma4:e4b  (~80 tok/s, 4B)
    ENTITY_EXTRACTION = "entity_extraction"
    QUERY_REFORMULATION = "query_reformulation"
    RESULT_FILTERING = "result_filtering"
    JSON_STRUCTURING = "json_structuring"
    EVIDENCE_SUMMARY = "evidence_summary"

    # Embeddings → nomic-embed-text
    EMBEDDING = "embedding"

    # Reasoning → deepseek-r1-abliterated:14b  (CoT, 14B)
    LOGIC_VERIFICATION = "logic_verification"
    CONTRADICTION_DETECTION = "contradiction_detection"
    TESTIMONY_COMPARISON = "testimony_comparison"

    # Deep analysis → nexus  (Gemma 4 26B Heretic)
    DEEP_ANALYSIS = "deep_analysis"
    HYPOTHESIS_SCORING = "hypothesis_scoring"
    SUSPECT_PROFILE = "suspect_profile"
    FINAL_REPORT = "final_report"
    INCREMENTAL_REEVAL = "incremental_reeval"

    # Vision tasks → gemma4:e4b (fast) or qwen3-vl:8b (deep)
    IMAGE_DESCRIPTION = "image_description"
    IMAGE_ENTITY_EXTRACTION = "image_entity_extraction"
    IMAGE_SCENE_ANALYSIS = "image_scene_analysis"
    IMAGE_COMPARISON = "image_comparison"

    # Audio → voxtral-mini:4b
    AUDIO_TRANSCRIPTION = "audio_transcription"

    # Forensic trace analysis → qwen3-vl:8b (deep vision)
    TRACE_ANALYSIS = "trace_analysis"


# ---------------------------------------------------------------------------
# Routing table
# ---------------------------------------------------------------------------

# Each entry: (settings attribute for model name, timeout in seconds, heavy?)
_ROUTE_TABLE: dict[TaskType, tuple[str, int, bool]] = {
    # --- Light / fast (gemma4:e4b) ---
    TaskType.ENTITY_EXTRACTION:    ("model_fast", 30, False),
    TaskType.QUERY_REFORMULATION:  ("model_fast", 15, False),
    TaskType.RESULT_FILTERING:     ("model_fast", 20, False),
    TaskType.JSON_STRUCTURING:     ("model_fast", 20, False),
    TaskType.EVIDENCE_SUMMARY:     ("model_fast", 30, False),

    # --- Embedding ---
    TaskType.EMBEDDING:            ("model_embedding", 10, False),

    # --- Reasoning (deepseek-r1, 14B -- heavy) ---
    TaskType.LOGIC_VERIFICATION:   ("model_reasoning", 120, True),
    TaskType.CONTRADICTION_DETECTION: ("model_reasoning", 120, True),
    TaskType.TESTIMONY_COMPARISON: ("model_reasoning", 120, True),

    # --- Deep analysis (nexus 26B -- heavy) ---
    TaskType.DEEP_ANALYSIS:        ("model_deep", 600, True),
    TaskType.HYPOTHESIS_SCORING:   ("model_deep", 600, True),
    TaskType.SUSPECT_PROFILE:      ("model_deep", 600, True),
    TaskType.FINAL_REPORT:         ("model_deep", 600, True),
    TaskType.INCREMENTAL_REEVAL:   ("model_deep", 600, True),

    # --- Vision (gemma4:e4b fast, qwen3-vl:8b deep) ---
    TaskType.IMAGE_DESCRIPTION:        ("model_vision", 60, False),
    TaskType.IMAGE_ENTITY_EXTRACTION:  ("model_vision", 60, False),
    TaskType.IMAGE_SCENE_ANALYSIS:     ("model_vision_deep", 180, True),
    TaskType.IMAGE_COMPARISON:         ("model_vision_deep", 180, True),

    # --- Audio (voxtral-mini:4b) ---
    TaskType.AUDIO_TRANSCRIPTION:      ("model_audio", 180, True),

    # --- Forensic traces (qwen3-vl:8b deep vision) ---
    TaskType.TRACE_ANALYSIS:           ("model_vision_deep", 180, True),
}

# Map model settings attributes to VRAMPriority for scheduler integration.
# Only used when a VRAMScheduler is provided.
_MODEL_ATTR_TO_PRIORITY: dict[str, str] = {
    "model_embedding": "EMBEDDING",
    "model_fast": "FAST_LLM",
    "model_vision": "FAST_LLM",
    "model_vision_deep": "VISION",
    "model_reasoning": "REASONING",
    "model_deep": "DEEP",
    "model_audio": "VISION",
}


class LLMRouter:
    """Route task requests to the appropriate Ollama model.

    Supports two VRAM management modes:

    1. **VRAMScheduler mode** (recommended): Pass a ``VRAMScheduler`` instance.
       Uses priority queues with model affinity for optimal GPU utilisation.

    2. **Legacy lock mode** (backward compatible): No scheduler provided.
       Falls back to a simple ``asyncio.Lock`` for heavy-model serialisation.

    Usage::

        # With scheduler (recommended)
        from nexus.events.vram_scheduler import VRAMScheduler
        scheduler = VRAMScheduler()
        router = LLMRouter(vram_scheduler=scheduler)

        # Without scheduler (backward compatible)
        router = LLMRouter()

        text = await router.route(TaskType.ENTITY_EXTRACTION, prompt, system=...)
        data = await router.route_json(TaskType.ENTITY_EXTRACTION, prompt)
        vec  = await router.embed("some text")
    """

    def __init__(
        self,
        client: OllamaClient | None = None,
        vram_scheduler: VRAMScheduler | None = None,
    ) -> None:
        self.client = client or OllamaClient()
        self._vram_scheduler = vram_scheduler

        # Legacy fallback: simple lock when no scheduler provided
        self._heavy_lock = asyncio.Lock()

        if vram_scheduler:
            logger.info("LLMRouter using VRAMScheduler (priority queue mode)")
        else:
            logger.info("LLMRouter using legacy asyncio.Lock (simple mode)")

    # ------------------------------------------------------------------
    # VRAM access helper
    # ------------------------------------------------------------------

    @asynccontextmanager
    async def _gpu_context(
        self, model_attr: str, model: str, heavy: bool, label: str,
    ) -> AsyncIterator[None]:
        """Unified GPU access gate.

        When a VRAMScheduler is available, delegates to it.
        Otherwise falls back to the legacy heavy lock.
        """
        if self._vram_scheduler:
            from nexus.events.vram_scheduler import VRAMPriority
            priority_name = _MODEL_ATTR_TO_PRIORITY.get(model_attr, "DEEP")
            priority = VRAMPriority[priority_name]
            async with self._vram_scheduler.gpu_access(priority, model, label):
                yield
        elif heavy:
            if self._heavy_lock.locked():
                logger.warning(
                    "VRAM lock contention: {} waiting -- another heavy model call in progress",
                    label,
                )
                t0 = time.monotonic()
            else:
                t0 = None
            async with self._heavy_lock:
                if t0 is not None:
                    waited = time.monotonic() - t0
                    if waited > _LOCK_WAIT_WARN_SECONDS:
                        logger.warning(
                            "VRAM lock: {} acquired after {:.1f}s wait",
                            label,
                            waited,
                        )
                yield
        else:
            yield

    def _get_keep_alive(self, model: str) -> str:
        """Return the appropriate keep_alive for a model.

        Only applies when VRAMScheduler is active (it manages VRAM lifetimes).
        Without a scheduler, use the default "10m".
        """
        if self._vram_scheduler:
            from nexus.events.vram_scheduler import get_keep_alive
            return get_keep_alive(model)
        return "10m"

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    async def route(
        self,
        task_type: TaskType,
        prompt: str,
        *,
        system: str | None = None,
    ) -> str:
        """Route a text-generation task and return the raw response.

        Returns an empty string on unrecoverable OOM (after VRAM recovery
        attempt) rather than crashing the caller.
        """
        model_attr, timeout, heavy = _ROUTE_TABLE[task_type]
        model: str = getattr(settings, model_attr)
        keep_alive = self._get_keep_alive(model)

        logger.info(
            "Routing {} -> {} (timeout={}s, heavy={}, keep_alive={})",
            task_type.value,
            model,
            timeout,
            heavy,
            keep_alive,
        )

        try:
            async with self._gpu_context(
                model_attr, model, heavy, f"route({task_type.value})",
            ):
                return await self.client.generate(
                    model=model,
                    prompt=prompt,
                    system=system,
                    timeout=timeout,
                    keep_alive=keep_alive,
                )
        except _OOMError:
            logger.error(
                "route({}): unrecoverable OOM on model '{}' — returning empty string",
                task_type.value,
                model,
            )
            return ""

    async def route_json(
        self,
        task_type: TaskType,
        prompt: str,
        *,
        system: str | None = None,
    ) -> dict[str, Any]:
        """Route a task that must return structured JSON.

        If the model returns unparsable JSON (generate_json falls back to
        ``{}``), a warning is logged but no exception is raised — callers
        receive the empty dict and can decide how to proceed.
        """
        model_attr, timeout, heavy = _ROUTE_TABLE[task_type]
        model: str = getattr(settings, model_attr)
        keep_alive = self._get_keep_alive(model)

        logger.info(
            "Routing JSON {} -> {} (timeout={}s, heavy={}, keep_alive={})",
            task_type.value,
            model,
            timeout,
            heavy,
            keep_alive,
        )

        try:
            async with self._gpu_context(
                model_attr, model, heavy, f"route_json({task_type.value})",
            ):
                result = await self.client.generate_json(
                    model=model,
                    prompt=prompt,
                    system=system,
                    timeout=timeout,
                )
        except _OOMError:
            logger.error(
                "route_json({}): unrecoverable OOM on model '{}' — returning {{}}",
                task_type.value,
                model,
            )
            return {}

        if not result:
            logger.warning(
                "route_json({}): model '{}' returned empty/unparsable JSON — "
                "returning {{}} to caller",
                task_type.value,
                model,
            )

        return result

    async def route_vision(
        self,
        task_type: TaskType,
        prompt: str,
        image_path: str | Path,
        *,
        system: str | None = None,
    ) -> str:
        """Route a vision task to the appropriate VLM model."""
        model_attr, timeout, heavy = _ROUTE_TABLE[task_type]
        model: str = getattr(settings, model_attr)
        keep_alive = self._get_keep_alive(model)

        logger.info(
            "Routing vision {} -> {} (timeout={}s, heavy={}, keep_alive={})",
            task_type.value,
            model,
            timeout,
            heavy,
            keep_alive,
        )

        try:
            async with self._gpu_context(
                model_attr, model, heavy, f"route_vision({task_type.value})",
            ):
                return await self.client.generate_with_image(
                    model=model,
                    prompt=prompt,
                    image_path=image_path,
                    system=system,
                    timeout=timeout,
                    keep_alive=keep_alive,
                )
        except _OOMError:
            logger.error(
                "route_vision({}): unrecoverable OOM on model '{}' — returning empty string",
                task_type.value,
                model,
            )
            return ""

    async def embed(self, text: str) -> list[float]:
        """Embed a single text using the configured embedding model."""
        keep_alive = self._get_keep_alive(settings.model_embedding)
        return await self.client.embed(text, keep_alive=keep_alive)

    async def embed_batch(self, texts: list[str]) -> list[list[float]]:
        """Embed a batch of texts using the configured embedding model."""
        keep_alive = self._get_keep_alive(settings.model_embedding)
        return await self.client.embed_batch(texts, keep_alive=keep_alive)
