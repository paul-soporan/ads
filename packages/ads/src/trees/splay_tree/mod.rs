pub mod arena;
pub mod raw;
pub mod safe;

use crate::traits::{core::OrderedMap, diagnostics::TreeDiagnostics};

pub trait SplayTreeVariant<K, V>: OrderedMap<K, V> + TreeDiagnostics {
    type NodeView<'a>: Clone
    where
        Self: 'a;

    fn root_view<'a>(&'a self) -> Option<Self::NodeView<'a>>;
    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a>;
    fn contains_adaptive(&mut self, key: &K) -> bool;
}

impl<K: Ord, V> SplayTreeVariant<K, V> for safe::SplayTree<K, V> {
    type NodeView<'a>
        = safe::SplayNodeView<K, V>
    where
        Self: 'a;

    fn root_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.root_view()
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a> {
        cursor.node_view()
    }

    fn contains_adaptive(&mut self, key: &K) -> bool {
        safe::SplayTree::contains_adaptive(self, key)
    }
}

impl<K: Ord, V> SplayTreeVariant<K, V> for raw::SplayTree<K, V> {
    type NodeView<'a>
        = raw::SplayNodeView<K, V>
    where
        Self: 'a;

    fn root_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.root_view()
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a> {
        cursor.node_view()
    }

    fn contains_adaptive(&mut self, key: &K) -> bool {
        raw::SplayTree::contains_adaptive(self, key)
    }
}

impl<K: Ord, V> SplayTreeVariant<K, V> for arena::SplayTree<K, V> {
    type NodeView<'a>
        = arena::SplayNodeView<'a, K, V>
    where
        Self: 'a;

    fn root_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.root_view()
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a> {
        cursor.node_view()
    }

    fn contains_adaptive(&mut self, key: &K) -> bool {
        arena::SplayTree::contains_adaptive(self, key)
    }
}

#[cfg(test)]
trait SplayNodeViewLike<K>: Clone {
    fn key_cloned(&self) -> K
    where
        K: Clone;
    fn left_view(&self) -> Option<Self>;
    fn right_view(&self) -> Option<Self>;
    fn parent_view(&self) -> Option<Self>;
}

#[cfg(test)]
impl<K: Clone, V> SplayNodeViewLike<K> for safe::SplayNodeView<K, V> {
    fn key_cloned(&self) -> K {
        self.key().clone()
    }

    fn left_view(&self) -> Option<Self> {
        self.left()
    }

    fn right_view(&self) -> Option<Self> {
        self.right()
    }

    fn parent_view(&self) -> Option<Self> {
        self.parent()
    }
}

#[cfg(test)]
impl<K: Clone, V> SplayNodeViewLike<K> for raw::SplayNodeView<K, V> {
    fn key_cloned(&self) -> K {
        self.key().clone()
    }

    fn left_view(&self) -> Option<Self> {
        self.left()
    }

    fn right_view(&self) -> Option<Self> {
        self.right()
    }

    fn parent_view(&self) -> Option<Self> {
        self.parent()
    }
}

#[cfg(test)]
impl<'a, K: Clone, V> SplayNodeViewLike<K> for arena::SplayNodeView<'a, K, V> {
    fn key_cloned(&self) -> K {
        self.key().clone()
    }

    fn left_view(&self) -> Option<Self> {
        self.left()
    }

    fn right_view(&self) -> Option<Self> {
        self.right()
    }

    fn parent_view(&self) -> Option<Self> {
        self.parent()
    }
}

#[cfg(test)]
fn assert_splay_invariants<K, N>(root: Option<N>)
where
    K: Ord + Clone + std::fmt::Debug,
    N: SplayNodeViewLike<K>,
{
    if let Some(root) = root {
        assert!(root.parent_view().is_none(), "Root must not have a parent");
        let _ = check_node_recursive::<K, N>(root);
    }
}

#[cfg(test)]
fn check_node_recursive<K, N>(node: N) -> usize
where
    K: Ord + Clone + std::fmt::Debug,
    N: SplayNodeViewLike<K>,
{
    let key = node.key_cloned();

    let left_height = if let Some(left) = node.left_view() {
        assert!(left.key_cloned() < key, "BST property violation: left child >= parent");
        assert!(left.parent_view().is_some(), "child node must expose parent");
        check_node_recursive::<K, N>(left)
    } else {
        0
    };

    let right_height = if let Some(right) = node.right_view() {
        assert!(key < right.key_cloned(), "BST property violation: right child <= parent");
        assert!(right.parent_view().is_some(), "child node must expose parent");
        check_node_recursive::<K, N>(right)
    } else {
        0
    };

    1 + usize::max(left_height, right_height)
}

