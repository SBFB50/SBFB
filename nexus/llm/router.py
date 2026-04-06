"""
NEXUS -- Routeur multi-modeles LLM.

Dispatche chaque tache vers le modele Ollama optimal en fonction de sa
nature (extraction legere, raisonnement, analyse profonde, embeddings).

VRAM Management (RTX 5080, 16 GB partagee):
- Un seul "gros" modele (nexus 26B, deepseek 14B) en VRAM a la fois.
- Les modeles legers (gemma4:e4b, nomic-embed-text) coexistent.
- Un ``asyncio.Lock`` serialise les appels aux gros modeles pour
  eviter les OOM et les swaps GPU incessants.
"""

from __future__ import annotations

import asyncio
import time
from enum import Enum
from pathlib import Path
from typing import Any

from loguru import logger

from nexus.config import settings
from nexus.llm.ollama_client import OllamaClient

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

    # --- Reasoning (deepseek-r1, 14B — heavy) ---
    TaskType.LOGIC_VERIFICATION:   ("model_reasoning", 120, True),
    TaskType.CONTRADICTION_DETECTION: ("model_reasoning", 120, True),
    TaskType.TESTIMONY_COMPARISON: ("model_reasoning", 120, True),

    # --- Deep analysis (nexus 26B — heavy) ---
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


class LLMRouter:
    """Route task requests to the appropriate Ollama model.

    Usage::

        router = LLMRouter()
        text = await router.route(TaskType.ENTITY_EXTRACTION, prompt, system=...)
        data = await router.route_json(TaskType.ENTITY_EXTRACTION, prompt)
        vec  = await router.embed("some text")
    """

    def __init__(self, client: OllamaClient | None = None) -> None:
        self.client = client or OllamaClient()
        # Serialises heavy-model calls so only one large model occupies
        # VRAM at a time (prevents OOM on the shared 16 GB pool).
        self._heavy_lock = asyncio.Lock()

    # ------------------------------------------------------------------
    # Lock helper
    # ------------------------------------------------------------------

    async def _acquire_heavy_lock(self, task_label: str) -> None:
        """Acquire the heavy-model lock, logging if the wait is long.

        The caller MUST still use ``async with self._heavy_lock:`` (which
        handles release on CancelledError).  This helper is only used to
        emit a warning when another heavy call is already in progress.
        """
        if self._heavy_lock.locked():
            logger.warning(
                "VRAM lock contention: {} waiting -- another heavy model call in progress",
                task_label,
            )
            t0 = time.monotonic()
            # We don't actually acquire here; the ``async with`` block does.
            # But we record the wait start so we can log duration afterwards.
            while self._heavy_lock.locked():
                await asyncio.sleep(0.25)
                elapsed = time.monotonic() - t0
                if elapsed > _LOCK_WAIT_WARN_SECONDS and int(elapsed) % 30 == 0:
                    logger.warning(
                        "VRAM lock: {} still waiting after {:.0f}s",
                        task_label,
                        elapsed,
                    )

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
        """Route a text-generation task and return the raw response."""
        model_attr, timeout, heavy = _ROUTE_TABLE[task_type]
        model: str = getattr(settings, model_attr)

        logger.info(
            "Routing {} → {} (timeout={}s, heavy={})",
            task_type.value,
            model,
            timeout,
            heavy,
        )

        if heavy:
            await self._acquire_heavy_lock(f"route({task_type.value})")
            async with self._heavy_lock:
                return await self.client.generate(
                    model=model,
                    prompt=prompt,
                    system=system,
                    timeout=timeout,
                )
        return await self.client.generate(
            model=model,
            prompt=prompt,
            system=system,
            timeout=timeout,
        )

    async def route_json(
        self,
        task_type: TaskType,
        prompt: str,
        *,
        system: str | None = None,
    ) -> dict[str, Any]:
        """Route a task that must return structured JSON."""
        model_attr, timeout, heavy = _ROUTE_TABLE[task_type]
        model: str = getattr(settings, model_attr)

        logger.info(
            "Routing JSON {} → {} (timeout={}s, heavy={})",
            task_type.value,
            model,
            timeout,
            heavy,
        )

        if heavy:
            await self._acquire_heavy_lock(f"route_json({task_type.value})")
            async with self._heavy_lock:
                return await self.client.generate_json(
                    model=model,
                    prompt=prompt,
                    system=system,
                    timeout=timeout,
                )
        return await self.client.generate_json(
            model=model,
            prompt=prompt,
            system=system,
            timeout=timeout,
        )

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

        logger.info(
            "Routing vision {} → {} (timeout={}s, heavy={})",
            task_type.value,
            model,
            timeout,
            heavy,
        )

        if heavy:
            await self._acquire_heavy_lock(f"route_vision({task_type.value})")
            async with self._heavy_lock:
                return await self.client.generate_with_image(
                    model=model,
                    prompt=prompt,
                    image_path=image_path,
                    system=system,
                    timeout=timeout,
                )
        return await self.client.generate_with_image(
            model=model,
            prompt=prompt,
            image_path=image_path,
            system=system,
            timeout=timeout,
        )

    async def embed(self, text: str) -> list[float]:
        """Embed a single text using the configured embedding model."""
        return await self.client.embed(text)

    async def embed_batch(self, texts: list[str]) -> list[list[float]]:
        """Embed a batch of texts using the configured embedding model."""
        return await self.client.embed_batch(texts)
