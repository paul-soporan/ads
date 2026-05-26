#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;
mod shared;

use iai_callgrind::{library_benchmark_group, main};
use shared::*;

library_benchmark_group!(
    name = micro_dsu_callgrind_group;
    benchmarks =
        callgrind_micro_dsu_union_find_ads_dsu_safe,
        callgrind_micro_dsu_union_find_ads_dsu_raw,
        callgrind_micro_dsu_union_find_ads_dsu_arena,
        callgrind_motivational_dsu_connectivity_union_find_ads_dsu_arena_O_alpha_N,
        callgrind_motivational_dsu_connectivity_union_find_naive_O_N_union
);

main!(library_benchmark_groups = micro_dsu_callgrind_group);
