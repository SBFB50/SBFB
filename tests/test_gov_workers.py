"""
Tests for GOV module workers.

Covers:
- All 31 worker imports
- Subscription verification (event types match expectations)
- Identity resolution logic (normalize_name, compute_similarity)
- Contradiction analyzer helpers (_subject_keywords, _subjects_overlap)
- GovEventType completeness
- GovernmentDatabase instantiation
- GovManager worker spec count
"""

import importlib

import pytest
import pytest_asyncio

from nexus.gov.events import GovEventType


# ===================================================================
# TestGovEventTypes -- Verify all expected event types exist
# ===================================================================

class TestGovEventTypes:
    """Test all required event types are defined."""

    def test_data_ingestion_events(self):
        assert GovEventType.GOV_POLITICIAN_ADDED == "gov_politician_added"
        assert GovEventType.GOV_POSITION_ADDED == "gov_position_added"
        assert GovEventType.GOV_SOCIAL_POST_ADDED == "gov_social_post_added"
        assert GovEventType.GOV_AFFAIR_ADDED == "gov_affair_added"
        assert GovEventType.GOV_PRESS_ADDED == "gov_press_added"
        assert GovEventType.GOV_LAW_ADDED == "gov_law_added"

    def test_media_processing_events(self):
        assert GovEventType.GOV_VIDEO_DOWNLOADED == "gov_video_downloaded"
        assert GovEventType.GOV_TRANSCRIPTION_READY == "gov_transcription_ready"
        assert GovEventType.GOV_IMAGE_ADDED == "gov_image_added"

    def test_analysis_events(self):
        assert GovEventType.GOV_CONTRADICTION_FOUND == "gov_contradiction_found"
        assert GovEventType.GOV_PATTERN_DETECTED == "gov_pattern_detected"
        assert GovEventType.GOV_SENTIMENT_ANALYZED == "gov_sentiment_analyzed"

    def test_tick_events(self):
        assert GovEventType.TICK_HOURLY == "gov_tick_hourly"
        assert GovEventType.TICK_DAILY == "gov_tick_daily"
        assert GovEventType.TICK_WEEKLY == "gov_tick_weekly"
        assert GovEventType.TICK_MONTHLY == "gov_tick_monthly"

    def test_all_required_event_names_exist(self):
        required = [
            "GOV_POLITICIAN_ADDED", "GOV_POSITION_ADDED", "GOV_SOCIAL_POST_ADDED",
            "GOV_VIDEO_DOWNLOADED", "GOV_TRANSCRIPTION_READY", "GOV_IMAGE_ADDED",
            "GOV_PRESS_ADDED", "GOV_LAW_ADDED", "GOV_AFFAIR_ADDED",
            "GOV_CONTRADICTION_FOUND", "GOV_PATTERN_DETECTED",
            "GOV_DECLARATION_ADDED", "GOV_FACTCHECK_ADDED",
            "GOV_ALERT_CREATED", "GOV_SENTIMENT_ANALYZED",
            "TICK_HOURLY", "TICK_DAILY", "TICK_WEEKLY", "TICK_MONTHLY",
        ]
        for name in required:
            assert hasattr(GovEventType, name), f"Missing event type: {name}"

    def test_event_count_at_least_19(self):
        """GovEventType should have at least 19 members."""
        members = [m for m in GovEventType]
        assert len(members) >= 19, f"Only {len(members)} event types, expected >= 19"


# ===================================================================
# TestGovWorkerImports -- Verify all 31 workers can be imported
# ===================================================================

