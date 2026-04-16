#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Smoke test for the SBFB pkarr-relay Docker image.
# Runs the container, waits for HTTP server to come up, curls the
# dashboard index (GET /) to confirm liveness, tears down.
#
# Usage: pkarr-relay-healthcheck.sh <image-ref>
#
#   image-ref : full OCI ref, e.g.
#               ghcr.io/sbfb50/pkarr-relay:latest
#               ghcr.io/sbfb50/pkarr-relay@sha256:...
#
# Exit codes:
#   0  dashboard serves HTTP 200
#   1  image pull failed
#   2  container failed to start
#   3  dashboard unreachable within timeout
#
# Called from .github/workflows/build-pkarr-image.yml and from
# PKARR_RELAY_OPS.md §4 local deploy validation.

set -euo pipefail

IMAGE_REF="${1:?image ref required}"
CONTAINER_NAME="pkarr-relay-smoke-$$"
HOST_PORT="${HOST_PORT:-38881}"
STARTUP_TIMEOUT="${STARTUP_TIMEOUT:-30}"

cleanup() {
    docker rm -f "${CONTAINER_NAME}" >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "[smoke] image=${IMAGE_REF}"

if ! docker pull "${IMAGE_REF}"; then
    echo "[smoke] FAIL pull"
    exit 1
fi

echo "[smoke] starting container (host port ${HOST_PORT})"
if ! docker run -d --rm --name "${CONTAINER_NAME}" \
        -p "127.0.0.1:${HOST_PORT}:6881/tcp" \
        "${IMAGE_REF}" >/dev/null; then
    echo "[smoke] FAIL run"
    exit 2
fi

echo "[smoke] waiting up to ${STARTUP_TIMEOUT}s for dashboard"
deadline=$(( $(date +%s) + STARTUP_TIMEOUT ))
while (( $(date +%s) < deadline )); do
    if curl -fsS "http://127.0.0.1:${HOST_PORT}/" > /dev/null; then
        echo "[smoke] OK dashboard responded"
        exit 0
    fi
    sleep 1
done

echo "[smoke] FAIL dashboard unreachable after ${STARTUP_TIMEOUT}s"
echo "[smoke] container logs ----"
docker logs "${CONTAINER_NAME}" 2>&1 | head -40 || true
exit 3
