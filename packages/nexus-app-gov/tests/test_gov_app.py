"""Smoke tests for the minimal gov app migration."""

from __future__ import annotations

import pytest
from nexus_app_gov import POLITICAL_CONTRADICTION_PROMPT, GovApp
from nexus_sdk import AppContext, ComputeClient


def test_gov_app_manifest_and_descriptors() -> None:
    app = GovApp()
    assert app.manifest.name == "gov"
    assert app.manifest.version == "0.1.0"

    routes = app.routes()
    workers = app.workers()
    tabs = app.tabs()

    assert len(routes) == 1
    assert routes[0].path == "/statements"

    assert len(workers) == 1
    assert workers[0].name == "contradiction_detector"
    assert workers[0].model == "stub-model:latest"

    assert len(tabs) == 1
    assert tabs[0].name == "Contradictions"


def test_political_contradiction_prompt_is_present() -> None:
    assert "contradiction" in POLITICAL_CONTRADICTION_PROMPT.lower()
    assert "{statements}" in POLITICAL_CONTRADICTION_PROMPT


@pytest.mark.asyncio
async def test_on_start_and_list_statements() -> None:
    app = GovApp()
    ctx = AppContext(
        compute=ComputeClient("http://127.0.0.1:65500"),
        project_name="gov-test",
    )
    await app.on_start(ctx)
    body = await app.list_statements()
    assert body["app"] == "gov"
    assert body["status"] == "ready"
    await app.on_stop()
