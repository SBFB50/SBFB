"""
nexus-worker — CLI entry points.

Commands:
    nexus-worker register --server URL --name NAME
    nexus-worker start [--no-dashboard]
    nexus-worker stats
    nexus-worker config
"""

from __future__ import annotations

import argparse
import asyncio
import sys
from pathlib import Path

from loguru import logger

from worker import __version__
from worker.config import load_config, save_config, get_config_path, is_registered
from worker.gpu_detect import detect_gpu, format_vram
from worker.client import NexusClient
from worker.engine import WorkerEngine, WorkerState


def _setup_logger(verbose: bool = False) -> None:
    """Configure loguru for CLI output."""
    logger.remove()
    level = "DEBUG" if verbose else "INFO"
    logger.add(sys.stderr, level=level, format="<level>{message}</level>")


# ============================================================================
# Register command
# ============================================================================

async def _do_register(server: str, name: str) -> None:
    """Register this node with the NEXUS compute server."""
    from rich.console import Console
    console = Console()

    console.print(f"\n[bold]NEXUS Worker Registration[/]\n")

    # Detect GPU
    console.print("[dim]Detecting GPU...[/]")
    gpu = detect_gpu()
    console.print(f"  GPU: [bold]{gpu['gpu_model']}[/] ({format_vram(gpu['vram_mb'])})")
    console.print(f"  Platform: {gpu['platform']}")

    if gpu["vram_mb"] == 0:
        console.print("[red]No GPU detected. A GPU is required to contribute.[/]")
        return

    # Detect Ollama
    ollama_version = ""
    try:
        import httpx
        async with httpx.AsyncClient(timeout=5.0) as client:
            resp = await client.get("http://localhost:11434/api/version")
            if resp.status_code == 200:
                ollama_version = resp.json().get("version", "")
                console.print(f"  Ollama: v{ollama_version}")
    except Exception:
        console.print("[yellow]  Ollama: not detected (install from ollama.com)[/]")

    # Generate Ed25519 keypair for result signing
    private_key_pem = ""
    public_key_pem = ""
    try:
        from nexus.compute.crypto import generate_keypair
        priv, pub = generate_keypair()
        if priv and pub:
            private_key_pem = priv.decode("utf-8")
            public_key_pem = pub.decode("utf-8")
            console.print("  Ed25519: [green]keypair generated[/]")
    except ImportError:
        console.print("  Ed25519: [yellow]cryptography not installed (signing disabled)[/]")

    # Register with server
    console.print(f"\n[dim]Registering with {server}...[/]")
    client = NexusClient(server_url=server)

    try:
        result = await client.register(
            name=name,
            gpu_model=gpu["gpu_model"],
            vram_mb=gpu["vram_mb"],
            platform=gpu["platform"],
            ollama_version=ollama_version,
            public_key_pem=public_key_pem,
        )
    except Exception as exc:
        console.print(f"[red]Registration failed: {exc}[/]")
        await client.close()
        return

    await client.close()

    # Save config (private key stays local)
    config = load_config()
    config.update({
        "server_url": server,
        "node_id": result["node_id"],
        "api_key": result["api_key"],
        "name": name,
        "gpu_model": gpu["gpu_model"],
        "vram_mb": gpu["vram_mb"],
        "platform": gpu["platform"],
        "private_key_pem": private_key_pem,
    })
    save_config(config)

    console.print(f"\n[green bold]Registration successful![/]")
    console.print(f"  Node ID: {result['node_id'][:12]}...")
    console.print(f"  Config saved to: {get_config_path()}")
    console.print(f"\n  Run [bold]nexus-worker start[/] to begin contributing.\n")


# ============================================================================
# Start command
# ============================================================================

