# SPDX-License-Identifier: AGPL-3.0-or-later
"""Deploy endpoints for web apps.

Sprint 12 Phase B — ``POST /project/deploy`` (upload zip, private only).
Sprint 14 Phase A — ``POST /project/deploy-from-repo`` (clone + verify
+ provenance, public apps).

The deploy-from-repo endpoint:

1. Clones the repo with ``git clone --depth 1``.
2. Verifies ``SBFB.json`` (node_id matches the daemon).
3. Checks the repo is public (HTTP HEAD).
4. Zips the content (excludes ``.git/``).
5. Generates and signs ``provenance.json`` (SLSA L1).
6. Includes ``provenance.json`` in the zip.
7. Stores the zip via the daemon's ``POST /publish-blob``.
8. Publishes a v4 announcement with ``provenance_hash``.
"""

from __future__ import annotations

import asyncio
import io
import json
import os
import re
import shutil
import subprocess
import tempfile
import zipfile
from pathlib import Path

import httpx
import nexus_core
import structlog
from fastapi import APIRouter, Form, HTTPException, Request, UploadFile
from pydantic import BaseModel, field_validator

from nexus_coordinator.api.daemon import _daemon_base_url, _read_running_state
from nexus_coordinator.forge import is_repo_public, normalize_clone_url
from nexus_coordinator.provenance import (
    generate_provenance,
    provenance_blake3_hex,
    provenance_to_json,
)

_log = structlog.get_logger(__name__)

router = APIRouter(tags=["deploy"])

# 100 MB — consistent with blob-serve daemon DEFAULT_MAX_DECOMPRESSED_BYTES.
MAX_DEPLOY_BYTES: int = 100 * 1024 * 1024


def _validate_zip(data: bytes) -> None:
    """Raise HTTPException(400) if ``data`` is not a valid zip with index.html."""
    try:
        with zipfile.ZipFile(io.BytesIO(data), "r") as zf:
            names = zf.namelist()
    except (zipfile.BadZipFile, Exception) as e:
        raise HTTPException(status_code=400, detail=f"invalid zip archive: {e}") from e

    if "index.html" not in names:
        raise HTTPException(
            status_code=400,
            detail="zip archive must contain an index.html at the root",
        )


async def _store_blob(request: Request, zip_bytes: bytes) -> str:
    """Store bytes as a blob via the daemon's POST /publish-blob.

    Returns the hex hash of the stored blob.
    """
    state = _read_running_state()
    if state is None:
        raise HTTPException(status_code=503, detail="shell-daemon not running")

    url = f"{_daemon_base_url(state)}/publish-blob"
    client: httpx.AsyncClient = request.app.state.daemon_httpx_client
    try:
        resp = await client.post(
            url,
            content=zip_bytes,
            headers={"Content-Type": "application/octet-stream"},
        )
    except httpx.HTTPError as e:
        raise HTTPException(status_code=503, detail=f"daemon unreachable: {e}") from e

    if resp.status_code != 200:
        raise HTTPException(
            status_code=502,
            detail=f"daemon /publish-blob returned {resp.status_code}: {resp.text}",
        )

    body = resp.json()
    return body["hash"]


async def _publish_with_archive(
    request: Request,
    hash_hex: str,
    repo_url: str | None = None,
    provenance_hash: str | None = None,
) -> None:
    """Publish announcement with archive hash, optional repo_url and provenance_hash."""
    coord = request.app.state.coordinator
    state = _read_running_state()
    if state is None:
        _log.warning("publish skipped: daemon not running")
        return

    url = f"{_daemon_base_url(state)}/publish"
    payload: dict = {
        "project_name": coord.project_name,
        "category": coord.config.identity.description or "general",
        "description": coord.config.identity.description or coord.project_name,
        "apps": list(coord.apps.keys()),
        "archive_hash": hash_hex,
    }
    if repo_url:
        payload["repo_url"] = repo_url
    if provenance_hash:
        payload["provenance_hash"] = provenance_hash
    client: httpx.AsyncClient = request.app.state.daemon_httpx_client
    try:
        resp = await client.post(url, json=payload)
        if resp.status_code != 200:
            _log.warning("publish returned non-200", status=resp.status_code)
    except httpx.HTTPError as e:
        _log.warning("publish failed", error=str(e))


