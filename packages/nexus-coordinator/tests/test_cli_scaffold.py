# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 15 Phase C — tests for the ``sbfb init`` scaffold command.

Exercises the scaffold command via Typer's ``CliRunner``:

- Happy paths for html / react / pyodide templates (expected files
  created, placeholders substituted).
- Error paths (destination exists, unknown type).
- Daemon-absent fallback (SBFB.json retains ``{{NODE_ID}}``
  placeholder when no running.json exists).
- Daemon-present path (SBFB.json contains the daemon's node_id).
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from nexus_coordinator.cli.commands.scaffold import (
    NODE_ID_PLACEHOLDER,
    PROJECT_NAME_PLACEHOLDER,
    TemplateType,
)
from nexus_coordinator.cli.sbfb_main import app
from typer.testing import CliRunner


@pytest.fixture
def runner() -> CliRunner:
    return CliRunner()


@pytest.fixture
def no_daemon(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> Path:
    """Point `_daemon_running_json_path` at a non-existent file.

    Ensures tests don't accidentally read the developer's real
    daemon state.
    """
    fake = tmp_path / "nonexistent-nexus-grid" / "shell-daemon" / "running.json"
    monkeypatch.setattr(
        "nexus_coordinator.cli.commands.scaffold._daemon_running_json_path",
        lambda: fake,
    )
    return fake


@pytest.fixture
def mock_daemon_running(monkeypatch: pytest.MonkeyPatch, tmp_path: Path):
    """Return a helper that writes a fake running.json with a given node_id."""

    def _mock(node_id: str) -> Path:
        running = tmp_path / "fake-nexus-grid" / "shell-daemon" / "running.json"
        running.parent.mkdir(parents=True, exist_ok=True)
        running.write_text(
            json.dumps({"node_id": node_id, "api_port": 7000}),
            encoding="utf-8",
        )
        monkeypatch.setattr(
            "nexus_coordinator.cli.commands.scaffold._daemon_running_json_path",
            lambda: running,
        )
        return running

    return _mock


# ---------------------------------------------------------------
# Happy paths — per template type
# ---------------------------------------------------------------


class TestInitHtml:
    def test_creates_expected_files(self, runner: CliRunner, tmp_path: Path, no_daemon: Path) -> None:
        dest = tmp_path / "my-html-app"
        result = runner.invoke(app, ["init", "html", str(dest)])
        assert result.exit_code == 0, result.stdout
        assert (dest / "index.html").is_file()
        assert (dest / "SBFB.json").is_file()
        assert (dest / "README.md").is_file()
        assert (dest / ".gitignore").is_file()

    def test_substitutes_project_name(self, runner: CliRunner, tmp_path: Path, no_daemon: Path) -> None:
        dest = tmp_path / "my-cool-app"
        runner.invoke(app, ["init", "html", str(dest)])
        sbfb = json.loads((dest / "SBFB.json").read_text(encoding="utf-8"))
        assert sbfb["project_name"] == "my-cool-app"
        assert sbfb["template"] == "html"
        index_html = (dest / "index.html").read_text(encoding="utf-8")
        assert "my-cool-app" in index_html

    def test_leaves_node_id_placeholder_when_no_daemon(
        self, runner: CliRunner, tmp_path: Path, no_daemon: Path
    ) -> None:
        dest = tmp_path / "no-daemon-app"
        result = runner.invoke(app, ["init", "html", str(dest)])
        assert result.exit_code == 0
        sbfb = json.loads((dest / "SBFB.json").read_text(encoding="utf-8"))
        assert sbfb["node_id"] == NODE_ID_PLACEHOLDER
        # The warning should be visible on stdout.
        assert "shell-daemon not running" in result.stdout.lower() or "Warning" in result.stdout

    def test_substitutes_node_id_from_daemon(
        self,
        runner: CliRunner,
        tmp_path: Path,
        mock_daemon_running,
    ) -> None:
        fake_node_id = "de" * 32
        mock_daemon_running(fake_node_id)
        dest = tmp_path / "with-daemon-app"
        result = runner.invoke(app, ["init", "html", str(dest)])
        assert result.exit_code == 0
        sbfb = json.loads((dest / "SBFB.json").read_text(encoding="utf-8"))
        assert sbfb["node_id"] == fake_node_id
        # No warning, and no placeholder left behind.
        assert NODE_ID_PLACEHOLDER not in (dest / "SBFB.json").read_text(encoding="utf-8")


class TestInitReact:
    def test_creates_expected_files(self, runner: CliRunner, tmp_path: Path, no_daemon: Path) -> None:
        dest = tmp_path / "my-react-app"
        result = runner.invoke(app, ["init", "react", str(dest)])
        assert result.exit_code == 0, result.stdout
        # Minimal Vite structure.
        assert (dest / "package.json").is_file()
        assert (dest / "vite.config.ts").is_file()
        assert (dest / "tsconfig.json").is_file()
        assert (dest / "index.html").is_file()
        assert (dest / "src" / "main.tsx").is_file()
        assert (dest / "src" / "App.tsx").is_file()
        assert (dest / "SBFB.json").is_file()

    def test_substitutes_project_name_in_package_json(self, runner: CliRunner, tmp_path: Path, no_daemon: Path) -> None:
        dest = tmp_path / "my-react-app"
        runner.invoke(app, ["init", "react", str(dest)])
        pkg = json.loads((dest / "package.json").read_text(encoding="utf-8"))
        assert pkg["name"] == "my-react-app"

    def test_substitutes_project_name_in_app_tsx(self, runner: CliRunner, tmp_path: Path, no_daemon: Path) -> None:
        dest = tmp_path / "substituted-react"
        runner.invoke(app, ["init", "react", str(dest)])
        app_tsx = (dest / "src" / "App.tsx").read_text(encoding="utf-8")
        assert "substituted-react" in app_tsx
        assert PROJECT_NAME_PLACEHOLDER not in app_tsx


class TestInitPyodide:
    def test_creates_expected_files(self, runner: CliRunner, tmp_path: Path, no_daemon: Path) -> None:
        dest = tmp_path / "my-pyodide-app"
        result = runner.invoke(app, ["init", "pyodide", str(dest)])
        assert result.exit_code == 0, result.stdout
        assert (dest / "index.html").is_file()
        assert (dest / "SBFB.json").is_file()
        assert (dest / "README.md").is_file()

    def test_readme_mentions_bundle_requirement(self, runner: CliRunner, tmp_path: Path, no_daemon: Path) -> None:
        dest = tmp_path / "pyodide-bundle-info"
        runner.invoke(app, ["init", "pyodide", str(dest)])
        readme = (dest / "README.md").read_text(encoding="utf-8")
        # The CSP constraint is mentioned so users know why a bundle
        # is required instead of a CDN.
        assert "CSP" in readme or "csp" in readme.lower()
        assert "pyodide" in readme.lower()


# ---------------------------------------------------------------
# Error paths
# ---------------------------------------------------------------


class TestInitErrors:
    def test_rejects_existing_destination(self, runner: CliRunner, tmp_path: Path, no_daemon: Path) -> None:
        dest = tmp_path / "already-here"
        dest.mkdir()
        result = runner.invoke(app, ["init", "html", str(dest)])
        assert result.exit_code != 0
        # Nothing should have been written inside the existing directory.
        # Typer emits BadParameter to stderr in click >= 8.2, so we
        # assert on the filesystem invariant rather than on stdout.
        assert list(dest.iterdir()) == []

    def test_rejects_unknown_template(self, runner: CliRunner, tmp_path: Path, no_daemon: Path) -> None:
        dest = tmp_path / "bad-template"
        result = runner.invoke(app, ["init", "svelte", str(dest)])
        assert result.exit_code != 0
        # Typer emits its own "Invalid value" message for enum mismatch.
        assert not dest.exists()


# ---------------------------------------------------------------
# Placeholder integrity
# ---------------------------------------------------------------


class TestPlaceholderIntegrity:
    def test_html_sbfb_json_is_valid_json_with_daemon(
        self, runner: CliRunner, tmp_path: Path, mock_daemon_running
    ) -> None:
        mock_daemon_running("ab" * 32)
        dest = tmp_path / "valid-json"
        runner.invoke(app, ["init", "html", str(dest)])
        sbfb = json.loads((dest / "SBFB.json").read_text(encoding="utf-8"))
        assert set(sbfb.keys()) >= {"node_id", "project_name", "template"}

    def test_no_placeholders_remain_when_daemon_running(
        self, runner: CliRunner, tmp_path: Path, mock_daemon_running
    ) -> None:
        mock_daemon_running("cd" * 32)
        dest = tmp_path / "no-placeholders"
        runner.invoke(app, ["init", "html", str(dest)])
        for f in dest.rglob("*"):
            if f.is_file():
                text = f.read_text(encoding="utf-8")
                assert NODE_ID_PLACEHOLDER not in text, f"stray placeholder in {f}"
                assert PROJECT_NAME_PLACEHOLDER not in text, f"stray placeholder in {f}"

    def test_malformed_running_json_falls_back_to_placeholder(
        self,
        runner: CliRunner,
        tmp_path: Path,
        monkeypatch: pytest.MonkeyPatch,
    ) -> None:
        # Simulate a corrupt running.json — scaffold must not crash.
        running = tmp_path / "corrupt" / "running.json"
        running.parent.mkdir(parents=True)
        running.write_text("this is not json", encoding="utf-8")
        monkeypatch.setattr(
            "nexus_coordinator.cli.commands.scaffold._daemon_running_json_path",
            lambda: running,
        )
        dest = tmp_path / "fallback-app"
        result = runner.invoke(app, ["init", "html", str(dest)])
        assert result.exit_code == 0, result.stdout
        sbfb = json.loads((dest / "SBFB.json").read_text(encoding="utf-8"))
        assert sbfb["node_id"] == NODE_ID_PLACEHOLDER


# ---------------------------------------------------------------
# TemplateType enum sanity
# ---------------------------------------------------------------


class TestTemplateType:
    def test_all_three_members_exist(self) -> None:
        assert TemplateType.html.value == "html"
        assert TemplateType.react.value == "react"
        assert TemplateType.pyodide.value == "pyodide"
