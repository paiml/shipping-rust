# Criterion bench results — 1005a55

    host=mac-server
    os=Linux 6.8.0-107-generic x86_64
    cpu=Intel(R) Xeon(R) W-3245 CPU @ 3.20GHz
    cores=32
    rustc=rustc 1.95.0 (59807616e 2026-04-14)
    cargo=cargo 1.95.0 (f2d3ce0bd 2026-03-21)
    git_sha=1005a55
    git_ref=refs/heads/main
    ts=2026-04-28T14:36:22Z

## Throughput (rows/sec)

| Size | Mean | Std Dev | Throughput |
|------|------|---------|------------|
| 1000 | 163011 ns | 1372 ns | 6134555 rows/sec |
| 10000 | 1515682 ns | 40834 ns | 6597688 rows/sec |
| 100000 | 15159020 ns | 172458 ns | 6596732 rows/sec |