@router.post("/project/deploy")
async def deploy_project(
    archive: UploadFile,
    request: Request,
    repo_url: str | None = Form(default=None),
) -> dict:
    """Upload a zip archive and publish to the P2P network.

    Since Sprint 14, public projects must use
    ``POST /project/deploy-from-repo`` instead (verified deploy).
    This endpoint is restricted to private projects only.
    """
    coord = request.app.state.coordinator
    # Sprint 14 Phase D: public projects must use deploy-from-repo.
    if coord.config.network.visibility == "public":
        raise HTTPException(
            status_code=400,
            detail=(
                "Public projects must use POST /project/deploy-from-repo "
                "for verified deploy. This endpoint is for private apps only."
            ),
        )

    zip_bytes = await archive.read()
    if len(zip_bytes) > MAX_DEPLOY_BYTES:
        raise HTTPException(
            status_code=413,
            detail=(
                f"Upload exceeds the maximum allowed size of "
                f"{MAX_DEPLOY_BYTES} bytes ({MAX_DEPLOY_BYTES // (1024 * 1024)} MB)"
            ),
        )
    _log.info("deploy: received zip", size=len(zip_bytes))

    _validate_zip(zip_bytes)

    hash_hex = await _store_blob(request, zip_bytes)
    _log.info("deploy: blob stored", hash=hash_hex)

    await _publish_with_archive(request, hash_hex, repo_url=repo_url)

    return {"deployed": True, "hash": hash_hex}


# ------------------------------------------------------------------
# Sprint 14 Phase A — deploy from repo
# ------------------------------------------------------------------

# Limits for git clone safety (D4).
MAX_CLONE_BYTES: int = 500 * 1024 * 1024  # 500 MB
CLONE_TIMEOUT_SECS: int = 30
CHECKOUT_TIMEOUT_SECS: int = 10

# A full Git SHA-1 commit hash: 40 lowercase hex characters.
_SHA_PATTERN = re.compile(r"^[a-f0-9]{40}$")


class DeployFromRepoBody(BaseModel):
    """Request body for ``POST /project/deploy-from-repo``."""

    repo_url: str
    commit_sha: str | None = None

    @field_validator("commit_sha")
    @classmethod
    def _validate_commit_sha(cls, v: str | None) -> str | None:
        """Accept only full 40-char hex SHAs (normalized to lowercase)."""
        if v is None:
            return v
        normalized = v.lower()
        if not _SHA_PATTERN.match(normalized):
            raise ValueError(
                "commit_sha must be a full 40-character hex SHA (short SHAs and branch/tag names are not supported)"
            )
        return normalized


