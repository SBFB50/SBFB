"""End-to-end tests for the GOV pipeline (no external services).

Tests the full lifecycle: politician creation, positions, contradictions,
alerts, laws, social posts, scan logs, and identity resolution -- all using
an in-memory SQLite database.
"""

import sqlite3

import pytest
import aiosqlite

from nexus.gov.db import GovernmentDatabase, _GOV_CREATE_TABLES, _GOV_CREATE_INDEXES


# ============================================================================
# Fixtures
# ============================================================================


@pytest.fixture
async def gov_db():
    """Create an in-memory gov database with all tables."""
    conn = await aiosqlite.connect(":memory:")
    conn.row_factory = aiosqlite.Row
    await conn.execute("PRAGMA journal_mode = WAL")
    await conn.execute("PRAGMA foreign_keys = ON")

    # Create all gov tables and indexes (skip FTS -- triggers reference
    # content tables by rowid which requires the content= sync machinery;
    # the CRUD layer does not depend on FTS for basic operations).
    await conn.executescript(_GOV_CREATE_TABLES)
    await conn.executescript(_GOV_CREATE_INDEXES)

    db = GovernmentDatabase(conn)
    yield db
    await conn.close()


# ============================================================================
# E2E tests
# ============================================================================


class TestE2EGovPipeline:
    """Test the full GOV data pipeline."""

    @pytest.mark.asyncio
    async def test_create_politician_and_positions(self, gov_db):
        """Create a politician, add positions, verify counts."""
        # Create politician
        pol = await gov_db.create_politician(
            name="Test Politicien",
            chamber="assemblee",
            party="TEST",
            role="depute",
        )
        assert pol["id"]
        assert pol["name"] == "Test Politicien"

        # Create positions
        for i in range(5):
            await gov_db.create_position(
                politician_id=pol["id"],
                subject=f"Sujet {i}",
                position_type="vote",
                position_text=f"Vote sur le sujet {i}",
                stance="pour" if i % 2 == 0 else "contre",
                source_url=f"https://example.com/vote/{i}",
                source_type="assemblee_nationale",
                date=f"2026-01-{10 + i:02d}",
            )

        # Verify
        positions = await gov_db.list_positions_by_politician(pol["id"])
        assert len(positions) == 5

        stats = await gov_db.get_stats()
        assert stats["politicians"] >= 1
        assert stats["positions"] >= 5

    @pytest.mark.asyncio
    async def test_contradiction_creation(self, gov_db):
        """Create contradicting positions and a contradiction record."""
        pol = await gov_db.create_politician(
            name="Contradicteur", chamber="senat", party="X"
        )

        pos_a = await gov_db.create_position(
            politician_id=pol["id"],
            subject="Immigration",
            position_type="tweet",
            position_text="Je suis pour l'immigration",
            stance="pour",
            source_url="https://twitter.com/test/1",
            source_type="twitter",
            date="2024-01-15",
        )
        pos_b = await gov_db.create_position(
            politician_id=pol["id"],
            subject="Immigration",
            position_type="vote",
            position_text="Vote contre le projet de loi immigration",
            stance="contre",
            source_url="https://assemblee.fr/vote/123",
            source_type="assemblee_nationale",
            date="2024-06-20",
        )

        contra = await gov_db.create_contradiction(
            politician_id=pol["id"],
            position_a_id=pos_a["id"],
            position_b_id=pos_b["id"],
            subject="Immigration",
            description="Dit etre pour l'immigration mais vote contre le projet de loi",
            severity="high",
        )

        assert contra["id"]
        assert contra["severity"] == "high"

        contras = await gov_db.list_contradictions_by_politician(pol["id"])
        assert len(contras) == 1

    @pytest.mark.asyncio
    async def test_alert_system(self, gov_db):
        """Create and manage alerts."""
        alert = await gov_db.create_alert(
            alert_type="contradiction",
            title="Nouvelle contradiction detectee",
            description="Test alert",
            severity="high",
        )
        assert alert["id"]

        alerts = await gov_db.list_alerts()
        assert len(alerts) >= 1
        assert alerts[0]["is_read"] in (0, False)

        await gov_db.mark_alert_read(alert["id"])
        updated = await gov_db.get_alert(alert["id"])
        assert updated["is_read"] in (1, True)

    @pytest.mark.asyncio
    async def test_law_creation(self, gov_db):
        """Create and query laws."""
        law = await gov_db.create_law(
            uid="JORFTEXT000012345",
            title="Loi relative a la transparence politique",
            procedure="PJL",
            status="promulgue",
            legislature="17",
        )
        assert law["id"]

        found = await gov_db.get_law_by_uid("JORFTEXT000012345")
        assert found is not None
        assert found["title"] == "Loi relative a la transparence politique"

    @pytest.mark.asyncio
    async def test_social_post_dedup(self, gov_db):
        """Social posts deduplicate by (platform, post_id)."""
        pol = await gov_db.create_politician(
            name="Social Test", chamber="assemblee", party="Y"
        )

        post1 = await gov_db.create_social_post(
            politician_id=pol["id"],
            platform="twitter",
            post_id="12345",
            content="Premier tweet",
            url="https://x.com/test/12345",
        )
        assert post1["id"]

        # Second insert with same platform+post_id should not create duplicate
        # (implementation returns existing row on IntegrityError)
        count_before = await gov_db.count_social_posts()
        try:
            await gov_db.create_social_post(
                politician_id=pol["id"],
                platform="twitter",
                post_id="12345",
                content="Duplicate tweet",
                url="https://x.com/test/12345",
            )
        except Exception:
            pass  # Expected -- unique constraint
        count_after = await gov_db.count_social_posts()
        assert count_after == count_before  # No duplicate

    @pytest.mark.asyncio
    async def test_scan_log_checkpoint(self, gov_db):
        """Scan log supports checkpoint/resume."""
        scan = await gov_db.create_scan_log(scan_type="test_scan")
        assert scan["id"]

        # Update checkpoint
        await gov_db.update_scan_checkpoint(
            scan["id"], "phase_2", 50, {"key": "value"}
        )

        # Retrieve resumable scan
        resumable = await gov_db.get_resumable_scan("test_scan")
        assert resumable is not None
        assert resumable["current_phase"] == "phase_2"
        assert resumable["phase_offset"] == 50

    @pytest.mark.asyncio
    async def test_identity_resolution(self, gov_db):
        """Identity resolver normalizes French names correctly."""
        from nexus.gov.identity import normalize_name, compute_similarity

        # Normalization
        assert normalize_name("M. Jean-Pierre Dupont") == "jean pierre dupont"
        assert normalize_name("Mme Marie Le Gall") == "marie gall"

        # Similarity
        score = compute_similarity("emmanuel macron", "emmanuel macron")
        assert score > 0.99

        score2 = compute_similarity("marine le pen", "jean luc melenchon")
        assert score2 < 0.70  # Different people, below auto-link threshold

    @pytest.mark.asyncio
    async def test_full_pipeline_flow(self, gov_db):
        """Simulate: create politician -> positions -> contradiction -> alert -> stats."""
        # 1. Politician
        pol = await gov_db.create_politician(
            name="Pipeline Test", chamber="assemblee", party="PIPE"
        )

        # 2. Positions
        pos1 = await gov_db.create_position(
            politician_id=pol["id"],
            subject="Ecologie",
            position_type="tweet",
            position_text="L'ecologie est ma priorite",
            stance="pour",
            source_url="https://twitter.com/pipe/1",
            source_type="twitter",
            date="2025-01-01",
        )
        pos2 = await gov_db.create_position(
            politician_id=pol["id"],
            subject="Ecologie",
            position_type="vote",
            position_text="Vote contre la loi climat",
            stance="contre",
            source_url="https://assemblee.fr/vote/eco1",
            source_type="assemblee_nationale",
            date="2025-06-15",
        )

        # 3. Contradiction
        contra = await gov_db.create_contradiction(
            politician_id=pol["id"],
            position_a_id=pos1["id"],
            position_b_id=pos2["id"],
            subject="Ecologie",
            description="Contradiction: tweet pro-ecologie vs vote anti-climat",
            severity="high",
        )

        # 4. Alert
        alert = await gov_db.create_alert(
            alert_type="contradiction",
            title="Contradiction ecologie",
            description="Pipeline Test: ecologie contradiction",
            severity="high",
            politician_id=pol["id"],
        )

        # 5. Stats
        stats = await gov_db.get_stats()
        assert stats["politicians"] >= 1
        assert stats["positions"] >= 2
        assert stats["contradictions"] >= 1
        assert stats["alerts"] >= 1

    @pytest.mark.asyncio
    async def test_politician_search(self, gov_db):
        """Search politicians by name."""
        await gov_db.create_politician(
            name="Marine Le Pen", chamber="assemblee", party="RN"
        )
        await gov_db.create_politician(
            name="Emmanuel Macron", chamber="assemblee", party="RE"
        )

        results = await gov_db.search_politicians("Marine")
        assert len(results) == 1
        assert results[0]["name"] == "Marine Le Pen"

        results_all = await gov_db.search_politicians("a")
        assert len(results_all) >= 1  # At least Macron matches

    @pytest.mark.asyncio
    async def test_mandate_lifecycle(self, gov_db):
        """Create mandates for a politician."""
        pol = await gov_db.create_politician(
            name="Mandate Test", chamber="senat", party="LR"
        )

        mandate = await gov_db.create_mandate(
            politician_id=pol["id"],
            type="senateur",
            title="Senateur des Yvelines",
            institution="Senat",
            start_date="2023-10-01",
            is_current=True,
        )
        assert mandate["id"]
        assert mandate["is_current"] is True

        mandates = await gov_db.list_mandates_by_politician(pol["id"])
        assert len(mandates) == 1
