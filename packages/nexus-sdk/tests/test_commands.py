"""Unit tests for the Sprint 8 Phase A ``@nexus_command`` surface.

Covers:

- :class:`nexus_sdk.commands.CommandDescriptor` — Pydantic v2
  contract validation (extra forbid, frozen, length caps,
  schema_version literal, default icon/group, JSON roundtrip).
- :func:`nexus_sdk.decorators.nexus_command` — attribute
  apposition on a coroutine method.
- :meth:`nexus_sdk.NexusApp.commands` — descriptor collection
  across an app hierarchy.
- :meth:`nexus_sdk.NexusApp.invoke_command` — runtime dispatch
  to the decorated method + unknown-name handling.
"""

from __future__ import annotations

import json

import pytest
from nexus_sdk import (
    AppManifest,
    CommandDescriptor,
    NexusApp,
    nexus_command,
    nexus_worker,
)
from pydantic import ValidationError

# ---------------------------------------------------------------------------
# CommandDescriptor Pydantic contract
# ---------------------------------------------------------------------------


def test_command_descriptor_minimal_is_valid() -> None:
    cmd = CommandDescriptor(name="detect", description="Détecter les contradictions")
    assert cmd.name == "detect"
    assert cmd.description == "Détecter les contradictions"
    assert cmd.icon == "sparkles"  # default
    assert cmd.group == "Actions"  # default
    assert cmd.schema_version == 1


def test_command_descriptor_rejects_extra_field() -> None:
    with pytest.raises(ValidationError):
        CommandDescriptor(
            name="x",
            description="y",
            evil_field="surprise",  # type: ignore[call-arg]
        )


def test_command_descriptor_is_frozen() -> None:
    # Pydantic v2 frozen=True raises on assignment after
    # construction — the shell relies on this to cache the
    # descriptor list across re-renders without defensive copies.
    cmd = CommandDescriptor(name="x", description="y")
    with pytest.raises(ValidationError):
        cmd.name = "other"  # type: ignore[misc]


def test_command_descriptor_name_length_caps() -> None:
    # 64-char max (D5 frozen). 65 must fail.
    CommandDescriptor(name="a" * 64, description="ok")
    with pytest.raises(ValidationError):
        CommandDescriptor(name="a" * 65, description="ok")
    # Empty name is refused (min_length=1).
    with pytest.raises(ValidationError):
        CommandDescriptor(name="", description="ok")


def test_command_descriptor_description_length_cap() -> None:
    CommandDescriptor(name="x", description="a" * 280)
    with pytest.raises(ValidationError):
        CommandDescriptor(name="x", description="a" * 281)


def test_command_descriptor_schema_version_literal_is_frozen_at_one() -> None:
    cmd = CommandDescriptor(name="x", description="y")
    assert cmd.schema_version == 1
    # A caller trying to ship a different literal is rejected.
    with pytest.raises(ValidationError):
        CommandDescriptor(
            schema_version=2,  # type: ignore[arg-type]
            name="x",
            description="y",
        )


def test_command_descriptor_icon_group_caps() -> None:
    CommandDescriptor(name="x", description="y", icon="a" * 32, group="b" * 32)
    with pytest.raises(ValidationError):
        CommandDescriptor(name="x", description="y", icon="a" * 33)
    with pytest.raises(ValidationError):
        CommandDescriptor(name="x", description="y", group="b" * 33)


def test_command_descriptor_json_roundtrip_preserves_fields() -> None:
    # The coordinator serializes descriptors to JSON for the
    # React shell; the Zod mirror must see the exact same
    # payload. Snapshot the payload shape so any regression
    # trips here before it reaches the shell.
    cmd = CommandDescriptor(
        name="detect",
        description="Détecter",
        icon="refresh",
        group="Gov",
    )
    raw = json.loads(cmd.model_dump_json())
    assert raw == {
        "schema_version": 1,
        "name": "detect",
        "description": "Détecter",
        "icon": "refresh",
        "group": "Gov",
    }
    back = CommandDescriptor(**raw)
    assert back == cmd


# ---------------------------------------------------------------------------
# @nexus_command decorator + NexusApp.commands() + invoke_command
# ---------------------------------------------------------------------------


class _CmdFixtureApp(NexusApp):
    manifest = AppManifest(name="cmdfix", version="0.1.0")

    last_invoked: str = ""

    @nexus_command("detect", description="Détecter les contradictions")
    async def cmd_detect(self) -> dict[str, str]:
        self.last_invoked = "detect"
        return {"navigation": {"path": "/app/cmdfix/tabs/contradictions"}}

    @nexus_command(
        "refresh",
        description="Rafraîchir la liste",
        icon="refresh",
        group="Gov",
    )
    async def cmd_refresh(self) -> None:
        self.last_invoked = "refresh"
        return None

    @nexus_worker(name="echo", model="stub-model")
    async def worker_echo(self, ctx):  # type: ignore[no-untyped-def]
        return {}

    async def on_start(self, ctx) -> None:  # type: ignore[no-untyped-def]
        pass

    async def on_stop(self) -> None:
        pass


def test_nexus_command_decorator_attaches_metadata() -> None:
    # The decorator is a no-op wrapper that sets a
    # `__nexus_command__` dict on the function — verify the
    # attribute shape matches what `collect_decorators` expects.
    fn = _CmdFixtureApp.cmd_detect
    meta = getattr(fn, "__nexus_command__")
    assert meta["name"] == "detect"
    assert meta["description"] == "Détecter les contradictions"
    assert meta["icon"] == "sparkles"
    assert meta["group"] == "Actions"


def test_nexus_app_commands_returns_descriptors_for_all_decorations() -> None:
    app = _CmdFixtureApp()
    cmds = app.commands()
    assert len(cmds) == 2
    names = {c.name for c in cmds}
    assert names == {"detect", "refresh"}
    refresh = next(c for c in cmds if c.name == "refresh")
    assert refresh.icon == "refresh"
    assert refresh.group == "Gov"


@pytest.mark.asyncio
async def test_invoke_command_runs_decorated_method() -> None:
    app = _CmdFixtureApp()
    result = await app.invoke_command("detect")
    assert result == {"navigation": {"path": "/app/cmdfix/tabs/contradictions"}}
    assert app.last_invoked == "detect"


@pytest.mark.asyncio
async def test_invoke_command_unknown_raises_lookup_error() -> None:
    app = _CmdFixtureApp()
    with pytest.raises(LookupError, match="unknown_cmd"):
        await app.invoke_command("unknown_cmd")
