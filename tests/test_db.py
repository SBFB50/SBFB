"""Tests for the SQLite database layer (nexus.db.sqlite_db.Database).

All tests use an in-memory database -- no file I/O, no external services.
"""

import pytest
import pytest_asyncio


# =====================================================================
# Cases CRUD
# =====================================================================


@pytest.mark.asyncio
async def test_create_case(db):
    case = await db.create_case(name="Doe Case", description="A cold case")
    assert case is not None
    assert case["name"] == "Doe Case"
    assert case["description"] == "A cold case"
    assert case["status"] == "active"
    assert "id" in case
    assert "created_at" in case


@pytest.mark.asyncio
async def test_get_case(db):
    case = await db.create_case(name="Get Test")
    fetched = await db.get_case(case["id"])
    assert fetched is not None
    assert fetched["id"] == case["id"]
    assert fetched["name"] == "Get Test"


@pytest.mark.asyncio
async def test_get_case_not_found(db):
    result = await db.get_case("nonexistent-id")
    assert result is None


@pytest.mark.asyncio
async def test_list_cases(db):
    await db.create_case(name="Case A")
    await db.create_case(name="Case B", status="closed")
    all_cases = await db.list_cases()
    assert len(all_cases) >= 2

    active = await db.list_cases(status="active")
    closed = await db.list_cases(status="closed")
    assert all(c["status"] == "active" for c in active)
    assert all(c["status"] == "closed" for c in closed)


@pytest.mark.asyncio
async def test_update_case(db):
    case = await db.create_case(name="Original")
    updated = await db.update_case(case["id"], name="Updated", status="closed")
    assert updated["name"] == "Updated"
    assert updated["status"] == "closed"
    # updated_at should have changed
    assert updated["updated_at"] >= case["updated_at"]


@pytest.mark.asyncio
async def test_update_case_no_fields(db):
    case = await db.create_case(name="NoChange")
    result = await db.update_case(case["id"])
    assert result["name"] == "NoChange"


@pytest.mark.asyncio
async def test_delete_case(db):
    case = await db.create_case(name="ToDelete")
    deleted = await db.delete_case(case["id"])
    assert deleted is True
    assert await db.get_case(case["id"]) is None


@pytest.mark.asyncio
async def test_delete_case_not_found(db):
    deleted = await db.delete_case("does-not-exist")
    assert deleted is False


# =====================================================================
# Evidence CRUD
# =====================================================================


@pytest.mark.asyncio
async def test_create_evidence(db):
    case = await db.create_case(name="Evidence Case")
    ev = await db.create_evidence(
        case_id=case["id"],
        title="Witness Statement",
        evidence_type="text",
        source="interview",
        reliability=80,
        raw_text="I saw him at 10pm.",
    )
    assert ev["title"] == "Witness Statement"
    assert ev["evidence_type"] == "text"
    assert ev["reliability"] == 80
    assert ev["status"] == "pending"


@pytest.mark.asyncio
async def test_get_evidence(db):
    case = await db.create_case(name="EvGet")
    ev = await db.create_evidence(
        case_id=case["id"], title="Doc", evidence_type="pdf"
    )
    fetched = await db.get_evidence(ev["id"])
    assert fetched is not None
    assert fetched["id"] == ev["id"]


@pytest.mark.asyncio
async def test_get_evidence_not_found(db):
    assert await db.get_evidence("no-such-id") is None


@pytest.mark.asyncio
async def test_list_evidence_by_case(db):
    case = await db.create_case(name="EvList")
    await db.create_evidence(
        case_id=case["id"], title="E1", evidence_type="text"
    )
    await db.create_evidence(
        case_id=case["id"], title="E2", evidence_type="pdf", status="processed"
    )
    all_ev = await db.list_evidence_by_case(case["id"])
    assert len(all_ev) == 2

    pending = await db.list_evidence_by_case(case["id"], status="pending")
    assert len(pending) == 1
    assert pending[0]["title"] == "E1"


@pytest.mark.asyncio
async def test_update_evidence(db):
    case = await db.create_case(name="EvUpdate")
    ev = await db.create_evidence(
        case_id=case["id"], title="Old", evidence_type="text"
    )
    updated = await db.update_evidence(ev["id"], title="New", reliability=90)
    assert updated["title"] == "New"
    assert updated["reliability"] == 90


