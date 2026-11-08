# Criterion bench results — 9e8a0ae

    host=mac-server
    os=Linux 6.8.0-138-generic x86_64
    cpu=Intel(R) Xeon(R) W-3245 CPU @ 3.20GHz
    cores=32
    rustc=rustc 1.95.0 (59807616e 2026-04-14)
    cargo=cargo 1.95.0 (f2d3ce0bd 2026-03-21)
    git_sha=9e8a0ae
    git_ref=refs/heads/main
    ts=2026-11-08T12:46:20Z

## Throughput (rows/sec)

| Size | Mean | Std Dev | Throughput |
|------|------|---------|------------|
| 1000 | 174775 ns | 3917 ns | 5721628 rows/sec |
| 10000 | 1523821 ns | 39569 ns | 6562450 rows/sec |
| 100000 | 15041456 ns | 432925 ns | 6648292 rows/sec |
