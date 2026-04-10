"""NexusApp base class + manifest + descriptors.

An *app* is a self-contained plugin that a coordinator hosts:
routes are mounted under ``/app/<name>/...``, workers are
registered as named compute consumers, tabs are advertised on
the frontend manifest (Sprint 5).

A concrete app subclasses :class:`NexusApp`, defines its
:class:`AppManifest` as a class attribute, and annotates its
methods with :func:`nexus_route` / :func:`nexus_worker` /
:func:`nexus_tab` decorators. The base class introspects the
decorated methods at instance-construction time and produces the
descriptor lists the coordinator consumes.
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any, Callable

from pydantic import BaseModel, Field

from nexus_sdk.compute_client import ComputeClient
from nexus_sdk.registry import collect_decorators


class AppManifest(BaseModel):
    """Declarative metadata for an app.

    Every app ships exactly one of these. The coordinator serves
    it on ``GET /app/<name>/manifest`` so the frontend (Sprint 5)
    can build its sidebar and route table.
    """

    name: str = Field(..., min_length=1, max_length=64)
    version: str = Field(..., min_length=1)
    author: str = ""
    description: str = ""
    dependencies: list[str] = Field(default_factory=list)
    license: str = "AGPL-3.0"


@dataclass
class WorkerDescriptor:
    """One @nexus_worker-decorated method on an app."""

    name: str
    model: str
    fn: Callable[..., Any]


@dataclass
class TabDescriptor:
    """One @nexus_tab-decorated method on an app."""

    name: str
    icon: str
    fn: Callable[..., Any]


@dataclass
class RouteDescriptor:
    """One @nexus_route-decorated method on an app."""

    path: str
    methods: tuple[str, ...]
    fn: Callable[..., Any]


@dataclass
class AppContext:
    """Context passed into ``on_start`` / ``on_stop``.

    Carries the references an app needs to reach the outside
    world: a :class:`ComputeClient` that forwards compute
    requests to the coordinator, and a free-form ``extras`` dict
    for future plumbing.
    """

    compute: ComputeClient
    project_name: str
    extras: dict[str, Any] = field(default_factory=dict)


class NexusApp(ABC):
    """Base class for every nexus-grid app.

    Subclasses:

    1. Set ``manifest`` as a class attribute of type
       :class:`AppManifest`.
    2. Decorate methods with :func:`nexus_route`,
       :func:`nexus_worker`, :func:`nexus_tab` to advertise
       functionality.
    3. Implement :meth:`on_start` and :meth:`on_stop`; the
       coordinator calls them at boot and shutdown.

    Example::

        class MyApp(NexusApp):
            manifest = AppManifest(name="my", version="0.1.0")

            @nexus_route("/hello", methods=["GET"])
            async def hello(self, request):
                return {"msg": "hi"}

            async def on_start(self, ctx):
                self.ctx = ctx

            async def on_stop(self):
                pass
    """

    manifest: AppManifest  # must be set by subclasses

    def __init__(self) -> None:
        if not hasattr(type(self), "manifest"):
            raise TypeError(f"{type(self).__name__} must declare a class-level `manifest: AppManifest`")
        self._routes, self._workers, self._tabs = collect_decorators(type(self))

    # ------------------------------------------------------------------
    # Descriptors consumed by the coordinator's loader
    # ------------------------------------------------------------------

    def routes(self) -> list[RouteDescriptor]:
        return [RouteDescriptor(path=r["path"], methods=tuple(r["methods"]), fn=r["fn"]) for r in self._routes]

    def workers(self) -> list[WorkerDescriptor]:
        return [WorkerDescriptor(name=w["name"], model=w["model"], fn=w["fn"]) for w in self._workers]

    def tabs(self) -> list[TabDescriptor]:
        return [TabDescriptor(name=t["name"], icon=t["icon"], fn=t["fn"]) for t in self._tabs]

    # ------------------------------------------------------------------
    # Lifecycle hooks — subclasses must implement
    # ------------------------------------------------------------------

    @abstractmethod
    async def on_start(self, ctx: AppContext) -> None:
        """Called once at coordinator boot, after every route is
        registered and before any incoming request is served."""

    @abstractmethod
    async def on_stop(self) -> None:
        """Called on graceful coordinator shutdown. Release any
        open resources here."""