@pytest.mark.asyncio
async def test_delete_evidence(db):
    case = await db.create_case(name="EvDel")
    ev = await db.create_evidence(
        case_id=case["id"], title="Del", evidence_type="text"
    )
    assert await db.delete_evidence(ev["id"]) is True
    assert await db.get_evidence(ev["id"]) is None
    assert await db.delete_evidence(ev["id"]) is False


@pytest.mark.asyncio
async def test_evidence_metadata_json(db):
    case = await db.create_case(name="EvMeta")
    meta = {"pages": 5, "language": "fr"}
    ev = await db.create_evidence(
        case_id=case["id"],
        title="Meta",
        evidence_type="pdf",
        metadata=meta,
    )
    assert ev["metadata"] == meta


# =====================================================================
# Entities CRUD
# =====================================================================


@pytest.mark.asyncio
async def test_create_entity(db):
    case = await db.create_case(name="EntCase")
    ent = await db.create_entity(
        case_id=case["id"],
        name="John Doe",
        entity_type="person",
        aliases=["JD", "Johnny"],
        description="Primary suspect",
    )
    assert ent["name"] == "John Doe"
    assert ent["entity_type"] == "person"
    assert ent["aliases"] == ["JD", "Johnny"]


@pytest.mark.asyncio
async def test_list_entities_by_case_with_filter(db):
    case = await db.create_case(name="EntFilter")
    await db.create_entity(
        case_id=case["id"], name="Paris", entity_type="location"
    )
    await db.create_entity(
        case_id=case["id"], name="Jane", entity_type="person"
    )
    locations = await db.list_entities_by_case(
        case["id"], entity_type="location"
    )
    assert len(locations) == 1
    assert locations[0]["name"] == "Paris"


@pytest.mark.asyncio
async def test_update_entity(db):
    case = await db.create_case(name="EntUp")
    ent = await db.create_entity(
        case_id=case["id"], name="OldName", entity_type="person"
    )
    updated = await db.update_entity(ent["id"], name="NewName")
    assert updated["name"] == "NewName"


# =====================================================================
# Entity Mentions
# =====================================================================


@pytest.mark.asyncio
async def test_create_entity_mention(db):
    case = await db.create_case(name="MentionCase")
    ent = await db.create_entity(
        case_id=case["id"], name="X", entity_type="person"
    )
    ev = await db.create_evidence(
        case_id=case["id"], title="T", evidence_type="text"
    )
    mention = await db.create_entity_mention(
        entity_id=ent["id"],
        evidence_id=ev["id"],
        context="saw X at the bar",
        confidence=0.95,
    )
    assert mention["entity_id"] == ent["id"]
    assert mention["confidence"] == 0.95


@pytest.mark.asyncio
async def test_list_mentions_by_entity(db):
    case = await db.create_case(name="MentList")
    ent = await db.create_entity(
        case_id=case["id"], name="Y", entity_type="person"
    )
    ev1 = await db.create_evidence(
        case_id=case["id"], title="T1", evidence_type="text"
    )
    ev2 = await db.create_evidence(
        case_id=case["id"], title="T2", evidence_type="text"
    )
    await db.create_entity_mention(entity_id=ent["id"], evidence_id=ev1["id"])
    await db.create_entity_mention(entity_id=ent["id"], evidence_id=ev2["id"])
    mentions = await db.list_mentions_by_entity(ent["id"])
    assert len(mentions) == 2


# =====================================================================
# Hypotheses + Snapshots
# =====================================================================


@pytest.mark.asyncio
async def test_create_hypothesis(db):
    case = await db.create_case(name="HypCase")
    hyp = await db.create_hypothesis(
        case_id=case["id"],
        title="Suspect A did it",
        description="Based on alibi inconsistency",
        current_score=65.0,
    )
    assert hyp["title"] == "Suspect A did it"
    assert hyp["current_score"] == 65.0
    assert hyp["status"] == "active"


@pytest.mark.asyncio
async def test_update_hypothesis(db):
    case = await db.create_case(name="HypUp")
    hyp = await db.create_hypothesis(
        case_id=case["id"],
        title="Hyp",
        description="Desc",
    )
    updated = await db.update_hypothesis(
        hyp["id"], current_score=80.0, status="confirmed"
    )
    assert updated["current_score"] == 80.0
    assert updated["status"] == "confirmed"


