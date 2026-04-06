"""Tests for configuration (nexus.config.Settings)."""

import pytest

from nexus.config import Settings


# =====================================================================
# Default values
# =====================================================================


class TestSettingsDefaults:

    def test_fastapi_defaults(self):
        s = Settings()
        assert s.nexus_host == "0.0.0.0"
        assert s.nexus_port == 8000
        assert s.nexus_debug is True

    def test_model_defaults(self):
        s = Settings()
        assert s.model_fast == "gemma4:e4b"
        assert s.model_reasoning == "huihui_ai/deepseek-r1-abliterated:14b"
        assert s.model_deep == "nexus"
        assert s.model_embedding == "nomic-embed-text"
        assert s.model_audio == "voxtral-mini:4b"

    def test_neo4j_defaults(self):
        s = Settings()
        assert s.neo4j_uri == "bolt://localhost:7687"
        assert s.neo4j_user == "neo4j"
        assert s.neo4j_password == "nexus2026"

    def test_chromadb_defaults(self):
        s = Settings()
        assert s.chroma_host == "localhost"
        assert s.chroma_port == 8100

    def test_search_defaults(self):
        s = Settings()
        assert s.searxng_url == "http://localhost:8888"
        assert s.robin_url == "http://localhost:9090"

    def test_ollama_default(self):
        s = Settings()
        assert s.ollama_base_url == "http://localhost:11434"

    def test_rag_defaults(self):
        s = Settings()
        assert s.rag_chunk_size == 512
        assert s.rag_chunk_overlap == 128
        assert s.rag_top_k == 20
        assert s.rag_min_reliability == 0

    def test_monitoring_intervals(self):
        s = Settings()
        assert s.clearweb_interval == 6 * 3600
        assert s.darkweb_interval == 24 * 3600

    def test_investigation_loop_defaults(self):
        s = Settings()
        assert s.investigation_cycle_minutes == 30
        assert s.auto_ingest_relevance_threshold == 50.0
        assert s.full_reevaluation_every_n_cycles == 6
        assert s.max_auto_ingest_per_cycle == 5
        assert s.max_new_queries_per_cycle == 3

    def test_auto_module_flags(self):
        s = Settings()
        assert s.auto_osint_recon is True
        assert s.auto_geocode is True
        assert s.auto_image_analysis is True
        assert s.auto_forensic_analysis is True
        assert s.auto_visual_embeddings is True
        assert s.auto_domain_recon is True
        assert s.auto_timeline_rebuild is True

    def test_storage_paths(self):
        s = Settings()
        assert str(s.data_dir).endswith("data")
        assert str(s.upload_dir).endswith("uploads")
        assert str(s.sqlite_path).endswith("nexus.db")
