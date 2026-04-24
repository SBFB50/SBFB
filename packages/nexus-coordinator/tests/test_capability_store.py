# SPDX-License-Identifier: AGPL-3.0-or-later
"""CapabilitiesStore + @require_capability + admin_check tests — Sprint 25 Phase D.

Tests are fully hermetic: each test gets a fresh ``tmp_path``.
Admin checks are mocked to avoid requiring root/elevated during CI.
"""

from __future__ import annotations

import json
import sys
import tomllib
from pathlib import Path
from unittest.mock import patch

import pytest
from fastapi import FastAPI
from fastapi.testclient import TestClient
from nexus_coordinator.capability_store import (
    KNOWN_CAPABILITIES,
    CapabilitiesStore,
    _compute_integrity_hash,
    require_capability,
)

# -- helpers --


def _load_store(tmp_path: Path) -> CapabilitiesStore:
    return CapabilitiesStore.load(tmp_path / "capabilities.toml")


def _tamper(path: Path, replacement: str) -> None:
    """Overwrite the file with *replacement* (no valid hash)."""
    path.write_text(replacement, encoding="utf-8")


# ---------------------------------------------------------------------------
# D.1 CapabilitiesStore — load / verify / tamper
# ---------------------------------------------------------------------------


class TestStoreLoad:
    def test_load_creates_default_when_missing(self, tmp_path: Path) -> None:
        path = tmp_path / "capabilities.toml"
        assert not path.exists()
        store = CapabilitiesStore.load(path)
        assert path.exists()
        for name in KNOWN_CAPABILITIES:
            assert store.is_enabled(name) is False

    def test_load_valid_toml_round_trips(self, tmp_path: Path) -> None:
        path = tmp_path / "capabilities.toml"
        store = CapabilitiesStore.load(path)
        store2 = CapabilitiesStore.load(path)
        assert store2.audit_trail() == store.audit_trail()

    def test_load_tampered_fallback_all_off(self, tmp_path: Path) -> None:
        path = tmp_path / "capabilities.toml"
        CapabilitiesStore.load(path)
        _tamper(path, 'version = 1\nintegrity_hash = "sha256-bad"\n')
        store = CapabilitiesStore.load(path)
        for name in KNOWN_CAPABILITIES:
            assert store.is_enabled(name) is False

    def test_load_corrupt_toml_fallback_all_off(self, tmp_path: Path) -> None:
        path = tmp_path / "capabilities.toml"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("{{{{not valid toml", encoding="utf-8")
        store = CapabilitiesStore.load(path)
        for name in KNOWN_CAPABILITIES:
            assert store.is_enabled(name) is False

    def test_load_missing_capability_section_defaults_off(self, tmp_path: Path) -> None:
        path = tmp_path / "capabilities.toml"
        store = CapabilitiesStore.load(path)
        assert store.is_enabled("nonexistent_cap") is False


class TestStoreIntegrity:
    def test_integrity_hash_present_in_toml(self, tmp_path: Path) -> None:
        path = tmp_path / "capabilities.toml"
        CapabilitiesStore.load(path)
        data = tomllib.loads(path.read_text(encoding="utf-8"))
        assert "integrity_hash" in data
        assert data["integrity_hash"].startswith("sha256-")

    def test_integrity_hash_verifies_correctly(self, tmp_path: Path) -> None:
        path = tmp_path / "capabilities.toml"
        CapabilitiesStore.load(path)
        text = path.read_text(encoding="utf-8")
        data = tomllib.loads(text)
        stored = data["integrity_hash"]
        computed = _compute_integrity_hash(text)
        assert stored == computed

    def test_manual_edit_detected_as_tamper(self, tmp_path: Path) -> None:
        path = tmp_path / "capabilities.toml"
        CapabilitiesStore.load(path)
        text = path.read_text(encoding="utf-8")
        tampered = text.replace("enabled = false", "enabled = true", 1)
        path.write_text(tampered, encoding="utf-8")
        store = CapabilitiesStore.load(path)
        for name in KNOWN_CAPABILITIES:
            assert store.is_enabled(name) is False


# ---------------------------------------------------------------------------
# D.1 CapabilitiesStore — enable / disable
# ---------------------------------------------------------------------------


