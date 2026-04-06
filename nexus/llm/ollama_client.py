"""
NEXUS -- Client Ollama unifie (async).

Utilise le SDK Python officiel `ollama` (AsyncClient) pour toutes les
interactions avec le serveur Ollama local.  Fournit retry automatique,
logging structure et gestion des erreurs (timeout, OOM, connexion).
"""

from __future__ import annotations

import asyncio
import base64
import json
from pathlib import Path
from typing import Any

import httpx
from loguru import logger
from ollama import AsyncClient, RequestError, ResponseError
from tenacity import (
    retry,
    retry_if_exception_type,
    stop_after_attempt,
    wait_exponential,
)

from nexus.config import settings

# ---------------------------------------------------------------------------
# Retry policy — 3 attempts, exponential backoff (1s → 2s → 4s)
# Retries on transient network / timeout errors only.
# ---------------------------------------------------------------------------
_RETRY_KWARGS: dict[str, Any] = dict(
    stop=stop_after_attempt(3),
    wait=wait_exponential(multiplier=1, min=1, max=8),
    retry=retry_if_exception_type((httpx.ConnectError, httpx.TimeoutException)),
    reraise=True,
    before_sleep=lambda rs: logger.warning(
        "Ollama retry #{} after {} — {}",
        rs.attempt_number,
        rs.outcome.exception().__class__.__name__,
        rs.outcome.exception(),
    ),
)


