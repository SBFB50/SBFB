"""
NEXUS -- Benchmark API.

Endpoints for listing available benchmarks and injecting evidence.
"""

from __future__ import annotations

import asyncio
import json
from pathlib import Path
from typing import Any

from fastapi import APIRouter, BackgroundTasks, Depends, Request
from loguru import logger

from nexus.api.deps import get_database, get_evidence_processor
from nexus.db.sqlite_db import Database

router = APIRouter(prefix="/api/benchmark", tags=["benchmark"])

# Global lock to serialise wave injection — prevents VRAM saturation
_INJECT_LOCK = asyncio.Lock()

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
    request: Request,
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

    # Capture app state singletons NOW (background tasks can't access request)
    app_state = {
        "router": getattr(request.app.state, "router", None),
        "neo4j": getattr(request.app.state, "neo4j", None),
        "chroma": getattr(request.app.state, "chroma", None),
        "entity_extractor": getattr(request.app.state, "entity_extractor", None),
        "investigation_manager": getattr(request.app.state, "investigation_manager", None),
    }

    # Start FULL benchmark pipeline in background
    background_tasks.add_task(
        _run_full_benchmark, case_id, bench_key, app_state
    )

    return {
        "case_id": case_id,
        "name": case["name"],
        "status": "running_full_pipeline",
        "total_evidence": len(manifest.get("evidence", [])),
    }


@router.post("/inject/{case_id}/{bench_key}/wave/{wave}")
async def inject_wave(
    case_id: str,
    bench_key: str,
    wave: int,
    request: Request,
    background_tasks: BackgroundTasks,
) -> dict[str, Any]:
    """Inject a specific wave of evidence in background."""
    if bench_key not in KNOWN_BENCHMARKS:
        from fastapi import HTTPException
        raise HTTPException(404, f"Benchmark '{bench_key}' not found")

    app_state = {
        "router": getattr(request.app.state, "router", None),
        "neo4j": getattr(request.app.state, "neo4j", None),
        "chroma": getattr(request.app.state, "chroma", None),
        "entity_extractor": getattr(request.app.state, "entity_extractor", None),
    }
    background_tasks.add_task(_inject_wave, case_id, bench_key, wave, app_state)
    return {"status": "injecting", "wave": wave}


async def _inject_wave(case_id: str, bench_key: str, wave: int, app_state: dict | None = None) -> None:
    """Background task: inject all evidence for a wave.

    Serialised via ``_INJECT_LOCK`` so that only one wave runs at a
    time, preventing VRAM saturation from parallel LLM calls.
    Reuses shared singletons passed from the endpoint via app_state.
    """
    async with _INJECT_LOCK:
        from nexus.db.sqlite_db import get_db, Database
        from nexus.core.evidence_processor import EvidenceProcessor
        from nexus.config import settings

        app_state = app_state or {}
        router = app_state.get("router")
        if router is None:
            from nexus.llm.ollama_client import OllamaClient
            from nexus.llm.router import LLMRouter
            router = LLMRouter(OllamaClient())

        entity_extractor = app_state.get("entity_extractor")
        neo4j = app_state.get("neo4j")
        chroma = app_state.get("chroma")

        info = KNOWN_BENCHMARKS[bench_key]
        manifest_path = BENCHMARK_DIR / info["dir"] / "manifest.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

        wave_evidence = [e for e in manifest.get("evidence", []) if e.get("wave", 1) == wave]

        logger.info("Benchmark inject: case={} bench={} wave={} evidence={}",
                    case_id[:8], bench_key, wave, len(wave_evidence))

        for ev in wave_evidence:
            file_path = BENCHMARK_DIR / info["dir"] / ev.get("file", "")
            if not file_path.exists():
                logger.warning("Benchmark file missing: {}", file_path)
                continue

            text = file_path.read_text(encoding="utf-8")

            try:
                async with get_db() as conn:
                    db = Database(conn)
                    processor = EvidenceProcessor(
                        db=db,
                        router=router,
                        upload_dir=settings.upload_dir,
                        neo4j=neo4j,
                        chroma=chroma,
                        entity_extractor=entity_extractor,
                    )
                    await processor.process_text_input(
                        case_id=case_id,
                        title=ev.get("title", file_path.stem),
                        text=text,
                        source=ev.get("source", "Benchmark"),
                    )
                logger.info("Benchmark injected: {}", ev.get("title", "?")[:50])
            except Exception as exc:
                logger.error("Benchmark inject failed: {} — {}", ev.get("title", "?")[:50], exc)

            # Let the GPU breathe between evidence items
            await asyncio.sleep(2)

        logger.info("Benchmark wave {} complete for case {}", wave, case_id[:8])


