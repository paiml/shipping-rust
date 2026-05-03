# Criterion bench results — 5c24327

    host=mac-server
    os=Linux 6.8.0-107-generic x86_64
    cpu=Intel(R) Xeon(R) W-3245 CPU @ 3.20GHz
    cores=32
    rustc=rustc 1.95.0 (59807616e 2026-04-14)
    cargo=cargo 1.95.0 (f2d3ce0bd 2026-03-21)
    git_sha=5c24327
    git_ref=refs/heads/main
    ts=2026-05-03T06:47:18Z

## Throughput (rows/sec)

| Size | Mean | Std Dev | Throughput |
|------|------|---------|------------|
| 1000 | 185810 ns | 27136 ns | 5381832 rows/sec |
| 10000 | 1724128 ns | 324166 ns | 5800033 rows/sec |
| 100000 | 16481757 ns | 469772 ns | 6067314 rows/sec |
