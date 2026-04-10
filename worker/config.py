"""
nexus-worker — Local configuration management.

Stores credentials and settings in ~/.nexus-worker/config.json.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Optional


_CONFIG_DIR = Path.home() / ".nexus-worker"
_CONFIG_FILE = _CONFIG_DIR / "config.json"

_DEFAULTS: dict[str, Any] = {
    "server_url": "",
    "node_id": "",
    "api_key": "",
    "name": "",
    "gpu_model": "",
    "vram_mb": 0,
    "platform": "",
    "ollama_url": "http://localhost:11434",
    "poll_interval": 2.0,
    "heartbeat_interval": 15.0,
    "private_key_pem": "",
}


def _ensure_dir() -> None:
    _CONFIG_DIR.mkdir(parents=True, exist_ok=True)


def load_config() -> dict[str, Any]:
    """Load config from disk, or return defaults if missing."""
    if _CONFIG_FILE.exists():
        try:
            data = json.loads(_CONFIG_FILE.read_text(encoding="utf-8"))
            merged = {**_DEFAULTS, **data}
            return merged
        except (json.JSONDecodeError, OSError):
            pass
    return dict(_DEFAULTS)


def save_config(config: dict[str, Any]) -> None:
    """Save config to disk."""
    _ensure_dir()
    _CONFIG_FILE.write_text(
        json.dumps(config, indent=2, ensure_ascii=False),
        encoding="utf-8",
    )


def get_config_path() -> Path:
    return _CONFIG_FILE


def is_registered() -> bool:
    """Check if this worker has been registered (has api_key)."""
    config = load_config()
    return bool(config.get("api_key")) and bool(config.get("server_url"))
