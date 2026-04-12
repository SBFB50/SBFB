# SPDX-License-Identifier: AGPL-3.0-or-later
"""GovApp — nexus-grid port of the legacy government monitoring stack.

Sprint 4 Phase D shipped a minimal stub (one route, one worker,
one tab). Sprint 8 grows that stub to **nineteen tabs** across
three batches:

Phase B — Batch 1 (read-only browse of the core gov schema)
    - **Dashboard**  — aggregate counts across the gov tables
    - **Politiciens** — list of politicians (paginated to 50)
    - **Politicien** — detail view for the first politician
    - **Biographie** — mandates + party memberships chronology
    - **Positions** — recent positions across all politicians
    - **Sujets** — aggregate of ``gov_positions.subject`` by count

Phase C — Batch 2 (operational + content tabs)
    - **Contradictions** — real TabView replacing the Sprint 4
      stub: table joined with politician names, per-subject
      chart_bar, summary metrics (total / high severity /
      verified)
    - **Scan** — most recent ``gov_scan_log`` rows
    - **Workers** — ``gov_scan_log`` aggregated by ``scan_type``
      (one row per legacy worker with its run stats)
    - **Pipeline** — ETL snapshot (status distribution,
      currently running scans with their current_phase, recent
      tail)
    - **Social** — recent ``gov_social_posts`` joined with
      politician names + platform breakdown
    - **Press** — recent ``gov_press`` entries with sentiment
    - **Transcriptions** — recent ``gov_transcriptions`` entries
      joined with politician names

Phase D — Batch 3 (alerting + archives + RAG)
    - **Alertes** — recent ``gov_alerts`` with severity chart
      and three summary metrics (total / unread / high severity)
    - **Affaires** — recent ``gov_affairs`` joined with
      politician names with a status chart
    - **Lois** — recent ``gov_laws`` with status chart and
      summary metrics (total / promulgated / average duration)
    - **Factchecks** — recent ``gov_factchecks`` joined with
      politician names with a rating chart
    - **Recherche** — RAG semantic search button that dispatches
      an example query to the new ``rag_search`` worker via
      :meth:`nexus_sdk.AppContext.submit_task`
    - **Question** — RAG question-answering button that
      dispatches a sample question to the ``rag_ask`` worker

Phase D also registers two new ``@nexus_worker`` handlers
(``rag_search`` and ``rag_ask``) that act as smoke handlers for
the ``task_submit`` action wired from the TabView button blocks.

Data plumbing
-------------

All tabs read through :attr:`nexus_sdk.AppContext.db` — an
:class:`nexus_sdk.AppDatabaseClient` wired by the coordinator
loader. By default the client points at a per-app SQLite file
under the coordinator's project tree; :meth:`on_start` checks
whether the legacy ``nexus/gov/govdata.db`` exists and swaps the
client over to it in that case. If the legacy DB is missing (a
fresh install without a prior scrape run), every tab handler
falls back to an empty TabView state.

Every query lives in :mod:`nexus_app_gov.queries` so the tab
handler code is small — the handler's job is to shape a
:class:`nexus_sdk.view.TabView` around the query output, not to
wrangle SQL.
"""

from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from nexus_sdk import (
    AppContext,
    AppDatabaseClient,
    AppManifest,
    DatabaseError,
    NexusApp,
    button_task,
    nexus_app_files,
    nexus_command,
    nexus_route,
    nexus_tab,
    nexus_worker,
)
from nexus_sdk.view import (
    TabBlock,
    TabView,
    TabViewV2,
    chart_bar,
    empty,
    file_upload_block,
    heading,
    kv,
    metric,
    section,
    table_,
    text,
)

from nexus_app_gov import queries
from nexus_app_gov.filters import PoliticiansFilter
from nexus_app_gov.prompts import (
    POLITICAL_CONTRADICTION_PROMPT,
    RAG_ASK_PROMPT,
    RAG_SEARCH_PROMPT,
)


def _truncate(value: str, max_len: int) -> str:
    """Clip ``value`` to ``max_len`` characters with an ellipsis
    suffix when clipped. Used by Phase C tabs to keep long
    descriptions and social posts readable in the tabular view
    without pushing the shell's renderer to overflow."""
    if value is None:
        return ""
    if len(value) <= max_len:
        return value
    return value[: max(0, max_len - 1)].rstrip() + "…"


def _legacy_govdata_db_path() -> Path:
    """Resolve ``<repo-root>/nexus/gov/govdata.db``.

    The gov app source file lives at
    ``packages/nexus-app-gov/src/nexus_app_gov/app.py``. Walking
    four parents lands on the repo root, where the legacy gov
    package sits under ``nexus/gov/``. The actual SQLite file is
    populated by the legacy scrape workers — if it does not
    exist at boot time, :meth:`GovApp.on_start` leaves the
    loader's default per-app SQLite in place.
    """
    return Path(__file__).resolve().parents[4] / "nexus" / "gov" / "govdata.db"


