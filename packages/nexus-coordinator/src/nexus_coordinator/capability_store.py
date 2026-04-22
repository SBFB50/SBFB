# SPDX-License-Identifier: AGPL-3.0-or-later
"""Capabilities gate-off-by-default store + ``@require_capability`` decorator.

Sprint 25 Phase D — D5 per CAPABILITY_TOGGLES.md design.

The store manages ``~/.sbfb/capabilities.toml``:
- TOML schema ``version = 1`` with per-capability ``[capability.X]``
  sections (``enabled``, ``enabled_at``, ``enabled_by``).
- Anti-tamper ``integrity_hash`` (SHA-256 of file excluding hash line).
- On tamper detect → fallback all-OFF + structlog warning.
- On missing file → create default all-OFF + valid hash.

The ``@require_capability`` decorator gates FastAPI endpoints — 403
when the named capability is disabled.
"""

from __future__ import annotations

import hashlib
import os
import tomllib
from dataclasses import dataclass
from datetime import datetime, timezone
from functools import wraps
from pathlib import Path
from typing import Any

import structlog
import tomli_w
from fastapi import HTTPException

_log = structlog.get_logger(__name__)

KNOWN_CAPABILITIES: frozenset[str] = frozenset(
    {
        "tool_calling",
        "streaming_bridge",
        "mcp_server_expose",
        "federation_canary",
        "rag_retrieval",
        "biometric_gate",
    }
)

CAPABILITY_DESCRIPTIONS: dict[str, str] = {
    "tool_calling": "LLM tool-calling / function-calling surface (OWASP LLM06:2025 Excessive Agency).",
    "streaming_bridge": "Streaming postMessage bridge method task_submit_streaming.",
    "mcp_server_expose": "MCP server endpoint exposed to external LLM agents.",
    "federation_canary": "Cross-jurisdiction warrant canary cosigning (FROST K-of-N).",
    "rag_retrieval": "RAG retrieval from external document sources.",
    "biometric_gate": "OS-level biometric gate for T2 loopback tier.",
}


@dataclass(frozen=True, slots=True)
class CapabilityEntry:
    enabled: bool
    enabled_at: str
    enabled_by: str


def _sbfb_dir() -> Path:
    home = os.environ.get("HOME") or os.environ.get("USERPROFILE") or ""
    if not home:
        return Path.cwd() / ".sbfb"
    return Path(home) / ".sbfb"


def default_capabilities_path() -> Path:
    return _sbfb_dir() / "capabilities.toml"


def _compute_integrity_hash(file_text: str) -> str:
    lines = file_text.splitlines(keepends=True)
    filtered = [line for line in lines if not line.startswith("integrity_hash")]
    return "sha256-" + hashlib.sha256("".join(filtered).encode()).hexdigest()


