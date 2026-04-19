# SPDX-License-Identifier: AGPL-3.0-or-later
"""Quarantine CLI smoke tests — Sprint 21 Phase D.

The CLI sub-app is registered on the top-level Typer app via
``app.add_typer(quarantine_cmds.app, name="quarantine")`` in
``cli/main.py``. These tests hit the registered command tree
through ``CliRunner`` to assert the wiring is valid (so a typo in
``add_typer`` cannot ship undetected).

Functional flow tests (Coordinator transient + queue access) live
in ``test_quarantine_queue.py`` to keep the CLI suite fast and
focused on the surface area.
"""

from __future__ import annotations

from nexus_coordinator.cli.main import app as cli_app
from typer.testing import CliRunner


def test_cli_quarantine_help_exposes_subcommands() -> None:
    """``nexus-coordinator quarantine --help`` lists list/flush/drop.

    A successful exit + presence of the three sub-command names in
    the help text confirms ``add_typer(name="quarantine")`` landed
    cleanly and every command registered without import error.
    """
    runner = CliRunner()
    result = runner.invoke(cli_app, ["quarantine", "--help"])
    assert result.exit_code == 0, result.output
    assert "list" in result.output
    assert "flush" in result.output
    assert "drop" in result.output
