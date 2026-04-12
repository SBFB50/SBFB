# SPDX-License-Identifier: AGPL-3.0-or-later
"""Config load/save and env-override roundtrip tests."""

from __future__ import annotations

from pathlib import Path

from nexus_coordinator.config import CoordinatorConfig


def test_defaults_produce_sensible_values() -> None:
    cfg = CoordinatorConfig()
    assert cfg.network.api_host == "127.0.0.1"
    assert cfg.network.api_port == 8765
    assert cfg.network.visibility == "private"
    assert cfg.policy.claim_timeout_secs >= 10
    assert cfg.identity.author_id is None
    assert cfg.identity.doc_id is None


def test_save_and_load_roundtrip(tmp_path: Path) -> None:
    cfg = CoordinatorConfig()
    cfg.identity.name = "demo"
    cfg.identity.description = "round-trip test"
    cfg.identity.author_id = "a" * 64
    cfg.identity.doc_id = "b" * 64
    cfg.network.api_port = 9001
    cfg.network.visibility = "public"
    cfg.policy.claim_timeout_secs = 600

    target = tmp_path / "coordinator.toml"
    cfg.save(target)
    assert target.exists()

    reloaded = CoordinatorConfig.load(target)
    assert reloaded.identity.name == "demo"
    assert reloaded.identity.description == "round-trip test"
    assert reloaded.identity.author_id == "a" * 64
    assert reloaded.identity.doc_id == "b" * 64
    assert reloaded.network.api_port == 9001
    assert reloaded.network.visibility == "public"
    assert reloaded.policy.claim_timeout_secs == 600


def test_load_missing_file_returns_defaults(tmp_path: Path) -> None:
    # Nothing written yet — load should fall back to pure defaults.
    cfg = CoordinatorConfig.load(tmp_path / "does-not-exist.toml")
    assert cfg.network.api_port == 8765
    assert cfg.identity.author_id is None


def test_env_override_wins_over_defaults(monkeypatch) -> None:
    # Nested delimiter is `__`, prefix is `NEXUS_COORD__`.
    monkeypatch.setenv("NEXUS_COORD__NETWORK__API_PORT", "12345")
    monkeypatch.setenv("NEXUS_COORD__NETWORK__VISIBILITY", "public")
    cfg = CoordinatorConfig()
    assert cfg.network.api_port == 12345
    assert cfg.network.visibility == "public"