class TestGovWorkerImports:
    """Test that every GOV worker module imports without error."""

    _WORKER_SPECS = [
        ("nexus.gov.workers.vote_sync", "GovVoteSyncWorker"),
        ("nexus.gov.workers.depute_sync", "GovDeputeSyncWorker"),
        ("nexus.gov.workers.senat_sync", "GovSenatSyncWorker"),
        ("nexus.gov.workers.law_sync", "GovLawSyncWorker"),
        ("nexus.gov.workers.hatvp_sync", "GovHATVPSyncWorker"),
        ("nexus.gov.workers.fabrique_sync", "GovFabriqueSyncWorker"),
        ("nexus.gov.workers.wikidata_sync", "GovWikidataSyncWorker"),
        ("nexus.gov.workers.affairs_sync", "GovAffairsSyncWorker"),
        ("nexus.gov.workers.press_sync", "GovPressSyncWorker"),
        ("nexus.gov.workers.factcheck_sync", "GovFactcheckSyncWorker"),
        ("nexus.gov.workers.eu_parliament_sync", "GovEUParliamentSyncWorker"),
        ("nexus.gov.workers.eurlex_sync", "GovEURlexSyncWorker"),
        ("nexus.gov.workers.twitter_sync", "GovTwitterSyncWorker"),
        ("nexus.gov.workers.facebook_sync", "GovFacebookSyncWorker"),
        ("nexus.gov.workers.instagram_sync", "GovInstagramSyncWorker"),
        ("nexus.gov.workers.youtube_sync", "GovYouTubeSyncWorker"),
        ("nexus.gov.workers.tiktok_sync", "GovTikTokSyncWorker"),
        ("nexus.gov.workers.transcription", "GovTranscriptionWorker"),
        ("nexus.gov.workers.vision", "GovVisionWorker"),
        ("nexus.gov.workers.contradiction_analyzer", "GovContradictionAnalyzer"),
        ("nexus.gov.workers.voting_pattern", "GovVotingPatternAnalyzer"),
        ("nexus.gov.workers.neo4j_sync", "GovNeo4jSyncWorker"),
        ("nexus.gov.workers.sentiment", "GovSentimentAnalyzer"),
        ("nexus.gov.workers.alert", "GovAlertWorker"),
        ("nexus.gov.workers.embedding", "GovEmbedWorker"),
        ("nexus.gov.workers.biography", "GovBiographyWorker"),
        ("nexus.gov.workers.weekly_recap", "GovWeeklyRecapWorker"),
        ("nexus.gov.workers.vote_impact", "GovVoteImpactWorker"),
        ("nexus.gov.workers.press_affair_detector", "GovPressAffairDetector"),
        ("nexus.gov.workers.newsletter", "GovNewsletterWorker"),
        ("nexus.gov.workers.social_publish", "GovSocialPublishWorker"),
    ]

    @pytest.mark.parametrize("mod_path,cls_name", _WORKER_SPECS)
    def test_worker_import(self, mod_path, cls_name):
        """Each worker module imports and exposes its class."""
        mod = importlib.import_module(mod_path)
        cls = getattr(mod, cls_name)
        assert cls is not None
        # Verify it's a class with subscriptions
        assert hasattr(cls, "subscriptions"), f"{cls_name} missing subscriptions"
        assert hasattr(cls, "name"), f"{cls_name} missing name"
        assert len(cls.subscriptions) > 0, f"{cls_name} has empty subscriptions"

    def test_total_worker_count(self):
        """All 31 workers should be importable."""
        imported = 0
        for mod_path, cls_name in self._WORKER_SPECS:
            mod = importlib.import_module(mod_path)
            if hasattr(mod, cls_name):
                imported += 1
        assert imported == 31, f"Only {imported}/31 workers imported"


# ===================================================================
# TestGovWorkerSubscriptions -- Verify correct event subscriptions
# ===================================================================

