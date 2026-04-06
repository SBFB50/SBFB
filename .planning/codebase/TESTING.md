# Testing Patterns

**Analysis Date:** 2026-04-06

## Test Framework

**Runner:**
- pytest with `pytest-asyncio` for async tests
- Config: `pytest.ini` at project root

**Assertion Library:**
- Built-in `assert` statements
- `pytest.approx()` for floating-point comparisons
- `pytest.raises()` for exception testing

**Run Commands:**
```bash
python -m pytest tests/ -v           # Run all 233 tests
python -m pytest tests/test_db.py -v # Run specific file
python -m pytest tests/ -k "test_create_case" # Run by name pattern
```

**Configuration (`pytest.ini`):**
```ini
[pytest]
asyncio_mode = auto
testpaths = tests
```

## Test File Organization

**Location:**
- All tests in `tests/` directory at project root (separate from source)
- No co-located tests inside `nexus/` or `web/`

**Naming:**
- Test files: `test_{module}.py` -- e.g., `test_db.py`, `test_parsers.py`, `test_api.py`
- Test functions: `test_{behavior}` -- e.g., `test_create_case`, `test_geocode_success`
- Test classes: `Test{Feature}` -- e.g., `TestCleanLlmResponse`, `TestBloodPatternGeometry`

**Structure:**
```
tests/
  __init__.py
  conftest.py              # Shared fixtures (memory_conn, db)
  test_api.py              # 18 tests -- FastAPI endpoints (mocked services)
  test_chunker.py          # 14 tests -- Semantic text chunker
  test_config.py           # 17 tests -- Settings defaults + model routing
  test_db.py               # 41 tests -- SQLite CRUD for all tables
  test_forensics.py        # 29 tests -- BPA geometry + physics sim
  test_geo.py              # 16 tests -- Geo mapper with mocked HTTP
  test_ingest.py           # 20 tests -- Text/PDF parsers + file hashing
  test_parsers.py          # 50 tests -- LLM response parsing + JSON repair
  test_suspect_scorer.py   # 28 tests -- Scoring formulas + composite scores
  bench_kulik_progressive.py   # Integration benchmark (not pytest)
  bench_real_cases.py          # Integration benchmark (not pytest)
  run_benchmark.py             # Integration benchmark runner (not pytest)
  generate_benchmark.py        # Benchmark data generator
```

**Test Count by Module:**
| File | Tests | Lines | What it covers |
|------|-------|-------|----------------|
| `test_parsers.py` | 50 | 343 | LLM response cleaning, JSON extraction, entity/relation parsing |
| `test_db.py` | 41 | 726 | Full SQLite CRUD for cases, evidence, entities, hypotheses, suspects, alerts, audit, cascade deletes |
| `test_forensics.py` | 29 | 334 | Blood pattern geometry, physics simulation, sound propagation, cast-off |
| `test_suspect_scorer.py` | 28 | 274 | Evidence scoring, contradiction scoring, hypothesis scoring, composite formula |
| `test_ingest.py` | 20 | 253 | Text parsing, file hashing, MIME detection |
| `test_api.py` | 18 | 285 | FastAPI endpoint CRUD, Pydantic validation, mocked dependencies |
| `test_config.py` | 17 | 118 | Settings defaults, model routing table completeness |
| `test_geo.py` | 16 | 217 | Geocoding, map data building, travel time verification |
| `test_chunker.py` | 14 | 195 | Text splitting, overlap, metadata, boundary detection |
| **Total** | **233** | **2,745** | |

## Test Structure

**Suite Organization:**

Tests use two patterns:

1. **Class-based grouping** for modules with many behaviors (used in `test_parsers.py`, `test_forensics.py`, `test_config.py`, `test_geo.py`, `test_ingest.py`, `test_suspect_scorer.py`):
```python
class TestCleanLlmResponse:

    def test_removes_think_tags(self):
        raw = "<think>Let me reason...</think>The answer is 42."
        assert clean_llm_response(raw) == "The answer is 42."

    def test_extracts_content_from_json_code_fence(self):
        raw = '```json\n{"key": "value"}\n```'
        assert clean_llm_response(raw) == '{"key": "value"}'
```

2. **Flat function grouping** with section headers for CRUD (used in `test_db.py`, `test_api.py`):
```python
# =====================================================================
# Cases CRUD
# =====================================================================

@pytest.mark.asyncio
async def test_create_case(db):
    case = await db.create_case(name="Doe Case", description="A cold case")
    assert case is not None
    assert case["name"] == "Doe Case"
    assert case["status"] == "active"
```

