#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;
mod shared;

use iai_callgrind::{library_benchmark_group, main};
use shared::*;

library_benchmark_group!(
    name = macro_read_heavy_callgrind_group;
    benchmarks =
        callgrind_macro_read_heavy_mix_std_btreemap,
        callgrind_macro_read_heavy_mix_std_hashmap,
        callgrind_macro_read_heavy_mix_bst_safe,
        callgrind_macro_read_heavy_mix_bst_raw,
        callgrind_macro_read_heavy_mix_bst_arena,
        callgrind_macro_read_heavy_mix_btree_safe_t8,
        callgrind_macro_read_heavy_mix_btree_raw_t8,
        callgrind_macro_read_heavy_mix_btree_arena_t8,
        callgrind_macro_read_heavy_mix_avl_safe,
        callgrind_macro_read_heavy_mix_avl_raw,
        callgrind_macro_read_heavy_mix_avl_arena,
        callgrind_macro_read_heavy_mix_rbt_safe,
        callgrind_macro_read_heavy_mix_rbt_raw,
        callgrind_macro_read_heavy_mix_rbt_arena,
        callgrind_macro_read_heavy_mix_splay_safe,
        callgrind_macro_read_heavy_mix_splay_safe_adaptive,
        callgrind_macro_read_heavy_mix_splay_raw,
        callgrind_macro_read_heavy_mix_splay_raw_adaptive,
        callgrind_macro_read_heavy_mix_splay_arena,
        callgrind_macro_read_heavy_mix_splay_arena_adaptive
);

main!(library_benchmark_groups = macro_read_heavy_callgrind_group);
