# SPDX-License-Identifier: AGPL-3.0-or-later
"""``nexus-coordinator quarantine`` sub-app: list / flush / drop.

Sprint 21 Phase D — operator-facing CLI for the quarantine queue.
The user-facing wording in kickoff §D4 ligne 594 is ``sbfb
quarantine list|flush|drop``; the binary name is the coordinator
Typer entry point — an eventual ``sbfb`` shell alias is hors-scope
Phase D (S22+ UX naming).

The CLI starts a transient :class:`Coordinator`, accesses
``coord.quarantine_queue`` directly, and stops the coordinator on
exit. This pattern mirrors ``cli/commands/invite.py`` and shares
its caveat: a production coordinator already running on the same
project would conflict with the transient one on the iroh node
data dir. Stop the production coord first, then run the CLI, then
restart. (HTTP-loopback CLI variant could remove this caveat in a
future sprint, but the REST endpoints are already in place — a
future ``--remote`` flag could switch the implementation without
breaking the CLI contract.)
"""

from __future__ import annotations

import asyncio
import json
from datetime import datetime, timezone

import typer
from rich.console import Console
from rich.table import Table

from nexus_coordinator.coordinator import Coordinator

app = typer.Typer(help="Inspect and resolve quarantined gossip messages.", no_args_is_help=True)
console = Console()


@app.command("list")
def list_cmd(
    project: str = typer.Argument(..., help="Project name — must exist on disk."),
    status: str = typer.Option(
        "pending",
        help="Filter by flush_status: pending | flushed | dropped | all.",
    ),
    json_output: bool = typer.Option(
        False,
        "--json",
        help="Emit JSON instead of a Rich table (script-friendly).",
    ),
) -> None:
    """List quarantine entries for the given project."""
    asyncio.run(_do_list(project, status, json_output))


async def _do_list(project: str, status: str, json_output: bool) -> None:
    coord = Coordinator(project_name=project)
    await coord.start()
    try:
        queue = coord.quarantine_queue
        assert queue is not None, "quarantine queue should be initialised by start()"
        rows = await queue.list(status=status)
        if json_output:
            payload = {
                "entries": [
                    {
                        "id": r["id"],
                        "topic": r["topic"],
                        "sender_pubkey_hex": bytes(r["sender_pubkey"]).hex(),
                        "payload_bytes_hex": bytes(r["payload_bytes"]).hex(),
                        "received_at_epoch_s": r["received_at_epoch_s"],
                        "rate_strikes": r["rate_strikes"],
                        "pow_status": r["pow_status"],
                        "flush_status": r["flush_status"],
                    }
                    for r in rows
                ],
                "count": len(rows),
            }
            console.print_json(json.dumps(payload))
            return
        if not rows:
            console.print(f"[dim]no quarantine entries with status={status}[/dim]")
            return
        table = Table(show_header=True, header_style="bold cyan")
        table.add_column("id")
        table.add_column("topic")
        table.add_column("sender (hex prefix)")
        table.add_column("received_at (UTC)")
        table.add_column("strikes")
        table.add_column("pow")
        table.add_column("status")
        for r in rows:
            sender_hex = bytes(r["sender_pubkey"]).hex()
            received = datetime.fromtimestamp(r["received_at_epoch_s"], tz=timezone.utc)
            table.add_row(
                str(r["id"]),
                r["topic"],
                sender_hex[:16] + "…",
                received.isoformat(timespec="seconds"),
                str(r["rate_strikes"]),
                r["pow_status"],
                r["flush_status"],
            )
        console.print(table)
    finally:
        await coord.stop()


@app.command("flush")
def flush_cmd(
    project: str = typer.Argument(..., help="Project name."),
    row_id: int = typer.Argument(..., help="Quarantine row id (cf. `quarantine list`)."),
) -> None:
    """Mark a pending entry as flushed (operator accept)."""
    asyncio.run(_do_set_status(project, row_id, "flushed"))


@app.command("drop")
def drop_cmd(
    project: str = typer.Argument(..., help="Project name."),
    row_id: int = typer.Argument(..., help="Quarantine row id (cf. `quarantine list`)."),
) -> None:
    """Mark a pending entry as dropped (operator reject)."""
    asyncio.run(_do_set_status(project, row_id, "dropped"))


async def _do_set_status(project: str, row_id: int, new_status: str) -> None:
    coord = Coordinator(project_name=project)
    await coord.start()
    try:
        queue = coord.quarantine_queue
        assert queue is not None
        if new_status == "flushed":
            updated = await queue.flush(row_id)
        elif new_status == "dropped":
            updated = await queue.drop(row_id)
        else:
            raise typer.BadParameter(f"unknown new_status {new_status!r}")
        if updated:
            console.print(f"[green]✓[/green] quarantine row {row_id} → {new_status}")
        else:
            console.print(f"[red]error[/red]: row {row_id} not found or already non-pending")
            raise typer.Exit(code=1)
    finally:
        await coord.stop()
