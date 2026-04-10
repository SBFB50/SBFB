"""nexus-sdk — plugin SDK for nexus-grid apps.

Core public API:

- :class:`AppManifest` + :class:`NexusApp` — base classes every
  app extends.
- :func:`nexus_route`, :func:`nexus_worker`, :func:`nexus_tab` —
  decorators that register handlers on the app instance.
- :class:`ComputeClient` — submit tasks to the local coordinator
  via its FastAPI ``/tasks/submit`` endpoint.
- :func:`discover_apps` — importlib entry_points loader the
  coordinator calls at boot.
"""

from nexus_sdk import view
from nexus_sdk.app import AppContext, AppManifest, NexusApp, TabDescriptor, WorkerDescriptor
from nexus_sdk.compute_client import ComputeClient
from nexus_sdk.decorators import nexus_route, nexus_tab, nexus_worker
from nexus_sdk.loader import discover_apps
from nexus_sdk.view import (
    TabBlock,
    TabView,
    badge_list,
    button_route,
    button_task,
    chart_bar,
    chart_line,
    empty,
    heading,
    kv,
    metric,
    section,
    table_,
    text,
)

__version__ = "0.1.0"

__all__ = [
    "AppContext",
    "AppManifest",
    "ComputeClient",
    "NexusApp",
    "TabBlock",
    "TabDescriptor",
    "TabView",
    "WorkerDescriptor",
    "__version__",
    "badge_list",
    "button_route",
    "button_task",
    "chart_bar",
    "chart_line",
    "discover_apps",
    "empty",
    "heading",
    "kv",
    "metric",
    "nexus_route",
    "nexus_tab",
    "nexus_worker",
    "section",
    "table_",
    "text",
    "view",
]
