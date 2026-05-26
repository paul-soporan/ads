use std::collections::{BTreeMap, HashMap, LinkedList, VecDeque};
use std::path::PathBuf;

use crate::common;
use crate::generators::{
    LargePayload, read_heavy_ops, short_strings, temporal_locality_queries, uniform_keys,
    write_heavy_ops, zipfian_queries,
};
use ads::traits::core::{Map, PriorityQueue, Sequence, SequenceCursor};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

type SinglySafe = ads::linked::singly_linked_list::safe::SinglyLinkedList<u64>;
type SinglyRaw = ads::linked::singly_linked_list::raw::SinglyLinkedList<u64>;
type SinglyArena = ads::linked::singly_linked_list::arena::SinglyLinkedList<u64>;
type DoublySafe = ads::linked::doubly_linked_list::safe::DoublyLinkedList<u64>;
type DoublyRaw = ads::linked::doubly_linked_list::raw::DoublyLinkedList<u64>;
type DoublyArena = ads::linked::doubly_linked_list::arena::DoublyLinkedList<u64>;

pub fn resolve_dhat_dir() -> PathBuf {
    let dhat_dir = if let Ok(dir) = std::env::var("ADS_DHAT_DIR") {
        PathBuf::from(dir)
    } else if let Ok(path) = std::env::var("ADS_DHAT_FILE") {
        PathBuf::from(path)
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target/dhat"))
    } else {
        PathBuf::from("target/dhat")
    };

    let _ = std::fs::create_dir_all(&dhat_dir);
    dhat_dir
}

fn profile_case<F>(
    dhat_dir: &PathBuf,
    workload: &str,
    payload: &str,
    operation: &str,
    implementation: &str,
    size: usize,
    run: F,
) where
    F: FnOnce() -> usize,
{
    let path = dhat_dir.join(format!(
        "dhat__workload_{workload}__payload_{payload}__op_{operation}__impl_{implementation}__size_{size}.json"
    ));

    let profiler = dhat::Profiler::builder().file_name(path).build();
    let _ = run();
    drop(profiler);
}

fn profile_map_u64<M: common::BenchMap>(
    dhat_dir: &PathBuf,
    workload: &str,
    operation: &str,
    implementation: &str,
    size: usize,
    seed: u64,
) {
    match operation {
        "insert" => {
            let keys = uniform_keys(size, seed);
            profile_case(
                dhat_dir,
                workload,
                "u64",
                operation,
                implementation,
                size,
                || common::map_insert_bench::<M>(&keys),
            );
        }
        "contains" => {
            let keys = uniform_keys(size, seed);
            let queries = zipfian_queries(size, size, seed + 11);
            profile_case(
                dhat_dir,
                workload,
                "u64",
                operation,
                implementation,
                size,
                || common::map_contains_bench::<M>(&keys, &queries),
            );
        }
        "remove" => {
            let keys = uniform_keys(size, seed);
            profile_case(
                dhat_dir,
                workload,
                "u64",
                operation,
                implementation,
                size,
                || common::map_remove_bench::<M>(&keys),
            );
        }
        "mix_read" => {
            let prefill = uniform_keys(size, seed);
            let ops = read_heavy_ops(size, size, seed + 21);
            profile_case(
                dhat_dir,
                workload,
                "u64",
                "mix",
                implementation,
                size,
                || common::map_mixed_ops_bench::<M>(&prefill, &ops),
            );
        }
        "mix_write" => {
            let prefill = uniform_keys(size, seed);
            let ops = write_heavy_ops(size, size, seed + 21);
            profile_case(
                dhat_dir,
                workload,
                "u64",
                "mix",
                implementation,
                size,
                || common::map_mixed_ops_bench::<M>(&prefill, &ops),
            );
        }
        "thrash" => {
            let prefill = uniform_keys(size, seed);
            let remove_keys = uniform_keys(size, seed + 11);
            let mut insert_keys = uniform_keys(size * 2, seed + 22);
            insert_keys.truncate(size);
            for key in &mut insert_keys {
                *key = key.wrapping_add(size as u64);
            }

            profile_case(
                dhat_dir,
                workload,
                "u64",
                operation,
                implementation,
                size,
                || common::map_thrashing_bench::<M>(&prefill, &remove_keys, &insert_keys),
            );
        }
        _ => {}
    }
}

fn profile_map_u64_adaptive<M: common::BenchAdaptiveMap>(
    dhat_dir: &PathBuf,
    workload: &str,
    operation: &str,
    implementation: &str,
    size: usize,
    seed: u64,
) {
    match operation {
        "contains" => {
            let keys = uniform_keys(size, seed);
            let queries = zipfian_queries(size, size, seed + 11);
            profile_case(
                dhat_dir,
                workload,
                "u64",
                operation,
                implementation,
                size,
                || common::map_contains_adaptive_bench::<M>(&keys, &queries),
            );
        }
        "mix_read" => {
            let prefill = uniform_keys(size, seed);
            let ops = read_heavy_ops(size, size, seed + 21);
            profile_case(
                dhat_dir,
                workload,
                "u64",
                "mix",
                implementation,
                size,
                || common::map_mixed_ops_adaptive_bench::<M>(&prefill, &ops),
            );
        }
        "mix_write" => {
            let prefill = uniform_keys(size, seed);
            let ops = write_heavy_ops(size, size, seed + 21);
            profile_case(
                dhat_dir,
                workload,
                "u64",
                "mix",
                implementation,
                size,
                || common::map_mixed_ops_adaptive_bench::<M>(&prefill, &ops),
            );
        }
        _ => {}
    }
}

fn profile_map_strings<M: common::BenchStringMap>(
    dhat_dir: &PathBuf,
    implementation: &str,
    size: usize,
    seed: u64,
) {
    let keys = short_strings(size, seed);
    let mut queries = short_strings(size, seed + 77);
    queries.extend(keys.iter().take(size / 8).cloned());

    profile_case(
        dhat_dir,
        "micro_maps_strings",
        "string",
        "insert",
        implementation,
        size,
        || common::string_map_insert_bench::<M>(&keys),
    );

    profile_case(
        dhat_dir,
        "micro_maps_strings",
        "string",
        "contains",
        implementation,
        size,
        || common::string_map_contains_bench::<M>(&keys, &queries),
    );
}

