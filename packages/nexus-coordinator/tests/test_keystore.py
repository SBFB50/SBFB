"""Keystore tests: generate, reload, and perm enforcement.

These tests exercise the real nexus_core.load_or_generate_secret
binding, so they implicitly verify the Day 0 PyO3 wheel is
installed correctly in the dev venv.
"""

from __future__ import annotations

import os
import platform
import stat
from pathlib import Path

import pytest
from nexus_coordinator.keystore import load_or_generate_keypair


def test_first_call_generates_file(tmp_path: Path) -> None:
    key_path = tmp_path / "coord.key"
    assert not key_path.exists()

    kp = load_or_generate_keypair(key_path)

    assert key_path.exists()
    assert len(kp.secret) == 32
    assert len(kp.public) == 32
    assert kp.path == key_path


def test_second_call_reloads_same_key(tmp_path: Path) -> None:
    key_path = tmp_path / "coord.key"
    a = load_or_generate_keypair(key_path)
    b = load_or_generate_keypair(key_path)
    assert a.secret == b.secret
    assert a.public == b.public


def test_key_file_is_owner_only_on_posix(tmp_path: Path) -> None:
    if platform.system() == "Windows":
        pytest.skip("Windows relies on profile ACLs, not POSIX perms")
    key_path = tmp_path / "coord.key"
    load_or_generate_keypair(key_path)
    mode = stat.S_IMODE(os.stat(key_path).st_mode)
    assert mode == 0o600, f"expected 0o600, got {oct(mode)}"


def test_parent_dir_is_created(tmp_path: Path) -> None:
    key_path = tmp_path / "some" / "nested" / "path" / "coord.key"
    assert not key_path.parent.exists()
    load_or_generate_keypair(key_path)
    assert key_path.exists()
    assert key_path.parent.is_dir()
