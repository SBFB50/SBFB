"""
NEXUS -- OSINT Reconnaissance API router.

Endpoints for passive recon: email checks (holehe + social),
username lookups, domain WHOIS/DNS, and automated entity scanning.
"""

from __future__ import annotations

from typing import Any, Dict, List

from fastapi import APIRouter, Depends, HTTPException, Query
from loguru import logger

from nexus.api.deps import get_database, paginated_response
from nexus.db.sqlite_db import Database
from nexus.recon.domain_recon import DomainRecon
from nexus.recon.holehe_recon import HoleheRecon
from nexus.recon.social_recon import SocialRecon

router = APIRouter(prefix="/api", tags=["recon"])

# Shared recon tool instances (stateless, safe to reuse)
_holehe = HoleheRecon()
_social = SocialRecon()
_domain = DomainRecon()


# ====================================================================
# Email recon
# ====================================================================

@router.post("/recon/email/{email}")
async def recon_email(email: str) -> Dict[str, Any]:
    """Check an email against holehe (120+ sites) and social platforms.

    Returns combined results from both tools.
    """
    logger.info("Recon: email scan for {}", email)

    holehe_results = await _holehe.check_email(email)
    social_results = await _social.search_email_username(email)

    return {
        "email": email,
        "holehe": holehe_results,
        "social": social_results,
        "holehe_count": len(holehe_results),
        "social_found": sum(1 for r in social_results if r.get("exists")),
    }


# ====================================================================
# Username recon
# ====================================================================

@router.post("/recon/username/{username}")
async def recon_username(username: str) -> Dict[str, Any]:
    """Search for a username across major social platforms."""
    logger.info("Recon: username scan for {}", username)

    results = await _social.search_username(username)

    return {
        "username": username,
        "results": results,
        "found_count": sum(1 for r in results if r.get("exists")),
    }


# ====================================================================
# Domain recon
# ====================================================================

@router.post("/recon/domain/{domain}")
async def recon_domain(domain: str) -> Dict[str, Any]:
    """WHOIS + DNS lookup for a domain."""
    logger.info("Recon: domain scan for {}", domain)

    whois_data = await _domain.whois_lookup(domain)
    dns_data = await _domain.dns_lookup(domain)

    return {
        "domain": domain,
        "whois": whois_data,
        "dns": dns_data,
    }


# ====================================================================
# Case-scoped recon results
# ====================================================================

@router.get("/cases/{case_id}/recon")
async def get_case_recon(
    case_id: str,
    limit: int = Query(100, ge=1, le=1000),
    offset: int = Query(0, ge=0),
    db: Database = Depends(get_database),
):
    """Return entities with recon metadata for a given case, with pagination.

    Filters entities of type ``email`` or ``account`` that have a
    non-null ``metadata`` field containing recon results.
    """
    case = await db.get_case(case_id)
    if not case:
        raise HTTPException(status_code=404, detail="Dossier introuvable")

    entities = await db.list_entities_by_case(case_id, limit=100_000)

    recon_entities = []
    for entity in entities:
        if entity.get("entity_type") in ("email", "account"):
            meta = entity.get("metadata")
            if meta and isinstance(meta, dict) and meta.get("recon"):
                recon_entities.append(entity)

    return paginated_response(recon_entities, offset, limit)


# ====================================================================
# Auto-scan all entities in a case
# ====================================================================

@router.post("/cases/{case_id}/recon/auto")
async def auto_recon(
    case_id: str,
    db: Database = Depends(get_database),
) -> Dict[str, Any]:
    """Run recon automatically on all email/account entities in a case.

    For each entity:
    - email entities  -> holehe + social search
    - account entities -> social username search

    Results are stored in the entity's ``metadata.recon`` field.
    """
    case = await db.get_case(case_id)
    if not case:
        raise HTTPException(status_code=404, detail="Dossier introuvable")

    entities = await db.list_entities_by_case(case_id)

    scanned = 0
    errors = 0
    results_summary: List[Dict[str, Any]] = []

    for entity in entities:
        entity_type = entity.get("entity_type")
        entity_name = entity.get("name", "")
        entity_id = entity.get("id")

        if entity_type not in ("email", "account"):
            continue

        try:
            recon_data: Dict[str, Any] = {}

            if entity_type == "email":
                holehe_results = await _holehe.check_email(entity_name)
                social_results = await _social.search_email_username(entity_name)
                recon_data = {
                    "holehe": holehe_results,
                    "social": social_results,
                    "holehe_count": len(holehe_results),
                    "social_found": sum(
                        1 for r in social_results if r.get("exists")
                    ),
                }
            elif entity_type == "account":
                social_results = await _social.search_username(entity_name)
                recon_data = {
                    "social": social_results,
                    "social_found": sum(
                        1 for r in social_results if r.get("exists")
                    ),
                }

            # Merge recon data into existing metadata
            existing_meta = entity.get("metadata") or {}
            if not isinstance(existing_meta, dict):
                existing_meta = {}
            existing_meta["recon"] = recon_data

            await db.update_entity(entity_id, metadata=existing_meta)

            scanned += 1
            results_summary.append({
                "entity_id": entity_id,
                "name": entity_name,
                "type": entity_type,
                "recon": recon_data,
            })

            logger.info(
                "Recon auto: scanned entity '{}' ({})", entity_name, entity_type
            )

        except Exception as exc:
            errors += 1
            logger.exception(
                "Recon auto: error scanning entity '{}' ({}): {}",
                entity_name, entity_type, exc,
            )

    logger.info(
        "Recon auto: case {} -- {} scanned, {} errors",
        case_id[:8], scanned, errors,
    )

    return {
        "case_id": case_id,
        "scanned": scanned,
        "errors": errors,
        "results": results_summary,
    }