fn profile_map_strings_adaptive<M: common::BenchAdaptiveStringMap>(
    dhat_dir: &PathBuf,
    implementation: &str,
    size: usize,
    seed: u64,
) {
    let keys = short_strings(size, seed);
    let mut queries = short_strings(size, seed + 77);
    queries.extend(keys.iter().take(size / 8).cloned());

    profile_case(
        dhat_dir,
        "micro_maps_strings",
        "string",
        "contains",
        implementation,
        size,
        || common::string_map_contains_adaptive_bench::<M>(&keys, &queries),
    );
}

fn profile_map_payload<M: common::BenchPayloadMap>(
    dhat_dir: &PathBuf,
    implementation: &str,
    size: usize,
    seed: u64,
) {
    let keys = uniform_keys(size, seed);
    let queries = zipfian_queries(size, size, seed + 66);

    profile_case(
        dhat_dir,
        "micro_maps_large_payload",
        "large_payload",
        "insert",
        implementation,
        size,
        || common::payload_map_insert_bench::<M>(&keys),
    );

    profile_case(
        dhat_dir,
        "micro_maps_large_payload",
        "large_payload",
        "contains",
        implementation,
        size,
        || common::payload_map_contains_bench::<M>(&keys, &queries),
    );
}

fn profile_map_payload_adaptive<M: common::BenchAdaptivePayloadMap>(
    dhat_dir: &PathBuf,
    implementation: &str,
    size: usize,
    seed: u64,
) {
    let keys = uniform_keys(size, seed);
    let queries = temporal_locality_queries(size, size, seed + 66);

    profile_case(
        dhat_dir,
        "micro_maps_large_payload",
        "large_payload",
        "contains",
        implementation,
        size,
        || common::payload_map_contains_adaptive_bench::<M>(&keys, &queries),
    );
}

