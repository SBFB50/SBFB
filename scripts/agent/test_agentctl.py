#!/usr/bin/env python3
"""Regression tests for agentctl phase-title parsing.

Locks the unbounded-phase widening: the phase token grammar is
``[A-Z]+[0-9]?`` (A..Z, then AA, AB, ... with an optional sub-phase
digit), a strict superset of the historical single-letter
``[A-Z][0-9]?`` so every A-G sprint keeps parsing byte-identically.
``Phase 0`` (audit gate) stays a ``chore(planning)`` convention and is
deliberately NOT parsed as an implementation phase (no bare-number
phase parses), so the audit-commit fail-open path is preserved.

Run: ``python scripts/agent/test_agentctl.py`` (exit 0 = all pass).
"""
from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

_AGENTCTL = Path(__file__).resolve().parent / "agentctl.py"
_spec = importlib.util.spec_from_file_location("agentctl_under_test", _AGENTCTL)
assert _spec and _spec.loader
agentctl = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(agentctl)


def _title(sprint: str, phase: str, scope: str | None = None) -> str:
    scope = scope or f"sprint{sprint}"
    return f"feat({scope}): Sprint {sprint} Phase {phase} — exemple"


def _check(cond: bool, msg: str) -> int:
    print(("ok:   " if cond else "FAIL: ") + msg)
    return 0 if cond else 1


def main() -> int:
    fails = 0

    # Backward compat: every historical single-letter phase parses unchanged.
    for ph in ["A", "B", "C", "D", "E", "F", "G"]:
        s, p = agentctl.phase_from_title(_title("76", ph))
        fails += _check((s, p) == ("76", ph.lower()), f"Phase {ph} -> ('76','{ph.lower()}')")

    # Unbounded: past G, past Z, multi-letter, optional sub-phase digit.
    for ph, want in [("H", "h"), ("Z", "z"), ("AA", "aa"), ("AB", "ab"), ("AB1", "ab1"), ("A1", "a1")]:
        s, p = agentctl.phase_from_title(_title("77", ph))
        fails += _check((s, p) == ("77", want), f"Phase {ph} -> ('77','{want}')")

    # Fallback regex (commit scope != sprint{N}) widened too.
    s, p = agentctl.phase_from_title(_title("77", "AA", scope="feed"))
    fails += _check((s, p) == ("77", "aa"), "fallback scope Phase AA -> ('77','aa')")

    # requires_codex follows the widening (phase-impl commits carry evidence).
    fails += _check(agentctl.phase_commit_requires_codex(_title("77", "AA")), "Phase AA requires codex")
    fails += _check(agentctl.phase_commit_requires_codex(_title("76", "G")), "Phase G requires codex")

    # Non-phase / chore titles still do not parse as impl phases (fail-open).
    s, p = agentctl.phase_from_title("chore(planning): Sprint 76 audit findings")
    fails += _check((s, p) == (None, None), "chore(planning) audit title -> (None, None)")
    fails += _check(
        not agentctl.phase_commit_requires_codex("chore(planning): Sprint 76 audit findings"),
        "chore(planning) does not require codex",
    )

    # A bare-number phase (e.g. Phase 8, or the Phase 0 audit-gate convention)
    # must NOT parse as an implementation phase.
    for n in ["0", "8"]:
        s, p = agentctl.phase_from_title(_title("77", n))
        fails += _check((s, p) == (None, None), f"Phase {n} (bare number) does not parse as impl phase")

    if fails:
        print(f"\n{fails} FAILED")
        return 1
    print("\nall agentctl phase-parsing tests passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
