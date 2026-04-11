"""``/app/{app_name}`` routing for installed nexus-sdk apps.

Mounts every :class:`nexus_sdk.NexusApp` discovered through the
``nexus.apps`` entry points at coordinator boot. Phase D scope:

- ``GET /app`` — list every installed app with its manifest.
- ``GET /app/{name}/manifest`` — the full manifest + descriptor
  counts for a single app.
- Every ``@nexus_route(path)`` on an app is reachable at
  ``/app/{name}{path}`` with the declared HTTP methods.

Sprint 6 Phase A adds :class:`nexus_sdk.view.TabView` validation
on ``GET /app/{name}/tabs/{tab}/descriptor`` so schema-driven
tabs ship a well-formed payload. Legacy dicts are still accepted
with ``legacy_descriptor: true`` flag for one release.
"""

from __future__ import annotations

import inspect
import logging
from typing import TYPE_CHECKING, Any

from fastapi import APIRouter, HTTPException, Request
from nexus_sdk.view import TabView
from pydantic import ValidationError

if TYPE_CHECKING:
    from nexus_sdk import NexusApp

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/app", tags=["apps"])


def _apps(request: Request) -> dict[str, "NexusApp"]:
    coord = request.app.state.coordinator
    return getattr(coord, "apps", {})


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
    }


@router.get("/{name}/tabs/{tab_name}/descriptor")
async def app_tab_descriptor(request: Request, name: str, tab_name: str) -> dict[str, Any]:
    """Invoke a single tab descriptor function, awaiting if async.

    Sprint 5 Phase B: the ``/manifest`` endpoint calls each tab's
    descriptor synchronously via :func:`_maybe_call`, which
    short-circuits async descriptors with a placeholder note so
    the manifest response cannot hang. This endpoint exists so the
    shell can *explicitly* invoke an async descriptor on demand
    (user clicks "Invoquer" in the UI) and get the real result.

    Returns ``{"descriptor": ...}`` on success. A missing app or
    tab returns 404; descriptor exceptions propagate as 500 with
    the exception message in the detail field.
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

    return _coerce_tab_view(descriptor, name, tab_name)


def _coerce_tab_view(descriptor: Any, app_name: str, tab_name: str) -> dict[str, Any]:
    """Validate ``descriptor`` against the Sprint 6 TabView schema.

    When the descriptor validates, return ``{"descriptor": <dumped>,
    "legacy_descriptor": false}``. When it fails to validate, log a
    warning and return the original ``{"descriptor": <raw>,
    "legacy_descriptor": true}`` so pre-Sprint-6 apps keep working
    for one release while they are ported.

    # TODO(Sprint 8): remove the legacy_descriptor fallback once the
    # 19-tab nexus-app-gov migration lands. The fallback is a
    # transition aid for ONE release only, per sprint6_plan.md §D3
    # and docs/shell/PATTERNS.md §P8. This comment is the code-level
    # sentinel the Sprint 6 audit finding D-1 asked for — grep for
    # "Sprint 8" to find every such marker before cutting the release.
    """
    if isinstance(descriptor, TabView):
        return {"descriptor": descriptor.model_dump(), "legacy_descriptor": False}
    try:
        validated = TabView.model_validate(descriptor)
    except ValidationError as exc:
        logger.warning(
            "tab descriptor for app=%r tab=%r is legacy (not a TabView): %s",
            app_name,
            tab_name,
            exc.errors(include_url=False),
        )
        return {"descriptor": descriptor, "legacy_descriptor": True}
    return {"descriptor": validated.model_dump(), "legacy_descriptor": False}


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


def legacy_descriptor_sweep(
    apps: dict[str, "NexusApp"],
) -> dict[str, list[str]]:
    """Sweep every sync tab descriptor and return the ones that still
    fail TabView.model_validate.

    Sprint 6 audit finding D-3: the coordinator used to have no
    boot-time visibility into "N apps still returning legacy
    descriptors". The warning logged per-request on first miss, but
    an operator never saw the pending migration until a user
    clicked around in the shell.

    This helper is called from
    :meth:`nexus_coordinator.coordinator.Coordinator.start` after
    apps are mounted. It iterates each app's tabs, calls the
    synchronous descriptor function (async ones are skipped — the
    sweep must not block boot), and runs the result through
    ``TabView.model_validate``. Tabs that fail are grouped per app
    and returned as ``{app_name: [tab_name, ...]}``.

    The caller logs a single INFO line with the aggregated count —
    one log entry, not one per failing tab, so that a coordinator
    with a heavily legacy app does not flood its start output.

    Pure function: does not log, does not mutate. Caller owns the
    presentation and log emission. This keeps the helper trivial to
    unit-test without a structlog capture fixture.
    """
    legacy: dict[str, list[str]] = {}
    for app_name, app in apps.items():
        failing_tabs: list[str] = []
        for tab in app.tabs():
            if inspect.iscoroutinefunction(tab.fn):
                # Async descriptors: skip. Calling them synchronously
                # at boot would hang a real gov tab that fetches over
                # HTTP. Async tabs are validated on their first
                # /app/.../descriptor invocation via _coerce_tab_view.
                continue
            try:
                descriptor = tab.fn(app)
            except Exception:  # noqa: BLE001
                # Descriptor raised — treat as legacy / broken so the
                # operator sees it in the summary. A raising descriptor
                # is not strictly "legacy" but it's still unreleasable,
                # so surfacing it here is more useful than ignoring.
                failing_tabs.append(tab.name)
                continue
            if isinstance(descriptor, TabView):
                continue
            try:
                TabView.model_validate(descriptor)
            except ValidationError:
                failing_tabs.append(tab.name)
        if failing_tabs:
            legacy[app_name] = failing_tabs
    return legacy
