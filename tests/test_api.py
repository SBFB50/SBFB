"""Tests for FastAPI endpoints (with mocked services).

Mocks the lifespan and all heavy dependencies (Ollama, Neo4j, ChromaDB)
so tests run with zero external services.
"""

import pytest
from unittest.mock import AsyncMock, MagicMock, patch
from datetime import datetime

from fastapi import FastAPI
from fastapi.testclient import TestClient


# =====================================================================
# Build a test app with mocked dependencies
# =====================================================================

def _make_test_app():
    """Build a FastAPI app without the real lifespan (no Ollama/Neo4j/Chroma).

    Instead, we directly include the routers and override dependencies.
    """
    from nexus.api import cases, evidence, entities
    from nexus.api.deps import get_database, get_case_manager, get_evidence_processor

    app = FastAPI()
    app.include_router(cases.router)
    app.include_router(evidence.router)
    app.include_router(entities.router)

    # Add health check directly
    @app.get("/api/health")
    async def health():
        return {"status": "ok", "version": "0.1.0"}

    return app


@pytest.fixture
def mock_db():
    """A mock Database with async methods."""
    db = AsyncMock()
    return db


@pytest.fixture
def mock_case_manager():
    """A mock CaseManager with async methods."""
    mgr = AsyncMock()
    return mgr


@pytest.fixture
def client(mock_db, mock_case_manager):
    """TestClient with mocked deps."""
    from nexus.api.deps import get_database, get_case_manager

    app = _make_test_app()

    # Override dependency injection
    app.dependency_overrides[get_database] = lambda: mock_db
    app.dependency_overrides[get_case_manager] = lambda: mock_case_manager

    return TestClient(app), mock_db, mock_case_manager


# =====================================================================
# Health endpoint
# =====================================================================


def test_health_endpoint(client):
    tc, _, _ = client
    resp = tc.get("/api/health")
    assert resp.status_code == 200
    data = resp.json()
    assert data["status"] == "ok"


# =====================================================================
# Cases endpoints
# =====================================================================


def test_create_case(client):
    from nexus.db.models import Case

    tc, _, mgr = client
    now = datetime.now(tz=None)
    case_obj = Case(
        id="case-1",
        name="Test Case",
        reference=None,
        description="desc",
        status="active",
        created_at=now,
        updated_at=now,
    )
    mgr.create_case.return_value = case_obj

    resp = tc.post("/api/cases", json={"name": "Test Case", "description": "desc"})
    assert resp.status_code == 201
    data = resp.json()
    assert data["name"] == "Test Case"
    mgr.create_case.assert_called_once()


def test_list_cases(client):
    tc, _, mgr = client
    mgr.list_cases.return_value = []
    resp = tc.get("/api/cases")
    assert resp.status_code == 200
    assert resp.json() == []


def test_get_case_not_found(client):
    tc, _, mgr = client
    from nexus.core.case_manager import CaseManager

    mgr.get_case.side_effect = ValueError("Case not found: bad-id")
    resp = tc.get("/api/cases/bad-id")
    assert resp.status_code == 404


def test_delete_case(client):
    tc, _, mgr = client
    mgr.delete_case.return_value = None
    resp = tc.delete("/api/cases/case-1")
    assert resp.status_code == 204


def test_delete_case_not_found(client):
    tc, _, mgr = client
    mgr.delete_case.side_effect = ValueError("Case not found")
    resp = tc.delete("/api/cases/not-found")
    assert resp.status_code == 404


# =====================================================================
# Evidence endpoints
# =====================================================================


def test_list_evidence(client):
    tc, db, _ = client
    db.list_evidence_by_case.return_value = []
    resp = tc.get("/api/cases/case-1/evidence")
    assert resp.status_code == 200
    assert resp.json() == []


def test_get_evidence_not_found(client):
    tc, db, _ = client
    db.get_evidence.return_value = None
    resp = tc.get("/api/evidence/ev-bad")
    assert resp.status_code == 404


def test_get_evidence_found(client):
    tc, db, _ = client
    now = datetime.utcnow().isoformat()
    db.get_evidence.return_value = {
        "id": "ev-1",
        "case_id": "case-1",
        "title": "Doc",
        "evidence_type": "text",
        "source": None,
        "source_date": None,
        "ingestion_date": now,
        "reliability": 50,
        "file_path": None,
        "raw_text": "content",
        "summary": None,
        "metadata": None,
        "status": "pending",
        "created_at": now,
    }
    resp = tc.get("/api/evidence/ev-1")
    assert resp.status_code == 200
    data = resp.json()
    assert data["id"] == "ev-1"
    assert data["title"] == "Doc"


def test_delete_evidence(client):
    tc, db, _ = client
    db.delete_evidence.return_value = True
    resp = tc.delete("/api/evidence/ev-1")
    assert resp.status_code == 204


def test_delete_evidence_not_found(client):
    tc, db, _ = client
    db.delete_evidence.return_value = False
    resp = tc.delete("/api/evidence/ev-bad")
    assert resp.status_code == 404


# =====================================================================
# Entities endpoints
# =====================================================================


def test_list_entities(client):
    tc, db, _ = client
    db.list_entities_by_case.return_value = []
    resp = tc.get("/api/cases/case-1/entities")
    assert resp.status_code == 200
    assert resp.json() == []


def test_get_entity_not_found(client):
    tc, db, _ = client
    db.get_entity.return_value = None
    resp = tc.get("/api/entities/ent-bad")
    assert resp.status_code == 404


def test_get_entity_found(client):
    tc, db, _ = client
    now = datetime.utcnow().isoformat()
    db.get_entity.return_value = {
        "id": "ent-1",
        "case_id": "case-1",
        "name": "John Doe",
        "entity_type": "person",
        "aliases": ["JD"],
        "description": None,
        "first_seen": None,
        "metadata": None,
        "created_at": now,
    }
    resp = tc.get("/api/entities/ent-1")
    assert resp.status_code == 200
    data = resp.json()
    assert data["name"] == "John Doe"
    assert data["entity_type"] == "person"


def test_list_entity_mentions(client):
    tc, db, _ = client
    now = datetime.utcnow().isoformat()
    db.get_entity.return_value = {
        "id": "ent-1",
        "case_id": "case-1",
        "name": "X",
        "entity_type": "person",
        "aliases": None,
        "description": None,
        "first_seen": None,
        "metadata": None,
        "created_at": now,
    }
    db.list_mentions_by_entity.return_value = []
    resp = tc.get("/api/entities/ent-1/mentions")
    assert resp.status_code == 200
    assert resp.json() == []


def test_list_entity_mentions_entity_not_found(client):
    tc, db, _ = client
    db.get_entity.return_value = None
    resp = tc.get("/api/entities/bad-id/mentions")
    assert resp.status_code == 404


# =====================================================================
# Pydantic validation
# =====================================================================


def test_create_case_missing_name(client):
    tc, _, mgr = client
    resp = tc.post("/api/cases", json={"description": "no name"})
    assert resp.status_code == 422  # Pydantic validation error


def test_create_case_invalid_status(client):
    tc, _, mgr = client
    resp = tc.post(
        "/api/cases",
        json={"name": "X", "status": "invalid_status"},
    )
    assert resp.status_code == 422
