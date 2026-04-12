# SPDX-License-Identifier: AGPL-3.0-or-later
"""Tests for ``nexus-coordinator migrate`` CLI subcommand (Sprint 9 Phase D).

The 5 scenarios listed in ``.planning/sprint9_plan.md`` §7.3 are
covered here. Each test uses a monkeypatched ``nexus_grid_tmp``
fixture for path isolation and a monkeypatched ``discover_apps``
that returns a fake app with a ``migrations_dir`` pointing at a
tmp directory.
"""

from __future__ import annotations

from pathlib import Path
from types import SimpleNamespace

import nexus_sdk
import pytest
from nexus_coordinator.cli.main import app as cli_app
from typer.testing import CliRunner

runner = CliRunner()


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _create_project(root: Path, name: str) -> Path:
    """Create a minimal project directory so the CLI's config check passes."""
    proj = root / "projects" / name
    proj.mkdir(parents=True, exist_ok=True)
    (proj / "coordinator.toml").write_text(f'[identity]\nname = "{name}"\n', encoding="utf-8")
    return proj


def _fake_app(name: str, migrations_dir: Path | None) -> SimpleNamespace:
    """Return a duck-typed object that satisfies the CLI's
    ``app.manifest.name`` / ``app.manifest.migrations_dir`` reads."""
    manifest = SimpleNamespace(name=name, migrations_dir=migrations_dir)
    return SimpleNamespace(manifest=manifest)


# ---------------------------------------------------------------------------
# 1 — test_cli_migrate_plan_lists_pending
# ---------------------------------------------------------------------------


def test_cli_migrate_plan_lists_pending(
    nexus_grid_tmp: Path,
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    _create_project(nexus_grid_tmp, "demo")
    mig_dir = tmp_path / "mig"
    mig_dir.mkdir()
    (mig_dir / "001_init.sql").write_text("CREATE TABLE demo (id INTEGER)", encoding="utf-8")
    monkeypatch.setattr(nexus_sdk, "discover_apps", lambda: [_fake_app("testapp", mig_dir)])

    result = runner.invoke(
        cli_app,
        ["migrate", "--project", "demo", "--app", "testapp", "--plan"],
    )
    assert result.exit_code == 0
    assert "001_init" in result.output
    assert "1 pending" in result.output


# ---------------------------------------------------------------------------
# 2 — test_cli_migrate_apply_happy_path
# ---------------------------------------------------------------------------


def test_cli_migrate_apply_happy_path(
    nexus_grid_tmp: Path,
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    _create_project(nexus_grid_tmp, "demo")
    mig_dir = tmp_path / "mig"
    mig_dir.mkdir()
    (mig_dir / "001_init.sql").write_text("CREATE TABLE demo (id INTEGER)", encoding="utf-8")
    monkeypatch.setattr(nexus_sdk, "discover_apps", lambda: [_fake_app("testapp", mig_dir)])

    result = runner.invoke(
        cli_app,
        ["migrate", "--project", "demo", "--app", "testapp", "--apply"],
    )
    assert result.exit_code == 0
    assert "1 applied" in result.output

    # Second run should show nothing to apply
    result2 = runner.invoke(
        cli_app,
        ["migrate", "--project", "demo", "--app", "testapp", "--apply"],
    )
    assert result2.exit_code == 0
    assert "nothing to apply" in result2.output


# ---------------------------------------------------------------------------
# 3 — test_cli_migrate_unknown_app_exits_1
# ---------------------------------------------------------------------------


def test_cli_migrate_unknown_app_exits_1(
    nexus_grid_tmp: Path,
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    _create_project(nexus_grid_tmp, "demo")
    monkeypatch.setattr(nexus_sdk, "discover_apps", lambda: [_fake_app("real", tmp_path)])

    result = runner.invoke(
        cli_app,
        ["migrate", "--project", "demo", "--app", "ghost", "--plan"],
    )
    assert result.exit_code == 1
    assert "not found" in result.output


# ---------------------------------------------------------------------------
# 4 — test_cli_migrate_all_apps_when_no_app_arg
# ---------------------------------------------------------------------------


def test_cli_migrate_all_apps_when_no_app_arg(
    nexus_grid_tmp: Path,
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    _create_project(nexus_grid_tmp, "demo")
    mig1 = tmp_path / "mig1"
    mig1.mkdir()
    (mig1 / "001_init.sql").write_text("CREATE TABLE app1 (id INTEGER)", encoding="utf-8")
    mig2 = tmp_path / "mig2"
    mig2.mkdir()
    (mig2 / "001_setup.sql").write_text("CREATE TABLE app2 (id INTEGER)", encoding="utf-8")
    monkeypatch.setattr(
        nexus_sdk,
        "discover_apps",
        lambda: [_fake_app("app1", mig1), _fake_app("app2", mig2)],
    )

    result = runner.invoke(
        cli_app,
        ["migrate", "--project", "demo", "--plan"],
    )
    assert result.exit_code == 0
    assert "app1" in result.output
    assert "app2" in result.output


# ---------------------------------------------------------------------------
# 5 — test_cli_migrate_refuses_to_run_on_unknown_project
# ---------------------------------------------------------------------------


def test_cli_migrate_refuses_to_run_on_unknown_project(
    nexus_grid_tmp: Path,
) -> None:
    result = runner.invoke(
        cli_app,
        ["migrate", "--project", "nonexistent", "--plan"],
    )
    assert result.exit_code == 1
    assert "does not exist" in result.output
