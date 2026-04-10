"""App discovery via ``importlib.metadata`` entry points.

An app advertises itself on the ``nexus.apps`` entry point group.
At coordinator boot, :func:`discover_apps` enumerates every
installed entry, imports its target class, and returns a list of
instantiated :class:`NexusApp` objects the coordinator can drive.
"""

from __future__ import annotations

import importlib.metadata
import logging
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from nexus_sdk.app import NexusApp

_log = logging.getLogger(__name__)

ENTRY_POINT_GROUP = "nexus.apps"


def discover_apps() -> list["NexusApp"]:
    """Return every installed app, instantiated and ready to start.

    Import failures on individual entries are logged as warnings
    and skipped — a single broken third-party app should never
    prevent the coordinator from booting.
    """
    from nexus_sdk.app import NexusApp  # local import to avoid cycle

    apps: list[NexusApp] = []
    try:
        entries = importlib.metadata.entry_points(group=ENTRY_POINT_GROUP)
    except TypeError:
        # Python 3.9 legacy fallback (we require 3.13 so this
        # is just a belt-and-braces path).
        entries = importlib.metadata.entry_points().get(ENTRY_POINT_GROUP, [])  # type: ignore[attr-defined]

    for entry in entries:
        try:
            cls = entry.load()
        except Exception as e:  # noqa: BLE001
            _log.warning("failed to load nexus.apps entry %r: %s", entry.name, e)
            continue
        if not isinstance(cls, type) or not issubclass(cls, NexusApp):
            _log.warning(
                "nexus.apps entry %r does not point at a NexusApp subclass, got %r",
                entry.name,
                cls,
            )
            continue
        try:
            instance = cls()
        except Exception as e:  # noqa: BLE001
            _log.warning("failed to instantiate %r: %s", entry.name, e)
            continue
        apps.append(instance)
    return apps
