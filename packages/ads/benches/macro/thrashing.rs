#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;

use std::collections::{BTreeMap, HashMap};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use generators::uniform_keys;

fn thrashing_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("macro_thrashing_u64");

    for &size in &[2_000usize, 20_000usize] {
        group.throughput(Throughput::Elements(size as u64));

        let prefill = uniform_keys(size, 0x7000 + size as u64);
        let remove_keys = uniform_keys(size, 0x7100 + size as u64);
        let mut insert_keys = uniform_keys(size * 2, 0x7200 + size as u64);
        insert_keys.truncate(size);
        for key in &mut insert_keys {
            *key = key.wrapping_add(size as u64);
        }

        group.bench_function(BenchmarkId::new("thrash/std_btreemap", size), |b| {
            b.iter(|| {
                common::map_thrashing_bench::<BTreeMap<u64, u64>>(
                    &prefill,
                    &remove_keys,
                    &insert_keys,
                )
            })
        });
        group.bench_function(BenchmarkId::new("thrash/std_hashmap", size), |b| {
            b.iter(|| {
                common::map_thrashing_bench::<HashMap<u64, u64>>(
                    &prefill,
                    &remove_keys,
                    &insert_keys,
                )
            })
        });
        group.bench_function(BenchmarkId::new("thrash/bst_safe", size), |b| {
            b.iter(|| {
                common::map_thrashing_bench::<common::BstSafe>(&prefill, &remove_keys, &insert_keys)
            })
        });
        group.bench_function(BenchmarkId::new("thrash/bst_raw", size), |b| {
            b.iter(|| {
                common::map_thrashing_bench::<common::BstRaw>(&prefill, &remove_keys, &insert_keys)
            })
        });
        group.bench_function(BenchmarkId::new("thrash/bst_arena", size), |b| {
            b.iter(|| {
                common::map_thrashing_bench::<common::BstArena>(
                    &prefill,
                    &remove_keys,
                    &insert_keys,
                )
            })
        });
        group.bench_function(BenchmarkId::new("thrash/avl_safe", size), |b| {
            b.iter(|| {
                common::map_thrashing_bench::<common::AvlSafe>(&prefill, &remove_keys, &insert_keys)
            })
        });
        group.bench_function(BenchmarkId::new("thrash/avl_raw", size), |b| {
            b.iter(|| {
                common::map_thrashing_bench::<common::AvlRaw>(&prefill, &remove_keys, &insert_keys)
            })
        });
        group.bench_function(BenchmarkId::new("thrash/avl_arena", size), |b| {
            b.iter(|| {
                common::map_thrashing_bench::<common::AvlArena>(
                    &prefill,
                    &remove_keys,
                    &insert_keys,
                )
            })
        });
        group.bench_function(BenchmarkId::new("thrash/rbt_safe", size), |b| {
            b.iter(|| {
                common::map_thrashing_bench::<common::RbSafe>(&prefill, &remove_keys, &insert_keys)
            })
        });
        group.bench_function(BenchmarkId::new("thrash/rbt_raw", size), |b| {
            b.iter(|| {
                common::map_thrashing_bench::<common::RbRaw>(&prefill, &remove_keys, &insert_keys)
            })
        });
        group.bench_function(BenchmarkId::new("thrash/rbt_arena", size), |b| {
            b.iter(|| {
                common::map_thrashing_bench::<common::RbArena>(&prefill, &remove_keys, &insert_keys)
            })
        });
        group.bench_function(BenchmarkId::new("thrash/btree_safe_t8", size), |b| {
            b.iter(|| {
                common::map_thrashing_bench::<common::BtSafe>(&prefill, &remove_keys, &insert_keys)
            })
        });
        group.bench_function(BenchmarkId::new("thrash/btree_raw_t8", size), |b| {
            b.iter(|| {
                common::map_thrashing_bench::<common::BtRaw>(&prefill, &remove_keys, &insert_keys)
            })
        });
        group.bench_function(BenchmarkId::new("thrash/btree_arena_t8", size), |b| {
            b.iter(|| {
                common::map_thrashing_bench::<common::BtArena>(&prefill, &remove_keys, &insert_keys)
            })
        });
        group.bench_function(BenchmarkId::new("thrash/splay_safe", size), |b| {
            b.iter(|| {
                common::map_thrashing_bench::<common::SplaySafe>(
                    &prefill,
                    &remove_keys,
                    &insert_keys,
                )
            })
        });
        group.bench_function(BenchmarkId::new("thrash/splay_raw", size), |b| {
            b.iter(|| {
                common::map_thrashing_bench::<common::SplayRaw>(
                    &prefill,
                    &remove_keys,
                    &insert_keys,
                )
            })
        });
        group.bench_function(BenchmarkId::new("thrash/splay_arena", size), |b| {
            b.iter(|| {
                common::map_thrashing_bench::<common::SplayArena>(
                    &prefill,
                    &remove_keys,
                    &insert_keys,
                )
            })
        });
    }

    group.finish();
}

criterion_group!(macro_thrashing, thrashing_benches);
criterion_main!(macro_thrashing);
