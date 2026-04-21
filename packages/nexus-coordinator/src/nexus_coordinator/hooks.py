# SPDX-License-Identifier: AGPL-3.0-or-later
"""Dispatch lifecycle hooks — fire-and-forget observer pipeline.

A1 TaskDispatchHooks: 5 lifecycle events injectable via HookRunner.
Consumer initial: DivergenceScorer (Sprint 24 Phase D).

S25 candidate events (not implemented S24, fire-and-forget only):
on_worker_timeout, on_retry (veto semantics, not fire-and-forget).
"""

from __future__ import annotations

import time
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any

import structlog

_log = structlog.get_logger(__name__)

EVENTS = frozenset(
    {
        "on_claim_broadcast",
        "on_task_dispatched",
        "on_result_received",
        "on_validator_post_task",
        "on_quarantine_enqueue",
    }
)


@dataclass
class HookContext:
    """Payload passed to every hook on each event."""

    event: str
    task_id: str = ""
    timestamp: float = 0.0
    metadata: dict[str, Any] = field(default_factory=dict)


class DispatchHook(ABC):
    """Base contract for dispatch lifecycle observers."""

    @property
    @abstractmethod
    def name(self) -> str: ...

    @abstractmethod
    async def __call__(self, ctx: HookContext) -> None: ...


class HookRunner:
    """Ordered composite — fires hooks sequentially, error-resilient."""

    def __init__(self, hooks: list[DispatchHook] | None = None) -> None:
        self._hooks: list[DispatchHook] = list(hooks) if hooks else []

    async def fire(
        self,
        event: str,
        *,
        task_id: str = "",
        metadata: dict[str, Any] | None = None,
    ) -> None:
        ctx = HookContext(
            event=event,
            task_id=task_id,
            timestamp=time.time(),
            metadata=metadata or {},
        )
        for hook in self._hooks:
            try:
                await hook(ctx)
            except Exception:
                _log.warning(
                    "hook_error",
                    hook_name=hook.name,
                    hook_event=event,
                    exc_info=True,
                )

    @property
    def hooks(self) -> list[DispatchHook]:
        return list(self._hooks)


__all__ = [
    "DispatchHook",
    "EVENTS",
    "HookContext",
    "HookRunner",
]
