#![allow(dead_code)]

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap as StdBinaryHeap, HashMap};
use std::hash::{BuildHasherDefault, Hasher};

use ads::traits::core::{
    DisjointSet as AdsDisjointSet, Map as AdsMap, PriorityQueue as AdsPriorityQueue,
};
use criterion::black_box;

use crate::generators::MapOp;
use crate::generators::payloads::LargePayload;

#[derive(Clone, Default)]
pub struct ZeroHasher(u64);

impl Hasher for ZeroHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _bytes: &[u8]) {
        self.0 = 0;
    }

    fn write_u64(&mut self, _i: u64) {
        self.0 = 0;
    }
}

pub type CollidingHashMap<K, V> = HashMap<K, V, BuildHasherDefault<ZeroHasher>>;

pub type BstSafe = ads::trees::binary_search_tree::safe::BinarySearchTree<u64, u64>;
pub type BstRaw = ads::trees::binary_search_tree::raw::BinarySearchTree<u64, u64>;
pub type BstArena = ads::trees::binary_search_tree::arena::BinarySearchTree<u64, u64>;

pub type AvlSafe = ads::trees::avl_tree::safe::AvlTree<u64, u64>;
pub type AvlRaw = ads::trees::avl_tree::raw::AvlTree<u64, u64>;
pub type AvlArena = ads::trees::avl_tree::arena::AvlTree<u64, u64>;

pub type RbSafe = ads::trees::red_black_tree::safe::RedBlackTree<u64, u64>;
pub type RbRaw = ads::trees::red_black_tree::raw::RedBlackTree<u64, u64>;
pub type RbArena = ads::trees::red_black_tree::arena::RedBlackTree<u64, u64>;

pub type BtSafe = ads::trees::b_tree::safe::BTree<u64, u64, 8>;
pub type BtRaw = ads::trees::b_tree::raw::BTree<u64, u64, 8>;
pub type BtArena = ads::trees::b_tree::arena::BTree<u64, u64, 8>;

pub type SplaySafe = ads::trees::splay_tree::safe::SplayTree<u64, u64>;
pub type SplayRaw = ads::trees::splay_tree::raw::SplayTree<u64, u64>;
pub type SplayArena = ads::trees::splay_tree::arena::SplayTree<u64, u64>;

pub type SkipSafe = ads::linked::skip_list::safe::SkipList<u64, u64>;
pub type SkipRaw = ads::linked::skip_list::raw::SkipList<u64, u64>;
pub type SkipArena = ads::linked::skip_list::arena::SkipList<u64, u64>;

pub type BtSafeDeg4 = ads::trees::b_tree::safe::BTree<u64, u64, 4>;
pub type BtRawDeg4 = ads::trees::b_tree::raw::BTree<u64, u64, 4>;
pub type BtArenaDeg4 = ads::trees::b_tree::arena::BTree<u64, u64, 4>;

pub type BtSafeDeg16 = ads::trees::b_tree::safe::BTree<u64, u64, 16>;
pub type BtRawDeg16 = ads::trees::b_tree::raw::BTree<u64, u64, 16>;
pub type BtArenaDeg16 = ads::trees::b_tree::arena::BTree<u64, u64, 16>;

pub type BtSafeDeg64 = ads::trees::b_tree::safe::BTree<u64, u64, 64>;
pub type BtRawDeg64 = ads::trees::b_tree::raw::BTree<u64, u64, 64>;
pub type BtArenaDeg64 = ads::trees::b_tree::arena::BTree<u64, u64, 64>;

pub type DsuSafe = ads::contiguous::disjoint_set::safe::DisjointSet<u64>;
pub type DsuRaw = ads::contiguous::disjoint_set::raw::DisjointSet<u64>;
pub type DsuArena = ads::contiguous::disjoint_set::arena::DisjointSet<u64>;

pub type BinarySafe = ads::contiguous::binary_heap::safe::BinaryHeap<u64>;
pub type BinaryRaw = ads::contiguous::binary_heap::raw::BinaryHeap<u64>;
pub type BinaryArena = ads::contiguous::binary_heap::arena::BinaryHeap<u64>;
pub type BinomialArena = ads::forests::binomial_heap::arena::BinomialHeap<u64>;
pub type FibonacciArena = ads::forests::fibonacci_heap::arena::FibonacciHeap<u64>;
pub type StdBinaryHeapMin = StdBinaryHeap<Reverse<u64>>;

