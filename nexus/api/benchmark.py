"""
NEXUS -- Benchmark API (event-driven).

Endpoints for listing available benchmarks and injecting evidence.
The benchmark pipeline is now event-driven: inject evidence, publish
EVIDENCE_ADDED events to the EventBus, and let the reactive workers
handle analysis, hypotheses, contradictions, and suspect scoring
automatically.
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

from nexus.api.deps import get_database
from nexus.db.sqlite_db import Database

router = APIRouter(prefix="/api/benchmark", tags=["benchmark"])

# Global lock to serialise wave injection -- prevents VRAM saturation
_INJECT_LOCK = asyncio.Lock()

# Timeout for a single evidence injection (seconds).
# Covers GLiNER + summary + embed + RAPTOR. Generous but not infinite.
_EVIDENCE_TIMEOUT = 300  # 5 minutes per evidence item

# Max time to wait for workers to produce hypotheses + suspects (seconds)
_WORKER_POLL_TIMEOUT = 600  # 10 minutes
_WORKER_POLL_INTERVAL = 10  # check every 10s

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
    "kulik-osint": {"dir": "kulik", "name": "Kulik OSINT (2002)", "manifest": "manifest-osint.json"},
    "gsk": {"dir": "golden-state-killer", "name": "Golden State Killer (1974-86)"},
    "moreau": {"dir": "affaire-moreau", "name": "Affaire Moreau (fictif)"},
    "jubillar": {"dir": "jubillar", "name": "Affaire Delphine Jubillar (2020)"},
    "mccann": {"dir": "mccann", "name": "Affaire Maddie McCann (2007)"},
}


@router.get("/available")
async def list_available() -> list[dict[str, Any]]:
    """List available benchmark cases with their evidence counts."""
    result = []
    for key, info in KNOWN_BENCHMARKS.items():
        manifest_path = BENCHMARK_DIR / info["dir"] / info.get("manifest", "manifest.json")
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
    manifest_path = BENCHMARK_DIR / info["dir"] / info.get("manifest", "manifest.json")
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
        "investigation_manager": getattr(request.app.state, "investigation_manager", None),
    }
    background_tasks.add_task(_inject_wave, case_id, bench_key, wave, app_state)
    return {"status": "injecting", "wave": wave}


def _get_event_bus(app_state: dict) -> Any | None:
    """Try to get the EventBus from the investigation manager.

    Returns the bus if the manager has one (ReactiveInvestigationManager),
    or None if it's the legacy InvestigationManager or absent.
    """
    inv_manager = app_state.get("investigation_manager")
    if inv_manager is None:
        return None
    # ReactiveInvestigationManager exposes .event_bus
    bus = getattr(inv_manager, "event_bus", None)
    if bus is not None:
        return bus
    # Also check ._bus (internal attribute)
    return getattr(inv_manager, "_bus", None)


async def _publish_evidence_added(
    app_state: dict,
    case_id: str,
    evidence_id: str,
    title: str,
) -> bool:
    """Publish an EVIDENCE_ADDED event to the EventBus if available.

    Returns True if published, False if no bus available.
    """
    bus = _get_event_bus(app_state)
    if bus is None:
        return False

    try:
        from nexus.events.types import EventType, NexusEvent

        event = NexusEvent(
            event_type=EventType.EVIDENCE_ADDED,
            case_id=case_id,
            payload={
                "evidence_id": evidence_id,
                "title": title,
                "source": "benchmark",
            },
            source_worker="benchmark",
        )
        accepted = await bus.publish(event)
        if accepted:
            logger.debug(
                "Benchmark: published EVIDENCE_ADDED for {} ({})",
                evidence_id[:8], title[:40],
            )
        return accepted
    except Exception as exc:
        logger.warning(
            "Benchmark: failed to publish EVIDENCE_ADDED for {}: {}",
            evidence_id[:8], exc,
        )
        return False


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

    After each evidence is processed, publishes an EVIDENCE_ADDED event
    to the EventBus so reactive workers can pick it up automatically.

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
        manifest_path = BENCHMARK_DIR / info["dir"] / info.get("manifest", "manifest.json")
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
                    evidence_obj = await asyncio.wait_for(
                        processor.process_text_input(
                            case_id=case_id,
                            title=ev.get("title", file_path.stem),
                            text=text,
                            source=ev.get("source", "Benchmark"),
                            source_date=ev.get("source_date"),
                            reliability=ev.get("reliability", 50),
                        ),
                        timeout=_EVIDENCE_TIMEOUT,
                    )
                result["ok"] += 1
                logger.info("Benchmark injected [{}/{}]: {}", idx, len(wave_evidence), title)

                # Publish EVIDENCE_ADDED event for reactive workers
                await _publish_evidence_added(
                    app_state, case_id, evidence_obj.id, title,
                )

            except asyncio.TimeoutError:
                err = f"Timeout after {_EVIDENCE_TIMEOUT}s"
                logger.error("Benchmark inject TIMEOUT: {} -- {}", title, err)
                result["failed"].append({"title": title, "error": err})
                prog["errors"].append(f"wave {wave}: {title} -- {err}")

            except Exception as exc:
                err = f"{type(exc).__name__}: {exc}"
                logger.error("Benchmark inject failed: {} -- {}", title, err)
                result["failed"].append({"title": title, "error": err})
                prog["errors"].append(f"wave {wave}: {title} -- {err}")

            # Let the GPU breathe between evidence items
            await asyncio.sleep(2)

        logger.info(
            "Benchmark wave {} complete for case {}: {}/{} ok",
            wave, case_id[:8], result["ok"], len(wave_evidence),
        )

    return result


async def _run_full_benchmark(case_id: str, bench_key: str, app_state: dict | None = None) -> None:
    """Full event-driven benchmark pipeline: inject all waves then let workers handle the rest.

    Flow:
      1. Inject all waves sequentially (each publishes EVIDENCE_ADDED events)
      2. Start investigation (reactive workers take over)
      3. Poll for completion (hypotheses > 0 and suspects > 0, max 10 min)
      4. Report results

    The old sequential approach (manually calling AnalysisPipeline,
    HypothesisEngine, ContradictionDetector, SuspectScorer) is removed.
    The EventBus + reactive workers handle everything automatically.
    """
    prog = _progress(case_id)
    t0 = time.monotonic()

    try:
        from nexus.db.sqlite_db import get_db, Database

        app_state = app_state or {}

        info = KNOWN_BENCHMARKS[bench_key]
        manifest_path = BENCHMARK_DIR / info["dir"] / info.get("manifest", "manifest.json")
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        all_evidence = manifest.get("evidence", [])
        waves = sorted(set(e.get("wave", 1) for e in all_evidence))

        # ==============================================================
        # 0. Check for OSINT mode (briefing only, no evidence files)
        # ==============================================================
        briefing_text = manifest.get("briefing")
        monitoring_queries = manifest.get("monitoring_queries", [])

        if briefing_text and not all_evidence:
            # OSINT mode: inject briefing + create monitoring jobs + start
            prog["status"] = "osint_briefing"
            prog["total_evidence"] = 1
            prog["current_step"] = "Injecting briefing..."

            async with get_db() as conn:
                db = Database(conn)
                from nexus.core.evidence_processor import EvidenceProcessor
                from nexus.config import settings as _settings

                router = app_state.get("router")
                if router is None:
                    from nexus.llm.ollama_client import OllamaClient
                    from nexus.llm.router import LLMRouter
                    router = LLMRouter(OllamaClient())

                processor = EvidenceProcessor(
                    db=db, router=router,
                    upload_dir=_settings.upload_dir,
                    neo4j=app_state.get("neo4j"),
                    chroma=app_state.get("chroma"),
                    entity_extractor=app_state.get("entity_extractor"),
                )
                ev = await asyncio.wait_for(
                    processor.process_text_input(
                        case_id=case_id,
                        title="Briefing initial",
                        text=briefing_text,
                        source="Benchmark briefing",
                    ),
                    timeout=_EVIDENCE_TIMEOUT,
                )
                prog["current_evidence"] = 1
                await _publish_evidence_added(app_state, case_id, ev.id, "Briefing initial")

            # Create monitoring jobs
            prog["current_step"] = "Creating monitoring jobs..."
            async with get_db() as conn:
                db = Database(conn)
                # Use short interval for fast iteration — re-search every 2 minutes
                for q in monitoring_queries:
                    await db.create_monitoring_job(
                        case_id=case_id, job_type="searxng",
                        query=q, interval_hours=0,
                    )
                logger.info("Benchmark OSINT: created {} monitoring jobs", len(monitoring_queries))

            # Start investigation — workers + monitoring loop take over
            prog["current_step"] = "Starting autonomous investigation..."
            inv_manager = app_state.get("investigation_manager")
            if inv_manager:
                await inv_manager.start_investigation(case_id)

            # Poll for results (indefinite for OSINT mode — monitoring loop keeps searching)
            prog["status"] = "osint_running"
            _OSINT_POLL_MAX = 10800  # 10800 * 10s = 30 hours
            for i in range(_OSINT_POLL_MAX):
                await asyncio.sleep(10)
                async with get_db() as conn:
                    db = Database(conn)
                    ev_count = len(await db.list_evidence_by_case(case_id))
                    ent_count = len(await db.list_entities_by_case(case_id))
                    hyp_list = await db.list_hypotheses_by_case(case_id)

                elapsed = time.monotonic() - t0
                prog["current_step"] = f"OSINT running... {ev_count} ev, {ent_count} ent, {len(hyp_list)} hyp ({elapsed:.0f}s)"
                prog["stats"] = {"evidence": ev_count, "entities": ent_count, "hypotheses": len(hyp_list)}

                # Never stop the OSINT investigation — let reactive loops compound.
                # The monitoring loop + workers keep running as long as the server is up.
                # The poll just reports progress, it doesn't control the investigation.

            # Mark progress but investigation continues in background
            prog["status"] = "running"
            prog["finished_at"] = None
            logger.info(
                "Benchmark OSINT poll ended for case {} after {:.0f}s — investigation continues",
                case_id[:8], time.monotonic() - t0,
            )
            return

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
            "Benchmark full: all waves done for case {} -- {}/{} ok, {} failed",
            case_id[:8], total_ok, len(all_evidence), total_failed,
        )

        # ==============================================================
        # 2. Start investigation (workers take over)
        # ==============================================================
        prog["current_step"] = "Starting investigation..."
        inv_manager = app_state.get("investigation_manager")
        if inv_manager is not None:
            try:
                started = await inv_manager.start_investigation(case_id)
                if started:
                    logger.info("Benchmark full: investigation STARTED for case {}", case_id[:8])
                else:
                    logger.info("Benchmark full: investigation already running for case {}", case_id[:8])
            except Exception as exc:
                err = f"start_investigation FAILED: {type(exc).__name__}: {exc}"
                logger.error("Benchmark {}", err)
                prog["errors"].append(err)
        else:
            logger.warning("Benchmark full: no investigation_manager -- workers won't run")

        # ==============================================================
        # 3. Wait for workers to produce hypotheses + suspects
        # ==============================================================
        prog["status"] = "analyzing"
        prog["current_step"] = "Waiting for analysis..."

        max_polls = _WORKER_POLL_TIMEOUT // _WORKER_POLL_INTERVAL
        analysis_complete = False

        for i in range(max_polls):
            await asyncio.sleep(_WORKER_POLL_INTERVAL)

            try:
                async with get_db() as conn:
                    db = Database(conn)
                    evidence_list = await db.list_evidence_by_case(case_id)
                    entities_list = await db.list_entities_by_case(case_id)
                    hypotheses_list = await db.list_hypotheses_by_case(case_id)
                    suspects_list = await db.list_suspects_by_case(case_id)

                    ev_count = len(evidence_list)
                    ent_count = len(entities_list)
                    hyp_count = len(hypotheses_list)
                    sus_count = len(suspects_list)

                    prog["current_step"] = (
                        f"Analysis in progress... "
                        f"({ev_count} ev, {ent_count} ent, {hyp_count} hyp, {sus_count} sus)"
                    )
                    prog["stats"] = {
                        "evidence": ev_count,
                        "entities": ent_count,
                        "hypotheses": hyp_count,
                        "suspects": sus_count,
                    }

                    if hyp_count > 0 and sus_count > 0:
                        logger.info(
                            "Benchmark full: analysis complete for case {} "
                            "({} hyp, {} sus) after {}s",
                            case_id[:8], hyp_count, sus_count,
                            (i + 1) * _WORKER_POLL_INTERVAL,
                        )
                        analysis_complete = True
                        break

            except Exception as exc:
                logger.warning("Benchmark poll error: {}", exc)

        if not analysis_complete:
            logger.warning(
                "Benchmark full: analysis timed out after {}s for case {} "
                "(workers may still be running)",
                _WORKER_POLL_TIMEOUT, case_id[:8],
            )
            prog["errors"].append(
                f"Analysis timed out after {_WORKER_POLL_TIMEOUT}s "
                f"(workers may still be running in background)"
            )

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
            "({} ok, {} failed, {} errors) ===",
            case_id[:8], elapsed, total_ok, total_failed, len(prog["errors"]),
        )

    except Exception as exc:
        # ABSOLUTE LAST RESORT -- catches anything that slipped through:
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
