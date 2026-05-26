pub mod arena;
pub mod raw;
pub mod safe;

use crate::traits::{core::OrderedMap, diagnostics::TreeDiagnostics};

pub trait BinarySearchTreeVariant<K, V>: OrderedMap<K, V> + TreeDiagnostics {
    type NodeView<'a>: Clone
    where
        Self: 'a;

    fn root_view<'a>(&'a self) -> Option<Self::NodeView<'a>>;
    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a>;
}

impl<K: Ord, V> BinarySearchTreeVariant<K, V> for safe::BinarySearchTree<K, V> {
    type NodeView<'a>
        = safe::BstNodeView<K, V>
    where
        Self: 'a;

    fn root_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.root_view()
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a> {
        cursor.node_view()
    }
}

impl<K: Ord, V> BinarySearchTreeVariant<K, V> for raw::BinarySearchTree<K, V> {
    type NodeView<'a>
        = raw::BstNodeView<K, V>
    where
        Self: 'a;

    fn root_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.root_view()
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a> {
        cursor.node_view()
    }
}

impl<K: Ord, V> BinarySearchTreeVariant<K, V> for arena::BinarySearchTree<K, V> {
    type NodeView<'a>
        = arena::BstNodeView<'a, K, V>
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
macro_rules! test_binary_search_tree_variant {
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
            }

            #[test]
            fn delete_leaf_one_child_two_children_and_root() {
                let mut tree = Tree::new();
                for value in [5, 3, 7, 2, 4, 6, 8] {
                    tree.insert(value, ());
                }

                assert!(tree.remove(&2).is_some());
                assert!(tree.remove(&3).is_some());
                assert!(tree.remove(&7).is_some());
                assert!(tree.remove(&5).is_some());
                assert!(tree.remove(&42).is_none());

                let sorted: Vec<_> = (&tree).into_iter().map(|(k, _)| k).collect();
                assert_eq!(sorted, vec![4, 6, 8]);
                assert_eq!(tree.node_count(), sorted.len());
            }

            #[test]
            fn into_iter_is_sorted() {
                let mut tree = Tree::new();
                for value in [10, 2, 8, 1, 3] {
                    tree.insert(value, ());
                }

                let items: Vec<_> = (&tree).into_iter().map(|(k, _)| k).collect();
                assert_eq!(items, vec![1, 2, 3, 8, 10]);
            }

            #[test]
            fn mixed_operations_match_btreemap_model() {
                let mut tree = Tree::new();
                let mut model = BTreeSet::new();

                for key in [8, 3, 10, 1, 6, 14, 4] {
                    assert_eq!(tree.insert(key, ()).is_some(), !model.insert(key));
                }

                for key in [6, 10, 13] {
                    assert_eq!(tree.insert(key, ()).is_some(), !model.insert(key));
                }

                for key in [1, 4, 6, 13, 99] {
                    assert_eq!(tree.contains_key(&key), model.contains(&key));
                }

                for key in [3, 14, 42] {
                    assert_eq!(tree.remove(&key).is_some(), model.remove(&key));
                }

                let tree_items: Vec<_> = (&tree).into_iter().map(|(k, _)| k).collect();
                let model_items: Vec<_> = model.into_iter().collect();
                assert_eq!(tree_items, model_items);
                assert_eq!(tree.node_count(), tree_items.len());
            }
        }
    };
}

#[cfg(test)]
test_binary_search_tree_variant!(safe_variant, safe::BinarySearchTree<i32, ()>);
#[cfg(test)]
test_binary_search_tree_variant!(raw_variant, raw::BinarySearchTree<i32, ()>);
#[cfg(test)]
test_binary_search_tree_variant!(arena_variant, arena::BinarySearchTree<i32, ()>);
