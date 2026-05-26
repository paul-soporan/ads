pub mod arena;
pub mod raw;
pub mod safe;

use crate::traits::{core::OrderedMap, diagnostics::TreeDiagnostics};

pub trait AvlTreeVariant<K, V>: OrderedMap<K, V> + TreeDiagnostics {
    type NodeView<'a>: Clone
    where
        Self: 'a;

    fn root_view<'a>(&'a self) -> Option<Self::NodeView<'a>>;
    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a>;
}

impl<K: Ord, V> AvlTreeVariant<K, V> for safe::AvlTree<K, V> {
    type NodeView<'a>
        = safe::AvlNodeView<K, V>
    where
        Self: 'a;

    fn root_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.root_view()
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a> {
        cursor.node_view()
    }
}

impl<K: Ord, V> AvlTreeVariant<K, V> for raw::AvlTree<K, V> {
    type NodeView<'a>
        = raw::AvlNodeView<K, V>
    where
        Self: 'a;

    fn root_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.root_view()
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a> {
        cursor.node_view()
    }
}

impl<K: Ord, V> AvlTreeVariant<K, V> for arena::AvlTree<K, V> {
    type NodeView<'a>
        = arena::AvlNodeView<'a, K, V>
    where
        Self: 'a;

    fn root_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.root_view()
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a> {
        cursor.node_view()
    }
}

#[cfg(test)]
trait AvlNodeViewLike<K>: Clone {
    fn key_cloned(&self) -> K
    where
        K: Clone;

    fn left_view(&self) -> Option<Self>;
    fn right_view(&self) -> Option<Self>;
    fn parent_view(&self) -> Option<Self>;
}

#[cfg(test)]
impl<K: Clone, V> AvlNodeViewLike<K> for safe::AvlNodeView<K, V> {
    fn key_cloned(&self) -> K
    where
        K: Clone,
    {
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
impl<K: Clone, V> AvlNodeViewLike<K> for raw::AvlNodeView<K, V> {
    fn key_cloned(&self) -> K
    where
        K: Clone,
    {
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
impl<'a, K: Clone, V> AvlNodeViewLike<K> for arena::AvlNodeView<'a, K, V> {
    fn key_cloned(&self) -> K
    where
        K: Clone,
    {
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
fn assert_avl_properties<K, N>(root: Option<N>)
where
    K: Ord + Clone + std::fmt::Debug,
    N: AvlNodeViewLike<K>,
{
    if let Some(root) = root {
        assert!(root.parent_view().is_none(), "root must not have a parent");
        let _ = check_node_recursive::<K, N>(root);
    }
}

#[cfg(test)]
fn check_node_recursive<K, N>(node: N) -> usize
where
    K: Ord + Clone + std::fmt::Debug,
    N: AvlNodeViewLike<K>,
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

    let balance = left_height as isize - right_height as isize;
    assert!(balance.abs() <= 1, "AVL balance factor must be in range [-1, 1], found {}", balance);

    1 + usize::max(left_height, right_height)
}

#[cfg(test)]
macro_rules! test_avl_tree_variant {
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

            type Tree = $tree_ty::AvlTree<i32, i32>;
            type ComplexityTree = $tree_ty::AvlTree<CountedKey, i32>;

            #[test]
            fn empty_tree() {
                let tree = Tree::new();
                assert!(tree.min_cursor().is_none());
                assert!(tree.max_cursor().is_none());
                assert!(!tree.contains_key(&5));
                assert_eq!(tree.height(), 0);
                assert_eq!(tree.node_count(), 0);
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
                assert_avl_properties(tree.root_view());
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
                assert_avl_properties(tree.root_view());
            }

            #[test]
            fn sorted_insertions_remain_balanced() {
                let mut tree = Tree::new();
                for value in 1..=100 {
                    tree.insert(value, value);
                    assert_avl_properties(tree.root_view());
                }

                assert!(tree.height() <= 8, "AVL height should stay logarithmic (log2(100) ~ 6.6, max height ~ 1.44*log2(100) ~ 9.5)");
            }

            #[test]
            fn deletion_keeps_balance() {
                let mut tree = Tree::new();
                for value in [10, 5, 15, 2, 7, 12, 20, 6, 8, 11, 13] {
                    tree.insert(value, value);
                }

                assert!(tree.remove(&2).is_some());
                assert_avl_properties(tree.root_view());
                assert!(tree.remove(&12).is_some());
                assert_avl_properties(tree.root_view());
                assert!(tree.remove(&10).is_some());
                assert_avl_properties(tree.root_view());
                assert!(tree.remove(&999).is_none());

                let items: Vec<_> = (&tree).into_iter().map(|(k, _)| k).collect();
                assert_eq!(items, vec![5, 6, 7, 8, 11, 13, 15, 20]);
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
                        assert_avl_properties(tree.root_view());
                    }
                }

                let actual: Vec<_> = (&tree).into_iter().collect();
                let expected: Vec<_> = model.into_iter().collect();
                assert_eq!(actual, expected);
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

                // For 8x increase in size, height should increase by ~3 (as it's log2).
                // Max height of AVL tree is ~1.44 * log2(n).
                // 1.44 * log2(128) = 1.44 * 7 ~ 10.08
                // 1.44 * log2(1024) = 1.44 * 10 ~ 14.4
                assert!(small <= 12, "height at size 128 is too large: {}", small);
                assert!(large <= 16, "height at size 1024 is too large: {}", large);
                assert!(large <= small + 6, "height grew too fast ({} -> {})", small, large);
            }

            #[test]
            fn search_comparison_count_scales_logarithmically() {
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

                // Average comparisons in search should be ~ log2(N).
                // log2(128) = 7, log2(1024) = 10.
                assert!(large <= small + 5, "average search cost grew too quickly ({} -> {})", small, large);
            }
        }
    };
}

#[cfg(test)]
test_avl_tree_variant!(safe_variant, safe);
#[cfg(test)]
test_avl_tree_variant!(raw_variant, raw);
#[cfg(test)]
test_avl_tree_variant!(arena_variant, arena);