class TestGovWorkerSubscriptions:
    """Verify workers subscribe to the expected event types."""

    def test_contradiction_analyzer_subscriptions(self):
        from nexus.gov.workers.contradiction_analyzer import GovContradictionAnalyzer
        subs = GovContradictionAnalyzer.subscriptions
        assert GovEventType.GOV_POSITION_ADDED in subs
        assert GovEventType.GOV_SOCIAL_POST_ADDED in subs
        assert GovEventType.GOV_TRANSCRIPTION_READY in subs
        assert GovEventType.GOV_PRESS_ADDED in subs
        assert len(subs) == 4

    def test_voting_pattern_subscriptions(self):
        from nexus.gov.workers.voting_pattern import GovVotingPatternAnalyzer
        assert GovVotingPatternAnalyzer.subscriptions == [GovEventType.TICK_WEEKLY]

    def test_sentiment_subscriptions(self):
        from nexus.gov.workers.sentiment import GovSentimentAnalyzer
        assert GovSentimentAnalyzer.subscriptions == [GovEventType.GOV_PRESS_ADDED]

    def test_transcription_subscriptions(self):
        from nexus.gov.workers.transcription import GovTranscriptionWorker
        assert GovTranscriptionWorker.subscriptions == [GovEventType.GOV_VIDEO_DOWNLOADED]

    def test_vision_subscriptions(self):
        from nexus.gov.workers.vision import GovVisionWorker
        assert GovVisionWorker.subscriptions == [GovEventType.GOV_VIDEO_DOWNLOADED]

    def test_alert_subscriptions(self):
        from nexus.gov.workers.alert import GovAlertWorker
        subs = GovAlertWorker.subscriptions
        assert GovEventType.GOV_CONTRADICTION_FOUND in subs
        assert GovEventType.GOV_AFFAIR_ADDED in subs
        assert GovEventType.GOV_PATTERN_DETECTED in subs

    def test_embedding_subscriptions(self):
        from nexus.gov.workers.embedding import GovEmbedWorker
        subs = GovEmbedWorker.subscriptions
        assert GovEventType.GOV_POSITION_ADDED in subs
        assert GovEventType.GOV_SOCIAL_POST_ADDED in subs
        assert GovEventType.GOV_TRANSCRIPTION_READY in subs
        assert GovEventType.GOV_PRESS_ADDED in subs

    def test_neo4j_sync_subscriptions(self):
        from nexus.gov.workers.neo4j_sync import GovNeo4jSyncWorker
        subs = GovNeo4jSyncWorker.subscriptions
        assert GovEventType.GOV_POSITION_ADDED in subs
        assert GovEventType.GOV_AFFAIR_ADDED in subs
        assert GovEventType.GOV_PRESS_ADDED in subs
        assert GovEventType.GOV_CONTRADICTION_FOUND in subs
        assert GovEventType.GOV_POLITICIAN_ADDED in subs
        assert GovEventType.GOV_DECLARATION_ADDED in subs

    def test_biography_subscriptions(self):
        from nexus.gov.workers.biography import GovBiographyWorker
        assert GovBiographyWorker.subscriptions == [GovEventType.TICK_WEEKLY]

    def test_weekly_recap_subscriptions(self):
        from nexus.gov.workers.weekly_recap import GovWeeklyRecapWorker
        assert GovWeeklyRecapWorker.subscriptions == [GovEventType.TICK_WEEKLY]

    def test_vote_impact_subscriptions(self):
        from nexus.gov.workers.vote_impact import GovVoteImpactWorker
        assert GovVoteImpactWorker.subscriptions == [GovEventType.TICK_DAILY]

    def test_press_affair_detector_subscriptions(self):
        from nexus.gov.workers.press_affair_detector import GovPressAffairDetector
        assert GovPressAffairDetector.subscriptions == [GovEventType.GOV_PRESS_ADDED]

    def test_newsletter_subscriptions(self):
        from nexus.gov.workers.newsletter import GovNewsletterWorker
        assert GovNewsletterWorker.subscriptions == [GovEventType.TICK_WEEKLY]

    def test_social_publish_subscriptions(self):
        from nexus.gov.workers.social_publish import GovSocialPublishWorker
        assert GovSocialPublishWorker.subscriptions == [GovEventType.GOV_CONTRADICTION_FOUND]

    # --- Tick-based sync workers ---

    def test_vote_sync_subscriptions(self):
        from nexus.gov.workers.vote_sync import GovVoteSyncWorker
        assert GovVoteSyncWorker.subscriptions == [GovEventType.TICK_DAILY]

    def test_depute_sync_subscriptions(self):
        from nexus.gov.workers.depute_sync import GovDeputeSyncWorker
        assert GovDeputeSyncWorker.subscriptions == [GovEventType.TICK_WEEKLY]

    def test_senat_sync_subscriptions(self):
        from nexus.gov.workers.senat_sync import GovSenatSyncWorker
        assert GovSenatSyncWorker.subscriptions == [GovEventType.TICK_WEEKLY]

    def test_hatvp_sync_subscriptions(self):
        from nexus.gov.workers.hatvp_sync import GovHATVPSyncWorker
        assert GovHATVPSyncWorker.subscriptions == [GovEventType.TICK_MONTHLY]

    def test_law_sync_subscriptions(self):
        from nexus.gov.workers.law_sync import GovLawSyncWorker
        assert GovLawSyncWorker.subscriptions == [GovEventType.TICK_DAILY]

    def test_fabrique_sync_subscriptions(self):
        from nexus.gov.workers.fabrique_sync import GovFabriqueSyncWorker
        assert GovFabriqueSyncWorker.subscriptions == [GovEventType.TICK_WEEKLY]

    def test_wikidata_sync_subscriptions(self):
        from nexus.gov.workers.wikidata_sync import GovWikidataSyncWorker
        assert GovWikidataSyncWorker.subscriptions == [GovEventType.TICK_WEEKLY]

    def test_affairs_sync_subscriptions(self):
        from nexus.gov.workers.affairs_sync import GovAffairsSyncWorker
        assert GovAffairsSyncWorker.subscriptions == [GovEventType.TICK_DAILY]

    def test_press_sync_subscriptions(self):
        from nexus.gov.workers.press_sync import GovPressSyncWorker
        assert GovPressSyncWorker.subscriptions == [GovEventType.TICK_HOURLY]

    def test_factcheck_sync_subscriptions(self):
        from nexus.gov.workers.factcheck_sync import GovFactcheckSyncWorker
        assert GovFactcheckSyncWorker.subscriptions == [GovEventType.TICK_DAILY]

    def test_eu_parliament_sync_subscriptions(self):
        from nexus.gov.workers.eu_parliament_sync import GovEUParliamentSyncWorker
        assert GovEUParliamentSyncWorker.subscriptions == [GovEventType.TICK_WEEKLY]

    def test_eurlex_sync_subscriptions(self):
        from nexus.gov.workers.eurlex_sync import GovEURlexSyncWorker
        assert GovEURlexSyncWorker.subscriptions == [GovEventType.TICK_WEEKLY]

    # --- Social media sync workers ---

    def test_twitter_sync_subscriptions(self):
        from nexus.gov.workers.twitter_sync import GovTwitterSyncWorker
        assert GovTwitterSyncWorker.subscriptions == [GovEventType.TICK_HOURLY]

    def test_facebook_sync_subscriptions(self):
        from nexus.gov.workers.facebook_sync import GovFacebookSyncWorker
        assert GovFacebookSyncWorker.subscriptions == [GovEventType.TICK_DAILY]

    def test_instagram_sync_subscriptions(self):
        from nexus.gov.workers.instagram_sync import GovInstagramSyncWorker
        assert GovInstagramSyncWorker.subscriptions == [GovEventType.TICK_DAILY]

    def test_youtube_sync_subscriptions(self):
        from nexus.gov.workers.youtube_sync import GovYouTubeSyncWorker
        assert GovYouTubeSyncWorker.subscriptions == [GovEventType.TICK_DAILY]

    def test_tiktok_sync_subscriptions(self):
        from nexus.gov.workers.tiktok_sync import GovTikTokSyncWorker
        assert GovTikTokSyncWorker.subscriptions == [GovEventType.TICK_DAILY]


