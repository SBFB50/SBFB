"""
nexus-worker — exo peer mode.

When running in exo mode, the worker contributes GPU layers to a
distributed model split across multiple machines instead of running
a complete Ollama model locally.

exo peer lifecycle:
1. Check if exo is installed
2. Start exo peer process pointing to the NEXUS swarm
3. Monitor peer health
4. Stop peer on shutdown

The server coordinates which model to split and how many layers
each peer should host based on VRAM.
"""

from __future__ import annotations

import asyncio
import shutil
import subprocess
from typing import Optional

from loguru import logger


class ExoPeer:
    """Manages the exo peer process on a contributor node.

    The peer contributes GPU layers to the distributed model.
    Communication is handled by exo's built-in peer discovery.
    """

    def __init__(
        self,
        initial_peers: str = "",
        port: int = 31330,
        num_blocks: Optional[int] = None,
    ) -> None:
        self._initial_peers = initial_peers
        self._port = port
        self._num_blocks = num_blocks  # None = auto-detect based on VRAM
        self._process: Optional[asyncio.subprocess.Process] = None
        self._running = False
        self._healthy = False

    @property
    def running(self) -> bool:
        return self._running

    @property
    def healthy(self) -> bool:
        return self._healthy

    @staticmethod
    def is_exo_installed() -> bool:
        """Check if exo is available on the system."""
        return shutil.which("exo") is not None

    async def start(self, model: str = "") -> bool:
        """Start the exo peer process.

        Returns True if started successfully.
        """
        if self._running:
            return True

        if not self.is_exo_installed():
            logger.error("exo not found. Install with: pip install exo")
            return False

        cmd = ["exo", "run"]

        if model:
            cmd.append(model)

        cmd.extend(["--port", str(self._port)])

        if self._initial_peers:
            cmd.extend(["--initial-peers", self._initial_peers])

        if self._num_blocks is not None:
            cmd.extend(["--num-blocks", str(self._num_blocks)])

        try:
            self._process = await asyncio.create_subprocess_exec(
                *cmd,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            self._running = True
            self._healthy = True

            # Start health monitor
            asyncio.create_task(self._monitor_process())

            logger.info("exo peer started (port: {}, model: {})", self._port, model or "auto")
            return True

        except Exception as exc:
            logger.error("Failed to start exo peer: {}", exc)
            return False

    async def stop(self) -> None:
        """Stop the exo peer process."""
        self._running = False
        self._healthy = False

        if self._process and self._process.returncode is None:
            self._process.terminate()
            try:
                await asyncio.wait_for(self._process.wait(), timeout=10)
            except asyncio.TimeoutError:
                self._process.kill()
                await self._process.wait()

        self._process = None
        logger.info("exo peer stopped")

    async def _monitor_process(self) -> None:
        """Monitor the exo process and detect crashes."""
        if not self._process:
            return

        try:
            returncode = await self._process.wait()
            self._healthy = False
            self._running = False
            if returncode != 0:
                stderr = ""
                if self._process.stderr:
                    stderr = (await self._process.stderr.read()).decode(errors="ignore")[:500]
                logger.warning("exo peer exited with code {} — {}", returncode, stderr)
        except asyncio.CancelledError:
            pass

    def get_status(self) -> dict:
        """Return peer status."""
        return {
            "running": self._running,
            "healthy": self._healthy,
            "port": self._port,
            "initial_peers": self._initial_peers,
            "pid": self._process.pid if self._process and self._process.returncode is None else None,
        }
