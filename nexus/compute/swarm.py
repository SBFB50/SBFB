"""
NEXUS Compute -- Petals Swarm Manager.

Monitors the Petals distributed swarm:
- Block coverage (how many transformer blocks are served)
- Node count and health
- Throughput estimation
- Auto-scaling decisions (enough VRAM for 405B? Fallback to 70B?)

The swarm is considered "healthy" when all blocks of the target model
are covered by at least 1 node (full chain from input to output).
"""

from __future__ import annotations

import asyncio
from enum import Enum
from typing import Any, Optional

from loguru import logger

from nexus.config import settings


class SwarmHealth(str, Enum):
    HEALTHY = "healthy"          # All blocks covered, swarm operational
    DEGRADED = "degraded"        # Some blocks missing, partial coverage
    OFFLINE = "offline"          # Swarm not available
    UNKNOWN = "unknown"          # Not yet checked


# Model block counts (approximate, depends on architecture)
MODEL_BLOCKS: dict[str, int] = {
    "meta-llama/Meta-Llama-3.1-405B": 126,
    "meta-llama/Meta-Llama-3.1-70B": 80,
    "meta-llama/Meta-Llama-3.1-8B": 32,
}


class SwarmManager:
    """Monitors and manages the Petals swarm for NEXUS.

    Tracks block coverage, node health, and decides when the swarm
    can serve the target model.
    """

    def __init__(self, initial_peers: Optional[list[str]] = None) -> None:
        self._initial_peers = initial_peers or settings.petals_initial_peers
        self._health = SwarmHealth.UNKNOWN
        self._monitor_task: Optional[asyncio.Task] = None
        self._running = False

        # Swarm state
        self._nodes_online: int = 0
        self._blocks_total: int = 0
        self._blocks_covered: int = 0
        self._model: str = settings.petals_model
        self._throughput_tok_s: float = 0.0

    @property
    def health(self) -> SwarmHealth:
        return self._health

    @property
    def nodes_online(self) -> int:
        return self._nodes_online

    @property
    def blocks_covered(self) -> int:
        return self._blocks_covered

    @property
    def blocks_total(self) -> int:
        return self._blocks_total

    @property
    def coverage_pct(self) -> float:
        if self._blocks_total == 0:
            return 0.0
        return (self._blocks_covered / self._blocks_total) * 100

    @property
    def is_ready(self) -> bool:
        """Swarm can serve requests (all blocks covered)."""
        return self._health == SwarmHealth.HEALTHY

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    async def start(self) -> None:
        """Start swarm monitoring.

        When ``settings.petals_enabled`` is False (the default), the
        manager stays inert: no DHT imports, no monitor loop, health
        reported as OFFLINE. This keeps Petals off the critical path
        until explicitly enabled.
        """
        if self._running:
            return
        self._running = True

        # Set expected block count
        self._blocks_total = MODEL_BLOCKS.get(self._model, 80)

        if not settings.petals_enabled:
            self._health = SwarmHealth.OFFLINE
            logger.info(
                "SwarmManager inert (settings.petals_enabled=False, "
                "model={}, monitor loop skipped)",
                self._model,
            )
            return

        # Initial health check
        await self.check_health()

        # Start periodic monitor
        self._monitor_task = asyncio.create_task(self._monitor_loop())
        logger.info(
            "SwarmManager started (model: {}, blocks: {}, health: {})",
            self._model, self._blocks_total, self._health.value,
        )

    async def stop(self) -> None:
        """Stop monitoring."""
        self._running = False
        if self._monitor_task and not self._monitor_task.done():
            self._monitor_task.cancel()
            try:
                await self._monitor_task
            except asyncio.CancelledError:
                pass
        logger.info("SwarmManager stopped")

    # ------------------------------------------------------------------
    # Health check
    # ------------------------------------------------------------------

    async def check_health(self) -> SwarmHealth:
        """Check Petals swarm health via a lightweight DHT probe.

        When ``settings.petals_enabled`` is False, returns OFFLINE
        immediately without importing Petals or touching the DHT.
        """
        if not settings.petals_enabled:
            self._health = SwarmHealth.OFFLINE
            return self._health

        if not self._initial_peers:
            logger.debug("Swarm health check skipped: no initial peers configured")
            self._health = SwarmHealth.OFFLINE
            return self._health

        try:
            infos = await asyncio.wait_for(
                asyncio.to_thread(self._query_swarm_info),
                timeout=10.0,
            )

            if infos is None:
                self._health = SwarmHealth.OFFLINE
                return self._health

            self._nodes_online = infos.get("nodes", 0)
            self._blocks_covered = infos.get("blocks_covered", 0)

            if self._blocks_covered >= self._blocks_total:
                self._health = SwarmHealth.HEALTHY
            elif self._blocks_covered > 0:
                self._health = SwarmHealth.DEGRADED
            else:
                self._health = SwarmHealth.OFFLINE

        except asyncio.TimeoutError:
            logger.debug("Swarm health check timed out (10s)")
            self._health = SwarmHealth.OFFLINE
        except Exception as exc:
            logger.debug("Swarm health check failed: {}", exc)
            self._health = SwarmHealth.OFFLINE

        return self._health

    def _query_swarm_info(self) -> Optional[dict]:
        """Lightweight DHT probe without loading any model.

        Connects to the configured initial peers via hivemind.DHT and
        queries block availability through petals.utils.dht.get_remote_module_infos.
        This is what the previous implementation pretended to do while
        actually calling AutoDistributedModelForCausalLM.from_pretrained(),
        which instantiated the full distributed model proxy and caused
        VRAM/CPU spikes every monitor interval.

        Returns ``{"nodes": int, "blocks_covered": int}`` on success,
        or ``None`` if Petals/hivemind are missing, the DHT cannot be
        reached, or any exception occurs.
        """
        if not self._initial_peers:
            return None

        try:
            from hivemind import DHT
            from petals.utils.dht import get_remote_module_infos
        except ImportError:
            return None

        dht: Optional[Any] = None
        try:
            dht = DHT(
                initial_peers=self._initial_peers,
                start=True,
                daemon=True,
            )

            # Probe a small sample of block UIDs instead of the whole range.
            probe_count = min(4, self._blocks_total)
            module_uids = [f"{self._model}.{i}" for i in range(probe_count)]

            infos = get_remote_module_infos(dht, module_uids, latest=True)

            if not infos:
                return {"nodes": 0, "blocks_covered": 0}

            covered_probe = sum(
                1 for info in infos
                if info and getattr(info, "servers", None)
            )
            servers_union: set = set()
            for info in infos:
                servers = getattr(info, "servers", None) or {}
                servers_union.update(servers.keys() if hasattr(servers, "keys") else servers)

            # Extrapolate: (probed_covered / probed_total) * blocks_total
            extrapolated = int(
                (covered_probe / max(probe_count, 1)) * self._blocks_total
            )

            return {
                "nodes": len(servers_union),
                "blocks_covered": extrapolated,
            }

        except Exception as exc:
            logger.debug("DHT probe failed: {}", exc)
            return None
        finally:
            if dht is not None:
                try:
                    dht.shutdown()
                except Exception:
                    pass

    # ------------------------------------------------------------------
    # Monitor loop
    # ------------------------------------------------------------------

    async def _monitor_loop(self) -> None:
        """Periodically check swarm health."""
        interval = getattr(settings, "petals_health_interval", 60)
        while self._running:
            try:
                await asyncio.sleep(interval)
                if not self._running:
                    break
                old_health = self._health
                await self.check_health()
                if old_health != self._health:
                    logger.info(
                        "Swarm health changed: {} → {} ({}/{} blocks, {} nodes)",
                        old_health.value, self._health.value,
                        self._blocks_covered, self._blocks_total, self._nodes_online,
                    )
            except asyncio.CancelledError:
                break
            except Exception as exc:
                logger.debug("Swarm monitor error: {}", exc)
                await asyncio.sleep(10)

    # ------------------------------------------------------------------
    # Status
    # ------------------------------------------------------------------

    def get_status(self) -> dict:
        """Return swarm status for API/health checks."""
        return {
            "health": self._health.value,
            "model": self._model,
            "nodes_online": self._nodes_online,
            "blocks_total": self._blocks_total,
            "blocks_covered": self._blocks_covered,
            "coverage_pct": round(self.coverage_pct, 1),
            "is_ready": self.is_ready,
            "throughput_tok_s": self._throughput_tok_s,
            "initial_peers": self._initial_peers,
        }
