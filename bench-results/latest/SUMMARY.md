# Criterion bench results — 2c7f90c

    host=mac-server
    os=Linux 6.8.0-138-generic x86_64
    cpu=Intel(R) Xeon(R) W-3245 CPU @ 3.20GHz
    cores=32
    rustc=rustc 1.95.0 (59807616e 2026-04-14)
    cargo=cargo 1.95.0 (f2d3ce0bd 2026-03-21)
    git_sha=2c7f90c
    git_ref=refs/heads/main
    ts=2026-09-13T10:53:05Z

## Throughput (rows/sec)

| Size | Mean | Std Dev | Throughput |
|------|------|---------|------------|
| 1000 | 558826 ns | 141660 ns | 1789467 rows/sec |
| 10000 | 4643331 ns | 885668 ns | 2153626 rows/sec |
| 100000 | 72857303 ns | 28903514 ns | 1372546 rows/sec |