**Setup/Teardown:**
- Shared fixtures in `tests/conftest.py` using `pytest_asyncio.fixture`
- In-memory SQLite database created per-test via `memory_conn` fixture
- `db` fixture wraps connection in a `Database` instance
- No global teardown needed (in-memory DB is disposed automatically)

**Assertion Style:**
- Simple `assert` with clear expressions: `assert result["name"] == "Test Case"`
- `pytest.approx()` for floats: `assert score == pytest.approx(52.4)`
- `pytest.raises()` for exceptions: `with pytest.raises(ValueError, match="positive"):`
- Status code assertions for API tests: `assert resp.status_code == 201`

## Mocking

**Framework:** `unittest.mock` (built-in) -- `AsyncMock`, `MagicMock`, `patch`

**Patterns:**

1. **AsyncMock for DB and service objects** (used in `test_api.py`):
```python
@pytest.fixture
def mock_db():
    """A mock Database with async methods."""
    db = AsyncMock()
    return db

@pytest.fixture
def client(mock_db, mock_case_manager):
    app = _make_test_app()
    app.dependency_overrides[get_database] = lambda: mock_db
    app.dependency_overrides[get_case_manager] = lambda: mock_case_manager
    return TestClient(app), mock_db, mock_case_manager
```

2. **Patch for external HTTP calls** (used in `test_geo.py`):
```python
with patch("nexus.core.geo_mapper.httpx.AsyncClient") as MockClient:
    mock_client = AsyncMock()
    mock_client.get.return_value = mock_response
    mock_client.__aenter__ = AsyncMock(return_value=mock_client)
    mock_client.__aexit__ = AsyncMock(return_value=None)
    MockClient.return_value = mock_client

    result = await mapper.geocode_address("Paris")
```

3. **Patch for method-level mocking** (used in `test_geo.py`):
```python
with patch.object(mapper, "calculate_route", new_callable=AsyncMock) as mock_route:
    mock_route.return_value = {"distance_km": 50.0, "duration_min": 40.0}
    result = await mapper.verify_travel_time("A", "B", claimed_minutes=45.0)
```

**What to Mock:**
- Ollama LLM calls (all test modules avoid real LLM inference)
- External HTTP services (Nominatim, OSRM, SearXNG, Robin)
- Neo4j and ChromaDB connections
- FastAPI dependencies via `app.dependency_overrides`

**What NOT to Mock:**
- Pure computation (forensic math, scoring formulas, text chunking, JSON parsing)
- In-memory SQLite operations (use real `aiosqlite` with `:memory:`)
- Pydantic model validation
- Configuration loading

## Fixtures and Factories

**Shared Fixtures (`tests/conftest.py`):**
```python
@pytest_asyncio.fixture
async def memory_conn():
    """Yield an aiosqlite connection backed by :memory:."""
    conn = await aiosqlite.connect(":memory:")
    conn.row_factory = aiosqlite.Row
    await conn.execute("PRAGMA foreign_keys=ON")
    from nexus.db.sqlite_db import _CREATE_TABLES, _CREATE_INDEXES
    await conn.executescript(_CREATE_TABLES)
    await conn.executescript(_CREATE_INDEXES)
    await conn.commit()
    yield conn
    await conn.close()

@pytest_asyncio.fixture
async def db(memory_conn):
    """Yield a Database instance bound to an in-memory connection."""
    from nexus.db.sqlite_db import Database
    return Database(memory_conn)
```

**Per-Test Fixtures (example from `test_api.py`):**
```python
@pytest.fixture
def client(mock_db, mock_case_manager):
    app = _make_test_app()
    app.dependency_overrides[get_database] = lambda: mock_db
    app.dependency_overrides[get_case_manager] = lambda: mock_case_manager
    return TestClient(app), mock_db, mock_case_manager
```

**Test Data:**
- No factory library (no factory_boy or similar)
- Test data created inline via DB methods: `await db.create_case(name="Test Case")`
- Forensic test data uses known mathematical values (e.g., `sin(30) = 0.5 → width=2.5, length=5.0`)
- Parser tests use raw LLM-like strings with edge cases embedded directly

**Location:**
- All fixtures in `tests/conftest.py` or inline in test files
- Benchmark data in `data/benchmark/` (separate from unit tests)

## Coverage

**Requirements:** No coverage target enforced. No coverage configuration file.

