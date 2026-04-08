"""
NEXUS -- Immutable, Decentralized Audit Trail.

Every investigation action is logged in THREE layers:
1. SQLite (fast query, local) — with hash chain for tamper detection
2. Append-only log files (one per case, plain text, human-readable)
3. Git commits (cryptographic hash, distributable, non-deletable)

Each entry contains a SHA-256 hash of (previous_hash + entry_data),
forming a hash chain like a blockchain. If any entry is modified or
deleted, the chain breaks and tampering is detected.

The audit is NON-BLOCKING: failures are caught and logged but never
disrupt the main operation.
"""

from __future__ import annotations

import asyncio
import hashlib
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Optional

from loguru import logger

from nexus.config import settings
from nexus.db.sqlite_db import Database


# Directory for append-only log files
AUDIT_LOG_DIR = Path(settings.data_dir) / "audit"


class AuditService:
    """Immutable, decentralized audit logging for investigations.

    Three persistence layers:
    - SQLite: fast queries, tamper-detectable via hash chain
    - Append-only files: data/audit/{case_id}.jsonl (one JSON per line)
    - Git: each entry is committed to data/audit/ git repo
    """

    def __init__(self, db: Database) -> None:
        self._db = db
        self._last_hash: dict[str, str] = {}  # case_id → last hash
        self._pending_tasks: set[asyncio.Task] = set()
        AUDIT_LOG_DIR.mkdir(parents=True, exist_ok=True)
        self._init_git_repo()

    def _init_git_repo(self) -> None:
        """Initialize a git repo in the audit directory if not exists."""
        git_dir = AUDIT_LOG_DIR / ".git"
        if not git_dir.exists():
            try:
                subprocess.run(
                    ["git", "init"],
                    cwd=str(AUDIT_LOG_DIR),
                    capture_output=True,
                    timeout=10,
                )
                # Set identity for commits
                subprocess.run(
                    ["git", "config", "user.name", "NEXUS-Audit"],
                    cwd=str(AUDIT_LOG_DIR), capture_output=True, timeout=5,
                )
                subprocess.run(
                    ["git", "config", "user.email", "audit@nexus.local"],
                    cwd=str(AUDIT_LOG_DIR), capture_output=True, timeout=5,
                )
                logger.info("Audit git repo initialized at {}", AUDIT_LOG_DIR)
            except Exception as exc:
                logger.warning("Could not init audit git repo: {}", exc)

    def _safe_create_task(self, coro, name: str) -> asyncio.Task:
        """Create an asyncio task with error logging and lifecycle tracking.

        - Names the task for debuggability
        - Logs exceptions via a done_callback (prevents silent failures)
        - Tracks the task in _pending_tasks so flush() can await them
        """
        task = asyncio.create_task(coro, name=name)
        self._pending_tasks.add(task)

        def _on_done(t: asyncio.Task) -> None:
            self._pending_tasks.discard(t)
            if t.cancelled():
                return
            exc = t.exception()
            if exc is not None:
                logger.error(
                    "Audit task '{}' failed: {}: {}",
                    t.get_name(),
                    type(exc).__name__,
                    exc,
                )

        task.add_done_callback(_on_done)
        return task

    async def flush(self) -> None:
        """Await all pending audit tasks. Useful for graceful shutdown."""
        if not self._pending_tasks:
            return
        tasks = list(self._pending_tasks)
        logger.debug("Flushing {} pending audit tasks", len(tasks))
        results = await asyncio.gather(*tasks, return_exceptions=True)
        for i, result in enumerate(results):
            if isinstance(result, Exception):
                logger.warning(
                    "Audit flush: task '{}' had error: {}",
                    tasks[i].get_name(),
                    result,
                )

    def _compute_hash(self, previous_hash: str, entry_data: str) -> str:
        """Compute SHA-256 hash linking to the previous entry."""
        payload = f"{previous_hash}|{entry_data}"
        return hashlib.sha256(payload.encode("utf-8")).hexdigest()

    def _get_last_hash(self, case_id: str) -> str:
        """Get the hash of the last audit entry for a case."""
        if case_id in self._last_hash:
            return self._last_hash[case_id]
        return "GENESIS"  # First entry in chain

    async def log(
        self,
        case_id: str,
        actor: str,
        action: str,
        summary: str,
        target_type: Optional[str] = None,
        target_id: Optional[str] = None,
        details: Optional[dict] = None,
        cycle_number: Optional[int] = None,
    ) -> Optional[dict]:
        """Log an action to all three audit layers.

        Fire-and-forget safe: catches all exceptions.
        """
        try:
            timestamp = datetime.now(timezone.utc).isoformat()
            details_json = json.dumps(details, default=str) if details else None

            # Build entry data for hashing
            entry_data = f"{timestamp}|{case_id}|{actor}|{action}|{summary}|{target_type}|{target_id}|{details_json}"

            # Compute hash chain
            prev_hash = self._get_last_hash(case_id)
            entry_hash = self._compute_hash(prev_hash, entry_data)

            # Layer 1: SQLite (with hash)
            row = await self._db.create_audit_entry(
                case_id=case_id,
                actor=actor,
                action=action,
                target_type=target_type,
                target_id=target_id,
                summary=summary,
                details=details_json,
                cycle_number=cycle_number,
                entry_hash=entry_hash,
                previous_hash=prev_hash,
            )

            # Update last hash cache
            self._last_hash[case_id] = entry_hash

            # Layer 2: Append-only JSONL file (background, non-blocking)
            entry_id = row["id"] if row else "unknown"
            self._safe_create_task(
                self._write_jsonl(case_id, {
                    "id": entry_id,
                    "timestamp": timestamp,
                    "case_id": case_id,
                    "actor": actor,
                    "action": action,
                    "target_type": target_type,
                    "target_id": target_id,
                    "summary": summary,
                    "details": details,
                    "cycle_number": cycle_number,
                    "hash": entry_hash,
                    "previous_hash": prev_hash,
                }),
                name=f"audit-jsonl-{case_id}-{entry_id}",
            )

            # Layer 3: Git commit (background, non-blocking)
            self._safe_create_task(
                self._git_commit(case_id, action, summary, entry_hash),
                name=f"audit-git-{case_id}-{entry_hash[:12]}",
            )

            return row

        except Exception as exc:
            logger.warning("Audit log failed (non-blocking): {}", exc)
            return None

    async def _write_jsonl(self, case_id: str, entry: dict) -> None:
        """Append one JSON line to the case's audit file."""
        try:
            log_file = AUDIT_LOG_DIR / f"{case_id}.jsonl"
            line = json.dumps(entry, ensure_ascii=False, default=str) + "\n"

            def _append() -> None:
                with log_file.open("a", encoding="utf-8") as fh:
                    fh.write(line)

            loop = asyncio.get_event_loop()
            await loop.run_in_executor(None, _append)
        except Exception as exc:
            logger.warning("Audit JSONL write failed: {}", exc)

    async def _git_commit(
        self, case_id: str, action: str, summary: str, entry_hash: str,
    ) -> None:
        """Git add + commit the audit file for this case."""
        try:
            jsonl_file = f"{case_id}.jsonl"
            proc = await asyncio.create_subprocess_exec(
                "git", "add", jsonl_file,
                cwd=str(AUDIT_LOG_DIR),
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            await asyncio.wait_for(proc.communicate(), timeout=10)

            commit_msg = f"[{action}] {summary[:100]} (hash:{entry_hash[:12]})"
            proc = await asyncio.create_subprocess_exec(
                "git", "commit", "-m", commit_msg, "--allow-empty",
                cwd=str(AUDIT_LOG_DIR),
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            await asyncio.wait_for(proc.communicate(), timeout=10)

        except Exception as exc:
            logger.debug("Audit git commit skipped: {}", exc)

    async def verify_chain(self, case_id: str) -> dict:
        """Verify the hash chain integrity for a case.

        Returns {valid: bool, entries_checked: int, broken_at: int | None}
        """
        entries = await self._db.get_investigation_timeline(case_id)
        if not entries:
            return {"valid": True, "entries_checked": 0, "broken_at": None}

        prev_hash = "GENESIS"
        for i, entry in enumerate(entries):
            stored_hash = entry.get("entry_hash", "")
            stored_prev = entry.get("previous_hash", "")

            if stored_prev != prev_hash:
                return {"valid": False, "entries_checked": i, "broken_at": i}

            # Recompute hash
            entry_data = (
                f"{entry.get('timestamp', '')}|{entry.get('case_id', '')}|"
                f"{entry.get('actor', '')}|{entry.get('action', '')}|"
                f"{entry.get('summary', '')}|{entry.get('target_type', '')}|"
                f"{entry.get('target_id', '')}|{entry.get('details', '')}"
            )
            expected = self._compute_hash(prev_hash, entry_data)

            if stored_hash != expected:
                return {"valid": False, "entries_checked": i, "broken_at": i}

            prev_hash = stored_hash

        return {"valid": True, "entries_checked": len(entries), "broken_at": None}

    # ================================================================
    # Convenience methods for common actions
    # ================================================================

    async def log_evidence_added(
        self, case_id: str, evidence_id: str, title: str,
        source: Optional[str] = None, actor: str = "user",
    ) -> None:
        await self.log(
            case_id, actor, "evidence_added",
            f"Preuve ajoutee: {title}",
            "evidence", evidence_id,
            {"title": title, "source": source},
        )

    async def log_entity_discovered(
        self, case_id: str, entity_id: str, name: str,
        entity_type: str, actor: str = "system",
    ) -> None:
        await self.log(
            case_id, actor, "entity_discovered",
            f"Entite decouverte: {name} ({entity_type})",
            "entity", entity_id,
            {"name": name, "type": entity_type},
        )

    async def log_hypothesis_scored(
        self, case_id: str, hyp_id: str, title: str,
        old_score: float, new_score: float, actor: str = "system",
    ) -> None:
        delta = new_score - old_score
        direction = "hausse" if delta > 0 else "baisse"
        await self.log(
            case_id, actor, "hypothesis_scored",
            f"Hypothese '{title[:80]}': {old_score:.0f}% -> {new_score:.0f}% ({direction})",
            "hypothesis", hyp_id,
            {"old_score": old_score, "new_score": new_score, "delta": delta},
        )

    async def log_hypothesis_created(
        self, case_id: str, hyp_id: str, title: str,
        score: float, actor: str = "system",
    ) -> None:
        await self.log(
            case_id, actor, "hypothesis_created",
            f"Nouvelle hypothese: {title[:80]} (score initial: {score:.0f}%)",
            "hypothesis", hyp_id,
            {"title": title, "initial_score": score},
        )

    async def log_contradiction_found(
        self, case_id: str, description: str,
        evidence_ids: Optional[list] = None, actor: str = "system",
    ) -> None:
        await self.log(
            case_id, actor, "contradiction_found",
            f"Contradiction detectee: {description[:200]}",
            details={"description": description, "evidence_ids": evidence_ids},
        )

    async def log_monitoring_result(
        self, case_id: str, result_id: str, title: str,
        relevance: float, actor: str = "monitoring",
    ) -> None:
        await self.log(
            case_id, actor, "monitoring_result",
            f"Resultat monitoring: {title[:100]} (pertinence: {relevance:.0f}%)",
            "monitoring_result", result_id,
            {"title": title, "relevance": relevance},
        )

    async def log_query_generated(
        self, case_id: str, query: str,
        actor: str = "autonomous_loop", cycle: Optional[int] = None,
    ) -> None:
        await self.log(
            case_id, actor, "query_generated",
            f"Nouvelle requete generee: {query[:100]}",
            details={"query": query}, cycle_number=cycle,
        )

    async def log_auto_ingest(
        self, case_id: str, result_id: str, evidence_id: str,
        title: str, actor: str = "autonomous_loop",
        cycle: Optional[int] = None,
    ) -> None:
        await self.log(
            case_id, actor, "evidence_ingested_auto",
            f"Auto-ingestion: {title[:100]}",
            "evidence", evidence_id,
            {"monitoring_result_id": result_id}, cycle_number=cycle,
        )

    async def log_self_questioning(
        self, case_id: str, top_hypothesis: str, summary: str,
        actor: str = "autonomous_loop", cycle: Optional[int] = None,
    ) -> None:
        await self.log(
            case_id, actor, "self_questioning",
            f"Auto-questionnement -- hypothese principale: {top_hypothesis[:80]}",
            details={"summary": summary[:settings.text_truncation_short]}, cycle_number=cycle,
        )

    async def log_analysis(
        self, case_id: str, run_id: str, run_type: str,
        status: str, actor: str = "system",
    ) -> None:
        await self.log(
            case_id, actor, f"analysis_{status}",
            f"Analyse {run_type} {status}",
            "analysis_run", run_id,
            {"run_type": run_type, "status": status},
        )

    async def log_investigation_event(
        self, case_id: str, event: str, actor: str = "system",
    ) -> None:
        await self.log(
            case_id, actor, f"investigation_{event}",
            f"Investigation {event}",
        )
