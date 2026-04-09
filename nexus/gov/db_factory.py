"""
NEXUS GOV -- Database Factory.

Returns the appropriate database backend based on config:
- SQLite (default, no extra deps)
- PostgreSQL (when GOV_DATABASE_URL is set)

Usage::

    from nexus.gov.db_factory import get_gov_database, get_gov_db_type

    # Check which backend is configured
    print(get_gov_db_type())  # "sqlite" or "postgresql"

    # Get an initialized database instance
    db = await get_gov_database()
    politicians = await db.list_politicians()
"""
from __future__ import annotations

from typing import Any, Union

from loguru import logger

from nexus.config import settings


async def get_gov_database() -> Any:
    """Get the appropriate gov database instance.

    - If ``settings.gov_database_url`` starts with ``postgres``, returns
      a :class:`PostgresGovernmentDatabase` backed by asyncpg.
    - Otherwise returns a :class:`GovernmentDatabase` backed by SQLite.

    The returned object exposes the same async CRUD interface regardless
    of the backend.
    """
    db_url = getattr(settings, "gov_database_url", "")

    if db_url and db_url.startswith("postgres"):
        from nexus.gov.db_postgres import PostgresGovernmentDatabase, init_postgres

        pool = await init_postgres(db_url)
        logger.info("GOV database: PostgreSQL ({})", db_url.split("@")[-1] if "@" in db_url else "local")
        return PostgresGovernmentDatabase(pool)

    # Default: SQLite
    from nexus.db.sqlite_db import get_db
    from nexus.gov.db import GovernmentDatabase

    conn = await get_db().__aenter__()
    logger.info("GOV database: SQLite")
    return GovernmentDatabase(conn)


def get_gov_db_type() -> str:
    """Return which backend is configured (without connecting).

    Returns ``"postgresql"`` or ``"sqlite"``.
    """
    db_url = getattr(settings, "gov_database_url", "")
    if db_url and db_url.startswith("postgres"):
        return "postgresql"
    return "sqlite"
