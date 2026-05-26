use std::collections::{BTreeMap, HashMap, LinkedList, VecDeque};
use std::hint::black_box;

use crate::common;
use crate::generators::{
    LargePayload, read_heavy_ops, short_strings, temporal_locality_queries, uniform_keys,
    write_heavy_ops, zipfian_queries,
};
use ads::traits::core::{Map, Sequence, SequenceCursor};
use iai_callgrind::library_benchmark;

type SinglySafe = ads::linked::singly_linked_list::safe::SinglyLinkedList<u64>;
type SinglyRaw = ads::linked::singly_linked_list::raw::SinglyLinkedList<u64>;
type SinglyArena = ads::linked::singly_linked_list::arena::SinglyLinkedList<u64>;
type DoublySafe = ads::linked::doubly_linked_list::safe::DoublyLinkedList<u64>;
type DoublyRaw = ads::linked::doubly_linked_list::raw::DoublyLinkedList<u64>;
type DoublyArena = ads::linked::doubly_linked_list::arena::DoublyLinkedList<u64>;

fn run_u64_insert<M: common::BenchMap>(size: usize, seed: u64) -> usize {
    let keys = uniform_keys(size, seed);
    black_box(common::map_insert_bench::<M>(&keys))
}

fn run_u64_contains<M: common::BenchMap>(size: usize, key_seed: u64, query_seed: u64) -> usize {
    let keys = uniform_keys(size, key_seed);
    let queries = zipfian_queries(size, size, query_seed);
    black_box(common::map_contains_bench::<M>(&keys, &queries))
}

fn run_u64_contains_adaptive<M: common::BenchAdaptiveMap>(
    size: usize,
    key_seed: u64,
    query_seed: u64,
) -> usize {
    let keys = uniform_keys(size, key_seed);
    let queries = zipfian_queries(size, size, query_seed);
    black_box(common::map_contains_adaptive_bench::<M>(&keys, &queries))
}

fn run_u64_remove<M: common::BenchMap>(size: usize, seed: u64) -> usize {
    let keys = uniform_keys(size, seed);
    black_box(common::map_remove_bench::<M>(&keys))
}

fn run_u64_mix_read_heavy<M: common::BenchMap>(size: usize, key_seed: u64, ops_seed: u64) -> usize {
    let prefill = uniform_keys(size, key_seed);
    let ops = read_heavy_ops(size, size, ops_seed);
    black_box(common::map_mixed_ops_bench::<M>(&prefill, &ops))
}

fn run_u64_mix_read_heavy_adaptive<M: common::BenchAdaptiveMap>(
    size: usize,
    key_seed: u64,
    ops_seed: u64,
) -> usize {
    let prefill = uniform_keys(size, key_seed);
    let ops = read_heavy_ops(size, size, ops_seed);
    black_box(common::map_mixed_ops_adaptive_bench::<M>(&prefill, &ops))
}

fn run_u64_mix_write_heavy<M: common::BenchMap>(
    size: usize,
    key_seed: u64,
    ops_seed: u64,
) -> usize {
    let prefill = uniform_keys(size, key_seed);
    let ops = write_heavy_ops(size, size, ops_seed);
    black_box(common::map_mixed_ops_bench::<M>(&prefill, &ops))
}

fn run_u64_mix_write_heavy_adaptive<M: common::BenchAdaptiveMap>(
    size: usize,
    key_seed: u64,
    ops_seed: u64,
) -> usize {
    let prefill = uniform_keys(size, key_seed);
    let ops = write_heavy_ops(size, size, ops_seed);
    black_box(common::map_mixed_ops_adaptive_bench::<M>(&prefill, &ops))
}

fn run_u64_thrash<M: common::BenchMap>(
    size: usize,
    prefill_seed: u64,
    remove_seed: u64,
    insert_seed: u64,
) -> usize {
    let prefill = uniform_keys(size, prefill_seed);
    let remove_keys = uniform_keys(size, remove_seed);
    let mut insert_keys = uniform_keys(size * 2, insert_seed);
    insert_keys.truncate(size);
    for key in &mut insert_keys {
        *key = key.wrapping_add(size as u64);
    }

    black_box(common::map_thrashing_bench::<M>(
        &prefill,
        &remove_keys,
        &insert_keys,
    ))
}

fn run_string_insert<M: common::BenchStringMap>(size: usize, seed: u64) -> usize {
    let keys = short_strings(size, seed);
    black_box(common::string_map_insert_bench::<M>(&keys))
}

fn run_string_contains<M: common::BenchStringMap>(
    size: usize,
    key_seed: u64,
    query_seed: u64,
) -> usize {
    let keys = short_strings(size, key_seed);
    let mut queries = short_strings(size, query_seed);
    queries.extend(keys.iter().take(size / 8).cloned());
    black_box(common::string_map_contains_bench::<M>(&keys, &queries))
}

fn run_string_contains_adaptive<M: common::BenchAdaptiveStringMap>(
    size: usize,
    key_seed: u64,
    query_seed: u64,
) -> usize {
    let keys = short_strings(size, key_seed);
    let mut queries = short_strings(size, query_seed);
    queries.extend(keys.iter().take(size / 8).cloned());
    black_box(common::string_map_contains_adaptive_bench::<M>(
        &keys, &queries,
    ))
}

fn run_payload_insert<M: common::BenchPayloadMap>(size: usize, seed: u64) -> usize {
    let keys = uniform_keys(size, seed);
    black_box(common::payload_map_insert_bench::<M>(&keys))
}

fn run_payload_contains<M: common::BenchPayloadMap>(
    size: usize,
    key_seed: u64,
    query_seed: u64,
) -> usize {
    let keys = uniform_keys(size, key_seed);
    let queries = temporal_locality_queries(size, size, query_seed);
    black_box(common::payload_map_contains_bench::<M>(&keys, &queries))
}

