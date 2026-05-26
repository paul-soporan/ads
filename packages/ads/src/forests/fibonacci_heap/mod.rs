use std::{
    cmp::Ordering as CmpOrdering,
    sync::{
        atomic::{AtomicUsize, Ordering as AtomicOrdering},
        Arc,
    },
};

use crate::traits::core::PriorityQueue;

pub mod arena;
pub mod raw;
pub mod safe;

#[cfg(test)]
#[derive(Debug, Clone)]
struct CountedValue {
    value: i32,
    comparisons: Arc<AtomicUsize>,
}

#[cfg(test)]
impl Eq for CountedValue {}

#[cfg(test)]
impl CountedValue {
    fn new(value: i32, comparisons: &Arc<AtomicUsize>) -> Self {
        Self {
            value,
            comparisons: comparisons.clone(),
        }
    }
}

#[cfg(test)]
impl Ord for CountedValue {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        self.comparisons.fetch_add(1, AtomicOrdering::Relaxed);
        self.value.cmp(&other.value)
    }
}

#[cfg(test)]
impl PartialOrd for CountedValue {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
impl PartialEq for CountedValue {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

#[cfg(test)]
trait FibonacciHeapVariant<T: Ord>: PriorityQueue<T> {
    fn view_from_cursor<'a>(heap: &'a Self, cursor: &Self::Cursor<'a>) -> Self::View<'a>;
}

#[cfg(test)]
impl<T: Ord> FibonacciHeapVariant<T> for safe::FibonacciHeap<T> {
    fn view_from_cursor<'a>(heap: &'a Self, cursor: &Self::Cursor<'a>) -> Self::View<'a> {
        heap.view_from_cursor(cursor)
    }
}

#[cfg(test)]
impl<T: Ord> FibonacciHeapVariant<T> for raw::FibonacciHeap<T> {
    fn view_from_cursor<'a>(heap: &'a Self, cursor: &Self::Cursor<'a>) -> Self::View<'a> {
        heap.view_from_cursor(cursor)
    }
}

#[cfg(test)]
impl<T: Ord> FibonacciHeapVariant<T> for arena::FibonacciHeap<T> {
    fn view_from_cursor<'a>(heap: &'a Self, cursor: &Self::Cursor<'a>) -> Self::View<'a> {
        heap.view_from_cursor(cursor)
    }
}

#[cfg(test)]
trait FibonacciNodeViewLike<T>: Clone {
    fn identity(&self) -> usize;
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
    fn identity(&self) -> usize {
        self.identity()
    }

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
    fn identity(&self) -> usize {
        self.identity()
    }

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
    fn identity(&self) -> usize {
        self.identity()
    }

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
        let first_child_id = child.as_ref().map(|c| c.identity());

        while let Some(child_node) = child {
            child_count += 1;
            let (subtree_nodes, _) = walk::<T, N>(child_node.clone(), Some(node_value.clone()));
            total_nodes += subtree_nodes;
            child = child_node.sibling_view();
            if child.as_ref().map(|c| c.identity()) == first_child_id {
                break;
            }
        }

        assert_eq!(
            child_count,
            node.degree_value(),
            "degree must match number of direct children"
        );

        (total_nodes, child_count)
    }

    let mut root = head;
    let first_root_id = root.as_ref().map(|r| r.identity());
    while let Some(root_node) = root {
        assert!(
            root_node.parent_view().is_none(),
            "root nodes must not have parents"
        );
        let _ = walk::<T, N>(root_node.clone(), None);
        root = root_node.sibling_view();
        if root.as_ref().map(|r| r.identity()) == first_root_id {
            break;
        }
    }
}

#[cfg(test)]
macro_rules! test_fibonacci_heap_variant {
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
            fn delete_edge_cases() {
                let mut heap = Heap::new();
                
                // Delete min
                heap.insert(10);
                heap.insert(20);
                let c10 = heap.search(&10).expect("c10");
                assert_eq!(heap.delete(c10), Some(10));
                assert_eq!(heap.extract_min(), Some(20));
                assert!(heap.is_empty());

                // Delete only node
                heap.insert(30);
                let c30 = heap.search(&30).expect("c30");
                assert_eq!(heap.delete(c30), Some(30));
                assert!(heap.is_empty());

                // Delete child node after consolidation
                for i in 0..16 { heap.insert(i); }
                heap.extract_min(); // triggers consolidation, 0 is gone, others are in trees
                let c15 = heap.search(&15).expect("c15");
                assert_eq!(heap.delete(c15), Some(15));
                assert_heap_invariants::<i32, _>(heap.head_view());
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
                                // Value might have been decreased or duplicate removed
                                reference.push(val); 
                            }
                        }
                        _ => {}
                    }
                    if rng.gen_bool(0.1) {
                        assert_heap_invariants::<i32, _>(heap.head_view());
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
            }
        }
    };
}