@router.post("/project/deploy-from-repo")
async def deploy_from_repo(body: DeployFromRepoBody, request: Request) -> dict:
    """Clone a public repo, verify SBFB.json, build zip, sign provenance.

    This is the verified deploy path for public apps. The coordinator
    clones the repository itself, ensuring the code on the network
    matches the code in the repo. Private apps must use
    ``POST /project/deploy`` (upload zip) instead.
    """
    coord = request.app.state.coordinator

    # Only public projects use deploy-from-repo.
    if coord.config.network.visibility != "public":
        raise HTTPException(
            status_code=400,
            detail="deploy-from-repo is for public projects only. Use POST /project/deploy for private apps.",
        )

    repo_url = normalize_clone_url(body.repo_url)
    if not repo_url or not repo_url.startswith("http"):
        raise HTTPException(status_code=400, detail="repo_url must be an HTTP(S) URL")

    # 1. Verify repo is public.
    if not await is_repo_public(repo_url):
        raise HTTPException(status_code=400, detail="Repository is not publicly accessible")

    # 2. Clone into a temporary directory.
    tmpdir = tempfile.mkdtemp(prefix="sbfb-deploy-")
    clone_dir = os.path.join(tmpdir, "repo")
    try:
        await _clone_repo(repo_url, clone_dir, sha=body.commit_sha)

        # 3. Check clone size.
        clone_size = _dir_size(clone_dir)
        if clone_size > MAX_CLONE_BYTES:
            raise HTTPException(
                status_code=413,
                detail=f"Repository exceeds {MAX_CLONE_BYTES // (1024 * 1024)} MB limit",
            )

        # 4. Verify SBFB.json.
        daemon_state = _read_running_state()
        if daemon_state is None:
            raise HTTPException(status_code=503, detail="shell-daemon not running")
        sbfb = _read_sbfb_json(clone_dir)
        if sbfb["node_id"] != daemon_state.node_id:
            raise HTTPException(
                status_code=400,
                detail=(
                    f"SBFB.json node_id ({sbfb['node_id'][:16]}...) does not match "
                    f"daemon node_id ({daemon_state.node_id[:16]}...)"
                ),
            )

        # 5. Verify index.html exists.
        if not os.path.isfile(os.path.join(clone_dir, "index.html")):
            raise HTTPException(status_code=400, detail="Repository must contain index.html at root")

        # 6. Get commit SHA.
        commit_sha = body.commit_sha or await _git_rev_parse(clone_dir)

        # 7. Zip the content (exclude .git/).
        zip_bytes = _zip_directory(clone_dir)
        _log.info("deploy-from-repo: zipped", size=len(zip_bytes))

        # 8. Compute artifact hash (BLAKE3 of zip without provenance).
        artifact_hash_bytes: bytes = nexus_core.blake3_digest(zip_bytes)
        artifact_hash_hex = artifact_hash_bytes.hex()

        # 9. Generate signed provenance.
        provenance = generate_provenance(
            repo_url=repo_url,
            commit_sha=commit_sha,
            artifact_hash=artifact_hash_hex,
            node_id_hex=daemon_state.node_id,
            secret=coord.keypair.secret,
        )

        # 10. Add provenance.json to the zip.
        zip_bytes = _add_to_zip(zip_bytes, "provenance.json", provenance_to_json(provenance))
        _log.info("deploy-from-repo: provenance added", size=len(zip_bytes))

        # 11. Store blob.
        hash_hex = await _store_blob(request, zip_bytes)
        _log.info("deploy-from-repo: blob stored", hash=hash_hex)

        # 12. Publish with provenance hash.
        prov_hash = provenance_blake3_hex(provenance)
        await _publish_with_archive(
            request,
            hash_hex,
            repo_url=repo_url,
            provenance_hash=prov_hash,
        )

        return {
            "deployed": True,
            "hash": hash_hex,
            "provenance_hash": prov_hash,
            "commit_sha": commit_sha,
        }
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)


# ------------------------------------------------------------------
# Helpers
# ------------------------------------------------------------------


async def _clone_repo(repo_url: str, dest: str, *, sha: str | None = None) -> None:
    """Clone a repo with timeout protection, optionally pinned to a SHA.

    When ``sha`` is ``None``, clones HEAD of the default branch with
    ``--depth 1 --single-branch`` (fast path, typical case).

    When ``sha`` is provided (validated as a 40-char hex by
    :class:`DeployFromRepoBody`), the clone is done in three steps:

    1. ``git clone --depth 1 --single-branch`` to bootstrap the repo.
    2. ``git fetch --depth 1 origin <sha>`` to pull the target commit
       without downloading the full history.
    3. ``git checkout FETCH_HEAD`` to move the working tree to it.

    Step 2 requires the remote to allow fetching by SHA. GitHub,
    GitLab, and Codeberg enable this for public repos by default
    (``uploadpack.allowReachableSHA1InWant=true``). Self-hosted Gitea
    instances may need explicit configuration.
    """
    # Step 1: initial shallow clone of HEAD.
    await _run_git(
        ["git", "clone", "--depth", "1", "--single-branch", repo_url, dest],
        timeout=CLONE_TIMEOUT_SECS,
        action="clone",
    )

    if sha is None:
        return

    # Step 2: fetch the specific SHA into the clone.
    await _run_git(
        ["git", "-C", dest, "fetch", "--depth", "1", "origin", sha],
        timeout=CLONE_TIMEOUT_SECS,
        action="fetch",
    )

    # Step 3: checkout the fetched commit in detached HEAD.
    await _run_git(
        ["git", "-C", dest, "checkout", "FETCH_HEAD"],
        timeout=CHECKOUT_TIMEOUT_SECS,
        action="checkout",
    )


