# SPDX-License-Identifier: AGPL-3.0-or-later
"""Result and claim validator.

The validator subscribes to the project doc via
:meth:`nexus_core.Doc.subscribe` and reacts to every ``claim:*``
and ``result:*`` entry:

- A ``claim:<task_id>`` write: deserialize the ClaimEntry, verify
  its Ed25519 signature via :func:`nexus_core.verify_claim_entry`,
  and mark the task ``claimed`` in ``task_state``.
- A ``result:<task_id>`` write: deserialize the ResultEntry,
  verify via the 3-layer :class:`nexus_core.Verifier` (Ed25519
  signature + model digest whitelist + logprob fingerprint) and,
  on pass, credit kudos via :class:`KudosLedger.credit`.

All inbound blob content is fetched through ``node.blobs()``; the
validator does not care whether the data was written locally
(Phase B tests) or arrived via remote sync (Phase D e2e), because
iroh-docs fetches blob content during sync before firing
``InsertRemote`` events.

The loop exposes two hooks for tests:

- :meth:`Validator.run_once` — process one iteration of the
  LiveEvent stream (with an optional max_events cap) and return
  a per-event summary.
- :meth:`Validator.run_forever` — drive the loop until the
  coordinator stops.
"""

from __future__ import annotations

import asyncio
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import nexus_core
import structlog

from nexus_coordinator.dispatcher import Dispatcher
from nexus_coordinator.kudos import KudosLedger

_log = structlog.get_logger(__name__)


@dataclass
class ValidationEvent:
    """Summary of a single validator iteration, surfaced by tests."""

    kind: str  # "claim" | "result_ok" | "result_rejected" | "noop"
    task_id: str | None = None
    worker_pubkey_hex: str | None = None
    reason: str | None = None


