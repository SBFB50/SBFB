# SPDX-License-Identifier: AGPL-3.0-or-later
"""Tor transport wrapper for the coordinator (Phase 1).

Manages the Tor availability state and provides a gate for outbound
HTTP routing. Phase 1 delivers the config infrastructure and
fallback mechanism; full HTTP-over-Tor routing is Phase 2.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

logger = logging.getLogger(__name__)


@dataclass
class TorConfig:
    """Parsed ``[tor]`` configuration block."""

    enabled: bool = False
    bootstrap_timeout_s: int = 30


@dataclass
class TorClientWrapper:
    """Coordinator-side Tor transport handle.

    Wraps the Rust ``TorTransport`` lifecycle:
    1. Parse config from ``tor.toml``
    2. Check if the ``tor`` feature is compiled into the PyO3 wheel
    3. Report availability for outbound HTTP routing decisions
    """

    config: TorConfig = field(default_factory=TorConfig)
    _feature_compiled: bool = False
    _bootstrapped: bool = False

    @classmethod
    def from_toml(cls, path: Path) -> TorClientWrapper:
        """Load config from a TOML file with a ``[tor]`` section."""
        cfg = TorConfig()
        if path.exists():
            try:
                import tomllib
            except ModuleNotFoundError:
                import tomli as tomllib  # type: ignore[no-redef]
            try:
                with open(path, "rb") as f:
                    data: dict[str, Any] = tomllib.load(f)
                tor_section = data.get("tor", {})
                cfg = TorConfig(
                    enabled=bool(tor_section.get("enabled", False)),
                    bootstrap_timeout_s=int(tor_section.get("bootstrap_timeout_s", 30)),
                )
            except Exception:
                logger.warning("failed to parse tor config at %s, using defaults", path)
        return cls(config=cfg, _feature_compiled=_check_tor_feature())

    async def bootstrap(self) -> None:
        """Attempt Tor bootstrap. No-op if disabled or feature absent."""
        if not self.config.enabled:
            logger.info("Tor transport disabled by configuration")
            return
        if not self._feature_compiled:
            logger.warning("Tor transport requested but nexus_core compiled without 'tor' feature")
            return
        logger.info(
            "Tor bootstrap requested (timeout %ds) — full wire deferred to Phase 2",
            self.config.bootstrap_timeout_s,
        )
        self._bootstrapped = False

    def is_available(self) -> bool:
        """Whether Tor-routed connections can be made right now."""
        return self._bootstrapped

    def health_check(self) -> bool:
        """Lightweight liveness probe."""
        return self.is_available()


def _check_tor_feature() -> bool:
    """Check whether the nexus_core wheel was compiled with ``tor``."""
    try:
        import nexus_core

        return nexus_core.tor_feature_compiled()
    except (ImportError, AttributeError):
        return False
