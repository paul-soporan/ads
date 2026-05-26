#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;

use ads::traits::core::PriorityQueue;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use generators::uniform_keys;

fn heaps_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("micro_heaps_u64");

    for &size in &[1_000usize, 10_000usize] {
        group.throughput(Throughput::Elements(size as u64));
        let keys = uniform_keys(size, 0x5000 + size as u64);

        group.bench_with_input(
            BenchmarkId::new("push_pop/std_binary_heap", size),
            &keys,
            |b, input| b.iter(|| common::heap_push_pop_bench::<common::StdBinaryHeapMin>(input)),
        );

        group.bench_with_input(
            BenchmarkId::new("push_pop/ads_binary_arena", size),
            &keys,
            |b, input| b.iter(|| common::heap_push_pop_bench::<common::BinaryArena>(input)),
        );

        group.bench_with_input(
            BenchmarkId::new("push_pop/ads_binary_safe", size),
            &keys,
            |b, input| b.iter(|| common::heap_push_pop_bench::<common::BinarySafe>(input)),
        );

        group.bench_with_input(
            BenchmarkId::new("push_pop/ads_binary_raw", size),
            &keys,
            |b, input| b.iter(|| common::heap_push_pop_bench::<common::BinaryRaw>(input)),
        );

        group.bench_with_input(
            BenchmarkId::new("push_pop/ads_binomial_arena", size),
            &keys,
            |b, input| b.iter(|| common::heap_push_pop_bench::<common::BinomialArena>(input)),
        );

        group.bench_with_input(
            BenchmarkId::new("push_pop/ads_fibonacci_arena", size),
            &keys,
            |b, input| b.iter(|| common::heap_push_pop_bench::<common::FibonacciArena>(input)),
        );
    }

    group.finish();
}

fn heap_merge_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("motivational_heap_merge_u64");

    for &size in &[1_000usize, 5_000usize] {
        group.throughput(Throughput::Elements(size as u64 * 2));
        let keys1 = uniform_keys(size, 0x1000 + size as u64);
        let keys2 = uniform_keys(size, 0x2000 + size as u64);

        macro_rules! bench_merge {
            ($name:expr, $ty:ty) => {
                group.bench_function(BenchmarkId::new($name, size), |b| {
                    b.iter(|| {
                        let mut h1 = <$ty>::from_iter(keys1.iter().cloned());
                        let mut h2 = <$ty>::from_iter(keys2.iter().cloned());
                        h1.merge(&mut h2);
                        criterion::black_box(h1.len())
                    })
                });
            }
        }

        bench_merge!("merge/ads_binary_arena", common::BinaryArena);
        bench_merge!("merge/ads_binomial_arena", common::BinomialArena);
        bench_merge!("merge/ads_fibonacci_arena", common::FibonacciArena);
    }

    group.finish();
}

criterion_group!(micro_heaps, heaps_benches, heap_merge_benches);
criterion_main!(micro_heaps);
