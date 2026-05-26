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
        callgrind_micro_heaps_push_pop_std_binary_heap_reverse,
        callgrind_micro_heaps_push_pop_ads_binary_heap_safe
);

main!(library_benchmark_groups = micro_heaps_callgrind_group);
