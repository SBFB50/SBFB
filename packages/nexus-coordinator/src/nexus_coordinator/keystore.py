"""Ed25519 keypair persistence for the coordinator.

Wraps :func:`nexus_core.load_or_generate_secret` with the
file-permission enforcement the Sprint 2 audit S1 item flagged:
after writing the secret, set the file to owner-read-only
(``0o600``) on POSIX. On Windows we rely on the user's profile
ACLs (``%APPDATA%`` is already user-scoped) but log an info line
so operators on shared machines know what to check.

The returned keypair is an :class:`LoadedKeypair` dataclass with
the secret and public bytes as raw :class:`bytes`. Sign operations
go through :func:`nexus_core.sign_task` /
:func:`nexus_core.sign_result` / :func:`nexus_core.sign_claim`,
which all take the 32-byte secret as their second argument.
"""

from __future__ import annotations

import os
import platform
import stat
import sys
from dataclasses import dataclass
from pathlib import Path

import nexus_core  # provided by nexus-core-py wheel
import structlog

_log = structlog.get_logger(__name__)


@dataclass(frozen=True, slots=True)
class LoadedKeypair:
    """Output of :func:`load_or_generate_keypair`.

    Attributes:
        secret: 32-byte Ed25519 secret key. Keep private; never
            log, never serialize.
        public: 32-byte Ed25519 public key. Safe to share; forms
            the project identity on the gossip network.
        path: Absolute path the secret was read from or written
            to.
    """

    secret: bytes
    public: bytes
    path: Path


def load_or_generate_keypair(path: Path) -> LoadedKeypair:
    """Load a coordinator keypair, generating one on first run.

    ``path`` is the absolute ``coord.key`` path for the project.
    The parent directory is created if it does not exist. On
    first run, a fresh Ed25519 keypair is generated via
    :func:`nexus_core.load_or_generate_secret` and the secret is
    written atomically under the supplied path; subsequent calls
    read the same file.

    After writing (or on every load, defensively), the file is
    chmodded to ``0o600`` on POSIX so only the owner can read the
    secret. Windows relies on the profile's default ACLs.
    """
    path.parent.mkdir(parents=True, exist_ok=True)

    keypair_dict = nexus_core.load_or_generate_secret(str(path))
    secret: bytes = keypair_dict["secret"]
    public: bytes = keypair_dict["public"]

    _enforce_owner_only_perms(path)

    _log.info(
        "coordinator keypair ready",
        path=str(path),
        pubkey_hex=public.hex(),
        newly_generated=_file_was_just_created(path),
    )
    return LoadedKeypair(secret=secret, public=public, path=path)


def _enforce_owner_only_perms(path: Path) -> None:
    """Set owner-only permissions on the secret file.

    On POSIX: ``chmod 0o600``. On Windows: log a note so the
    operator can verify the ACLs manually; we don't invoke
    ``icacls`` to avoid a subprocess spawn on every boot.
    """
    if platform.system() == "Windows":
        _log.debug(
            "skipping chmod on Windows — relies on user profile ACLs",
            path=str(path),
        )
        return
    try:
        os.chmod(path, stat.S_IRUSR | stat.S_IWUSR)  # 0o600
    except OSError as e:
        # Not fatal — the secret is still usable, we just couldn't
        # tighten perms. Log at warning level so ops notices.
        print(f"warning: could not chmod {path} to 0o600: {e}", file=sys.stderr)


def _file_was_just_created(path: Path) -> bool:
    """Best-effort: true if the file's mtime is within the last 2s.

    Used only for the log field ``newly_generated`` — not a
    security check.
    """
    try:
        import time

        return (time.time() - path.stat().st_mtime) < 2.0
    except OSError:
        return False
