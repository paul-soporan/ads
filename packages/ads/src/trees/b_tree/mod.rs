pub mod arena;
pub mod raw;
pub mod safe;

use crate::traits::{core::OrderedMap, diagnostics::TreeDiagnostics};

pub trait BTreeVariant<K, V>: OrderedMap<K, V> + TreeDiagnostics {
    type NodeView<'a>: Clone
    where
        Self: 'a;

    fn degree(&self) -> usize;
    fn root_view<'a>(&'a self) -> Option<Self::NodeView<'a>>;
    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a>;
}

impl<K: Ord, V, const T: usize> BTreeVariant<K, V> for safe::BTree<K, V, T> {
    type NodeView<'a>
        = safe::BTreeNodeView<K, V>
    where
        Self: 'a;

    fn degree(&self) -> usize {
        self.degree()
    }

    fn root_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.root_view()
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a> {
        cursor.node_view()
    }
}

impl<K: Ord, V, const T: usize> BTreeVariant<K, V> for raw::BTree<K, V, T> {
    type NodeView<'a>
        = raw::BTreeNodeView<K, V, T>
    where
        Self: 'a;

    fn degree(&self) -> usize {
        self.degree()
    }

    fn root_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.root_view()
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a> {
        cursor.node_view()
    }
}

impl<K: Ord, V, const T: usize> BTreeVariant<K, V> for arena::BTree<K, V, T> {
    type NodeView<'a>
        = arena::BTreeNodeView<'a, K, V, T>
    where
        Self: 'a;

    fn degree(&self) -> usize {
        self.degree()
    }

    fn root_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.root_view()
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a> {
        cursor.node_view()
    }
}

#[cfg(test)]
trait BTreeNodeViewLike<K>: Clone {
    fn keys_vec(&self) -> Vec<K>
    where
        K: Clone;

    fn is_leaf_node(&self) -> bool;
    fn children_vec(&self) -> Vec<Self>;
    fn has_parent(&self) -> bool;
}

#[cfg(test)]
impl<K: Clone, V: Clone> BTreeNodeViewLike<K> for safe::BTreeNodeView<K, V> {
    fn keys_vec(&self) -> Vec<K>
    where
        K: Clone,
    {
        self.keys()
    }

    fn is_leaf_node(&self) -> bool {
        self.is_leaf()
    }

    fn children_vec(&self) -> Vec<Self> {
        self.children()
    }

    fn has_parent(&self) -> bool {
        self.parent().is_some()
    }
}

#[cfg(test)]
impl<K: Clone, V: Clone, const T: usize> BTreeNodeViewLike<K> for raw::BTreeNodeView<K, V, T> {
    fn keys_vec(&self) -> Vec<K>
    where
        K: Clone,
    {
        self.keys()
    }

    fn is_leaf_node(&self) -> bool {
        self.is_leaf()
    }

    fn children_vec(&self) -> Vec<Self> {
        self.children()
    }

    fn has_parent(&self) -> bool {
        self.parent().is_some()
    }
}

#[cfg(test)]
impl<'a, K: Clone, V: Clone, const T: usize> BTreeNodeViewLike<K>
    for arena::BTreeNodeView<'a, K, V, T>
{
    fn keys_vec(&self) -> Vec<K>
    where
        K: Clone,
    {
        self.keys()
    }

    fn is_leaf_node(&self) -> bool {
        self.is_leaf()
    }

    fn children_vec(&self) -> Vec<Self> {
        self.children()
    }

    fn has_parent(&self) -> bool {
        self.parent().is_some()
    }
}

#[cfg(test)]
fn assert_btree_properties<K, N>(root: Option<N>, degree: usize)
where
    K: Ord + Clone + std::fmt::Debug,
    N: BTreeNodeViewLike<K>,
{
    if let Some(root) = root {
        check_node::<K, N>(root, degree, true, None, None);
    }
}

