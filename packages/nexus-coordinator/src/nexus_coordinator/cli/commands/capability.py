# SPDX-License-Identifier: AGPL-3.0-or-later
"""``nexus-admin capability`` sub-app: list / enable / disable / info / audit-trail.

Sprint 25 Phase D — operator CLI for the D5 capabilities gate.
Pattern mirrors ``quarantine.py`` S21 Phase D and ``canary.py``
S22 Phase E.

``enable`` / ``disable`` call :func:`~nexus_coordinator.admin_check.require_admin`
before mutating ``~/.sbfb/capabilities.toml``.
"""

from __future__ import annotations

import getpass
import json

import typer
from rich.console import Console
from rich.table import Table

from nexus_coordinator.admin_check import require_admin
from nexus_coordinator.capability_store import (
    CAPABILITY_DESCRIPTIONS,
    KNOWN_CAPABILITIES,
    CapabilitiesStore,
)

app = typer.Typer(
    help="Manage capability toggles (gate-off-by-default).",
    no_args_is_help=True,
)
console = Console()


@app.command("list")
def list_cmd(
    json_output: bool = typer.Option(False, "--json", help="Emit JSON (script-friendly)."),
) -> None:
    """Show all capabilities and their current status."""
    store = CapabilitiesStore.load()
    trail = store.audit_trail()
    if json_output:
        console.print_json(json.dumps({"capabilities": trail}))
        return
    table = Table(show_header=True, header_style="bold cyan")
    table.add_column("capability")
    table.add_column("enabled")
    table.add_column("enabled_at")
    table.add_column("enabled_by")
    for entry in trail:
        status = "[green]ON[/green]" if entry["enabled"] else "[dim]OFF[/dim]"
        table.add_row(
            entry["capability"],
            status,
            entry["enabled_at"] or "-",
            entry["enabled_by"] or "-",
        )
    console.print(table)


@app.command("enable")
def enable_cmd(
    name: str = typer.Argument(..., help="Capability name to enable."),
) -> None:
    """Enable a capability (requires admin privilege)."""
    _validate_name(name)
    require_admin()
    store = CapabilitiesStore.load()
    actor = getpass.getuser()
    store.enable(name, actor)
    console.print(f"[green]ok[/green] capability [cyan]{name}[/cyan] enabled by [cyan]{actor}[/cyan]")


@app.command("disable")
def disable_cmd(
    name: str = typer.Argument(..., help="Capability name to disable."),
) -> None:
    """Disable a capability (requires admin privilege)."""
    _validate_name(name)
    require_admin()
    store = CapabilitiesStore.load()
    store.disable(name)
    console.print(f"[green]ok[/green] capability [cyan]{name}[/cyan] disabled")


@app.command("info")
def info_cmd(
    name: str = typer.Argument(..., help="Capability name."),
) -> None:
    """Show description and current state for a capability."""
    _validate_name(name)
    store = CapabilitiesStore.load()
    entry = store.get(name)
    desc = CAPABILITY_DESCRIPTIONS.get(name, "no description")
    status = "ON" if entry and entry.enabled else "OFF"
    console.print(f"[bold]{name}[/bold] — {desc}")
    console.print(f"  status: [cyan]{status}[/cyan]")
    if entry and entry.enabled_at:
        console.print(f"  enabled_at: [cyan]{entry.enabled_at}[/cyan]")
        console.print(f"  enabled_by: [cyan]{entry.enabled_by}[/cyan]")


@app.command("audit-trail")
def audit_trail_cmd(
    json_output: bool = typer.Option(False, "--json", help="Emit JSON (script-friendly)."),
) -> None:
    """Print the capabilities audit trail from the TOML file."""
    store = CapabilitiesStore.load()
    trail = store.audit_trail()
    if json_output:
        console.print_json(json.dumps({"audit_trail": trail}))
        return
    active = [e for e in trail if e["enabled"]]
    if not active:
        console.print("[dim]no capability has been enabled yet[/dim]")
        return
    table = Table(show_header=True, header_style="bold cyan")
    table.add_column("capability")
    table.add_column("enabled_at")
    table.add_column("enabled_by")
    for entry in active:
        table.add_row(
            entry["capability"],
            entry["enabled_at"],
            entry["enabled_by"],
        )
    console.print(table)


def _validate_name(name: str) -> None:
    if name not in KNOWN_CAPABILITIES:
        console.print(
            f"[red]error[/red]: unknown capability [cyan]{name}[/cyan]. Known: {', '.join(sorted(KNOWN_CAPABILITIES))}"
        )
        raise typer.Exit(code=1)
