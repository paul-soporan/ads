#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;
mod shared;

use iai_callgrind::{library_benchmark_group, main};
use shared::*;

library_benchmark_group!(
    name = macro_thrashing_callgrind_group;
    benchmarks =
        callgrind_macro_thrashing_thrash_std_btreemap,
        callgrind_macro_thrashing_thrash_std_hashmap,
        callgrind_macro_thrashing_thrash_ads_bst_safe,
        callgrind_macro_thrashing_thrash_ads_bst_raw,
        callgrind_macro_thrashing_thrash_ads_bst_arena,
        callgrind_macro_thrashing_thrash_ads_btree_safe_t8,
        callgrind_macro_thrashing_thrash_ads_btree_raw_t8,
        callgrind_macro_thrashing_thrash_ads_btree_arena_t8,
        callgrind_macro_thrashing_thrash_ads_avl_safe,
        callgrind_macro_thrashing_thrash_ads_avl_raw,
        callgrind_macro_thrashing_thrash_ads_avl_arena,
        callgrind_macro_thrashing_thrash_ads_rbt_safe,
        callgrind_macro_thrashing_thrash_ads_rbt_raw,
        callgrind_macro_thrashing_thrash_ads_rbt_arena,
        callgrind_macro_thrashing_thrash_ads_splay_safe,
        callgrind_macro_thrashing_thrash_ads_splay_raw,
        callgrind_macro_thrashing_thrash_ads_splay_arena
);

main!(library_benchmark_groups = macro_thrashing_callgrind_group);
