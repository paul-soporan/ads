#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;

use std::collections::{BTreeMap, HashMap};

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use generators::{temporal_locality_queries, uniform_keys};

fn colliding_insert(keys: &[u64]) -> usize {
    let mut map = common::colliding_hasher_map();
    for &key in keys {
        let _ = map.insert(black_box(key), black_box(key ^ 0xCAFE_BABE));
    }
    black_box(map.len())
}

fn colliding_contains(keys: &[u64], queries: &[u64]) -> usize {
    let mut map: common::CollidingHashMap<u64, u64> = common::colliding_hasher_map();
    for &key in keys {
        let _ = map.insert(key, key);
    }

    let mut hits = 0usize;
    for query in queries {
        if map.contains_key(black_box(query)) {
            hits = hits.wrapping_add(1);
        }
    }

    black_box(hits)
}

fn hash_collision_sweeps(c: &mut Criterion) {
    let mut group = c.benchmark_group("sweep_hash_collisions_u64");

    for &size in &[1_000usize, 10_000usize] {
        group.throughput(Throughput::Elements(size as u64));
        let keys = uniform_keys(size, 0x9000 + size as u64);
        let queries = temporal_locality_queries(size, size, 0x9100 + size as u64);

        group.bench_with_input(
            BenchmarkId::new("insert/std_hashmap", size),
            &keys,
            |b, input| {
                b.iter(|| common::map_insert_bench::<HashMap<u64, u64>>(input));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("insert/hashmap_zero_hasher", size),
            &keys,
            |b, input| b.iter(|| colliding_insert(input)),
        );

        group.bench_with_input(
            BenchmarkId::new("contains/hashmap_zero_hasher", size),
            &queries,
            |b, input| b.iter(|| colliding_contains(&keys, input)),
        );
        group.bench_with_input(
            BenchmarkId::new("contains/std_btreemap_reference", size),
            &queries,
            |b, input| b.iter(|| common::map_contains_bench::<BTreeMap<u64, u64>>(&keys, input)),
        );
    }

    group.finish();
}

criterion_group!(sweep_hash_collisions, hash_collision_sweeps);
criterion_main!(sweep_hash_collisions);
