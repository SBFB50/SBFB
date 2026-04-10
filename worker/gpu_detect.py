"""
nexus-worker — GPU detection.

Detects GPU model and VRAM using multiple backends:
1. nvidia-ml-py (pynvml) — preferred for NVIDIA GPUs
2. nvidia-smi CLI — fallback for NVIDIA
3. Platform detection — for Apple Silicon (unified memory)

Returns a dict: {"gpu_model": str, "vram_mb": int, "platform": str}
"""

from __future__ import annotations

import platform
import re
import shutil
import subprocess
from typing import Any, Optional


def detect_gpu() -> dict[str, Any]:
    """Detect GPU model, VRAM, and platform.

    Tries multiple methods in order of reliability.
    Returns {"gpu_model": str, "vram_mb": int, "platform": str}.
    """
    plat = platform.system().lower()

    # Try pynvml first (most reliable for NVIDIA)
    result = _detect_pynvml()
    if result:
        result["platform"] = plat
        return result

    # Fallback: nvidia-smi CLI
    result = _detect_nvidia_smi()
    if result:
        result["platform"] = plat
        return result

    # Apple Silicon detection
    if plat == "darwin":
        result = _detect_apple_silicon()
        if result:
            result["platform"] = plat
            return result

    # Unknown GPU
    return {
        "gpu_model": "Unknown GPU",
        "vram_mb": 0,
        "platform": plat,
    }


def _detect_pynvml() -> Optional[dict]:
    """Detect NVIDIA GPU via nvidia-ml-py (pynvml)."""
    try:
        import pynvml
        pynvml.nvmlInit()
        handle = pynvml.nvmlDeviceGetHandleByIndex(0)
        name = pynvml.nvmlDeviceGetName(handle)
        if isinstance(name, bytes):
            name = name.decode("utf-8")
        mem_info = pynvml.nvmlDeviceGetMemoryInfo(handle)
        vram_mb = mem_info.total // (1024 * 1024)
        pynvml.nvmlShutdown()
        return {"gpu_model": name, "vram_mb": vram_mb}
    except Exception:
        return None


def _detect_nvidia_smi() -> Optional[dict]:
    """Detect NVIDIA GPU via nvidia-smi CLI."""
    if not shutil.which("nvidia-smi"):
        return None
    try:
        result = subprocess.run(
            ["nvidia-smi", "--query-gpu=name,memory.total", "--format=csv,noheader,nounits"],
            capture_output=True, text=True, timeout=10,
        )
        if result.returncode != 0:
            return None
        line = result.stdout.strip().split("\n")[0]
        parts = [p.strip() for p in line.split(",")]
        if len(parts) >= 2:
            name = parts[0]
            vram_mb = int(float(parts[1]))
            return {"gpu_model": name, "vram_mb": vram_mb}
    except Exception:
        pass
    return None


def _detect_apple_silicon() -> Optional[dict]:
    """Detect Apple Silicon unified memory."""
    try:
        result = subprocess.run(
            ["sysctl", "-n", "hw.memsize"],
            capture_output=True, text=True, timeout=5,
        )
        if result.returncode == 0:
            total_bytes = int(result.stdout.strip())
            # Apple Silicon uses unified memory — GPU gets ~75% of total
            unified_mb = total_bytes // (1024 * 1024)
            gpu_mb = int(unified_mb * 0.75)

            # Get chip name
            chip_result = subprocess.run(
                ["sysctl", "-n", "machdep.cpu.brand_string"],
                capture_output=True, text=True, timeout=5,
            )
            chip = chip_result.stdout.strip() if chip_result.returncode == 0 else "Apple Silicon"

            return {"gpu_model": chip, "vram_mb": gpu_mb}
    except Exception:
        pass
    return None


def format_vram(vram_mb: int) -> str:
    """Format VRAM in human-readable form."""
    if vram_mb >= 1024:
        return f"{vram_mb / 1024:.0f} GB"
    return f"{vram_mb} MB"
