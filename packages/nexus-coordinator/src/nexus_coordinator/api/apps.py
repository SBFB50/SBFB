"""``/app/{app_name}`` routing for installed nexus-sdk apps.

Mounts every :class:`nexus_sdk.NexusApp` discovered through the
``nexus.apps`` entry points at coordinator boot. Phase D scope:

- ``GET /app`` — list every installed app with its manifest.
- ``GET /app/{name}/manifest`` — the full manifest + descriptor
  counts for a single app.
- Every ``@nexus_route(path)`` on an app is reachable at
  ``/app/{name}{path}`` with the declared HTTP methods.

Sprint 5 will add streaming (SSE/WS) and frontend manifest
synthesis for the React sidebar.
"""

from __future__ import annotations

import inspect
from typing import TYPE_CHECKING, Any

from fastapi import APIRouter, HTTPException, Request

if TYPE_CHECKING:
    from nexus_sdk import NexusApp

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