pub type StrBstSafe = ads::trees::binary_search_tree::safe::BinarySearchTree<String, usize>;
pub type StrBstRaw = ads::trees::binary_search_tree::raw::BinarySearchTree<String, usize>;
pub type StrBstArena = ads::trees::binary_search_tree::arena::BinarySearchTree<String, usize>;

pub type StrAvlSafe = ads::trees::avl_tree::safe::AvlTree<String, usize>;
pub type StrAvlRaw = ads::trees::avl_tree::raw::AvlTree<String, usize>;
pub type StrAvlArena = ads::trees::avl_tree::arena::AvlTree<String, usize>;

pub type StrRbSafe = ads::trees::red_black_tree::safe::RedBlackTree<String, usize>;
pub type StrRbRaw = ads::trees::red_black_tree::raw::RedBlackTree<String, usize>;
pub type StrRbArena = ads::trees::red_black_tree::arena::RedBlackTree<String, usize>;

pub type StrBtSafe = ads::trees::b_tree::safe::BTree<String, usize, 8>;
pub type StrBtRaw = ads::trees::b_tree::raw::BTree<String, usize, 8>;
pub type StrBtArena = ads::trees::b_tree::arena::BTree<String, usize, 8>;

pub type StrSplaySafe = ads::trees::splay_tree::safe::SplayTree<String, usize>;
pub type StrSplayRaw = ads::trees::splay_tree::raw::SplayTree<String, usize>;
pub type StrSplayArena = ads::trees::splay_tree::arena::SplayTree<String, usize>;

pub type StrSkipSafe = ads::linked::skip_list::safe::SkipList<String, usize>;
pub type StrSkipRaw = ads::linked::skip_list::raw::SkipList<String, usize>;
pub type StrSkipArena = ads::linked::skip_list::arena::SkipList<String, usize>;

pub type PayloadBstSafe = ads::trees::binary_search_tree::safe::BinarySearchTree<u64, LargePayload>;
pub type PayloadBstRaw = ads::trees::binary_search_tree::raw::BinarySearchTree<u64, LargePayload>;
pub type PayloadBstArena =
    ads::trees::binary_search_tree::arena::BinarySearchTree<u64, LargePayload>;

pub type PayloadAvlSafe = ads::trees::avl_tree::safe::AvlTree<u64, LargePayload>;
pub type PayloadAvlRaw = ads::trees::avl_tree::raw::AvlTree<u64, LargePayload>;
pub type PayloadAvlArena = ads::trees::avl_tree::arena::AvlTree<u64, LargePayload>;

pub type PayloadRbSafe = ads::trees::red_black_tree::safe::RedBlackTree<u64, LargePayload>;
pub type PayloadRbRaw = ads::trees::red_black_tree::raw::RedBlackTree<u64, LargePayload>;
pub type PayloadRbArena = ads::trees::red_black_tree::arena::RedBlackTree<u64, LargePayload>;

pub type PayloadBtSafe = ads::trees::b_tree::safe::BTree<u64, LargePayload, 8>;
pub type PayloadBtRaw = ads::trees::b_tree::raw::BTree<u64, LargePayload, 8>;
pub type PayloadBtArena = ads::trees::b_tree::arena::BTree<u64, LargePayload, 8>;

pub type PayloadSplaySafe = ads::trees::splay_tree::safe::SplayTree<u64, LargePayload>;
pub type PayloadSplayRaw = ads::trees::splay_tree::raw::SplayTree<u64, LargePayload>;
pub type PayloadSplayArena = ads::trees::splay_tree::arena::SplayTree<u64, LargePayload>;

pub type PayloadSkipSafe = ads::linked::skip_list::safe::SkipList<u64, LargePayload>;
pub type PayloadSkipRaw = ads::linked::skip_list::raw::SkipList<u64, LargePayload>;
pub type PayloadSkipArena = ads::linked::skip_list::arena::SkipList<u64, LargePayload>;

pub trait BenchMap {
    fn new() -> Self;
    fn insert_value(&mut self, key: u64, value: u64);
    fn contains_value(&self, key: &u64) -> bool;
    fn remove_value(&mut self, key: &u64);
    fn len(&self) -> usize;
    fn clear_value(&mut self);
}

