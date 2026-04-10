"""
NEXUS Sync -- Client-side changeset receiver.

Connects to the NEXUS server WebSocket, receives changesets,
and applies them to a local SQLite database (read-only copy).

Handles:
- Initial snapshot download (if local DB missing)
- WebSocket connection with auto-reconnect
- Changeset application via cr-sqlite
- Version tracking for resumption
"""

from __future__ import annotations

import asyncio
import json
import sqlite3
from pathlib import Path
from typing import Any, Optional

from loguru import logger


class SyncReceiver:
    """Receives and applies database changesets from the NEXUS server.

    Usage::

        receiver = SyncReceiver(
            server_url="wss://nexusgov.fr/ws/sync",
            local_db_path="~/.nexus-worker/nexus_local.db",
        )
        await receiver.start()
        # ... runs in background ...
        await receiver.stop()
    """

    def __init__(
        self,
        server_url: str,
        local_db_path: str = "",
        snapshot_url: str = "",
    ) -> None:
        self._server_url = server_url
        self._local_db_path = local_db_path or str(
            Path.home() / ".nexus-worker" / "nexus_local.db"
        )
        if not snapshot_url:
            snapshot_url = server_url.replace("/ws/sync", "/api/sync/snapshot")
            snapshot_url = snapshot_url.replace("wss://", "https://").replace("ws://", "http://")
        self._snapshot_url = snapshot_url
        self._db: Optional[sqlite3.Connection] = None
        self._crsqlite_available = False
        self._running = False
        self._connected = False
        self._local_version: int = 0
        self._changes_applied: int = 0
        self._task: Optional[asyncio.Task] = None
        self._reconnect_delay: float = 1.0
        self._max_reconnect_delay: float = 60.0

    @property
    def connected(self) -> bool:
        return self._connected

    @property
    def local_version(self) -> int:
        return self._local_version

    @property
    def changes_applied(self) -> int:
        return self._changes_applied

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    async def start(self) -> None:
        """Start the sync receiver."""
        if self._running:
            return

        self._running = True

        # Ensure local DB exists
        await self._init_local_db()

        # Start WebSocket listener
        self._task = asyncio.create_task(self._connect_loop())
        logger.info("SyncReceiver started (local: {}, version: {})", self._local_db_path, self._local_version)

    async def stop(self) -> None:
        """Stop the sync receiver."""
        self._running = False
        self._connected = False

        if self._task and not self._task.done():
            self._task.cancel()
            try:
                await self._task
            except asyncio.CancelledError:
                pass

        if self._db:
            self._db.close()
            self._db = None

        logger.info("SyncReceiver stopped ({} changes applied)", self._changes_applied)

    # ------------------------------------------------------------------
    # Local DB initialization
    # ------------------------------------------------------------------

    async def _init_local_db(self) -> None:
        """Initialize the local SQLite database."""
        db_path = Path(self._local_db_path)
        db_path.parent.mkdir(parents=True, exist_ok=True)

        if not db_path.exists():
            # Download snapshot from server
            await self._download_snapshot()

        if not db_path.exists():
            # Snapshot download failed — create empty DB (sync will catch up later)
            logger.warning("No local DB available — creating empty database")

        self._db = sqlite3.connect(str(db_path))

        # Try to load cr-sqlite
        try:
            self._db.enable_load_extension(True)
            self._db.load_extension("crsqlite")
            self._crsqlite_available = True
            logger.info("Local cr-sqlite loaded")
        except (OSError, sqlite3.OperationalError):
            self._crsqlite_available = False
            logger.info("Local cr-sqlite not available — changesets won't apply")

    async def _download_snapshot(self) -> None:
        """Download the initial database snapshot from the server."""
        try:
            import httpx

            logger.info("Downloading database snapshot from {}...", self._snapshot_url)

            async with httpx.AsyncClient(timeout=300.0) as client:
                async with client.stream("GET", self._snapshot_url) as resp:
                    if resp.status_code != 200:
                        logger.warning("Snapshot download failed: HTTP {}", resp.status_code)
                        return

                    db_path = Path(self._local_db_path)
                    total = int(resp.headers.get("content-length", 0))
                    downloaded = 0

                    with open(db_path, "wb") as f:
                        async for chunk in resp.aiter_bytes(chunk_size=65536):
                            f.write(chunk)
                            downloaded += len(chunk)

                    logger.info(
                        "Snapshot downloaded: {:.1f} MB",
                        downloaded / (1024 * 1024),
                    )

        except ImportError:
            logger.warning("httpx not installed — cannot download snapshot")
        except Exception as exc:
            logger.error("Snapshot download failed: {}", exc)

    # ------------------------------------------------------------------
    # WebSocket connection loop
    # ------------------------------------------------------------------

    async def _connect_loop(self) -> None:
        """Connect to server WebSocket with auto-reconnect."""
        while self._running:
            try:
                await self._connect_once()
            except asyncio.CancelledError:
                break
            except Exception as exc:
                logger.debug("Sync connection error: {}", exc)

            if not self._running:
                break

            # Exponential backoff on reconnect
            self._connected = False
            logger.debug("Sync reconnecting in {:.0f}s...", self._reconnect_delay)
            await asyncio.sleep(self._reconnect_delay)
            self._reconnect_delay = min(
                self._reconnect_delay * 2,
                self._max_reconnect_delay,
            )

    async def _connect_once(self) -> None:
        """Single WebSocket connection attempt."""
        try:
            import websockets
        except ImportError:
            logger.warning("websockets not installed — sync disabled")
            self._running = False
            return

        async with websockets.connect(self._server_url) as ws:
            self._connected = True
            self._reconnect_delay = 1.0  # Reset backoff on success
            logger.info("Sync connected to {}", self._server_url)

            async for message in ws:
                if not self._running:
                    break

                try:
                    data = json.loads(message)
                    msg_type = data.get("type", "")

                    if msg_type == "version":
                        server_version = data.get("version", 0)
                        logger.debug("Server version: {}", server_version)

                    elif msg_type == "changes":
                        changes = data.get("changes", [])
                        if changes:
                            await self._apply_changes(changes)

                    elif msg_type == "ping":
                        pass  # Keepalive

                except json.JSONDecodeError:
                    pass

    # ------------------------------------------------------------------
    # Changeset application
    # ------------------------------------------------------------------

    async def _apply_changes(self, changes: list[list]) -> None:
        """Apply changesets to the local database."""
        if not self._db or not self._crsqlite_available:
            return

        try:
            applied = await asyncio.to_thread(self._apply_changes_sync, changes)
            self._changes_applied += applied
        except Exception as exc:
            logger.debug("Failed to apply {} changes: {}", len(changes), exc)

    def _apply_changes_sync(self, changes: list[list]) -> int:
        """Apply changesets synchronously (runs in thread)."""
        if not self._db:
            return 0

        count = 0
        for change in changes:
            try:
                self._db.execute(
                    "INSERT INTO crsql_changes ([table], [pk], [cid], [val], [col_version], [db_version]) "
                    "VALUES (?, ?, ?, ?, ?, ?)",
                    change,
                )
                count += 1
            except Exception:
                pass

        if count > 0:
            self._db.commit()
            # Update local version
            max_version = max((c[5] for c in changes if len(c) > 5), default=0)
            if max_version > self._local_version:
                self._local_version = max_version

        return count

    # ------------------------------------------------------------------
    # Status
    # ------------------------------------------------------------------

    def get_status(self) -> dict:
        """Return receiver status."""
        return {
            "running": self._running,
            "connected": self._connected,
            "crsqlite_available": self._crsqlite_available,
            "local_version": self._local_version,
            "changes_applied": self._changes_applied,
            "local_db_path": self._local_db_path,
        }
