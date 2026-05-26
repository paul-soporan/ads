#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;

use std::collections::{BTreeMap, HashMap};

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use generators::uniform_keys;

fn thrashing_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("macro_thrashing_u64");

    for &size in &[1_000usize, 10_000usize] {
        group.throughput(Throughput::Elements(size as u64));
        let prefill = uniform_keys(size, 0x9000 + size as u64);
        let remove_keys = uniform_keys(size, 0x9100 + size as u64);
        let mut insert_keys = uniform_keys(size * 2, 0x9200 + size as u64);
        insert_keys.truncate(size);
        for key in &mut insert_keys {
            *key = key.wrapping_add(size as u64);
        }

        group.bench_with_input(
            BenchmarkId::new("thrash/std_btreemap", size),
            &prefill,
            |b, prefill| {
                b.iter_batched(
                    || prefill.clone(),
                    |prefill| {
                        common::map_thrashing_bench::<BTreeMap<u64, u64>>(
                            &prefill,
                            &remove_keys,
                            &insert_keys,
                        )
                    },
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("thrash/std_hashmap", size),
            &prefill,
            |b, prefill| {
                b.iter_batched(
                    || prefill.clone(),
                    |prefill| {
                        common::map_thrashing_bench::<HashMap<u64, u64>>(
                            &prefill,
                            &remove_keys,
                            &insert_keys,
                        )
                    },
                    BatchSize::SmallInput,
                );
            },
        );

        macro_rules! bench_thrash {
            ($name:expr, $ty:ty) => {
                group.bench_with_input(BenchmarkId::new($name, size), &prefill, |b, prefill| {
                    b.iter_batched(
                        || prefill.clone(),
                        |prefill| {
                            common::map_thrashing_bench::<$ty>(&prefill, &remove_keys, &insert_keys)
                        },
                        BatchSize::SmallInput,
                    );
                });
            };
        }

        bench_thrash!("thrash/ads_bst_safe", common::BstSafe);
        bench_thrash!("thrash/ads_avl_safe", common::AvlSafe);
        bench_thrash!("thrash/ads_rbt_safe", common::RbSafe);
        bench_thrash!("thrash/ads_btree_safe_t8", common::BtSafe);
        bench_thrash!("thrash/ads_splay_safe", common::SplaySafe);

        bench_thrash!("thrash/ads_bst_raw", common::BstRaw);
        bench_thrash!("thrash/ads_avl_raw", common::AvlRaw);
        bench_thrash!("thrash/ads_rbt_raw", common::RbRaw);
        bench_thrash!("thrash/ads_btree_raw_t8", common::BtRaw);
        bench_thrash!("thrash/ads_splay_raw", common::SplayRaw);

        bench_thrash!("thrash/ads_bst_arena", common::BstArena);
        bench_thrash!("thrash/ads_avl_arena", common::AvlArena);
        bench_thrash!("thrash/ads_rbt_arena", common::RbArena);
        bench_thrash!("thrash/ads_btree_arena_t8", common::BtArena);
        bench_thrash!("thrash/ads_splay_arena", common::SplayArena);
    }

    group.finish();
}

criterion_group!(macro_thrashing, thrashing_benches);
criterion_main!(macro_thrashing);