class TestStoreEnableDisable:
    def test_enable_updates_hash(self, tmp_path: Path) -> None:
        path = tmp_path / "capabilities.toml"
        store = CapabilitiesStore.load(path)
        hash_before = tomllib.loads(path.read_text(encoding="utf-8"))["integrity_hash"]
        store.enable("tool_calling", "testuser")
        hash_after = tomllib.loads(path.read_text(encoding="utf-8"))["integrity_hash"]
        assert hash_before != hash_after
        assert store.is_enabled("tool_calling") is True

    def test_enable_persists_actor_and_timestamp(self, tmp_path: Path) -> None:
        path = tmp_path / "capabilities.toml"
        store = CapabilitiesStore.load(path)
        store.enable("mcp_server_expose", "alice")
        entry = store.get("mcp_server_expose")
        assert entry is not None
        assert entry.enabled is True
        assert entry.enabled_by == "alice"
        assert entry.enabled_at != ""

    def test_disable_clears_fields(self, tmp_path: Path) -> None:
        path = tmp_path / "capabilities.toml"
        store = CapabilitiesStore.load(path)
        store.enable("rag_retrieval", "bob")
        assert store.is_enabled("rag_retrieval") is True
        store.disable("rag_retrieval")
        assert store.is_enabled("rag_retrieval") is False
        entry = store.get("rag_retrieval")
        assert entry is not None
        assert entry.enabled_at == ""
        assert entry.enabled_by == ""

    def test_double_enable_idempotent(self, tmp_path: Path) -> None:
        path = tmp_path / "capabilities.toml"
        store = CapabilitiesStore.load(path)
        store.enable("tool_calling", "user1")
        store.enable("tool_calling", "user2")
        entry = store.get("tool_calling")
        assert entry is not None
        assert entry.enabled_by == "user2"

    def test_enable_unknown_capability_raises(self, tmp_path: Path) -> None:
        store = CapabilitiesStore.load(tmp_path / "capabilities.toml")
        with pytest.raises(ValueError, match="unknown capability"):
            store.enable("not_a_real_cap", "x")

    def test_disable_unknown_capability_raises(self, tmp_path: Path) -> None:
        store = CapabilitiesStore.load(tmp_path / "capabilities.toml")
        with pytest.raises(ValueError, match="unknown capability"):
            store.disable("not_a_real_cap")

    def test_enable_then_reload_persists(self, tmp_path: Path) -> None:
        path = tmp_path / "capabilities.toml"
        store = CapabilitiesStore.load(path)
        store.enable("streaming_bridge", "op")
        store2 = CapabilitiesStore.load(path)
        assert store2.is_enabled("streaming_bridge") is True


# ---------------------------------------------------------------------------
# D.1 CapabilitiesStore — audit_trail
# ---------------------------------------------------------------------------


class TestAuditTrail:
    def test_audit_trail_returns_all_capabilities(self, tmp_path: Path) -> None:
        store = CapabilitiesStore.load(tmp_path / "capabilities.toml")
        trail = store.audit_trail()
        names = {e["capability"] for e in trail}
        assert names == KNOWN_CAPABILITIES

    def test_audit_trail_reflects_enable(self, tmp_path: Path) -> None:
        store = CapabilitiesStore.load(tmp_path / "capabilities.toml")
        store.enable("tool_calling", "admin")
        trail = store.audit_trail()
        tc = next(e for e in trail if e["capability"] == "tool_calling")
        assert tc["enabled"] is True
        assert tc["enabled_by"] == "admin"

    def test_audit_trail_sorted_alphabetically(self, tmp_path: Path) -> None:
        store = CapabilitiesStore.load(tmp_path / "capabilities.toml")
        trail = store.audit_trail()
        names = [e["capability"] for e in trail]
        assert names == sorted(names)


# ---------------------------------------------------------------------------
# D.1 edge cases
# ---------------------------------------------------------------------------


class TestStoreEdgeCases:
    def test_empty_file_fallback_all_off(self, tmp_path: Path) -> None:
        path = tmp_path / "capabilities.toml"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("", encoding="utf-8")
        store = CapabilitiesStore.load(path)
        for name in KNOWN_CAPABILITIES:
            assert store.is_enabled(name) is False

    def test_version_field_preserved(self, tmp_path: Path) -> None:
        path = tmp_path / "capabilities.toml"
        CapabilitiesStore.load(path)
        data = tomllib.loads(path.read_text(encoding="utf-8"))
        assert data["version"] == 1

    def test_all_known_capabilities_present_in_default(self, tmp_path: Path) -> None:
        path = tmp_path / "capabilities.toml"
        CapabilitiesStore.load(path)
        data = tomllib.loads(path.read_text(encoding="utf-8"))
        for name in KNOWN_CAPABILITIES:
            assert name in data["capability"]


# ---------------------------------------------------------------------------
# D.3 Admin privilege check (mocked)
# ---------------------------------------------------------------------------


