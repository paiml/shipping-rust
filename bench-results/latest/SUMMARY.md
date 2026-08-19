# Criterion bench results — dc90031

    host=mac-server
    os=Linux 6.8.0-136-generic x86_64
    cpu=Intel(R) Xeon(R) W-3245 CPU @ 3.20GHz
    cores=32
    rustc=rustc 1.95.0 (59807616e 2026-04-14)
    cargo=cargo 1.95.0 (f2d3ce0bd 2026-03-21)
    git_sha=dc90031
    git_ref=refs/heads/main
    ts=2026-08-19T20:08:56Z

## Throughput (rows/sec)

| Size | Mean | Std Dev | Throughput |
|------|------|---------|------------|
| 1000 | 502505 ns | 391385 ns | 1990031 rows/sec |
| 10000 | 4449777 ns | 1337101 ns | 2247304 rows/sec |
| 100000 | 35898832 ns | 11425467 ns | 2785606 rows/sec |
