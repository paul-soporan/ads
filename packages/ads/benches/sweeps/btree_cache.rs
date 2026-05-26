#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use generators::{uniform_keys, zipfian_queries};

fn btree_cache_sweeps(c: &mut Criterion) {
    let mut group = c.benchmark_group("sweep_btree_cache_u64");

    for &size in &[1_000usize, 10_000usize] {
        group.throughput(Throughput::Elements(size as u64));
        let keys = uniform_keys(size, 0x8000 + size as u64);
        let queries = zipfian_queries(size, size, 0x8100 + size as u64);

        group.bench_with_input(BenchmarkId::new("insert/ads_btree_safe_t4", size), &keys, |b, input| {
            b.iter(|| common::map_insert_bench::<common::BtSafeDeg4>(input));
        });
        group.bench_with_input(BenchmarkId::new("insert/ads_btree_safe_t16", size), &keys, |b, input| {
            b.iter(|| common::map_insert_bench::<common::BtSafeDeg16>(input));
        });
        group.bench_with_input(BenchmarkId::new("insert/ads_btree_safe_t64", size), &keys, |b, input| {
            b.iter(|| common::map_insert_bench::<common::BtSafeDeg64>(input));
        });

        group.bench_with_input(BenchmarkId::new("insert/ads_btree_raw_t4", size), &keys, |b, input| {
            b.iter(|| common::map_insert_bench::<common::BtRawDeg4>(input));
        });
        group.bench_with_input(BenchmarkId::new("insert/ads_btree_raw_t16", size), &keys, |b, input| {
            b.iter(|| common::map_insert_bench::<common::BtRawDeg16>(input));
        });
        group.bench_with_input(BenchmarkId::new("insert/ads_btree_raw_t64", size), &keys, |b, input| {
            b.iter(|| common::map_insert_bench::<common::BtRawDeg64>(input));
        });

        group.bench_with_input(BenchmarkId::new("insert/ads_btree_arena_t4", size), &keys, |b, input| {
            b.iter(|| common::map_insert_bench::<common::BtArenaDeg4>(input));
        });
        group.bench_with_input(BenchmarkId::new("insert/ads_btree_arena_t16", size), &keys, |b, input| {
            b.iter(|| common::map_insert_bench::<common::BtArenaDeg16>(input));
        });
        group.bench_with_input(BenchmarkId::new("insert/ads_btree_arena_t64", size), &keys, |b, input| {
            b.iter(|| common::map_insert_bench::<common::BtArenaDeg64>(input));
        });

        group.bench_with_input(
            BenchmarkId::new("contains/ads_btree_safe_t4", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::BtSafeDeg4>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains/ads_btree_safe_t16", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::BtSafeDeg16>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains/ads_btree_safe_t64", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::BtSafeDeg64>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains/ads_btree_raw_t4", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::BtRawDeg4>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains/ads_btree_raw_t16", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::BtRawDeg16>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains/ads_btree_raw_t64", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::BtRawDeg64>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains/ads_btree_arena_t4", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::BtArenaDeg4>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains/ads_btree_arena_t16", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::BtArenaDeg16>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains/ads_btree_arena_t64", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::BtArenaDeg64>(&keys, input)),
        );
    }

    group.finish();
}

criterion_group!(sweep_btree_cache, btree_cache_sweeps);
criterion_main!(sweep_btree_cache);