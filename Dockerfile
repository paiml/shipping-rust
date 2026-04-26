# syntax=docker/dockerfile:1.7
#
# Reference container for the `etl` CLI from course c9 "Shipping Rust".
#
# Three stages:
#   1. `chef`    — cargo-chef + musl toolchain image used for both planning
#                   and building. Compiling against musl + targeting
#                   `x86_64-unknown-linux-musl` lets us copy a single static
#                   binary into a `scratch` final image.
#   2. `planner` — emit a `recipe.json` describing the workspace's
#                   dependency tree so dep compilation can be cached
#                   independently of source edits.
#   3. `builder` — `cargo chef cook` the recipe (cached deps), then build
#                   the actual workspace.
#   4. `runtime` — `FROM scratch`, copy in only the `etl` binary, set
#                   ENTRYPOINT.
#
# The result is a fully static, zero-distro container at <6 MB. There is
# no shell, no libc, no package manager — only the binary and what cargo
# linked into it.

ARG RUST_VERSION=1.85
ARG TARGET=x86_64-unknown-linux-musl

FROM rust:${RUST_VERSION}-slim AS chef
ARG TARGET
ENV CARGO_NET_RETRY=10 \
    CARGO_TERM_COLOR=always
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
        musl-tools \
        ca-certificates \
 && rm -rf /var/lib/apt/lists/*
RUN rustup target add ${TARGET}
RUN cargo install --locked cargo-chef@0.1.68
WORKDIR /work

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG TARGET
COPY --from=planner /work/recipe.json recipe.json
# Cook dependencies. This layer is cached unless Cargo.toml/Cargo.lock change.
RUN cargo chef cook \
        --release \
        --target ${TARGET} \
        --recipe-path recipe.json
COPY . .
RUN cargo build \
        --release \
        --target ${TARGET} \
        --bin etl
# The release profile already strips symbols, but run `strip` once more for
# good measure on the static target.
RUN strip /work/target/${TARGET}/release/etl

FROM scratch AS runtime
ARG TARGET
LABEL org.opencontainers.image.source="https://github.com/paiml/shipping-rust" \
      org.opencontainers.image.description="Reference etl CLI for course c9 (Shipping Rust)" \
      org.opencontainers.image.licenses="MIT OR Apache-2.0"
COPY --from=builder /work/target/${TARGET}/release/etl /etl
USER 65532:65532
ENTRYPOINT ["/etl"]