class TestAdminCheck:
    @pytest.mark.skipif(sys.platform == "win32", reason="Unix-only test")
    def test_unix_root_passes(self) -> None:
        from nexus_coordinator.admin_check import _require_admin_unix

        with patch("nexus_coordinator.admin_check.os.geteuid", return_value=0):
            _require_admin_unix()

    @pytest.mark.skipif(sys.platform == "win32", reason="Unix-only test")
    def test_unix_non_root_raises(self) -> None:
        from nexus_coordinator.admin_check import _require_admin_unix

        with patch("nexus_coordinator.admin_check.os.geteuid", return_value=1000):
            with pytest.raises(PermissionError, match="root privilege"):
                _require_admin_unix()

    @pytest.mark.skipif(sys.platform != "win32", reason="Windows-only test")
    def test_windows_non_admin_raises(self) -> None:
        from nexus_coordinator.admin_check import _require_admin_windows

        with patch("ctypes.windll.shell32.IsUserAnAdmin", return_value=0):
            with pytest.raises(PermissionError, match="elevated"):
                _require_admin_windows()


# ---------------------------------------------------------------------------
# D.4 @require_capability decorator
# ---------------------------------------------------------------------------


class TestRequireCapabilityDecorator:
    def _make_app(self) -> FastAPI:

        fastapi_app = FastAPI()

        @fastapi_app.get("/tool/test")
        @require_capability("tool_calling")
        async def tool_endpoint():
            return {"ok": True}

        return fastapi_app

    def test_decorator_disabled_returns_403(self, tmp_path: Path) -> None:
        import nexus_coordinator.capability_store as cs

        store = CapabilitiesStore.load(tmp_path / "capabilities.toml")
        original = cs._store
        cs._store = store
        try:
            app = self._make_app()
            client = TestClient(app, raise_server_exceptions=False)
            resp = client.get("/tool/test")
            assert resp.status_code == 403
            assert "disabled" in resp.json()["detail"]
        finally:
            cs._store = original

    def test_decorator_enabled_returns_200(self, tmp_path: Path) -> None:
        import nexus_coordinator.capability_store as cs

        store = CapabilitiesStore.load(tmp_path / "capabilities.toml")
        store.enable("tool_calling", "test")
        original = cs._store
        cs._store = store
        try:
            app = self._make_app()
            client = TestClient(app, raise_server_exceptions=False)
            resp = client.get("/tool/test")
            assert resp.status_code == 200
            assert resp.json() == {"ok": True}
        finally:
            cs._store = original

    def test_decorator_no_store_returns_403(self) -> None:
        import nexus_coordinator.capability_store as cs

        original = cs._store
        cs._store = None
        try:
            app = self._make_app()
            client = TestClient(app, raise_server_exceptions=False)
            resp = client.get("/tool/test")
            assert resp.status_code == 403
        finally:
            cs._store = original


# ---------------------------------------------------------------------------
# D.2 CLI commands (via Typer CliRunner)
# ---------------------------------------------------------------------------


class TestCLI:
    def test_cli_list_shows_all_capabilities(self, tmp_path: Path) -> None:
        from nexus_coordinator.cli.commands.capability import app as cli_app
        from typer.testing import CliRunner

        runner = CliRunner()
        with patch(
            "nexus_coordinator.cli.commands.capability.CapabilitiesStore.load",
            return_value=CapabilitiesStore.load(tmp_path / "capabilities.toml"),
        ):
            result = runner.invoke(cli_app, ["list", "--json"])
        assert result.exit_code == 0
        data = json.loads(result.stdout)
        names = {e["capability"] for e in data["capabilities"]}
        assert names == KNOWN_CAPABILITIES

    def test_cli_enable_requires_admin(self, tmp_path: Path) -> None:
        from nexus_coordinator.cli.commands.capability import app as cli_app
        from typer.testing import CliRunner

        runner = CliRunner()
        with patch(
            "nexus_coordinator.cli.commands.capability.require_admin",
            side_effect=PermissionError("not admin"),
        ):
            result = runner.invoke(cli_app, ["enable", "tool_calling"])
        assert result.exit_code != 0

    def test_cli_enable_non_admin_rejected(self, tmp_path: Path) -> None:
        from nexus_coordinator.cli.commands.capability import app as cli_app
        from typer.testing import CliRunner

        runner = CliRunner()
        with patch(
            "nexus_coordinator.cli.commands.capability.require_admin",
            side_effect=PermissionError("requires root"),
        ):
            result = runner.invoke(cli_app, ["enable", "tool_calling"])
        assert result.exit_code != 0

    def test_cli_enable_then_list_shows_on(self, tmp_path: Path) -> None:
        from nexus_coordinator.cli.commands.capability import app as cli_app
        from typer.testing import CliRunner

        path = tmp_path / "capabilities.toml"
        store = CapabilitiesStore.load(path)
        runner = CliRunner()
        with (
            patch(
                "nexus_coordinator.cli.commands.capability.require_admin",
            ),
            patch(
                "nexus_coordinator.cli.commands.capability.CapabilitiesStore.load",
                return_value=store,
            ),
        ):
            result = runner.invoke(cli_app, ["enable", "tool_calling"])
            assert result.exit_code == 0
            result2 = runner.invoke(cli_app, ["list", "--json"])
        assert result2.exit_code == 0
        data = json.loads(result2.stdout)
        tc = next(e for e in data["capabilities"] if e["capability"] == "tool_calling")
        assert tc["enabled"] is True

    def test_cli_info_shows_description(self, tmp_path: Path) -> None:
        from nexus_coordinator.cli.commands.capability import app as cli_app
        from typer.testing import CliRunner

        runner = CliRunner()
        with patch(
            "nexus_coordinator.cli.commands.capability.CapabilitiesStore.load",
            return_value=CapabilitiesStore.load(tmp_path / "capabilities.toml"),
        ):
            result = runner.invoke(cli_app, ["info", "tool_calling"])
        assert result.exit_code == 0
        assert "tool_calling" in result.stdout

    def test_cli_unknown_capability_rejected(self) -> None:
        from nexus_coordinator.cli.commands.capability import app as cli_app
        from typer.testing import CliRunner

        runner = CliRunner()
        result = runner.invoke(cli_app, ["enable", "fake_cap"])
        assert result.exit_code != 0


