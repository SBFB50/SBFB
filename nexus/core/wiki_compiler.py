"""
NEXUS -- Wiki Compiler.

Compiles investigation data into a live Markdown wiki per case.
Each case gets a wiki/ directory with interlinked pages organized
by type (entities, events, locations, evidence, analysis).

Uses gemma4:e4b (fast LLM) for compilation -- no heavy VRAM usage.

Features:
- YAML frontmatter with coverage tags, provenance, and metadata
- Provenance markers ([inferred], [ambiguous], [source: X])
- Contradiction flags from the contradictions table
- Cross-linking: auto-wraps entity mentions in [[wikilinks]]
"""

from __future__ import annotations

import hashlib
import re
from datetime import datetime, timezone
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
    # Coverage + Provenance helpers
    # ------------------------------------------------------------------

    @staticmethod
    def _compute_coverage(source_ids: list) -> str:
        """Compute coverage level from number of source evidence items.

        HIGH  = 3+ sources
        MEDIUM = 2 sources
        LOW   = 0-1 source
        """
        n = len(source_ids) if source_ids else 0
        if n >= 3:
            return "HIGH"
        if n == 2:
            return "MEDIUM"
        return "LOW"

    @staticmethod
    def _determine_provenance(has_contradictions: bool, source_count: int) -> str:
        """Determine the primary provenance type for a page.

        extracted  = directly from evidence text (single source)
        inferred   = LLM synthesis across multiple sources
        ambiguous  = sources disagree (contradiction detected)
        """
        if has_contradictions:
            return "ambiguous"
        if source_count >= 2:
            return "inferred"
        return "extracted"

    def _build_frontmatter(
        self,
        *,
        title: str,
        page_type: str,
        entity_type: str | None = None,
        source_ids: list[str] | None = None,
        has_contradictions: bool = False,
        tags: list[str] | None = None,
    ) -> str:
        """Build YAML frontmatter block for a wiki page."""
        source_ids = source_ids or []
        tags = tags or []
        coverage = self._compute_coverage(source_ids)
        provenance = self._determine_provenance(has_contradictions, len(source_ids))
        now = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S")

        lines = [
            "---",
            f"title: {title}",
            f"type: {page_type}",
        ]
        if entity_type:
            lines.append(f"entity_type: {entity_type}")
        lines.append(f"sources: [{', '.join(source_ids)}]")
        lines.append(f"coverage: {coverage}")
        lines.append(f"last_compiled: {now}")
        lines.append(f"provenance: {provenance}")
        if has_contradictions:
            lines.append("contradictions: true")
        if tags:
            lines.append(f"tags: [{', '.join(tags)}]")
        lines.append("---")
        return "\n".join(lines) + "\n\n"

    # ------------------------------------------------------------------
    # Contradiction helpers
    # ------------------------------------------------------------------

    async def _get_page_contradictions(
        self, case_id: str, source_ids: list[str]
    ) -> list[dict[str, Any]]:
        """Query contradictions table for any involving the given evidence IDs."""
        if not source_ids:
            return []
        all_contradictions = await self._db.list_contradictions_by_case(case_id)
        relevant = []
        source_set = set(source_ids)
        for c in all_contradictions:
            if c.get("evidence_1_id") in source_set or c.get("evidence_2_id") in source_set:
                relevant.append(c)
        return relevant

    def _format_contradictions_section(self, contradictions: list[dict[str, Any]]) -> str:
        """Format a ## Contradictions section from contradiction records."""
        if not contradictions:
            return ""
        lines = ["\n## Contradictions\n"]
        for c in contradictions:
            severity = c.get("severity", "medium")
            desc = c.get("description", "?")
            ev1 = c.get("evidence_1_title", c.get("evidence_1_id", "?"))
            ev2 = c.get("evidence_2_title", c.get("evidence_2_id", "?"))
            lines.append(
                f"- **[{severity}]** {desc}\n"
                f"  - Sources en conflit: {ev1} vs {ev2}\n"
                f"  - [ambiguous] Les sources divergent sur ce point"
            )
        return "\n".join(lines) + "\n"

    # ------------------------------------------------------------------
    # Cross-linking
    # ------------------------------------------------------------------

    async def cross_link_pages(self, case_id: str) -> int:
        """Cross-link all wiki pages: wrap unlinked entity mentions in [[wikilinks]].

        Returns the number of pages modified.
        """
        wiki_dir = self._wiki_dir(case_id)
        pages = await self._db.list_wiki_pages(case_id)

        # Build a map of entity names -> page paths (entities + locations)
        entity_names: dict[str, str] = {}
        for p in pages:
            if p["page_type"] in ("entity", "location"):
                entity_names[p["title"]] = p["page_path"]

        if not entity_names:
            return 0

        # Sort by length descending so longer names match first
        sorted_names = sorted(entity_names.keys(), key=len, reverse=True)

        modified_count = 0
        for p in pages:
            page_file = wiki_dir / p["page_path"]
            if not page_file.exists():
                continue
            content = page_file.read_text(encoding="utf-8")
            original = content

            # Split frontmatter from body to avoid linking inside YAML
            fm_end = 0
            if content.startswith("---"):
                second_fence = content.find("---", 3)
                if second_fence != -1:
                    fm_end = second_fence + 3

            frontmatter = content[:fm_end]
            body = content[fm_end:]

            for name in sorted_names:
                # Skip self-references (the page's own title)
                if name.lower() == p["title"].lower():
                    continue
                # Match the name only if not already inside [[ ]]
                # Pattern: name not preceded by [[ and not followed by ]]
                pattern = re.compile(
                    r'(?<!\[\[)' + re.escape(name) + r'(?!\]\])',
                    re.IGNORECASE,
                )
                body = pattern.sub(f"[[{name}]]", body, count=0)

            new_content = frontmatter + body
            if new_content != original:
                page_file.write_text(new_content, encoding="utf-8")
                modified_count += 1

        if modified_count:
            logger.info("Wiki cross-linked {} pages for case {}", modified_count, case_id)
        return modified_count

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

        # Check contradictions for this evidence
        contradictions = await self._get_page_contradictions(case_id, [evidence_id])
        has_contradictions = len(contradictions) > 0

        # Build tags from entity types
        ev_tags = list({e.get("entity_type", "other") for e in entities})
        ev_tags.append("preuve")

        # 1. Compile evidence page
        slug = self._slugify(ev.get("title", evidence_id[:8]))
        page_path = f"evidence/{slug}.md"
        existing = self._read_page(wiki_dir, page_path)

        # Strip existing frontmatter before passing to LLM
        existing_body = self._strip_frontmatter(existing)

        # Build frontmatter
        frontmatter = self._build_frontmatter(
            title=ev.get("title", "?"),
            page_type="evidence",
            source_ids=[evidence_id],
            has_contradictions=has_contradictions,
            tags=ev_tags,
        )

        from nexus.llm.prompts import WIKI_COMPILE_EVIDENCE_PROMPT
        prompt = WIKI_COMPILE_EVIDENCE_PROMPT.format(
            title=ev.get("title", "?"),
            source=ev.get("source", "inconnu"),
            source_date=ev.get("source_date", "inconnue"),
            reliability=ev.get("reliability", 50),
            content=(ev.get("summary") or ev.get("raw_text", ""))[:3000],
            entities=entities_text,
            existing_page=existing_body or "(nouvelle page)",
        )

        try:
            content = await self._router.route(TaskType.EVIDENCE_SUMMARY, prompt)
            # Strip any frontmatter the LLM might have generated
            content = self._strip_frontmatter(content)
            # Add contradiction section if relevant
            contradiction_section = self._format_contradictions_section(contradictions)
            # Assemble: frontmatter + LLM content + contradictions
            full_page = frontmatter + content + contradiction_section
            content_hash = self._write_page(wiki_dir, page_path, full_page)
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
                ent_existing_body = self._strip_frontmatter(ent_existing)

                # Gather all source_ids for this entity page
                ent_source_ids = await self._collect_entity_source_ids(case_id, ent["id"], ent_path)
                # Always include current evidence
                if evidence_id not in ent_source_ids:
                    ent_source_ids.append(evidence_id)

                ent_contradictions = await self._get_page_contradictions(case_id, ent_source_ids)
                ent_has_contradictions = len(ent_contradictions) > 0

                ent_tags = [ent_type]
                name_parts = ent["name"].lower().split()
                ent_tags.extend(name_parts[:3])

                ent_frontmatter = self._build_frontmatter(
                    title=ent["name"],
                    page_type="entity" if ent_type != "location" else "location",
                    entity_type=ent_type,
                    source_ids=ent_source_ids,
                    has_contradictions=ent_has_contradictions,
                    tags=ent_tags,
                )

                from nexus.llm.prompts import WIKI_UPDATE_ENTITY_PROMPT
                ent_prompt = WIKI_UPDATE_ENTITY_PROMPT.format(
                    entity_name=ent["name"],
                    entity_type=ent_type,
                    description=ent.get("description", ""),
                    new_info=f"Mentionne dans: {ev.get('title', '?')}\nContexte: {(ev.get('summary') or '')[:500]}",
                    existing_page=ent_existing_body or "(nouvelle page)",
                )

                ent_content = await self._router.route(TaskType.EVIDENCE_SUMMARY, ent_prompt)
                ent_content = self._strip_frontmatter(ent_content)
                ent_contradiction_section = self._format_contradictions_section(ent_contradictions)
                ent_full_page = ent_frontmatter + ent_content + ent_contradiction_section

                ent_hash = self._write_page(wiki_dir, ent_path, ent_full_page)
                await self._db.upsert_wiki_page(
                    case_id=case_id, page_path=ent_path,
                    page_type="location" if ent_type == "location" else "entity",
                    title=ent["name"], content_hash=ent_hash,
                    source_ids=ent_source_ids,
                )
                updated_pages.append(ent_path)
            except Exception as exc:
                logger.debug("Wiki entity page failed for {}: {}", ent["name"], exc)

        # 3. Cross-link all pages
        try:
            await self.cross_link_pages(case_id)
        except Exception as exc:
            logger.debug("WikiCompiler: cross-linking failed: %s", exc)

        # 4. Update index
        await self.rebuild_index(case_id)

        # 5. Update log
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
        existing_body = self._strip_frontmatter(existing)

        # Collect all evidence IDs referenced by hypotheses
        hyp_source_ids: list[str] = []
        for h in hypotheses:
            sup = h.get("supporting_evidence")
            if isinstance(sup, list):
                hyp_source_ids.extend(sup)
        hyp_source_ids = list(set(hyp_source_ids))

        hyp_contradictions = await self._get_page_contradictions(case_id, hyp_source_ids)
        has_contradictions = len(contradictions) > 0  # any contradictions in the case

        # Build frontmatter for analysis page
        analysis_tags = ["hypotheses", "analyse"]
        frontmatter = self._build_frontmatter(
            title="Hypotheses actives",
            page_type="analysis",
            source_ids=hyp_source_ids,
            has_contradictions=has_contradictions,
            tags=analysis_tags,
        )

        from nexus.llm.prompts import WIKI_COMPILE_HYPOTHESES_PROMPT
        prompt = WIKI_COMPILE_HYPOTHESES_PROMPT.format(
            hypotheses=hyp_text,
            contradictions=contra_text,
            suspects=sus_text,
            existing_page=existing_body or "(nouvelle page)",
        )

        try:
            content = await self._router.route(TaskType.EVIDENCE_SUMMARY, prompt)
            content = self._strip_frontmatter(content)
            full_page = frontmatter + content
            content_hash = self._write_page(wiki_dir, page_path, full_page)
            await self._db.upsert_wiki_page(
                case_id=case_id, page_path=page_path, page_type="analysis",
                title="Hypotheses actives", content_hash=content_hash,
                source_ids=hyp_source_ids,
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
        """Regenerate index.md from all wiki pages with coverage stats."""
        wiki_dir = self._wiki_dir(case_id)
        pages = await self._db.list_wiki_pages(case_id)
        case = await self._db.get_case(case_id)
        contradictions = await self._db.list_contradictions_by_case(case_id)

        evidence_links = []
        entity_links = []
        location_links = []
        analysis_links = []

        # Coverage counters
        coverage_counts = {"HIGH": 0, "MEDIUM": 0, "LOW": 0}
        provenance_counts = {"extracted": 0, "inferred": 0, "ambiguous": 0}

        for p in pages:
            source_ids = p.get("source_ids") or []
            coverage = self._compute_coverage(source_ids)
            coverage_counts[coverage] = coverage_counts.get(coverage, 0) + 1

            # Check if page has contradictions
            page_contras = await self._get_page_contradictions(case_id, source_ids)
            if page_contras:
                provenance_counts["ambiguous"] += 1
            elif len(source_ids) >= 2:
                provenance_counts["inferred"] += 1
            else:
                provenance_counts["extracted"] += 1

            coverage_badge = f" `{coverage}`"
            link = f"- [[{p['page_path']}|{p['title']}]]{coverage_badge}"
            if p["page_type"] == "evidence":
                evidence_links.append(link)
            elif p["page_type"] == "entity":
                entity_links.append(link)
            elif p["page_type"] == "location":
                location_links.append(link)
            elif p["page_type"] == "analysis":
                analysis_links.append(link)

        from nexus.llm.prompts import WIKI_INDEX_TEMPLATE
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
            coverage_high=coverage_counts["HIGH"],
            coverage_medium=coverage_counts["MEDIUM"],
            coverage_low=coverage_counts["LOW"],
            contradictions_count=len(contradictions),
            provenance_extracted=provenance_counts["extracted"],
            provenance_inferred=provenance_counts["inferred"],
            provenance_ambiguous=provenance_counts["ambiguous"],
        )

        self._write_page(wiki_dir, "index.md", content)

    # ------------------------------------------------------------------
    # update_log
    # ------------------------------------------------------------------

    async def update_log(self, case_id: str, action: str, details: str) -> None:
        """Append entry to log.md."""
        wiki_dir = self._wiki_dir(case_id)
        log_path = wiki_dir / "log.md"
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        entry = f"- **{timestamp}** | `{action}` | {details}\n"

        if log_path.exists():
            existing = log_path.read_text(encoding="utf-8")
        else:
            existing = "# Journal de compilation\n\n"

        log_path.write_text(existing + entry, encoding="utf-8")

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    @staticmethod
    def _strip_frontmatter(text: str) -> str:
        """Remove YAML frontmatter (---...---) from the beginning of text."""
        if not text or not text.startswith("---"):
            return text
        second_fence = text.find("---", 3)
        if second_fence == -1:
            return text
        return text[second_fence + 3:].lstrip("\n")

    async def _collect_entity_source_ids(
        self, case_id: str, entity_id: str, page_path: str
    ) -> list[str]:
        """Collect all evidence IDs that mention this entity, plus any already stored."""
        source_ids: list[str] = []
        # From existing wiki page record
        existing_page = await self._db.get_wiki_page(case_id, page_path)
        if existing_page and existing_page.get("source_ids"):
            source_ids.extend(existing_page["source_ids"])
        # From mentions
        mentions = await self._db.list_mentions_by_entity(entity_id)
        for m in mentions:
            eid = m.get("evidence_id")
            if eid and eid not in source_ids:
                source_ids.append(eid)
        return source_ids
