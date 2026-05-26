pub mod raw;
pub mod safe;
pub mod arena;

use crate::traits::{core::PriorityQueue, diagnostics::ForestDiagnostics};

pub trait BinomialHeapVariant<T>: PriorityQueue<T> + ForestDiagnostics {
    type NodeView<'a>: Clone
    where
        Self: 'a;

    fn head_view<'a>(&'a self) -> Option<Self::NodeView<'a>>;
    fn merge_with(&mut self, other: &mut Self);
}

impl<T: Ord> BinomialHeapVariant<T> for safe::BinomialHeap<T> {
    type NodeView<'a>
        = safe::BinomialNodeView<T>
    where
        Self: 'a;

    fn head_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.head_view()
    }

    fn merge_with(&mut self, other: &mut Self) {
        self.merge(other)
    }
}

impl<T: Ord> BinomialHeapVariant<T> for raw::BinomialHeap<T> {
    type NodeView<'a>
        = raw::BinomialNodeView<T>
    where
        Self: 'a;

    fn head_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.head_view()
    }

    fn merge_with(&mut self, other: &mut Self) {
        self.merge(other)
    }
}

impl<T: Ord> BinomialHeapVariant<T> for arena::BinomialHeap<T> {
    type NodeView<'a>
        = arena::BinomialNodeView<'a, T>
    where
        Self: 'a;

    fn head_view<'a>(&'a self) -> Option<Self::NodeView<'a>> {
        self.head_view()
    }

    fn merge_with(&mut self, other: &mut Self) {
        self.merge(other)
    }
}

#[cfg(test)]
trait BinomialNodeViewLike<T>: Clone {
    fn value_cloned(&self) -> T
    where
        T: Clone;
    fn degree_value(&self) -> usize;
    fn child_view(&self) -> Option<Self>;
    fn sibling_view(&self) -> Option<Self>;
    fn parent_view(&self) -> Option<Self>;
}

