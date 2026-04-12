# SPDX-License-Identifier: AGPL-3.0-or-later
"""SQLite-backed persistent state.

Two tables, both scoped to a single project (one SQLite file per
project under ``~/.nexus-grid/projects/<name>/state.sqlite``):

- ``task_state`` — tracks pending/claimed/completed/timed-out
  lifecycle for every TaskEntry the dispatcher has submitted.
  Used by the retry loop and exposed through ``GET /tasks``.
- ``kudos_ledger`` — append-only, Ed25519-signed hash chain of
  kudos credits. Each entry's ``entry_hash`` chains from the
  previous row so a 1-byte tamper anywhere in the ledger breaks
  the whole chain from that point on.

Migrations are handled in-process by :mod:`migrations`. On every
coordinator boot, the migrator ensures the schema is at the
current version before the dispatcher/validator touch the DB.
"""

from nexus_coordinator.db.migrations import LATEST_SCHEMA_VERSION, init_db

__all__ = ["LATEST_SCHEMA_VERSION", "init_db"]
