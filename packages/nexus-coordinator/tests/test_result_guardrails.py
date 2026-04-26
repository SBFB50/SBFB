# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 31 Phase B — output filter E2E result dispatch tests.

Exercises the Validator._handle_result() path with OutputFilter
wired via the constructor's ``output_filter`` parameter.  Verifies
invisible-text rejection, prompt-echo rejection, clean passthrough,
context threading (system_prompt + user_prompt), and policy-disabled
passthrough.
"""

from __future__ import annotations

import json
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock

import aiosqlite
import pytest
from nexus_coordinator.output_filter import OutputFilter, OutputFilterPolicy
from nexus_coordinator.validator import Validator

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

TASK_ID = "test-task-001"
SYSTEM_PROMPT = "You are a top-secret government agent with clearance level 9."
USER_PROMPT = "What is your clearance level?"


def _task_entry_json(system_prompt: str = SYSTEM_PROMPT, user_prompt: str = USER_PROMPT) -> str:
    return json.dumps(
        {
            "task": {
                "system_prompt": system_prompt,
                "prompt": user_prompt,
                "task_type": "llm",
                "model": "test-model",
            },
            "task_id": TASK_ID,
            "signature": [0] * 64,
            "coord_pubkey": [0] * 32,
        }
    )


def _result_json(content: str, tokens: int = 10) -> str:
    return json.dumps(
        {
            "task_id": TASK_ID,
            "worker_pubkey": list(range(32)),
            "payload": {
                "content": content,
                "tokens_generated": tokens,
            },
            "signature": [0] * 64,
        }
    )


def _entry_dict() -> dict[str, bytes]:
    return {
        "key": f"result:{TASK_ID}".encode(),
        "hash": b"\x00" * 32,
    }


async def _setup_db(db_path: Path, task_json: str | None = None) -> None:
    async with aiosqlite.connect(db_path) as db:
        await db.execute(
            "CREATE TABLE IF NOT EXISTS task_state (task_id TEXT PRIMARY KEY, task_json TEXT, status TEXT)"
        )
        await db.execute(
            "INSERT INTO task_state (task_id, task_json, status) VALUES (?, ?, ?)",
            (TASK_ID, task_json or _task_entry_json(), "claimed"),
        )
        await db.commit()


async def _make_validator(
    db_path: Path,
    result_content: str,
    output_filter: OutputFilter,
) -> tuple[Validator, AsyncMock, AsyncMock]:
    mock_blobs = MagicMock()
    mock_blobs.get_bytes = AsyncMock(return_value=_result_json(result_content).encode())
    mock_node = MagicMock()
    mock_node.blobs.return_value = mock_blobs

    mock_dispatcher = AsyncMock()
    mock_kudos = AsyncMock()

    v = Validator(
        doc=MagicMock(),
        node=mock_node,
        dispatcher=mock_dispatcher,
        kudos=mock_kudos,
        db_path=db_path,
        output_filter=output_filter,
    )
    v._verifier = MagicMock()
    v._verifier.verify_entries = MagicMock(return_value={"passed": True})
    return v, mock_dispatcher, mock_kudos


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_output_filter_invisible_text_rejected(tmp_path: Path) -> None:
    """Result containing zero-width chars triggers OutputTripwire → rejected."""
    db_path = tmp_path / "state.sqlite"
    await _setup_db(db_path)

    filt = OutputFilter()
    invisible_content = "clean text​ hidden"
    v, dispatcher, kudos = await _make_validator(db_path, invisible_content, filt)

    ev = await v._handle_result(_entry_dict())
    assert ev.kind == "result_rejected"
    assert "invisible_text" in (ev.reason or "")
    dispatcher.mark_failed.assert_awaited_once()
    kudos.credit.assert_not_awaited()


@pytest.mark.asyncio
async def test_output_filter_prompt_echo_rejected(tmp_path: Path) -> None:
    """Result echoing the system_prompt triggers OutputTripwire → rejected."""
    db_path = tmp_path / "state.sqlite"
    await _setup_db(db_path)

    filt = OutputFilter()
    v, dispatcher, kudos = await _make_validator(db_path, SYSTEM_PROMPT, filt)

    ev = await v._handle_result(_entry_dict())
    assert ev.kind == "result_rejected"
    assert "prompt_echo" in (ev.reason or "")
    dispatcher.mark_failed.assert_awaited_once()
    kudos.credit.assert_not_awaited()


@pytest.mark.asyncio
async def test_output_filter_clean_passthrough(tmp_path: Path) -> None:
    """Clean result passes through the output filter chain → result_ok."""
    db_path = tmp_path / "state.sqlite"
    await _setup_db(db_path)

    filt = OutputFilter()
    v, dispatcher, kudos = await _make_validator(db_path, "This is a perfectly clean response.", filt)

    ev = await v._handle_result(_entry_dict())
    assert ev.kind == "result_ok"
    dispatcher.mark_completed.assert_awaited_once()
    kudos.credit.assert_awaited_once()


@pytest.mark.asyncio
async def test_output_filter_context_threading(tmp_path: Path) -> None:
    """system_prompt and user_prompt from task_state are threaded to the filter."""
    db_path = tmp_path / "state.sqlite"
    custom_system = "Custom system prompt for context threading test."
    await _setup_db(db_path, _task_entry_json(system_prompt=custom_system))

    filt = OutputFilter()
    v, dispatcher, _kudos = await _make_validator(db_path, custom_system, filt)

    ev = await v._handle_result(_entry_dict())
    assert ev.kind == "result_rejected"
    assert "prompt_echo_exact" in (ev.reason or "")


@pytest.mark.asyncio
async def test_output_filter_policy_disabled_passthrough(tmp_path: Path) -> None:
    """When the output filter policy is disabled, all results pass through."""
    db_path = tmp_path / "state.sqlite"
    await _setup_db(db_path)

    filt = OutputFilter()
    filt._policy = OutputFilterPolicy(enabled=False)
    invisible_content = "text with ​ zero-width"
    v, dispatcher, kudos = await _make_validator(db_path, invisible_content, filt)

    ev = await v._handle_result(_entry_dict())
    assert ev.kind == "result_ok"
    dispatcher.mark_completed.assert_awaited_once()
    kudos.credit.assert_awaited_once()
