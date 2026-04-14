# SPDX-License-Identifier: AGPL-3.0-or-later
"""SO_PEERCRED helper for the coordinator UDS path.

Sprint 16 Phase B (D2 — UDS durcis avec SO_PEERCRED).

The coordinator binds a Unix Domain Socket at
``~/.sbfb/run/coordinator.sock`` (Linux/macOS only) beside its
TCP listener. The on-disk file mode 0600 + parent dir 0700 keeps
the socket scoped to the current user, which is the primary gate.

This helper exposes :func:`peer_uid` so tests and a future ASGI
middleware can verify the peer's OS uid. We do **not** wire the
verification into the auth middleware bypass yet — uvicorn's
ASGI scope does not expose the connection FD directly, so the
SO_PEERCRED bypass for the Python side is deferred to Sprint 17
when we either subclass uvicorn's protocol or move the
coordinator UDS surface behind the same Rust accept loop the
shell daemon already uses (cf. R3 in the Phase B plan).

Windows is unsupported on purpose. Named Pipes have a different
authentication model — DACL on the pipe handle does the gate
(see the daemon's :mod:`nexus_shell_daemon::named_pipe_server`).
The coordinator's Windows surface stays TCP+bearer and is
scheduled for a Sprint 17 Rust side-car if a CLI use-case
materializes.
"""

from __future__ import annotations

import os
import socket
import struct
import sys


def is_supported() -> bool:
    """Return ``True`` iff the current platform exposes a way to
    read peer credentials from a Unix Domain Socket.
    """
    return sys.platform in ("linux", "darwin", "freebsd")


def current_uid() -> int:
    """Return the effective uid of the current process.

    Wraps :func:`os.geteuid` so callers that mix peer + self uid
    reads see the same ``int`` shape on both sides. Raises
    ``OSError`` on Windows where ``geteuid`` does not exist.
    """
    if not hasattr(os, "geteuid"):
        raise OSError("geteuid is not available on this platform")
    return os.geteuid()


def peer_uid(sock: socket.socket) -> int:
    """Read the peer's uid from a connected Unix Domain Socket.

    Uses ``getsockopt(SOL_SOCKET, SO_PEERCRED)`` on Linux (returns
    a ``(pid, uid, gid)`` triple via ``struct.unpack``) or
    ``getpeereid`` on macOS / *BSD (returns ``(uid, gid)``).

    Raises ``OSError`` on Windows or any other platform without a
    peer-credentials primitive.
    """
    if sys.platform == "linux":
        # struct ucred = {pid_t pid, uid_t uid, gid_t gid} = 12 bytes (3 * i32)
        SO_PEERCRED = getattr(socket, "SO_PEERCRED", 17)
        data = sock.getsockopt(socket.SOL_SOCKET, SO_PEERCRED, struct.calcsize("3i"))
        _pid, uid, _gid = struct.unpack("3i", data)
        return uid
    if sys.platform in ("darwin", "freebsd"):
        # ctypes path: getpeereid is in libc on these platforms.
        # Avoid the ctypes import on Linux to keep the module
        # cheap to load in the hot start path.
        import ctypes
        import ctypes.util

        libc_name = ctypes.util.find_library("c") or "libc.so.6"
        libc = ctypes.CDLL(libc_name, use_errno=True)
        uid_t = ctypes.c_uint32
        gid_t = ctypes.c_uint32
        uid = uid_t(0)
        gid = gid_t(0)
        rc = libc.getpeereid(sock.fileno(), ctypes.byref(uid), ctypes.byref(gid))
        if rc != 0:
            err = ctypes.get_errno()
            raise OSError(err, os.strerror(err))
        return int(uid.value)
    raise OSError(f"SO_PEERCRED unavailable on {sys.platform}")


__all__ = ["current_uid", "is_supported", "peer_uid"]
