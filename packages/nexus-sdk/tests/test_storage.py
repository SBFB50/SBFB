# SPDX-License-Identifier: AGPL-3.0-or-later
"""Tests for :class:`nexus_sdk.storage.AppStorage` (Sprint 9 Phase B).

The 20 scenarios listed in ``.planning/sprint9_plan.md`` §5.2 are
covered here. The tests use a per-test ``tmp_path`` for the
storage file and either drive the deferred flush manually via
:meth:`AppStorage.flush_on_shutdown` (deterministic) or wait for
the real timer with a generous margin (the only test that
exercises the live coalescing path end-to-end).

Categories:

- Basic CRUD (1-7) — get/set roundtrip, delete, sorted keys,
  prefix filter, clear, JSON serialization of nested values.
- Coalescing + atomicity (8-13) — deferred flush schedule,
  multi-set coalesce into one os.replace, flush-on-shutdown,
  tmpfile rename, asyncio.Lock concurrent set, sequential
  set chain has no deadlock.
- Typed namespace (14-18) — model_validate roundtrip on get/set,
  invalid dict raises StorageSchemaError, default on missing,
  raw untyped fallback.
- Persistence (19-20) — across-restart roundtrip, lazy file
  creation on first set.
"""

from __future__ import annotations

import asyncio
import json
import os
from pathlib import Path
from typing import Any

import pytest
from nexus_sdk.storage import AppStorage, StorageSchemaError
from pydantic import BaseModel

# ---------------------------------------------------------------------------
# 1-7 — basic CRUD
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_storage_get_missing_key_returns_none(tmp_path: Path) -> None:
    """An untouched store returns ``None`` for any key — empty
    state is a normal state, not an error."""
    storage = AppStorage(tmp_path / "s.json")
    assert await storage.get("nope") is None


@pytest.mark.asyncio
async def test_storage_set_get_roundtrip_string(tmp_path: Path) -> None:
    """Happy path: set a string, get the string back. Pins the
    public read/write contract for the simplest value type."""
    storage = AppStorage(tmp_path / "s.json")
    await storage.set("greeting", "bonjour")
    assert await storage.get("greeting") == "bonjour"


@pytest.mark.asyncio
async def test_storage_set_get_roundtrip_nested_dict(tmp_path: Path) -> None:
    """A nested JSON-serializable value (dict containing list of
    dicts) round-trips through the in-memory cache without losing
    any structure. The serialisation only happens on flush, so
    this test asserts in-memory equality."""
    storage = AppStorage(tmp_path / "s.json")
    payload = {
        "filters": ["a", "b"],
        "ranges": [{"from": "2024-01-01", "to": "2026-12-31"}],
        "limit": 50,
    }
    await storage.set("query", payload)
    assert await storage.get("query") == payload


@pytest.mark.asyncio
async def test_storage_delete_key_removes_from_state(tmp_path: Path) -> None:
    """``delete`` removes the key. A subsequent ``get`` returns
    ``None`` and the key is no longer in the snapshot returned by
    ``keys``."""
    storage = AppStorage(tmp_path / "s.json")
    await storage.set("k", "v")
    await storage.delete("k")
    assert await storage.get("k") is None
    assert await storage.keys() == []


@pytest.mark.asyncio
async def test_storage_keys_returns_sorted_list(tmp_path: Path) -> None:
    """``keys`` returns a snapshot sorted lexicographically. Pins
    the contract: callers can rely on a stable order without
    sorting client-side."""
    storage = AppStorage(tmp_path / "s.json")
    await storage.set("zebra", 1)
    await storage.set("alpha", 2)
    await storage.set("mike", 3)
    assert await storage.keys() == ["alpha", "mike", "zebra"]


@pytest.mark.asyncio
async def test_storage_keys_with_prefix_filter(tmp_path: Path) -> None:
    """The ``prefix`` argument restricts the snapshot to keys that
    start with the given prefix. Other keys are not returned."""
    storage = AppStorage(tmp_path / "s.json")
    await storage.set("filters.politicians", {"chamber": "AN"})
    await storage.set("filters.parties", {"label": "PS"})
    await storage.set("ui.theme", "dark")
    assert await storage.keys(prefix="filters.") == [
        "filters.parties",
        "filters.politicians",
    ]


@pytest.mark.asyncio
async def test_storage_clear_removes_all_keys(tmp_path: Path) -> None:
    """``clear`` empties the store. ``keys`` then returns an
    empty list and any subsequent ``get`` is ``None``."""
    storage = AppStorage(tmp_path / "s.json")
    await storage.set("k1", 1)
    await storage.set("k2", 2)
    await storage.clear()
    assert await storage.keys() == []
    assert await storage.get("k1") is None


