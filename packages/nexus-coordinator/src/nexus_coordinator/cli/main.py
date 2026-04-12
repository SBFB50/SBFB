"""Entry point for the ``nexus-coordinator`` CLI.

Commands (Phase A):

- ``init <name>`` — create a project directory, keypair, and
  default ``coordinator.toml``.
- ``start <name>`` — boot the coordinator and serve the FastAPI
  control plane.

Phases B/C/D add ``stop``, ``invite``, ``stats``, ``kudos``, etc.
"""

from __future__ import annotations

import typer

from nexus_coordinator.cli.commands import invite as invite_cmds
from nexus_coordinator.cli.commands.init import init_cmd
from nexus_coordinator.cli.commands.migrate import migrate_cmd
from nexus_coordinator.cli.commands.start import start_cmd

app = typer.Typer(
    name="nexus-coordinator",
    help="Coordinator for a nexus-grid project — signs tasks, validates results, tracks kudos.",
    no_args_is_help=True,
    add_completion=False,
    rich_markup_mode="rich",
)

app.command("init", help="Create a new project directory, keypair, and config.")(init_cmd)
app.command("start", help="Boot the coordinator and serve the local control API.")(start_cmd)
app.command("migrate", help="Plan or apply database migrations for apps.")(migrate_cmd)
app.add_typer(invite_cmds.app, name="invite")


if __name__ == "__main__":
    app()
