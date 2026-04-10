"""The Coordinator process.

A coordinator is the long-lived Python process that owns a nexus-grid
project. Responsibilities (Sprint 4 Phase A scope only):

- Load the persistent Ed25519 keypair via
  :func:`nexus_coordinator.keystore.load_or_generate_keypair`.
- Boot an iroh :class:`nexus_core.Node` with that secret so the
  node id is stable across restarts.
- Create an author id on the first boot, reuse it on subsequent
  boots.
- Create a project doc on the first boot (namespace id persisted
  in ``coordinator.toml``), reopen it via
  :meth:`nexus_core.Node.docs_open` on later boots.
- Mint a write ticket for the project doc (used by Phase C to
  embed into invites).
- Expose a local-only FastAPI on ``127.0.0.1:<port>`` with
  ``/health`` and ``/project``.

Phases B/C/D extend this class with dispatcher, validator, kudos
ledger, invite routes, and SDK loader — but the boot path and the
Node ownership stay here.

Single-doc layout (Sprint 4 decision E): the project uses one doc
keyed by ``task:<id>`` / ``claim:<id>`` / ``result:<id>`` prefixes
rather than three separate docs, because a single ticket keeps
invites compact and the per-entry signature on TaskEntry /
ClaimEntry / ResultEntry already provides the authentication that
multi-doc permission separation would add.
"""

from __future__ import annotations

import asyncio
from contextlib import asynccontextmanager
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING

import nexus_core
import structlog

from nexus_coordinator.config import CoordinatorConfig
from nexus_coordinator.dispatcher import Dispatcher
from nexus_coordinator.keystore import LoadedKeypair, load_or_generate_keypair
from nexus_coordinator.kudos import KudosLedger
from nexus_coordinator.paths import (
    coord_config_path,
    coord_key_path,
    iroh_data_path,
    project_dir,
)
from nexus_coordinator.validator import Validator

if TYPE_CHECKING:
    pass

_log = structlog.get_logger(__name__)


@dataclass
class CoordinatorState:
    """Runtime state populated by :meth:`Coordinator.start`.

    Kept as a separate dataclass so every field's presence is a
    compile-time signal: either ``start()`` has completed and
    everything is filled in, or it hasn't and the fields are None.
    """

    node: object | None = None  # nexus_core.Node — dynamic import
    doc: object | None = None  # nexus_core.Doc
    author_id: str | None = None
    doc_id: str | None = None
    node_id: str | None = None
    tasks_doc_ticket: str | None = None
    api_server_task: asyncio.Task[None] | None = field(default=None)
    validator_task: asyncio.Task[None] | None = field(default=None)