# ---------------------------------------------------------------------------
# 8-13 — coalescing, atomic write, locking, lifespan
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_storage_set_triggers_deferred_flush(tmp_path: Path) -> None:
    """Mutating the store schedules a deferred flush via
    ``loop.call_later`` and stores the resulting
    :class:`asyncio.TimerHandle` on ``_flush_handle``. The test
    monkeypatches :meth:`AppStorage._schedule_flush` on the
    instance to capture the call without touching the loop
    globally — the timer is **mocked**, not awaited via real
    sleep."""
    storage = AppStorage(tmp_path / "s.json")
    captured: list[float] = []

    def fake_schedule() -> None:
        captured.append(storage.flush_delay_seconds)

    storage._schedule_flush = fake_schedule  # type: ignore[method-assign]
    await storage.set("k", "v")
    assert captured == [0.5]
    # In-memory state is the source of truth before any flush.
    assert await storage.get("k") == "v"


@pytest.mark.asyncio
async def test_storage_multiple_sets_coalesce_into_one_flush(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Five sets in a tight burst followed by a 0.7 s wait must
    result in **one** ``os.replace`` call. The deferred flush is
    rescheduled on every mutation so the burst collapses into a
    single on-disk write — the canonical TinyDB
    ``CachingMiddleware`` pattern."""
    import nexus_sdk.storage as storage_mod

    storage = AppStorage(tmp_path / "s.json", flush_delay_seconds=0.05)
    counter = {"calls": 0}
    real_replace = os.replace

    def counting_replace(src: Any, dst: Any) -> None:
        counter["calls"] += 1
        real_replace(src, dst)

    monkeypatch.setattr(storage_mod.os, "replace", counting_replace)

    for i in range(5):
        await storage.set(f"k{i}", f"v{i}")
    # Give the deferred timer time to fire (50 ms delay × 5
    # cancels means a real sleep of ~150 ms is comfortable).
    await asyncio.sleep(0.2)

    assert counter["calls"] == 1


@pytest.mark.asyncio
async def test_storage_flush_on_shutdown_writes_pending(tmp_path: Path) -> None:
    """``flush_on_shutdown`` cancels the deferred timer and writes
    the pending state to disk synchronously. The on-disk file is
    readable as JSON with ``schema=1`` and the payload contains
    the value the test set."""
    storage = AppStorage(tmp_path / "s.json")
    await storage.set("k", "v")
    assert (
        not (tmp_path / "s.json").exists()
        or json.loads((tmp_path / "s.json").read_text(encoding="utf-8"))["payload"] == {}
    )
    await storage.flush_on_shutdown()

    blob = json.loads((tmp_path / "s.json").read_text(encoding="utf-8"))
    assert blob == {"schema": 1, "payload": {"k": "v"}}
    assert storage.closed is True


@pytest.mark.asyncio
async def test_storage_atomic_rename_uses_tmpfile(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """The on-disk write goes through a sibling tmpfile that is
    swapped into place via :func:`os.replace`. The test spies on
    ``os.replace`` to capture the source path and asserts it is
    distinct from the final destination — the canonical
    POSIX/Windows atomic rename pattern."""
    import nexus_sdk.storage as storage_mod

    storage = AppStorage(tmp_path / "s.json")
    captured: list[tuple[str, str]] = []
    real_replace = os.replace

    def spy_replace(src: Any, dst: Any) -> None:
        captured.append((str(src), str(dst)))
        real_replace(src, dst)

    monkeypatch.setattr(storage_mod.os, "replace", spy_replace)

    await storage.set("k", "v")
    await storage.flush_on_shutdown()

    assert len(captured) == 1
    src, dst = captured[0]
    assert Path(src) != Path(dst), "tmpfile and target must be distinct paths"
    assert Path(dst) == storage.path
    # The tmpfile lives in the same directory as the target so
    # ``os.replace`` is intra-filesystem (cross-device replace is
    # unsupported on Linux).
    assert Path(src).parent == storage.path.parent
    assert Path(src).name.startswith(f".{storage.path.name}.")


@pytest.mark.asyncio
async def test_storage_concurrent_set_via_asyncio_lock(tmp_path: Path) -> None:
    """Two tasks calling :meth:`set` concurrently via
    :func:`asyncio.gather` see their writes serialised by the
    per-instance :class:`asyncio.Lock`. Both keys land in the
    in-memory state and the on-disk file (after shutdown) is
    valid JSON with both keys present."""
    storage = AppStorage(tmp_path / "s.json")

    async def writer(key: str, value: str) -> None:
        await storage.set(key, value)

    await asyncio.gather(writer("k1", "v1"), writer("k2", "v2"))
    await storage.flush_on_shutdown()

    blob = json.loads((tmp_path / "s.json").read_text(encoding="utf-8"))
    assert blob["payload"] == {"k1": "v1", "k2": "v2"}


@pytest.mark.asyncio
async def test_storage_reentry_same_task_does_not_deadlock(tmp_path: Path) -> None:
    """A handler that issues two sequential :meth:`set` calls in
    the same task must not deadlock — the lock is released
    between calls. The test runs two consecutive sets and asserts
    both completed within a generous timeout."""
    storage = AppStorage(tmp_path / "s.json")

    async def handler() -> None:
        await storage.set("k1", "v1")
        await storage.set("k2", "v2")

    await asyncio.wait_for(handler(), timeout=2.0)
    assert await storage.get("k1") == "v1"
    assert await storage.get("k2") == "v2"


# ---------------------------------------------------------------------------
# 14-18 — typed namespace
# ---------------------------------------------------------------------------


class _SampleSchema(BaseModel):
    """Sample Pydantic model for namespace tests.

    Carries a string and an int so the validation tests have
    something to misalign on."""

    chamber: str
    limit: int = 10


@pytest.mark.asyncio
async def test_storage_namespace_typed_get_returns_validated_model(
    tmp_path: Path,
) -> None:
    """``TypedNamespace.get`` runs ``Schema.model_validate`` on
    the raw value and returns a model instance — not the raw
    dict."""
    storage = AppStorage(tmp_path / "s.json")
    ns = storage.namespace("filters.sample", _SampleSchema)
    await ns.set(_SampleSchema(chamber="AN", limit=25))

    got = await ns.get()
    assert isinstance(got, _SampleSchema)
    assert got.chamber == "AN"
    assert got.limit == 25


@pytest.mark.asyncio
async def test_storage_namespace_typed_set_accepts_model_instance(
    tmp_path: Path,
) -> None:
    """``TypedNamespace.set`` accepts both a model instance and a
    plain dict — both flow through ``Schema.model_validate``."""
    storage = AppStorage(tmp_path / "s.json")
    ns = storage.namespace("filters.sample", _SampleSchema)

    await ns.set(_SampleSchema(chamber="Senat", limit=5))
    via_dict_ns = storage.namespace("filters.sample2", _SampleSchema)
    await via_dict_ns.set({"chamber": "AN", "limit": 99})

    assert (await ns.get()).chamber == "Senat"
    assert (await via_dict_ns.get()).limit == 99


@pytest.mark.asyncio
async def test_storage_namespace_typed_set_rejects_invalid_dict(
    tmp_path: Path,
) -> None:
    """A dict that does not validate against the schema raises
    :class:`StorageSchemaError` and the storage is left untouched
    — the new value never reaches the in-memory state."""
    storage = AppStorage(tmp_path / "s.json")
    ns = storage.namespace("filters.sample", _SampleSchema)
    await ns.set(_SampleSchema(chamber="AN", limit=10))

    with pytest.raises(StorageSchemaError) as excinfo:
        await ns.set({"chamber": 42, "limit": "not-an-int"})  # type: ignore[arg-type]
    assert "_SampleSchema" in str(excinfo.value)
    assert "filters.sample" in str(excinfo.value)

    # The pre-existing value must still be intact.
    got = await ns.get()
    assert got is not None and got.chamber == "AN"


@pytest.mark.asyncio
async def test_storage_namespace_typed_get_returns_default_on_missing_key(
    tmp_path: Path,
) -> None:
    """When the namespace key is absent, ``TypedNamespace.get``
    returns the supplied ``default`` (or ``None`` if not given)."""
    storage = AppStorage(tmp_path / "s.json")
    ns = storage.namespace("filters.sample", _SampleSchema)

    assert await ns.get() is None
    fallback = _SampleSchema(chamber="AN", limit=1)
    assert await ns.get(default=fallback) is fallback


@pytest.mark.asyncio
async def test_storage_namespace_untyped_fallback(tmp_path: Path) -> None:
    """A value written via the typed namespace remains readable
    through the raw :meth:`AppStorage.get` API as the dumped
    JSON dict — the typed wrapper is sugar over the untyped
    KV, not a separate store."""
    storage = AppStorage(tmp_path / "s.json")
    ns = storage.namespace("filters.sample", _SampleSchema)
    await ns.set(_SampleSchema(chamber="AN", limit=7))

    raw = await storage.get("filters.sample")
    assert raw == {"chamber": "AN", "limit": 7}


# ---------------------------------------------------------------------------
# 19-20 — persistence
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_storage_persists_across_restart(tmp_path: Path) -> None:
    """Instance 1 writes a value and flushes; instance 2 created
    on the same path observes the same value via ``get``. Pins
    the round-trip across the on-disk file."""
    path = tmp_path / "persist.json"

    s1 = AppStorage(path)
    await s1.set("filters.sample", {"chamber": "AN", "limit": 50})
    await s1.flush_on_shutdown()

    s2 = AppStorage(path)
    assert await s2.get("filters.sample") == {"chamber": "AN", "limit": 50}


@pytest.mark.asyncio
async def test_storage_missing_file_creates_lazy_on_first_set(
    tmp_path: Path,
) -> None:
    """Constructing an :class:`AppStorage` on a path that does
    not exist must not touch the filesystem. The file is created
    only after the first ``set`` reaches the on-disk flush —
    triggered here via ``flush_on_shutdown``."""
    path = tmp_path / "lazy.json"
    storage = AppStorage(path)
    assert not path.exists()

    # A read on a fresh store still does not create the file.
    assert await storage.get("nope") is None
    assert not path.exists()

    await storage.set("k", "v")
    await storage.flush_on_shutdown()
    assert path.exists()