@nexus_app_files(accept=["image/*", "application/pdf"])
class GovApp(NexusApp):
    """Political monitoring — Sprint 8 Phase B gov migration Batch 1."""

    manifest = AppManifest(
        name="gov",
        version="0.3.0",
        author="FlowUP",
        description="Government monitoring: politicians, positions, contradictions, "
        "laws, affairs, factchecks, alerts and RAG search across the legacy gov "
        "SQLite schema.",
        license="AGPL-3.0",
        migrations_dir=Path(__file__).parent / "migrations",
    )

    def __init__(self) -> None:
        super().__init__()
        self._ctx: AppContext | None = None

    async def on_start(self, ctx: AppContext) -> None:
        """Wire the two-client DB model, register typed storage
        namespaces, then keep the context for later tab handlers.

        The coordinator loader pre-wires a default writable
        :class:`AppDatabaseClient` at
        ``<project>/apps/gov/app.sqlite`` and stores it in
        ``ctx.dbs["default"]``. This hook adds:

        - ``ctx.db`` → legacy ``govdata.db`` (read-only) when the
          file exists, so all 19 tab handlers that do SELECT-only
          queries keep working unchanged.
        - ``ctx.dbs["gov"]`` → same as ``ctx.db`` (named alias
          for the legacy read-only client).
        - ``ctx.dbs["app"]`` → the writable ``app.sqlite`` from
          ``ctx.dbs["default"]`` (the migration runner targets
          this client to apply ``001_documents.sql`` etc.).

        Sprint 9 Phase B (D1 consumer): registers the
        ``politicians_filter`` typed namespace on
        ``ctx.namespaces`` so the coordinator's generic
        ``POST /app/gov/state/politicians_filter`` route can
        validate writes against :class:`PoliticiansFilter`
        without the coord having to import the schema.
        """
        legacy = _legacy_govdata_db_path()
        if legacy.exists():
            ctx.db = AppDatabaseClient(legacy, read_only=True)
        # Sprint 9 Phase D (D4 R6 consumer): named DB aliases.
        # "gov" is the read-only legacy client (or a fallback if
        # the legacy file is missing and ctx.db was left as None).
        # "app" is the writable per-app sqlite wired by the
        # coordinator loader as dbs["default"]; when running in
        # unit tests that skip the loader, "default" may be absent.
        if ctx.db is not None:
            ctx.dbs["gov"] = ctx.db
        if "default" in ctx.dbs:
            ctx.dbs["app"] = ctx.dbs["default"]
        if ctx.storage is not None:
            ctx.namespaces["politicians_filter"] = ctx.storage.namespace("filters.politicians", PoliticiansFilter)
        self._ctx = ctx

    async def on_stop(self) -> None:
        self._ctx = None

    # ------------------------------------------------------------------
    # Routes (legacy Sprint 4 stub retained for regression)
    # ------------------------------------------------------------------

    @nexus_route("/statements", methods=["GET"])
    async def list_statements(self) -> dict[str, Any]:
        """Tiny placeholder route kept from Sprint 4 so the
        coordinator's manifest still advertises a concrete URL
        under ``/app/gov/``. Real routes land in Sprint 9 when
        the mutation path is opened up."""
        return {
            "app": "gov",
            "status": "ready",
            "prompt_template": POLITICAL_CONTRADICTION_PROMPT.splitlines()[0],
        }

    # ------------------------------------------------------------------
    # Workers (legacy Sprint 4 stub retained for regression)
    # ------------------------------------------------------------------

    @nexus_worker(name="contradiction_detector", model="stub-model:latest")
    async def contradiction_detector(self, ctx: AppContext) -> dict[str, Any]:
        """Submit a contradiction-detection task via the
        coordinator's ``/tasks/submit`` endpoint. Retained from
        Sprint 4 so the existing regression baseline tests keep
        passing; Phase D adds the RAG search/ask workers below."""
        task = await ctx.compute.submit_task(
            task_type="contradiction_check",
            prompt=POLITICAL_CONTRADICTION_PROMPT.format(statements="(example)"),
            model="stub-model:latest",
            priority=5,
        )
        return {"task_id": task.task_id}

    # ------------------------------------------------------------------
    # Sprint 8 Phase D — RAG workers (rag_search + rag_ask)
    # ------------------------------------------------------------------

    @nexus_worker(name="rag_search", model="nomic-embed-text")
    async def rag_search(self, ctx: AppContext) -> dict[str, Any]:
        """Smoke handler for the ``gov.rag_search`` routing key.

        The Search tab's ``task_submit`` button forwards its
        payload through the coordinator's
        ``POST /app/gov/tasks/submit`` route, which calls into
        :meth:`nexus_sdk.AppContext.submit_task` — that path
        resolves the worker by name, looks up the declared model
        here (``nomic-embed-text``) and forwards the prompt to
        the worker daemon via
        :meth:`nexus_sdk.ComputeClient.submit_task`.

        This handler is an admin smoke entry point only. Calling
        it directly (e.g. from a test) submits an example task
        using :data:`nexus_app_gov.prompts.RAG_SEARCH_PROMPT` so
        the full compute plumbing can be exercised end-to-end
        without a live Search tab click.
        """
        task = await ctx.compute.submit_task(
            task_type="gov.rag_search",
            prompt=RAG_SEARCH_PROMPT.format(query="(example)"),
            model="nomic-embed-text",
            priority=5,
        )
        return {"task_id": task.task_id}

    # ------------------------------------------------------------------
    # Sprint 9 Phase C — D2 consumer (refresh_party_cache + party.refreshed)
    # ------------------------------------------------------------------

    @nexus_worker(name="refresh_party_cache", model="stub-model:latest")
    async def refresh_party_cache(self, ctx: AppContext) -> dict[str, Any]:
        """Re-read the legacy ``gov_parties`` table and announce
        the new count on :attr:`AppContext.events`.

        Sprint 9 Phase C (D2 consumer): the worker is the
        canonical "publisher" half of the AppContext.events
        story. The Politiciens tab subscribes to ``party.refreshed``
        through the SSE bridge and React Query invalidation,
        so a manual `gov.refresh_party_cache` invocation
        re-renders the live grid without a page reload.

        The handler is intentionally tiny: counting parties is
        cheap, the bus is in-process, and the only state the
        worker mutates is the publish-side of the bus. A
        missing ``gov_parties`` table degrades to ``count=0``
        rather than raising — the consumer side renders an
        empty grid in that case.
        """
        db = ctx.db
        count = 0
        if db is not None:
            try:
                row = await db.fetchone("SELECT COUNT(*) AS n FROM gov_parties")
            except DatabaseError:
                row = None
            if row is not None:
                count = int(row["n"])
        payload = {"count": count, "refreshed_at": datetime.now(timezone.utc).isoformat()}
        if ctx.events is not None:
            await ctx.events.publish("party.refreshed", payload)
        return payload

    @nexus_worker(
        name="rag_ask",
        model="juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m",
    )
    async def rag_ask(self, ctx: AppContext) -> dict[str, Any]:
        """Smoke handler for the ``gov.rag_ask`` routing key.

        Mirrors :meth:`rag_search` for open-ended question
        answering: the coordinator resolves the worker name,
        the declared model (``juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m``)
        flows into the ``/tasks/submit`` body, and the worker
        daemon routes the prompt to the local Ollama instance.
        Direct invocation submits an example question built
        from :data:`nexus_app_gov.prompts.RAG_ASK_PROMPT`.
        """
        task = await ctx.compute.submit_task(
            task_type="gov.rag_ask",
            prompt=RAG_ASK_PROMPT.format(question="(example)"),
            model="juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m",
            priority=5,
        )
        return {"task_id": task.task_id}

    # ------------------------------------------------------------------
    # Helpers for the new Batch 1 tabs
    # ------------------------------------------------------------------

    def _require_db(self) -> AppDatabaseClient | None:
        """Return the live ``AppDatabaseClient`` or ``None``.

        Tab handlers call this first; if ``on_start`` was never
        called (a pathological test scenario) the handler falls
        back to an empty TabView state.
        """
        if self._ctx is None:
            return None
        return self._ctx.db

    def _empty_tab(self, tab_name: str, title: str, reason: str) -> dict[str, Any]:
        """Render a uniform empty state when the DB is missing
        or a table is absent. Factored so every Batch 1 tab
        produces the same fallback shape — a single heading + an
        explanatory empty block.

        The TabView renderer treats a non-``None`` ``title`` as
        a top-level ``<h2>``, which would duplicate the heading
        block we emit below. We leave ``title`` at its default
        (``None``) so the heading block is the single source of
        the visual title.
        """
        return TabView(
            tab_name=tab_name,
            blocks=[
                heading(level=1, text=title),
                empty(text=reason),
            ],
        ).model_dump()

    # ------------------------------------------------------------------
    # Sprint 8 Phase C — Contradictions (upgrade of the Sprint 4 stub)
    # ------------------------------------------------------------------

    @nexus_tab(name="Contradictions", icon="alert-octagon")
    async def contradictions_tab(self) -> dict[str, Any]:
        """Table of contradictions joined with politician names,
        a ``chart_bar`` of contradictions per subject, and the
        three summary metrics (total / high severity / verified).

        Replaces the Sprint 4 placeholder — the tab is still
        called ``Contradictions`` and returns a
        ``tab_name="contradictions"`` TabView so the route
        ``/app/gov/tabs/Contradictions/descriptor`` remains
        stable. Falls back to the shared empty-state block when
        the legacy DB is absent or empty.
        """
        db = self._require_db()
        title = "Détection de contradictions"
        if db is None:
            return self._empty_tab("contradictions", title, "Base Gov indisponible.")

        payload = await queries.contradictions_overview_query(db, limit=50)
        rows = payload["rows"]
        by_subject = payload["by_subject"]
        summary = payload["summary"]

        if not rows and summary["total"] == 0:
            return self._empty_tab(
                "contradictions",
                title,
                "Aucune contradiction détectée — lancer l'analyseur pour alimenter la table.",
            )

        blocks: list[TabBlock] = [
            heading(level=1, text=title),
            text(
                text=POLITICAL_CONTRADICTION_PROMPT.splitlines()[0],
                muted=True,
            ),
            section(
                title="Résumé",
                blocks=[
                    metric(
                        label="Contradictions détectées",
                        value=summary["total"],
                        tone="warn",
                    ),
                    metric(
                        label="Sévérité haute",
                        value=summary["high"],
                        tone="danger" if summary["high"] > 0 else "neutral",
                    ),
                    metric(
                        label="Sources vérifiées",
                        value=summary["verified"],
                        tone="ok",
                    ),
                ],
            ),
        ]

        if by_subject:
            blocks.append(
                chart_bar(
                    label="Contradictions par sujet",
                    bars=[{"label": str(row["subject"]), "value": int(row["count"])} for row in by_subject],
                )
            )

        blocks.append(
            table_(
                columns=[
                    {"key": "politician_name", "label": "Politicien"},
                    {"key": "subject", "label": "Sujet"},
                    {"key": "severity", "label": "Sévérité"},
                    {"key": "description", "label": "Description"},
                    {"key": "detected_at", "label": "Détectée"},
                ],
                rows=[
                    {
                        "politician_name": str(row.get("politician_name") or "—"),
                        "subject": str(row.get("subject") or "—"),
                        "severity": str(row.get("severity") or "—"),
                        "description": _truncate(str(row.get("description") or ""), 140),
                        "detected_at": str(row.get("detected_at") or "—"),
                    }
                    for row in rows
                ],
                empty_text="Aucune contradiction.",
            )
        )

        return TabView(tab_name="contradictions", blocks=blocks).model_dump()

    # ------------------------------------------------------------------
    # Sprint 8 Phase B — Batch 1 tabs
    # ------------------------------------------------------------------

    @nexus_tab(name="Dashboard", icon="activity")
    async def dashboard_tab(self) -> dict[str, Any]:
        """Aggregate counts across the gov tables — the entry
        point for every Gov session."""
        db = self._require_db()
        title = "Tableau de bord Gov"
        if db is None:
            return self._empty_tab("dashboard", title, "Base Gov indisponible.")

        stats = await queries.dashboard_stats_query(db)

        blocks: list[TabBlock] = [
            heading(level=1, text=title),
            text(
                text="Aperçu des tables du socle Gov — politiciens, positions et contradictions.",
                muted=True,
            ),
            section(
                title="Inventaire",
                blocks=[
                    metric(label="Politiciens", value=stats["politicians"], tone="neutral"),
                    metric(
                        label="Politiciens actifs",
                        value=stats["active_politicians"],
                        tone="ok",
                    ),
                    metric(label="Positions", value=stats["positions"]),
                    metric(
                        label="Contradictions",
                        value=stats["contradictions"],
                        tone="warn",
                    ),
                    metric(label="Partis", value=stats["parties"]),
                ],
            ),
        ]

        top_subjects = stats.get("top_subjects") or []
        if top_subjects:
            blocks.append(
                chart_bar(
                    label="Sujets les plus débattus",
                    bars=[{"label": str(row["subject"]), "value": int(row["n"])} for row in top_subjects],
                )
            )
        else:
            blocks.append(empty(text="Aucun sujet indexé — le scrape initial n'a encore rien produit."))

        return TabView(tab_name="dashboard", blocks=blocks).model_dump()

    async def _load_politicians_filter(self) -> PoliticiansFilter:
        """Read the persisted Politiciens tab filter or return a
        fresh empty filter when none has been written yet.

        Sprint 9 Phase B (D1 consumer). The filter is stored at
        the ``filters.politicians`` key inside the per-app
        :class:`nexus_sdk.AppStorage` (registered as the
        ``politicians_filter`` typed namespace in
        :meth:`on_start`). A missing context, missing storage or
        missing key all collapse to an empty default — the
        Sprint 10 audit gate is the place to flag any of these
        as a bug, not the tab handler.
        """
        if self._ctx is None or self._ctx.storage is None:
            return PoliticiansFilter()
        ns = self._ctx.namespaces.get("politicians_filter")
        if ns is None:
            return PoliticiansFilter()
        return await ns.get(default=PoliticiansFilter())

    def _format_filter_summary(self, filt: PoliticiansFilter) -> str:
        """Render the active filter as a one-line French summary
        for the Politiciens tab descriptor.

        Empty filters become ``"Filtres : aucun"``; populated
        filters list each non-empty field as
        ``"Chambre = Assemblée · Recherche = Dupont"``. The
        Playwright spec greps the rendered summary to assert the
        filter survived a page reload, so the format is part of
        the public surface and changes here must update
        ``web/tests/gov-politicians-filter-persist.spec.ts``.
        """
        parts: list[str] = []
        if filt.chamber:
            parts.append(f"Chambre = {filt.chamber}")
        if filt.date_range is not None:
            start, end = filt.date_range
            parts.append(f"Période = {start.isoformat()}…{end.isoformat()}")
        if filt.search:
            parts.append(f"Recherche = {filt.search}")
        if not parts:
            return "Filtres : aucun"
        return "Filtres : " + " · ".join(parts)

    @nexus_tab(name="Politiciens", icon="users")
    async def politicians_tab(self) -> dict[str, Any]:
        """Paginated table of politicians (50 rows max).

        Sprint 9 Phase B (D1 consumer): reads the persisted
        :class:`PoliticiansFilter` from
        ``ctx.storage.namespace("filters.politicians", PoliticiansFilter)``
        and applies the chamber + search components to the
        underlying SQL query. The current filter state is also
        rendered as a muted text block at the top of the
        descriptor so the Playwright filter-persist spec can
        assert the filter survived a page reload without tying
        the test to internal storage paths.
        """
        db = self._require_db()
        title = "Politiciens"
        if db is None:
            return self._empty_tab("politiciens", title, "Base Gov indisponible.")

        active_filter = await self._load_politicians_filter()
        filter_summary = self._format_filter_summary(active_filter)

        rows = await queries.politicians_list_query(
            db,
            limit=50,
            chamber=active_filter.chamber,
            search=active_filter.search,
        )
        if not rows:
            return TabView(
                tab_name="politiciens",
                blocks=[
                    heading(level=1, text=title),
                    text(text=filter_summary, muted=True),
                    empty(text=("Aucun politicien référencé. Lancer un scrape pour peupler la base.")),
                ],
            ).model_dump()

        columns = [
            {"key": "name", "label": "Nom"},
            {"key": "chamber", "label": "Chambre"},
            {"key": "party", "label": "Parti"},
            {"key": "role", "label": "Rôle"},
            {"key": "constituency", "label": "Circonscription"},
            {"key": "active", "label": "Actif", "align": "right"},
        ]
        table_rows = [
            {
                "name": str(row.get("name") or "—"),
                "chamber": str(row.get("chamber") or "—"),
                "party": str(row.get("party") or "—"),
                "role": str(row.get("role") or "—"),
                "constituency": str(row.get("constituency") or "—"),
                "active": int(row.get("active") or 0),
            }
            for row in rows
        ]

        return TabView(
            tab_name="politiciens",
            blocks=[
                heading(level=1, text=title),
                text(
                    text=f"{len(rows)} politiciens listés (max 50).",
                    muted=True,
                ),
                text(text=filter_summary, muted=True),
                table_(columns=columns, rows=table_rows, empty_text="Aucun résultat."),
            ],
        ).model_dump()

    @nexus_tab(name="Politicien", icon="user")
    async def politician_detail_tab(self) -> dict[str, Any]:
        """Fiche du premier politicien (sélecteur Sprint 9)."""
        db = self._require_db()
        title = "Fiche politicien"
        if db is None:
            return self._empty_tab("politicien", title, "Base Gov indisponible.")

        detail = await queries.politician_detail_query(db)
        if detail is None:
            return self._empty_tab(
                "politicien",
                title,
                "Aucun politicien dans la base — lancer un scrape pour commencer.",
            )

        pol = detail["politician"]
        pol_name = str(pol.get("name") or "Inconnu")
        blocks: list[TabBlock] = [
            heading(level=1, text=pol_name),
            text(
                text="Aperçu Sprint 8 — sélecteur politicien ajouté en Sprint 9.",
                muted=True,
            ),
            kv(
                items=[
                    {"label": "Chambre", "value": str(pol.get("chamber") or "—")},
                    {"label": "Parti", "value": str(pol.get("party") or "—")},
                    {"label": "Rôle", "value": str(pol.get("role") or "—")},
                    {"label": "Circonscription", "value": str(pol.get("constituency") or "—")},
                    {"label": "Actif", "value": int(pol.get("active") or 0)},
                ]
            ),
            metric(
                label="Contradictions",
                value=detail.get("contradictions_count", 0),
                tone="warn",
            ),
        ]

        positions = detail.get("recent_positions") or []
        if positions:
            blocks.append(
                section(
                    title="Positions récentes",
                    blocks=[
                        table_(
                            columns=[
                                {"key": "subject", "label": "Sujet"},
                                {"key": "position_type", "label": "Type"},
                                {"key": "stance", "label": "Stance"},
                                {"key": "date", "label": "Date"},
                            ],
                            rows=[
                                {
                                    "subject": str(p.get("subject") or "—"),
                                    "position_type": str(p.get("position_type") or "—"),
                                    "stance": str(p.get("stance") or "—"),
                                    "date": str(p.get("date") or "—"),
                                }
                                for p in positions
                            ],
                            empty_text="Aucune position enregistrée.",
                        )
                    ],
                )
            )
        else:
            blocks.append(empty(text=f"Aucune position enregistrée pour {pol_name}."))

        return TabView(tab_name="politicien", blocks=blocks).model_dump()

    @nexus_tab(name="Biographie", icon="book-open")
    async def biography_tab(self) -> dict[str, Any]:
        """Chronologie mandats + appartenances partisanes."""
        db = self._require_db()
        title = "Biographie"
        if db is None:
            return self._empty_tab("biographie", title, "Base Gov indisponible.")

        bio = await queries.biography_query(db)
        if bio is None:
            return self._empty_tab(
                "biographie",
                title,
                "Aucun politicien dans la base — biographie non disponible.",
            )

        pol = bio["politician"]
        pol_name = str(pol.get("name") or "Inconnu")
        blocks: list[TabBlock] = [
            heading(level=1, text=f"Biographie — {pol_name}"),
            text(
                text="Aperçu Sprint 8 — sélecteur politicien ajouté en Sprint 9.",
                muted=True,
            ),
        ]

        mandates = bio.get("mandates") or []
        if mandates:
            blocks.append(
                section(
                    title="Mandats",
                    blocks=[
                        table_(
                            columns=[
                                {"key": "type", "label": "Type"},
                                {"key": "title", "label": "Titre"},
                                {"key": "institution", "label": "Institution"},
                                {"key": "start_date", "label": "Début"},
                                {"key": "end_date", "label": "Fin"},
                                {"key": "is_current", "label": "En cours", "align": "right"},
                            ],
                            rows=[
                                {
                                    "type": str(m.get("type") or "—"),
                                    "title": str(m.get("title") or "—"),
                                    "institution": str(m.get("institution") or "—"),
                                    "start_date": str(m.get("start_date") or "—"),
                                    "end_date": str(m.get("end_date") or "—"),
                                    "is_current": int(m.get("is_current") or 0),
                                }
                                for m in mandates
                            ],
                            empty_text="Aucun mandat connu.",
                        )
                    ],
                )
            )
        else:
            blocks.append(empty(text="Aucun mandat connu."))

        memberships = bio.get("party_memberships") or []
        if memberships:
            blocks.append(
                section(
                    title="Appartenances partisanes",
                    blocks=[
                        table_(
                            columns=[
                                {"key": "party_name", "label": "Parti"},
                                {"key": "short_name", "label": "Sigle"},
                                {"key": "start_date", "label": "Début"},
                                {"key": "end_date", "label": "Fin"},
                                {"key": "is_current", "label": "En cours", "align": "right"},
                            ],
                            rows=[
                                {
                                    "party_name": str(m.get("party_name") or "—"),
                                    "short_name": str(m.get("short_name") or "—"),
                                    "start_date": str(m.get("start_date") or "—"),
                                    "end_date": str(m.get("end_date") or "—"),
                                    "is_current": int(m.get("is_current") or 0),
                                }
                                for m in memberships
                            ],
                            empty_text="Aucune appartenance partisane connue.",
                        )
                    ],
                )
            )

        return TabView(tab_name="biographie", blocks=blocks).model_dump()

    @nexus_tab(name="Positions", icon="list")
    async def positions_tab(self) -> dict[str, Any]:
        """Liste des 50 positions les plus récentes (toutes chambres)."""
        db = self._require_db()
        title = "Positions récentes"
        if db is None:
            return self._empty_tab("positions", title, "Base Gov indisponible.")

        rows = await queries.positions_list_query(db, limit=50)
        if not rows:
            return self._empty_tab(
                "positions",
                title,
                "Aucune position enregistrée. Lancer un scrape des positions pour commencer.",
            )

        return TabView(
            tab_name="positions",
            blocks=[
                heading(level=1, text=title),
                text(text=f"{len(rows)} positions listées (50 max).", muted=True),
                table_(
                    columns=[
                        {"key": "politician_name", "label": "Politicien"},
                        {"key": "subject", "label": "Sujet"},
                        {"key": "position_type", "label": "Type"},
                        {"key": "stance", "label": "Stance"},
                        {"key": "date", "label": "Date"},
                    ],
                    rows=[
                        {
                            "politician_name": str(r.get("politician_name") or "—"),
                            "subject": str(r.get("subject") or "—"),
                            "position_type": str(r.get("position_type") or "—"),
                            "stance": str(r.get("stance") or "—"),
                            "date": str(r.get("date") or "—"),
                        }
                        for r in rows
                    ],
                    empty_text="Aucune position.",
                ),
            ],
        ).model_dump()

    # ------------------------------------------------------------------
    # Sprint 8 Phase C — Batch 2 tabs (Scan/Workers/Pipeline/Social/
    # Press/Transcriptions). Contradictions lives above under its
    # Sprint 4 legacy section, rewritten in-place.
    # ------------------------------------------------------------------

    @nexus_tab(name="Scan", icon="radar")
    async def scan_tab(self) -> dict[str, Any]:
        """List the 50 most recent ``gov_scan_log`` rows."""
        db = self._require_db()
        title = "Journal des scans"
        if db is None:
            return self._empty_tab("scan", title, "Base Gov indisponible.")

        rows = await queries.scan_log_recent_query(db, limit=50)
        if not rows:
            return self._empty_tab(
                "scan",
                title,
                "Aucun scan enregistré. Le journal sera peuplé dès qu'un worker legacy démarrera.",
            )

        return TabView(
            tab_name="scan",
            blocks=[
                heading(level=1, text=title),
                text(
                    text=f"{len(rows)} scans listés (50 max).",
                    muted=True,
                ),
                table_(
                    columns=[
                        {"key": "scan_type", "label": "Type"},
                        {"key": "status", "label": "Statut"},
                        {"key": "items_found", "label": "Trouvés", "align": "right"},
                        {"key": "items_new", "label": "Nouveaux", "align": "right"},
                        {"key": "started_at", "label": "Démarré"},
                        {"key": "completed_at", "label": "Terminé"},
                    ],
                    rows=[
                        {
                            "scan_type": str(r.get("scan_type") or "—"),
                            "status": str(r.get("status") or "—"),
                            "items_found": int(r.get("items_found") or 0),
                            "items_new": int(r.get("items_new") or 0),
                            "started_at": str(r.get("started_at") or "—"),
                            "completed_at": str(r.get("completed_at") or "—"),
                        }
                        for r in rows
                    ],
                    empty_text="Aucun scan.",
                ),
            ],
        ).model_dump()

    @nexus_tab(name="Workers", icon="cpu")
    async def workers_tab(self) -> dict[str, Any]:
        """``gov_scan_log`` aggregated by ``scan_type`` — one row
        per legacy worker module with its run statistics."""
        db = self._require_db()
        title = "Workers legacy"
        if db is None:
            return self._empty_tab("workers", title, "Base Gov indisponible.")

        rows = await queries.workers_state_query(db)
        if not rows:
            return self._empty_tab(
                "workers",
                title,
                "Aucun worker n'a encore produit de scan. Lancer un scrape pour peupler le journal.",
            )

        total_workers = len(rows)
        failing = sum(1 for r in rows if int(r.get("failures") or 0) > 0)
        idle = sum(1 for r in rows if (r.get("last_status") or "") != "running")

        return TabView(
            tab_name="workers",
            blocks=[
                heading(level=1, text=title),
                text(
                    text="Agrégation ``gov_scan_log`` par ``scan_type``. "
                    "Le dernier statut reflète le dernier run observé.",
                    muted=True,
                ),
                section(
                    title="Synthèse",
                    blocks=[
                        metric(label="Workers distincts", value=total_workers),
                        metric(
                            label="Avec échecs",
                            value=failing,
                            tone="warn" if failing > 0 else "neutral",
                        ),
                        metric(label="Au repos", value=idle),
                    ],
                ),
                table_(
                    columns=[
                        {"key": "scan_type", "label": "Worker"},
                        {"key": "total_runs", "label": "Runs", "align": "right"},
                        {"key": "successes", "label": "Succès", "align": "right"},
                        {"key": "failures", "label": "Échecs", "align": "right"},
                        {"key": "last_status", "label": "Dernier statut"},
                        {"key": "last_run", "label": "Dernier run"},
                        {"key": "items_new_total", "label": "Nouveaux cumul.", "align": "right"},
                    ],
                    rows=[
                        {
                            "scan_type": str(r.get("scan_type") or "—"),
                            "total_runs": int(r.get("total_runs") or 0),
                            "successes": int(r.get("successes") or 0),
                            "failures": int(r.get("failures") or 0),
                            "last_status": str(r.get("last_status") or "—"),
                            "last_run": str(r.get("last_run") or "—"),
                            "items_new_total": int(r.get("items_new_total") or 0),
                        }
                        for r in rows
                    ],
                    empty_text="Aucun worker.",
                ),
            ],
        ).model_dump()

    @nexus_tab(name="Pipeline", icon="workflow")
    async def pipeline_tab(self) -> dict[str, Any]:
        """Snapshot of the ETL pipeline: status distribution,
        scans currently ``running`` with their ``current_phase``,
        and a chronological tail."""
        db = self._require_db()
        title = "Pipeline ETL"
        if db is None:
            return self._empty_tab("pipeline", title, "Base Gov indisponible.")

        payload = await queries.pipeline_state_query(db)
        status_counts = payload["status_counts"]
        running = payload["running"]
        recent = payload["recent"]

        if not status_counts and not recent:
            return self._empty_tab(
                "pipeline",
                title,
                "Aucune activité pipeline enregistrée. Lancer un scan pour voir le pipeline en action.",
            )

        blocks: list[TabBlock] = [
            heading(level=1, text=title),
            text(
                text="Distribution des statuts de scan et suivi des scans en cours.",
                muted=True,
            ),
        ]

        if status_counts:
            blocks.append(
                chart_bar(
                    label="Distribution des statuts",
                    bars=[{"label": str(row["status"]), "value": int(row["count"])} for row in status_counts],
                )
            )

        running_rows = [
            {
                "scan_type": str(r.get("scan_type") or "—"),
                "current_phase": str(r.get("current_phase") or "—"),
                "phase_offset": int(r.get("phase_offset") or 0),
                "items_found": int(r.get("items_found") or 0),
                "items_new": int(r.get("items_new") or 0),
                "started_at": str(r.get("started_at") or "—"),
            }
            for r in running
        ]
        blocks.append(
            section(
                title="En cours",
                blocks=[
                    table_(
                        columns=[
                            {"key": "scan_type", "label": "Worker"},
                            {"key": "current_phase", "label": "Phase"},
                            {"key": "phase_offset", "label": "Offset", "align": "right"},
                            {"key": "items_found", "label": "Trouvés", "align": "right"},
                            {"key": "items_new", "label": "Nouveaux", "align": "right"},
                            {"key": "started_at", "label": "Démarré"},
                        ],
                        rows=running_rows,
                        empty_text="Aucun scan en cours.",
                    )
                ],
            )
        )

        recent_rows = [
            {
                "scan_type": str(r.get("scan_type") or "—"),
                "status": str(r.get("status") or "—"),
                "items_new": int(r.get("items_new") or 0),
                "started_at": str(r.get("started_at") or "—"),
                "completed_at": str(r.get("completed_at") or "—"),
            }
            for r in recent
        ]
        blocks.append(
            section(
                title="Historique récent",
                blocks=[
                    table_(
                        columns=[
                            {"key": "scan_type", "label": "Worker"},
                            {"key": "status", "label": "Statut"},
                            {"key": "items_new", "label": "Nouveaux", "align": "right"},
                            {"key": "started_at", "label": "Démarré"},
                            {"key": "completed_at", "label": "Terminé"},
                        ],
                        rows=recent_rows,
                        empty_text="Aucun historique.",
                    )
                ],
            )
        )

        return TabView(tab_name="pipeline", blocks=blocks).model_dump()

    @nexus_tab(name="Social", icon="share-2")
    async def social_tab(self) -> dict[str, Any]:
        """Recent social posts with per-platform chart_bar."""
        db = self._require_db()
        title = "Posts sociaux"
        if db is None:
            return self._empty_tab("social", title, "Base Gov indisponible.")

        rows = await queries.social_posts_query(db, limit=50)
        platforms = await queries.social_platform_breakdown_query(db)

        if not rows:
            return self._empty_tab(
                "social",
                title,
                "Aucun post social enregistré. Lancer un scrape réseau social pour alimenter la liste.",
            )

        blocks: list[TabBlock] = [
            heading(level=1, text=title),
            text(
                text=f"{len(rows)} posts listés (50 max).",
                muted=True,
            ),
        ]

        if platforms:
            blocks.append(
                chart_bar(
                    label="Posts par plateforme",
                    bars=[{"label": str(row["platform"]), "value": int(row["count"])} for row in platforms],
                )
            )

        blocks.append(
            table_(
                columns=[
                    {"key": "platform", "label": "Plateforme"},
                    {"key": "politician_name", "label": "Politicien"},
                    {"key": "content", "label": "Contenu"},
                    {"key": "posted_at", "label": "Publié"},
                    {"key": "likes", "label": "J'aime", "align": "right"},
                    {"key": "shares", "label": "Partages", "align": "right"},
                    {"key": "comments", "label": "Comm.", "align": "right"},
                ],
                rows=[
                    {
                        "platform": str(r.get("platform") or "—"),
                        "politician_name": str(r.get("politician_name") or "—"),
                        "content": _truncate(str(r.get("content") or ""), 160),
                        "posted_at": str(r.get("posted_at") or "—"),
                        "likes": int(r.get("likes") or 0),
                        "shares": int(r.get("shares") or 0),
                        "comments": int(r.get("comments") or 0),
                    }
                    for r in rows
                ],
                empty_text="Aucun post.",
            )
        )

        return TabView(tab_name="social", blocks=blocks).model_dump()

    @nexus_tab(name="Presse", icon="newspaper")
    async def press_tab(self) -> dict[str, Any]:
        """Recent press entries with their sentiment column."""
        db = self._require_db()
        title = "Revue de presse"
        if db is None:
            return self._empty_tab("presse", title, "Base Gov indisponible.")

        rows = await queries.press_list_query(db, limit=50)
        if not rows:
            return self._empty_tab(
                "presse",
                title,
                "Aucune entrée de presse. Lancer un scrape presse pour alimenter le flux.",
            )

        return TabView(
            tab_name="presse",
            blocks=[
                heading(level=1, text=title),
                text(
                    text=f"{len(rows)} articles listés (50 max).",
                    muted=True,
                ),
                table_(
                    columns=[
                        {"key": "title", "label": "Titre"},
                        {"key": "source_name", "label": "Source"},
                        {"key": "sentiment", "label": "Sentiment"},
                        {"key": "published_at", "label": "Publié"},
                        {"key": "summary", "label": "Résumé"},
                    ],
                    rows=[
                        {
                            "title": _truncate(str(r.get("title") or "—"), 120),
                            "source_name": str(r.get("source_name") or "—"),
                            "sentiment": str(r.get("sentiment") or "—"),
                            "published_at": str(r.get("published_at") or "—"),
                            "summary": _truncate(str(r.get("summary") or ""), 160),
                        }
                        for r in rows
                    ],
                    empty_text="Aucun article.",
                ),
            ],
        ).model_dump()

    @nexus_tab(name="Transcriptions", icon="file-audio")
    async def transcriptions_tab(self) -> dict[str, Any]:
        """Recent ``gov_transcriptions`` rows joined with
        politician name when available."""
        db = self._require_db()
        title = "Transcriptions"
        if db is None:
            return self._empty_tab("transcriptions", title, "Base Gov indisponible.")

        rows = await queries.transcriptions_list_query(db, limit=50)
        if not rows:
            return self._empty_tab(
                "transcriptions",
                title,
                "Aucune transcription enregistrée. Lancer un worker de transcription pour alimenter la liste.",
            )

        return TabView(
            tab_name="transcriptions",
            blocks=[
                heading(level=1, text=title),
                text(
                    text=f"{len(rows)} transcriptions listées (50 max).",
                    muted=True,
                ),
                table_(
                    columns=[
                        {"key": "title", "label": "Titre"},
                        {"key": "politician_name", "label": "Politicien"},
                        {"key": "source_type", "label": "Source"},
                        {"key": "duration_seconds", "label": "Durée (s)", "align": "right"},
                        {"key": "language", "label": "Langue"},
                        {"key": "model_used", "label": "Modèle"},
                        {"key": "created_at", "label": "Créée"},
                    ],
                    rows=[
                        {
                            "title": _truncate(str(r.get("title") or "—"), 120),
                            "politician_name": str(r.get("politician_name") or "—"),
                            "source_type": str(r.get("source_type") or "—"),
                            "duration_seconds": int(r.get("duration_seconds") or 0),
                            "language": str(r.get("language") or "—"),
                            "model_used": str(r.get("model_used") or "—"),
                            "created_at": str(r.get("created_at") or "—"),
                        }
                        for r in rows
                    ],
                    empty_text="Aucune transcription.",
                ),
            ],
        ).model_dump()

    @nexus_tab(name="Sujets", icon="tag")
    async def subjects_tab(self) -> dict[str, Any]:
        """Aggrégat ``gov_positions.subject`` trié par fréquence."""
        db = self._require_db()
        title = "Sujets"
        if db is None:
            return self._empty_tab("sujets", title, "Base Gov indisponible.")

        rows = await queries.subjects_aggregate_query(db, limit=20)
        if not rows:
            return self._empty_tab(
                "sujets",
                title,
                "Aucun sujet indexé. Les sujets apparaissent dès qu'une position est enregistrée.",
            )

        blocks: list[TabBlock] = [
            heading(level=1, text=title),
            text(
                text=f"{len(rows)} sujets classés par nombre de positions.",
                muted=True,
            ),
            chart_bar(
                label="Positions par sujet",
                bars=[{"label": str(row["subject"]), "value": int(row["count"])} for row in rows],
            ),
            table_(
                columns=[
                    {"key": "subject", "label": "Sujet"},
                    {"key": "count", "label": "Positions", "align": "right"},
                ],
                rows=[{"subject": str(row["subject"]), "count": int(row["count"])} for row in rows],
                empty_text="Aucun sujet.",
            ),
        ]

        return TabView(tab_name="sujets", blocks=blocks).model_dump()

    # ------------------------------------------------------------------
    # Sprint 8 Phase D — Batch 3 tabs (Alertes/Affaires/Lois/Factchecks
    # + Recherche/Question RAG buttons)
    # ------------------------------------------------------------------

    @nexus_tab(name="Alertes", icon="bell")
    async def alerts_tab(self) -> dict[str, Any]:
        """Recent ``gov_alerts`` rows with severity chart and
        three summary metrics (total / unread / high severity)."""
        db = self._require_db()
        title = "Alertes"
        if db is None:
            return self._empty_tab("alertes", title, "Base Gov indisponible.")

        payload = await queries.alerts_overview_query(db, limit=50)
        rows = payload["rows"]
        by_severity = payload["by_severity"]
        summary = payload["summary"]

        if not rows and summary["total"] == 0:
            return self._empty_tab(
                "alertes",
                title,
                "Aucune alerte enregistrée. Les alertes apparaissent lorsque le moteur détecte un événement à remonter.",
            )

        blocks: list[TabBlock] = [
            heading(level=1, text=title),
            text(
                text=f"{len(rows)} alertes listées (50 max).",
                muted=True,
            ),
            section(
                title="Résumé",
                blocks=[
                    metric(label="Alertes", value=summary["total"], tone="neutral"),
                    metric(
                        label="Non lues",
                        value=summary["unread"],
                        tone="warn" if summary["unread"] > 0 else "neutral",
                    ),
                    metric(
                        label="Sévérité haute",
                        value=summary["high"],
                        tone="danger" if summary["high"] > 0 else "neutral",
                    ),
                ],
            ),
        ]

        if by_severity:
            blocks.append(
                chart_bar(
                    label="Alertes par sévérité",
                    bars=[{"label": str(row["severity"]), "value": int(row["count"])} for row in by_severity],
                )
            )

        blocks.append(
            table_(
                columns=[
                    {"key": "alert_type", "label": "Type"},
                    {"key": "title", "label": "Titre"},
                    {"key": "severity", "label": "Sévérité"},
                    {"key": "politician_name", "label": "Politicien"},
                    {"key": "is_read", "label": "Lu", "align": "right"},
                    {"key": "created_at", "label": "Créée"},
                ],
                rows=[
                    {
                        "alert_type": str(r.get("alert_type") or "—"),
                        "title": _truncate(str(r.get("title") or "—"), 120),
                        "severity": str(r.get("severity") or "—"),
                        "politician_name": str(r.get("politician_name") or "—"),
                        "is_read": int(r.get("is_read") or 0),
                        "created_at": str(r.get("created_at") or "—"),
                    }
                    for r in rows
                ],
                empty_text="Aucune alerte.",
            )
        )

        return TabView(tab_name="alertes", blocks=blocks).model_dump()

    @nexus_tab(name="Affaires", icon="briefcase")
    async def affairs_tab(self) -> dict[str, Any]:
        """Recent ``gov_affairs`` rows joined with politician
        names plus a status ``chart_bar``."""
        db = self._require_db()
        title = "Affaires"
        if db is None:
            return self._empty_tab("affaires", title, "Base Gov indisponible.")

        payload = await queries.affairs_list_query(db, limit=50)
        rows = payload["rows"]
        by_status = payload["by_status"]

        if not rows:
            return self._empty_tab(
                "affaires",
                title,
                "Aucune affaire enregistrée. Lancer un scrape dédié pour alimenter la liste.",
            )

        blocks: list[TabBlock] = [
            heading(level=1, text=title),
            text(
                text=f"{len(rows)} affaires listées (50 max).",
                muted=True,
            ),
        ]

        if by_status:
            blocks.append(
                chart_bar(
                    label="Affaires par statut",
                    bars=[{"label": str(row["status"]), "value": int(row["count"])} for row in by_status],
                )
            )

        blocks.append(
            table_(
                columns=[
                    {"key": "politician_name", "label": "Politicien"},
                    {"key": "title", "label": "Titre"},
                    {"key": "category", "label": "Catégorie"},
                    {"key": "status", "label": "Statut"},
                    {"key": "involvement", "label": "Implication"},
                    {"key": "date_start", "label": "Début"},
                    {"key": "date_end", "label": "Fin"},
                ],
                rows=[
                    {
                        "politician_name": str(r.get("politician_name") or "—"),
                        "title": _truncate(str(r.get("title") or "—"), 120),
                        "category": str(r.get("category") or "—"),
                        "status": str(r.get("status") or "—"),
                        "involvement": str(r.get("involvement") or "—"),
                        "date_start": str(r.get("date_start") or "—"),
                        "date_end": str(r.get("date_end") or "—"),
                    }
                    for r in rows
                ],
                empty_text="Aucune affaire.",
            )
        )

        return TabView(tab_name="affaires", blocks=blocks).model_dump()

    @nexus_tab(name="Lois", icon="scale")
    async def laws_tab(self) -> dict[str, Any]:
        """Recent ``gov_laws`` rows with status chart and three
        summary metrics (total / promulgated / average duration
        in days)."""
        db = self._require_db()
        title = "Lois"
        if db is None:
            return self._empty_tab("lois", title, "Base Gov indisponible.")

        payload = await queries.laws_list_query(db, limit=50)
        rows = payload["rows"]
        by_status = payload["by_status"]
        summary = payload["summary"]

        if not rows and summary["total"] == 0:
            return self._empty_tab(
                "lois",
                title,
                "Aucune loi référencée. Lancer un scrape dédié pour peupler la table.",
            )

        blocks: list[TabBlock] = [
            heading(level=1, text=title),
            text(
                text=f"{len(rows)} lois listées (50 max).",
                muted=True,
            ),
            section(
                title="Résumé",
                blocks=[
                    metric(label="Lois", value=summary["total"], tone="neutral"),
                    metric(
                        label="Promulguées",
                        value=summary["promulgated"],
                        tone="ok" if summary["promulgated"] > 0 else "neutral",
                    ),
                    metric(
                        label="Durée moyenne",
                        value=summary["avg_duration"],
                        unit="j",
                        tone="neutral",
                    ),
                ],
            ),
        ]

        if by_status:
            blocks.append(
                chart_bar(
                    label="Lois par statut",
                    bars=[{"label": str(row["status"]), "value": int(row["count"])} for row in by_status],
                )
            )

        blocks.append(
            table_(
                columns=[
                    {"key": "uid", "label": "Réf."},
                    {"key": "title", "label": "Titre"},
                    {"key": "procedure", "label": "Procédure"},
                    {"key": "status", "label": "Statut"},
                    {"key": "date_initial", "label": "Dépôt"},
                    {"key": "date_promulgation", "label": "Promulgation"},
                    {"key": "duration_days", "label": "Durée (j)", "align": "right"},
                    {"key": "amendments_count", "label": "Amendements", "align": "right"},
                ],
                rows=[
                    {
                        "uid": str(r.get("uid") or "—"),
                        "title": _truncate(str(r.get("title") or "—"), 120),
                        "procedure": str(r.get("procedure") or "—"),
                        "status": str(r.get("status") or "—"),
                        "date_initial": str(r.get("date_initial") or "—"),
                        "date_promulgation": str(r.get("date_promulgation") or "—"),
                        "duration_days": int(r.get("duration_days") or 0),
                        "amendments_count": int(r.get("amendments_count") or 0),
                    }
                    for r in rows
                ],
                empty_text="Aucune loi.",
            )
        )

        return TabView(tab_name="lois", blocks=blocks).model_dump()

    @nexus_tab(name="Factchecks", icon="check-circle")
    async def factchecks_tab(self) -> dict[str, Any]:
        """Recent ``gov_factchecks`` rows joined with politician
        names and a rating ``chart_bar``."""
        db = self._require_db()
        title = "Factchecks"
        if db is None:
            return self._empty_tab("factchecks", title, "Base Gov indisponible.")

        payload = await queries.factchecks_list_query(db, limit=50)
        rows = payload["rows"]
        by_rating = payload["by_rating"]

        if not rows:
            return self._empty_tab(
                "factchecks",
                title,
                "Aucun factcheck enregistré. Lancer un scrape dédié pour alimenter la liste.",
            )

        blocks: list[TabBlock] = [
            heading(level=1, text=title),
            text(
                text=f"{len(rows)} factchecks listés (50 max).",
                muted=True,
            ),
        ]

        if by_rating:
            blocks.append(
                chart_bar(
                    label="Factchecks par verdict",
                    bars=[{"label": str(row["rating"]), "value": int(row["count"])} for row in by_rating],
                )
            )

        blocks.append(
            table_(
                columns=[
                    {"key": "claim", "label": "Affirmation"},
                    {"key": "politician_name", "label": "Politicien"},
                    {"key": "claimant", "label": "Auteur"},
                    {"key": "rating", "label": "Verdict"},
                    {"key": "reviewer", "label": "Vérificateur"},
                    {"key": "claim_date", "label": "Affirmée"},
                    {"key": "review_date", "label": "Vérifiée"},
                ],
                rows=[
                    {
                        "claim": _truncate(str(r.get("claim") or "—"), 140),
                        "politician_name": str(r.get("politician_name") or "—"),
                        "claimant": str(r.get("claimant") or "—"),
                        "rating": str(r.get("rating") or "—"),
                        "reviewer": str(r.get("reviewer") or "—"),
                        "claim_date": str(r.get("claim_date") or "—"),
                        "review_date": str(r.get("review_date") or "—"),
                    }
                    for r in rows
                ],
                empty_text="Aucun factcheck.",
            )
        )

        return TabView(tab_name="factchecks", blocks=blocks).model_dump()

    @nexus_tab(name="Recherche", icon="search")
    async def search_tab(self) -> dict[str, Any]:
        """RAG semantic-search entry point.

        v1 TabView has no free-form text input; the tab exposes
        a single button that dispatches a canned query to the
        ``gov.rag_search`` worker. A real search box lands in a
        future SDK version that adds form-field block kinds.
        """
        example_query = "Quels politiciens se sont exprimés sur le climat ?"
        return TabView(
            tab_name="recherche",
            blocks=[
                heading(level=1, text="Recherche RAG"),
                text(
                    text="Recherche sémantique sur l'ensemble du corpus Gov "
                    "(positions, lois, presse, transcriptions, factchecks).",
                    muted=True,
                ),
                section(
                    title="Exemple",
                    blocks=[
                        kv(
                            items=[
                                {"label": "Worker", "value": "gov.rag_search"},
                                {"label": "Modèle", "value": "nomic-embed-text"},
                                {"label": "Requête", "value": example_query},
                            ]
                        ),
                        button_task(
                            label="Lancer la recherche exemple",
                            worker="gov.rag_search",
                            payload={"query": example_query},
                            tone="neutral",
                        ),
                    ],
                ),
                text(
                    text="Le résultat sera visible dans l'onglet Tâches une fois la tâche traitée par un worker.",
                    muted=True,
                ),
            ],
        ).model_dump()

    # ------------------------------------------------------------------
    # Sprint 8 Phase E — command palette entries (@nexus_command)
    # ------------------------------------------------------------------

    @nexus_command(
        "new_scan",
        description="Lancer un nouveau scan des politiciens",
        icon="radar",
        group="Gov",
    )
    async def cmd_new_scan(self) -> dict[str, Any]:
        """Deep-link into the Scan tab.

        Sprint 8 Phase E ships the first app-contributed command
        palette entries. The handler returns a ``navigation``
        payload that the shell forwards to React Router; the Scan
        tab then shows the legacy scan log and (Sprint 9) a
        button to trigger a fresh scrape. Keeping the handler
        side-effect-free keeps Phase E free of any dispatcher
        plumbing — the user still has to click the button inside
        the tab to actually run a scan.
        """
        return {"navigation": {"path": "/app/gov/tabs/Scan"}}

    @nexus_command(
        "detect_contradictions",
        description="Détecter les contradictions politiques",
        icon="alert-octagon",
        group="Gov",
    )
    async def cmd_detect_contradictions(self) -> dict[str, Any]:
        """Deep-link into the Contradictions tab."""
        return {"navigation": {"path": "/app/gov/tabs/Contradictions"}}

    @nexus_command(
        "search_factchecks",
        description="Rechercher dans les fact-checks",
        icon="check-circle",
        group="Gov",
    )
    async def cmd_search_factchecks(self) -> dict[str, Any]:
        """Deep-link into the Factchecks tab."""
        return {"navigation": {"path": "/app/gov/tabs/Factchecks"}}

    @nexus_command(
        "view_alerts",
        description="Consulter les alertes récentes",
        icon="bell",
        group="Gov",
    )
    async def cmd_view_alerts(self) -> dict[str, Any]:
        """Deep-link into the Alertes tab."""
        return {"navigation": {"path": "/app/gov/tabs/Alertes"}}

    @nexus_tab(name="Question", icon="message-circle-question")
    async def ask_tab(self) -> dict[str, Any]:
        """RAG open-ended question entry point.

        Mirrors :meth:`search_tab` for the question-answering
        worker ``gov.rag_ask``. Same v1 limitation: the canned
        question is hardcoded until TabView v1.1 exposes a form
        block kind.
        """
        example_question = "Quelle est la position d'Alice Martin sur la loi climat ?"
        return TabView(
            tab_name="question",
            blocks=[
                heading(level=1, text="Question RAG"),
                text(
                    text="Question ouverte sur le corpus Gov — le worker "
                    "``gov.rag_ask`` récupère les sources pertinentes et "
                    "renvoie une réponse avec citations.",
                    muted=True,
                ),
                section(
                    title="Exemple",
                    blocks=[
                        kv(
                            items=[
                                {"label": "Worker", "value": "gov.rag_ask"},
                                {
                                    "label": "Modèle",
                                    "value": "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m",
                                },
                                {"label": "Question", "value": example_question},
                            ]
                        ),
                        button_task(
                            label="Poser la question exemple",
                            worker="gov.rag_ask",
                            payload={"question": example_question},
                            tone="neutral",
                        ),
                    ],
                ),
                text(
                    text="Le résultat sera visible dans l'onglet Tâches une fois la tâche traitée par un worker.",
                    muted=True,
                ),
            ],
        ).model_dump()

    # ------------------------------------------------------------------
    # Sprint 9 Phase E — 20th tab: Documents (file upload + CAS)
    # ------------------------------------------------------------------

    @nexus_tab(name="Documents", icon="file-text")
    async def documents_tab(self) -> dict[str, Any]:
        """List uploaded documents via ``ctx.dbs["app"]`` and render
        a ``file_upload`` block (v2 TabView) for drag-and-drop uploads.

        Sprint 9 Phase E (D3 consumer). The table reads from
        ``gov_documents`` populated by migration ``001_documents.sql``.
        The upload block posts to the coordinator's
        ``POST /app/gov/files/upload`` route via the ``@nexus_app_files``
        decorator on the class.
        """
        title = "Documents"
        if self._ctx is None:
            return self._empty_tab("documents", title, "Contexte non disponible.")

        app_db = self._ctx.dbs.get("app")
        rows: list[dict[str, Any]] = []
        if app_db is not None:
            try:
                rows = await app_db.fetchall(
                    "SELECT sha256, original_name, content_type, size, uploaded_at "
                    "FROM gov_documents ORDER BY uploaded_at DESC LIMIT 50"
                )
            except DatabaseError:
                rows = []

        blocks: list[TabBlock] = [
            heading(level=1, text=title),
            text(
                text="Documents PDF et images associés au dossier Gov. "
                "Glisser un fichier dans la zone ci-dessous pour l'ajouter.",
                muted=True,
            ),
        ]

        if rows:
            blocks.append(
                table_(
                    columns=[
                        {"key": "original_name", "label": "Nom"},
                        {"key": "content_type", "label": "Type"},
                        {"key": "size", "label": "Taille"},
                        {"key": "uploaded_at", "label": "Date"},
                    ],
                    rows=[
                        {
                            "original_name": str(r.get("original_name", "—")),
                            "content_type": str(r.get("content_type", "—")),
                            "size": str(r.get("size", 0)),
                            "uploaded_at": str(r.get("uploaded_at", "—")),
                        }
                        for r in rows
                    ],
                    empty_text="Aucun document.",
                )
            )
        else:
            blocks.append(empty(text="Aucun document téléversé."))

        return TabViewV2(
            tab_name="documents",
            blocks=[
                *blocks,
                file_upload_block(
                    label="Déposer un document",
                    accept=["image/*", "application/pdf"],
                ),
            ],
        ).model_dump()