pub fn profile_micro_maps(dhat_dir: &PathBuf) {
    for &size in &[1_000usize, 10_000usize] {
        profile_map_u64::<BTreeMap<u64, u64>>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "std_btreemap",
            size,
            0xA000 + size as u64,
        );
        profile_map_u64::<HashMap<u64, u64>>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "std_hashmap",
            size,
            0xA100 + size as u64,
        );
        profile_map_u64::<common::BstSafe>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "ads_bst_safe",
            size,
            0xA200 + size as u64,
        );
        profile_map_u64::<common::BstRaw>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "ads_bst_raw",
            size,
            0xA210 + size as u64,
        );
        profile_map_u64::<common::BstArena>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "ads_bst_arena",
            size,
            0xA220 + size as u64,
        );
        profile_map_u64::<common::BtSafe>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "ads_btree_safe_t8",
            size,
            0xAB00 + size as u64,
        );
        profile_map_u64::<common::BtRaw>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "ads_btree_raw_t8",
            size,
            0xAC00 + size as u64,
        );
        profile_map_u64::<common::BtArena>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "ads_btree_arena_t8",
            size,
            0xAD00 + size as u64,
        );
        profile_map_u64::<common::AvlSafe>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "ads_avl_safe",
            size,
            0xA500 + size as u64,
        );
        profile_map_u64::<common::AvlRaw>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "ads_avl_raw",
            size,
            0xA600 + size as u64,
        );
        profile_map_u64::<common::AvlArena>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "ads_avl_arena",
            size,
            0xA700 + size as u64,
        );
        profile_map_u64::<common::RbSafe>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "ads_rbt_safe",
            size,
            0xA800 + size as u64,
        );
        profile_map_u64::<common::RbRaw>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "ads_rbt_raw",
            size,
            0xA900 + size as u64,
        );
        profile_map_u64::<common::RbArena>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "ads_rbt_arena",
            size,
            0xAA00 + size as u64,
        );
        profile_map_u64::<common::SplaySafe>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "ads_splay_safe",
            size,
            0xAE00 + size as u64,
        );
        profile_map_u64::<common::SplayRaw>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "ads_splay_raw",
            size,
            0xAF00 + size as u64,
        );
        profile_map_u64::<common::SplayArena>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "ads_splay_arena",
            size,
            0xA010 + size as u64,
        );
        profile_map_u64::<common::SkipSafe>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "ads_skip_safe",
            size,
            0xA020 + size as u64,
        );
        profile_map_u64::<common::SkipRaw>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "ads_skip_raw",
            size,
            0xA030 + size as u64,
        );
        profile_map_u64::<common::SkipArena>(
            dhat_dir,
            "micro_maps_u64",
            "insert",
            "ads_skip_arena",
            size,
            0xA040 + size as u64,
        );

        profile_map_u64::<BTreeMap<u64, u64>>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "std_btreemap",
            size,
            0xB000 + size as u64,
        );
        profile_map_u64::<HashMap<u64, u64>>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "std_hashmap",
            size,
            0xB100 + size as u64,
        );
        profile_map_u64::<common::BstSafe>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_bst_safe",
            size,
            0xB200 + size as u64,
        );
        profile_map_u64::<common::BstRaw>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_bst_raw",
            size,
            0xB210 + size as u64,
        );
        profile_map_u64::<common::BstArena>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_bst_arena",
            size,
            0xB220 + size as u64,
        );
        profile_map_u64::<common::BtSafe>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_btree_safe_t8",
            size,
            0xBB00 + size as u64,
        );
        profile_map_u64::<common::BtRaw>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_btree_raw_t8",
            size,
            0xBC00 + size as u64,
        );
        profile_map_u64::<common::BtArena>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_btree_arena_t8",
            size,
            0xBD00 + size as u64,
        );
        profile_map_u64::<common::AvlSafe>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_avl_safe",
            size,
            0xB500 + size as u64,
        );
        profile_map_u64::<common::AvlRaw>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_avl_raw",
            size,
            0xB600 + size as u64,
        );
        profile_map_u64::<common::AvlArena>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_avl_arena",
            size,
            0xB700 + size as u64,
        );
        profile_map_u64::<common::RbSafe>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_rbt_safe",
            size,
            0xB800 + size as u64,
        );
        profile_map_u64::<common::RbRaw>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_rbt_raw",
            size,
            0xB900 + size as u64,
        );
        profile_map_u64::<common::RbArena>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_rbt_arena",
            size,
            0xBA00 + size as u64,
        );
        profile_map_u64::<common::SplaySafe>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_splay_safe",
            size,
            0xBE00 + size as u64,
        );
        profile_map_u64::<common::SplayRaw>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_splay_raw",
            size,
            0xBF00 + size as u64,
        );
        profile_map_u64::<common::SplayArena>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_splay_arena",
            size,
            0xB010 + size as u64,
        );
        profile_map_u64_adaptive::<common::SplaySafe>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_splay_safe_adaptive",
            size,
            0xB110 + size as u64,
        );
        profile_map_u64_adaptive::<common::SplayRaw>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_splay_raw_adaptive",
            size,
            0xB120 + size as u64,
        );
        profile_map_u64_adaptive::<common::SplayArena>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_splay_arena_adaptive",
            size,
            0xB130 + size as u64,
        );
        profile_map_u64::<common::SkipSafe>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_skip_safe",
            size,
            0xB020 + size as u64,
        );
        profile_map_u64::<common::SkipRaw>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_skip_raw",
            size,
            0xB030 + size as u64,
        );
        profile_map_u64::<common::SkipArena>(
            dhat_dir,
            "micro_maps_u64",
            "contains",
            "ads_skip_arena",
            size,
            0xB040 + size as u64,
        );

        profile_map_u64::<BTreeMap<u64, u64>>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "std_btreemap",
            size,
            0xC000 + size as u64,
        );
        profile_map_u64::<HashMap<u64, u64>>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "std_hashmap",
            size,
            0xC100 + size as u64,
        );
        profile_map_u64::<common::BstSafe>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "ads_bst_safe",
            size,
            0xC200 + size as u64,
        );
        profile_map_u64::<common::BstRaw>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "ads_bst_raw",
            size,
            0xC210 + size as u64,
        );
        profile_map_u64::<common::BstArena>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "ads_bst_arena",
            size,
            0xC220 + size as u64,
        );
        profile_map_u64::<common::BtSafe>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "ads_btree_safe_t8",
            size,
            0xCB00 + size as u64,
        );
        profile_map_u64::<common::BtRaw>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "ads_btree_raw_t8",
            size,
            0xCC00 + size as u64,
        );
        profile_map_u64::<common::BtArena>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "ads_btree_arena_t8",
            size,
            0xCD00 + size as u64,
        );
        profile_map_u64::<common::AvlSafe>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "ads_avl_safe",
            size,
            0xC500 + size as u64,
        );
        profile_map_u64::<common::AvlRaw>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "ads_avl_raw",
            size,
            0xC600 + size as u64,
        );
        profile_map_u64::<common::AvlArena>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "ads_avl_arena",
            size,
            0xC700 + size as u64,
        );
        profile_map_u64::<common::RbSafe>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "ads_rbt_safe",
            size,
            0xC800 + size as u64,
        );
        profile_map_u64::<common::RbRaw>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "ads_rbt_raw",
            size,
            0xC900 + size as u64,
        );
        profile_map_u64::<common::RbArena>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "ads_rbt_arena",
            size,
            0xCA00 + size as u64,
        );
        profile_map_u64::<common::SplaySafe>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "ads_splay_safe",
            size,
            0xCE00 + size as u64,
        );
        profile_map_u64::<common::SplayRaw>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "ads_splay_raw",
            size,
            0xCF00 + size as u64,
        );
        profile_map_u64::<common::SplayArena>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "ads_splay_arena",
            size,
            0xC010 + size as u64,
        );
        profile_map_u64::<common::SkipSafe>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "ads_skip_safe",
            size,
            0xC020 + size as u64,
        );
        profile_map_u64::<common::SkipRaw>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "ads_skip_raw",
            size,
            0xC030 + size as u64,
        );
        profile_map_u64::<common::SkipArena>(
            dhat_dir,
            "micro_maps_u64",
            "remove",
            "ads_skip_arena",
            size,
            0xC040 + size as u64,
        );

        profile_map_strings::<BTreeMap<String, usize>>(
            dhat_dir,
            "std_btreemap",
            size,
            0xD000 + size as u64,
        );
        profile_map_strings::<HashMap<String, usize>>(
            dhat_dir,
            "std_hashmap",
            size,
            0xD100 + size as u64,
        );
        profile_map_strings::<common::StrBstSafe>(
            dhat_dir,
            "ads_bst_safe",
            size,
            0xD150 + size as u64,
        );
        profile_map_strings::<common::StrBstRaw>(
            dhat_dir,
            "ads_bst_raw",
            size,
            0xD160 + size as u64,
        );
        profile_map_strings::<common::StrBstArena>(
            dhat_dir,
            "ads_bst_arena",
            size,
            0xD170 + size as u64,
        );
        profile_map_strings::<common::StrBtSafe>(
            dhat_dir,
            "ads_btree_safe_t8",
            size,
            0xD200 + size as u64,
        );
        profile_map_strings::<common::StrBtRaw>(
            dhat_dir,
            "ads_btree_raw_t8",
            size,
            0xD300 + size as u64,
        );
        profile_map_strings::<common::StrBtArena>(
            dhat_dir,
            "ads_btree_arena_t8",
            size,
            0xD400 + size as u64,
        );
        profile_map_strings::<common::StrAvlSafe>(
            dhat_dir,
            "ads_avl_safe",
            size,
            0xD500 + size as u64,
        );
        profile_map_strings::<common::StrAvlRaw>(
            dhat_dir,
            "ads_avl_raw",
            size,
            0xD600 + size as u64,
        );
        profile_map_strings::<common::StrAvlArena>(
            dhat_dir,
            "ads_avl_arena",
            size,
            0xD700 + size as u64,
        );
        profile_map_strings::<common::StrRbSafe>(
            dhat_dir,
            "ads_rbt_safe",
            size,
            0xD800 + size as u64,
        );
        profile_map_strings::<common::StrRbRaw>(
            dhat_dir,
            "ads_rbt_raw",
            size,
            0xD900 + size as u64,
        );
        profile_map_strings::<common::StrRbArena>(
            dhat_dir,
            "ads_rbt_arena",
            size,
            0xDA00 + size as u64,
        );
        profile_map_strings::<common::StrSplaySafe>(
            dhat_dir,
            "ads_splay_safe",
            size,
            0xDB00 + size as u64,
        );
        profile_map_strings::<common::StrSplayRaw>(
            dhat_dir,
            "ads_splay_raw",
            size,
            0xDC00 + size as u64,
        );
        profile_map_strings::<common::StrSplayArena>(
            dhat_dir,
            "ads_splay_arena",
            size,
            0xDD00 + size as u64,
        );
        profile_map_strings_adaptive::<common::StrSplaySafe>(
            dhat_dir,
            "ads_splay_safe_adaptive",
            size,
            0xDB10 + size as u64,
        );
        profile_map_strings_adaptive::<common::StrSplayRaw>(
            dhat_dir,
            "ads_splay_raw_adaptive",
            size,
            0xDC10 + size as u64,
        );
        profile_map_strings_adaptive::<common::StrSplayArena>(
            dhat_dir,
            "ads_splay_arena_adaptive",
            size,
            0xDD10 + size as u64,
        );
        profile_map_strings::<common::StrSkipSafe>(
            dhat_dir,
            "ads_skip_safe",
            size,
            0xDE00 + size as u64,
        );
        profile_map_strings::<common::StrSkipRaw>(
            dhat_dir,
            "ads_skip_raw",
            size,
            0xDF00 + size as u64,
        );
        profile_map_strings::<common::StrSkipArena>(
            dhat_dir,
            "ads_skip_arena",
            size,
            0xD010 + size as u64,
        );
    }

    for &size in &[500usize, 5_000usize] {
        profile_map_payload::<BTreeMap<u64, LargePayload>>(
            dhat_dir,
            "std_btreemap",
            size,
            0xE000 + size as u64,
        );
        profile_map_payload::<HashMap<u64, LargePayload>>(
            dhat_dir,
            "std_hashmap",
            size,
            0xE100 + size as u64,
        );
        profile_map_payload::<common::PayloadBstSafe>(
            dhat_dir,
            "ads_bst_safe",
            size,
            0xE150 + size as u64,
        );
        profile_map_payload::<common::PayloadBstRaw>(
            dhat_dir,
            "ads_bst_raw",
            size,
            0xE160 + size as u64,
        );
        profile_map_payload::<common::PayloadBstArena>(
            dhat_dir,
            "ads_bst_arena",
            size,
            0xE170 + size as u64,
        );
        profile_map_payload::<common::PayloadBtSafe>(
            dhat_dir,
            "ads_btree_safe_t8",
            size,
            0xE200 + size as u64,
        );
        profile_map_payload::<common::PayloadBtRaw>(
            dhat_dir,
            "ads_btree_raw_t8",
            size,
            0xE300 + size as u64,
        );
        profile_map_payload::<common::PayloadBtArena>(
            dhat_dir,
            "ads_btree_arena_t8",
            size,
            0xE400 + size as u64,
        );
        profile_map_payload::<common::PayloadAvlSafe>(
            dhat_dir,
            "ads_avl_safe",
            size,
            0xE500 + size as u64,
        );
        profile_map_payload::<common::PayloadAvlRaw>(
            dhat_dir,
            "ads_avl_raw",
            size,
            0xE600 + size as u64,
        );
        profile_map_payload::<common::PayloadAvlArena>(
            dhat_dir,
            "ads_avl_arena",
            size,
            0xE700 + size as u64,
        );
        profile_map_payload::<common::PayloadRbSafe>(
            dhat_dir,
            "ads_rbt_safe",
            size,
            0xE800 + size as u64,
        );
        profile_map_payload::<common::PayloadRbRaw>(
            dhat_dir,
            "ads_rbt_raw",
            size,
            0xE900 + size as u64,
        );
        profile_map_payload::<common::PayloadRbArena>(
            dhat_dir,
            "ads_rbt_arena",
            size,
            0xEA00 + size as u64,
        );
        profile_map_payload::<common::PayloadSplaySafe>(
            dhat_dir,
            "ads_splay_safe",
            size,
            0xEB00 + size as u64,
        );
        profile_map_payload::<common::PayloadSplayRaw>(
            dhat_dir,
            "ads_splay_raw",
            size,
            0xEC00 + size as u64,
        );
        profile_map_payload::<common::PayloadSplayArena>(
            dhat_dir,
            "ads_splay_arena",
            size,
            0xED00 + size as u64,
        );
        profile_map_payload_adaptive::<common::PayloadSplaySafe>(
            dhat_dir,
            "ads_splay_safe_adaptive",
            size,
            0xEB10 + size as u64,
        );
        profile_map_payload_adaptive::<common::PayloadSplayRaw>(
            dhat_dir,
            "ads_splay_raw_adaptive",
            size,
            0xEC10 + size as u64,
        );
        profile_map_payload_adaptive::<common::PayloadSplayArena>(
            dhat_dir,
            "ads_splay_arena_adaptive",
            size,
            0xED10 + size as u64,
        );
        profile_map_payload::<common::PayloadSkipSafe>(
            dhat_dir,
            "ads_skip_safe",
            size,
            0xEE00 + size as u64,
        );
        profile_map_payload::<common::PayloadSkipRaw>(
            dhat_dir,
            "ads_skip_raw",
            size,
            0xEF00 + size as u64,
        );
        profile_map_payload::<common::PayloadSkipArena>(
            dhat_dir,
            "ads_skip_arena",
            size,
            0xE010 + size as u64,
        );
    }
}