fn run_payload_contains_adaptive<M: common::BenchAdaptivePayloadMap>(
    size: usize,
    key_seed: u64,
    query_seed: u64,
) -> usize {
    let keys = uniform_keys(size, key_seed);
    let queries = temporal_locality_queries(size, size, query_seed);
    black_box(common::payload_map_contains_adaptive_bench::<M>(
        &keys, &queries,
    ))
}

fn run_collision_insert(size: usize, seed: u64) -> usize {
    let keys = uniform_keys(size, seed);
    let mut map = common::colliding_hasher_map();
    for &key in &keys {
        let _ = map.insert(black_box(key), black_box(key ^ 0xCAFE_BABE));
    }
    black_box(map.len())
}

fn run_collision_contains(size: usize, key_seed: u64, query_seed: u64) -> usize {
    let keys = uniform_keys(size, key_seed);
    let queries = temporal_locality_queries(size, size, query_seed);
    let mut map: common::CollidingHashMap<u64, u64> = common::colliding_hasher_map();
    for &key in &keys {
        let _ = map.insert(key, key);
    }

    let mut hits = 0usize;
    for query in &queries {
        if map.contains_key(black_box(query)) {
            hits = hits.wrapping_add(1);
        }
    }

    black_box(hits)
}

fn run_heap_push_pop<P: common::BenchPriorityQueue>(size: usize, seed: u64) -> usize {
    let keys = uniform_keys(size, seed);
    black_box(common::heap_push_pop_bench::<P>(&keys))
}

fn run_heap_merge<
    P: common::BenchPriorityQueue + ads::traits::core::PriorityQueue<u64> + FromIterator<u64>,
>(
    size: usize,
    seed1: u64,
    seed2: u64,
) -> usize {
    let keys1 = uniform_keys(size, seed1);
    let keys2 = uniform_keys(size, seed2);
    let mut h1 = P::from_iter(keys1.iter().cloned());
    let mut h2 = P::from_iter(keys2.iter().cloned());
    h1.merge(&mut h2);
    black_box(h1.len())
}

fn run_dsu_workload<D: common::BenchDisjointSet>(size: usize) -> usize {
    black_box(common::dsu_workload::<D>(size))
}

fn run_dsu_connectivity<D: common::BenchDisjointSet>(size: usize) -> usize {
    let mut ds = D::new();
    for i in 0..size {
        ds.make_set_value(i as u64);
    }
    for i in 0..(size / 2) {
        ds.union_values(&(i as u64), &((i + size / 2) as u64));
    }
    let mut checksum = 0usize;
    for i in 0..size {
        if ds.same_set_values(&(i as u64), &0u64) {
            checksum = checksum.wrapping_add(1);
        }
    }
    black_box(checksum)
}

fn run_sequence_indexing<S: Sequence<u64> + Default>(size: usize) -> usize {
    let mut values = S::default();
    for i in 0..size {
        values.push_back(i as u64);
    }
    let mut checksum = 0usize;
    for i in (0..size).step_by(10) {
        if let Some(cursor) = values.cursor_at(i) {
            checksum = checksum.wrapping_add(cursor.index());
        }
    }
    black_box(checksum)
}

fn run_sequence_vecdeque(size: usize) -> usize {
    let mut values = VecDeque::with_capacity(size);
    for index in 0..size {
        values.push_back(index as u64);
    }

    let mut checksum = 0usize;
    while let Some(value) = values.pop_front() {
        checksum = checksum.wrapping_add(value as usize);
    }
    black_box(checksum)
}

fn run_sequence_vec(size: usize) -> usize {
    let mut values = Vec::with_capacity(size);
    for index in 0..size {
        values.push(index as u64);
    }

    let mut checksum = 0usize;
    while let Some(value) = values.pop() {
        checksum = checksum.wrapping_add(value as usize);
    }
    black_box(checksum)
}

fn run_sequence_linked_list(size: usize) -> usize {
    let mut values = LinkedList::new();
    for index in 0..size {
        values.push_back(index as u64);
    }

    let mut checksum = 0usize;
    while let Some(value) = values.pop_front() {
        checksum = checksum.wrapping_add(value as usize);
    }
    black_box(checksum)
}

fn run_ads_sequence<S>(size: usize) -> usize
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
    black_box(checksum)
}

macro_rules! define_u64_micro_maps {
    ($insert_fn:ident, $contains_fn:ident, $remove_fn:ident, $impl_ty:ty, $seed:expr) => {
        #[library_benchmark]
        #[bench::n1k(1_000)]
        #[bench::n10k(10_000)]
        pub fn $insert_fn(size: usize) -> usize {
            run_u64_insert::<$impl_ty>(size, 0xA000 + $seed + size as u64)
        }

        #[library_benchmark]
        #[bench::n1k(1_000)]
        #[bench::n10k(10_000)]
        pub fn $contains_fn(size: usize) -> usize {
            run_u64_contains::<$impl_ty>(
                size,
                0xB000 + $seed + size as u64,
                0xB800 + $seed + size as u64,
            )
        }

        #[library_benchmark]
        #[bench::n1k(1_000)]
        #[bench::n10k(10_000)]
        pub fn $remove_fn(size: usize) -> usize {
            run_u64_remove::<$impl_ty>(size, 0xC000 + $seed + size as u64)
        }
    };
}

macro_rules! define_u64_macro_mix {
    ($read_fn:ident, $write_fn:ident, $impl_ty:ty, $seed:expr) => {
        #[library_benchmark]
        #[bench::n1k(1_000)]
        #[bench::n10k(10_000)]
        pub fn $read_fn(size: usize) -> usize {
            run_u64_mix_read_heavy::<$impl_ty>(
                size,
                0xD000 + $seed + size as u64,
                0xD800 + $seed + size as u64,
            )
        }

        #[library_benchmark]
        #[bench::n1k(1_000)]
        #[bench::n10k(10_000)]
        pub fn $write_fn(size: usize) -> usize {
            run_u64_mix_write_heavy::<$impl_ty>(
                size,
                0xE000 + $seed + size as u64,
                0xE800 + $seed + size as u64,
            )
        }
    };
}

