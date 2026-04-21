# SPDX-License-Identifier: AGPL-3.0-or-later
"""Contract tests — RerunSampler + DivergenceScorer + RerunConfig.

Sprint 24 Phase D: re-run sampling + divergence detection.
"""

from __future__ import annotations

from pathlib import Path
from unittest.mock import AsyncMock

import pytest
from nexus_coordinator.hooks import HookContext, HookRunner
from nexus_coordinator.rerun import (
    DivergenceScorer,
    RerunConfig,
    RerunSampler,
)

# ---------------------------------------------------------------------------
# 1-3: RerunSampler rate behaviour
# ---------------------------------------------------------------------------


def test_sampler_rate_0_never_reruns() -> None:
    sampler = RerunSampler(sample_rate=0.0)
    assert all(not sampler.should_rerun(f"t-{i}") for i in range(1000))


def test_sampler_rate_1_always_reruns() -> None:
    sampler = RerunSampler(sample_rate=1.0)
    assert all(sampler.should_rerun(f"t-{i}") for i in range(100))


def test_sampler_rate_distribution() -> None:
    sampler = RerunSampler(sample_rate=0.05)
    hits = sum(sampler.should_rerun(f"t-{i}") for i in range(1000))
    assert 30 <= hits <= 70, f"expected ~50 re-runs, got {hits}"


# ---------------------------------------------------------------------------
# 4-5: DivergenceScorer hash comparison
# ---------------------------------------------------------------------------


def test_divergence_scorer_identical() -> None:
    h = b"\xaa" * 32
    assert DivergenceScorer.score(h, h) == 0.0


def test_divergence_scorer_mismatch() -> None:
    assert DivergenceScorer.score(b"\xaa" * 32, b"\xbb" * 32) == 1.0


# ---------------------------------------------------------------------------
# 6: Quarantine trigger on divergence
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_divergence_scorer_triggers_quarantine(tmp_path: Path) -> None:
    from nexus_coordinator.db.migrations import init_db

    db_path = tmp_path / "test.db"
    await init_db(db_path)

    import aiosqlite

    async with aiosqlite.connect(db_path) as db:
        await db.execute(
            "INSERT INTO task_state (task_id, state, task_json, task_type, model, priority, submitted_at, result_hash)"
            " VALUES (?, 'completed', '{}', 'test', 'model', 5, 1000, ?)",
            ("t-original", b"\xaa" * 32),
        )
        await db.commit()

    sampler = RerunSampler(sample_rate=1.0)
    sampler.register_rerun("t-original", "rerun-abc")

    mock_quarantine = AsyncMock()
    mock_quarantine.add = AsyncMock(return_value=1)

    scorer = DivergenceScorer(
        sampler=sampler,
        db_path=db_path,
        quarantine=mock_quarantine,
    )

    ctx = HookContext(
        event="on_result_received",
        task_id="rerun-abc",
        timestamp=1000.0,
        metadata={
            "worker_pubkey_hex": "bb" * 32,
            "result_hash": b"\xcc" * 32,
        },
    )
    await scorer(ctx)

    mock_quarantine.add.assert_called_once()
    call_kwargs = mock_quarantine.add.call_args[1]
    assert call_kwargs["topic"] == "divergence"
    assert call_kwargs["pow_status"] == "valid"


# ---------------------------------------------------------------------------
# 7: DivergenceScorer as hook in HookRunner
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_divergence_scorer_as_hook(tmp_path: Path) -> None:
    from nexus_coordinator.db.migrations import init_db

    db_path = tmp_path / "test.db"
    await init_db(db_path)

    import aiosqlite

    async with aiosqlite.connect(db_path) as db:
        await db.execute(
            "INSERT INTO task_state (task_id, state, task_json, task_type, model, priority, submitted_at, result_hash)"
            " VALUES (?, 'completed', '{}', 'test', 'model', 5, 1000, ?)",
            ("t-orig", b"\xaa" * 32),
        )
        await db.commit()

    sampler = RerunSampler(sample_rate=1.0)
    sampler.register_rerun("t-orig", "rerun-xyz")

    scorer = DivergenceScorer(sampler=sampler, db_path=db_path)
    runner = HookRunner([scorer])

    await runner.fire(
        "on_result_received",
        task_id="rerun-xyz",
        metadata={
            "worker_pubkey_hex": "cc" * 32,
            "result_hash": b"\xaa" * 32,
        },
    )


# ---------------------------------------------------------------------------
# 8: Re-run task distinct ID
# ---------------------------------------------------------------------------


def test_rerun_task_distinct_id() -> None:
    sampler = RerunSampler(sample_rate=1.0)
    id1 = sampler.make_rerun_id("t-original")
    id2 = sampler.make_rerun_id("t-original")
    assert id1 != id2
    assert id1.startswith("rerun-")
    assert id2.startswith("rerun-")
    assert sampler.get_original(id1) == "t-original"
    assert sampler.get_original(id2) == "t-original"
    assert sampler.is_rerun(id1)
    assert not sampler.is_rerun("t-original")


# ---------------------------------------------------------------------------
# 9-10: RerunConfig TOML parsing
# ---------------------------------------------------------------------------


def test_rerun_config_parse(tmp_path: Path) -> None:
    cfg_path = tmp_path / "rerun.toml"
    cfg_path.write_text("rerun_sample_rate = 0.03\n")
    config = RerunConfig.from_toml(cfg_path)
    assert config.sample_rate == pytest.approx(0.03)


def test_rerun_config_invalid_rate(tmp_path: Path) -> None:
    cfg_path = tmp_path / "rerun.toml"
    cfg_path.write_text("rerun_sample_rate = 2.5\n")
    config = RerunConfig.from_toml(cfg_path)
    assert config.sample_rate == pytest.approx(1.0)