async def _run_git(cmd: list[str], *, timeout: int, action: str) -> None:
    """Run a git subprocess with a hard timeout.

    Raises :class:`HTTPException` (400 on non-zero exit, 408 on
    timeout) with a truncated stderr excerpt as the error detail so
    the API caller gets a meaningful message (for example
    ``git fetch failed: fatal: remote SHA not found``).
    """
    try:
        proc = await asyncio.create_subprocess_exec(
            *cmd,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
        )
        _, stderr = await asyncio.wait_for(proc.communicate(), timeout=timeout)
        if proc.returncode != 0:
            detail = stderr.decode("utf-8", errors="replace").strip()[:500]
            raise HTTPException(status_code=400, detail=f"git {action} failed: {detail}")
    except asyncio.TimeoutError as e:
        raise HTTPException(
            status_code=408,
            detail=f"git {action} timed out after {timeout}s",
        ) from e


async def _git_rev_parse(repo_dir: str) -> str:
    """Get the HEAD commit SHA from a cloned repo."""
    proc = await asyncio.create_subprocess_exec(
        "git",
        "rev-parse",
        "HEAD",
        cwd=repo_dir,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
    )
    stdout, _ = await proc.communicate()
    return stdout.decode("utf-8").strip()


def _read_sbfb_json(repo_dir: str) -> dict:
    """Read and validate SBFB.json from the repo root."""
    path = os.path.join(repo_dir, "SBFB.json")
    if not os.path.isfile(path):
        raise HTTPException(status_code=400, detail="Repository must contain SBFB.json at root")
    try:
        with open(path, encoding="utf-8") as f:
            data = json.load(f)
    except (json.JSONDecodeError, OSError) as e:
        raise HTTPException(status_code=400, detail=f"Invalid SBFB.json: {e}") from e
    if "node_id" not in data:
        raise HTTPException(status_code=400, detail="SBFB.json must contain a 'node_id' field")
    return data


def _zip_directory(src_dir: str) -> bytes:
    """Zip a directory, excluding .git/ and validating paths."""
    buf = io.BytesIO()
    src = Path(src_dir)
    with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED) as zf:
        for root, dirs, files in os.walk(src):
            # Exclude .git directory.
            dirs[:] = [d for d in dirs if d != ".git"]
            for fname in files:
                full = Path(root) / fname
                arcname = str(full.relative_to(src))
                # Path traversal protection.
                if ".." in arcname or arcname.startswith("/"):
                    _log.warning("skipping suspicious path", path=arcname)
                    continue
                # No symlinks.
                if full.is_symlink():
                    _log.warning("skipping symlink", path=arcname)
                    continue
                zf.write(full, arcname)
    return buf.getvalue()


def _add_to_zip(zip_bytes: bytes, name: str, content: str) -> bytes:
    """Add a file to an existing zip archive (returns new bytes)."""
    buf = io.BytesIO(zip_bytes)
    with zipfile.ZipFile(buf, "a", zipfile.ZIP_DEFLATED) as zf:
        zf.writestr(name, content)
    return buf.getvalue()


def _dir_size(path: str) -> int:
    """Compute total size of all files in a directory tree."""
    total = 0
    for dirpath, _dirs, filenames in os.walk(path):
        for f in filenames:
            fp = os.path.join(dirpath, f)
            if not os.path.islink(fp):
                total += os.path.getsize(fp)
    return total
