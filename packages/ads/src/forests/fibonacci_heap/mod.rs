pub mod arena;
pub mod raw;
pub mod safe;

use crate::traits::{core::PriorityQueue, diagnostics::ForestDiagnostics};

pub trait FibonacciHeapVariant<T>: PriorityQueue<T> + ForestDiagnostics {
    type NodeView<'a>: Clone
    where
        Self: 'a;

    fn head_view<'a>(&'a self) -> Option<Self::NodeView<'a>>;
    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a>;
    fn merge_with(&mut self, other: &mut Self);
}

impl<T: Ord> FibonacciHeapVariant<T> for safe::FibonacciHeap<T> {
    type NodeView<'a>
        = safe::FibonacciNodeView<T>
    where
        Self: 'a;

    fn head_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.head_view()
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a> {
        cursor.node_view()
    }

    fn merge_with(&mut self, other: &mut Self) {
        self.merge(other)
    }
}

impl<T: Ord> FibonacciHeapVariant<T> for raw::FibonacciHeap<T> {
    type NodeView<'a>
        = raw::FibonacciNodeView<T>
    where
        Self: 'a;

    fn head_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.head_view()
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a> {
        cursor.node_view()
    }

    fn merge_with(&mut self, other: &mut Self) {
        self.merge(other)
    }
}

impl<T: Ord> FibonacciHeapVariant<T> for arena::FibonacciHeap<T> {
    type NodeView<'a>
        = arena::FibonacciNodeView<'a, T>
    where
        Self: 'a;

    fn head_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.head_view()
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::NodeView<'a> {
        cursor
            .node_view(self)
            .expect("cursor must reference a live node")
    }

    fn merge_with(&mut self, other: &mut Self) {
        self.merge(other)
    }
}

#[cfg(test)]
trait FibonacciNodeViewLike<T>: Clone {
    fn value_cloned(&self) -> T
    where
        T: Clone;
    fn degree_value(&self) -> usize;
    fn child_view(&self) -> Option<Self>;
    fn sibling_view(&self) -> Option<Self>;
    fn parent_view(&self) -> Option<Self>;
}

#[cfg(test)]
impl<T: Ord> FibonacciNodeViewLike<T> for safe::FibonacciNodeView<T> {
    fn value_cloned(&self) -> T
    where
        T: Clone,
    {
        self.value().clone()
    }

    fn degree_value(&self) -> usize {
        self.degree()
    }

    fn child_view(&self) -> Option<Self> {
        self.child()
    }

    fn sibling_view(&self) -> Option<Self> {
        self.sibling()
    }

    fn parent_view(&self) -> Option<Self> {
        self.parent()
    }
}

#[cfg(test)]
impl<T: Ord> FibonacciNodeViewLike<T> for raw::FibonacciNodeView<T> {
    fn value_cloned(&self) -> T
    where
        T: Clone,
    {
        self.value().clone()
    }

    fn degree_value(&self) -> usize {
        self.degree()
    }

    fn child_view(&self) -> Option<Self> {
        self.child()
    }

    fn sibling_view(&self) -> Option<Self> {
        self.sibling()
    }

    fn parent_view(&self) -> Option<Self> {
        self.parent()
    }
}

#[cfg(test)]
impl<'a, T: Ord> FibonacciNodeViewLike<T> for arena::FibonacciNodeView<'a, T> {
    fn value_cloned(&self) -> T
    where
        T: Clone,
    {
        self.value().clone()
    }

    fn degree_value(&self) -> usize {
        self.degree()
    }

    fn child_view(&self) -> Option<Self> {
        self.child()
    }

    fn sibling_view(&self) -> Option<Self> {
        self.sibling()
    }

    fn parent_view(&self) -> Option<Self> {
        self.parent()
    }
}

#[cfg(test)]
fn assert_heap_invariants<T, N>(head: Option<N>)
where
    T: Ord + Clone + std::fmt::Debug,
    N: FibonacciNodeViewLike<T>,
{
    fn walk<T, N>(node: N, parent: Option<T>) -> (usize, usize)
    where
        T: Ord + Clone + std::fmt::Debug,
        N: FibonacciNodeViewLike<T>,
    {
        let node_value = node.value_cloned();
        if let Some(parent_value) = parent {
            assert!(
                parent_value <= node_value,
                "heap property violated: parent > child"
            );
            assert!(
                node.parent_view().is_some(),
                "non-root child should expose parent"
            );
        }

        let mut child_count = 0usize;
        let mut total_nodes = 1usize;
        let mut child = node.child_view();
        while let Some(child_node) = child {
            child_count += 1;
            let (subtree_nodes, _) = walk::<T, N>(child_node.clone(), Some(node_value.clone()));
            total_nodes += subtree_nodes;
            child = child_node.sibling_view();
        }

        assert_eq!(
            child_count,
            node.degree_value(),
            "degree must match number of direct children"
        );

        (total_nodes, child_count)
    }

    let mut root = head;
    while let Some(root_node) = root {
        assert!(
            root_node.parent_view().is_none(),
            "root nodes must not have parents"
        );
        let _ = walk::<T, N>(root_node.clone(), None);
        root = root_node.sibling_view();
    }
}