pub fn profile_macro_read_heavy(dhat_dir: &PathBuf) {
    for &size in &[1_000usize, 10_000usize] {
        profile_map_u64::<BTreeMap<u64, u64>>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "std_btreemap",
            size,
            0xF000 + size as u64,
        );
        profile_map_u64::<HashMap<u64, u64>>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "std_hashmap",
            size,
            0xF100 + size as u64,
        );
        profile_map_u64::<common::BstSafe>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "ads_bst_safe",
            size,
            0xF150 + size as u64,
        );
        profile_map_u64::<common::BstRaw>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "ads_bst_raw",
            size,
            0xF160 + size as u64,
        );
        profile_map_u64::<common::BstArena>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "ads_bst_arena",
            size,
            0xF170 + size as u64,
        );
        profile_map_u64::<common::BtSafe>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "ads_btree_safe_t8",
            size,
            0xF200 + size as u64,
        );
        profile_map_u64::<common::BtRaw>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "ads_btree_raw_t8",
            size,
            0xF300 + size as u64,
        );
        profile_map_u64::<common::BtArena>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "ads_btree_arena_t8",
            size,
            0xF400 + size as u64,
        );
        profile_map_u64::<common::AvlSafe>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "ads_avl_safe",
            size,
            0xF500 + size as u64,
        );
        profile_map_u64::<common::AvlRaw>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "ads_avl_raw",
            size,
            0xF600 + size as u64,
        );
        profile_map_u64::<common::AvlArena>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "ads_avl_arena",
            size,
            0xF700 + size as u64,
        );
        profile_map_u64::<common::RbSafe>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "ads_rbt_safe",
            size,
            0xF800 + size as u64,
        );
        profile_map_u64::<common::RbRaw>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "ads_rbt_raw",
            size,
            0xF900 + size as u64,
        );
        profile_map_u64::<common::RbArena>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "ads_rbt_arena",
            size,
            0xFA00 + size as u64,
        );
        profile_map_u64::<common::SplaySafe>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "ads_splay_safe",
            size,
            0xFB00 + size as u64,
        );
        profile_map_u64::<common::SplayRaw>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "ads_splay_raw",
            size,
            0xFC00 + size as u64,
        );
        profile_map_u64::<common::SplayArena>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "ads_splay_arena",
            size,
            0xFD00 + size as u64,
        );
        profile_map_u64_adaptive::<common::SplaySafe>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "ads_splay_safe_adaptive",
            size,
            0xFB10 + size as u64,
        );
        profile_map_u64_adaptive::<common::SplayRaw>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "ads_splay_raw_adaptive",
            size,
            0xFC10 + size as u64,
        );
        profile_map_u64_adaptive::<common::SplayArena>(
            dhat_dir,
            "macro_read_heavy_u64",
            "mix_read",
            "ads_splay_arena_adaptive",
            size,
            0xFD10 + size as u64,
        );
    }
}

