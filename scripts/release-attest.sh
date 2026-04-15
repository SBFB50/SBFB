#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 18 Phase B — reproducible release build + SLSA in-toto attestation.
#
# Usage:
#   ./scripts/release-attest.sh <binary>
#
# Where <binary> is one of the workspace members known to produce a
# distributable artifact:
#   - nexus-launcher
#   - nexus-worker
#   - nexus-shell-daemon
#   - nexus-core-py  (wheel, built via maturin)
#
# Output (in $DIST, default: dist/):
#   <binary>-<os>-<arch>[.exe]            the artifact itself
#   <binary>-<os>-<arch>[.exe].sha256     sha256sum line
#   <binary>-<os>-<arch>[.exe].intoto.jsonl   SLSA v1.0 provenance (unsigned)
#
# The attestation follows the in-toto statement format with
# `predicateType: https://slsa.dev/provenance/v1`. If `cosign` is on
# $PATH and `COSIGN_KEY` (or GitHub OIDC via `COSIGN_EXPERIMENTAL=1`)
# is configured, a detached signature is emitted next to the jsonl
# file. Without cosign the attestation stays unsigned — downstream
# verifiers can still replay the build and compare SHA256.

set -euo pipefail

BINARY="${1:-}"
if [[ -z "$BINARY" ]]; then
  echo "usage: $0 <binary>" >&2
  echo "  where binary is nexus-launcher, nexus-worker, nexus-shell-daemon or nexus-core-py" >&2
  exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

DIST="${DIST:-$REPO_ROOT/dist}"
mkdir -p "$DIST"

# --- Platform detection ----------------------------------------------------
case "$(uname -s)" in
  Linux*)                OS="linux";   EXT="";      SHA_CMD="sha256sum" ;;
  Darwin*)               OS="macos";   EXT="";      SHA_CMD="shasum -a 256" ;;
  MINGW*|MSYS*|CYGWIN*)  OS="windows"; EXT=".exe";  SHA_CMD="sha256sum" ;;
  *)                     echo "unsupported OS $(uname -s)" >&2; exit 3 ;;
esac

case "$(uname -m)" in
  x86_64|amd64)   ARCH="x86_64" ;;
  arm64|aarch64)  ARCH="arm64" ;;
  *)              echo "unsupported arch $(uname -m)" >&2; exit 3 ;;
esac

# --- SOURCE_DATE_EPOCH pinned to the commit timestamp ---------------------
# Injected here rather than hardcoded in .cargo/config.toml so dev builds
# keep current-time timestamps (expected by cargo's freshness checker)
# while release builds stay deterministic per-commit.
if [[ -z "${SOURCE_DATE_EPOCH:-}" ]]; then
  SOURCE_DATE_EPOCH="$(git log -1 --format=%ct)"
fi
export SOURCE_DATE_EPOCH

COMMIT_SHA="$(git rev-parse HEAD)"
COMMIT_SHORT="$(git rev-parse --short HEAD)"
BUILD_STARTED_ON="$(date -u -d "@$SOURCE_DATE_EPOCH" +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null \
                    || date -u -r "$SOURCE_DATE_EPOCH" +"%Y-%m-%dT%H:%M:%SZ")"

echo "==> release-attest: binary=$BINARY os=$OS arch=$ARCH"
echo "    commit=$COMMIT_SHORT  SOURCE_DATE_EPOCH=$SOURCE_DATE_EPOCH"

# --- Build -----------------------------------------------------------------
if [[ "$BINARY" == "nexus-core-py" ]]; then
  # Python extension wheel built via maturin. Reproducibility here is
  # best-effort: maturin honors SOURCE_DATE_EPOCH for the wheel zip
  # timestamps and we pin the interpreter ABI via --locked + pyproject.
  ARTIFACT_NAME="nexus_core_py"
  echo "==> maturin build --release --locked --manifest-path crates/nexus-core-py/Cargo.toml --out $DIST"
  maturin build --release --locked \
    --manifest-path crates/nexus-core-py/Cargo.toml \
    --out "$DIST"
  WHEEL_PATH="$(ls "$DIST"/${ARTIFACT_NAME}-*.whl | head -n 1)"
  ARTIFACT_PATH="$WHEEL_PATH"
  ARTIFACT_BASENAME="$(basename "$ARTIFACT_PATH")"