**Current Gaps (not tested at all):**
- `nexus/core/autonomous_loop.py` -- The OODA loop (most complex module, 600+ lines)
- `nexus/core/evidence_processor.py` -- Full ingestion pipeline
- `nexus/core/hypothesis_engine.py` -- Hypothesis generation/scoring
- `nexus/core/contradiction_detector.py` -- Contradiction detection
- `nexus/core/analysis_pipeline.py` -- Multi-model analysis
- `nexus/core/retriever.py` -- RAG retriever
- `nexus/core/summary_tree.py` -- RAPTOR hierarchical summaries
- `nexus/db/neo4j_db.py` -- Neo4j graph operations
- `nexus/db/chroma_db.py` -- ChromaDB vector operations
- `nexus/monitoring/` -- All monitoring modules (scheduler, SearXNG, Robin)
- `nexus/recon/` -- All OSINT modules (holehe, social, domain)
- `nexus/export/` -- Report generation
- `nexus/core/image_analyzer.py` -- VLM analysis
- `nexus/vision/` -- DINOv2/CLIP image search
- `web/` -- No frontend tests at all (no Vitest, no React Testing Library)

**Well-Tested Areas:**
- SQLite CRUD (`test_db.py` -- 41 tests covering all 15 tables, cascade deletes)
- LLM response parsing (`test_parsers.py` -- 50 tests covering edge cases)
- Forensic math (`test_forensics.py` -- 29 tests with known-value verification)
- Suspect scoring formulas (`test_suspect_scorer.py` -- 28 tests)
- Configuration defaults (`test_config.py` -- 17 tests)
- API endpoints (`test_api.py` -- 18 tests with mocked dependencies)

## Test Types

**Unit Tests (233 total):**
- Pure computation tests (no mocks): forensics, chunker, parsers, scorer, config
- Database integration tests: real in-memory SQLite with full schema
- API endpoint tests: FastAPI TestClient with mocked services

**Integration Tests (outside pytest, manual run):**
- `tests/run_benchmark.py` -- Injects benchmark case via HTTP API, runs analysis waves
- `tests/bench_kulik_progressive.py` -- Progressive cold case resolution test
- `tests/bench_real_cases.py` -- Multi-case benchmark evaluation
- These require running services (FastAPI, Ollama, Neo4j, ChromaDB)
- Not part of `pytest` suite; run manually with `python tests/run_benchmark.py`

**E2E Tests:**
- No automated E2E tests
- Manual E2E via React dashboard at `http://localhost:3002` → Benchmark page

## Common Patterns

**Async Testing:**
```python
@pytest.mark.asyncio
async def test_create_case(db):
    case = await db.create_case(name="Doe Case", description="A cold case")
    assert case is not None
    assert case["name"] == "Doe Case"
```

Note: `asyncio_mode = auto` in `pytest.ini` means `@pytest.mark.asyncio` is required on every async test.

**Error Testing:**
```python
def test_impact_angle_invalid_zero_length(self):
    bpa = self._make_analyzer()
    with pytest.raises(ValueError, match="positive"):
        bpa.calculate_impact_angle(width=1.0, length=0.0)
```

**Boundary Testing:**
```python
def test_evidence_score_capped_at_100(self):
    """Many high-quality mentions should cap at 100."""
    mentions = [{"confidence": 1.0, "evidence_id": f"e{i}"} for i in range(10)]
    ev_map = {f"e{i}": {"reliability": 100} for i in range(10)}
    score = compute_evidence_score(mentions, ev_map)
    assert score == 100.0
```

**Null/Empty Input Testing:**
```python
def test_empty_text(chunker):
    assert chunker.chunk_text("") == []
    assert chunker.chunk_text("   ") == []
    assert chunker.chunk_text(None) == []
```

**Cascade/Cleanup Testing:**
```python
@pytest.mark.asyncio
async def test_cascade_delete_removes_all_children(db):
    """Deleting a case must remove all dependent rows."""
    case = await db.create_case(name="CascadeCase")
    cid = case["id"]
    ev = await db.create_evidence(case_id=cid, title="Ev", evidence_type="text")
    # ... create children ...
    deleted = await db.delete_case(cid)
    assert deleted is True
    assert await db.get_evidence(ev["id"]) is None
```

## When Adding New Tests

**For a new core module:**
1. Create `tests/test_{module}.py`
2. If it needs DB: use the `db` fixture from conftest
3. If it needs LLM: mock via `AsyncMock` (never call real Ollama in tests)
4. If it has pure-math methods: test those without mocks first
5. Use class-based grouping if the module has multiple distinct features

**For a new API endpoint:**
1. Add to `tests/test_api.py` (or create `test_api_{resource}.py` if large)
2. Mock dependencies via `app.dependency_overrides`
3. Test happy path + 404 + validation error (422)

**For new React components:**
- No testing infrastructure exists yet
- Would need: Vitest + React Testing Library + `@testing-library/jest-dom`
- Config would go in `web/vitest.config.ts`

---

*Testing analysis: 2026-04-06*
