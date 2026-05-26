pub mod arena;
pub mod raw;
pub mod safe;

use crate::traits::core::PriorityQueue;

pub trait BinaryHeapVariant<T>: PriorityQueue<T> {
    type NodeView<'a>: Clone
    where
        Self: 'a;
}

impl<T: Ord> BinaryHeapVariant<T> for safe::BinaryHeap<T> {
    type NodeView<'a>
        = safe::BinaryHeapView<'a, T>
    where
        Self: 'a;
}

impl<T: Ord> BinaryHeapVariant<T> for raw::BinaryHeap<T> {
    type NodeView<'a>
        = raw::BinaryHeapView<'a, T>
    where
        Self: 'a;
}

impl<T: Ord> BinaryHeapVariant<T> for arena::BinaryHeap<T> {
    type NodeView<'a>
        = arena::BinaryHeapView<'a, T>
    where
        Self: 'a;
}

#[cfg(test)]
macro_rules! test_binary_heap_variant {
    ($module:ident, $heap_ty:ident) => {
        mod $module {
            use super::*;
            use crate::traits::core::PriorityQueue;

            type Heap = $heap_ty::BinaryHeap<i32>;

            #[test]
            fn empty_heap_behaves_as_expected() {
                let mut heap = Heap::new();
                assert!(heap.is_empty());
                assert_eq!(heap.len(), 0);
                assert!(heap.peek().is_none());
                assert_eq!(heap.pop(), None);

                heap.push(10);
                assert!(!heap.is_empty());
                heap.clear();
                assert!(heap.is_empty());
            }

            #[test]
            fn push_peek_and_pop_are_min_heap_ordered() {
                let mut heap = Heap::new();
                for value in [5, 3, 9, 1, 8, 2, 4] {
                    heap.push(value);
                }

                {
                    let cursor = heap.peek().expect("peek cursor");
                    let view = heap.view_from_cursor(&cursor);
                    assert_eq!(*view.value(), 1);
                }

                let mut popped = Vec::new();
                while let Some(value) = heap.pop() {
                    popped.push(value);
                }

                assert_eq!(popped, vec![1, 2, 3, 4, 5, 8, 9]);
                assert!(heap.is_empty());
            }

            #[test]
            fn mixed_operations_match_sorted_reference() {
                let mut heap = Heap::new();
                let mut expected = Vec::new();

                for value in [12, 4, 18, 7, 2, 10, 15, 1] {
                    heap.push(value);
                    expected.push(value);
                }

                let removed = heap.cursor(&10).expect("cursor 10");
                if let Some(val) = heap.remove_cursor(removed) {
                    assert_eq!(val, 10);
                    expected.retain(|v| *v != 10);
                }

                expected.sort_unstable();
                let mut actual = Vec::new();
                while let Some(value) = heap.pop() {
                    actual.push(value);
                }

                assert_eq!(actual, expected);
            }

            #[test]
            fn stress_random_operations() {
                use rand::{Rng, SeedableRng, rngs::StdRng};
                use std::collections::BinaryHeap as StdHeap;
                use std::cmp::Reverse;

                let mut heap = Heap::new();
                let mut reference = StdHeap::new();
                let mut rng = StdRng::seed_from_u64(42);

                for _ in 0..1000 {
                    match rng.gen_range(0..2) {
                        0 => { // push
                            let val = rng.gen_range(0..2000);
                            heap.push(val);
                            reference.push(Reverse(val));
                        }
                        1 if !heap.is_empty() => { // pop
                            let actual = heap.pop();
                            let expected = reference.pop().map(|Reverse(v)| v);
                            assert_eq!(actual, expected);
                        }
                        _ => {}
                    }
                }
                assert_eq!(heap.len(), reference.len());
            }
        }
    };
}

#[cfg(test)]
test_binary_heap_variant!(safe_variant, safe);
#[cfg(test)]
test_binary_heap_variant!(raw_variant, raw);
#[cfg(test)]
test_binary_heap_variant!(arena_variant, arena);
