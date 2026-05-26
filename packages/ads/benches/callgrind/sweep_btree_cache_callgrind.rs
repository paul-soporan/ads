#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;
mod shared;

use iai_callgrind::{library_benchmark_group, main};
use shared::*;

library_benchmark_group!(
    name = sweep_btree_cache_callgrind_group;
    benchmarks =
        callgrind_sweep_btree_cache_insert_btree_safe_t4,
        callgrind_sweep_btree_cache_insert_btree_safe_t16,
        callgrind_sweep_btree_cache_insert_btree_safe_t64,
        callgrind_sweep_btree_cache_insert_btree_raw_t4,
        callgrind_sweep_btree_cache_insert_btree_raw_t16,
        callgrind_sweep_btree_cache_insert_btree_raw_t64,
        callgrind_sweep_btree_cache_insert_btree_arena_t4,
        callgrind_sweep_btree_cache_insert_btree_arena_t16,
        callgrind_sweep_btree_cache_insert_btree_arena_t64,
        callgrind_sweep_btree_cache_contains_zipf_btree_safe_t4,
        callgrind_sweep_btree_cache_contains_zipf_btree_safe_t16,
        callgrind_sweep_btree_cache_contains_zipf_btree_safe_t64,
        callgrind_sweep_btree_cache_contains_zipf_btree_raw_t4,
        callgrind_sweep_btree_cache_contains_zipf_btree_raw_t16,
        callgrind_sweep_btree_cache_contains_zipf_btree_raw_t64,
        callgrind_sweep_btree_cache_contains_zipf_btree_arena_t4,
        callgrind_sweep_btree_cache_contains_zipf_btree_arena_t16,
        callgrind_sweep_btree_cache_contains_zipf_btree_arena_t64
);

main!(library_benchmark_groups = sweep_btree_cache_callgrind_group);
