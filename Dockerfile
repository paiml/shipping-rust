# syntax=docker/dockerfile:1.7
#
# Reference container for the `etl` CLI from course c9 "Shipping Rust".
#
# We deliberately avoid cargo-chef here. The forjar project (paiml's IaC
# tool, https://github.com/paiml/forjar) ships a plain multi-stage
# Dockerfile and demonstrates that for a small workspace the layer
# savings from chef are not worth the extra dependency. We rely on
# Docker's stock layer cache: workspace manifests copy first so when
# only sources change Docker can reuse the manifest layers, and the
# `cargo build` step's own layer is reused as long as none of the COPY
# inputs above it have changed.
#
# Two stages:
#   1. `builder` — rust:slim + musl toolchain. Compiling against
#                  `x86_64-unknown-linux-musl` produces a single static
#                  binary suitable for a `scratch` final image.
#   2. `runtime` — `FROM scratch`, copies in only the `etl` binary, runs
#                  as user 65532.
#
# The result is a fully static, zero-distro container under 2 MB. There is
# no shell, no libc, no package manager — only the binary and what cargo
# linked into it.

ARG RUST_VERSION=1.85
ARG TARGET=x86_64-unknown-linux-musl

# Stage 1 — build the static musl binary.
FROM rust:${RUST_VERSION}-slim AS builder
ARG TARGET
ENV CARGO_NET_RETRY=10 \
    CARGO_TERM_COLOR=always
RUN rustup target add ${TARGET} \
 && apt-get update \
 && apt-get install -y --no-install-recommends musl-tools ca-certificates \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /build
# Copy workspace manifests + sources together. Docker's layer cache reuses
# these COPY layers (and the cargo build layer below them) as long as the
# inputs are unchanged — no cargo-chef helper required.
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY etl-core etl-core
COPY etl-cli etl-cli
COPY etl-bench etl-bench
RUN cargo build --release --target ${TARGET} --bin etl --locked
RUN strip target/${TARGET}/release/etl

# Stage 2 — minimal runtime image. No shell, no libc, no package manager.
FROM scratch AS runtime
ARG TARGET
LABEL org.opencontainers.image.source="https://github.com/paiml/shipping-rust" \
      org.opencontainers.image.description="Reference etl CLI for course c9 (Shipping Rust)" \
      org.opencontainers.image.licenses="MIT OR Apache-2.0"
COPY --from=builder /build/target/${TARGET}/release/etl /etl
USER 65532:65532
ENTRYPOINT ["/etl"]