async def _do_start(no_dashboard: bool = False, exo_mode: bool = False, petals_mode: bool = False, sync_mode: bool = False) -> None:
    """Start the worker and begin processing tasks."""
    from rich.console import Console
    console = Console()

    if not is_registered():
        console.print("[red]Not registered. Run: nexus-worker register --server URL --name NAME[/]")
        return

    config = load_config()
    server_url = config["server_url"]
    api_key = config["api_key"]
    name = config["name"]
    gpu_model = config.get("gpu_model", "Unknown")
    vram_mb = config.get("vram_mb", 0)

    mode_label = "[magenta]Petals server[/]" if petals_mode else "[cyan]exo peer[/]" if exo_mode else "Ollama local"
    console.print(f"\n[bold]NEXUS Worker — {name}[/]")
    console.print(f"  GPU: {gpu_model} ({format_vram(vram_mb)})")
    console.print(f"  Server: {server_url}")
    console.print(f"  Mode: {mode_label}")
    console.print(f"  Starting...\n")

    # Start exo peer if requested
    exo_peer = None
    if exo_mode:
        from worker.exo_peer import ExoPeer
        if not ExoPeer.is_exo_installed():
            console.print("[red]exo not found. Install with: pip install exo[/]")
            return
        exo_peer = ExoPeer(initial_peers=server_url)
        started = await exo_peer.start()
        if not started:
            console.print("[red]Failed to start exo peer.[/]")
            return
        console.print("[green]exo peer started[/]")

    # Start Petals server peer if requested
    petals_proc = None
    if petals_mode:
        try:
            import subprocess
            import shutil
            if not shutil.which("python"):
                console.print("[red]Python not found for Petals server[/]")
                return
            console.print("[dim]Starting Petals server peer...[/]")
            petals_proc = await asyncio.create_subprocess_exec(
                "python", "-m", "petals.cli.run_server",
                config.get("petals_model", "meta-llama/Meta-Llama-3.1-405B"),
                "--port", "31330",
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            console.print("[green]Petals server peer started (port 31330)[/]")
        except Exception as exc:
            console.print(f"[red]Petals server failed: {exc}[/]")
            console.print("[yellow]Install with: pip install petals[/]")

    # Start sync receiver if requested
    sync_receiver = None
    if sync_mode:
        try:
            from nexus.sync.receiver import SyncReceiver
            ws_url = server_url.replace("http://", "ws://").replace("https://", "wss://")
            sync_receiver = SyncReceiver(
                server_url=f"{ws_url}/ws/sync",
                snapshot_url=f"{server_url}/api/sync/snapshot",
            )
            await sync_receiver.start()
            console.print(f"[green]Sync receiver started (local DB: ~/.nexus-worker/nexus_local.db)[/]")
        except ImportError:
            console.print("[yellow]nexus.sync not available — sync disabled[/]")
        except Exception as exc:
            console.print(f"[yellow]Sync failed: {exc}[/]")

    # Create client and engine
    client = NexusClient(
        server_url=server_url,
        api_key=api_key,
    )
    engine = WorkerEngine(
        client=client,
        ollama_url=config.get("ollama_url", "http://localhost:11434"),
        poll_interval=config.get("poll_interval", 2.0),
        heartbeat_interval=config.get("heartbeat_interval", 15.0),
        node_id=config.get("node_id", ""),
        private_key_pem=config.get("private_key_pem", "").encode("utf-8"),
    )

    # Start engine
    await engine.start()

    if no_dashboard:
        # Simple mode: just run until Ctrl+C
        try:
            while engine.state != WorkerState.STOPPED:
                await asyncio.sleep(1)
        except (KeyboardInterrupt, asyncio.CancelledError):
            pass
    else:
        # Dashboard mode
        from worker.dashboard import run_dashboard

        # Run dashboard and keyboard handler concurrently
        dashboard_task = asyncio.create_task(
            run_dashboard(engine, name, gpu_model, vram_mb)
        )
        keyboard_task = asyncio.create_task(
            _keyboard_handler(engine)
        )

        try:
            await asyncio.gather(dashboard_task, keyboard_task, return_exceptions=True)
        except (KeyboardInterrupt, asyncio.CancelledError):
            pass

    await engine.stop()
    if exo_peer:
        await exo_peer.stop()
    if petals_proc and petals_proc.returncode is None:
        petals_proc.terminate()
        try:
            await asyncio.wait_for(petals_proc.wait(), timeout=10)
        except asyncio.TimeoutError:
            petals_proc.kill()
    if sync_receiver:
        await sync_receiver.stop()
    console.print("\n[dim]Worker stopped.[/]")


async def _keyboard_handler(engine: WorkerEngine) -> None:
    """Handle keyboard input (Q=quit, P=pause/resume, S=stats)."""
    loop = asyncio.get_event_loop()

    try:
        while engine.state != WorkerState.STOPPED:
            # Non-blocking read with asyncio
            try:
                key = await asyncio.wait_for(
                    loop.run_in_executor(None, _read_key),
                    timeout=0.5,
                )
            except asyncio.TimeoutError:
                continue

            if key in ("q", "Q"):
                await engine.stop()
                break
            elif key in ("p", "P"):
                if engine.state == WorkerState.PAUSED:
                    engine.resume()
                else:
                    engine.pause()
    except (asyncio.CancelledError, EOFError):
        pass


def _read_key() -> str:
    """Read a single key from stdin (blocking)."""
    try:
        import msvcrt
        if msvcrt.kbhit():
            return msvcrt.getch().decode("utf-8", errors="ignore")
    except ImportError:
        pass

    try:
        import sys
        import select
        if select.select([sys.stdin], [], [], 0.1)[0]:
            return sys.stdin.read(1)
    except Exception:
        pass

    raise TimeoutError()


# ============================================================================
# Stats command
# ============================================================================

async def _do_stats() -> None:
    """Show current network stats and leaderboard."""
    from rich.console import Console
    from rich.table import Table
    console = Console()

    if not is_registered():
        console.print("[red]Not registered. Run: nexus-worker register --server URL --name NAME[/]")
        return

    config = load_config()
    client = NexusClient(server_url=config["server_url"], api_key=config["api_key"])

    try:
        stats = await client.get_stats()
        lb_data = await client.get_leaderboard(limit=20)
        model_status = await client.get_model_status()
    except Exception as exc:
        console.print(f"[red]Failed to fetch stats: {exc}[/]")
        await client.close()
        return

    await client.close()

    # Network stats
    console.print(f"\n[bold]NEXUS Network Stats[/]\n")
    console.print(f"  Nodes online: [bold]{stats.get('nodes_online', 0)}[/]")
    console.print(f"  VRAM total: [bold]{stats.get('vram_total_gb', 0):.0f} GB[/]")
    console.print(f"  Model: [bold]{stats.get('current_model', '?')}[/] ({stats.get('model_tier', '?')})")
    console.print(f"  Tasks today: [bold]{stats.get('tasks_today', 0):,}[/]")
    console.print(f"  Pending: {stats.get('tasks_pending', 0)} | Assigned: {stats.get('tasks_assigned', 0)}")

    # Model status
    ts = model_status.get("transition_state", "stable")
    if ts != "stable":
        console.print(f"\n  [yellow]Transition: {ts} ({model_status.get('readiness_pct', 0):.0f}% ready)[/]")

    # Leaderboard
    entries = lb_data.get("entries", [])
    if entries:
        console.print(f"\n[bold]Leaderboard[/]\n")
        table = Table(show_header=True, header_style="bold")
        table.add_column("#", width=4)
        table.add_column("Name", width=20)
        table.add_column("GPU", width=20)
        table.add_column("Tasks", justify="right", width=10)
        table.add_column("Speed", justify="right", width=10)

        name = config.get("name", "")
        for e in entries:
            style = "bold green" if e.get("name") == name else ""
            table.add_row(
                str(e.get("rank", "")),
                e.get("name", ""),
                e.get("gpu_model", ""),
                f"{e.get('tasks_completed', 0):,}",
                f"{e.get('avg_tokens_per_sec', 0):.1f} t/s",
                style=style,
            )

        console.print(table)

    console.print()


# ============================================================================
# Config command
# ============================================================================

def _do_config() -> None:
    """Show current configuration."""
    from rich.console import Console
    console = Console()

    config = load_config()
    console.print(f"\n[bold]NEXUS Worker Config[/]")
    console.print(f"  Path: {get_config_path()}")
    console.print(f"  Server: {config.get('server_url', '(not set)')}")
    console.print(f"  Name: {config.get('name', '(not set)')}")
    console.print(f"  Node ID: {config.get('node_id', '(not set)')[:12]}...")
    console.print(f"  API Key: {'***' if config.get('api_key') else '(not set)'}")
    console.print(f"  GPU: {config.get('gpu_model', '?')} ({format_vram(config.get('vram_mb', 0))})")
    console.print(f"  Ollama URL: {config.get('ollama_url', 'http://localhost:11434')}")
    console.print()


# ============================================================================
# Main CLI
# ============================================================================

def main() -> None:
    """CLI entry point."""
    parser = argparse.ArgumentParser(
        prog="nexus-worker",
        description="NEXUS GPU Contributor — share your GPU power for political transparency",
    )
    parser.add_argument("--version", action="version", version=f"nexus-worker {__version__}")
    parser.add_argument("-v", "--verbose", action="store_true", help="Verbose logging")

    sub = parser.add_subparsers(dest="command")

    # register
    reg = sub.add_parser("register", help="Register this GPU with the NEXUS network")
    reg.add_argument("--server", required=True, help="Server URL (e.g. https://nexusgov.fr)")
    reg.add_argument("--name", required=True, help="Your contributor display name")

    # start
    start = sub.add_parser("start", help="Start processing tasks")
    start.add_argument("--no-dashboard", action="store_true", help="Run without Rich TUI")
    start.add_argument("--exo", action="store_true", help="Run as exo peer (distributed model layers)")
    start.add_argument("--petals", action="store_true", help="Run as Petals server (host transformer blocks)")
    start.add_argument("--sync", action="store_true", help="Enable local DB sync (cr-sqlite real-time)")

    # stats
    sub.add_parser("stats", help="Show network stats and leaderboard")

    # config
    sub.add_parser("config", help="Show current configuration")

    args = parser.parse_args()
    _setup_logger(args.verbose if hasattr(args, "verbose") else False)

    if args.command == "register":
        asyncio.run(_do_register(args.server, args.name))
    elif args.command == "start":
        asyncio.run(_do_start(no_dashboard=args.no_dashboard, exo_mode=args.exo, petals_mode=args.petals, sync_mode=args.sync))
    elif args.command == "stats":
        asyncio.run(_do_stats())
    elif args.command == "config":
        _do_config()
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
