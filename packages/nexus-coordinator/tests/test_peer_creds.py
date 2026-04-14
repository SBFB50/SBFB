# SPDX-License-Identifier: AGPL-3.0-or-later
"""Unit tests for :mod:`nexus_coordinator.peer_creds`.

Sprint 16 Phase B (D2). The tests are mostly negative on Windows
(verify that the helper raises a clean OSError) and positive on
Linux/macOS (verify that a socketpair returns the current uid).
"""

from __future__ import annotations

import os
import socket
import sys

import pytest
from nexus_coordinator import peer_creds


def test_is_supported_matches_platform() -> None:
    assert peer_creds.is_supported() == (sys.platform in ("linux", "darwin", "freebsd"))


@pytest.mark.skipif(sys.platform == "win32", reason="geteuid is POSIX-only")
def test_current_uid_matches_geteuid() -> None:
    assert peer_creds.current_uid() == os.geteuid()


@pytest.mark.skipif(sys.platform == "win32", reason="UDS unsupported")
def test_peer_uid_returns_self_for_local_socketpair() -> None:
    a, b = socket.socketpair(socket.AF_UNIX, socket.SOCK_STREAM)
    try:
        uid = peer_creds.peer_uid(a)
        assert uid == os.geteuid()
        # Symmetric: reading from the other end matches too.
        uid2 = peer_creds.peer_uid(b)
        assert uid2 == os.geteuid()
    finally:
        a.close()
        b.close()


@pytest.mark.skipif(sys.platform != "win32", reason="Windows-specific behavior")
def test_peer_uid_raises_on_windows() -> None:
    # On Windows the helper is unavailable — connection auth lives
    # in the Named Pipe DACL handled by the Rust daemon side.
    with pytest.raises(OSError):
        peer_creds.peer_uid(socket.socket())


@pytest.mark.skipif(sys.platform != "win32", reason="Windows-specific behavior")
def test_current_uid_raises_on_windows() -> None:
    with pytest.raises(OSError):
        peer_creds.current_uid()
