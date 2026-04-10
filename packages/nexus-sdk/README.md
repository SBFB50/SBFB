# nexus-sdk

Build apps that run as plugins inside a `nexus-coordinator` process.
Define routes, workers, and tabs; the coordinator discovers and
mounts them at boot.

## Hello world

```python
from nexus_sdk import AppManifest, NexusApp, nexus_route, nexus_worker, nexus_tab

class HelloApp(NexusApp):
    manifest = AppManifest(name="hello", version="0.1.0")

    @nexus_route("/hello")
    async def hello(self):
        return {"message": "hi from nexus-sdk"}

    @nexus_worker(name="hello_worker", model="stub-model:latest")
    async def worker(self, ctx):
        return await ctx.compute.submit_task(
            task_type="analysis",
            prompt="Say hi",
            model="stub-model:latest",
        )

    @nexus_tab(name="Hello", icon="wave")
    def tab(self):
        return {"description": "Hello world tab"}
```

Package it with an `entry_points = {"nexus.apps": ["hello = mypkg:HelloApp"]}`
stanza and `uv sync`; the coordinator's `loader.discover_apps()`
picks it up next boot.
