"""
NEXUS Sync -- Server-side WebSocket changeset broadcaster.

Monitors the SQLite database for changes (via cr-sqlite's crsql_changes
virtual table) and broadcasts binary changesets to all connected users.

Architecture:
  Server writes to nexus.db (GOV tables) →
  cr-sqlite generates changesets →
  SyncBroadcaster polls every 100ms →
  WebSocket broadcast to all connected users →
  Users apply changesets to local SQLite (read-only)

Graceful fallback: if crsqlite extension is not available, sync is
disabled and the system operates in normal API mode.
"""

from __future__ import annotations

import asyncio
import json
import sqlite3
import time
from pathlib import Path
from typing import Any, Optional

from fastapi import WebSocket, WebSocketDisconnect
from loguru import logger

from nexus.config import settings


# Tables to sync (GOV data — public, read-only for users)
SYNC_TABLES = [
    "gov_politicians",
    "gov_positions",
    "gov_contradictions",
    "gov_mandates",
    "gov_parties",
    "gov_party_memberships",
    "gov_laws",
    "gov_press",
    "gov_social_posts",
    "gov_alerts",
    "gov_transcriptions",
    "gov_affairs",
    "gov_declarations",
    "gov_factchecks",
    "gov_external_ids",
]


class SyncBroadcaster:
    """Broadcasts database changesets to connected WebSocket clients.

    Uses cr-sqlite's crsql_changes virtual table to detect changes
    and send them as JSON-encoded changesets.
    """

    def __init__(self, db_path: Optional[str] = None) -> None:
        self._db_path = db_path or str(settings.sqlite_path)
        self._db: Optional[sqlite3.Connection] = None
        self._crsqlite_available = False
        self._running = False
        self._poll_task: Optional[asyncio.Task] = None
        self._clients: set[WebSocket] = set()
        self._db_version: int = 0
        self._poll_interval: float = 0.1  # 100ms
        self._changes_sent: int = 0

    @property
    def client_count(self) -> int:
        return len(self._clients)

    @property
    def db_version(self) -> int:
        return self._db_version

    @property
    def crsqlite_available(self) -> bool:
        return self._crsqlite_available

    @property
    def changes_sent(self) -> int:
        return self._changes_sent

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    async def start(self) -> None:
        """Initialize cr-sqlite and start polling for changes."""
        if self._running:
            return

        try:
            self._db = sqlite3.connect(self._db_path)

            # Try to load cr-sqlite extension
            try:
                self._db.enable_load_extension(True)
                self._db.load_extension("crsqlite")
                self._crsqlite_available = True

                # Enable CRDT on GOV tables
                for table in SYNC_TABLES:
                    try:
                        self._db.execute(f"SELECT crsql_as_crr('{table}')")
                    except sqlite3.OperationalError:
                        pass  # Table might not exist yet

                self._db.commit()

                # Get current version
                try:
                    cursor = self._db.execute(
                        "SELECT max(db_version) FROM crsql_changes"
                    )
                    row = cursor.fetchone()
                    self._db_version = row[0] if row and row[0] else 0
                except sqlite3.OperationalError:
                    self._db_version = 0

                logger.info(
                    "SyncBroadcaster started (cr-sqlite loaded, version: {}, tables: {})",
                    self._db_version, len(SYNC_TABLES),
                )

            except (OSError, sqlite3.OperationalError) as exc:
                logger.info("cr-sqlite not available — sync in passthrough mode ({})", exc)
                self._crsqlite_available = False

            self._running = True

            # Only poll if crsqlite is available (otherwise just serve WebSocket keepalives)
            if self._crsqlite_available:
                self._poll_task = asyncio.create_task(self._poll_loop())

        except Exception as exc:
            logger.error("SyncBroadcaster failed to start: {}", exc)

    async def stop(self) -> None:
        """Stop polling and close all connections."""
        self._running = False

        if self._poll_task and not self._poll_task.done():
            self._poll_task.cancel()
            try:
                await self._poll_task
            except asyncio.CancelledError:
                pass

        # Close all WebSocket clients
        for ws in list(self._clients):
            try:
                await ws.close()
            except Exception:
                pass
        self._clients.clear()

        if self._db:
            self._db.close()
            self._db = None

        logger.info("SyncBroadcaster stopped (sent {} changesets)", self._changes_sent)

    # ------------------------------------------------------------------
    # WebSocket handler
    # ------------------------------------------------------------------

    async def handle_client(self, ws: WebSocket) -> None:
        """Handle a new WebSocket client connection."""
        await ws.accept()
        self._clients.add(ws)

        try:
            # Send current version so client knows where it stands
            await ws.send_json({
                "type": "version",
                "version": self._db_version,
                "tables": SYNC_TABLES,
                "crsqlite": self._crsqlite_available,
            })

            # Keep connection alive (client is read-only, no incoming messages expected)
            while self._running:
                try:
                    # Wait for client messages (ping/pong or disconnect)
                    await asyncio.wait_for(ws.receive_text(), timeout=30.0)
                except asyncio.TimeoutError:
                    # Send keepalive ping
                    try:
                        await ws.send_json({"type": "ping", "version": self._db_version})
                    except Exception:
                        break

        except WebSocketDisconnect:
            pass
        except Exception:
            pass
        finally:
            self._clients.discard(ws)

    # ------------------------------------------------------------------
    # Change polling + broadcast
    # ------------------------------------------------------------------

    async def _poll_loop(self) -> None:
        """Poll cr-sqlite for changes and broadcast to clients."""
        while self._running:
            try:
                await asyncio.sleep(self._poll_interval)

                if not self._crsqlite_available or not self._db or not self._clients:
                    continue

                # Query new changes since last version
                changes = await asyncio.to_thread(self._fetch_changes)

                if changes:
                    payload = json.dumps({
                        "type": "changes",
                        "version": self._db_version,
                        "changes": changes,
                    })

                    # Broadcast to all connected clients
                    disconnected = []
                    for ws in list(self._clients):
                        try:
                            await ws.send_text(payload)
                        except Exception:
                            disconnected.append(ws)

                    for ws in disconnected:
                        self._clients.discard(ws)

                    self._changes_sent += len(changes)

            except asyncio.CancelledError:
                break
            except Exception as exc:
                logger.debug("Sync poll error: {}", exc)
                await asyncio.sleep(1.0)

    def _fetch_changes(self) -> list[list]:
        """Fetch new changesets from cr-sqlite (synchronous, runs in thread)."""
        if not self._db:
            return []

        try:
            cursor = self._db.execute(
                "SELECT [table], [pk], [cid], [val], [col_version], [db_version] "
                "FROM crsql_changes WHERE db_version > ?",
                [self._db_version],
            )
            rows = cursor.fetchall()

            if rows:
                max_version = max(r[5] for r in rows)
                self._db_version = max_version

            return [list(r) for r in rows]

        except Exception:
            return []

    # ------------------------------------------------------------------
    # Status
    # ------------------------------------------------------------------

    def get_status(self) -> dict:
        """Return broadcaster status for health checks."""
        return {
            "running": self._running,
            "crsqlite_available": self._crsqlite_available,
            "db_version": self._db_version,
            "clients_connected": len(self._clients),
            "changes_sent": self._changes_sent,
            "tables": SYNC_TABLES,
        }
