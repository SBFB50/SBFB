"""
NEXUS -- Robin dark web / Tor search monitor.

Robin (apurvsg/robin) runs as a Docker container with Tor built-in.
It does NOT expose a REST API -- it works via CLI only.
We invoke it via ``docker exec`` subprocess calls.

Usage::

    robin = RobinMonitor()
    if await robin.is_available():
        results = await robin.search("keywords")
"""

from __future__ import annotations

import asyncio
import json
import re
from pathlib import Path
from typing import Any

from loguru import logger


ROBIN_CONTAINER = "nexus-robin"


class RobinMonitor:
    """Dark web search via Robin CLI inside Docker container.

    Robin searches 15+ .onion search engines through Tor and returns
    results as markdown investigation reports.  We parse those reports
    to extract structured results.
    """

    def __init__(self, container: str = ROBIN_CONTAINER) -> None:
        self._container = container

    # ------------------------------------------------------------------
    # Availability
    # ------------------------------------------------------------------

    async def is_available(self) -> bool:
        """Check whether the Robin container is running."""
        try:
            proc = await asyncio.create_subprocess_exec(
                "docker", "inspect", "-f", "{{.State.Running}}", self._container,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            stdout, _ = await asyncio.wait_for(proc.communicate(), timeout=10)
            return stdout.decode().strip() == "true"
        except Exception:
            logger.debug("Robin container '{}' not available", self._container)
            return False

    # ------------------------------------------------------------------
    # Search
    # ------------------------------------------------------------------

    async def search(
        self,
        query: str,
        *,
        max_results: int = 10,
        model: str = "nexus",
    ) -> list[dict[str, Any]]:
        """Search the dark web through Robin CLI.

        Runs ``docker exec nexus-robin robin cli --query "..." --model ...``
        and parses the markdown output for links/titles.

        Args:
            query: The search string.
            max_results: Cap on returned results.
            model: Ollama model Robin should use for query refinement.

        Returns:
            List of dicts with keys: url, title, snippet, source.
        """
        cmd = [
            "docker", "exec", self._container,
            "robin", "cli",
            "--query", query,
            "--model", model,
            "--report-format", "markdown",
        ]

        try:
            proc = await asyncio.create_subprocess_exec(
                *cmd,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            # Tor is slow -- generous 5 min timeout
            stdout, stderr = await asyncio.wait_for(
                proc.communicate(), timeout=300,
            )
        except asyncio.TimeoutError:
            logger.error("Robin timeout searching '{}' (Tor slow)", query)
            return []
        except FileNotFoundError:
            logger.error("docker command not found")
            return []
        except Exception:
            logger.exception("Robin unexpected error for query '{}'", query)
            return []

        if proc.returncode != 0:
            err = stderr.decode(errors="replace")[:500]
            logger.error("Robin CLI error (rc={}): {}", proc.returncode, err)
            return []

        output = stdout.decode(errors="replace")
        return self._parse_markdown_results(output, max_results)

    # ------------------------------------------------------------------
    # Page fetching (via Tor inside Robin container)
    # ------------------------------------------------------------------

    async def fetch_page(self, url: str) -> str:
        """Fetch a .onion page through Robin's built-in Tor.

        Uses ``docker exec`` + curl via the SOCKS5 proxy that Robin
        maintains internally (127.0.0.1:9050 inside the container).
        """
        cmd = [
            "docker", "exec", self._container,
            "curl", "-sS", "--max-time", "60",
            "--socks5-hostname", "127.0.0.1:9050",
            url,
        ]
        try:
            proc = await asyncio.create_subprocess_exec(
                *cmd,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            stdout, _ = await asyncio.wait_for(proc.communicate(), timeout=120)
            return stdout.decode(errors="replace")
        except asyncio.TimeoutError:
            logger.error("Robin fetch timeout for '{}'", url[:100])
            return ""
        except Exception:
            logger.exception("Robin fetch failed for '{}'", url[:100])
            return ""

    # ------------------------------------------------------------------
    # Parsing
    # ------------------------------------------------------------------

    @staticmethod
    def _parse_markdown_results(
        markdown: str, max_results: int
    ) -> list[dict[str, Any]]:
        """Extract structured results from Robin's markdown report.

        Robin outputs investigation reports with sections containing
        links in markdown format: ``[title](url)`` and surrounding text.
        """
        results: list[dict[str, Any]] = []
        # Match markdown links: [title](url)
        link_pattern = re.compile(r"\[([^\]]+)\]\((https?://[^\)]+)\)")

        lines = markdown.split("\n")
        for i, line in enumerate(lines):
            for match in link_pattern.finditer(line):
                title = match.group(1).strip()
                url = match.group(2).strip()

                # Grab surrounding text as snippet
                snippet_parts = []
                # Line after the link
                if i + 1 < len(lines) and lines[i + 1].strip():
                    snippet_parts.append(lines[i + 1].strip()[:200])
                # Or the line itself minus the link
                remainder = link_pattern.sub("", line).strip(" -•*")
                if remainder:
                    snippet_parts.insert(0, remainder[:200])

                results.append({
                    "url": url,
                    "title": title,
                    "snippet": " ".join(snippet_parts)[:300] if snippet_parts else "",
                    "source": "robin/tor",
                })

                if len(results) >= max_results:
                    break
            if len(results) >= max_results:
                break

        logger.debug(
            "Robin parsed {} results from markdown ({} chars)",
            len(results), len(markdown),
        )
        return results