pub trait BenchAdaptiveMap {
    fn new() -> Self;
    fn insert_value(&mut self, key: u64, value: u64);
    fn contains_adaptive_value(&mut self, key: &u64) -> bool;
    fn remove_value(&mut self, key: &u64);
    fn len(&self) -> usize;
}

impl BenchMap for BTreeMap<u64, u64> {
    fn new() -> Self {
        Self::new()
    }

    fn insert_value(&mut self, key: u64, value: u64) {
        let _ = self.insert(key, value);
    }

    fn contains_value(&self, key: &u64) -> bool {
        self.contains_key(key)
    }

    fn remove_value(&mut self, key: &u64) {
        let _ = self.remove(key);
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn clear_value(&mut self) {
        self.clear();
    }
}

impl BenchMap for HashMap<u64, u64> {
    fn new() -> Self {
        Self::new()
    }

    fn insert_value(&mut self, key: u64, value: u64) {
        let _ = self.insert(key, value);
    }

    fn contains_value(&self, key: &u64) -> bool {
        self.contains_key(key)
    }

    fn remove_value(&mut self, key: &u64) {
        let _ = self.remove(key);
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn clear_value(&mut self) {
        self.clear();
    }
}

macro_rules! impl_ads_bench_map {
    ($ty:ty) => {
        impl BenchMap for $ty {
            fn new() -> Self {
                Self::new()
            }

            fn insert_value(&mut self, key: u64, value: u64) {
                let _ = AdsMap::insert(self, key, value);
            }

            fn contains_value(&self, key: &u64) -> bool {
                AdsMap::contains_key(self, key)
            }

            fn remove_value(&mut self, key: &u64) {
                let _ = AdsMap::remove(self, key);
            }

            fn len(&self) -> usize {
                AdsMap::len(self)
            }

            fn clear_value(&mut self) {
                AdsMap::clear(self);
            }
        }
    };
}

impl_ads_bench_map!(BstSafe);
impl_ads_bench_map!(BstRaw);
impl_ads_bench_map!(BstArena);
impl_ads_bench_map!(AvlSafe);
impl_ads_bench_map!(AvlRaw);
impl_ads_bench_map!(AvlArena);
impl_ads_bench_map!(RbSafe);
impl_ads_bench_map!(RbRaw);
impl_ads_bench_map!(RbArena);
impl_ads_bench_map!(BtSafe);
impl_ads_bench_map!(BtRaw);
impl_ads_bench_map!(BtArena);
impl_ads_bench_map!(SplaySafe);
impl_ads_bench_map!(SplayRaw);
impl_ads_bench_map!(SplayArena);
impl_ads_bench_map!(BtSafeDeg4);
impl_ads_bench_map!(BtRawDeg4);
impl_ads_bench_map!(BtArenaDeg4);
impl_ads_bench_map!(BtSafeDeg16);
impl_ads_bench_map!(BtRawDeg16);
impl_ads_bench_map!(BtArenaDeg16);
impl_ads_bench_map!(BtSafeDeg64);
impl_ads_bench_map!(BtRawDeg64);
impl_ads_bench_map!(BtArenaDeg64);
impl_ads_bench_map!(SkipSafe);
impl_ads_bench_map!(SkipRaw);
impl_ads_bench_map!(SkipArena);

macro_rules! impl_splay_adaptive_bench_map {
    ($ty:ty) => {
        impl BenchAdaptiveMap for $ty {
            fn new() -> Self {
                Self::new()
            }

            fn insert_value(&mut self, key: u64, value: u64) {
                let _ = AdsMap::insert(self, key, value);
            }

            fn contains_adaptive_value(&mut self, key: &u64) -> bool {
                Self::contains_adaptive(self, key)
            }

            fn remove_value(&mut self, key: &u64) {
                let _ = AdsMap::remove(self, key);
            }

            fn len(&self) -> usize {
                AdsMap::len(self)
            }
        }
    };
}

impl_splay_adaptive_bench_map!(SplaySafe);
impl_splay_adaptive_bench_map!(SplayRaw);
impl_splay_adaptive_bench_map!(SplayArena);

#[allow(clippy::ptr_arg)]
pub trait BenchStringMap {
    fn new() -> Self;
    fn insert_value(&mut self, key: String, value: usize);
    fn contains_value(&self, key: &String) -> bool;
    fn len(&self) -> usize;
}

#[allow(clippy::ptr_arg)]
pub trait BenchAdaptiveStringMap {
    fn new() -> Self;
    fn insert_value(&mut self, key: String, value: usize);
    fn contains_adaptive_value(&mut self, key: &String) -> bool;
    fn len(&self) -> usize;
}

impl BenchStringMap for BTreeMap<String, usize> {
    fn new() -> Self {
        Self::new()
    }

