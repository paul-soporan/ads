#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use generators::uniform_keys;

fn heaps_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("micro_heaps_u64");

    for &size in &[2_000usize, 20_000usize, 100_000usize] {
        group.throughput(Throughput::Elements(size as u64));
        let keys = uniform_keys(size, 0x5000 + size as u64);

        group.bench_with_input(
            BenchmarkId::new("push_pop/std_binary_heap_reverse", size),
            &keys,
            |b, input| b.iter(|| common::heap_push_pop_bench::<common::StdMinHeap>(input)),
        );

        group.bench_with_input(
            BenchmarkId::new("push_pop/ads_binary_heap_safe", size),
            &keys,
            |b, input| b.iter(|| common::heap_push_pop_bench::<common::AdsHeap>(input)),
        );
    }

    group.finish();
}

criterion_group!(micro_heaps, heaps_benches);
criterion_main!(micro_heaps);