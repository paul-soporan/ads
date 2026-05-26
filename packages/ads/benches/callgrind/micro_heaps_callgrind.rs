#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;
mod shared;

use iai_callgrind::{library_benchmark_group, main};
use shared::*;

library_benchmark_group!(
    name = micro_heaps_callgrind_group;
    benchmarks =
        callgrind_micro_heaps_push_pop_std_binary_heap,
        callgrind_micro_heaps_push_pop_ads_binary_arena,
        callgrind_micro_heaps_push_pop_ads_binary_safe,
        callgrind_micro_heaps_push_pop_ads_binary_raw,
        callgrind_micro_heaps_push_pop_ads_binomial_arena,
        callgrind_micro_heaps_push_pop_ads_fibonacci_arena,
        callgrind_motivational_heap_merge_merge_ads_binary_arena,
        callgrind_motivational_heap_merge_merge_ads_binomial_arena,
        callgrind_motivational_heap_merge_merge_ads_fibonacci_arena
);

main!(library_benchmark_groups = micro_heaps_callgrind_group);
