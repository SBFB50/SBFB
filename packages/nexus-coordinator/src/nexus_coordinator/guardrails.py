# SPDX-License-Identifier: AGPL-3.0-or-later
"""Guardrails pipeline — declarative composable checker chain.

Pattern: openai-agents-python v0.14.3 (@input_guardrail /
@output_guardrail + GuardrailFunctionOutput + tripwire exceptions).
Design doc: docs/security/GUARDRAILS_ARCHITECTURE.md.
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any

import structlog

_log = structlog.get_logger(__name__)


@dataclass
class GuardrailContext:
    """Shared context passed to every guardrail in the chain."""

    task_id: str = ""
    system_prompt: str = ""
    user_prompt: str = ""
    metadata: dict[str, Any] = field(default_factory=dict)


@dataclass(frozen=True)
class GuardrailOutcome:
    """Result of a single guardrail check."""

    passed: bool
    tripwire: bool
    guardrail_name: str
    evidence: dict[str, Any] = field(default_factory=dict)
    mutated_value: str | None = None
    latency_ms: float = 0.0


class InputTripwire(Exception):
    """Raised when an input guardrail triggers a hard block."""

    def __init__(self, guardrail_name: str, evidence: dict[str, Any]) -> None:
        self.guardrail_name = guardrail_name
        self.evidence = evidence
        super().__init__(f"Input tripwire: {guardrail_name}")


class OutputTripwire(Exception):
    """Raised when an output guardrail triggers a hard block."""

    def __init__(self, guardrail_name: str, evidence: dict[str, Any]) -> None:
        self.guardrail_name = guardrail_name
        self.evidence = evidence
        super().__init__(f"Output tripwire: {guardrail_name}")


class Guardrail(ABC):
    """Base contract for all pipeline checkers."""

    @property
    @abstractmethod
    def name(self) -> str: ...

    @property
    def direction(self) -> str:
        return "input"

    @abstractmethod
    async def check(self, ctx: GuardrailContext, value: str) -> GuardrailOutcome: ...

    async def on_tripwire(self, ctx: GuardrailContext, outcome: GuardrailOutcome) -> None:
        pass


class GuardrailChain:
    """Ordered pipeline of guardrails with short-circuit on tripwire."""

    def __init__(self, guardrails: list[Guardrail] | None = None) -> None:
        self._guardrails: list[Guardrail] = list(guardrails) if guardrails else []

    async def run(self, ctx: GuardrailContext, value: str) -> str:
        """Run all guardrails in order. Returns the (possibly mutated) value.

        Short-circuits on the first tripwire: calls on_tripwire then
        raises InputTripwire or OutputTripwire depending on direction.
        """
        current = value
        for g in self._guardrails:
            outcome = await g.check(ctx, current)
            if outcome.tripwire:
                await g.on_tripwire(ctx, outcome)
                if g.direction == "output":
                    raise OutputTripwire(guardrail_name=g.name, evidence=outcome.evidence)
                raise InputTripwire(guardrail_name=g.name, evidence=outcome.evidence)
            if outcome.mutated_value is not None:
                current = outcome.mutated_value
        return current

    @property
    def guardrails(self) -> list[Guardrail]:
        return list(self._guardrails)


GUARDRAIL_STAGES: frozenset[str] = frozenset(
    {
        "on_claim_broadcast",
        "on_task_dispatched",
        "on_result_received",
        "on_validator_post_task",
        "on_quarantine_enqueue",
    }
)

StageGuardrailMap = dict[str, GuardrailChain]


def validate_stage_guard_map(stage_guards: StageGuardrailMap) -> None:
    """Raise ValueError if any key is not in GUARDRAIL_STAGES."""
    invalid = set(stage_guards.keys()) - GUARDRAIL_STAGES
    if invalid:
        raise ValueError(f"Invalid guardrail stages: {sorted(invalid)}")


__all__ = [
    "GUARDRAIL_STAGES",
    "Guardrail",
    "GuardrailChain",
    "GuardrailContext",
    "GuardrailOutcome",
    "InputTripwire",
    "OutputTripwire",
    "StageGuardrailMap",
    "validate_stage_guard_map",
]