macro_rules! define_u64_micro_contains_adaptive {
    ($contains_fn:ident, $impl_ty:ty, $seed:expr) => {
        #[library_benchmark]
        #[bench::n1k(1_000)]
        #[bench::n10k(10_000)]
        pub fn $contains_fn(size: usize) -> usize {
            run_u64_contains_adaptive::<$impl_ty>(
                size,
                0xB1000 + $seed + size as u64,
                0xB1800 + $seed + size as u64,
            )
        }
    };
}

macro_rules! define_u64_macro_mix_adaptive {
    ($read_fn:ident, $write_fn:ident, $impl_ty:ty, $seed:expr) => {
        #[library_benchmark]
        #[bench::n1k(1_000)]
        #[bench::n10k(10_000)]
        pub fn $read_fn(size: usize) -> usize {
            run_u64_mix_read_heavy_adaptive::<$impl_ty>(
                size,
                0xD1000 + $seed + size as u64,
                0xD1800 + $seed + size as u64,
            )
        }

        #[library_benchmark]
        #[bench::n1k(1_000)]
        #[bench::n10k(10_000)]
        pub fn $write_fn(size: usize) -> usize {
            run_u64_mix_write_heavy_adaptive::<$impl_ty>(
                size,
                0xE1000 + $seed + size as u64,
                0xE1800 + $seed + size as u64,
            )
        }
    };
}

macro_rules! define_u64_macro_thrash {
    ($thrash_fn:ident, $impl_ty:ty, $seed:expr) => {
        #[library_benchmark]
        #[bench::n1k(1_000)]
        #[bench::n10k(10_000)]
        pub fn $thrash_fn(size: usize) -> usize {
            run_u64_thrash::<$impl_ty>(
                size,
                0xF000 + $seed + size as u64,
                0xF800 + $seed + size as u64,
                0xFA00 + $seed + size as u64,
            )
        }
    };
}

macro_rules! define_string_micro_maps {
    ($insert_fn:ident, $contains_fn:ident, $impl_ty:ty, $seed:expr) => {
        #[library_benchmark]
        #[bench::n1k(1_000)]
        #[bench::n10k(10_000)]
        pub fn $insert_fn(size: usize) -> usize {
            run_string_insert::<$impl_ty>(size, 0x11000 + $seed + size as u64)
        }

        #[library_benchmark]
        #[bench::n1k(1_000)]
        #[bench::n10k(10_000)]
        pub fn $contains_fn(size: usize) -> usize {
            run_string_contains::<$impl_ty>(
                size,
                0x11500 + $seed + size as u64,
                0x11600 + $seed + size as u64,
            )
        }
    };
}

macro_rules! define_string_micro_contains_adaptive {
    ($contains_fn:ident, $impl_ty:ty, $seed:expr) => {
        #[library_benchmark]
        #[bench::n1k(1_000)]
        #[bench::n10k(10_000)]
        pub fn $contains_fn(size: usize) -> usize {
            run_string_contains_adaptive::<$impl_ty>(
                size,
                0x116000 + $seed + size as u64,
                0x116800 + $seed + size as u64,
            )
        }
    };
}

macro_rules! define_payload_micro_maps {
    ($insert_fn:ident, $contains_fn:ident, $impl_ty:ty, $seed:expr) => {
        #[library_benchmark]
        #[bench::n500(500)]
        #[bench::n5k(5_000)]
        pub fn $insert_fn(size: usize) -> usize {
            run_payload_insert::<$impl_ty>(size, 0x12000 + $seed + size as u64)
        }

        #[library_benchmark]
        #[bench::n500(500)]
        #[bench::n5k(5_000)]
        pub fn $contains_fn(size: usize) -> usize {
            run_payload_contains::<$impl_ty>(
                size,
                0x12500 + $seed + size as u64,
                0x12600 + $seed + size as u64,
            )
        }
    };
}

macro_rules! define_payload_micro_contains_adaptive {
    ($contains_fn:ident, $impl_ty:ty, $seed:expr) => {
        #[library_benchmark]
        #[bench::n500(500)]
        #[bench::n5k(5_000)]
        pub fn $contains_fn(size: usize) -> usize {
            run_payload_contains_adaptive::<$impl_ty>(
                size,
                0x126000 + $seed + size as u64,
                0x126800 + $seed + size as u64,
            )
        }
    };
}

macro_rules! define_motivational_heap_merge {
    ($merge_fn:ident, $impl_ty:ty, $seed:expr) => {
        #[library_benchmark]
        #[bench::n1k(1_000)]
        #[bench::n5k(5_000)]
        pub fn $merge_fn(size: usize) -> usize {
            run_heap_merge::<$impl_ty>(
                size,
                0x1000 + $seed + size as u64,
                0x2000 + $seed + size as u64,
            )
        }
    };
}

macro_rules! define_motivational_dsu_connectivity {
    ($connectivity_fn:ident, $impl_ty:ty) => {
        #[library_benchmark]
        #[bench::n100(100)]
        #[bench::n500(500)]
        #[bench::n1k(1_000)]
        pub fn $connectivity_fn(size: usize) -> usize {
            run_dsu_connectivity::<$impl_ty>(size)
        }
    };
}

#[library_benchmark]
#[bench::n100(100)]
#[bench::n500(500)]
#[bench::n1k(1_000)]
pub fn callgrind_motivational_dsu_connectivity_union_find_naive_O_N_union(size: usize) -> usize {
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
    black_box(checksum)
}

