# SPDX-License-Identifier: AGPL-3.0-or-later
"""Contract tests — DispatchHook ABC + HookRunner + 5 lifecycle events.

Sprint 24 Phase C: A1 TaskDispatchHooks.
"""

from __future__ import annotations

from unittest.mock import AsyncMock, patch

import pytest
from nexus_coordinator.hooks import (
    DispatchHook,
    HookContext,
    HookRunner,
)

# ---------------------------------------------------------------------------
# Test helpers
# ---------------------------------------------------------------------------


class RecordingHook(DispatchHook):
    def __init__(self, label: str = "recorder") -> None:
        self._label = label
        self.events: list[HookContext] = []

    @property
    def name(self) -> str:
        return self._label

    async def __call__(self, ctx: HookContext) -> None:
        self.events.append(ctx)


class FailingHook(DispatchHook):
    @property
    def name(self) -> str:
        return "failing_hook"

    async def __call__(self, ctx: HookContext) -> None:
        raise RuntimeError("boom")


# ---------------------------------------------------------------------------
# 1-2: ABC + HookContext
# ---------------------------------------------------------------------------


def test_hook_abc_not_instantiable() -> None:
    with pytest.raises(TypeError):
        DispatchHook()  # type: ignore[abstract]


def test_hook_context_fields() -> None:
    ctx = HookContext(event="on_task_dispatched", task_id="t-1", timestamp=1.0, metadata={"k": "v"})
    assert ctx.event == "on_task_dispatched"
    assert ctx.task_id == "t-1"
    assert ctx.timestamp == 1.0
    assert ctx.metadata == {"k": "v"}


# ---------------------------------------------------------------------------
# 3-6: HookRunner behaviour
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_runner_single_hook_fires() -> None:
    hook = RecordingHook()
    runner = HookRunner([hook])
    await runner.fire("on_task_dispatched", task_id="t-1")
    assert len(hook.events) == 1
    assert hook.events[0].event == "on_task_dispatched"
    assert hook.events[0].task_id == "t-1"
    assert hook.events[0].timestamp > 0


@pytest.mark.asyncio
async def test_runner_multi_hook_ordering() -> None:
    calls: list[str] = []

    class OrderHook(DispatchHook):
        def __init__(self, label: str) -> None:
            self._label = label

        @property
        def name(self) -> str:
            return self._label

        async def __call__(self, ctx: HookContext) -> None:
            calls.append(self._label)

    runner = HookRunner([OrderHook("first"), OrderHook("second"), OrderHook("third")])
    await runner.fire("on_task_dispatched")
    assert calls == ["first", "second", "third"]


@pytest.mark.asyncio
async def test_runner_error_resilience() -> None:
    after = RecordingHook("after_fail")
    runner = HookRunner([FailingHook(), after])
    await runner.fire("on_task_dispatched", task_id="t-1")
    assert len(after.events) == 1


@pytest.mark.asyncio
async def test_runner_no_hooks_noop() -> None:
    runner = HookRunner([])
    await runner.fire("on_task_dispatched", task_id="t-1")


# ---------------------------------------------------------------------------
# 7-11: Specific event strings
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_on_claim_broadcast_fires() -> None:
    hook = RecordingHook()
    runner = HookRunner([hook])
    await runner.fire("on_claim_broadcast", task_id="t-1", metadata={"worker_pubkey_hex": "aa" * 32})
    assert hook.events[0].event == "on_claim_broadcast"
    assert hook.events[0].metadata["worker_pubkey_hex"] == "aa" * 32


@pytest.mark.asyncio
async def test_on_task_dispatched_fires() -> None:
    hook = RecordingHook()
    runner = HookRunner([hook])
    await runner.fire("on_task_dispatched", task_id="t-2", metadata={"task_type": "analysis"})
    assert hook.events[0].event == "on_task_dispatched"
    assert hook.events[0].metadata["task_type"] == "analysis"


@pytest.mark.asyncio
async def test_on_result_received_fires() -> None:
    hook = RecordingHook()
    runner = HookRunner([hook])
    await runner.fire("on_result_received", task_id="t-3", metadata={"tokens": 42})
    assert hook.events[0].event == "on_result_received"
    assert hook.events[0].metadata["tokens"] == 42


@pytest.mark.asyncio
async def test_on_validator_post_task_fires() -> None:
    hook = RecordingHook()
    runner = HookRunner([hook])
    await runner.fire("on_validator_post_task", task_id="t-4", metadata={"outcome": "ok"})
    assert hook.events[0].event == "on_validator_post_task"
    assert hook.events[0].metadata["outcome"] == "ok"


@pytest.mark.asyncio
async def test_on_quarantine_enqueue_fires() -> None:
    hook = RecordingHook()
    runner = HookRunner([hook])
    await runner.fire("on_quarantine_enqueue", task_id="t-5", metadata={"reason": "invisible_text"})
    assert hook.events[0].event == "on_quarantine_enqueue"
    assert hook.events[0].metadata["reason"] == "invisible_text"


# ---------------------------------------------------------------------------
# 12: Dispatcher integration
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_dispatcher_integration_hooks(tmp_path) -> None:
    from nexus_coordinator.dispatcher import Dispatcher, SubmitRequest

    hook = RecordingHook()
    runner = HookRunner([hook])

    mock_doc = AsyncMock()

    with patch("nexus_coordinator.dispatcher.nexus_core") as mock_nc:
        mock_nc.sign_task.return_value = '{"task":{},"author_pubkey":[],"signature":""}'

        dispatcher = Dispatcher(
            db_path=tmp_path / "test.db",
            doc=mock_doc,
            author_id="test-author",
            coord_secret=b"\x00" * 32,
            hook_runner=runner,
        )
        await dispatcher.init()

        task_id = await dispatcher.submit(
            SubmitRequest(
                task_type="test",
                prompt="hello",
                model="test-model",
            )
        )

    assert len(hook.events) == 1
    assert hook.events[0].event == "on_task_dispatched"
    assert hook.events[0].task_id == task_id
    assert hook.events[0].metadata["task_type"] == "test"
    assert hook.events[0].metadata["model"] == "test-model"
