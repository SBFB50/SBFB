# SPDX-License-Identifier: AGPL-3.0-or-later
"""``nexus-coordinator canary`` sub-app: rotate / status.

Sprint 22 Phase E — operator-facing CLI for the watermark canari-
input primitive. The ``rotate`` command mints a fresh signed
:class:`~nexus_coordinator.canary_input.CanaryInputSet` (seeded
with :data:`~nexus_coordinator.canary_input.DEFAULT_SEED_PROMPTS`
by default, customisable via ``--prompts``) and installs it under
the configured set path. The ``status`` command prints the current
policy + injector/observer counters + the most recent divergence
records.

Follows the Sprint 21 Phase D ``quarantine`` CLI pattern : a
transient :class:`~nexus_coordinator.coordinator.Coordinator` is
started for the duration of the command, the relevant subsystem is
accessed directly, and the coordinator is stopped on exit. A
production coordinator already running on the same project would
collide on the iroh data directory — stop it first, then run the
CLI, then restart.
"""

from __future__ import annotations

import asyncio
import json
from pathlib import Path

import typer
from rich.console import Console
from rich.table import Table

from nexus_coordinator.canary_input import (
    DEFAULT_SEED_PROMPTS,
    CanaryPrompt,
    build_canary_input_set,
)
from nexus_coordinator.coordinator import Coordinator

app = typer.Typer(
    help="Rotate or inspect the watermark canari-input spot-check set.",
    no_args_is_help=True,
)
console = Console()


@app.command("rotate")
def rotate_cmd(
    project: str = typer.Argument(..., help="Project name — must exist on disk."),
    prompts_file: Path | None = typer.Option(
        None,
        "--prompts",
        help=(
            "Optional JSON file with a list of "
            "{prompt_id, prompt, expected_answer, tolerance?} entries. "
            "Falls back to DEFAULT_SEED_PROMPTS."
        ),
    ),
    output: Path | None = typer.Option(
        None,
        "--output",
        "-o",
        help=("Destination path for the signed set JSON. Default: coordinator's canary_input effective set path."),
    ),
) -> None:
    """Generate a fresh signed CanaryInputSet and install it."""
    asyncio.run(_do_rotate(project, prompts_file, output))


async def _do_rotate(
    project: str,
    prompts_file: Path | None,
    output: Path | None,
) -> None:
    prompts = _load_prompts(prompts_file)
    coord = Coordinator(project_name=project)
    await coord.start()
    try:
        manager = getattr(coord, "canary_input", None)
        if manager is None:
            console.print("[red]error[/red]: canary_input manager not initialised on coordinator")
            raise typer.Exit(code=1)
        keypair = coord._keypair  # noqa: SLF001 — CLI runs inside the coord process
        if keypair is None:
            console.print("[red]error[/red]: coordinator keypair not loaded")
            raise typer.Exit(code=1)
        canary_set = build_canary_input_set(
            prompts,
            keypair.secret,
            keypair.public,
        )
        manager.rotate(canary_set)
        if output is not None:
            # Explicit `--output` writes a second copy alongside the
            # managed path so the operator can archive the rotation.
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_text(
                canary_set.model_dump_json(indent=2),
                encoding="utf-8",
            )
            console.print(f"[dim]archive copy written to {output}[/dim]")
        target = manager._effective_set_path()  # noqa: SLF001
        console.print(
            f"[green]✓[/green] canary-input set rotated → [cyan]{target}[/cyan]",
        )
        console.print(
            f"  version=[cyan]{canary_set.version}[/cyan]"
            f" prompts=[cyan]{len(canary_set.prompts)}[/cyan]"
            f" created_at=[cyan]{canary_set.created_at_unix}[/cyan]",
        )
    finally:
        await coord.stop()


