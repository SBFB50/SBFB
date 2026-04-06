# Coding Conventions

**Analysis Date:** 2026-04-06

## Naming Patterns

**Python Files:**
- Use `snake_case.py` for all modules: `evidence_processor.py`, `ollama_client.py`, `sqlite_db.py`
- API routers match the resource name: `cases.py`, `evidence.py`, `suspects.py`
- Test files prefixed with `test_`: `test_db.py`, `test_parsers.py`, `test_api.py`

**Python Functions:**
- Use `snake_case` for all functions and methods: `create_case()`, `extract_entities()`, `parse_json_safe()`
- Private methods prefixed with underscore: `_extract_text()`, `_generate_summary()`, `_handle_response_error()`
- Async methods use `async def` consistently with the `await` keyword

**Python Variables:**
- Use `snake_case`: `case_id`, `evidence_type`, `raw_text`
- Constants in `UPPER_SNAKE_CASE`: `_ROUTE_TABLE`, `_MIME_TO_EVIDENCE_TYPE`, `FUZZY_THRESHOLD`, `_SCORE_SHIFT_THRESHOLD`
- Private class attributes prefixed with underscore: `self._db`, `self._router`, `self._running`

**Python Types/Classes:**
- Use `PascalCase`: `EvidenceProcessor`, `LLMRouter`, `TaskType`, `BloodPatternAnalyzer`
- Pydantic models follow `{Entity}`, `{Entity}Base`, `{Entity}Create`, `{Entity}Update` pattern
- Enum classes use `PascalCase` with `UPPER_SNAKE_CASE` values: `TaskType.ENTITY_EXTRACTION`

**TypeScript Files:**
- Use `PascalCase.tsx` for React components and pages: `Dashboard.tsx`, `Evidence.tsx`, `Layout.tsx`
- Use `camelCase.ts` for hooks and stores: `useApi.ts`, `useCase.ts`, `caseStore.ts`
- Use `camelCase.ts` for utility modules: `client.ts`

**TypeScript Functions:**
- React components use `PascalCase`: `export default function Dashboard()`
- Hooks prefixed with `use`: `useActiveCase()`, `useEvidence()`, `useCases()`
- API functions use `camelCase`: `getCases()`, `submitTextEvidence()`, `scoreAllSuspects()`

## Code Style

**Formatting:**
- No Prettier or Black configured for Python
- No `.prettierrc` file present
- Python uses 4-space indentation consistently
- TypeScript uses 2-space indentation (Vite default)
- Line lengths are not enforced but generally stay under 120 characters

**Linting:**
- Python: No explicit linter config (no `.flake8`, `pyproject.toml` lint section, or `ruff.toml`)
- TypeScript: ESLint configured at `web/eslint.config.js` with:
  - `@eslint/js` recommended rules
  - `typescript-eslint` recommended rules
  - `eslint-plugin-react-hooks` flat config
  - `eslint-plugin-react-refresh` Vite config
- Run frontend lint: `cd web && npx eslint .`

## Import Organization

**Python Order (observed pattern):**
1. `from __future__ import annotations` (always first when present)
2. Standard library imports: `asyncio`, `json`, `re`, `uuid`, `hashlib`
3. Third-party imports: `loguru`, `fastapi`, `pydantic`, `tenacity`, `ollama`
4. Local imports: `from nexus.config import settings`, `from nexus.db.sqlite_db import Database`

**Python Example (from `nexus/core/evidence_processor.py`):**
```python
from __future__ import annotations

import hashlib
import shutil
import uuid
from pathlib import Path
from typing import Any, BinaryIO

from loguru import logger

from nexus.config import settings
from nexus.core.audit import AuditService
from nexus.core.chunker import TextChunker
```

**TypeScript Order (observed pattern):**
1. External library imports: `react`, `react-router-dom`, `@tanstack/react-query`
2. Local imports: `../api/client`, `../stores/caseStore`, `../hooks/useApi`
3. Component imports: `../components/Layout`, `../components/MetricCard`

**Path Aliases:**
- Python: No path aliases; uses absolute imports from `nexus.*` package root
- TypeScript: No path aliases configured in `tsconfig.json`; uses relative imports `../api/client`

## Error Handling

**Patterns:**

1. **Non-fatal catch-and-continue** -- The dominant pattern across `nexus/core/`. Operations that fail do not crash the pipeline; they log and continue:
```python
try:
    entities = await self._extract_and_save_entities(case_id, evidence_id, raw_text or "")
    logger.info("Saved {} entities for evidence {}", len(entities), evidence_id)
except Exception as exc:
    logger.error("Entity extraction failed for {}: {}", evidence_id, exc)
    # Non-fatal: we continue to summary
```

