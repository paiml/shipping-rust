# Criterion bench results — 719f76c

```
host=noah-Lambda-Vector
os=Linux 6.8.0-90-generic x86_64
cpu=AMD Ryzen Threadripper 7960X 24-Cores
cores=48
rustc=rustc 1.95.0 (59807616e 2026-04-14)
cargo=cargo 1.95.0 (f2d3ce0bd 2026-03-21)
git_sha=719f76c
git_ref=refs/heads/feat/bench-workflow
ts=2026-04-28T10:58:41Z
trigger=local-seed-run
```

## Throughput (rows/sec)

| Size | Mean (ns) | Std Dev (ns) | Throughput |
|------|-----------|--------------|------------|
| 1000    | 111,057   | 4,348        | 9.00 Melem/s |
| 10000   | 1,025,980 | 27,510       | 9.75 Melem/s |
| 100000  | 10,375,079 | 286,954      | 9.64 Melem/s |

## How to read these results

Each row reports the mean wall-clock time per `etl_core::run` call
across the full input (1k / 10k / 100k synthetic CSV rows). The
throughput column is `size / mean` — what matters is that it stays
roughly constant across sizes (it does, ~9.6 Melem/s here) which
confirms the pipeline is **linear** in input size.

Per-call overhead (CSV reader init, allocation) is amortized away by
10k rows; the 1k size catches startup-cost regressions.

## Reproducing locally

```bash
make bench                    # full criterion suite (~30s)
make bench-smoke              # CI smoke mode (compile + run-once)
```

## Automated re-runs

[`.github/workflows/bench.yml`](../.github/workflows/bench.yml) runs the
full suite on a self-hosted `intel` runner (paiml org runner pool):

- on `workflow_dispatch` (manual)
- weekly via cron (Sundays 06:00 UTC)
- on push to `main` when `etl-core/`, `etl-bench/`, or `Cargo.{toml,lock}` change

Results are committed back to this directory by the workflow.
The HTML report is also available as a workflow artifact.