define_u64_micro_maps!(callgrind_micro_maps_insert_std_btreemap, callgrind_micro_maps_contains_std_btreemap, callgrind_micro_maps_remove_std_btreemap, BTreeMap<u64, u64>, 11);
define_u64_micro_maps!(callgrind_micro_maps_insert_std_hashmap, callgrind_micro_maps_contains_std_hashmap, callgrind_micro_maps_remove_std_hashmap, HashMap<u64, u64>, 23);
define_u64_micro_maps!(
    callgrind_micro_maps_insert_ads_bst_safe,
    callgrind_micro_maps_contains_ads_bst_safe,
    callgrind_micro_maps_remove_ads_bst_safe,
    common::BstSafe,
    29
);
define_u64_micro_maps!(
    callgrind_micro_maps_insert_ads_bst_raw,
    callgrind_micro_maps_contains_ads_bst_raw,
    callgrind_micro_maps_remove_ads_bst_raw,
    common::BstRaw,
    31
);
define_u64_micro_maps!(
    callgrind_micro_maps_insert_ads_bst_arena,
    callgrind_micro_maps_contains_ads_bst_arena,
    callgrind_micro_maps_remove_ads_bst_arena,
    common::BstArena,
    37
);
define_u64_micro_maps!(
    callgrind_micro_maps_insert_ads_btree_safe_t8,
    callgrind_micro_maps_contains_ads_btree_safe_t8,
    callgrind_micro_maps_remove_ads_btree_safe_t8,
    common::BtSafe,
    41
);
define_u64_micro_maps!(
    callgrind_micro_maps_insert_ads_btree_raw_t8,
    callgrind_micro_maps_contains_ads_btree_raw_t8,
    callgrind_micro_maps_remove_ads_btree_raw_t8,
    common::BtRaw,
    43
);
define_u64_micro_maps!(
    callgrind_micro_maps_insert_ads_btree_arena_t8,
    callgrind_micro_maps_contains_ads_btree_arena_t8,
    callgrind_micro_maps_remove_ads_btree_arena_t8,
    common::BtArena,
    47
);
define_u64_micro_maps!(
    callgrind_micro_maps_insert_ads_avl_safe,
    callgrind_micro_maps_contains_ads_avl_safe,
    callgrind_micro_maps_remove_ads_avl_safe,
    common::AvlSafe,
    53
);
define_u64_micro_maps!(
    callgrind_micro_maps_insert_ads_avl_raw,
    callgrind_micro_maps_contains_ads_avl_raw,
    callgrind_micro_maps_remove_ads_avl_raw,
    common::AvlRaw,
    59
);
define_u64_micro_maps!(
    callgrind_micro_maps_insert_ads_avl_arena,
    callgrind_micro_maps_contains_ads_avl_arena,
    callgrind_micro_maps_remove_ads_avl_arena,
    common::AvlArena,
    61
);
define_u64_micro_maps!(
    callgrind_micro_maps_insert_ads_rbt_safe,
    callgrind_micro_maps_contains_ads_rbt_safe,
    callgrind_micro_maps_remove_ads_rbt_safe,
    common::RbSafe,
    67
);
define_u64_micro_maps!(
    callgrind_micro_maps_insert_ads_rbt_raw,
    callgrind_micro_maps_contains_ads_rbt_raw,
    callgrind_micro_maps_remove_ads_rbt_raw,
    common::RbRaw,
    71
);
define_u64_micro_maps!(
    callgrind_micro_maps_insert_ads_rbt_arena,
    callgrind_micro_maps_contains_ads_rbt_arena,
    callgrind_micro_maps_remove_ads_rbt_arena,
    common::RbArena,
    73
);
define_u64_micro_maps!(
    callgrind_micro_maps_insert_ads_splay_safe,
    callgrind_micro_maps_contains_ads_splay_safe,
    callgrind_micro_maps_remove_ads_splay_safe,
    common::SplaySafe,
    79
);
define_u64_micro_maps!(
    callgrind_micro_maps_insert_ads_splay_raw,
    callgrind_micro_maps_contains_ads_splay_raw,
    callgrind_micro_maps_remove_ads_splay_raw,
    common::SplayRaw,
    83
);
define_u64_micro_maps!(
    callgrind_micro_maps_insert_ads_splay_arena,
    callgrind_micro_maps_contains_ads_splay_arena,
    callgrind_micro_maps_remove_ads_splay_arena,
    common::SplayArena,
    89
);
define_u64_micro_contains_adaptive!(
    callgrind_micro_maps_contains_ads_splay_safe_adaptive,
    common::SplaySafe,
    179
);
define_u64_micro_contains_adaptive!(
    callgrind_micro_maps_contains_ads_splay_raw_adaptive,
    common::SplayRaw,
    183
);
define_u64_micro_contains_adaptive!(
    callgrind_micro_maps_contains_ads_splay_arena_adaptive,
    common::SplayArena,
    189
);
define_u64_micro_maps!(
    callgrind_micro_maps_insert_ads_skip_safe,
    callgrind_micro_maps_contains_ads_skip_safe,
    callgrind_micro_maps_remove_ads_skip_safe,
    common::SkipSafe,
    97
);
define_u64_micro_maps!(
    callgrind_micro_maps_insert_ads_skip_raw,
    callgrind_micro_maps_contains_ads_skip_raw,
    callgrind_micro_maps_remove_ads_skip_raw,
    common::SkipRaw,
    101
);
define_u64_micro_maps!(
    callgrind_micro_maps_insert_ads_skip_arena,
    callgrind_micro_maps_contains_ads_skip_arena,
    callgrind_micro_maps_remove_ads_skip_arena,
    common::SkipArena,
    103
);