class CapabilitiesStore:
    def __init__(self, path: Path, capabilities: dict[str, CapabilityEntry]) -> None:
        self._path = path
        self._capabilities = dict(capabilities)

    @classmethod
    def load(cls, path: Path | None = None) -> CapabilitiesStore:
        if path is None:
            path = default_capabilities_path()
        if not path.exists():
            store = cls._default(path)
            store._write()
            _log.info(
                "capabilities.toml created (all OFF)",
                path=str(path),
            )
            return store

        text = path.read_text(encoding="utf-8")
        try:
            data = tomllib.loads(text)
        except Exception:
            _log.warning(
                "capabilities.toml parse error, fallback all-OFF",
                path=str(path),
            )
            return cls._default(path)

        stored_hash = data.get("integrity_hash", "")
        computed = _compute_integrity_hash(text)
        if stored_hash != computed:
            _log.warning(
                "capabilities.toml integrity mismatch, fallback all-OFF",
                path=str(path),
                stored=stored_hash,
                computed=computed,
            )
            return cls._default(path)

        caps: dict[str, CapabilityEntry] = {}
        cap_section = data.get("capability", {})
        for name in sorted(KNOWN_CAPABILITIES):
            section = cap_section.get(name, {})
            caps[name] = CapabilityEntry(
                enabled=bool(section.get("enabled", False)),
                enabled_at=str(section.get("enabled_at", "")),
                enabled_by=str(section.get("enabled_by", "")),
            )
        return cls(path, caps)

    @classmethod
    def _default(cls, path: Path) -> CapabilitiesStore:
        caps = {
            name: CapabilityEntry(enabled=False, enabled_at="", enabled_by="") for name in sorted(KNOWN_CAPABILITIES)
        }
        return cls(path, caps)

    @property
    def path(self) -> Path:
        return self._path

    def is_enabled(self, cap_name: str) -> bool:
        entry = self._capabilities.get(cap_name)
        if entry is None:
            return False
        return entry.enabled

    def enable(self, cap_name: str, actor: str) -> None:
        if cap_name not in KNOWN_CAPABILITIES:
            raise ValueError(f"unknown capability: {cap_name}")
        now = datetime.now(timezone.utc).isoformat(timespec="seconds")
        previous = self.is_enabled(cap_name)
        self._capabilities[cap_name] = CapabilityEntry(
            enabled=True,
            enabled_at=now,
            enabled_by=actor,
        )
        self._write()
        _log.info(
            "capability_changed",
            capability=cap_name,
            previous=previous,
            new=True,
            actor=actor,
        )

    def disable(self, cap_name: str) -> None:
        if cap_name not in KNOWN_CAPABILITIES:
            raise ValueError(f"unknown capability: {cap_name}")
        previous = self.is_enabled(cap_name)
        self._capabilities[cap_name] = CapabilityEntry(
            enabled=False,
            enabled_at="",
            enabled_by="",
        )
        self._write()
        _log.info(
            "capability_changed",
            capability=cap_name,
            previous=previous,
            new=False,
        )

    def get(self, cap_name: str) -> CapabilityEntry | None:
        return self._capabilities.get(cap_name)

    def audit_trail(self) -> list[dict[str, Any]]:
        return [
            {
                "capability": name,
                "enabled": entry.enabled,
                "enabled_at": entry.enabled_at,
                "enabled_by": entry.enabled_by,
            }
            for name, entry in sorted(self._capabilities.items())
        ]

    def _to_dict(self) -> dict[str, Any]:
        data: dict[str, Any] = {"version": 1, "capability": {}}
        for name, entry in sorted(self._capabilities.items()):
            data["capability"][name] = {
                "enabled": entry.enabled,
                "enabled_at": entry.enabled_at,
                "enabled_by": entry.enabled_by,
            }
        return data

    def _write(self) -> None:
        data = self._to_dict()
        body = tomli_w.dumps(data)
        hash_val = "sha256-" + hashlib.sha256(body.encode()).hexdigest()
        data["integrity_hash"] = hash_val
        final = tomli_w.dumps(data)
        self._path.parent.mkdir(parents=True, exist_ok=True)
        self._path.write_text(final, encoding="utf-8")


# -- Module-level singleton for @require_capability decorator --

_store: CapabilitiesStore | None = None


def init_capabilities(path: Path | None = None) -> CapabilitiesStore:
    global _store  # noqa: PLW0603
    _store = CapabilitiesStore.load(path)
    return _store


def get_capabilities_store() -> CapabilitiesStore | None:
    return _store


def require_capability(cap_name: str):
    """FastAPI decorator — 403 when the named capability is disabled."""

    def decorator(func):
        @wraps(func)
        async def wrapper(*args, **kwargs):
            store = _store
            if store is None or not store.is_enabled(cap_name):
                raise HTTPException(
                    403,
                    detail=(
                        f"capability '{cap_name}' is disabled. "
                        f"Run `nexus-admin capability enable {cap_name}` "
                        "to activate."
                    ),
                )
            return await func(*args, **kwargs)

        return wrapper

    return decorator