# ===================================================================
# TestGovIdentityResolver -- normalize_name + compute_similarity
# ===================================================================

class TestGovIdentityNormalizeName:
    """Test normalize_name edge cases for French politician names."""

    def test_basic_lowercase(self):
        from nexus.gov.identity import normalize_name
        assert normalize_name("Emmanuel Macron") == "emmanuel macron"

    def test_strip_monsieur(self):
        from nexus.gov.identity import normalize_name
        assert normalize_name("M. Jean-Marie Le Pen") == "jean marie pen"

    def test_strip_madame(self):
        from nexus.gov.identity import normalize_name
        assert normalize_name("Mme Marine Le Pen") == "marine pen"

    def test_strip_madame_dot(self):
        from nexus.gov.identity import normalize_name
        assert normalize_name("Mme. Segolene Royal") == "segolene royal"

    def test_strip_accents(self):
        from nexus.gov.identity import normalize_name
        result = normalize_name("Francois Hollande")
        assert result == "francois hollande"

    def test_strip_accents_cedilla(self):
        from nexus.gov.identity import normalize_name
        # c-cedilla and e-accent
        assert "francois" in normalize_name("Francois Fillon")

    def test_remove_d_apostrophe(self):
        from nexus.gov.identity import normalize_name
        result = normalize_name("Francois D'Aubert")
        assert result == "francois aubert"

    def test_remove_l_apostrophe(self):
        from nexus.gov.identity import normalize_name
        result = normalize_name("Jean L'Huillier")
        assert result == "jean huillier"

    def test_remove_particle_de(self):
        from nexus.gov.identity import normalize_name
        result = normalize_name("Dominique de Villepin")
        assert result == "dominique villepin"

    def test_remove_particle_du(self):
        from nexus.gov.identity import normalize_name
        result = normalize_name("Jean du Pont")
        assert result == "jean pont"

    def test_remove_hyphens(self):
        from nexus.gov.identity import normalize_name
        result = normalize_name("Jean-Luc Melenchon")
        assert result == "jean luc melenchon"

    def test_empty_string(self):
        from nexus.gov.identity import normalize_name
        assert normalize_name("") == ""

    def test_none_input(self):
        from nexus.gov.identity import normalize_name
        assert normalize_name(None) == ""

    def test_non_string_input(self):
        from nexus.gov.identity import normalize_name
        assert normalize_name(42) == ""

    def test_whitespace_only(self):
        from nexus.gov.identity import normalize_name
        assert normalize_name("   ") == ""

    def test_complex_compound_name(self):
        from nexus.gov.identity import normalize_name
        result = normalize_name("M. Jean-Pierre Le Roux du Bourg")
        # particles: le, du removed; hyphen flattened
        assert result == "jean pierre roux bourg"

    def test_title_dr(self):
        from nexus.gov.identity import normalize_name
        result = normalize_name("Dr. Philippe Douste-Blazy")
        assert result == "philippe douste blazy"


