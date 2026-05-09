# Local dev helper — NOT the CI pipeline (see .github/workflows/).
# Pin to match CI's dtolnay/rust-toolchain@stable at time of build.
FROM rust:1.95.0
RUN rustup component add rustfmt clippy && \
    curl -LsSf https://get.nexte.st/0.9.133/linux | tar zxf - -C /usr/local/bin
WORKDIR /workspace
