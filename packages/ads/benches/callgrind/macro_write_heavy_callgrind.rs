#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;
mod shared;

use iai_callgrind::{library_benchmark_group, main};
use shared::*;

library_benchmark_group!(
    name = macro_write_heavy_callgrind_group;
    benchmarks =
        callgrind_macro_write_heavy_mix_std_btreemap,
        callgrind_macro_write_heavy_mix_std_hashmap,
        callgrind_macro_write_heavy_mix_ads_bst_safe,
        callgrind_macro_write_heavy_mix_ads_bst_raw,
        callgrind_macro_write_heavy_mix_ads_bst_arena,
        callgrind_macro_write_heavy_mix_ads_btree_safe_t8,
        callgrind_macro_write_heavy_mix_ads_btree_raw_t8,
        callgrind_macro_write_heavy_mix_ads_btree_arena_t8,
        callgrind_macro_write_heavy_mix_ads_avl_safe,
        callgrind_macro_write_heavy_mix_ads_avl_raw,
        callgrind_macro_write_heavy_mix_ads_avl_arena,
        callgrind_macro_write_heavy_mix_ads_rbt_safe,
        callgrind_macro_write_heavy_mix_ads_rbt_raw,
        callgrind_macro_write_heavy_mix_ads_rbt_arena,
        callgrind_macro_write_heavy_mix_ads_splay_safe,
        callgrind_macro_write_heavy_mix_ads_splay_safe_adaptive,
        callgrind_macro_write_heavy_mix_ads_splay_raw,
        callgrind_macro_write_heavy_mix_ads_splay_raw_adaptive,
        callgrind_macro_write_heavy_mix_ads_splay_arena,
        callgrind_macro_write_heavy_mix_ads_splay_arena_adaptive
);

main!(library_benchmark_groups = macro_write_heavy_callgrind_group);
