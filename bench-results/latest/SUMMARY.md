# Criterion bench results — 5c24327

    host=mac-server
    os=Linux 6.8.0-111-generic x86_64
    cpu=Intel(R) Xeon(R) W-3245 CPU @ 3.20GHz
    cores=32
    rustc=rustc 1.95.0 (59807616e 2026-04-14)
    cargo=cargo 1.95.0 (f2d3ce0bd 2026-03-21)
    git_sha=5c24327
    git_ref=refs/heads/main
    ts=2026-05-24T07:00:46Z

## Throughput (rows/sec)

| Size | Mean | Std Dev | Throughput |
|------|------|---------|------------|
| 1000 | 306132 ns | 73978 ns | 3266562 rows/sec |
| 10000 | 3092721 ns | 237957 ns | 3233399 rows/sec |
| 100000 | 29846643 ns | 2807205 ns | 3350461 rows/sec |
