"""
NEXUS -- Wiki Compiler.

Compiles investigation data into a live Markdown wiki per case.
Each case gets a wiki/ directory with interlinked pages organized
by type (entities, events, locations, evidence, analysis).

Uses gemma4:e4b (fast LLM) for compilation — no heavy VRAM usage.
"""

from __future__ import annotations

import hashlib
import re
from pathlib import Path
from typing import Any, Optional

from loguru import logger

from nexus.config import settings
from nexus.db.sqlite_db import Database
from nexus.llm.router import LLMRouter, TaskType


class WikiCompiler:
    """Compiles investigation data into a Markdown wiki."""

    def __init__(self, db: Database, router: LLMRouter) -> None:
        self._db = db
        self._router = router

    def _wiki_dir(self, case_id: str) -> Path:
        """Get wiki directory for a case, creating it if needed."""
        d = settings.data_dir / "cases" / case_id / "wiki"
        d.mkdir(parents=True, exist_ok=True)
        return d

    def _slugify(self, text: str) -> str:
        """Convert text to a filesystem-safe slug."""
        s = text.lower().strip()
        s = re.sub(r'[àâä]', 'a', s)
        s = re.sub(r'[éèêë]', 'e', s)
        s = re.sub(r'[îï]', 'i', s)
        s = re.sub(r'[ôö]', 'o', s)
        s = re.sub(r'[ùûü]', 'u', s)
        s = re.sub(r'[ç]', 'c', s)
        s = re.sub(r'[^a-z0-9]+', '-', s)
        return s.strip('-')[:60]

    def _read_page(self, wiki_dir: Path, page_path: str) -> str:
        """Read existing page content, or empty string."""
        p = wiki_dir / page_path
        if p.exists():
            return p.read_text(encoding="utf-8")
        return ""

    def _write_page(self, wiki_dir: Path, page_path: str, content: str) -> str:
        """Write page content and return its SHA256 hash."""
        p = wiki_dir / page_path
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(content, encoding="utf-8")
        return hashlib.sha256(content.encode()).hexdigest()[:16]

    # ------------------------------------------------------------------
    # compile_evidence
    # ------------------------------------------------------------------

    async def compile_evidence(self, case_id: str, evidence_id: str) -> list[str]:
        """Compile an evidence item into wiki pages. Returns list of updated page paths."""
        ev = await self._db.get_evidence(evidence_id)
        if not ev:
            return []

        wiki_dir = self._wiki_dir(case_id)
        updated_pages: list[str] = []

        # Get entities linked to this evidence
        mentions = await self._db.list_mentions_by_evidence(evidence_id)
        entity_ids = [m["entity_id"] for m in mentions]
        entities = []
        for eid in entity_ids:
            ent = await self._db.get_entity(eid)
            if ent:
                entities.append(ent)

        entities_text = "\n".join(
            f"- {e['name']} ({e['entity_type']})" for e in entities
        ) or "(aucune)"

        # 1. Compile evidence page
        slug = self._slugify(ev.get("title", evidence_id[:8]))
        page_path = f"evidence/{slug}.md"
        existing = self._read_page(wiki_dir, page_path)

        from nexus.llm.prompts import WIKI_COMPILE_EVIDENCE_PROMPT
        prompt = WIKI_COMPILE_EVIDENCE_PROMPT.format(
            title=ev.get("title", "?"),
            source=ev.get("source", "inconnu"),
            source_date=ev.get("source_date", "inconnue"),
            reliability=ev.get("reliability", 50),
            content=(ev.get("summary") or ev.get("raw_text", ""))[:3000],
            entities=entities_text,
            existing_page=existing or "(nouvelle page)",
        )

        try:
            content = await self._router.route(TaskType.EVIDENCE_SUMMARY, prompt)
            content_hash = self._write_page(wiki_dir, page_path, content)
            await self._db.upsert_wiki_page(
                case_id=case_id, page_path=page_path, page_type="evidence",
                title=ev.get("title", "?"), content_hash=content_hash,
                source_ids=[evidence_id],
            )
            updated_pages.append(page_path)
        except Exception as exc:
            logger.error("Wiki compile evidence failed: {}", exc)

        # 2. Update/create entity pages
        for ent in entities:
            try:
                ent_slug = self._slugify(ent["name"])
                ent_type = ent.get("entity_type", "other")
                folder = "locations" if ent_type == "location" else "entities"
                ent_path = f"{folder}/{ent_slug}.md"
                ent_existing = self._read_page(wiki_dir, ent_path)

                from nexus.llm.prompts import WIKI_UPDATE_ENTITY_PROMPT
                ent_prompt = WIKI_UPDATE_ENTITY_PROMPT.format(
                    entity_name=ent["name"],
                    entity_type=ent_type,
                    description=ent.get("description", ""),
                    new_info=f"Mentionne dans: {ev.get('title', '?')}\nContexte: {(ev.get('summary') or '')[:500]}",
                    existing_page=ent_existing or "(nouvelle page)",
                )

                ent_content = await self._router.route(TaskType.EVIDENCE_SUMMARY, ent_prompt)
                ent_hash = self._write_page(wiki_dir, ent_path, ent_content)
                await self._db.upsert_wiki_page(
                    case_id=case_id, page_path=ent_path,
                    page_type="location" if ent_type == "location" else "entity",
                    title=ent["name"], content_hash=ent_hash,
                )
                updated_pages.append(ent_path)
            except Exception as exc:
                logger.debug("Wiki entity page failed for {}: {}", ent["name"], exc)

        # 3. Update index
        await self.rebuild_index(case_id)

        # 4. Update log
        await self.update_log(case_id, "evidence_compiled", f"Compiled: {ev.get('title', '?')}")

        logger.info("Wiki compiled evidence '{}': {} pages updated", ev.get("title", "?")[:40], len(updated_pages))
        return updated_pages

    # ------------------------------------------------------------------
    # compile_hypothesis_update
    # ------------------------------------------------------------------

    async def compile_hypothesis_update(self, case_id: str) -> str | None:
        """Recompile the hypotheses analysis page."""
        wiki_dir = self._wiki_dir(case_id)
        page_path = "analysis/hypotheses.md"

        hypotheses = await self._db.list_hypotheses_by_case(case_id)
        contradictions = await self._db.list_contradictions_by_case(case_id)
        suspects = await self._db.list_suspects_by_case(case_id)

        hyp_text = "\n".join(
            f"- [{h.get('current_score', 0):.0f}%] {h.get('title', '?')}: {h.get('description', '')[:100]}"
            for h in hypotheses
        ) or "(aucune hypothese)"

        contra_text = "\n".join(
            f"- [{c.get('severity', '?')}] {c.get('description', '')[:100]}"
            for c in contradictions[:10]
        ) or "(aucune)"

        sus_text = "\n".join(
            f"- {s.get('entity_name', '?')} (score: {s.get('suspicion_score', 0):.1f})"
            for s in suspects[:10]
        ) or "(aucun)"

        existing = self._read_page(wiki_dir, page_path)

        from nexus.llm.prompts import WIKI_COMPILE_HYPOTHESES_PROMPT
        prompt = WIKI_COMPILE_HYPOTHESES_PROMPT.format(
            hypotheses=hyp_text,
            contradictions=contra_text,
            suspects=sus_text,
            existing_page=existing or "(nouvelle page)",
        )

        try:
            content = await self._router.route(TaskType.EVIDENCE_SUMMARY, prompt)
            content_hash = self._write_page(wiki_dir, page_path, content)
            await self._db.upsert_wiki_page(
                case_id=case_id, page_path=page_path, page_type="analysis",
                title="Hypotheses actives", content_hash=content_hash,
            )
            await self.update_log(case_id, "hypotheses_compiled", f"{len(hypotheses)} hypotheses")
            return page_path
        except Exception as exc:
            logger.error("Wiki hypothesis compilation failed: {}", exc)
            return None

    # ------------------------------------------------------------------
    # rebuild_index
    # ------------------------------------------------------------------

    async def rebuild_index(self, case_id: str) -> None:
        """Regenerate index.md from all wiki pages."""
        wiki_dir = self._wiki_dir(case_id)
        pages = await self._db.list_wiki_pages(case_id)
        case = await self._db.get_case(case_id)

        evidence_links = []
        entity_links = []
        location_links = []
        analysis_links = []

        for p in pages:
            link = f"- [[{p['page_path']}|{p['title']}]]"
            if p["page_type"] == "evidence":
                evidence_links.append(link)
            elif p["page_type"] == "entity":
                entity_links.append(link)
            elif p["page_type"] == "location":
                location_links.append(link)
            elif p["page_type"] == "analysis":
                analysis_links.append(link)

        from nexus.llm.prompts import WIKI_INDEX_TEMPLATE
        from datetime import datetime
        content = WIKI_INDEX_TEMPLATE.format(
            case_name=case.get("name", "?") if case else "?",
            reference=case.get("reference", "?") if case else "?",
            last_compiled=datetime.now().strftime("%Y-%m-%d %H:%M"),
            evidence_count=len(evidence_links),
            evidence_links="\n".join(evidence_links) or "_(aucune preuve compilee)_",
            entity_count=len(entity_links),
            entity_links="\n".join(entity_links) or "_(aucune entite)_",
            location_count=len(location_links),
            location_links="\n".join(location_links) or "_(aucun lieu)_",
        )

        self._write_page(wiki_dir, "index.md", content)

    # ------------------------------------------------------------------
    # update_log
    # ------------------------------------------------------------------

    async def update_log(self, case_id: str, action: str, details: str) -> None:
        """Append entry to log.md."""
        wiki_dir = self._wiki_dir(case_id)
        log_path = wiki_dir / "log.md"
        from datetime import datetime
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        entry = f"- **{timestamp}** | `{action}` | {details}\n"

        if log_path.exists():
            existing = log_path.read_text(encoding="utf-8")
        else:
            existing = "# Journal de compilation\n\n"

        log_path.write_text(existing + entry, encoding="utf-8")
