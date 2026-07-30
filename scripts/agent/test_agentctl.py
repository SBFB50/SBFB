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


# --- pytest-discoverable cases (G-PHASE-GRAMMAR, Sprint 83 Phase A) --------
# These assert-style functions are also driven by ``main()`` below so that
# both ``python scripts/agent/test_agentctl.py`` and ``pytest`` exercise
# them. Sprint 83 is the first sprint to reach bi-LETTER phase labels
# (AA, AI2, AJ1); earlier sprints only reached letter+digit (e.g. N2, S4).


def test_bi_letter_phase_labels_parse() -> None:
    """The bi-letter / sub-phase labels used by Sprint 83 all parse.

    ``[A-Z]+[0-9]?`` already covers them (the FORMS ``A1``/``AA``/``AB1``
    were exercised since S77); this locks the specific S83 strings and
    asserts each is treated as a codex-requiring implementation phase.
    """
    for phase, want in [("A1", "a1"), ("W1", "w1"), ("AA", "aa"), ("AI2", "ai2"), ("AJ1", "aj1")]:
        sprint, parsed = agentctl.phase_from_title(_title("83", phase))
        assert (sprint, parsed) == ("83", want), f"Phase {phase} -> ('83','{want}'), got {(sprint, parsed)}"
        assert agentctl.phase_commit_requires_codex(_title("83", phase)), f"Phase {phase} must require codex"


def test_numeric_phase_label_refused() -> None:
    """A bare-number phase (``Phase 3``) must NOT parse as an impl phase.

    The ``Phase 0`` audit-gate convention and the audit-commit fail-open
    path both depend on bare numbers never parsing.
    """
    for n in ["3", "0", "8"]:
        sprint, parsed = agentctl.phase_from_title(_title("83", n))
        assert (sprint, parsed) == (None, None), f"Phase {n} (bare number) must not parse, got {(sprint, parsed)}"
        assert not agentctl.phase_commit_requires_codex(_title("83", n)), f"Phase {n} must not require codex"


def test_phase_grammar_shared_across_hooks() -> None:
    """agentctl et les deux hooks Git parsent les labels S83 identiquement.

    Preuve COMPORTEMENTALE (pas seulement une sous-chaîne présente) : on
    rejoue l'extraction des hooks — ``grep -oE 'Phase[[:space:]]+[A-Z]+[0-9]?'``
    puis ``awk '{print $NF}'``, répliquée en Python — sur chaque label réel et
    on la compare à ``agentctl.phase_from_title``. La divergence CONNUE et
    bornée sur les labels malformés multi-chiffres (frontière ``\\b`` stricte
    côté agentctl vs grep glouton côté hooks) est asservie explicitement pour
    qu'elle ne dérive pas silencieusement — aucun label en usage ne la déclenche.

    NB divergence intentionnelle non testée ici : ``agentctl`` exclut ``chore``
    de ``phase_commit_requires_codex`` (helper), tandis que le hook lightcheck
    l'inclut (``:289``) — le HOOK est l'autorité de commit, l'exclusion agentctl
    est un fail-open documenté (cf. preflight §S4).
    """
    import re as _re

    phase_extractor = "Phase[[:space:]]+[A-Z]+[0-9]?"
    hooks = [
        agentctl.ROOT / ".claude" / "hooks" / "phase-precommit-lightcheck.sh",
        agentctl.ROOT / ".claude" / "hooks" / "phase-auditor-gate.sh",
    ]
    # (a) les deux hooks portent le MÊME extracteur littéral que la grammaire.
    for hook in hooks:
        text = hook.read_text(encoding="utf-8")
        assert phase_extractor in text, f"{hook.name} doit extraire la phase avec '{phase_extractor}'"
    grammar = "[A-Z]+[0-9]?"
    assert grammar in agentctl.PHASE_TITLE_RE.pattern, "PHASE_TITLE_RE doit porter le token de grammaire"
    assert grammar in agentctl.PHASE_TITLE_FALLBACK_RE.pattern, "FALLBACK doit porter le token de grammaire"

    def _hook_extract(title: str) -> str | None:
        """Réplique l'extraction des hooks (grep glouton + awk NF)."""
        m = _re.search(r"Phase\s+([A-Z]+[0-9]?)", title)
        return m.group(1) if m else None

    # (b) parité comportementale sur les labels réels S83.
    for phase in ["A1", "W1", "AA", "AI2", "AJ1"]:
        title = _title("83", phase)
        _, py_phase = agentctl.phase_from_title(title)
        hook_phase = _hook_extract(title)
        assert hook_phase is not None and py_phase == hook_phase.lower(), (
            f"parité rompue pour Phase {phase}: agentctl={py_phase!r} hook={hook_phase!r}"
        )

    # (c) divergence connue et bornée sur un label MALFORMÉ multi-chiffres :
    #     agentctl rejette (frontière \b), le grep glouton des hooks tronque.
    malformed = _title("83", "AI22")
    _, py_malformed = agentctl.phase_from_title(malformed)
    assert py_malformed is None, "agentctl doit rejeter 'AI22' (frontière \\b stricte)"
    assert _hook_extract(malformed) == "AI2", "grep glouton des hooks tronque 'AI22'->'AI2' (divergence connue, hors usage)"


def test_profile_golden_guard() -> None:
    """Une source du profil Factory stagée sans le golden est refusée.

    Miroir du Check 11 du hook shell. Les chemins viennent du golden lui-même
    (``sources[].path``), donc le test couvre les six sans en recopier aucune :
    une septième source ajoutée à l'enum Rust est protégée sans toucher ici.

    Discriminance : chaque source est asservie DANS LES DEUX SENS — refusée
    seule, acceptée avec le golden. Un test qui ne vérifierait que le refus
    passerait encore si la fonction refusait tout.
    """
    sources = agentctl.profile_sources()
    assert len(sources) >= 2, f"le golden doit exposer ses sources, vu {sources!r}"
    for src in sources:
        assert agentctl.profile_golden_errors([src]), f"{src} stagée sans le golden doit être refusée"
        assert not agentctl.profile_golden_errors([src, agentctl.PROFILE_GOLDEN]), (
            f"{src} stagée AVEC le golden doit passer"
        )
    assert not agentctl.profile_golden_errors(["README.md"]), "un fichier hors profil ne dit rien"
    assert not agentctl.profile_golden_errors([]), "un index vide ne dit rien"


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

    # Drive the pytest-discoverable G-PHASE-GRAMMAR cases (Sprint 83 Phase A)
    # so `python test_agentctl.py` and `pytest` stay in lockstep.
    for tfn in (
        test_bi_letter_phase_labels_parse,
        test_numeric_phase_label_refused,
        test_phase_grammar_shared_across_hooks,
        test_profile_golden_guard,
    ):
        try:
            tfn()
            fails += _check(True, f"{tfn.__name__} passed")
        except AssertionError as exc:
            fails += _check(False, f"{tfn.__name__}: {exc}")

    if fails:
        print(f"\n{fails} FAILED")
        return 1
    print("\nall agentctl phase-parsing tests passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