2. **HTTP exception conversion** -- API routers catch `ValueError` and convert to `HTTPException`:
```python
try:
    return await mgr.get_case(case_id)
except ValueError as exc:
    raise HTTPException(status_code=404, detail=str(exc))
```

3. **OOM/connection error handler** -- Global exception handler in `nexus/main.py` catches Ollama errors and returns 503:
```python
if isinstance(exc, (httpx.ConnectError, httpx.TimeoutException)):
    return JSONResponse(status_code=503, content={"detail": "LLM service unavailable"})
```

4. **Retry with tenacity** -- `nexus/llm/ollama_client.py` retries on transient errors (3 attempts, exponential backoff):
```python
retry=retry_if_exception_type((httpx.ConnectError, httpx.TimeoutException)),
stop=stop_after_attempt(3),
wait=wait_exponential(multiplier=1, min=1, max=8),
```

5. **Graceful degradation** -- At startup, Neo4j and ChromaDB failures are caught and the app runs in degraded mode:
```python
except Exception as exc:
    logger.warning("Neo4j unavailable -- running in degraded mode (no graph): {}", exc)
```

**When adding new error handling:**
- Use `except Exception as exc:` + `logger.error(...)` for non-blocking pipeline steps
- Use `except ValueError` + `HTTPException` for API route input validation failures
- Never silence exceptions without logging them
- Mark non-blocking failures explicitly in log messages: `"(non-blocking)"`

## Logging

**Framework:** Loguru (`from loguru import logger`)

**Patterns:**
- Every module imports Loguru: `from loguru import logger`
- Use structured string formatting (not f-strings): `logger.info("Processing evidence {}", evidence_id)`
- Log levels:
  - `logger.debug()` -- Detailed internal state (prompt lengths, response sizes, skipped items)
  - `logger.info()` -- Significant operations (entity extraction results, cycle completion)
  - `logger.warning()` -- Degraded functionality, non-fatal failures, missing optional services
  - `logger.error()` -- Failed operations that should have succeeded
- Uvicorn logs are intercepted and routed through Loguru via `_InterceptHandler` in `nexus/main.py`

**When adding new logging:**
- Always use `logger` from Loguru (never `print()` or `logging.getLogger()`)
- Use `{}` placeholders, not f-strings or `%s`: `logger.info("Saved {} items", count)`
- Include identifiers in log messages: case_id, evidence_id, model name

## Comments

**When to Comment:**
- Module-level docstrings are mandatory on every Python file (triple-quote at top)
- Class docstrings explain purpose and usage patterns
- Inline comments for non-obvious logic only
- Section headers use `# ====` blocks to separate major code regions

**Docstring Style:**
```python
"""
NEXUS -- Evidence Processor.

Full ingestion pipeline for uploaded files and manual text input:
save file, detect type, extract text, create DB entry, run entity
extraction via LLM, generate summary.

Usage::

    async with get_db() as conn:
        db = Database(conn)
        proc = EvidenceProcessor(db, router, settings.upload_dir)
        evidence = await proc.process_upload(case_id, file, "Report.pdf")
"""
```

**Section Delimiters (used throughout codebase):**
```python
# =====================================================================
# Cases CRUD
# =====================================================================
```

## Function Design

**Size:** Functions range from 5-50 lines. Complex pipeline methods (`process_upload`, OODA cycle phases) can reach 80+ lines but are subdivided with numbered comment steps.

**Parameters:** Use keyword-only for optional params (via `*`). Type hints on all parameters. Default values for optional fields.

**Return Values:** Always type-hinted. Async functions return domain objects or dicts. DB methods return `dict | None`. API endpoints return Pydantic models.

## Module Design

**Exports:** No `__all__` lists. Modules export via direct import. No barrel files (`__init__.py` files are empty).

**Dependency Injection:** FastAPI `Depends()` for request-scoped services. Singletons on `app.state` for shared clients. Constructor injection for service classes:
```python
class EvidenceProcessor:
    def __init__(self, db: Database, router: LLMRouter, upload_dir: Path, neo4j=None, chroma=None):
```

**Pydantic Model Hierarchy:** Every DB table has a 4-model pattern in `nexus/db/models.py`:
- `{Entity}Base` -- shared fields
- `{Entity}Create` -- input for POST (may omit auto-generated fields)
- `{Entity}Update` -- all fields Optional for partial updates
- `{Entity}` -- full response with id + timestamps, `model_config = {"from_attributes": True}`

## Language Convention

- **Code and comments:** Written in English
- **LLM prompts:** Written in French (`nexus/llm/prompts.py`)
- **User-facing strings and logs:** Mix of French and English (investigation domain terms in French)
- **Module-level docstrings:** French description of purpose, English for technical details

---

*Convention analysis: 2026-04-06*
