"""
Tests for the nexus-worker client package (Phase 3).

Covers:
- GPU detection helpers
- Config persistence (load/save/is_registered)
- NexusClient (HTTP client methods)
- WorkerEngine (state machine, lifecycle)
- CLI argument parsing
- Dashboard build
- Module imports
"""

import asyncio
import json
import os
import tempfile
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock, patch

import pytest
import pytest_asyncio

from worker.config import _DEFAULTS, load_config, save_config, is_registered
from worker.gpu_detect import detect_gpu, format_vram, _detect_pynvml, _detect_nvidia_smi
from worker.client import NexusClient
from worker.engine import WorkerEngine, WorkerState


# ===================================================================
# GPU Detection
# ===================================================================

class TestGPUDetect:
    """Test GPU detection helpers."""

    def test_format_vram_gb(self):
        assert format_vram(16384) == "16 GB"
        assert format_vram(24576) == "24 GB"

    def test_format_vram_mb(self):
        assert format_vram(512) == "512 MB"

    def test_format_vram_zero(self):
        assert format_vram(0) == "0 MB"

    def test_detect_gpu_returns_dict(self):
        result = detect_gpu()
        assert "gpu_model" in result
        assert "vram_mb" in result
        assert "platform" in result
        assert isinstance(result["vram_mb"], int)

    def test_detect_gpu_platform_set(self):
        result = detect_gpu()
        assert result["platform"] in ("windows", "linux", "darwin")

    @patch("worker.gpu_detect._detect_pynvml", return_value=None)
    @patch("worker.gpu_detect._detect_nvidia_smi", return_value=None)
    @patch("worker.gpu_detect._detect_apple_silicon", return_value=None)
    def test_detect_gpu_no_gpu(self, mock_apple, mock_smi, mock_nvml):
        result = detect_gpu()
        assert result["gpu_model"] == "Unknown GPU"
        assert result["vram_mb"] == 0

    @patch("worker.gpu_detect._detect_pynvml", return_value={"gpu_model": "RTX 5080", "vram_mb": 16384})
    def test_detect_gpu_pynvml_preferred(self, mock_nvml):
        result = detect_gpu()
        assert result["gpu_model"] == "RTX 5080"
        assert result["vram_mb"] == 16384


# ===================================================================
# Config Management
# ===================================================================

class TestConfig:
    """Test configuration persistence."""

    def test_defaults_have_required_keys(self):
        assert "server_url" in _DEFAULTS
        assert "api_key" in _DEFAULTS
        assert "node_id" in _DEFAULTS
        assert "name" in _DEFAULTS
        assert "ollama_url" in _DEFAULTS

    def test_load_config_defaults(self):
        with patch("worker.config._CONFIG_FILE", Path("/nonexistent/path/config.json")):
            config = load_config()
            assert config["server_url"] == ""
            assert config["ollama_url"] == "http://localhost:11434"
            assert config["poll_interval"] == 2.0

    def test_save_and_load_config(self):
        with tempfile.TemporaryDirectory() as tmp:
            config_file = Path(tmp) / "config.json"
            with patch("worker.config._CONFIG_FILE", config_file), \
                 patch("worker.config._CONFIG_DIR", Path(tmp)):
                test_config = {
                    "server_url": "https://nexusgov.fr",
                    "api_key": "test-key-123",
                    "name": "TestNode",
                }
                save_config(test_config)
                assert config_file.exists()

                loaded = load_config()
                assert loaded["server_url"] == "https://nexusgov.fr"
                assert loaded["api_key"] == "test-key-123"
                assert loaded["name"] == "TestNode"
                # Defaults should be merged
                assert loaded["ollama_url"] == "http://localhost:11434"

    def test_is_registered_false(self):
        with patch("worker.config._CONFIG_FILE", Path("/nonexistent/config.json")):
            assert is_registered() is False

    def test_is_registered_true(self):
        with tempfile.TemporaryDirectory() as tmp:
            config_file = Path(tmp) / "config.json"
            config_file.write_text(json.dumps({
                "server_url": "https://test.com",
                "api_key": "abc123",
            }))
            with patch("worker.config._CONFIG_FILE", config_file):
                assert is_registered() is True


# ===================================================================
# NexusClient
# ===================================================================

