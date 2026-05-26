pub mod arena;
pub mod raw;
pub mod safe;

use crate::traits::{core::OrderedMap, diagnostics::TreeDiagnostics};

pub trait RedBlackTreeVariant<K, V>: OrderedMap<K, V> + TreeDiagnostics {
    type NodeView<'a>: Clone
    where
        Self: 'a;

    fn root_view<'a>(&'a self) -> Option<Self::NodeView<'a>>;
    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a>;
}

impl<K: Ord, V> RedBlackTreeVariant<K, V> for safe::RedBlackTree<K, V> {
    type NodeView<'a>
        = safe::RbNodeView<K, V>
    where
        Self: 'a;

    fn root_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.root_view()
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a> {
        cursor.node_view()
    }
}

impl<K: Ord, V> RedBlackTreeVariant<K, V> for raw::RedBlackTree<K, V> {
    type NodeView<'a>
        = raw::RbNodeView<K, V>
    where
        Self: 'a;

    fn root_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.root_view()
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a> {
        cursor.node_view()
    }
}

impl<K: Ord, V> RedBlackTreeVariant<K, V> for arena::RedBlackTree<K, V> {
    type NodeView<'a>
        = arena::RbNodeView<'a, K, V>
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
trait RbNodeViewLike<K>: Clone {
    fn key_cloned(&self) -> K
    where
        K: Clone;
    fn is_black(&self) -> bool;
    fn is_red(&self) -> bool;
    fn left_view(&self) -> Option<Self>;
    fn right_view(&self) -> Option<Self>;
    fn parent_view(&self) -> Option<Self>;
}

#[cfg(test)]
impl<K: Clone, V> RbNodeViewLike<K> for safe::RbNodeView<K, V> {
    fn key_cloned(&self) -> K {
        self.key().clone()
    }

    fn is_black(&self) -> bool {
        self.color() == safe::NodeColor::Black
    }

