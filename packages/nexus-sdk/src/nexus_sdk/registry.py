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


def collect_decorators(
    cls: type,
) -> tuple[list[dict[str, Any]], list[dict[str, Any]], list[dict[str, Any]]]:
    """Scan ``cls`` (and its bases) for decorated methods.

    Returns three lists: ``(routes, workers, tabs)``. Each entry
    is a dict ready to feed into the matching dataclass
    constructor (RouteDescriptor, WorkerDescriptor, TabDescriptor).
    """
    routes: list[dict[str, Any]] = []
    workers: list[dict[str, Any]] = []
    tabs: list[dict[str, Any]] = []

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

    return routes, workers, tabs