#[cfg(test)]
macro_rules! test_binomial_heap_variant {
    ($module:ident, $heap_ty:ty) => {
        mod $module {
            use super::*;
            use crate::traits::diagnostics::ForestDiagnostics;

            type Heap = $heap_ty;

            #[test]
            fn empty_heap() {
                let mut heap = Heap::new();
                assert!(heap.is_empty());
                assert!(heap.min().is_none());
                assert_eq!(heap.extract_min(), None);
                assert_eq!(heap.root_count(), 0);
                assert_eq!(heap.node_count(), 0);
            }

            #[test]
            fn insert_min_and_extract_order() {
                let mut heap = Heap::new();
                for value in [5, 3, 7, 2, 4, 6, 8, 1] {
                    heap.insert(value);
                }

                assert_eq!(*heap.min().expect("min").value(), 1);
                let mut extracted = Vec::new();
                while let Some(value) = heap.extract_min() {
                    extracted.push(value);
                }
                assert_eq!(extracted, vec![1, 2, 3, 4, 5, 6, 7, 8]);
                assert!(heap.is_empty());
            }

            #[test]
            fn merge_and_delete() {
                let mut left = Heap::new();
                for value in [5, 1, 8] {
                    left.insert(value);
                }

                let mut right = Heap::new();
                for value in [3, 7, 2] {
                    right.insert(value);
                }

                left.merge(&mut right);
                assert!(right.is_empty());
                assert_eq!(left.node_count(), 6);

                let cursor = left.search(&7).expect("search");
                assert_eq!(left.delete(cursor), Some(7));

                let mut extracted = Vec::new();
                while let Some(value) = left.extract_min() {
                    extracted.push(value);
                }
                assert_eq!(extracted, vec![1, 2, 3, 5, 8]);
            }

            #[test]
            fn decrease_key() {
                let mut heap = Heap::new();
                for value in [50, 40, 30] {
                    heap.insert(value);
                }

                let cursor = heap.search(&50).expect("cursor");
                heap.decrease_key(cursor, 10);

                assert_eq!(*heap.min().expect("min").value(), 10);
                assert_eq!(heap.extract_min(), Some(10));
                assert_heap_invariants::<i32, _>(heap.head_view());
            }

            #[test]
            fn mixed_operations_match_sorted_reference() {
                let mut heap = Heap::new();
                let mut expected = Vec::new();

                for value in [17, 4, 11, 2, 19, 8, 1, 13, 6] {
                    heap.insert(value);
                    expected.push(value);
                }

                let c19 = heap.search(&19).expect("cursor 19");
                heap.decrease_key(c19, 3);
                expected.retain(|v| *v != 19);
                expected.push(3);

                let c11 = heap.search(&11).expect("cursor 11");
                assert_eq!(heap.delete(c11), Some(11));
                expected.retain(|v| *v != 11);

                expected.sort_unstable();
                let mut actual = Vec::new();
                while let Some(value) = heap.extract_min() {
                    actual.push(value);
                }

                assert_eq!(actual, expected);
            }

            #[test]
            fn invariants_hold_after_many_decrease_key_operations() {
                let mut heap = Heap::new();
                for value in 0..64 {
                    heap.insert(value + 100);
                }

                for target in [163, 157, 149, 141, 133, 125, 117] {
                    let cursor = heap.search(&target).expect("cursor");
                    heap.decrease_key(cursor, target - 100);
                    assert_heap_invariants::<i32, _>(heap.head_view());
                }
            }

            #[test]
            fn diagnostics() {
                let mut heap = Heap::new();
                for value in 1..=32 {
                    heap.insert(value);
                }

                assert_eq!(heap.node_count(), 32);
                assert!(heap.root_count() > 0);
                assert!(heap.max_root_degree() > 0);
            }
        }
    };
}

#[cfg(test)]
test_binomial_heap_variant!(safe_variant, safe::FibonacciHeap<i32>);
#[cfg(test)]
test_binomial_heap_variant!(raw_variant, raw::FibonacciHeap<i32>);
#[cfg(test)]
test_binomial_heap_variant!(arena_variant, arena::FibonacciHeap<i32>);
