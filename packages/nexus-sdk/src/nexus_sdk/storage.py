"""AppContext.storage — per-app per-project JSON KV with typed namespaces.

Sprint 9 Phase B (D1 impl). Every app gets its own
:class:`AppStorage` wired by the coordinator loader as
``ctx.storage``. The store persists a flat ``str -> JSON`` map at
``<projects_root>/<project>/apps/<app>/storage.json`` with this
on-disk shape::

    {"schema": 1, "payload": {"some.key": "value", ...}}

Design choices (frozen Sprint 9 Day 0 D1)
-----------------------------------------

- **Per-app, per-project scope.** A given ``AppStorage`` instance
  is bound to a single ``(project, app)`` pair. The coordinator
  constructs one per app at boot and assigns it to
  :attr:`nexus_sdk.AppContext.storage`. An app cannot read or
  write another app's storage and never sees the underlying path.
- **Lazy file creation.** The on-disk file is materialised only on
  the first :meth:`AppStorage.set` / :meth:`delete` / :meth:`clear`
  that flushes a non-empty payload. Constructing an :class:`AppStorage`
  on a path that does not exist is a pure in-memory operation.
- **Atomic rename via tmpfile + os.replace.** Writes go through a
  sibling tmpfile (created via :func:`tempfile.mkstemp` in the
  parent directory so it lives on the same filesystem) and are
  swapped into place with :func:`os.replace`. This is the canonical
  POSIX + Windows atomic rename pattern used by diskcache, TinyDB
  and pydantic-settings — a partial write can never leave the
  storage file in a half-updated state, even on crash.
- **Write coalescing via :func:`asyncio.AbstractEventLoop.call_later`.**
  Each mutation marks the store dirty and schedules a single
  deferred flush 500 ms later. Subsequent mutations cancel and
  reschedule the same timer so a burst of N writes results in one
  on-disk write — pattern lifted from TinyDB's
  ``CachingMiddleware``. The delay is configurable via
  :class:`AppStorage`'s ``flush_delay_seconds`` constructor kwarg
  for tests that want to trigger the timer deterministically.
- **One ``asyncio.Lock`` per instance.** All mutators and
  accessors take the lock so the in-memory dict is never observed
  in a half-updated shape from a concurrent task. The lock is held
  only during in-memory work — the deferred flush re-acquires it
  separately, so a long-running set never stalls a peer get
  beyond the time it takes to mutate one dict slot.
- **Flush-on-shutdown via :meth:`flush_on_shutdown`.** The
  coordinator's lifespan tear-down calls this on every app's
  storage so any pending coalesced write lands on disk before the
  process exits. The method cancels any in-flight timer and writes
  synchronously under the lock; subsequent writes are accepted but
  the timer-driven flush is skipped because the instance is closed.
- **Untyped API:** ``await ctx.storage.get(key)`` /
  ``set(key, value)`` / ``delete(key)`` /
  ``keys(prefix=None)`` / ``clear()``. Values must be
  JSON-serialisable; the validation happens at flush time so
  callers see a clear :class:`TypeError` from the standard
  ``json`` encoder rather than a custom wrapping.
- **Typed API:** ``ns = ctx.storage.namespace(key, Schema)`` returns
  a :class:`TypedNamespace` that wraps a single key with a Pydantic
  model. ``await ns.get()`` runs ``Schema.model_validate()`` on the
  raw value and returns the validated instance (or raises
  :class:`StorageSchemaError` on drift); ``await ns.set(value)``
  validates the input and writes the dumped JSON form. The drift
  detection is the critical Sprint 9 promise — if a future app
  version reads a storage file written by an older version with an
  incompatible schema, the namespace get raises a structured error
  instead of silently returning a half-typed model.

Anti-patterns explicitly rejected
---------------------------------

- **SQLite.** Redundant with :class:`nexus_sdk.AppDatabaseClient`
  and overkill for the soft UI state this primitive serves.
- **iroh-docs.** Storage is strictly local to the coordinator
  process; replication would add a network layer for no win.
- **File locking (``fcntl`` / ``msvcrt``).** Sprint 7 D1 froze the
  coordinator as a strict singleton, so cross-process locks are
  not needed.
- **Pickle (``shelve`` / ``sqlitedict``).** Pickle is non-portable
  across Python versions and a security footgun on untrusted
  input — JSON is the only serialisation format on the storage
  surface.

Reference: ``.planning/sprint9_kickoff.md`` §4 D1,
``.planning/sprint9_plan.md`` §5 Phase B, ``docs/shell/PATTERNS.md``
P13.
"""

