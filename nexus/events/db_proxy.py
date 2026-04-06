"""
NEXUS -- Database proxy for long-lived workers.

Workers are long-lived asyncio tasks that need DB access but cannot
hold a single connection open forever.  DatabaseProxy opens a fresh
aiosqlite connection per method call, wrapping the Database class.

This is transparent to workers: they call the same methods they
would on a regular Database instance.
"""

from __future__ import annotations

from typing import Any

from nexus.db.sqlite_db import Database, get_db


class DatabaseProxy:
    """Proxy that opens a fresh DB connection for each method call.

    Workers use this instead of a raw ``Database(conn)`` instance.
    Supports all read/write methods from ``Database``.

    Usage::

        proxy = DatabaseProxy()
        evidence = await proxy.get_evidence("ev-123")
        entities = await proxy.list_entities_by_case("case-1")
    """

    def __getattr__(self, name: str) -> Any:
        """Intercept attribute access and return an async wrapper.

        For any method M on Database, calling ``proxy.M(...)`` will:
        1. Open a fresh ``get_db()`` connection
        2. Create ``Database(conn)``
        3. Call ``Database.M(...)``
        4. Return the result (connection auto-closes via context manager)
        """
        async def _method_proxy(*args: Any, **kwargs: Any) -> Any:
            async with get_db() as conn:
                db = Database(conn)
                method = getattr(db, name)
                return await method(*args, **kwargs)

        return _method_proxy