#[cfg(test)]
fn check_node<K, N>(node: N, t: usize, is_root: bool, min: Option<K>, max: Option<K>) -> usize
where
    K: Ord + Clone + std::fmt::Debug,
    N: BTreeNodeViewLike<K>,
{
    let keys = node.keys_vec();
    let key_count = keys.len();

    if !is_root {
        assert!(key_count >= t - 1, "node underflow: expected >= {}, found {}", t-1, key_count);
    }
    assert!(key_count <= 2 * t - 1, "node overflow: expected <= {}, found {}", 2*t-1, key_count);

    for i in 0..key_count {
        if let Some(ref m) = min {
            assert!(keys[i] > *m, "key {:?} must be greater than min {:?}", keys[i], m);
        }
        if let Some(ref m) = max {
            assert!(keys[i] < *m, "key {:?} must be less than max {:?}", keys[i], m);
        }
        if i > 0 {
            assert!(keys[i-1] < keys[i], "keys must be strictly increasing and unique (in this test)");
        }
    }

    let children = node.children_vec();
    let mut total_count = key_count;

    if node.is_leaf_node() {
        assert!(children.is_empty(), "leaf nodes must not have children");
    } else {
        assert_eq!(children.len(), key_count + 1, "invalid child count");
        for i in 0..children.len() {
            let child_min = if i == 0 { min.clone() } else { Some(keys[i-1].clone()) };
            let child_max = if i == key_count { max.clone() } else { Some(keys[i].clone()) };
            
            assert!(children[i].has_parent(), "child must have a parent");
            total_count += check_node::<K, N>(children[i].clone(), t, false, child_min, child_max);
        }
    }
    total_count
}

