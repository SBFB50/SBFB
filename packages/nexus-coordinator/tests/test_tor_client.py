# SPDX-License-Identifier: AGPL-3.0-or-later
"""Tests for the Tor transport coordinator wrapper (Sprint 31 Phase C)."""

from __future__ import annotations

from pathlib import Path

import pytest
from nexus_coordinator.tor_client import TorClientWrapper, TorConfig


class TestTorConfig:
    def test_defaults(self) -> None:
        cfg = TorConfig()
        assert cfg.enabled is False
        assert cfg.bootstrap_timeout_s == 30

    def test_from_toml_enabled(self, tmp_path: Path) -> None:
        p = tmp_path / "tor.toml"
        p.write_text("[tor]\nenabled = true\nbootstrap_timeout_s = 45\n")
        wrapper = TorClientWrapper.from_toml(p)
        assert wrapper.config.enabled is True
        assert wrapper.config.bootstrap_timeout_s == 45

    def test_from_toml_missing_file(self, tmp_path: Path) -> None:
        wrapper = TorClientWrapper.from_toml(tmp_path / "nonexistent.toml")
        assert wrapper.config.enabled is False

    def test_from_toml_malformed(self, tmp_path: Path) -> None:
        p = tmp_path / "tor.toml"
        p.write_text("not valid toml {{{{")
        wrapper = TorClientWrapper.from_toml(p)
        assert wrapper.config.enabled is False


class TestTorClientWrapper:
    @pytest.mark.asyncio
    async def test_disabled_noop(self) -> None:
        wrapper = TorClientWrapper(config=TorConfig(enabled=False))
        await wrapper.bootstrap()
        assert wrapper.is_available() is False

    @pytest.mark.asyncio
    async def test_enabled_without_feature(self) -> None:
        wrapper = TorClientWrapper(
            config=TorConfig(enabled=True),
            _feature_compiled=False,
        )
        await wrapper.bootstrap()
        assert wrapper.is_available() is False

    def test_health_check_mirrors_available(self) -> None:
        wrapper = TorClientWrapper()
        assert wrapper.health_check() is False
        assert wrapper.health_check() == wrapper.is_available()
