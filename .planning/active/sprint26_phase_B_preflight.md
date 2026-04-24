# Sprint 26 Phase B — preflight G8

Date : 2026-04-24 | HEAD : `23b8833` | Verdict : **PLAN-ADAPT**

## Memory consultation (Step 1.5)

- feedback_approach.md : "chercher les projets open source existants
  avant de coder from scratch" + "G8 = mecanisme procedural pour le
  principe pick-deepest" + "OSS prior art OBLIGATOIRE avant chaque
  phase (G10)"
- feedback_context7_systematic.md : context7 obligatoire avant tout
  code touchant lib/API/spec. MCP spec 2025-11-25 = spec touchee.
- Tensions plan vs memory : le plan propose un `McpHandler` hand-rolled
  (~600 LOC JSON-RPC manuel). La regle memory "chercher les projets
  OSS existants avant de coder from scratch" impose de verifier si un
  SDK officiel couvre le besoin.

## S1a — OSS prior art research

- Projets recherches :
  - `modelcontextprotocol/python-sdk` (GitHub, MIT, v1.27.0 PyPI
    2026-04-02) — SDK officiel MCP Python par Anthropic, transfere
    Linux Foundation Agentic AI Foundation 2026
  - context7 `/modelcontextprotocol/python-sdk` (High reputation,
    benchmark 81.3, 301 snippets)

- context7 queries :
  1. `resolve-library-id("MCP Python SDK")` → `/modelcontextprotocol/python-sdk`
  2. `query-docs("MCP server FastAPI Streamable HTTP transport tool
     registration")` → FastMCP quickstart + Streamable HTTP config +
     low-level Server + mount Starlette + OAuth TokenVerifier
  3. `query-docs("mount MCP server into existing FastAPI Starlette
     ASGI app middleware authentication")` → Mount pattern +
     TokenVerifier protocol + CORS middleware

- Finding : **LIB-EXISTS** — le SDK officiel `mcp` (v1.27.0, MIT,
  modelcontextprotocol/python-sdk) couvre 100% du perimetre Phase B :

  | Fonctionnalite plan | SDK equivalent |
  |---|---|
  | `McpHandler.__init__` + dispatch JSON-RPC | `FastMCP("SBFB")` ou `Server("sbfb")` — dispatch automatique |
  | `_initialize()` retourne capabilities | Automatique a l'initialisation du serveur SDK |
  | `_list_tools()` retourne 3 tools + schemas | `@mcp.tool()` decorator avec JSON Schema auto-genere |
  | `_call_tool()` dispatch vers services | `@mcp.tool()` handler async avec type hints |
  | `_error()` codes -32601 / -32700 | JSON-RPC error handling automatique (spec-compliant) |
  | `POST /mcp` + `GET /mcp` FastAPI endpoints | `mcp.streamable_http_app()` mount Starlette ASGI |
  | JSON Schema statique 3 tools | Auto-genere depuis signatures Python |

  Le plan proposait ~600 LOC de JSON-RPC hand-rolled. Le SDK reduit
  ca a ~150 LOC (tool impls + mount + capability gate middleware).

  Plan proposait : `McpHandler` class avec dispatch manuel `method ==
  "initialize"` / `"tools/list"` / `"tools/call"`, JSON-RPC framing
  manuel, error codes manuels, FastAPI router separe `api/mcp.py`.

  OSS montre : `FastMCP` gere tout le protocol internement. Mount
  via `app.mount("/mcp", mcp.streamable_http_app())`. Les tools se
  declarent via `@mcp.tool()` decorators. Le SDK garantit la
  conformite spec 2025-11-25 et les futures evolutions protocol.

## Plan adaptation

### Approche corrigee

1. **Ajouter `mcp>=1.27` a `pyproject.toml`** du coordinator
   (dep MIT, compatible AGPL-3.0).

2. **`mcp_server.py`** : creer un `FastMCP("sbfb", stateless_http=True,
   json_response=True)` avec 3 tools via `@mcp.tool()` :
   - `task_submit(project_id, prompt, model?)` → dispatch vers
     coordinator submit logic
   - `storage_get(project_id, key)` → lecture storage coordinator
   - `storage_set(project_id, key, value)` → ecriture storage coordinator
   Les tools accedent au coordinator via un setter module-level
   (`set_coordinator(coord)`) appele dans `create_app()`.

