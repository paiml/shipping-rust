//! Throughput benchmarks for `etl_core::run`.
//!
//! Three sizes (1 000 / 10 000 / 100 000 rows) measured with
//! `Throughput::Elements` so criterion reports rows/sec. Each row is a
//! synthetic fruit measurement (`id,fruit,weight_g`).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use std::io::Cursor;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use etl_bench::synth_csv;
use etl_core::run;

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("etl_throughput");
    for &n in &[1_000usize, 10_000, 100_000] {
        let input = synth_csv(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &input, |b, input| {
            b.iter(|| {
                let mut sink = Vec::with_capacity(input.len());
                let report =
                    run(Cursor::new(input.as_slice()), &mut sink).expect("bench pipeline failed");
                debug_assert_eq!(report.rows_in, report.rows_out + report.rows_rejected);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_throughput);
criterion_main!(benches);