from __future__ import annotations

import asyncio
import json
import os
import tempfile
from pathlib import Path
from typing import Any, Generic, TypeVar

from pydantic import BaseModel, ValidationError

__all__ = [
    "AppStorage",
    "StorageSchemaError",
    "TypedNamespace",
]


_SCHEMA_VERSION = 1
_DEFAULT_FLUSH_DELAY_SECONDS = 0.5


class StorageSchemaError(Exception):
    """Raised when an :class:`AppStorage` payload or a
    :class:`TypedNamespace` value fails Pydantic validation.

    Two distinct failure modes converge on this exception:

    1. **Drift on read.** ``TypedNamespace.get`` finds a raw value
       at its key whose shape no longer matches the schema (the
       canonical Sprint 9 → Sprint 10 audit gate scenario: a
       storage file written by an older app version with a now
       incompatible schema). The error message cites the key and
       the schema name so a future migration handler can fix the
       data without grepping.
    2. **Rejection on write.** ``TypedNamespace.set`` is called
       with a value that does not validate against the schema.
       This is a programmer error and the exception bubbles up
       like any other validation failure.

    Both cases preserve the underlying
    :class:`pydantic.ValidationError` on ``__cause__``.
    """


T = TypeVar("T", bound=BaseModel)


class TypedNamespace(Generic[T]):
    """Pydantic-validated facade over a single :class:`AppStorage` key.

    Constructed via :meth:`AppStorage.namespace` rather than
    instantiated directly. Holds three things:

    - A reference to the underlying :class:`AppStorage` (so the
      lock and the on-disk write path are shared).
    - The flat key the namespace lives at (a single string;
      typically dotted, e.g. ``"filters.politicians"``).
    - The Pydantic model class used to validate values on read and
      write.

    The namespace is intentionally **not** a sub-tree — one key,
    one model, one document. Apps that need a richer namespace
    layout register multiple namespaces with distinct keys.
    """

    def __init__(self, storage: "AppStorage", key: str, schema: type[T]) -> None:
        self._storage = storage
        self._key = key
        self._schema = schema

    @property
    def key(self) -> str:
        """Return the underlying flat storage key."""
        return self._key

    @property
    def schema(self) -> type[T]:
        """Return the Pydantic model class bound to this namespace."""
        return self._schema

    async def get(self, default: T | None = None) -> T | None:
        """Return the validated model stored at the namespace key.

        When the key is missing, returns ``default`` (which itself
        defaults to ``None``). When the key is present but the
        stored payload no longer matches the schema, raises
        :class:`StorageSchemaError` with a message that names the
        key and the schema; the original
        :class:`pydantic.ValidationError` is preserved on
        ``__cause__``.
        """
        raw = await self._storage.get(self._key)
        if raw is None:
            return default
        try:
            return self._schema.model_validate(raw)
        except ValidationError as exc:
            raise StorageSchemaError(
                f"AppStorage value at key {self._key!r} does not match schema {self._schema.__name__}: {exc}"
            ) from exc

    async def set(self, value: T | dict[str, Any]) -> None:
        """Validate ``value`` against the schema and persist it.

        ``value`` may be a model instance OR a plain dict — both
        flow through ``Schema.model_validate()`` so the same drift
        detection guarantees apply on write. A non-validating
        input raises :class:`StorageSchemaError` and the storage
        is left untouched.
        """
        try:
            validated = self._schema.model_validate(value)
        except ValidationError as exc:
            raise StorageSchemaError(
                f"value rejected by schema {self._schema.__name__} for key {self._key!r}: {exc}"
            ) from exc
        await self._storage.set(self._key, validated.model_dump(mode="json"))

    async def delete(self) -> None:
        """Remove the namespace key from the underlying storage.

        Convenience wrapper around :meth:`AppStorage.delete` so
        callers do not need to remember the underlying key.
        """
        await self._storage.delete(self._key)


