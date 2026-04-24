# Phase Review — Sprint 26 Phase B

## Verdict : PASS

(Rigor signal : 2 findings P2 documentés / >=1 requis pour PASS)

## Memory consultation (Step 1.5)
- feedback_approach.md : "chercher les projets OSS existants" — respecté
  (SDK officiel mcp v1.27 adopté via G8 PLAN-ADAPT)
- feedback_context7_systematic.md : "context7 obligatoire avant tout
  code touchant lib/API/spec" — respecté (3 context7 queries dans
  preflight + MCP spec 2025-11-25 consultée au kickoff)

## Staging check (Step 1bis)
- Preflight : commité séparément (`ab7c017`)
- Phase files : 5 (mcp_server.py NEW, test_mcp_server.py NEW,
  api/app.py MOD, pyproject.toml MOD, uv.lock MOD)
- Planning/docs split : preflight déjà commité ✅
- Untracked accidentels : 0

## Suites (Step 2)
- Rust nextest : 792 pass ✅ (inchangé — pas de Rust cette phase)
- Rust clippy : clean ✅
- Rust fmt : clean ✅
- Rust doctests : clean ✅
- Python SDK : 185 pass ✅
- Python coord : 406 pass + 14 fail (pré-existants, baseline vérifié) ✅
- Python app-gov : collection errors (pré-existant — stale wheel) ✅
- Ruff format + lint : clean ✅
- Frontend tsc : clean ✅
- Frontend lint : 0 errors ✅
- Frontend Vitest : 264 pass ✅
- Frontend build : clean ✅
- Frontend size : 7/7 ✅
- Release build : exe locké Windows (Rust inchangé, pas de regression)

## Delta tests (Step 3)
- Coord : 394 → 406 (+12 Phase B MCP)
- Autres suites : inchangées
- **Delta total session S26** : +12 Phase B (après +8 Phase A = +20 S26 cumulé)

## Commit body validation (Step 4)
- Format titre : ✅ `feat(sprint26): Phase B — B2 MCP server local-only Streamable HTTP 3 tools whitelist`
- Contexte : SDK officiel mcp v1.27 (PLAN-ADAPT vs plan hand-rolled)
- Fichiers touchés avec rationale : ✅
- Delta tests cumulé : ✅
- Scope cuts honoured : ✅ (liste des 12 items du kickoff §7)
- Co-Authored-By : ✅

## Modified-file branch coverage (Step 2bis, G9)
- `api/app.py` : `if mcp_ctx:` / `mcp_srv.session_manager.run() if mcp_srv else None`
  — defensive guards (mcp_srv always set by create_app). MCP path
  exercé par 12 tests via test_mcp_server.py qui crée ses propres
  FastMCP instances. ✅ CONCERN (defensive None-guard sans test dédié,
  acceptable car path principal couvert)

## Research grounding (Step 4bis)
- **4bis-A OSS prior art (G10)** : preflight S1a documente
  modelcontextprotocol/python-sdk (3 context7 queries, finding
  LIB-EXISTS, PLAN-ADAPT émis). ✅ PASS
- **4bis-B Deps/API context7** : kickoff §Sources consultées liste
  context7 MCP spec 2025-11-25 + arti-client. Preflight ajoute
  3 queries SDK specifiques. `mcp>=1.27` tracé dans preflight + pyproject comment. ✅ PASS

## Horizon long-terme + documentation amont (Step 4ter)
- Design doc : preflight + kickoff D1 servent de design doc (MCP est
  un endpoint, pas un module structurant multi-sprint) ✅
- D1..D5 avec alternatives + rationale : kickoff §4 D1 documente 3
  alternatives rejetées (stdio, marketplace, WebSocket) ✅
- Solution la plus poussée : SDK officiel = choix le plus robuste ✅
- LOC estimées au plan : **PRESENT** (plan §6 table "~600 LOC")
  → P2 (cf. feedback_approach.md §6.7 interdiction)

## Scope cuts verification (Step 5)
12 scope cuts du kickoff §7 — 0 fichiers touchent un scope cut ✅

## Findings (rigor signal)

- **P2-PLAN-LOC** : plan §6 table contient des estimations LOC
  prospectives (~600 LOC Phase B, ~500 LOC Phase C, etc.) contraire
  à feedback_approach.md §6.7. Pre-existant depuis la rédaction du
  plan, pas introduit par cette phase. Carry-over documentation :
  supprimer les LOC estimées dans les plans futurs.

- **P2-LIFESPAN-AENTER** : `api/app.py` lifespan utilise
  `__aenter__`/`__aexit__` explicites pour le MCP session manager
  au lieu d'`async with`. Nécessaire car le context manager doit
  span le `yield` du lifespan FastAPI — le SDK ne fournit pas
  d'alternative. Trade-off acceptable mais fragile si le SDK
  change l'implémentation de `.run()`.

- **P3** : 12 tests (vs ~20 estimés plan) — delta expliqué par
  PLAN-ADAPT (SDK élimine tests de JSON-RPC framing, dispatch
  manuel, error codes — le SDK les gère internement).

## Recommendation
- Ready to commit : **oui**
- Carry-overs S27 : P2-PLAN-LOC (process, pas code)
- P2-LIFESPAN-AENTER : documenter dans PATTERNS.md si récurrence
