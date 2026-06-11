# Local dev helper — NOT the CI pipeline (see .woodpecker.yml).
# Pin to match Woodpecker CI's rust image version to prevent
# rustfmt drift between local and CI (cf. PATTERNS.md §P45).
FROM rust:1.94
# libgtk-3-dev: nexus-launcher links GTK on Linux (atk/gdk pkg-config),
# required for any --workspace / --all-targets build (S73 wrap-up recipe,
# re-pinned here S75 Phase G after a local image drift rebuilt without it).
RUN apt-get update && apt-get install -y --no-install-recommends libgtk-3-dev && \
    rm -rf /var/lib/apt/lists/* && \
    rustup component add rustfmt clippy && \
    curl -LsSf https://get.nexte.st/0.9.133/linux | tar zxf - -C /usr/local/bin
WORKDIR /workspace
