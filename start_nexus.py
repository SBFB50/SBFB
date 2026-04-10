#!/usr/bin/env python3
"""NEXUS GOV — Production Launcher with Rich Terminal UI.

Starts Docker, SearXNG, checks Ollama model, Backend (live logs), Frontend.
"""

import asyncio
import json
import os
import socket
import subprocess
import sys
import time
import webbrowser
from pathlib import Path
from urllib.request import urlopen, Request
from urllib.error import URLError

from rich.console import Console
from rich.panel import Panel

# ── Constants ──────────────────────────────────────────────────

ROOT = Path(__file__).parent
WEB_DIR = ROOT / "web"
OLLAMA_URL = os.environ.get("OLLAMA_HOST", "http://localhost:11434")
OLLAMA_MODEL = "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m"
EMBED_MODEL = "nomic-embed-text"
SEARXNG_PORT = 8888

LOGO = r"""[bold cyan]
    _   _________  ____  _______
   / | / / ____/ |/ / / / / ___/
  /  |/ / __/  |   / / / /\__ \
 / /|  / /___ /   / /_/ /___/ /
/_/ |_/_____//_/|_\____//____/[/bold cyan]
[dim white] Cold Case Investigation + Political Intelligence[/dim white]
[dim white] 31 workers | 19 tabs | Distributed GPU | 100% local[/dim white]"""

console = Console()


# ── Helpers ────────────────────────────────────────────────────

def check_port(port: int) -> bool:
    try:
        s = socket.socket()
        s.settimeout(2)
        s.connect(("127.0.0.1", port))
        s.close()
        return True
    except Exception:
        return False


def ollama_has_model(model: str) -> bool:
    """Check if Ollama has the model loaded."""
    try:
        req = Request(f"{OLLAMA_URL}/api/tags", method="GET")
        resp = urlopen(req, timeout=5)
        data = json.loads(resp.read())
        names = [m.get("name", "") for m in data.get("models", [])]
        return any(model in n or n.startswith(model.split(":")[0]) for n in names)
    except Exception:
        return False


def ollama_pull(model: str) -> bool:
    """Pull an Ollama model (blocking, shows progress)."""
    console.print(f"      Downloading [bold]{model}[/bold]... (this may take a while)")
    try:
        proc = subprocess.run(
            ["ollama", "pull", model],
            timeout=1800,  # 30 min max
        )
        return proc.returncode == 0
    except Exception as exc:
        console.print(f"      [red]Pull failed: {exc}[/red]")
        return False


# ── Service launchers ──────────────────────────────────────────

