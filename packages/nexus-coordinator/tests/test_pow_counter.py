# SPDX-License-Identifier: AGPL-3.0-or-later
"""Tests for pow_counter — Sprint 23 Phase C."""

from __future__ import annotations

from pathlib import Path
from unittest.mock import patch

import pytest
from nexus_coordinator.pow_counter import PowCounter


@pytest.fixture
async def counter(tmp_path: Path) -> PowCounter:
    c = PowCounter(tmp_path / "pow_counts.db")
    await c.open()
    yield c
    await c.close()


@pytest.mark.asyncio
async def test_increment_returns_increasing_count(counter: PowCounter) -> None:
    assert await counter.increment("alice", "llama3") == 1
    assert await counter.increment("alice", "llama3") == 2
    assert await counter.increment("alice", "llama3") == 3


@pytest.mark.asyncio
async def test_get_count_returns_zero_when_absent(counter: PowCounter) -> None:
    assert await counter.get_count("nobody", "none") == 0


@pytest.mark.asyncio
async def test_get_count_reflects_increments(counter: PowCounter) -> None:
    await counter.increment("bob", "mistral")
    await counter.increment("bob", "mistral")
    assert await counter.get_count("bob", "mistral") == 2


@pytest.mark.asyncio
async def test_per_consumer_isolation(counter: PowCounter) -> None:
    await counter.increment("alice", "llama3")
    await counter.increment("alice", "llama3")
    await counter.increment("bob", "llama3")
    assert await counter.get_count("alice", "llama3") == 2
    assert await counter.get_count("bob", "llama3") == 1


@pytest.mark.asyncio
async def test_per_model_isolation(counter: PowCounter) -> None:
    await counter.increment("alice", "llama3")
    await counter.increment("alice", "llama3")
    await counter.increment("alice", "mistral")
    assert await counter.get_count("alice", "llama3") == 2
    assert await counter.get_count("alice", "mistral") == 1


@pytest.mark.asyncio
async def test_reset_expired_clears_old_rows(counter: PowCounter) -> None:
    # Insert a row dated yesterday
    with patch("nexus_coordinator.pow_counter._today_utc", return_value="2020-01-01"):
        await counter.increment("alice", "llama3")

    # Now "today" is different → reset should clear it
    deleted = await counter.reset_expired()
    assert deleted == 1
    assert await counter.get_count("alice", "llama3") == 0


@pytest.mark.asyncio
async def test_increment_resets_stale_row(counter: PowCounter) -> None:
    # Insert row as "yesterday"
    with patch("nexus_coordinator.pow_counter._today_utc", return_value="2020-01-01"):
        await counter.increment("alice", "llama3")
        assert await counter.get_count("alice", "llama3") == 1

    # Increment today → resets to 1 (not 2)
    assert await counter.increment("alice", "llama3") == 1


@pytest.mark.asyncio
async def test_reset_expired_returns_zero_when_nothing_to_clear(
    counter: PowCounter,
) -> None:
    await counter.increment("alice", "llama3")
    deleted = await counter.reset_expired()
    assert deleted == 0