else
  echo "==> cargo build --release --locked -p $BINARY"
  cargo build --release --locked -p "$BINARY"

  SRC="target/release/${BINARY}${EXT}"
  if [[ ! -f "$SRC" ]]; then
    echo "build did not produce $SRC" >&2
    exit 4
  fi

  ARTIFACT_BASENAME="${BINARY}-${OS}-${ARCH}${EXT}"
  ARTIFACT_PATH="$DIST/$ARTIFACT_BASENAME"
  cp "$SRC" "$ARTIFACT_PATH"
  [[ -z "$EXT" ]] && chmod +x "$ARTIFACT_PATH"
fi

# --- SHA256 ----------------------------------------------------------------
echo "==> sha256"
# Emit the classic "<hex>  <basename>" form consumed by `sha256sum -c`.
(
  cd "$DIST"
  $SHA_CMD "$ARTIFACT_BASENAME" > "${ARTIFACT_BASENAME}.sha256"
)
ARTIFACT_SHA256="$(awk '{print $1}' "$DIST/${ARTIFACT_BASENAME}.sha256")"
echo "    sha256=$ARTIFACT_SHA256"

# --- SLSA in-toto attestation (unsigned statement) -------------------------
# Schema: https://slsa.dev/spec/v1.0/provenance
# Shape: a single in-toto Statement v1 wrapping a SLSA Provenance v1 predicate.
echo "==> attestation"
CARGO_LOCK_SHA="$(${SHA_CMD%% *} Cargo.lock | awk '{print $1}')"
ATTESTATION_PATH="$DIST/${ARTIFACT_BASENAME}.intoto.jsonl"
BUILDER_ID="${GITHUB_SERVER_URL:-https://github.com}/${GITHUB_REPOSITORY:-SBFB50/SBFB}/.github/workflows/release.yml@${GITHUB_REF:-refs/heads/master}"
BUILD_INVOCATION_ID="${GITHUB_RUN_ID:-local-$(date -u +%s)}"

cat > "$ATTESTATION_PATH" <<EOF
{"_type":"https://in-toto.io/Statement/v1","subject":[{"name":"${ARTIFACT_BASENAME}","digest":{"sha256":"${ARTIFACT_SHA256}"}}],"predicateType":"https://slsa.dev/provenance/v1","predicate":{"buildDefinition":{"buildType":"https://slsa.dev/container-based-build/v0.1?sbfb=release-attest.sh","externalParameters":{"binary":"${BINARY}","os":"${OS}","arch":"${ARCH}"},"internalParameters":{"SOURCE_DATE_EPOCH":"${SOURCE_DATE_EPOCH}","profile":"release","locked":true,"codegen-units":1,"lto":"fat","strip":"symbols"},"resolvedDependencies":[{"uri":"git+${GITHUB_SERVER_URL:-https://github.com}/${GITHUB_REPOSITORY:-SBFB50/SBFB}@${COMMIT_SHA}","digest":{"sha1":"${COMMIT_SHA}"}},{"name":"Cargo.lock","digest":{"sha256":"${CARGO_LOCK_SHA}"}}]},"runDetails":{"builder":{"id":"${BUILDER_ID}"},"metadata":{"invocationId":"${BUILD_INVOCATION_ID}","startedOn":"${BUILD_STARTED_ON}"},"byproducts":[]}}}
EOF

echo "    attestation=$ATTESTATION_PATH"

# --- Optional cosign signature (keyless OIDC or key-based) -----------------
if command -v cosign >/dev/null 2>&1; then
  if [[ -n "${COSIGN_KEY:-}" ]] || [[ "${COSIGN_EXPERIMENTAL:-0}" == "1" ]]; then
    echo "==> cosign sign-blob"
    cosign sign-blob --yes \
      --output-signature "${ATTESTATION_PATH}.sig" \
      "$ATTESTATION_PATH"
    echo "    signature=${ATTESTATION_PATH}.sig"
  else
    echo "    cosign present but COSIGN_KEY / COSIGN_EXPERIMENTAL=1 not set — skipping signature"
  fi
else
  echo "    cosign not installed — emitting unsigned attestation only"
fi

echo "==> done"
echo "    artifact:    $ARTIFACT_PATH"
echo "    sha256 file: $DIST/${ARTIFACT_BASENAME}.sha256"
echo "    attestation: $ATTESTATION_PATH"
