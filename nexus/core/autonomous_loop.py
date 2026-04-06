"""
NEXUS -- Autonomous Investigation Loop.

The brain of NEXUS. For each active case, this loop runs continuously:

1. OBSERVE  -- Check for new monitoring results, new evidence
2. ORIENT   -- Ingest new data, OSINT recon, geocode, image analysis, visual embeddings
3. DECIDE   -- Re-evaluate hypotheses, detect contradictions, forensic analysis, timeline
4. ACT      -- Generate new search queries, OSINT enrichment, domain recon
5. QUESTION -- Challenge top hypothesis, periodic reports, automated backups

This implements the OODA loop (Observe-Orient-Decide-Act) adapted for
criminal investigation, with an added self-questioning step.

ALL 21 modules are connected:
  - EvidenceProcessor      (ORIENT: auto-ingest monitoring results)
  - AnalysisPipeline       (DECIDE: incremental analysis)
  - HypothesisEngine       (DECIDE: generate/evaluate hypotheses)
  - ContradictionDetector  (DECIDE: find inconsistencies)
  - AlertManager           (DECIDE: create alerts)
  - HoleheRecon            (ORIENT: email existence check)
  - SocialRecon            (ORIENT: username lookup across platforms)
  - DomainRecon            (ACT: WHOIS/DNS on email domains)
  - GeoMapper              (ORIENT: geocode location entities)
  - ImageAnalyzer          (ORIENT: VLM analysis of image evidence)
  - ImageSearchEngine      (ORIENT: index images in DINOv2/CLIP)
  - VisualEmbedder         (ORIENT: generate visual embeddings)
  - BloodPatternAnalyzer   (DECIDE: classify blood patterns)
  - TraceAnalyzer          (DECIDE: analyze physical traces)
  - AcousticAnalyzer       (DECIDE: forensic audio analysis)
  - TimelineBuilder        (DECIDE: rebuild chronological timeline)
  - ReportGenerator        (QUESTION: periodic investigation reports)
  - BackupManager          (QUESTION: automated database backups)

The loop runs as a background task in the FastAPI lifespan.
"""

from __future__ import annotations

import asyncio
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from loguru import logger

from nexus.config import settings
from nexus.core.audit import AuditService
from nexus.db.sqlite_db import Database, get_db
from nexus.llm.prompts import ADAPTIVE_QUERY_PROMPT, SELF_QUESTIONING_PROMPT
from nexus.llm.router import LLMRouter, TaskType
from nexus.monitoring.alert_manager import AlertManager


