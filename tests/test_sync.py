"""
Tests for Phase 9 — Real-time sync (cr-sqlite WebSocket).

Covers:
- SyncBroadcaster initialization and status
- SyncReceiver initialization and status
- SYNC_TABLES configuration
- Config defaults
- Module imports
"""

import pytest

from nexus.sync.broadcaster import SyncBroadcaster, SYNC_TABLES
from nexus.sync.receiver import SyncReceiver
from nexus.config import Settings


# ===================================================================
# SyncBroadcaster
# ===================================================================

class TestSyncBroadcaster:
    """Test server-side sync broadcaster."""

    def test_init_default(self):
        bc = SyncBroadcaster()
        assert bc.client_count == 0
        assert bc.db_version == 0
        assert bc.changes_sent == 0

    def test_crsqlite_initially_false(self):
        bc = SyncBroadcaster()
        assert bc.crsqlite_available is False

    def test_get_status(self):
        bc = SyncBroadcaster()
        status = bc.get_status()
        assert "running" in status
        assert "crsqlite_available" in status
        assert "db_version" in status
        assert "clients_connected" in status
        assert "changes_sent" in status
        assert "tables" in status
        assert status["running"] is False

    @pytest.mark.asyncio
    async def test_stop_without_start(self):
        bc = SyncBroadcaster()
        await bc.stop()  # Should not raise


# ===================================================================
# SyncReceiver
# ===================================================================

class TestSyncReceiver:
    """Test client-side sync receiver."""

    def test_init(self):
        rx = SyncReceiver(server_url="wss://nexusgov.fr/ws/sync")
        assert rx.connected is False
        assert rx.local_version == 0
        assert rx.changes_applied == 0

    def test_get_status(self):
        rx = SyncReceiver(server_url="wss://test.com/ws/sync")
        status = rx.get_status()
        assert "running" in status
        assert "connected" in status
        assert "local_version" in status
        assert "changes_applied" in status
        assert "local_db_path" in status
        assert status["running"] is False

    @pytest.mark.asyncio
    async def test_stop_without_start(self):
        rx = SyncReceiver(server_url="wss://test.com/ws/sync")
        await rx.stop()  # Should not raise

    def test_default_local_path(self):
        rx = SyncReceiver(server_url="wss://test.com/ws/sync")
        assert "nexus_local.db" in rx._local_db_path

    def test_snapshot_url_derived(self):
        rx = SyncReceiver(server_url="wss://nexusgov.fr/ws/sync")
        assert rx._snapshot_url == "https://nexusgov.fr/api/sync/snapshot"

    def test_snapshot_url_ws_to_http(self):
        rx = SyncReceiver(server_url="ws://localhost:8000/ws/sync")
        assert rx._snapshot_url == "http://localhost:8000/api/sync/snapshot"


# ===================================================================
# SYNC_TABLES
# ===================================================================

class TestSyncTables:
    """Test sync table configuration."""

    def test_tables_not_empty(self):
        assert len(SYNC_TABLES) >= 15

    def test_core_tables_present(self):
        assert "gov_politicians" in SYNC_TABLES
        assert "gov_positions" in SYNC_TABLES
        assert "gov_contradictions" in SYNC_TABLES
        assert "gov_laws" in SYNC_TABLES
        assert "gov_press" in SYNC_TABLES

    def test_all_tables_are_strings(self):
        for table in SYNC_TABLES:
            assert isinstance(table, str)
            assert table.startswith("gov_")


# ===================================================================
# Config defaults
# ===================================================================

class TestPhase9Config:
    """Test Phase 9 config defaults."""

    def test_sync_disabled_by_default(self):
        s = Settings()
        assert s.sync_enabled is False

    def test_sync_poll_interval(self):
        s = Settings()
        assert s.sync_poll_interval == 0.1


# ===================================================================
# Module imports
# ===================================================================

class TestPhase9Imports:
    """Test Phase 9 imports."""

    def test_import_broadcaster(self):
        from nexus.sync.broadcaster import SyncBroadcaster, SYNC_TABLES
        assert SyncBroadcaster is not None

    def test_import_receiver(self):
        from nexus.sync.receiver import SyncReceiver
        assert SyncReceiver is not None

    def test_import_api(self):
        from nexus.sync.api import router
        assert router is not None

    def test_import_package(self):
        from nexus.sync import SyncBroadcaster, SyncReceiver
        assert SyncBroadcaster is not None
        assert SyncReceiver is not None
