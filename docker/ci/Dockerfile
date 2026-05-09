FROM rust:latest
RUN rustup component add rustfmt clippy && \
    curl -LsSf https://get.nexte.st/latest/linux | tar zxf - -C /usr/local/bin
WORKDIR /workspace