class AutonomousInvestigator:
    """Autonomous investigation daemon for a single case.

    Runs continuously, connecting all NEXUS engines together:
    - MonitoringScheduler finds new information
    - EvidenceProcessor ingests it
    - AnalysisPipeline analyzes it
    - HypothesisEngine re-evaluates theories
    - ContradictionDetector finds inconsistencies
    - Recon modules (holehe, social, domain) enrich entities
    - GeoMapper geocodes location entities
    - ImageAnalyzer + ImageSearchEngine process image evidence
    - BloodPatternAnalyzer + TraceAnalyzer do forensic analysis
    - TimelineBuilder reconstructs chronology
    - ReportGenerator produces periodic reports
    - BackupManager secures the database
    - Self-questioning generates new search directions

    IMPORTANT: This class opens its own DB connections per-operation
    via ``get_db()`` because it runs as a long-lived background task,
    outside any request scope.
    """

    def __init__(
        self,
        case_id: str,
        router: LLMRouter,
        chroma: Any,
        neo4j: Any,
        entity_extractor: Any = None,
    ) -> None:
        self._case_id = case_id
        self._router = router
        self._chroma = chroma
        self._neo4j = neo4j
        self._entity_extractor = entity_extractor
        self._running = False
        self._cycle_count = 0
        self._last_action: str | None = None
        self._last_cycle_at: str | None = None
        self._started_at: str | None = None
        # Per-tool status tracking for the UI
        self._tool_status: dict[str, dict[str, Any]] = {}

    # ------------------------------------------------------------------
    # Public interface
    # ------------------------------------------------------------------

    @property
    def case_id(self) -> str:
        return self._case_id

    @property
    def cycle_count(self) -> int:
        return self._cycle_count

    @property
    def is_running(self) -> bool:
        return self._running

    @property
    def last_action(self) -> str | None:
        return self._last_action

    @property
    def last_cycle_at(self) -> str | None:
        return self._last_cycle_at

    @property
    def started_at(self) -> str | None:
        return self._started_at

    def _track_tool(self, name: str, status: str, detail: str = "", file: str = "") -> None:
        """Update per-tool status for the investigation UI."""
        self._tool_status[name] = {
            "status": status,  # idle, running, done, error
            "detail": detail,
            "file": file,
            "updated_at": datetime.now(timezone.utc).isoformat(),
            "cycle": self._cycle_count,
        }

    def get_status(self) -> dict[str, Any]:
        """Return a status dict for the API."""
        return {
            "case_id": self._case_id,
            "running": self._running,
            "cycle_count": self._cycle_count,
            "last_action": self._last_action,
            "last_cycle_at": self._last_cycle_at,
            "started_at": self._started_at,
            "tools": self._tool_status,
        }

    async def _audit_log(self, actor, action, summary, **kwargs):
        """Fire-and-forget audit log via a fresh DB connection."""
        try:
            async with get_db() as conn:
                audit = AuditService(Database(conn))
                await audit.log(
                    case_id=self._case_id,
                    actor=actor,
                    action=action,
                    summary=summary,
                    **kwargs,
                )
        except Exception as exc:
            logger.warning("Audit log failed (non-blocking): {}", exc)

    async def run(self) -> None:
        """Main investigation loop -- runs until stopped."""
        self._running = True
        self._started_at = datetime.now(timezone.utc).isoformat()
        logger.info("Autonomous investigator STARTED for case {}", self._case_id)

        await self._audit_log(
            "autonomous_loop", "investigation_started",
            "Boucle autonome demarree",
        )

        while self._running:
            try:
                self._cycle_count += 1
                self._last_cycle_at = datetime.now(timezone.utc).isoformat()
                logger.info(
                    "=== Case {} -- OODA Cycle {} ===",
                    self._case_id,
                    self._cycle_count,
                )

                # Open ONE DB connection for the entire cycle
                async with get_db() as conn:
                    db = Database(conn)

                    # PHASE 1: OBSERVE -- What's new?
                    self._last_action = "OBSERVE"
                    self._track_tool("monitoring", "running", "Recherche nouveaux resultats", "monitoring/scheduler.py")
                    new_results = await self._observe(db)
                    self._track_tool("monitoring", "done", f"{len(new_results)} resultats pertinents")

                    # PHASE 2: ORIENT -- Ingest, recon, geocode, image analysis
                    self._last_action = "ORIENT"
                    new_evidence_ids = await self._orient(db, new_results)

                    # PHASE 3: DECIDE -- Analyze, hypotheses, contradictions, forensics, timeline
                    self._last_action = "DECIDE"
                    decisions = await self._decide(db, new_evidence_ids)

                    # PHASE 4: ACT -- Search queries, OSINT enrichment, domain recon
                    self._last_action = "ACT"
                    await self._act(db, decisions)

                    # PHASE 5: QUESTION -- Self-questioning, reports, backups
                    self._last_action = "QUESTION"
                    await self._question(db)

                self._last_action = "SLEEPING"
                logger.info(
                    "=== Case {} -- Cycle {} complete, sleeping {}min ===",
                    self._case_id,
                    self._cycle_count,
                    settings.investigation_cycle_minutes,
                )

                # Sleep between cycles
                await asyncio.sleep(settings.investigation_cycle_minutes * 60)

            except asyncio.CancelledError:
                logger.info(
                    "Autonomous investigator CANCELLED for case {}",
                    self._case_id,
                )
                break
            except Exception as e:
                logger.error(
                    "Investigation cycle error for case {}: {}",
                    self._case_id,
                    e,
                )
                self._last_action = f"ERROR: {e}"
                # Wait 5 minutes before retrying on error
                await asyncio.sleep(300)

        self._running = False
        await self._audit_log(
            "autonomous_loop", "investigation_stopped",
            f"Boucle autonome arretee apres {self._cycle_count} cycles",
        )
        logger.info(
            "Autonomous investigator STOPPED for case {} after {} cycles",
            self._case_id,
            self._cycle_count,
        )

    async def stop(self) -> None:
        """Signal the loop to stop after current cycle."""
        self._running = False

    # ================================================================
    # PHASE 1: OBSERVE -- Check for new monitoring results
    # ================================================================

    async def _observe(self, db: Database) -> list[dict[str, Any]]:
        """Check for unreviewed monitoring results with high relevance."""
        results = await db.list_results_by_case(self._case_id)

        new_results = [
            r
            for r in results
            if not r.get("reviewed") and not r.get("is_duplicate")
        ]

        # Filter: only auto-ingest if relevance >= threshold
        threshold = settings.auto_ingest_relevance_threshold
        high_relevance = [
            r
            for r in new_results
            if (r.get("relevance_score") or 0) >= threshold
        ]

        if high_relevance:
            logger.info(
                "OBSERVE: {} new relevant results (of {} unreviewed) for case {}",
                len(high_relevance),
                len(new_results),
                self._case_id,
            )
            # Audit: log each observed result
            for r in high_relevance:
                await self._audit_log(
                    "autonomous_loop", "monitoring_result",
                    f"Resultat observe: {(r.get('title') or 'N/A')[:100]} "
                    f"(pertinence: {r.get('relevance_score', 0):.0f}%)",
                    target_type="monitoring_result",
                    target_id=r.get("id"),
                    details={"title": r.get("title"), "relevance": r.get("relevance_score")},
                    cycle_number=self._cycle_count,
                )
        else:
            logger.debug(
                "OBSERVE: No new relevant results for case {} ({} unreviewed below threshold)",
                self._case_id,
                len(new_results),
            )

        return high_relevance

    # ================================================================
    # PHASE 2: ORIENT -- Ingest new data, recon, geocode, images
    # ================================================================

    async def _orient(self, db: Database, new_results: list[dict[str, Any]]) -> list[str]:
        """Auto-ingest monitoring results, then enrich with recon/geo/images."""
        new_evidence_ids: list[str] = []

        # --- 2a. Auto-ingest monitoring results as evidence ---
        self._track_tool("evidence_processor", "running", f"Ingestion de {len(new_results)} resultats", "core/evidence_processor.py")
        new_evidence_ids = await self._orient_ingest(db, new_results)
        self._track_tool("evidence_processor", "done", f"{len(new_evidence_ids)} preuves ingerees")

        # --- 2b. Periodic Neo4j re-sync (catches failed syncs) ---
        if self._neo4j and self._cycle_count % 3 == 0:
            self._track_tool("neo4j_sync", "running", "Re-sync graphe", "db/neo4j_db.py")
            await self._orient_neo4j_resync(db)
            self._track_tool("neo4j_sync", "done", "Graphe synchronise")

        # --- 2c. OSINT recon on new entities (email/username) ---
        if settings.auto_osint_recon:
            self._track_tool("osint_recon", "running", "Holehe + social scan", "recon/")
            await self._orient_osint_recon(db)
            self._track_tool("osint_recon", "done", "Scan OSINT termine")

        # --- 2d. Geocode location entities ---
        if settings.auto_geocode:
            self._track_tool("geo_mapper", "running", "Geocodage lieux", "core/geo_mapper.py")
            await self._orient_geocode(db)
            self._track_tool("geo_mapper", "done", "Lieux geocodes")

        # --- 2e. Analyse image evidence (VLM) ---
        if settings.auto_image_analysis:
            self._track_tool("image_analyzer", "running", "Analyse VLM images", "core/image_analyzer.py")
            await self._orient_image_analysis(db)
            self._track_tool("image_analyzer", "done", "Images analysees")

        # --- 2f. Index images in DINOv2/CLIP ---
        if settings.auto_visual_embeddings:
            self._track_tool("visual_embedder", "running", "Indexation DINOv2/CLIP", "vision/embeddings.py")
            await self._orient_visual_embeddings(db)
            self._track_tool("visual_embedder", "done", "Embeddings visuels indexes")

        return new_evidence_ids

    async def _orient_ingest(self, db: Database, new_results: list[dict[str, Any]]) -> list[str]:
        """Auto-ingest promising monitoring results as evidence."""
        new_evidence_ids: list[str] = []
        max_ingest = settings.max_auto_ingest_per_cycle

        # Create processor once for all results (reuses GLiNER singleton)
        from nexus.core.evidence_processor import EvidenceProcessor

        processor = EvidenceProcessor(
            db=db,
            router=self._router,
            upload_dir=settings.upload_dir,
            neo4j=self._neo4j,
            chroma=self._chroma,
            entity_extractor=self._entity_extractor,
        )

        for result in new_results[:max_ingest]:
            try:

                # Build text from monitoring result
                text = (
                    f"Source: {result.get('url', 'unknown')}\n"
                    f"Titre: {result.get('title', '')}\n"
                    f"Contenu: {result.get('snippet', '')}\n"
                    f"Moteur: {result.get('source_engine', '')}\n"
                    f"Date: {result.get('found_at', '')}"
                )

                title_raw = result.get("title", "Resultat monitoring")
                title = f"[AUTO-MONITORING] {title_raw[:100]}"

                evidence = await processor.process_text_input(
                    case_id=self._case_id,
                    title=title,
                    text=text,
                    source=(
                        f"Monitoring automatique -- "
                        f"{result.get('source_engine', 'SearXNG')}"
                    ),
                )

                new_evidence_ids.append(evidence.id)

                # Mark monitoring result as reviewed
                await db.update_monitoring_result(
                    result["id"], reviewed=True
                )

                # Audit: log auto-ingestion
                audit = AuditService(db)
                await audit.log_auto_ingest(
                    case_id=self._case_id,
                    result_id=result["id"],
                    evidence_id=evidence.id,
                    title=title,
                    cycle=self._cycle_count,
                )

                logger.info(
                    "ORIENT: Auto-ingested monitoring result {} -> evidence {}",
                    result["id"][:8],
                    evidence.id[:8],
                )

            except Exception as e:
                logger.error(
                    "ORIENT: Failed to ingest result {}: {}",
                    result.get("id", "?")[:8],
                    e,
                )

        return new_evidence_ids

    async def _orient_neo4j_resync(self, db: Database) -> None:
        """Re-sync evidence and entities to Neo4j for any items not yet synced.

        Catches evidence that was ingested but whose Neo4j sync failed.
        Runs every 3 cycles to avoid excessive load. Uses MERGE (idempotent).
        """
        try:
            evidence_list = await db.list_evidence_by_case(self._case_id)
            entities = await db.list_entities_by_case(self._case_id)
            processed = [ev for ev in evidence_list if ev.get("status") == "processed"]

            ev_ok, ev_fail = 0, 0
            ent_ok, ent_fail = 0, 0
            link_ok, link_fail = 0, 0

            # Sync evidence nodes
            for ev in processed:
                try:
                    await self._neo4j.sync_evidence(
                        evidence_id=ev["id"],
                        case_id=self._case_id,
                        title=ev["title"],
                        evidence_type=ev["evidence_type"],
                        reliability=ev.get("reliability", 50),
                    )
                    ev_ok += 1
                except Exception:
                    ev_fail += 1

            # Sync entity nodes
            for ent in entities:
                try:
                    await self._neo4j.sync_entity(ent, self._case_id)
                    ent_ok += 1
                except Exception:
                    ent_fail += 1

            # Link evidence to entities via mentions (only processed evidence)
            for ev in processed:
                mentions = await db.list_mentions_by_evidence(ev["id"])
                for m in mentions:
                    try:
                        await self._neo4j.link_evidence_to_entity(
                            ev["id"], m["entity_id"]
                        )
                        link_ok += 1
                    except Exception:
                        link_fail += 1

            logger.info(
                "ORIENT: Neo4j re-sync for case {} — "
                "evidence: {}/{}, entities: {}/{}, links: {}/{}",
                self._case_id,
                ev_ok, ev_ok + ev_fail,
                ent_ok, ent_ok + ent_fail,
                link_ok, link_ok + link_fail,
            )
            if ev_fail or ent_fail or link_fail:
                logger.warning(
                    "ORIENT: Neo4j re-sync had failures — "
                    "ev_fail={}, ent_fail={}, link_fail={}",
                    ev_fail, ent_fail, link_fail,
                )
            await self._audit_log(
                "autonomous_loop", "neo4j_resync",
                f"Re-sync Neo4j: {ev_ok} preuves, {ent_ok} entites, {link_ok} liens"
                f" (echecs: {ev_fail + ent_fail + link_fail})",
                details={
                    "evidence": {"ok": ev_ok, "fail": ev_fail},
                    "entities": {"ok": ent_ok, "fail": ent_fail},
                    "links": {"ok": link_ok, "fail": link_fail},
                },
                cycle_number=self._cycle_count,
            )

        except Exception as e:
            logger.warning("ORIENT: Neo4j re-sync failed: {}", e)

    async def _orient_osint_recon(self, db: Database) -> None:
        """OSINT recon: for each new email/account entity, run holehe + social_recon."""
        try:
            entities = await db.list_entities_by_case(self._case_id)

            for ent in entities:
                etype = ent.get("entity_type", "")
                name = ent.get("name", "")
                metadata = (
                    json.loads(ent.get("metadata") or "{}")
                    if isinstance(ent.get("metadata"), str)
                    else (ent.get("metadata") or {})
                )

                # Skip if already scanned
                if metadata.get("recon_done"):
                    continue

                # --- Holehe: check email existence on 120+ services ---
                if etype == "email" and "@" in name:
                    try:
                        from nexus.recon.holehe_recon import HoleheRecon

                        recon = HoleheRecon()
                        results = await recon.check_email(name)
                        if results:
                            metadata["holehe_results"] = results
                            metadata["holehe_count"] = len(results)
                            await self._audit_log(
                                "autonomous_loop", "osint_holehe",
                                f"Holehe email {name}: {len(results)} comptes trouves",
                                target_type="entity",
                                target_id=ent["id"],
                                details={"email": name, "results": results},
                                cycle_number=self._cycle_count,
                            )
                        # Rate-limit between OSINT calls
                        await asyncio.sleep(settings.auto_recon_rate_limit)
                    except Exception as e:
                        logger.warning("ORIENT: Holehe failed for {}: {}", name, e)

                # --- Social recon: check username on major platforms ---
                if etype in ("email", "account", "person"):
                    try:
                        from nexus.recon.social_recon import SocialRecon

                        social = SocialRecon()
                        # For email, extract username part
                        if etype == "email" and "@" in name:
                            profiles = await social.search_email_username(name)
                        else:
                            # For account/person, search the name directly
                            search_name = name.replace(" ", "")
                            if len(search_name) >= 3:
                                profiles = await social.search_username(search_name)
                            else:
                                profiles = []

                        found = [p for p in profiles if p.get("exists")]
                        if found:
                            metadata["social_profiles"] = found
                            metadata["social_count"] = len(found)
                            await self._audit_log(
                                "autonomous_loop", "osint_social",
                                f"Social recon {name}: {len(found)} profils trouves",
                                target_type="entity",
                                target_id=ent["id"],
                                details={"name": name, "profiles": found},
                                cycle_number=self._cycle_count,
                            )
                        # Rate-limit
                        await asyncio.sleep(settings.auto_recon_rate_limit)
                    except Exception as e:
                        logger.warning("ORIENT: Social recon failed for {}: {}", name, e)

                # Mark entity as scanned (even if no results) to avoid re-scanning
                metadata["recon_done"] = True
                await db.update_entity(ent["id"], metadata=json.dumps(metadata))

        except Exception as e:
            logger.warning("ORIENT: OSINT recon phase failed: {}", e)

    async def _orient_geocode(self, db: Database) -> None:
        """Geocode all location entities that haven't been geocoded yet."""
        try:
            from nexus.core.geo_mapper import GeoMapper

            geo = GeoMapper(db)
            results = await geo.geocode_entities(self._case_id)

            newly_geocoded = [r for r in results if r.get("status") == "geocoded"]
            if newly_geocoded:
                logger.info(
                    "ORIENT: Geocoded {} locations for case {}",
                    len(newly_geocoded),
                    self._case_id,
                )
                for r in newly_geocoded:
                    # Enrich entity description with GPS coordinates so retriever sees them
                    entity_id = r.get("entity_id")
                    if entity_id:
                        try:
                            ent = await db.get_entity(entity_id)
                            if ent:
                                geo_info = f" [GPS: {r['lat']:.4f},{r['lon']:.4f}]"
                                desc = (ent.get("description") or "") + geo_info
                                await db.update_entity(entity_id, description=desc)
                        except Exception as ge:
                            logger.debug("ORIENT: Could not enrich entity {}: {}", entity_id[:8], ge)
                    await self._audit_log(
                        "autonomous_loop", "geocode",
                        f"Lieu geocode: {r['name']} -> {r['lat']},{r['lon']}",
                        target_type="entity",
                        target_id=r.get("entity_id"),
                        details={"lat": r["lat"], "lon": r["lon"], "name": r["name"]},
                        cycle_number=self._cycle_count,
                    )
        except Exception as e:
            logger.warning("ORIENT: Geocoding phase failed: {}", e)

    async def _orient_image_analysis(self, db: Database) -> None:
        """Analyse image evidence that hasn't been processed yet via VLM."""
        try:
            all_evidence = await db.list_evidence_by_case(self._case_id)

            for ev in all_evidence:
                if ev.get("evidence_type") == "image" and not ev.get("summary"):
                    file_path = ev.get("file_path", "")
                    if not file_path or not Path(file_path).exists():
                        continue

                    try:
                        from nexus.core.image_analyzer import ImageAnalyzer

                        analyzer = ImageAnalyzer(
                            router=self._router,
                            db=db,
                            chroma=self._chroma,
                        )
                        await analyzer.process_evidence_image(
                            self._case_id, ev["id"], file_path
                        )
                        await self._audit_log(
                            "autonomous_loop", "image_analyzed",
                            f"Image analysee: {ev.get('title', '?')}",
                            target_type="evidence",
                            target_id=ev["id"],
                            cycle_number=self._cycle_count,
                        )
                        logger.info(
                            "ORIENT: Image analysed: {} ({})",
                            ev.get("title", "?"),
                            ev["id"][:8],
                        )
                    except Exception as e:
                        logger.warning(
                            "ORIENT: Image analysis failed for {}: {}",
                            ev["id"][:8], e,
                        )
        except Exception as e:
            logger.warning("ORIENT: Image analysis phase failed: {}", e)

    async def _orient_visual_embeddings(self, db: Database) -> None:
        """Index image evidence in DINOv2/CLIP for visual similarity search."""
        try:
            all_evidence = await db.list_evidence_by_case(self._case_id)

            image_evidence = [
                ev for ev in all_evidence
                if ev.get("evidence_type") == "image"
                and ev.get("file_path")
                and Path(ev["file_path"]).exists()
            ]

            if not image_evidence:
                return

            # Check which images are already indexed by looking at metadata
            to_index = []
            for ev in image_evidence:
                metadata = (
                    json.loads(ev.get("metadata") or "{}")
                    if isinstance(ev.get("metadata"), str)
                    else (ev.get("metadata") or {})
                )
                if not metadata.get("visual_embeddings_indexed"):
                    to_index.append(ev)

            if not to_index:
                return

            from nexus.vision.embeddings import VisualEmbedder
            from nexus.vision.image_search import ImageSearchEngine

            embedder = VisualEmbedder()
            search_engine = ImageSearchEngine(self._chroma, embedder)

            indexed_count = 0
            for ev in to_index:
                try:
                    search_engine.index_image(
                        evidence_id=ev["id"],
                        case_id=self._case_id,
                        image_path=ev["file_path"],
                        description=ev.get("summary", ""),
                    )
                    # Mark as indexed in metadata
                    metadata = (
                        json.loads(ev.get("metadata") or "{}")
                        if isinstance(ev.get("metadata"), str)
                        else (ev.get("metadata") or {})
                    )
                    metadata["visual_embeddings_indexed"] = True
                    await db.update_evidence(ev["id"], metadata=json.dumps(metadata))
                    indexed_count += 1
                except Exception as e:
                    logger.warning(
                        "ORIENT: Visual embedding failed for {}: {}",
                        ev["id"][:8], e,
                    )

            # Free GPU memory after batch embedding
            embedder.unload_all()

            if indexed_count > 0:
                logger.info(
                    "ORIENT: Indexed {} images in DINOv2/CLIP for case {}",
                    indexed_count, self._case_id,
                )
                await self._audit_log(
                    "autonomous_loop", "visual_embeddings_indexed",
                    f"{indexed_count} images indexees dans DINOv2/CLIP",
                    details={"count": indexed_count},
                    cycle_number=self._cycle_count,
                )

        except Exception as e:
            logger.warning("ORIENT: Visual embeddings phase failed: {}", e)

    # ================================================================
    # PHASE 3: DECIDE -- Analysis, hypotheses, contradictions,
    #                     forensics, timeline
    # ================================================================

    async def _decide(self, db: Database, new_evidence_ids: list[str]) -> dict[str, Any]:
        """Re-analyze the case if new evidence was added."""
        decisions: dict[str, Any] = {
            "analysis_run": None,
            "contradictions": [],
            "score_shifts": [],
        }

        if not new_evidence_ids:
            # Always run DECIDE on cycle 1 (initial analysis of existing evidence)
            if self._cycle_count == 1:
                logger.info(
                    "DECIDE: First cycle -- running initial analysis on existing evidence"
                )
            # Even without new evidence, periodically re-evaluate
            elif self._cycle_count % settings.full_reevaluation_every_n_cycles != 0:
                logger.debug(
                    "DECIDE: No new evidence and not a re-evaluation cycle, skipping"
                )
                return decisions
            logger.info(
                "DECIDE: Periodic full re-evaluation (cycle {})",
                self._cycle_count,
            )

        # --- 3a. Incremental analysis for new evidence ---
        if new_evidence_ids:
            self._track_tool("analysis_pipeline", "running", f"Analyse de {len(new_evidence_ids)} preuves", "core/analysis_pipeline.py")
            from nexus.core.analysis_pipeline import AnalysisPipeline

            for ev_id in new_evidence_ids:
                try:
                    pipeline = AnalysisPipeline(
                        db=db,
                        router=self._router,
                        chroma=self._chroma,
                        neo4j=self._neo4j,
                    )
                    run = await pipeline.run_incremental_analysis(
                        case_id=self._case_id,
                        trigger="autonomous_loop",
                        new_evidence_id=ev_id,
                    )
                    decisions["analysis_run"] = {
                        "id": run.id,
                        "status": run.status,
                    }
                    # Audit: log analysis completed
                    audit = AuditService(db)
                    await audit.log_analysis(
                        case_id=self._case_id,
                        run_id=run.id,
                        run_type="incremental",
                        status=run.status,
                        actor="autonomous_loop",
                    )
                    logger.info(
                        "DECIDE: Incremental analysis completed for evidence {}",
                        ev_id[:8],
                    )
                except Exception as e:
                    logger.error(
                        "DECIDE: Analysis failed for evidence {}: {}",
                        ev_id[:8],
                        e,
                    )

        if new_evidence_ids:
            self._track_tool("analysis_pipeline", "done", "Analyse incrementale terminee")

        # --- 3b. Re-evaluate ALL hypotheses ---
        self._track_tool("hypothesis_engine", "running", "Generation/evaluation hypotheses", "core/hypothesis_engine.py")
        try:
            from nexus.core.hypothesis_engine import HypothesisEngine

            engine = HypothesisEngine(
                db=db,
                router=self._router,
                chroma=self._chroma,
                neo4j=self._neo4j,
            )

            hypotheses = await db.list_hypotheses_by_case(
                self._case_id, status="active"
            )
            if not hypotheses:
                # No hypotheses yet -- generate initial ones
                logger.info(
                    "DECIDE: No hypotheses exist, generating initial set"
                )
                await engine.generate_hypotheses(self._case_id)
                hypotheses = await db.list_hypotheses_by_case(
                    self._case_id, status="active"
                )

            if hypotheses:
                snapshots = await engine.evaluate_all(self._case_id)
                decisions["score_shifts"] = [
                    s
                    for s in snapshots
                    if abs(s.get("delta", 0)) > 15
                ]
                # Audit: log each significant score shift
                audit = AuditService(db)
                for s in snapshots:
                    if abs(s.get("delta", 0)) > 5:
                        hyp = await db.get_hypothesis(s.get("hypothesis_id", ""))
                        hyp_title = hyp.get("title", "?") if hyp else "?"
                        await audit.log_hypothesis_scored(
                            case_id=self._case_id,
                            hyp_id=s.get("hypothesis_id", ""),
                            title=hyp_title,
                            old_score=s.get("previous_score", 0),
                            new_score=s.get("score", 0),
                            actor="autonomous_loop",
                        )
                logger.info(
                    "DECIDE: Re-evaluated {} hypotheses",
                    len(snapshots),
                )
        except Exception as e:
            logger.error("DECIDE: Hypothesis evaluation failed: {}", e)
            self._track_tool("hypothesis_engine", "error", str(e))
        else:
            self._track_tool("hypothesis_engine", "done", f"{len(snapshots) if 'snapshots' in dir() else 0} hypotheses evaluees")

        # --- 3c. Detect contradictions ---
        self._track_tool("contradiction_detector", "running", "Detection contradictions", "core/contradiction_detector.py")
        try:
            from nexus.core.contradiction_detector import (
                ContradictionDetector,
            )

            detector = ContradictionDetector(
                db=db, router=self._router
            )
            contradictions = await detector.detect_contradictions(
                self._case_id
            )
            decisions["contradictions"] = contradictions

            if contradictions:
                logger.info(
                    "DECIDE: Found {} contradictions",
                    len(contradictions),
                )
                alert_mgr = AlertManager(db)
                audit = AuditService(db)
                for c in contradictions:
                    await alert_mgr.create_contradiction_alert(
                        case_id=self._case_id,
                        details=c.get("description", str(c)),
                    )
                    # Audit: log each contradiction
                    await audit.log_contradiction_found(
                        case_id=self._case_id,
                        description=c.get("description", str(c)),
                        actor="autonomous_loop",
                    )

                # --- Auto-adjust hypothesis scores based on contradictions ---
                hypotheses = await db.list_hypotheses_by_case(
                    self._case_id, status="active"
                )
                for c in contradictions:
                    c_text = (
                        c.get("description", "")
                        + " " + str(c.get("evidence_a_title", ""))
                        + " " + str(c.get("evidence_b_title", ""))
                    ).lower()
                    c_words = {w for w in c_text.split() if len(w) > 3}
                    for h in hypotheses:
                        h_text = (
                            h.get("title", "") + " " + h.get("description", "")
                        ).lower()
                        h_words = {w for w in h_text.split() if len(w) > 3}
                        overlap = c_words & h_words
                        if len(overlap) >= 2:
                            current_score = h.get("current_score", 50.0)
                            new_score = max(0.0, current_score - 5.0)
                            if new_score != current_score:
                                await db.update_hypothesis(
                                    h["id"], current_score=new_score
                                )
                                await db.create_hypothesis_snapshot(
                                    hypothesis_id=h["id"],
                                    score=new_score,
                                    contradicting=[c.get("description", "")],
                                    reasoning=(
                                        f"Auto-downgrade: contradiction overlap "
                                        f"({', '.join(sorted(overlap)[:5])})"
                                    ),
                                    trigger="contradiction_auto_adjust",
                                )
                                logger.info(
                                    "DECIDE: Hypothesis '{}' score adjusted {:.0f} -> {:.0f} due to contradiction",
                                    h.get("title", "?")[:30],
                                    current_score,
                                    new_score,
                                )
        except Exception as e:
            logger.error("DECIDE: Contradiction detection failed: {}", e)
            self._track_tool("contradiction_detector", "error", str(e))

        # --- 3d. Score all suspects ---
        if settings.auto_suspect_scoring:
            self._track_tool("suspect_scorer", "running", "Scoring suspects 5 facteurs", "core/suspect_scorer.py")
            await self._decide_suspect_scoring(db)

        # --- 3e. Forensic analysis on image evidence ---
        if settings.auto_forensic_analysis:
            self._track_tool("forensics", "running", "BPA + traces + acoustique", "forensics/")
            await self._decide_forensic_analysis(db)
            self._track_tool("forensics", "done", "Analyse forensique terminee")

        # --- 3f. Rebuild timeline ---
        if settings.auto_timeline_rebuild:
            self._track_tool("timeline_builder", "running", "Reconstruction chronologique", "core/timeline_builder.py")
            await self._decide_timeline(db)
            self._track_tool("timeline_builder", "done", "Timeline reconstruite")

        # --- 3g. Rebuild summary tree periodically (every 3 cycles) ---
        if self._cycle_count % 3 == 0:
            try:
                from nexus.core.summary_tree import SummaryTree

                tree = SummaryTree(db, self._router, self._chroma)
                await tree.rebuild_tree(self._case_id)
                logger.info(
                    "DECIDE: Summary tree rebuilt for case {} (cycle {})",
                    self._case_id[:8],
                    self._cycle_count,
                )
                await self._audit_log(
                    "autonomous_loop", "summary_tree_rebuilt",
                    f"Arbre de resumes RAPTOR reconstruit (cycle {self._cycle_count})",
                    cycle_number=self._cycle_count,
                )
            except Exception as e:
                logger.warning("DECIDE: Summary tree rebuild failed: {}", e)

        return decisions

    async def _decide_suspect_scoring(self, db: Database) -> None:
        """Score all person entities as suspects and evaluate profiles."""
        try:
            from nexus.core.suspect_scorer import SuspectScorer

            scorer = SuspectScorer(
                db=db, router=self._router, neo4j=self._neo4j
            )
            results = await scorer.score_all_suspects(
                self._case_id, trigger="autonomous_loop"
            )

            if results:
                logger.info(
                    "DECIDE: Scored {} suspects for case {}",
                    len(results), self._case_id,
                )
                # Log top suspects
                for r in results[:3]:
                    f = r.get("factors", {})
                    await self._audit_log(
                        "autonomous_loop", "suspect_scored",
                        f"Suspect {r.get('name', '?')}: {r.get('score', 0):.1f} "
                        f"(G={f.get('graph', 0):.0f} E={f.get('evidence', 0):.0f} "
                        f"C={f.get('contradiction', 0):.0f} P={f.get('profile', 0):.0f} "
                        f"H={f.get('hypothesis', 0):.0f})",
                        target_type="suspect",
                        target_id=r.get("suspect_id"),
                        details=r,
                        cycle_number=self._cycle_count,
                    )

            # Evaluate profiles via LLM periodically
            if (
                settings.auto_suspect_profile_every_n_cycles > 0
                and self._cycle_count % settings.auto_suspect_profile_every_n_cycles == 0
                and results  # Only if there are suspects to profile
            ):
                for r in results:
                    entity_id = r.get("entity_id", "")
                    name = r.get("name", "?")
                    profile_score = r.get("factors", {}).get("profile", 0)
                    # Evaluate if never profiled (0) or periodically refresh (every 9 cycles)
                    needs_profile = (
                        profile_score == 0
                        or self._cycle_count % (settings.auto_suspect_profile_every_n_cycles * 3) == 0
                    )
                    if entity_id and needs_profile:
                        try:
                            await scorer.evaluate_profile(
                                self._case_id, entity_id
                            )
                            logger.info(
                                "DECIDE: Profile evaluated for suspect {} ({})",
                                entity_id[:8], name,
                            )
                        except Exception as exc:
                            logger.warning(
                                "DECIDE: Profile evaluation failed for {}: {}",
                                name, exc,
                            )

        except Exception as e:
            logger.error("DECIDE: Suspect scoring failed: {}", e)

    async def _decide_forensic_analysis(self, db: Database) -> None:
        """Run forensic analysis (blood pattern, traces) on image evidence."""
        try:
            all_evidence = await db.list_evidence_by_case(self._case_id)

            for ev in all_evidence:
                if ev.get("evidence_type") != "image":
                    continue

                file_path = ev.get("file_path", "")
                if not file_path or not Path(file_path).exists():
                    continue

                # Check metadata to see if forensic analysis was already done
                metadata = (
                    json.loads(ev.get("metadata") or "{}")
                    if isinstance(ev.get("metadata"), str)
                    else (ev.get("metadata") or {})
                )
                if metadata.get("forensic_analyzed"):
                    continue

                # Determine what kind of forensic analysis to run based
                # on the evidence summary/description (keywords)
                summary = (ev.get("summary") or "").lower()
                title = (ev.get("title") or "").lower()
                combined = f"{summary} {title}"

                # Blood pattern analysis
                blood_keywords = (
                    "sang", "blood", "tache", "spatter", "eclaboussure",
                    "hemoglobine", "rouge", "flaque",
                )
                if any(kw in combined for kw in blood_keywords):
                    try:
                        from nexus.forensics.blood_pattern import BloodPatternAnalyzer

                        bpa = BloodPatternAnalyzer(self._router)
                        result = await bpa.classify_pattern(file_path)
                        metadata["bpa_result"] = result
                        # Append BPA findings to evidence summary so retriever can find them
                        bpa_summary = (
                            f"\n[FORENSIQUE BPA]: {result.get('pattern_type', 'inconnu')}"
                            f" — {(result.get('description') or '')[:300]}"
                        )
                        current_summary = ev.get("summary") or ""
                        await db.update_evidence(ev["id"], summary=current_summary + bpa_summary)
                        await self._audit_log(
                            "autonomous_loop", "forensic_bpa",
                            f"Analyse BPA: {ev.get('title', '?')} -> {result.get('pattern_type', 'inconnu')}",
                            target_type="evidence",
                            target_id=ev["id"],
                            details=result,
                            cycle_number=self._cycle_count,
                        )
                        logger.info(
                            "DECIDE: BPA classification done for {}",
                            ev["id"][:8],
                        )
                    except Exception as e:
                        logger.warning(
                            "DECIDE: BPA failed for {}: {}",
                            ev["id"][:8], e,
                        )

                # Trace analysis (fingerprints, shoe prints, tire tracks, etc.)
                trace_keywords = (
                    "empreinte", "fingerprint", "trace", "pneu", "tire",
                    "chaussure", "shoe", "outil", "tool", "verre", "glass",
                    "fibre", "cheveu", "hair",
                )
                if any(kw in combined for kw in trace_keywords):
                    try:
                        from nexus.forensics.trace_analyzer import TraceAnalyzer

                        tracer = TraceAnalyzer(self._router)
                        result = await tracer.analyze_trace(file_path, trace_type="auto")
                        metadata["trace_result"] = result
                        # Append trace findings to evidence summary so retriever can find them
                        trace_summary = (
                            f"\n[FORENSIQUE TRACE]: type={result.get('type', 'auto')}"
                            f" — {(result.get('description') or result.get('analysis') or '')[:300]}"
                        )
                        current_summary = ev.get("summary") or ""
                        await db.update_evidence(ev["id"], summary=current_summary + trace_summary)
                        await self._audit_log(
                            "autonomous_loop", "forensic_trace",
                            f"Analyse trace: {ev.get('title', '?')} -> type={result.get('type', 'auto')}",
                            target_type="evidence",
                            target_id=ev["id"],
                            details=result,
                            cycle_number=self._cycle_count,
                        )
                        logger.info(
                            "DECIDE: Trace analysis done for {}",
                            ev["id"][:8],
                        )
                    except Exception as e:
                        logger.warning(
                            "DECIDE: Trace analysis failed for {}: {}",
                            ev["id"][:8], e,
                        )

                # Mark as forensic-analyzed to avoid re-processing
                metadata["forensic_analyzed"] = True
                await db.update_evidence(ev["id"], metadata=json.dumps(metadata))

            # --- Audio forensic analysis ---
            for ev in all_evidence:
                if ev.get("evidence_type") != "audio":
                    continue

                file_path = ev.get("file_path", "")
                if not file_path or not Path(file_path).exists():
                    continue

                metadata = (
                    json.loads(ev.get("metadata") or "{}")
                    if isinstance(ev.get("metadata"), str)
                    else (ev.get("metadata") or {})
                )
                if metadata.get("acoustic_analyzed"):
                    continue

                try:
                    from nexus.forensics.acoustic_analysis import AcousticAnalyzer

                    acoustic = AcousticAnalyzer(self._router)
                    result = await acoustic.analyze_audio_forensic(file_path)
                    metadata["acoustic_result"] = {
                        "transcription": result.get("transcription", "")[:2000],
                        "events_count": len(result.get("events", [])),
                        "status": result.get("status"),
                    }
                    metadata["acoustic_analyzed"] = True
                    await db.update_evidence(ev["id"], metadata=json.dumps(metadata))

                    # If transcription was produced, store it as raw_text
                    transcription = result.get("transcription", "")
                    if transcription and not ev.get("raw_text"):
                        await db.update_evidence(ev["id"], raw_text=transcription)

                    # Append acoustic findings to evidence summary so retriever can find them
                    forensic_text = result.get("forensic_analysis") or ""
                    events = result.get("events", [])
                    events_desc = ", ".join(
                        e.get("description", "") for e in events[:5]
                    ) if events else ""
                    acoustic_summary = (
                        f"\n[FORENSIQUE ACOUSTIQUE]: {forensic_text[:300]}"
                    )
                    if events_desc:
                        acoustic_summary += f" Evenements: {events_desc[:200]}"
                    current_summary = ev.get("summary") or ""
                    await db.update_evidence(ev["id"], summary=current_summary + acoustic_summary)

                    await self._audit_log(
                        "autonomous_loop", "forensic_acoustic",
                        f"Analyse acoustique: {ev.get('title', '?')} -> "
                        f"{len(result.get('events', []))} evenements detectes",
                        target_type="evidence",
                        target_id=ev["id"],
                        details=metadata["acoustic_result"],
                        cycle_number=self._cycle_count,
                    )
                    logger.info(
                        "DECIDE: Acoustic analysis done for {}",
                        ev["id"][:8],
                    )
                except Exception as e:
                    logger.warning(
                        "DECIDE: Acoustic analysis failed for {}: {}",
                        ev["id"][:8], e,
                    )

        except Exception as e:
            logger.warning("DECIDE: Forensic analysis phase failed: {}", e)

    async def _decide_timeline(self, db: Database) -> None:
        """Rebuild the chronological timeline for the case."""
        try:
            from nexus.core.timeline_builder import TimelineBuilder

            builder = TimelineBuilder(db, self._neo4j)
            timeline = await builder.build_timeline(self._case_id)

            if timeline:
                # Build a detailed timeline summary for the analysis context
                timeline_lines = []
                for evt in timeline[:30]:  # cap at 30 events
                    date = evt.get("date", "?")
                    desc = (evt.get("description") or evt.get("event") or "")[:120]
                    source = evt.get("source", "")
                    line = f"  - [{date}] {desc}"
                    if source:
                        line += f" (source: {source})"
                    timeline_lines.append(line)
                timeline_text = "\n".join(timeline_lines)
                output_summary = (
                    f"CHRONOLOGIE ({len(timeline)} evenements, "
                    f"du {timeline[0].get('date', '?')} au {timeline[-1].get('date', '?')}):\n"
                    f"{timeline_text}"
                )

                # Store timeline as analysis run — output_summary readable by retriever
                await db.create_analysis_run(
                    case_id=self._case_id,
                    run_type="timeline_rebuild",
                    trigger="autonomous_loop",
                    status="completed",
                    model_used="N/A",
                    input_summary=f"Timeline cycle {self._cycle_count}",
                    output_summary=output_summary[:4000],
                )
                await self._audit_log(
                    "autonomous_loop", "timeline_rebuilt",
                    f"Timeline reconstruite: {len(timeline)} evenements",
                    details={
                        "count": len(timeline),
                        "earliest": timeline[0].get("date") if timeline else None,
                        "latest": timeline[-1].get("date") if timeline else None,
                    },
                    cycle_number=self._cycle_count,
                )
                logger.info(
                    "DECIDE: Timeline rebuilt with {} entries for case {}",
                    len(timeline), self._case_id,
                )
        except Exception as e:
            logger.warning("DECIDE: Timeline rebuild failed: {}", e)

    # ================================================================
    # PHASE 4: ACT -- Generate search queries, OSINT enrichment,
    #                  domain recon
    # ================================================================

    async def _act(self, db: Database, decisions: dict[str, Any]) -> None:
        """Adapt monitoring based on what we learned."""
        # --- 4a. Generate new search queries (existing) ---
        await self._act_generate_queries(db, decisions)

        # --- 4b. OSINT enrichment: create monitoring jobs from recon results ---
        await self._act_osint_enrichment(db)

        # --- 4c. Domain recon on email entity domains ---
        if settings.auto_domain_recon:
            await self._act_domain_recon(db)

    async def _act_generate_queries(self, db: Database, decisions: dict[str, Any]) -> None:
        """Ask the LLM to generate new search queries based on current state."""
        try:
            # Get current state
            hypotheses = await db.list_hypotheses_by_case(
                self._case_id, status="active"
            )
            entities = await db.list_entities_by_case(self._case_id)
            existing_jobs = await db.list_jobs_by_case(self._case_id)
            existing_queries = {j["query"] for j in existing_jobs}

            if not hypotheses:
                logger.debug("ACT: No hypotheses, skipping query generation")
                return

            # Ask the LLM: "Based on current hypotheses and evidence,
            # what should we search for next?"
            hypotheses_text = "\n".join(
                [
                    f"- {h['title']} (score: {h['current_score']})"
                    for h in hypotheses
                ]
            )
            entities_text = "\n".join(
                [
                    f"- {e['name']} ({e['entity_type']})"
                    for e in entities[:20]
                ]
            )
            queries_text = "\n".join(
                [f"- {q}" for q in existing_queries]
            )
            contradictions_text = "\n".join(
                [
                    str(c)
                    for c in decisions.get("contradictions", [])[:5]
                ]
            )

            prompt = ADAPTIVE_QUERY_PROMPT.format(
                hypotheses=hypotheses_text or "(aucune)",
                entities=entities_text or "(aucune)",
                existing_queries=queries_text or "(aucune)",
                contradictions=contradictions_text or "(aucune)",
            )

            response = await self._router.route_json(
                TaskType.QUERY_REFORMULATION, prompt
            )

            new_queries = response.get("queries", [])
            max_new = settings.max_new_queries_per_cycle

            created_count = 0
            for q in new_queries:
                if created_count >= max_new:
                    break

                query_text = (
                    q.get("query", q)
                    if isinstance(q, dict)
                    else str(q)
                )

                if not query_text or query_text in existing_queries:
                    continue

                await db.create_monitoring_job(
                    case_id=self._case_id,
                    job_type="searxng",
                    query=query_text,
                    interval_hours=12,
                )
                created_count += 1
                # Audit: log query generation
                audit = AuditService(db)
                await audit.log_query_generated(
                    case_id=self._case_id,
                    query=query_text,
                    cycle=self._cycle_count,
                )
                logger.info(
                    "ACT: New monitoring job created: '{}'",
                    query_text[:60],
                )

            if created_count:
                logger.info(
                    "ACT: Created {} new monitoring jobs for case {}",
                    created_count,
                    self._case_id,
                )

        except Exception as e:
            logger.error(
                "ACT: Adaptive query generation failed: {}", e
            )

    async def _act_osint_enrichment(self, db: Database) -> None:
        """Create monitoring jobs from OSINT recon results.

        If holehe found an Instagram account, create a monitoring job
        to periodically search for that profile. If social recon found
        a Twitter profile, monitor it.
        """
        try:
            entities = await db.list_entities_by_case(self._case_id)
            existing_jobs = await db.list_jobs_by_case(self._case_id)
            existing_queries = {j["query"] for j in existing_jobs}

            created_count = 0

            for ent in entities:
                metadata = (
                    json.loads(ent.get("metadata") or "{}")
                    if isinstance(ent.get("metadata"), str)
                    else (ent.get("metadata") or {})
                )

                # Skip if already enriched
                if metadata.get("enrichment_done"):
                    continue

                # From holehe results: monitor discovered accounts
                holehe_results = metadata.get("holehe_results", [])
                for hit in holehe_results[:5]:  # Cap at 5 services per entity
                    site = hit.get("site", "")
                    if not site:
                        continue
                    query = f'"{ent["name"]}" site:{site}'
                    if query not in existing_queries:
                        await db.create_monitoring_job(
                            case_id=self._case_id,
                            job_type="searxng",
                            query=query,
                            interval_hours=24,
                        )
                        existing_queries.add(query)
                        created_count += 1

                # From social recon: monitor found profiles
                social_profiles = metadata.get("social_profiles", [])
                for profile in social_profiles[:5]:
                    url = profile.get("url", "")
                    platform = profile.get("platform", "")
                    if not url:
                        continue
                    query = f'"{ent["name"]}" site:{platform}.com'
                    if query not in existing_queries:
                        await db.create_monitoring_job(
                            case_id=self._case_id,
                            job_type="searxng",
                            query=query,
                            interval_hours=24,
                        )
                        existing_queries.add(query)
                        created_count += 1

                if holehe_results or social_profiles:
                    metadata["enrichment_done"] = True
                    await db.update_entity(
                        ent["id"], metadata=json.dumps(metadata)
                    )

            if created_count:
                logger.info(
                    "ACT: Created {} OSINT enrichment monitoring jobs for case {}",
                    created_count, self._case_id,
                )
                await self._audit_log(
                    "autonomous_loop", "osint_enrichment",
                    f"{created_count} jobs de monitoring OSINT crees",
                    details={"count": created_count},
                    cycle_number=self._cycle_count,
                )

        except Exception as e:
            logger.warning("ACT: OSINT enrichment failed: {}", e)

    async def _act_domain_recon(self, db: Database) -> None:
        """Run WHOIS/DNS recon on domains from email entities."""
        try:
            entities = await db.list_entities_by_case(
                self._case_id, entity_type="email"
            )

            for ent in entities:
                name = ent.get("name", "")
                if "@" not in name:
                    continue

                metadata = (
                    json.loads(ent.get("metadata") or "{}")
                    if isinstance(ent.get("metadata"), str)
                    else (ent.get("metadata") or {})
                )

                # Skip if domain recon already done
                if metadata.get("domain_recon_done"):
                    continue

                domain = name.split("@")[1]

                # Skip common freemail domains (no investigative value)
                freemail_domains = {
                    "gmail.com", "yahoo.com", "hotmail.com", "outlook.com",
                    "live.com", "aol.com", "icloud.com", "protonmail.com",
                    "proton.me", "mail.com", "gmx.com", "yandex.com",
                }
                if domain.lower() in freemail_domains:
                    metadata["domain_recon_done"] = True
                    metadata["domain_recon_skipped"] = "freemail"
                    await db.update_entity(ent["id"], metadata=json.dumps(metadata))
                    continue

                try:
                    from nexus.recon.domain_recon import DomainRecon

                    drecon = DomainRecon()

                    whois_info = await drecon.whois_lookup(domain)
                    dns_info = await drecon.dns_lookup(domain)

                    metadata["domain_recon_done"] = True
                    metadata["whois"] = {
                        "registrar": whois_info.get("registrar"),
                        "creation_date": whois_info.get("creation_date"),
                        "registrant_name": whois_info.get("registrant_name"),
                        "registrant_email": whois_info.get("registrant_email"),
                        "name_servers": whois_info.get("name_servers", []),
                    }
                    metadata["dns"] = dns_info

                    await db.update_entity(ent["id"], metadata=json.dumps(metadata))

                    await self._audit_log(
                        "autonomous_loop", "domain_recon",
                        f"Domain recon {domain}: registrar={whois_info.get('registrar', '?')}",
                        target_type="entity",
                        target_id=ent["id"],
                        details={"domain": domain, "whois": metadata["whois"], "dns": dns_info},
                        cycle_number=self._cycle_count,
                    )
                    logger.info(
                        "ACT: Domain recon done for {} (entity {})",
                        domain, ent["id"][:8],
                    )

                    # Rate-limit
                    await asyncio.sleep(settings.auto_recon_rate_limit)

                except Exception as e:
                    logger.warning(
                        "ACT: Domain recon failed for {}: {}", domain, e,
                    )
                    metadata["domain_recon_done"] = True
                    metadata["domain_recon_error"] = str(e)
                    await db.update_entity(ent["id"], metadata=json.dumps(metadata))

        except Exception as e:
            logger.warning("ACT: Domain recon phase failed: {}", e)

    # ================================================================
    # PHASE 5: QUESTION -- Self-questioning, reports, backups
    # ================================================================

    async def _question(self, db: Database) -> None:
        """Self-questioning, periodic reports, and automated backups."""
        # --- 5a. Adversarial self-questioning (existing) ---
        await self._question_self_questioning(db)

        # --- 5b. Periodic report generation ---
        if (
            settings.auto_report_every_n_cycles > 0
            and self._cycle_count % settings.auto_report_every_n_cycles == 0
        ):
            await self._question_periodic_report(db)

        # --- 5c. Automated backup ---
        if (
            settings.auto_backup_every_n_cycles > 0
            and self._cycle_count % settings.auto_backup_every_n_cycles == 0
        ):
            await self._question_backup()

    async def _question_self_questioning(self, db: Database) -> None:
        """Adversarial thinking against the top hypothesis."""
        try:
            hypotheses = await db.list_hypotheses_by_case(
                self._case_id, status="active"
            )
            if not hypotheses:
                logger.debug(
                    "QUESTION: No hypotheses to question for case {}",
                    self._case_id,
                )
                return

            # Get top hypothesis
            top = max(
                hypotheses,
                key=lambda h: h.get("current_score", 0),
            )

            evidence = await db.list_evidence_by_case(self._case_id)
            evidence_summaries = "\n".join(
                [
                    f"- [{e.get('title', '?')}]: "
                    f"{(e.get('summary') or '')[:200]}"
                    for e in evidence[:15]
                ]
            )

            all_hyps_text = "\n".join(
                [
                    f"- {h['title']} ({h['current_score']}%)"
                    for h in hypotheses
                ]
            )

            prompt = SELF_QUESTIONING_PROMPT.format(
                top_hypothesis=top["title"],
                top_score=top["current_score"],
                top_description=top.get("description", ""),
                all_hypotheses=all_hyps_text,
                evidence_summaries=evidence_summaries or "(aucune preuve)",
            )

            response = await self._router.route(
                TaskType.DEEP_ANALYSIS, prompt
            )

            # Store the questioning result as a special analysis run
            await db.create_analysis_run(
                case_id=self._case_id,
                run_type="self_questioning",
                trigger="autonomous_loop",
                status="completed",
                model_used="nexus",
                input_summary=(
                    f"Self-questioning cycle {self._cycle_count} -- "
                    f"Top hypothesis: {top['title']} ({top['current_score']}%)"
                ),
                output_summary=response[:2000],
            )

            # Audit: log self-questioning
            audit = AuditService(db)
            await audit.log_self_questioning(
                case_id=self._case_id,
                top_hypothesis=top["title"],
                summary=response,
                cycle=self._cycle_count,
            )

            logger.info(
                "QUESTION: Self-questioning completed for case {}",
                self._case_id,
            )

        except Exception as e:
            logger.error(
                "QUESTION: Self-questioning failed for case {}: {}",
                self._case_id,
                e,
            )

    async def _question_periodic_report(self, db: Database) -> None:
        """Generate a periodic progression report."""
        try:
            from nexus.export.report_generator import ReportGenerator

            gen = ReportGenerator(db, self._router)
            report = await gen.generate_summary_report(self._case_id)

            # Store report as an analysis run
            report_summary = (
                report.get("sections", {}).get("summary", "")[:2000]
            )
            await db.create_analysis_run(
                case_id=self._case_id,
                run_type="periodic_report",
                trigger="autonomous_loop",
                status="completed",
                model_used="nexus",
                input_summary=f"Rapport periodique cycle {self._cycle_count}",
                output_summary=report_summary,
            )

            await self._audit_log(
                "autonomous_loop", "periodic_report",
                f"Rapport periodique genere (cycle {self._cycle_count})",
                details={
                    "cycle": self._cycle_count,
                    "evidence_count": report.get("sections", {}).get("evidence_count"),
                    "hypotheses_count": report.get("sections", {}).get("hypotheses_count"),
                    "unread_alerts": report.get("sections", {}).get("unread_alerts"),
                },
                cycle_number=self._cycle_count,
            )
            logger.info(
                "QUESTION: Periodic report generated for case {} (cycle {})",
                self._case_id, self._cycle_count,
            )

        except Exception as e:
            logger.warning(
                "QUESTION: Periodic report generation failed: {}", e,
            )

    async def _question_backup(self) -> None:
        """Run an automated database backup."""
        try:
            from nexus.core.backup import BackupManager

            bm = BackupManager()
            backup_id = await bm.create_backup()

            await self._audit_log(
                "autonomous_loop", "auto_backup",
                f"Backup automatique cree: {backup_id}",
                details={"backup_id": backup_id, "cycle": self._cycle_count},
                cycle_number=self._cycle_count,
            )
            logger.info(
                "QUESTION: Auto backup created for case {} -> {}",
                self._case_id, backup_id,
            )

        except Exception as e:
            logger.warning("QUESTION: Auto backup failed: {}", e)
