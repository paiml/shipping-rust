# Changelog

All notable changes to this project are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Breaking — example domain refactor.** The reference dataset is now fruit
  measurements instead of generic person records. CSV input columns are
  `id,fruit,weight_g` (previously `id,name,age`); output records carry a
  `size_bucket` of `Small` (<100 g), `Medium` (100–299 g), `Large` (≥300 g),
  or `Unknown` (missing weight) instead of an `age_bucket`.
  - Renamed `AgeBucket` → `SizeBucket`, `Record::name` → `Record::fruit`,
    `Record::age_bucket` → `Record::size_bucket`.
  - Renamed `ErrorKind::EmptyName` → `EmptyFruit`,
    `ErrorKind::InvalidAge` → `InvalidWeight`.
  - All test fixtures, bench inputs, README examples, and error messages
    refer to fruit / weight consistently — one domain runs through the
    whole workspace.
- **Container layering — plain multi-stage build.** Both `Dockerfile`
  and `Dockerfile.distroless-cc` use a plain multi-stage pattern that
  matches [`paiml/forjar`](https://github.com/paiml/forjar)'s own
  Dockerfile, with no external Rust build-cache helpers. Workspace
  manifests are copied before sources so `cargo fetch --locked` caches
  independently of source edits — stock Docker layer caching is
  sufficient at this workspace size.
- **Rust toolchain pin bumped 1.85.0 → 1.95.0** across
  `rust-toolchain.toml`, `[workspace.package].rust-version`, the CI
  matrix MSRV entry, both Dockerfiles' `rust:1.95-slim` digest, and
  the README MSRV badge + prose. Auto-applied four clippy 1.95
  diagnostics to keep `clippy -D warnings` green: `io_other_error`
  ×2 in `etl-core`, `map_unwrap_or` ×2 in `etl-cli`.

### Added

- `assets/hero.png` (1280×640) — repo hero image used by the GitHub
  social card and the README header. Drawn from SVG primitives in
  `assets/hero.svg`; converted with `cairosvg`.
- **Tag-driven release workflow** — `.github/workflows/release.yml`
  publishes prebuilt `etl` binaries to the GitHub Release page (4
  Linux targets: x86_64 / aarch64 × musl / gnu, each with a `.sha256`
  companion) and scratch + distroless container images to
  `ghcr.io/paiml/shipping-rust` (`:latest`, `:distroless`, `:vX.Y.Z`,
  `:vX.Y.Z-distroless`) on every `vX.Y.Z` tag. Aarch64 targets cross-
  compile through pinned `cross` v0.2.5; x86_64 targets are native.
  Container package is public — anonymous `docker pull` works without
  a GHCR login. No crates.io publish; shipping-rust is a teaching
  reference, not a published crate.
- **CI `gate` aggregator job** — collapses the strategy.matrix
  (`gate-matrix (1.95.0)` and `gate-matrix (stable)`) into a single
  status check named literally `gate`. The branch ruleset on `main`
  requires that exact name, which matrix expansion does not emit.
- **`pmat comply` CI gate (stable-only)** — paiml's quality + compliance
  check joins the existing `bashrs lint Makefile + Dockerfiles` and
  `pv lint contracts/` steps via the same prebuilt-CLI install loop
  (binaries downloaded from each tool's GitHub release rather than
  built from source).
- **README badges** — CI status, license (MIT OR Apache-2.0), MSRV,
  100% line coverage, and `<2 MB` container size; plus a Distribution
  / Releases section documenting the GitHub Release tarballs and
  GHCR image tags.
- **Local-CI parity** — `make ship-ready` now also runs `bashrs lint`
  and `pmat comply`, so a green local run matches the CI `gate`
  aggregator.
- **`.pmat-gates.toml`** — declarative shadow of the thresholds the
  `gate` job enforces (MSRV 1.95.0, 100% line coverage, clippy `-D
  warnings`, audit + deny, 8 MiB binary budget, criterion harness).
  Lifts `pmat repo-score` from 85.0/A- to 87.5/A- by closing the
  PMAT Compliance gap (2.5/5 → 5.0/5).
- **Standalone bench workflow** — `.github/workflows/bench.yml` runs
  the full criterion suite (warmup + 100 samples × 3 sizes) on a
  self-hosted `intel` runner from the paiml org runner pool.
  Triggered manually, weekly via cron, or on `main` pushes that
  touch `etl-core/` / `etl-bench/` / `Cargo.{toml,lock}`. Results
  are committed back into `bench-results/latest/` (SUMMARY.md +
  per-bench `estimates.json` + criterion HTML report). Self-hosted
  runner picked for measurement quality — bare-metal CV stays under
  1% vs 5-15% on shared GitHub-hosted VMs.
- **`bench-results/`** — committed criterion output as a teaching
  artifact. Seed numbers (Threadripper 7960X, ~9.6M rows/sec across
  all three sizes) plus a README explaining what's tracked, why
  benchmark output belongs in the repo, and how to read the JSON
  estimates. Workflow runs overwrite `latest/`; git history
  preserves drift.
- **README throughput badge + Benchmarks section** — links to
  `bench-results/latest/SUMMARY.md` and explains the smoke-vs-full
  split between `gate` and `bench` workflows.

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
- Multi-stage `Dockerfile` (plain multi-stage, musl + scratch) — final
  image contains only the `etl` binary and runs as user `65532`.
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