pub fn profile_macro_write_heavy(dhat_dir: &PathBuf) {
    for &size in &[1_000usize, 10_000usize] {
        profile_map_u64::<BTreeMap<u64, u64>>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "std_btreemap",
            size,
            0x11000 + size as u64,
        );
        profile_map_u64::<HashMap<u64, u64>>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "std_hashmap",
            size,
            0x11100 + size as u64,
        );
        profile_map_u64::<common::BstSafe>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "ads_bst_safe",
            size,
            0x11150 + size as u64,
        );
        profile_map_u64::<common::BstRaw>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "ads_bst_raw",
            size,
            0x11160 + size as u64,
        );
        profile_map_u64::<common::BstArena>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "ads_bst_arena",
            size,
            0x11170 + size as u64,
        );
        profile_map_u64::<common::BtSafe>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "ads_btree_safe_t8",
            size,
            0x11200 + size as u64,
        );
        profile_map_u64::<common::BtRaw>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "ads_btree_raw_t8",
            size,
            0x11300 + size as u64,
        );
        profile_map_u64::<common::BtArena>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "ads_btree_arena_t8",
            size,
            0x11400 + size as u64,
        );
        profile_map_u64::<common::AvlSafe>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "ads_avl_safe",
            size,
            0x11500 + size as u64,
        );
        profile_map_u64::<common::AvlRaw>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "ads_avl_raw",
            size,
            0x11600 + size as u64,
        );
        profile_map_u64::<common::AvlArena>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "ads_avl_arena",
            size,
            0x11700 + size as u64,
        );
        profile_map_u64::<common::RbSafe>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "ads_rbt_safe",
            size,
            0x11800 + size as u64,
        );
        profile_map_u64::<common::RbRaw>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "ads_rbt_raw",
            size,
            0x11900 + size as u64,
        );
        profile_map_u64::<common::RbArena>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "ads_rbt_arena",
            size,
            0x11A00 + size as u64,
        );
        profile_map_u64::<common::SplaySafe>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "ads_splay_safe",
            size,
            0x11B00 + size as u64,
        );
        profile_map_u64::<common::SplayRaw>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "ads_splay_raw",
            size,
            0x11C00 + size as u64,
        );
        profile_map_u64::<common::SplayArena>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "ads_splay_arena",
            size,
            0x11D00 + size as u64,
        );
        profile_map_u64_adaptive::<common::SplaySafe>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "ads_splay_safe_adaptive",
            size,
            0x11B10 + size as u64,
        );
        profile_map_u64_adaptive::<common::SplayRaw>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "ads_splay_raw_adaptive",
            size,
            0x11C10 + size as u64,
        );
        profile_map_u64_adaptive::<common::SplayArena>(
            dhat_dir,
            "macro_write_heavy_u64",
            "mix_write",
            "ads_splay_arena_adaptive",
            size,
            0x11D10 + size as u64,
        );
    }
}

