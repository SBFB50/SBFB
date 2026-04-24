# SPDX-License-Identifier: AGPL-3.0-or-later
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
from nexus_sdk.app import (
    AppContext,
    AppManifest,
    NexusApp,
    RouteDescriptor,
    TabDescriptor,
    TaskHandlerDescriptor,
    WorkerDescriptor,
    WorkerNotFound,
)
from nexus_sdk.commands import CommandDescriptor
from nexus_sdk.compute_client import ComputeClient
from nexus_sdk.db import AppDatabaseClient, DatabaseError
from nexus_sdk.decorators import nexus_app_files, nexus_command, nexus_route, nexus_tab, nexus_worker, task_handler
from nexus_sdk.events import AppEvents, EventEnvelope, EventOverflowPolicy
from nexus_sdk.files import AppFileStore, FileHandle, FileManifest, FileTypeError
from nexus_sdk.html_render import render_tabview_to_html
from nexus_sdk.loader import discover_apps
from nexus_sdk.migrations import MigrationRunner, MigrationTamperedError, PendingMigration
from nexus_sdk.storage import AppStorage, StorageSchemaError, TypedNamespace
from nexus_sdk.view import (
    AnyTabView,
    TabBlock,
    TabBlockFileUpload,
    TabView,
    TabViewV1,
    TabViewV2,
    badge_list,
    button_route,
    button_task,
    chart_bar,
    chart_line,
    empty,
    file_upload_block,
    heading,
    kv,
    metric,
    section,
    table_,
    text,
)

__version__ = "0.1.0"

__all__ = [
    "AnyTabView",
    "AppContext",
    "AppDatabaseClient",
    "AppEvents",
    "AppFileStore",
    "AppManifest",
    "AppStorage",
    "CommandDescriptor",
    "EventEnvelope",
    "EventOverflowPolicy",
    "ComputeClient",
    "DatabaseError",
    "FileHandle",
    "FileManifest",
    "FileTypeError",
    "MigrationRunner",
    "MigrationTamperedError",
    "NexusApp",
    "PendingMigration",
    "RouteDescriptor",
    "StorageSchemaError",
    "TabBlock",
    "TabBlockFileUpload",
    "TabDescriptor",
    "TabView",
    "TaskHandlerDescriptor",
    "TabViewV1",
    "TabViewV2",
    "TypedNamespace",
    "WorkerDescriptor",
    "WorkerNotFound",
    "__version__",
    "badge_list",
    "button_route",
    "button_task",
    "chart_bar",
    "chart_line",
    "discover_apps",
    "empty",
    "file_upload_block",
    "heading",
    "kv",
    "metric",
    "nexus_app_files",
    "nexus_command",
    "nexus_route",
    "nexus_tab",
    "nexus_worker",
    "render_tabview_to_html",
    "section",
    "table_",
    "task_handler",
    "text",
    "view",
]
