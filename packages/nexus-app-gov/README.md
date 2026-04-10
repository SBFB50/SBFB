# nexus-app-gov

Sprint 4 Phase D **minimal migration** of the legacy `nexus/gov/`
Python package to the new `NexusApp` SDK shape.

Scope is deliberately small per the sprint kickoff decision F:
exactly one route, one worker, one tab, plus the
`POLITICAL_CONTRADICTION_PROMPT` constant extracted from
`nexus/engine/__init__.py`. The rest of the legacy gov tabs and
workers migrate in v1.1 when the coordinator gains app-scoped
route mounting.
