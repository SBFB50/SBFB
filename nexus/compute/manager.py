"""
NEXUS Compute -- Compute Manager.

Orchestrates the distributed GPU computing system:
- Initializes the compute database
- Starts the ModelSelector (auto-scaling model selection)
- Starts the TaskDispatcher (task queue, reaper, heartbeat monitor)
- Provides the entry point for main.py lifecycle integration
"""

from __future__ import annotations

import asyncio
from typing import Any, Optional

from loguru import logger

from nexus.compute.db import ComputeDatabase, init_compute_db
from nexus.compute.dispatcher import SpotCheckCoordinator, TaskDispatcher
from nexus.compute.events import ComputeEventType, ComputeDatabaseProxy
from nexus.compute.model_selector import ModelSelector
from nexus.compute.self_worker import SelfWorker
from nexus.engine import EventBus, NexusEvent


class ComputeManager:
    """Manages the distributed GPU computing pipeline.

    Lifecycle:
        manager = ComputeManager()
        await manager.start()   # Init DB, start selector + dispatcher
        ...
        await manager.stop()    # Graceful shutdown
    """

    def __init__(self, bus: Optional[EventBus] = None) -> None:
        self._bus = bus
        self._model_selector: Optional[ModelSelector] = None
        self._dispatcher: Optional[TaskDispatcher] = None
        self._spot_check_coordinator: Optional[SpotCheckCoordinator] = None
        self._self_worker: Optional[SelfWorker] = None
        self._db_proxy: Optional[ComputeDatabaseProxy] = None
        self._running = False

    @property
    def model_selector(self) -> Optional[ModelSelector]:
        return self._model_selector

    @property
    def dispatcher(self) -> Optional[TaskDispatcher]:
        return self._dispatcher

    @property
    def spot_check_coordinator(self) -> Optional[SpotCheckCoordinator]:
        return self._spot_check_coordinator

    @property
    def self_worker(self) -> Optional[SelfWorker]:
        return self._self_worker

    @property
    def db_proxy(self) -> Optional[ComputeDatabaseProxy]:
        return self._db_proxy

    @property
    def running(self) -> bool:
        return self._running

    async def start(self) -> None:
        """Start the distributed computing system."""
        if self._running:
            logger.warning("ComputeManager already running")
            return

        logger.info("ComputeManager starting...")

        # Initialize compute tables
        await init_compute_db()

        # Create DB proxy for long-lived operations
        self._db_proxy = ComputeDatabaseProxy()

        # Create and start the model selector (auto-scaling)
        self._model_selector = ModelSelector()
        await self._model_selector.start()

        # Create and start the task dispatcher (uses model selector)
        self._dispatcher = TaskDispatcher(
            bus=self._bus,
            model_selector=self._model_selector,
        )
        await self._dispatcher.start()

        # Create and start the spot-check coordinator (consumes
        # COMPUTE_SPOT_CHECK_NEEDED and creates duplicate tasks for
        # cross-verification). Inert when no bus is wired.
        self._spot_check_coordinator = SpotCheckCoordinator(bus=self._bus)
        await self._spot_check_coordinator.start()

        # Auto-start self-worker (this server contributes its own GPU)
        self._self_worker = SelfWorker()
        await self._self_worker.start()

        self._running = True
        logger.info(
            "ComputeManager started — model: {}, tier: {}, self-worker: {}",
            self._model_selector.target_model,
            self._model_selector.target_tier,
            "active" if self._self_worker.running else "disabled (no GPU)",
        )

    async def stop(self) -> None:
        """Stop the distributed computing system."""
        if not self._running:
            return

        logger.info("ComputeManager stopping...")
        self._running = False

        if self._self_worker:
            await self._self_worker.stop()
            self._self_worker = None

        if self._spot_check_coordinator:
            await self._spot_check_coordinator.stop()
            self._spot_check_coordinator = None

        if self._dispatcher:
            await self._dispatcher.stop()
            self._dispatcher = None

        if self._model_selector:
            await self._model_selector.stop()
            self._model_selector = None

        self._db_proxy = None
        logger.info("ComputeManager stopped")

    def get_status(self) -> dict:
        """Return status for health check endpoint."""
        return {
            "running": self._running,
            "model_selector": self._model_selector.get_status() if self._model_selector else {},
            "dispatcher": self._dispatcher.get_status() if self._dispatcher else {},
            "spot_check": {
                "running": self._spot_check_coordinator.running if self._spot_check_coordinator else False,
                "created": self._spot_check_coordinator.spot_checks_created if self._spot_check_coordinator else 0,
                "skipped": self._spot_check_coordinator.spot_checks_skipped if self._spot_check_coordinator else 0,
            },
            "self_worker": self._self_worker.get_status() if self._self_worker else {},
        }