class Validator:
    """Consumes LiveEvents on the project doc and acts on them."""

    def __init__(
        self,
        *,
        doc: Any,  # nexus_core.Doc
        node: Any,  # nexus_core.Node
        dispatcher: Dispatcher,
        kudos: KudosLedger,
        db_path: Path,
    ) -> None:
        self._doc = doc
        self._node = node
        self._dispatcher = dispatcher
        self._kudos = kudos
        self._db_path = db_path
        self._subscription: Any | None = None
        self._verifier: Any = nexus_core.Verifier()

    async def start(self) -> None:
        """Open a LiveEvent subscription on the doc."""
        self._subscription = await self._doc.subscribe()

    async def stop(self) -> None:
        """Close the subscription."""
        if self._subscription is not None:
            try:
                await self._subscription.close()
            except Exception as e:  # noqa: BLE001
                _log.debug("subscription close raised", error=str(e))
            self._subscription = None

    async def run_forever(self) -> None:
        """Long-lived consumer loop for the Phase A/B in-process
        coordinator. Phase C+ will embed this into the uvicorn
        lifespan."""
        if self._subscription is None:
            await self.start()
        while True:
            try:
                await self._step_once()
            except asyncio.CancelledError:
                break
            except Exception as e:  # noqa: BLE001
                _log.error("validator step raised, backing off", error=str(e))
                await asyncio.sleep(0.5)

    async def run_once(self, max_events: int = 1) -> list[ValidationEvent]:
        """Process up to ``max_events`` LiveEvents and return the
        summaries. Used by tests that want deterministic drain
        semantics."""
        if self._subscription is None:
            await self.start()
        out: list[ValidationEvent] = []
        for _ in range(max_events):
            ev = await self._step_once()
            if ev is not None:
                out.append(ev)
        return out

    async def _step_once(self) -> ValidationEvent | None:
        """Pull one event off the subscription and handle it.

        Returns the ValidationEvent summary or None if the event
        was not a task/claim/result write (e.g. neighbor_up).
        """
        if self._subscription is None:
            raise RuntimeError("validator not started")
        raw = await self._subscription.next_event()
        if raw is None:
            # Stream ended — in practice only happens during
            # shutdown. Callers can treat None as a signal to exit.
            raise asyncio.CancelledError()

        kind = raw.get("kind")
        if kind not in ("insert_local", "insert_remote"):
            return ValidationEvent(kind="noop", reason=f"ignored {kind}")

        entry = raw.get("entry")
        if not isinstance(entry, dict):
            return ValidationEvent(kind="noop", reason="no entry payload")
        key: bytes = entry.get("key", b"")
        if key.startswith(b"claim:"):
            return await self._handle_claim(entry)
        if key.startswith(b"result:"):
            return await self._handle_result(entry)
        # task:* writes come from the dispatcher itself; nothing
        # to do on the validator side.
        return ValidationEvent(kind="noop", reason=f"ignored key prefix {key[:16]!r}")

    async def _fetch_content(self, entry_hash: bytes) -> bytes | None:
        """Read the blob content for a doc entry by its hash."""
        try:
            return await self._node.blobs().get_bytes(entry_hash)
        except Exception as e:  # noqa: BLE001
            _log.warning("blob fetch failed", hash=entry_hash.hex(), error=str(e))
            return None

    async def _handle_claim(self, entry: dict[str, Any]) -> ValidationEvent:
        task_id = entry["key"].decode("utf-8").removeprefix("claim:")
        content = await self._fetch_content(entry["hash"])
        if content is None:
            return ValidationEvent(kind="noop", task_id=task_id, reason="content missing")
        try:
            nexus_core.verify_claim_entry(content.decode("utf-8"))
        except Exception as e:  # noqa: BLE001
            _log.warning("claim signature invalid", task_id=task_id, error=str(e))
            return ValidationEvent(kind="noop", task_id=task_id, reason=f"bad claim sig: {e}")

        claim_entry = json.loads(content.decode("utf-8"))
        worker_pubkey: list[int] = claim_entry["worker_pubkey"]
        worker_pubkey_bytes = bytes(worker_pubkey)
        await self._dispatcher.mark_claimed(task_id, worker_pubkey_bytes)
        _log.info("claim validated", task_id=task_id, worker_pubkey_hex=worker_pubkey_bytes.hex())
        return ValidationEvent(
            kind="claim",
            task_id=task_id,
            worker_pubkey_hex=worker_pubkey_bytes.hex(),
        )

    async def _handle_result(self, entry: dict[str, Any]) -> ValidationEvent:
        task_id = entry["key"].decode("utf-8").removeprefix("result:")
        content = await self._fetch_content(entry["hash"])
        if content is None:
            return ValidationEvent(kind="noop", task_id=task_id, reason="content missing")

        result_json = content.decode("utf-8")
        # Fetch the TaskEntry this result is for, from the task_state
        # mirror (so we don't re-scan the doc on every result).
        import aiosqlite

        async with aiosqlite.connect(self._db_path) as db:
            async with db.execute("SELECT task_json FROM task_state WHERE task_id = ?", (task_id,)) as cursor:
                row = await cursor.fetchone()
        if row is None:
            reason = "unknown task_id"
            _log.warning("result for unknown task", task_id=task_id)
            return ValidationEvent(kind="result_rejected", task_id=task_id, reason=reason)
        task_entry_json: str = row[0]

        # 3-layer verification via the Rust Verifier.
        report = self._verifier.verify_entries(task_entry_json, result_json, "")
        if not report.get("passed", False):
            reason = str(report)
            await self._dispatcher.mark_failed(task_id, reason[:400])
            _log.warning("result verification failed", task_id=task_id, report=report)
            return ValidationEvent(
                kind="result_rejected",
                task_id=task_id,
                reason=reason,
            )

        # Parse just enough of the result to pull out the worker
        # pubkey + tokens count for the kudos credit.
        result_entry = json.loads(result_json)
        worker_pubkey = bytes(result_entry["worker_pubkey"])
        tokens = int(result_entry["payload"]["tokens_generated"])
        await self._dispatcher.mark_completed(task_id, entry["hash"])
        await self._kudos.credit(
            worker_pubkey=worker_pubkey,
            task_id=task_id,
            tokens=tokens,
        )
        _log.info(
            "result validated and kudos credited",
            task_id=task_id,
            tokens=tokens,
            worker_pubkey_hex=worker_pubkey.hex(),
        )
        return ValidationEvent(
            kind="result_ok",
            task_id=task_id,
            worker_pubkey_hex=worker_pubkey.hex(),
        )