@pytest.mark.asyncio
async def test_list_hypotheses_by_case(db):
    case = await db.create_case(name="HypList")
    await db.create_hypothesis(
        case_id=case["id"], title="H1", description="D1", current_score=90
    )
    await db.create_hypothesis(
        case_id=case["id"], title="H2", description="D2", current_score=30
    )
    hyps = await db.list_hypotheses_by_case(case["id"])
    assert len(hyps) == 2
    # Ordered by score DESC
    assert hyps[0]["current_score"] >= hyps[1]["current_score"]


@pytest.mark.asyncio
async def test_create_hypothesis_snapshot(db):
    case = await db.create_case(name="SnapCase")
    hyp = await db.create_hypothesis(
        case_id=case["id"], title="SnapH", description="D"
    )
    snap = await db.create_hypothesis_snapshot(
        hypothesis_id=hyp["id"],
        score=72.5,
        supporting=["evidence A"],
        contradicting=["evidence B"],
        reasoning="Score shifted because...",
        trigger="new_evidence",
        model_used="nexus",
    )
    assert snap["score"] == 72.5
    assert snap["supporting"] == ["evidence A"]
    assert snap["contradicting"] == ["evidence B"]


@pytest.mark.asyncio
async def test_list_snapshots_by_hypothesis(db):
    case = await db.create_case(name="SnapList")
    hyp = await db.create_hypothesis(
        case_id=case["id"], title="SL", description="D"
    )
    await db.create_hypothesis_snapshot(hypothesis_id=hyp["id"], score=50)
    await db.create_hypothesis_snapshot(hypothesis_id=hyp["id"], score=60)
    snaps = await db.list_snapshots_by_hypothesis(hyp["id"])
    assert len(snaps) == 2


# =====================================================================
# Alerts
# =====================================================================


@pytest.mark.asyncio
async def test_create_alert(db):
    case = await db.create_case(name="AlertCase")
    alert = await db.create_alert(
        case_id=case["id"],
        alert_type="new_evidence",
        severity="warning",
        title="New evidence found",
        message="Document uploaded",
    )
    assert alert["alert_type"] == "new_evidence"
    assert alert["severity"] == "warning"
    assert alert["is_read"] == 0


@pytest.mark.asyncio
async def test_mark_alert_read(db):
    case = await db.create_case(name="AlertRead")
    alert = await db.create_alert(
        case_id=case["id"],
        alert_type="new_evidence",
        title="T",
        message="M",
    )
    assert await db.mark_alert_read(alert["id"]) is True


@pytest.mark.asyncio
async def test_count_unread_alerts(db):
    case = await db.create_case(name="AlertCount")
    await db.create_alert(
        case_id=case["id"], alert_type="new_evidence", title="A", message="M"
    )
    await db.create_alert(
        case_id=case["id"], alert_type="score_shift", title="B", message="M"
    )
    assert await db.count_unread_alerts(case["id"]) == 2

    alerts = await db.list_alerts_by_case(case["id"])
    await db.mark_alert_read(alerts[0]["id"])
    assert await db.count_unread_alerts(case["id"]) == 1


# =====================================================================
# Monitoring Jobs + Results
# =====================================================================


@pytest.mark.asyncio
async def test_create_monitoring_job(db):
    case = await db.create_case(name="MonJob")
    job = await db.create_monitoring_job(
        case_id=case["id"],
        job_type="searxng",
        query="John Doe murder 2019",
        interval_hours=6,
    )
    assert job["query"] == "John Doe murder 2019"
    assert job["interval_hours"] == 6
    assert job["is_active"] == 1


@pytest.mark.asyncio
async def test_create_monitoring_result(db):
    case = await db.create_case(name="MonResult")
    job = await db.create_monitoring_job(
        case_id=case["id"], job_type="searxng", query="test"
    )
    result = await db.create_monitoring_result(
        job_id=job["id"],
        case_id=case["id"],
        url="https://example.com/article",
        title="Breaking news",
        snippet="Related to the case...",
        source_engine="google",
        relevance_score=85.0,
    )
    assert result["url"] == "https://example.com/article"
    assert result["is_new"] == 1

    # Job results_count should have incremented
    updated_job = await db._get_monitoring_job(job["id"])
    assert updated_job["results_count"] == 1


# =====================================================================
# Audit Log
# =====================================================================


