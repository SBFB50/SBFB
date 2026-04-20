# SPDX-License-Identifier: AGPL-3.0-or-later
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
from nexus_sdk import (
    AppContext,
    AppDatabaseClient,
    AppEvents,
    AppFileStore,
    AppStorage,
    ComputeClient,
    MigrationRunner,
    MigrationTamperedError,
    NexusApp,
    discover_apps,
)

from nexus_coordinator.canary_registry import CanaryRegistry
from nexus_coordinator.config import CoordinatorConfig
from nexus_coordinator.contributor_registry import ContributorRegistry
from nexus_coordinator.dispatcher import Dispatcher, SubmitRequest
from nexus_coordinator.invite import InviteLedger
from nexus_coordinator.keystore import LoadedKeypair, load_or_generate_keypair
from nexus_coordinator.kudos import KudosLedger
from nexus_coordinator.paths import (
    app_db_path,
    app_storage_path,
    app_uploads_path,
    canary_registry_path,
    contributor_registry_path,
    coord_config_path,
    coord_key_path,
    iroh_data_path,
    project_dir,
)
from nexus_coordinator.quarantine_queue import QuarantineQueue
from nexus_coordinator.upload_queue import UploadQueue
from nexus_coordinator.validator import Validator


async def _dispatcher_emit_adapter(dispatcher: Dispatcher, payload: dict[str, object]) -> str:
    """Bridge a queue payload back to :meth:`Dispatcher.submit`.

    The upload queue stores every task as a plain dict (JSON
    round-trip) so the SQLite column type stays simple. This
    helper re-materialises a :class:`SubmitRequest` from the dict
    before handing it to the dispatcher, tolerating extra keys
    that later revisions may add and stripping the ``version``
    marker that :meth:`Dispatcher.submit` stamps server-side.
    """
    known = {
        "task_type",
        "prompt",
        "model",
        "system_prompt",
        "priority",
        "parent_task_id",
        "metadata",
        "task_id",
        "is_open_source",
        "estimated_watts",
        "estimated_vram_mb",
        "estimated_hours",
    }
    kwargs = {k: v for k, v in payload.items() if k in known}
    return await dispatcher.submit(SubmitRequest(**kwargs))  # type: ignore[arg-type]


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
        self.invite_ledger: InviteLedger | None = None
        # Sprint 19 Phase D — delayed upload queue (anti-correlation
        # against short-window traffic observers). Populated in
        # :meth:`start` after the dispatcher is ready; torn down in
        # :meth:`stop` ahead of the iroh node so any pending row is
        # drained to the gossip doc before the Endpoint closes.
        self.upload_queue: UploadQueue | None = None
        # Sprint 21 Phase D — quarantine queue (defense-in-depth third
        # leg of the C-DosFlood S21 triangle: rate-limit + PoW +
        # quarantine). Populated in :meth:`start`; torn down in
        # :meth:`stop`. Audit entries (flushed/dropped) survive
        # restart; only pending entries are auto-dropped at TTL.
        self.quarantine_queue: QuarantineQueue | None = None
        # Sprint 20 Phase E.3 — federated warrant canary registry.
        # Aggregates observed canaries + duress acks across the
        # network, exposes freshness via ``GET /api/canary/network-
        # health``. Eagerly instantiated here (no iroh dependency)
        # so the API endpoint works even before :meth:`start` boots
        # the iroh node — useful for inspect-only operator flows.
        self.canary_registry: CanaryRegistry = CanaryRegistry(canary_registry_path())
        # Sprint 22 Phase C — contributor attestation registry
        # (Couche 2). Signed at verified-deploy time in
        # :mod:`nexus_coordinator.api.deploy` and consumed by the
        # daemon loopback proxy at
        # ``GET /api/contributor/verify/...`` for the curator-list
        # Couche 2 governance-strong gate. Eagerly instantiated
        # like :attr:`canary_registry` so the REST endpoint works
        # before :meth:`start` boots the iroh node.
        self.contributor_registry: ContributorRegistry = ContributorRegistry(contributor_registry_path())
        self.apps: dict[str, NexusApp] = {}
        self.app_contexts: dict[str, AppContext] = {}

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

        # Sprint 19 Phase D — spawn the delayed upload queue now
        # that the dispatcher is available as its emit sink. The
        # queue owns its own SQLite file so a heavy rewrite of the
        # table schema never collides with the task_state mirror.
        uq_cfg = self.config.upload_queue
        upload_queue_db = self.project_dir / "upload_queue.sqlite"
        dispatcher = self.dispatcher
        self.upload_queue = UploadQueue(
            db_path=upload_queue_db,
            emit_fn=lambda payload: _dispatcher_emit_adapter(dispatcher, payload),
            mean_jitter_s=uq_cfg.mean_jitter_s,
            max_jitter_s=uq_cfg.max_jitter_s,
            flush_interval_s=uq_cfg.flush_interval_s,
            soft_cap=uq_cfg.soft_cap,
            hard_cap=uq_cfg.hard_cap,
            enabled=uq_cfg.enabled,
        )
        await self.upload_queue.start()

        # Sprint 21 Phase D — quarantine queue. Owns its own SQLite
        # WAL file in the project dir so it never collides with
        # upload_queue.sqlite or state.sqlite. Wire-up from the
        # subscriber gossip path is hors-scope Phase D (carry S22+);
        # for now the queue is exercised by tests + the operator
        # CLI directly.
        qq_cfg = self.config.quarantine_queue
        quarantine_db = self.project_dir / "quarantine.sqlite"
        self.quarantine_queue = QuarantineQueue(
            db_path=quarantine_db,
            ttl_seconds=qq_cfg.ttl_seconds,
            sweep_interval_s=qq_cfg.sweep_interval_s,
        )
        await self.quarantine_queue.start()

        self.validator = Validator(
            doc=self.state.doc,
            node=self.state.node,
            dispatcher=self.dispatcher,
            kudos=self.kudos_ledger,
            db_path=state_db,
        )
        await self.validator.start()

        # 8. Phase C: invite ledger for coordinator-issued invite
        #    tokens (mint / list / revoke). Shares the state.sqlite
        #    file with task_state + kudos_ledger; the table is
        #    created lazily by InviteLedger.init().
        self.invite_ledger = InviteLedger(
            db_path=state_db,
            coord_secret=self._keypair.secret,
        )
        await self.invite_ledger.init()

        # 9. Phase D: discover SDK apps via entry_points and run
        #    each app's on_start with a ComputeClient bound to
        #    this coordinator's local API. The FastAPI factory
        #    mounts /app/{name}/... routes from self.apps; a
        #    failing app is logged and skipped so a single broken
        #    third-party plugin cannot block boot.
        loopback_url = f"http://{self.config.network.api_host}:{self.config.network.api_port}"
        compute = ComputeClient(loopback_url)
        self.apps = {}
        # Sprint 8 Phase A (D1/D2): keep every app's AppContext
        # around so the /app/{name}/tasks/submit and commands
        # routes can delegate back through `ctx.submit_task(...)`
        # / `app.invoke_command(...)` without reconstructing a
        # fresh context per request.
        self.app_contexts: dict[str, AppContext] = {}
        for app in discover_apps():
            try:
                # Sprint 8 Phase B (D3 impl): wire a default
                # AppDatabaseClient pointing at the per-app
                # SQLite file under the project tree. The app
                # may swap this attribute in its on_start hook
                # to point at an external file (that's how
                # nexus-app-gov redirects to the legacy
                # nexus/gov/govdata.db). The parent directory is
                # created lazily so a pristine project doesn't
                # accumulate empty app/*/ subtrees until the app
                # actually writes to its SQLite file.
                default_db_path = app_db_path(self.project_name, app.manifest.name)
                default_db_path.parent.mkdir(parents=True, exist_ok=True)
                # Sprint 9 Phase B (D1 impl): wire a per-app
                # AppStorage at apps/<name>/storage.json. The
                # file is created lazily by AppStorage on the
                # first flush; the parent directory already
                # exists from the AppDatabaseClient mkdir above.
                storage_path = app_storage_path(self.project_name, app.manifest.name)
                # Sprint 9 Phase C (D2 impl): wire a per-app
                # AppEvents bus alongside the storage. The bus is
                # in-process only — events do not survive a
                # coordinator restart and are not replicated
                # cross-node — and the lifespan close in stop()
                # below drains every subscriber gracefully.
                #
                # Sprint 9 Phase D (D4 R6): the default writable
                # AppDatabaseClient is stored in both ``ctx.db``
                # (backward compat) and ``ctx.dbs["default"]``
                # so the migration runner can target it after
                # on_start, even if the app swaps ``ctx.db``.
                default_db = AppDatabaseClient(default_db_path)
                # Sprint 9 Phase E (D3 impl): wire a per-app
                # AppFileStore at apps/<name>/uploads/. The dir
                # is created lazily by AppFileStore.store(); the
                # coordinator only supplies the path and the app
                # name. Apps without @nexus_app_files still get
                # the store wired — the files router checks the
                # decorator separately before accepting uploads.
                uploads_path = app_uploads_path(self.project_name, app.manifest.name)
                ctx = AppContext(
                    compute=compute,
                    project_name=self.project_name,
                    app_name=app.manifest.name,
                    db=default_db,
                    storage=AppStorage(storage_path),
                    events=AppEvents(),
                    files=AppFileStore(uploads_path, app.manifest.name),
                    _app=app,
                )
                await app.on_start(ctx)
                # Sprint 9 Phase D (D4 impl): run the migration
                # runner AFTER on_start (so the app can populate
                # ctx.dbs with additional clients) but BEFORE
                # the dispatcher tick. The runner targets
                # dbs["default"] — the writable per-app SQLite
                # wired above — regardless of what the app did
                # to ctx.db in its on_start hook.
                if app.manifest.migrations_dir is not None:
                    try:
                        mig_runner = MigrationRunner(
                            ctx.dbs["default"],
                            app.manifest.migrations_dir,
                        )
                        mig_applied = await mig_runner.apply()
                        if mig_applied:
                            _log.info(
                                "migrations applied",
                                app=app.manifest.name,
                                count=len(mig_applied),
                                versions=[m.version for m in mig_applied],
                            )
                    except MigrationTamperedError:
                        _log.error(
                            "FATAL: migration tampered, aborting coordinator boot",
                            app=app.manifest.name,
                        )
                        raise
                    except Exception as e:  # noqa: BLE001
                        _log.warning(
                            "migration runner failed, skipping app",
                            app=app.manifest.name,
                            error=str(e),
                        )
                        continue
            except Exception as e:  # noqa: BLE001
                _log.warning(
                    "app on_start failed, skipping",
                    app=app.manifest.name,
                    error=str(e),
                )
                continue
            self.apps[app.manifest.name] = app
            self.app_contexts[app.manifest.name] = ctx
            _log.info(
                "app mounted",
                name=app.manifest.name,
                version=app.manifest.version,
                routes=len(app.routes()),
                workers=len(app.workers()),
                tabs=len(app.tabs()),
                commands=len(app.commands()),
            )

        # 10. Sprint 11 Phase A: auto-publish to the P2P network
        #     if visibility is public. Non-blocking: if the daemon
        #     is not reachable the coordinator still boots normally.
        if self.config.network.visibility == "public":
            await self._auto_publish()

    async def _auto_publish(self) -> None:
        """Announce this project on the P2P network via the daemon.

        Sprint 11 Phase A + Sprint 12 Phase B extension: if the
        project has TabView tabs, pre-render them to HTML, pack
        into a zip, store as blob, and publish a v2 announcement
        with ``archive_hash``. Non-fatal: a daemon that is offline
        or unreachable is logged and ignored.
        """
        from nexus_coordinator.api.daemon import _daemon_base_url, _read_running_state

        state = _read_running_state()
        if state is None:
            _log.warning("auto-publish skipped: shell-daemon not running")
            return

        import httpx

        base_url = _daemon_base_url(state)
        archive_hash: str | None = None

        # Sprint 12 Phase B: pre-render TabView tabs → zip → blob.
        archive_hash = await self._build_and_store_archive(base_url)

        url = f"{base_url}/publish"
        payload: dict[str, object] = {
            "project_name": self.project_name,
            "category": self.config.identity.description or "general",
            "description": self.config.identity.description or self.project_name,
            "apps": list(self.apps.keys()),
        }
        if archive_hash is not None:
            payload["archive_hash"] = archive_hash

        try:
            async with httpx.AsyncClient(timeout=httpx.Timeout(5.0)) as client:
                resp = await client.post(url, json=payload)
            if resp.status_code == 200:
                _log.info(
                    "project published to P2P network",
                    project=self.project_name,
                    archive_hash=archive_hash,
                )
            else:
                _log.warning(
                    "auto-publish returned non-200",
                    status=resp.status_code,
                    body=resp.text,
                )
        except httpx.HTTPError as e:
            _log.warning("auto-publish failed", error=str(e))

    async def _build_and_store_archive(self, daemon_base_url: str) -> str | None:
        """Pre-render TabView tabs to HTML, zip, store as blob.

        Returns the hex hash of the stored blob, or ``None`` if
        no tabs were rendered or the daemon is unreachable.
        """
        import inspect
        import io
        import zipfile

        import httpx
        from nexus_sdk.html_render import render_tabview_to_html
        from nexus_sdk.view import TabView, TabViewV2

        zip_buf = io.BytesIO()
        file_count = 0

        with zipfile.ZipFile(zip_buf, "w", zipfile.ZIP_DEFLATED) as zf:
            for app_name, app in self.apps.items():
                first_tab_name: str | None = None
                for tab in app.tabs():
                    try:
                        if inspect.iscoroutinefunction(tab.fn):
                            descriptor = await tab.fn(app)
                        else:
                            descriptor = tab.fn(app)
                    except Exception as e:  # noqa: BLE001
                        _log.warning(
                            "tab descriptor failed during archive build",
                            app=app_name,
                            tab=tab.name,
                            error=str(e),
                        )
                        continue

                    # Normalize to dict
                    if isinstance(descriptor, (TabView, TabViewV2)):
                        desc_dict = descriptor.model_dump(mode="json")
                    elif isinstance(descriptor, dict):
                        desc_dict = descriptor
                    else:
                        continue

                    html_str = render_tabview_to_html(
                        desc_dict,
                        title=f"{app_name} — {tab.name}",
                    )
                    zf.writestr(f"{app_name}/{tab.name}.html", html_str)
                    file_count += 1

                    if first_tab_name is None:
                        first_tab_name = tab.name

                # index.html per app = redirect to first tab
                if first_tab_name is not None:
                    zf.writestr(
                        f"{app_name}/index.html",
                        f'<meta http-equiv="refresh" content="0;url={first_tab_name}.html">',
                    )
                    file_count += 1

            # Root index.html = redirect to first app
            if self.apps:
                first_app = next(iter(self.apps))
                zf.writestr(
                    "index.html",
                    f'<meta http-equiv="refresh" content="0;url={first_app}/index.html">',
                )
                file_count += 1

        if file_count == 0:
            _log.info("archive build: no tabs to render, skipping")
            return None

        zip_bytes = zip_buf.getvalue()
        _log.info("archive built", size=len(zip_bytes), files=file_count)

        # Store as blob via daemon
        url = f"{daemon_base_url}/publish-blob"
        try:
            async with httpx.AsyncClient(timeout=httpx.Timeout(10.0)) as client:
                resp = await client.post(
                    url,
                    content=zip_bytes,
                    headers={"Content-Type": "application/octet-stream"},
                )
            if resp.status_code == 200:
                hash_hex = resp.json()["hash"]
                _log.info("archive blob stored", hash=hash_hex)
                return hash_hex
            else:
                _log.warning(
                    "publish-blob returned non-200",
                    status=resp.status_code,
                )
                return None
        except httpx.HTTPError as e:
            _log.warning("publish-blob failed", error=str(e))
            return None

    async def stop(self) -> None:
        """Shut down the iroh node and cancel any background tasks.

        Safe to call multiple times; a second call is a no-op.
        """
        if self.apps:
            # Sprint 9 Phase B (D1 lifespan): drain every app's
            # AppStorage before its on_stop hook so any deferred
            # coalesced flush lands on disk. The flush is
            # synchronous under the storage lock — it cancels the
            # outstanding timer and writes the current state.
            # Failures are logged but do not prevent the rest of
            # the teardown from running so a single broken app
            # cannot leak the iroh node.
            for name, ctx in list(self.app_contexts.items()):
                if ctx.storage is None:
                    continue
                try:
                    await ctx.storage.flush_on_shutdown()
                except Exception as e:  # noqa: BLE001
                    _log.warning(
                        "app storage flush_on_shutdown raised",
                        app=name,
                        error=str(e),
                    )
            # Sprint 9 Phase C (D2 lifespan): close every app's
            # AppEvents bus before its on_stop hook so any
            # subscriber currently iterating its receive stream
            # exits the async-for loop on a clean EndOfStream
            # instead of hanging on a vanished sender. Failures
            # are logged but never block the rest of teardown.
            for name, ctx in list(self.app_contexts.items()):
                if ctx.events is None:
                    continue
                try:
                    await ctx.events.aclose()
                except Exception as e:  # noqa: BLE001
                    _log.warning(
                        "app events aclose raised",
                        app=name,
                        error=str(e),
                    )
            for app in list(self.apps.values()):
                try:
                    await app.on_stop()
                except Exception as e:  # noqa: BLE001
                    _log.warning("app on_stop raised", app=app.manifest.name, error=str(e))
            self.apps = {}
            self.app_contexts = {}

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

        # Sprint 19 Phase D — drain the delayed upload queue before
        # we tear down the iroh node. ``drain=True`` force-flushes
        # every pending row regardless of ``deliver_at`` so a
        # user-facing submit that was still in jitter land at
        # shutdown does not silently disappear.
        if self.upload_queue is not None:
            try:
                await self.upload_queue.shutdown(drain=True)
            except Exception as e:  # noqa: BLE001 — best-effort
                _log.warning("upload queue shutdown raised", error=str(e))
            self.upload_queue = None

        # Sprint 21 Phase D — stop the quarantine sweep loop. The
        # SQLite file is left intact: audit entries (flushed /
        # dropped) survive the restart so an operator can review
        # past decisions.
        if self.quarantine_queue is not None:
            try:
                await self.quarantine_queue.shutdown()
            except Exception as e:  # noqa: BLE001 — best-effort
                _log.warning("quarantine queue shutdown raised", error=str(e))
            self.quarantine_queue = None

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
