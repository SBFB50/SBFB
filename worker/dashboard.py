"""
nexus-worker — Rich TUI dashboard.

Real-time terminal dashboard showing:
- GPU info and current model
- Task processing status
- Session statistics (tasks, uptime, speed)
- Network stats and leaderboard
- Keyboard controls (Q=quit, P=pause, S=stats)
"""

from __future__ import annotations

import asyncio
import time
from typing import Any, Optional

from rich.console import Console
from rich.layout import Layout
from rich.live import Live
from rich.panel import Panel
from rich.table import Table
from rich.text import Text

from worker.engine import WorkerEngine, WorkerState
from worker.gpu_detect import format_vram


def _format_uptime(seconds: float) -> str:
    """Format seconds as Xh Ym."""
    h = int(seconds // 3600)
    m = int((seconds % 3600) // 60)
    if h > 0:
        return f"{h}h {m:02d}m"
    return f"{m}m"


def _state_color(state: WorkerState) -> str:
    """Color for worker state."""
    return {
        WorkerState.IDLE: "yellow",
        WorkerState.PULLING_MODEL: "cyan",
        WorkerState.PROCESSING: "green",
        WorkerState.PAUSED: "dim",
        WorkerState.ERROR: "red",
        WorkerState.STOPPED: "red",
    }.get(state, "white")


def build_dashboard(
    engine: WorkerEngine,
    name: str,
    gpu_model: str,
    vram_mb: int,
) -> Panel:
    """Build the Rich dashboard panel for a single frame."""

    state = engine.state
    color = _state_color(state)

    # Header
    lines: list[str] = []
    lines.append(f"[bold white]NEXUS GPU Contributor[/] — [bold cyan]{name}[/]")
    lines.append("")

    # GPU info
    lines.append(f"  GPU: [bold]{gpu_model}[/] ({format_vram(vram_mb)})")
    model_display = engine.current_model or "(waiting...)"
    lines.append(f"  Model: [bold]{model_display}[/]")
    lines.append(f"  Status: [{color}]{state.value}[/]")

    # Current task
    task = engine.current_task
    if task and state == WorkerState.PROCESSING:
        lines.append("")
        lines.append(f"  [bold]Current task:[/]")
        lines.append(f"    Type: {task.get('task_type', '?')}")
        lines.append(f"    ID: {task.get('task_id', '?')[:12]}...")
    elif state == WorkerState.PULLING_MODEL:
        lines.append("")
        lines.append(f"  [cyan]Pulling model...[/]")

    # Session stats
    lines.append("")
    uptime = _format_uptime(engine.uptime_seconds)
    speed = f"{engine.last_tokens_per_sec:.1f}" if engine.last_tokens_per_sec > 0 else "-"
    lines.append(f"  [bold]Session:[/] {engine.session_tasks} tasks | {uptime} uptime")
    lines.append(f"  [bold]Speed:[/] {speed} tokens/s")
    lines.append(f"  [bold]Total tokens:[/] {engine.total_tokens:,}")
    if engine.session_errors > 0:
        lines.append(f"  [bold]Errors:[/] [red]{engine.session_errors}[/]")

    # Network stats
    ns = engine.network_stats
    if ns:
        lines.append("")
        lines.append(f"  [bold]Network:[/]")
        lines.append(f"    Nodes online: {ns.get('nodes_online', 0)} ({ns.get('vram_total_gb', 0):.0f} GB VRAM)")
        lines.append(f"    Model actif: {ns.get('current_model', '?')}")
        tasks_today = ns.get('tasks_today', 0)
        lines.append(f"    Tasks today: {tasks_today:,}")

    # Leaderboard (top 5)
    lb = engine.leaderboard
    if lb:
        lines.append("")
        lines.append(f"  [bold]Leaderboard:[/]")
        for entry in lb[:5]:
            marker = " →" if entry.get("name") == name else "  "
            completed = entry.get("tasks_completed", 0)
            lines.append(f"   {marker} {entry.get('rank', '?')}. {entry.get('name', '?'):16s} {completed:,} tasks")

    # Controls
    lines.append("")
    lines.append(f"  [dim]\\[Q] Quit  \\[P] Pause/Resume[/]")

    content = "\n".join(lines)
    return Panel(content, title="[bold]NEXUS Worker[/]", border_style="blue")


async def run_dashboard(
    engine: WorkerEngine,
    name: str,
    gpu_model: str,
    vram_mb: int,
) -> None:
    """Run the Rich Live dashboard until the engine stops."""
    console = Console()

    with Live(
        build_dashboard(engine, name, gpu_model, vram_mb),
        console=console,
        refresh_per_second=2,
        screen=False,
    ) as live:
        while engine.state != WorkerState.STOPPED:
            live.update(build_dashboard(engine, name, gpu_model, vram_mb))
            await asyncio.sleep(0.5)
