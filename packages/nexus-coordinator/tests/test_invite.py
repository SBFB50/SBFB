"""Invite v2 tests: mint → decode → persist → revoke roundtrip.

Exercises the full coordinator invite layer (InviteLedger) on
top of the real PyO3 bindings that call
``nexus-worker-core::invite`` for the wire format. No mocks.
"""

from __future__ import annotations

from pathlib import Path

import nexus_core
import pytest
from fastapi.testclient import TestClient
from nexus_coordinator.api.app import create_app
from nexus_coordinator.coordinator import Coordinator


@pytest.mark.asyncio
async def test_mint_invite_persists_and_decodes(nexus_grid_tmp: Path) -> None:
    coord = Coordinator(project_name="demo-invite")
    await coord.start()
    try:
        assert coord.invite_ledger is not None
        ledger = coord.invite_ledger
        record = await ledger.mint(
            project_id=coord.state.doc_id,  # type: ignore[arg-type]
            project_name="demo-invite",
            scope="worker",
            tasks_doc_ticket=coord.state.tasks_doc_ticket,
            expiry_secs=3600,
            note="unit test batch",
        )
        assert record.id.startswith("inv-")
        assert record.wire.startswith("nx1")
        assert record.scope == "worker"

        # Decode via the Rust bindings and verify every field
        # comes back unchanged.
        decoded = ledger.decode(record.wire)
        assert decoded["version"] == 2
        assert decoded["scope"] == "worker"
        assert decoded["project_name"] == "demo-invite"
        assert decoded["tasks_doc_ticket"] == coord.state.tasks_doc_ticket
        assert decoded["expires_at_unix"] == record.expires_at

        # Listed.
        rows = await ledger.list_invites()
        assert len(rows) == 1
        assert rows[0].id == record.id
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_mint_worker_without_ticket_raises(nexus_grid_tmp: Path) -> None:
    coord = Coordinator(project_name="demo-invite-2")
    await coord.start()
    try:
        ledger = coord.invite_ledger
        assert ledger is not None
        with pytest.raises(ValueError, match="tasks_doc_ticket"):
            await ledger.mint(
                project_id="proj-x",
                project_name="X",
                scope="worker",
                tasks_doc_ticket=None,
                expiry_secs=3600,
            )
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_mint_observer_without_ticket_is_allowed(nexus_grid_tmp: Path) -> None:
    coord = Coordinator(project_name="demo-invite-obs")
    await coord.start()
    try:
        ledger = coord.invite_ledger
        assert ledger is not None
        record = await ledger.mint(
            project_id="proj-x",
            project_name="X",
            scope="observer",
            tasks_doc_ticket=None,
            expiry_secs=3600,
        )
        assert record.scope == "observer"
        decoded = ledger.decode(record.wire)
        assert decoded["scope"] == "observer"
        assert decoded["tasks_doc_ticket"] is None
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_revoke_invite_updates_record(nexus_grid_tmp: Path) -> None:
    coord = Coordinator(project_name="demo-invite-rev")
    await coord.start()
    try:
        ledger = coord.invite_ledger
        assert ledger is not None
        record = await ledger.mint(
            project_id="proj-x",
            project_name="X",
            scope="worker",
            tasks_doc_ticket="fake-ticket",
            expiry_secs=3600,
        )
        assert await ledger.revoke(record.id) is True
        assert await ledger.revoke(record.id) is False  # idempotent: already revoked
        updated = await ledger.get(record.id)
        assert updated is not None
        assert updated.revoked_at is not None
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_decode_rejects_expired_invite(nexus_grid_tmp: Path) -> None:
    coord = Coordinator(project_name="demo-invite-exp")
    await coord.start()
    try:
        # Mint an invite that expires in the past by using now_unix
        # override on decode.
        wire = nexus_core.mint_invite(
            coord.keypair.secret,
            "proj-x",
            "X",
            None,
            "fake-ticket",
            "worker",
            1_000_000_000,  # year 2001
        )
        with pytest.raises(ValueError, match="expired"):
            nexus_core.decode_invite(wire, 2_000_000_000)
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_api_create_and_list_invites(nexus_grid_tmp: Path) -> None:
    coord = Coordinator(project_name="demo-invite-api")
    await coord.start()
    try:
        app = create_app(coord)
        with TestClient(app) as client:
            r = client.post(
                "/invite/create",
                json={"scope": "worker", "expiry_secs": 3600, "note": "api test"},
            )
            assert r.status_code == 200, r.text
            body = r.json()
            assert body["wire"].startswith("nx1")
            assert body["scope"] == "worker"
            invite_id = body["id"]

            r = client.get("/invite")
            assert r.status_code == 200
            assert r.json()["count"] == 1

            r = client.delete(f"/invite/{invite_id}")
            assert r.status_code == 200
            assert r.json() == {"id": invite_id, "revoked": True}

            r = client.delete(f"/invite/{invite_id}")
            assert r.status_code == 404
    finally:
        await coord.stop()