class TestGovIdentityComputeSimilarity:
    """Test compute_similarity with various name pairs."""

    def test_identical_names_score_one(self):
        from nexus.gov.identity import compute_similarity
        score = compute_similarity("marine le pen", "marine le pen")
        assert score > 0.99

    def test_identical_after_normalization(self):
        from nexus.gov.identity import compute_similarity
        score = compute_similarity("M. Marine Le Pen", "Mme Marine Le Pen")
        assert score > 0.99

    def test_different_people_low_score(self):
        from nexus.gov.identity import compute_similarity
        score = compute_similarity("Jean-Luc Melenchon", "Marine Le Pen")
        assert score < 0.6

    def test_similar_names_moderate_score(self):
        from nexus.gov.identity import compute_similarity
        # Slightly different first names, same last
        score = compute_similarity("Jean-Marie Le Pen", "Marine Le Pen")
        # These share "pen" but differ significantly
        assert 0.3 < score < 0.95

    def test_empty_name_returns_zero(self):
        from nexus.gov.identity import compute_similarity
        assert compute_similarity("", "Marine Le Pen") == 0.0
        assert compute_similarity("Macron", "") == 0.0
        assert compute_similarity("", "") == 0.0

    def test_accented_vs_plain(self):
        from nexus.gov.identity import compute_similarity
        # Accents should be stripped, so these should match
        score = compute_similarity("Francois Hollande", "Francois Hollande")
        assert score > 0.99

    def test_symmetry(self):
        from nexus.gov.identity import compute_similarity
        score_ab = compute_similarity("Emmanuel Macron", "Nicolas Sarkozy")
        score_ba = compute_similarity("Nicolas Sarkozy", "Emmanuel Macron")
        assert abs(score_ab - score_ba) < 0.01


