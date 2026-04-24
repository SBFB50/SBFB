# SPDX-License-Identifier: AGPL-3.0-or-later
"""OS-level admin privilege check for ``nexus-admin`` mutations.

Sprint 25 Phase D — D5 capabilities gate-off-by-default.

Unix: ``os.geteuid() == 0``.
Windows: ``IsUserAnAdmin()`` + Mandatory Integrity Level >= High
(defense-in-depth per CAPABILITY_TOGGLES.md §4.1).
"""

from __future__ import annotations

import os
import sys


def require_admin() -> None:
    """Raise :class:`PermissionError` unless the process has admin privilege."""
    if sys.platform == "win32":
        _require_admin_windows()
    else:
        _require_admin_unix()


def _require_admin_unix() -> None:
    if os.geteuid() != 0:
        raise PermissionError("nexus-admin requires root privilege. Run with sudo.")


def _require_admin_windows() -> None:
    import ctypes

    if not ctypes.windll.shell32.IsUserAnAdmin():
        raise PermissionError("nexus-admin requires elevated command prompt. Run as Administrator.")
    _check_mil_high()


def _check_mil_high() -> None:
    """Verify Mandatory Integrity Level >= High (``0x3000``)."""
    import ctypes
    import ctypes.wintypes

    TOKEN_QUERY = 0x0008
    TokenIntegrityLevel = 25
    SECURITY_MANDATORY_HIGH_RID = 0x3000

    advapi32 = ctypes.windll.advapi32
    kernel32 = ctypes.windll.kernel32

    token = ctypes.wintypes.HANDLE()
    if not advapi32.OpenProcessToken(kernel32.GetCurrentProcess(), TOKEN_QUERY, ctypes.byref(token)):
        raise PermissionError("nexus-admin: cannot query process token.")

    try:
        needed = ctypes.wintypes.DWORD()
        advapi32.GetTokenInformation(token, TokenIntegrityLevel, None, 0, ctypes.byref(needed))
        buf = (ctypes.c_char * needed.value)()
        if not advapi32.GetTokenInformation(token, TokenIntegrityLevel, buf, needed, ctypes.byref(needed)):
            raise PermissionError("nexus-admin: cannot read integrity level.")

        sid_ptr = ctypes.cast(buf, ctypes.POINTER(ctypes.c_void_p))[0]
        sub_count_ptr = advapi32.GetSidSubAuthorityCount(sid_ptr)
        if not sub_count_ptr:
            raise PermissionError("nexus-admin: NULL SidSubAuthorityCount (malformed SID).")
        count = ctypes.cast(sub_count_ptr, ctypes.POINTER(ctypes.c_ubyte))[0]
        sub_auth_ptr = advapi32.GetSidSubAuthority(sid_ptr, count - 1)
        if not sub_auth_ptr:
            raise PermissionError("nexus-admin: NULL SidSubAuthority (malformed SID).")
        rid = ctypes.cast(sub_auth_ptr, ctypes.POINTER(ctypes.wintypes.DWORD))[0]

        if rid < SECURITY_MANDATORY_HIGH_RID:
            raise PermissionError("nexus-admin requires High Mandatory Integrity Level.")
    finally:
        kernel32.CloseHandle(token)
