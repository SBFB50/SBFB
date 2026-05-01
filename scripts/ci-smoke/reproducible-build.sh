#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 18 Phase B — smoke-test that `scripts/release-attest.sh`
# produces byte-identical artefacts for two back-to-back invocations
# of the same commit.
#
# This is the local equivalent of the GitHub Actions reproducibility
# check : rebuild the same binary twice, pinning SOURCE_DATE_EPOCH to
# the commit timestamp, and assert the SHA256 matches.
#
# Usage:
#   bash tests/ci-smoke/reproducible-build.sh [binary]
#
# Default binary is nexus-launcher (smallest of the three, fastest
# build). Pass `nexus-worker` or `nexus-shell-daemon` to test those.
#
# Exit codes:
#   0  — SHA256 matches between both builds.
#   1  — mismatch, artefact not reproducible.
#   2  — usage / environment error.

set -euo pipefail

BINARY="${1:-nexus-launcher}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

red()   { printf '\033[31m%s\033[0m\n' "$*"; }
green() { printf '\033[32m%s\033[0m\n' "$*"; }
bold()  { printf '\033[1m%s\033[0m\n' "$*"; }

bold "[reproducible-build] target=$BINARY"

# Two separate output dirs so the second build does not clobber the
# sha256 of the first run before we compare.
OUT1="$(mktemp -d -t sbfb-rebuild-1-XXXXXX)"
OUT2="$(mktemp -d -t sbfb-rebuild-2-XXXXXX)"
trap 'rm -rf "$OUT1" "$OUT2"' EXIT

# Force-invalidate the cargo release cache between runs so we really
# exercise codegen both times. Without this the second build would be
# a no-op and the comparison would be vacuous.
clean_release_target() {
    cargo clean --release -p "$BINARY" >/dev/null 2>&1 || true
}

bold "[1/2] first build"
clean_release_target
DIST="$OUT1" bash scripts/release-attest.sh "$BINARY" >/dev/null
SHA1="$(awk '{print $1}' "$OUT1"/"$BINARY"-*.sha256 | head -n 1)"
echo "  sha256_1 = $SHA1"

bold "[2/2] second build"
clean_release_target
DIST="$OUT2" bash scripts/release-attest.sh "$BINARY" >/dev/null
SHA2="$(awk '{print $1}' "$OUT2"/"$BINARY"-*.sha256 | head -n 1)"
echo "  sha256_2 = $SHA2"

if [[ "$SHA1" == "$SHA2" ]]; then
    green "[reproducible-build] sha256 match — build is reproducible"
    exit 0
else
    red "[reproducible-build] sha256 MISMATCH — build is NOT reproducible"
    echo "  sha256_1 = $SHA1"
    echo "  sha256_2 = $SHA2"
    echo "  OUT1 = $OUT1"
    echo "  OUT2 = $OUT2"
    # Do not delete the temp dirs on failure so the user can diff.
    trap - EXIT
    exit 1
fi