class AppStorage:
    """Per-app per-project JSON KV with deferred coalesced flush.

    Parameters
    ----------
    storage_path:
        Path to the on-disk JSON file. Constructing an instance
        does not touch the filesystem — the file is materialised
        on the first flush triggered by a mutating operation.
    flush_delay_seconds:
        How long to wait after a mutation before persisting the
        in-memory state. Defaults to 0.5 seconds. Tests that want
        deterministic timer behaviour can pass a small value or
        monkeypatch :meth:`_schedule_flush` directly.
    """

    def __init__(
        self,
        storage_path: Path | str,
        *,
        flush_delay_seconds: float = _DEFAULT_FLUSH_DELAY_SECONDS,
    ) -> None:
        self._path = Path(storage_path)
        self._flush_delay_seconds = float(flush_delay_seconds)
        self._data: dict[str, Any] = {}
        self._loaded = False
        self._lock = asyncio.Lock()
        self._dirty = False
        self._flush_handle: asyncio.TimerHandle | None = None
        self._closed = False

    # ------------------------------------------------------------------
    # Public read-only properties
    # ------------------------------------------------------------------

    @property
    def path(self) -> Path:
        """Return the on-disk JSON path this instance writes to."""
        return self._path

    @property
    def flush_delay_seconds(self) -> float:
        """Return the deferred flush delay in seconds."""
        return self._flush_delay_seconds

    @property
    def closed(self) -> bool:
        """Return whether :meth:`flush_on_shutdown` has been called."""
        return self._closed

    # ------------------------------------------------------------------
    # Untyped CRUD surface
    # ------------------------------------------------------------------

    async def get(self, key: str) -> Any:
        """Return the value stored at ``key`` or ``None`` if absent.

        Loads the on-disk file lazily on the first call.
        """
        async with self._lock:
            self._ensure_loaded_locked()
            return self._data.get(key)

    async def set(self, key: str, value: Any) -> None:
        """Set ``key`` to ``value`` and schedule a deferred flush.

        ``value`` must be JSON-serialisable; the encoding happens
        at flush time so a non-serialisable value surfaces as a
        :class:`TypeError` from the standard ``json`` module on
        the next flush. Callers that want immediate validation can
        wrap the call in ``json.dumps(value)``.
        """
        async with self._lock:
            self._ensure_loaded_locked()
            self._data[key] = value
            self._mark_dirty_locked()

    async def delete(self, key: str) -> None:
        """Remove ``key`` from the store.

        A missing key is a no-op — no error is raised so callers
        can use ``delete`` as an idempotent reset.
        """
        async with self._lock:
            self._ensure_loaded_locked()
            if key in self._data:
                del self._data[key]
                self._mark_dirty_locked()

    async def keys(self, prefix: str | None = None) -> list[str]:
        """Return the sorted list of keys, optionally filtered by prefix.

        The list is a snapshot — mutations after the call do not
        affect it. Sorting is lexicographical (Python default) so
        the result is stable across calls.
        """
        async with self._lock:
            self._ensure_loaded_locked()
            ks = sorted(self._data.keys())
            if prefix is not None:
                ks = [k for k in ks if k.startswith(prefix)]
            return ks

    async def clear(self) -> None:
        """Remove every key from the store and schedule a flush.

        A no-op when the store is already empty (no flush is
        scheduled in that case so a clear-on-empty does not write
        an unnecessary on-disk update).
        """
        async with self._lock:
            self._ensure_loaded_locked()
            if self._data:
                self._data = {}
                self._mark_dirty_locked()

    # ------------------------------------------------------------------
    # Typed namespace factory
    # ------------------------------------------------------------------

    def namespace(self, key: str, schema: type[T]) -> TypedNamespace[T]:
        """Return a :class:`TypedNamespace` bound to ``key`` and ``schema``.

        Multiple calls with the same ``(key, schema)`` pair return
        independent :class:`TypedNamespace` instances; they share
        the underlying :class:`AppStorage` so writes through one
        are observable through the other.
        """
        return TypedNamespace(self, key, schema)

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    async def flush_on_shutdown(self) -> None:
        """Cancel any pending coalesced flush and write synchronously.

        Called by the coordinator's lifespan tear-down on every
        app's storage. After the call returns the instance is
        marked closed: subsequent mutations are still accepted in
        memory but no new timer is scheduled, so the
        :meth:`set` / :meth:`delete` / :meth:`clear` calls become
        in-memory only and a fresh
        :class:`AppStorage` constructed on the same path will not
        observe them. The intent is that the coordinator owns the
        storage lifecycle — apps that mutate after shutdown are
        racing the host and lose.
        """
        if self._flush_handle is not None:
            self._flush_handle.cancel()
            self._flush_handle = None
        async with self._lock:
            if self._dirty and self._loaded:
                self._write_blob_locked()
        self._closed = True

    # ------------------------------------------------------------------
    # Internal helpers (locked = caller already owns ``self._lock``)
    # ------------------------------------------------------------------

    def _ensure_loaded_locked(self) -> None:
        """Read the on-disk JSON file the first time it is needed.

        Synchronous because :class:`pathlib.Path.read_text` is fast
        for the small payloads this primitive serves (UI filters,
        feature flags, last-selected items). The whole call sits
        under :attr:`_lock` so a concurrent first-read sees the
        same loaded state.
        """
        if self._loaded:
            return
        if self._path.exists():
            try:
                blob = json.loads(self._path.read_text(encoding="utf-8"))
            except (OSError, json.JSONDecodeError) as exc:
                raise StorageSchemaError(f"AppStorage failed to load {self._path.as_posix()!r}: {exc}") from exc
            if not isinstance(blob, dict):
                raise StorageSchemaError(f"AppStorage payload at {self._path.as_posix()!r} is not a JSON object")
            schema = blob.get("schema")
            if schema != _SCHEMA_VERSION:
                raise StorageSchemaError(
                    f"AppStorage schema {schema!r} unsupported "
                    f"(expected {_SCHEMA_VERSION}) at {self._path.as_posix()!r}"
                )
            payload = blob.get("payload")
            if not isinstance(payload, dict):
                raise StorageSchemaError(f"AppStorage payload at {self._path.as_posix()!r} is not a JSON object")
            self._data = dict(payload)
        else:
            self._data = {}
        self._loaded = True

    def _mark_dirty_locked(self) -> None:
        """Mark the in-memory state dirty and (re)schedule a flush.

        A pending timer is cancelled before a new one is scheduled
        so a burst of N writes coalesces into a single on-disk
        flush. When :attr:`_closed` is set the schedule is
        skipped — the storage is in shutdown mode and the host
        owns the next write decision.
        """
        self._dirty = True
        if self._closed:
            return
        if self._flush_handle is not None:
            self._flush_handle.cancel()
            self._flush_handle = None
        self._schedule_flush()

    def _schedule_flush(self) -> None:
        """Schedule the deferred flush via the running event loop.

        Extracted into a method so tests can monkeypatch it on a
        single instance to capture the scheduling without touching
        the loop globally. The default implementation calls
        ``loop.call_later(self._flush_delay_seconds, self._on_flush_timer)``
        and stores the returned :class:`asyncio.TimerHandle` on
        :attr:`_flush_handle` so a subsequent mutation can cancel it.
        """
        try:
            loop = asyncio.get_running_loop()
        except RuntimeError:
            # No running loop — happens in tests that exercise the
            # locked helpers directly. The next mutation under a
            # running loop will reschedule, and shutdown still
            # writes synchronously.
            return
        self._flush_handle = loop.call_later(self._flush_delay_seconds, self._on_flush_timer)

    def _on_flush_timer(self) -> None:
        """Timer callback that hands off to the async flush task.

        Synchronous because :meth:`asyncio.AbstractEventLoop.call_later`
        invokes a sync callable — we cannot ``await`` here. The
        method spawns an async task that takes the lock and writes.
        """
        self._flush_handle = None
        if self._closed:
            return
        try:
            loop = asyncio.get_running_loop()
        except RuntimeError:
            return
        loop.create_task(self._deferred_flush())

    async def _deferred_flush(self) -> None:
        """Async wrapper around the locked write path.

        Acquires :attr:`_lock` cleanly so it never collides with a
        concurrent ``set`` / ``delete``; if the dirty flag was
        cleared between the timer firing and the task running
        (e.g. a manual ``flush_on_shutdown`` raced ahead) the
        method is a no-op.
        """
        async with self._lock:
            if self._dirty:
                self._write_blob_locked()

    def _write_blob_locked(self) -> None:
        """Persist the in-memory state to disk via atomic rename.

        Writes to a sibling tmpfile created with
        :func:`tempfile.mkstemp` so it lives on the same filesystem
        (cross-device :func:`os.replace` is unsupported on Linux),
        then swaps the file into place with :func:`os.replace`. On
        any failure the tmpfile is removed and the exception
        propagates so the caller sees the real error.
        """
        self._path.parent.mkdir(parents=True, exist_ok=True)
        blob = {"schema": _SCHEMA_VERSION, "payload": self._data}
        fd, tmp_name = tempfile.mkstemp(
            prefix=f".{self._path.name}.",
            suffix=".tmp",
            dir=str(self._path.parent),
        )
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as f:
                json.dump(
                    blob,
                    f,
                    ensure_ascii=False,
                    sort_keys=True,
                    separators=(",", ":"),
                )
            os.replace(tmp_name, str(self._path))
        except Exception:
            try:
                os.unlink(tmp_name)
            except OSError:
                pass
            raise
        self._dirty = False
