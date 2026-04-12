"""``nexus-coordinator migrate`` — run the DB migration runner.

Sprint 9 Phase D (D4 CLI). Provides ``--plan`` (dry-run list of
pending migrations) and ``--apply`` (execute pending migrations)
modes. When ``--app`` is omitted, iterates every discovered app
that declares ``AppManifest.migrations_dir``.
"""

from __future__ import annotations

import asyncio

import structlog
import typer
from rich.console import Console

from nexus_coordinator.paths import app_db_path, coord_config_path

console = Console()
_log = structlog.get_logger(__name__)


def migrate_cmd(
    project: str = typer.Option(..., "--project", help="Project name."),
    app_name: str | None = typer.Option(
        None,
        "--app",
        help="App name. When omitted, all apps with migrations_dir are processed.",
    ),
    plan: bool = typer.Option(False, "--plan", help="List pending migrations without applying."),
    apply: bool = typer.Option(False, "--apply", help="Apply pending migrations."),
) -> None:
    """Plan or apply database migrations for one or all apps."""
    if not plan and not apply:
        console.print("[red]error[/red]: pass either --plan or --apply.")
        raise typer.Exit(code=1)

    config_path = coord_config_path(project)
    if not config_path.exists():
        console.print(
            f"[red]error[/red]: project [bold]{project}[/bold] does not exist.\n"
            f"Run [bold]nexus-coordinator init {project}[/bold] first."
        )
        raise typer.Exit(code=1)

    asyncio.run(_run_migrate(project, app_name, plan=plan, do_apply=apply))


async def _run_migrate(
    project: str,
    app_name: str | None,
    *,
    plan: bool,
    do_apply: bool,
) -> None:
    from nexus_sdk import (
        AppDatabaseClient,
        MigrationRunner,
        discover_apps,
    )

    apps = list(discover_apps())
    if app_name is not None:
        apps = [a for a in apps if a.manifest.name == app_name]
        if not apps:
            console.print(f"[red]error[/red]: app [bold]{app_name}[/bold] not found.")
            raise typer.Exit(code=1)

    found_any = False
    for app in apps:
        if app.manifest.migrations_dir is None:
            continue
        found_any = True
        db_path = app_db_path(project, app.manifest.name)
        db_path.parent.mkdir(parents=True, exist_ok=True)
        client = AppDatabaseClient(db_path)
        runner = MigrationRunner(client, app.manifest.migrations_dir)

        if plan:
            pending = await runner.plan()
            if pending:
                console.print(f"\n[bold]{app.manifest.name}[/bold] — {len(pending)} pending:")
                for m in pending:
                    console.print(f"  {m.version:03d}_{m.slug}.sql  (sha256: {m.sha256[:12]}...)")
            else:
                console.print(f"\n[bold]{app.manifest.name}[/bold] — up to date.")

        if do_apply:
            applied = await runner.apply()
            if applied:
                console.print(f"\n[bold]{app.manifest.name}[/bold] — {len(applied)} applied:")
                for m in applied:
                    console.print(f"  {m.version:03d}_{m.slug}.sql")
            else:
                console.print(f"\n[bold]{app.manifest.name}[/bold] — nothing to apply.")

    if not found_any:
        console.print("[yellow]No apps with migrations_dir found.[/yellow]")
