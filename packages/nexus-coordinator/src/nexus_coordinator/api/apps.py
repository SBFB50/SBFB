# SPDX-License-Identifier: AGPL-3.0-or-later
"""``/app/{app_name}`` routing for installed nexus-sdk apps.

Mounts every :class:`nexus_sdk.NexusApp` discovered through the
``nexus.apps`` entry points at coordinator boot.

Surface:

- ``GET  /app``                                       — list installed apps
- ``GET  /app/{name}/manifest``                       — full manifest
- ``GET  /app/{name}/tabs/{tab_name}/descriptor``     — TabView descriptor
- ``POST /app/{name}/tasks/submit``                   — Sprint 8 D1 task submit
- ``GET  /app/{name}/commands``                       — Sprint 8 D2 palette
- ``POST /app/{name}/commands/{cmd_name}/invoke``     — Sprint 8 D2 invoke

Sprint 6 Phase A added :class:`nexus_sdk.view.TabView` validation
on the descriptor route with a one-release ``legacy_descriptor``
fallback. Sprint 8 Phase A (D4) retires that fallback entirely:
a tab that cannot produce a valid TabView now fails the request
with HTTP 422 and a structured error detail instead of silently
shipping a degraded payload to the shell.
"""

from __future__ import annotations

import inspect
import logging
from typing import TYPE_CHECKING, Any

from fastapi import APIRouter, HTTPException, Request
from nexus_sdk import CommandDescriptor, StorageSchemaError, WorkerNotFound
from nexus_sdk.view import AnyTabView, TabView, TabViewV2
from pydantic import BaseModel, Field, TypeAdapter, ValidationError

_AnyTabViewAdapter = TypeAdapter(AnyTabView)

if TYPE_CHECKING:
    from nexus_sdk import AppContext, NexusApp

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/app", tags=["apps"])


def _apps(request: Request) -> dict[str, "NexusApp"]:
    coord = request.app.state.coordinator
    return getattr(coord, "apps", {})


def _app_contexts(request: Request) -> dict[str, "AppContext"]:
    coord = request.app.state.coordinator
    return getattr(coord, "app_contexts", {})


@router.get("")
async def list_apps(request: Request) -> dict[str, Any]:
    """Return every installed app with a short summary."""
    apps = _apps(request)
    return {
        "apps": [
            {
                "name": app.manifest.name,
                "version": app.manifest.version,
                "description": app.manifest.description,
                "routes": len(app.routes()),
                "workers": len(app.workers()),
                "tabs": len(app.tabs()),
                "commands": len(app.commands()),
            }
            for app in apps.values()
        ],
        "count": len(apps),
    }


@router.get("/{name}/manifest")
async def app_manifest(request: Request, name: str) -> dict[str, Any]:
    """Full manifest + descriptor list for one app."""
    apps = _apps(request)
    app = apps.get(name)
    if app is None:
        raise HTTPException(status_code=404, detail=f"app {name!r} not installed")
    return {
        "manifest": app.manifest.model_dump(),
        "routes": [{"path": r.path, "methods": list(r.methods)} for r in app.routes()],
        "workers": [{"name": w.name, "model": w.model} for w in app.workers()],
        "tabs": [
            {
                "name": t.name,
                "icon": t.icon,
                "descriptor": _maybe_call(t.fn, app),
            }
            for t in app.tabs()
        ],
        "commands": [c.model_dump() for c in app.commands()],
        "task_handlers": [
            {
                "name": th.name,
                "request_schema": th.request_schema,
                "response_schema": th.response_schema,
            }
            for th in app.task_handlers()
        ],
    }


