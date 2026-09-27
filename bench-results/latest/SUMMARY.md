# Criterion bench results — 9e8a0ae

    host=mac-server
    os=Linux 6.8.0-138-generic x86_64
    cpu=Intel(R) Xeon(R) W-3245 CPU @ 3.20GHz
    cores=32
    rustc=rustc 1.95.0 (59807616e 2026-04-14)
    cargo=cargo 1.95.0 (f2d3ce0bd 2026-03-21)
    git_sha=9e8a0ae
    git_ref=refs/heads/main
    ts=2026-09-27T11:08:28Z

## Throughput (rows/sec)

| Size | Mean | Std Dev | Throughput |
|------|------|---------|------------|
| 1000 | 1801371 ns | 5642307 ns | 555133 rows/sec |
| 10000 | 9507470 ns | 21108142 ns | 1051804 rows/sec |
| 100000 | 137120414 ns | 179834061 ns | 729286 rows/sec |