class Coordinator:
    """Orchestrates the boot + shutdown of a single project.

    Lifecycle::

        coord = Coordinator(project_name="demo")
        await coord.start()
        # ... serve requests, dispatch tasks, observe results ...
        await coord.stop()

    The constructor is synchronous — it only resolves paths and
    loads config. All iroh / network / filesystem-for-state work
    happens in :meth:`start`.
    """

    def __init__(self, project_name: str, *, config: CoordinatorConfig | None = None) -> None:
        self.project_name = project_name
        self.project_dir: Path = project_dir(project_name)
        self.key_path: Path = coord_key_path(project_name)
        self.config_path: Path = coord_config_path(project_name)
        self.data_dir: Path = iroh_data_path(project_name)

        if config is None:
            config = CoordinatorConfig.load(self.config_path)
        self.config: CoordinatorConfig = config
        # Force the project name to match the directory — the
        # on-disk layout is the source of truth.
        self.config.identity.name = project_name

        self.state = CoordinatorState()
        self._keypair: LoadedKeypair | None = None
        self.dispatcher: Dispatcher | None = None
        self.kudos_ledger: KudosLedger | None = None
        self.validator: Validator | None = None

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    async def start(self) -> None:
        """Boot the iroh node, open or create the project doc,
        mint a write ticket, and return.

        Idempotent-ish: calling twice on the same instance raises
        ``RuntimeError`` because a second boot would leak the
        first node.
        """
        if self.state.node is not None:
            raise RuntimeError("coordinator already started")

        self.project_dir.mkdir(parents=True, exist_ok=True)

        # 1. Keypair
        self._keypair = load_or_generate_keypair(self.key_path)

        # 2. iroh Node with the persistent secret so node_id is
        #    stable across reboots, plus a persistent docs/author
        #    storage under iroh-data/ so the author and the project
        #    namespace survive process restarts.
        self.data_dir.mkdir(parents=True, exist_ok=True)
        self.state.node = await nexus_core.create_node_with_secret(
            self._keypair.secret,
            str(self.data_dir),
        )
        self.state.node_id = self.state.node.node_id
        _log.info(
            "iroh endpoint ready",
            node_id=self.state.node_id,
            project=self.project_name,
        )

        # 3. Author id — persisted across reboots so every write
        #    comes from the same identity. iroh-docs author_default
        #    would give us that for free, but we stick to
        #    author_create + persist to make the identity explicit
        #    in coordinator.toml.
        if self.config.identity.author_id is None:
            self.state.author_id = await self.state.node.docs_author_create()
            self.config.identity.author_id = self.state.author_id
            _log.info("author id minted", author_id=self.state.author_id)
        else:
            # We can't recover the author secret from the on-disk
            # docs store on every iroh build; the create/persist
            # approach above assumes the store path is stable so
            # author_create returns the same id after reboot
            # (iroh-docs persists authors in its default store).
            # On the very first boot with a pre-existing coordinator.toml
            # from a transplanted setup, we fall back to creating a
            # new one.
            self.state.author_id = self.config.identity.author_id

        # 4. Project doc — create on first boot, reopen on later
        #    boots via the persisted namespace id.
        if self.config.identity.doc_id is None:
            doc = await self.state.node.docs_create()
            self.state.doc = doc
            self.state.doc_id = await doc.id()
            self.config.identity.doc_id = self.state.doc_id
            _log.info("project doc created", doc_id=self.state.doc_id)
        else:
            doc = await self.state.node.docs_open(self.config.identity.doc_id)
            if doc is None:
                # The doc id in the config points at a doc that no
                # longer exists in this node's store — e.g. the
                # iroh-data dir was cleaned. Recreate and warn.
                _log.warning(
                    "persisted doc_id not found in local store, creating a new one",
                    stale_doc_id=self.config.identity.doc_id,
                )
                doc = await self.state.node.docs_create()
                self.state.doc = doc
                self.state.doc_id = await doc.id()
                self.config.identity.doc_id = self.state.doc_id
            else:
                self.state.doc = doc
                self.state.doc_id = self.config.identity.doc_id
                _log.info("project doc reopened", doc_id=self.state.doc_id)

        # 5. Mint a write ticket for the doc so Phase C can embed
        #    it into invites. Stored in-memory, not persisted — a
        #    fresh ticket is minted on every boot (cheap, and
        #    avoids stale address info).
        self.state.tasks_doc_ticket = await self.state.doc.share_write()

        # 6. Persist the (possibly updated) config.
        self.config.save(self.config_path)

        # 7. Phase B: wire up the SQLite-backed dispatcher, kudos
        #    ledger, and doc-subscription validator. The state.sqlite
        #    file lives alongside the iroh-data directory in the
        #    project root.
        state_db = self.project_dir / "state.sqlite"
        self.dispatcher = Dispatcher(
            db_path=state_db,
            doc=self.state.doc,
            author_id=self.state.author_id,  # type: ignore[arg-type]
            coord_secret=self._keypair.secret,
        )
        await self.dispatcher.init()

        self.kudos_ledger = KudosLedger(
            db_path=state_db,
            coord_secret=self._keypair.secret,
        )

        self.validator = Validator(
            doc=self.state.doc,
            node=self.state.node,
            dispatcher=self.dispatcher,
            kudos=self.kudos_ledger,
            db_path=state_db,
        )
        await self.validator.start()

    async def stop(self) -> None:
        """Shut down the iroh node and cancel any background tasks.

        Safe to call multiple times; a second call is a no-op.
        """
        if self.state.validator_task is not None:
            self.state.validator_task.cancel()
            try:
                await self.state.validator_task
            except (asyncio.CancelledError, Exception):
                pass
            self.state.validator_task = None

        if self.validator is not None:
            await self.validator.stop()
            self.validator = None

        if self.state.api_server_task is not None:
            self.state.api_server_task.cancel()
            try:
                await self.state.api_server_task
            except (asyncio.CancelledError, Exception):
                pass
            self.state.api_server_task = None

        if self.state.node is not None:
            try:
                await self.state.node.shutdown()
            except Exception as e:  # noqa: BLE001 — best-effort
                _log.warning("iroh node shutdown raised", error=str(e))
            self.state.node = None
            self.state.doc = None
            _log.info("coordinator stopped", project=self.project_name)

    # ------------------------------------------------------------------
    # Convenience
    # ------------------------------------------------------------------

    @property
    def keypair(self) -> LoadedKeypair:
        """Return the loaded keypair, raising if ``start`` hasn't run."""
        if self._keypair is None:
            raise RuntimeError("coordinator not started; call start() first")
        return self._keypair

    def health_payload(self) -> dict[str, object]:
        """Build the body for the ``GET /health`` endpoint."""
        return {
            "status": "ok" if self.state.node is not None else "booting",
            "project": self.project_name,
            "node_id": self.state.node_id,
            "doc_id": self.state.doc_id,
            "author_id": self.state.author_id,
            "version": "0.1.0",
        }

    def project_payload(self) -> dict[str, object]:
        """Build the body for the ``GET /project`` endpoint."""
        return {
            "name": self.project_name,
            "description": self.config.identity.description,
            "visibility": self.config.network.visibility,
            "doc_id": self.state.doc_id,
            "author_id": self.state.author_id,
            "tasks_doc_ticket_prefix": (
                self.state.tasks_doc_ticket[:64] + "…"
                if self.state.tasks_doc_ticket and len(self.state.tasks_doc_ticket) > 64
                else self.state.tasks_doc_ticket
            ),
        }


@asynccontextmanager
async def running_coordinator(project_name: str):
    """Async context manager that starts a coordinator and stops it
    on exit.

    Used by integration tests — and by the CLI ``start`` command
    when it embeds its own lifespan around uvicorn.
    """
    coord = Coordinator(project_name)
    await coord.start()
    try:
        yield coord
    finally:
        await coord.stop()