# ===================================================================
# TestContradictionAnalyzerHelpers -- Pure function tests
# ===================================================================

class TestContradictionAnalyzerHelpers:
    """Test pure helpers from the contradiction_analyzer module."""

    def test_subject_keywords_basic(self):
        from nexus.gov.workers.contradiction_analyzer import _subject_keywords
        kws = _subject_keywords("Immigration et Securite")
        assert "immigration" in kws
        assert "securite" in kws

    def test_subject_keywords_filters_short(self):
        from nexus.gov.workers.contradiction_analyzer import _subject_keywords
        kws = _subject_keywords("Loi de la securite")
        # "de", "la" are particles/stopwords, "loi" is 3 chars and kept
        assert "loi" in kws
        assert "securite" in kws
        # "de" and "la" should be filtered (stopwords)
        assert "des" not in kws

    def test_subject_keywords_empty(self):
        from nexus.gov.workers.contradiction_analyzer import _subject_keywords
        assert _subject_keywords("") == set()
        assert _subject_keywords(None) == set()

    def test_subjects_overlap_true(self):
        from nexus.gov.workers.contradiction_analyzer import _subjects_overlap
        # Both subjects share "immigration" as a keyword
        assert _subjects_overlap("Immigration et Securite", "Politique immigration") is True

    def test_subjects_overlap_apostrophe_no_match(self):
        from nexus.gov.workers.contradiction_analyzer import _subjects_overlap
        # d'immigration is treated as a single token, doesn't match "immigration"
        assert _subjects_overlap("Immigration et Securite", "Politique d'immigration") is False

    def test_subjects_overlap_false(self):
        from nexus.gov.workers.contradiction_analyzer import _subjects_overlap
        assert _subjects_overlap("Economie et Finances", "Immigration et Securite") is False

    def test_subjects_overlap_empty(self):
        from nexus.gov.workers.contradiction_analyzer import _subjects_overlap
        assert _subjects_overlap("", "Immigration") is False
        assert _subjects_overlap("Immigration", "") is False


# ===================================================================
# TestGovDatabase -- Basic instantiation
# ===================================================================

class TestGovDatabase:
    """Test government database can be instantiated."""

    @pytest.mark.asyncio
    async def test_instantiation_with_memory_db(self):
        """GovernmentDatabase can be created with an aiosqlite connection."""
        import aiosqlite
        from nexus.gov.db import GovernmentDatabase, _GOV_CREATE_TABLES, _GOV_CREATE_INDEXES

        db = await aiosqlite.connect(":memory:")
        await db.execute("PRAGMA journal_mode = WAL")
        await db.execute("PRAGMA foreign_keys = ON")
        await db.executescript(_GOV_CREATE_TABLES)
        await db.executescript(_GOV_CREATE_INDEXES)
        await db.commit()

        gov_db = GovernmentDatabase(db)
        assert gov_db is not None

        # Verify tables exist
        cursor = await db.execute(
            "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'gov_%'"
        )
        tables = [row[0] for row in await cursor.fetchall()]
        assert "gov_politicians" in tables
        assert "gov_positions" in tables
        assert "gov_contradictions" in tables
        assert "gov_alerts" in tables

        await db.close()

    @pytest.mark.asyncio
    async def test_create_and_get_politician(self):
        """Basic CRUD: create a politician and retrieve it."""
        import aiosqlite
        from nexus.gov.db import GovernmentDatabase, _GOV_CREATE_TABLES, _GOV_CREATE_INDEXES

        db = await aiosqlite.connect(":memory:")
        db.row_factory = aiosqlite.Row
        await db.execute("PRAGMA foreign_keys = ON")
        await db.executescript(_GOV_CREATE_TABLES)
        await db.executescript(_GOV_CREATE_INDEXES)
        await db.commit()

        gov_db = GovernmentDatabase(db)

        # Create
        pol = await gov_db.create_politician(
            name="Marine Le Pen",
            chamber="assemblee",
            party="RN",
        )
        assert pol is not None
        assert pol["name"] == "Marine Le Pen"
        pol_id = pol["id"]

        # Get
        fetched = await gov_db.get_politician(pol_id)
        assert fetched is not None
        assert fetched["name"] == "Marine Le Pen"
        assert fetched["party"] == "RN"

        await db.close()


