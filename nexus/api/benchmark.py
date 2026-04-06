"""
NEXUS -- Benchmark API.

Endpoints for listing available benchmarks and injecting evidence.
"""

from __future__ import annotations

import asyncio
import json
import time
import traceback
from pathlib import Path
from typing import Any

from fastapi import APIRouter, BackgroundTasks, Depends, Request
from loguru import logger

from nexus.api.deps import get_database, get_evidence_processor
from nexus.db.sqlite_db import Database

router = APIRouter(prefix="/api/benchmark", tags=["benchmark"])

# Global lock to serialise wave injection — prevents VRAM saturation
_INJECT_LOCK = asyncio.Lock()

# Timeout for a single evidence injection (seconds).
# Covers GLiNER + summary + embed + RAPTOR. Generous but not infinite.
_EVIDENCE_TIMEOUT = 300  # 5 minutes per evidence item

# Timeout for post-injection pipeline steps (analysis, hypotheses, etc.)
_PIPELINE_STEP_TIMEOUT = 600  # 10 minutes per step

# In-memory progress tracker so the frontend can poll status.
# Keyed by case_id. Values are dicts with wave/step/status info.
_BENCHMARK_PROGRESS: dict[str, dict[str, Any]] = {}


def _progress(case_id: str) -> dict[str, Any]:
    """Get or create the progress dict for a benchmark run."""
    if case_id not in _BENCHMARK_PROGRESS:
        _BENCHMARK_PROGRESS[case_id] = {
            "status": "starting",
            "current_wave": 0,
            "total_waves": 0,
            "current_evidence": 0,
            "total_evidence": 0,
            "current_step": "",
            "errors": [],
            "started_at": time.time(),
            "finished_at": None,
        }
    return _BENCHMARK_PROGRESS[case_id]

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


@router.get("/progress/{case_id}")
async def get_benchmark_progress(case_id: str) -> dict[str, Any]:
    """Poll progress of a running benchmark pipeline."""
    if case_id in _BENCHMARK_PROGRESS:
        return _BENCHMARK_PROGRESS[case_id]
    return {"status": "unknown", "message": "No benchmark running for this case"}


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


async def _inject_wave(
    case_id: str,
    bench_key: str,
    wave: int,
    app_state: dict | None = None,
) -> dict[str, Any]:
    """Background task: inject all evidence for a wave.

    Serialised via ``_INJECT_LOCK`` so that only one wave runs at a
    time, preventing VRAM saturation from parallel LLM calls.
    Reuses shared singletons passed from the endpoint via app_state.

    Returns a dict with ``ok`` count and ``failed`` list for reporting.
    """
    result = {"ok": 0, "failed": [], "wave": wave}

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

        # Update progress tracker
        prog = _progress(case_id)
        prog["current_wave"] = wave

        for idx, ev in enumerate(wave_evidence, 1):
            title = ev.get("title", "?")[:60]
            file_path = BENCHMARK_DIR / info["dir"] / ev.get("file", "")

            prog["current_evidence"] = prog.get("_wave_base", 0) + idx
            prog["current_step"] = f"wave {wave}: injecting {title}"

            if not file_path.exists():
                err = f"File missing: {file_path}"
                logger.warning("Benchmark {}", err)
                result["failed"].append({"title": title, "error": err})
                prog["errors"].append(f"wave {wave}: {err}")
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
                    # Wrap with timeout so a hung LLM call doesn't block forever
                    await asyncio.wait_for(
                        processor.process_text_input(
                            case_id=case_id,
                            title=ev.get("title", file_path.stem),
                            text=text,
                            source=ev.get("source", "Benchmark"),
                        ),
                        timeout=_EVIDENCE_TIMEOUT,
                    )
                result["ok"] += 1
                logger.info("Benchmark injected [{}/{}]: {}", idx, len(wave_evidence), title)

            except asyncio.TimeoutError:
                err = f"Timeout after {_EVIDENCE_TIMEOUT}s"
                logger.error("Benchmark inject TIMEOUT: {} — {}", title, err)
                result["failed"].append({"title": title, "error": err})
                prog["errors"].append(f"wave {wave}: {title} — {err}")

            except Exception as exc:
                err = f"{type(exc).__name__}: {exc}"
                logger.error("Benchmark inject failed: {} — {}", title, err)
                result["failed"].append({"title": title, "error": err})
                prog["errors"].append(f"wave {wave}: {title} — {err}")

            # Let the GPU breathe between evidence items
            await asyncio.sleep(2)

        logger.info(
            "Benchmark wave {} complete for case {}: {}/{} ok",
            wave, case_id[:8], result["ok"], len(wave_evidence),
        )

    return result


