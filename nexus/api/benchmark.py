"""
NEXUS -- Benchmark API.

Endpoints for listing available benchmarks and injecting evidence.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from fastapi import APIRouter, BackgroundTasks, Depends
from loguru import logger

from nexus.api.deps import get_database, get_evidence_processor
from nexus.db.sqlite_db import Database

router = APIRouter(prefix="/api/benchmark", tags=["benchmark"])

BENCHMARK_DIR = Path(__file__).resolve().parent.parent.parent / "data" / "benchmark"

KNOWN_BENCHMARKS = {
    "kulik": {"dir": "kulik", "name": "Affaire Elodie Kulik (2002)"},
    "gsk": {"dir": "golden-state-killer", "name": "Golden State Killer (1974-86)"},
    "moreau": {"dir": "affaire-moreau", "name": "Affaire Moreau (fictif)"},
}


@router.get("/available")
async def list_available() -> list[dict[str, Any]]:
    """List available benchmark cases with their evidence counts."""
    result = []
    for key, info in KNOWN_BENCHMARKS.items():
        manifest_path = BENCHMARK_DIR / info["dir"] / "manifest.json"
        if manifest_path.exists():
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            evidence = manifest.get("evidence", [])
            waves = sorted(set(e.get("wave", 1) for e in evidence))
            result.append({
                "key": key,
                "name": info["name"],
                "evidence_count": len(evidence),
                "waves": len(waves),
                "has_ground_truth": "ground_truth" in manifest,
            })
    return result


@router.post("/launch/{bench_key}")
async def launch_benchmark(
    bench_key: str,
    background_tasks: BackgroundTasks,
    db: Database = Depends(get_database),
) -> dict[str, Any]:
    """Create a case and start injecting evidence for a benchmark."""
    if bench_key not in KNOWN_BENCHMARKS:
        from fastapi import HTTPException
        raise HTTPException(404, f"Benchmark '{bench_key}' not found")

    info = KNOWN_BENCHMARKS[bench_key]
    manifest_path = BENCHMARK_DIR / info["dir"] / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    case_data = manifest.get("case", {})

    # Create the case
    case = await db.create_case(
        name=case_data.get("name", info["name"]),
        reference=case_data.get("reference", f"#BENCH-{bench_key.upper()}"),
        description=case_data.get("description", ""),
    )
    case_id = case["id"]

    # Start background injection of wave 1
    background_tasks.add_task(
        _inject_wave, case_id, bench_key, wave=1
    )

    return {
        "case_id": case_id,
        "name": case["name"],
        "status": "injecting_wave_1",
        "total_evidence": len(manifest.get("evidence", [])),
    }


@router.post("/inject/{case_id}/{bench_key}/wave/{wave}")
async def inject_wave(
    case_id: str,
    bench_key: str,
    wave: int,
    background_tasks: BackgroundTasks,
) -> dict[str, Any]:
    """Inject a specific wave of evidence in background."""
    if bench_key not in KNOWN_BENCHMARKS:
        from fastapi import HTTPException
        raise HTTPException(404, f"Benchmark '{bench_key}' not found")

    background_tasks.add_task(_inject_wave, case_id, bench_key, wave)
    return {"status": "injecting", "wave": wave}


async def _inject_wave(case_id: str, bench_key: str, wave: int) -> None:
    """Background task: inject all evidence for a wave."""
    from nexus.db.sqlite_db import get_db, Database
    from nexus.core.evidence_processor import EvidenceProcessor
    from nexus.llm.ollama_client import OllamaClient
    from nexus.llm.router import LLMRouter
    from nexus.config import settings

    info = KNOWN_BENCHMARKS[bench_key]
    manifest_path = BENCHMARK_DIR / info["dir"] / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

    wave_evidence = [e for e in manifest.get("evidence", []) if e.get("wave", 1) == wave]

    logger.info("Benchmark inject: case={} bench={} wave={} evidence={}",
                case_id[:8], bench_key, wave, len(wave_evidence))

    ollama = OllamaClient()
    router = LLMRouter(ollama)

    for ev in wave_evidence:
        file_path = BENCHMARK_DIR / info["dir"] / ev.get("file", "")
        if not file_path.exists():
            logger.warning("Benchmark file missing: {}", file_path)
            continue

        text = file_path.read_text(encoding="utf-8")

        try:
            async with get_db() as conn:
                db = Database(conn)
                processor = EvidenceProcessor(db=db, router=router, upload_dir=settings.upload_dir)
                await processor.process_text_input(
                    case_id=case_id,
                    title=ev.get("title", file_path.stem),
                    text=text,
                    source=ev.get("source", "Benchmark"),
                )
            logger.info("Benchmark injected: {}", ev.get("title", "?")[:50])
        except Exception as exc:
            logger.error("Benchmark inject failed: {} — {}", ev.get("title", "?")[:50], exc)

    logger.info("Benchmark wave {} complete for case {}", wave, case_id[:8])