# ---------------------------------------------------------------------------
# D.5 Semgrep rule (structural validation)
# ---------------------------------------------------------------------------


class TestSemgrepRule:
    def test_semgrep_rule_is_valid_yaml(self) -> None:
        import yaml

        rule_path = Path(__file__).resolve().parents[3] / ".semgrep" / "capability_gate.yml"
        if not rule_path.exists():
            rule_path = Path("C:/Users/FlowUP/Documents/Code/nexus/.semgrep/capability_gate.yml")
        data = yaml.safe_load(rule_path.read_text(encoding="utf-8"))
        assert "rules" in data
        rule = data["rules"][0]
        assert rule["id"] == "require-capability-check"
        assert rule["severity"] == "ERROR"
        assert "python" in rule["languages"]

    def test_semgrep_rule_targets_risk_paths(self) -> None:

        rule_path = Path(__file__).resolve().parents[3] / ".semgrep" / "capability_gate.yml"
        if not rule_path.exists():
            rule_path = Path("C:/Users/FlowUP/Documents/Code/nexus/.semgrep/capability_gate.yml")
        text = rule_path.read_text(encoding="utf-8")
        assert "/tool/" in text
        assert "/rag/" in text
        assert "/mcp/" in text


# ---------------------------------------------------------------------------
# P2-HASH-1 — tomli_w determinism round-trip (Sprint 26 Phase A)
# ---------------------------------------------------------------------------


class TestTomlDeterminism:
    def test_toml_roundtrip_determinism(self, tmp_path: Path) -> None:
        import tomli_w

        path = tmp_path / "capabilities.toml"
        store = CapabilitiesStore.load(path)
        store.enable("tool_calling", "user1")
        store2 = CapabilitiesStore.load(path)
        store2.enable("tool_calling", store.get("tool_calling").enabled_by)
        data = tomllib.loads(path.read_text(encoding="utf-8"))
        rewrite = tomli_w.dumps(data)
        data2 = tomllib.loads(rewrite)
        rewrite2 = tomli_w.dumps(data2)
        assert rewrite == rewrite2


# ---------------------------------------------------------------------------
# P2-ADMIN-1 — NULL SID guard (Sprint 26 Phase A)
# ---------------------------------------------------------------------------


class TestAdminCheckNullSid:
    def test_check_mil_high_has_null_guards(self) -> None:
        import inspect

        from nexus_coordinator.admin_check import _check_mil_high

        source = inspect.getsource(_check_mil_high)
        assert "NULL SidSubAuthorityCount" in source
        assert "NULL SidSubAuthority" in source


# ---------------------------------------------------------------------------
# P2-CAPS-1 — directory permissions (Sprint 26 Phase A)
# ---------------------------------------------------------------------------


class TestCapabilityStoreDirPermissions:
    @pytest.mark.skipif(sys.platform == "win32", reason="Unix-only chmod test")
    def test_sbfb_dir_permissions_0o700(self, tmp_path: Path) -> None:
        import os

        sbfb_dir = tmp_path / ".sbfb"
        path = sbfb_dir / "capabilities.toml"
        CapabilitiesStore.load(path)
        mode = os.stat(sbfb_dir).st_mode & 0o777
        assert mode == 0o700
