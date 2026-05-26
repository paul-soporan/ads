#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;

use std::collections::{BTreeMap, HashMap};

use criterion::{BenchmarkId, Criterion, Throughput};
use generators::{short_strings, uniform_keys, zipfian_queries};

pub fn bench_u64_maps(c: &mut Criterion) {
    let mut group = c.benchmark_group("micro_maps_u64");

    for &size in &[1_000usize, 10_000usize] {
        group.throughput(Throughput::Elements(size as u64));
        let keys = uniform_keys(size, 0x1000 + size as u64);
        let queries = zipfian_queries(size, size, 0x2000 + size as u64);

        group.bench_with_input(
            BenchmarkId::new("insert/std_btreemap", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<BTreeMap<u64, u64>>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/std_hashmap", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<HashMap<u64, u64>>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/bst_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<common::BstSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/bst_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<common::BstRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/bst_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<common::BstArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/avl_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<common::AvlSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/avl_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<common::AvlRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/avl_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<common::AvlArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/rbt_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<common::RbSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/rbt_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<common::RbRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/rbt_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<common::RbArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/btree_safe_t8", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<common::BtSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/btree_raw_t8", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<common::BtRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/btree_arena_t8", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<common::BtArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/splay_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<common::SplaySafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/splay_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<common::SplayRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/splay_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<common::SplayArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/skip_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<common::SkipSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/skip_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<common::SkipRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/skip_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<common::SkipArena>(input));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("contains_zipf/std_btreemap", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<BTreeMap<u64, u64>>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/std_hashmap", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<HashMap<u64, u64>>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/bst_safe", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::BstSafe>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/bst_raw", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::BstRaw>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/bst_arena", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::BstArena>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/avl_safe", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::AvlSafe>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/avl_raw", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::AvlRaw>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/avl_arena", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::AvlArena>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/rbt_safe", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::RbSafe>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/rbt_raw", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::RbRaw>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/rbt_arena", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::RbArena>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/btree_safe_t8", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::BtSafe>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/btree_raw_t8", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::BtRaw>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/btree_arena_t8", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::BtArena>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/splay_safe", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::SplaySafe>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/splay_safe_adaptive", size),
            &queries,
            |b, input| {
                b.iter(|| common::map_contains_adaptive_bench::<common::SplaySafe>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/splay_raw", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::SplayRaw>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/splay_raw_adaptive", size),
            &queries,
            |b, input| {
                b.iter(|| common::map_contains_adaptive_bench::<common::SplayRaw>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/splay_arena", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::SplayArena>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/splay_arena_adaptive", size),
            &queries,
            |b, input| {
                b.iter(|| common::map_contains_adaptive_bench::<common::SplayArena>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/skip_safe", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::SkipSafe>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/skip_raw", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::SkipRaw>(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/skip_arena", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<common::SkipArena>(&keys, input)),
        );

        group.bench_with_input(
            BenchmarkId::new("remove/std_btreemap", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<BTreeMap<u64, u64>>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/std_hashmap", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<HashMap<u64, u64>>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/bst_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<common::BstSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/bst_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<common::BstRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/bst_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<common::BstArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/avl_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<common::AvlSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/avl_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<common::AvlRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/avl_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<common::AvlArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/rbt_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<common::RbSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/rbt_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<common::RbRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/rbt_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<common::RbArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/btree_safe_t8", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<common::BtSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/btree_raw_t8", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<common::BtRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/btree_arena_t8", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<common::BtArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/splay_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<common::SplaySafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/splay_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<common::SplayRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/splay_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<common::SplayArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/skip_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<common::SkipSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/skip_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<common::SkipRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("remove/skip_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_remove_bench::<common::SkipArena>(input));
            },
        );
    }

    group.finish();
}

pub fn bench_string_maps(c: &mut Criterion) {
    let mut group = c.benchmark_group("micro_maps_strings");

    for &size in &[1_000usize, 10_000usize] {
        group.throughput(Throughput::Elements(size as u64));
        let keys = short_strings(size, 0x3000 + size as u64);
        let mut queries = short_strings(size, 0x3100 + size as u64);
        queries.extend(keys.iter().take(size / 8).cloned());

        group.bench_with_input(
            BenchmarkId::new("insert/std_btreemap", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<BTreeMap<String, usize>>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/std_hashmap", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<HashMap<String, usize>>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/btree_safe_t8", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<common::StrBtSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/btree_raw_t8", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<common::StrBtRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/btree_arena_t8", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<common::StrBtArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/bst_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<common::StrBstSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/bst_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<common::StrBstRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/bst_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<common::StrBstArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/avl_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<common::StrAvlSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/avl_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<common::StrAvlRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/avl_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<common::StrAvlArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/rbt_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<common::StrRbSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/rbt_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<common::StrRbRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/rbt_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<common::StrRbArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/splay_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<common::StrSplaySafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/splay_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<common::StrSplayRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/splay_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<common::StrSplayArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/skip_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<common::StrSkipSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/skip_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<common::StrSkipRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/skip_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::string_map_insert_bench::<common::StrSkipArena>(input));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("contains_mixed/std_btreemap", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::string_map_contains_bench::<BTreeMap<String, usize>>(&keys, input)
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/std_hashmap", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<HashMap<String, usize>>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/bst_safe", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<common::StrBstSafe>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/bst_raw", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<common::StrBstRaw>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/bst_arena", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<common::StrBstArena>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/avl_safe", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<common::StrAvlSafe>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/avl_raw", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<common::StrAvlRaw>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/avl_arena", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<common::StrAvlArena>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/rbt_safe", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<common::StrRbSafe>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/rbt_raw", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<common::StrRbRaw>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/rbt_arena", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<common::StrRbArena>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/btree_safe_t8", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<common::StrBtSafe>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/btree_raw_t8", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<common::StrBtRaw>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/btree_arena_t8", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<common::StrBtArena>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/splay_safe", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<common::StrSplaySafe>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/splay_safe_adaptive", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::string_map_contains_adaptive_bench::<common::StrSplaySafe>(&keys, input)
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/splay_raw", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<common::StrSplayRaw>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/splay_raw_adaptive", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::string_map_contains_adaptive_bench::<common::StrSplayRaw>(&keys, input)
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/splay_arena", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<common::StrSplayArena>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/splay_arena_adaptive", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::string_map_contains_adaptive_bench::<common::StrSplayArena>(
                        &keys, input,
                    )
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/skip_safe", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<common::StrSkipSafe>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/skip_raw", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<common::StrSkipRaw>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_mixed/skip_arena", size),
            &queries,
            |b, input| {
                b.iter(|| common::string_map_contains_bench::<common::StrSkipArena>(&keys, input))
            },
        );
    }

    group.finish();
}

pub fn bench_large_payload_maps(c: &mut Criterion) {
    let mut group = c.benchmark_group("micro_maps_large_payload");

    for &size in &[500usize, 5_000usize] {
        group.throughput(Throughput::Elements(size as u64));
        let keys = uniform_keys(size, 0x4000 + size as u64);
        let queries = zipfian_queries(size, size, 0x4100 + size as u64);

        group.bench_with_input(
            BenchmarkId::new("insert/std_btreemap", size),
            &keys,
            |b, input| {
                b.iter(|| {
                    common::payload_map_insert_bench::<BTreeMap<u64, generators::LargePayload>>(
                        input,
                    )
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/std_hashmap", size),
            &keys,
            |b, input| {
                b.iter(|| {
                    common::payload_map_insert_bench::<HashMap<u64, generators::LargePayload>>(
                        input,
                    )
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/btree_safe_t8", size),
            &keys,
            |b, input| {
                b.iter(|| common::payload_map_insert_bench::<common::PayloadBtSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/btree_raw_t8", size),
            &keys,
            |b, input| {
                b.iter(|| common::payload_map_insert_bench::<common::PayloadBtRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/btree_arena_t8", size),
            &keys,
            |b, input| {
                b.iter(|| common::payload_map_insert_bench::<common::PayloadBtArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/bst_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::payload_map_insert_bench::<common::PayloadBstSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/bst_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::payload_map_insert_bench::<common::PayloadBstRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/bst_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::payload_map_insert_bench::<common::PayloadBstArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/avl_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::payload_map_insert_bench::<common::PayloadAvlSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/avl_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::payload_map_insert_bench::<common::PayloadAvlRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/avl_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::payload_map_insert_bench::<common::PayloadAvlArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/rbt_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::payload_map_insert_bench::<common::PayloadRbSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/rbt_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::payload_map_insert_bench::<common::PayloadRbRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/rbt_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::payload_map_insert_bench::<common::PayloadRbArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/splay_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::payload_map_insert_bench::<common::PayloadSplaySafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/splay_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::payload_map_insert_bench::<common::PayloadSplayRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/splay_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::payload_map_insert_bench::<common::PayloadSplayArena>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/skip_safe", size),
            &keys,
            |b, input| {
                b.iter(|| common::payload_map_insert_bench::<common::PayloadSkipSafe>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/skip_raw", size),
            &keys,
            |b, input| {
                b.iter(|| common::payload_map_insert_bench::<common::PayloadSkipRaw>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/skip_arena", size),
            &keys,
            |b, input| {
                b.iter(|| common::payload_map_insert_bench::<common::PayloadSkipArena>(input));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("contains_zipf/std_btreemap", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::payload_map_contains_bench::<BTreeMap<u64, generators::LargePayload>>(
                        &keys, input,
                    )
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/std_hashmap", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::payload_map_contains_bench::<HashMap<u64, generators::LargePayload>>(
                        &keys, input,
                    )
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/bst_safe", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::payload_map_contains_bench::<common::PayloadBstSafe>(&keys, input)
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/bst_raw", size),
            &queries,
            |b, input| {
                b.iter(|| common::payload_map_contains_bench::<common::PayloadBstRaw>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/bst_arena", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::payload_map_contains_bench::<common::PayloadBstArena>(&keys, input)
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/avl_safe", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::payload_map_contains_bench::<common::PayloadAvlSafe>(&keys, input)
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/avl_raw", size),
            &queries,
            |b, input| {
                b.iter(|| common::payload_map_contains_bench::<common::PayloadAvlRaw>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/avl_arena", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::payload_map_contains_bench::<common::PayloadAvlArena>(&keys, input)
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/rbt_safe", size),
            &queries,
            |b, input| {
                b.iter(|| common::payload_map_contains_bench::<common::PayloadRbSafe>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/rbt_raw", size),
            &queries,
            |b, input| {
                b.iter(|| common::payload_map_contains_bench::<common::PayloadRbRaw>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/rbt_arena", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::payload_map_contains_bench::<common::PayloadRbArena>(&keys, input)
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/btree_safe_t8", size),
            &queries,
            |b, input| {
                b.iter(|| common::payload_map_contains_bench::<common::PayloadBtSafe>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/splay_safe", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::payload_map_contains_bench::<common::PayloadSplaySafe>(&keys, input)
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/splay_safe_adaptive", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::payload_map_contains_adaptive_bench::<common::PayloadSplaySafe>(
                        &keys, input,
                    )
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/splay_raw", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::payload_map_contains_bench::<common::PayloadSplayRaw>(&keys, input)
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/splay_raw_adaptive", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::payload_map_contains_adaptive_bench::<common::PayloadSplayRaw>(
                        &keys, input,
                    )
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/splay_arena", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::payload_map_contains_bench::<common::PayloadSplayArena>(&keys, input)
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/splay_arena_adaptive", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::payload_map_contains_adaptive_bench::<common::PayloadSplayArena>(
                        &keys, input,
                    )
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/btree_raw_t8", size),
            &queries,
            |b, input| {
                b.iter(|| common::payload_map_contains_bench::<common::PayloadBtRaw>(&keys, input))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/btree_arena_t8", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::payload_map_contains_bench::<common::PayloadBtArena>(&keys, input)
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/skip_safe", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::payload_map_contains_bench::<common::PayloadSkipSafe>(&keys, input)
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/skip_raw", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::payload_map_contains_bench::<common::PayloadSkipRaw>(&keys, input)
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("contains_zipf/skip_arena", size),
            &queries,
            |b, input| {
                b.iter(|| {
                    common::payload_map_contains_bench::<common::PayloadSkipArena>(&keys, input)
                })
            },
        );
    }

    group.finish();
}
