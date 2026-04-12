#!/usr/bin/env bash
#
# Sprint 9 Phase A (D5) — idempotent dev setup.
#
# Closes H-3 (`nexus_core` wheel editable install drift). A fresh
# checkout can now run `./scripts/setup.sh` once and have a usable
# Python workspace + working PyO3 wheel without remembering the
# `unset CONDA_PREFIX && VIRTUAL_ENV=... maturin develop` ritual.
#
# How it works:
#
# 1. Create `.venv/` if it does not exist (uv venv).
# 2. `uv sync --all-extras --all-packages` resolves the entire
#    workspace plus every member's `[project.optional-dependencies]
#    test` extra. This installs nexus-sdk, nexus-coordinator,
#    nexus-app-gov, hello-world-app, the test extras (pytest,
#    pytest-asyncio, pytest-timeout) and triggers maturin to build
#    the nexus-core-py wheel from source the first time.
# 3. Hash `Cargo.lock` + `crates/nexus-core-{rs,py}/src` into
#    `.venv/.nexus-core-hash`. uv caches the maturin build but does
#    NOT rebuild it on source changes unless we pass `--refresh`.
#    The hash detects a Rust source drift between runs.
# 4. If the hash drifted OR if `import nexus_core` is missing OR if
#    `nexus_core.sign_curator_list` is missing (which is a sentinel
#    for the rust src predating Sprint 7 D3), force a rebuild via
#    `uv pip install -e crates/nexus-core-py --refresh`. uv handles
#    the maturin invocation and tracks the install in its lockfile,
#    so subsequent `uv sync` runs leave the wheel alone.
#
# Usage:
#   ./scripts/setup.sh          # idempotent setup
#   ./scripts/setup.sh --force  # always rebuild the wheel

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

FORCE=0
if [[ "${1:-}" == "--force" ]]; then
  FORCE=1
fi

VENV_DIR="$REPO_ROOT/.venv"
HASH_FILE="$VENV_DIR/.nexus-core-hash"

if [[ ! -d "$VENV_DIR" ]]; then
  echo "==> creating uv venv (.venv/)"
  uv venv
fi

echo "==> uv sync --all-extras --all-packages (workspace + test extras)"
uv sync --all-extras --all-packages

# Pick the venv python on Windows or POSIX layouts so the import
# probe below works on both.
VENV_PY=""
if [[ -x "$VENV_DIR/Scripts/python.exe" ]]; then
  VENV_PY="$VENV_DIR/Scripts/python.exe"
elif [[ -x "$VENV_DIR/bin/python" ]]; then
  VENV_PY="$VENV_DIR/bin/python"
else
  echo "==> ERROR: cannot find python interpreter inside $VENV_DIR" >&2
  exit 1
fi

compute_hash() {
  # Concatenate every tracked Rust source under the two core crates plus
  # the workspace Cargo.lock, pipe through sha256sum, keep only the digest.
  (
    cat Cargo.lock 2>/dev/null || true
    find crates/nexus-core-rs/src crates/nexus-core-py/src -type f \
      \( -name '*.rs' -o -name '*.toml' \) -print0 2>/dev/null \
      | sort -z \
      | xargs -0 cat 2>/dev/null || true
  ) | sha256sum | awk '{print $1}'
}

CURRENT_HASH="$(compute_hash)"
PREVIOUS_HASH=""
if [[ -f "$HASH_FILE" ]]; then
  PREVIOUS_HASH="$(cat "$HASH_FILE")"
fi

# Sentinel-feature probe: `sign_curator_list` was added in Sprint 7
# Phase B and is expected on every modern build. If uv served a
# stale cached wheel that predates it, we still need to rebuild.
WHEEL_OK=0
if "$VENV_PY" -c "import nexus_core; assert hasattr(nexus_core, 'sign_curator_list')" >/dev/null 2>&1; then
  WHEEL_OK=1
fi

if [[ "$FORCE" -eq 0 \
   && "$CURRENT_HASH" == "$PREVIOUS_HASH" \
   && -n "$PREVIOUS_HASH" \
   && "$WHEEL_OK" -eq 1 ]]; then
  echo "==> nexus_core wheel up to date (hash $CURRENT_HASH), skipping rebuild"
  exit 0
fi

if [[ "$WHEEL_OK" -eq 0 ]]; then
  echo "==> nexus_core wheel missing or stale, forcing rebuild via uv pip install -e --refresh"
else
  echo "==> nexus_core source hash changed, forcing rebuild"
  echo "    previous=${PREVIOUS_HASH:-<none>}"
  echo "    current=$CURRENT_HASH"
fi

uv pip install -e crates/nexus-core-py --refresh

mkdir -p "$VENV_DIR"
echo "$CURRENT_HASH" >"$HASH_FILE"

echo "==> setup.sh done. Hash: $CURRENT_HASH"