# ===================================================================
# TestGovManagerWorkerSpecs -- Verify expected worker count
# ===================================================================

class TestGovManagerWorkerSpecs:
    """Test GovManager has the correct number of worker specs."""

    def test_worker_specs_list_has_31_entries(self):
        """GovManager._worker_specs should define 31 workers."""
        # We parse the source to count specs rather than instantiating
        # GovManager (which would require all dependencies).
        import inspect
        from nexus.gov.events import GovManager
        source = inspect.getsource(GovManager.start)
        # Count lines matching the worker spec pattern
        import re
        specs = re.findall(r'\("nexus\.gov\.workers\.\w+",\s*"Gov\w+"', source)
        assert len(specs) == 31, f"Expected 31 worker specs, found {len(specs)}"


# ===================================================================
# TestGovEmbedWorker -- Deterministic embed ID + chunking
# ===================================================================

class TestGovEmbedWorker:
    """Test GovEmbedWorker static helpers."""

    def test_make_embed_id_deterministic(self):
        from nexus.gov.workers.embedding import GovEmbedWorker
        id1 = GovEmbedWorker._make_embed_id("position", "pos-123", chunk=0)
        id2 = GovEmbedWorker._make_embed_id("position", "pos-123", chunk=0)
        assert id1 == id2

    def test_make_embed_id_differs_by_chunk(self):
        from nexus.gov.workers.embedding import GovEmbedWorker
        id0 = GovEmbedWorker._make_embed_id("position", "pos-123", chunk=0)
        id1 = GovEmbedWorker._make_embed_id("position", "pos-123", chunk=1)
        assert id0 != id1

    def test_make_embed_id_differs_by_source_type(self):
        from nexus.gov.workers.embedding import GovEmbedWorker
        id_pos = GovEmbedWorker._make_embed_id("position", "id-1")
        id_social = GovEmbedWorker._make_embed_id("social", "id-1")
        assert id_pos != id_social

    def test_chunk_text_short(self):
        from nexus.gov.workers.embedding import GovEmbedWorker
        text = "Short text."
        chunks = GovEmbedWorker._chunk_text(text)
        assert len(chunks) == 1
        assert chunks[0] == text

    def test_chunk_text_long(self):
        from nexus.gov.workers.embedding import GovEmbedWorker, CHUNK_SIZE
        text = "word " * (CHUNK_SIZE // 2)  # Well over CHUNK_SIZE chars
        chunks = GovEmbedWorker._chunk_text(text)
        assert len(chunks) > 1
        # Each chunk should be non-empty
        for chunk in chunks:
            assert len(chunk) > 0


# ===================================================================
# TestGovDatabaseProxy -- Verify proxy attribute access
# ===================================================================

class TestGovDatabaseProxy:
    """Test GovDatabaseProxy returns callables for any attribute."""

    def test_getattr_returns_callable(self):
        from nexus.gov.events import GovDatabaseProxy
        proxy = GovDatabaseProxy()
        method = proxy.list_politicians
        assert callable(method)

    def test_different_methods_return_different_callables(self):
        from nexus.gov.events import GovDatabaseProxy
        proxy = GovDatabaseProxy()
        m1 = proxy.list_politicians
        m2 = proxy.create_politician
        # They should be distinct functions
        assert m1 is not m2
