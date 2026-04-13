# SPDX-License-Identifier: AGPL-3.0-or-later
"""Multi-forge URL detection and helpers for deploy-from-repo.

Sprint 14 Phase A — supports GitHub, GitLab, Codeberg, and generic
Gitea instances. The coordinator uses these helpers to:

1. Detect the forge type from a repo URL.
2. Construct a raw-file URL to pre-check ``SBFB.json`` without cloning.
3. Check whether a repository is publicly accessible.
4. Normalize the clone URL (strip fragments, trailing slashes).
"""

from __future__ import annotations

import re
from enum import Enum
from typing import NamedTuple

import httpx
import structlog

_log = structlog.get_logger(__name__)


class ForgeType(str, Enum):
    GITHUB = "github"
    GITLAB = "gitlab"
    CODEBERG = "codeberg"
    GITEA = "gitea"
    UNKNOWN = "unknown"


class ForgeInfo(NamedTuple):
    forge: ForgeType
    owner: str
    repo: str
    host: str


# Patterns match the most common forge URLs.
# Order matters — Codeberg before generic Gitea.
_FORGE_PATTERNS: list[tuple[re.Pattern[str], ForgeType]] = [
    (re.compile(r"https?://github\.com/(?P<owner>[^/]+)/(?P<repo>[^/.]+)"), ForgeType.GITHUB),
    (re.compile(r"https?://gitlab\.com/(?P<owner>[^/]+)/(?P<repo>[^/.]+)"), ForgeType.GITLAB),
    (re.compile(r"https?://codeberg\.org/(?P<owner>[^/]+)/(?P<repo>[^/.]+)"), ForgeType.CODEBERG),
    # Generic Gitea: any host with /<owner>/<repo> path.
    # Must be last — it's a fallback for self-hosted instances.
    (re.compile(r"https?://(?P<host>[^/]+)/(?P<owner>[^/]+)/(?P<repo>[^/.]+)"), ForgeType.GITEA),
]


def detect_forge(repo_url: str) -> ForgeInfo:
    """Detect the forge type from a repository URL.

    Returns a :class:`ForgeInfo` with the parsed owner, repo, host,
    and forge type. Unknown forges return ``ForgeType.UNKNOWN`` with
    empty owner/repo fields.
    """
    url = normalize_clone_url(repo_url)
    for pattern, forge_type in _FORGE_PATTERNS:
        m = pattern.match(url)
        if m:
            owner = m.group("owner")
            repo = m.group("repo")
            host = m.group("host") if "host" in m.groupdict() else ""
            return ForgeInfo(forge=forge_type, owner=owner, repo=repo, host=host)
    return ForgeInfo(forge=ForgeType.UNKNOWN, owner="", repo="", host="")


def raw_file_url(repo_url: str, path: str, ref: str = "HEAD") -> str | None:
    """Construct a URL to fetch a raw file from the repo without cloning.

    Returns ``None`` if the forge type is unknown (generic Gitea
    instances don't have a standardized raw URL pattern that works
    without API tokens).
    """
    info = detect_forge(repo_url)

    if info.forge == ForgeType.GITHUB:
        return f"https://raw.githubusercontent.com/{info.owner}/{info.repo}/{ref}/{path}"
    if info.forge == ForgeType.GITLAB:
        return f"https://gitlab.com/{info.owner}/{info.repo}/-/raw/{ref}/{path}"
    if info.forge == ForgeType.CODEBERG:
        return f"https://codeberg.org/{info.owner}/{info.repo}/raw/branch/{ref}/{path}"
    if info.forge == ForgeType.GITEA:
        return f"https://{info.host}/{info.owner}/{info.repo}/raw/branch/{ref}/{path}"

    return None


async def is_repo_public(repo_url: str) -> bool:
    """Check whether a repository is publicly accessible.

    Does a HEAD request to the repo's web URL. Returns ``True`` if
    the server responds with 200 (public). Returns ``False`` on 404,
    403, or any error.
    """
    url = normalize_clone_url(repo_url)
    try:
        async with httpx.AsyncClient(timeout=10.0, follow_redirects=True) as client:
            resp = await client.head(url)
            return resp.status_code == 200
    except httpx.HTTPError as e:
        _log.debug("is_repo_public check failed", url=url, error=str(e))
        return False


def normalize_clone_url(repo_url: str) -> str:
    """Normalize a repo URL for cloning.

    Strips trailing slashes, ``.git`` suffix, and URL fragments.
    """
    url = repo_url.strip()
    # Remove fragment.
    url = url.split("#")[0]
    # Remove query string.
    url = url.split("?")[0]
    # Remove trailing slashes.
    url = url.rstrip("/")
    # Remove .git suffix (common in clone URLs).
    if url.endswith(".git"):
        url = url[:-4]
    return url
