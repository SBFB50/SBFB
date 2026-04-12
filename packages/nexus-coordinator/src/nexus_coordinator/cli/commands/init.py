# SPDX-License-Identifier: AGPL-3.0-or-later
"""``nexus-coordinator init <name>`` — bootstrap a new project directory."""

from __future__ import annotations

import typer
from rich.console import Console

from nexus_coordinator.config import CoordinatorConfig
from nexus_coordinator.keystore import load_or_generate_keypair
from nexus_coordinator.paths import coord_config_path, coord_key_path, project_dir

console = Console()


def init_cmd(
    name: str = typer.Argument(..., help="Project name (used as the directory name)."),
    description: str = typer.Option("", "--description", "-d", help="Human description."),
    public: bool = typer.Option(
        False,
        "--public/--private",
        help="Publish the coordinator node id to the iroh pkarr DHT (Phase C+).",
    ),
    api_port: int = typer.Option(
        8765,
        "--port",
        help="Default bind port for the FastAPI control API.",
    ),
) -> None:
    """Create ``~/.nexus-grid/projects/<name>/`` with a keypair and
    default ``coordinator.toml``.

    Safe to re-run: if a ``coordinator.toml`` already exists at the
    target path, the command refuses to overwrite and exits
    non-zero. Delete the directory manually if you want to start
    fresh.
    """
    pdir = project_dir(name)
    config_path = coord_config_path(name)
    key_path = coord_key_path(name)

    if config_path.exists():
        console.print(
            f"[red]error[/red]: project [bold]{name}[/bold] already exists at [cyan]{pdir}[/cyan].\n"
            "Delete the directory manually if you want to re-init."
        )
        raise typer.Exit(code=1)

    pdir.mkdir(parents=True, exist_ok=True)

    # Generate the keypair first so a failure here doesn't leave a
    # half-written config file behind.
    kp = load_or_generate_keypair(key_path)

    config = CoordinatorConfig()
    config.identity.name = name
    config.identity.description = description
    config.network.api_port = api_port
    config.network.visibility = "public" if public else "private"
    config.save(config_path)

    console.print(f"[green]✓[/green] initialized project [bold]{name}[/bold]")
    console.print(f"  dir       : [cyan]{pdir}[/cyan]")
    console.print(f"  config    : [cyan]{config_path}[/cyan]")
    console.print(f"  keypair   : [cyan]{key_path}[/cyan]")
    console.print(f"  pubkey    : [magenta]{kp.public.hex()}[/magenta]")
    console.print(f"  visibility: {config.network.visibility}")
    console.print(f"  api_port  : {api_port}")
    console.print(f"\nNext: [bold]nexus-coordinator start {name}[/bold] to boot the iroh Node and control API.")
