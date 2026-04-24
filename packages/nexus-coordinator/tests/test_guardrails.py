# SPDX-License-Identifier: AGPL-3.0-or-later
"""Contract tests — Guardrail ABC + GuardrailChain + 4 adapters.

Sprint 24 Phase B: B1 guardrails pipeline declaratif.
"""

from __future__ import annotations

import random

import pytest
from nexus_coordinator.guardrails import (
    Guardrail,
    GuardrailChain,
    GuardrailContext,
    GuardrailOutcome,
    InputTripwire,
)

# ---------------------------------------------------------------------------
# Test helpers
# ---------------------------------------------------------------------------


class PassGuardrail(Guardrail):
    @property
    def name(self) -> str:
        return "pass_guard"

    async def check(self, ctx: GuardrailContext, value: str) -> GuardrailOutcome:
        return GuardrailOutcome(passed=True, tripwire=False, guardrail_name=self.name)


class TripGuardrail(Guardrail):
    @property
    def name(self) -> str:
        return "trip_guard"

    async def check(self, ctx: GuardrailContext, value: str) -> GuardrailOutcome:
        return GuardrailOutcome(
            passed=False,
            tripwire=True,
            guardrail_name=self.name,
            evidence={"reason": "test"},
        )


class RecordingGuardrail(Guardrail):
    def __init__(self, label: str, calls: list[str]) -> None:
        self._label = label
        self._calls = calls

    @property
    def name(self) -> str:
        return self._label

    async def check(self, ctx: GuardrailContext, value: str) -> GuardrailOutcome:
        self._calls.append(self._label)
        return GuardrailOutcome(passed=True, tripwire=False, guardrail_name=self.name)


# ---------------------------------------------------------------------------
# 1-3: ABC + Outcome
# ---------------------------------------------------------------------------


def test_guardrail_abc_not_instantiable() -> None:
    with pytest.raises(TypeError):
        Guardrail()  # type: ignore[abstract]


def test_guardrail_outcome_passed() -> None:
    outcome = GuardrailOutcome(passed=True, tripwire=False, guardrail_name="test")
    assert outcome.passed is True
    assert outcome.tripwire is False
    assert outcome.mutated_value is None


def test_guardrail_outcome_tripwire() -> None:
    outcome = GuardrailOutcome(passed=False, tripwire=True, guardrail_name="test", evidence={"key": "val"})
    assert outcome.passed is False
    assert outcome.tripwire is True
    assert outcome.evidence == {"key": "val"}


# ---------------------------------------------------------------------------
# 4-8: Chain behaviour
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_chain_empty_passes() -> None:
    chain = GuardrailChain([])
    result = await chain.run(GuardrailContext(), "hello")
    assert result == "hello"


@pytest.mark.asyncio
async def test_chain_single_pass() -> None:
    chain = GuardrailChain([PassGuardrail()])
    result = await chain.run(GuardrailContext(), "hello")
    assert result == "hello"


@pytest.mark.asyncio
async def test_chain_single_trip() -> None:
    chain = GuardrailChain([TripGuardrail()])
    with pytest.raises(InputTripwire) as exc_info:
        await chain.run(GuardrailContext(), "hello")
    assert exc_info.value.guardrail_name == "trip_guard"


@pytest.mark.asyncio
async def test_chain_short_circuit() -> None:
    calls: list[str] = []
    chain = GuardrailChain([TripGuardrail(), RecordingGuardrail("should_not_run", calls)])
    with pytest.raises(InputTripwire):
        await chain.run(GuardrailContext(), "hello")
    assert "should_not_run" not in calls


@pytest.mark.asyncio
async def test_chain_ordering() -> None:
    calls: list[str] = []
    chain = GuardrailChain(
        [
            RecordingGuardrail("first", calls),
            RecordingGuardrail("second", calls),
            RecordingGuardrail("third", calls),
        ]
    )
    await chain.run(GuardrailContext(), "hello")
    assert calls == ["first", "second", "third"]


# ---------------------------------------------------------------------------
# 9-13: Per-adapter tests
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_pii_input_guardrail_redacts() -> None:
    from nexus_coordinator.pii_redactor import PiiInputGuardrail, PiiRedactor

    redactor = PiiRedactor(enable_presidio=False)
    guardrail = PiiInputGuardrail(redactor)
    outcome = await guardrail.check(GuardrailContext(), "email alice@example.com")
    assert outcome.passed is True
    assert outcome.tripwire is False
    assert outcome.mutated_value is not None
    assert "alice@example.com" not in outcome.mutated_value


