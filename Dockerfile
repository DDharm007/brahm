# Multi-stage zero-Rust runtime image for running BRAHM cleanly anywhere Docker is available
FROM rust:1.83-slim-bookworm AS builder

WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends build-essential && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates git && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/brahm /usr/local/bin/brahm

WORKDIR /workspace
ENTRYPOINT ["brahm"]
CMD ["--help"]
