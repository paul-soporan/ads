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
}

#[cfg(test)]
fn assert_avl_properties<K, N>(root: Option<N>)
where
    K: Ord + Clone + std::fmt::Debug,
    N: AvlNodeViewLike<K>,
{
    if let Some(root) = root {
        let _ = check_node::<K, N>(root);
    }
}

#[cfg(test)]
fn check_node<K, N>(node: N) -> usize
where
    K: Ord + Clone + std::fmt::Debug,
    N: AvlNodeViewLike<K>,
{
    let key = node.key_cloned();

    let left_height = if let Some(left) = node.left_view() {
        assert!(left.key_cloned() < key, "left child key must be smaller");
        check_node::<K, N>(left)
    } else {
        0
    };

    let right_height = if let Some(right) = node.right_view() {
        assert!(key < right.key_cloned(), "right child key must be larger");
        check_node::<K, N>(right)
    } else {
        0
    };

    let balance = left_height as isize - right_height as isize;
    assert!(balance.abs() <= 1, "AVL balance invariant violated");

    1 + usize::max(left_height, right_height)
}

#[cfg(test)]
macro_rules! test_avl_tree_variant {
    ($module:ident, $tree_ty:ty) => {
        mod $module {
            use super::*;
            use std::collections::BTreeSet;

            use crate::traits::core::Map;
            use crate::traits::diagnostics::TreeDiagnostics;

            type Tree = $tree_ty;

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
                tree.insert(5, ());
                tree.insert(3, ());
                tree.insert(7, ());

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
                    tree.insert(value, ());
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
                    tree.insert(value, ());
                }

                assert_avl_properties(tree.root_view());
                assert!(tree.height() <= 8, "AVL height should stay logarithmic");
            }

            #[test]
            fn deletion_keeps_balance() {
                let mut tree = Tree::new();
                for value in [10, 5, 15, 2, 7, 12, 20, 6, 8, 11, 13] {
                    tree.insert(value, ());
                }

                assert!(tree.remove(&2).is_some());
                assert!(tree.remove(&12).is_some());
                assert!(tree.remove(&10).is_some());
                assert!(tree.remove(&999).is_none());

                assert_avl_properties(tree.root_view());
                let items: Vec<_> = (&tree).into_iter().map(|(k, _)| k).collect();
                assert_eq!(items, vec![5, 6, 7, 8, 11, 13, 15, 20]);
            }

            #[test]
            fn mixed_operations_match_btreemap_model() {
                let mut tree = Tree::new();
                let mut model = BTreeSet::new();

                for key in [20, 4, 26, 3, 9, 15, 30] {
                    assert_eq!(tree.insert(key, ()).is_some(), !model.insert(key));
                    assert_avl_properties(tree.root_view());
                }

                for key in [9, 15, 2] {
                    assert_eq!(tree.insert(key, ()).is_some(), !model.insert(key));
                    assert_avl_properties(tree.root_view());
                }

                for key in [26, 4, 999] {
                    assert_eq!(tree.remove(&key).is_some(), model.remove(&key));
                    assert_avl_properties(tree.root_view());
                }

                let tree_items: Vec<_> = (&tree).into_iter().map(|(k, _)| k).collect();
                let model_items: Vec<_> = model.into_iter().collect();
                assert_eq!(tree_items, model_items);
            }
        }
    };
}

#[cfg(test)]
test_avl_tree_variant!(safe_variant, safe::AvlTree<i32, ()>);
#[cfg(test)]
test_avl_tree_variant!(raw_variant, raw::AvlTree<i32, ()>);
#[cfg(test)]
test_avl_tree_variant!(arena_variant, arena::AvlTree<i32, ()>);
