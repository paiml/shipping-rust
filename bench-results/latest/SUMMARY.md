# Criterion bench results — 5c24327

    host=mac-server
    os=Linux 6.8.0-111-generic x86_64
    cpu=Intel(R) Xeon(R) W-3245 CPU @ 3.20GHz
    cores=32
    rustc=rustc 1.95.0 (59807616e 2026-04-14)
    cargo=cargo 1.95.0 (f2d3ce0bd 2026-03-21)
    git_sha=5c24327
    git_ref=refs/heads/main
    ts=2026-05-17T06:52:50Z

## Throughput (rows/sec)

| Size | Mean | Std Dev | Throughput |
|------|------|---------|------------|
| 1000 | 166010 ns | 27833 ns | 6023721 rows/sec |
| 10000 | 1503362 ns | 169328 ns | 6651757 rows/sec |
| 100000 | 15910296 ns | 3459726 ns | 6285238 rows/sec |
