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
        check_node::<K, N>(root, degree, true);
    }
}

#[cfg(test)]
fn check_node<K, N>(node: N, t: usize, is_root: bool)
where
    K: Ord + Clone + std::fmt::Debug,
    N: BTreeNodeViewLike<K>,
{
    let keys = node.keys_vec();
    let key_count = keys.len();

    if !is_root {
        assert!(key_count >= t - 1, "node underflow");
    }
    assert!(key_count < 2 * t, "node overflow");

    for i in 0..key_count.saturating_sub(1) {
        assert!(keys[i] <= keys[i + 1], "keys must be sorted");
    }

    let children = node.children_vec();
    if node.is_leaf_node() {
        assert!(children.is_empty(), "leaf nodes must not have children");
    } else {
        assert_eq!(children.len(), key_count + 1, "invalid child count");
        for child in children {
            assert!(child.has_parent(), "child must have a parent");
            check_node::<K, N>(child, t, false);
        }
    }
}

#[cfg(test)]
macro_rules! test_b_tree_variant {
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
                assert_btree_properties(tree.root_view(), tree.degree());
            }

            #[test]
            fn insert_contains_and_boundaries() {
                let mut tree = Tree::new();
                for value in [10, 20, 5, 6, 12] {
                    tree.insert(value, ());
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
                    tree.insert(value, ());
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
                    tree.insert(value, ());
                }

                assert!(tree.remove(&40).is_some());
                assert!(tree.remove(&20).is_some());
                assert!(tree.remove(&200).is_none());
                assert_btree_properties(tree.root_view(), tree.degree());
            }

            #[test]
            fn larger_delete_sequence() {
                let mut tree = Tree::new();
                for value in 1..=50 {
                    tree.insert(value, ());
                }
                assert_btree_properties(tree.root_view(), tree.degree());

                for value in (1..=50).step_by(3) {
                    assert!(tree.remove(&value).is_some());
                    assert_btree_properties(tree.root_view(), tree.degree());
                }

                for value in 1..=50 {
                    if value % 3 == 1 {
                        assert!(!tree.contains_key(&value));
                    } else {
                        assert!(tree.contains_key(&value));
                    }
                }
            }

            #[test]
            fn mixed_operations_match_btreemap_model() {
                let mut tree = Tree::new();
                let mut model = BTreeSet::new();

                for key in [18, 7, 25, 3, 11, 20, 30] {
                    assert_eq!(tree.insert(key, ()).is_some(), !model.insert(key));
                    assert_btree_properties(tree.root_view(), tree.degree());
                }

                for key in [11, 28, 1] {
                    assert_eq!(tree.insert(key, ()).is_some(), !model.insert(key));
                    assert_btree_properties(tree.root_view(), tree.degree());
                }

                for key in [7, 20, 999] {
                    assert_eq!(tree.remove(&key).is_some(), model.remove(&key));
                    assert_btree_properties(tree.root_view(), tree.degree());
                }

                let tree_items: Vec<_> = (&tree).into_iter().map(|(k, _)| k).collect();
                let model_items: Vec<_> = model.into_iter().collect();
                assert_eq!(tree_items, model_items);
            }
        }
    };
}

#[cfg(test)]
test_b_tree_variant!(safe_variant_t2, safe::BTree<i32, (), 2>);
#[cfg(test)]
test_b_tree_variant!(safe_variant_t3, safe::BTree<i32, (), 3>);
#[cfg(test)]
test_b_tree_variant!(raw_variant_t2, raw::BTree<i32, (), 2>);
#[cfg(test)]
test_b_tree_variant!(raw_variant_t3, raw::BTree<i32, (), 3>);
#[cfg(test)]
test_b_tree_variant!(arena_variant_t2, arena::BTree<i32, (), 2>);
#[cfg(test)]
test_b_tree_variant!(arena_variant_t3, arena::BTree<i32, (), 3>);
