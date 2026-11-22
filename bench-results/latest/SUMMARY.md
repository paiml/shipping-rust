# Criterion bench results — 6dd528f

    host=mac-server
    os=Linux 6.8.0-138-generic x86_64
    cpu=Intel(R) Xeon(R) W-3245 CPU @ 3.20GHz
    cores=32
    rustc=rustc 1.95.0 (59807616e 2026-04-14)
    cargo=cargo 1.95.0 (f2d3ce0bd 2026-03-21)
    git_sha=6dd528f
    git_ref=refs/heads/main
    ts=2026-11-22T13:50:00Z

## Throughput (rows/sec)

| Size | Mean | Std Dev | Throughput |
|------|------|---------|------------|
| 1000 | 173332 ns | 7307 ns | 5769281 rows/sec |
| 10000 | 1563843 ns | 28048 ns | 6394505 rows/sec |
| 100000 | 16116867 ns | 1006953 ns | 6204680 rows/sec |
