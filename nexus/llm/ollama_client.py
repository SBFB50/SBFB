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
# OOM sentinel — raised internally to short-circuit retries
# ---------------------------------------------------------------------------

class _OOMError(Exception):
    """Raised when Ollama reports out-of-memory so retry logic can fast-fail."""

    def __init__(self, model: str, message: str) -> None:
        self.model = model
        super().__init__(message)


# ---------------------------------------------------------------------------
# Retry policy — 3 attempts, exponential backoff (1s → 2s → 4s)
# Retries on transient network / timeout errors only.
# Does NOT retry on _OOMError (handled inline via _oom_retried flag).
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


def _extract_balanced(text: str, open_ch: str, close_ch: str) -> str | None:
    """Extract the first balanced ``open_ch … close_ch`` block from *text*.

    Handles arbitrary nesting depth (unlike a one-level regex).
    Respects JSON string literals so braces inside quotes are skipped.
    Returns ``None`` if no balanced block is found.
    """
    start = text.find(open_ch)
    if start == -1:
        return None
    depth = 0
    in_string = False
    escape = False
    for i in range(start, len(text)):
        ch = text[i]
        if escape:
            escape = False
            continue
        if ch == "\\":
            escape = True
            continue
        if ch == '"':
            in_string = not in_string
            continue
        if in_string:
            continue
        if ch == open_ch:
            depth += 1
        elif ch == close_ch:
            depth -= 1
            if depth == 0:
                return text[start : i + 1]
    return None


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
        _oom_retried: bool = False,
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
            response = await self._client.generate(
                model=model,
                prompt=prompt,
                system=system or "",
                format=format or "",
                options={"num_ctx": 16384},
                keep_alive=keep_alive or "10m",
            )
            text: str = response.response or ""
            logger.debug(
                "generate | model={} response_len={} eval_tokens={}",
                model,
                len(text),
                response.eval_count,
            )
            return text

        except ResponseError as exc:
            if self._is_oom(exc):
                if _oom_retried:
                    logger.error(
                        "OOM on model '{}' after VRAM recovery — "
                        "failing fast, no more retries.",
                        model,
                    )
                    raise _OOMError(model, str(exc)) from exc
                logger.warning(
                    "OOM on model '{}'. Unloading other models and retrying once...",
                    model,
                )
                await self._unload_all_except(model)
                return await self.generate(
                    model=model,
                    prompt=prompt,
                    system=system,
                    format=format,
                    timeout=timeout,
                    keep_alive=keep_alive,
                    _oom_retried=True,
                )
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

        Returns the parsed dict.  If the model returns invalid JSON,
        attempts regex extraction of ``{...}`` / ``[...]`` blocks.
        Returns ``{}`` as a last resort (never raises on parse failure).
        """
        raw = await self.generate(
            model=model,
            prompt=prompt,
            system=system,
            format="json",
            timeout=timeout,
        )
        return self._safe_parse_json(raw, model)

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
        _oom_retried: bool = False,
    ) -> str:
        """Generate text from a prompt + image using a vision-capable model.

        Reads the image file, encodes it as base64, and sends it via
        the Ollama chat endpoint with the ``images`` field.
        The *timeout* parameter is enforced via ``asyncio.wait_for``.

        Args:
            keep_alive: How long to keep the model loaded (e.g., "10m", "5m").
                        Defaults to "10m" if not specified.
        """
        image_bytes = Path(image_path).read_bytes()
        image_b64 = base64.b64encode(image_bytes).decode("utf-8")

        logger.debug(
            "generate_with_image | model={} prompt_len={} image={} timeout={}s keep_alive={}",
            model,
            len(prompt),
            Path(image_path).name,
            timeout,
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
                    options={"num_ctx": 16384},
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
                "Vision timeout: model='{}' exceeded {}s for image '{}'",
                model,
                timeout,
                Path(image_path).name,
            )
            return ""
        except ResponseError as exc:
            if self._is_oom(exc):
                if _oom_retried:
                    logger.error(
                        "OOM on vision model '{}' after VRAM recovery — "
                        "failing fast, no more retries.",
                        model,
                    )
                    raise _OOMError(model, str(exc)) from exc
                logger.warning(
                    "OOM on vision model '{}'. Unloading other models "
                    "and retrying once...",
                    model,
                )
                await self._unload_all_except(model)
                return await self.generate_with_image(
                    model=model,
                    prompt=prompt,
                    image_path=image_path,
                    system=system,
                    timeout=timeout,
                    keep_alive=keep_alive,
                    _oom_retried=True,
                )
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
            response = await self._client.embed(
                model=model,
                input=text,
                keep_alive=keep_alive or "30m",
            )
            return list(response.embeddings[0])
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
            response = await self._client.embed(
                model=model,
                input=texts,
                keep_alive=keep_alive or "30m",
            )
            return [list(v) for v in response.embeddings]
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
    def _is_oom(exc: ResponseError) -> bool:
        """Return True if the ResponseError indicates an out-of-memory condition."""
        msg = str(exc).lower()
        return "out of memory" in msg or "oom" in msg

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

    async def _unload_all_except(self, keep_model: str) -> None:
        """Unload every loaded model except *keep_model* to reclaim VRAM."""
        try:
            response = await self._client.ps()
            loaded = [m.model for m in response.models if m.model != keep_model]
        except Exception as exc:
            logger.warning("Could not list running models for VRAM recovery: {}", exc)
            return

        if not loaded:
            logger.info("No other models loaded — nothing to unload for VRAM recovery.")
            return

        logger.info("VRAM recovery: unloading {} model(s): {}", len(loaded), loaded)
        for m in loaded:
            await self.unload_model(m)

    @staticmethod
    def _safe_parse_json(raw: str, model: str) -> dict:
        """Parse *raw* as JSON, falling back to brace-matching extraction.

        Returns ``{}`` if all parsing attempts fail (never raises).
        """
        if not raw or not raw.strip():
            logger.warning(
                "generate_json: empty response from model '{}', returning {{}}",
                model,
            )
            return {}

        # --- Attempt 1: direct parse ---
        try:
            result = json.loads(raw)
            if isinstance(result, dict):
                return result
            if isinstance(result, list):
                # Wrap bare list so callers always get a dict
                return {"items": result}
            # Scalar — wrap it
            return {"value": result}
        except (json.JSONDecodeError, ValueError):
            pass

        # --- Attempt 2: brace-matched extraction of first JSON object ---
        extracted = _extract_balanced(raw, "{", "}")
        if extracted:
            try:
                return json.loads(extracted)
            except (json.JSONDecodeError, ValueError):
                pass

        # --- Attempt 3: bracket-matched extraction of first JSON array ---
        extracted = _extract_balanced(raw, "[", "]")
        if extracted:
            try:
                result = json.loads(extracted)
                return {"items": result} if isinstance(result, list) else {}
            except (json.JSONDecodeError, ValueError):
                pass

        # --- All attempts failed ---
        logger.warning(
            "generate_json: could not parse JSON from model '{}'. "
            "Raw response (first 500 chars): {}",
            model,
            raw[:500],
        )
        return {}