define_u64_macro_mix!(callgrind_macro_read_heavy_mix_std_btreemap, callgrind_macro_write_heavy_mix_std_btreemap, BTreeMap<u64, u64>, 11);
define_u64_macro_mix!(callgrind_macro_read_heavy_mix_std_hashmap, callgrind_macro_write_heavy_mix_std_hashmap, HashMap<u64, u64>, 23);
define_u64_macro_mix!(
    callgrind_macro_read_heavy_mix_ads_bst_safe,
    callgrind_macro_write_heavy_mix_ads_bst_safe,
    common::BstSafe,
    29
);
define_u64_macro_mix!(
    callgrind_macro_read_heavy_mix_ads_bst_raw,
    callgrind_macro_write_heavy_mix_ads_bst_raw,
    common::BstRaw,
    31
);
define_u64_macro_mix!(
    callgrind_macro_read_heavy_mix_ads_bst_arena,
    callgrind_macro_write_heavy_mix_ads_bst_arena,
    common::BstArena,
    37
);
define_u64_macro_mix!(
    callgrind_macro_read_heavy_mix_ads_btree_safe_t8,
    callgrind_macro_write_heavy_mix_ads_btree_safe_t8,
    common::BtSafe,
    41
);
define_u64_macro_mix!(
    callgrind_macro_read_heavy_mix_ads_btree_raw_t8,
    callgrind_macro_write_heavy_mix_ads_btree_raw_t8,
    common::BtRaw,
    43
);
define_u64_macro_mix!(
    callgrind_macro_read_heavy_mix_ads_btree_arena_t8,
    callgrind_macro_write_heavy_mix_ads_btree_arena_t8,
    common::BtArena,
    47
);
define_u64_macro_mix!(
    callgrind_macro_read_heavy_mix_ads_avl_safe,
    callgrind_macro_write_heavy_mix_ads_avl_safe,
    common::AvlSafe,
    53
);
define_u64_macro_mix!(
    callgrind_macro_read_heavy_mix_ads_avl_raw,
    callgrind_macro_write_heavy_mix_ads_avl_raw,
    common::AvlRaw,
    59
);
define_u64_macro_mix!(
    callgrind_macro_read_heavy_mix_ads_avl_arena,
    callgrind_macro_write_heavy_mix_ads_avl_arena,
    common::AvlArena,
    61
);
define_u64_macro_mix!(
    callgrind_macro_read_heavy_mix_ads_rbt_safe,
    callgrind_macro_write_heavy_mix_ads_rbt_safe,
    common::RbSafe,
    67
);
define_u64_macro_mix!(
    callgrind_macro_read_heavy_mix_ads_rbt_raw,
    callgrind_macro_write_heavy_mix_ads_rbt_raw,
    common::RbRaw,
    71
);
define_u64_macro_mix!(
    callgrind_macro_read_heavy_mix_ads_rbt_arena,
    callgrind_macro_write_heavy_mix_ads_rbt_arena,
    common::RbArena,
    73
);
define_u64_macro_mix!(
    callgrind_macro_read_heavy_mix_ads_splay_safe,
    callgrind_macro_write_heavy_mix_ads_splay_safe,
    common::SplaySafe,
    79
);
define_u64_macro_mix!(
    callgrind_macro_read_heavy_mix_ads_splay_raw,
    callgrind_macro_write_heavy_mix_ads_splay_raw,
    common::SplayRaw,
    83
);
define_u64_macro_mix!(
    callgrind_macro_read_heavy_mix_ads_splay_arena,
    callgrind_macro_write_heavy_mix_ads_splay_arena,
    common::SplayArena,
    89
);
define_u64_macro_mix_adaptive!(
    callgrind_macro_read_heavy_mix_ads_splay_safe_adaptive,
    callgrind_macro_write_heavy_mix_ads_splay_safe_adaptive,
    common::SplaySafe,
    179
);
define_u64_macro_mix_adaptive!(
    callgrind_macro_read_heavy_mix_ads_splay_raw_adaptive,
    callgrind_macro_write_heavy_mix_ads_splay_raw_adaptive,
    common::SplayRaw,
    183
);
define_u64_macro_mix_adaptive!(
    callgrind_macro_read_heavy_mix_ads_splay_arena_adaptive,
    callgrind_macro_write_heavy_mix_ads_splay_arena_adaptive,
    common::SplayArena,
    189
);

define_u64_macro_thrash!(callgrind_macro_thrashing_thrash_std_btreemap, BTreeMap<u64, u64>, 11);
define_u64_macro_thrash!(callgrind_macro_thrashing_thrash_std_hashmap, HashMap<u64, u64>, 23);
define_u64_macro_thrash!(
    callgrind_macro_thrashing_thrash_ads_bst_safe,
    common::BstSafe,
    29
);
define_u64_macro_thrash!(
    callgrind_macro_thrashing_thrash_ads_bst_raw,
    common::BstRaw,
    31
);
define_u64_macro_thrash!(
    callgrind_macro_thrashing_thrash_ads_bst_arena,
    common::BstArena,
    37
);
define_u64_macro_thrash!(
    callgrind_macro_thrashing_thrash_ads_btree_safe_t8,
    common::BtSafe,
    41
);
define_u64_macro_thrash!(
    callgrind_macro_thrashing_thrash_ads_btree_raw_t8,
    common::BtRaw,
    43
);
define_u64_macro_thrash!(
    callgrind_macro_thrashing_thrash_ads_btree_arena_t8,
    common::BtArena,
    47
);
define_u64_macro_thrash!(
    callgrind_macro_thrashing_thrash_ads_avl_safe,
    common::AvlSafe,
    53
);
define_u64_macro_thrash!(
    callgrind_macro_thrashing_thrash_ads_avl_raw,
    common::AvlRaw,
    59
);
define_u64_macro_thrash!(
    callgrind_macro_thrashing_thrash_ads_avl_arena,
    common::AvlArena,
    61
);
define_u64_macro_thrash!(
    callgrind_macro_thrashing_thrash_ads_rbt_safe,
    common::RbSafe,
    67
);
define_u64_macro_thrash!(
    callgrind_macro_thrashing_thrash_ads_rbt_raw,
    common::RbRaw,
    71
);
define_u64_macro_thrash!(
    callgrind_macro_thrashing_thrash_ads_rbt_arena,
    common::RbArena,
    73
);
define_u64_macro_thrash!(
    callgrind_macro_thrashing_thrash_ads_splay_safe,
    common::SplaySafe,
    79
);
define_u64_macro_thrash!(
    callgrind_macro_thrashing_thrash_ads_splay_raw,
    common::SplayRaw,
    83
);
define_u64_macro_thrash!(
    callgrind_macro_thrashing_thrash_ads_splay_arena,
    common::SplayArena,
    89
);

