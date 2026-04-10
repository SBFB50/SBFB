"""GovApp: minimal nexus-app-gov migration.

Exposes a single route (``GET /statements``), a single worker
(``contradiction_detector`` backed by
:data:`POLITICAL_CONTRADICTION_PROMPT`), and a single tab
descriptor so the coordinator's manifest has something to
advertise on Phase D's closure criteria.

Phase D is deliberately small — Decision F of the sprint kickoff
caps gov migration at 1/1/1 to keep Sprint 4 to 14 days. The full
19-tab / 31-worker port from the legacy ``nexus/gov/`` tree lands
in v1.1.
"""

from __future__ import annotations

from typing import Any

from nexus_sdk import (
    AppContext,
    AppManifest,
    NexusApp,
    nexus_route,
    nexus_tab,
    nexus_worker,
)

from nexus_app_gov.prompts import POLITICAL_CONTRADICTION_PROMPT


class GovApp(NexusApp):
    """Political contradiction detection — minimal plugin."""

    manifest = AppManifest(
        name="gov",
        version="0.1.0",
        author="FlowUP",
        description="Detect logical contradictions in political statements via LLM analysis.",
        license="AGPL-3.0",
    )

    def __init__(self) -> None:
        super().__init__()
        self._ctx: AppContext | None = None

    async def on_start(self, ctx: AppContext) -> None:
        self._ctx = ctx

    async def on_stop(self) -> None:
        self._ctx = None

    # ------------------------------------------------------------------
    # Routes
    # ------------------------------------------------------------------

    @nexus_route("/statements", methods=["GET"])
    async def list_statements(self) -> dict[str, Any]:
        """Tiny placeholder route so the coordinator's manifest
        has a concrete URL to expose under ``/app/gov/``.

        Phase D+ migration will wire this to the legacy
        ``nexus/gov/`` handlers.
        """
        return {
            "app": "gov",
            "status": "ready",
            "prompt_template": POLITICAL_CONTRADICTION_PROMPT.splitlines()[0],
        }

    # ------------------------------------------------------------------
    # Workers
    # ------------------------------------------------------------------

    @nexus_worker(name="contradiction_detector", model="stub-model:latest")
    async def contradiction_detector(self, ctx: AppContext) -> dict[str, Any]:
        """Submit a contradiction-detection task via the
        coordinator's ``/tasks/submit`` endpoint.

        Returns the submitted task id so callers can poll
        ``/tasks`` or ``/results`` for the final answer.
        """
        task = await ctx.compute.submit_task(
            task_type="contradiction_check",
            prompt=POLITICAL_CONTRADICTION_PROMPT.format(statements="(example)"),
            model="stub-model:latest",
            priority=5,
        )
        return {"task_id": task.task_id}

    # ------------------------------------------------------------------
    # Tabs
    # ------------------------------------------------------------------

    @nexus_tab(name="Contradictions", icon="alert-octagon")
    def contradictions_tab(self) -> dict[str, Any]:
        return {
            "description": "Review detected contradictions across political statements.",
            "route": "/statements",
        }
