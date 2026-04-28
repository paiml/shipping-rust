# shipping-rust — c9 Shipping Rust reference workspace.
#
# Idiomatic make targets that mirror what `gate` runs in CI. On a
# developer machine, converge on these — if `make ship-ready` is
# green, the workspace is mergeable.
#
# Linted by `bashrs lint Makefile` in CI (paiml/bashrs ≥ 6.65).

.SUFFIXES:
.DELETE_ON_ERROR:
.ONESHELL:

CARGO ?= cargo
RUSTDOCFLAGS ?= -D warnings
RUSTFLAGS ?= -D warnings

.PHONY: help build test test-doc lint fmt fmt-check doc coverage \
	audit deny release size bench bench-smoke contracts contracts-check \
	bashrs-lint comply docker docker-distroless clean ship-ready

help:
	@echo "shipping-rust — make targets"
	@echo ""
	@echo "  build          cargo build --workspace"
	@echo "  test           cargo test --workspace --all-targets"
	@echo "  test-doc       cargo test --workspace --doc"
	@echo "  fmt            cargo fmt --all"
	@echo "  fmt-check      cargo fmt --all -- --check (CI gate)"
	@echo "  lint           cargo clippy -D warnings"
	@echo "  doc            cargo doc with -D warnings"
	@echo "  coverage       cargo llvm-cov --fail-under-lines 100"
	@echo "  audit          cargo audit --deny warnings"
	@echo "  deny           cargo deny check"
	@echo "  release        cargo build --release --workspace"
	@echo "  size           print release etl binary size"
	@echo "  bench          cargo bench --workspace"
	@echo "  bench-smoke    cargo bench -- --test (no measurements)"
	@echo "  contracts-check  pv lint contracts/ — validate provable-contract YAML"
	@echo "  bashrs-lint    bashrs lint Makefile + Dockerfiles"
	@echo "  comply         pmat comply — paiml quality + compliance check"
	@echo "  docker         docker build (musl + scratch, <2 MB image)"
	@echo "  docker-distroless  docker build (distroless cc, glibc variant)"
	@echo "  ship-ready     full local CI gate (fmt-check, lint, doc, test, coverage, audit, deny, release, size, bench-smoke, contracts-check, bashrs-lint, comply)"
	@echo "  clean          cargo clean"

build:
	$(CARGO) build --workspace

test:
	$(CARGO) test --workspace --all-targets

test-doc:
	$(CARGO) test --workspace --doc

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

lint:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

doc:
	RUSTDOCFLAGS="$(RUSTDOCFLAGS)" $(CARGO) doc --workspace --no-deps

coverage:
	$(CARGO) llvm-cov --workspace --fail-under-lines 100

audit:
	$(CARGO) audit --deny warnings

deny:
	$(CARGO) deny check

release:
	$(CARGO) build --release --workspace

size: release
	@bin="target/release/etl"; \
	test -f "$$bin" || (echo "missing $$bin — run make release first" && exit 1); \
	stat -c "%n: %s bytes" "$$bin"

bench:
	$(CARGO) bench --workspace

bench-smoke:
	$(CARGO) bench --workspace -- --test

contracts-check:
	pv lint contracts

bashrs-lint:
	bashrs lint Makefile Dockerfile Dockerfile.distroless-cc

comply:
	pmat comply

docker:
	docker build -t shipping-rust:latest .

docker-distroless:
	docker build -f Dockerfile.distroless-cc -t shipping-rust:distroless .

ship-ready: fmt-check lint doc test test-doc coverage audit deny release size bench-smoke contracts-check bashrs-lint comply  ## the full CI gate, run on a developer machine
	@echo ""
	@echo "ship-ready: all gates green"

clean:
	$(CARGO) clean