define_motivational_heap_merge!(
    callgrind_motivational_heap_merge_merge_ads_binary_arena,
    common::BinaryArena,
    301
);
define_motivational_heap_merge!(
    callgrind_motivational_heap_merge_merge_ads_binomial_arena,
    common::BinomialArena,
    302
);
define_motivational_heap_merge!(
    callgrind_motivational_heap_merge_merge_ads_fibonacci_arena,
    common::FibonacciArena,
    303
);

define_motivational_dsu_connectivity!(
    callgrind_motivational_dsu_connectivity_union_find_ads_dsu_arena_O_alpha_N,
    common::DsuArena
);

define_string_micro_maps!(callgrind_micro_maps_strings_insert_std_btreemap, callgrind_micro_maps_strings_contains_std_btreemap, BTreeMap<String, usize>, 1);
define_string_micro_maps!(callgrind_micro_maps_strings_insert_std_hashmap, callgrind_micro_maps_strings_contains_std_hashmap, HashMap<String, usize>, 2);
define_string_micro_maps!(
    callgrind_micro_maps_strings_insert_ads_bst_safe,
    callgrind_micro_maps_strings_contains_ads_bst_safe,
    common::StrBstSafe,
    3
);
define_string_micro_maps!(
    callgrind_micro_maps_strings_insert_ads_bst_raw,
    callgrind_micro_maps_strings_contains_ads_bst_raw,
    common::StrBstRaw,
    4
);
define_string_micro_maps!(
    callgrind_micro_maps_strings_insert_ads_bst_arena,
    callgrind_micro_maps_strings_contains_ads_bst_arena,
    common::StrBstArena,
    5
);
define_string_micro_maps!(
    callgrind_micro_maps_strings_insert_ads_btree_safe_t8,
    callgrind_micro_maps_strings_contains_ads_btree_safe_t8,
    common::StrBtSafe,
    6
);
define_string_micro_maps!(
    callgrind_micro_maps_strings_insert_ads_btree_raw_t8,
    callgrind_micro_maps_strings_contains_ads_btree_raw_t8,
    common::StrBtRaw,
    7
);
define_string_micro_maps!(
    callgrind_micro_maps_strings_insert_ads_btree_arena_t8,
    callgrind_micro_maps_strings_contains_ads_btree_arena_t8,
    common::StrBtArena,
    8
);
define_string_micro_maps!(
    callgrind_micro_maps_strings_insert_ads_avl_safe,
    callgrind_micro_maps_strings_contains_ads_avl_safe,
    common::StrAvlSafe,
    9
);
define_string_micro_maps!(
    callgrind_micro_maps_strings_insert_ads_avl_raw,
    callgrind_micro_maps_strings_contains_ads_avl_raw,
    common::StrAvlRaw,
    10
);
define_string_micro_maps!(
    callgrind_micro_maps_strings_insert_ads_avl_arena,
    callgrind_micro_maps_strings_contains_ads_avl_arena,
    common::StrAvlArena,
    11
);
define_string_micro_maps!(
    callgrind_micro_maps_strings_insert_ads_rbt_safe,
    callgrind_micro_maps_strings_contains_ads_rbt_safe,
    common::StrRbSafe,
    12
);
define_string_micro_maps!(
    callgrind_micro_maps_strings_insert_ads_rbt_raw,
    callgrind_micro_maps_strings_contains_ads_rbt_raw,
    common::StrRbRaw,
    13
);
define_string_micro_maps!(
    callgrind_micro_maps_strings_insert_ads_rbt_arena,
    callgrind_micro_maps_strings_contains_ads_rbt_arena,
    common::StrRbArena,
    14
);
define_string_micro_maps!(
    callgrind_micro_maps_strings_insert_ads_splay_safe,
    callgrind_micro_maps_strings_contains_ads_splay_safe,
    common::StrSplaySafe,
    15
);
define_string_micro_maps!(
    callgrind_micro_maps_strings_insert_ads_splay_raw,
    callgrind_micro_maps_strings_contains_ads_splay_raw,
    common::StrSplayRaw,
    16
);
define_string_micro_maps!(
    callgrind_micro_maps_strings_insert_ads_splay_arena,
    callgrind_micro_maps_strings_contains_ads_splay_arena,
    common::StrSplayArena,
    17
);
define_string_micro_contains_adaptive!(
    callgrind_micro_maps_strings_contains_ads_splay_safe_adaptive,
    common::StrSplaySafe,
    115
);
define_string_micro_contains_adaptive!(
    callgrind_micro_maps_strings_contains_ads_splay_raw_adaptive,
    common::StrSplayRaw,
    116
);
define_string_micro_contains_adaptive!(
    callgrind_micro_maps_strings_contains_ads_splay_arena_adaptive,
    common::StrSplayArena,
    117
);
define_string_micro_maps!(
    callgrind_micro_maps_strings_insert_ads_skip_safe,
    callgrind_micro_maps_strings_contains_ads_skip_safe,
    common::StrSkipSafe,
    18
);
define_string_micro_maps!(
    callgrind_micro_maps_strings_insert_ads_skip_raw,
    callgrind_micro_maps_strings_contains_ads_skip_raw,
    common::StrSkipRaw,
    19
);
define_string_micro_maps!(
    callgrind_micro_maps_strings_insert_ads_skip_arena,
    callgrind_micro_maps_strings_contains_ads_skip_arena,
    common::StrSkipArena,
    20
);

