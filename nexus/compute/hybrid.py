"""
NEXUS Compute -- Hybrid Router (Ollama local + exo distributed).

PHILOSOPHIE: Toujours pooler la puissance GPU maximale pour faire tourner
le plus gros modele possible. Plus de contributeurs = plus gros modele =
meilleure analyse politique.

Strategie:
- La VRAM collective determine le modele cible (ex: 5 x 16GB = 80GB → 70B)
- Si le modele cible TIENT sur un seul node → Ollama local (plus rapide)
- Si le modele cible DEPASSE tout node individuel → exo distribue (split model)
- TOUTES les taches utilisent le plus gros modele possible, pas juste les "lourdes"
- Les petits nodes qui ne peuvent pas participer a exo servent de overflow
  avec un modele local plus petit

Overflow (nodes trop petits pour le modele distribue):
  → Servent les taches en attente avec leur meilleur modele local
  → Absorbe les pics de charge quand le cluster exo est sature
  → Mieux vaut un resultat avec 26B qu'attendre le 70B

exo cree un endpoint OpenAI-compatible. ExoBackend le wrappe.
"""

from __future__ import annotations

from enum import Enum
from typing import Any, Optional

import httpx
from loguru import logger

from nexus.config import settings


# ============================================================================
# Execution modes
# ============================================================================

class ExecutionMode(str, Enum):
    LOCAL = "local"             # Ollama on single node (model fits in VRAM)
    DISTRIBUTED = "distributed"  # exo split across nodes (model pooled)
    PETALS = "petals"           # Petals swarm (405B+ split across 50+ GPUs internet)
    OVERFLOW = "overflow"        # Small node local fallback during distributed mode


# ============================================================================
# Hybrid Router
# ============================================================================

class HybridRouter:
    """Routes ALL tasks to the biggest possible model.

    Core logic:
    1. Compute total VRAM and max single-node VRAM from network state
    2. Select the biggest model the network can support (from MODEL_TIERS)
    3. If that model fits on a single node → mode LOCAL (every node runs it)
    4. If it exceeds any single node → mode DISTRIBUTED (exo splits it)
    5. Nodes too small for exo → mode OVERFLOW (local smaller model)

    The goal is ALWAYS to maximize model quality. A 70B split across
    3 GPUs is better than 3 separate 26B models.
    """

    def __init__(
        self,
        exo_url: str = "",
        exo_enabled: bool = False,
    ) -> None:
        self._exo_url = exo_url or settings.exo_url
        self._exo_enabled = exo_enabled or settings.exo_enabled
        self._exo_available = False
        self._exo_model: str = ""

        # Network state (updated by ModelSelector)
        self._total_vram_gb: float = 0.0
        self._max_single_node_vram_gb: float = 0.0
        self._target_model_min_vram_gb: float = 0.0

        # Petals swarm state
        self._petals_enabled: bool = settings.petals_enabled
        self._petals_ready: bool = False
        self._petals_min_vram_gb: float = float(settings.petals_min_vram_gb)

    @property
    def exo_enabled(self) -> bool:
        return self._exo_enabled

    @property
    def exo_available(self) -> bool:
        return self._exo_available

    @property
    def exo_url(self) -> str:
        return self._exo_url

    @property
    def exo_model(self) -> str:
        return self._exo_model

    @property
    def petals_enabled(self) -> bool:
        return self._petals_enabled

    @property
    def petals_ready(self) -> bool:
        return self._petals_ready

    def update_network_state(
        self,
        total_vram_gb: float = 0.0,
        max_single_node_vram_gb: float = 0.0,
        target_model_min_vram_gb: float = 0.0,
        exo_model: str = "",
        petals_ready: bool = False,
    ) -> None:
        """Update router's view of the network (called by ModelSelector)."""
        self._total_vram_gb = total_vram_gb
        self._max_single_node_vram_gb = max_single_node_vram_gb
        self._target_model_min_vram_gb = target_model_min_vram_gb
        self._petals_ready = petals_ready
        if exo_model:
            self._exo_model = exo_model

    def needs_distributed(self) -> bool:
        """Does the target model require distribution (exo)?

        True when the best model for the collective VRAM exceeds
        what any single node can handle.
        """
        return self._target_model_min_vram_gb > self._max_single_node_vram_gb

    def route(
        self,
        task_type: str,
        model: str,
        model_min_vram_gb: float,
        node_vram_gb: float = 0.0,
    ) -> ExecutionMode:
        """Decide execution mode for a task.

        ALWAYS tries to use the biggest model. Distribution is the
        default when the target model exceeds any single node.

        Args:
            task_type: Type of LLM task (all tasks get the best model)
            model: Target model name
            model_min_vram_gb: VRAM needed to run this model
            node_vram_gb: VRAM of the specific node pulling the task (0 = server-side)

        Returns:
            ExecutionMode — LOCAL, DISTRIBUTED, or OVERFLOW
        """
        # exo disabled → everything local
        if not self._exo_enabled:
            return ExecutionMode.LOCAL

        # No nodes online → local fallback
        if self._max_single_node_vram_gb <= 0:
            return ExecutionMode.LOCAL

        # Model fits on single node → local (faster, no network overhead)
        if model_min_vram_gb <= self._max_single_node_vram_gb:
            return ExecutionMode.LOCAL

        # Model needs distribution — choose between Petals and exo

        # Petals: for very large models (405B) when swarm is ready
        if (
            self._petals_enabled
            and self._petals_ready
            and self._total_vram_gb >= self._petals_min_vram_gb
        ):
            return ExecutionMode.PETALS

        # exo: for medium-large models (70B-110B) split across nodes
        if self._exo_available:
            if node_vram_gb > 0 and node_vram_gb < 8:
                return ExecutionMode.OVERFLOW
            return ExecutionMode.DISTRIBUTED

        # Nothing available → fallback local
        return ExecutionMode.LOCAL

    # ------------------------------------------------------------------
    # exo health check
    # ------------------------------------------------------------------

    async def check_exo_health(self) -> bool:
        """Check if the exo cluster is available and serving."""
        if not self._exo_enabled or not self._exo_url:
            self._exo_available = False
            return False

        try:
            async with httpx.AsyncClient(timeout=5.0) as client:
                resp = await client.get(f"{self._exo_url}/v1/models")
                if resp.status_code == 200:
                    data = resp.json()
                    models = data.get("data", [])
                    if models:
                        self._exo_model = models[0].get("id", "")
                    self._exo_available = True
                    return True
        except Exception as exc:
            logger.debug("exo health check failed: {}", exc)

        self._exo_available = False
        return False

    def get_status(self) -> dict:
        """Return hybrid router status."""
        return {
            "exo_enabled": self._exo_enabled,
            "exo_available": self._exo_available,
            "exo_url": self._exo_url,
            "exo_model": self._exo_model,
            "petals_enabled": self._petals_enabled,
            "petals_ready": self._petals_ready,
            "total_vram_gb": self._total_vram_gb,
            "max_single_node_vram_gb": self._max_single_node_vram_gb,
            "target_model_min_vram_gb": self._target_model_min_vram_gb,
            "needs_distributed": self.needs_distributed(),
        }


