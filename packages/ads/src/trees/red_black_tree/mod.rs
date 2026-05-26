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
trait RbNodeViewLike: Clone {
    fn is_black(&self) -> bool;
    fn is_red(&self) -> bool;
    fn left_view(&self) -> Option<Self>;
    fn right_view(&self) -> Option<Self>;
}

#[cfg(test)]
impl<K, V> RbNodeViewLike for safe::RbNodeView<K, V> {
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
}

#[cfg(test)]
impl<K, V> RbNodeViewLike for raw::RbNodeView<K, V> {
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
}

#[cfg(test)]
impl<'a, K, V> RbNodeViewLike for arena::RbNodeView<'a, K, V> {
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
}

#[cfg(test)]
fn assert_rb_properties<N>(root: Option<N>)
where
    N: RbNodeViewLike,
{
    if let Some(root) = root {
        assert!(root.is_black(), "Root must be black");
        let _ = check_node::<N>(root);
    }
}

#[cfg(test)]
fn check_node<N>(node: N) -> usize
where
    N: RbNodeViewLike,
{
    if node.is_red() {
        if let Some(left) = node.left_view() {
            assert!(left.is_black(), "Red node left child must be black");
        }
        if let Some(right) = node.right_view() {
            assert!(right.is_black(), "Red node right child must be black");
        }
    }

    let left_bh = node.left_view().map_or(1, |left| check_node::<N>(left));
    let right_bh = node.right_view().map_or(1, |right| check_node::<N>(right));
    assert_eq!(left_bh, right_bh, "Black heights must match on both sides");

    left_bh + usize::from(node.is_black())
}

#[cfg(test)]
macro_rules! test_red_black_tree_variant {
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
                assert_rb_properties(tree.root_view());
            }

            #[test]
            fn insert_and_contains() {
                let mut tree = Tree::new();
                for value in [5, 3, 7, 2, 4, 6, 8] {
                    tree.insert(value, ());
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
                    tree.insert(value, ());
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
                    tree.insert(value, ());
                }
                assert_eq!(*tree.root_view().expect("root").key(), 20);
                assert_rb_properties(tree.root_view());

                let mut tree = Tree::new();
                for value in [30, 20, 10] {
                    tree.insert(value, ());
                }
                assert_eq!(*tree.root_view().expect("root").key(), 20);
                assert_rb_properties(tree.root_view());

                let mut tree = Tree::new();
                for value in [30, 10, 20] {
                    tree.insert(value, ());
                }
                assert_eq!(*tree.root_view().expect("root").key(), 20);
                assert_rb_properties(tree.root_view());
            }

            #[test]
            fn deletion_and_rank_queries() {
                let mut tree = Tree::new();
                for value in [10, 5, 15, 2, 7, 12, 20, 6, 8] {
                    tree.insert(value, ());
                }

                assert!(tree.remove(&5).is_some());
                assert!(tree.remove(&10).is_some());
                assert!(tree.remove(&99).is_none());
                assert_rb_properties(tree.root_view());

                let sorted: Vec<_> = (&tree).into_iter().map(|(k, _)| k).collect();
                for (rank, value) in sorted.iter().enumerate() {
                    let selected = tree.select(rank).expect("selected");
                    assert_eq!(*selected.key(), *value);
                }
            }

            #[test]
            fn mixed_operations_match_btreemap_model() {
                let mut tree = Tree::new();
                let mut model = BTreeSet::new();

                for key in [41, 38, 31, 12, 19, 8] {
                    assert_eq!(tree.insert(key, ()).is_some(), !model.insert(key));
                    assert_rb_properties(tree.root_view());
                }

                for key in [19, 25, 32] {
                    assert_eq!(tree.insert(key, ()).is_some(), !model.insert(key));
                    assert_rb_properties(tree.root_view());
                }

                for key in [31, 8, 99] {
                    assert_eq!(tree.remove(&key).is_some(), model.remove(&key));
                    assert_rb_properties(tree.root_view());
                }

                let tree_items: Vec<_> = (&tree).into_iter().map(|(k, _)| k).collect();
                let model_items: Vec<_> = model.into_iter().collect();
                assert_eq!(tree_items, model_items);
            }
        }
    };
}

#[cfg(test)]
test_red_black_tree_variant!(safe_variant, safe::RedBlackTree<i32, ()>);
#[cfg(test)]
test_red_black_tree_variant!(raw_variant, raw::RedBlackTree<i32, ()>);
#[cfg(test)]
test_red_black_tree_variant!(arena_variant, arena::RedBlackTree<i32, ()>);
