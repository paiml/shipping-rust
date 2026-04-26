# Changelog

All notable changes to this project are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-04-26

Initial reference release. Companion workspace for course **c9 — Shipping Rust:
Cargo, CI, Benchmarks & Containers**.

### Added

- `etl-core` — typed CSV → JSON Lines pipeline (`run`) with row-aligned
  `Report` (rows in / rows out / rows rejected / errors_by_kind), explicit
  `EtlError` and `ErrorKind` enums, and an `AgeBucket` boundary at 18 / 65.
- `etl-cli` — `etl` binary that reads CSV from `--input` (path or `-`),
  writes JSON Lines to `--output` (path or `-`), and emits the report on
  stderr. Two provable contracts hold per run:
  - `ROWS_IN_EQUALS_ROWS_OUT` — `rows_in == rows_out + rows_rejected`.
  - `REPORT_JSON_ROUNDTRIPS` — the report serializes and parses back to
    an identical value.
- `etl-bench` — `synth_csv(n)` fixture generator used by criterion benches
  at 1k / 10k / 100k row sizes (`Throughput::Elements`).
- Workspace MSRV pinned at **1.85.0** via `rust-toolchain.toml`; release
  profile uses `lto = "fat"`, `panic = "abort"`, `strip = "symbols"`.
- 100% line coverage gate via `cargo llvm-cov --fail-under-lines 100`
  (37 tests across unit + integration).
- CI workflow (`.github/workflows/ci.yml`) — single `gate` job runs
  fmt / clippy / doc / test / coverage / audit / deny / release-build /
  binary-size budget / bench-smoke against MSRV and stable.
- Multi-stage `Dockerfile` (cargo-chef + musl + scratch) — final image
  contains only the `etl` binary and runs as user `65532`.
- Alternative `Dockerfile.distroless-cc` for glibc-linked workloads.
- Supply-chain hygiene — `cargo audit`, `cargo deny` (registry & git
  source allow-list, license allow-list, version-conflict warnings).
- Dual MIT / Apache-2.0 licensing (standard Rust ecosystem pattern).

### Quality gates (verified at 0.1.0)

- `cargo fmt --check` — clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean (with
  workspace-level `unsafe_code = "forbid"`, `unwrap_used = "warn"`,
  `panic = "warn"`, `pedantic` enabled).
- `cargo doc --workspace --no-deps` with `-D warnings` — clean.
- `cargo llvm-cov --workspace --fail-under-lines 100` — passes.
- `cargo audit --deny warnings` — clean.
- `cargo deny check` — clean.
- `pmat comply` — `Status: COMPLIANT`.
