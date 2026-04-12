# SPDX-License-Identifier: AGPL-3.0-or-later
"""Pydantic filter models persisted via :class:`nexus_sdk.AppStorage`.

Sprint 9 Phase B (D1 consumer). Each gov tab that surfaces a
client-side filter ships its filter shape as a Pydantic model in
this module so the persistence layer is typed end-to-end:

- The model is the contract between the tab handler and the
  storage namespace (``ctx.storage.namespace("filters.X", Model)``).
- ``extra="forbid"`` is sacred — a future schema change that adds
  or renames a field surfaces as a structured
  :class:`nexus_sdk.StorageSchemaError` on read instead of
  silently dropping the unknown field. Sprint 10 Phase 0 audit
  gate verifies this explicitly.
- Date fields use :class:`datetime.date` (not strings) so a drift
  caused by a string written by an older app version raises a
  validation error rather than carrying a half-typed value.
"""

from __future__ import annotations

from datetime import date

from pydantic import BaseModel, ConfigDict


class PoliticiansFilter(BaseModel):
    """Persistent filter state for the ``Politiciens`` tab.

    Three optional fields cover the v1 filter surface:

    - ``chamber`` — restrict the listing to one chamber name (the
      raw string stored in ``gov_politicians.chamber``, e.g.
      ``"Assemblée"`` or ``"Sénat"``).
    - ``date_range`` — a ``(from, to)`` tuple of dates that scopes
      a future Sprint 10 timeline filter; persisted now so the
      drift detection contract is exercised in Phase B.
    - ``search`` — free-text substring matched against politician
      names. Empty string is treated like ``None`` by the tab
      handler (so that submitting an empty input box clears the
      filter rather than re-applying it).

    All fields default to ``None`` so an empty filter is the
    canonical "no restriction" state and a fresh app boot reads
    a missing key as ``PoliticiansFilter()``.
    """

    model_config = ConfigDict(extra="forbid")

    chamber: str | None = None
    date_range: tuple[date, date] | None = None
    search: str | None = None
