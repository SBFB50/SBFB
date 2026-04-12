# SPDX-License-Identifier: AGPL-3.0-or-later
"""Command palette descriptors for nexus-grid apps.

Sprint 7 D5 (frozen in the Sprint 7 kickoff) defined the
``@nexus_command`` SDK surface; Sprint 8 Phase A implements it.

Apps annotate a coroutine method with ``@nexus_command`` and the
React shell's Command Palette exposes it as a first-class action
under a dedicated ``App: <name>`` group. The coordinator serves
the descriptors on ``GET /app/{name}/commands`` and forwards
invocations through ``POST /app/{name}/commands/{cmd}/invoke``.

Schema is frozen at version 1 — additive extensions bump the
version and the shell-side Zod mirror in lock-step.
"""

from __future__ import annotations

from typing import Literal

from pydantic import BaseModel, ConfigDict, Field


class CommandDescriptor(BaseModel):
    """A single command palette entry exposed by a :class:`NexusApp`.

    ``extra="forbid"`` and ``frozen=True`` make this a
    purely-declarative contract: any extra field from a future
    SDK version is rejected by the coordinator (defense in depth
    against silent drifts), and individual instances cannot be
    mutated after construction so the descriptor list returned by
    :meth:`NexusApp.commands` is safe to share across callers
    without defensive copies.

    Field caps keep the palette fetch response bounded: a
    malicious app shipping 10 MB descriptions would balloon the
    React Query cache otherwise.
    """

    model_config = ConfigDict(extra="forbid", frozen=True)

    schema_version: Literal[1] = 1
    name: str = Field(..., min_length=1, max_length=64)
    description: str = Field(..., min_length=1, max_length=280)
    icon: str = Field("sparkles", max_length=32)
    group: str = Field("Actions", max_length=32)