@pytest.mark.asyncio
async def test_create_audit_entry(db):
    case = await db.create_case(name="AuditCase")
    entry = await db.create_audit_entry(
        case_id=case["id"],
        actor="system",
        action="evidence_added",
        target_type="evidence",
        target_id="ev-123",
        summary="Evidence uploaded",
    )
    assert entry["actor"] == "system"
    assert entry["action"] == "evidence_added"
    assert entry["summary"] == "Evidence uploaded"


@pytest.mark.asyncio
async def test_list_audit_log(db):
    case = await db.create_case(name="AuditList")
    await db.create_audit_entry(
        case_id=case["id"],
        actor="system",
        action="evidence_added",
        summary="S1",
    )
    await db.create_audit_entry(
        case_id=case["id"],
        actor="user",
        action="case_created",
        summary="S2",
    )
    all_entries = await db.list_audit_log(case["id"])
    assert len(all_entries) == 2

    filtered = await db.list_audit_log(case["id"], actor="user")
    assert len(filtered) == 1
    assert filtered[0]["actor"] == "user"


# =====================================================================
# Cascade Delete
# =====================================================================


@pytest.mark.asyncio
async def test_cascade_delete_removes_all_children(db):
    """Deleting a case must remove all dependent rows."""
    case = await db.create_case(name="CascadeCase")
    cid = case["id"]

    # Create evidence, entity, mention, hypothesis, snapshot, alert, audit, job
    ev = await db.create_evidence(
        case_id=cid, title="Ev", evidence_type="text"
    )
    ent = await db.create_entity(
        case_id=cid, name="Ent", entity_type="person"
    )
    await db.create_entity_mention(
        entity_id=ent["id"], evidence_id=ev["id"]
    )
    hyp = await db.create_hypothesis(
        case_id=cid, title="H", description="D"
    )
    await db.create_hypothesis_snapshot(hypothesis_id=hyp["id"], score=50)
    await db.create_alert(
        case_id=cid, alert_type="new_evidence", title="A", message="M"
    )
    await db.create_audit_entry(
        case_id=cid, actor="system", action="case_created", summary="S"
    )
    job = await db.create_monitoring_job(
        case_id=cid, job_type="searxng", query="q"
    )
    await db.create_monitoring_result(
        job_id=job["id"], case_id=cid, title="R"
    )
    await db.create_location(case_id=cid, name="Paris", lat=48.85, lon=2.35)

    # Delete cascade
    deleted = await db.delete_case(cid)
    assert deleted is True

    # Verify everything is gone
    assert await db.get_case(cid) is None
    assert await db.get_evidence(ev["id"]) is None
    assert await db.get_entity(ent["id"]) is None
    assert await db.get_hypothesis(hyp["id"]) is None
    assert len(await db.list_alerts_by_case(cid)) == 0
    assert len(await db.list_audit_log(cid)) == 0
    assert len(await db.list_jobs_by_case(cid)) == 0
    assert len(await db.list_locations_by_case(cid)) == 0


# =====================================================================
# Locations
# =====================================================================


@pytest.mark.asyncio
async def test_create_location(db):
    case = await db.create_case(name="LocCase")
    loc = await db.create_location(
        case_id=case["id"],
        name="Crime Scene",
        address="123 Rue du Crime, Paris",
        lat=48.8566,
        lon=2.3522,
        location_type="crime_scene",
    )
    assert loc["name"] == "Crime Scene"
    assert loc["lat"] == 48.8566
    assert loc["location_type"] == "crime_scene"


@pytest.mark.asyncio
async def test_list_locations_by_case(db):
    case = await db.create_case(name="LocList")
    await db.create_location(case_id=case["id"], name="A")
    await db.create_location(case_id=case["id"], name="B")
    locs = await db.list_locations_by_case(case["id"])
    assert len(locs) == 2


@pytest.mark.asyncio
async def test_get_location_by_entity(db):
    case = await db.create_case(name="LocEnt")
    ent = await db.create_entity(
        case_id=case["id"], name="Bar X", entity_type="location"
    )
    loc = await db.create_location(
        case_id=case["id"],
        name="Bar X",
        entity_id=ent["id"],
        lat=48.0,
        lon=2.0,
    )
    fetched = await db.get_location_by_entity(ent["id"])
    assert fetched is not None
    assert fetched["id"] == loc["id"]


# =====================================================================
# Suspects CRUD
# =====================================================================


