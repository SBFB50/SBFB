"""Minimal nexus-sdk example app — 1 route, 1 worker, 1 tab."""

from nexus_sdk import (
    AppContext,
    AppManifest,
    NexusApp,
    nexus_route,
    nexus_tab,
    nexus_worker,
)


class HelloApp(NexusApp):
    manifest = AppManifest(
        name="hello",
        version="0.1.0",
        description="Minimal hello world — under 100 LOC total.",
    )

    def __init__(self) -> None:
        super().__init__()
        self._ctx: AppContext | None = None

    async def on_start(self, ctx: AppContext) -> None:
        self._ctx = ctx

    async def on_stop(self) -> None:
        self._ctx = None

    @nexus_route("/hello")
    async def hello(self) -> dict[str, str]:
        return {"message": "hi from nexus-sdk"}

    @nexus_worker(name="hello_worker", model="stub-model:latest")
    async def worker(self, ctx: AppContext) -> dict[str, str]:
        task = await ctx.compute.submit_task(
            task_type="analysis",
            prompt="Say hi",
            model="stub-model:latest",
        )
        return {"task_id": task.task_id}

    @nexus_tab(name="Hello", icon="wave")
    def tab(self) -> dict[str, str]:
        return {"description": "Hello world"}
