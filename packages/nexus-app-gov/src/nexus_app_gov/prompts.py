"""Prompts used by the gov app.

Extracted from the legacy ``nexus/engine/__init__.py``
per the Sprint 4 Phase D refactor so the core engine stays
app-agnostic. Keeping every gov-specific prompt string in this
module lets Sprint 5 delete it cleanly when the frontend stops
referencing the gov export directly.
"""

POLITICAL_CONTRADICTION_PROMPT = """\
You are an expert in detecting logical contradictions in political
statements. Given a set of statements, identify any pair that
contradicts one another, cite the exact phrases, and explain the
contradiction in plain language. Return your answer as a JSON
object with keys `contradictions` (a list of {a, b, explanation}
objects) and `confidence` (a float in [0, 1]).

Statements:
{statements}
"""
