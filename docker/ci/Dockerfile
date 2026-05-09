# Local dev helper — NOT the CI pipeline (see .woodpecker.yml).
# Pin to match Woodpecker CI's rust image version to prevent
# rustfmt drift between local and CI (cf. PATTERNS.md §P45).
FROM rust:1.94
RUN rustup component add rustfmt clippy && \
    curl -LsSf https://get.nexte.st/0.9.133/linux | tar zxf - -C /usr/local/bin
WORKDIR /workspace