#[cfg(test)]
macro_rules! test_b_tree_variant {
    ($module:ident, $tree_ty:ident, $t:expr) => {
        mod $module {
            use super::*;
            use std::collections::BTreeMap;
            use rand::{Rng, SeedableRng, rngs::StdRng};
            use std::sync::{Arc, atomic::{AtomicUsize, Ordering as AtomicOrdering}};

            use crate::traits::core::Map;
            use crate::traits::diagnostics::TreeDiagnostics;

            #[derive(Debug, Clone)]
            struct CountedKey {
                value: i32,
                comparisons: Arc<AtomicUsize>,
            }

            impl CountedKey {
                fn new(value: i32, comparisons: &Arc<AtomicUsize>) -> Self {
                    Self { value, comparisons: comparisons.clone() }
                }
            }

            impl Eq for CountedKey {}
            impl PartialEq for CountedKey {
                fn eq(&self, other: &Self) -> bool { self.value == other.value }
            }

            impl Ord for CountedKey {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    self.comparisons.fetch_add(1, AtomicOrdering::Relaxed);
                    self.value.cmp(&other.value)
                }
            }

            impl PartialOrd for CountedKey {
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }

            type Tree = $tree_ty::BTree<i32, i32, $t>;
            type ComplexityTree = $tree_ty::BTree<CountedKey, i32, $t>;

            #[test]
            fn empty_tree() {
                let tree = Tree::new();
                assert!(tree.min_cursor().is_none());
                assert!(tree.max_cursor().is_none());
                assert!(!tree.contains_key(&5));
                assert_eq!(tree.height(), 0);
                assert_eq!(tree.node_count(), 0);
                assert_btree_properties(tree.root_view(), tree.degree());
            }

            #[test]
            fn insert_contains_and_boundaries() {
                let mut tree = Tree::new();
                for value in [10, 20, 5, 6, 12] {
                    tree.insert(value, value * 10);
                }

                assert!(tree.contains_key(&10));
                assert!(tree.contains_key(&12));
                assert!(!tree.contains_key(&100));
                assert_eq!(*tree.min_cursor().expect("min").key(), 5);
                assert_eq!(*tree.max_cursor().expect("max").key(), 20);
                assert_btree_properties(tree.root_view(), tree.degree());
            }

            #[test]
            fn predecessor_successor() {
                let mut tree = Tree::new();
                for value in [10, 20, 30, 40, 50, 60, 70, 80, 90] {
                    tree.insert(value, value);
                }

                let c30 = tree.cursor(&30).expect("c30");
                let c40 = tree.cursor(&40).expect("c40");
                let c90 = tree.cursor(&90).expect("c90");
                let c10 = tree.cursor(&10).expect("c10");

                assert_eq!(*c30.successor().expect("succ").key(), 40);
                assert_eq!(*c40.predecessor().expect("pred").key(), 30);
                assert!(c90.successor().is_none());
                assert!(c10.predecessor().is_none());
                assert_btree_properties(tree.root_view(), tree.degree());
            }

            #[test]
            fn delete_leaf_and_internal() {
                let mut tree = Tree::new();
                for value in [10, 20, 30, 40, 50, 60] {
                    tree.insert(value, value);
                }

                assert!(tree.remove(&40).is_some());
                assert_btree_properties(tree.root_view(), tree.degree());
                assert!(tree.remove(&20).is_some());
                assert_btree_properties(tree.root_view(), tree.degree());
                assert!(tree.remove(&200).is_none());
                assert_btree_properties(tree.root_view(), tree.degree());
            }

            #[test]
            fn stress_random_operations() {
                let mut tree = Tree::new();
                let mut model = BTreeMap::new();
                let mut rng = StdRng::seed_from_u64(42);

                for _ in 0..1000 {
                    match rng.gen_range(0..4) {
                        0 => { // Insert
                            let k = rng.gen_range(0..500);
                            let v = rng.gen_range(0..1000);
                            assert_eq!(tree.insert(k, v), model.insert(k, v));
                        }
                        1 if !model.is_empty() => { // Remove
                            let keys: Vec<_> = model.keys().cloned().collect();
                            let k = keys[rng.gen_range(0..keys.len())];
                            assert_eq!(tree.remove(&k), model.remove(&k));
                        }
                        2 if !model.is_empty() => { // Search
                            let keys: Vec<_> = model.keys().cloned().collect();
                            let k = keys[rng.gen_range(0..keys.len())];
                            assert!(tree.contains_key(&k));
                        }
                        3 => { // Search non-existent
                            let k = rng.gen_range(500..1000);
                            assert!(!tree.contains_key(&k));
                        }
                        _ => {}
                    }
                    if rng.gen_bool(0.05) {
                        assert_btree_properties(tree.root_view(), tree.degree());
                    }
                }

                let actual: Vec<_> = (&tree).into_iter().collect();
                let expected: Vec<_> = model.into_iter().collect();
                assert_eq!(actual, expected);
            }

            #[test]
            fn complexity_scaling_is_logarithmic() {
                fn run_case(size: i32) -> usize {
                    let comparisons = Arc::new(AtomicUsize::new(0));
                    let mut tree = ComplexityTree::new();
                    for value in 0..size {
                        tree.insert(CountedKey::new(value, &comparisons), value);
                    }

                    comparisons.store(0, AtomicOrdering::Relaxed);
                    let trials = 100;
                    let mut rng = StdRng::seed_from_u64(42);
                    for _ in 0..trials {
                        let value = rng.gen_range(0..size);
                        let key = CountedKey::new(value, &comparisons);
                        assert!(tree.contains_key(&key));
                    }
                    comparisons.load(AtomicOrdering::Relaxed) / trials
                }

                let small = run_case(128);
                let large = run_case(1024);

                // Average comparisons in B-Tree search is ~ log_t(N) * binary_search_in_node.
                // For t=2, it's roughly log2(N).
                // log2(128) = 7, log2(1024) = 10.
                assert!(large <= small + 8, "average search cost grew too quickly ({} -> {})", small, large);
            }

            #[test]
            fn height_scales_logarithmically() {
                fn run_case(size: i32) -> usize {
                    let mut tree = Tree::new();
                    for value in 0..size {
                        tree.insert(value, value);
                    }
                    tree.height()
                }

                let small = run_case(128);
                let large = run_case(1024);

                // Height of B-Tree is <= log_t((N+1)/2) + 1.
                // For t=2, log2(64.5) + 1 ~ 7.
                // For t=3, log3(64.5) + 1 ~ 4.8.
                assert!(small <= 8, "height at size 128 is too large: {}", small);
                assert!(large <= 12, "height at size 1024 is too large: {}", large);
            }
        }
    };
}

#[cfg(test)]
test_b_tree_variant!(safe_variant_t2, safe, 2);
#[cfg(test)]
test_b_tree_variant!(safe_variant_t3, safe, 3);
#[cfg(test)]
test_b_tree_variant!(raw_variant_t2, raw, 2);
#[cfg(test)]
test_b_tree_variant!(raw_variant_t3, raw, 3);
#[cfg(test)]
test_b_tree_variant!(arena_variant_t2, arena, 2);
#[cfg(test)]
test_b_tree_variant!(arena_variant_t3, arena, 3);
