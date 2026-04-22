# SPDX-License-Identifier: AGPL-3.0-or-later
"""StageGuardrailMap integration tests — Sprint 25 Phase C.

Tests the multi-stage guardrail pipeline: type contract, backward
compat (input_chain → stage_guards, output_filter → stage_guards),
chain routing per lifecycle event, tripwire propagation, error
resilience, and output safety migration from inline to chain.
"""

from __future__ import annotations

from unittest.mock import MagicMock

import pytest
from nexus_coordinator.guardrails import (
    GUARDRAIL_STAGES,
    Guardrail,
    GuardrailChain,
    GuardrailContext,
    GuardrailOutcome,
    InputTripwire,
    OutputTripwire,
    StageGuardrailMap,
)
from nexus_coordinator.hooks import EVENTS

# ---------------------------------------------------------------------------
# Test helpers
# ---------------------------------------------------------------------------


class PassGuardrail(Guardrail):
    @property
    def name(self) -> str:
        return "pass_guard"

    async def check(self, ctx: GuardrailContext, value: str) -> GuardrailOutcome:
        return GuardrailOutcome(passed=True, tripwire=False, guardrail_name=self.name)


class MutatingGuardrail(Guardrail):
    def __init__(self, suffix: str = "_mutated") -> None:
        self._suffix = suffix

    @property
    def name(self) -> str:
        return "mutating_guard"

    async def check(self, ctx: GuardrailContext, value: str) -> GuardrailOutcome:
        return GuardrailOutcome(
            passed=True,
            tripwire=False,
            guardrail_name=self.name,
            mutated_value=value + self._suffix,
        )


class InputTripGuardrail(Guardrail):
    @property
    def name(self) -> str:
        return "input_trip"

    async def check(self, ctx: GuardrailContext, value: str) -> GuardrailOutcome:
        return GuardrailOutcome(
            passed=False,
            tripwire=True,
            guardrail_name=self.name,
            evidence={"reason": "blocked"},
        )