define_payload_micro_maps!(callgrind_micro_maps_large_payload_insert_std_btreemap, callgrind_micro_maps_large_payload_contains_std_btreemap, BTreeMap<u64, LargePayload>, 1);
define_payload_micro_maps!(callgrind_micro_maps_large_payload_insert_std_hashmap, callgrind_micro_maps_large_payload_contains_std_hashmap, HashMap<u64, LargePayload>, 2);
define_payload_micro_maps!(
    callgrind_micro_maps_large_payload_insert_ads_bst_safe,
    callgrind_micro_maps_large_payload_contains_ads_bst_safe,
    common::PayloadBstSafe,
    3
);
define_payload_micro_maps!(
    callgrind_micro_maps_large_payload_insert_ads_bst_raw,
    callgrind_micro_maps_large_payload_contains_ads_bst_raw,
    common::PayloadBstRaw,
    4
);
define_payload_micro_maps!(
    callgrind_micro_maps_large_payload_insert_ads_bst_arena,
    callgrind_micro_maps_large_payload_contains_ads_bst_arena,
    common::PayloadBstArena,
    5
);
define_payload_micro_maps!(
    callgrind_micro_maps_large_payload_insert_ads_btree_safe_t8,
    callgrind_micro_maps_large_payload_contains_ads_btree_safe_t8,
    common::PayloadBtSafe,
    6
);
define_payload_micro_maps!(
    callgrind_micro_maps_large_payload_insert_ads_btree_raw_t8,
    callgrind_micro_maps_large_payload_contains_ads_btree_raw_t8,
    common::PayloadBtRaw,
    7
);
define_payload_micro_maps!(
    callgrind_micro_maps_large_payload_insert_ads_btree_arena_t8,
    callgrind_micro_maps_large_payload_contains_ads_btree_arena_t8,
    common::PayloadBtArena,
    8
);
define_payload_micro_maps!(
    callgrind_micro_maps_large_payload_insert_ads_avl_safe,
    callgrind_micro_maps_large_payload_contains_ads_avl_safe,
    common::PayloadAvlSafe,
    9
);
define_payload_micro_maps!(
    callgrind_micro_maps_large_payload_insert_ads_avl_raw,
    callgrind_micro_maps_large_payload_contains_ads_avl_raw,
    common::PayloadAvlRaw,
    10
);
define_payload_micro_maps!(
    callgrind_micro_maps_large_payload_insert_ads_avl_arena,
    callgrind_micro_maps_large_payload_contains_ads_avl_arena,
    common::PayloadAvlArena,
    11
);
define_payload_micro_maps!(
    callgrind_micro_maps_large_payload_insert_ads_rbt_safe,
    callgrind_micro_maps_large_payload_contains_ads_rbt_safe,
    common::PayloadRbSafe,
    12
);
define_payload_micro_maps!(
    callgrind_micro_maps_large_payload_insert_ads_rbt_raw,
    callgrind_micro_maps_large_payload_contains_ads_rbt_raw,
    common::PayloadRbRaw,
    13
);
define_payload_micro_maps!(
    callgrind_micro_maps_large_payload_insert_ads_rbt_arena,
    callgrind_micro_maps_large_payload_contains_ads_rbt_arena,
    common::PayloadRbArena,
    14
);
define_payload_micro_maps!(
    callgrind_micro_maps_large_payload_insert_ads_splay_safe,
    callgrind_micro_maps_large_payload_contains_ads_splay_safe,
    common::PayloadSplaySafe,
    15
);
define_payload_micro_maps!(
    callgrind_micro_maps_large_payload_insert_ads_splay_raw,
    callgrind_micro_maps_large_payload_contains_ads_splay_raw,
    common::PayloadSplayRaw,
    16
);
define_payload_micro_maps!(
    callgrind_micro_maps_large_payload_insert_ads_splay_arena,
    callgrind_micro_maps_large_payload_contains_ads_splay_arena,
    common::PayloadSplayArena,
    17
);
define_payload_micro_contains_adaptive!(
    callgrind_micro_maps_large_payload_contains_ads_splay_safe_adaptive,
    common::PayloadSplaySafe,
    215
);
define_payload_micro_contains_adaptive!(
    callgrind_micro_maps_large_payload_contains_ads_splay_raw_adaptive,
    common::PayloadSplayRaw,
    216
);
define_payload_micro_contains_adaptive!(
    callgrind_micro_maps_large_payload_contains_ads_splay_arena_adaptive,
    common::PayloadSplayArena,
    217
);
define_payload_micro_maps!(
    callgrind_micro_maps_large_payload_insert_ads_skip_safe,
    callgrind_micro_maps_large_payload_contains_ads_skip_safe,
    common::PayloadSkipSafe,
    18
);
define_payload_micro_maps!(
    callgrind_micro_maps_large_payload_insert_ads_skip_raw,
    callgrind_micro_maps_large_payload_contains_ads_skip_raw,
    common::PayloadSkipRaw,
    19
);
define_payload_micro_maps!(
    callgrind_micro_maps_large_payload_insert_ads_skip_arena,
    callgrind_micro_maps_large_payload_contains_ads_skip_arena,
    common::PayloadSkipArena,
    20
);

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_btree_cache_insert_ads_btree_safe_t4(size: usize) -> usize {
    run_u64_insert::<common::BtSafeDeg4>(size, 0x13000 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_btree_cache_insert_ads_btree_safe_t16(size: usize) -> usize {
    run_u64_insert::<common::BtSafeDeg16>(size, 0x13100 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_btree_cache_insert_ads_btree_safe_t64(size: usize) -> usize {
    run_u64_insert::<common::BtSafeDeg64>(size, 0x13200 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_btree_cache_contains_ads_btree_safe_t4(size: usize) -> usize {
    run_u64_contains::<common::BtSafeDeg4>(size, 0x13300 + size as u64, 0x13380 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_btree_cache_contains_ads_btree_safe_t16(size: usize) -> usize {
    run_u64_contains::<common::BtSafeDeg16>(size, 0x13400 + size as u64, 0x13480 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_btree_cache_contains_ads_btree_safe_t64(size: usize) -> usize {
    run_u64_contains::<common::BtSafeDeg64>(size, 0x13500 + size as u64, 0x13580 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_btree_cache_insert_ads_btree_raw_t4(size: usize) -> usize {
    run_u64_insert::<common::BtRawDeg4>(size, 0x13590 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_btree_cache_insert_ads_btree_raw_t16(size: usize) -> usize {
    run_u64_insert::<common::BtRawDeg16>(size, 0x135A0 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_btree_cache_insert_ads_btree_raw_t64(size: usize) -> usize {
    run_u64_insert::<common::BtRawDeg64>(size, 0x135B0 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_btree_cache_insert_ads_btree_arena_t4(size: usize) -> usize {
    run_u64_insert::<common::BtArenaDeg4>(size, 0x135C0 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_btree_cache_insert_ads_btree_arena_t16(size: usize) -> usize {
    run_u64_insert::<common::BtArenaDeg16>(size, 0x135D0 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_btree_cache_insert_ads_btree_arena_t64(size: usize) -> usize {
    run_u64_insert::<common::BtArenaDeg64>(size, 0x135E0 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_btree_cache_contains_ads_btree_raw_t4(size: usize) -> usize {
    run_u64_contains::<common::BtRawDeg4>(size, 0x135F0 + size as u64, 0x135F8 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_btree_cache_contains_ads_btree_raw_t16(size: usize) -> usize {
    run_u64_contains::<common::BtRawDeg16>(size, 0x13600 + size as u64, 0x13608 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_btree_cache_contains_ads_btree_raw_t64(size: usize) -> usize {
    run_u64_contains::<common::BtRawDeg64>(size, 0x13610 + size as u64, 0x13618 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_btree_cache_contains_ads_btree_arena_t4(size: usize) -> usize {
    run_u64_contains::<common::BtArenaDeg4>(size, 0x13620 + size as u64, 0x13628 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_btree_cache_contains_ads_btree_arena_t16(size: usize) -> usize {
    run_u64_contains::<common::BtArenaDeg16>(size, 0x13630 + size as u64, 0x13638 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_btree_cache_contains_ads_btree_arena_t64(size: usize) -> usize {
    run_u64_contains::<common::BtArenaDeg64>(size, 0x13640 + size as u64, 0x13648 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_hash_collisions_insert_std_hashmap(size: usize) -> usize {
    run_u64_insert::<HashMap<u64, u64>>(size, 0x13610 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_hash_collisions_insert_hashmap_zero_hasher(size: usize) -> usize {
    run_collision_insert(size, 0x13600 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_hash_collisions_contains_hashmap_zero_hasher(size: usize) -> usize {
    run_collision_contains(size, 0x13700 + size as u64, 0x13780 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_sweep_hash_collisions_contains_std_btreemap_reference(size: usize) -> usize {
    let keys = uniform_keys(size, 0x13800 + size as u64);
    let queries = temporal_locality_queries(size, size, 0x13880 + size as u64);
    black_box(common::map_contains_bench::<BTreeMap<u64, u64>>(
        &keys, &queries,
    ))
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_micro_heaps_push_pop_std_binary_heap(size: usize) -> usize {
    run_heap_push_pop::<common::StdBinaryHeapMin>(size, 0x13900 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_micro_heaps_push_pop_ads_binary_arena(size: usize) -> usize {
    run_heap_push_pop::<common::BinaryArena>(size, 0x13A00 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_micro_heaps_push_pop_ads_binary_safe(size: usize) -> usize {
    run_heap_push_pop::<common::BinarySafe>(size, 0x13A10 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_micro_heaps_push_pop_ads_binary_raw(size: usize) -> usize {
    run_heap_push_pop::<common::BinaryRaw>(size, 0x13A15 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_micro_heaps_push_pop_ads_binomial_arena(size: usize) -> usize {
    run_heap_push_pop::<common::BinomialArena>(size, 0x13A20 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_micro_heaps_push_pop_ads_fibonacci_arena(size: usize) -> usize {
    run_heap_push_pop::<common::FibonacciArena>(size, 0x13A30 + size as u64)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_micro_dsu_union_find_ads_dsu_safe(size: usize) -> usize {
    run_dsu_workload::<common::DsuSafe>(size)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_micro_dsu_union_find_ads_dsu_raw(size: usize) -> usize {
    run_dsu_workload::<common::DsuRaw>(size)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_micro_dsu_union_find_ads_dsu_arena(size: usize) -> usize {
    run_dsu_workload::<common::DsuArena>(size)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_micro_sequences_push_pop_std_vec(size: usize) -> usize {
    run_sequence_vec(size)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_micro_sequences_push_pop_std_vecdeque(size: usize) -> usize {
    run_sequence_vecdeque(size)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_micro_sequences_push_pop_std_linked_list(size: usize) -> usize {
    run_sequence_linked_list(size)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_micro_sequences_push_pop_ads_singly_safe(size: usize) -> usize {
    run_ads_sequence::<SinglySafe>(size)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_micro_sequences_push_pop_ads_singly_raw(size: usize) -> usize {
    run_ads_sequence::<SinglyRaw>(size)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_micro_sequences_push_pop_ads_singly_arena(size: usize) -> usize {
    run_ads_sequence::<SinglyArena>(size)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_micro_sequences_push_pop_ads_doubly_safe(size: usize) -> usize {
    run_ads_sequence::<DoublySafe>(size)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_micro_sequences_push_pop_ads_doubly_raw(size: usize) -> usize {
    run_ads_sequence::<DoublyRaw>(size)
}

#[library_benchmark]
#[bench::n1k(1_000)]
#[bench::n10k(10_000)]
pub fn callgrind_micro_sequences_push_pop_ads_doubly_arena(size: usize) -> usize {
    run_ads_sequence::<DoublyArena>(size)
}

#[library_benchmark]
#[bench::n100(100)]
#[bench::n1k(1_000)]
pub fn callgrind_micro_sequences_indexing_index_ads_singly_safe(size: usize) -> usize {
    run_sequence_indexing::<SinglySafe>(size)
}

#[library_benchmark]
#[bench::n100(100)]
#[bench::n1k(1_000)]
pub fn callgrind_micro_sequences_indexing_index_ads_doubly_safe(size: usize) -> usize {
    run_sequence_indexing::<DoublySafe>(size)
}

#[library_benchmark]
#[bench::n100(100)]
#[bench::n1k(1_000)]
pub fn callgrind_micro_sequences_indexing_index_ads_skip_arena(size: usize) -> usize {
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
    black_box(checksum)
}
