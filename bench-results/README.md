# bench-results

Criterion benchmark output for the `etl-core` pipeline, committed
back to the repo by [`.github/workflows/bench.yml`](../.github/workflows/bench.yml).

## Why commit benchmark output?

Two reasons.

**1. Teaching artifact.** This repo is the companion to course **c9 —
Shipping Rust**. A learner reading the README should be able to click
through to actual numbers without having to clone, install criterion,
and run benches themselves. The committed SUMMARY.md is the lowest
friction path to "is this fast?".

**2. Regression signal.** Each automated run overwrites
[`latest/`](latest/), but git history preserves prior runs. A
multi-month drift on `etl_throughput/100000` is a real signal that
something in the pipeline (or its dependencies) regressed.

## What's here

```
bench-results/
├── README.md          ← this file
└── latest/
    ├── SUMMARY.md     ← human-readable rows/sec table
    ├── meta.txt       ← host, CPU, rustc, git SHA, timestamp
    ├── etl_throughput/
    │   ├── 1000/estimates.json
    │   ├── 10000/estimates.json
    │   └── 100000/estimates.json
    └── report/
        └── index.html  ← criterion's HTML report (open in a browser)
```

The `*.json` files are the canonical numbers — `mean`, `median`,
`std_dev`, `slope`, `median_abs_dev`, each with a 95% confidence
interval. The HTML report renders those same numbers as charts.

The full per-sample data + violin plots stay in the workflow
artifact (30-day retention) — committing them would bloat the repo
without much teaching value. If you need them, grab the
`criterion-report-<sha>` artifact from the bench workflow run.

## What's not here

- **No per-sample raw data** (`new/sample.json`, `*.svg`) — too large.
- **No baseline comparisons** (`base/` dirs from criterion) — those
  exist on the runner during a run but aren't committed. The
  workflow uses `--save-baseline main` so locally you can run
  `cargo bench -- --baseline main` after pulling to see your delta.

## Reading the JSON

```bash
jq '.mean.point_estimate' bench-results/latest/etl_throughput/100000/estimates.json
# 10375079.43   ← nanoseconds for 100k rows
# 100000 / 10375079e-9 ≈ 9.64M rows/sec
```

## Triggering a fresh run

```bash
gh workflow run bench.yml -R paiml/shipping-rust
```

Runs on the next available `[self-hosted, intel]` runner from the
paiml org pool. Takes ~3-5 minutes wall-clock end-to-end (full
warmup + 100 samples × 3 sizes + report rendering + commit).

## Why a self-hosted runner?

Criterion's statistical model assumes low-variance measurements.
GitHub-hosted runners share VMs and the coefficient of variation
(CV) is typically 5-15%, which makes it hard to detect regressions
smaller than ~10%. On a dedicated bare-metal box, CV stays under
1% — so a 2% regression is a real signal, not noise.

This repo is small enough that the cost difference is negligible
either way; the choice is purely about signal quality.
