# SPDX-License-Identifier: AGPL-3.0-or-later
"""Entry point for the ``sbfb`` CLI.

Sprint 15 Phase C — developer-facing command line to scaffold apps
ready for publication on the SBFB network.

Commands:

- ``sbfb init <type> <path>`` — scaffold a new app from a template
  (html | react | pyodide).

Note: this is a distinct CLI from ``nexus-coordinator``. The
coordinator CLI bootstraps a coordinator node (keypair + config).
The ``sbfb`` CLI is for app authors who want to publish content to
the network.
"""

from __future__ import annotations

import typer

from nexus_coordinator.cli.commands.scaffold import scaffold_cmd

app = typer.Typer(
    name="sbfb",
    help="SBFB developer CLI — scaffold and publish apps.",
    no_args_is_help=True,
    add_completion=False,
    rich_markup_mode="rich",
)


@app.callback()
def _root_callback() -> None:
    """SBFB developer CLI root.

    Empty callback present so Typer treats this as a multi-command
    app even when only one subcommand is registered — ``sbfb init``
    must be invoked explicitly by name, not as a positional.
    """
    # Intentionally empty — see docstring.


app.command(
    "init",
    help="Scaffold a new SBFB app from a template (html | react | pyodide).",
)(scaffold_cmd)


if __name__ == "__main__":
    app()