@pytest.mark.asyncio
async def test_output_safety_guardrail_clean() -> None:
    from nexus_coordinator.output_filter import OutputFilter, OutputSafetyGuardrail

    guardrail = OutputSafetyGuardrail(OutputFilter())
    ctx = GuardrailContext(system_prompt="You are a helper.", user_prompt="Hello")
    outcome = await guardrail.check(ctx, "This is a clean response.")
    assert outcome.passed is True
    assert outcome.tripwire is False


@pytest.mark.asyncio
async def test_output_safety_guardrail_trip() -> None:
    from nexus_coordinator.output_filter import OutputFilter, OutputSafetyGuardrail

    guardrail = OutputSafetyGuardrail(OutputFilter())
    ctx = GuardrailContext(system_prompt="secret system prompt", user_prompt="q")
    invisible = "clean text ​ hidden"
    outcome = await guardrail.check(ctx, invisible)
    assert outcome.passed is False
    assert outcome.tripwire is True
    assert outcome.evidence["reason"] == "invisible_text"


@pytest.mark.asyncio
async def test_quarantine_guardrail_trip() -> None:
    from nexus_coordinator.quarantine_queue import QuarantineGuardrail

    guardrail = QuarantineGuardrail(condition=lambda ctx, v: "quarantine_me" in v)
    ctx = GuardrailContext(task_id="t-123")
    outcome = await guardrail.check(ctx, "this should quarantine_me now")
    assert outcome.tripwire is True
    assert outcome.evidence["task_id"] == "t-123"


@pytest.mark.asyncio
async def test_canary_input_guardrail_injects() -> None:
    from nexus_coordinator.canary_input import (
        CanaryInputGuardrail,
        CanaryInputInjector,
        CanaryInputSet,
        CanaryPrompt,
    )

    prompts = [CanaryPrompt(prompt_id="c1", prompt="What is 2+2?", expected_answer="4")]
    canary_set = CanaryInputSet(
        version=1,
        created_at_unix=0,
        prompts=prompts,
        coord_pubkey_hex="aa" * 32,
        signature_hex="bb" * 64,
    )
    injector = CanaryInputInjector(canary_set, inject_rate=1, rng=random.Random(42))
    guardrail = CanaryInputGuardrail(injector)
    outcome = await guardrail.check(GuardrailContext(), "original prompt")
    assert outcome.passed is True
    assert outcome.evidence["injected"] is True
    assert outcome.mutated_value == "What is 2+2?"


# ---------------------------------------------------------------------------
# 14-15: Integration (chain with real adapters / backward compat)
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_dispatcher_uses_input_chain() -> None:
    from nexus_coordinator.pii_redactor import PiiInputGuardrail, PiiRedactor

    redactor = PiiRedactor(enable_presidio=False)
    chain = GuardrailChain([PiiInputGuardrail(redactor)])
    result = await chain.run(
        GuardrailContext(task_id="t-test"),
        "contact alice@example.com now",
    )
    assert "alice@example.com" not in result
    assert "<EMAIL_ADDRESS_1>" in result


@pytest.mark.asyncio
async def test_dispatcher_no_chain_fallback() -> None:
    from nexus_coordinator.pii_redactor import PiiRedactor

    redactor = PiiRedactor(enable_presidio=False)
    direct = redactor.redact("contact alice@example.com now")
    assert "alice@example.com" not in direct
    assert "<EMAIL_ADDRESS_1>" in direct


# ---------------------------------------------------------------------------
# P2-STAGE-1 — StageGuardrailMap key validation (Sprint 26 Phase A)
# ---------------------------------------------------------------------------


def test_stage_guards_valid_keys_accepted() -> None:
    from nexus_coordinator.guardrails import GUARDRAIL_STAGES, validate_stage_guard_map

    valid_map = {stage: GuardrailChain([]) for stage in GUARDRAIL_STAGES}
    validate_stage_guard_map(valid_map)


def test_stage_guards_invalid_key_raises() -> None:
    from nexus_coordinator.guardrails import validate_stage_guard_map

    invalid_map = {"on_task_dispatched": GuardrailChain([]), "bogus_stage": GuardrailChain([])}
    with pytest.raises(ValueError, match="Invalid guardrail stages"):
        validate_stage_guard_map(invalid_map)
