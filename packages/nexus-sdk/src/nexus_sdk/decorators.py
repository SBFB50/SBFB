"""Method decorators the SDK consumer uses to advertise features.

Each decorator is a no-op wrapper that just tags the function
with a private attribute. The heavy lifting (collection into
descriptor lists) happens in :mod:`nexus_sdk.registry` when the
app instance is constructed.
"""

from __future__ import annotations

from typing import Any, Callable

from nexus_sdk.registry import ROUTE_ATTR, TAB_ATTR, WORKER_ATTR


def nexus_route(
    path: str,
    *,
    methods: list[str] | tuple[str, ...] = ("GET",),
) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
    """Mark a method as an HTTP route on the app.

    The coordinator mounts every decorated method under
    ``/app/<app-name>/<path>`` using the listed methods.

    Example::

        @nexus_route("/status", methods=["GET"])
        async def status(self, request):
            return {"status": "ok"}
    """

    def wrap(fn: Callable[..., Any]) -> Callable[..., Any]:
        setattr(fn, ROUTE_ATTR, {"path": path, "methods": tuple(methods)})
        return fn

    return wrap


def nexus_worker(*, name: str, model: str) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
    """Mark a method as a named compute worker on the app.

    The coordinator registers the worker in its scheduler; other
    parts of the same app (or other apps) can dispatch work to
    it by name.
    """

    def wrap(fn: Callable[..., Any]) -> Callable[..., Any]:
        setattr(fn, WORKER_ATTR, {"name": name, "model": model})
        return fn

    return wrap


def nexus_tab(*, name: str, icon: str = "") -> Callable[[Callable[..., Any]], Callable[..., Any]]:
    """Mark a method as a frontend tab descriptor.

    The return value of the decorated method is surfaced in the
    ``GET /app/<name>/manifest`` endpoint so the Sprint 5 React
    frontend can build its sidebar dynamically.
    """

    def wrap(fn: Callable[..., Any]) -> Callable[..., Any]:
        setattr(fn, TAB_ATTR, {"name": name, "icon": icon})
        return fn

    return wrap
