# SPDX-License-Identifier: AGPL-3.0-or-later
"""Per-class decorator registry used by :mod:`nexus_sdk.decorators`.

The decorators attach small metadata tuples to the wrapped
function via private attributes (``__nexus_route__``, etc.);
:func:`collect_decorators` walks a class at instance construction
time and pulls them out so the :class:`NexusApp` base class can
hand typed descriptor lists to the coordinator.
"""

from __future__ import annotations

from typing import Any

ROUTE_ATTR = "__nexus_route__"
WORKER_ATTR = "__nexus_worker__"
TAB_ATTR = "__nexus_tab__"
COMMAND_ATTR = "__nexus_command__"
FILES_ATTR = "__nexus_app_files__"


def collect_decorators(
    cls: type,
) -> tuple[
    list[dict[str, Any]],
    list[dict[str, Any]],
    list[dict[str, Any]],
    list[dict[str, Any]],
]:
    """Scan ``cls`` (and its bases) for decorated methods.

    Returns four lists: ``(routes, workers, tabs, commands)``.
    Each entry is a dict ready to feed into the matching
    dataclass / Pydantic constructor (RouteDescriptor,
    WorkerDescriptor, TabDescriptor, CommandDescriptor).

    Sprint 8 Phase A adds the fourth bucket for
    :func:`nexus_sdk.nexus_command`. Keeping it a positional
    return tuple (rather than a named tuple or dict) matches the
    existing unpacking convention in :class:`NexusApp.__init__`.

    Sprint 9 Phase A (T12) — ``workers``, ``tabs`` and ``commands``
    are sorted by their ``name`` key before return. The previous
    order was whatever ``dir(cls)`` produced, which is
    alphabetical by attribute name on CPython as an implementation
    detail rather than a documented guarantee. Making the sort
    explicit locks the order against PyPy, ``__slots__``
    reshuffles, and method renames. ``routes`` is sorted by
    ``path`` for symmetry.
    """
    routes: list[dict[str, Any]] = []
    workers: list[dict[str, Any]] = []
    tabs: list[dict[str, Any]] = []
    commands: list[dict[str, Any]] = []

    for name in dir(cls):
        try:
            attr = getattr(cls, name)
        except AttributeError:
            continue
        if not callable(attr):
            continue

        if hasattr(attr, ROUTE_ATTR):
            meta = getattr(attr, ROUTE_ATTR)
            routes.append({"path": meta["path"], "methods": meta["methods"], "fn": attr})
        if hasattr(attr, WORKER_ATTR):
            meta = getattr(attr, WORKER_ATTR)
            workers.append({"name": meta["name"], "model": meta["model"], "fn": attr})
        if hasattr(attr, TAB_ATTR):
            meta = getattr(attr, TAB_ATTR)
            tabs.append({"name": meta["name"], "icon": meta["icon"], "fn": attr})
        if hasattr(attr, COMMAND_ATTR):
            meta = getattr(attr, COMMAND_ATTR)
            commands.append(
                {
                    "name": meta["name"],
                    "description": meta["description"],
                    "icon": meta["icon"],
                    "group": meta["group"],
                    "fn": attr,
                }
            )

    routes.sort(key=lambda d: d["path"])
    workers.sort(key=lambda d: d["name"])
    tabs.sort(key=lambda d: d["name"])
    commands.sort(key=lambda d: d["name"])

    return routes, workers, tabs, commands
