"""
NEXUS -- Domain reconnaissance (WHOIS + DNS).

Provides WHOIS lookups via the ``python-whois`` library and DNS
resolution via ``socket`` + ``nslookup`` subprocess.
"""

from __future__ import annotations

import asyncio
import re
import socket
from typing import Any, Dict, List, Optional

from loguru import logger


class DomainRecon:
    """WHOIS and DNS lookup tools for domain investigation.

    Usage::

        recon = DomainRecon()
        whois_info = await recon.whois_lookup("example.com")
        dns_info   = await recon.dns_lookup("example.com")
    """

    # ------------------------------------------------------------------
    # WHOIS
    # ------------------------------------------------------------------

    async def whois_lookup(self, domain: str) -> Dict[str, Any]:
        """Perform a WHOIS lookup on *domain*.

        Uses ``python-whois`` (imported as ``whois``) in a thread pool
        because it is a blocking library.

        Returns:
            Dict with keys: registrar, creation_date, expiration_date,
            name_servers, registrant_name, registrant_email, raw.
        """
        domain = domain.strip().lower()
        logger.info("DomainRecon: WHOIS lookup for {}", domain)

        loop = asyncio.get_running_loop()
        try:
            result = await loop.run_in_executor(None, self._whois_sync, domain)
            return result
        except Exception:
            logger.exception("DomainRecon: WHOIS lookup failed for {}", domain)
            return {
                "domain": domain,
                "error": "WHOIS lookup failed",
                "registrar": None,
                "creation_date": None,
                "expiration_date": None,
                "name_servers": [],
                "registrant_name": None,
                "registrant_email": None,
            }

    def _whois_sync(self, domain: str) -> Dict[str, Any]:
        """Blocking WHOIS query via python-whois."""
        import whois  # python-whois

        w = whois.whois(domain)

        # python-whois returns dates as datetime or list of datetimes
        creation = w.creation_date
        if isinstance(creation, list):
            creation = creation[0] if creation else None

        expiration = w.expiration_date
        if isinstance(expiration, list):
            expiration = expiration[0] if expiration else None

        name_servers = w.name_servers or []
        if isinstance(name_servers, str):
            name_servers = [name_servers]
        # Normalise to lowercase strings
        name_servers = [ns.lower() for ns in name_servers if ns]

        return {
            "domain": domain,
            "registrar": w.registrar,
            "creation_date": str(creation) if creation else None,
            "expiration_date": str(expiration) if expiration else None,
            "name_servers": sorted(set(name_servers)),
            "registrant_name": getattr(w, "name", None),
            "registrant_email": getattr(w, "emails", None),
            "raw": str(w.text)[:2000] if hasattr(w, "text") and w.text else None,
        }

    # ------------------------------------------------------------------
    # DNS
    # ------------------------------------------------------------------

    async def dns_lookup(self, domain: str) -> Dict[str, Any]:
        """Resolve DNS records (A, MX, NS) for *domain*.

        Uses ``socket.getaddrinfo`` for A records (in a thread pool) and
        ``nslookup`` subprocess for MX and NS records.

        Returns:
            Dict with keys: domain, a_records, mx_records, ns_records.
        """
        domain = domain.strip().lower()
        logger.info("DomainRecon: DNS lookup for {}", domain)

        # Run A-record lookup and nslookup concurrently
        a_task = asyncio.ensure_future(self._resolve_a_records(domain))
        mx_task = asyncio.ensure_future(self._nslookup(domain, "MX"))
        ns_task = asyncio.ensure_future(self._nslookup(domain, "NS"))

        a_records, mx_records, ns_records = await asyncio.gather(
            a_task, mx_task, ns_task
        )

        return {
            "domain": domain,
            "a_records": a_records,
            "mx_records": mx_records,
            "ns_records": ns_records,
        }

    async def _resolve_a_records(self, domain: str) -> List[str]:
        """Resolve A records using socket.getaddrinfo in a thread pool."""
        loop = asyncio.get_running_loop()
        try:
            infos = await loop.run_in_executor(
                None,
                lambda: socket.getaddrinfo(domain, None, socket.AF_INET),
            )
            # Extract unique IPs
            ips = sorted({info[4][0] for info in infos})
            return ips
        except socket.gaierror:
            logger.debug("DomainRecon: no A records for {}", domain)
            return []
        except Exception:
            logger.debug("DomainRecon: A record lookup failed for {}", domain)
            return []

    async def _nslookup(
        self, domain: str, record_type: str
    ) -> List[str]:
        """Run ``nslookup -type=<record_type> <domain>`` and parse output."""
        try:
            proc = await asyncio.create_subprocess_exec(
                "nslookup", f"-type={record_type}", domain,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            stdout, _ = await asyncio.wait_for(
                proc.communicate(), timeout=15.0
            )
        except FileNotFoundError:
            logger.debug("DomainRecon: nslookup not found on this system")
            return []
        except asyncio.TimeoutError:
            logger.debug(
                "DomainRecon: nslookup timed out for {} {}", record_type, domain
            )
            return []
        except Exception:
            logger.debug(
                "DomainRecon: nslookup error for {} {}", record_type, domain
            )
            return []

        raw = stdout.decode(errors="replace")
        return self._parse_nslookup(raw, record_type)

    def _parse_nslookup(self, raw: str, record_type: str) -> List[str]:
        """Extract records from nslookup output."""
        records: List[str] = []

        if record_type == "MX":
            # Lines like: "mail exchanger = 10 mx.example.com"
            for match in re.finditer(
                r"mail exchanger\s*=\s*\d+\s+(\S+)", raw, re.IGNORECASE
            ):
                val = match.group(1).rstrip(".")
                if val:
                    records.append(val.lower())

        elif record_type == "NS":
            # Lines like: "nameserver = ns1.example.com"
            for match in re.finditer(
                r"nameserver\s*=\s*(\S+)", raw, re.IGNORECASE
            ):
                val = match.group(1).rstrip(".")
                if val:
                    records.append(val.lower())

        return sorted(set(records))