pub fn profile_macro_thrashing(dhat_dir: &PathBuf) {
    for &size in &[1_000usize, 10_000usize] {
        profile_map_u64::<BTreeMap<u64, u64>>(
            dhat_dir,
            "macro_thrashing_u64",
            "thrash",
            "std_btreemap",
            size,
            0x12000 + size as u64,
        );
        profile_map_u64::<HashMap<u64, u64>>(
            dhat_dir,
            "macro_thrashing_u64",
            "thrash",
            "std_hashmap",
            size,
            0x12100 + size as u64,
        );
        profile_map_u64::<common::BstSafe>(
            dhat_dir,
            "macro_thrashing_u64",
            "thrash",
            "ads_bst_safe",
            size,
            0x12150 + size as u64,
        );
        profile_map_u64::<common::BstRaw>(
            dhat_dir,
            "macro_thrashing_u64",
            "thrash",
            "ads_bst_raw",
            size,
            0x12160 + size as u64,
        );
        profile_map_u64::<common::BstArena>(
            dhat_dir,
            "macro_thrashing_u64",
            "thrash",
            "ads_bst_arena",
            size,
            0x12170 + size as u64,
        );
        profile_map_u64::<common::BtSafe>(
            dhat_dir,
            "macro_thrashing_u64",
            "thrash",
            "ads_btree_safe_t8",
            size,
            0x12200 + size as u64,
        );
        profile_map_u64::<common::BtRaw>(
            dhat_dir,
            "macro_thrashing_u64",
            "thrash",
            "ads_btree_raw_t8",
            size,
            0x12300 + size as u64,
        );
        profile_map_u64::<common::BtArena>(
            dhat_dir,
            "macro_thrashing_u64",
            "thrash",
            "ads_btree_arena_t8",
            size,
            0x12400 + size as u64,
        );
        profile_map_u64::<common::AvlSafe>(
            dhat_dir,
            "macro_thrashing_u64",
            "thrash",
            "ads_avl_safe",
            size,
            0x12500 + size as u64,
        );
        profile_map_u64::<common::AvlRaw>(
            dhat_dir,
            "macro_thrashing_u64",
            "thrash",
            "ads_avl_raw",
            size,
            0x12600 + size as u64,
        );
        profile_map_u64::<common::AvlArena>(
            dhat_dir,
            "macro_thrashing_u64",
            "thrash",
            "ads_avl_arena",
            size,
            0x12700 + size as u64,
        );
        profile_map_u64::<common::RbSafe>(
            dhat_dir,
            "macro_thrashing_u64",
            "thrash",
            "ads_rbt_safe",
            size,
            0x12800 + size as u64,
        );
        profile_map_u64::<common::RbRaw>(
            dhat_dir,
            "macro_thrashing_u64",
            "thrash",
            "ads_rbt_raw",
            size,
            0x12900 + size as u64,
        );
        profile_map_u64::<common::RbArena>(
            dhat_dir,
            "macro_thrashing_u64",
            "thrash",
            "ads_rbt_arena",
            size,
            0x12A00 + size as u64,
        );
        profile_map_u64::<common::SplaySafe>(
            dhat_dir,
            "macro_thrashing_u64",
            "thrash",
            "ads_splay_safe",
            size,
            0x12B00 + size as u64,
        );
        profile_map_u64::<common::SplayRaw>(
            dhat_dir,
            "macro_thrashing_u64",
            "thrash",
            "ads_splay_raw",
            size,
            0x12C00 + size as u64,
        );
        profile_map_u64::<common::SplayArena>(
            dhat_dir,
            "macro_thrashing_u64",
            "thrash",
            "ads_splay_arena",
            size,
            0x12D00 + size as u64,
        );
    }
}

pub fn profile_micro_heaps(dhat_dir: &PathBuf) {
    for &size in &[1_000usize, 10_000usize] {
        let keys = uniform_keys(size, 0x13000 + size as u64);
        profile_case(
            dhat_dir,
            "micro_heaps_u64",
            "u64",
            "push_pop",
            "std_binary_heap",
            size,
            || common::heap_push_pop_bench::<common::StdBinaryHeapMin>(&keys),
        );
        profile_case(
            dhat_dir,
            "micro_heaps_u64",
            "u64",
            "push_pop",
            "ads_binary_arena",
            size,
            || common::heap_push_pop_bench::<common::BinaryArena>(&keys),
        );
        profile_case(
            dhat_dir,
            "micro_heaps_u64",
            "u64",
            "push_pop",
            "ads_binary_safe",
            size,
            || common::heap_push_pop_bench::<common::BinarySafe>(&keys),
        );
        profile_case(
            dhat_dir,
            "micro_heaps_u64",
            "u64",
            "push_pop",
            "ads_binary_raw",
            size,
            || common::heap_push_pop_bench::<common::BinaryRaw>(&keys),
        );
        profile_case(
            dhat_dir,
            "micro_heaps_u64",
            "u64",
            "push_pop",
            "ads_binomial_arena",
            size,
            || common::heap_push_pop_bench::<common::BinomialArena>(&keys),
        );
        profile_case(
            dhat_dir,
            "micro_heaps_u64",
            "u64",
            "push_pop",
            "ads_fibonacci_arena",
            size,
            || common::heap_push_pop_bench::<common::FibonacciArena>(&keys),
        );
    }

    for &size in &[1_000usize, 5_000usize] {
        let keys1 = uniform_keys(size, 0x1000 + size as u64);
        let keys2 = uniform_keys(size, 0x2000 + size as u64);

        profile_case(
            dhat_dir,
            "motivational_heap_merge_u64",
            "u64",
            "merge",
            "ads_binary_arena",
            size,
            || {
                let mut h1 = common::BinaryArena::from_iter(keys1.iter().cloned());
                let mut h2 = common::BinaryArena::from_iter(keys2.iter().cloned());
                h1.merge(&mut h2);
                h1.len()
            },
        );
        profile_case(
            dhat_dir,
            "motivational_heap_merge_u64",
            "u64",
            "merge",
            "ads_binomial_arena",
            size,
            || {
                let mut h1 = common::BinomialArena::from_iter(keys1.iter().cloned());
                let mut h2 = common::BinomialArena::from_iter(keys2.iter().cloned());
                h1.merge(&mut h2);
                h1.len()
            },
        );
        profile_case(
            dhat_dir,
            "motivational_heap_merge_u64",
            "u64",
            "merge",
            "ads_fibonacci_arena",
            size,
            || {
                let mut h1 = common::FibonacciArena::from_iter(keys1.iter().cloned());
                let mut h2 = common::FibonacciArena::from_iter(keys2.iter().cloned());
                h1.merge(&mut h2);
                h1.len()
            },
        );
    }
}

