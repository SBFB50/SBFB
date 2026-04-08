"""
NEXUS -- Holehe email reconnaissance.

Checks whether an email address is registered on 120+ online services
using holehe (passive, no login required).  Runs holehe as a subprocess
to avoid event-loop conflicts (holehe uses httpx/trio internally).
"""

from __future__ import annotations

import asyncio
import csv
import io
import re
from typing import Any, Dict, List

from loguru import logger


class HoleheRecon:
    """Passive email-existence checker powered by holehe.

    Usage::

        recon = HoleheRecon()
        hits = await recon.check_email("target@example.com")
        # [{"site": "twitter.com", "domain": "twitter.com", "exists": True}, ...]
    """

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    async def check_email(self, email: str) -> List[Dict[str, Any]]:
        """Run holehe against *email* and return sites where it is registered.

        Returns:
            List of dicts with keys: site, domain, exists (bool).
            Only sites where the email **exists** are returned.
        """
        email = email.strip().lower()
        if not re.match(r"^[^@\s]+@[^@\s]+\.[^@\s]+$", email):
            logger.warning("HoleheRecon: invalid email format: {}", email)
            return []

        logger.info("HoleheRecon: checking email {}", email)

        try:
            proc = await asyncio.create_subprocess_exec(
                "holehe", email, "--only-used", "--no-color", "-C",
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            stdout, stderr = await asyncio.wait_for(
                proc.communicate(), timeout=120.0
            )
        except FileNotFoundError:
            logger.error(
                "HoleheRecon: holehe not found -- install with: pip install holehe"
            )
            return []
        except asyncio.TimeoutError:
            logger.error("HoleheRecon: holehe timed out after 120s for {}", email)
            return []
        except Exception as exc:
            logger.error("HoleheRecon: unexpected error running holehe: {}", exc)
            return []

        if proc.returncode != 0:
            err_text = stderr.decode(errors="replace").strip()
            logger.warning(
                "HoleheRecon: holehe exited with code {} -- {}",
                proc.returncode, err_text[:200],
            )

        raw_output = stdout.decode(errors="replace")
        return self._parse_csv_output(raw_output, email)

    # ------------------------------------------------------------------
    # Parsing
    # ------------------------------------------------------------------

    def _parse_csv_output(
        self, raw: str, email: str
    ) -> List[Dict[str, Any]]:
        """Parse holehe CSV output (from -C flag).

        CSV columns: name,domain,exists,http_status,rate_limit
        """
        results: List[Dict[str, Any]] = []

        reader = csv.DictReader(io.StringIO(raw))
        for row in reader:
            exists_val = row.get("exists", "").strip().lower()
            if exists_val in ("true", "1", "yes"):
                results.append({
                    "site": row.get("name", "").strip(),
                    "domain": row.get("domain", "").strip(),
                    "exists": True,
                })

        # Fallback: if CSV parsing yielded nothing, try line-based parsing
        # holehe also prints "[+] site.com" for found accounts
        if not results:
            results = self._parse_text_output(raw)

        logger.info(
            "HoleheRecon: {} found {} sites for {}",
            "holehe", len(results), email,
        )
        return results

    def _parse_text_output(self, raw: str) -> List[Dict[str, Any]]:
        """Fallback parser for holehe plain-text output lines like [+] site.com."""
        results: List[Dict[str, Any]] = []
        for line in raw.splitlines():
            line = line.strip()
            if line.startswith("[+]"):
                # e.g. "[+] twitter.com"
                site = line[3:].strip().rstrip(":")
                if site:
                    results.append({
                        "site": site,
                        "domain": site,
                        "exists": True,
                    })
        return results
