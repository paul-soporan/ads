pub mod arena;
pub mod raw;
pub mod safe;

#[cfg(test)]
macro_rules! test_singly_linked_list_variant {
    ($module:ident, $list_ty:ident) => {
        mod $module {
            use super::*;
            use std::collections::VecDeque;
            use rand::{Rng, SeedableRng, rngs::StdRng};

            use crate::traits::core::{Sequence, SequenceMutGuard};
            use crate::traits::diagnostics::SequenceDiagnostics;

            type List = $list_ty::SinglyLinkedList<i32>;

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
                for value in [10, 20, 30, 40] {
                    list.push_back(value);
                }

                let cursor = list.cursor_at(2).expect("cursor at 2");
                assert_eq!(cursor.index(), 2);
                assert_eq!(*cursor.value(), 30);

                list.get_mut(1)
                    .expect("value at 1")
                    .with_mut(|value| *value = 25);
                assert_eq!(list.cursor_at(1).map(|c| *c.value()), Some(25));
            }

            #[test]
            fn clear_resets_state() {
                let mut list = List::new();
                for value in 0..8 {
                    list.push_back(value);
                }
                assert_eq!(list.len(), 8);

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

                // pop_front is O(1) walk steps
                let before = list.walk_steps();
                list.pop_front();
                let after = list.walk_steps();
                assert_eq!(after - before, 0, "pop_front should be O(1) walk steps (0 steps)");

                // cursor_at(i) is O(i) walk steps when value is accessed
                let before = list.walk_steps();
                let cursor = list.cursor_at(50).expect("cursor at 50");
                let _ = cursor.value();
                let after = list.walk_steps();
                assert_eq!(after - before, 50, "accessing cursor value at 50 should be 50 walk steps");

                // pop_back is O(N) walk steps
                let before = list.walk_steps();
                list.pop_back();
                let after = list.walk_steps();
                // pop_back needs to reach node at N-2 to update its next pointer.
                assert!(after - before >= (n - 3), "pop_back should be O(N) walk steps");
            }
        }
    };
}

#[cfg(test)]
test_singly_linked_list_variant!(safe_variant, safe);
#[cfg(test)]
test_singly_linked_list_variant!(raw_variant, raw);
#[cfg(test)]
test_singly_linked_list_variant!(arena_variant, arena);