    fn insert_value(&mut self, key: String, value: usize) {
        let _ = self.insert(key, value);
    }

    fn contains_value(&self, key: &String) -> bool {
        self.contains_key(key)
    }

    fn len(&self) -> usize {
        self.len()
    }
}

impl BenchStringMap for HashMap<String, usize> {
    fn new() -> Self {
        Self::new()
    }

    fn insert_value(&mut self, key: String, value: usize) {
        let _ = self.insert(key, value);
    }

    fn contains_value(&self, key: &String) -> bool {
        self.contains_key(key)
    }

    fn len(&self) -> usize {
        self.len()
    }
}

macro_rules! impl_string_map_bench {
    ($ty:ty) => {
        impl BenchStringMap for $ty {
            fn new() -> Self {
                Self::new()
            }

            fn insert_value(&mut self, key: String, value: usize) {
                let _ = AdsMap::insert(self, key, value);
            }

            fn contains_value(&self, key: &String) -> bool {
                AdsMap::contains_key(self, key)
            }

            fn len(&self) -> usize {
                AdsMap::len(self)
            }
        }
    };
}

impl_string_map_bench!(StrBstSafe);
impl_string_map_bench!(StrBstRaw);
impl_string_map_bench!(StrBstArena);
impl_string_map_bench!(StrAvlSafe);
impl_string_map_bench!(StrAvlRaw);
impl_string_map_bench!(StrAvlArena);
impl_string_map_bench!(StrRbSafe);
impl_string_map_bench!(StrRbRaw);
impl_string_map_bench!(StrRbArena);
impl_string_map_bench!(StrBtSafe);
impl_string_map_bench!(StrBtRaw);
impl_string_map_bench!(StrBtArena);
impl_string_map_bench!(StrSplaySafe);
impl_string_map_bench!(StrSplayRaw);
impl_string_map_bench!(StrSplayArena);
impl_string_map_bench!(StrSkipSafe);
impl_string_map_bench!(StrSkipRaw);
impl_string_map_bench!(StrSkipArena);

macro_rules! impl_splay_adaptive_string_map_bench {
    ($ty:ty) => {
        impl BenchAdaptiveStringMap for $ty {
            fn new() -> Self {
                Self::new()
            }

            fn insert_value(&mut self, key: String, value: usize) {
                let _ = AdsMap::insert(self, key, value);
            }

            fn contains_adaptive_value(&mut self, key: &String) -> bool {
                Self::contains_adaptive(self, key)
            }

            fn len(&self) -> usize {
                AdsMap::len(self)
            }
        }
    };
}

impl_splay_adaptive_string_map_bench!(StrSplaySafe);
impl_splay_adaptive_string_map_bench!(StrSplayRaw);
impl_splay_adaptive_string_map_bench!(StrSplayArena);

pub trait BenchPayloadMap {
    fn new() -> Self;
    fn insert_value(&mut self, key: u64, value: LargePayload);
    fn contains_value(&self, key: &u64) -> bool;
    fn len(&self) -> usize;
}

pub trait BenchAdaptivePayloadMap {
    fn new() -> Self;
    fn insert_value(&mut self, key: u64, value: LargePayload);
    fn contains_adaptive_value(&mut self, key: &u64) -> bool;
    fn len(&self) -> usize;
}

impl BenchPayloadMap for BTreeMap<u64, LargePayload> {
    fn new() -> Self {
        Self::new()
    }

    fn insert_value(&mut self, key: u64, value: LargePayload) {
        let _ = self.insert(key, value);
    }

    fn contains_value(&self, key: &u64) -> bool {
        self.contains_key(key)
    }

    fn len(&self) -> usize {
        self.len()
    }
}

impl BenchPayloadMap for HashMap<u64, LargePayload> {
    fn new() -> Self {
        Self::new()
    }

    fn insert_value(&mut self, key: u64, value: LargePayload) {
        let _ = self.insert(key, value);
    }