#[cfg(test)]
impl<T: Ord> BinomialNodeViewLike<T> for safe::BinomialNodeView<T> {
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
impl<T: Ord> BinomialNodeViewLike<T> for raw::BinomialNodeView<T> {
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
impl<'a, T: Ord> BinomialNodeViewLike<T> for arena::BinomialNodeView<'a, T> {
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
fn assert_binomial_invariants<T, N>(head: Option<N>)
where
    T: Ord + Clone + std::fmt::Debug,
    N: BinomialNodeViewLike<T>,
{
    fn walk<T, N>(node: N, parent: Option<T>) -> usize
    where
        T: Ord + Clone + std::fmt::Debug,
        N: BinomialNodeViewLike<T>,
    {
        let node_value = node.value_cloned();
        if let Some(parent_value) = parent {
            assert!(
                parent_value <= node_value,
                "heap property violated: parent > child"
            );
            assert!(node.parent_view().is_some(), "child should expose parent");
        }

        let mut child_count = 0usize;
        let mut total_nodes = 1usize;
        let mut child = node.child_view();
        while let Some(child_node) = child {
            child_count += 1;
            total_nodes += walk::<T, N>(child_node.clone(), Some(node_value.clone()));
            child = child_node.sibling_view();
        }

        assert_eq!(child_count, node.degree_value(), "degree/child-count mismatch");
        assert_eq!(total_nodes, 1 << child_count, "binomial tree size must be 2^k");
        total_nodes
    }

    let mut root = head;
    let mut last_degree: Option<usize> = None;
    while let Some(root_node) = root {
        assert!(root_node.parent_view().is_none(), "root must have no parent");
        let degree = root_node.degree_value();
        if let Some(last) = last_degree {
            assert!(degree > last, "root list must be sorted by strictly increasing degree");
        }
        last_degree = Some(degree);
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
                    assert_binomial_invariants::<i32, _>(heap.head_view());
                }

                {
                    let min_cursor = heap.min().expect("min");
                    assert_eq!(*heap.view_from_cursor(&min_cursor).value(), 1);
                }
                let mut extracted = Vec::new();
                while let Some(value) = heap.extract_min() {
                    extracted.push(value);
                    assert_binomial_invariants::<i32, _>(heap.head_view());
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
                assert_binomial_invariants::<i32, _>(left.head_view());

                let cursor = left.search(&7).expect("search");
                assert_eq!(left.delete(cursor), Some(7));
                assert_binomial_invariants::<i32, _>(left.head_view());

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

                {
                    let min_cursor = heap.min().expect("min");
                    assert_eq!(*heap.view_from_cursor(&min_cursor).value(), 10);
                }
                assert_eq!(heap.extract_min(), Some(10));
                assert_binomial_invariants::<i32, _>(heap.head_view());
            }

            #[test]
            fn stress_random_operations() {
                use rand::{Rng, SeedableRng, rngs::StdRng};
                let mut heap = Heap::new();
                let mut reference = Vec::new();
                let mut rng = StdRng::seed_from_u64(42);

                for _ in 0..500 {
                    match rng.gen_range(0..4) {
                        0 => { // Insert
                            let val = rng.gen_range(0..1000);
                            heap.insert(val);
                            reference.push(val);
                        }
                        1 if !heap.is_empty() => { // Extract Min
                            let actual = heap.extract_min();
                            reference.sort_unstable();
                            let expected = if reference.is_empty() { None } else { Some(reference.remove(0)) };
                            assert_eq!(actual, expected);
                        }
                        2 if !heap.is_empty() => { // Decrease Key
                            let idx = rng.gen_range(0..reference.len());
                            let old_val = reference[idx];
                            if let Some(cursor) = heap.search(&old_val) {
                                let new_val = rng.gen_range(-500..old_val);
                                heap.decrease_key(cursor, new_val);
                                reference[idx] = new_val;
                            }
                        }
                        3 if !heap.is_empty() => { // Delete
                            let idx = rng.gen_range(0..reference.len());
                            let val = reference.remove(idx);
                            if let Some(cursor) = heap.search(&val) {
                                assert_eq!(heap.delete(cursor), Some(val));
                            } else {
                                reference.push(val); 
                            }
                        }
                        _ => {}
                    }
                    if rng.gen_bool(0.1) {
                        assert_binomial_invariants::<i32, _>(heap.head_view());
                    }
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
macro_rules! test_binomial_heap_complexity_variant {
    ($module:ident, $heap_ty:ty) => {
        mod $module {
            use super::*;
            use std::sync::{Arc, atomic::{AtomicUsize, Ordering as AtomicOrdering}};

            type Heap = $heap_ty;

            #[derive(Debug, Clone)]
            struct CountedValue {
                value: i32,
                comparisons: Arc<AtomicUsize>,
            }

            impl CountedValue {
                fn new(value: i32, comparisons: &Arc<AtomicUsize>) -> Self {
                    Self { value, comparisons: comparisons.clone() }
                }
            }

            impl Eq for CountedValue {}
            impl PartialEq for CountedValue {
                fn eq(&self, other: &Self) -> bool { self.value == other.value }
            }

            impl Ord for CountedValue {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    self.comparisons.fetch_add(1, AtomicOrdering::Relaxed);
                    self.value.cmp(&other.value)
                }
            }

            impl PartialOrd for CountedValue {
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }

            #[test]
            fn push_comparison_count_scales_logarithmically() {
                fn run_case(size: i32) -> usize {
                    let comparisons = Arc::new(AtomicUsize::new(0));
                    let mut heap = Heap::new();
                    for value in 0..size {
                        heap.insert(CountedValue::new(value, &comparisons));
                    }
                    comparisons.load(AtomicOrdering::Relaxed)
                }

                let small = run_case(32);
                let large = run_case(64);

                // For doubling size, cost should slightly more than double (N log N total)
                // but let's look at average per push. 
                // Average push cost for N items is ~1.5 comparisons in some binomial heap impls,
                // but total is O(N). Let's check total scaling.
                assert!(large <= small * 3, "total push cost grew too quickly ({} -> {})", small, large);
            }

            #[test]
            fn extract_min_comparison_count_scales_logarithmically() {
                fn run_case(size: i32) -> usize {
                    let comparisons = Arc::new(AtomicUsize::new(0));
                    let mut heap = Heap::new();
                    for value in (0..size).rev() {
                        heap.insert(CountedValue::new(value, &comparisons));
                    }
                    comparisons.store(0, AtomicOrdering::Relaxed);
                    while heap.extract_min().is_some() {}
                    comparisons.load(AtomicOrdering::Relaxed)
                }

                let small = run_case(32);
                let large = run_case(64);

                assert!(large <= small * 3, "total extract_min cost grew too quickly ({} -> {})", small, large);
            }
        }
    };
}

#[cfg(test)]
test_binomial_heap_variant!(safe_variant, safe::BinomialHeap<i32>);
#[cfg(test)]
test_binomial_heap_variant!(raw_variant, raw::BinomialHeap<i32>);
#[cfg(test)]
test_binomial_heap_variant!(arena_variant, arena::BinomialHeap<i32>);

#[cfg(test)]
test_binomial_heap_complexity_variant!(safe_complexity, safe::BinomialHeap<CountedValue>);
#[cfg(test)]
test_binomial_heap_complexity_variant!(raw_complexity, raw::BinomialHeap<CountedValue>);
#[cfg(test)]
test_binomial_heap_complexity_variant!(arena_complexity, arena::BinomialHeap<CountedValue>);
