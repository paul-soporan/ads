#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;
mod shared;

use iai_callgrind::{library_benchmark_group, main};
use shared::*;

library_benchmark_group!(
    name = sweep_hash_collisions_callgrind_group;
    benchmarks =
        callgrind_sweep_hash_collisions_insert_std_hashmap,
        callgrind_sweep_hash_collisions_insert_hashmap_zero_hasher,
        callgrind_sweep_hash_collisions_contains_temporal_hashmap_zero_hasher,
        callgrind_sweep_hash_collisions_contains_temporal_std_btreemap_reference
);

main!(library_benchmark_groups = sweep_hash_collisions_callgrind_group);