#[cfg(test)]
macro_rules! test_splay_tree_variant {
    ($module:ident, $tree_ty:ident) => {
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

            type Tree = $tree_ty::SplayTree<i32, i32>;
            type ComplexityTree = $tree_ty::SplayTree<CountedKey, i32>;

            #[test]
            fn empty_tree() {
                let tree = Tree::new();
                assert!(tree.min_cursor().is_none());
                assert!(tree.max_cursor().is_none());
                assert!(!tree.contains_key(&5));
                assert_eq!(tree.height(), 0);
                assert_eq!(tree.node_count(), 0);
                assert_splay_invariants(tree.root_view());
            }

            #[test]
            fn insert_and_contains() {
                let mut tree = Tree::new();
                tree.insert(5, 50);
                tree.insert(3, 30);
                tree.insert(7, 70);

                assert!(tree.contains_key(&5));
                assert!(tree.contains_key(&3));
                assert!(tree.contains_key(&7));
                assert!(!tree.contains_key(&4));
                assert_splay_invariants(tree.root_view());
            }

            #[test]
            fn min_max_and_neighbors() {
                let mut tree = Tree::new();
                for value in [5, 3, 7, 2, 4, 6, 8] {
                    tree.insert(value, value * 10);
                }

                assert_eq!(*tree.min_cursor().expect("min").key(), 2);
                assert_eq!(*tree.max_cursor().expect("max").key(), 8);

                let c2 = tree.cursor(&2).expect("c2");
                let c5 = tree.cursor(&5).expect("c5");
                let c8 = tree.cursor(&8).expect("c8");

                assert_eq!(*c2.successor().expect("succ").key(), 3);
                assert_eq!(*c5.successor().expect("succ").key(), 6);
                assert!(c8.successor().is_none());

                assert_eq!(*c8.predecessor().expect("pred").key(), 7);
                assert_eq!(*c5.predecessor().expect("pred").key(), 4);
                assert!(c2.predecessor().is_none());
                assert_splay_invariants(tree.root_view());
            }

            #[test]
            fn delete_logic() {
                let mut tree = Tree::new();
                for value in [5, 3, 7, 2, 4, 6, 8] {
                    tree.insert(value, value);
                }

                assert!(tree.remove(&2).is_some());
                assert_splay_invariants(tree.root_view());
                assert!(tree.remove(&3).is_some());
                assert_splay_invariants(tree.root_view());
                assert!(tree.remove(&7).is_some());
                assert_splay_invariants(tree.root_view());
                assert!(tree.remove(&5).is_some());
                assert_splay_invariants(tree.root_view());
                assert!(tree.remove(&42).is_none());

                let sorted: Vec<_> = (&tree).into_iter().map(|(k, _)| k).collect();
                assert_eq!(sorted, vec![4, 6, 8]);
                assert_eq!(tree.node_count(), sorted.len());
            }

            #[test]
            fn adaptive_contains_splays_on_hit() {
                let mut tree = Tree::new();
                for value in [5, 3, 7, 2, 4, 6, 8] {
                    tree.insert(value, value);
                }

                assert!(tree.contains_adaptive(&4));
                assert_eq!(tree.root_view().map(|node| *node.key()), Some(4));
                assert_splay_invariants(tree.root_view());
            }

            #[test]
            fn adaptive_contains_splays_last_visited_on_miss() {
                let mut tree = Tree::new();
                for value in [5, 3, 7, 2, 4, 6, 8] {
                    tree.insert(value, value);
                }

                assert!(!tree.contains_adaptive(&9));
                assert_eq!(tree.root_view().map(|node| *node.key()), Some(8));
                assert_splay_invariants(tree.root_view());

                assert!(!tree.contains_adaptive(&1));
                assert_eq!(tree.root_view().map(|node| *node.key()), Some(2));
                assert_splay_invariants(tree.root_view());
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
                        2 if !model.is_empty() => { // Search (Adaptive)
                            let keys: Vec<_> = model.keys().cloned().collect();
                            let k = keys[rng.gen_range(0..keys.len())];
                            assert!(tree.contains_adaptive(&k));
                        }
                        3 => { // Search non-existent (Adaptive)
                            let k = rng.gen_range(500..1000);
                            assert!(!tree.contains_adaptive(&k));
                        }
                        _ => {}
                    }
                    if rng.gen_bool(0.05) {
                        assert_splay_invariants(tree.root_view());
                    }
                }

                let actual: Vec<_> = (&tree).into_iter().collect();
                let expected: Vec<_> = model.into_iter().collect();
                assert_eq!(actual, expected);
            }

            #[test]
            fn amortized_search_cost_is_logarithmic() {
                fn run_case(size: i32) -> usize {
                    let comparisons = Arc::new(AtomicUsize::new(0));
                    let mut tree = ComplexityTree::new();
                    for value in 0..size {
                        tree.insert(CountedKey::new(value, &comparisons), value);
                    }

                    comparisons.store(0, AtomicOrdering::Relaxed);
                    let trials = 200;
                    let mut rng = StdRng::seed_from_u64(42);
                    for _ in 0..trials {
                        let value = rng.gen_range(0..size);
                        let key = CountedKey::new(value, &comparisons);
                        assert!(tree.contains_adaptive(&key));
                    }
                    comparisons.load(AtomicOrdering::Relaxed) / trials
                }

                let small = run_case(128);
                let large = run_case(1024);

                assert!(large <= small + 15, "average search cost grew too quickly ({} -> {})", small, large);
            }
            
            #[test]
            fn repetitive_access_is_very_cheap() {
                let comparisons = Arc::new(AtomicUsize::new(0));
                let mut tree = ComplexityTree::new();
                for value in 0..100 {
                    tree.insert(CountedKey::new(value, &comparisons), value);
                }

                let key = CountedKey::new(50, &comparisons);
                tree.contains_adaptive(&key);
                
                comparisons.store(0, AtomicOrdering::Relaxed);
                for _ in 0..100 {
                    assert!(tree.contains_adaptive(&key));
                }
                
                let avg = comparisons.load(AtomicOrdering::Relaxed) / 100;
                assert!(avg <= 2, "repetitive access should be O(1) - avg comparisons: {}", avg);
            }
        }
    };
}

#[cfg(test)]
test_splay_tree_variant!(safe_variant, safe);
#[cfg(test)]
test_splay_tree_variant!(raw_variant, raw);
#[cfg(test)]
test_splay_tree_variant!(arena_variant, arena);
