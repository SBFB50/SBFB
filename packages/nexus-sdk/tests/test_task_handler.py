# SPDX-License-Identifier: AGPL-3.0-or-later
"""Tests for the @task_handler decorator and registry integration."""

from __future__ import annotations

from nexus_sdk import (
    AppContext,
    AppManifest,
    NexusApp,
    TaskHandlerDescriptor,
    task_handler,
)
from nexus_sdk.registry import TASK_HANDLER_ATTR
from pydantic import BaseModel, Field


class TranslateRequest(BaseModel):
    text: str
    target_lang: str


class TranslateResponse(BaseModel):
    translated: str


class SummarizeRequest(BaseModel):
    document: str
    max_length: int = Field(default=200, ge=10, le=5000)


class SummarizeResponse(BaseModel):
    summary: str
    word_count: int


class HandlerApp(NexusApp):
    manifest = AppManifest(name="handler-test", version="0.1.0")

    @task_handler(TranslateRequest, TranslateResponse)
    async def translate(self, req: TranslateRequest) -> TranslateResponse:
        return TranslateResponse(translated="done")

    @task_handler(SummarizeRequest, SummarizeResponse)
    async def summarize(self, req: SummarizeRequest) -> SummarizeResponse:
        return SummarizeResponse(summary="short", word_count=1)

    async def on_start(self, ctx: AppContext) -> None:
        pass

    async def on_stop(self) -> None:
        pass


def test_task_handler_stores_schema() -> None:
    @task_handler(TranslateRequest, TranslateResponse)
    async def my_handler(req: TranslateRequest) -> TranslateResponse:
        return TranslateResponse(translated="x")

    meta = getattr(my_handler, TASK_HANDLER_ATTR)
    assert "request_schema" in meta
    assert "response_schema" in meta


def test_task_handler_request_schema_valid_json_schema() -> None:
    @task_handler(TranslateRequest, TranslateResponse)
    async def my_handler(req: TranslateRequest) -> TranslateResponse:
        return TranslateResponse(translated="x")

    meta = getattr(my_handler, TASK_HANDLER_ATTR)
    schema = meta["request_schema"]
    assert schema["type"] == "object"
    assert "text" in schema["properties"]
    assert "target_lang" in schema["properties"]
    assert set(schema["required"]) == {"text", "target_lang"}


def test_task_handler_response_schema_valid_json_schema() -> None:
    @task_handler(TranslateRequest, TranslateResponse)
    async def my_handler(req: TranslateRequest) -> TranslateResponse:
        return TranslateResponse(translated="x")

    meta = getattr(my_handler, TASK_HANDLER_ATTR)
    schema = meta["response_schema"]
    assert schema["type"] == "object"
    assert "translated" in schema["properties"]


def test_task_handler_with_optional_fields() -> None:
    @task_handler(SummarizeRequest, SummarizeResponse)
    async def my_handler(req: SummarizeRequest) -> SummarizeResponse:
        return SummarizeResponse(summary="x", word_count=1)

    meta = getattr(my_handler, TASK_HANDLER_ATTR)
    schema = meta["request_schema"]
    assert "max_length" in schema["properties"]
    assert schema["properties"]["max_length"]["default"] == 200
    assert "document" in schema["required"]
    assert "max_length" not in schema["required"]


def test_task_handler_registry_collects() -> None:
    app = HandlerApp()
    handlers = app.task_handlers()
    assert len(handlers) == 2
    names = {h.name for h in handlers}
    assert "translate" in names
    assert "summarize" in names


def test_task_handler_registry_sorted_by_name() -> None:
    app = HandlerApp()
    handlers = app.task_handlers()
    assert handlers[0].name < handlers[1].name


def test_task_handler_descriptor_has_schemas() -> None:
    app = HandlerApp()
    translate = next(h for h in app.task_handlers() if h.name == "translate")
    assert isinstance(translate, TaskHandlerDescriptor)
    assert translate.request_schema["type"] == "object"
    assert translate.response_schema["type"] == "object"
    assert "text" in translate.request_schema["properties"]
    assert "translated" in translate.response_schema["properties"]


def test_task_handler_captures_docstring() -> None:
    @task_handler(TranslateRequest, TranslateResponse)
    async def documented_handler(req: TranslateRequest) -> TranslateResponse:
        """Translate text into target language."""
        return TranslateResponse(translated="x")

    meta = getattr(documented_handler, TASK_HANDLER_ATTR)
    assert meta["description"] == "Translate text into target language."

    @task_handler(TranslateRequest, TranslateResponse)
    async def undocumented_handler(req: TranslateRequest) -> TranslateResponse:
        return TranslateResponse(translated="x")

    meta2 = getattr(undocumented_handler, TASK_HANDLER_ATTR)
    assert meta2["description"] == ""


def test_task_handler_descriptor_has_description() -> None:
    class DocApp(NexusApp):
        manifest = AppManifest(name="doc-test", version="0.1.0")

        @task_handler(TranslateRequest, TranslateResponse)
        async def translate(self, req: TranslateRequest) -> TranslateResponse:
            """Translate things."""
            return TranslateResponse(translated="x")

        async def on_start(self, ctx: AppContext) -> None:
            pass

        async def on_stop(self) -> None:
            pass

    app = DocApp()
    handlers = app.task_handlers()
    assert len(handlers) == 1
    assert handlers[0].description == "Translate things."


def test_app_without_task_handlers_returns_empty() -> None:
    class PlainApp(NexusApp):
        manifest = AppManifest(name="plain", version="0.1.0")

        async def on_start(self, ctx: AppContext) -> None:
            pass

        async def on_stop(self) -> None:
            pass

    app = PlainApp()
    assert app.task_handlers() == []