async def _run_full_benchmark(case_id: str, bench_key: str, app_state: dict | None = None) -> None:
    """Full benchmark pipeline: inject all waves → analyze → hypotheses → suspects.

    Reuses shared singletons passed from the endpoint via app_state
    to respect VRAM serialization and avoid redundant connections.
    """
    from nexus.db.sqlite_db import get_db, Database

    app_state = app_state or {}
    router = app_state.get("router")
    if router is None:
        from nexus.llm.ollama_client import OllamaClient
        from nexus.llm.router import LLMRouter
        router = LLMRouter(OllamaClient())

    neo4j = app_state.get("neo4j")
    chroma = app_state.get("chroma")
    entity_extractor = app_state.get("entity_extractor")

    info = KNOWN_BENCHMARKS[bench_key]
    manifest_path = BENCHMARK_DIR / info["dir"] / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    waves = sorted(set(e.get("wave", 1) for e in manifest.get("evidence", [])))

    # 1. Inject all waves sequentially
    for wave in waves:
        logger.info("Benchmark full: injecting wave {} for case {}", wave, case_id[:8])
        await _inject_wave(case_id, bench_key, wave, app_state)
        await asyncio.sleep(3)

    logger.info("Benchmark full: all waves injected for case {}", case_id[:8])

    # 2. Analyze
    try:
        async with get_db() as conn:
            db = Database(conn)
            from nexus.core.analysis_pipeline import AnalysisPipeline
            pipeline = AnalysisPipeline(db=db, router=router, chroma=chroma, neo4j=neo4j)
            logger.info("Benchmark full: running analysis...")
            await pipeline.run_full_analysis(case_id)
    except Exception as exc:
        logger.error("Benchmark analysis failed: {}", exc)

    await asyncio.sleep(2)

    # 3. Generate hypotheses
    try:
        async with get_db() as conn:
            db = Database(conn)
            from nexus.core.hypothesis_engine import HypothesisEngine
            engine = HypothesisEngine(db=db, router=router, chroma=chroma, neo4j=neo4j)
            logger.info("Benchmark full: generating hypotheses...")
            await engine.generate_hypotheses(case_id)
    except Exception as exc:
        logger.error("Benchmark hypothesis generation failed: {}", exc)

    await asyncio.sleep(2)

    # 4. Detect contradictions
    try:
        async with get_db() as conn:
            db = Database(conn)
            from nexus.core.contradiction_detector import ContradictionDetector
            detector = ContradictionDetector(db=db, router=router)
            logger.info("Benchmark full: detecting contradictions...")
            await detector.detect_contradictions(case_id)
    except Exception as exc:
        logger.error("Benchmark contradiction detection failed: {}", exc)

    await asyncio.sleep(2)

    # 5. Score suspects
    try:
        async with get_db() as conn:
            db = Database(conn)
            from nexus.core.suspect_scorer import SuspectScorer
            scorer = SuspectScorer(db=db, router=router, neo4j=neo4j)
            logger.info("Benchmark full: scoring suspects...")
            await scorer.score_all_suspects(case_id, trigger="benchmark")
    except Exception as exc:
        logger.error("Benchmark suspect scoring failed: {}", exc)

    # 6. Neo4j sync (uses shared neo4j client)
    if neo4j is not None:
        try:
            async with get_db() as conn:
                db = Database(conn)
                evidence = await db.list_evidence_by_case(case_id)
                entities = await db.list_entities_by_case(case_id)
                for ev in evidence:
                    if ev.get("status") == "processed":
                        await neo4j.sync_evidence(
                            ev["id"], case_id, ev["title"],
                            ev["evidence_type"], ev.get("reliability", 50),
                        )
                for ent in entities:
                    await neo4j.sync_entity(ent, case_id)
                # Link only via actual mentions
                for ent in entities:
                    mentions = await db.list_mentions_by_entity(ent["id"])
                    for m in mentions:
                        await neo4j.link_evidence_to_entity(m["evidence_id"], ent["id"])
            logger.info("Benchmark full: Neo4j synced")
        except Exception as exc:
            logger.error("Benchmark Neo4j sync failed: {}", exc)

    # 7. Start autonomous investigation loop via shared manager
    try:
        inv_manager = app_state.get("investigation_manager")
        if inv_manager is not None:
            started = await inv_manager.start_investigation(case_id)
            if started:
                logger.info("Benchmark full: autonomous investigation STARTED via shared manager")
            else:
                logger.info("Benchmark full: investigation already running for case {}", case_id[:8])
        else:
            # Fallback: create a standalone manager
            from nexus.core.investigation_manager import InvestigationManager
            manager = InvestigationManager(
                router=router, chroma=chroma, neo4j=neo4j,
                entity_extractor=entity_extractor,
            )
            await manager.start_investigation(case_id)
            logger.info("Benchmark full: autonomous investigation STARTED (standalone manager)")
    except Exception as exc:
        logger.error("Benchmark autonomous loop start failed: {}", exc)

    logger.info("=== BENCHMARK PIPELINE COMPLETE for case {} — autonomous loop running ===", case_id[:8])