pub fn profile_micro_dsu(dhat_dir: &PathBuf) {
    for &size in &[1_000usize, 10_000usize] {
        profile_case(
            dhat_dir,
            "micro_dsu_u64",
            "u64",
            "union_find",
            "ads_dsu_safe",
            size,
            || common::dsu_workload::<common::DsuSafe>(size),
        );
        profile_case(
            dhat_dir,
            "micro_dsu_u64",
            "u64",
            "union_find",
            "ads_dsu_raw",
            size,
            || common::dsu_workload::<common::DsuRaw>(size),
        );
        profile_case(
            dhat_dir,
            "micro_dsu_u64",
            "u64",
            "union_find",
            "ads_dsu_arena",
            size,
            || common::dsu_workload::<common::DsuArena>(size),
        );
    }

    for &size in &[100usize, 500usize, 1_000usize] {
        profile_case(
            dhat_dir,
            "motivational_dsu_connectivity_u64",
            "u64",
            "union_find",
            "naive_O_N_union",
            size,
            || {
                struct NaiveDisjointSet {
                    parent: Vec<usize>,
                }

                impl NaiveDisjointSet {
                    fn new(size: usize) -> Self {
                        Self {
                            parent: (0..size).collect(),
                        }
                    }

                    fn find(&self, i: usize) -> usize {
                        self.parent[i]
                    }

                    fn union(&mut self, i: usize, j: usize) {
                        let root_i = self.find(i);
                        let root_j = self.find(j);
                        if root_i != root_j {
                            for k in 0..self.parent.len() {
                                if self.parent[k] == root_j {
                                    self.parent[k] = root_i;
                                }
                            }
                        }
                    }
                }

                let mut ds = NaiveDisjointSet::new(size);
                for i in 0..(size / 2) {
                    ds.union(i, i + size / 2);
                }

                let mut checksum = 0usize;
                for i in 0..size {
                    if ds.find(i) == ds.find(0) {
                        checksum = checksum.wrapping_add(1);
                    }
                }
                checksum
            },
        );

        profile_case(
            dhat_dir,
            "motivational_dsu_connectivity_u64",
            "u64",
            "union_find",
            "ads_dsu_arena_O_alpha_N",
            size,
            || common::dsu_workload::<common::DsuArena>(size),
        );
    }
}

pub fn profile_micro_sequences(dhat_dir: &PathBuf) {
    for &size in &[1_000usize, 10_000usize] {
        profile_case(
            dhat_dir,
            "micro_sequences_u64",
            "u64",
            "push_pop",
            "std_vec",
            size,
            || {
                let mut values = Vec::with_capacity(size);
                for index in 0..size {
                    values.push(index as u64);
                }
                let mut checksum = 0usize;
                while let Some(value) = values.pop() {
                    checksum = checksum.wrapping_add(value as usize);
                }
                checksum
            },
        );

        profile_case(
            dhat_dir,
            "micro_sequences_u64",
            "u64",
            "push_pop",
            "std_vecdeque",
            size,
            || {
                let mut values = VecDeque::with_capacity(size);
                for index in 0..size {
                    values.push_back(index as u64);
                }
                let mut checksum = 0usize;
                while let Some(value) = values.pop_front() {
                    checksum = checksum.wrapping_add(value as usize);
                }
                checksum
            },
        );

        profile_case(
            dhat_dir,
            "micro_sequences_u64",
            "u64",
            "push_pop",
            "std_linked_list",
            size,
            || {
                let mut values = LinkedList::new();
                for index in 0..size {
                    values.push_back(index as u64);
                }
                let mut checksum = 0usize;
                while let Some(value) = values.pop_front() {
                    checksum = checksum.wrapping_add(value as usize);
                }
                checksum
            },
        );

        profile_case(
            dhat_dir,
            "micro_sequences_u64",
            "u64",
            "push_pop",
            "ads_singly_safe",
            size,
            || profile_ads_sequence::<SinglySafe>(size),
        );
        profile_case(
            dhat_dir,
            "micro_sequences_u64",
            "u64",
            "push_pop",
            "ads_singly_raw",
            size,
            || profile_ads_sequence::<SinglyRaw>(size),
        );
        profile_case(
            dhat_dir,
            "micro_sequences_u64",
            "u64",
            "push_pop",
            "ads_singly_arena",
            size,
            || profile_ads_sequence::<SinglyArena>(size),
        );
        profile_case(
            dhat_dir,
            "micro_sequences_u64",
            "u64",
            "push_pop",
            "ads_doubly_safe",
            size,
            || profile_ads_sequence::<DoublySafe>(size),
        );
        profile_case(
            dhat_dir,
            "micro_sequences_u64",
            "u64",
            "push_pop",
            "ads_doubly_raw",
            size,
            || profile_ads_sequence::<DoublyRaw>(size),
        );
        profile_case(
            dhat_dir,
            "micro_sequences_u64",
            "u64",
            "push_pop",
            "ads_doubly_arena",
            size,
            || profile_ads_sequence::<DoublyArena>(size),
        );
    }
}

fn profile_ads_sequence<S>(size: usize) -> usize
where
    S: Sequence<u64> + Default,
{
    let mut values = S::default();
    for index in 0..size {
        values.push_back(index as u64);
    }

    let mut checksum = 0usize;
    while let Some(value) = values.pop_front() {
        checksum = checksum.wrapping_add(value as usize);
    }
    checksum
}

pub fn profile_micro_sequences_indexing(dhat_dir: &PathBuf) {
    for &size in &[100usize, 1_000usize] {
        profile_case(
            dhat_dir,
            "micro_sequences_indexing_u64",
            "u64",
            "index",
            "ads_singly_safe",
            size,
            || {
                let mut list = SinglySafe::default();
                for i in 0..size {
                    list.push_back(i as u64);
                }
                let mut checksum = 0usize;
                for i in (0..size).step_by(10) {
                    if let Some(c) = ads::traits::core::Sequence::cursor_at(&list, i) {
                        checksum =
                            checksum.wrapping_add(ads::traits::core::SequenceCursor::index(&c));
                    }
                }
                checksum
            },
        );

        profile_case(
            dhat_dir,
            "micro_sequences_indexing_u64",
            "u64",
            "index",
            "ads_doubly_safe",
            size,
            || {
                let mut list = DoublySafe::default();
                for i in 0..size {
                    list.push_back(i as u64);
                }
                let mut checksum = 0usize;
                for i in (0..size).step_by(10) {
                    if let Some(c) = ads::traits::core::Sequence::cursor_at(&list, i) {
                        checksum =
                            checksum.wrapping_add(ads::traits::core::SequenceCursor::index(&c));
                    }
                }
                checksum
            },
        );

        profile_case(
            dhat_dir,
            "micro_sequences_indexing_u64",
            "u64",
            "index",
            "ads_skip_arena",
            size,
            || {
                let mut list = ads::linked::skip_list::arena::SkipList::new();
                for i in 0..size {
                    list.insert(i as u64, i as u64);
                }
                let mut checksum = 0usize;
                for i in (0..size).step_by(10) {
                    if let Some(c) = list.cursor_at(i) {
                        checksum = checksum.wrapping_add(c.index());
                    }
                }
                checksum
            },
        );
    }
}