@pytest.mark.asyncio
async def test_create_suspect(db):
    case = await db.create_case(name="SuspectCase")
    ent = await db.create_entity(
        case_id=case["id"], name="John Doe", entity_type="person"
    )
    suspect = await db.create_suspect(
        case_id=case["id"],
        entity_id=ent["id"],
        suspicion_score=65.0,
        alibi_status="weak",
        relationship_to_victim="neighbor",
        notes="Seen near the scene",
    )
    assert suspect is not None
    assert suspect["entity_id"] == ent["id"]
    assert suspect["suspicion_score"] == 65.0
    assert suspect["alibi_status"] == "weak"
    assert suspect["relationship_to_victim"] == "neighbor"
    assert suspect["notes"] == "Seen near the scene"
    assert "id" in suspect
    assert "created_at" in suspect


@pytest.mark.asyncio
async def test_get_suspect_by_entity(db):
    case = await db.create_case(name="SuspEntCase")
    ent = await db.create_entity(
        case_id=case["id"], name="Jane", entity_type="person"
    )
    created = await db.create_suspect(
        case_id=case["id"],
        entity_id=ent["id"],
        suspicion_score=40.0,
    )
    fetched = await db.get_suspect_by_entity(case["id"], ent["id"])
    assert fetched is not None
    assert fetched["id"] == created["id"]
    assert fetched["suspicion_score"] == 40.0

    # Non-existent entity returns None
    result = await db.get_suspect_by_entity(case["id"], "nonexistent")
    assert result is None


@pytest.mark.asyncio
async def test_update_suspect(db):
    case = await db.create_case(name="SuspUpdate")
    ent = await db.create_entity(
        case_id=case["id"], name="Bob", entity_type="person"
    )
    suspect = await db.create_suspect(
        case_id=case["id"],
        entity_id=ent["id"],
        suspicion_score=30.0,
        alibi_status="unknown",
    )
    updated = await db.update_suspect(
        suspect["id"],
        suspicion_score=80.0,
        alibi_status="none",
        graph_score=50.0,
    )
    assert updated["suspicion_score"] == 80.0
    assert updated["alibi_status"] == "none"
    assert updated["graph_score"] == 50.0
    assert updated["updated_at"] >= suspect["updated_at"]


@pytest.mark.asyncio
async def test_suspect_snapshot(db):
    case = await db.create_case(name="SuspSnap")
    ent = await db.create_entity(
        case_id=case["id"], name="Alice", entity_type="person"
    )
    suspect = await db.create_suspect(
        case_id=case["id"],
        entity_id=ent["id"],
        suspicion_score=55.0,
    )
    snap = await db.create_suspect_snapshot(
        suspect_id=suspect["id"],
        suspicion_score=55.0,
        graph_score=20.0,
        evidence_score=60.0,
        trigger="initial_scoring",
        model_used="nexus",
    )
    assert snap["suspicion_score"] == 55.0
    assert snap["graph_score"] == 20.0
    assert snap["evidence_score"] == 60.0
    assert snap["trigger"] == "initial_scoring"

    # Create a second snapshot
    snap2 = await db.create_suspect_snapshot(
        suspect_id=suspect["id"],
        suspicion_score=70.0,
        trigger="re_evaluation",
    )
    snaps = await db.list_suspect_snapshots(suspect["id"])
    assert len(snaps) == 2


@pytest.mark.asyncio
async def test_cascade_delete_suspects(db):
    """Deleting a case must remove suspects and their snapshots."""
    case = await db.create_case(name="CascadeSusp")
    cid = case["id"]

    ent = await db.create_entity(
        case_id=cid, name="Suspect X", entity_type="person"
    )
    suspect = await db.create_suspect(
        case_id=cid,
        entity_id=ent["id"],
        suspicion_score=50.0,
    )
    await db.create_suspect_snapshot(
        suspect_id=suspect["id"],
        suspicion_score=50.0,
    )

    # Also create other children to ensure full cascade still works
    ev = await db.create_evidence(
        case_id=cid, title="Ev", evidence_type="text"
    )
    await db.create_entity_mention(
        entity_id=ent["id"], evidence_id=ev["id"]
    )

    # Delete cascade
    deleted = await db.delete_case(cid)
    assert deleted is True

    # Verify suspects and snapshots are gone
    assert await db.get_suspect(suspect["id"]) is None
    suspects_list = await db.list_suspects_by_case(cid)
    assert len(suspects_list) == 0

    # Verify other children also gone
    assert await db.get_case(cid) is None
    assert await db.get_evidence(ev["id"]) is None