#[cfg(test)]
macro_rules! test_fibonacci_heap_complexity_variant {
    ($module:ident, $heap_ty:ty) => {
        mod $module {
            use super::*;

            type Heap = $heap_ty;

            #[test]
            fn insert_comparison_count_is_linear_with_one_per_op() {
                let comparisons = Arc::new(AtomicUsize::new(0));
                let mut heap = Heap::new();

                for value in 0..100 {
                    heap.insert(CountedValue::new(value, &comparisons));
                }

                // 100 inserts: 1st is free, next 99 each compare with current min.
                assert_eq!(
                    comparisons.load(AtomicOrdering::Relaxed),
                    99,
                    "each insert after the first should perform exactly 1 comparison to update min"
                );
            }

            #[test]
            fn merge_comparison_count_is_one() {
                let comparisons = Arc::new(AtomicUsize::new(0));
                let mut h1 = Heap::new();
                let mut h2 = Heap::new();

                for value in 0..50 {
                    h1.insert(CountedValue::new(value, &comparisons));
                    h2.insert(CountedValue::new(value + 100, &comparisons));
                }

                comparisons.store(0, AtomicOrdering::Relaxed);
                h1.merge(&mut h2);

                assert_eq!(
                    comparisons.load(AtomicOrdering::Relaxed),
                    1,
                    "merge should perform exactly 1 comparison to update min"
                );
            }

            #[test]
            fn extract_min_comparison_count_scales_reasonably() {
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

                let small = run_case(16);
                let medium = run_case(32);
                let large = run_case(64);

                assert!(
                    medium <= (small as f64 * 3.5) as usize,
                    "comparison count grew too quickly for extract_min (total): 16 -> 32 ({} -> {})",
                    small,
                    medium
                );
                assert!(
                    large <= (medium as f64 * 3.5) as usize,
                    "comparison count grew too quickly for extract_min (total): 32 -> 64 ({} -> {})",
                    medium,
                    large
                );
            }

            #[test]
            fn decrease_key_comparison_count_scales_reasonably() {
                fn run_case(size: i32) -> usize {
                    let comparisons = Arc::new(AtomicUsize::new(0));
                    let search_comparisons = Arc::new(AtomicUsize::new(0));
                    let mut heap = Heap::new();

                    for value in 0..size {
                        heap.insert(CountedValue::new(value, &comparisons));
                    }

                    heap.extract_min();

                    let mut cursors = Vec::new();
                    for probe in (1..size).rev() {
                        let probe_value = CountedValue::new(probe, &search_comparisons);
                        if let Some(cursor) = heap.search(&probe_value)
                            && FibonacciHeapVariant::view_from_cursor(&heap, &cursor)
                                .parent()
                                .is_some()
                        {
                            cursors.push(cursor);
                        }

                        if cursors.len() == 4 {
                            break;
                        }
                    }

                    assert_eq!(
                        cursors.len(),
                        4,
                        "expected to find enough non-root cursors for decrease_key smoke test"
                    );

                    comparisons.store(0, AtomicOrdering::Relaxed);
                    for (index, cursor) in cursors.into_iter().enumerate() {
                        heap.decrease_key(
                            cursor,
                            CountedValue::new(-(index as i32) - 1, &comparisons),
                        );
                    }

                    comparisons.load(AtomicOrdering::Relaxed)
                }

                let small = run_case(32);
                let large = run_case(64);

                assert!(
                    large <= (small as f64 * 1.5) as usize,
                    "comparison count grew too quickly for decrease_key ({} -> {})",
                    small,
                    large
                );
            }

            #[test]
            fn delete_comparison_count_scales_reasonably() {
                fn run_case(size: i32) -> usize {
                    let comparisons = Arc::new(AtomicUsize::new(0));
                    let search_comparisons = Arc::new(AtomicUsize::new(0));
                    let mut heap = Heap::new();

                    for value in 0..size {
                        heap.insert(CountedValue::new(value, &comparisons));
                    }

                    heap.extract_min(); // triggers consolidation

                    let mut cursors = Vec::new();
                    for probe in (1..size).rev() {
                        let probe_value = CountedValue::new(probe, &search_comparisons);
                        if let Some(cursor) = heap.search(&probe_value) {
                            cursors.push(cursor);
                        }
                        if cursors.len() == 4 { break; }
                    }

                    comparisons.store(0, AtomicOrdering::Relaxed);
                    for cursor in cursors {
                        heap.delete(cursor);
                    }

                    comparisons.load(AtomicOrdering::Relaxed)
                }

                let small = run_case(32);
                let large = run_case(64);

                assert!(
                    large <= (small as f64 * 2.0) as usize,
                    "comparison count grew too quickly for delete ({} -> {})",
                    small,
                    large
                );
            }
        }
    };
}

#[cfg(test)]
test_fibonacci_heap_variant!(safe_variant, safe::FibonacciHeap<i32>);
#[cfg(test)]
test_fibonacci_heap_variant!(raw_variant, raw::FibonacciHeap<i32>);
#[cfg(test)]
test_fibonacci_heap_variant!(arena_variant, arena::FibonacciHeap<i32>);

#[cfg(test)]
test_fibonacci_heap_complexity_variant!(safe_complexity, safe::FibonacciHeap<CountedValue>);
#[cfg(test)]
test_fibonacci_heap_complexity_variant!(raw_complexity, raw::FibonacciHeap<CountedValue>);
#[cfg(test)]
test_fibonacci_heap_complexity_variant!(arena_complexity, arena::FibonacciHeap<CountedValue>);
