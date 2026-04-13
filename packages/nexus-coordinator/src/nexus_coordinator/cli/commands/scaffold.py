# SPDX-License-Identifier: AGPL-3.0-or-later
"""``sbfb init <type> <path>`` — scaffold a new SBFB app from a template.

Sprint 15 Phase C. Copies one of the bundled templates
(``html`` / ``react`` / ``pyodide``) into a new directory and
substitutes two placeholders:

- ``{{NODE_ID}}`` — the hex node_id of the local SBFB daemon.
  Read from ``~/nexus-grid/shell-daemon/running.json`` if the
  daemon is running, otherwise left as-is so the user can fill
  it manually before publishing.
- ``{{PROJECT_NAME}}`` — the basename of the destination path.

The templates themselves live under
``nexus_coordinator/templates/<type>/`` and are installed as
package data.
"""

from __future__ import annotations

import importlib.resources
import json
from enum import Enum
from pathlib import Path

import typer
from rich.console import Console

console = Console()


class TemplateType(str, Enum):
    """Available app scaffolding templates."""

    html = "html"
    react = "react"
    pyodide = "pyodide"


# Marker string the user sees in SBFB.json when no daemon was
# running at init time. Matches the literal in the template files
# so callers can grep for it.
NODE_ID_PLACEHOLDER = "{{NODE_ID}}"
PROJECT_NAME_PLACEHOLDER = "{{PROJECT_NAME}}"


def _daemon_running_json_path() -> Path:
    """Path the shell daemon writes its liveness state to."""
    return Path.home() / "nexus-grid" / "shell-daemon" / "running.json"


def _read_local_node_id() -> str | None:
    """Return the local daemon's node_id hex, or ``None`` if absent.

    The function never raises — any parse / IO error falls through
    to ``None`` so ``sbfb init`` can still scaffold without a
    running daemon (the placeholder stays in the generated file).
    """
    path = _daemon_running_json_path()
    if not path.is_file():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return None
    node_id = data.get("node_id")
    return node_id if isinstance(node_id, str) and node_id else None


def _template_root() -> Path:
    """Resolve the bundled templates directory as a concrete Path.

    ``importlib.resources.files`` returns a ``Traversable``. For
    package-data directories installed via hatchling ``package-data``
    (our case), that Traversable is backed by a filesystem path so
    we can cast it to ``Path`` safely. Tests that exercise an
    editable install hit the source tree directly.
    """
    files = importlib.resources.files("nexus_coordinator") / "templates"
    return Path(str(files))


def _substitute(text: str, *, node_id: str, project_name: str) -> str:
    return text.replace(NODE_ID_PLACEHOLDER, node_id).replace(PROJECT_NAME_PLACEHOLDER, project_name)


# File suffixes we treat as text (placeholder substitution applies).
# Any other suffix is copied verbatim in case the template ever ships
# binary assets (images, wasm). Today all templates are text.
_TEXT_SUFFIXES = {
    ".html",
    ".css",
    ".js",
    ".jsx",
    ".ts",
    ".tsx",
    ".json",
    ".md",
    ".txt",
    ".gitignore",
    ".sh",
    "",  # dotfiles with no suffix (.gitignore) — Path.suffix is ""
}


def _copy_template(template_name: str, dest: Path, *, node_id: str, project_name: str) -> list[Path]:
    """Copy the named template into ``dest``, returning relative paths created."""
    src = _template_root() / template_name
    if not src.is_dir():
        raise typer.BadParameter(f"template '{template_name}' not found at {src}")

    created: list[Path] = []
    for item in src.rglob("*"):
        if not item.is_file():
            continue
        rel = item.relative_to(src)
        target = dest / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        if item.suffix in _TEXT_SUFFIXES or item.name.startswith("."):
            text = item.read_text(encoding="utf-8")
            substituted = _substitute(text, node_id=node_id, project_name=project_name)
            target.write_text(substituted, encoding="utf-8")
        else:
            target.write_bytes(item.read_bytes())
        created.append(rel)
    return created


def scaffold_cmd(
    template: TemplateType = typer.Argument(
        ...,
        help="Template type: html (minimal), react (Vite), or pyodide (Python in browser).",
    ),
    path: Path = typer.Argument(
        ...,
        help="Destination directory. Must not already exist.",
    ),
) -> None:
    """Scaffold a new SBFB app from a template.

    Example:

        sbfb init html ./my-app
        sbfb init react ./chat
        sbfb init pyodide ./compute
    """
    if path.exists():
        raise typer.BadParameter(f"destination '{path}' already exists; refusing to overwrite")

    project_name = path.name or "sbfb-app"
    node_id = _read_local_node_id() or NODE_ID_PLACEHOLDER

    path.mkdir(parents=True)
    created = _copy_template(template.value, path, node_id=node_id, project_name=project_name)

    console.print(
        f"[green]Created[/green] {template.value} app at [bold]{path}[/bold] ([cyan]{len(created)}[/cyan] files)"
    )

    if node_id == NODE_ID_PLACEHOLDER:
        console.print(
            "[yellow]Warning:[/yellow] shell-daemon not running. "
            f"Edit SBFB.json and replace {NODE_ID_PLACEHOLDER} with your "
            "node_id before publishing."
        )
    else:
        console.print(f"[dim]node_id prefilled: {node_id[:16]}...[/dim]")

    # Guidance for the user based on template.
    if template == TemplateType.react:
        console.print("[dim]Next: cd into the directory, `npm install && npm run build`, then deploy dist/.[/dim]")
    elif template == TemplateType.pyodide:
        console.print("[dim]Next: download Pyodide 0.29.3 into ./pyodide/ (see README.md).[/dim]")
    else:
        console.print("[dim]Next: customize index.html and deploy the folder as a zip.[/dim]")