@router.get("/{name}/tabs/{tab_name}/descriptor")
async def app_tab_descriptor(request: Request, name: str, tab_name: str) -> dict[str, Any]:
    """Invoke a single tab descriptor function, awaiting if async.

    Sprint 8 Phase A (D4): the previous one-release
    ``legacy_descriptor`` fallback is retired. A descriptor that
    does not produce a valid :class:`nexus_sdk.view.TabView`
    fails the request with HTTP 422 and a structured detail
    carrying the Pydantic error message — no silent degradation.
    The shell side (``web/src/api/coordinator.ts::getAppTabDescriptor``)
    surfaces the 422 as a visible error banner instead of
    rendering the raw payload under a ``legacy`` flag.

    Returns ``{"descriptor": ...}`` on success. A missing app or
    tab returns 404; a descriptor exception propagates as 500
    with the exception message in the detail field.
    """
    apps = _apps(request)
    app = apps.get(name)
    if app is None:
        raise HTTPException(status_code=404, detail=f"app {name!r} not installed")

    tab = next((t for t in app.tabs() if t.name == tab_name), None)
    if tab is None:
        raise HTTPException(
            status_code=404,
            detail=f"tab {tab_name!r} not found on app {name!r}",
        )

    try:
        if inspect.iscoroutinefunction(tab.fn):
            descriptor = await tab.fn(app)
        else:
            descriptor = tab.fn(app)
    except Exception as e:  # noqa: BLE001
        raise HTTPException(
            status_code=500,
            detail=f"tab descriptor raised: {type(e).__name__}: {e}",
        ) from e

    if isinstance(descriptor, (TabView, TabViewV2)):
        return {"descriptor": descriptor.model_dump()}
    try:
        validated = _AnyTabViewAdapter.validate_python(descriptor)
    except ValidationError as exc:
        logger.warning(
            "tab descriptor for app=%r tab=%r failed TabView validation: %s",
            name,
            tab_name,
            exc.errors(include_url=False),
        )
        raise HTTPException(
            status_code=422,
            detail=(
                f"tab {tab_name!r} on app {name!r} returned an invalid "
                f"descriptor: {exc.error_count()} field error(s). The "
                "Sprint 8 removal of the legacy fallback means every "
                "tab must ship a schema-valid TabView."
            ),
        ) from exc
    return {"descriptor": validated.model_dump()}


# =================================================================
# Sprint 8 Phase A — D1 submit_task + D2 commands routes
# =================================================================


class SubmitAppTaskRequest(BaseModel):
    """Body of ``POST /app/{name}/tasks/submit``.

    Sprint 7 D4 frozen signature: ``worker`` is a routing key,
    ``payload`` is a free-form JSON dict that gets serialized
    (sorted keys) into the coordinator dispatcher's ``prompt``
    field, and optional ``priority`` / ``parent_task_id`` ride
    through as metadata.
    """

    model_config = {"extra": "forbid"}

    worker: str = Field(..., min_length=1, max_length=128)
    payload: dict[str, Any] = Field(default_factory=dict)
    priority: int = Field(5, ge=0, le=10)
    parent_task_id: str | None = None


@router.post("/{name}/tasks/submit")
async def submit_app_task(request: Request, name: str, body: SubmitAppTaskRequest) -> dict[str, str]:
    """Submit a task through an app's :class:`AppContext`.

    The loader wires each app's backref into its context at
    ``on_start`` time (Sprint 8 Phase A); this route simply
    fetches the stored context and delegates. The routing key
    is resolved via :meth:`NexusApp.resolve_worker` which raises
    :class:`WorkerNotFound` if the string doesn't map to a
    registered worker — the route surfaces that as HTTP 422.
    """
    apps = _apps(request)
    if name not in apps:
        raise HTTPException(status_code=404, detail=f"app {name!r} not installed")

    ctx = _app_contexts(request).get(name)
    if ctx is None:
        raise HTTPException(
            status_code=500,
            detail=(
                f"app {name!r} has no bound context — this is a coordinator "
                "bug; the loader must set ctx in coord.app_contexts before "
                "mounting routes."
            ),
        )

    try:
        task_id = await ctx.submit_task(
            body.worker,
            body.payload,
            priority=body.priority,
            parent_task_id=body.parent_task_id,
        )
    except WorkerNotFound as e:
        raise HTTPException(status_code=422, detail=str(e)) from e
    return {"task_id": task_id}