    fn contains_value(&self, key: &u64) -> bool {
        self.contains_key(key)
    }

    fn len(&self) -> usize {
        self.len()
    }
}

macro_rules! impl_payload_map_bench {
    ($ty:ty) => {
        impl BenchPayloadMap for $ty {
            fn new() -> Self {
                Self::new()
            }

            fn insert_value(&mut self, key: u64, value: LargePayload) {
                let _ = AdsMap::insert(self, key, value);
            }

            fn contains_value(&self, key: &u64) -> bool {
                AdsMap::contains_key(self, key)
            }

            fn len(&self) -> usize {
                AdsMap::len(self)
            }
        }
    };
}

impl_payload_map_bench!(PayloadBstSafe);
impl_payload_map_bench!(PayloadBstRaw);
impl_payload_map_bench!(PayloadBstArena);
impl_payload_map_bench!(PayloadAvlSafe);
impl_payload_map_bench!(PayloadAvlRaw);
impl_payload_map_bench!(PayloadAvlArena);
impl_payload_map_bench!(PayloadRbSafe);
impl_payload_map_bench!(PayloadRbRaw);
impl_payload_map_bench!(PayloadRbArena);
impl_payload_map_bench!(PayloadBtSafe);
impl_payload_map_bench!(PayloadBtRaw);
impl_payload_map_bench!(PayloadBtArena);
impl_payload_map_bench!(PayloadSplaySafe);
impl_payload_map_bench!(PayloadSplayRaw);
impl_payload_map_bench!(PayloadSplayArena);
impl_payload_map_bench!(PayloadSkipSafe);
impl_payload_map_bench!(PayloadSkipRaw);
impl_payload_map_bench!(PayloadSkipArena);

macro_rules! impl_splay_adaptive_payload_map_bench {
    ($ty:ty) => {
        impl BenchAdaptivePayloadMap for $ty {
            fn new() -> Self {
                Self::new()
            }

            fn insert_value(&mut self, key: u64, value: LargePayload) {
                let _ = AdsMap::insert(self, key, value);
            }

            fn contains_adaptive_value(&mut self, key: &u64) -> bool {
                Self::contains_adaptive(self, key)
            }

            fn len(&self) -> usize {
                AdsMap::len(self)
            }
        }
    };
}

impl_splay_adaptive_payload_map_bench!(PayloadSplaySafe);
impl_splay_adaptive_payload_map_bench!(PayloadSplayRaw);
impl_splay_adaptive_payload_map_bench!(PayloadSplayArena);

pub trait BenchPriorityQueue {
    fn new() -> Self;
    fn push_value(&mut self, value: u64);
    fn pop_value(&mut self) -> Option<u64>;
}

macro_rules! impl_ads_bench_pq {
    ($ty:ty) => {
        impl BenchPriorityQueue for $ty {
            fn new() -> Self {
                Self::new()
            }

            fn push_value(&mut self, value: u64) {
                AdsPriorityQueue::push(self, value);
            }

            fn pop_value(&mut self) -> Option<u64> {
                AdsPriorityQueue::pop(self)
            }
        }
    };
}

impl BenchPriorityQueue for BinarySafe {
    fn new() -> Self {
        Self::new()
    }

    fn push_value(&mut self, value: u64) {
        AdsPriorityQueue::push(self, value);
    }

    fn pop_value(&mut self) -> Option<u64> {
        AdsPriorityQueue::pop(self)
    }
}

impl_ads_bench_pq!(BinaryRaw);
impl_ads_bench_pq!(BinaryArena);
impl_ads_bench_pq!(BinomialArena);
impl_ads_bench_pq!(FibonacciArena);

impl BenchPriorityQueue for StdBinaryHeapMin {
    fn new() -> Self {
        Self::new()
    }

    fn push_value(&mut self, value: u64) {
        self.push(Reverse(value));
    }

    fn pop_value(&mut self) -> Option<u64> {
        self.pop().map(|x| x.0)
    }
}

pub trait BenchDisjointSet {
    fn new() -> Self;
    fn make_set_value(&mut self, value: u64);
    fn union_values(&mut self, left: &u64, right: &u64) -> bool;
    fn same_set_values(&mut self, left: &u64, right: &u64) -> bool;
    fn find_value(&mut self, value: &u64) -> bool;
}

impl BenchDisjointSet for DsuSafe {
    fn new() -> Self {
        Self::new()
    }