def _load_prompts(prompts_file: Path | None) -> list[CanaryPrompt]:
    if prompts_file is None:
        return [CanaryPrompt(prompt_id=pid, prompt=p, expected_answer=a) for (pid, p, a) in DEFAULT_SEED_PROMPTS]
    raw = prompts_file.read_text(encoding="utf-8")
    entries = json.loads(raw)
    if not isinstance(entries, list):
        raise typer.BadParameter(
            f"--prompts {prompts_file} must contain a JSON list, got {type(entries).__name__}",
        )
    return [CanaryPrompt(**e) for e in entries]


@app.command("status")
def status_cmd(
    project: str = typer.Argument(..., help="Project name."),
    json_output: bool = typer.Option(
        False,
        "--json",
        help="Emit JSON instead of a Rich table (script-friendly).",
    ),
) -> None:
    """Display the current set + injector/observer counters."""
    asyncio.run(_do_status(project, json_output))


async def _do_status(project: str, json_output: bool) -> None:
    coord = Coordinator(project_name=project)
    await coord.start()
    try:
        manager = getattr(coord, "canary_input", None)
        if manager is None:
            console.print("[red]error[/red]: canary_input manager not initialised")
            raise typer.Exit(code=1)
        policy = manager.policy
        current_set = manager.current_set
        inj_stats = manager.injector.stats
        obs_stats = manager.observer.stats
        divergences = manager.observer.recent_divergences(limit=10)
        if json_output:
            payload = {
                "policy": {
                    "enabled": policy.enabled,
                    "inject_rate": policy.inject_rate,
                    "default_tolerance": policy.default_tolerance,
                    "rotation_frequency_days": policy.rotation_frequency_days,
                    "set_path": policy.set_path,
                },
                "set": {
                    "loaded": current_set is not None,
                    "version": current_set.version if current_set else None,
                    "prompt_count": len(current_set.prompts) if current_set else 0,
                    "created_at_unix": current_set.created_at_unix if current_set else None,
                },
                "injector_stats": inj_stats,
                "observer_stats": obs_stats,
                "recent_divergences": [d.to_dict() for d in divergences],
            }
            console.print_json(json.dumps(payload))
            return
        console.print(
            f"[bold]policy[/bold] enabled=[cyan]{policy.enabled}[/cyan]"
            f" inject_rate=1/[cyan]{policy.inject_rate}[/cyan]"
            f" default_tolerance=[cyan]{policy.default_tolerance}[/cyan]",
        )
        if current_set is None:
            console.print("[yellow]set[/yellow]: [dim]none loaded — run `canary rotate`[/dim]")
        else:
            console.print(
                f"[bold]set[/bold] version=[cyan]{current_set.version}[/cyan]"
                f" prompts=[cyan]{len(current_set.prompts)}[/cyan]"
                f" created_at=[cyan]{current_set.created_at_unix}[/cyan]",
            )
        console.print(
            f"[bold]injector[/bold] seen=[cyan]{inj_stats['seen']}[/cyan]"
            f" injected=[cyan]{inj_stats['injected']}[/cyan]",
        )
        console.print(
            f"[bold]observer[/bold] observed=[cyan]{obs_stats['observed']}[/cyan]"
            f" alerts=[cyan]{obs_stats['alerts']}[/cyan]"
            f" ring=[cyan]{obs_stats['ring_size']}[/cyan]",
        )
        if not divergences:
            console.print("[dim]no divergences recorded[/dim]")
            return
        table = Table(show_header=True, header_style="bold cyan")
        table.add_column("prompt_id")
        table.add_column("similarity")
        table.add_column("observed_at_unix")
        table.add_column("worker (hex prefix)")
        for r in divergences:
            worker = (r.worker_pubkey_hex or "")[:16] + "…" if r.worker_pubkey_hex else "-"
            table.add_row(
                r.prompt_id,
                f"{r.similarity:.3f}",
                str(r.observed_at_unix),
                worker,
            )
        console.print(table)
    finally:
        await coord.stop()
