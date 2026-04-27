# SPDX-License-Identifier: AGPL-3.0-or-later
"""``nexus-coordinator start <name>`` — boot the coordinator process."""

from __future__ import annotations

import asyncio
import logging
import os
import signal
import sys
from pathlib import Path

import structlog
import typer
import uvicorn
from rich.console import Console

from nexus_coordinator.api.app import create_app
from nexus_coordinator.auth import coordinator_socket_path, sbfb_run_dir
from nexus_coordinator.coordinator import Coordinator
from nexus_coordinator.paths import coord_config_path, project_dir
from nexus_coordinator.registry import remove_running_state, write_running_state

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
    cors_origin: list[str] | None = typer.Option(
        None,
        "--cors-origin",
        help="Extra CORS origins to allow (repeatable). Env fallback: NEXUS_COORD__NETWORK__CORS_ORIGINS (comma-separated).",
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

    resolved_cors: list[str] = list(cors_origin) if cors_origin else []
    if not resolved_cors:
        env_cors = os.environ.get("NEXUS_COORD__NETWORK__CORS_ORIGINS", "")
        if env_cors:
            resolved_cors = [o.strip() for o in env_cors.split(",") if o.strip()]

    asyncio.run(_run(name, port=port, host=host, cors_origins=resolved_cors or None))


async def _run(
    name: str,
    *,
    port: int | None,
    host: str | None,
    cors_origins: list[str] | None,
) -> None:
    """Inner async runner split out so the top-level command stays
    sync-friendly for Typer."""
    coord = Coordinator(name)
    if port is not None:
        coord.config.network.api_port = port
    if host is not None:
        coord.config.network.api_host = host

    log = structlog.get_logger(__name__)

    await coord.start()

    # Sprint 5 Phase A D1: write the shell-facing running.json
    # entry. Removed in the `finally:` block below on clean
    # shutdown; crashes leave the file behind, and the shell
    # detects stale entries via the /health roundtrip.
    try:
        write_running_state(coord)
    except Exception as e:  # noqa: BLE001
        log.warning(
            "failed to write running.json; shell discovery will skip this coordinator",
            error=str(e),
        )

    app = create_app(coord, cors_origins=cors_origins)

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

    # Sprint 16 Phase B (D2): bind a parallel UDS server on
    # Linux/macOS at ~/.sbfb/run/coordinator.sock with mode 0600.
    # The bearer token middleware still runs over UDS — uvicorn's
    # ASGI scope does not expose the connection FD so the
    # SO_PEERCRED bypass equivalent to the Rust daemon's UDS path
    # is deferred to Sprint 17 (cf. R3 in sprint16_plan.md).
    # File-mode 0600 + parent dir 0700 are the gate.
    # Windows: deferred to Sprint 17 — the Named Pipe DACL path
    # would require a Rust side-car or pywin32 (R3 decision
    # documented in the Phase B kickoff §D2 implications).
    uds_server, uds_path = _maybe_build_uds_server(app, log)

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
    uds_task: asyncio.Task[None] | None = None
    if uds_server is not None:
        uds_task = asyncio.create_task(uds_server.serve(), name="uvicorn-server-uds")

    try:
        # Race the uvicorn task vs the stop event; whichever fires
        # first drives the shutdown path.
        stopper = asyncio.create_task(stop_event.wait(), name="stop-waiter")
        watch_set: set[asyncio.Task[object]] = {server_task, stopper}
        if uds_task is not None:
            watch_set.add(uds_task)
        done, pending = await asyncio.wait(
            watch_set,
            return_when=asyncio.FIRST_COMPLETED,
        )
        for task in pending:
            task.cancel()
    except (KeyboardInterrupt, asyncio.CancelledError):
        pass
    finally:
        server.should_exit = True
        if uds_server is not None:
            uds_server.should_exit = True
        try:
            await asyncio.wait_for(server_task, timeout=5)
        except (asyncio.TimeoutError, asyncio.CancelledError):
            server_task.cancel()
        if uds_task is not None:
            try:
                await asyncio.wait_for(uds_task, timeout=5)
            except (asyncio.TimeoutError, asyncio.CancelledError):
                uds_task.cancel()
        if uds_path is not None and uds_path.exists():
            try:
                uds_path.unlink()
            except OSError:
                pass  # best-effort cleanup
        await coord.stop()
        # Sprint 5 Phase A D1: best-effort running.json removal.
        # Crashes before this point leave the file behind; the
        # shell detects those via /health roundtrip.
        remove_running_state(name)


def _maybe_build_uds_server(
    app: object,
    log: structlog.BoundLogger,
) -> tuple[uvicorn.Server | None, Path | None]:
    """Return ``(server, sock_path)`` for the parallel UDS uvicorn
    on Linux/macOS, or ``(None, None)`` on Windows / when no path
    can be resolved. Sprint 16 Phase B (D2).
    """
    if sys.platform not in ("linux", "darwin", "freebsd"):
        return None, None
    sock_path = coordinator_socket_path()
    if sock_path is None:
        log.warning("could not resolve coordinator UDS path; UDS server disabled")
        return None, None

    run_dir = sbfb_run_dir()
    if run_dir is not None:
        run_dir.mkdir(parents=True, exist_ok=True)
        try:
            os.chmod(run_dir, 0o700)
        except OSError as e:  # pragma: no cover — non-fatal
            log.warning("failed to chmod run dir 0700", path=str(run_dir), error=str(e))

    if sock_path.exists():
        try:
            sock_path.unlink()
        except OSError as e:
            log.warning("could not remove stale UDS socket", path=str(sock_path), error=str(e))
            return None, None

    config = uvicorn.Config(
        app=app,
        uds=str(sock_path),
        log_config=None,
        access_log=False,
    )
    server = uvicorn.Server(config)

    # uvicorn binds the socket inside `serve()` (lifespan). chmod
    # the socket file once it appears: schedule a tiny background
    # task that polls for the socket's existence and applies 0600.
    async def _chmod_when_bound() -> None:
        for _ in range(50):  # ~5 seconds total
            if sock_path.exists():
                try:
                    os.chmod(sock_path, 0o600)
                except OSError as e:  # pragma: no cover
                    log.warning(
                        "failed to chmod UDS socket 0600",
                        path=str(sock_path),
                        error=str(e),
                    )
                return
            await asyncio.sleep(0.1)
        log.warning("UDS socket did not appear within 5s; skipping chmod", path=str(sock_path))

    asyncio.get_event_loop().create_task(_chmod_when_bound(), name="uds-chmod")
    log.info("coordinator UDS server bound", path=str(sock_path))
    return server, sock_path


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