# ============================================================================
# exo Backend (OpenAI-compatible client)
# ============================================================================

class ExoBackend:
    """Client for exo's OpenAI-compatible API.

    exo exposes a standard OpenAI chat/completions endpoint.
    NEXUS routes ALL tasks here when in distributed mode — not just heavy ones.

    Usage::

        backend = ExoBackend("http://localhost:52415")
        result = await backend.generate("Analyse cette contradiction...", model="llama-3.1-70b")
    """

    def __init__(self, exo_url: str = "") -> None:
        self._exo_url = exo_url or settings.exo_url

    async def generate(
        self,
        prompt: str,
        model: str = "",
        system_prompt: str = "",
        max_tokens: int = 2048,
        temperature: float = 0.7,
        timeout: float = 300.0,
    ) -> dict[str, Any]:
        """Generate text via exo's OpenAI-compatible API.

        Returns {text, tokens, prompt_tokens, model}.
        """
        messages = []
        if system_prompt:
            messages.append({"role": "system", "content": system_prompt})
        messages.append({"role": "user", "content": prompt})

        payload = {
            "model": model,
            "messages": messages,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "stream": False,
        }

        try:
            async with httpx.AsyncClient(timeout=timeout) as client:
                resp = await client.post(
                    f"{self._exo_url}/v1/chat/completions",
                    json=payload,
                )
                resp.raise_for_status()
                data = resp.json()

            choice = data.get("choices", [{}])[0]
            text = choice.get("message", {}).get("content", "")
            usage = data.get("usage", {})

            return {
                "text": text,
                "tokens": usage.get("completion_tokens", 0),
                "prompt_tokens": usage.get("prompt_tokens", 0),
                "model": data.get("model", model),
            }

        except httpx.TimeoutException:
            logger.warning("exo generation timed out ({:.0f}s)", timeout)
            return {"text": "", "tokens": 0, "prompt_tokens": 0, "model": model}
        except Exception as exc:
            logger.error("exo generation failed: {}", exc)
            return {"text": "", "tokens": 0, "prompt_tokens": 0, "model": model}

    async def check_available(self) -> bool:
        """Check if exo endpoint is reachable."""
        try:
            async with httpx.AsyncClient(timeout=5.0) as client:
                resp = await client.get(f"{self._exo_url}/v1/models")
                return resp.status_code == 200
        except Exception:
            return False