class OllamaClient:
    """Async wrapper around the official Ollama Python SDK."""

    def __init__(self, base_url: str | None = None) -> None:
        self._base_url = base_url or settings.ollama_base_url
        self._client = AsyncClient(host=self._base_url)
        logger.info("OllamaClient initialise (host={})", self._base_url)

    # ------------------------------------------------------------------
    # Text generation
    # ------------------------------------------------------------------

    @retry(**_RETRY_KWARGS)
    async def generate(
        self,
        model: str,
        prompt: str,
        *,
        system: str | None = None,
        format: str | None = None,
        timeout: float = 300,
        keep_alive: str | None = None,
    ) -> str:
        """Generate a text completion.

        Returns the raw response text.

        Args:
            keep_alive: How long to keep the model loaded (e.g., "10m", "3m").
                        Defaults to "10m" if not specified.
        """
        logger.debug(
            "generate | model={} prompt_len={} format={} keep_alive={}",
            model,
            len(prompt),
            format,
            keep_alive or "10m",
        )
        try:
            response = await asyncio.wait_for(
                self._client.generate(
                    model=model,
                    prompt=prompt,
                    system=system or "",
                    format=format or "",
                    options={"num_ctx": 8192},
                    keep_alive=keep_alive or "10m",
                ),
                timeout=timeout,
            )
            text: str = response.response or ""
            logger.debug(
                "generate | model={} response_len={} eval_tokens={}",
                model,
                len(text),
                response.eval_count,
            )
            return text

        except asyncio.TimeoutError:
            logger.error(
                "Ollama generate timeout after {}s (model={})", timeout, model,
            )
            raise httpx.TimeoutException(
                f"Ollama generate timed out after {timeout}s for model {model}"
            )
        except ResponseError as exc:
            self._handle_response_error(exc, model)
            raise
        except RequestError as exc:
            logger.error("Ollama request error (model={}): {}", model, exc)
            raise

    @retry(**_RETRY_KWARGS)
    async def generate_json(
        self,
        model: str,
        prompt: str,
        *,
        system: str | None = None,
        timeout: float = 300,
    ) -> dict:
        """Generate a response and force JSON output format.

        Returns the parsed dict.  Raises ``json.JSONDecodeError`` if the
        model returns invalid JSON despite the format constraint.
        """
        raw = await self.generate(
            model=model,
            prompt=prompt,
            system=system,
            format="json",
            timeout=timeout,
        )
        return json.loads(raw)

    # ------------------------------------------------------------------
    # Vision (image + text)
    # ------------------------------------------------------------------

    @retry(**_RETRY_KWARGS)
    async def generate_with_image(
        self,
        model: str,
        prompt: str,
        image_path: str | Path,
        *,
        system: str | None = None,
        timeout: float = 120,
        keep_alive: str | None = None,
    ) -> str:
        """Generate text from a prompt + image using a vision-capable model.

        Reads the image file, encodes it as base64, and sends it via
        the Ollama chat endpoint with the ``images`` field.

        Args:
            keep_alive: How long to keep the model loaded (e.g., "10m", "5m").
                        Defaults to "10m" if not specified.
        """
        image_bytes = Path(image_path).read_bytes()
        image_b64 = base64.b64encode(image_bytes).decode("utf-8")

        logger.debug(
            "generate_with_image | model={} prompt_len={} image={} keep_alive={}",
            model,
            len(prompt),
            Path(image_path).name,
            keep_alive or "10m",
        )

        messages: list[dict[str, Any]] = []
        if system:
            messages.append({"role": "system", "content": system})
        messages.append({
            "role": "user",
            "content": prompt,
            "images": [image_b64],
        })

        try:
            response = await asyncio.wait_for(
                self._client.chat(
                    model=model,
                    messages=messages,
                    options={"num_ctx": 8192},
                    keep_alive=keep_alive or "10m",
                ),
                timeout=timeout,
            )
            text: str = response.message.content or ""
            logger.debug(
                "generate_with_image | model={} response_len={}",
                model,
                len(text),
            )
            return text

        except asyncio.TimeoutError:
            logger.error(
                "Ollama vision timeout after {}s (model={})", timeout, model,
            )
            raise httpx.TimeoutException(
                f"Ollama vision timed out after {timeout}s for model {model}"
            )
        except ResponseError as exc:
            self._handle_response_error(exc, model)
            raise
        except RequestError as exc:
            logger.error("Ollama request error (model={}): {}", model, exc)
            raise

    # ------------------------------------------------------------------
    # Embeddings
    # ------------------------------------------------------------------

    @retry(**_RETRY_KWARGS)
    async def embed(
        self,
        text: str,
        *,
        model: str | None = None,
        keep_alive: str | None = None,
        timeout: float = 60,
    ) -> list[float]:
        """Embed a single text and return its vector.

        Args:
            keep_alive: How long to keep the embedding model loaded.
                        Defaults to "30m" (embeddings are cheap, stay resident).
            timeout: Maximum seconds to wait for a response.
        """
        model = model or settings.model_embedding
        logger.debug(
            "embed | model={} text_len={} keep_alive={}",
            model, len(text), keep_alive or "30m",
        )
        try:
            response = await asyncio.wait_for(
                self._client.embed(
                    model=model,
                    input=text,
                    keep_alive=keep_alive or "30m",
                ),
                timeout=timeout,
            )
            # embed() returns EmbedResponse with `.embeddings` -- list of vectors
            # For a single input, take the first vector.
            return list(response.embeddings[0])
        except asyncio.TimeoutError:
            logger.error(
                "Ollama embed timeout after {}s (model={})", timeout, model,
            )
            raise httpx.TimeoutException(
                f"Ollama embed timed out after {timeout}s for model {model}"
            )
        except ResponseError as exc:
            self._handle_response_error(exc, model)
            raise

    @retry(**_RETRY_KWARGS)
    async def embed_batch(
        self,
        texts: list[str],
        *,
        model: str | None = None,
        keep_alive: str | None = None,
        timeout: float = 120,
    ) -> list[list[float]]:
        """Embed a batch of texts in a single call.

        The Ollama ``embed`` endpoint accepts a list of strings natively.

        Args:
            keep_alive: How long to keep the embedding model loaded.
                        Defaults to "30m".
            timeout: Maximum seconds to wait for a response.
        """
        if not texts:
            return []
        model = model or settings.model_embedding
        logger.debug(
            "embed_batch | model={} count={} keep_alive={}",
            model, len(texts), keep_alive or "30m",
        )
        try:
            response = await asyncio.wait_for(
                self._client.embed(
                    model=model,
                    input=texts,
                    keep_alive=keep_alive or "30m",
                ),
                timeout=timeout,
            )
            return [list(v) for v in response.embeddings]
        except asyncio.TimeoutError:
            logger.error(
                "Ollama embed_batch timeout after {}s (model={}, count={})",
                timeout, model, len(texts),
            )
            raise httpx.TimeoutException(
                f"Ollama embed_batch timed out after {timeout}s for model {model}"
            )
        except ResponseError as exc:
            self._handle_response_error(exc, model)
            raise

    # ------------------------------------------------------------------
    # Model management
    # ------------------------------------------------------------------

    async def unload_model(self, model: str) -> bool:
        """Force-unload a model from VRAM by sending keep_alive=0.

        Returns True if the unload succeeded, False on error.
        """
        logger.info("Unloading model from VRAM: {}", model)
        try:
            await asyncio.wait_for(
                self._client.generate(
                    model=model,
                    prompt="",
                    keep_alive=0,
                ),
                timeout=30,
            )
            logger.info("Model unloaded from VRAM: {}", model)
            return True
        except asyncio.TimeoutError:
            logger.warning("Timeout unloading model: {}", model)
            return False
        except (ResponseError, RequestError) as exc:
            logger.warning("Failed to unload model {}: {}", model, exc)
            return False
        except Exception as exc:
            logger.warning("Unexpected error unloading model {}: {}", model, exc)
            return False

    # ------------------------------------------------------------------
    # Health / discovery
    # ------------------------------------------------------------------

    async def check_health(self) -> bool:
        """Return True if Ollama is reachable, False otherwise."""
        try:
            await self._client.list()
            return True
        except Exception as exc:
            logger.warning("Ollama health check failed: {}", exc)
            return False

    async def list_models(self) -> list[str]:
        """Return the names of locally available models."""
        try:
            response = await self._client.list()
            return [m.model for m in response.models if m.model]
        except Exception as exc:
            logger.error("Failed to list models: {}", exc)
            return []

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    @staticmethod
    def _handle_response_error(exc: ResponseError, model: str) -> None:
        """Log structured information about Ollama response errors."""
        msg = str(exc).lower()
        if "out of memory" in msg or "oom" in msg:
            logger.error(
                "OOM: model '{}' requires more VRAM than available. "
                "Consider unloading other models first.",
                model,
            )
        elif "not found" in msg:
            logger.error(
                "Model '{}' not found. Run: ollama pull {}",
                model,
                model,
            )
        else:
            logger.error("Ollama ResponseError (model={}): {}", model, exc)