class OutputTripGuardrail(Guardrail):
    @property
    def name(self) -> str:
        return "output_trip"

    @property
    def direction(self) -> str:
        return "output"

    async def check(self, ctx: GuardrailContext, value: str) -> GuardrailOutcome:
        return GuardrailOutcome(
            passed=False,
            tripwire=True,
            guardrail_name=self.name,
            evidence={"reason": "unsafe_output", "risk_score": 1.0},
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


class ErrorGuardrail(Guardrail):
    @property
    def name(self) -> str:
        return "error_guard"

    async def check(self, ctx: GuardrailContext, value: str) -> GuardrailOutcome:
        raise RuntimeError("guardrail internal error")


# ---------------------------------------------------------------------------
# 1-3: Type contract + stage constants
# ---------------------------------------------------------------------------


def test_guardrail_stages_match_hook_events() -> None:
    assert GUARDRAIL_STAGES == EVENTS


def test_stage_guardrail_map_is_dict_type() -> None:
    m: StageGuardrailMap = {"on_task_dispatched": GuardrailChain([])}
    assert isinstance(m, dict)


def test_guardrail_stages_contains_five_events() -> None:
    assert len(GUARDRAIL_STAGES) == 5
    assert "on_task_dispatched" in GUARDRAIL_STAGES
    assert "on_result_received" in GUARDRAIL_STAGES
    assert "on_claim_broadcast" in GUARDRAIL_STAGES
    assert "on_validator_post_task" in GUARDRAIL_STAGES
    assert "on_quarantine_enqueue" in GUARDRAIL_STAGES


# ---------------------------------------------------------------------------
# 4-6: Dispatcher backward compat + stage_guards wiring
# ---------------------------------------------------------------------------


def test_dispatcher_input_chain_wraps_to_stage_guards() -> None:
    from nexus_coordinator.dispatcher import Dispatcher

    chain = GuardrailChain([PassGuardrail()])
    d = Dispatcher(
        db_path=MagicMock(),
        doc=MagicMock(),
        author_id="test",
        coord_secret=b"\x00" * 32,
        input_chain=chain,
    )
    assert d._stage_guards.get("on_task_dispatched") is chain


def test_dispatcher_stage_guards_takes_precedence() -> None:
    from nexus_coordinator.dispatcher import Dispatcher

    old_chain = GuardrailChain([PassGuardrail()])
    new_chain = GuardrailChain([MutatingGuardrail()])
    guards: StageGuardrailMap = {"on_task_dispatched": new_chain}
    d = Dispatcher(
        db_path=MagicMock(),
        doc=MagicMock(),
        author_id="test",
        coord_secret=b"\x00" * 32,
        input_chain=old_chain,
        stage_guards=guards,
    )
    assert d._stage_guards.get("on_task_dispatched") is new_chain


def test_dispatcher_no_chain_empty_stage_guards() -> None:
    from nexus_coordinator.dispatcher import Dispatcher

    d = Dispatcher(
        db_path=MagicMock(),
        doc=MagicMock(),
        author_id="test",
        coord_secret=b"\x00" * 32,
    )
    assert d._stage_guards == {}


# ---------------------------------------------------------------------------
# 7-9: Validator backward compat + stage_guards wiring
# ---------------------------------------------------------------------------


def test_validator_output_filter_wraps_to_stage_guards() -> None:
    from nexus_coordinator.output_filter import OutputFilter
    from nexus_coordinator.validator import Validator

    filt = OutputFilter()
    v = Validator(
        doc=MagicMock(),
        node=MagicMock(),
        dispatcher=MagicMock(),
        kudos=MagicMock(),
        db_path=MagicMock(),
        output_filter=filt,
    )
    chain = v._stage_guards.get("on_result_received")
    assert chain is not None
    assert len(chain.guardrails) == 1
    assert chain.guardrails[0].name == "output_safety"


def test_validator_stage_guards_takes_precedence() -> None:
    from nexus_coordinator.output_filter import OutputFilter
    from nexus_coordinator.validator import Validator

    filt = OutputFilter()
    custom_chain = GuardrailChain([PassGuardrail()])
    guards: StageGuardrailMap = {"on_result_received": custom_chain}
    v = Validator(
        doc=MagicMock(),
        node=MagicMock(),
        dispatcher=MagicMock(),
        kudos=MagicMock(),
        db_path=MagicMock(),
        output_filter=filt,
        stage_guards=guards,
    )
    assert v._stage_guards.get("on_result_received") is custom_chain


def test_validator_no_filter_empty_stage_guards() -> None:
    from nexus_coordinator.validator import Validator

    v = Validator(
        doc=MagicMock(),
        node=MagicMock(),
        dispatcher=MagicMock(),
        kudos=MagicMock(),
        db_path=MagicMock(),
    )
    assert v._stage_guards == {}


# ---------------------------------------------------------------------------
# 10-12: Chain routing per stage event
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_stage_guards_input_chain_mutates_value() -> None:
    chain = GuardrailChain([MutatingGuardrail("_redacted")])
    ctx = GuardrailContext(task_id="t-1", user_prompt="hello")
    result = await chain.run(ctx, "sensitive data")
    assert result == "sensitive data_redacted"


@pytest.mark.asyncio
async def test_stage_guards_absent_stage_passthrough() -> None:
    guards: StageGuardrailMap = {"on_task_dispatched": GuardrailChain([PassGuardrail()])}
    result_chain = guards.get("on_result_received")
    assert result_chain is None


@pytest.mark.asyncio
async def test_stage_guards_multiple_stages_independent() -> None:
    input_calls: list[str] = []
    output_calls: list[str] = []
    guards: StageGuardrailMap = {
        "on_task_dispatched": GuardrailChain([RecordingGuardrail("input_g", input_calls)]),
        "on_result_received": GuardrailChain([RecordingGuardrail("output_g", output_calls)]),
    }
    ctx = GuardrailContext()
    await guards["on_task_dispatched"].run(ctx, "prompt")
    assert input_calls == ["input_g"]
    assert output_calls == []
    await guards["on_result_received"].run(ctx, "response")
    assert output_calls == ["output_g"]
    assert input_calls == ["input_g"]


# ---------------------------------------------------------------------------
# 13-15: Tripwire propagation
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_input_tripwire_propagates_from_stage_guard() -> None:
    guards: StageGuardrailMap = {
        "on_task_dispatched": GuardrailChain([InputTripGuardrail()]),
    }
    with pytest.raises(InputTripwire) as exc_info:
        await guards["on_task_dispatched"].run(GuardrailContext(), "bad input")
    assert exc_info.value.guardrail_name == "input_trip"
    assert exc_info.value.evidence["reason"] == "blocked"


@pytest.mark.asyncio
async def test_output_tripwire_propagates_from_stage_guard() -> None:
    guards: StageGuardrailMap = {
        "on_result_received": GuardrailChain([OutputTripGuardrail()]),
    }
    with pytest.raises(OutputTripwire) as exc_info:
        await guards["on_result_received"].run(GuardrailContext(), "unsafe output")
    assert exc_info.value.guardrail_name == "output_trip"
    assert exc_info.value.evidence["reason"] == "unsafe_output"


@pytest.mark.asyncio
async def test_tripwire_short_circuits_chain() -> None:
    calls: list[str] = []
    guards: StageGuardrailMap = {
        "on_task_dispatched": GuardrailChain(
            [
                InputTripGuardrail(),
                RecordingGuardrail("should_not_run", calls),
            ]
        ),
    }
    with pytest.raises(InputTripwire):
        await guards["on_task_dispatched"].run(GuardrailContext(), "x")
    assert "should_not_run" not in calls


# ---------------------------------------------------------------------------
# 16-18: Error resilience + edge cases
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_stage_guards_chain_error_propagates() -> None:
    guards: StageGuardrailMap = {
        "on_task_dispatched": GuardrailChain([ErrorGuardrail()]),
    }
    with pytest.raises(RuntimeError, match="guardrail internal error"):
        await guards["on_task_dispatched"].run(GuardrailContext(), "x")


@pytest.mark.asyncio
async def test_empty_chain_passthrough() -> None:
    guards: StageGuardrailMap = {
        "on_task_dispatched": GuardrailChain([]),
    }
    result = await guards["on_task_dispatched"].run(GuardrailContext(), "unchanged")
    assert result == "unchanged"


@pytest.mark.asyncio
async def test_stage_guards_empty_map_has_no_chains() -> None:
    guards: StageGuardrailMap = {}
    for stage in GUARDRAIL_STAGES:
        assert guards.get(stage) is None


# ---------------------------------------------------------------------------
# 19-21: Output safety migration from inline to chain
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_output_safety_guardrail_via_chain_clean() -> None:
    from nexus_coordinator.output_filter import OutputFilter, OutputSafetyGuardrail

    filt = OutputFilter()
    chain = GuardrailChain([OutputSafetyGuardrail(filt)])
    ctx = GuardrailContext(system_prompt="You are a helper.", user_prompt="Hello")
    result = await chain.run(ctx, "This is a clean response.")
    assert result == "This is a clean response."


@pytest.mark.asyncio
async def test_output_safety_guardrail_via_chain_tripwires() -> None:
    from nexus_coordinator.output_filter import OutputFilter, OutputSafetyGuardrail

    filt = OutputFilter()
    chain = GuardrailChain([OutputSafetyGuardrail(filt)])
    ctx = GuardrailContext(system_prompt="secret system prompt", user_prompt="q")
    invisible = "clean text ​ hidden"
    with pytest.raises(OutputTripwire) as exc_info:
        await chain.run(ctx, invisible)
    assert exc_info.value.evidence["reason"] == "invisible_text"


@pytest.mark.asyncio
async def test_output_safety_prompt_echo_via_chain() -> None:
    from nexus_coordinator.output_filter import OutputFilter, OutputSafetyGuardrail

    filt = OutputFilter()
    chain = GuardrailChain([OutputSafetyGuardrail(filt)])
    system_prompt = "You are a top-secret government agent with clearance level 9."
    ctx = GuardrailContext(system_prompt=system_prompt, user_prompt="q")
    with pytest.raises(OutputTripwire) as exc_info:
        await chain.run(ctx, system_prompt)
    assert exc_info.value.evidence["reason"] == "prompt_echo_exact"


# ---------------------------------------------------------------------------
# 22-24: Chain ordering + composition
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_chain_ordering_within_stage() -> None:
    calls: list[str] = []
    chain = GuardrailChain(
        [
            RecordingGuardrail("first", calls),
            RecordingGuardrail("second", calls),
            RecordingGuardrail("third", calls),
        ]
    )
    guards: StageGuardrailMap = {"on_task_dispatched": chain}
    await guards["on_task_dispatched"].run(GuardrailContext(), "x")
    assert calls == ["first", "second", "third"]


@pytest.mark.asyncio
async def test_mutating_chain_composition() -> None:
    chain = GuardrailChain(
        [
            MutatingGuardrail("_a"),
            MutatingGuardrail("_b"),
        ]
    )
    result = await chain.run(GuardrailContext(), "start")
    assert result == "start_a_b"


@pytest.mark.asyncio
async def test_stage_guards_can_hold_all_five_stages() -> None:
    guards: StageGuardrailMap = {stage: GuardrailChain([PassGuardrail()]) for stage in GUARDRAIL_STAGES}
    assert len(guards) == 5
    for stage in GUARDRAIL_STAGES:
        result = await guards[stage].run(GuardrailContext(), "val")
        assert result == "val"