@router.get("/{name}/commands", response_model=list[CommandDescriptor])
async def list_app_commands(request: Request, name: str) -> list[CommandDescriptor]:
    """Return every ``@nexus_command``-decorated entry on an app.

    Sprint 8 Phase A (D2): the shell's Command Palette polls
    this route for each enrolled app and merges the results
    into a dedicated ``App: <name>`` group in the palette.
    """
    apps = _apps(request)
    app = apps.get(name)
    if app is None:
        raise HTTPException(status_code=404, detail=f"app {name!r} not installed")
    return app.commands()


# =================================================================
# Sprint 9 Phase B — D1 typed namespace setter
# =================================================================


@router.post("/{name}/state/{ns_key}")
async def set_app_state(
    request: Request,
    name: str,
    ns_key: str,
    body: dict[str, Any],
) -> dict[str, Any]:
    """Persist a value into one of an app's typed storage namespaces.

    Sprint 9 Phase B (D1 consumer wiring): apps register typed
    :class:`nexus_sdk.TypedNamespace` instances on
    :attr:`nexus_sdk.AppContext.namespaces` from their
    ``on_start`` hook. This route looks up the namespace by key,
    forwards the JSON body through ``Schema.model_validate()``
    via :meth:`TypedNamespace.set`, and writes the validated
    value to the underlying :class:`nexus_sdk.AppStorage`. The
    coordinator never imports the schema directly — the typed
    namespace registration is the only contract — so adding a
    new typed namespace consumer is a pure app-side change.

    Returns ``{"ok": True}`` on success. Failure modes:

    - 404 — unknown app or unknown namespace key.
    - 422 — body fails the bound schema's
      :class:`pydantic.ValidationError` (raised here as
      :class:`nexus_sdk.StorageSchemaError`); the detail field
      carries the underlying validation message.
    - 503 — the app context exists but ``ctx.storage`` is
      ``None``, which signals a coordinator bug because the
      loader always wires it.
    """
    apps = _apps(request)
    if name not in apps:
        raise HTTPException(status_code=404, detail=f"app {name!r} not installed")
    ctx = _app_contexts(request).get(name)
    if ctx is None:
        raise HTTPException(
            status_code=500,
            detail=(f"app {name!r} has no bound context — coordinator loader bug"),
        )
    if ctx.storage is None:
        raise HTTPException(
            status_code=503,
            detail=f"app {name!r} has no AppContext.storage wired",
        )
    namespace = ctx.namespaces.get(ns_key)
    if namespace is None:
        raise HTTPException(
            status_code=404,
            detail=(
                f"app {name!r} has no typed namespace registered under {ns_key!r}; the app must register it on on_start"
            ),
        )
    try:
        await namespace.set(body)
    except StorageSchemaError as exc:
        raise HTTPException(status_code=422, detail=str(exc)) from exc
    return {"ok": True}


@router.post("/{name}/commands/{cmd_name}/invoke")
async def invoke_app_command(request: Request, name: str, cmd_name: str) -> dict[str, Any]:
    """Invoke a command palette entry on an app.

    The returned value is passed straight through to the shell,
    which expects either ``None`` (no-op) or a dict of the form
    ``{"navigation": {"path": str}}`` to trigger a client-side
    route change. Any other shape is treated as a no-op by the
    palette client.
    """
    apps = _apps(request)
    app = apps.get(name)
    if app is None:
        raise HTTPException(status_code=404, detail=f"app {name!r} not installed")
    try:
        result = await app.invoke_command(cmd_name)
    except LookupError as e:
        raise HTTPException(status_code=404, detail=str(e)) from e
    return {"result": result}


# =================================================================
# Internal helpers
# =================================================================


def _maybe_call(fn: Any, app: Any) -> Any:
    """Invoke a tab descriptor function if it's synchronous.

    Returns the return value when the tab's ``fn`` is sync; for
    async functions returns a placeholder note so listing the
    manifest cannot hang on a slow descriptor.
    """
    if inspect.iscoroutinefunction(fn):
        return {"note": "async descriptor — call /app/{name}/tabs/{tab} to invoke"}
    try:
        return fn(app)
    except Exception as e:  # noqa: BLE001
        return {"error": str(e)}