async def _run_full_benchmark(case_id: str, bench_key: str, app_state: dict | None = None) -> None:
    """Full benchmark pipeline: inject all waves -> analyze -> hypotheses -> suspects.

    Reuses shared singletons passed from the endpoint via app_state
    to respect VRAM serialization and avoid redundant connections.

    This function is the TOP-LEVEL entry point for the background task.
    It wraps everything in try/except so that no exception can escape
    silently (FastAPI BackgroundTasks swallows exceptions to stderr,
    bypassing loguru).
    """
    prog = _progress(case_id)
    t0 = time.monotonic()

    try:
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
        all_evidence = manifest.get("evidence", [])
        waves = sorted(set(e.get("wave", 1) for e in all_evidence))

        prog["status"] = "injecting"
        prog["total_waves"] = len(waves)
        prog["total_evidence"] = len(all_evidence)

        # ==============================================================
        # 1. Inject all waves sequentially
        # ==============================================================
        total_ok = 0
        total_failed = 0
        evidence_seen = 0

        for wave in waves:
            wave_count = sum(1 for e in all_evidence if e.get("wave", 1) == wave)
            prog["_wave_base"] = evidence_seen  # for per-evidence counting

            logger.info(
                "Benchmark full: injecting wave {}/{} ({} evidence) for case {}",
                wave, len(waves), wave_count, case_id[:8],
            )

            try:
                wave_result = await _inject_wave(case_id, bench_key, wave, app_state)
                total_ok += wave_result["ok"]
                total_failed += len(wave_result["failed"])
            except Exception as exc:
                # Wave-level failure (lock issue, import error, etc.)
                err = f"Wave {wave} CRASHED: {type(exc).__name__}: {exc}"
                logger.error("Benchmark {}", err)
                prog["errors"].append(err)
                total_failed += wave_count

            evidence_seen += wave_count
            await asyncio.sleep(3)

        logger.info(
            "Benchmark full: all waves done for case {} — {}/{} ok, {} failed",
            case_id[:8], total_ok, len(all_evidence), total_failed,
        )

        # ==============================================================
        # 2. Post-injection pipeline steps (each with timeout + catch)
        # ==============================================================

        async def _run_step(step_name: str, coro):
            """Run a pipeline step with timeout and error isolation."""
            prog["current_step"] = step_name
            prog["status"] = step_name
            logger.info("Benchmark full: {} ...", step_name)
            try:
                await asyncio.wait_for(coro, timeout=_PIPELINE_STEP_TIMEOUT)
                logger.info("Benchmark full: {} OK", step_name)
            except asyncio.TimeoutError:
                err = f"{step_name} TIMEOUT after {_PIPELINE_STEP_TIMEOUT}s"
                logger.error("Benchmark {}", err)
                prog["errors"].append(err)
            except Exception as exc:
                err = f"{step_name} FAILED: {type(exc).__name__}: {exc}"
                logger.error("Benchmark {}", err)
                prog["errors"].append(err)

        # 2a. Analyze
        async def _do_analysis():
            async with get_db() as conn:
                db = Database(conn)
                from nexus.core.analysis_pipeline import AnalysisPipeline
                pipeline = AnalysisPipeline(db=db, router=router, chroma=chroma, neo4j=neo4j)
                await pipeline.run_full_analysis(case_id)

        await _run_step("analysis", _do_analysis())
        await asyncio.sleep(2)

        # 2b. Hypotheses
        async def _do_hypotheses():
            async with get_db() as conn:
                db = Database(conn)
                from nexus.core.hypothesis_engine import HypothesisEngine
                engine = HypothesisEngine(db=db, router=router, chroma=chroma, neo4j=neo4j)
                await engine.generate_hypotheses(case_id)

        await _run_step("hypothesis_generation", _do_hypotheses())
        await asyncio.sleep(2)

        # 2c. Contradictions
        async def _do_contradictions():
            async with get_db() as conn:
                db = Database(conn)
                from nexus.core.contradiction_detector import ContradictionDetector
                detector = ContradictionDetector(db=db, router=router)
                await detector.detect_contradictions(case_id)

        await _run_step("contradiction_detection", _do_contradictions())
        await asyncio.sleep(2)

        # 2d. Suspect scoring
        async def _do_suspects():
            async with get_db() as conn:
                db = Database(conn)
                from nexus.core.suspect_scorer import SuspectScorer
                scorer = SuspectScorer(db=db, router=router, neo4j=neo4j)
                await scorer.score_all_suspects(case_id, trigger="benchmark")

        await _run_step("suspect_scoring", _do_suspects())
        await asyncio.sleep(2)

        # 2e. Neo4j sync
        if neo4j is not None:
            async def _do_neo4j_sync():
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
                    for ent in entities:
                        mentions = await db.list_mentions_by_entity(ent["id"])
                        for m in mentions:
                            await neo4j.link_evidence_to_entity(m["evidence_id"], ent["id"])

            await _run_step("neo4j_sync", _do_neo4j_sync())

        # 2f. Start autonomous investigation loop
        try:
            prog["current_step"] = "autonomous_loop_start"
            inv_manager = app_state.get("investigation_manager")
            if inv_manager is not None:
                started = await inv_manager.start_investigation(case_id)
                if started:
                    logger.info("Benchmark full: autonomous investigation STARTED via shared manager")
                else:
                    logger.info("Benchmark full: investigation already running for case {}", case_id[:8])
            else:
                from nexus.core.investigation_manager import InvestigationManager
                manager = InvestigationManager(
                    router=router, chroma=chroma, neo4j=neo4j,
                    entity_extractor=entity_extractor,
                )
                await manager.start_investigation(case_id)
                logger.info("Benchmark full: autonomous investigation STARTED (standalone manager)")
        except Exception as exc:
            err = f"autonomous_loop_start FAILED: {type(exc).__name__}: {exc}"
            logger.error("Benchmark {}", err)
            prog["errors"].append(err)

        # ==============================================================
        # Done
        # ==============================================================
        elapsed = time.monotonic() - t0
        prog["status"] = "complete"
        prog["finished_at"] = time.time()
        prog["elapsed_seconds"] = round(elapsed, 1)
        prog["injection_summary"] = {"ok": total_ok, "failed": total_failed}

        logger.info(
            "=== BENCHMARK PIPELINE COMPLETE for case {} in {:.0f}s "
            "({} ok, {} failed, {} errors) — autonomous loop running ===",
            case_id[:8], elapsed, total_ok, total_failed, len(prog["errors"]),
        )

    except Exception as exc:
        # ABSOLUTE LAST RESORT — this catches anything that slipped through:
        # import errors, manifest parse failures, unexpected exceptions.
        elapsed = time.monotonic() - t0
        tb = traceback.format_exc()
        logger.error(
            "BENCHMARK PIPELINE CRASHED for case {} after {:.0f}s: {}\n{}",
            case_id[:8], elapsed, exc, tb,
        )
        prog["status"] = "crashed"
        prog["finished_at"] = time.time()
        prog["elapsed_seconds"] = round(elapsed, 1)
        prog["errors"].append(f"PIPELINE CRASH: {type(exc).__name__}: {exc}")