    fn make_set_value(&mut self, value: u64) {
        let _ = AdsDisjointSet::make_set(self, value);
    }

    fn union_values(&mut self, left: &u64, right: &u64) -> bool {
        AdsDisjointSet::union(self, left, right)
    }

    fn same_set_values(&mut self, left: &u64, right: &u64) -> bool {
        AdsDisjointSet::same_set(self, left, right)
    }

    fn find_value(&mut self, value: &u64) -> bool {
        AdsDisjointSet::find(self, value).is_some()
    }
}

impl BenchDisjointSet for DsuRaw {
    fn new() -> Self {
        Self::new()
    }

    fn make_set_value(&mut self, value: u64) {
        let _ = AdsDisjointSet::make_set(self, value);
    }

    fn union_values(&mut self, left: &u64, right: &u64) -> bool {
        AdsDisjointSet::union(self, left, right)
    }

    fn same_set_values(&mut self, left: &u64, right: &u64) -> bool {
        AdsDisjointSet::same_set(self, left, right)
    }

    fn find_value(&mut self, value: &u64) -> bool {
        AdsDisjointSet::find(self, value).is_some()
    }
}

impl BenchDisjointSet for DsuArena {
    fn new() -> Self {
        Self::new()
    }

    fn make_set_value(&mut self, value: u64) {
        let _ = AdsDisjointSet::make_set(self, value);
    }

    fn union_values(&mut self, left: &u64, right: &u64) -> bool {
        AdsDisjointSet::union(self, left, right)
    }

    fn same_set_values(&mut self, left: &u64, right: &u64) -> bool {
        AdsDisjointSet::same_set(self, left, right)
    }

