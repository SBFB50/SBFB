"""NexusApp base class + manifest + descriptors.

An *app* is a self-contained plugin that a coordinator hosts:
routes are mounted under ``/app/<name>/...``, workers are
registered as named compute consumers, tabs are advertised on
the frontend manifest (Sprint 5), and Sprint 8 Phase A adds a
fourth bucket — command palette entries via ``@nexus_command``.

A concrete app subclasses :class:`NexusApp`, defines its
:class:`AppManifest` as a class attribute, and annotates its
methods with :func:`nexus_route` / :func:`nexus_worker` /
:func:`nexus_tab` / :func:`nexus_command` decorators. The base
class introspects the decorated methods at instance-construction
time and produces the descriptor lists the coordinator consumes.
"""

from __future__ import annotations

import json
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any, Callable

from pydantic import BaseModel, Field

from nexus_sdk.commands import CommandDescriptor
from nexus_sdk.compute_client import ComputeClient
from nexus_sdk.db import AppDatabaseClient
from nexus_sdk.registry import collect_decorators


class WorkerNotFound(LookupError):
    """Raised when :meth:`NexusApp.resolve_worker` cannot resolve
    a routing key to a registered worker.

    The error carries the attempted routing key in the message
    so the coordinator proxy can surface a 422 with a useful
    hint (``"worker 'gov.rag' not found on app 'gov'"``).
    """


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
    requests to the coordinator, the project the app is hosted
    in, the app's own name (used by :meth:`submit_task` for
    routing), an :class:`AppDatabaseClient` wired by the
    coordinator loader at boot (Sprint 8 Phase B — apps may
    swap the field in their ``on_start`` override to point at a
    different SQLite file), and a free-form ``extras`` dict for
    future plumbing.

    The ``_app`` field is a backref to the :class:`NexusApp`
    instance that owns this context. The coordinator loader
    wires it before calling ``on_start(ctx)``. Unit tests that
    construct an ``AppContext`` directly may leave it ``None``;
    :meth:`submit_task` refuses to run without it so such tests
    still see a clear error instead of a silent NoneType.
    """

    compute: ComputeClient
    project_name: str
    app_name: str = ""
    db: AppDatabaseClient | None = None
    extras: dict[str, Any] = field(default_factory=dict)
    # Back-reference to the NexusApp this context serves. Wired
    # by the coordinator loader in Sprint 8 Phase A; tests that
    # skip the loader may leave it None.
    _app: "NexusApp | None" = None

    async def submit_task(
        self,
        worker: str,
        payload: dict[str, Any],
        *,
        priority: int = 5,
        parent_task_id: str | None = None,
    ) -> str:
        """Submit a task to the coordinator's dispatcher.

        Sprint 7 D4 frozen signature. ``worker`` is a routing key
        in one of two shapes:

        - ``"<worker>"`` — resolved against the current app's
          own ``@nexus_worker`` registrations.
        - ``"<app>.<worker>"`` — cross-app shape. Sprint 8 only
          accepts the form where ``<app>`` matches this app's
          manifest name; genuine cross-app dispatch is deferred.

        The method resolves ``worker`` via
        :meth:`NexusApp.resolve_worker` (which raises
        :class:`WorkerNotFound` on miss), serializes ``payload``
        as deterministic JSON into the prompt slot of the
        coordinator's ``/tasks/submit`` body, carries
        ``parent_task_id`` through the metadata map for causal
        ordering, and returns the task id the coordinator
        assigned.
        """
        if self._app is None:
            raise RuntimeError(
                "AppContext.submit_task requires a NexusApp backref; "
                "the coordinator loader wires this before on_start — "
                "unit tests that construct AppContext manually must "
                "pass `_app=<app_instance>` explicitly."
            )
        desc = self._app.resolve_worker(worker)
        prompt = json.dumps(payload, sort_keys=True)
        metadata: dict[str, str] = {}
        if parent_task_id is not None:
            metadata["parent_task_id"] = parent_task_id
        task = await self.compute.submit_task(
            task_type=worker,
            prompt=prompt,
            model=desc.model,
            priority=priority,
            metadata=metadata if metadata else None,
        )
        return task.task_id


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
        (
            self._routes,
            self._workers,
            self._tabs,
            self._commands,
        ) = collect_decorators(type(self))

    # ------------------------------------------------------------------
    # Descriptors consumed by the coordinator's loader
    # ------------------------------------------------------------------

    def routes(self) -> list[RouteDescriptor]:
        return [RouteDescriptor(path=r["path"], methods=tuple(r["methods"]), fn=r["fn"]) for r in self._routes]

    def workers(self) -> list[WorkerDescriptor]:
        return [WorkerDescriptor(name=w["name"], model=w["model"], fn=w["fn"]) for w in self._workers]

    def tabs(self) -> list[TabDescriptor]:
        return [TabDescriptor(name=t["name"], icon=t["icon"], fn=t["fn"]) for t in self._tabs]

    def commands(self) -> list[CommandDescriptor]:
        """Return Pydantic-validated command descriptors for
        every ``@nexus_command``-decorated method.

        Sprint 8 Phase A (D2 impl) — the React shell's Command
        Palette merges the output of this method across every
        app enrolled on the coordinator into a dedicated
        ``App: <name>`` group.
        """
        return [
            CommandDescriptor(
                name=c["name"],
                description=c["description"],
                icon=c["icon"],
                group=c["group"],
            )
            for c in self._commands
        ]

    # ------------------------------------------------------------------
    # Routing helpers
    # ------------------------------------------------------------------

    def resolve_worker(self, routing_key: str) -> WorkerDescriptor:
        """Resolve a routing key to one of this app's registered
        workers.

        Accepted shapes:

        - ``"<worker>"`` — exact match against
          :meth:`workers`. Raises :class:`WorkerNotFound` if
          the current app has no such worker.
        - ``"<app>.<worker>"`` — the ``<app>`` prefix must equal
          ``self.manifest.name`` or the resolver refuses. True
          cross-app dispatch is deferred to a future sprint
          where the coordinator loader exposes a registry of
          sibling apps.

        The first match wins; in practice a given app
        should not register two workers with the same name so
        ordering is not a contract.
        """
        if "." in routing_key:
            app_prefix, worker_name = routing_key.split(".", 1)
            if app_prefix != self.manifest.name:
                raise WorkerNotFound(
                    f"routing key {routing_key!r} targets app {app_prefix!r}, but this app is {self.manifest.name!r}"
                )
            key = worker_name
        else:
            key = routing_key

        for w in self.workers():
            if w.name == key:
                return w
        raise WorkerNotFound(f"worker {key!r} not found on app {self.manifest.name!r}")

    async def invoke_command(self, cmd_name: str) -> Any:
        """Invoke a ``@nexus_command``-decorated method by name.

        Used by the coordinator's ``POST /app/{name}/commands/{cmd}/invoke``
        route. The return value is forwarded verbatim to the
        caller (the shell expects either ``None``, or a dict
        of the form ``{"navigation": {"path": str}}``).
        """
        for c in self._commands:
            if c["name"] == cmd_name:
                fn = c["fn"]
                result = fn(self)
                if hasattr(result, "__await__"):
                    return await result
                return result
        raise LookupError(f"command {cmd_name!r} not found on app {self.manifest.name!r}")

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