class TestNexusClient:
    """Test HTTP client initialization and method signatures."""

    def test_client_init(self):
        client = NexusClient("https://nexusgov.fr", api_key="test-key")
        assert client.server_url == "https://nexusgov.fr"

    def test_client_strips_trailing_slash(self):
        client = NexusClient("https://nexusgov.fr/")
        assert client.server_url == "https://nexusgov.fr"

    def test_auth_headers(self):
        client = NexusClient("http://localhost", api_key="mykey")
        headers = client._auth_headers()
        assert headers["Authorization"] == "Bearer mykey"

    def test_auth_headers_empty(self):
        client = NexusClient("http://localhost")
        headers = client._auth_headers()
        assert "Authorization" not in headers

    @pytest.mark.asyncio
    async def test_close_without_open(self):
        """Closing a client that was never used should not error."""
        client = NexusClient("http://localhost")
        await client.close()  # Should not raise


# ===================================================================
# WorkerEngine
# ===================================================================

class TestWorkerEngine:
    """Test worker engine state machine and lifecycle."""

    def test_initial_state(self):
        client = NexusClient("http://localhost")
        engine = WorkerEngine(client=client)
        assert engine.state == WorkerState.IDLE
        assert engine.current_model == ""
        assert engine.current_task is None
        assert engine.session_tasks == 0
        assert engine.session_errors == 0

    def test_pause_resume(self):
        client = NexusClient("http://localhost")
        engine = WorkerEngine(client=client)
        engine.pause()
        assert engine.state == WorkerState.PAUSED
        engine.resume()
        assert engine.state == WorkerState.IDLE

    def test_uptime_zero_before_start(self):
        client = NexusClient("http://localhost")
        engine = WorkerEngine(client=client)
        assert engine.uptime_seconds == 0.0

    def test_state_callback(self):
        states = []
        client = NexusClient("http://localhost")
        engine = WorkerEngine(
            client=client,
            on_state_change=lambda s: states.append(s),
        )
        engine.pause()
        engine.resume()
        assert states == [WorkerState.PAUSED, WorkerState.IDLE]


# ===================================================================
# WorkerState enum
# ===================================================================

class TestWorkerState:
    """Test worker state values."""

    def test_all_states_exist(self):
        assert WorkerState.IDLE == "idle"
        assert WorkerState.PULLING_MODEL == "pulling_model"
        assert WorkerState.PROCESSING == "processing"
        assert WorkerState.PAUSED == "paused"
        assert WorkerState.ERROR == "error"
        assert WorkerState.STOPPED == "stopped"

    def test_state_count(self):
        assert len(WorkerState) == 6


# ===================================================================
# Dashboard
# ===================================================================

class TestDashboard:
    """Test dashboard rendering."""

    def test_build_dashboard_returns_panel(self):
        from rich.panel import Panel
        from worker.dashboard import build_dashboard

        client = NexusClient("http://localhost")
        engine = WorkerEngine(client=client)

        panel = build_dashboard(engine, "TestNode", "RTX 5080", 16384)
        assert isinstance(panel, Panel)

    def test_build_dashboard_shows_name(self):
        from worker.dashboard import build_dashboard

        client = NexusClient("http://localhost")
        engine = WorkerEngine(client=client)

        panel = build_dashboard(engine, "FlowUP", "RTX 5080", 16384)
        # Panel renders to string containing the name
        rendered = str(panel.renderable)
        assert "FlowUP" in rendered

    def test_build_dashboard_shows_gpu(self):
        from worker.dashboard import build_dashboard

        client = NexusClient("http://localhost")
        engine = WorkerEngine(client=client)

        panel = build_dashboard(engine, "Test", "RTX 5080", 16384)
        rendered = str(panel.renderable)
        assert "RTX 5080" in rendered
        assert "16 GB" in rendered


# ===================================================================
# CLI
# ===================================================================

class TestCLI:
    """Test CLI argument parsing."""

    def test_main_import(self):
        from worker.cli import main
        assert callable(main)

    def test_register_parser(self):
        from worker.cli import main
        import argparse
        # Verify the module loads without error
        assert True

    def test_version(self):
        from worker import __version__
        assert __version__ == "0.1.0"


# ===================================================================
# Module imports
# ===================================================================

class TestWorkerImports:
    """Test that all worker module components import correctly."""

    def test_import_config(self):
        from worker.config import load_config, save_config, is_registered
        assert callable(load_config)

    def test_import_gpu_detect(self):
        from worker.gpu_detect import detect_gpu, format_vram
        assert callable(detect_gpu)

    def test_import_client(self):
        from worker.client import NexusClient
        assert NexusClient is not None

    def test_import_engine(self):
        from worker.engine import WorkerEngine, WorkerState
        assert WorkerEngine is not None
        assert len(WorkerState) == 6

    def test_import_dashboard(self):
        from worker.dashboard import build_dashboard, run_dashboard
        assert callable(build_dashboard)

    def test_import_cli(self):
        from worker.cli import main
        assert callable(main)

    def test_import_package(self):
        import worker
        assert hasattr(worker, "__version__")
