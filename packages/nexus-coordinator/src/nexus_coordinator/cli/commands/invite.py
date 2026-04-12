# SPDX-License-Identifier: AGPL-3.0-or-later
"""``nexus-coordinator invite`` sub-app: create / list / revoke."""

from __future__ import annotations

import asyncio

import typer
from rich.console import Console
from rich.table import Table

from nexus_coordinator.coordinator import Coordinator

app = typer.Typer(help="Manage invite tokens for a project.", no_args_is_help=True)
console = Console()


@app.command("create")
def create(
    project: str = typer.Argument(..., help="Project name — must exist on disk."),
    scope: str = typer.Option("worker", help="'worker' or 'observer'"),
    expiry: str = typer.Option("7d", help="Expiry duration, e.g. 1h, 7d, 30d."),
    max_uses: int | None = typer.Option(None, help="Optional cap on how many workers may join with this invite."),
    note: str | None = typer.Option(None, help="Human note (batch id, deployment tag, ...)."),
) -> None:
    """Mint a new invite for the given project and print it to stdout."""
    expiry_secs = _parse_duration(expiry)
    asyncio.run(_do_create(project, scope, expiry_secs, max_uses, note))


async def _do_create(project: str, scope: str, expiry_secs: int, max_uses: int | None, note: str | None) -> None:
    coord = Coordinator(project_name=project)
    await coord.start()
    try:
        ledger = coord.invite_ledger
        assert ledger is not None, "invite ledger should be initialised by start()"
        record = await ledger.mint(
            project_id=coord.state.doc_id or "",
            project_name=project,
            scope=scope,
            tasks_doc_ticket=coord.state.tasks_doc_ticket,
            expiry_secs=expiry_secs,
            max_uses=max_uses,
            note=note,
        )
        console.print(f"[green]✓[/green] invite minted: [bold]{record.id}[/bold]")
        console.print(f"  scope    : {record.scope}")
        console.print(f"  expires  : unix {record.expires_at}")
        if record.max_uses is not None:
            console.print(f"  max_uses : {record.max_uses}")
        if record.note:
            console.print(f"  note     : {record.note}")
        console.print()
        console.print("[bold cyan]Share this token with the worker:[/bold cyan]")
        console.print(record.wire)
    finally:
        await coord.stop()


@app.command("list")
def list_cmd(
    project: str = typer.Argument(..., help="Project name."),
) -> None:
    """List all invites stored in the project's invites table."""
    asyncio.run(_do_list(project))


async def _do_list(project: str) -> None:
    coord = Coordinator(project_name=project)
    await coord.start()
    try:
        ledger = coord.invite_ledger
        assert ledger is not None
        records = await ledger.list_invites()
        if not records:
            console.print("[dim]no invites yet[/dim]")
            return
        table = Table(show_header=True, header_style="bold cyan")
        table.add_column("id")
        table.add_column("scope")
        table.add_column("expires_at")
        table.add_column("uses")
        table.add_column("status")
        table.add_column("note")
        for r in records:
            status = "[red]revoked[/red]" if r.revoked_at else "[green]active[/green]"
            uses = f"{r.uses_count}/{r.max_uses}" if r.max_uses else f"{r.uses_count}"
            table.add_row(r.id, r.scope, str(r.expires_at), uses, status, r.note or "")
        console.print(table)
    finally:
        await coord.stop()


@app.command("revoke")
def revoke(
    project: str = typer.Argument(..., help="Project name."),
    invite_id: str = typer.Argument(..., help="Invite id (inv-...)."),
) -> None:
    """Revoke an invite. The signed wire token itself cannot be
    invalidated cryptographically, but the coordinator will
    refuse to accept it once marked revoked."""
    asyncio.run(_do_revoke(project, invite_id))


async def _do_revoke(project: str, invite_id: str) -> None:
    coord = Coordinator(project_name=project)
    await coord.start()
    try:
        ledger = coord.invite_ledger
        assert ledger is not None
        ok = await ledger.revoke(invite_id)
        if ok:
            console.print(f"[green]✓[/green] revoked {invite_id}")
        else:
            console.print(f"[red]error[/red]: {invite_id} not found or already revoked")
            raise typer.Exit(code=1)
    finally:
        await coord.stop()


def _parse_duration(expr: str) -> int:
    """Parse '1h' / '7d' / '30d' / '3600' into seconds."""
    expr = expr.strip().lower()
    if not expr:
        raise typer.BadParameter("empty duration")
    if expr.isdigit():
        return int(expr)
    unit = expr[-1]
    value_str = expr[:-1]
    if not value_str.isdigit():
        raise typer.BadParameter(f"bad duration {expr!r}, expected e.g. '7d', '1h', '3600'")
    value = int(value_str)
    if unit == "s":
        return value
    if unit == "m":
        return value * 60
    if unit == "h":
        return value * 3600
    if unit == "d":
        return value * 86400
    raise typer.BadParameter(f"unknown unit {unit!r} in duration {expr!r}")
