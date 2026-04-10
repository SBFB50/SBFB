"""``nexus-coordinator start <name>`` — boot the coordinator process."""

from __future__ import annotations

import asyncio
import logging
import signal

import structlog
import typer
import uvicorn
from rich.console import Console

from nexus_coordinator.api.app import create_app
from nexus_coordinator.coordinator import Coordinator
from nexus_coordinator.paths import coord_config_path, project_dir

console = Console()


def start_cmd(
    name: str = typer.Argument(..., help="Project name — must have been created via `init` first."),
    port: int | None = typer.Option(
        None,
        "--port",
        help="Override the bind port (defaults to coordinator.toml network.api_port).",
    ),
    host: str | None = typer.Option(
        None,
        "--host",
        help="Override the bind host (defaults to coordinator.toml network.api_host).",
    ),
) -> None:
    """Boot the iroh Node, open the project doc, and serve the
    FastAPI control plane.

    Blocks until SIGINT or SIGTERM; on either, the coordinator is
    stopped gracefully and the iroh endpoint is closed.
    """
    config_path = coord_config_path(name)
    if not config_path.exists():
        console.print(
            f"[red]error[/red]: project [bold]{name}[/bold] does not exist.\n"
            f"Run [bold]nexus-coordinator init {name}[/bold] first."
        )
        raise typer.Exit(code=1)

    _configure_logging()

    _log = structlog.get_logger(__name__)
    _log.info(
        "starting coordinator",
        project=name,
        project_dir=str(project_dir(name)),
    )

    asyncio.run(_run(name, port=port, host=host))


async def _run(name: str, *, port: int | None, host: str | None) -> None:
    """Inner async runner split out so the top-level command stays
    sync-friendly for Typer."""
    coord = Coordinator(name)
    if port is not None:
        coord.config.network.api_port = port
    if host is not None:
        coord.config.network.api_host = host

    await coord.start()
    app = create_app(coord)

    # uvicorn.Server lets us run the server as a coroutine inside
    # the same event loop that owns the iroh Node, so the FastAPI
    # handlers can `await` coordinator state directly.
    server_config = uvicorn.Config(
        app=app,
        host=coord.config.network.api_host,
        port=coord.config.network.api_port,
        log_config=None,  # we already configured structlog
        access_log=False,
    )
    server = uvicorn.Server(server_config)

    stop_event = asyncio.Event()

    def _signal_handler() -> None:
        stop_event.set()

    loop = asyncio.get_running_loop()
    if hasattr(signal, "SIGTERM"):
        try:
            loop.add_signal_handler(signal.SIGTERM, _signal_handler)
        except NotImplementedError:
            # Windows asyncio proactor doesn't implement
            # add_signal_handler; we fall back to catching
            # KeyboardInterrupt on SIGINT below.
            pass

    server_task = asyncio.create_task(server.serve(), name="uvicorn-server")

    try:
        # Race the uvicorn task vs the stop event; whichever fires
        # first drives the shutdown path.
        stopper = asyncio.create_task(stop_event.wait(), name="stop-waiter")
        done, pending = await asyncio.wait(
            {server_task, stopper},
            return_when=asyncio.FIRST_COMPLETED,
        )
        for task in pending:
            task.cancel()
    except (KeyboardInterrupt, asyncio.CancelledError):
        pass
    finally:
        server.should_exit = True
        try:
            await asyncio.wait_for(server_task, timeout=5)
        except (asyncio.TimeoutError, asyncio.CancelledError):
            server_task.cancel()
        await coord.stop()


def _configure_logging() -> None:
    """Set up structlog with a console-friendly renderer.

    Plain-text on stdout for the interactive `start` command;
    Phase B/C can wire a JSON renderer when running under
    systemd/Docker.
    """
    logging.basicConfig(level=logging.INFO, format="%(message)s")
    structlog.configure(
        processors=[
            structlog.processors.add_log_level,
            structlog.processors.TimeStamper(fmt="iso", utc=True),
            structlog.dev.ConsoleRenderer(colors=False),
        ],
        wrapper_class=structlog.make_filtering_bound_logger(logging.INFO),
        cache_logger_on_first_use=True,
    )
