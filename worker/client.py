"""
nexus-worker — HTTP client for the NEXUS compute server API.

Handles all communication with the server:
- Registration
- Heartbeats
- Task pulling
- Result submission
- Stats/leaderboard queries
- Model readiness reporting
"""

from __future__ import annotations

from typing import Any, Optional

import httpx
from loguru import logger


class NexusClient:
    """HTTP client for communicating with the NEXUS compute server."""

    def __init__(self, server_url: str, api_key: str = "", timeout: float = 30.0) -> None:
        self._server_url = server_url.rstrip("/")
        self._api_key = api_key
        self._timeout = timeout
        self._http: Optional[httpx.AsyncClient] = None

    @property
    def server_url(self) -> str:
        return self._server_url

    async def _ensure_client(self) -> httpx.AsyncClient:
        if self._http is None or self._http.is_closed:
            self._http = httpx.AsyncClient(
                base_url=self._server_url,
                timeout=self._timeout,
                headers=self._auth_headers(),
            )
        return self._http

    def _auth_headers(self) -> dict[str, str]:
        if self._api_key:
            return {"Authorization": f"Bearer {self._api_key}"}
        return {}

    async def close(self) -> None:
        if self._http and not self._http.is_closed:
            await self._http.aclose()
            self._http = None

    # ------------------------------------------------------------------
    # Registration (no auth needed)
    # ------------------------------------------------------------------

    async def register(
        self,
        name: str,
        gpu_model: str,
        vram_mb: int,
        platform: str = "",
        ollama_version: str = "",
        public_key_pem: str = "",
    ) -> dict[str, Any]:
        """Register this node with the server. Returns {node_id, api_key, ...}."""
        client = await self._ensure_client()
        payload: dict[str, Any] = {
            "name": name,
            "gpu_model": gpu_model,
            "vram_mb": vram_mb,
            "platform": platform,
            "ollama_version": ollama_version,
        }
        if public_key_pem:
            payload["public_key_pem"] = public_key_pem
        resp = await client.post("/api/compute/register", json=payload)
        resp.raise_for_status()
        return resp.json()

    # ------------------------------------------------------------------
    # Authenticated endpoints
    # ------------------------------------------------------------------

    async def heartbeat(self, current_model: str = "", status: str = "idle") -> dict[str, Any]:
        """Send heartbeat. Returns {status, model_required, message}."""
        client = await self._ensure_client()
        resp = await client.post("/api/compute/heartbeat", json={
            "current_model": current_model,
            "status": status,
        })
        resp.raise_for_status()
        return resp.json()

    async def pull_task(self) -> Optional[dict[str, Any]]:
        """Pull next task from queue. Returns task dict or None if 204."""
        client = await self._ensure_client()
        resp = await client.get("/api/compute/task")
        if resp.status_code == 204:
            return None
        resp.raise_for_status()
        return resp.json()

    async def submit_result(
        self,
        task_id: str,
        result_text: str,
        tokens_generated: int = 0,
        generation_time_ms: int = 0,
        model_digest: str = "",
        logprobs: str = "",
        signature: str = "",
    ) -> dict[str, Any]:
        """Submit task result. Returns {accepted, task_id, message, trust_delta}."""
        client = await self._ensure_client()
        resp = await client.post("/api/compute/result", json={
            "task_id": task_id,
            "result_text": result_text,
            "tokens_generated": tokens_generated,
            "generation_time_ms": generation_time_ms,
            "model_digest": model_digest,
            "logprobs": logprobs,
            "signature": signature,
        })
        resp.raise_for_status()
        return resp.json()

    async def report_model_ready(self, model: str, model_digest: str = "") -> dict[str, Any]:
        """Report that a model has been pulled successfully."""
        client = await self._ensure_client()
        resp = await client.post("/api/compute/model/ready", json={
            "model": model,
            "model_digest": model_digest,
        })
        resp.raise_for_status()
        return resp.json()

    # ------------------------------------------------------------------
    # Public endpoints (no auth needed)
    # ------------------------------------------------------------------

    async def get_stats(self) -> dict[str, Any]:
        """Get network stats."""
        client = await self._ensure_client()
        resp = await client.get("/api/compute/stats")
        resp.raise_for_status()
        return resp.json()

    async def get_leaderboard(self, limit: int = 10) -> dict[str, Any]:
        """Get contributor leaderboard."""
        client = await self._ensure_client()
        resp = await client.get("/api/compute/leaderboard", params={"limit": limit})
        resp.raise_for_status()
        return resp.json()

    async def get_model_status(self) -> dict[str, Any]:
        """Get current model selection status."""
        client = await self._ensure_client()
        resp = await client.get("/api/compute/model/status")
        resp.raise_for_status()
        return resp.json()
