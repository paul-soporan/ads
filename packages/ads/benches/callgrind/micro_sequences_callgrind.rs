#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;
mod shared;

use iai_callgrind::{library_benchmark_group, main};
use shared::*;

library_benchmark_group!(
    name = micro_sequences_callgrind_group;
    benchmarks =
        callgrind_micro_sequences_push_pop_std_vec,
        callgrind_micro_sequences_push_pop_std_vecdeque,
        callgrind_micro_sequences_push_pop_std_linked_list,
        callgrind_micro_sequences_push_pop_singly_safe,
        callgrind_micro_sequences_push_pop_singly_raw,
        callgrind_micro_sequences_push_pop_singly_arena,
        callgrind_micro_sequences_push_pop_doubly_safe,
        callgrind_micro_sequences_push_pop_doubly_raw,
        callgrind_micro_sequences_push_pop_doubly_arena
);

main!(library_benchmark_groups = micro_sequences_callgrind_group);
