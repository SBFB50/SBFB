# syntax=docker/dockerfile:1.7
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Pkarr relay self-hosted image for the SBFB federation.
# Wraps the upstream `pkarr-relay` crate (pubky/pkarr, MIT).
#
# Caret range `^0.11` picks up 0.11.x patches (CVE backports)
# without jumping to 0.12 breaking. Bump deliberately after
# upstream changelog review. Reference upstream release:
# v5.0.4 repo tag (2026-03-25), crate pkarr-relay@0.11.5.
#
# Design doc : .planning/research/S19_phase_E_pkarr_relay_design.md
#
# TODO(T23) : pin FROM base images to `@sha256:<digest>` at release
# tag time for reproducible builds. Tracked in docs/rust/PATTERNS.md
# §T23 (audit finding S19 P2-E1). Current tag-only pin is mutable
# upstream and weakens SLSA-L3 reproducibility.

FROM rust:1.94-slim-bookworm AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install pkarr-relay --version "^0.11" --locked --root /build/dist

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates curl tini \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --no-create-home --uid 10001 --shell /usr/sbin/nologin pkarr \
    && mkdir -p /var/lib/pkarr/cache \
    && chown -R pkarr:pkarr /var/lib/pkarr

COPY --from=builder /build/dist/bin/pkarr-relay /usr/local/bin/pkarr-relay
COPY config.toml /etc/pkarr/config.toml

USER pkarr
VOLUME ["/var/lib/pkarr/cache"]
EXPOSE 6881/tcp 6881/udp

# pkarr-relay 0.11.5 routes: GET / (dashboard), GET /:pubkey,
# PUT /:pubkey. Pas d'endpoint /health dedie — on utilise /.
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD curl -fsS http://127.0.0.1:6881/ > /dev/null || exit 1

ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/pkarr-relay"]
CMD ["--config", "/etc/pkarr/config.toml"]