pub fn profile_sweep_btree_cache(dhat_dir: &PathBuf) {
    for &size in &[1_000usize, 10_000usize] {
        profile_map_u64::<common::BtSafeDeg4>(
            dhat_dir,
            "sweep_btree_cache_u64",
            "insert",
            "ads_btree_safe_t4",
            size,
            0x14000 + size as u64,
        );
        profile_map_u64::<common::BtSafeDeg16>(
            dhat_dir,
            "sweep_btree_cache_u64",
            "insert",
            "ads_btree_safe_t16",
            size,
            0x14100 + size as u64,
        );
        profile_map_u64::<common::BtSafeDeg64>(
            dhat_dir,
            "sweep_btree_cache_u64",
            "insert",
            "ads_btree_safe_t64",
            size,
            0x14200 + size as u64,
        );
        profile_map_u64::<common::BtRawDeg4>(
            dhat_dir,
            "sweep_btree_cache_u64",
            "insert",
            "ads_btree_raw_t4",
            size,
            0x14600 + size as u64,
        );
        profile_map_u64::<common::BtRawDeg16>(
            dhat_dir,
            "sweep_btree_cache_u64",
            "insert",
            "ads_btree_raw_t16",
            size,
            0x14700 + size as u64,
        );
        profile_map_u64::<common::BtRawDeg64>(
            dhat_dir,
            "sweep_btree_cache_u64",
            "insert",
            "ads_btree_raw_t64",
            size,
            0x14800 + size as u64,
        );
        profile_map_u64::<common::BtArenaDeg4>(
            dhat_dir,
            "sweep_btree_cache_u64",
            "insert",
            "ads_btree_arena_t4",
            size,
            0x14900 + size as u64,
        );
        profile_map_u64::<common::BtArenaDeg16>(
            dhat_dir,
            "sweep_btree_cache_u64",
            "insert",
            "ads_btree_arena_t16",
            size,
            0x14A00 + size as u64,
        );
        profile_map_u64::<common::BtArenaDeg64>(
            dhat_dir,
            "sweep_btree_cache_u64",
            "insert",
            "ads_btree_arena_t64",
            size,
            0x14B00 + size as u64,
        );
        profile_map_u64::<common::BtSafeDeg4>(
            dhat_dir,
            "sweep_btree_cache_u64",
            "contains",
            "ads_btree_safe_t4",
            size,
            0x14300 + size as u64,
        );
        profile_map_u64::<common::BtSafeDeg16>(
            dhat_dir,
            "sweep_btree_cache_u64",
            "contains",
            "ads_btree_safe_t16",
            size,
            0x14400 + size as u64,
        );
        profile_map_u64::<common::BtSafeDeg64>(
            dhat_dir,
            "sweep_btree_cache_u64",
            "contains",
            "ads_btree_safe_t64",
            size,
            0x14500 + size as u64,
        );
        profile_map_u64::<common::BtRawDeg4>(
            dhat_dir,
            "sweep_btree_cache_u64",
            "contains",
            "ads_btree_raw_t4",
            size,
            0x14C00 + size as u64,
        );
        profile_map_u64::<common::BtRawDeg16>(
            dhat_dir,
            "sweep_btree_cache_u64",
            "contains",
            "ads_btree_raw_t16",
            size,
            0x14D00 + size as u64,
        );
        profile_map_u64::<common::BtRawDeg64>(
            dhat_dir,
            "sweep_btree_cache_u64",
            "contains",
            "ads_btree_raw_t64",
            size,
            0x14E00 + size as u64,
        );
        profile_map_u64::<common::BtArenaDeg4>(
            dhat_dir,
            "sweep_btree_cache_u64",
            "contains",
            "ads_btree_arena_t4",
            size,
            0x14F00 + size as u64,
        );
        profile_map_u64::<common::BtArenaDeg16>(
            dhat_dir,
            "sweep_btree_cache_u64",
            "contains",
            "ads_btree_arena_t16",
            size,
            0x15000 + size as u64,
        );
        profile_map_u64::<common::BtArenaDeg64>(
            dhat_dir,
            "sweep_btree_cache_u64",
            "contains",
            "ads_btree_arena_t64",
            size,
            0x15100 + size as u64,
        );
    }
}

pub fn profile_sweep_hash_collisions(dhat_dir: &PathBuf) {
    for &size in &[1_000usize, 10_000usize] {
        let keys = uniform_keys(size, 0x15000 + size as u64);
        profile_case(
            dhat_dir,
            "sweep_hash_collisions_u64",
            "u64",
            "insert",
            "std_hashmap",
            size,
            || common::map_insert_bench::<HashMap<u64, u64>>(&keys),
        );

        profile_case(
            dhat_dir,
            "sweep_hash_collisions_u64",
            "u64",
            "insert",
            "hashmap_zero_hasher",
            size,
            || {
                let mut map = common::colliding_hasher_map();
                for &key in &keys {
                    let _ = map.insert(key, key ^ 0xCAFE_BABE);
                }
                map.len()
            },
        );

        let queries = temporal_locality_queries(size, size, 0x15100 + size as u64);
        profile_case(
            dhat_dir,
            "sweep_hash_collisions_u64",
            "u64",
            "contains",
            "hashmap_zero_hasher",
            size,
            || {
                let mut map: common::CollidingHashMap<u64, u64> = common::colliding_hasher_map();
                for &key in &keys {
                    let _ = map.insert(key, key);
                }
                let mut hits = 0usize;
                for query in &queries {
                    if map.contains_key(query) {
                        hits = hits.wrapping_add(1);
                    }
                }
                hits
            },
        );

        profile_case(
            dhat_dir,
            "sweep_hash_collisions_u64",
            "u64",
            "contains",
            "std_btreemap_reference",
            size,
            || common::map_contains_bench::<BTreeMap<u64, u64>>(&keys, &queries),
        );
    }
}
