"""
NEXUS Sync — Real-time database synchronization via cr-sqlite.

Server broadcasts changesets via WebSocket.
Clients apply changesets to local read-only SQLite copies.
Single writer (server) → many readers (users) = zero API calls for queries.
"""

from nexus.sync.broadcaster import SyncBroadcaster
from nexus.sync.receiver import SyncReceiver

__all__ = ["SyncBroadcaster", "SyncReceiver"]