    fn find_value(&mut self, value: &u64) -> bool {
        AdsDisjointSet::find(self, value).is_some()
    }
}

pub fn colliding_hasher_map<K, V>() -> CollidingHashMap<K, V> {
    HashMap::with_hasher(BuildHasherDefault::default())
}

pub fn map_insert_bench<M: BenchMap>(keys: &[u64]) -> usize {
    let mut map = M::new();
    for &key in keys {
        map.insert_value(black_box(key), black_box(key ^ 0x9E37_79B9));
    }
    black_box(map.len())
}

pub fn map_contains_bench<M: BenchMap>(keys: &[u64], queries: &[u64]) -> usize {
    let mut map = M::new();
    for &key in keys {
        map.insert_value(key, key ^ 0x9E37_79B9);
    }

    let mut hits = 0usize;
    for &query in queries {
        if map.contains_value(black_box(&query)) {
            hits = hits.wrapping_add(1);
        }
    }
    black_box(hits)
}

pub fn map_contains_adaptive_bench<M: BenchAdaptiveMap>(keys: &[u64], queries: &[u64]) -> usize {
    let mut map = M::new();
    for &key in keys {
        map.insert_value(key, key ^ 0x9E37_79B9);
    }

    let mut hits = 0usize;
    for &query in queries {
        if map.contains_adaptive_value(black_box(&query)) {
            hits = hits.wrapping_add(1);
        }
    }
    black_box(hits)
}

pub fn map_remove_bench<M: BenchMap>(keys: &[u64]) -> usize {
    let mut map = M::new();
    for &key in keys {
        map.insert_value(key, key);
    }

    for key in keys {
        map.remove_value(black_box(key));
    }
    black_box(map.len())
}

pub fn map_mixed_ops_bench<M: BenchMap>(prefill_keys: &[u64], ops: &[MapOp]) -> usize {
    let mut map = M::new();
    for &key in prefill_keys {
        map.insert_value(key, key);
    }

    let mut checksum = 0usize;
    for op in ops {
        match *op {
            MapOp::Contains(key) => {
                if map.contains_value(black_box(&key)) {
                    checksum = checksum.wrapping_add(1);
                }
            }
            MapOp::Upsert(key, value) => {
                map.insert_value(black_box(key), black_box(value));
            }
            MapOp::Remove(key) => {
                map.remove_value(black_box(&key));
            }
        }
    }

    black_box(checksum ^ map.len())
}

pub fn map_mixed_ops_adaptive_bench<M: BenchAdaptiveMap>(
    prefill_keys: &[u64],
    ops: &[MapOp],
) -> usize {
    let mut map = M::new();
    for &key in prefill_keys {
        map.insert_value(key, key);
    }

    let mut checksum = 0usize;
    for op in ops {
        match *op {
            MapOp::Contains(key) => {
                if map.contains_adaptive_value(black_box(&key)) {
                    checksum = checksum.wrapping_add(1);
                }
            }
            MapOp::Upsert(key, value) => {
                map.insert_value(black_box(key), black_box(value));
            }
            MapOp::Remove(key) => {
                map.remove_value(black_box(&key));
            }
        }
    }

    black_box(checksum ^ map.len())
}

pub fn map_thrashing_bench<M: BenchMap>(
    prefill: &[u64],
    remove_keys: &[u64],
    insert_keys: &[u64],
) -> usize {
    let mut map = M::new();
    for &key in prefill {
        map.insert_value(key, key);
    }

    let mut checksum = 0usize;
    for step in 0..remove_keys.len() {
        let remove_key = remove_keys[step];
        let insert_key = insert_keys[step];

        if step % 2 == 0 {
            map.remove_value(black_box(&remove_key));
            map.insert_value(black_box(insert_key), black_box(insert_key ^ 0xAAAA_5555));
        } else {
            map.insert_value(black_box(insert_key), black_box(insert_key ^ 0x5555_AAAA));
            map.remove_value(black_box(&remove_key));
        }

        checksum = checksum.wrapping_add(map.len());
    }

    black_box(checksum)
}

pub fn string_map_insert_bench<M: BenchStringMap>(keys: &[String]) -> usize {
    let mut map = M::new();
    for (index, key) in keys.iter().enumerate() {
        map.insert_value(black_box(key.clone()), black_box(index));
    }
    black_box(map.len())
}

pub fn string_map_contains_bench<M: BenchStringMap>(keys: &[String], queries: &[String]) -> usize {
    let mut map = M::new();
    for (index, key) in keys.iter().enumerate() {
        map.insert_value(key.clone(), index);
    }

    let mut hits = 0usize;
    for query in queries {
        if map.contains_value(black_box(query)) {
            hits = hits.wrapping_add(1);
        }
    }

    black_box(hits)
}

pub fn string_map_contains_adaptive_bench<M: BenchAdaptiveStringMap>(
    keys: &[String],
    queries: &[String],
) -> usize {
    let mut map = M::new();
    for (index, key) in keys.iter().enumerate() {
        map.insert_value(key.clone(), index);
    }

    let mut hits = 0usize;
    for query in queries {
        if map.contains_adaptive_value(black_box(query)) {
            hits = hits.wrapping_add(1);
        }
    }

    black_box(hits)
}

pub fn payload_map_insert_bench<M: BenchPayloadMap>(keys: &[u64]) -> usize {
    let mut map = M::new();
    for &key in keys {
        map.insert_value(black_box(key), black_box(LargePayload::new(key)));
    }
    black_box(map.len())
}

pub fn payload_map_contains_bench<M: BenchPayloadMap>(keys: &[u64], queries: &[u64]) -> usize {
    let mut map = M::new();
    for &key in keys {
        map.insert_value(key, LargePayload::new(key));
    }

    let mut hits = 0usize;
    for query in queries {
        if map.contains_value(black_box(query)) {
            hits = hits.wrapping_add(1);
        }
    }
    black_box(hits)
}

pub fn payload_map_contains_adaptive_bench<M: BenchAdaptivePayloadMap>(
    keys: &[u64],
    queries: &[u64],
) -> usize {
    let mut map = M::new();
    for &key in keys {
        map.insert_value(key, LargePayload::new(key));
    }

    let mut hits = 0usize;
    for query in queries {
        if map.contains_adaptive_value(black_box(query)) {
            hits = hits.wrapping_add(1);
        }
    }
    black_box(hits)
}

pub fn heap_push_pop_bench<P: BenchPriorityQueue>(input: &[u64]) -> usize {
    let mut pq = P::new();
    for &value in input {
        pq.push_value(black_box(value));
    }

    let mut checksum = 0usize;
    while let Some(value) = pq.pop_value() {
        checksum = checksum.wrapping_add(value as usize);
    }
    black_box(checksum)
}

pub fn dsu_workload<D: BenchDisjointSet>(size: usize) -> usize {
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
