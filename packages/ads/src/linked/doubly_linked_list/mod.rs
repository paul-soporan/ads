pub mod arena;
pub mod raw;
pub mod safe;

#[cfg(test)]
macro_rules! test_doubly_linked_list_variant {
    ($module:ident, $list_ty:ident) => {
        mod $module {
            use super::*;
            use std::collections::VecDeque;
            use rand::{Rng, SeedableRng, rngs::StdRng};

            use crate::traits::core::{Sequence, SequenceMutGuard};
            use crate::traits::diagnostics::SequenceDiagnostics;

            type List = $list_ty::DoublyLinkedList<i32>;

            #[test]
            fn push_and_pop_front_back() {
                let mut list = List::new();
                assert!(list.is_empty());

                list.push_front(2);
                list.push_front(1);
                list.push_back(3);
                list.push_back(4);

                assert_eq!(list.len(), 4);
                assert_eq!(list.pop_front(), Some(1));
                assert_eq!(list.pop_back(), Some(4));
                assert_eq!(list.pop_front(), Some(2));
                assert_eq!(list.pop_back(), Some(3));
                assert_eq!(list.pop_front(), None);
                assert!(list.is_empty());
            }

            #[test]
            fn cursor_and_get_mut_by_index() {
                let mut list = List::new();
                for value in [5, 10, 15, 20, 25] {
                    list.push_back(value);
                }

                let cursor = list.cursor_at(3).expect("cursor at 3");
                assert_eq!(cursor.index(), 3);
                assert_eq!(*cursor.value(), 20);

                list.get_mut(0)
                    .expect("first value")
                    .with_mut(|value| *value = 7);
                list.get_mut(4)
                    .expect("last value")
                    .with_mut(|value| *value = 30);

                assert_eq!(list.cursor_at(0).map(|c| *c.value()), Some(7));
                assert_eq!(list.cursor_at(4).map(|c| *c.value()), Some(30));
            }

            #[test]
            fn clear_resets_state() {
                let mut list = List::new();
                for value in 0..6 {
                    list.push_back(value);
                }
                assert_eq!(list.len(), 6);

                list.clear();

                assert_eq!(list.len(), 0);
                assert!(list.is_empty());
                assert!(list.cursor_at(0).is_none());
                assert_eq!(list.pop_back(), None);
            }

            #[test]
            fn stress_random_operations() {
                let mut list = List::new();
                let mut model = VecDeque::new();
                let mut rng = StdRng::seed_from_u64(42);

                for _ in 0..500 {
                    match rng.gen_range(0..4) {
                        0 => { // push_front
                            let val = rng.gen_range(0..1000);
                            list.push_front(val);
                            model.push_front(val);
                        }
                        1 => { // push_back
                            let val = rng.gen_range(0..1000);
                            list.push_back(val);
                            model.push_back(val);
                        }
                        2 if !model.is_empty() => { // pop_front
                            assert_eq!(list.pop_front(), model.pop_front());
                        }
                        3 if !model.is_empty() => { // pop_back
                            assert_eq!(list.pop_back(), model.pop_back());
                        }
                        _ => {}
                    }
                }

                let actual: Vec<_> = (0..model.len())
                    .map(|i| *list.cursor_at(i).expect("cursor").value())
                    .collect();
                let expected: Vec<_> = model.iter().copied().collect();
                assert_eq!(actual, expected);
            }

            #[test]
            fn complexity_verification() {
                let mut list = List::new();
                let n = 100usize;
                for i in 0..n {
                    list.push_back(i as i32);
                }

                // pop_front is O(1)
                let before = list.walk_steps();
                list.pop_front();
                let after = list.walk_steps();
                assert_eq!(after - before, 0, "pop_front should be 0 walk steps");

                // pop_back is O(1) in doubly linked list
                let before = list.walk_steps();
                list.pop_back();
                let after = list.walk_steps();
                assert_eq!(after - before, 0, "pop_back should be 0 walk steps in doubly linked list");

                // cursor_at(i) is O(min(i, N-i)) when value is accessed
                let before = list.walk_steps();
                let cursor_10 = list.cursor_at(10).expect("cursor at 10");
                let _ = cursor_10.value();
                let after = list.walk_steps();
                assert_eq!(after - before, 10, "accessing cursor value at 10 from head should be 10 steps");

                let before = list.walk_steps();
                let cursor_95 = list.cursor_at(95).expect("cursor at 95");
                let _ = cursor_95.value();
                let after = list.walk_steps();
                // length is 98 now. index 95 is (98-1-95) = 2 steps from tail.
                assert_eq!(after - before, 2, "accessing cursor value at 95 from tail should be minimal steps");
            }
        }
    };
}

#[cfg(test)]
test_doubly_linked_list_variant!(safe_variant, safe);
#[cfg(test)]
test_doubly_linked_list_variant!(raw_variant, raw);
#[cfg(test)]
test_doubly_linked_list_variant!(arena_variant, arena);
