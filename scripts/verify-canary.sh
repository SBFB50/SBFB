#!/usr/bin/env bash
#
# Sprint 18 Phase E2 — warrant canary verifier.
#
# Thin wrapper around `nexus-shell-daemon canary verify` that
# a fresh cloner can run without booting the full daemon:
#
#   ./scripts/verify-canary.sh
#   ./scripts/verify-canary.sh path/to/CANARY.txt
#
# Exit codes:
#   0 — canary parses and its Ed25519 signature validates
#   1 — canary file missing, malformed, or signature rejected
#   2 — the nexus-shell-daemon binary is missing (run `cargo build
#       -p nexus-shell-daemon` first)
#
# The script deliberately stays portable (bash 3.2+, no jq, no
# gpg) so a journalist or curious user with just a git clone +
# Rust toolchain can verify the declaration offline.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
canary_file="${1:-${repo_root}/CANARY.txt}"

if [[ ! -f "${canary_file}" ]]; then
    echo "verify-canary: file not found: ${canary_file}" >&2
    exit 1
fi

# Resolve the daemon binary: prefer debug (what the script's
# `cargo build -p nexus-shell-daemon` hint produces) over a
# possibly-stale release build from a previous sprint.
daemon_bin=""
for candidate in \
    "${repo_root}/target/debug/nexus-shell-daemon" \
    "${repo_root}/target/debug/nexus-shell-daemon.exe" \
    "${repo_root}/target/release/nexus-shell-daemon" \
    "${repo_root}/target/release/nexus-shell-daemon.exe"; do
    if [[ -x "${candidate}" ]]; then
        daemon_bin="${candidate}"
        break
    fi
done

if [[ -z "${daemon_bin}" ]]; then
    echo "verify-canary: nexus-shell-daemon binary not found" >&2
    echo "  run: cargo build -p nexus-shell-daemon" >&2
    exit 2
fi

# The subcommand writes an "canary OK" line on success and a
# non-zero exit code on any parse / signature error, so we can
# just let it drive the script's exit status.
exec "${daemon_bin}" canary verify --input "${canary_file}"