async def start_docker():
    """Start Docker Compose services."""
    console.print("[yellow][1/5][/yellow] Docker services...")
    try:
        proc = await asyncio.create_subprocess_exec(
            "docker", "compose", "up", "-d",
            cwd=str(ROOT),
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        await proc.communicate()
        if proc.returncode == 0:
            console.print("      [bold green]OK[/bold green] Neo4j + ChromaDB + Robin")
        else:
            console.print("      [yellow]WARNING[/yellow] Docker failed (continuing)")
    except FileNotFoundError:
        console.print("      [yellow]SKIP[/yellow] Docker not found")


async def check_searxng():
    """Check SearXNG is running, start container if not."""
    console.print("[yellow][2/5][/yellow] SearXNG (port 8888)...")
    if check_port(SEARXNG_PORT):
        console.print("      [bold green]OK[/bold green] SearXNG running")
        return

    # Try to start the container
    try:
        proc = await asyncio.create_subprocess_exec(
            "docker", "start", "nexus-searxng",
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        await proc.communicate()
        # Wait a bit for it to come up
        for _ in range(10):
            if check_port(SEARXNG_PORT):
                console.print("      [bold green]OK[/bold green] SearXNG started")
                return
            await asyncio.sleep(1)
    except Exception:
        pass

    console.print("      [yellow]SKIP[/yellow] SearXNG not available (social media scraping limited)")


async def check_ollama():
    """Check Ollama is running and models are available."""
    console.print("[yellow][3/5][/yellow] Ollama models...")

    # Check Ollama is running
    if not check_port(11434):
        console.print("      [bold red]ERROR[/bold red] Ollama not running! Start it: [bold]ollama serve[/bold]")
        return False

    # Check main model
    if ollama_has_model(OLLAMA_MODEL):
        console.print(f"      [bold green]OK[/bold green] {OLLAMA_MODEL}")
    else:
        console.print(f"      [yellow]MISSING[/yellow] {OLLAMA_MODEL}")
        ollama_pull(OLLAMA_MODEL)
        if ollama_has_model(OLLAMA_MODEL):
            console.print(f"      [bold green]OK[/bold green] {OLLAMA_MODEL} pulled")
        else:
            console.print(f"      [bold red]FAILED[/bold red] Could not pull {OLLAMA_MODEL}")
            return False

    # Check embedding model
    if ollama_has_model(EMBED_MODEL):
        console.print(f"      [bold green]OK[/bold green] {EMBED_MODEL}")
    else:
        console.print(f"      [yellow]MISSING[/yellow] {EMBED_MODEL}")
        ollama_pull(EMBED_MODEL)

    return True


async def start_frontend():
    """Start Vite frontend in background."""
    console.print("[yellow][4/5][/yellow] Frontend (Vite :3002)...")
    npx = "npx.cmd" if sys.platform == "win32" else "npx"
    try:
        await asyncio.create_subprocess_exec(
            npx, "vite", "--host", "0.0.0.0", "--port", "3002",
            cwd=str(WEB_DIR),
            stdout=asyncio.subprocess.DEVNULL,
            stderr=asyncio.subprocess.DEVNULL,
            creationflags=subprocess.CREATE_NEW_PROCESS_GROUP if sys.platform == "win32" else 0,
        )
        for _ in range(20):
            if check_port(3002):
                console.print("      [bold green]OK[/bold green] http://localhost:3002")
                return True
            await asyncio.sleep(1)
        console.print("      [bold red]FAILED[/bold red] Frontend timeout")
        return False
    except Exception as exc:
        console.print(f"      [bold red]FAILED[/bold red] {exc}")
        return False


async def open_browser_when_ready():
    """Wait for backend to be ready, then open browser."""
    for _ in range(180):
        if check_port(8000):
            await asyncio.sleep(2)
            webbrowser.open("http://localhost:3002")
            console.print()
            console.print(Panel(
                "[bold green]All systems operational![/bold green]\n\n"
                "  Frontend:  [link=http://localhost:3002]http://localhost:3002[/link]\n"
                "  Backend:   [link=http://localhost:8000]http://localhost:8000[/link]\n"
                "  Swagger:   [link=http://localhost:8000/docs]http://localhost:8000/docs[/link]\n"
                "  Neo4j:     [link=http://localhost:7474]http://localhost:7474[/link]\n"
                "  Network:   [link=http://localhost:3002/network]http://localhost:3002/network[/link]\n\n"
                "  [dim]GPU Compute:  /api/compute/stats[/dim]\n"
                "  [dim]Leaderboard: /api/compute/leaderboard[/dim]\n"
                "  [dim]Contribute:  pip install nexus-worker[/dim]",
                title="[bold white]NEXUS Ready[/bold white]",
                border_style="green",
                padding=(1, 2),
            ))
            return
        await asyncio.sleep(1)
    console.print("[bold red]Backend did not start within 3 minutes.[/bold red]")


# ── Main ───────────────────────────────────────────────────────

async def main():
    console.clear()
    console.print(Panel(LOGO, border_style="bright_cyan", padding=(1, 4)))
    console.print()

    # 1. Docker
    await start_docker()

    # 2. SearXNG
    await check_searxng()

    # 3. Ollama models
    ollama_ok = await check_ollama()
    if not ollama_ok:
        console.print()
        console.print("[bold red]Cannot start without Ollama + LLM model.[/bold red]")
        console.print("[dim]Start Ollama and pull the model, then retry.[/dim]")
        return

    # 4. Frontend
    console.print()
    await start_frontend()

    # 5. Backend (foreground with live logs)
    console.print()
    console.print("[yellow][5/5][/yellow] Backend (FastAPI :8000)")
    console.print("      Logs en direct. [dim]Ctrl+C pour arreter.[/dim]")
    console.print()
    console.rule("[bold cyan]Backend Logs[/bold cyan]")
    console.print()

    # Open browser in background when backend is ready
    browser_task = asyncio.create_task(open_browser_when_ready())

    # Run uvicorn in foreground — logs visible
    proc = await asyncio.create_subprocess_exec(
        sys.executable, "-m", "uvicorn", "nexus.main:app",
        "--host", "0.0.0.0", "--port", "8000",
        "--log-level", "info",
        cwd=str(ROOT),
    )

    try:
        await proc.wait()
    except (KeyboardInterrupt, asyncio.CancelledError):
        proc.terminate()
        console.print("\n[yellow]Backend stopped.[/yellow]")
    finally:
        browser_task.cancel()


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        console.print("\n[yellow]NEXUS shutdown.[/yellow]")
