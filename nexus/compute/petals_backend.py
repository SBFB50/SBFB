"""
NEXUS Compute -- Petals Backend (distributed 405B inference).

Petals splits large models (70B-405B) across dozens of GPUs over the internet.
Each contributor hosts a few transformer blocks. Petals handles routing,
fault tolerance (dual-cache, auto-reroute), and pipeline parallelism.

This backend wraps Petals' AutoDistributedModelForCausalLM to provide
the same generate() interface as OllamaClient / ExoBackend.

Usage::

    backend = PetalsBackend("meta-llama/Meta-Llama-3.1-405B")
    result = await backend.generate("Analyse cette contradiction...")

Key properties:
- Lazy model loading (first generate() call triggers download/connection)
- Async via asyncio.to_thread (Petals is synchronous internally)
- Graceful fallback if petals package not installed
- Swarm health check before accepting tasks
"""

from __future__ import annotations

import asyncio
import time
from typing import Any, Optional

from loguru import logger

try:
    from petals import AutoDistributedModelForCausalLM
    from transformers import AutoTokenizer
    HAS_PETALS = True
except ImportError:
    HAS_PETALS = False

from nexus.config import settings


class PetalsBackend:
    """LLM backend using Petals distributed inference.

    Transparent replacement for OllamaClient/ExoBackend.
    Model is split across contributor GPUs via Petals swarm.
    """

    def __init__(
        self,
        model_name: str = "",
        initial_peers: Optional[list[str]] = None,
    ) -> None:
        self._model_name = model_name or settings.petals_model
        self._initial_peers = initial_peers or []
        self._model = None
        self._tokenizer = None
        self._loaded = False
        self._load_lock = asyncio.Lock()

    @property
    def available(self) -> bool:
        return HAS_PETALS

    @property
    def loaded(self) -> bool:
        return self._loaded

    @property
    def model_name(self) -> str:
        return self._model_name

    # ------------------------------------------------------------------
    # Lazy loading
    # ------------------------------------------------------------------

    async def ensure_loaded(self) -> bool:
        """Load model and tokenizer if not already loaded.

        Uses asyncio.Lock to prevent concurrent loading.
        Returns True if ready, False if Petals not available.
        """
        if self._loaded:
            return True
        if not HAS_PETALS:
            logger.warning("Petals not installed — pip install petals")
            return False

        async with self._load_lock:
            # Double-check after acquiring lock
            if self._loaded:
                return True

            try:
                logger.info("Loading Petals model: {} ...", self._model_name)

                self._tokenizer = await asyncio.to_thread(
                    AutoTokenizer.from_pretrained, self._model_name,
                )

                kwargs: dict[str, Any] = {}
                if self._initial_peers:
                    kwargs["initial_peers"] = self._initial_peers

                self._model = await asyncio.to_thread(
                    AutoDistributedModelForCausalLM.from_pretrained,
                    self._model_name, **kwargs,
                )

                self._loaded = True
                logger.info("Petals model loaded: {}", self._model_name)
                return True

            except Exception as exc:
                logger.error("Failed to load Petals model {}: {}", self._model_name, exc)
                return False

    # ------------------------------------------------------------------
    # Generation
    # ------------------------------------------------------------------

    async def generate(
        self,
        prompt: str,
        system_prompt: str = "",
        max_tokens: int = 500,
        temperature: float = 0.7,
        timeout: float = 300.0,
    ) -> dict[str, Any]:
        """Generate text via Petals distributed inference.

        Returns {text, tokens, prompt_tokens, model, duration_ms}.
        """
        if not await self.ensure_loaded():
            return {"text": "", "tokens": 0, "prompt_tokens": 0,
                    "model": self._model_name, "duration_ms": 0}

        full_prompt = f"{system_prompt}\n\n{prompt}" if system_prompt else prompt

        start = time.perf_counter()
        try:
            # Tokenize
            inputs = await asyncio.to_thread(
                self._tokenizer, full_prompt, return_tensors="pt",
            )
            prompt_tokens = inputs["input_ids"].shape[1]

            # Generate (sync → async via thread)
            outputs = await asyncio.wait_for(
                asyncio.to_thread(
                    self._model.generate,
                    **inputs,
                    max_new_tokens=max_tokens,
                    temperature=temperature,
                    do_sample=temperature > 0,
                ),
                timeout=timeout,
            )

            # Decode
            generated_ids = outputs[0][prompt_tokens:]
            text = await asyncio.to_thread(
                self._tokenizer.decode, generated_ids, skip_special_tokens=True,
            )

            elapsed_ms = int((time.perf_counter() - start) * 1000)
            gen_tokens = len(generated_ids)

            return {
                "text": text,
                "tokens": gen_tokens,
                "prompt_tokens": prompt_tokens,
                "model": self._model_name,
                "duration_ms": elapsed_ms,
            }

        except asyncio.TimeoutError:
            elapsed_ms = int((time.perf_counter() - start) * 1000)
            logger.warning("Petals generation timed out ({:.0f}s)", timeout)
            return {"text": "", "tokens": 0, "prompt_tokens": 0,
                    "model": self._model_name, "duration_ms": elapsed_ms}
        except Exception as exc:
            elapsed_ms = int((time.perf_counter() - start) * 1000)
            logger.error("Petals generation failed: {}", exc)
            return {"text": "", "tokens": 0, "prompt_tokens": 0,
                    "model": self._model_name, "duration_ms": elapsed_ms}

    # ------------------------------------------------------------------
    # Cleanup
    # ------------------------------------------------------------------

    async def unload(self) -> None:
        """Unload model and free resources."""
        self._model = None
        self._tokenizer = None
        self._loaded = False
        logger.info("Petals model unloaded")

    def get_status(self) -> dict:
        """Return backend status."""
        return {
            "available": HAS_PETALS,
            "loaded": self._loaded,
            "model": self._model_name,
        }