    fn is_red(&self) -> bool {
        self.color() == safe::NodeColor::Red
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
impl<K: Clone, V> RbNodeViewLike<K> for raw::RbNodeView<K, V> {
    fn key_cloned(&self) -> K {
        self.key().clone()
    }

    fn is_black(&self) -> bool {
        self.color() == raw::NodeColor::Black
    }

    fn is_red(&self) -> bool {
        self.color() == raw::NodeColor::Red
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
impl<'a, K: Clone, V> RbNodeViewLike<K> for arena::RbNodeView<'a, K, V> {
    fn key_cloned(&self) -> K {
        self.key().clone()
    }

    fn is_black(&self) -> bool {
        self.color() == arena::NodeColor::Black
    }

    fn is_red(&self) -> bool {
        self.color() == arena::NodeColor::Red
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
fn assert_rb_properties<K, N>(root: Option<N>)
where
    K: Ord + Clone + std::fmt::Debug,
    N: RbNodeViewLike<K>,
{
    if let Some(root) = root {
        assert!(root.is_black(), "Root must be black");
        assert!(root.parent_view().is_none(), "Root must not have a parent");
        let _ = check_node_recursive::<K, N>(root);
    }
}

#[cfg(test)]
fn check_node_recursive<K, N>(node: N) -> usize
where
    K: Ord + Clone + std::fmt::Debug,
    N: RbNodeViewLike<K>,
{
    let key = node.key_cloned();

    if node.is_red() {
        if let Some(left) = node.left_view() {
            assert!(left.is_black(), "Red node left child must be black (no red-red violation)");
        }
        if let Some(right) = node.right_view() {
            assert!(right.is_black(), "Red node right child must be black (no red-red violation)");
        }
    }

    let left_bh = if let Some(left) = node.left_view() {
        assert!(left.key_cloned() < key, "BST property violation: left child >= parent");
        assert!(left.parent_view().is_some(), "child node must expose parent");
        check_node_recursive::<K, N>(left)
    } else {
        1 // leaf sentinel is black
    };

    let right_bh = if let Some(right) = node.right_view() {
        assert!(key < right.key_cloned(), "BST property violation: right child <= parent");
        assert!(right.parent_view().is_some(), "child node must expose parent");
        check_node_recursive::<K, N>(right)
    } else {
        1 // leaf sentinel is black
    };

    assert_eq!(left_bh, right_bh, "Black heights must match on both sides at key {:?}", key);

    left_bh + usize::from(node.is_black())
}

#[cfg(test)]
macro_rules! test_red_black_tree_variant {
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

            type Tree = $tree_ty::RedBlackTree<i32, i32>;
            type ComplexityTree = $tree_ty::RedBlackTree<CountedKey, i32>;

            #[test]
            fn empty_tree() {
                let tree = Tree::new();
                assert!(tree.min_cursor().is_none());
                assert!(tree.max_cursor().is_none());
                assert!(!tree.contains_key(&5));
                assert_eq!(tree.height(), 0);
                assert_eq!(tree.node_count(), 0);
                assert_rb_properties(tree.root_view());
            }

            #[test]
            fn insert_and_contains() {
                let mut tree = Tree::new();
                for value in [5, 3, 7, 2, 4, 6, 8] {
                    tree.insert(value, value * 10);
                }

                for value in [5, 3, 7, 2, 4, 6, 8] {
                    assert!(tree.contains_key(&value));
                }
                assert!(!tree.contains_key(&9));
                assert_rb_properties(tree.root_view());
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
                let c4 = tree.cursor(&4).expect("c4");
                let c8 = tree.cursor(&8).expect("c8");
                let c5 = tree.cursor(&5).expect("c5");

                assert_eq!(*c2.successor().expect("succ").key(), 3);
                assert_eq!(*c4.successor().expect("succ").key(), 5);
                assert!(c8.successor().is_none());

                assert_eq!(*c8.predecessor().expect("pred").key(), 7);
                assert_eq!(*c5.predecessor().expect("pred").key(), 4);
                assert!(c2.predecessor().is_none());
                assert_rb_properties(tree.root_view());
            }

            #[test]
            fn rotations_and_balancing() {
                let mut tree = Tree::new();
                for value in [10, 20, 30] {
                    tree.insert(value, value);
                }
                // In RB tree, inserting 10, 20, 30 usually results in 20 being the black root.
                assert_eq!(*tree.root_view().expect("root").key(), 20);
                assert_rb_properties(tree.root_view());

                let mut tree = Tree::new();
                for value in [30, 20, 10] {
                    tree.insert(value, value);
                }
                assert_eq!(*tree.root_view().expect("root").key(), 20);
                assert_rb_properties(tree.root_view());
            }

            #[test]
            fn deletion_and_rank_queries() {
                let mut tree = Tree::new();
                for value in [10, 5, 15, 2, 7, 12, 20, 6, 8] {
                    tree.insert(value, value);
                }

                assert!(tree.remove(&5).is_some());
                assert_rb_properties(tree.root_view());
                assert!(tree.remove(&10).is_some());
                assert_rb_properties(tree.root_view());
                assert!(tree.remove(&15).is_some());
                assert_rb_properties(tree.root_view());

                let sorted: Vec<_> = (&tree).into_iter().map(|(k, _)| k).collect();
                for (rank, value) in sorted.iter().enumerate() {
                    let selected = tree.select(rank).expect("selected");
                    assert_eq!(*selected.key(), *value);
                }
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
                        assert_rb_properties(tree.root_view());
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

                // Max height of RB tree is 2 * log2(n+1).
                // 2 * log2(129) ~ 2 * 7.01 ~ 14.02
                // 2 * log2(1025) ~ 2 * 10.00 ~ 20.
                assert!(small <= 15, "height at size 128 is too large: {}", small);
                assert!(large <= 22, "height at size 1024 is too large: {}", large);
                assert!(large <= small + 8, "height grew too fast ({} -> {})", small, large);
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

                // Average search cost in RB tree is ~ log2(N).
                assert!(large <= small + 6, "average search cost grew too quickly ({} -> {})", small, large);
            }
        }
    };
}

#[cfg(test)]
test_red_black_tree_variant!(safe_variant, safe);
#[cfg(test)]
test_red_black_tree_variant!(raw_variant, raw);
#[cfg(test)]
test_red_black_tree_variant!(arena_variant, arena);
