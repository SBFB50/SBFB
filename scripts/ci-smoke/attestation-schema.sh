#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 18 Phase B — validate the SLSA in-toto attestation emitted
# by `scripts/release-attest.sh` against the SLSA Provenance v1.0
# schema.
#
# We check structure + mandatory fields (jq) rather than pulling a
# full JSON Schema validator : the attestation is small, flat, and
# emitted by a script we control. Schema drift between SLSA versions
# is rare enough that catching the obvious shape issues here covers
# 95% of the failure modes.
#
# Usage:
#   bash tests/ci-smoke/attestation-schema.sh [binary]
#
# Default binary is nexus-launcher. The script builds the artefact
# via release-attest.sh then asserts on the resulting .intoto.jsonl.
#
# Exit codes:
#   0  — attestation matches the expected shape.
#   1  — one or more required fields missing / malformed.
#   2  — prerequisites missing (jq not installed).

set -euo pipefail

BINARY="${1:-nexus-launcher}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

red()   { printf '\033[31m%s\033[0m\n' "$*"; }
green() { printf '\033[32m%s\033[0m\n' "$*"; }
bold()  { printf '\033[1m%s\033[0m\n' "$*"; }

if ! command -v jq >/dev/null 2>&1; then
    red "jq not installed — required to parse attestation JSON"
    exit 2
fi

bold "[attestation-schema] target=$BINARY"

OUT="$(mktemp -d -t sbfb-attest-XXXXXX)"
trap 'rm -rf "$OUT"' EXIT

DIST="$OUT" bash scripts/release-attest.sh "$BINARY" >/dev/null

ATT="$(ls "$OUT"/"$BINARY"-*.intoto.jsonl | head -n 1)"
if [[ ! -f "$ATT" ]]; then
    red "no .intoto.jsonl emitted in $OUT"
    exit 1
fi

errors=0
check() {
    local query="$1" expected_type="$2" label="$3"
    local actual
    actual="$(jq -r "$query // empty" "$ATT")"
    if [[ -z "$actual" ]]; then
        red "  missing: $label ($query)"
        errors=$((errors + 1))
    elif [[ "$expected_type" == "sha256" ]] && ! [[ "$actual" =~ ^[a-f0-9]{64}$ ]]; then
        red "  malformed sha256: $label = $actual"
        errors=$((errors + 1))
    elif [[ "$expected_type" == "sha1" ]] && ! [[ "$actual" =~ ^[a-f0-9]{40}$ ]]; then
        red "  malformed sha1: $label = $actual"
        errors=$((errors + 1))
    else
        echo "  ok: $label = $actual"
    fi
}

# in-toto Statement v1 envelope
check '._type'                                  literal  "_type"
check '.predicateType'                          literal  "predicateType"
check '.subject[0].name'                        literal  "subject[0].name"
check '.subject[0].digest.sha256'               sha256   "subject[0].digest.sha256"

# SLSA Provenance v1 predicate
check '.predicate.buildDefinition.buildType'                           literal  "buildType"
check '.predicate.buildDefinition.externalParameters.binary'           literal  "externalParameters.binary"
check '.predicate.buildDefinition.externalParameters.os'               literal  "externalParameters.os"
check '.predicate.buildDefinition.externalParameters.arch'             literal  "externalParameters.arch"
check '.predicate.buildDefinition.internalParameters."SOURCE_DATE_EPOCH"' literal  "SOURCE_DATE_EPOCH"
check '.predicate.buildDefinition.internalParameters.profile'          literal  "profile"
check '.predicate.buildDefinition.resolvedDependencies[0].uri'         literal  "resolvedDependencies[0].uri"
check '.predicate.buildDefinition.resolvedDependencies[0].digest.sha1' sha1     "resolvedDependencies[0].digest.sha1"
check '.predicate.buildDefinition.resolvedDependencies[1].name'        literal  "Cargo.lock dep name"
check '.predicate.buildDefinition.resolvedDependencies[1].digest.sha256' sha256 "Cargo.lock sha256"
check '.predicate.runDetails.builder.id'                               literal  "runDetails.builder.id"
check '.predicate.runDetails.metadata.invocationId'                    literal  "runDetails.metadata.invocationId"
check '.predicate.runDetails.metadata.startedOn'                       literal  "runDetails.metadata.startedOn"

# Exact-match sanity : predicateType must be SLSA Provenance v1
actual_ptype="$(jq -r '.predicateType' "$ATT")"
if [[ "$actual_ptype" != "https://slsa.dev/provenance/v1" ]]; then
    red "  predicateType expected 'https://slsa.dev/provenance/v1', got '$actual_ptype'"
    errors=$((errors + 1))
fi

actual_type="$(jq -r '._type' "$ATT")"
if [[ "$actual_type" != "https://in-toto.io/Statement/v1" ]]; then
    red "  _type expected 'https://in-toto.io/Statement/v1', got '$actual_type'"
    errors=$((errors + 1))
fi

# Cross-check : subject sha256 must match the artefact file on disk.
ARTIFACT="$(ls "$OUT"/"$BINARY"-* | grep -vE '\.(sha256|intoto\.jsonl|sig)$' | head -n 1)"
case "$(uname -s)" in
    Linux*|MINGW*|MSYS*|CYGWIN*) ACTUAL_SHA="$(sha256sum "$ARTIFACT" | awk '{print $1}')" ;;
    Darwin*)                     ACTUAL_SHA="$(shasum -a 256 "$ARTIFACT" | awk '{print $1}')" ;;
esac
CLAIMED_SHA="$(jq -r '.subject[0].digest.sha256' "$ATT")"
if [[ "$ACTUAL_SHA" != "$CLAIMED_SHA" ]]; then
    red "  subject sha256 does not match artefact on disk"
    echo "    claimed: $CLAIMED_SHA"
    echo "    actual:  $ACTUAL_SHA"
    errors=$((errors + 1))
else
    echo "  ok: subject sha256 matches artefact bytes"
fi

if [[ "$errors" -eq 0 ]]; then
    green "[attestation-schema] all checks passed"
    exit 0
else
    red   "[attestation-schema] $errors check(s) failed"
    exit 1
fi