3. **`api/mcp.py` SUPPRIME** : pas de router FastAPI separe. Le SDK
   gere les routes `/mcp` (POST + GET) via le mount Starlette.

4. **`api/app.py`** : mount le MCP app avec capability gate middleware :
   ```python
   from nexus_coordinator.mcp_server import create_mcp_app
   mcp_app = create_mcp_app(coordinator)
   app.mount("/mcp", mcp_app)
   ```
   Le `LoopbackAuthMiddleware` existant (bearer + Origin + Host)
   s'applique DEJA a toutes les routes y compris les mounts —
   pas besoin de re-implementer bearer/Origin pour MCP.

5. **Capability gate** : ASGI middleware wrappant le MCP app qui
   verifie `is_enabled("mcp_server_expose")` → 403 si desactive.
   Pattern analogue a `@require_capability` mais au niveau ASGI
   (car le SDK gere ses propres routes).

### Fichiers impactes vs plan

| Plan original | Approche corrigee |
|---|---|
| Nouveau `mcp_server.py` (~300 LOC JSON-RPC handler) | Nouveau `mcp_server.py` (~100 LOC tools + mount factory) |
| Nouveau `api/mcp.py` (~150 LOC FastAPI endpoints) | **Supprime** — SDK gere le routing |
| Modif `api/__init__.py` ou router setup | Modif `api/app.py` — `app.mount("/mcp", ...)` |
| N/A | Modif `pyproject.toml` — `mcp>=1.27` |

### Tests impactes

Memes scenarios que le plan (~15-20 tests) mais adaptes :
- `test_mcp_initialize` → round-trip JSON-RPC initialize
- `test_mcp_list_tools` → 3 tools retournes avec schemas
- `test_mcp_call_*` → dispatch vers coordinator logic
- `test_mcp_unknown_method` → SDK retourne -32601 automatiquement
- `test_mcp_invalid_json` → SDK retourne -32700 automatiquement
- `test_mcp_capability_gate_*` → teste le ASGI middleware
- `test_mcp_auth_*` → couvert par `LoopbackAuthMiddleware` existant
  (tests supplementaires specifiques MCP pour confirmer l'heritage)

## Scans S1b/S2/S3/S4

- **S1b deps** : `mcp>=1.27` nouvelle dep. Deps transitives :
  `httpx`, `pydantic`, `starlette`, `anyio` — toutes deja presentes
  dans le coordinator (FastAPI les requiert). Pas de CVE connu sur
  `mcp` 1.27. Pas de breaking change vs contexte (premiere adoption).
  **Clean.**

- **S2 historiques** : 5 fichiers cibles scannes, 0 commit
  DEVIATION/rejected/scope-cut/threat-model. Archive scan MCP-related :
  0 finding. Memory feedback scan : aucune regle en tension.
  **Clean.**

- **S3 threat model** : fast-path verified. Phase B ajoute un
  endpoint group `/mcp` — pas un nouveau composant de securite
  (reutilise bearer + Origin + capability gate existants).
  HARDENING_ROADMAP §3 S26 confirme B2 MCP server en scope.
  Aucune regression threat T0-T5 : le MCP est local-only
  (bind 127.0.0.1), protege par le triple check loopback S16.
  **Clean.**

- **S4 wire format** : fast-path verified. MCP est local-only
  HTTP coordinator, pas de wire format P2P. Pas de
  `*_FORMAT_VERSION` touche. `canonical.rs` non touche. Day 0
  preservees (D1 confirme Streamable HTTP local-only, pas
  stdio). **Clean.**

## Telemetrie preflight

- Duree totale : ~8m
- S1a : ~5m / 1 projet OSS consulte (modelcontextprotocol/python-sdk)
  + 3 context7 queries / finding : LIB-EXISTS
- S1b : ~1m / 1 lib nouvelle (mcp>=1.27) / finding : clean
- S2 : ~1m / 5 fichiers + archive scan / finding : clean
- S3 : fast-path / ~30s
- S4 : fast-path / ~30s

## Action

Proceder code Phase B avec approche corrigee (SDK officiel `mcp`
v1.27+ au lieu de JSON-RPC hand-rolled). Commit body documente la
deviation vs plan §Phase B : "Plan proposait McpHandler hand-rolled,
preflight S1a identifie SDK officiel mcp v1.27 (LIB-EXISTS), adapte
vers FastMCP mount."
