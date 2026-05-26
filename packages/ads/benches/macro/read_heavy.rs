#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;

use std::collections::{BTreeMap, HashMap};

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use generators::{read_heavy_ops, uniform_keys};

fn read_heavy_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("macro_read_heavy_u64");

    for &size in &[5_000usize, 20_000usize] {
        group.throughput(Throughput::Elements(size as u64));

        let prefill = uniform_keys(size, 0x6000 + size as u64);
        let ops = read_heavy_ops(size, size, 0x6100 + size as u64);

        group.bench_with_input(
            BenchmarkId::new("mix/std_btreemap", size),
            &ops,
            |b, input| {
                b.iter_batched(
                    || &prefill,
                    |keys| common::map_mixed_ops_bench::<BTreeMap<u64, u64>>(keys, input),
                    BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("mix/std_hashmap", size),
            &ops,
            |b, input| {
                b.iter_batched(
                    || &prefill,
                    |keys| common::map_mixed_ops_bench::<HashMap<u64, u64>>(keys, input),
                    BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(BenchmarkId::new("mix/bst_safe", size), &ops, |b, input| {
            b.iter_batched(
                || &prefill,
                |keys| common::map_mixed_ops_bench::<common::BstSafe>(keys, input),
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("mix/bst_raw", size), &ops, |b, input| {
            b.iter_batched(
                || &prefill,
                |keys| common::map_mixed_ops_bench::<common::BstRaw>(keys, input),
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("mix/bst_arena", size), &ops, |b, input| {
            b.iter_batched(
                || &prefill,
                |keys| common::map_mixed_ops_bench::<common::BstArena>(keys, input),
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("mix/avl_safe", size), &ops, |b, input| {
            b.iter_batched(
                || &prefill,
                |keys| common::map_mixed_ops_bench::<common::AvlSafe>(keys, input),
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("mix/avl_raw", size), &ops, |b, input| {
            b.iter_batched(
                || &prefill,
                |keys| common::map_mixed_ops_bench::<common::AvlRaw>(keys, input),
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("mix/avl_arena", size), &ops, |b, input| {
            b.iter_batched(
                || &prefill,
                |keys| common::map_mixed_ops_bench::<common::AvlArena>(keys, input),
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("mix/rbt_safe", size), &ops, |b, input| {
            b.iter_batched(
                || &prefill,
                |keys| common::map_mixed_ops_bench::<common::RbSafe>(keys, input),
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("mix/rbt_raw", size), &ops, |b, input| {
            b.iter_batched(
                || &prefill,
                |keys| common::map_mixed_ops_bench::<common::RbRaw>(keys, input),
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("mix/rbt_arena", size), &ops, |b, input| {
            b.iter_batched(
                || &prefill,
                |keys| common::map_mixed_ops_bench::<common::RbArena>(keys, input),
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(
            BenchmarkId::new("mix/btree_safe_t8", size),
            &ops,
            |b, input| {
                b.iter_batched(
                    || &prefill,
                    |keys| common::map_mixed_ops_bench::<common::BtSafe>(keys, input),
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("mix/btree_raw_t8", size),
            &ops,
            |b, input| {
                b.iter_batched(
                    || &prefill,
                    |keys| common::map_mixed_ops_bench::<common::BtRaw>(keys, input),
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("mix/btree_arena_t8", size),
            &ops,
            |b, input| {
                b.iter_batched(
                    || &prefill,
                    |keys| common::map_mixed_ops_bench::<common::BtArena>(keys, input),
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("mix/splay_safe", size),
            &ops,
            |b, input| {
                b.iter_batched(
                    || &prefill,
                    |keys| common::map_mixed_ops_bench::<common::SplaySafe>(keys, input),
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("mix/splay_safe_adaptive", size),
            &ops,
            |b, input| {
                b.iter_batched(
                    || &prefill,
                    |keys| common::map_mixed_ops_adaptive_bench::<common::SplaySafe>(keys, input),
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(BenchmarkId::new("mix/splay_raw", size), &ops, |b, input| {
            b.iter_batched(
                || &prefill,
                |keys| common::map_mixed_ops_bench::<common::SplayRaw>(keys, input),
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(
            BenchmarkId::new("mix/splay_raw_adaptive", size),
            &ops,
            |b, input| {
                b.iter_batched(
                    || &prefill,
                    |keys| common::map_mixed_ops_adaptive_bench::<common::SplayRaw>(keys, input),
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("mix/splay_arena", size),
            &ops,
            |b, input| {
                b.iter_batched(
                    || &prefill,
                    |keys| common::map_mixed_ops_bench::<common::SplayArena>(keys, input),
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("mix/splay_arena_adaptive", size),
            &ops,
            |b, input| {
                b.iter_batched(
                    || &prefill,
                    |keys| common::map_mixed_ops_adaptive_bench::<common::SplayArena>(keys, input),
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(macro_read_heavy, read_heavy_benches);
criterion_main!(macro_read_heavy);
