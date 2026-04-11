"""GovApp — nexus-grid port of the legacy government monitoring stack.

Sprint 4 Phase D shipped a minimal stub (one route, one worker,
one tab). Sprint 8 Phase B grows that stub to **seven tabs**:
the original ``Contradictions`` placeholder (Phase C will upgrade
it to a real descriptor) plus six new read-only tabs that browse
the legacy SQLite schema directly:

- **Dashboard** — aggregate counts across the gov tables
- **Politiciens** — list of politicians (paginated to 50)
- **Politicien** — detail view for the first politician (Sprint 9
  polish will add a per-tab selector)
- **Biographie** — mandates + party memberships chronology for
  the first politician
- **Positions** — recent positions across all politicians
- **Sujets** — aggregate of ``gov_positions.subject`` by count

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

from pathlib import Path
from typing import Any

from nexus_sdk import (
    AppContext,
    AppDatabaseClient,
    AppManifest,
    NexusApp,
    nexus_route,
    nexus_tab,
    nexus_worker,
)
from nexus_sdk.view import (
    TabBlock,
    TabView,
    chart_bar,
    empty,
    heading,
    kv,
    metric,
    section,
    table_,
    text,
)

from nexus_app_gov import queries
from nexus_app_gov.prompts import POLITICAL_CONTRADICTION_PROMPT


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


class GovApp(NexusApp):
    """Political monitoring — Sprint 8 Phase B gov migration Batch 1."""

    manifest = AppManifest(
        name="gov",
        version="0.2.0",
        author="FlowUP",
        description="Government monitoring: politicians, positions, contradictions, "
        "mandates and party history across the legacy gov SQLite schema.",
        license="AGPL-3.0",
    )

    def __init__(self) -> None:
        super().__init__()
        self._ctx: AppContext | None = None

    async def on_start(self, ctx: AppContext) -> None:
        """Swap ``ctx.db`` onto the legacy govdata.db file when
        it exists, then keep the context for later tab handlers.

        The loader pre-wires a default :class:`AppDatabaseClient`
        at ``<project>/apps/gov/app.sqlite`` — we leave that in
        place when the legacy file is absent so tab handlers
        still have a live client to call against (their queries
        will hit ``DatabaseError`` on missing tables and fall
        back to empty state).
        """
        legacy = _legacy_govdata_db_path()
        if legacy.exists():
            ctx.db = AppDatabaseClient(legacy)
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
        passing; Phase D will add the RAG search/ask workers."""
        task = await ctx.compute.submit_task(
            task_type="contradiction_check",
            prompt=POLITICAL_CONTRADICTION_PROMPT.format(statements="(example)"),
            model="stub-model:latest",
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
    # Legacy Sprint 4 tab — kept as a stub, Phase C upgrades it
    # ------------------------------------------------------------------

    @nexus_tab(name="Contradictions", icon="alert-octagon")
    def contradictions_tab(self) -> dict[str, Any]:
        """Sprint 4 stub kept intact — Phase C of Sprint 8
        upgrades this to a full table + chart descriptor. Don't
        touch it yet: the regression test in
        ``test_schema_driven_descriptor_validates`` still asserts
        the legacy heading-only shape."""
        return TabView(
            tab_name="contradictions",
            title="Détection de contradictions",
            blocks=[
                heading(level=1, text="Analyse de cohérence politique"),
                text(
                    text=POLITICAL_CONTRADICTION_PROMPT.splitlines()[0],
                    muted=True,
                ),
                metric(label="Déclarations analysées", value=0),
                metric(label="Contradictions détectées", value=0, tone="warn"),
                empty(
                    text="Aucune analyse en cours — soumettre un lot via /statements",
                ),
            ],
        ).model_dump()

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

    @nexus_tab(name="Politiciens", icon="users")
    async def politicians_tab(self) -> dict[str, Any]:
        """Paginated table of politicians (50 rows max)."""
        db = self._require_db()
        title = "Politiciens"
        if db is None:
            return self._empty_tab("politiciens", title, "Base Gov indisponible.")

        rows = await queries.politicians_list_query(db, limit=50)
        if not rows:
            return self._empty_tab(
                "politiciens",
                title,
                "Aucun politicien référencé. Lancer un scrape pour peupler la base.",
            )

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
                text(text=f"{len(rows)} politiciens listés (max 50).", muted=True),
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
