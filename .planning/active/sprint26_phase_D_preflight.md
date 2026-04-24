# Sprint 26 Phase D — preflight G8

Date : 2026-04-24 | HEAD : `8b71042` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, OSS prior art obligatoire, context7 systematique — aucune tension avec Phase D
- feedback_context7_systematic.md : context7 requis sur Pydantic (lib externe) — fait ci-dessous S1b

## Scans (all clean)
- S1a OSS prior art : 3 projets recherches (OpenAI Agents SDK `@function_tool`, Pydantic AI `@agent.tool`, MCP Python SDK), APPROACH-ALIGNED — le pattern decorateur + `model_json_schema()` Pydantic v2 est le standard SOTA. Phase D variante explicite `(RequestModel, ResponseModel)` adaptee au cas SBFB (paire request/response structuree) — clean
- S1b deps : Pydantic v2 `model_json_schema()` confirme stable par context7 (API inchangee, pas de CVE), 0 delta — clean
- S2 historiques : 3 fichiers cibles (decorators.py, registry.py, manifest.py) + archive planning scannes, 0 decision historique traversee — clean
- S3 threat model : fast-path verified, Phase D ne cree pas de nouveau composant securite ni wire format. HARDENING_ROADMAP S26 inclut C2 @task_handler SDK — aligned — clean
- S4 wire format : fast-path verified, Phase D ne touche pas canonical.rs/schemas/, VERSION=1 preserved, Day 0 D3 preserved — clean

## Telemetrie preflight
- Duree totale : ~3m
- S1a : ~2m / 3 projets OSS consultes (OpenAI Agents SDK, Pydantic AI, MCP Python SDK) / finding : APPROACH-ALIGNED (clean)
- S1b : ~1m / 1 lib scannee (Pydantic v2) / finding : clean
- S2 : ~30s / 3 fichiers + archive scannes / finding : clean
- S3 : fast-path / ~15s
- S4 : fast-path / ~15s

## Action
Proceder code phase D.
